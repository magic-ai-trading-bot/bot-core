#!/usr/bin/env bash

# @spec:NFR-OPS-001 - Automated MongoDB Backup System
# @ref:specs/05-operations/5.3-disaster-recovery/DR-PLAN.md
# @test:TC-DR-001, TC-DR-002

set -euo pipefail

# Script: backup-mongodb.sh
# Purpose: Automated MongoDB backup with compression, encryption, verification
# Usage: ./backup-mongodb.sh [full|incremental]
# Version: 1.0.0
# Last Updated: 2025-11-18

#=============================================================================
# CONFIGURATION
#=============================================================================

# Load environment variables
if [ -f "$(dirname "$0")/../../.env" ]; then
    # shellcheck disable=SC1091
    source "$(dirname "$0")/../../.env"
fi

# Backup configuration
BACKUP_TYPE="${1:-full}"  # full or incremental
TIMESTAMP=$(date +%Y%m%d_%H%M%S)
BACKUP_BASE_DIR="${BACKUP_DIR:-/backups}"
BACKUP_DIR="$BACKUP_BASE_DIR/mongodb"
TEMP_DIR="/tmp/mongodb-backup-$$"
LOG_DIR="${LOG_DIR:-/var/log/backups}"
LOG_FILE="$LOG_DIR/mongodb-backup-$(date +%Y%m%d).log"

# MongoDB configuration
MONGO_CONTAINER="${MONGO_CONTAINER:-mongodb-primary}"
DATABASE_URL="${DATABASE_URL:-mongodb://admin:password@localhost:27017/bot_core}"
DB_NAME="${DB_NAME:-bot_core}"

# Compression & encryption
COMPRESSION="${COMPRESSION:-gzip}"  # gzip, bzip2, xz
ENCRYPTION_ENABLED="${ENCRYPTION_ENABLED:-true}"
GPG_RECIPIENT="${GPG_RECIPIENT:-backup@botcore.app}"
GPG_PASSPHRASE_FILE="${GPG_PASSPHRASE_FILE:-/etc/backup-secrets/gpg-passphrase}"

# Retention policy
RETENTION_DAILY="${RETENTION_DAILY:-7}"      # Keep 7 daily backups
RETENTION_WEEKLY="${RETENTION_WEEKLY:-4}"     # Keep 4 weekly backups
RETENTION_MONTHLY="${RETENTION_MONTHLY:-12}"  # Keep 12 monthly backups

# Notifications
SLACK_WEBHOOK="${SLACK_WEBHOOK:-}"
EMAIL_RECIPIENT="${EMAIL_RECIPIENT:-ops@botcore.app}"
ENABLE_NOTIFICATIONS="${ENABLE_NOTIFICATIONS:-true}"

# Monitoring
HEALTHCHECK_URL="${HEALTHCHECK_URL:-}"  # Optional: healthchecks.io or similar
METRICS_FILE="/var/lib/prometheus/node_exporter/backup_metrics.prom"

#=============================================================================
# LOGGING FUNCTIONS
#=============================================================================

log() {
    local level="$1"
    shift
    local message="$*"
    local timestamp
    timestamp=$(date '+%Y-%m-%d %H:%M:%S')
    echo "[$timestamp] [$level] $message" | tee -a "$LOG_FILE"
}

log_info() {
    log "INFO" "$@"
}

log_warn() {
    log "WARN" "$@"
}

log_error() {
    log "ERROR" "$@"
}

log_success() {
    log "SUCCESS" "$@"
}

#=============================================================================
# NOTIFICATION FUNCTIONS
#=============================================================================

send_slack_notification() {
    local status="$1"
    local message="$2"

    if [ "$ENABLE_NOTIFICATIONS" != "true" ] || [ -z "$SLACK_WEBHOOK" ]; then
        return 0
    fi

    local color="good"
    [ "$status" = "error" ] && color="danger"
    [ "$status" = "warning" ] && color="warning"

    curl -X POST "$SLACK_WEBHOOK" \
        -H 'Content-Type: application/json' \
        -d "{
            \"attachments\": [{
                \"color\": \"$color\",
                \"title\": \"MongoDB Backup - $status\",
                \"text\": \"$message\",
                \"footer\": \"Bot-Core Backup System\",
                \"ts\": $(date +%s)
            }]
        }" 2>&1 | tee -a "$LOG_FILE" || true
}

send_email_notification() {
    local subject="$1"
    local body="$2"

    if [ "$ENABLE_NOTIFICATIONS" != "true" ] || [ -z "$EMAIL_RECIPIENT" ]; then
        return 0
    fi

    echo "$body" | mail -s "$subject" "$EMAIL_RECIPIENT" || true
}

ping_healthcheck() {
    local status="$1"  # start, success, fail

    if [ -z "$HEALTHCHECK_URL" ]; then
        return 0
    fi

    local url="$HEALTHCHECK_URL"
    [ "$status" = "fail" ] && url="${HEALTHCHECK_URL}/fail"

    curl -fsS -m 10 --retry 3 "$url" >/dev/null 2>&1 || true
}

#=============================================================================
# METRICS FUNCTIONS
#=============================================================================

update_metrics() {
    local backup_success="$1"
    local backup_size="$2"
    local backup_duration="$3"

    mkdir -p "$(dirname "$METRICS_FILE")"

    cat > "$METRICS_FILE" <<EOF
# HELP mongodb_backup_last_success_timestamp Last successful backup timestamp
# TYPE mongodb_backup_last_success_timestamp gauge
mongodb_backup_last_success_timestamp{type="$BACKUP_TYPE"} $(date +%s)

# HELP mongodb_backup_success Last backup status (1=success, 0=failure)
# TYPE mongodb_backup_success gauge
mongodb_backup_success{type="$BACKUP_TYPE"} $backup_success

# HELP mongodb_backup_size_bytes Backup size in bytes
# TYPE mongodb_backup_size_bytes gauge
mongodb_backup_size_bytes{type="$BACKUP_TYPE"} $backup_size

# HELP mongodb_backup_duration_seconds Backup duration in seconds
# TYPE mongodb_backup_duration_seconds gauge
mongodb_backup_duration_seconds{type="$BACKUP_TYPE"} $backup_duration
EOF
}

#=============================================================================
# BACKUP FUNCTIONS
#=============================================================================

setup_directories() {
    log_info "Setting up directories..."

    mkdir -p "$BACKUP_DIR"
    mkdir -p "$LOG_DIR"
    mkdir -p "$TEMP_DIR"

    chmod 700 "$BACKUP_DIR"
    chmod 700 "$TEMP_DIR"

    log_success "Directories created"
}

check_prerequisites() {
    log_info "Checking prerequisites..."

    # Check if MongoDB container is running
    if ! docker ps | grep -q "$MONGO_CONTAINER"; then
        log_error "MongoDB container '$MONGO_CONTAINER' is not running"
        return 1
    fi

    # Check disk space (need at least 10GB free)
    local available_space
    available_space=$(df -BG "$BACKUP_DIR" | awk 'NR==2 {print $4}' | sed 's/G//')

    if [ "$available_space" -lt 10 ]; then
        log_error "Insufficient disk space: ${available_space}GB available, need at least 10GB"
        return 1
    fi

    # Check required tools
    local required_tools="docker mongodump tar"
    [ "$COMPRESSION" = "gzip" ] && required_tools="$required_tools gzip"
    [ "$COMPRESSION" = "bzip2" ] && required_tools="$required_tools bzip2"
    [ "$COMPRESSION" = "xz" ] && required_tools="$required_tools xz"
    [ "$ENCRYPTION_ENABLED" = "true" ] && required_tools="$required_tools gpg"

    for tool in $required_tools; do
        if ! command -v "$tool" &> /dev/null; then
            log_error "Required tool not found: $tool"
            return 1
        fi
    done

    log_success "All prerequisites met"
    return 0
}

perform_backup() {
    log_info "Starting $BACKUP_TYPE backup..."

    local backup_name="mongodb_${BACKUP_TYPE}_${TIMESTAMP}"
    local backup_path="$TEMP_DIR/$backup_name"

    mkdir -p "$backup_path"

    # Perform mongodump
    local dump_cmd="mongodump --uri='$DATABASE_URL' --out='$backup_path' --gzip"

    if [ "$BACKUP_TYPE" = "incremental" ]; then
        # For incremental, use oplog
        dump_cmd="$dump_cmd --oplog"
    fi

    log_info "Executing: mongodump (credentials hidden)"

    if docker exec "$MONGO_CONTAINER" bash -c "$dump_cmd" 2>&1 | tee -a "$LOG_FILE"; then
        log_success "MongoDB dump completed"
    else
        log_error "MongoDB dump failed"
        return 1
    fi

    # Create metadata file
    cat > "$backup_path/backup-metadata.json" <<EOF
{
    "backup_type": "$BACKUP_TYPE",
    "timestamp": "$TIMESTAMP",
    "database": "$DB_NAME",
    "hostname": "$(hostname)",
    "mongodb_version": "$(docker exec "$MONGO_CONTAINER" mongosh --quiet --eval 'db.version()')",
    "backup_size_mb": $(du -sm "$backup_path" | awk '{print $1}'),
    "created_at": "$(date -u +%Y-%m-%dT%H:%M:%SZ)"
}
EOF

    echo "$backup_path"
    return 0
}

compress_backup() {
    local backup_path="$1"
    local compressed_file

    log_info "Compressing backup with $COMPRESSION..."

    case "$COMPRESSION" in
        gzip)
            compressed_file="${backup_path}.tar.gz"
            tar czf "$compressed_file" -C "$(dirname "$backup_path")" "$(basename "$backup_path")"
            ;;
        bzip2)
            compressed_file="${backup_path}.tar.bz2"
            tar cjf "$compressed_file" -C "$(dirname "$backup_path")" "$(basename "$backup_path")"
            ;;
        xz)
            compressed_file="${backup_path}.tar.xz"
            tar cJf "$compressed_file" -C "$(dirname "$backup_path")" "$(basename "$backup_path")"
            ;;
        *)
            log_error "Unknown compression: $COMPRESSION"
            return 1
            ;;
    esac

    if [ -f "$compressed_file" ]; then
        local original_size
        local compressed_size
        original_size=$(du -sm "$backup_path" | awk '{print $1}')
        compressed_size=$(du -sm "$compressed_file" | awk '{print $1}')

        log_success "Backup compressed: ${original_size}MB -> ${compressed_size}MB"

        # Remove uncompressed backup
        rm -rf "$backup_path"

        echo "$compressed_file"
        return 0
    else
        log_error "Compression failed"
        return 1
    fi
}

encrypt_backup() {
    local compressed_file="$1"
    local encrypted_file="${compressed_file}.gpg"

    if [ "$ENCRYPTION_ENABLED" != "true" ]; then
        echo "$compressed_file"
        return 0
    fi

    log_info "Encrypting backup..."

    if [ -f "$GPG_PASSPHRASE_FILE" ]; then
        # Encrypt with passphrase
        gpg --batch --yes --passphrase-file "$GPG_PASSPHRASE_FILE" \
            --symmetric --cipher-algo AES256 \
            --output "$encrypted_file" "$compressed_file"
    elif [ -n "$GPG_RECIPIENT" ]; then
        # Encrypt with public key
        gpg --batch --yes --recipient "$GPG_RECIPIENT" \
            --encrypt --output "$encrypted_file" "$compressed_file"
    else
        log_error "No encryption method configured"
        return 1
    fi

    if [ -f "$encrypted_file" ]; then
        log_success "Backup encrypted"

        # Remove unencrypted file
        shred -u "$compressed_file"

        echo "$encrypted_file"
        return 0
    else
        log_error "Encryption failed"
        return 1
    fi
}

verify_backup() {
    local backup_file="$1"

    log_info "Verifying backup integrity..."

    # Check file exists and is not empty
    if [ ! -f "$backup_file" ]; then
        log_error "Backup file not found: $backup_file"
        return 1
    fi

    if [ ! -s "$backup_file" ]; then
        log_error "Backup file is empty: $backup_file"
        return 1
    fi

    # Generate and store checksum
    local checksum_file="${backup_file}.sha256"
    sha256sum "$backup_file" > "$checksum_file"

    log_info "Checksum: $(cat "$checksum_file")"
    log_success "Backup verification complete"

    return 0
}

move_to_final_location() {
    local backup_file="$1"
    local final_dir="$BACKUP_DIR"

    # Organize by date
    local year month day
    year=$(date +%Y)
    month=$(date +%m)
    day=$(date +%d)

    final_dir="$final_dir/$year/$month"
    mkdir -p "$final_dir"

    log_info "Moving backup to final location: $final_dir"

    local final_path="$final_dir/$(basename "$backup_file")"
    mv "$backup_file" "$final_path"

    # Move checksum too
    if [ -f "${backup_file}.sha256" ]; then
        mv "${backup_file}.sha256" "${final_path}.sha256"
    fi

    log_success "Backup saved: $final_path"

    echo "$final_path"
    return 0
}

cleanup_old_backups() {
    log_info "Cleaning up old backups per retention policy..."

    # Daily backups (keep last 7)
    find "$BACKUP_DIR" -name "mongodb_full_*.gpg" -mtime +$RETENTION_DAILY -delete

    # Weekly backups (keep on Sundays for 4 weeks)
    # TODO: Implement weekly logic

    # Monthly backups (keep first of month for 12 months)
    # TODO: Implement monthly logic

    log_success "Cleanup complete"
}

#=============================================================================
# MAIN EXECUTION
#=============================================================================

main() {
    local start_time
    start_time=$(date +%s)

    log_info "==================================================================="
    log_info "MongoDB Backup Started - Type: $BACKUP_TYPE"
    log_info "==================================================================="

    ping_healthcheck "start"

    # Setup
    setup_directories || exit 1
    check_prerequisites || exit 1

    # Perform backup
    local backup_path
    if ! backup_path=$(perform_backup); then
        log_error "Backup failed"
        send_slack_notification "error" "MongoDB backup failed during dump phase"
        send_email_notification "Backup Failed" "MongoDB backup failed. Check logs: $LOG_FILE"
        update_metrics 0 0 0
        ping_healthcheck "fail"
        exit 1
    fi

    # Compress
    local compressed_file
    if ! compressed_file=$(compress_backup "$backup_path"); then
        log_error "Compression failed"
        send_slack_notification "error" "MongoDB backup failed during compression"
        update_metrics 0 0 0
        ping_healthcheck "fail"
        exit 1
    fi

    # Encrypt
    local final_file
    if ! final_file=$(encrypt_backup "$compressed_file"); then
        log_error "Encryption failed"
        send_slack_notification "error" "MongoDB backup failed during encryption"
        update_metrics 0 0 0
        ping_healthcheck "fail"
        exit 1
    fi

    # Verify
    if ! verify_backup "$final_file"; then
        log_error "Verification failed"
        send_slack_notification "error" "MongoDB backup failed verification"
        update_metrics 0 0 0
        ping_healthcheck "fail"
        exit 1
    fi

    # Move to final location
    local final_path
    if ! final_path=$(move_to_final_location "$final_file"); then
        log_error "Failed to move backup to final location"
        send_slack_notification "error" "MongoDB backup failed during finalization"
        update_metrics 0 0 0
        ping_healthcheck "fail"
        exit 1
    fi

    # Upload to cloud (if configured)
    if [ -f "$(dirname "$0")/../backup-storage/upload-to-cloud.sh" ]; then
        log_info "Uploading to cloud storage..."
        bash "$(dirname "$0")/../backup-storage/upload-to-cloud.sh" "$final_path" || log_warn "Cloud upload failed"
    fi

    # Cleanup
    cleanup_old_backups
    rm -rf "$TEMP_DIR"

    # Calculate metrics
    local end_time duration backup_size
    end_time=$(date +%s)
    duration=$((end_time - start_time))
    backup_size=$(stat -f%z "$final_path" 2>/dev/null || stat -c%s "$final_path")

    update_metrics 1 "$backup_size" "$duration"
    ping_healthcheck "success"

    log_info "==================================================================="
    log_success "MongoDB Backup Complete"
    log_info "Backup file: $final_path"
    log_info "Backup size: $(du -h "$final_path" | awk '{print $1}')"
    log_info "Duration: ${duration}s"
    log_info "==================================================================="

    send_slack_notification "success" "MongoDB $BACKUP_TYPE backup completed in ${duration}s. Size: $(du -h "$final_path" | awk '{print $1}')"

    exit 0
}

# Run main function
main "$@"
