# Security Policy

## Supported versions

Security fixes are accepted against the latest published `0.1.x` line of this repository's host wiring crates (`*-uf-embedded`, shell packages, and related adapters in this workspace).

## Reporting a vulnerability

Please **do not** open a public GitHub issue for security-sensitive reports.

Prefer one of the following:

1. **GitHub Security Advisories** — use [Report a vulnerability](https://github.com/{{gh_owner}}/{{gh_repo}}/security/advisories/new) on this repository when available.
2. Contact the maintainers privately via the repository owner listed at https://github.com/{{gh_owner}}/{{gh_repo}}.

Include:

- a description of the issue and its impact
- steps to reproduce or a proof of concept when possible
- affected crate names and versions

We will acknowledge receipt as soon as practical and coordinate a fix and disclosure timeline with you.

## Scope

In scope: vulnerabilities in this repository's host wiring, shell, and documentation that could cause unsafe production defaults, plus CI/supply-chain issues in this repository.

Out of scope: vulnerabilities solely in third-party dependencies or upstream family crates unless this template mishandles them in a security-relevant way.

## Host defaults (embedded)

| Control | Default / requirement |
|---------|------------------------|
| Listen address | Binary [`listen_addr`] defaults to `127.0.0.1:3000` unless `SITE_ADDR` / `LEPTOS_SITE_ADDR` is set. Leptos metadata / `scripts/serve-local.sh` may bind `0.0.0.0:3000` for LAN previews — do not expose that on the public internet without TLS and auth hardening. |
| SQLite file mode | `chmod 0600` on primary DB files after connect when path is not `:memory:`; WAL/SHM sidecars should match when present |
| Gauge | `gauge-app` **is mounted** (PermissionRoutes) in this starter; the Gauge library also boots for permission logicals |
| Dependency pins | Prefer `Cargo.lock` + `cargo build --locked` (CI clippy/test use `--locked`). UF git deps stay `branch = "main"` (fleet ship policy); the lockfile is the reproducible pin. Run `cargo deny check` / soft `cargo audit` in CI. |
| Session | Cookie Secure follows `UF_PUBLIC_BASE_URL` / `UF_SESSION_SECURE`; SameSite `Lax`; 24h inactivity expiry; file-backed store under `UF_SESSION_DIR` (default `data/sessions`) with a 60s expired-row sweeper. CSRF Origin/Referer checks on mutating requests. |
| Request limits | Body 2 MiB; concurrency 256; request timeout 60s; WebSocket cap 64; auth-ish POST rate limits per client IP. |
| Privacy bypass | Never set `VALENCE_PRIVACY_BYPASS` or `VALENCE_PRIVACY_BYPASS_FORCE_ON` in production (bench/test only). |
| e2e seed route | Feature `e2e-host-conformance` is debug-only; never enable on release hosts. |

## Configure host hardening (first success)

Set a public base and session directory, then start the host so CSRF, Secure
cookies, and file sessions share one Origin story:

```bash
export UF_PUBLIC_BASE_URL=https://app.example.com
export UF_SESSION_DIR=data/sessions
export PHOTON_TRANSPORT_KEY=<base64-32-bytes>
# optional extras for CSRF / Photon WS:
# export PHOTON_LEPTOS_ALLOWED_ORIGINS=https://preview.example.com
cargo leptos serve --release --split
```

Observable outcomes: session JSON files appear under `UF_SESSION_DIR` after
login; a cross-site mutating POST with a foreign `Origin` returns **403**;
auth-ish POST bursts return **429**. Details and status table:
`server` crate module `request_hardening`. Deploy knobs: [docs/deploy.md](docs/deploy.md).

### HTTPS cookie variant

When TLS terminates at a proxy and the app still sees `http://`, set
`UF_SESSION_SECURE=1` so the session cookie keeps the Secure flag. For local
HTTP debugging only, `UF_SESSION_SECURE=0` forces Secure off.

### Intentional vs fixed surfaces

| Surface | Posture |
|---------|---------|
| AccountEmail addresses | Owner (via Account) + System only; principal search uses display name + short id |
| Chronon/Boson runtime params and run history | ChrononAdmin / BosonAdmin |
| Schema / privacy-policy catalog | Open (compile-time public schemas) |
| `/orbital` component preview | Mounted; search fixture uses mock data |
| ProfilePhoto HTTP serve | No System fallback on denial; entity read still `AUTHENTICATED` |
| Platform System Chronon jobs | Interactive `run_now` refused (schedule-only) |
