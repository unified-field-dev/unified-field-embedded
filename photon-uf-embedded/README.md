# photon-uf-embedded

Embedded host wiring for Photon: SQLite storage (`SqliteStoragePort`) for the single-process `server` binary.

| Setting | Default |
|---------|---------|
| Continuum | Embedded / local |
| Storage | SQLite (`photon::SqliteStoragePort`) |
| Cross-node push | Off (single process) |

## Env

| Variable | Default |
|----------|---------|
| `PHOTON_SQLITE_PATH` | `data/photon.sqlite3` |

Wiring only — does not redefine Photon backend semantics. Use `photon_uf_embedded::build_runtime` from the host.

```rust
use photon_uf_embedded::build_runtime as build_photon_runtime;

let runtime = build_photon_runtime(valence_factory).await?;
```

`build_runtime` opens the SQLite file (creating parent directories as needed), builds `Photon` with `auto_registry()`, and calls `photon::configure` so macros / publish paths resolve process-wide.
