# chronon-uf-embedded

Embedded Chronon host wiring for {{app_title}}: SQLite `SchedulerStore` (`SqliteSchedulerStore`) via `ChrononBuilder::embedded()`.

## Env

| Variable | Default |
|----------|---------|
| `CHRONON_SQLITE_PATH` | `data/chronon.sqlite3` |

## Usage

```rust
use chronon_uf_embedded::build_chronon_runtime;

let runtime = build_chronon_runtime(valence_factory).await?;
```

`build_chronon_runtime` opens the SQLite file (creating parent directories as needed), builds a `Chronon` with `auto_registry()` + `embedded()`, registers default jobs, and spawns the tick/worker run loop. The returned `ChrononEmbeddedRuntime` carries a `shutdown: Arc<Notify>` for cooperative shutdown.

## Spectra telemetry

`build_chronon_runtime` installs a Spectra-backed sink from `CHRONON_TELEMETRY` via `chronon-spectra-telemetry::install_from_env` (call after Spectra SQLite boot in the host — see `spectra-uf-embedded`).
