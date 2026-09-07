//! In-process platform composition for the embedded host.
//!
//! Boot order: Spectra → Valence → Chronon → Boson → Photon → [`HiggsConfig`].
//! Leptos / Axum wiring stays in [`super::build_router`]. Customize here when you
//! swap, disable, or retarget a runtime; keep HTTP layers in `build_router`.

use std::sync::Arc;

use higgs::HiggsConfig;
use photon::Photon;
use valence::DatabaseRouter;

use crate::process_valence_factory::ProcessValenceFactory;
use crate::valence_bootstrap;

/// Fully composed embedded platform runtimes (no HTTP / Leptos shell).
///
/// Holds Valence, Spectra, Chronon/Boson/Photon runtimes, Higgs, and the external
/// Valence factory. Keep this struct alive for the process lifetime so Chronon
/// loops and Photon storage stay open. Customize fields only when adding a new
/// in-process runtime; path overrides belong in env / `*-uf-embedded` builders.
///
/// # Examples
///
/// ```rust,ignore
/// use server::platform::{compose_embedded_platform, EmbeddedPlatform};
///
/// let platform: EmbeddedPlatform = compose_embedded_platform().await?;
/// assert!(!platform.default_backend_key.is_empty());
/// println!("spectra + higgs composed");
/// Ok::<(), anyhow::Error>(())
/// ```
pub struct EmbeddedPlatform {
    /// Process-wide Valence router.
    pub valence_router: Arc<DatabaseRouter>,
    /// Default backend key for auth and factories.
    pub default_backend_key: String,
    /// Spectra handle (metrics + events SQLite).
    pub spectra: Arc<spectra::Spectra>,
    /// Chronon SQLite runtime (keeps run loops alive).
    pub chronon: Arc<chronon_uf_embedded::ChrononEmbeddedRuntime>,
    /// Boson SQLite runtime.
    pub boson: Arc<boson_uf_embedded::BosonEmbeddedRuntime>,
    /// Photon SQLite runtime.
    pub photon_runtime: Arc<photon_uf_embedded::PhotonEmbeddedRuntime>,
    /// Shared Photon client (same as `photon_runtime.photon`).
    pub photon: Arc<Photon>,
    /// Higgs config with Chronon / Boson / Photon wired.
    pub higgs: Arc<HiggsConfig>,
}

/// Compose Spectra, Valence, Chronon, Boson, Photon, and [`HiggsConfig`].
///
/// Reads the same env vars as the full host (`VALENCE_SQLITE_PATH`,
/// `CHRONON_SQLITE_PATH`, `BOSON_SQLITE_PATH`, `PHOTON_SQLITE_PATH`,
/// Spectra paths, `PHOTON_TRANSPORT_KEY`, …). Call once at process start before
/// HTTP or lab work that needs Higgs.
///
/// # Errors
///
/// Returns an error if any SQLite open, runtime build, or Higgs construction fails.
///
/// # Examples
///
/// See crate [Compose embedded platform](../index.html#compose-embedded-platform).
pub async fn compose_embedded_platform() -> anyhow::Result<EmbeddedPlatform> {
    // 1) Spectra first — metrics/events backends before Valence telemetry install.
    //    Swap/disable: edit spectra_uf_embedded or skip install_from_env below.
    let spectra = spectra_uf_embedded::install_embedded_sqlite().await?;
    valence_spectra_telemetry::install_from_env();

    // 2) Valence SQLite + logical router groups (default/chronon/boson/photon/…).
    //    Path: VALENCE_SQLITE_PATH; groups: valence_bootstrap::router_groups.
    let boot = valence_bootstrap::db_and_router().await?;
    let valence_router = boot.router;
    let default_backend_key = boot.default_backend_key;
    // 3) Factories: internal may mint System for Chronon/Boson workers and boot jobs.
    //    Workers already rebuild Valence from persisted `actor_json` on every tick —
    //    they are the consumer. Persist-time overwrite (`upsert_job_with_valence`) and
    //    `validate_external_job_actor_json` on the no-Valence upsert path are the hard
    //    gates. Use `ProcessValenceFactory::as_external_valence_factory` only for a
    //    future HTTP/worker-bridge that deserializes untrusted actor JSON at the edge.
    let valence_factory = ProcessValenceFactory::as_internal_valence_factory(
        valence_router.clone(),
        default_backend_key.clone(),
    );
    let higgs_factory = ProcessValenceFactory::as_higgs_factory(
        valence_router.clone(),
        default_backend_key.clone(),
    );

    // 4) In-process runtimes — keep returned handles alive (Chronon loops / Photon store).
    //    Path env: CHRONON_SQLITE_PATH / BOSON_SQLITE_PATH / PHOTON_SQLITE_PATH.
    //    To disable a runtime for a fork, stop building it here and drop Higgs wiring.
    let chronon_rt =
        chronon_uf_embedded::build_chronon_runtime(Arc::clone(&valence_factory)).await?;
    let boson_rt = boson_uf_embedded::build_boson_runtime(Arc::clone(&valence_factory)).await?;
    let photon_rt = photon_uf_embedded::build_runtime(Arc::clone(&valence_factory)).await?;
    let photon = photon_rt.photon.clone();

    let chronon_backend = Arc::clone(&chronon_rt.backend);
    let script_registry = Arc::clone(&chronon_rt.registry);
    let chronon_scheduler = Arc::clone(&chronon_rt.scheduler);
    let boson_backend = Arc::clone(&boson_rt.backend);

    // 5) Higgs ties Valence + Chronon + Boson + Photon for SSR / server fns.
    let higgs = Arc::new(
        HiggsConfig::builder()
            .valence_factory_arc(higgs_factory)
            .chronon(
                Arc::clone(&chronon_scheduler),
                Arc::clone(&chronon_backend),
                Arc::clone(&script_registry),
            )
            .boson(Arc::clone(&boson_backend))
            .photon(Arc::clone(&photon))
            .build()
            .map_err(|e| anyhow::anyhow!("HiggsConfig build failed: {e}"))?,
    );

    Ok(EmbeddedPlatform {
        valence_router,
        default_backend_key,
        spectra,
        chronon: Arc::new(chronon_rt),
        boson: Arc::new(boson_rt),
        photon_runtime: Arc::new(photon_rt),
        photon,
        higgs,
    })
}
