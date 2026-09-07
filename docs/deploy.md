# Deploy / ops knobs

Run this host behind your process manager or container entrypoint. Knobs are environment variables plus the listen address. Keep SQLite files on durable disk. Local first boot and `--split` rules live in [Run locally](run-locally.md).

## Build and start

`[workspace.metadata.leptos]` sets `bin-package = "server"` and `site-root = "target/site"`. Build assets and the server binary together:

```bash
export PHOTON_TRANSPORT_KEY=<base64-32-bytes>
export UF_PUBLIC_BASE_URL=https://app.example.com
# optional: SITE_ADDR, VALENCE_SQLITE_PATH, other paths from below
cargo leptos build --release --split
```

Then either:

```bash
cargo leptos serve --release --split
```

or run the `server` binary your process manager owns, with the same env and with site files available under `target/site` (pkg, CSS, favicon) the way a leptos release layout expects. Plain `cargo run -p server --release --features ssr,server-embedded` without a prior leptos build will boot HTTP but will not have the hydrate WASM/pkg tree unless you copy or point at a built `target/site`.

## Listen address

`server::listen_addr` order: `LEPTOS_SITE_ADDR` → `SITE_ADDR` → `127.0.0.1:3000`.

| Goal | Setting |
|------|---------|
| Loopback only | Leave both unset (or set `SITE_ADDR=127.0.0.1:3000`) |
| LAN / container bind | `SITE_ADDR=0.0.0.0:3000` (or another port) |

`[workspace.metadata.leptos] site-addr` feeds `cargo-leptos` only. Production entrypoints that call `server::run` without leptos should set `SITE_ADDR` explicitly.

## Public base URL

```bash
export UF_PUBLIC_BASE_URL=https://app.example.com
```

Used for OAuth callback construction, WebAuthn RP origin, and the Photon WebSocket Origin allowlist seed. Default when unset: `http://127.0.0.1:3000`.

## Session cookie Secure

`session_cookie_secure` turns Secure on when the public base starts with `https://`. Override:

| `UF_SESSION_SECURE` | Effect |
|---------------------|--------|
| `1` / `true` | Always Secure (TLS terminated at a proxy while the app still sees `http://`) |
| `0` / `false` | Always off (local Secure debugging) |

Cookie name `session`, path `/`, SameSite `Lax`. Sessions persist as JSON files
under `UF_SESSION_DIR` (default `data/sessions`, mode `0700` / files `0600`) with
a 60s expired-row sweeper and 24h inactivity expiry.

### CSRF and rate limits

Mutating requests (`POST` / `PUT` / `PATCH` / `DELETE`) are checked against
`UF_PUBLIC_BASE_URL` and `PHOTON_LEPTOS_ALLOWED_ORIGINS` (same allowlist seed as
Photon WS). Auth-ish paths without a matching Origin or Referer return **403**.
Foreign Origins return **403**. Per-IP limits return **429** (20 auth-ish POSTs
per minute, 120 other POSTs). Concurrent `/ws*` upgrades cap at **64** (**503**
when full).

### e2e seed feature

Never enable `e2e-host-conformance` on a release binary. The crate
`compile_error!`s when that feature is on without `debug_assertions`.

## SQLite paths

Point each store at a durable volume. Defaults live under `data/`; see the table in [Run locally](run-locally.md). File-backed Valence, Chronon, Boson, Photon, and Spectra SQLite files are `chmod 0600` after connect (WAL/SHM sidecars when present).

## Photon transport key

```bash
export PHOTON_TRANSPORT_KEY=<base64-32-bytes>
```

Required. Boot fails if missing or invalid. Do not set `PHOTON_ALLOW_DEV_TRANSPORT_KEY` outside local experiments.

## WebSocket Origin allowlist

Built from `UF_PUBLIC_BASE_URL`, plus loopback `:3000` origins for local leptos, plus comma-separated extras:

```bash
export PHOTON_LEPTOS_ALLOWED_ORIGINS=https://preview.example.com,https://staging.example.com
```

Origins that are absent from the list are rejected on Photon WS connect.

## What not to expose

- Keep `PHOTON_TRANSPORT_KEY` and OAuth client secrets out of logs, images, and public env dumps.
- Prefer loopback bind until you intentionally open LAN or a reverse proxy.
- Do not ship `PHOTON_ALLOW_DEV_TRANSPORT_KEY=1` or the development transport key into production.
- Never set `VALENCE_PRIVACY_BYPASS` or `VALENCE_PRIVACY_BYPASS_FORCE_ON` in production (bench/test only).
- SQLite files under `data/` hold auth and secrets logicals; treat them as credential material (mode `0600` after connect on Valence and the `*-uf-embedded` stores).

## Related guides

- [Swap or disable a runtime](swap-or-disable-runtime.md)
- [Data & Valence bootstrap](data-and-valence-bootstrap.md)
- [Auth / session surface](auth-and-session.md)
