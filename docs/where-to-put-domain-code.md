# Where to put your domain code

This host is the shell. Domain behavior lives in uf-app crates (UI) and matching worker / service crates (Valence models, Chronon scripts, Boson tasks, Photon topics). Host glue stays thin.

## Clear homes

| Concern | Home | Notes |
|---------|------|-------|
| Leptos routes / pages | Your `*-uf-app` crate; mount via `MainAppRouteChildren` | See [Add or remove a uf-app](add-or-remove-uf-app.md) |
| Valence models / privacy / services | Worker or `service/` crate next to the uf-app | Force-link the worker from `server` so `sync_typed_tables_from_registry` sees schemas |
| Chronon jobs / scripts | Worker `scripts/` modules; register on Chronon at worker or host boot | Host already registers Valence platform sweeps |
| Boson tasks / side effects | Worker `side_effects/` and `#[boson::task]` handlers | Need Boson runtime alive in `compose_embedded_platform` |
| Photon topics / realtime | Worker event types; UI consumes via Photon leptos helpers | Needs Photon + transport key |
| Appearance / branding SSR helpers | `lepton-appearance` in this workspace, or product helpers | Keep Orbital shell in `app` |
| Host boot / bind / composition | `server` (`platform`, `valence_bootstrap`, `build_router`) | Change only when ops or runtime topology changes |

Reference layout: `counter-app` (UI) + `counter-app-worker` (models, scripts, tasks, topics). Both are git deps in the workspace root `Cargo.toml` (not in-tree directories). Host wiring to copy: `app/src/main_app_routes.rs`, `app/Cargo.toml` features, `app/build.rs` `extra_packages`, and `server/src/lib.rs` (`use counter_app_worker as _` plus boot seed under `server-embedded`). The host seeds the counter singleton and bot users from that worker at boot when `server-embedded` is on.

## Leave alone (unless you mean to)

- `compose_embedded_platform` boot order and `*-uf-embedded` builders (ops / runtime swaps: [Swap or disable a runtime](swap-or-disable-runtime.md))
- `ProcessValenceFactory` internal vs external vs Higgs factories (actor JSON security boundary)
- `app::shell` / `App` Orbital chrome and `frontend::hydrate` (stock document / WASM entry; ordinary product work mounts uf-apps via `MainAppRouteChildren`)
- Generated `OUT_DIR` route tables from `uf_codegen` (edit `MainAppRouteChildren` and `build.rs` instead)
- Marketing / Coming Soon (lives on `unified-field-site`)

## Growing a first-party app

1. Create (or vendor) a uf-app + worker pair with Valence models and UI routes; add both as path or git deps in the workspace `Cargo.toml`.
2. Mount routes and Cargo features ([Add or remove a uf-app](add-or-remove-uf-app.md)).
3. Force-link the worker from `server/src/lib.rs` and enable it on the `server-embedded` feature in `server/Cargo.toml` so registry sync and Chronon/Boson macros resolve.
4. Keep request handlers on the session Valence from Higgs ([Auth / session surface](auth-and-session.md)).
5. Rebuild with `cargo leptos watch --split` and hit the new paths.

## Related

- [Run locally](run-locally.md)
- [Data & Valence bootstrap](data-and-valence-bootstrap.md)
