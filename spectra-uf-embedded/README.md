# spectra-uf-embedded

Installs durable SQLite Spectra metrics/events storage for the embedded host
(`install_embedded_sqlite`), using sqlx so a single `libsqlite3-sys` links in-process.

## Env

| Variable | Default |
|----------|---------|
| `SPECTRA_METRICS_SQLITE_PATH` | `data/spectra-metrics.sqlite3` |
| `SPECTRA_EVENTS_SQLITE_PATH` | `data/spectra-events.sqlite3` |

Family telemetry adapters are separate crates, installed after this boot:

- `valence-spectra-telemetry::install_from_env` (server)
- `chronon-spectra-telemetry::install_from_env` (`chronon-uf-embedded`)
- `boson-spectra-telemetry::install_ops_log_from_env` (`boson-uf-embedded`)
