#!/usr/bin/env bash

# @spec:NFR-OPS-010 - Google Cloud Storage Backup Upload
# @ref:specs/05-operations/5.3-disaster-recovery/DR-PLAN.md

set -euo pipefail

BACKUP_FILE="$1"
GCS_BUCKET="${GCS_BACKUP_BUCKET:-gs://bot-core-backups}"
LOG_FILE="${LOG_DIR:-/var/log/backups}/gcs-upload-$(date +%Y%m%d).log"

log_info() {
    echo "[$(date '+%Y-%m-%d %H:%M:%S')] [INFO] $*" | tee -a "$LOG_FILE"
}

mkdir -p "$(dirname "$LOG_FILE")"

if [ -z "$BACKUP_FILE" ]; then
    echo "Usage: $0 <backup-file>"
    exit 1
fi

if ! command -v gsutil &>/dev/null; then
    log_info "gsutil not found - install Google Cloud SDK"
    exit 1
fi

log_info "Uploading to GCS: $BACKUP_FILE -> $GCS_BUCKET"

BACKUP_TYPE=$(basename "$BACKUP_FILE" | cut -d_ -f1-2)
YEAR=$(date +%Y)
MONTH=$(date +%m)
GCS_PATH="$GCS_BUCKET/$BACKUP_TYPE/$YEAR/$MONTH/$(basename "$BACKUP_FILE")"

# Upload with parallel composite uploads
gsutil -m cp "$BACKUP_FILE" "$GCS_PATH" 2>&1 | tee -a "$LOG_FILE"

if [ ${PIPESTATUS[0]} -eq 0 ]; then
    log_info "Upload successful: $GCS_PATH"

    # Upload checksum
    if [ -f "${BACKUP_FILE}.sha256" ]; then
        gsutil cp "${BACKUP_FILE}.sha256" "${GCS_PATH}.sha256" 2>&1 | tee -a "$LOG_FILE"
    fi

    # Set storage class to Nearline for cost optimization
    gsutil rewrite -s NEARLINE "$GCS_PATH"

    log_info "GCS upload complete"
    exit 0
else
    log_info "Upload failed"
    exit 1
fi
