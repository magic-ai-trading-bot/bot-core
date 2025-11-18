#!/usr/bin/env bash

# @spec:NFR-OPS-014 - Backup Cleanup and Retention Policy
# @ref:specs/05-operations/5.3-disaster-recovery/DR-PLAN.md

set -euo pipefail

# Script: cleanup-old-backups.sh
# Purpose: Enforce backup retention policy
# Usage: ./cleanup-old-backups.sh [--dry-run]
# Version: 1.0.0

BACKUP_DIR="${BACKUP_DIR:-/backups}"
LOG_FILE="${LOG_DIR:-/var/log/backups}/cleanup-$(date +%Y%m%d).log"
DRY_RUN="${1:-}"

# Retention policy (days)
RETENTION_DAILY=7        # Keep daily backups for 7 days
RETENTION_WEEKLY=28      # Keep weekly backups for 4 weeks
RETENTION_MONTHLY=365    # Keep monthly backups for 1 year

log_info() {
    echo "[$(date '+%Y-%m-%d %H:%M:%S')] [INFO] $*" | tee -a "$LOG_FILE"
}

mkdir -p "$(dirname "$LOG_FILE")"

log_info "========================================="
log_info "Backup Cleanup Started"
[ "$DRY_RUN" = "--dry-run" ] && log_info "MODE: DRY RUN (no files will be deleted)"
log_info "========================================="

deleted_count=0
kept_count=0
freed_space=0

cleanup_directory() {
    local dir="$1"
    local retention_days="$2"
    local pattern="$3"

    if [ ! -d "$dir" ]; then
        log_info "Directory not found: $dir"
        return
    fi

    log_info "Cleaning $dir (retention: ${retention_days} days)"

    while IFS= read -r file; do
        local file_age_days
        file_age_days=$(( ($(date +%s) - $(stat -f%m "$file" 2>/dev/null || stat -c%Y "$file")) / 86400 ))

        if [ "$file_age_days" -gt "$retention_days" ]; then
            local file_size
            file_size=$(stat -f%z "$file" 2>/dev/null || stat -c%s "$file")

            if [ "$DRY_RUN" = "--dry-run" ]; then
                log_info "Would delete (${file_age_days}d old): $(basename "$file")"
            else
                log_info "Deleting (${file_age_days}d old): $(basename "$file")"
                rm -f "$file"
                # Also delete checksum file
                rm -f "${file}.sha256"
            fi

            ((deleted_count++))
            freed_space=$((freed_space + file_size))
        else
            ((kept_count++))
        fi
    done < <(find "$dir" -name "$pattern" -type f ! -name "*.sha256")
}

# Cleanup MongoDB daily backups
cleanup_directory "$BACKUP_DIR/mongodb" "$RETENTION_DAILY" "mongodb_full_*.tar.gz*"
cleanup_directory "$BACKUP_DIR/mongodb" "$RETENTION_DAILY" "mongodb_incremental_*.tar.gz*"

# Cleanup volume backups
cleanup_directory "$BACKUP_DIR/volumes" "$RETENTION_DAILY" "*.tar.gz"

# Cleanup config backups
cleanup_directory "$BACKUP_DIR/config" "$RETENTION_MONTHLY" "config_*.tar.gz"

# Cleanup pre-restore snapshots (keep for 7 days)
if [ -d "$BACKUP_DIR/pre-restore-snapshots" ]; then
    cleanup_directory "$BACKUP_DIR/pre-restore-snapshots" 7 "pre_restore_*.tar.gz"
fi

# Report
freed_mb=$((freed_space / 1024 / 1024))

log_info "========================================="
log_info "Cleanup Summary"
log_info "Files deleted: $deleted_count"
log_info "Files kept: $kept_count"
log_info "Space freed: ${freed_mb}MB"
log_info "========================================="

exit 0
