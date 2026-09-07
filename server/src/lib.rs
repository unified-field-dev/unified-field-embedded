//! Full-product SSR host: Spectra, Valence, Chronon, Boson, Photon, Higgs, and Axum Leptos.
//!
//! Boots SQLite adapters in process, builds [`HiggsConfig`], then serves the product
//! shell (platform admin apps, counter, auth, tags, welcome) on one port. Marketing
//! and Coming Soon stay on `unified-field-site`; this template is the customize /
//! grow-your-app skeleton.
//!
//! ## Features
//!
//! - **Host process entry** — Resolves [`listen_addr`] and runs [`run`] until exit,
//!   binding loopback `:3000` by default. [Get started](#serve-host)
//! - **Listen address** — Parses `SITE_ADDR` / `LEPTOS_SITE_ADDR` into a
//!   [`SocketAddr`] via [`listen_addr`]. [Get started](#resolve-listen-address)
//! - **HTTP router stack** — Builds the HTTP stack via [`build_router`]: platform
//!   runtimes, sessions, Photon WS, static pkg, and Leptos routes.
//!   [Get started](#build-axum-router)
//! - **Embedded platform** — Composes Spectra → Valence → Chronon → Boson → Photon →
//!   [`HiggsConfig`] without HTTP via [`platform::compose_embedded_platform`].
//!   [Get started](#compose-embedded-platform)
//! - **Valence SQLite router** — Opens SQLite and builds the process
//!   [`valence::DatabaseRouter`] through [`valence_bootstrap::db_and_router`].
//!   [Get started](#bootstrap-valence-sqlite)
//! - **Process Valence factories** — Internal System mint and Higgs SSR factories via
//!   [`process_valence_factory::ProcessValenceFactory`]. Use
//!   [`process_valence_factory::ProcessValenceFactory::as_external_valence_factory`]
//!   at worker-bridge call sites. [Get started](#wire-process-valence-factories)
//! - **Host hardening (operator)** — CSRF Origin checks, POST rate limits, WebSocket
//!   caps, and file-backed sessions. Configure via `SECURITY.md` and
//!   `docs/deploy.md` in this repository (not a separate published crate API).
//!
//! ## Serve host
//!
//! [`run`] is the binary entry used by `cargo leptos` / `cargo run -p server`. It
//! resolves the listen address, builds the host, binds TCP, serves until Ctrl-C,
//! then drains Chronon. Call it from `main` at process startup when you want the
//! full product host on one port.
//!
//! **Prerequisites:** `server-embedded` (default); SQLite paths and
//! `PHOTON_TRANSPORT_KEY` as documented in the root README; site assets under
//! `target/site` when serving pkg. Process entry should set
//! `VALENCE_OWNERSHIP_UNIFIED_FETCH` (binary `main` defaults it to `0`).
//!
//! ```rust,ignore
//! use server::run;
//!
//! #[tokio::main]
//! async fn main() -> anyhow::Result<()> {
//!     // Canonical entry: `run` calls listen_addr → build_host → bind_tcp → serve.
//!     run().await?;
//!     println!("embedded host serve returned");
//!     Ok(())
//! }
//! ```
//!
//! On success logs `embedded host listening on http://…` and accepts HTTP until
//! shutdown. Invalid `SITE_ADDR` returns an error from [`listen_addr`]. Next:
//! [Resolve listen address](#resolve-listen-address) or [Build Axum
//! router](#build-axum-router).
//!
//! ## Resolve listen address
//!
//! [`listen_addr`] picks the bind target before TCP listen. It defaults to loopback
//! `:3000` so a local `cargo run` does not expose every interface. Call it once at
//! host startup (or from tests) when you need the address without calling [`run`].
//!
//! **Prerequisites:** Optional `SITE_ADDR` or `LEPTOS_SITE_ADDR` set to a valid
//! socket string; otherwise loopback is used.
//!
//! ```rust,ignore
//! use std::net::SocketAddr;
//! use server::listen_addr;
//!
//! let addr: SocketAddr = listen_addr()?;
//! assert_eq!(addr, "127.0.0.1:3000".parse::<SocketAddr>()?);
//! Ok::<(), anyhow::Error>(())
//! ```
//!
//! On success you get a [`SocketAddr`]. A malformed env value returns `Err`. Variant:
//! set `SITE_ADDR=0.0.0.0:3000` when LAN bind is intentional.
//!
//! ## Build Axum router
//!
//! [`build_router`] opens the embedded platform, seeds Valence tables and demo data,
//! then layers session auth, Photon WebSocket routes, compression, static files, and
//! Leptos SSR. Call it at host boot when you need the `Router` without owning the
//! listen loop (tests, custom `serve`).
//!
//! **Prerequisites:** Same env as [`platform::compose_embedded_platform`]; Leptos
//! configuration available via `get_configuration`.
//!
//! ```rust,ignore
//! use axum::Router;
//! use server::build_router;
//!
//! let app: Router = build_router().await?;
//! // build_router calls compose_embedded_platform() then mounts Leptos + Photon WS
//! println!("router routes composed");
//! let _: Router = app;
//! ```
//!
//! On success you get a ready Axum `Router`. SQLite open failures and missing Photon
//! transport keys fail closed during platform compose. Variant: call
//! [`platform::compose_embedded_platform`] alone when HTTP is not needed.
//!
//! ## Compose embedded platform
//!
//! [`platform::compose_embedded_platform`] installs Spectra SQLite, boots Valence,
//! builds Chronon/Boson/Photon runtimes, and returns [`platform::EmbeddedPlatform`]
//! with a wired [`HiggsConfig`]. Use this at process start for labs and headless
//! checks that mirror production boot without Leptos.
//!
//! **Prerequisites:** Writable data dirs; `PHOTON_TRANSPORT_KEY`; optional
//! `*_SQLITE_PATH` overrides.
//!
//! ```rust,ignore
//! use server::platform::{compose_embedded_platform, EmbeddedPlatform};
//! use higgs::HiggsConfig;
//!
//! let platform: EmbeddedPlatform = compose_embedded_platform().await?;
//! let higgs: &HiggsConfig = platform.higgs.as_ref();
//! assert!(!platform.default_backend_key.is_empty());
//! assert!(std::sync::Arc::strong_count(&platform.higgs) >= 1);
//! println!("HiggsConfig ready via EmbeddedPlatform");
//! ```
//!
//! On success Chronon run loops stay alive on the returned handles. Failures from
//! any adapter propagate as `anyhow::Error`. Next: [Bootstrap Valence
//! SQLite](#bootstrap-valence-sqlite) or wire `higgs` into Axum via [`build_router`].
//!
//! ## Bootstrap Valence SQLite
//!
//! [`valence_bootstrap::db_and_router`] connects the process SQLite file and builds a
//! shared [`valence::DatabaseRouter`] with logical groups (default, chronon, boson,
//! photon, permissions, secrets). Call it once at boot after Spectra install and
//! before Chronon/Boson/Photon factories need a router.
//!
//! **Prerequisites:** Writable parent for `VALENCE_SQLITE_PATH` (default
//! `data/{{project-name}}.sqlite3`), or `:memory:` for ephemeral labs.
//!
//! ```rust,ignore
//! use server::valence_bootstrap::{db_and_router, BootstrappedValence};
//! use valence::DatabaseRouter;
//!
//! let boot: BootstrappedValence = db_and_router().await?;
//! let router: &DatabaseRouter = boot.router.as_ref();
//! assert!(!boot.default_backend_key.is_empty());
//! println!("router backends ready: {}", router as *const _ as usize != 0);
//! Ok::<(), anyhow::Error>(())
//! ```
//!
//! On success you hold an `Arc<DatabaseRouter>` and default backend key. Connect or
//! permission harden failures return `Err`. Customize: change `VALENCE_SQLITE_PATH` or
//! edit [`valence_bootstrap::router_groups`] when adding logical stores.
//!
//! ## Wire process Valence factories
//!
//! [`process_valence_factory::ProcessValenceFactory`] builds per-request
//! [`valence::Valence`] from the process router. Internal factories may mint System
//! for Chronon/Boson/Photon boot; the Higgs factory backs SSR `valence()` / escape
//! hatches. Wire these once inside [`platform::compose_embedded_platform`]. For
//! untrusted actor JSON, call
//! [`process_valence_factory::ProcessValenceFactory::as_external_valence_factory`]
//! at the bridge site (not held on [`platform::EmbeddedPlatform`] until a consumer
//! exists).
//!
//! **Prerequisites:** A live [`valence::DatabaseRouter`] and default backend key from
//! [`valence_bootstrap::db_and_router`].
//!
//! ```rust,ignore
//! use std::sync::Arc;
//! use server::process_valence_factory::ProcessValenceFactory;
//! use valence::{DatabaseRouter, ValenceFactory};
//! use higgs::HiggsValenceFactory;
//!
//! fn wire(router: Arc<DatabaseRouter>, key: String) {
//!     let internal: Arc<dyn ValenceFactory> =
//!         ProcessValenceFactory::as_internal_valence_factory(router.clone(), key.clone());
//!     let higgs_f: Arc<dyn HiggsValenceFactory> =
//!         ProcessValenceFactory::as_higgs_factory(router.clone(), key.clone());
//!     // When adding a worker bridge, use as_external_valence_factory there:
//!     let _external: Arc<dyn ValenceFactory> =
//!         ProcessValenceFactory::as_external_valence_factory(router, key);
//!     assert!(Arc::strong_count(&internal) >= 1);
//!     assert!(Arc::strong_count(&higgs_f) >= 1);
//! }
//! ```
//!
//! On success Chronon/Boson/Photon boot with the internal factory. Customize: keep
//! System mint only on the internal/Higgs path; never hand the internal factory to
//! worker bridges.
//!
//! ## Feature flags
//!
//! | Flag | Role |
//! |------|------|
//! | `server-embedded` (default) | SQLite runtimes, auth, counter worker, full platform admin UI |
//! | `ssr` | Empty marker kept for cargo-leptos feature pairing |

use std::net::SocketAddr;
use std::path::PathBuf;
use std::sync::Arc;

use axum::error_handling::HandleErrorLayer;
use axum::extract::{DefaultBodyLimit, FromRef};
use axum::http::{header, HeaderValue, StatusCode};
use axum::middleware::from_fn;
use axum::{BoxError, Extension, Router};
use axum_login::AuthManagerLayerBuilder;
use higgs::HiggsConfig;
use lepton_host_adapter::files::{files_routes, FileByteBackend, FilesConfig, LocalDiskBlobStore};
use lepton_host_adapter::{session_snapshot_middleware, Backend, PhotonAuth};
use leptos::config::get_configuration;
use leptos::prelude::*;
use leptos_axum::{generate_route_list, LeptosRoutes};
use photon::Photon;
use photon_axum::{ws_router, HasPhoton};
use soliton::listener::{resolve_listen_addr, ListenAddrDefault};
use soliton::{bind_tcp, health_router, serve_with_graceful_shutdown};
use std::time::Duration;
use tower::limit::ConcurrencyLimitLayer;
use tower::timeout::TimeoutLayer;
use tower::ServiceBuilder;
use tower_http::compression::CompressionLayer;
use tower_http::services::{ServeDir, ServeFile};
use tower_http::set_header::SetResponseHeaderLayer;
use tower_sessions::session_store::ExpiredDeletion;
use tower_sessions::{Expiry, SessionManagerLayer};

mod expiring_session_store;
pub mod platform;
mod request_hardening;

#[cfg(feature = "e2e-host-conformance")]
mod e2e_support;
pub mod process_valence_factory;
pub mod valence_bootstrap;

use expiring_session_store::FileSessionStore;
use request_hardening::{request_hardening_middleware, CsrfConfig};

// Force-link identity Valence schemas (inventory) — single registration path.
#[cfg(feature = "server-embedded")]
use lepton_identity as _;
// Force-link counter worker schemas (inventory) when the optional dep is enabled.
#[cfg(feature = "server-embedded")]
use counter_app_worker as _;
#[cfg(feature = "server-embedded")]
use gauge as _;
#[cfg(feature = "server-embedded")]
use neutrino as _;
#[cfg(feature = "server-embedded")]
use record_history as _;
#[cfg(feature = "server-embedded")]
use tag as _;
#[cfg(feature = "server-embedded")]
use uf_welcome as _;

/// Shared Axum/Leptos state held on the process [`Router`].
///
/// Carries Leptos options, Higgs, Photon, Spectra, and the SQLite runtime handles
/// that must stay alive for the process lifetime. Customize when you need extra
/// request context (add a field + `provide_context` in [`build_router`]), not for
/// swapping storage paths (use env / `*-uf-embedded` builders instead).
///
/// # Examples
///
/// ```rust,ignore
/// use server::AppState;
/// // Built inside build_router; FromRef extracts LeptosOptions for SSR.
/// fn uses_leptos(state: &AppState) {
///     let _opts = axum::extract::FromRef::<AppState>::from_ref(state);
///     println!("AppState carries Higgs + Photon for request handlers");
/// }
/// ```
#[derive(Clone)]
pub struct AppState {
    leptos_options: LeptosOptions,
    higgs: Arc<HiggsConfig>,
    /// Shared by Photon files routes and the optional E2E seed handler.
    #[allow(dead_code)]
    valence_router: Arc<valence::DatabaseRouter>,
    /// Logical backend used by the optional E2E seed handler.
    #[allow(dead_code)]
    default_backend_key: String,
    photon: Arc<Photon>,
    #[allow(dead_code)]
    spectra: Arc<spectra::Spectra>,
    /// Keeps Chronon SQLite runtime alive for the process lifetime.
    #[allow(dead_code)]
    chronon: Option<Arc<chronon_uf_embedded::ChrononEmbeddedRuntime>>,
    /// Keeps Boson SQLite runtime alive for the process lifetime.
    #[allow(dead_code)]
    boson: Option<Arc<boson_uf_embedded::BosonEmbeddedRuntime>>,
    /// Keeps Photon SQLite runtime alive for the process lifetime.
    #[allow(dead_code)]
    photon_runtime: Option<Arc<photon_uf_embedded::PhotonEmbeddedRuntime>>,
    /// WebSocket `Origin` allowlist (`UF_PUBLIC_BASE_URL` + `PHOTON_LEPTOS_ALLOWED_ORIGINS`).
    allowed_ws_origins: Arc<[String]>,
    /// Present only in host-conformance builds.
    #[cfg(feature = "e2e-host-conformance")]
    e2e_seed_token: Option<Arc<str>>,
}

impl FromRef<AppState> for LeptosOptions {
    fn from_ref(state: &AppState) -> LeptosOptions {
        state.leptos_options.clone()
    }
}

impl HasPhoton for AppState {
    fn photon_arc(&self) -> Arc<Photon> {
        self.photon.clone()
    }

    fn allow_ws_origin(&self, origin: Option<&str>) -> bool {
        let Some(origin) = origin else {
            return false;
        };
        self.allowed_ws_origins
            .iter()
            .any(|allowed| allowed == origin)
    }
}

/// Build the Photon WebSocket `Origin` allowlist for [`HasPhoton`].
///
/// Seeds from `UF_PUBLIC_BASE_URL`, adds loopback `:3000` origins only when that
/// base is itself loopback/localhost, then merges comma-separated
/// `PHOTON_LEPTOS_ALLOWED_ORIGINS`. Production HTTPS bases do not permanently
/// trust `http://127.0.0.1:3000`.
///
/// # Examples
///
/// ```rust,ignore
/// let origins = ws_origin_allowlist("https://app.example.com");
/// assert!(origins.iter().any(|o| o == "https://app.example.com"));
/// assert!(!origins.iter().any(|o| o == "http://127.0.0.1:3000"));
/// let local = ws_origin_allowlist("http://127.0.0.1:3000");
/// assert!(local.iter().any(|o| o == "http://localhost:3000"));
/// ```
pub(crate) fn ws_origin_allowlist(public_base: &str) -> Arc<[String]> {
    let mut origins = Vec::new();
    let trimmed = public_base.trim().trim_end_matches('/');
    if !trimmed.is_empty() {
        origins.push(trimmed.to_string());
    }
    if public_base_is_loopback(trimmed) {
        for o in [
            "http://127.0.0.1:3000",
            "http://localhost:3000",
            "http://0.0.0.0:3000",
        ] {
            if !origins.iter().any(|e| e == o) {
                origins.push(o.to_string());
            }
        }
    }
    if let Ok(raw) = std::env::var("PHOTON_LEPTOS_ALLOWED_ORIGINS") {
        for part in raw.split(',') {
            let o = part.trim().trim_end_matches('/');
            if !o.is_empty() && !origins.iter().any(|e| e == o) {
                origins.push(o.to_string());
            }
        }
    }
    Arc::from(origins)
}

fn public_base_is_loopback(public_base: &str) -> bool {
    let lower = public_base.to_ascii_lowercase();
    lower.contains("127.0.0.1")
        || lower.contains("localhost")
        || lower.contains("0.0.0.0")
        || lower.contains("[::1]")
}

/// Decide whether the session cookie should set `Secure`.
///
/// HTTPS public bases default on; HTTP previews stay off so browsers accept the
/// cookie on `http://127.0.0.1:3000`. Override with `UF_SESSION_SECURE=0|1` /
/// `true|false`. Customize when terminating TLS at a proxy while the app still
/// sees `http://` (force `1`) or when debugging Secure issues locally (force `0`).
///
/// # Examples
///
/// ```rust,ignore
/// assert!(session_cookie_secure("https://app.example.com"));
/// assert!(!session_cookie_secure("http://127.0.0.1:3000"));
/// ```
pub(crate) fn session_cookie_secure(public_base: &str) -> bool {
    if let Ok(raw) = std::env::var("UF_SESSION_SECURE") {
        let v = raw.trim();
        if v == "1" || v.eq_ignore_ascii_case("true") {
            return true;
        }
        if v == "0" || v.eq_ignore_ascii_case("false") {
            return false;
        }
    }
    public_base
        .trim()
        .to_ascii_lowercase()
        .starts_with("https://")
}

/// Axum router plus Chronon handle for graceful host shutdown.
///
/// Prefer [`build_host`] from [`run`] so Chronon loops are signaled after serve
/// returns. [`build_router`] returns only the router for tests that do not need
/// shutdown ownership.
pub struct BuiltHost {
    /// Ready HTTP stack (static pkg, Photon WS, Leptos, auth).
    pub router: axum::Router,
    /// Chronon runtime keep-alive; notify `shutdown` and abort `run_loop` on exit.
    pub chronon: Arc<chronon_uf_embedded::ChrononEmbeddedRuntime>,
}

/// Build the Axum router for the embedded Leptos shell (+ platform runtimes).
///
/// Calls [`platform::compose_embedded_platform`], seeds Valence/demo data, then
/// mounts sessions, Photon WS, static pkg, and Leptos routes. Prefer this at host
/// boot when HTTP is required; use [`platform::compose_embedded_platform`] alone for
/// headless labs. Customize middleware/layers here; swap runtimes in `platform` /
/// `*-uf-embedded` instead.
///
/// Expects `VALENCE_OWNERSHIP_UNIFIED_FETCH` to be set by the process entry
/// (`server` binary `main` or lab installer) — this function does not mutate env.
///
/// # Errors
///
/// Returns an error when platform compose, Valence seed, auth services, or Leptos
/// configuration fails.
///
/// # Examples
///
/// See [Build Axum router](index.html#build-axum-router).
pub async fn build_router() -> anyhow::Result<axum::Router> {
    Ok(build_host().await?.router)
}

/// Build the router and retain the Chronon runtime for process shutdown.
///
/// Same compose/seed/mount path as [`build_router`], plus an owned Chronon handle
/// so [`run`] can signal shutdown after Axum stops accepting.
///
/// # Errors
///
/// Same as [`build_router`].
pub async fn build_host() -> anyhow::Result<BuiltHost> {
    let platform = platform::compose_embedded_platform().await?;
    let chronon = platform.chronon.clone();
    let router = build_router_from_platform(platform).await?;
    Ok(BuiltHost { router, chronon })
}

async fn build_router_from_platform(
    platform: platform::EmbeddedPlatform,
) -> anyhow::Result<axum::Router> {
    let valence_router = platform.valence_router.clone();
    let default_backend_key = platform.default_backend_key.clone();
    let spectra = platform.spectra.clone();
    let photon = platform.photon.clone();
    let higgs = platform.higgs.clone();

    let chronon_backend = Arc::clone(&platform.chronon.backend);
    let script_registry = Arc::clone(&platform.chronon.registry);
    let boson_backend = Arc::clone(&platform.boson.backend);

    // TTL ensure + deletion/TTL Chronon registration (mirror archive web-app-template boot).
    {
        let boot_valence = valence::Valence::builder()
            .database_router(valence_router.clone())
            .default_backend_key(default_backend_key.clone())
            .with_actor(valence::Actor::initialize_system_context())
            .build()
            .map_err(|e| anyhow::anyhow!("boot valence build failed: {e}"))?;
        boot_valence
            .sync_typed_tables_from_registry()
            .await
            .map_err(|e| anyhow::anyhow!("typed table sync failed: {e}"))?;
        if let Err(e) = boot_valence.ensure_ttl_for_all().await {
            log::warn!("[server] ensure_ttl_for_all failed: {e}");
        }
        valence_platform::deletion::dispatch::register_deletion_dispatch(Arc::clone(
            &chronon_backend,
        ));
        valence_platform::ttl::sweep::register_ttl_service(Arc::clone(&chronon_backend));
        valence_platform::iter::dispatch::register_iter_dispatch(Arc::clone(&chronon_backend));
        if let Err(e) =
            valence_platform::deletion::sweep::resync_valence_deletion_sweep_job_cron_if_present(
                chronon_backend.as_ref(),
                &boot_valence,
            )
            .await
        {
            log::warn!("[server] valence deletion sweep cron resync failed: {e}");
        }
        if let Err(e) = valence_platform::ttl::sweep::resync_valence_ttl_sweep_job_cron_if_present(
            chronon_backend.as_ref(),
            &boot_valence,
        )
        .await
        {
            log::warn!("[server] valence TTL sweep cron resync failed: {e}");
        }

        // Global counter create policy is AUTHENTICATED-only; anonymous demo clicks
        // only UPDATE. Seed the singleton under System so first-click create isn't required.
        #[cfg(feature = "server-embedded")]
        {
            use valence::Model;
            match counter_app_worker::generated::Counter::get("singleton", &boot_valence).await {
                Ok(None) => match counter_app_worker::service::set_global(0, &boot_valence).await {
                    Ok(_) => log::info!("[server] seeded counter singleton at 0"),
                    Err(e) => log::warn!("[server] seed counter singleton failed: {e}"),
                },
                Ok(Some(_)) => {}
                Err(e) => log::warn!("[server] counter singleton probe failed: {e}"),
            }
        }

        // Seed counter demo bots after Boson is configured so UserCounter side effects
        // can enqueue leaderboard notification tasks (mirrors web-app-template boot).
        #[cfg(feature = "server-embedded")]
        if let Err(e) =
            counter_app_worker::scripts::ensure_bot_users::ensure_bot_users_seed(&boot_valence)
                .await
        {
            log::warn!("[server] ensure_bot_users_seed failed: {e}");
        }
    }

    let conf = get_configuration(None)?;
    let leptos_options = conf.leptos_options;
    let routes = generate_route_list(app::App);

    let public_base =
        std::env::var("UF_PUBLIC_BASE_URL").unwrap_or_else(|_| "http://127.0.0.1:3000".to_string());
    let auth_services = {
        use lepton_auth::services::LeptonAuthServicesBuilder;
        use neutrino::vault::store_from_valence;
        use uf_oauth_boot::resolve_oauth_config_from_neutrino;

        let mut builder = LeptonAuthServicesBuilder::new().public_base_url(public_base.clone());
        builder = builder.email(
            lepton_smtp::EmailServiceBuilder::new()
                .noop()
                .build()
                .map_err(|e| anyhow::anyhow!("email service: {e}"))?,
        );
        builder = builder.sms(
            lepton_sms::SmsServiceBuilder::new()
                .noop()
                .build()
                .map_err(|e| anyhow::anyhow!("sms service: {e}"))?,
        );
        let oauth_valence = valence::Valence::builder()
            .database_router(valence_router.clone())
            .default_backend_key(default_backend_key.clone())
            .with_actor(valence::Actor::initialize_system_context())
            .build()
            .map_err(|e| anyhow::anyhow!("oauth boot valence: {e}"))?;
        let store = store_from_valence(oauth_valence);
        match resolve_oauth_config_from_neutrino(&store, &public_base, true).await {
            Ok(Some(cfg)) => {
                builder = builder.oauth(cfg.into_lepton());
            }
            Ok(None) => {}
            Err(e) => {
                log::warn!("[server] OAuth Neutrino resolve skipped: {e}");
            }
        }
        #[cfg(feature = "server-embedded")]
        {
            // Prefer host origin; fall back to loopback for local embedded.
            let without_scheme = public_base
                .trim_start_matches("https://")
                .trim_start_matches("http://");
            let rp_id = without_scheme
                .split(['/', ':', '?'])
                .next()
                .filter(|s| !s.is_empty())
                .unwrap_or("127.0.0.1")
                .to_string();
            builder = builder.webauthn_rp(lepton_auth::devices::WebauthnRpConfig {
                rp_id,
                rp_origin: public_base.clone(),
                rp_name: "{{app_title}}".into(),
            });
        }
        Arc::new(
            builder
                .build()
                .map_err(|e| anyhow::anyhow!("LeptonAuthServices build: {e}"))?,
        )
    };

    let allowed_ws_origins = ws_origin_allowlist(&public_base);
    #[cfg(feature = "e2e-host-conformance")]
    let e2e_seed_token = Some(e2e_support::require_seed_token()?);
    let state = AppState {
        leptos_options: leptos_options.clone(),
        higgs: higgs.clone(),
        valence_router: valence_router.clone(),
        default_backend_key: default_backend_key.clone(),
        photon: photon.clone(),
        spectra: spectra.clone(),
        chronon: Some(platform.chronon.clone()),
        boson: Some(platform.boson.clone()),
        photon_runtime: Some(platform.photon_runtime.clone()),
        allowed_ws_origins: Arc::clone(&allowed_ws_origins),
        #[cfg(feature = "e2e-host-conformance")]
        e2e_seed_token,
    };

    // Durable file session store with background expired-row deletion.
    let session_dir = std::env::var("UF_SESSION_DIR").unwrap_or_else(|_| "data/sessions".into());
    let session_store = FileSessionStore::new(&session_dir)
        .map_err(|e| anyhow::anyhow!("session store dir {session_dir}: {e}"))?;
    let deletion_store = session_store.clone();
    tokio::task::spawn(async move {
        let mut interval = tokio::time::interval(tokio::time::Duration::from_secs(60));
        interval.tick().await;
        loop {
            interval.tick().await;
            let _ = deletion_store.delete_expired().await;
        }
    });
    // Session cookie: Secure follows UF_PUBLIC_BASE_URL (HTTP previews vs HTTPS).
    let session_layer = SessionManagerLayer::new(session_store)
        .with_secure(session_cookie_secure(&public_base))
        .with_same_site(tower_sessions::cookie::SameSite::Lax)
        .with_expiry(Expiry::OnInactivity(time::Duration::hours(24)))
        .with_name("session")
        .with_path("/");
    let backend = Backend::new(Arc::clone(higgs.valence_factory()));
    let auth_layer = AuthManagerLayerBuilder::new(backend, session_layer).build();
    let csrf_config = CsrfConfig {
        public_base: public_base.clone(),
        allowed_origins: Arc::clone(&allowed_ws_origins),
    };

    let state_for_shell = state.clone();
    let state_for_ctx = state.clone();
    let auth_services_for_ctx = auth_services.clone();

    // `/health` stays off AuthManagerLayer so probes get a plain 2xx.
    #[cfg_attr(not(feature = "e2e-host-conformance"), allow(unused_mut))]
    let mut public_probes = Router::<AppState>::new().merge(health_router());
    #[cfg(feature = "e2e-host-conformance")]
    {
        // The route bypasses session auth but requires the startup bearer token.
        public_probes = e2e_support::mount_seed_route(public_probes);
    }

    // Layer order matters for a template fork:
    //   1) static /pkg /fonts /css before Leptos (avoid SSR-wrapping WASM as HTML)
    //   2) Photon WS routes
    //   3) Leptos SSR + shell
    //   4) files / compression / session snapshot / auth / valence Extension
    // Serve site assets before Leptos routes. A catch-all `/*any` 404 would otherwise
    // SSR-wrap `/pkg/*`, `/fonts/*`, and root CSS as `text/html` (breaks WASM + fonts).
    let site_root = PathBuf::from(leptos_options.site_root.as_ref());
    let pkg_dir = site_root.join(leptos_options.site_pkg_dir.as_ref());
    let fonts_dir = site_root.join("fonts");

    let mut app = Router::new()
        .nest_service("/pkg", ServeDir::new(pkg_dir))
        .nest_service("/fonts", ServeDir::new(fonts_dir))
        .route_service(
            "/favicon.ico",
            ServeFile::new(site_root.join("favicon.ico")),
        )
        .route_service(
            "/apple-touch-icon.png",
            ServeFile::new(site_root.join("apple-touch-icon.png")),
        )
        .route_service("/main.css", ServeFile::new(site_root.join("main.css")))
        .route_service(
            "/orbital-theme-baseline.css",
            ServeFile::new(site_root.join("orbital-theme-baseline.css")),
        );
    // Photon WebSocket mount (needs PHOTON_TRANSPORT_KEY + Origin allowlist).
    app = ws_router::<AppState, PhotonAuth>(app);
    app = app
        .leptos_routes_with_context(
            &state,
            routes,
            move || {
                // Per-request contexts for SSR server fns (Higgs, Chronon, Boson, Photon, auth).
                gauge_app::wire_gauge_permissions();
                let higgs = &state_for_ctx.higgs;
                provide_context(higgs.clone());
                provide_context(higgs.valence_factory().clone());
                provide_context(chronon_backend.clone());
                provide_context(script_registry.clone());
                provide_context(boson_backend.clone());
                provide_context(photon.clone());
                provide_context(spectra.clone());
                lepton_auth::services::provide_auth_services(auth_services_for_ctx.clone());
            },
            move || {
                let options = LeptosOptions::from_ref(&state_for_shell);
                app::shell(options)
            },
        )
        .fallback(leptos_axum::file_and_error_handler_with_context::<
            AppState,
            _,
        >(move || {}, app::shell));

    let file_store: Arc<dyn FileByteBackend> = Arc::new(LocalDiskBlobStore::default_uploads());
    let files_config = FilesConfig::new(default_backend_key.clone());

    let secure_cookies = session_cookie_secure(&public_base);
    let mut protected = app
        .merge(files_routes(file_store, files_config))
        // Compress `/pkg` WASM chunks and other static/SSR responses (br/gzip).
        .layer(CompressionLayer::new())
        .layer(from_fn(session_snapshot_middleware))
        .layer(auth_layer)
        .layer(Extension(valence_router))
        // Inner → outer: hardening sees CsrfConfig Extension inserted by outer layer.
        .layer(from_fn(request_hardening_middleware))
        .layer(Extension(csrf_config))
        .layer(DefaultBodyLimit::max(2 * 1024 * 1024))
        .layer(ConcurrencyLimitLayer::new(256))
        .layer(
            ServiceBuilder::new()
                .layer(HandleErrorLayer::new(|_: BoxError| async {
                    StatusCode::REQUEST_TIMEOUT
                }))
                .layer(TimeoutLayer::new(Duration::from_secs(60))),
        )
        .layer(SetResponseHeaderLayer::overriding(
            header::X_CONTENT_TYPE_OPTIONS,
            HeaderValue::from_static("nosniff"),
        ))
        .layer(SetResponseHeaderLayer::overriding(
            header::X_FRAME_OPTIONS,
            HeaderValue::from_static("DENY"),
        ))
        .layer(SetResponseHeaderLayer::overriding(
            header::REFERRER_POLICY,
            HeaderValue::from_static("strict-origin-when-cross-origin"),
        ))
        .layer(SetResponseHeaderLayer::overriding(
            header::CONTENT_SECURITY_POLICY,
            HeaderValue::from_static(
                "default-src 'self'; script-src 'self' 'unsafe-inline' 'wasm-unsafe-eval'; \
                 style-src 'self' 'unsafe-inline'; img-src 'self' data: blob:; \
                 connect-src 'self' ws: wss:; font-src 'self'; frame-ancestors 'none'; \
                 base-uri 'self'; form-action 'self'",
            ),
        ));
    if secure_cookies {
        protected = protected.layer(SetResponseHeaderLayer::if_not_present(
            header::STRICT_TRANSPORT_SECURITY,
            HeaderValue::from_static("max-age=31536000; includeSubDomains"),
        ));
    }

    Ok(public_probes.merge(protected).with_state(state))
}

/// Bind address for the embedded host (`LEPTOS_SITE_ADDR` / `SITE_ADDR` or loopback).
///
/// Defaults to `127.0.0.1:3000` so a local `cargo run` does not expose every
/// interface (EM-02). Set `SITE_ADDR=0.0.0.0:3000` when LAN bind is intentional.
/// Customize only the env var unless you need a different default port in code.
///
/// # Errors
///
/// Returns an error when an env var is set but is not a valid socket address.
///
/// # Examples
///
/// See [Resolve listen address](index.html#resolve-listen-address).
pub fn listen_addr() -> anyhow::Result<SocketAddr> {
    resolve_listen_addr(ListenAddrDefault::Loopback { port: 3000 })
        .map_err(|e| anyhow::anyhow!("{e}"))
}

/// Run the embedded host until Ctrl-C (or equivalent), then drain Chronon.
///
/// Resolves [`listen_addr`], builds the host via [`build_host`], binds TCP, and
/// serves with graceful shutdown. After Axum stops, signals Chronon shutdown and
/// aborts the run loop (same pattern as `{{crate_name}}_e2e::IsolatedLab` Drop).
/// Call from `main` (see `server/src/main.rs`) at process startup.
///
/// # Errors
///
/// Propagates listen-address, router build, bind, or serve failures.
///
/// # Examples
///
/// See [Serve host](index.html#serve-host).
pub async fn run() -> anyhow::Result<()> {
    let addr = listen_addr()?;
    let BuiltHost { router, chronon } = build_host().await?;
    let listener = bind_tcp(&addr).await?;
    log::info!("embedded host listening on http://{addr}");
    let serve_result = serve_with_graceful_shutdown(listener, router, async {
        let _ = tokio::signal::ctrl_c().await;
    })
    .await;
    chronon.shutdown.notify_waiters();
    chronon.run_loop.abort();
    serve_result
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Mutex;

    static LISTEN_ADDR_ENV_LOCK: Mutex<()> = Mutex::new(());

    /// TM-08: this test binary only builds when the seed feature is off.
    #[cfg(not(feature = "e2e-host-conformance"))]
    #[test]
    fn e2e_host_conformance_feature_absent_by_default() {
        // Empty body: presence of this cfg(test) item is the assertion.
    }

    /// TM-08: feature builds compile the private seed module.
    #[cfg(feature = "e2e-host-conformance")]
    #[test]
    fn e2e_host_conformance_feature_compiles_seed_module() {
        let _ = std::any::type_name_of_val(&e2e_support::require_seed_token);
    }

    #[test]
    fn listen_addr_default_happy_path() {
        let _guard = LISTEN_ADDR_ENV_LOCK.lock().expect("listen addr env lock");
        std::env::remove_var("SITE_ADDR");
        std::env::remove_var("LEPTOS_SITE_ADDR");
        let addr = listen_addr().expect("default listen addr");
        assert_eq!(addr, SocketAddr::from(([127, 0, 0, 1], 3000)));
    }

    #[test]
    fn listen_addr_valid_env_happy_path() {
        let _guard = LISTEN_ADDR_ENV_LOCK.lock().expect("listen addr env lock");
        std::env::set_var("SITE_ADDR", "127.0.0.1:8080");
        std::env::remove_var("LEPTOS_SITE_ADDR");
        let addr = listen_addr().expect("SITE_ADDR listen addr");
        assert_eq!(addr, "127.0.0.1:8080".parse().expect("addr"));
        std::env::remove_var("SITE_ADDR");
    }

    #[test]
    fn listen_addr_invalid_env_returns_error_sad() {
        let _guard = LISTEN_ADDR_ENV_LOCK.lock().expect("listen addr env lock");
        std::env::set_var("SITE_ADDR", "not-a-socket-addr");
        std::env::remove_var("LEPTOS_SITE_ADDR");
        assert!(listen_addr().is_err());
        std::env::remove_var("SITE_ADDR");
    }
}
