# Debugging

## Log output

The server uses `tracing-subscriber` for structured logs. Control verbosity with the `RUST_LOG` environment variable:

```sh
RUST_LOG=debug just server
RUST_LOG=ocg_server=trace just server  # ocg-server only, trace level
```

The log format is configured in `server.yml` under `log.format`. Options are `json` (default in production) and `pretty` (useful for local development):

```yaml
log:
  format: pretty
```

## Common errors

### `tailwindcss not found in PATH`

The server uses a standalone Tailwind CSS binary to build stylesheets. Install it:

```sh
# macOS
brew install tailwindcss

# Or download the standalone binary from
# https://github.com/tailwindlabs/tailwindcss/releases
# and place it somewhere on your PATH
```

### `tern: command not found`

The database migration tool is a Go binary not bundled with the repo:

```sh
go install github.com/jackc/tern/v2@v2.3.2
export PATH="$HOME/go/bin:$PATH"
```

### Playwright browser executable missing

```sh
cd tests/unit
npx playwright install
# or:
just e2e-install
```

### Login redirects behave strangely locally

Confirm that `server.base_url`, cookie settings, and OAuth provider redirect URIs all match the URL you use in the browser. A mismatch between `http://localhost:9000` and `http://127.0.0.1:9000` will cause redirect failures with OAuth providers.

### Database connection refused

Check that PostgreSQL is running and that the connection parameters in `tern.conf` and `server.yml` match. The server expects PostgreSQL 17; run `psql --version` to verify.

### Migrations fail on `ocg_tests_contract`

The contract test recipe drops and recreates the database on each run. If a previous run left it in a bad state, run `just db-drop-tests-contract` manually, then retry.

## EC2 production logs

```sh
sudo journalctl -u ocg-server -n 100 --no-pager
sudo journalctl -u goup-mcp -n 50 --no-pager
```

## Inspecting the database

```sh
just db-client           # connect to main database
just db-client-tests     # connect to test database
just db-client-tests-e2e # connect to e2e database
```
