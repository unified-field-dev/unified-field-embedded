# Auth / session surface

Session cookies, Photon WebSocket Origin checks, and lepton-auth routes are mounted by the host. OAuth client secrets resolve through Neutrino sealed storage at boot.

## Session cookie

`build_router` installs `tower-sessions` + `axum-login` with:

- Cookie name `session`, path `/`, SameSite `Lax`
- Secure flag from `session_cookie_secure(UF_PUBLIC_BASE_URL)` (HTTPS → on; HTTP → off)
- Override: `UF_SESSION_SECURE=0|1` / `true|false`

Auth backend is `Backend::new` over the Higgs Valence factory (session actor, privacy policies intact). Prefer `Higgs::valence()` / `require_session` / `server(auth)` in product handlers. Do not mint System actors mid-request to bypass privacy; see Higgs crate SECURITY notes and `ProcessValenceFactory` (external factory rejects System-shaped actor JSON).

## Public base URL

```bash
export UF_PUBLIC_BASE_URL=https://app.example.com
```

Default `http://127.0.0.1:3000`. Feeds OAuth redirects, WebAuthn RP id/origin, and the WS allowlist seed.

## WebSocket Origin allowlist

`ws_origin_allowlist` includes:

1. Trimmed `UF_PUBLIC_BASE_URL`
2. Loopback `:3000` origins for local leptos
3. Extra origins from `PHOTON_LEPTOS_ALLOWED_ORIGINS` (comma-separated)

Photon WS rejects origins outside that list.

## Lepton-auth routes

`LeptonAuthRoutes` is already a child of `MainAppRouteChildren` (signup / login / account flows). Leave it mounted unless you replace the auth UI entirely.

## OAuth secrets

1. Enable provider features on `lepton-auth-app` / `lepton-app` for both `hydrate` and `ssr` (stock template already enables Google and GitHub OAuth features).
2. Set `UF_PUBLIC_BASE_URL` and either `UF_OAUTH_USE_MOCK=1` or provider client ids.
3. Client **secrets** resolve through `uf-oauth-boot` from Neutrino sealed secrets (`store_from_valence`). Env `UF_OAUTH_*_CLIENT_SECRET` seeds the vault on first boot.
4. Callback path: `/auth/oauth/callback`.

Do not commit secrets. Prefer Neutrino / env injection at deploy time ([Deploy / ops knobs](deploy.md)).

## WebAuthn

With `server-embedded`, boot configures WebAuthn RP from the public base host (fallback RP id `127.0.0.1`). Align `UF_PUBLIC_BASE_URL` with the browser origin users actually open.

## Supersedes

Earlier planning notes in [HIGGS_SESSION_VALENCE_DOC_MAP.md](HIGGS_SESSION_VALENCE_DOC_MAP.md) are historical. Prefer this guide plus Higgs / `server` rustdoc for session Valence contracts.

## Related

- [Data & Valence bootstrap](data-and-valence-bootstrap.md)
- [Run locally](run-locally.md)
