//! Boson SQLite adapter for in-process {{app_title}} hosts.
//!
//! Opens a queue backend, builds Boson with a Valence execution context factory,
//! calls [`boson_runtime::configure`], and exposes a coordinator adapter on
//! [`BosonEmbeddedRuntime`]. Customize the SQLite path via env; keep the runtime
//! handle alive for the worker loop.
//!
//! ## Features
//!
//! - **Boson SQLite runtime** — Builds queue + worker settings via
//!   [`build_boson_runtime`] so tasks enqueue and run inside the host process.
//!   [Get started](#wire-boson-runtime)
//!
//! ## Wire Boson runtime
//!
//! [`build_boson_runtime`] is the boot entry for hosts that keep Boson in-process.
//! Call it once at startup after Valence is available; retain the runtime so the
//! worker loop and coordinator backend stay alive.
//!
//! **Prerequisites:** Writable SQLite parent; [`valence::ValenceFactory`]; optional
//! `BOSON_TELEMETRY=off` in tests.
//!
//! ```rust,ignore
//! use std::sync::Arc;
//! use boson_uf_embedded::{build_boson_runtime, BosonEmbeddedRuntime};
//! use valence::ValenceFactory;
//!
//! async fn boot(factory: Arc<dyn ValenceFactory>) -> anyhow::Result<()> {
//!     let runtime: BosonEmbeddedRuntime = build_boson_runtime(factory).await?;
//!     // build_boson_runtime calls boson_runtime::configure
//!     assert!(runtime.boson.worker_settings().lease_ttl_secs > 0);
//!     Ok(())
//! }
//! ```
//!
//! On success the coordinator adapter is ready for Higgs / app server functions.
//! SQLite open or builder errors return `Err`. Path override: `BOSON_SQLITE_PATH`
//! (default `data/boson.sqlite3`).
//!
//! ## Env
//!
//! | Variable | Default |
//! |----------|---------|
//! | `BOSON_SQLITE_PATH` | `data/boson.sqlite3` |

use std::path::{Path, PathBuf};
use std::sync::Arc;

use boson::SqliteQueueBackend;
use boson_coordinator::{BosonCoordinatorBackend, CoordinatorAdapter};
use boson_core::{IdempotencyMode, QueueBackend};
use boson_runtime::Boson;
use boson_valence_identity::ValenceExecutionContextFactory;
use valence::ValenceFactory;

/// Default Boson `SQLite` path for embedded hosts.
///
/// Override with [`SQLITE_PATH_ENV`] rather than editing this constant unless
/// changing the template default for a fork.
///
/// # Examples
///
/// ```rust,ignore
/// use boson_uf_embedded::DEFAULT_SQLITE_PATH;
/// assert_eq!(DEFAULT_SQLITE_PATH, "data/boson.sqlite3");
/// ```
pub const DEFAULT_SQLITE_PATH: &str = "data/boson.sqlite3";
/// Env var for Boson `SQLite` path.
///
/// # Examples
///
/// ```rust,ignore
/// use boson_uf_embedded::SQLITE_PATH_ENV;
/// assert_eq!(SQLITE_PATH_ENV, "BOSON_SQLITE_PATH");
/// ```
pub const SQLITE_PATH_ENV: &str = "BOSON_SQLITE_PATH";
/// Lease TTL (seconds) for embedded workers — enables heartbeats + expired-lease reclaim.
///
/// Customize only when you need a different reclaim window; keep > 0 so the reaper runs.
///
/// # Examples
///
/// ```rust,ignore
/// use boson_uf_embedded::DEFAULT_EMBEDDED_LEASE_TTL_SECS;
/// assert!(DEFAULT_EMBEDDED_LEASE_TTL_SECS > 0);
/// ```
pub const DEFAULT_EMBEDDED_LEASE_TTL_SECS: i64 = 30;

/// Resolve Boson SQLite path from env or [`DEFAULT_SQLITE_PATH`].
///
/// # Examples
///
/// ```rust,ignore
/// use boson_uf_embedded::path_from_env;
/// println!("{}", path_from_env().display());
/// ```
pub(crate) fn path_from_env() -> PathBuf {
    std::env::var(SQLITE_PATH_ENV)
        .ok()
        .filter(|s| !s.trim().is_empty())
        .map_or_else(|| PathBuf::from(DEFAULT_SQLITE_PATH), PathBuf::from)
}

/// Ensure the parent directory for Boson SQLite exists.
///
/// # Errors
///
/// Returns an error when `create_dir_all` fails.
///
/// # Examples
///
/// ```rust,ignore
/// use std::path::Path;
/// use boson_uf_embedded::ensure_parent;
/// ensure_parent(Path::new("data/boson.sqlite3"))?;
/// Ok::<(), anyhow::Error>(())
/// ```
pub(crate) fn ensure_parent(path: &Path) -> anyhow::Result<()> {
    if let Some(parent) = path.parent() {
        if !parent.as_os_str().is_empty() {
            std::fs::create_dir_all(parent)?;
        }
    }
    Ok(())
}

/// Best-effort `0600` on the SQLite file and WAL/SHM sidecars when present.
pub(crate) fn restrict_sqlite_file_permissions(path: &Path) -> anyhow::Result<()> {
    use std::os::unix::fs::PermissionsExt;
    let meta = std::fs::metadata(path)?;
    let mut perms = meta.permissions();
    perms.set_mode(0o600);
    std::fs::set_permissions(path, perms.clone())?;
    for suffix in ["-wal", "-shm"] {
        let mut sidecar = std::ffi::OsString::from(path.as_os_str());
        sidecar.push(suffix);
        let sidecar = Path::new(&sidecar);
        if sidecar.exists() {
            let _ = std::fs::set_permissions(sidecar, perms.clone());
        }
    }
    Ok(())
}

/// Boson runtime assembled for an embedded host process.
///
/// Keep alive so the worker loop and coordinator adapter remain available to Higgs.
///
/// # Examples
///
/// See [Wire Boson runtime](index.html#wire-boson-runtime).
pub struct BosonEmbeddedRuntime {
    /// Engine handle (worker loop spawned by builder by default).
    pub boson: Arc<Boson>,
    /// Product coordinator adapter for Higgs / app server functions.
    pub backend: Arc<dyn BosonCoordinatorBackend>,
}

/// Build Boson with `SQLite` queue and coordinator adapter.
///
/// Calls [`boson_runtime::configure`]. Call once at boot after Valence is ready.
///
/// # Errors
///
/// Returns an error if `SQLite` open or Boson builder construction fails.
///
/// # Examples
///
/// See [Wire Boson runtime](index.html#wire-boson-runtime).
pub async fn build_boson_runtime(
    valence_factory: Arc<dyn ValenceFactory>,
) -> anyhow::Result<BosonEmbeddedRuntime> {
    // Path: BOSON_SQLITE_PATH. lease_ttl_secs > 0 keeps heartbeats + reclaim live.
    // Retain the runtime so configure() + worker loop stay process-scoped.
    boson_spectra_telemetry::install_ops_log_from_env();

    let path = path_from_env();
    ensure_parent(&path)?;
    let queue: Arc<dyn QueueBackend> = Arc::new(
        SqliteQueueBackend::new(&path)
            .await
            .map_err(|e| anyhow::anyhow!("Boson SQLite open failed: {e}"))?,
    );
    restrict_sqlite_file_permissions(&path)
        .map_err(|e| anyhow::anyhow!("Boson SQLite chmod 0600 failed: {e}"))?;

    let boson = Boson::builder()
        .queue_backend(queue)
        .execution_context_factory(ValenceExecutionContextFactory::new(valence_factory))
        .auto_registry()
        .idempotency_mode(IdempotencyMode::Lwt)
        .lease_ttl_secs(DEFAULT_EMBEDDED_LEASE_TTL_SECS)
        .build()
        .map_err(|e| anyhow::anyhow!("Boson build failed: {e}"))?;

    let boson = Arc::new(boson);
    boson_runtime::configure((*boson).clone());

    let backend: Arc<dyn BosonCoordinatorBackend> =
        Arc::new(CoordinatorAdapter::new(Arc::clone(&boson)));

    if let Err(e) =
        boson_coordinator::ensure_default_task_configs_embedded(Arc::clone(&backend)).await
    {
        log::warn!("[boson-uf-embedded] ensure default task configs: {e}");
    }

    Ok(BosonEmbeddedRuntime { boson, backend })
}

#[cfg(test)]
mod tests {
    use super::*;
    use valence::{Error, Result, Valence};

    static ENV_LOCK: tokio::sync::Mutex<()> = tokio::sync::Mutex::const_new(());

    struct StubValenceFactory;

    impl ValenceFactory for StubValenceFactory {
        fn build(&self, _actor_json: &serde_json::Value) -> Result<Valence> {
            Err(Error::Internal("stub valence factory".into()))
        }
    }

    fn stub_factory() -> Arc<dyn ValenceFactory> {
        Arc::new(StubValenceFactory)
    }

    #[tokio::test]
    async fn build_boson_runtime_happy_path() {
        let _guard = ENV_LOCK.lock().await;
        std::env::set_var("BOSON_TELEMETRY", "off");
        let dir = tempfile::tempdir().expect("tempdir");
        std::env::set_var(SQLITE_PATH_ENV, dir.path().join("boson.sqlite3"));

        let runtime = build_boson_runtime(stub_factory())
            .await
            .expect("build_boson_runtime");
        assert_eq!(
            runtime.boson.worker_settings().lease_ttl_secs,
            DEFAULT_EMBEDDED_LEASE_TTL_SECS
        );
        assert!(
            runtime.boson.worker_settings().lease_ttl_secs > 0,
            "embedded lease TTL must enable reaper"
        );
        let _ = runtime.backend.as_ref();
    }

    #[tokio::test]
    async fn build_boson_runtime_parent_not_dir_sad() {
        let _guard = ENV_LOCK.lock().await;
        std::env::set_var("BOSON_TELEMETRY", "off");
        let dir = tempfile::tempdir().expect("tempdir");
        let blocker = dir.path().join("not-a-dir");
        std::fs::write(&blocker, b"x").expect("write blocker");
        std::env::set_var(SQLITE_PATH_ENV, blocker.join("boson.sqlite3"));

        let Err(err) = build_boson_runtime(stub_factory()).await else {
            panic!("parent is a file");
        };
        let msg = err.to_string();
        assert!(
            msg.contains("Not a directory")
                || msg.contains("not a directory")
                || msg.contains("File exists")
                || msg.contains("os error"),
            "unexpected error: {msg}"
        );
    }
}
