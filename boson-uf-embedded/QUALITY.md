# boson-wiring-server Quality Gates

Last updated: 2026-07-02

## quality review baseline

- `cargo` fmt/clippy/test/doc gates

## Local commands

```bash
export CARGO_BUILD_JOBS=1 CARGO_TARGET_DIR=target-boson-extract
cargo check -p boson-wiring-server --features photon-events,remote-http
```
