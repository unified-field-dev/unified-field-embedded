//! Chronon SQLite adapter for in-process {{app_title}} hosts.
//!
//! Opens a scheduler store, wraps the coordinator backend, registers default embedded
//! jobs, and keeps tick/worker loops alive on [`ChrononEmbeddedRuntime`].
//! Customize the SQLite path via env; keep the returned handle for the process lifetime.
//!
//! ## Features
//!
//! - **Chronon SQLite runtime** — Builds store + coordinator + run loop via
//!   [`build_chronon_runtime`] so scripts and jobs run inside the host process.
//!   [Get started](#wire-chronon-runtime)
//!
//! ## Wire Chronon runtime
//!
//! [`build_chronon_runtime`] starts Chronon for hosts that schedule jobs locally.
//! Call it once at process boot after Valence is ready; keep the returned handle
//! (and its `shutdown` / `run_loop`) for the process lifetime.
//!
//! **Prerequisites:** Writable SQLite parent path; [`valence::ValenceFactory`]; optional
//! `CHRONON_TELEMETRY=off` in tests.
//!
//! ```rust,ignore
//! use std::sync::Arc;
//! use chronon_uf_embedded::{build_chronon_runtime, ChrononEmbeddedRuntime};
//! use valence::ValenceFactory;
//!
//! async fn boot(factory: Arc<dyn ValenceFactory>) -> anyhow::Result<()> {
//!     let runtime: ChrononEmbeddedRuntime = build_chronon_runtime(factory).await?;
//!     assert!(Arc::strong_count(&runtime.backend) >= 1);
//!     runtime.shutdown.notify_waiters();
//!     runtime.run_loop.abort();
//!     Ok(())
//! }
//! ```
//!
//! On success default embedded jobs are registered and loops are running. SQLite or
//! builder failures return `Err`. Path override: `CHRONON_SQLITE_PATH` (default
//! `data/chronon.sqlite3`).
//!
//! ## Env
//!
//! | Variable | Default |
//! |----------|---------|
//! | `CHRONON_SQLITE_PATH` | `data/chronon.sqlite3` |

use std::path::{Path, PathBuf};
use std::sync::Arc;

use async_trait::async_trait;
use chronon::SqliteSchedulerStore;
use chronon_coordinator::{
    snapshot_actor_json, validate_external_job_actor_json, ChrononCoordinatorBackend, ChrononError,
    Job, JobRevision, Result as ChrononResult, Run, Scheduler, ScriptRegistry,
};
use chronon_core::store::SchedulerStore;
use chronon_runtime::{ChrononBuilder, CoordinatorService};
use chronon_valence_identity::ValenceScriptContextFactory;
use tokio::sync::Notify;
use tokio::task::JoinHandle;
use valence::{Valence, ValenceFactory};

/// Default Chronon `SQLite` path for embedded hosts.
///
/// Override with [`SQLITE_PATH_ENV`] rather than editing this constant unless
/// changing the template default for a fork.
///
/// # Examples
///
/// ```rust,ignore
/// use chronon_uf_embedded::DEFAULT_SQLITE_PATH;
/// assert_eq!(DEFAULT_SQLITE_PATH, "data/chronon.sqlite3");
/// ```
pub const DEFAULT_SQLITE_PATH: &str = "data/chronon.sqlite3";
/// Env var for Chronon `SQLite` path.
///
/// Set before [`build_chronon_runtime`] when the default path is wrong for your deploy.
///
/// # Examples
///
/// ```rust,ignore
/// use chronon_uf_embedded::SQLITE_PATH_ENV;
/// assert_eq!(SQLITE_PATH_ENV, "CHRONON_SQLITE_PATH");
/// ```
pub const SQLITE_PATH_ENV: &str = "CHRONON_SQLITE_PATH";

/// Resolve Chronon SQLite path from env or [`DEFAULT_SQLITE_PATH`].
///
/// # Examples
///
/// ```rust,ignore
/// use chronon_uf_embedded::path_from_env;
/// println!("{}", path_from_env().display());
/// ```
pub(crate) fn path_from_env() -> PathBuf {
    std::env::var(SQLITE_PATH_ENV)
        .ok()
        .filter(|s| !s.trim().is_empty())
        .map_or_else(|| PathBuf::from(DEFAULT_SQLITE_PATH), PathBuf::from)
}

/// Ensure the parent directory for Chronon SQLite exists.
///
/// # Errors
///
/// Returns an error when `create_dir_all` fails.
///
/// # Examples
///
/// ```rust,ignore
/// use std::path::Path;
/// use chronon_uf_embedded::ensure_parent;
/// ensure_parent(Path::new("data/chronon.sqlite3"))?;
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

fn map_err(err: impl std::fmt::Display) -> ChrononError {
    ChrononError::Internal(err.to_string())
}

struct UpstreamCoordinatorAdapter {
    coordinator: Arc<CoordinatorService>,
}

#[async_trait]
impl ChrononCoordinatorBackend for UpstreamCoordinatorAdapter {
    async fn load_jobs_from_db(&self) -> ChrononResult<()> {
        Ok(())
    }

    async fn upsert_job(&self, job: Job) -> ChrononResult<()> {
        // Untrusted / no-Valence path: reject System-shaped actor JSON (boot uses
        // upsert_job_with_valence with a System Valence snapshot instead).
        validate_external_job_actor_json(&job.actor_json)?;
        self.coordinator.upsert_job(job).await.map_err(map_err)
    }

    async fn upsert_job_with_valence(&self, valence: &Valence, mut job: Job) -> ChrononResult<()> {
        // Preserve System on platform/default jobs so an admin cron edit does not
        // rebind the job to the editing user.
        if let Some(existing) = self.get_job(&job.job_id).await {
            if validate_external_job_actor_json(&existing.actor_json).is_err() {
                job.actor_json = existing.actor_json;
                return self.coordinator.upsert_job(job).await.map_err(map_err);
            }
        }
        job.actor_json = snapshot_actor_json(valence)?;
        // Trusted snapshot may be System (boot default jobs); skip external validate.
        self.coordinator.upsert_job(job).await.map_err(map_err)
    }

    async fn get_job(&self, job_id: &str) -> Option<Job> {
        self.coordinator.get_job(job_id).await
    }

    async fn get_job_by_name(&self, job_name: &str) -> Option<Job> {
        self.coordinator.get_job_by_name(job_name).await
    }

    async fn list_jobs(&self) -> Vec<Job> {
        // Trait returns `Vec<Job>` (not `Result`); log failures instead of silent empty.
        match self.coordinator.list_jobs().await {
            Ok(jobs) => jobs,
            Err(e) => {
                log::warn!("[chronon-uf-embedded] list_jobs failed; returning empty: {e}");
                Vec::new()
            }
        }
    }

    async fn list_runs(
        &self,
        job_id: Option<&str>,
        status: Option<&str>,
        offset: usize,
        limit: usize,
    ) -> ChrononResult<Vec<Run>> {
        self.coordinator
            .list_runs(job_id, status, offset, limit)
            .await
            .map_err(map_err)
    }

    async fn get_run(&self, run_id: &str) -> ChrononResult<Option<Run>> {
        self.coordinator.get_run(run_id).await.map_err(map_err)
    }

    async fn pause_job(&self, job_id: &str) -> ChrononResult<()> {
        self.coordinator.pause_job(job_id).await.map_err(map_err)
    }

    async fn resume_job(&self, job_id: &str) -> ChrononResult<()> {
        self.coordinator.resume_job(job_id).await.map_err(map_err)
    }

    async fn list_revisions(&self, job_id_or_name: &str) -> ChrononResult<Vec<JobRevision>> {
        let job_id = match self.coordinator.get_job(job_id_or_name).await {
            Some(job) => job.job_id,
            None => match self.coordinator.get_job_by_name(job_id_or_name).await {
                Some(job) => job.job_id,
                None => job_id_or_name.to_string(),
            },
        };
        self.coordinator
            .list_revisions(&job_id)
            .await
            .map_err(map_err)
    }

    async fn update_job_config(&self, _job_id: &str, updated: Job) -> ChrononResult<()> {
        // Trusted path (may carry System actor from platform jobs).
        self.coordinator.upsert_job(updated).await.map_err(map_err)
    }

    async fn update_job_config_with_valence(
        &self,
        valence: &Valence,
        _job_id: &str,
        updated: Job,
    ) -> ChrononResult<()> {
        self.upsert_job_with_valence(valence, updated).await
    }

    async fn run_now(&self, job_id: &str) -> ChrononResult<String> {
        self.coordinator.run_now(job_id).await.map_err(map_err)
    }

    async fn run_now_with_params(
        &self,
        job_id: &str,
        params_override: Option<serde_json::Value>,
    ) -> ChrononResult<String> {
        self.coordinator
            .run_now_with_params(job_id, params_override)
            .await
            .map_err(map_err)
    }
}

/// Chronon runtime assembled for an embedded host process.
///
/// Keep alive (and signal `shutdown` on exit) so tick/worker loops stay coherent.
/// Customize via [`build_chronon_runtime`] env/factory, not by swapping fields later.
///
/// # Examples
///
/// See [Wire Chronon runtime](index.html#wire-chronon-runtime).
pub struct ChrononEmbeddedRuntime {
    /// Product coordinator adapter for Higgs / app server functions.
    pub backend: Arc<dyn ChrononCoordinatorBackend>,
    /// Script registry for Higgs / UI discovery.
    pub registry: Arc<ScriptRegistry>,
    /// Thin scheduler handle wrapping the registry.
    pub scheduler: Arc<Scheduler>,
    /// Background embedded tick + worker loops.
    pub run_loop: JoinHandle<chronon_core::Result<()>>,
    /// Shutdown signal for the run loop.
    pub shutdown: Arc<Notify>,
}

/// Build Chronon with `SQLite` store, start loops, register default jobs.
///
/// Call once at process boot after Valence is ready.
///
/// # Errors
///
/// Returns an error if `SQLite` open or Chronon builder construction fails.
///
/// # Examples
///
/// See [Wire Chronon runtime](index.html#wire-chronon-runtime).
pub async fn build_chronon_runtime(
    valence_factory: Arc<dyn ValenceFactory>,
) -> anyhow::Result<ChrononEmbeddedRuntime> {
    // Path: CHRONON_SQLITE_PATH. Keep ChrononEmbeddedRuntime for process lifetime
    // so tick/worker loops stay alive; call shutdown/abort only in tests/labs.
    let path = path_from_env();
    ensure_parent(&path)?;
    let store: Arc<dyn SchedulerStore> = Arc::new(
        SqliteSchedulerStore::new(&path)
            .await
            .map_err(|e| anyhow::anyhow!("Chronon SQLite open failed: {e}"))?,
    );
    restrict_sqlite_file_permissions(&path)
        .map_err(|e| anyhow::anyhow!("Chronon SQLite chmod 0600 failed: {e}"))?;

    let context_factory = Arc::new(ValenceScriptContextFactory::new(Arc::clone(
        &valence_factory,
    )));
    let telemetry = chronon_spectra_telemetry::install_from_env();
    let mut chronon = ChrononBuilder::new()
        .scheduler_store(Arc::clone(&store))
        .context_factory(context_factory)
        .telemetry_sink(telemetry)
        .auto_registry()
        .embedded()
        .build()
        .map_err(|e| anyhow::anyhow!("Chronon build failed: {e}"))?;

    let coordinator = Arc::new(CoordinatorService::new(Arc::clone(&chronon.store)));
    let upstream_registry = Arc::clone(&chronon.executor().registry);
    let backend: Arc<dyn ChrononCoordinatorBackend> = Arc::new(UpstreamCoordinatorAdapter {
        coordinator: Arc::clone(&coordinator),
    });
    let scheduler = Arc::new(Scheduler::from_registry(Arc::clone(&upstream_registry)));
    let registry = scheduler.registry_arc();

    chronon_coordinator::register_default_jobs_embedded(
        Arc::clone(&backend),
        Arc::clone(&valence_factory),
    )
    .await;

    let shutdown = chronon.shutdown_handle();
    let run_loop = tokio::spawn(async move { chronon.run().await });

    Ok(ChrononEmbeddedRuntime {
        backend,
        registry,
        scheduler,
        run_loop,
        shutdown,
    })
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
    async fn build_chronon_runtime_happy_path() {
        let _guard = ENV_LOCK.lock().await;
        std::env::set_var("CHRONON_TELEMETRY", "off");
        let dir = tempfile::tempdir().expect("tempdir");
        std::env::set_var(SQLITE_PATH_ENV, dir.path().join("chronon.sqlite3"));

        let runtime = build_chronon_runtime(stub_factory())
            .await
            .expect("build_chronon_runtime");
        let _ = runtime.backend.as_ref();
        let _ = runtime.registry.as_ref();
        // Signal shutdown; do not await the join handle (embedded run may not exit promptly).
        runtime.shutdown.notify_waiters();
        runtime.run_loop.abort();
    }

    #[tokio::test]
    async fn build_chronon_runtime_parent_not_dir_sad() {
        let _guard = ENV_LOCK.lock().await;
        std::env::set_var("CHRONON_TELEMETRY", "off");
        let dir = tempfile::tempdir().expect("tempdir");
        let blocker = dir.path().join("not-a-dir");
        std::fs::write(&blocker, b"x").expect("write blocker");
        std::env::set_var(SQLITE_PATH_ENV, blocker.join("chronon.sqlite3"));

        let Err(err) = build_chronon_runtime(stub_factory()).await else {
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
