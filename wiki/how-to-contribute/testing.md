# Testing

The project has four distinct test layers, each covering a different scope.

## Rust unit and integration tests

Run with:
```sh
just server-tests
# or directly:
cargo test -p ocg-server
```

Tests live alongside the code they test in `#[cfg(test)] mod tests` blocks. The database layer uses a mock (`ocg-server/src/db/mock.rs`) implementing `DynDB`, so most handler and service tests do not require a live database.

## Database contract tests

These tests verify that the Rust DB queries work against a real PostgreSQL instance. They are tagged `#[ignore]` and run with a dedicated database to avoid contaminating development data.

```sh
just db-contract-tests
```

This recipe:
1. Drops and recreates `ocg_tests_contract`
2. Runs migrations against it
3. Loads seed data from `database/tests/data/contract.sql`
4. Runs `cargo test -p ocg-server db_contracts -- --ignored --test-threads=1`

Contract tests live in `ocg-server/src/db/contract_tests.rs`. Each test function sets up minimal data, calls a DB method, and asserts the result. They run single-threaded to avoid transaction conflicts.

## Database pgTAP tests

pgTAP tests run SQL assertions directly in PostgreSQL. They verify schema constraints, triggers, and stored functions.

```sh
just db-tests
```

This recipe creates `ocg_tests`, enables the `pgtap` extension, runs migrations, then calls `pg_prove` against the test files in `database/tests/`. Requires `pg_prove` installed (`cpanm TAP::Parser::SourceHandler::pgTAP`).

## Frontend unit tests

JavaScript utility functions in `ocg-server/static/js/` are tested with Web Test Runner.

```sh
npm --prefix tests/unit test
```

Test files live in `tests/unit/`. The config is at `tests/unit/web-test-runner.config.mjs`.

## End-to-end tests (Playwright)

Full browser tests covering auth, site navigation, group/event flows, and dashboard actions.

```sh
# One-time setup
just e2e-install   # runs npx playwright install

# Create and seed the e2e database
just db-recreate-tests-e2e
just db-load-tests-e2e-data

# In one terminal: start the e2e server
just e2e-server

# In another terminal: run all tests
just e2e-tests

# Run a single test file
just e2e-tests tests/e2e/auth/login.spec.js
```

E2e tests are in `tests/e2e/` grouped by domain: `auth/`, `site/`, `dashboard/`. Shared fixtures and utilities are in `tests/e2e/fixtures.js` and `tests/e2e/utils.js`. The Playwright config is at `tests/e2e/playwright.config.js`.

The e2e server reads a separate config file (default: `~/.config/ocg/server-tests-e2e.yml`) to isolate the test database.

## Test type reference

| Layer | Command | Requires DB | Scope |
|-------|---------|-------------|-------|
| Rust unit/integration | `just server-tests` | No (mock) | Handlers, services, utils |
| DB contract | `just db-contract-tests` | Yes (`ocg_tests_contract`) | SQL queries |
| pgTAP | `just db-tests` | Yes (`ocg_tests`) | Schema, triggers, functions |
| JS unit | `npm --prefix tests/unit test` | No | Frontend utilities |
| E2e | `just e2e-tests` | Yes (`ocg_tests_e2e`) | Full browser flows |
