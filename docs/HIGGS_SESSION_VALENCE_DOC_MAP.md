# Documentation Map — Higgs session Valence adoption

> **Superseded for strangers.** Prefer [auth-and-session.md](auth-and-session.md)
> and [data-and-valence-bootstrap.md](data-and-valence-bootstrap.md), plus Higgs /
> `server` rustdoc. This table remains a historical planning artifact.

| Behavior / contract | Discovery path | Compose into | Root / module updates | Item rustdoc | Example ladder | Out of scope | Lint / gate |
|---------------------|----------------|--------------|----------------------|--------------|----------------|--------------|-------------|
| Prefer `Higgs::valence()` + `require_session` / `server(auth)` | `higgs` crate root → SECURITY / context module → `Higgs::valence` | higgs crate-root security notes; SECURITY.md host checklist | Short “typical flow” + link | `# Errors` on valence/require_session | Highlight: auth + valence; Mid: anonymous vs session; Detailed: existing examples | L2/L3 product guides | `missing_docs`; `cargo doc -p higgs` |
| Soft name `system_valence` removed (breaking) | crate root security notes → migration one-liner | SECURITY.md + crate root | “Use `valence`; elevated API is `unsafe_system_valence` only” | N/A (deleted) | — | Changelog outside rustdoc OK in SECURITY | doc links retargeted |
| `unsafe_system_valence` escape hatch — do not use for user CRUD | crate root → `Higgs::unsafe_system_valence` | SECURITY.md Integrator table | Prefer valence; elevation bypasses privacy | Contract + `# Errors`; why-not prose | Mid: show prefer valence vs escape hatch (no product names) | Gauge/fleet/product names | L1 dep-tree only |
| `RejectExternalSystemActor` on shared factory | `higgs::actor_policy` module → SECURITY checklist | actor_policy module docs; SECURITY host checklist | Install on factories rebuilding from external actor JSON | Policy validate examples | Highlight in actor_policy | Host product wiring beyond embedded | — |
| Embedded host factory installs policy | `server` process_valence_factory module docs | embedded README “Run the host” / composed mount if needed | One sentence + link to higgs actor_policy | — | — | Fleet template | — |

## Prerequisite doc repair

Higgs `security/higgs-hardening` already has SECURITY + crate-root notes; strengthen `unsafe_*` wording and scrub out-of-tree product names as part of `document-unsafe`.
