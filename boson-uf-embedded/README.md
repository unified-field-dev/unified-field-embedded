# boson-uf-embedded

Embedded host wiring for Boson: SQLite queue backend (`SqliteQueueBackend`).

## Defaults

| Setting | Value |
|---------|-------|
| Queue backend | SQLite (`boson::SqliteQueueBackend`) |
| Idempotency | **`None`** (off) — at-least-once; no enqueue dedup |

Override idempotency per task with `#[boson::task(idempotency_mode = "lwt")]` or a persisted `TaskConfig`. The boson-app UI does not expose this field.

## Env

| Variable | Default |
|----------|---------|
| `BOSON_SQLITE_PATH` | `data/boson.sqlite3` |

## Spectra telemetry

Install via `boson-spectra-telemetry` (`install_ops_log_from_env`) after Spectra SQLite boot — see server composition.

## Usage

```rust
let runtime = boson_uf_embedded::build_boson_runtime(valence_factory).await?;
```

`build_boson_runtime` opens the SQLite file (creating parent directories as needed), builds `Boson` with `auto_registry()`, and calls `boson_runtime::configure` so the process-global handle resolves.
