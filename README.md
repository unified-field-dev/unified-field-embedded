# {{app_title}}

[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](LICENSE)

Full-product embedded host template: one process with auth, SQLite persistence,
background work, realtime, and platform admin UI (Valence, Chronon, Photon,
Boson, Spectra, Gauge, Neutrino, user, notifications) always compiled in.

**Create a host** with [cargo-generate](docs/create-from-template.md) (primary).
GitHub **Use this template** copies Liquid markers as-is and does not fill in
`project-name` / `app_title` — prefer cargo-generate.

Public marketing and Coming Soon live in
[`unified-field-site`](https://github.com/unified-field-dev/unified-field-site).

## Create a host

```bash
cargo install cargo-generate
cargo generate --git https://github.com/unified-field-dev/unified-field-embedded --allow-commands
cd <project-name>
export PHOTON_TRANSPORT_KEY=cGhvdG9uLWRldi10cmFuc3BvcnQta2V5LTMyYnl0ZXM=
cargo leptos watch --split
```

Details, GitHub-button caveat, and hydrate 404 checklist:
[docs/create-from-template.md](docs/create-from-template.md).

## Features

- **Host process entry** — Bind and serve the full product on one port via
  `server::run`. [Run locally](docs/run-locally.md); rustdoc: `server` Serve host.
- **Embedded platform** — Compose Spectra → Valence → Chronon → Boson → Photon →
  Higgs in process. [Swap or disable a runtime](docs/swap-or-disable-runtime.md).
- **Valence SQLite router** — Durable local store and logical backend groups.
  [Data & Valence bootstrap](docs/data-and-valence-bootstrap.md).
- **HTTP router stack** — Sessions, Photon WS, static pkg, and Leptos SSR layers.
  Ops knobs: [Deploy](docs/deploy.md), [Auth / session](docs/auth-and-session.md).
- **uf-app route table** — Add or remove product apps via `MainAppRouteChildren`.
  [Add or remove a uf-app](docs/add-or-remove-uf-app.md).
- **Domain vs host glue** — Where UI, workers, and models belong.
  [Where to put domain code](docs/where-to-put-domain-code.md).

The product surface is always on (no `full` / `server-full` feature gate).
Unknown routes are 404.

## Run locally

After generate (or on a generated checkout), set a **DEV ONLY** Photon transport
key, start the split Leptos watch, and open the loopback URL.

```bash
# DEV ONLY — never reuse in shared or production hosts.
#   openssl rand -base64 32
export PHOTON_TRANSPORT_KEY=CHANGE_ME
cargo leptos watch --split
```

`--split` is required (lazy WASM routes). Open `http://127.0.0.1:3000`.

Step-by-step SQLite defaults, listen address, and OAuth: [docs/run-locally.md](docs/run-locally.md).

Template-repo maintainers: this Liquid source tree does not build in place. Use
`scripts/verify-template-generate.sh` (generate scratch → check) or the upstream
`template-ci` workflow.

## Customize deployment

Bind address, public URL, SQLite paths, Photon transport key, session cookie
Secure, and WebSocket Origin allowlist:

- [docs/deploy.md](docs/deploy.md): ops knobs and exposure boundaries
- [docs/swap-or-disable-runtime.md](docs/swap-or-disable-runtime.md): Boson / Chronon / Photon / Spectra builders
- [docs/data-and-valence-bootstrap.md](docs/data-and-valence-bootstrap.md): `VALENCE_SQLITE_PATH`, `db_and_router`, `router_groups`
- [docs/auth-and-session.md](docs/auth-and-session.md): session Secure, Origin allowlist, OAuth / Neutrino

## Grow your app

Add or trim uf-apps and place domain code beside the shell:

- [docs/add-or-remove-uf-app.md](docs/add-or-remove-uf-app.md): `MainAppRouteChildren` + Cargo features
- [docs/where-to-put-domain-code.md](docs/where-to-put-domain-code.md): UI routes, workers, Valence models vs host glue

## API docs (rustdoc)

Scored host packages teach customize surfaces in rustdoc (purpose, when to edit,
examples, env). On a **generated** tree:

```bash
cargo doc -p server -p app -p frontend -p photon-uf-embedded \
  -p chronon-uf-embedded -p boson-uf-embedded -p spectra-uf-embedded \
  -p {{project-name}}-e2e --no-deps --open
```

Start at `server` (`compose_embedded_platform`, `build_router`, `listen_addr`) and
`app` (`MainAppRouteChildren`). Leave `frontend::hydrate` and stock `app::shell` /
`App` alone unless you intend to change the WASM entry or document chrome.

## Workspace layout

| Area | Role |
|------|------|
| `server` | Host bootstrap, platform compose, HTTP layers |
| `app` | Route table customize surface (`MainAppRouteChildren`) |
| `frontend` | Stock WASM hydrate entry (rarely edited) |
| `*-uf-embedded` crates | In-process builders for jobs, queues, realtime, and telemetry |
| `{{project-name}}-e2e` | IsolatedLab composition scenarios |

## Logging

The host binary uses `log` + `simple_logger` (Info by default). Soliton and some
upstream crates may emit `tracing` events; this template does not install a
tracing subscriber. Prefer `log` macros in host and `*-uf-embedded` code so
messages show up under the binary logger.

## Verify

On a **generated** host:

```bash
CARGO_BUILD_JOBS=1 cargo check -p server --features server-embedded
cargo test -p {{project-name}}-e2e -- --test-threads=1
cargo test -p app --features ssr,server-embedded --test full_product_surface_hygiene

UF_E2E_SEED_TOKEN="$(openssl rand -hex 32)" \
VALENCE_SQLITE_PATH=":memory:" \
cargo leptos end-to-end --project {{project-name}} --split \
  --bin-features ssr,server-embedded,e2e-host-conformance \
  --lib-features hydrate
```

Playwright details: [end2end/README.md](end2end/README.md). Maintainer checklist:
[docs/VERIFICATION.md](docs/VERIFICATION.md). Upstream template CI generates a
scratch host first (`scripts/verify-template-generate.sh`).

## License

MIT. See [LICENSE](LICENSE), [CONTRIBUTING.md](CONTRIBUTING.md), [SECURITY.md](SECURITY.md),
and [CODE_OF_CONDUCT.md](CODE_OF_CONDUCT.md).
