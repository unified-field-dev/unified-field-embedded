# Test Map — Higgs session Valence adoption (embedded host)

> **Maintainers only.** Strangers should use [VERIFICATION.md](VERIFICATION.md) and
> the guides linked from the root README. This map is an internal audit artifact.

Blast radius: L1 higgs + L4 embedded host + patched kits on the `server-embedded` graph.
AWS campaign: **not required** (correctness / API gating; no new campaign scenarios).
Bench / PERFORMANCE_STUDY: **not required** (correctness-only; no claimed hot-path change).

| Behavior / invariant | Risk if untested | Unit | Integration | E2E / IsolatedLab | AWS | Bench | Not testing |
|----------------------|------------------|------|-------------|-------------------|-----|-------|-------------|
| Hosted SSR uses `Higgs::valence()` after session; no `system_valence` / `unsafe_system_valence` on interactive paths | Privacy bypass / IDOR via System actor | Grep-gate CI note; higgs `valence` happy path already in higgs-core | `cargo check -p server --features server-embedded`; lepton-appearance SSR check | Host smoke `CARGO_BUILD_JOBS=1 cargo leptos watch --split` (smoke only) | waived | waived | Manual click-through as primary coverage |
| Unauthenticated appearance get/save fails closed | Appearance read/write without login | — | lepton-appearance: require_session / missing session → `ServerFnError` | Deferred IsolatedLab appearance sad | waived | waived | Full browser e2e this pass |
| Authenticated owner can create/update own `UserAppearance` via session Valence | Lazy-create appearance broken after dropping System | Schema policy: create allows `OWNER_BY_USER_FIELD` | Appearance service / lepton-appearance SSR create path | Deferred happy IsolatedLab | waived | waived | Cross-user appearance IDOR suite (policy unit elsewhere) |
| `RejectExternalSystemActor` on embedded factory rejects external System actor JSON | Worker/control-plane mint of System from untrusted JSON | higgs-core `external_actor_json_policy_*` | Embedded factory build with System actor_json → reject | — | waived | waived | Photon/Boson live replay |
| Soft-named `Higgs::system_valence` deleted | Callers silently keep soft API | Compile-fail of any remaining `.system_valence()` on Higgs | Workspace check after deletion | — | waived | waived | Semver publish matrix |
| `unsafe_system_valence` documented prefer-not; L1-only wording | Integrators keep elevating | rustdoc / SECURITY review | `cargo doc -p higgs --features ssr` | — | waived | waived | Out-of-tree product tutorials |
| Gauge permission resolve by name uses session Valence | Name→id lookup elevates | — | gauge `resolve_permission_id_by_name` session + AUTHENTICATED read | — | waived | waived | Full GaugeAdmin UI matrix |
| Gauge service no longer mints System via `as_system_valence` for interactive ops; policies allow AUTHENTICATED where server-fn authz applies | Permission CRUD fails or stays elevated | Policy schema compile | gauge service paths with session actor | — | waived | waived | Nested vendored gauge copies under other products |

## Smoke vs validation

| Artifact | Kind |
|----------|------|
| `CARGO_BUILD_JOBS=1 cargo leptos watch --split` | **Smoke** — graph builds and listens |
| Higgs / kit unit + integration tests named in rows above | **Validation** |
| Grep gate zero hosted `.system_valence(` / `unsafe_system_valence` | **Validation** (absence contract) |
