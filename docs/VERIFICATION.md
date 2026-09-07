# {{project-name}} verification

Re-run after code or wiring changes on a **generated** host (Liquid template
source does not build in place). Upstream template maintainers: run
`scripts/verify-template-generate.sh` first, then the gates below inside the
scratch tree. Marketing and Coming Soon live on `unified-field-site`.

Playwright under `end2end/` is **UI** smoke and is **not** the Layer 2 gate
(audit requires backend IsolatedLab / in-process e2e only).

## Environment

```bash
export CARGO_BUILD_JOBS=1
export PHOTON_ALLOW_DEV_TRANSPORT_KEY=1
export PHOTON_TRANSPORT_KEY=AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA=
```

## Layer 1 — Unit + integration (CI)

```bash
cargo fmt --all --check
cargo test -p photon-uf-embedded -p boson-uf-embedded -p chronon-uf-embedded \
  -p spectra-uf-embedded
cargo test -p server --features server-embedded --lib --test listen_addr_contract
cargo test -p server --features server-embedded,e2e-host-conformance --lib e2e_support
cargo test -p app --features ssr,server-embedded --test full_product_surface_hygiene
cargo clippy -p photon-uf-embedded -p boson-uf-embedded -p chronon-uf-embedded \
  -p spectra-uf-embedded -p {{project-name}}-e2e \
  --all-targets -- -D warnings
cargo clippy -p server --features server-embedded --all-targets -- -D warnings
```

### TEST_MAP

| Behavior | Level | Happy | Sad | Notes |
|----------|-------|-------|-----|-------|
| `photon_uf_embedded::build_runtime` | unit + integ | temp SQLite + transport key | parent-not-dir; missing `PHOTON_TRANSPORT_KEY` | `tests/runtime_contract.rs` |
| `boson_uf_embedded::build_boson_runtime` | unit + integ | temp SQLite | parent-not-dir | `tests/runtime_contract.rs` |
| `chronon_uf_embedded::build_chronon_runtime` | unit + integ | temp SQLite + shutdown | parent-not-dir | `tests/runtime_contract.rs` |
| `spectra_uf_embedded::install_embedded_sqlite` | unit + integ | temp metrics/events paths | parent-not-dir | `tests/runtime_contract.rs` |
| `server::listen_addr` | unit + integ | default / valid `SITE_ADDR` | garbage env → `Err` | no bind |
| `e2e_support` seed token gate | unit | non-empty token | missing/empty token; non-Bearer header | feature `e2e-host-conformance` only |
| Seed feature compiled out | unit | default `server-embedded` build | — | `e2e_host_conformance_feature_absent_by_default` |
| Full-product surface hygiene | unit | platform apps present; no `full`/`server-full`/marketing/Coming Soon | missing platform deps | `app/tests/full_product_surface_hygiene.rs` |
| `server::platform::compose_embedded_platform` | e2e (IsolatedLab) | see Layer 2 | see Layer 2 | same boot order as `build_router` |
| Host HTTP / Leptos shell | e2e | see Playwright | see Playwright | host conformance; not Layer 2 |
| Photon ops shell mount | UI smoke | `e2e.l5.photon_shell_smoke` (`/photon` root + nav; anon auth gate) | — | Full ops catalog: `photon-uf-app-e2e` |
| AWS / soak | AWS | — | — | **waived** |
| Micro-benchmarks | bench | — | — | **waived** |

## Layer 2 — E2E IsolatedLab (CI)

```bash
cargo test -p {{project-name}}-e2e -- --test-threads=1
```

| Scenario ID | Kind | Asserts |
|-------------|------|---------|
| `host-composition-boot-happy` | validating happy | Spectra + Valence + Chronon + Boson + Photon boot; Higgs exposes subsystems |
| `host-composition-photon-key-sad` | validating sad | Missing `PHOTON_TRANSPORT_KEY` fails closed |

Harness: [`{{project-name}}-e2e`](../{{project-name}}-e2e/README.md).

## Playwright host conformance (CI)

This browser gate verifies host-owned route composition, hydration, fallbacks,
and real session propagation. It does not replace IsolatedLab or product-owned
Playwright catalogs.

```bash
export UF_E2E_SEED_TOKEN="$(openssl rand -hex 32)"
export VALENCE_SQLITE_PATH=":memory:"

cargo leptos end-to-end --project {{project-name}} --split \
  --bin-features ssr,server-embedded,e2e-host-conformance \
  --lib-features hydrate
```

See [`end2end/README.md`](../end2end/README.md) for focused scenarios and the
fixture safety contract.

## PR CI parity

Match `.github/workflows/ci.yml` on a generated host (or upstream
`.github/workflows/template-ci.yml`, which generates first): `fmt`, `clippy`
(adapters + `{{project-name}}-e2e` +
server), Layer 1 + Layer 2 under `test`, and the `e2e` Playwright job with
`UF_E2E_SEED_TOKEN` + `VALENCE_SQLITE_PATH=:memory:`.

## Notes

- Prefer `*_happy_path` / `*_sad` test names for audit detection.
- Do not treat `end2end/` Playwright as a substitute for `{{project-name}}-e2e`.
