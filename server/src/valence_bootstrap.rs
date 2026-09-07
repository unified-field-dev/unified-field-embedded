//! Valence bootstrap for the embedded host: SQLite (persistent file).
//!
//! Opens the process database and registers logical backend groups used by auth
//! and subsystem factories. This repo is embedded-only — the remote Hybrid
//! IndraDB + Postgres topology lives in `unified-field-cloud`, not here.
//! Customize `VALENCE_SQLITE_PATH` or [`router_groups`] when you add stores;
//! leave Hybrid wiring to the cloud host.

use std::path::PathBuf;
use std::sync::Arc;

use anyhow::Context;
use valence::{
    register_backend_logical_names_slices, router_key, DatabaseBackend, DatabaseRouter,
    RegisterBackendLogicalNamesOptions, SqliteBackend, SQLITE_ENGINE_ID,
};

/// Logical names registered on the shared store (no `gluon`).
///
/// Each inner slice is a logical namespace sharing the SQLite backend. Edit when
/// you add a new subsystem that needs its own Valence logical name; keep
/// `default` first so auth and factories resolve the stock backend key.
///
/// # Examples
///
/// ```rust,ignore
/// use server::valence_bootstrap::router_groups;
///
/// let groups = router_groups();
/// assert!(groups.iter().any(|g| g.contains(&"default")));
/// assert!(groups.iter().any(|g| g.contains(&"secrets")));
/// ```
pub fn router_groups() -> &'static [&'static [&'static str]] {
    &[
        &["default"],
        &["chronon"],
        &["boson"],
        &["photon"],
        &["permissions"],
        &["secrets"],
    ]
}

/// Default file path for embedded SQLite when `VALENCE_SQLITE_PATH` is unset.
///
/// Point `VALENCE_SQLITE_PATH` at another file (or `:memory:`) instead of editing
/// this constant unless you are changing the template default for forks.
///
/// # Examples
///
/// ```rust,ignore
/// use server::valence_bootstrap::DEFAULT_SQLITE_PATH;
/// assert_eq!(DEFAULT_SQLITE_PATH, "data/{{project-name}}.sqlite3");
/// ```
pub const DEFAULT_SQLITE_PATH: &str = "data/{{project-name}}.sqlite3";

/// Result of host Valence bootstrap.
///
/// Holds the process [`DatabaseRouter`] and default backend key. Pass both into
/// [`crate::process_valence_factory::ProcessValenceFactory`] at boot. Customize
/// by changing path/env before calling [`db_and_router`], not by mutating fields
/// after the fact.
///
/// # Examples
///
/// ```rust,ignore
/// use server::valence_bootstrap::{db_and_router, BootstrappedValence};
///
/// let boot: BootstrappedValence = db_and_router().await?;
/// assert!(!boot.default_backend_key.is_empty());
/// Ok::<(), anyhow::Error>(())
/// ```
pub struct BootstrappedValence {
    /// Process-wide database router shared by auth and subsystem factories.
    pub router: Arc<DatabaseRouter>,
    /// Default router key (`"{engine}:{logical}"`) for auth and factories.
    pub default_backend_key: String,
}

/// Connect SQLite storage and build the process [`DatabaseRouter`].
///
/// Env: `VALENCE_SQLITE_PATH` selects the database file (default
/// [`DEFAULT_SQLITE_PATH`]); `:memory:` gives an ephemeral store. Call once at
/// boot after Spectra install and before Chronon/Boson/Photon factories.
///
/// # Errors
///
/// Returns an error when the parent directory cannot be created, SQLite connect
/// fails, or permission harden fails on a file-backed path.
///
/// # Examples
///
/// See crate [Bootstrap Valence SQLite](../index.html#bootstrap-valence-sqlite).
pub async fn db_and_router() -> anyhow::Result<BootstrappedValence> {
    // Customize store location via VALENCE_SQLITE_PATH (file or `:memory:`).
    // Do not invent Hybrid/Postgres wiring here — that lives on the cloud host.
    let path = std::env::var("VALENCE_SQLITE_PATH").unwrap_or_else(|_| DEFAULT_SQLITE_PATH.into());
    if path != ":memory:" {
        ensure_parent_dir(&path)?;
    }
    log::info!("valence bootstrap: sqlite path={path}");
    let backend: Arc<dyn DatabaseBackend> = Arc::new(
        SqliteBackend::connect(&path)
            .await
            .with_context(|| format!("connect SqliteBackend at {path}"))?,
    );
    if path != ":memory:" {
        restrict_sqlite_file_permissions(&path)?;
    }

    // Logical names share this backend; add a slice in router_groups() for a new subsystem.
    let mut router = DatabaseRouter::new();
    register_backend_logical_names_slices(
        &mut router,
        backend,
        router_groups(),
        RegisterBackendLogicalNamesOptions::default(),
    );
    let default_backend_key = router_key("default", SQLITE_ENGINE_ID);
    Ok(BootstrappedValence {
        router: Arc::new(router),
        default_backend_key,
    })
}

/// Ensure the parent directory for a file-backed SQLite path exists.
///
/// Called from [`db_and_router`] before connect. Customize only if your path
/// layout needs different creation rules; `:memory:` skips this helper.
///
/// # Errors
///
/// Returns an error when `create_dir_all` fails.
///
/// # Examples
///
/// ```rust,ignore
/// use server::valence_bootstrap::ensure_parent_dir;
///
/// ensure_parent_dir("data/{{project-name}}.sqlite3")?;
/// println!("parent data/ ready");
/// Ok::<(), anyhow::Error>(())
/// ```
pub(crate) fn ensure_parent_dir(path: &str) -> anyhow::Result<()> {
    let p = PathBuf::from(path);
    if let Some(parent) = p.parent().filter(|p| !p.as_os_str().is_empty()) {
        std::fs::create_dir_all(parent)
            .with_context(|| format!("create SQLite parent directory {}", parent.display()))?;
    }
    Ok(())
}

/// Best-effort `0600` on the SQLite file so other local users cannot read secrets DBs (EM-03).
///
/// Runs after a successful file-backed connect. Skip for `:memory:`. Customize if
/// your deploy needs a different mode (document why); do not leave world-readable
/// secrets DBs on shared hosts.
///
/// # Errors
///
/// Returns an error when `stat` or `chmod` fails.
///
/// # Examples
///
/// ```rust,ignore
/// use server::valence_bootstrap::restrict_sqlite_file_permissions;
///
/// // After SqliteBackend::connect on a real file:
/// restrict_sqlite_file_permissions("data/{{project-name}}.sqlite3")?;
/// println!("sqlite mode set to 0600");
/// Ok::<(), anyhow::Error>(())
/// ```
pub(crate) fn restrict_sqlite_file_permissions(path: &str) -> anyhow::Result<()> {
    use std::os::unix::fs::PermissionsExt;
    let meta = std::fs::metadata(path)
        .with_context(|| format!("stat SQLite file {path} for permission harden"))?;
    let mut perms = meta.permissions();
    perms.set_mode(0o600);
    std::fs::set_permissions(path, perms.clone())
        .with_context(|| format!("chmod 0600 SQLite file {path}"))?;
    for suffix in ["-wal", "-shm"] {
        let sidecar = format!("{path}{suffix}");
        if std::path::Path::new(&sidecar).exists() {
            let _ = std::fs::set_permissions(&sidecar, perms.clone());
        }
    }
    Ok(())
}
