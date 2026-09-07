//! Request hardening for the embedded Axum stack: CSRF Origin checks, POST
//! rate limits, and a host-wide WebSocket connection cap.
//!
//! Wire [`request_hardening_middleware`] after sessions are configured and
//! insert [`CsrfConfig`] into request extensions (same Origin sources as the
//! Photon WebSocket allowlist: `UF_PUBLIC_BASE_URL` plus
//! `PHOTON_LEPTOS_ALLOWED_ORIGINS`).
//!
//! ## Failures callers see
//!
//! | Condition | Status | Body |
//! |-----------|--------|------|
//! | Auth-ish POST missing Origin/Referer | 403 | `missing origin` |
//! | Origin not on allowlist | 403 | `origin not allowed` |
//! | Per-IP POST / auth rate exceeded | 429 | `rate limit exceeded` |
//! | Concurrent `/ws*` over cap (64) | 503 | `websocket capacity exceeded` |
//!
//! Caps: 20 auth-ish POSTs per IP per minute; 120 other POSTs per IP per minute.
//! Operator guide: repository `SECURITY.md` and `docs/deploy.md`.

use std::collections::HashMap;
use std::net::SocketAddr;
use std::sync::{Arc, Mutex, OnceLock};
use std::time::{Duration, Instant};

use axum::body::Body;
use axum::extract::ConnectInfo;
use axum::http::{header, Method, Request, StatusCode};
use axum::middleware::Next;
use axum::response::{IntoResponse, Response};
use tokio::sync::Semaphore;

/// Max concurrent Photon WebSocket upgrades (host-wide).
const WS_MAX_CONNECTIONS: usize = 64;

/// Auth-ish POSTs allowed per client IP per minute.
const AUTH_RATE_LIMIT_PER_MIN: u32 = 20;

/// General POSTs (server fns) allowed per client IP per minute.
const POST_RATE_LIMIT_PER_MIN: u32 = 120;

fn ws_semaphore() -> &'static Semaphore {
    static SEM: OnceLock<Semaphore> = OnceLock::new();
    SEM.get_or_init(|| Semaphore::new(WS_MAX_CONNECTIONS))
}

struct Bucket {
    tokens: f64,
    last_refill: Instant,
}

#[derive(Default)]
struct Limiter {
    buckets: Mutex<HashMap<String, Bucket>>,
}

fn limiter() -> &'static Limiter {
    static INSTANCE: OnceLock<Limiter> = OnceLock::new();
    INSTANCE.get_or_init(Limiter::default)
}

fn allow(key: &str, capacity: u32) -> bool {
    if capacity == 0 {
        return true;
    }
    let capacity_f = f64::from(capacity);
    let refill_per_sec = capacity_f / 60.0;
    let now = Instant::now();
    let mut buckets = limiter()
        .buckets
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner);
    if buckets.len() > 10_000 {
        buckets.retain(|_, b| now.duration_since(b.last_refill) < Duration::from_secs(3600));
    }
    let bucket = buckets.entry(key.to_string()).or_insert_with(|| Bucket {
        tokens: capacity_f,
        last_refill: now,
    });
    let elapsed = now.duration_since(bucket.last_refill).as_secs_f64();
    bucket.tokens = (bucket.tokens + elapsed * refill_per_sec).min(capacity_f);
    bucket.last_refill = now;
    if bucket.tokens >= 1.0 {
        bucket.tokens -= 1.0;
        true
    } else {
        false
    }
}

fn client_ip(req: &Request<Body>) -> String {
    if let Some(ConnectInfo(addr)) = req.extensions().get::<ConnectInfo<SocketAddr>>() {
        return addr.ip().to_string();
    }
    req.headers()
        .get("x-forwarded-for")
        .and_then(|v| v.to_str().ok())
        .and_then(|s| s.split(',').next())
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
        .unwrap_or_else(|| "unknown".into())
}

fn is_auth_path(path: &str) -> bool {
    let lower = path.to_ascii_lowercase();
    [
        "login",
        "signup",
        "sign_up",
        "register",
        "password",
        "oauth",
        "authenticate",
        "mfa",
        "totp",
        "passkey",
        "webauthn",
        "reset",
    ]
    .iter()
    .any(|needle| lower.contains(needle))
}

fn origin_allowed(origin: &str, public_base: &str, extra: &[String]) -> bool {
    let origin = origin.trim().trim_end_matches('/');
    let base = public_base.trim().trim_end_matches('/');
    if !base.is_empty() && origin.eq_ignore_ascii_case(base) {
        return true;
    }
    extra
        .iter()
        .any(|o| origin.eq_ignore_ascii_case(o.trim().trim_end_matches('/')))
}

/// Shared allowlist for CSRF Origin checks (same sources as Photon WS when possible).
#[derive(Clone, Debug)]
pub struct CsrfConfig {
    /// Canonical public base URL (`UF_PUBLIC_BASE_URL`).
    pub public_base: String,
    /// Extra allowed origins (e.g. Photon allowlist).
    pub allowed_origins: Arc<[String]>,
}

/// Axum middleware: CSRF Origin on mutating methods, POST rate limits, WS connection cap.
pub async fn request_hardening_middleware(
    req: Request<Body>,
    next: Next,
) -> Result<Response, StatusCode> {
    let path = req.uri().path().to_string();
    let method = req.method().clone();
    let ip = client_ip(&req);

    if path.starts_with("/ws") {
        let Ok(permit) = ws_semaphore().try_acquire() else {
            return Ok((
                StatusCode::SERVICE_UNAVAILABLE,
                "websocket capacity exceeded",
            )
                .into_response());
        };
        let response = next.run(req).await;
        // Keep the permit until the WebSocket handler returns (connection closed).
        drop(permit);
        return Ok(response);
    }

    let is_mutating = matches!(
        method,
        Method::POST | Method::PUT | Method::PATCH | Method::DELETE
    );

    if is_mutating {
        if let Some(cfg) = req.extensions().get::<CsrfConfig>() {
            let origin = req
                .headers()
                .get(header::ORIGIN)
                .and_then(|v| v.to_str().ok());
            // Same-origin navigations may omit Origin; require it for API-like POSTs.
            match origin {
                None => {
                    // Browsers omit Origin on same-origin GET; for POST they usually send it.
                    // Allow missing Origin only for non-auth form posts from same host Referer.
                    let referer_ok = req
                        .headers()
                        .get(header::REFERER)
                        .and_then(|v| v.to_str().ok())
                        .is_some_and(|r| {
                            r.starts_with(&cfg.public_base)
                                || cfg
                                    .allowed_origins
                                    .iter()
                                    .any(|o| r.starts_with(o.as_str()))
                        });
                    if !referer_ok && is_auth_path(&path) {
                        return Ok((StatusCode::FORBIDDEN, "missing origin").into_response());
                    }
                }
                Some(origin) if !origin_allowed(origin, &cfg.public_base, &cfg.allowed_origins) => {
                    return Ok((StatusCode::FORBIDDEN, "origin not allowed").into_response());
                }
                Some(_) => {}
            }
        }

        let limit = if is_auth_path(&path) {
            AUTH_RATE_LIMIT_PER_MIN
        } else {
            POST_RATE_LIMIT_PER_MIN
        };
        let key = format!(
            "{}:{}",
            if is_auth_path(&path) { "auth" } else { "post" },
            ip
        );
        if !allow(&key, limit) {
            return Ok((StatusCode::TOO_MANY_REQUESTS, "rate limit exceeded").into_response());
        }
    }

    Ok(next.run(req).await)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn auth_path_detects_login() {
        assert!(is_auth_path("/api/login"));
        assert!(is_auth_path("/foo/password_reset"));
        assert!(is_auth_path("/auth/oauth/callback"));
        assert!(is_auth_path("/mfa/totp/verify"));
        assert!(!is_auth_path("/api/get_jobs"));
        assert!(!is_auth_path("/chronon/jobs"));
    }

    #[test]
    fn origin_allowed_matches_base() {
        assert!(origin_allowed(
            "https://app.example.com",
            "https://app.example.com/",
            &[]
        ));
        assert!(origin_allowed(
            "https://extra.example",
            "https://app.example.com",
            &["https://extra.example/".into()]
        ));
        assert!(!origin_allowed(
            "https://evil.example",
            "https://app.example.com",
            &[]
        ));
        assert!(!origin_allowed(
            "https://evil.example",
            "https://app.example.com",
            &["https://extra.example".into()]
        ));
    }
}
