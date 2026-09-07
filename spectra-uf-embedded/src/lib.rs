//! Spectra SQLite adapters for in-process {{app_title}} hosts.
//!
//! Opens durable metrics and events backends with sqlx (shared `libsqlite3-sys` with
//! other host SQLite crates) and returns a configured [`spectra::Spectra`] handle.
//! Customize paths via env; call [`install_embedded_sqlite`] first in boot order.
//!
//! ## Features
//!
//! - **Spectra SQLite backends** — Installs metrics + events stores via
//!   [`install_embedded_sqlite`] so the host records counters and events locally.
//!   [Get started](#install-spectra-sqlite)
//!
//! ## Install Spectra SQLite
//!
//! [`install_embedded_sqlite`] builds Spectra with embedded SQLite backends. Call it
//! first in the host boot order (before Valence telemetry install and other runtimes)
//! so metrics and events have a place to land.
//!
//! **Prerequisites:** Writable parents for metrics and events paths; sqlx SQLite
//! available in the dependency graph.
//!
//! ```rust,ignore
//! use std::sync::Arc;
//! use spectra::Spectra;
//! use spectra_uf_embedded::install_embedded_sqlite;
//!
//! async fn boot() -> anyhow::Result<Arc<Spectra>> {
//!     let spectra = install_embedded_sqlite().await?;
//!     assert!(Arc::strong_count(&spectra) >= 1);
//!     println!("spectra sqlite backends installed");
//!     Ok(spectra)
//! }
//! ```
//!
//! On success you hold an `Arc<Spectra>` ready for Higgs and kit telemetry. Directory
//! or SQLite open failures return `Err`. Override paths with the env table below.
//!
//! ## Env
//!
//! | Variable | Default |
//! |----------|---------|
//! | `SPECTRA_METRICS_SQLITE_PATH` | `data/spectra-metrics.sqlite3` |
//! | `SPECTRA_EVENTS_SQLITE_PATH` | `data/spectra-events.sqlite3` |

mod sqlx_store;

use std::path::{Path, PathBuf};
use std::sync::Arc;

use spectra::Spectra;

pub use sqlx_store::{SqlxEventsBackend, SqlxMetricsBackend};

/// Default metrics `SQLite` path for embedded hosts.
///
/// # Examples
///
/// ```rust,ignore
/// use spectra_uf_embedded::DEFAULT_METRICS_PATH;
/// assert_eq!(DEFAULT_METRICS_PATH, "data/spectra-metrics.sqlite3");
/// ```
pub const DEFAULT_METRICS_PATH: &str = "data/spectra-metrics.sqlite3";
/// Default events `SQLite` path for embedded hosts.
///
/// # Examples
///
/// ```rust,ignore
/// use spectra_uf_embedded::DEFAULT_EVENTS_PATH;
/// assert_eq!(DEFAULT_EVENTS_PATH, "data/spectra-events.sqlite3");
/// ```
pub const DEFAULT_EVENTS_PATH: &str = "data/spectra-events.sqlite3";

/// Env var for metrics `SQLite` path.
///
/// # Examples
///
/// ```rust,ignore
/// use spectra_uf_embedded::METRICS_PATH_ENV;
/// assert_eq!(METRICS_PATH_ENV, "SPECTRA_METRICS_SQLITE_PATH");
/// ```
pub const METRICS_PATH_ENV: &str = "SPECTRA_METRICS_SQLITE_PATH";
/// Env var for events `SQLite` path.
///
/// # Examples
///
/// ```rust,ignore
/// use spectra_uf_embedded::EVENTS_PATH_ENV;
/// assert_eq!(EVENTS_PATH_ENV, "SPECTRA_EVENTS_SQLITE_PATH");
/// ```
pub const EVENTS_PATH_ENV: &str = "SPECTRA_EVENTS_SQLITE_PATH";

/// Resolve a Spectra path from env or the given default.
///
/// # Examples
///
/// ```rust,ignore
/// use spectra_uf_embedded::{path_from_env, METRICS_PATH_ENV, DEFAULT_METRICS_PATH};
/// println!("{}", path_from_env(METRICS_PATH_ENV, DEFAULT_METRICS_PATH).display());
/// ```
pub(crate) fn path_from_env(var: &str, default: &str) -> PathBuf {
    std::env::var(var)
        .ok()
        .filter(|s| !s.trim().is_empty())
        .map_or_else(|| PathBuf::from(default), PathBuf::from)
}

/// Ensure the parent directory for a Spectra SQLite path exists.
///
/// # Errors
///
/// Returns an error when `create_dir_all` fails.
///
/// # Examples
///
/// ```rust,ignore
/// use std::path::Path;
/// use spectra_uf_embedded::ensure_parent;
/// ensure_parent(Path::new("data/spectra-metrics.sqlite3"))?;
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

/// Install Spectra with durable `SQLite` backends for embedded hosts.
///
/// Call first in host boot order before Valence telemetry and other runtimes.
///
/// # Errors
///
/// Returns an error if directories cannot be created, `SQLite` open fails, or Spectra build fails.
///
/// # Examples
///
/// See [Install Spectra SQLite](index.html#install-spectra-sqlite).
pub async fn install_embedded_sqlite() -> anyhow::Result<Arc<Spectra>> {
    // Call first in host boot. Paths: SPECTRA_METRICS_SQLITE_PATH /
    // SPECTRA_EVENTS_SQLITE_PATH (see Env table on the crate root).
    let metrics_path = path_from_env(METRICS_PATH_ENV, DEFAULT_METRICS_PATH);
    let events_path = path_from_env(EVENTS_PATH_ENV, DEFAULT_EVENTS_PATH);
    ensure_parent(&metrics_path)?;
    ensure_parent(&events_path)?;

    let metrics = SqlxMetricsBackend::open(&metrics_path)
        .await
        .map_err(|e| anyhow::anyhow!("Spectra metrics SQLite open failed: {e}"))?;
    let events = SqlxEventsBackend::open(&events_path)
        .await
        .map_err(|e| anyhow::anyhow!("Spectra events SQLite open failed: {e}"))?;
    restrict_sqlite_file_permissions(&metrics_path)
        .map_err(|e| anyhow::anyhow!("Spectra metrics SQLite chmod 0600 failed: {e}"))?;
    restrict_sqlite_file_permissions(&events_path)
        .map_err(|e| anyhow::anyhow!("Spectra events SQLite chmod 0600 failed: {e}"))?;

    let spectra = Spectra::builder()
        .metrics_backend(Arc::new(metrics))
        .events_backend(Arc::new(events))
        .embedded()
        .build()
        .map_err(|e| anyhow::anyhow!("Spectra SQLite build failed: {e}"))?;

    Ok(Arc::new(spectra))
}

#[cfg(test)]
mod tests {
    use super::*;

    static ENV_LOCK: tokio::sync::Mutex<()> = tokio::sync::Mutex::const_new(());

    #[tokio::test]
    async fn install_embedded_sqlite_happy_path() {
        let _guard = ENV_LOCK.lock().await;
        let dir = tempfile::tempdir().expect("tempdir");
        std::env::set_var(METRICS_PATH_ENV, dir.path().join("m.sqlite3"));
        std::env::set_var(EVENTS_PATH_ENV, dir.path().join("e.sqlite3"));
        let spectra = install_embedded_sqlite()
            .await
            .expect("install_embedded_sqlite");
        let _ = spectra.as_ref();
    }

    #[tokio::test]
    async fn install_embedded_sqlite_parent_not_dir_sad() {
        let _guard = ENV_LOCK.lock().await;
        let dir = tempfile::tempdir().expect("tempdir");
        let blocker = dir.path().join("not-a-dir");
        std::fs::write(&blocker, b"x").expect("write blocker");
        std::env::set_var(METRICS_PATH_ENV, blocker.join("m.sqlite3"));
        std::env::set_var(EVENTS_PATH_ENV, dir.path().join("e.sqlite3"));

        let Err(err) = install_embedded_sqlite().await else {
            panic!("metrics parent is a file");
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
