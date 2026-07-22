# Deployment

GOUP Alliance runs on AWS EC2 behind nginx, with Docker images built by CI and Kubernetes (Helm) available as an alternative deployment target.

## EC2 deployment (primary)

The `scripts/bootstrap-ec2.sh` script handles the full initial setup of an EC2 instance:

1. Installs system dependencies (Rust, Node.js, PostgreSQL client, tailwindcss, tern)
2. Creates the database and runs migrations
3. Optionally seeds initial alliance/group data
4. Builds `ocg-server` with `cargo build --release`
5. Installs and starts a `systemd` service (`ocg-server`)

```sh
./scripts/bootstrap-ec2.sh
```

Key environment variables that control bootstrap behavior:

| Variable | Default | Description |
|----------|---------|-------------|
| `BOOTSTRAP_BUILD` | `true` | Whether to run `cargo build` |
| `BOOTSTRAP_MIGRATE` | `true` | Whether to run migrations |
| `BOOTSTRAP_SEED` | `true` | Whether to seed initial data |
| `BOOTSTRAP_SYSTEMD_SERVICE` | `true` | Whether to install systemd service |
| `CARGO_BUILD_JOBS` | `1` | Parallel build jobs |

### Updating after a code change

```sh
cd ~/goup.vc
git pull origin main
cargo build --release -p ocg-server
sudo systemctl restart ocg-server
sudo systemctl status ocg-server --no-pager
```

If database migrations changed:
```sh
just db-migrate
```

## MCP server

The MCP service runs as a separate systemd unit (`goup-mcp`). Set it up with:

```sh
./scripts/setup-mcp-ec2.sh
```

Update after `mcp/` changes:
```sh
git pull origin main
sudo systemctl restart goup-mcp
```

## nginx configuration

The nginx config template is in `scripts/nginx-goup.conf`. It proxies:
- `/` to `ocg-server` on port 9000
- `/mcp` to `goup-mcp` on port 8787

SSL termination is handled by nginx; both services listen on localhost only.

## Docker images

`docker-compose.dev.yml` provides a local development environment with PostgreSQL, the tern migration runner, and a seed step:

```sh
docker compose -f docker-compose.dev.yml up
```

CI builds production images via `.github/workflows/build-images.yml` on every push to `main`. Images are pushed to a container registry and tagged with the Git SHA.

## Kubernetes / Helm

A Helm chart lives in `charts/goup/`. Key values in `charts/goup/values.yaml`:

| Key | Description |
|-----|-------------|
| `db.host` / `db.dbname` / `db.user` / `db.password` | PostgreSQL connection |
| `images.provider` | `db` or `s3` |
| `email.smtp.*` | SMTP server settings |
| `server.base_url` | Public-facing URL |
| `imageTag` | Docker image tag to deploy |

Install with:
```sh
helm install goup charts/goup/ -f my-values.yaml
```

## Configuration files

The server looks for its config at `~/.config/ocg/server.yml` by default (override with `OCG_SERVER_CONFIG`). The tern migration tool reads `~/.config/ocg/tern.conf`.

All config keys can be overridden with `OCG_` prefixed environment variables using double underscores for nesting (e.g., `OCG_DB__HOST=myhost`). See [`reference/configuration.md`](reference/configuration.md) for the full reference.

## Systemd services

| Service | Binary | Port | Log command |
|---------|--------|------|-------------|
| `ocg-server` | `ocg-server` | 9000 | `journalctl -u ocg-server -n 100 --no-pager` |
| `goup-mcp` | `node mcp/server.mjs` | 8787 | `journalctl -u goup-mcp -n 50 --no-pager` |
