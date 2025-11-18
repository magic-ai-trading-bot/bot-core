#!/usr/bin/env bash

# @spec:NFR-OPS-002 - Docker Volume Backup System
# @ref:specs/05-operations/5.3-disaster-recovery/DR-PLAN.md

set -euo pipefail

# Script: backup-volumes.sh
# Purpose: Backup Docker volumes (data, logs, models, config)
# Usage: ./backup-volumes.sh
# Version: 1.0.0

TIMESTAMP=$(date +%Y%m%d_%H%M%S)
BACKUP_DIR="${BACKUP_DIR:-/backups}/volumes"
LOG_FILE="${LOG_DIR:-/var/log/backups}/volumes-backup-$(date +%Y%m%d).log"

log_info() {
    echo "[$(date '+%Y-%m-%d %H:%M:%S')] [INFO] $*" | tee -a "$LOG_FILE"
}

log_error() {
    echo "[$(date '+%Y-%m-%d %H:%M:%S')] [ERROR] $*" | tee -a "$LOG_FILE"
}

mkdir -p "$BACKUP_DIR" "$(dirname "$LOG_FILE")"

log_info "Starting Docker volumes backup..."

# List of critical volumes to backup
VOLUMES=(
    "rust_target_cache"
    "redis_data"
    "rabbitmq_data"
    "kong_data"
    "prometheus_data"
    "grafana_data"
)

# List of critical directories to backup
DIRECTORIES=(
    "./python-ai-service/models"
    "./python-ai-service/data"
    "./python-ai-service/logs"
    "./rust-core-engine/data"
    "./rust-core-engine/logs"
    "./nextjs-ui-dashboard/logs"
)

backup_success=0
backup_failed=0

# Backup Docker volumes
for volume in "${VOLUMES[@]}"; do
    log_info "Backing up volume: $volume"

    if docker volume inspect "$volume" &>/dev/null; then
        backup_file="$BACKUP_DIR/${volume}_${TIMESTAMP}.tar.gz"

        docker run --rm \
            -v "$volume:/data" \
            -v "$BACKUP_DIR:/backup" \
            alpine \
            tar czf "/backup/$(basename "$backup_file")" -C /data . 2>&1 | tee -a "$LOG_FILE"

        if [ -f "$backup_file" ]; then
            sha256sum "$backup_file" > "${backup_file}.sha256"
            log_info "Volume $volume backed up: $backup_file"
            ((backup_success++))
        else
            log_error "Failed to backup volume: $volume"
            ((backup_failed++))
        fi
    else
        log_error "Volume not found: $volume"
        ((backup_failed++))
    fi
done

# Backup directories
for dir in "${DIRECTORIES[@]}"; do
    if [ -d "$dir" ]; then
        log_info "Backing up directory: $dir"

        dir_name=$(echo "$dir" | tr '/' '_' | sed 's/^\._//')
        backup_file="$BACKUP_DIR/${dir_name}_${TIMESTAMP}.tar.gz"

        tar czf "$backup_file" "$dir" 2>&1 | tee -a "$LOG_FILE"

        if [ -f "$backup_file" ]; then
            sha256sum "$backup_file" > "${backup_file}.sha256"
            log_info "Directory $dir backed up: $backup_file"
            ((backup_success++))
        else
            log_error "Failed to backup directory: $dir"
            ((backup_failed++))
        fi
    else
        log_info "Directory not found (skipping): $dir"
    fi
done

# Cleanup old backups (keep last 7 days)
find "$BACKUP_DIR" -name "*.tar.gz" -mtime +7 -delete
find "$BACKUP_DIR" -name "*.sha256" -mtime +7 -delete

log_info "Volume backup complete: $backup_success succeeded, $backup_failed failed"

[ $backup_failed -eq 0 ] && exit 0 || exit 1
