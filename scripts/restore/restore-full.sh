#!/usr/bin/env bash

# @spec:NFR-OPS-008 - Full System Restore
# @ref:specs/05-operations/5.3-disaster-recovery/DR-PLAN.md

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
LOG_FILE="${LOG_DIR:-/var/log/backups}/restore-full-$(date +%Y%m%d_%H%M%S).log"

log_info() {
    echo "[$(date '+%Y-%m-%d %H:%M:%S')] [INFO] $*" | tee -a "$LOG_FILE"
}

log_error() {
    echo "[$(date '+%Y-%m-%d %H:%M:%S')] [ERROR] $*" | tee -a "$LOG_FILE"
}

mkdir -p "$(dirname "$LOG_FILE")"

echo "========================================="
echo "FULL SYSTEM RESTORE"
echo "========================================="
echo ""
echo "WARNING: This will restore:"
echo "  - MongoDB database"
echo "  - Docker volumes"
echo "  - Configuration files"
echo ""
read -rp "Continue? (yes/no): " confirm

if [ "$confirm" != "yes" ]; then
    echo "Restore cancelled"
    exit 0
fi

log_info "Starting full system restore..."
start_time=$(date +%s)

# Step 1: Restore configuration
log_info "Step 1/3: Restore configuration"
echo "Available config backups:"
find "${BACKUP_DIR:-/backups}/config" -name "config_*.tar.gz" | sort -r | head -5
read -rp "Enter config backup file path: " config_backup

if ! bash "$SCRIPT_DIR/restore-config.sh" "$config_backup"; then
    log_error "Config restore failed"
    exit 1
fi

# Step 2: Restore MongoDB
log_info "Step 2/3: Restore MongoDB"
if ! bash "$SCRIPT_DIR/restore-mongodb.sh"; then
    log_error "MongoDB restore failed"
    exit 1
fi

# Step 3: Restore volumes (optional)
log_info "Step 3/3: Restore volumes"
read -rp "Restore volumes? (yes/no): " restore_volumes

if [ "$restore_volumes" = "yes" ]; then
    echo "Available volume backups:"
    find "${BACKUP_DIR:-/backups}/volumes" -name "*.tar.gz" | sort -r | head -10

    read -rp "Enter volume backup directory or 'skip': " volume_dir

    if [ "$volume_dir" != "skip" ] && [ -d "$volume_dir" ]; then
        for backup in "$volume_dir"/*.tar.gz; do
            log_info "Restoring: $backup"
            bash "$SCRIPT_DIR/restore-volumes.sh" "$backup" || log_error "Failed: $backup"
        done
    fi
fi

end_time=$(date +%s)
duration=$((end_time - start_time))

log_info "========================================="
log_info "Full system restore complete"
log_info "Duration: ${duration}s"
log_info "========================================="

exit 0
