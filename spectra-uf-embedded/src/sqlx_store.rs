//! sqlx `SQLite` adapters implementing Spectra storage ports (same table layout as Spectra sqlite).
//!
//! Used by [`crate::install_embedded_sqlite`]. Customize only if you need a different
//! SQL dialect or schema; path selection stays on the crate root env helpers.

use std::path::{Path, PathBuf};

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use serde_json::Value;
use spectra_core::{
    Error, EventAggregateResult, EventRow, EventStorageBackend, EventsAggregateFilter,
    EventsQueryFilter, LabelMatcher, MetricPoint, MetricsQueryRange, MetricsStorageBackend, Result,
    StorageEngineType,
};
use sqlx::sqlite::{SqliteConnectOptions, SqlitePoolOptions};
use sqlx::{Row, SqlitePool};

const METRICS_DDL: &str = r"
CREATE TABLE IF NOT EXISTS spectra_metrics (
    name TEXT NOT NULL,
    kind TEXT NOT NULL,
    value REAL NOT NULL,
    labels TEXT NOT NULL,
    ts TEXT NOT NULL,
    correlation_id TEXT
);
CREATE INDEX IF NOT EXISTS idx_spectra_metrics_name_ts ON spectra_metrics(name, ts);
";

const EVENTS_DDL: &str = r"
CREATE TABLE IF NOT EXISTS spectra_events (
    table_name TEXT NOT NULL,
    fields TEXT NOT NULL,
    ts TEXT NOT NULL,
    correlation_id TEXT
);
CREATE INDEX IF NOT EXISTS idx_spectra_events_table_ts ON spectra_events(table_name, ts);
";

fn map_sqlx(e: &sqlx::Error) -> Error {
    Error::Internal(format!("sqlite: {e}"))
}

fn ts_to_rfc3339(ts: DateTime<Utc>) -> String {
    ts.to_rfc3339()
}

fn parse_ts(s: &str) -> Result<DateTime<Utc>> {
    DateTime::parse_from_rfc3339(s)
        .map(|dt| dt.with_timezone(&Utc))
        .map_err(|e| Error::Internal(format!("invalid metric/event timestamp: {e}")))
}

fn labels_match(labels: &Value, matchers: &[LabelMatcher]) -> bool {
    matchers.iter().all(|m| {
        labels
            .get(&m.key)
            .and_then(|v| v.as_str())
            .is_some_and(|v| v == m.value)
    })
}

async fn open_pool(path: &Path, ddl: &str) -> Result<SqlitePool> {
    if let Some(parent) = path.parent() {
        if !parent.as_os_str().is_empty() {
            std::fs::create_dir_all(parent).map_err(Error::Io)?;
        }
    }
    let opts = SqliteConnectOptions::new()
        .filename(path)
        .create_if_missing(true);
    let pool = SqlitePoolOptions::new()
        .max_connections(4)
        .connect_with(opts)
        .await
        .map_err(|e| map_sqlx(&e))?;
    sqlx::raw_sql(ddl)
        .execute(&pool)
        .await
        .map_err(|e| map_sqlx(&e))?;
    Ok(pool)
}

/// Durable metrics storage backed by sqlx `SQLite`.
///
/// Opened by [`crate::install_embedded_sqlite`]. Prefer that installer over
/// constructing backends by hand unless you are testing storage in isolation.
///
/// # Examples
///
/// ```rust,ignore
/// use spectra_uf_embedded::SqlxMetricsBackend;
/// let metrics = SqlxMetricsBackend::open("data/spectra-metrics.sqlite3").await?;
/// assert!(metrics.path().ends_with("spectra-metrics.sqlite3"));
/// Ok::<(), spectra_core::Error>(())
/// ```
#[derive(Clone)]
pub struct SqlxMetricsBackend {
    pool: SqlitePool,
    path: PathBuf,
}

impl SqlxMetricsBackend {
    /// Open or create the metrics database and apply DDL.
    ///
    /// Creates parent directories as needed. Customize the path argument (or env
    /// via [`crate::install_embedded_sqlite`]); leave DDL alone unless schema forks.
    ///
    /// # Errors
    ///
    /// Returns a Spectra storage error when directory create, connect, or DDL fails.
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// use spectra_uf_embedded::SqlxMetricsBackend;
    /// let backend = SqlxMetricsBackend::open("data/spectra-metrics.sqlite3").await?;
    /// println!("metrics at {}", backend.path().display());
    /// Ok::<(), spectra_core::Error>(())
    /// ```
    pub async fn open(path: impl AsRef<Path>) -> Result<Self> {
        let path = path.as_ref().to_path_buf();
        let pool = open_pool(&path, METRICS_DDL).await?;
        Ok(Self { pool, path })
    }

    /// Filesystem path of the backing database.
    ///
    /// Useful in tests and ops checks. Not a customize hook — change path at open.
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// use spectra_uf_embedded::SqlxMetricsBackend;
    /// let backend = SqlxMetricsBackend::open("data/spectra-metrics.sqlite3").await?;
    /// assert!(!backend.path().as_os_str().is_empty());
    /// Ok::<(), spectra_core::Error>(())
    /// ```
    pub fn path(&self) -> &Path {
        &self.path
    }
}

#[async_trait]
impl MetricsStorageBackend for SqlxMetricsBackend {
    fn engine_type(&self) -> StorageEngineType {
        StorageEngineType::Sqlite
    }

    async fn record_counter(
        &self,
        name: &str,
        labels: &Value,
        delta: i64,
        ts: DateTime<Utc>,
    ) -> Result<()> {
        sqlx::query(
            "INSERT INTO spectra_metrics (name, kind, value, labels, ts, correlation_id) \
             VALUES (?, 'counter', ?, ?, ?, NULL)",
        )
        .bind(name)
        .bind({
            #[allow(clippy::cast_precision_loss)]
            {
                delta as f64
            }
        })
        .bind(labels.to_string())
        .bind(ts_to_rfc3339(ts))
        .execute(&self.pool)
        .await
        .map_err(|e| map_sqlx(&e))?;
        Ok(())
    }

    async fn record_gauge(
        &self,
        name: &str,
        labels: &Value,
        value: f64,
        ts: DateTime<Utc>,
    ) -> Result<()> {
        sqlx::query(
            "INSERT INTO spectra_metrics (name, kind, value, labels, ts, correlation_id) \
             VALUES (?, 'gauge', ?, ?, ?, NULL)",
        )
        .bind(name)
        .bind(value)
        .bind(labels.to_string())
        .bind(ts_to_rfc3339(ts))
        .execute(&self.pool)
        .await
        .map_err(|e| map_sqlx(&e))?;
        Ok(())
    }

    async fn query_range(&self, query: MetricsQueryRange) -> Result<Vec<MetricPoint>> {
        let rows = sqlx::query(
            "SELECT value, labels, ts FROM spectra_metrics \
             WHERE name = ? AND ts >= ? AND ts <= ? ORDER BY ts ASC",
        )
        .bind(&query.metric_name)
        .bind(ts_to_rfc3339(query.start))
        .bind(ts_to_rfc3339(query.end))
        .fetch_all(&self.pool)
        .await
        .map_err(|e| map_sqlx(&e))?;

        let mut out = Vec::new();
        for row in rows {
            let value: f64 = row.try_get(0).map_err(|e| map_sqlx(&e))?;
            let labels_s: String = row.try_get(1).map_err(|e| map_sqlx(&e))?;
            let ts_s: String = row.try_get(2).map_err(|e| map_sqlx(&e))?;
            let labels: Value = serde_json::from_str(&labels_s)
                .map_err(|e| Error::Internal(format!("labels json: {e}")))?;
            if labels_match(&labels, &query.label_matchers) {
                out.push(MetricPoint {
                    ts: parse_ts(&ts_s)?,
                    value,
                    labels,
                });
            }
        }
        Ok(out)
    }
}

/// Durable events storage backed by sqlx `SQLite`.
///
/// Opened by [`crate::install_embedded_sqlite`]. Prefer that installer over
/// constructing backends by hand unless you are testing storage in isolation.
///
/// # Examples
///
/// ```rust,ignore
/// use spectra_uf_embedded::SqlxEventsBackend;
/// let events = SqlxEventsBackend::open("data/spectra-events.sqlite3").await?;
/// assert!(events.path().ends_with("spectra-events.sqlite3"));
/// Ok::<(), spectra_core::Error>(())
/// ```
#[derive(Clone)]
pub struct SqlxEventsBackend {
    pool: SqlitePool,
    path: PathBuf,
}

impl SqlxEventsBackend {
    /// Open or create the events database and apply DDL.
    ///
    /// Creates parent directories as needed. Customize the path argument (or env
    /// via [`crate::install_embedded_sqlite`]); leave DDL alone unless schema forks.
    ///
    /// # Errors
    ///
    /// Returns a Spectra storage error when directory create, connect, or DDL fails.
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// use spectra_uf_embedded::SqlxEventsBackend;
    /// let backend = SqlxEventsBackend::open("data/spectra-events.sqlite3").await?;
    /// println!("events at {}", backend.path().display());
    /// Ok::<(), spectra_core::Error>(())
    /// ```
    pub async fn open(path: impl AsRef<Path>) -> Result<Self> {
        let path = path.as_ref().to_path_buf();
        let pool = open_pool(&path, EVENTS_DDL).await?;
        Ok(Self { pool, path })
    }

    /// Filesystem path of the backing database.
    ///
    /// Useful in tests and ops checks. Not a customize hook — change path at open.
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// use spectra_uf_embedded::SqlxEventsBackend;
    /// let backend = SqlxEventsBackend::open("data/spectra-events.sqlite3").await?;
    /// assert!(!backend.path().as_os_str().is_empty());
    /// Ok::<(), spectra_core::Error>(())
    /// ```
    pub fn path(&self) -> &Path {
        &self.path
    }
}

#[async_trait]
impl EventStorageBackend for SqlxEventsBackend {
    fn engine_type(&self) -> StorageEngineType {
        StorageEngineType::Sqlite
    }

    async fn append_row(
        &self,
        table: &str,
        fields: &Value,
        ts: DateTime<Utc>,
        correlation_id: Option<&str>,
    ) -> Result<()> {
        sqlx::query(
            "INSERT INTO spectra_events (table_name, fields, ts, correlation_id) \
             VALUES (?, ?, ?, ?)",
        )
        .bind(table)
        .bind(fields.to_string())
        .bind(ts_to_rfc3339(ts))
        .bind(correlation_id)
        .execute(&self.pool)
        .await
        .map_err(|e| map_sqlx(&e))?;
        Ok(())
    }

    async fn query_rows(&self, filter: EventsQueryFilter) -> Result<Vec<EventRow>> {
        let start = filter.start.map(ts_to_rfc3339);
        let end = filter.end.map(ts_to_rfc3339);
        let rows = sqlx::query(
            "SELECT fields, ts FROM spectra_events WHERE table_name = ? \
             AND (? IS NULL OR ts >= ?) AND (? IS NULL OR ts <= ?)",
        )
        .bind(&filter.table)
        .bind(start.as_deref())
        .bind(start.as_deref())
        .bind(end.as_deref())
        .bind(end.as_deref())
        .fetch_all(&self.pool)
        .await
        .map_err(|e| map_sqlx(&e))?;

        let mut out = Vec::new();
        for row in rows {
            let fields_s: String = row.try_get(0).map_err(|e| map_sqlx(&e))?;
            let ts_s: String = row.try_get(1).map_err(|e| map_sqlx(&e))?;
            out.push(EventRow {
                ts: parse_ts(&ts_s)?,
                fields: serde_json::from_str(&fields_s)
                    .map_err(|e| Error::Internal(format!("fields json: {e}")))?,
            });
        }
        let mut filter = filter;
        if filter.limit.is_none() {
            filter.limit = Some(1000);
        }
        Ok(spectra_core::finalize_event_rows(out, &filter))
    }

    async fn query_aggregate(
        &self,
        _filter: EventsAggregateFilter,
    ) -> Result<EventAggregateResult> {
        Ok(EventAggregateResult::TimeSeries {
            series: Vec::new(),
            headline: Vec::new(),
        })
    }
}
