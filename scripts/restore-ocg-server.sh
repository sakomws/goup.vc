#!/usr/bin/env bash
# Put a backed-up release binary back and restart ocg-server.
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
BINARY="${ROOT_DIR}/target/release/ocg-server"
BACKUP_DIR="${OCG_SERVER_BACKUP_DIR:-${HOME}/ocg-server-backups}"
SOURCE="${1:-${BACKUP_DIR}/ocg-server.prev}"

if [[ ! -e "${SOURCE}" ]]; then
    echo "No backup at ${SOURCE}" >&2
    exit 1
fi

resolved="$(readlink -f "${SOURCE}")"
if [[ ! -f "${resolved}" ]]; then
    echo "Backup is not a file: ${SOURCE}" >&2
    exit 1
fi

mkdir -p "$(dirname "${BINARY}")"
cp -a "${resolved}" "${BINARY}"
echo "Restored ${resolved} to ${BINARY}"

if command -v systemctl >/dev/null 2>&1 && systemctl list-unit-files 'ocg-server.service' --no-legend 2>/dev/null | grep -q '^ocg-server.service'; then
    sudo systemctl restart ocg-server
    sudo systemctl --no-pager --full status ocg-server
else
    echo "Restart the server so it loads the restored binary."
fi
