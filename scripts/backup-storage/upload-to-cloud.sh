#!/usr/bin/env bash

# @spec:NFR-OPS-012 - Universal Cloud Upload Dispatcher
# @ref:specs/05-operations/5.3-disaster-recovery/DR-PLAN.md

set -euo pipefail

BACKUP_FILE="$1"
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
LOG_FILE="${LOG_DIR:-/var/log/backups}/cloud-upload-$(date +%Y%m%d).log"

log_info() {
    echo "[$(date '+%Y-%m-%d %H:%M:%S')] [INFO] $*" | tee -a "$LOG_FILE"
}

mkdir -p "$(dirname "$LOG_FILE")"

if [ -z "$BACKUP_FILE" ]; then
    echo "Usage: $0 <backup-file>"
    exit 1
fi

# Configuration
ENABLE_S3="${ENABLE_S3_BACKUP:-false}"
ENABLE_GCS="${ENABLE_GCS_BACKUP:-false}"
ENABLE_SFTP="${ENABLE_SFTP_BACKUP:-false}"

log_info "Starting cloud upload for: $BACKUP_FILE"

success=0
failed=0

# Upload to S3
if [ "$ENABLE_S3" = "true" ]; then
    log_info "Uploading to AWS S3..."
    if bash "$SCRIPT_DIR/upload-to-s3.sh" "$BACKUP_FILE"; then
        log_info "S3 upload: SUCCESS"
        ((success++))
    else
        log_info "S3 upload: FAILED"
        ((failed++))
    fi
fi

# Upload to GCS
if [ "$ENABLE_GCS" = "true" ]; then
    log_info "Uploading to Google Cloud Storage..."
    if bash "$SCRIPT_DIR/upload-to-gcs.sh" "$BACKUP_FILE"; then
        log_info "GCS upload: SUCCESS"
        ((success++))
    else
        log_info "GCS upload: FAILED"
        ((failed++))
    fi
fi

# Upload to SFTP
if [ "$ENABLE_SFTP" = "true" ]; then
    log_info "Uploading to SFTP..."
    if bash "$SCRIPT_DIR/upload-to-sftp.sh" "$BACKUP_FILE"; then
        log_info "SFTP upload: SUCCESS"
        ((success++))
    else
        log_info "SFTP upload: FAILED"
        ((failed++))
    fi
fi

log_info "Cloud upload complete: $success succeeded, $failed failed"

[ $failed -eq 0 ] && exit 0 || exit 1
