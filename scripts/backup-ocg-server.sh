#!/usr/bin/env bash
# Copy the current release binary aside before a build replaces it.
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
BINARY="${ROOT_DIR}/target/release/ocg-server"
BACKUP_DIR="${OCG_SERVER_BACKUP_DIR:-${HOME}/ocg-server-backups}"
KEEP="${OCG_SERVER_BACKUP_KEEP:-5}"

if [[ ! -f "${BINARY}" ]]; then
    echo "No release binary to back up at ${BINARY}"
    exit 0
fi

mkdir -p "${BACKUP_DIR}"

sha="unknown"
if git -C "${ROOT_DIR}" rev-parse --short HEAD >/dev/null 2>&1; then
    sha="$(git -C "${ROOT_DIR}" rev-parse --short HEAD)"
fi
stamp="$(date -u +%Y%m%dT%H%M%SZ)"
dest="${BACKUP_DIR}/ocg-server-${stamp}-${sha}"

cp -a "${BINARY}" "${dest}"
ln -sfn "${dest}" "${BACKUP_DIR}/ocg-server.prev"
echo "Backed up ${BINARY} to ${dest}"

# Keep the newest snapshots, including the one ocg-server.prev points at.
mapfile -t snapshots < <(find "${BACKUP_DIR}" -maxdepth 1 -type f -name 'ocg-server-*' | sort)
if [[ "${#snapshots[@]}" -gt "${KEEP}" ]]; then
    remove_count=$((${#snapshots[@]} - KEEP))
    for old in "${snapshots[@]:0:remove_count}"; do
        if [[ "$(readlink -f "${BACKUP_DIR}/ocg-server.prev")" == "$(readlink -f "${old}")" ]]; then
            continue
        fi
        rm -f "${old}"
    done
fi
