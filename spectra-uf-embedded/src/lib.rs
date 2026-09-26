//! Spectra SQLite adapters for in-process Unified Field hosts.
//!
//! Opens one durable metrics + events `SQLite` file pair **per distinct `store:` name**
//! declared across every `spectra_schema!`/`spectra_metric!` linked into the binary (see
//! [`spectra_core::collect_distinct_spectra_store_names`]) with sqlx (shared
//! `libsqlite3-sys` with other host SQLite crates), and returns a configured
//! [`spectra::Spectra`] handle. Each store's files are physically isolated from every
//! other store's — no product's telemetry volume or lock contention affects another's.
//! Customize the base directory via env; call [`install_embedded_sqlite`] first in boot
//! order.
//!
//! ## Features
//!
//! - **Per-store Spectra SQLite backends** — Installs one metrics + events file pair per
//!   declared store via [`install_embedded_sqlite`]. [Get started](#install-spectra-sqlite)
//! - **Event chart aggregates** — [`SqlxEventsBackend`] `query_aggregate` loads matching
//!   event rows and buckets them for Time series / Line (or groups for Pie / Bar when
//!   `group_by_field` is set). [Get started](#query-event-aggregates)
//!
//! ## Install Spectra SQLite
//!
//! [`install_embedded_sqlite`] builds Spectra with one embedded SQLite backend pair per
//! distinct declared `store:` name. Call it first in the host boot order (before Valence
//! telemetry install and other runtimes) so metrics and events have a place to land.
//!
//! **Prerequisites:** A writable base directory; sqlx SQLite available in the
//! dependency graph.
//!
//! ```rust,ignore
//! use std::sync::Arc;
//! use spectra::Spectra;
//! use spectra_uf_embedded::install_embedded_sqlite;
//!
//! async fn boot() -> anyhow::Result<Arc<Spectra>> {
//!     let spectra = install_embedded_sqlite().await?;
//!     assert!(Arc::strong_count(&spectra) >= 1);
//!     println!("spectra sqlite backends installed, one file pair per declared store");
//!     Ok(spectra)
//! }
//! ```
//!
//! On success you hold an `Arc<Spectra>` ready for Higgs and kit telemetry. Directory
//! or SQLite open failures for any one store return `Err`. Override the base directory
//! with the env var below.
//!
//! ## Query event aggregates
//!
//! Explore chart views call `query_aggregate` on the embedded events file. Event log
//! stays on `query_rows`. After [`SqlxEventsBackend::open`] (or
//! [`install_embedded_sqlite`]), append rows then aggregate with Count (or Sum) over a
//! time range. Empty tables return an empty series; open/query failures return `Err`.
//!
//! **Prerequisites:** A writable events SQLite path; at least one in-range row for a
//! non-empty chart.
//!
//! ```rust,ignore
//! use chrono::{Duration, Utc};
//! use serde_json::json;
//! use spectra_core::{
//!     EventAggregateResult, EventMeasure, EventStorageBackend, EventsAggregateFilter,
//!     GridFilterModel,
//! };
//! use spectra_uf_embedded::SqlxEventsBackend;
//!
//! async fn chart_from_embedded_sqlite(dir: &std::path::Path) -> spectra_core::Result<f64> {
//!     let backend = SqlxEventsBackend::open(dir.join("events.sqlite3")).await?;
//!     let end = Utc::now();
//!     let start = end - Duration::hours(1);
//!     backend
//!         .append_row(
//!             "demo.events",
//!             &json!({"severity": "info"}),
//!             end - Duration::minutes(5),
//!             None,
//!         )
//!         .await?;
//!     let result = backend
//!         .query_aggregate(EventsAggregateFilter {
//!             table: "demo.events".into(),
//!             start,
//!             end,
//!             partition: None,
//!             filter: GridFilterModel::default(),
//!             measure: EventMeasure::Count,
//!             measure_field: None,
//!             time_bucket_secs: Some(3600),
//!             group_by_field: None,
//!         })
//!         .await?;
//!     let total = match result {
//!         EventAggregateResult::TimeSeries { series, .. } => series
//!             .iter()
//!             .flat_map(|s| s.points.iter())
//!             .map(|p| p.value)
//!             .sum(),
//!         other => panic!("expected time series, got {other:?}"),
//!     };
//!     assert!((total - 1.0).abs() < f64::EPSILON);
//!     Ok(total)
//! }
//! ```
//!
//! Pie and Bar need a non-empty `group_by_field`. Next: Event log stays `query_rows`.
//!
//! ## Env
//!
//! | Variable | Default |
//! |----------|---------|
//! | `SPECTRA_STORE_BASE_PATH` | `data/spectra-stores` |
//!
//! Each store's files land at `{base}/{store}/spectra-{metrics,events}.sqlite3`, e.g.
//! `data/spectra-stores/default/spectra-metrics.sqlite3` or
//! `data/spectra-stores/counter/spectra-events.sqlite3` for a product declaring
//! `store: "counter"`. Isolation is always on for this installer — there is no shared-file
//! mode or `SPECTRA_STORE_ISOLATION` switch.

mod sqlx_store;

use std::path::{Path, PathBuf};
use std::sync::Arc;

use spectra::Spectra;

pub use sqlx_store::{SqlxEventsBackend, SqlxMetricsBackend};

/// Default base directory for per-store `SQLite` files.
///
/// # Examples
///
/// ```rust,ignore
/// use spectra_uf_embedded::DEFAULT_STORE_BASE_PATH;
/// assert_eq!(DEFAULT_STORE_BASE_PATH, "data/spectra-stores");
/// ```
pub const DEFAULT_STORE_BASE_PATH: &str = "data/spectra-stores";

/// Env var overriding the per-store `SQLite` base directory.
///
/// # Examples
///
/// ```rust,ignore
/// use spectra_uf_embedded::SPECTRA_STORE_BASE_PATH_ENV;
/// assert_eq!(SPECTRA_STORE_BASE_PATH_ENV, "SPECTRA_STORE_BASE_PATH");
/// ```
pub const SPECTRA_STORE_BASE_PATH_ENV: &str = "SPECTRA_STORE_BASE_PATH";

/// Resolve a Spectra path from env or the given default.
///
/// # Examples
///
/// ```rust,ignore
/// use spectra_uf_embedded::{path_from_env, SPECTRA_STORE_BASE_PATH_ENV, DEFAULT_STORE_BASE_PATH};
/// println!("{}", path_from_env(SPECTRA_STORE_BASE_PATH_ENV, DEFAULT_STORE_BASE_PATH).display());
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

/// Install Spectra with durable, per-store `SQLite` backends for embedded hosts.
///
/// Opens one metrics + events file pair per distinct `store:` name declared across every
/// `spectra_schema!`/`spectra_metric!` linked into the binary (see
/// [`spectra_core::collect_distinct_spectra_store_names`]), so each product's telemetry is
/// physically isolated on disk. Call first in host boot order before Valence telemetry and
/// other runtimes.
///
/// # Errors
///
/// Returns an error if any store's directory cannot be created, `SQLite` open fails, a
/// declared store name is not a valid Spectra identifier, or Spectra build fails.
///
/// # Examples
///
/// See [Install Spectra SQLite](index.html#install-spectra-sqlite).
pub async fn install_embedded_sqlite() -> anyhow::Result<Arc<Spectra>> {
    // Call first in host boot. Base directory: SPECTRA_STORE_BASE_PATH (see Env table on
    // the crate root); one subdirectory per declared store under it.
    let base = path_from_env(SPECTRA_STORE_BASE_PATH_ENV, DEFAULT_STORE_BASE_PATH);
    let mut builder = Spectra::builder();

    for store in spectra_core::collect_distinct_spectra_store_names() {
        spectra_core::validate_spectra_ident(&store)
            .map_err(|e| anyhow::anyhow!("invalid spectra store name {store:?}: {e}"))?;

        let dir = base.join(&store);
        let metrics_path = dir.join("spectra-metrics.sqlite3");
        let events_path = dir.join("spectra-events.sqlite3");
        ensure_parent(&metrics_path)?;
        ensure_parent(&events_path)?;

        let metrics = SqlxMetricsBackend::open(&metrics_path)
            .await
            .map_err(|e| anyhow::anyhow!("Spectra metrics SQLite open failed ({store}): {e}"))?;
        let events = SqlxEventsBackend::open(&events_path)
            .await
            .map_err(|e| anyhow::anyhow!("Spectra events SQLite open failed ({store}): {e}"))?;
        restrict_sqlite_file_permissions(&metrics_path).map_err(|e| {
            anyhow::anyhow!("Spectra metrics SQLite chmod 0600 failed ({store}): {e}")
        })?;
        restrict_sqlite_file_permissions(&events_path).map_err(|e| {
            anyhow::anyhow!("Spectra events SQLite chmod 0600 failed ({store}): {e}")
        })?;

        log::info!(
            "spectra.embedded.store_opened: store={store} metrics_path={} events_path={}",
            metrics_path.display(),
            events_path.display(),
        );

        builder = if store == "default" {
            builder
                .metrics_backend(Arc::new(metrics))
                .events_backend(Arc::new(events))
        } else {
            builder.store_backend(store, Arc::new(metrics), Arc::new(events))
        };
    }

    let spectra = builder
        .embedded()
        .build()
        .map_err(|e| anyhow::anyhow!("Spectra SQLite build failed: {e}"))?;

    Ok(Arc::new(spectra))
}

// Test-only schemas so `collect_distinct_spectra_store_names()` has non-default stores to
// isolate in `install_embedded_sqlite_isolates_stores_happy` below. Declared at module
// scope (not inside `mod tests`) so `inventory::submit!` links them into the test binary
// exactly once regardless of which test file triggers `SchemaRegistry::global()` first.
#[cfg(test)]
spectra::spectra_metric! {
    SpectraUfEmbeddedIsolationTestACounter {
        store: "spectra_uf_embedded_isolation_test_a",
        name: "spectra_uf_embedded_isolation_test_a_counter",
        version: "0.1.0",
        description: "test-only metric proving per-store SQLite file isolation",
    }
}

#[cfg(test)]
spectra::spectra_metric! {
    SpectraUfEmbeddedIsolationTestBCounter {
        store: "spectra_uf_embedded_isolation_test_b",
        name: "spectra_uf_embedded_isolation_test_b_counter",
        version: "0.1.0",
        description: "test-only metric proving per-store SQLite file isolation",
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use spectra_core::{MetricsQueryRange, MetricsStorageBackend};

    static ENV_LOCK: tokio::sync::Mutex<()> = tokio::sync::Mutex::const_new(());

    #[tokio::test]
    async fn install_embedded_sqlite_happy_path() {
        let _guard = ENV_LOCK.lock().await;
        let dir = tempfile::tempdir().expect("tempdir");
        std::env::set_var(SPECTRA_STORE_BASE_PATH_ENV, dir.path());
        let spectra = install_embedded_sqlite()
            .await
            .expect("install_embedded_sqlite");
        let _ = spectra.as_ref();
    }

    #[tokio::test]
    async fn install_embedded_sqlite_per_store_dir_not_writable_sad() {
        let _guard = ENV_LOCK.lock().await;
        let dir = tempfile::tempdir().expect("tempdir");
        // "default" sorts first among distinct store names (BTreeSet) and always exists,
        // so blocking its subdirectory with a plain file reproduces a per-store open
        // failure without depending on which other stores are linked into this binary.
        let blocker = dir.path().join("default");
        std::fs::write(&blocker, b"x").expect("write blocker");
        std::env::set_var(SPECTRA_STORE_BASE_PATH_ENV, dir.path());

        let Err(err) = install_embedded_sqlite().await else {
            panic!("default store's directory is a file, not a directory");
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

    #[tokio::test]
    // Two deliberately parallel store-a/store-b bindings read more clearly than
    // artificially distinct names here; clippy's `similar_names` only flags the trailing
    // letter.
    #[allow(clippy::similar_names)]
    async fn install_embedded_sqlite_isolates_stores_happy() {
        let _guard = ENV_LOCK.lock().await;
        let dir = tempfile::tempdir().expect("tempdir");
        std::env::set_var(SPECTRA_STORE_BASE_PATH_ENV, dir.path());

        let spectra = install_embedded_sqlite()
            .await
            .expect("install_embedded_sqlite");
        let router = spectra.router();
        let ts = chrono::Utc::now();

        router
            .resolve_metrics("spectra_uf_embedded_isolation_test_a_counter")
            .record_counter(
                "spectra_uf_embedded_isolation_test_a_counter",
                &serde_json::json!({}),
                1,
                ts,
            )
            .await
            .expect("record store a counter");
        router
            .resolve_metrics("spectra_uf_embedded_isolation_test_b_counter")
            .record_counter(
                "spectra_uf_embedded_isolation_test_b_counter",
                &serde_json::json!({}),
                1,
                ts,
            )
            .await
            .expect("record store b counter");

        // Open each store's file directly (bypassing the router entirely) to prove the
        // rows are physically separated on disk, not just logically namespaced.
        let store_a_file = SqlxMetricsBackend::open(
            dir.path()
                .join("spectra_uf_embedded_isolation_test_a")
                .join("spectra-metrics.sqlite3"),
        )
        .await
        .expect("open store a file directly");
        let store_b_file = SqlxMetricsBackend::open(
            dir.path()
                .join("spectra_uf_embedded_isolation_test_b")
                .join("spectra-metrics.sqlite3"),
        )
        .await
        .expect("open store b file directly");

        let range = |metric_name: &str| MetricsQueryRange {
            metric_name: metric_name.to_string(),
            start: ts - chrono::Duration::seconds(5),
            end: ts + chrono::Duration::seconds(5),
            label_matchers: vec![],
        };

        let a_has_a = store_a_file
            .query_range(range("spectra_uf_embedded_isolation_test_a_counter"))
            .await
            .expect("query store a file for its own counter");
        assert_eq!(a_has_a.len(), 1, "store a's file must contain its own row");

        let a_has_b = store_a_file
            .query_range(range("spectra_uf_embedded_isolation_test_b_counter"))
            .await
            .expect("query store a file for store b's counter");
        assert!(
            a_has_b.is_empty(),
            "store a's file must not contain store b's row"
        );

        let b_has_b = store_b_file
            .query_range(range("spectra_uf_embedded_isolation_test_b_counter"))
            .await
            .expect("query store b file for its own counter");
        assert_eq!(b_has_b.len(), 1, "store b's file must contain its own row");

        let b_has_a = store_b_file
            .query_range(range("spectra_uf_embedded_isolation_test_a_counter"))
            .await
            .expect("query store b file for store a's counter");
        assert!(
            b_has_a.is_empty(),
            "store b's file must not contain store a's row"
        );
    }
}
