# Tooling

## Build system — just

All day-to-day tasks are defined in `justfile` at the repository root. Install `just` from <https://just.systems> and run `just --list` to see available recipes.

### Server recipes

| Recipe | Description |
|---|---|
| `just server` | Run `ocg-server` with `cargo run` |
| `just server-watch` | Run with `watchexec` auto-reload |
| `just server-build` | Build the server binary without running |
| `just server-tests` | Run Rust unit and integration tests |
| `just server-fmt-and-lint` | `cargo fmt`, `cargo check`, `cargo clippy` |

### Redirector recipes

| Recipe | Description |
|---|---|
| `just redirector` | Run `ocg-redirector` |
| `just redirector-build` | Build the redirector binary |
| `just redirector-tests` | Run redirector tests |
| `just redirector-fmt-and-lint` | Format and lint redirector code |

### Database recipes

| Recipe | Description |
|---|---|
| `just db-recreate` | Drop, create, and migrate main database |
| `just db-recreate-tests` | Drop, create, and migrate test database |
| `just db-recreate-tests-contract` | Drop, create, migrate, and seed contract test database |
| `just db-recreate-tests-e2e` | Drop, create, and migrate e2e test database |
| `just db-migrate` | Apply pending migrations to main database |
| `just db-migrate-tests` | Apply pending migrations to test database |
| `just db-tests` | Recreate test database and run pgTAP tests |
| `just db-tests-file <file>` | Run pgTAP tests in a single file |
| `just db-contract-tests` | Recreate contract database and run contract tests |
| `just db-client` | Open a psql session on the main database |
| `just db-server <data_dir>` | Start a local PostgreSQL server |

### Frontend and e2e recipes

| Recipe | Description |
|---|---|
| `just frontend-fmt-and-lint` | Run `prettier` on JS files and `djlint` on templates |
| `just frontend-unit-tests` | Run web-test-runner unit tests via `npm test` |
| `just e2e-install` | Install npm dependencies and Playwright browsers |
| `just e2e-server` | Start server with e2e config |
| `just e2e-server-watch` | Start server with e2e config and auto-reload |
| `just e2e-tests` | Run the Playwright e2e suite |
| `just e2e-update-snapshots` | Update visual regression snapshots |

## Linters

### clippy (Rust)

`cargo clippy --all-targets --all-features -- --deny warnings` is run as part of `just server-fmt-and-lint` and in CI (`lint-and-test-server` job). All clippy warnings are treated as errors.

### rustfmt

`cargo fmt --all -- --check` is run in CI. Locally, `just server-fmt-and-lint` runs `cargo fmt` (without `--check`) to format in place.

The formatter is configured by `.rustfmt.toml` at the repository root.

### djlint (Jinja/HTML templates)

`djlint` version 1.36.4 lints and checks formatting of templates under `ocg-server/templates/`. Configuration is in `ocg-server/templates/.djlintrc`.

Run locally:

```sh
djlint --check --configuration ocg-server/templates/.djlintrc ocg-server/templates
```

Or via the justfile:

```sh
just frontend-fmt-and-lint
```

### prettier (JavaScript)

`prettier` version 3.9.1 checks formatting of JS files under `ocg-server/static/js/`. Configuration is in `ocg-server/static/js/.prettierrc.yaml`.

`just frontend-fmt-and-lint` runs prettier with `--write` (formats in place). CI uses `--check` (dry run).

### markdownlint

Markdown files are linted in CI using `markdownlint-cli2` (`lint-markdown-files` job). Configuration follows the defaults.

### cargo audit

Dependency vulnerabilities are checked with `cargo audit` in the `audit-dependencies` CI job. Audit configuration is in `.cargo/audit.toml`.

## Code generators

The server build script generates the Tailwind CSS output. This requires the `tailwindcss` standalone binary on `$PATH`. The build fails without it.

There are no other code generators. SQL migrations are hand-written in `database/migrations/`.

## CI tooling

Three workflow files live under `.github/workflows/`:

- **`ci.yml`** — runs on every PR targeting `main`. Uses `dorny/paths-filter` to skip jobs when no relevant files changed. Jobs: `audit-dependencies`, `check-js-files-format`, `lint-and-test-server`, `lint-markdown-files`, `lint-templates`, `test-database`, `test-database-contracts`, `test-frontend`, `test-mcp`.
- **`e2e.yml`** — runs the Playwright e2e suite (separate from the main CI workflow).
- **`build-images.yml`** — builds and pushes Docker images.

The Rust toolchain version is pinned in `ci.yml` (`dtolnay/rust-toolchain@master` with `toolchain: 1.96.0`). Use the same version locally to avoid fmt or clippy divergence.
