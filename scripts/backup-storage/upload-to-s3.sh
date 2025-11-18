#!/usr/bin/env bash

# @spec:NFR-OPS-009 - AWS S3 Backup Upload
# @ref:specs/05-operations/5.3-disaster-recovery/DR-PLAN.md

set -euo pipefail

BACKUP_FILE="$1"
S3_BUCKET="${S3_BACKUP_BUCKET:-s3://bot-core-backups}"
AWS_REGION="${AWS_REGION:-us-east-1}"
LOG_FILE="${LOG_DIR:-/var/log/backups}/s3-upload-$(date +%Y%m%d).log"

log_info() {
    echo "[$(date '+%Y-%m-%d %H:%M:%S')] [INFO] $*" | tee -a "$LOG_FILE"
}

mkdir -p "$(dirname "$LOG_FILE")"

if [ -z "$BACKUP_FILE" ]; then
    echo "Usage: $0 <backup-file>"
    exit 1
fi

if [ ! -f "$BACKUP_FILE" ]; then
    log_info "Backup file not found: $BACKUP_FILE"
    exit 1
fi

# Check AWS CLI
if ! command -v aws &>/dev/null; then
    log_info "AWS CLI not found - install with: pip install awscli"
    exit 1
fi

log_info "Uploading to S3: $BACKUP_FILE -> $S3_BUCKET"

# Determine S3 path based on backup type and date
BACKUP_TYPE=$(basename "$BACKUP_FILE" | cut -d_ -f1-2)  # mongodb_full or mongodb_incremental
YEAR=$(date +%Y)
MONTH=$(date +%m)
S3_PATH="$S3_BUCKET/$BACKUP_TYPE/$YEAR/$MONTH/$(basename "$BACKUP_FILE")"

# Upload with server-side encryption
aws s3 cp "$BACKUP_FILE" "$S3_PATH" \
    --region "$AWS_REGION" \
    --storage-class STANDARD_IA \
    --server-side-encryption AES256 \
    --metadata "backup-date=$(date -u +%Y-%m-%dT%H:%M:%SZ),hostname=$(hostname)" \
    2>&1 | tee -a "$LOG_FILE"

if [ ${PIPESTATUS[0]} -eq 0 ]; then
    log_info "Upload successful: $S3_PATH"

    # Upload checksum if exists
    if [ -f "${BACKUP_FILE}.sha256" ]; then
        aws s3 cp "${BACKUP_FILE}.sha256" "${S3_PATH}.sha256" \
            --region "$AWS_REGION" 2>&1 | tee -a "$LOG_FILE"
    fi

    # Verify upload
    if aws s3 ls "$S3_PATH" &>/dev/null; then
        log_info "Verification: File exists in S3"
        exit 0
    else
        log_info "Verification failed: File not found in S3"
        exit 1
    fi
else
    log_info "Upload failed"
    exit 1
fi
