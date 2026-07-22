# Development workflow

## Branch strategy

Work is done on feature branches off `main`. Branch names typically follow the pattern `feat/<topic>`, `fix/<topic>`, or `chore/<topic>`. After review, branches are squash-merged to `main` via GitHub pull requests.

## Day-to-day cycle

1. Pull latest `main` and create a feature branch.
2. Start the database and run migrations:
   ```sh
   just db-migrate
   ```
3. Start the server:
   ```sh
   just server
   # or, with auto-reload on source changes:
   just server-watch
   ```
4. Make changes, run the relevant tests (see [testing](testing.md)).
5. Lint before pushing:
   ```sh
   cargo fmt --check
   cargo clippy --all-targets
   ```
6. Open a pull request against `main`.

## Key `just` recipes

| Recipe | What it does |
|--------|-------------|
| `just server` | Run `ocg-server` with the default config |
| `just server-watch` | Run with `watchexec` for auto-reload |
| `just server-tests` | Run all Rust tests |
| `just db-migrate` | Apply pending schema migrations |
| `just db-create` | Create the main database |
| `just db-recreate` | Drop and recreate the main database |
| `just db-tests` | Run pgTAP database tests |
| `just db-contract-tests` | Run Rust DB contract tests |
| `just e2e-tests` | Run Playwright end-to-end tests |
| `just e2e-server` | Start server in e2e mode |
| `just lint-templates` | Run `djlint` on MiniJinja templates |
| `just lint-js` | Run `prettier` on JavaScript files |

Run `just --list` in the repository root for a complete reference.

## CI checks

The CI pipeline (`.github/workflows/ci.yml`) uses path filters so only the jobs relevant to changed files run. Jobs include:

- `audit-dependencies` — `cargo audit` for known CVEs
- `lint-server` — `cargo fmt --check` and `cargo clippy`
- `test-server` — `cargo test`
- `test-database` — pgTAP migrations and SQL tests
- `test-database-contracts` — Rust DB contract tests
- `lint-javascript` — `prettier --check`
- `lint-templates` — `djlint`
- `lint-markdown` — markdown lint
- `check-mcp` — `node --check` on `mcp/server.mjs`

A separate `e2e.yml` workflow runs Playwright tests. `build-images.yml` builds and pushes Docker images on pushes to `main`.

## Adding a new database migration

1. Create a new `.sql` file in `database/migrations/schema/` with the next sequential number (e.g., `0100_my_change.sql`).
2. Run `just db-migrate` locally to verify it applies cleanly.
3. Never edit existing migration files — only add new ones.

## Changing configuration

Configuration is loaded from a YAML file and `OCG_*` env vars. If you add a new config key, update `ocg-server/src/config.rs` and document the new key in [`reference/configuration.md`](../reference/configuration.md).
