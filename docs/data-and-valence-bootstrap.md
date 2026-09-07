# Data & Valence bootstrap

Valence opens once at process start through `server::valence_bootstrap::db_and_router`, then Chronon / Boson / Photon factories share that router. This template is SQLite embedded only. Remote Hybrid (IndraDB + Postgres) lives on the cloud host, outside this repo.

## Path

```bash
export VALENCE_SQLITE_PATH=data/{{project-name}}.sqlite3   # default
# or
export VALENCE_SQLITE_PATH=:memory:
```

Constant default: `valence_bootstrap::DEFAULT_SQLITE_PATH`. File parents are created; file-backed DBs get mode `0600` after connect.

## `db_and_router`

1. Resolve `VALENCE_SQLITE_PATH`
2. Connect `SqliteBackend`
3. Harden file permissions when not `:memory:`
4. Register logical backend groups via `router_groups()`
5. Return `BootstrappedValence { router, default_backend_key }`

`default_backend_key` is `router_key("default", SQLITE_ENGINE_ID)` for auth and factories.

## `router_groups`

Stock logical namespaces on the shared SQLite backend:

- `default`
- `chronon`
- `boson`
- `photon`
- `permissions`
- `secrets`

Add a slice when a new subsystem needs its own logical name. Keep `default` first so auth and factories resolve the stock backend key. There is no `gluon` group on this host.

## Boot placement

`compose_embedded_platform` calls `db_and_router` after Spectra install and before Chronon / Boson / Photon. `build_router` then:

- Syncs typed tables from the Valence registry
- Ensures TTL metadata
- Registers Valence platform Chronon jobs (deletion / TTL / iter)
- Seeds demo counter data when `server-embedded` is on

SQLite embedded defaults `VALENCE_OWNERSHIP_UNIFIED_FETCH=0` in the `server` binary
`main` and in `{{project-name}}-e2e::install_lab_env` (ownership JOIN fetch path). Set
the env var yourself before boot when you need a different value; `build_router`
does not mutate process env.

## Cloud Hybrid

Do not wire Hybrid IndraDB + Postgres into this template. Fork or use `unified-field-cloud` (or the fleet hosts) when you need that topology. Path and `router_groups` edits here stay on the single SQLite file.

## Related

- [Deploy / ops knobs](deploy.md)
- Session Valence usage from request handlers: [Auth / session surface](auth-and-session.md)
- Maintainer verification: [VERIFICATION.md](VERIFICATION.md)
