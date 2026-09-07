# Embedded host conformance E2E

Playwright checks contracts owned by the full-product host: the frozen mounted
route inventory, redirects, 404 fallbacks, split-WASM hydration, browser
failures, and real Lepton session propagation across app families.

Product CRUD, queue, scheduler, permission, secret, and telemetry catalogs stay
in their product repositories. This suite is also separate from the Layer 2
`{{project-name}}-e2e` in-process composition gate.

## Run the full suite

Install the browser dependencies once:

```bash
cd end2end
npm ci
npx playwright install chromium
cd ..
```

Use an ephemeral bearer token and in-memory Valence database:

```bash
export UF_E2E_SEED_TOKEN="$(openssl rand -hex 32)"
export VALENCE_SQLITE_PATH=":memory:"

cargo leptos end-to-end --project {{project-name}} --split \
  --bin-features ssr,server-embedded,e2e-host-conformance \
  --lib-features hydrate
```

A zero exit status means the Rust host started, hydrated all canonical routes,
and passed the browser assertions.

## Focused runs

With the host already listening at `PLAYWRIGHT_BASE_URL` (default
`http://127.0.0.1:3000`):

```bash
cd end2end
npx playwright test route-smoke.spec.ts
npx playwright test platform-routes.spec.ts host-routing.spec.ts host-fallbacks.spec.ts
npx playwright test host-auth-session.spec.ts # requires UF_E2E_SEED_TOKEN
```

## Scenario groups

| File | TM / scenario focus |
|------|---------------------|
| `route-smoke.spec.ts` | TM-01 frozen inventory + TM-04 integrity probe |
| `platform-routes.spec.ts` | TM-02 platform ownership + Spectra anon gates |
| `host-routing.spec.ts` | TM-03 redirects and 404 (no Coming Soon) |
| `host-fallbacks.spec.ts` | TM-03 explicit / nested 404 |
| `host-auth-session.spec.ts` | TM-05 / TM-06 / TM-08 seed + session |
| `diagnostics-unit.spec.ts` | SM-04 query stripping |
| `photon-shell.spec.ts` | Photon nav + anonymous outlet denial |
| `counter-click-demo.spec.ts` | Host-mounted counter increment |
| `auth-menu-shell.spec.ts` | Anonymous app-bar sign-in controls |

The frozen route data is in `tests/support/routes.ts`.

## Fixture safety

`POST /api/test/seed-data` exists only when the server is compiled with
`e2e-host-conformance`. The server fails startup if `UF_E2E_SEED_TOKEN` is
missing or empty, and requests need an exact bearer token. Use synthetic
`@example.test` users only.

The seed response omits passwords, reset material, and TOTP data. Browser
diagnostics strip query strings and never record authorization headers.
CI uploads Playwright failure artifacts for seven days.
