#!/usr/bin/env bash

# @spec:NFR-OPS-011 - SFTP Backup Upload
# @ref:specs/05-operations/5.3-disaster-recovery/DR-PLAN.md

set -euo pipefail

BACKUP_FILE="$1"
SFTP_HOST="${SFTP_HOST:-backup.example.com}"
SFTP_USER="${SFTP_USER:-bot-core-backup}"
SFTP_PORT="${SFTP_PORT:-22}"
SFTP_DIR="${SFTP_DIR:-/backups/bot-core}"
SSH_KEY="${SFTP_SSH_KEY:-~/.ssh/backup_key}"
LOG_FILE="${LOG_DIR:-/var/log/backups}/sftp-upload-$(date +%Y%m%d).log"

log_info() {
    echo "[$(date '+%Y-%m-%d %H:%M:%S')] [INFO] $*" | tee -a "$LOG_FILE"
}

mkdir -p "$(dirname "$LOG_FILE")"

if [ -z "$BACKUP_FILE" ]; then
    echo "Usage: $0 <backup-file>"
    exit 1
fi

if ! command -v sftp &>/dev/null; then
    log_info "sftp not found"
    exit 1
fi

log_info "Uploading to SFTP: $BACKUP_FILE -> $SFTP_USER@$SFTP_HOST:$SFTP_DIR"

# Determine remote path
BACKUP_TYPE=$(basename "$BACKUP_FILE" | cut -d_ -f1-2)
YEAR=$(date +%Y)
MONTH=$(date +%m)
REMOTE_DIR="$SFTP_DIR/$BACKUP_TYPE/$YEAR/$MONTH"
REMOTE_FILE="$REMOTE_DIR/$(basename "$BACKUP_FILE")"

# Create remote directory and upload
sftp -P "$SFTP_PORT" -i "$SSH_KEY" "$SFTP_USER@$SFTP_HOST" <<EOF | tee -a "$LOG_FILE"
mkdir $SFTP_DIR
mkdir $SFTP_DIR/$BACKUP_TYPE
mkdir $SFTP_DIR/$BACKUP_TYPE/$YEAR
mkdir $SFTP_DIR/$BACKUP_TYPE/$YEAR/$MONTH
put $BACKUP_FILE $REMOTE_FILE
bye
EOF

if [ ${PIPESTATUS[0]} -eq 0 ]; then
    log_info "Upload successful: $REMOTE_FILE"

    # Upload checksum
    if [ -f "${BACKUP_FILE}.sha256" ]; then
        sftp -P "$SFTP_PORT" -i "$SSH_KEY" "$SFTP_USER@$SFTP_HOST" <<EOF
put ${BACKUP_FILE}.sha256 ${REMOTE_FILE}.sha256
bye
EOF
    fi

    exit 0
else
    log_info "Upload failed"
    exit 1
fi
