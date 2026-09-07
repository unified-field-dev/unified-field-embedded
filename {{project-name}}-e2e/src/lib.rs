//! `IsolatedLab` harness for embedded host composition (Spectra/Chronon/Boson/Photon/Higgs).
//!
//! Mirrors `server::platform::compose_embedded_platform` boot order without the
//! Leptos/Axum shell. Use this crate to smoke host composition in CI; scenario
//! IDs live under `tests/`.
//!
//! ## Features
//!
//! - **Isolated lab host smoke** — [`boot_isolated_lab`] plus [`install_lab_env`]
//!   boot the full in-process stack under a temp root for Layer-2 scenarios.
//!   [Get started](#smoke-host-composition)
//!
//! ## Smoke host composition
//!
//! [`boot_isolated_lab`] installs lab env vars, then boots Spectra → Valence →
//! Chronon → Boson → Photon → Higgs under a temp directory. Call it once at test
//! startup from validating tests tagged with [`scenarios::HOST_COMPOSITION_BOOT_HAPPY`]
//! (or the Photon-key sad scenario). Drop the [`IsolatedLab`] to shut Chronon
//! loops down.
//!
//! **Prerequisites:** Workspace crates built; writable temp dirs; Photon transport
//! key for the happy path (`with_photon_key = true`).
//!
//! ```rust,ignore
//! use std::path::Path;
//! use {{crate_name}}_e2e::{boot_isolated_lab, install_lab_env, IsolatedLab, scenarios};
//!
//! async fn smoke() -> anyhow::Result<()> {
//!     assert_eq!(
//!         scenarios::HOST_COMPOSITION_BOOT_HAPPY,
//!         "host-composition-boot-happy"
//!     );
//!     // Equivalent setup: install_lab_env(root, true) then compose runtimes.
//!     let lab: IsolatedLab = boot_isolated_lab(true).await?;
//!     assert!(std::sync::Arc::strong_count(&lab.higgs) >= 1);
//!     install_lab_env(Path::new("/tmp/uf-lab-demo"), true);
//!     Ok(())
//! }
//! ```
//!
//! On success you hold live runtime handles under [`IsolatedLab`]. Missing Photon
//! keys fail the sad scenario closed. Customize scenario IDs only when adding
//! Layer-2 cases; keep boot order aligned with `server::platform`.

use std::path::{Path, PathBuf};
use std::sync::Arc;

use higgs::{HiggsConfig, HiggsValenceFactory};
use valence::{
    register_backend_logical_names_slices, router_key, Actor, DatabaseBackend, DatabaseRouter,
    RegisterBackendLogicalNamesOptions, SqliteBackend, Valence, ValenceFactory, SQLITE_ENGINE_ID,
};

/// Stable scenario catalog (Layer 2).
pub mod scenarios {
    /// Validating happy — full Spectra→Valence→Chronon→Boson→Photon→Higgs boot.
    ///
    /// Tag [`IsolatedLab`] tests that expect successful composition with a Photon key.
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// use {{crate_name}}_e2e::scenarios::HOST_COMPOSITION_BOOT_HAPPY;
    /// assert_eq!(HOST_COMPOSITION_BOOT_HAPPY, "host-composition-boot-happy");
    /// ```
    pub const HOST_COMPOSITION_BOOT_HAPPY: &str = "host-composition-boot-happy";
    /// Validating sad — Photon transport key missing; composition fails closed.
    ///
    /// Tag tests that assert boot fails without `PHOTON_TRANSPORT_KEY`.
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// use {{crate_name}}_e2e::scenarios::HOST_COMPOSITION_PHOTON_KEY_SAD;
    /// assert_eq!(HOST_COMPOSITION_PHOTON_KEY_SAD, "host-composition-photon-key-sad");
    /// ```
    pub const HOST_COMPOSITION_PHOTON_KEY_SAD: &str = "host-composition-photon-key-sad";
}

/// Dev transport key (base64 of 32 bytes) used by Photon smoke / CI.
///
/// Installed by [`install_lab_env`] when `with_photon_key` is true. Do not use in
/// production; customize only for local key-format experiments.
///
/// # Examples
///
/// ```rust,ignore
/// use {{crate_name}}_e2e::DEV_PHOTON_TRANSPORT_KEY;
/// assert!(!DEV_PHOTON_TRANSPORT_KEY.is_empty());
/// ```
pub const DEV_PHOTON_TRANSPORT_KEY: &str = "cGhvdG9uLWRldi10cmFuc3BvcnQta2V5LTMyYnl0ZXM=";

/// Logical Valence groups registered on the shared `SQLite` store (no `gluon`).
fn router_groups() -> &'static [&'static [&'static str]] {
    &[
        &["default"],
        &["chronon"],
        &["boson"],
        &["photon"],
        &["permissions"],
        &["secrets"],
    ]
}

/// Process factory shared by Valence + Higgs during lab boot.
///
/// Lab-only: allows System mint for composition smoke. Do not copy this into
/// production worker bridges — use
/// `server::process_valence_factory::ProcessValenceFactory::as_external_valence_factory`
/// for untrusted actor JSON.
struct LabValenceFactory {
    router: Arc<DatabaseRouter>,
    default_backend_key: String,
}

impl ValenceFactory for LabValenceFactory {
    fn build(&self, actor_json: &serde_json::Value) -> valence::Result<Valence> {
        let actor: Actor = serde_json::from_value(actor_json.clone())
            .map_err(|e| valence::Error::Internal(format!("Failed to deserialize actor: {e}")))?;
        Valence::builder()
            .database_router(self.router.clone())
            .default_backend_key(self.default_backend_key.clone())
            .with_actor(actor)
            .build()
    }
}

impl HiggsValenceFactory for LabValenceFactory {
    fn build(&self, actor_json: &serde_json::Value) -> anyhow::Result<Valence> {
        ValenceFactory::build(self, actor_json).map_err(anyhow::Error::from)
    }
}

/// `IsolatedLab` session: temp `SQLite` roots + composed platform handles.
///
/// Dropping signals Chronon shutdown and aborts the run loop. Customize by
/// calling [`boot_isolated_lab`] with/without a Photon key; do not construct
/// manually.
///
/// # Examples
///
/// See [Smoke host composition](index.html#smoke-host-composition).
pub struct IsolatedLab {
    /// Temp directory owning all `SQLite` files (kept until this struct drops).
    #[allow(dead_code)]
    root: tempfile::TempDir,
    /// Composed Higgs config with Chronon / Boson / Photon wired.
    pub higgs: Arc<HiggsConfig>,
    /// Chronon runtime (shutdown signaled on drop).
    pub chronon: Arc<chronon_uf_embedded::ChrononEmbeddedRuntime>,
    /// Boson runtime.
    pub boson: Arc<boson_uf_embedded::BosonEmbeddedRuntime>,
    /// Photon runtime.
    pub photon: Arc<photon_uf_embedded::PhotonEmbeddedRuntime>,
    /// Spectra handle.
    pub spectra: Arc<spectra::Spectra>,
}

impl Drop for IsolatedLab {
    fn drop(&mut self) {
        self.chronon.shutdown.notify_waiters();
        self.chronon.run_loop.abort();
    }
}

/// Point all embedded `SQLite` env vars at `root` and optionally install a Photon transport key.
///
/// Called by [`boot_isolated_lab`]. Call directly when a test needs env setup
/// without a full boot. Customize paths only through the `root` argument.
///
/// # Examples
///
/// ```rust,ignore
/// use std::path::Path;
/// use {{crate_name}}_e2e::install_lab_env;
/// let root = Path::new("/tmp/uf-lab");
/// install_lab_env(root, true);
/// println!("lab env pointed at {}", root.display());
/// ```
pub fn install_lab_env(root: &Path, with_photon_key: bool) {
    std::env::set_var("VALENCE_OWNERSHIP_UNIFIED_FETCH", "0");
    std::env::set_var("VALENCE_SQLITE_PATH", root.join("{{project-name}}.sqlite3"));
    std::env::set_var(
        spectra_uf_embedded::METRICS_PATH_ENV,
        root.join("spectra-metrics.sqlite3"),
    );
    std::env::set_var(
        spectra_uf_embedded::EVENTS_PATH_ENV,
        root.join("spectra-events.sqlite3"),
    );
    std::env::set_var(
        chronon_uf_embedded::SQLITE_PATH_ENV,
        root.join("chronon.sqlite3"),
    );
    std::env::set_var(
        boson_uf_embedded::SQLITE_PATH_ENV,
        root.join("boson.sqlite3"),
    );
    std::env::set_var(
        photon_uf_embedded::SQLITE_PATH_ENV,
        root.join("photon.sqlite3"),
    );
    std::env::set_var("CHRONON_TELEMETRY", "off");
    std::env::set_var("BOSON_TELEMETRY", "off");

    if with_photon_key {
        std::env::set_var("PHOTON_TRANSPORT_KEY", DEV_PHOTON_TRANSPORT_KEY);
    } else {
        std::env::remove_var("PHOTON_TRANSPORT_KEY");
        std::env::remove_var("PHOTON_ALLOW_DEV_TRANSPORT_KEY");
    }
}

/// Open lab Valence SQLite and register the same logical groups as the host.
///
/// Internal to [`boot_isolated_lab`]. Customize group lists only if the host
/// [`server::valence_bootstrap::router_groups`] change — keep them aligned.
///
/// # Errors
///
/// Returns an error when SQLite connect or router registration fails.
///
/// # Examples
///
/// ```rust,ignore
/// use std::path::PathBuf;
/// use {{crate_name}}_e2e::boot_valence;
/// let (router, key) = boot_valence(PathBuf::from("data/lab.sqlite3")).await?;
/// assert!(!key.is_empty());
/// Ok::<(), anyhow::Error>(())
/// ```
pub(crate) async fn boot_valence(path: PathBuf) -> anyhow::Result<(Arc<DatabaseRouter>, String)> {
    let backend: Arc<dyn DatabaseBackend> = Arc::new(
        SqliteBackend::connect(path.to_string_lossy().as_ref())
            .await
            .map_err(|e| anyhow::anyhow!("Valence SQLite open failed: {e}"))?,
    );
    let mut router = DatabaseRouter::new();
    register_backend_logical_names_slices(
        &mut router,
        backend,
        router_groups(),
        RegisterBackendLogicalNamesOptions::default(),
    );
    let default_backend_key = router_key("default", SQLITE_ENGINE_ID);
    Ok((Arc::new(router), default_backend_key))
}

/// Boot Spectra → Valence → Chronon → Boson → Photon → Higgs under a temp root.
///
/// Primary entry for Layer-2 host composition smokes. Prefer this over calling
/// `server::platform::compose_embedded_platform` when tests need isolated temp DBs.
///
/// # Errors
///
/// Propagates `SQLite` / runtime / Higgs construction failures.
///
/// # Examples
///
/// See [Smoke host composition](index.html#smoke-host-composition).
pub async fn boot_isolated_lab(with_photon_key: bool) -> anyhow::Result<IsolatedLab> {
    let root = tempfile::tempdir().map_err(|e| anyhow::anyhow!("tempdir: {e}"))?;
    install_lab_env(root.path(), with_photon_key);

    let spectra = spectra_uf_embedded::install_embedded_sqlite().await?;
    let (valence_router, default_backend_key) =
        boot_valence(root.path().join("{{project-name}}.sqlite3")).await?;

    let factory = Arc::new(LabValenceFactory {
        router: valence_router,
        default_backend_key,
    });
    let valence_factory: Arc<dyn ValenceFactory> = factory.clone();
    let higgs_factory: Arc<dyn HiggsValenceFactory> = factory;

    let chronon_rt =
        chronon_uf_embedded::build_chronon_runtime(Arc::clone(&valence_factory)).await?;
    let boson_rt = boson_uf_embedded::build_boson_runtime(Arc::clone(&valence_factory)).await?;
    let photon_rt = photon_uf_embedded::build_runtime(Arc::clone(&valence_factory)).await?;

    let chronon = Arc::new(chronon_rt);
    let boson = Arc::new(boson_rt);
    let photon = Arc::new(photon_rt);

    let higgs = Arc::new(
        HiggsConfig::builder()
            .valence_factory_arc(higgs_factory)
            .chronon(
                Arc::clone(&chronon.scheduler),
                Arc::clone(&chronon.backend),
                Arc::clone(&chronon.registry),
            )
            .boson(Arc::clone(&boson.backend))
            .photon(Arc::clone(&photon.photon))
            .build()
            .map_err(|e| anyhow::anyhow!("HiggsConfig build failed: {e}"))?,
    );

    Ok(IsolatedLab {
        root,
        higgs,
        chronon,
        boson,
        photon,
        spectra,
    })
}
