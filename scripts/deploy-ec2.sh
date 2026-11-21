#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "${ROOT_DIR}"

CARGO_BUILD_JOBS="${CARGO_BUILD_JOBS:-1}"
TERN_CONF="${TERN_CONF:-${HOME}/.config/ocg/tern.conf}"
DEPLOY_PULL="${DEPLOY_PULL:-true}"
DEPLOY_BINARY="${DEPLOY_BINARY:-}"
DEPLOY_RESTART_MCP="${DEPLOY_RESTART_MCP:-auto}"
HEALTHCHECK_URL="${DEPLOY_HEALTHCHECK_URL:-http://127.0.0.1:9000/health-check}"

# Pull the latest main branch when this script is run directly on EC2.
if [[ "${DEPLOY_PULL}" == "true" ]]; then
    git fetch origin main
    git checkout main
    git pull --ff-only origin main
fi

echo "Deploying $(git rev-parse --short HEAD) from $(git rev-parse --abbrev-ref HEAD)"

# Apply schema and function migrations with the host tern config.
(
    cd database/migrations
    TERN_CONF="${TERN_CONF}" ./migrate.sh
)

echo "Backing up the current release binary"
"${ROOT_DIR}/scripts/backup-ocg-server.sh"

if [[ -n "${DEPLOY_BINARY}" ]]; then
    echo "Installing prebuilt ocg-server from ${DEPLOY_BINARY}"
    if pgrep -u "$(id -u)" -f 'cargo build --release -p ocg-server' >/dev/null; then
        echo "Warning: a cargo release build is still running and may later overwrite target/release/ocg-server"
    fi
    mkdir -p target/release
    install -m 0755 "${DEPLOY_BINARY}" target/release/ocg-server.new
    target/release/ocg-server.new --help >/dev/null
    mv -f target/release/ocg-server.new target/release/ocg-server
else
    echo "Building ocg-server with CARGO_BUILD_JOBS=${CARGO_BUILD_JOBS}"
    CARGO_BUILD_JOBS="${CARGO_BUILD_JOBS}" cargo build --release -p ocg-server
fi

sudo systemctl restart ocg-server
sudo systemctl --no-pager --full status ocg-server

# Keep the production reverse proxy aligned with the versioned configuration.
nginx_user="$(awk '$1 == "user" && !found { sub(";", "", $2); print $2; found = 1 }' /etc/nginx/nginx.conf 2>/dev/null || true)"
if [[ -z "${nginx_user}" ]]; then
    if id -u www-data >/dev/null 2>&1; then
        nginx_user="www-data"
    else
        nginx_user="nginx"
    fi
fi
sudo install -d -o "${nginx_user}" -g "$(id -gn "${nginx_user}")" /var/cache/nginx/goup-pages
sudo install -m 0644 scripts/nginx-goup.conf /etc/nginx/conf.d/goup.conf
sudo nginx -t
sudo systemctl reload nginx

# Restart MCP only when that service exists and mcp/ files changed.
if systemctl list-unit-files --type=service --no-legend | grep -q '^goup-mcp.service'; then
    mcp_changed="false"
    if [[ "${DEPLOY_RESTART_MCP}" == "true" ]]; then
        mcp_changed="true"
    elif [[ "${DEPLOY_RESTART_MCP}" == "auto" ]] && git rev-parse --verify ORIG_HEAD >/dev/null 2>&1; then
        if git diff --name-only ORIG_HEAD HEAD | grep -q '^mcp/'; then
            mcp_changed="true"
        fi
    fi

    if [[ "${mcp_changed}" == "true" ]]; then
        sudo systemctl restart goup-mcp
        sudo systemctl --no-pager --full status goup-mcp
    fi
fi

curl -fsS -o /dev/null --max-time 20 --retry 10 --retry-delay 2 --retry-connrefused "${HEALTHCHECK_URL}"
echo "Deployed $(git rev-parse --short HEAD)"
