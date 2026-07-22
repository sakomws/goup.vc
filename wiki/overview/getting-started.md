# Getting started

## Prerequisites

| Tool | Version | Notes |
|------|---------|-------|
| Rust | stable (current) | Install via [rustup](https://rustup.rs/) |
| PostgreSQL | 17 | `brew install postgresql@17` on macOS |
| tern | latest | `go install github.com/jackc/tern/v2@latest` |
| Node.js | 18 + | Required for `mcp/` and frontend tests |
| just | latest | `brew install just` or `cargo install just` |
| Playwright | via npm | Installed by `just e2e-install` |

Optional tools used in development:

- `watchexec` — for `just server-watch` / `just e2e-server-watch`
- `pg_prove` — for running pgTAP database tests
- `djlint` and `prettier` — for template and JS linting

## Clone and build

```sh
git clone https://github.com/goup-vc/goup.vc.git
cd goup.vc

# Build both binaries
cargo build -p ocg-server
cargo build -p ocg-redirector
```

## Database setup

The `justfile` uses environment variables to locate PostgreSQL. All variables have defaults; override as needed.

| Variable | Default |
|----------|---------|
| `OCG_DB_HOST` | `localhost` |
| `OCG_DB_NAME` | `ocg` |
| `OCG_DB_USER` | `postgres` |
| `OCG_DB_PORT` | `5432` |
| `OCG_PG_BIN` | `/opt/homebrew/opt/postgresql@17/bin` |
| `OCG_CONFIG` | `$HOME/.config/ocg` |

Create and migrate the main database:

```sh
just db-create
just db-migrate
```

`just db-migrate` calls `database/migrations/migrate.sh` with the tern config at `$OCG_CONFIG/tern.conf`. Create that file before running migrations (see `database/migrations/README.md` for the tern config format).

## Server configuration

The server reads a YAML config file and `OCG_*` environment variables (nested keys separated by `__`). Place a minimal config at `$HOME/.config/ocg/server.yml`:

```yaml
db:
  host: localhost
  dbname: ocg
  user: postgres

email:
  from_address: dev@example.com
  from_name: OCG Dev
  smtp:
    host: localhost
    port: 1025
    username: ""
    password: ""

server:
  addr: "127.0.0.1:9000"
  base_url: "http://localhost:9000"
  login:
    email: true
    github: false
    linkedin: false
```

See `ocg-server/src/config.rs` for all available keys and their types.

## Run the server

```sh
just server
```

This runs `cargo run -p ocg-server -- -c $HOME/.config/ocg/server.yml`. The server listens on `127.0.0.1:9000` by default.

For auto-reload on source changes:

```sh
just server-watch   # requires watchexec
```

## Running tests

### Rust unit and integration tests

```sh
just server-tests
```

### Database contract tests

```sh
# Creates ocg_tests_contract, seeds it, and runs cargo tests tagged db_contracts
just db-contract-tests
```

### Database pgTAP tests

```sh
# Creates and migrates ocg_tests, then runs pg_prove
just db-tests
```

### Frontend unit tests

```sh
npm --prefix tests/unit test
```

### End-to-end (Playwright)

```sh
# One-time setup
just e2e-install

# Create and migrate the e2e database
just db-recreate-tests-e2e
just db-load-tests-e2e-data

# Start the e2e server in one terminal
just e2e-server

# Run tests in another terminal
just e2e-tests
```

The e2e server config lives at `$HOME/.config/ocg/server-tests-e2e.yml` (override with `OCG_SERVER_CONFIG_TESTS_E2E`).

## MCP server

```sh
cd mcp
npm install
node server.mjs
```

The MCP server reads connection details from its own config. See `mcp/README.md` for details.
