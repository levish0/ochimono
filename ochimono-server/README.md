# ochimono-server

Rust game server workspace, licensed under AGPL-3.0-only by the repository root LICENSE.

## Structure

- `crates/server`: executable and library; HTTP handlers, services, repositories and application state.
- `crates/auth-core`: opaque tokens and Redis session storage.
- `crates/config`: typed process configuration.
- `crates/dto`: request/response types, validated extractors and OpenAPI schemas.
- `crates/errors`: domain error handlers and public error codes.
- `crates/entity`: SeaORM PostgreSQL models.
- `crates/migration`: SeaORM schema migrations and CLI.
- `xtask`: deterministic OpenAPI export and drift checking.

## Local development

Use Rust 1.98.1, Cargo, Docker Engine and just. Run commands from `ochimono-server`.
Copy `.env.example` to `.env`; the database values are disposable local development credentials.

```sh
just dev
just run
just openapi
just check
just e2e
```

`just dev` starts PostgreSQL and Redis and applies migrations. Authentication is disabled by default; health and Swagger remain available without infrastructure. Authentication endpoints return 503 while disabled.

- `GET /health/live`: process liveness, not database readiness.
- `GET /swagger.json`: generated HTTP contract.
- `/swagger-ui/`: interactive API documentation.

## Google authentication

Configure a Google OAuth web client, set `GOOGLE_CLIENT_ID`, `GOOGLE_CLIENT_SECRET`, `GOOGLE_REDIRECT_URI` and `BROWSER_ORIGIN`, then set `AUTH_ENABLED=true`. Register the exact redirect URI with Google. The redirect belongs to the UI, which submits its authorization code and state to the login endpoint. The example callback path is a suggested UI route; this workspace does not implement that page.

`BROWSER_ORIGIN` is an exact origin without a trailing slash. HTTPS is required except for loopback development, where the server must also bind to loopback. Production UI and API should share a site so SameSite=Lax cookies work; proxy `/v0` to the server or use same-site subdomains. Mutating browser requests must include the configured Origin; cross-origin fetches must use `credentials: 'include'`.

1. `POST /v0/auth/oauth/google/authorize` with `{ "remember_me": true }` returns `auth_url` and sets a browser binding cookie. Redirect the browser to that URL.
2. `POST /v0/auth/oauth/google/login` with `{ "code": "...", "state": "..." }` consumes the browser-bound state and verifies the Google identity using PKCE.
3. Existing accounts receive `status: "signed_in"`, a user and a session cookie. New identities receive `status: "pending_signup"` and a `pending_token`.
4. `POST /v0/auth/complete-signup` with the pending token and a unique handle creates the account. Handles contain 3–24 lowercase ASCII letters, digits or underscores. The pending proof expires after ten minutes and is single-use; a consumed proof requires restarting Google login, including after a conflicting signup.

Accounts are identified by Google's subject ID. Matching email addresses never merge accounts.

The HttpOnly session cookie contains an opaque random token; Redis stores only its hash. HTTPS uses the `__Host-ochimono_session` cookie name. Sessions have a seven-day sliding expiry and a thirty-day absolute lifetime. `remember_me` controls whether the cookie survives browser restarts.

- `GET /v0/auth/me`: current user.
- `POST /v0/auth/refresh`: extend the current session without changing its token or absolute lifetime.
- `POST /v0/auth/logout`: revoke the current session and clear its cookie.
- `GET /v0/auth/sessions`: list active sessions using public management IDs.
- `DELETE /v0/auth/sessions/{id}`: revoke an owned session.
- `DELETE /v0/auth/sessions`: revoke all sessions for the current account.

Revocation is checked through Redis and the PostgreSQL account generation. Session renewal uses an atomic comparison so it cannot recreate a revoked session. Authentication requests are limited to 120 per minute per transport peer; forwarding headers are not trusted. A reverse proxy therefore shares that limit unless a trusted-proxy policy is implemented.

## Validation

`just check` runs formatting, Clippy, unit tests and OpenAPI drift checking. `just e2e` runs the TCP HTTP contract and shutdown test.

For integration tests, provide `TEST_DATABASE_URL` pointing to a migrated disposable PostgreSQL database and `SESSION_REDIS_URL` pointing to disposable Redis, then run `just auth-test`. Apply migrations with `DATABASE_URL` targeting that test database. Redis tests exercise expiry, generation fencing, revocation, renewal and read-only access. The server test enters at the verified Google identity boundary and exercises signup and cookie/session behavior over HTTP; it does not perform a live Google login. The server test CI provisions both services and runs these tests explicitly.

## Infrastructure and contracts

`just up` starts the containerized HTTP process with authentication disabled. Use `just run` with the local environment for authentication development. Compose binds service ports to loopback and is not a production deployment.

`just down` retains data. `just prune` removes development volumes; `just migrate-fresh` drops and rebuilds the schema. `just publish <registry/image:tag>` builds and pushes the supplied image.

Generate `swagger.json` with `just openapi`; do not edit it manually. CI checks the generated contract against the committed artifact.

Rooms, WebSocket messages, matchmaking and game execution are not implemented yet. They will use the shared Rust engine and a separate WebSocket protocol contract.
