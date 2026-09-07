# {{project-name}}-e2e

In-process **IsolatedLab** correctness scenarios for the embedded host composition
(Spectra → Valence → Chronon → Boson → Photon → Higgs). No UI / Playwright.

## Run

```bash
export CARGO_BUILD_JOBS=1
cargo test -p {{project-name}}-e2e -- --test-threads=1
```

## Scenario catalog

| Scenario ID | Kind | Asserts |
|-------------|------|---------|
| `host-composition-boot-happy` | validating happy | All subsystem SQLite runtimes boot; Higgs exposes Chronon/Boson/Photon |
| `host-composition-photon-key-sad` | validating sad | Missing `PHOTON_TRANSPORT_KEY` fails closed before Higgs is built |

See [`docs/VERIFICATION.md`](../docs/VERIFICATION.md).
