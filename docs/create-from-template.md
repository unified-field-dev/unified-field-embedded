# Create a host from this template

This repository is a **cargo-generate** template (and a GitHub template repo).
The source tree uses Liquid placeholders such as `{{project-name}}`. It does
**not** build until you generate a host.

## Primary path: cargo-generate

```bash
cargo install cargo-generate
cargo generate --git https://github.com/unified-field-dev/unified-field-embedded --allow-commands
# prompts: project-name, app_title, gh_owner, gh_repo (gh_repo defaults to project-name)
# --allow-commands lets the post-hook install .github/workflows/ci.yml
cd my-uf-app   # or the project-name you chose
export PHOTON_TRANSPORT_KEY=cGhvdG9uLWRldi10cmFuc3BvcnQta2V5LTMyYnl0ZXM=
cargo leptos watch --split
```

Open `http://127.0.0.1:3000`. The browser document title should match `app_title`.

Non-interactive defaults (CI / scripts):

```bash
cargo generate --git https://github.com/unified-field-dev/unified-field-embedded \
  --name my-uf-app \
  --define app_title="My App" \
  --define gh_owner=unified-field-dev \
  --define gh_repo=my-uf-app \
  --allow-commands
```

## GitHub "Use this template"

GitHub copies files as-is. It does **not** substitute Liquid variables. Prefer
`cargo generate`. If you already used the GitHub button, run cargo-generate from
the template git URL into a fresh directory instead of hand-editing every
Liquid marker.

## After generate

| Knob | Where it landed |
|------|-----------------|
| Kebab project name | Leptos `[workspace.metadata.leptos] name`, `/pkg/*.js`, default Valence DB `data/<name>.sqlite3`, IsolatedLab crate `<name>-e2e` |
| Display title | HTML `title`, WebAuthn `rp_name`, README H1 |
| GitHub URLs | CONTRIBUTING / SECURITY / CoC |

Trim mounted uf-apps in `app/src/main_app_routes.rs` (see
[add-or-remove-uf-app.md](add-or-remove-uf-app.md)). Deploy knobs stay env-based:
[run-locally.md](run-locally.md), [deploy.md](deploy.md).

### Hydrate 404 checklist

If the browser requests a missing `/pkg/*.js` file, confirm these three match:

1. `[[workspace.metadata.leptos]] name` in `Cargo.toml`
2. `LEPTOS_OUTPUT_NAME` / paths in `scripts/serve-local.sh`
3. `cargo leptos end-to-end --project <same-name>`

Always build with `--split` for lazy WASM routes.

## Verify a generated host

```bash
CARGO_BUILD_JOBS=1 cargo check -p server --features server-embedded
cargo test -p my-uf-app-e2e -- --test-threads=1   # use your project-name
```

Maintainer checklist: [VERIFICATION.md](VERIFICATION.md). Upstream template CI
generates a scratch host first; your generated repo uses a flat `.github/workflows/ci.yml`.
