#!/usr/bin/env bash

# @spec:NFR-OPS-006 - Docker Volume Restore
# @ref:specs/05-operations/5.3-disaster-recovery/DR-PLAN.md

set -euo pipefail

BACKUP_FILE="$1"
LOG_FILE="${LOG_DIR:-/var/log/backups}/restore-volumes-$(date +%Y%m%d_%H%M%S).log"

log_info() {
    echo "[$(date '+%Y-%m-%d %H:%M:%S')] [INFO] $*" | tee -a "$LOG_FILE"
}

mkdir -p "$(dirname "$LOG_FILE")"

if [ -z "$BACKUP_FILE" ]; then
    echo "Usage: $0 <backup-file.tar.gz>"
    exit 1
fi

if [ ! -f "$BACKUP_FILE" ]; then
    log_info "Backup file not found: $BACKUP_FILE"
    exit 1
fi

# Extract volume name from filename
VOLUME_NAME=$(basename "$BACKUP_FILE" | sed 's/_[0-9]*\.tar\.gz$//')

log_info "Restoring volume: $VOLUME_NAME from $BACKUP_FILE"

# Create volume if doesn't exist
if ! docker volume inspect "$VOLUME_NAME" &>/dev/null; then
    log_info "Creating volume: $VOLUME_NAME"
    docker volume create "$VOLUME_NAME"
fi

# Restore volume
docker run --rm \
    -v "$VOLUME_NAME:/data" \
    -v "$(dirname "$BACKUP_FILE"):/backup" \
    alpine \
    sh -c "cd /data && tar xzf /backup/$(basename "$BACKUP_FILE")"

log_info "Volume restored: $VOLUME_NAME"
exit 0
