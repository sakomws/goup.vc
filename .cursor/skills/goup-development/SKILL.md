---
name: goup-development
description: Implements and reviews goup.vc changes across its Rust/Axum server, PostgreSQL migrations and functions, Askama/HTMX dashboard UI, browser tests, and Helm deployment charts. Use when modifying goup.vc application code, database behavior, UI interactions, tests, or Kubernetes configuration.
---

# GOUP Development

## Architecture

- Backend: Rust/Axum in `ocg-server/`; keep handlers thin and use the typed DB traits in `ocg-server/src/db/`.
- Database: PostgreSQL schema changes and SQL functions live under `database/migrations/`. Follow existing numbered, idempotent migration patterns.
- UI: Askama templates in `ocg-server/templates/`, HTMX for server interactions, and page-specific JavaScript under `ocg-server/static/js/`.
- Deployment: Helm chart configuration is under `charts/goup/`.

## Implementation workflow

1. Trace the request through its route, handler, database trait/function, template, and JavaScript before editing.
2. Enforce authorization in router middleware. Do not rely on hidden UI controls for access control.
3. Keep private data in dedicated, permission-protected queries. Do not extend public/read queries with sensitive fields solely for an admin feature.
4. For schema/data changes, add an idempotent migration and update matching Rust models, DB methods, mocks, and database tests.
5. For tabbed or HTMX UI, verify DOM ownership: each section’s controls must be inside its `data-content` panel and form.
6. Add focused regression coverage for bugs and changed behavior.

## Validation

- Rust: `cargo fmt --check --manifest-path ocg-server/Cargo.toml` and the narrowest relevant `cargo test` or `cargo check`.
- SQL: run the project’s database migration/function tests when the local database environment is available.
- UI: run the relevant test under `tests/unit/` or `tests/e2e/`; report missing local tooling instead of claiming it passed.
- Helm: run chart lint/template validation before changing deployment manifests.

## Safety rules

- Treat member contact data, authentication, payments, and invitations as sensitive.
- Make bulk data changes deterministic and idempotent; resolve users by an explicit verified username or email.
- Preserve HTMX response headers and existing pagination/filter semantics when adding dashboard endpoints.
