#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "${ROOT_DIR}"

CARGO_BUILD_JOBS="${CARGO_BUILD_JOBS:-1}"
TERN_CONF="${TERN_CONF:-${HOME}/.config/ocg/tern.conf}"
DEPLOY_PULL="${DEPLOY_PULL:-true}"
DEPLOY_RESTART_MCP="${DEPLOY_RESTART_MCP:-auto}"
HEALTHCHECK_URL="${DEPLOY_HEALTHCHECK_URL:-http://127.0.0.1:9000}"

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

echo "Building ocg-server with CARGO_BUILD_JOBS=${CARGO_BUILD_JOBS}"
CARGO_BUILD_JOBS="${CARGO_BUILD_JOBS}" cargo build --release -p ocg-server

sudo systemctl restart ocg-server
sudo systemctl --no-pager --full status ocg-server

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

curl -fsS -I --max-time 20 "${HEALTHCHECK_URL}" >/dev/null
echo "Deployed $(git rev-parse --short HEAD)"
