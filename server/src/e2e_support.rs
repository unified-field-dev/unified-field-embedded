//! Test-only seed route for host Playwright conformance (`e2e-host-conformance`).
//!
//! Compiles only when the Cargo feature is enabled. Requires `UF_E2E_SEED_TOKEN`
//! at router construction. Responses acknowledge success without password or
//! TOTP material.
//!
//! Fixture Valence starts as [`Actor::initialize_system_context`] under the
//! explicit test-harness allowlist. It is never derived from a session actor.

#[cfg(all(feature = "e2e-host-conformance", not(debug_assertions)))]
compile_error!(
    "e2e-host-conformance must not be enabled in release builds (debug_assertions required)"
);

use std::sync::Arc;

use axum::extract::State;
use axum::http::{HeaderMap, StatusCode};
use axum::routing::post;
use axum::{Json, Router};
use lepton_test_support::{run_seed, SeedError, SeedRequest};
use serde::Serialize;
use valence::{Actor, Valence};

use crate::AppState;

/// Opaque acknowledgement that excludes credentials.
#[derive(Debug, Serialize)]
pub struct SeedAck {
    /// Echoed scenario id.
    pub scenario: String,
    /// Seeded synthetic email.
    pub email: String,
}

/// Configuration failures when enabling the seed route.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SeedConfigError {
    /// `UF_E2E_SEED_TOKEN` is unset.
    MissingToken,
    /// `UF_E2E_SEED_TOKEN` is present but empty/whitespace.
    EmptyToken,
}

impl std::fmt::Display for SeedConfigError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::MissingToken => write!(
                f,
                "UF_E2E_SEED_TOKEN must be set when e2e-host-conformance is enabled"
            ),
            Self::EmptyToken => write!(
                f,
                "UF_E2E_SEED_TOKEN must be non-empty when e2e-host-conformance is enabled"
            ),
        }
    }
}

impl std::error::Error for SeedConfigError {}

/// Load and validate the bearer token once during router construction.
///
/// # Errors
///
/// Returns [`SeedConfigError`] when `UF_E2E_SEED_TOKEN` is missing or empty.
pub fn require_seed_token() -> Result<Arc<str>, SeedConfigError> {
    let raw = std::env::var("UF_E2E_SEED_TOKEN").map_err(|_| SeedConfigError::MissingToken)?;
    let trimmed = raw.trim();
    if trimmed.is_empty() {
        return Err(SeedConfigError::EmptyToken);
    }
    Ok(Arc::from(trimmed))
}

/// Create fixture-only System Valence. This never derives from a session actor.
fn fixture_system_valence(
    router: Arc<valence::DatabaseRouter>,
    default_backend_key: &str,
) -> Result<Valence, SeedError> {
    Valence::builder()
        .database_router(router)
        .default_backend_key(default_backend_key.to_string())
        .with_actor(Actor::initialize_system_context())
        .build()
        .map_err(|_| SeedError::Persistence {
            operation: "fixture_system_valence",
        })
}

fn bearer_matches(headers: &HeaderMap, expected: &str) -> bool {
    let Some(value) = headers.get(axum::http::header::AUTHORIZATION) else {
        return false;
    };
    let Ok(raw) = value.to_str() else {
        return false;
    };
    let Some(presented) = raw.strip_prefix("Bearer ") else {
        return false;
    };
    constant_time_eq(presented.as_bytes(), expected.as_bytes())
}

/// Constant-time compare for bearer tokens (length mismatch fails closed).
fn constant_time_eq(a: &[u8], b: &[u8]) -> bool {
    if a.len() != b.len() {
        return false;
    }
    let mut diff = 0u8;
    for (x, y) in a.iter().zip(b.iter()) {
        diff |= x ^ y;
    }
    diff == 0
}

fn map_seed_error(err: &SeedError) -> (StatusCode, String) {
    let status = match err {
        SeedError::UnknownScenario { .. } | SeedError::InvalidInput { .. } => {
            StatusCode::BAD_REQUEST
        }
        SeedError::Persistence { .. }
        | SeedError::Crypto { .. }
        | SeedError::Trust(_)
        | SeedError::Contact(_) => StatusCode::INTERNAL_SERVER_ERROR,
    };
    log::warn!(
        "[e2e_seed] operation=e2e_seed_request reason_class={}",
        err.reason_class()
    );
    (status, err.reason_class().to_string())
}

async fn seed_handler(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(body): Json<SeedRequest>,
) -> Result<Json<SeedAck>, (StatusCode, String)> {
    let scenario = body.scenario.clone();
    log::debug!("[e2e_seed] operation=e2e_seed_request scenario={scenario}");

    let Some(token) = state.e2e_seed_token.as_ref() else {
        log::warn!("[e2e_seed] operation=e2e_seed_denied reason_class=missing_token_config");
        return Err((StatusCode::UNAUTHORIZED, "unauthorized".into()));
    };
    if !bearer_matches(&headers, token) {
        log::warn!("[e2e_seed] operation=e2e_seed_denied reason_class=token_mismatch");
        return Err((StatusCode::UNAUTHORIZED, "unauthorized".into()));
    }

    let valence = fixture_system_valence(
        Arc::clone(&state.valence_router),
        &state.default_backend_key,
    )
    .map_err(|e| map_seed_error(&e))?;
    let response = run_seed(&valence, body)
        .await
        .map_err(|e| map_seed_error(&e))?;

    log::debug!("[e2e_seed] operation=e2e_seed_request scenario={scenario} outcome=ok");
    Ok(Json(SeedAck {
        scenario: response.scenario,
        email: response.email,
    }))
}

/// Mount `POST /api/test/seed-data` on a router that already has [`AppState`].
pub fn mount_seed_route(router: Router<AppState>) -> Router<AppState> {
    router.route("/api/test/seed-data", post(seed_handler))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::{Mutex, MutexGuard, PoisonError};

    static ENV_LOCK: Mutex<()> = Mutex::new(());

    fn lock_seed_env() -> MutexGuard<'static, ()> {
        ENV_LOCK.lock().unwrap_or_else(PoisonError::into_inner)
    }

    #[test]
    fn seed_token_missing_fails_configuration() {
        let _guard = lock_seed_env();
        std::env::remove_var("UF_E2E_SEED_TOKEN");
        let err = require_seed_token().expect_err("missing token");
        assert_eq!(err, SeedConfigError::MissingToken);
    }

    #[test]
    fn seed_token_empty_fails_configuration() {
        let _guard = lock_seed_env();
        std::env::set_var("UF_E2E_SEED_TOKEN", "   ");
        let err = require_seed_token().expect_err("empty token");
        assert_eq!(err, SeedConfigError::EmptyToken);
        std::env::remove_var("UF_E2E_SEED_TOKEN");
    }

    #[test]
    fn seed_token_happy_path() {
        let _guard = lock_seed_env();
        std::env::set_var("UF_E2E_SEED_TOKEN", "test-token-abc");
        let token = require_seed_token().expect("token");
        assert_eq!(&*token, "test-token-abc");
        std::env::remove_var("UF_E2E_SEED_TOKEN");
    }

    #[test]
    fn bearer_matches_exact_prefix() {
        let mut headers = HeaderMap::new();
        headers.insert(
            axum::http::header::AUTHORIZATION,
            "Bearer secret".parse().expect("header"),
        );
        assert!(bearer_matches(&headers, "secret"));
        assert!(!bearer_matches(&headers, "other"));
    }

    #[test]
    fn bearer_rejects_missing_and_non_bearer() {
        let empty = HeaderMap::new();
        assert!(!bearer_matches(&empty, "secret"));

        let mut basic = HeaderMap::new();
        basic.insert(
            axum::http::header::AUTHORIZATION,
            "Basic secret".parse().expect("header"),
        );
        assert!(!bearer_matches(&basic, "secret"));
    }

    #[test]
    fn seed_ack_omits_sensitive_fields() {
        let ack = SeedAck {
            scenario: "auth_basic_user".into(),
            email: "a@example.test".into(),
        };
        let json = serde_json::to_string(&ack).expect("json");
        assert!(!json.contains("password"));
        assert!(!json.contains("totp"));
        assert!(!json.contains("reset"));
        assert!(json.contains("auth_basic_user"));
        assert!(json.contains("a@example.test"));
    }
}
