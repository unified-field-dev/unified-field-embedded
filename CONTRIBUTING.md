# Contributing to {{app_title}}

Thank you for improving this host template.

## Development setup

1. Clone [{{gh_owner}}/{{gh_repo}}](https://github.com/{{gh_owner}}/{{gh_repo}})
2. Install Rust stable and Leptos tooling as needed
3. Prefer checking one package at a time:

```bash
cargo check -p photon-uf-embedded --features ssr
cargo check -p boson-uf-embedded
cargo check -p chronon-uf-embedded
cargo check -p spectra-uf-embedded
cargo check -p server --features server-embedded
```

## Code of conduct

Participation is governed by [`CODE_OF_CONDUCT.md`](CODE_OF_CONDUCT.md). Security reports: [`SECURITY.md`](SECURITY.md).

## Pull requests

- Prefer small, focused PRs.
- Update [`README.md`](README.md) when host wiring or mount steps change.
- Re-run the maintainer checklist in [`docs/VERIFICATION.md`](docs/VERIFICATION.md) after wiring changes.

## Template maintainers

This repository is a Liquid / cargo-generate source tree. It does not build in
place. Use `scripts/verify-template-generate.sh` (or the `template-ci` workflow)
to generate a scratch host before fmt/clippy/test.
