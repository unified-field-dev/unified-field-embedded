# Swap or disable a runtime (Boson / Chronon / Photon / Spectra)

Platform runtimes compose in `server::platform::compose_embedded_platform`. HTTP, sessions, and Leptos stay in `server::build_router`. Change adapters in `compose_embedded_platform` and the `*-uf-embedded` builders; keep boot order intact.

## Boot order

1. Spectra SQLite (`spectra_uf_embedded::install_embedded_sqlite`)
2. Valence Spectra telemetry install
3. Valence `db_and_router`
4. Chronon (`chronon_uf_embedded::build_chronon_runtime`)
5. Boson (`boson_uf_embedded::build_boson_runtime`)
6. Photon (`photon_uf_embedded::build_runtime`)
7. `HiggsConfig` (Chronon + Boson + Photon + Valence factory)

`build_router` then seeds Valence tables / demo data and mounts Axum + Leptos + Photon WS.

## Env retarget (keep all runtimes)

| Runtime | Path / knobs |
|---------|----------------|
| Spectra | `SPECTRA_METRICS_SQLITE_PATH`, `SPECTRA_EVENTS_SQLITE_PATH` |
| Chronon | `CHRONON_SQLITE_PATH`; `CHRONON_TELEMETRY=off` in labs |
| Boson | `BOSON_SQLITE_PATH`; `BOSON_TELEMETRY=off` in labs |
| Photon | `PHOTON_SQLITE_PATH`, `PHOTON_TRANSPORT_KEY` |

Builders open parents, connect SQLite, and configure process-global handles. Crate READMEs: [spectra-uf-embedded](../spectra-uf-embedded/README.md), [chronon-uf-embedded](../chronon-uf-embedded/README.md), [boson-uf-embedded](../boson-uf-embedded/README.md), [photon-uf-embedded](../photon-uf-embedded/README.md).

## Swap an adapter

1. Implement or depend on the replacement builder (same traits / return shapes the host expects: Chronon scheduler+backend+registry, Boson backend, Photon client, Spectra handle).
2. Call it from `compose_embedded_platform` in place of the stock `*_uf_embedded` call.
3. Keep Higgs builder wiring: Chronon, Boson, and Photon still need live handles if product code or platform apps call those subsystems.
4. Rebuild and run `cargo test -p {{project-name}}-e2e -- --test-threads=1` (or an equivalent IsolatedLab smoke) so composition still boots.

## Disable a runtime

This template’s Higgs and platform admin UI assume Chronon, Boson, Photon, and Spectra are present. Disabling one means coordinated edits:

1. Stop calling its `*_uf_embedded` builder in `compose_embedded_platform`.
2. Drop or stub the matching `HiggsConfig::builder()` method so Higgs construction still succeeds.
3. Remove or gate the matching `*Routes` / uf-app from `MainAppRouteChildren` and Cargo features ([Add or remove a uf-app](add-or-remove-uf-app.md)).
4. Drop `AppState` keep-alive fields and `provide_context` lines in `build_router` for that runtime.
5. Expect Valence platform jobs that register Chronon sweeps, Photon WS routes, or Boson side effects to need the same trim.

Partial disable without those follow-ups leaves boot or first-request failures. Prefer env retarget when you only need different disk paths.

## Headless composition

Labs and tests can call `compose_embedded_platform` without HTTP. Shared env install helpers live in `{{project-name}}-e2e` (`install_lab_env`, scenario IDs in that crate’s README).

## Next

- Valence store layout: [Data & Valence bootstrap](data-and-valence-bootstrap.md)
- Ops exposure: [Deploy / ops knobs](deploy.md)
