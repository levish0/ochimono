# ochimono-server

Single-region Rust workspace, licensed under AGPL-3.0-only by the repository root LICENSE.

## Structure

- `crates/server`: executable and library; `api`, `service`, `repository`, `state` stay inside this crate.
- `crates/config`: typed process configuration.
- `crates/dto`: HTTP request/response types and OpenAPI schemas.
- `crates/errors`: HTTP error responses.
- `crates/entity`: location for PostgreSQL entities (no models yet).
- `crates/migration`: SeaORM migration runner (no domain migrations yet).
- `xtask`: deterministic OpenAPI export and drift checking.

The sibling engine and UI retain their current structure. Authentication primitives and a shared WebSocket protocol will be added with their implementations. API is not a separate crate.

## Local development

Use Rust 1.98.1, Cargo and just. Run commands from `ochimono-server`.
Copy `.env.example` to `.env` for local database commands; it contains disposable development credentials only.

```sh
just run
just openapi
just check
just e2e
```

- `GET /health/live`: process liveness only.
- `GET /swagger.json`: generated HTTP contract, identical to the committed artifact.
- `/swagger-ui/`: interactive API documentation.

The HTTP scaffold does not connect to PostgreSQL or Redis. Liveness is not a database readiness check. Unknown routes return a JSON 404. Google login, WebSocket rooms, matchmaking and game execution are not implemented or exposed.

## Infrastructure

Docker Engine is required for `just dev`, `just up` and `just build`.
`just dev` starts PostgreSQL and Redis and runs the migration CLI. The migration list is currently empty; this does not create account/game tables. `just up` also starts the containerized HTTP server. Local ports bind to loopback; this Compose file is not a production deployment.

`just down` retains data. `just prune` removes development volumes; `just migrate-fresh` drops and rebuilds the schema. `just publish <registry/image:tag>` explicitly builds and pushes the supplied image.

`just e2e` runs an actual TCP HTTP contract/shutdown test without Docker. Database-backed authentication and game E2E coverage must be added as those features are implemented.

## Contracts and next implementation boundaries

Generate `swagger.json` with `just openapi`; do not edit it manually. CI uses `cargo xtask openapi --check`, which compares against freshly generated content without modifying the artifact. WebSocket message schemas will be generated separately from Rust protocol types.

Google-only authentication belongs in the server service/repository modules, with reusable token/session code in an auth-core crate. Room and match state need explicit ownership, bounded input/output queues, reconnect policy and idempotent result persistence. They must use the shared Rust engine before official game results are supported.

Heavy replay/media processing should execute outside the match process. No multi-region replication or messaging infrastructure is included.

## Scaffold validation (2026-09-08)

Locally passed: workspace Clippy with warnings denied, workspace tests (including one TCP HTTP contract/shutdown test), rustfmt, OpenAPI export and stale-artifact rejection, just recipe parsing, and Compose configuration validation.

Docker Desktop's Linux engine was unavailable, so container builds, PostgreSQL/Redis startup and migration execution against PostgreSQL have not been verified locally. The updated CI workflows have not been run remotely. No images have been published or deployed.
