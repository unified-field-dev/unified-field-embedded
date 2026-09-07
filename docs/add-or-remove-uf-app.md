# Add or remove a uf-app

The product route table is `app::main_app_routes::MainAppRouteChildren`. Edit that component and the matching Cargo features/deps, rebuild with `--split`, and the path either mounts or falls through to `HostNotFoundPage`.

## Where routes live

`app/src/main_app_routes.rs` mounts:

- Demos: counter, welcome, apps directory, tags, orbital preview
- Auth: `LeptonAuthRoutes`
- Platform admin: notifications, user, Valence, Chronon, Photon, Spectra, Boson, Gauge (permissions), Neutrino (secrets)

Catch-all `/*any` → `HostNotFoundPage` (product 404). Marketing and Coming Soon stay on `unified-field-site`.

`app/build.rs` feeds `uf_codegen::generate_registered_routes` so Orbital appearance / page-view tracking see the same package set. Keep `extra_packages` and `MainAppRouteChildren` in sync when you add or drop a uf-app.

## Add a uf-app

Stock demos such as `counter-app` are workspace git dependencies in the root `Cargo.toml`, not folders inside this repo. Copy the wiring pattern from the files below (and from that git checkout if you need the app’s own sources).

1. Depend on the uf-app crate from `app/Cargo.toml`, and enable it under both the `hydrate` and `ssr` feature lists (mirror how `counter-app` is listed today).
2. Import its `*Routes` component in `app/src/main_app_routes.rs` and insert a child under the parent route (keep the `/*any` catch-all last).
3. Append the package name to `extra_packages` in `app/build.rs` so Orbital appearance / page-view tracking sees the same set.
4. If the app ships Valence models, Chronon scripts, or Boson tasks, add the worker crate under `server/Cargo.toml` (usually gated on `server-embedded`) and force-link it from `server/src/lib.rs` the way `counter_app_worker` is linked (`use counter_app_worker as _;` under `#[cfg(feature = "server-embedded")]`). Inventory registration only runs for crates linked into the server binary.
5. Rebuild:

```bash
cargo leptos watch --split
```

Matched paths render the new app. Wrong or missing Cargo features usually fail at compile time; skipping `extra_packages` or the server force-link shows up as missing routes in appearance tracking or missing tables/jobs at boot.

## Remove a uf-app

1. Delete its `<…Routes />` child from `MainAppRouteChildren`.
2. Drop unused deps and feature entries from `app/Cargo.toml` (hydrate + ssr).
3. Remove the package from `app/build.rs` `extra_packages`.
4. Rebuild. Former URLs hit `HostNotFoundPage`.

Leave auth (`LeptonAuthRoutes`) and at least one navigable product surface if you still want a usable shell.

## Hygiene check

```bash
cargo test -p app --features ssr,server-embedded --test full_product_surface_hygiene
```

That test asserts the stock full-product surface. After intentional trims, update or gate the test so CI matches your fork’s route set.

## Next

- Domain UI vs workers: [Where to put your domain code](where-to-put-domain-code.md)
- Runtime adapters stay in `*-uf-embedded`: [Swap or disable a runtime](swap-or-disable-runtime.md)
