#!/usr/bin/env bash

# @spec:NFR-OPS-003 - Configuration Backup System
# @ref:specs/05-operations/5.3-disaster-recovery/DR-PLAN.md

set -euo pipefail

# Script: backup-config.sh
# Purpose: Backup all configuration files
# Usage: ./backup-config.sh
# Version: 1.0.0

TIMESTAMP=$(date +%Y%m%d_%H%M%S)
BACKUP_DIR="${BACKUP_DIR:-/backups}/config"
LOG_FILE="${LOG_DIR:-/var/log/backups}/config-backup-$(date +%Y%m%d).log"

log_info() {
    echo "[$(date '+%Y-%m-%d %H:%M:%S')] [INFO] $*" | tee -a "$LOG_FILE"
}

mkdir -p "$BACKUP_DIR" "$(dirname "$LOG_FILE")"

log_info "Starting configuration backup..."

# Configuration files to backup
CONFIG_FILES=(
    "docker-compose.yml"
    "docker-compose.dev.yml"
    "docker-compose.prod.yml"
    ".env.example"
    "Makefile"
    "rust-core-engine/config.toml"
    "rust-core-engine/Cargo.toml"
    "python-ai-service/config.yaml"
    "python-ai-service/requirements.txt"
    "python-ai-service/pyproject.toml"
    "nextjs-ui-dashboard/package.json"
    "nextjs-ui-dashboard/vite.config.ts"
    "nextjs-ui-dashboard/tailwind.config.ts"
)

# Configuration directories
CONFIG_DIRS=(
    "infrastructure/"
    ".github/"
    "scripts/"
    ".claude/"
)

backup_file="$BACKUP_DIR/config_${TIMESTAMP}.tar.gz"

log_info "Creating config backup: $backup_file"

# Create tar archive
tar czf "$backup_file" \
    "${CONFIG_FILES[@]}" \
    "${CONFIG_DIRS[@]}" \
    2>&1 | tee -a "$LOG_FILE" || true

if [ -f "$backup_file" ]; then
    sha256sum "$backup_file" > "${backup_file}.sha256"
    log_info "Configuration backup complete: $backup_file"
    log_info "Size: $(du -h "$backup_file" | awk '{print $1}')"

    # Cleanup old backups (keep last 30 days)
    find "$BACKUP_DIR" -name "config_*.tar.gz" -mtime +30 -delete
    find "$BACKUP_DIR" -name "*.sha256" -mtime +30 -delete

    exit 0
else
    log_info "Configuration backup failed"
    exit 1
fi
