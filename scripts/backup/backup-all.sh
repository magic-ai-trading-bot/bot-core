#!/usr/bin/env bash

# @spec:NFR-OPS-004 - Complete System Backup
# @ref:specs/05-operations/5.3-disaster-recovery/DR-PLAN.md

set -euo pipefail

# Script: backup-all.sh
# Purpose: Complete system backup (database + volumes + config)
# Usage: ./backup-all.sh
# Version: 1.0.0

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
LOG_FILE="${LOG_DIR:-/var/log/backups}/backup-all-$(date +%Y%m%d).log"

log_info() {
    echo "[$(date '+%Y-%m-%d %H:%M:%S')] [INFO] $*" | tee -a "$LOG_FILE"
}

log_error() {
    echo "[$(date '+%Y-%m-%d %H:%M:%S')] [ERROR] $*" | tee -a "$LOG_FILE"
}

mkdir -p "$(dirname "$LOG_FILE")"

log_info "========================================="
log_info "Starting complete system backup"
log_info "========================================="

start_time=$(date +%s)
success_count=0
fail_count=0

# 1. Backup MongoDB
log_info "Step 1/3: Backing up MongoDB..."
if bash "$SCRIPT_DIR/backup-mongodb.sh" full; then
    log_info "MongoDB backup: SUCCESS"
    ((success_count++))
else
    log_error "MongoDB backup: FAILED"
    ((fail_count++))
fi

# 2. Backup volumes
log_info "Step 2/3: Backing up Docker volumes..."
if bash "$SCRIPT_DIR/backup-volumes.sh"; then
    log_info "Volume backup: SUCCESS"
    ((success_count++))
else
    log_error "Volume backup: FAILED"
    ((fail_count++))
fi

# 3. Backup configuration
log_info "Step 3/3: Backing up configuration..."
if bash "$SCRIPT_DIR/backup-config.sh"; then
    log_info "Config backup: SUCCESS"
    ((success_count++))
else
    log_error "Config backup: FAILED"
    ((fail_count++))
fi

end_time=$(date +%s)
duration=$((end_time - start_time))

log_info "========================================="
log_info "Complete system backup finished"
log_info "Duration: ${duration}s"
log_info "Success: $success_count, Failed: $fail_count"
log_info "========================================="

# Send notification
if [ "$fail_count" -eq 0 ]; then
    log_info "All backups completed successfully"
    exit 0
else
    log_error "Some backups failed - check logs"
    exit 1
fi
