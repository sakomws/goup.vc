# Patterns and conventions

## Project structure conventions

### One module per domain

Each feature domain has its own module in each layer:

- `ocg-server/src/handlers/<domain>.rs` — HTTP handlers
- `ocg-server/src/db/<domain>.rs` — database queries
- `ocg-server/src/types/<domain>.rs` — domain types (structs, enums)
- `ocg-server/src/services/<domain>.rs` — background services

When a module grows large it is promoted to a directory (e.g., `handlers/site/`, `services/meetings/`).

### Trait objects for infrastructure

Infrastructure dependencies (database, image storage, notifications, payments, meetings) are accessed through dynamic trait objects rather than concrete types. This pattern allows test doubles and makes dependencies explicit.

```rust
// In db.rs
pub(crate) type DynDB = Arc<dyn DB + Send + Sync>;

// In router.rs State
pub db: DynDB,
pub image_storage: DynImageStorage,
pub notifications_manager: DynNotificationsManager,
```

New infrastructure integrations should follow the same pattern: define a trait, implement it on a concrete struct, expose a `Dyn*` type alias.

## Configuration

### Adding a new config key

1. Add a field to the appropriate struct in `ocg-server/src/config.rs`.
2. If the field is required, add validation in the struct's `validate()` method.
3. If it is optional and gated behind a feature, wrap it in `Option<T>`.
4. Document the corresponding `OCG_*` env var name in the field's doc comment. Nested structs use `__` as separator (e.g., `OCG_MEETINGS__ZOOM__ENABLED`).

Config is loaded in `Config::new()` using Figment — no registration step is needed.

### Environment variable naming

All env vars use the `OCG_` prefix. Nested config keys map to `__`-separated env var names. Examples:

| Config path | Env var |
|-------------|---------|
| `server.addr` | `OCG_SERVER__ADDR` |
| `meetings.zoom.enabled` | `OCG_MEETINGS__ZOOM__ENABLED` |
| `integrations.you_com.api_key` | `OCG_INTEGRATIONS__YOU_COM__API_KEY` |

## HTTP handlers

### Handler signature

Handlers are plain async functions. Dependencies are extracted from the axum `State` using `FromRef`. Do not accept the full `State` struct unless necessary — extract only what the handler needs:

```rust
pub(crate) async fn page(
    AxumState(db): AxumState<DynDB>,
    AxumState(server_cfg): AxumState<HttpServerConfig>,
    Path(alliance): Path<String>,
) -> impl IntoResponse {
    // ...
}
```

### HTMX fragments vs full pages

Check the `hx-request` or `x-ocg-fetch` header to decide whether to render a partial fragment or a full HTML page. The `refresh_stale_clients` middleware in `ocg-server/src/router.rs` handles stale-client detection automatically — handlers do not need to inspect the commit SHA header themselves.

### Input validation

Input structs are validated with `garde`. Annotate fields with `#[garde(...)]` rules and call `.validate()` before passing to the database layer. Common validators are centralised in `ocg-server/src/validation.rs`.

### Error handling

Handlers return `impl IntoResponse`. Use `StatusCode` for simple errors. For user-visible errors, render a template with an error message. Avoid `unwrap()` in handler code; use `?` or explicit error handling.

## Database layer

### Query placement

SQL queries belong in `ocg-server/src/db/<domain>.rs` as methods on `PgDB`. Do not embed SQL in handlers or services. Method names should describe what is returned or mutated (e.g., `get_group_by_slug`, `add_event_attendee`).

### Prepared statements

Use `deadpool-postgres` prepared statements via `client.prepare_cached()`. This avoids repeated parse-and-plan overhead on repeated queries.

### Contract tests

Database contract tests live in `ocg-server/src/db/<domain>.rs` under `#[cfg(test)]` blocks, gated with `#[ignore]` so they only run when explicitly invoked (`just db-contract-tests`). They require the contract test database (`ocg_tests_contract`) to be seeded with `database/tests/data/contract.sql`.

## Database migrations

- Migration files go in `database/migrations/schema/`.
- Files are numbered sequentially: `NNNN_description.sql`.
- Each file should be self-contained and idempotent where possible.
- Run `just db-migrate` to apply pending migrations to the main database.
- Add a corresponding seed row to `database/tests/data/contract.sql` if the migration adds reference data.

## Services (background workers)

Background services are started in `ocg-server/src/main.rs` and run on separate Tokio tasks. Each service module exposes a `run()` or similar entry point. Use `tokio::time::interval` for periodic tasks and rely on the `DynDB` handle (already connection-pooled) rather than creating ad-hoc connections.

## Rust style

- Run `cargo fmt` before committing. The `just server-fmt-and-lint` task runs `cargo fmt`, `cargo check`, and `cargo clippy --deny warnings`.
- All `clippy` warnings are treated as errors in CI.
- Prefer `pub(crate)` over `pub` for items that are not part of a public API.
- Use `#[instrument]` from `tracing` on public handler and service functions for automatic span creation.
- Doc comments (`///`) are expected on public and `pub(crate)` items.

## Frontend

- JavaScript files live in `ocg-server/static/js/`. Format with `prettier` using the config at `ocg-server/static/js/.prettierrc.yaml`.
- HTML templates live in `ocg-server/templates/`. Lint with `djlint` using the config at `ocg-server/templates/.djlintrc`.
- Run `just frontend-fmt-and-lint` to check both.

## Tests

| Test type | Location | Run with |
|-----------|----------|---------|
| Rust unit | `ocg-server/src/` (`#[test]`) | `just server-tests` |
| DB contract | `ocg-server/src/db/` (`#[ignore]`) | `just db-contract-tests` |
| DB pgTAP | `database/tests/` | `just db-tests` |
| Frontend unit | `tests/unit/` | `npm --prefix tests/unit test` |
| E2E (Playwright) | `tests/e2e/` | `just e2e-tests` |

Visual snapshot tests are tagged `@visual` and updated with `just e2e-update-snapshots`.
