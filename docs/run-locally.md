# Run locally

Clone this repo, boot the full-product host on loopback, and open the UI.

## Prerequisites

- Rust stable (and the toolchain `cargo-leptos` expects for this workspace)
- [`cargo-leptos`](https://github.com/leptos-rs/cargo-leptos) installed
- Writable `data/` under the workspace root (created on first boot for SQLite files)

## Boot

```bash
# DEV ONLY — generate a fresh key (do not commit real secrets):
#   openssl rand -hex 32   # then base64-encode 32 raw bytes for Photon
#   openssl rand -base64 32
export PHOTON_TRANSPORT_KEY=CHANGE_ME
cargo leptos watch --split
# or: cargo leptos serve --split
```

`--split` is required. App families load as lazy WASM chunks; omit it and hydrate fails.

Production-shaped local build:

```bash
cargo leptos build --release --split
```

Binary features (set by `[workspace.metadata.leptos]`): `ssr` + `server-embedded` for the server package, `hydrate` for the frontend WASM.

On a constrained machine:

```bash
CARGO_BUILD_JOBS=1 cargo check -p server --features server-embedded
```

## URL and listen address

Open `http://127.0.0.1:3000` (or `http://localhost:3000`).

`server::listen_addr` resolves `LEPTOS_SITE_ADDR`, then `SITE_ADDR`, then defaults to `127.0.0.1:3000`. `cargo leptos` injects `LEPTOS_SITE_ADDR` from `[workspace.metadata.leptos] site-addr` (`0.0.0.0:3000` in this template), so a leptos watch session may bind all interfaces. Plain `cargo run -p server` without those env vars stays on loopback.

Health probe: `GET /health` (outside the session auth layer).

## SQLite defaults

| Store | Env | Default path |
|-------|-----|--------------|
| Valence | `VALENCE_SQLITE_PATH` | `data/{{project-name}}.sqlite3` |
| Chronon | `CHRONON_SQLITE_PATH` | `data/chronon.sqlite3` |
| Boson | `BOSON_SQLITE_PATH` | `data/boson.sqlite3` |
| Photon | `PHOTON_SQLITE_PATH` | `data/photon.sqlite3` |
| Spectra metrics | `SPECTRA_METRICS_SQLITE_PATH` | `data/spectra-metrics.sqlite3` |
| Spectra events | `SPECTRA_EVENTS_SQLITE_PATH` | `data/spectra-events.sqlite3` |

Ephemeral Valence: `VALENCE_SQLITE_PATH=:memory:`.

Photon boot fails closed without `PHOTON_TRANSPORT_KEY` (base64 of 32 bytes). Use a
DEV-only placeholder locally (`CHANGE_ME` until you generate one with
`openssl rand -base64 32` or `openssl rand -hex 32` + base64). Generate a unique key
for any shared or long-lived environment.

## Optional OAuth on local

Set `UF_PUBLIC_BASE_URL` (default `http://127.0.0.1:3000`) and either `UF_OAUTH_USE_MOCK=1` or provider client ids. Secrets land in Neutrino via `uf-oauth-boot`; see [Auth / session surface](auth-and-session.md).

## Next

- Release build, process entry, bind/URL/Secure/Photon: [Deploy / ops knobs](deploy.md)
- Trim or add product routes: [Add or remove a uf-app](add-or-remove-uf-app.md)
- Domain UI vs workers: [Where to put your domain code](where-to-put-domain-code.md)
