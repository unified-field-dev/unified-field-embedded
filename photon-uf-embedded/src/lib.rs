//! Photon SQLite adapter for in-process {{app_title}} hosts.
//!
//! Opens durable Photon storage, builds a [`PhotonEmbeddedRuntime`], and calls
//! [`photon::configure`] so publish paths and macros resolve in the same process.
//! Customize the SQLite path via env; keep configure-on-boot so macros do not see
//! "Photon not configured".
//!
//! ## Features
//!
//! - **Photon SQLite runtime** — Builds storage from [`SQLITE_PATH_ENV`], constructs
//!   Photon with `auto_registry`, and installs the process-global handle via
//!   [`build_runtime`]. [Get started](#wire-photon-runtime)
//!
//! ## Wire Photon runtime
//!
//! [`build_runtime`] is the boot entry for hosts that keep Photon in-process with
//! SQLite. Pass a [`valence::ValenceFactory`] once at worker or HTTP process startup
//! after Valence is up and before handlers publish events.
//!
//! **Prerequisites:** Writable parent for the SQLite file; `PHOTON_TRANSPORT_KEY`
//! when Photon crypto requires it; a live [`valence::ValenceFactory`].
//!
//! ```rust,ignore
//! use std::sync::Arc;
//! use photon_uf_embedded::{build_runtime, PhotonEmbeddedRuntime};
//! use valence::ValenceFactory;
//!
//! async fn boot(factory: Arc<dyn ValenceFactory>) -> anyhow::Result<()> {
//!     let runtime: PhotonEmbeddedRuntime = build_runtime(factory).await?;
//!     // build_runtime calls photon::configure(runtime.photon.clone())
//!     println!("photon configured: {}", runtime.photon.as_ref() as *const _ as usize != 0);
//!     Ok(())
//! }
//! ```
//!
//! On success macros and publish paths see the configured Photon client. SQLite open
//! or builder errors return `Err`. Override the path with `PHOTON_SQLITE_PATH`
//! (default `data/photon.sqlite3`).
//!
//! ## Env
//!
//! | Variable | Default |
//! |----------|---------|
//! | `PHOTON_SQLITE_PATH` | `data/photon.sqlite3` |

use std::path::{Path, PathBuf};
use std::sync::Arc;

use photon::{Photon, SqliteStoragePort};
use valence::ValenceFactory;

/// Default Photon `SQLite` path for embedded hosts.
///
/// Override with [`SQLITE_PATH_ENV`] instead of editing this constant unless you
/// are changing the template default for a fork.
///
/// # Examples
///
/// ```rust,ignore
/// use photon_uf_embedded::DEFAULT_SQLITE_PATH;
/// assert_eq!(DEFAULT_SQLITE_PATH, "data/photon.sqlite3");
/// ```
pub const DEFAULT_SQLITE_PATH: &str = "data/photon.sqlite3";
/// Env var for Photon `SQLite` path (also read by [`SqliteStoragePort::from_env`]).
///
/// Set before [`build_runtime`] when the default path is wrong for your deploy.
///
/// # Examples
///
/// ```rust,ignore
/// use photon_uf_embedded::SQLITE_PATH_ENV;
/// assert_eq!(SQLITE_PATH_ENV, "PHOTON_SQLITE_PATH");
/// ```
pub const SQLITE_PATH_ENV: &str = "PHOTON_SQLITE_PATH";

/// Resolve Photon SQLite path from [`SQLITE_PATH_ENV`] or [`DEFAULT_SQLITE_PATH`].
///
/// # Examples
///
/// ```rust,ignore
/// use photon_uf_embedded::path_from_env;
/// println!("photon sqlite path: {}", path_from_env().display());
/// ```
pub(crate) fn path_from_env() -> PathBuf {
    std::env::var(SQLITE_PATH_ENV)
        .ok()
        .filter(|s| !s.trim().is_empty())
        .map_or_else(|| PathBuf::from(DEFAULT_SQLITE_PATH), PathBuf::from)
}

/// Ensure the parent directory for Photon SQLite exists.
///
/// # Errors
///
/// Returns an error when `create_dir_all` fails.
///
/// # Examples
///
/// ```rust,ignore
/// use std::path::Path;
/// use photon_uf_embedded::ensure_parent;
/// ensure_parent(Path::new("data/photon.sqlite3"))?;
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

/// Runtime handle for embedded Photon hosts.
///
/// Keep alive for the process lifetime so storage and the configured client stay open.
///
/// # Examples
///
/// See [Wire Photon runtime](index.html#wire-photon-runtime).
pub struct PhotonEmbeddedRuntime {
    /// Factory used by host code to open Valence handles.
    pub valence_factory: Arc<dyn ValenceFactory>,
    /// Constructed Photon client (`SQLite` storage).
    pub photon: Arc<Photon>,
}

/// Build Photon with `SQLite` storage and install the process-global handle.
///
/// Calls [`photon::configure`] so macros / publish paths resolve without
/// "Photon not configured". Call once at boot after Valence is ready.
///
/// # Errors
///
/// Returns an error if `SQLite` open or Photon builder construction fails.
///
/// # Examples
///
/// See [Wire Photon runtime](index.html#wire-photon-runtime).
pub async fn build_runtime(
    valence_factory: Arc<dyn ValenceFactory>,
) -> anyhow::Result<PhotonEmbeddedRuntime> {
    // Path: PHOTON_SQLITE_PATH (default data/photon.sqlite3). Keep the returned
    // runtime alive; configure() installs the process-global Photon handle.
    let path = path_from_env();
    ensure_parent(&path)?;
    let path_str = path.to_string_lossy();
    let storage = SqliteStoragePort::open(path_str.as_ref())
        .await
        .map_err(|e| anyhow::anyhow!("Photon SQLite open failed: {e}"))?;
    restrict_sqlite_file_permissions(&path)
        .map_err(|e| anyhow::anyhow!("Photon SQLite chmod 0600 failed: {e}"))?;

    let photon = Photon::builder()
        .storage_port(Arc::new(storage))
        .auto_registry()
        .build()
        .map_err(|e| anyhow::anyhow!("Photon build failed: {e}"))?;

    photon::configure(photon.clone());
    Ok(PhotonEmbeddedRuntime {
        valence_factory,
        photon: Arc::new(photon),
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
    async fn build_runtime_happy_path() {
        let _guard = ENV_LOCK.lock().await;
        std::env::set_var(
            "PHOTON_TRANSPORT_KEY",
            "cGhvdG9uLWRldi10cmFuc3BvcnQta2V5LTMyYnl0ZXM=",
        );
        let dir = tempfile::tempdir().expect("tempdir");
        std::env::set_var(SQLITE_PATH_ENV, dir.path().join("photon.sqlite3"));

        let factory = stub_factory();
        let runtime = build_runtime(Arc::clone(&factory))
            .await
            .expect("build_runtime");
        assert!(Arc::ptr_eq(&runtime.valence_factory, &factory));
        let _photon: &Photon = runtime.photon.as_ref();
    }

    #[tokio::test]
    async fn build_runtime_parent_not_dir_sad() {
        let _guard = ENV_LOCK.lock().await;
        let dir = tempfile::tempdir().expect("tempdir");
        let blocker = dir.path().join("not-a-dir");
        std::fs::write(&blocker, b"x").expect("write blocker");
        std::env::set_var(SQLITE_PATH_ENV, blocker.join("photon.sqlite3"));

        let Err(err) = build_runtime(stub_factory()).await else {
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
