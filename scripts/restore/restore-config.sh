#!/usr/bin/env bash

# @spec:NFR-OPS-007 - Configuration Restore
# @ref:specs/05-operations/5.3-disaster-recovery/DR-PLAN.md

set -euo pipefail

BACKUP_FILE="$1"
RESTORE_DIR="${2:-.}"
LOG_FILE="${LOG_DIR:-/var/log/backups}/restore-config-$(date +%Y%m%d_%H%M%S).log"

log_info() {
    echo "[$(date '+%Y-%m-%d %H:%M:%S')] [INFO] $*" | tee -a "$LOG_FILE"
}

mkdir -p "$(dirname "$LOG_FILE")"

if [ -z "$BACKUP_FILE" ]; then
    echo "Usage: $0 <backup-file.tar.gz> [restore-directory]"
    exit 1
fi

log_info "Restoring configuration from: $BACKUP_FILE"
log_info "Restore directory: $RESTORE_DIR"

# Verify checksum
if [ -f "${BACKUP_FILE}.sha256" ]; then
    log_info "Verifying checksum..."
    sha256sum -c "${BACKUP_FILE}.sha256" || exit 1
fi

# Create backup of current config
CURRENT_BACKUP="/tmp/config-before-restore-$(date +%Y%m%d_%H%M%S).tar.gz"
log_info "Backing up current config to: $CURRENT_BACKUP"
tar czf "$CURRENT_BACKUP" \
    docker-compose*.yml \
    Makefile \
    .env.example \
    infrastructure/ \
    .github/ \
    scripts/ \
    2>/dev/null || true

# Extract backup
log_info "Extracting configuration..."
tar xzf "$BACKUP_FILE" -C "$RESTORE_DIR"

log_info "Configuration restored successfully"
log_info "Previous config saved at: $CURRENT_BACKUP"
exit 0
