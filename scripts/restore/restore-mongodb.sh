#!/usr/bin/env bash

# @spec:NFR-OPS-005 - MongoDB Restore System
# @ref:specs/05-operations/5.3-disaster-recovery/DR-PLAN.md

set -euo pipefail

# Script: restore-mongodb.sh
# Purpose: Restore MongoDB from backup with verification
# Usage: ./restore-mongodb.sh [backup-file] [--dry-run]
# Version: 1.0.0

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
LOG_FILE="${LOG_DIR:-/var/log/backups}/restore-$(date +%Y%m%d_%H%M%S).log"
TEMP_DIR="/tmp/mongodb-restore-$$"

# Configuration
MONGO_CONTAINER="${MONGO_CONTAINER:-mongodb-primary}"
DATABASE_URL="${DATABASE_URL:-mongodb://admin:password@localhost:27017/bot_core}"
DB_NAME="${DB_NAME:-bot_core}"
DRY_RUN=false

log_info() {
    echo "[$(date '+%Y-%m-%d %H:%M:%S')] [INFO] $*" | tee -a "$LOG_FILE"
}

log_warn() {
    echo "[$(date '+%Y-%m-%d %H:%M:%S')] [WARN] $*" | tee -a "$LOG_FILE"
}

log_error() {
    echo "[$(date '+%Y-%m-%d %H:%M:%S')] [ERROR] $*" | tee -a "$LOG_FILE"
}

log_success() {
    echo "[$(date '+%Y-%m-%d %H:%M:%S')] [SUCCESS] $*" | tee -a "$LOG_FILE"
}

mkdir -p "$(dirname "$LOG_FILE")" "$TEMP_DIR"

show_usage() {
    cat <<EOF
Usage: $0 [OPTIONS] [BACKUP_FILE]

Restore MongoDB database from backup file.

OPTIONS:
    --dry-run           Validate backup without restoring
    --list              List available backups
    --help              Show this help message

EXAMPLES:
    # Interactive mode (select from available backups)
    $0

    # Restore specific backup
    $0 /backups/mongodb/2025/11/mongodb_full_20251118_020000.tar.gz.gpg

    # Dry run (validation only)
    $0 --dry-run /backups/mongodb/2025/11/mongodb_full_20251118_020000.tar.gz.gpg

    # List available backups
    $0 --list
EOF
}

list_available_backups() {
    local backup_dir="${BACKUP_DIR:-/backups}/mongodb"

    log_info "Available backups:"
    echo ""

    if [ ! -d "$backup_dir" ]; then
        log_error "Backup directory not found: $backup_dir"
        return 1
    fi

    find "$backup_dir" -name "mongodb_*.tar.gz*" -type f | sort -r | while read -r backup; do
        local size date
        size=$(du -h "$backup" | awk '{print $1}')
        date=$(stat -f%Sm -t "%Y-%m-%d %H:%M:%S" "$backup" 2>/dev/null || \
               stat -c%y "$backup" | cut -d. -f1)

        echo "  [$date] $backup ($size)"
    done

    echo ""
}

select_backup_interactive() {
    local backup_dir="${BACKUP_DIR:-/backups}/mongodb"

    if [ ! -d "$backup_dir" ]; then
        log_error "Backup directory not found: $backup_dir"
        return 1
    fi

    log_info "Available backups (newest first):"
    echo ""

    mapfile -t backups < <(find "$backup_dir" -name "mongodb_*.tar.gz*" -type f | sort -r | head -20)

    if [ ${#backups[@]} -eq 0 ]; then
        log_error "No backups found in $backup_dir"
        return 1
    fi

    local i=1
    for backup in "${backups[@]}"; do
        local size date
        size=$(du -h "$backup" | awk '{print $1}')
        date=$(stat -f%Sm -t "%Y-%m-%d %H:%M:%S" "$backup" 2>/dev/null || \
               stat -c%y "$backup" | cut -d. -f1)

        printf "%2d) [%s] %s (%s)\n" "$i" "$date" "$(basename "$backup")" "$size"
        ((i++))
    done

    echo ""
    read -rp "Select backup number (1-${#backups[@]}) or 'q' to quit: " selection

    if [ "$selection" = "q" ]; then
        log_info "Restore cancelled by user"
        exit 0
    fi

    if ! [[ "$selection" =~ ^[0-9]+$ ]] || [ "$selection" -lt 1 ] || [ "$selection" -gt "${#backups[@]}" ]; then
        log_error "Invalid selection"
        return 1
    fi

    echo "${backups[$((selection - 1))]}"
}

verify_backup_file() {
    local backup_file="$1"

    log_info "Verifying backup file: $backup_file"

    if [ ! -f "$backup_file" ]; then
        log_error "Backup file not found: $backup_file"
        return 1
    fi

    # Check checksum if available
    if [ -f "${backup_file}.sha256" ]; then
        log_info "Verifying checksum..."

        if sha256sum -c "${backup_file}.sha256" 2>&1 | tee -a "$LOG_FILE"; then
            log_success "Checksum verification passed"
        else
            log_error "Checksum verification failed"
            return 1
        fi
    else
        log_warn "No checksum file found - skipping verification"
    fi

    log_success "Backup file verified"
    return 0
}

decrypt_backup() {
    local encrypted_file="$1"
    local decrypted_file="${TEMP_DIR}/$(basename "${encrypted_file%.gpg}")"

    if [[ ! "$encrypted_file" =~ \.gpg$ ]]; then
        # Not encrypted
        echo "$encrypted_file"
        return 0
    fi

    log_info "Decrypting backup..."

    local gpg_passphrase_file="${GPG_PASSPHRASE_FILE:-/etc/backup-secrets/gpg-passphrase}"

    if [ -f "$gpg_passphrase_file" ]; then
        gpg --batch --yes --passphrase-file "$gpg_passphrase_file" \
            --decrypt --output "$decrypted_file" "$encrypted_file" 2>&1 | tee -a "$LOG_FILE"
    else
        gpg --batch --yes --decrypt --output "$decrypted_file" "$encrypted_file" 2>&1 | tee -a "$LOG_FILE"
    fi

    if [ -f "$decrypted_file" ]; then
        log_success "Backup decrypted"
        echo "$decrypted_file"
        return 0
    else
        log_error "Decryption failed"
        return 1
    fi
}

decompress_backup() {
    local compressed_file="$1"
    local extract_dir="${TEMP_DIR}/extracted"

    mkdir -p "$extract_dir"

    log_info "Decompressing backup..."

    if tar -tzf "$compressed_file" &>/dev/null; then
        tar -xzf "$compressed_file" -C "$extract_dir" 2>&1 | tee -a "$LOG_FILE"
    elif tar -tjf "$compressed_file" &>/dev/null; then
        tar -xjf "$compressed_file" -C "$extract_dir" 2>&1 | tee -a "$LOG_FILE"
    elif tar -tJf "$compressed_file" &>/dev/null; then
        tar -xJf "$compressed_file" -C "$extract_dir" 2>&1 | tee -a "$LOG_FILE"
    else
        log_error "Unknown compression format"
        return 1
    fi

    if [ -d "$extract_dir" ]; then
        log_success "Backup decompressed"
        echo "$extract_dir"
        return 0
    else
        log_error "Decompression failed"
        return 1
    fi
}

create_pre_restore_snapshot() {
    log_info "Creating pre-restore snapshot..."

    local snapshot_dir="${BACKUP_DIR:-/backups}/pre-restore-snapshots"
    local snapshot_name="pre_restore_$(date +%Y%m%d_%H%M%S)"

    mkdir -p "$snapshot_dir"

    log_info "Taking snapshot: $snapshot_name"

    docker exec "$MONGO_CONTAINER" mongodump \
        --uri="$DATABASE_URL" \
        --out="/tmp/$snapshot_name" \
        --gzip 2>&1 | tee -a "$LOG_FILE"

    # Copy snapshot out of container
    docker cp "$MONGO_CONTAINER:/tmp/$snapshot_name" "$snapshot_dir/"

    if [ -d "$snapshot_dir/$snapshot_name" ]; then
        tar czf "$snapshot_dir/${snapshot_name}.tar.gz" -C "$snapshot_dir" "$snapshot_name"
        rm -rf "$snapshot_dir/$snapshot_name"

        log_success "Pre-restore snapshot created: $snapshot_dir/${snapshot_name}.tar.gz"
        echo "$snapshot_dir/${snapshot_name}.tar.gz"
        return 0
    else
        log_error "Failed to create pre-restore snapshot"
        return 1
    fi
}

perform_restore() {
    local restore_dir="$1"

    log_info "Performing restore from: $restore_dir"

    # Find the actual dump directory
    local dump_dir
    if [ -d "$restore_dir/mongodb_full_"* ]; then
        dump_dir=$(find "$restore_dir" -type d -name "mongodb_full_*" -o -name "mongodb_incremental_*" | head -1)
    elif [ -d "$restore_dir/$DB_NAME" ]; then
        dump_dir="$restore_dir"
    else
        log_error "Cannot find dump directory in: $restore_dir"
        return 1
    fi

    log_info "Dump directory: $dump_dir"

    # Copy dump to container
    docker cp "$dump_dir" "$MONGO_CONTAINER:/tmp/restore/"

    # Perform restore
    log_info "Running mongorestore..."

    docker exec "$MONGO_CONTAINER" mongorestore \
        --uri="$DATABASE_URL" \
        --dir="/tmp/restore/$(basename "$dump_dir")" \
        --drop \
        --gzip 2>&1 | tee -a "$LOG_FILE"

    if [ $? -eq 0 ]; then
        log_success "Restore completed successfully"
        return 0
    else
        log_error "Restore failed"
        return 1
    fi
}

verify_restore() {
    log_info "Verifying restored data..."

    # Run verification queries
    docker exec "$MONGO_CONTAINER" mongosh --quiet --eval "
        use $DB_NAME;

        print('Database: $DB_NAME');
        print('Collections:');
        db.getCollectionNames().forEach(function(name) {
            print('  - ' + name + ': ' + db[name].count() + ' documents');
        });

        // Check critical collections
        var critical = ['users', 'trades', 'portfolios', 'strategies'];
        var errors = [];

        critical.forEach(function(coll) {
            if (!db.getCollectionNames().includes(coll)) {
                errors.push('Missing collection: ' + coll);
            } else if (db[coll].count() === 0) {
                errors.push('Empty collection: ' + coll);
            }
        });

        if (errors.length > 0) {
            print('ERRORS:');
            errors.forEach(function(e) { print('  ! ' + e); });
            quit(1);
        } else {
            print('Verification: PASSED');
        }
    " 2>&1 | tee -a "$LOG_FILE"

    if [ ${PIPESTATUS[0]} -eq 0 ]; then
        log_success "Data verification passed"
        return 0
    else
        log_error "Data verification failed"
        return 1
    fi
}

cleanup() {
    log_info "Cleaning up temporary files..."
    rm -rf "$TEMP_DIR"
    docker exec "$MONGO_CONTAINER" rm -rf /tmp/restore 2>/dev/null || true
}

main() {
    local backup_file=""

    # Parse arguments
    while [ $# -gt 0 ]; do
        case "$1" in
            --dry-run)
                DRY_RUN=true
                shift
                ;;
            --list)
                list_available_backups
                exit 0
                ;;
            --help)
                show_usage
                exit 0
                ;;
            *)
                backup_file="$1"
                shift
                ;;
        esac
    done

    log_info "========================================="
    log_info "MongoDB Restore Process"
    log_info "========================================="

    # Select backup if not provided
    if [ -z "$backup_file" ]; then
        log_info "No backup file specified - entering interactive mode"
        if ! backup_file=$(select_backup_interactive); then
            log_error "Failed to select backup"
            exit 1
        fi
    fi

    log_info "Selected backup: $backup_file"

    # Verify backup file
    if ! verify_backup_file "$backup_file"; then
        log_error "Backup verification failed"
        exit 1
    fi

    # Decrypt
    local decrypted_file
    if ! decrypted_file=$(decrypt_backup "$backup_file"); then
        log_error "Decryption failed"
        cleanup
        exit 1
    fi

    # Decompress
    local restore_dir
    if ! restore_dir=$(decompress_backup "$decrypted_file"); then
        log_error "Decompression failed"
        cleanup
        exit 1
    fi

    if [ "$DRY_RUN" = true ]; then
        log_info "DRY RUN: Backup validation successful"
        log_info "Restore directory: $restore_dir"
        ls -lah "$restore_dir"
        cleanup
        exit 0
    fi

    # Confirm restore
    echo ""
    log_warn "WARNING: This will replace the current database with backup data"
    read -rp "Are you sure you want to continue? (yes/no): " confirm

    if [ "$confirm" != "yes" ]; then
        log_info "Restore cancelled by user"
        cleanup
        exit 0
    fi

    # Create pre-restore snapshot
    local snapshot_file
    if ! snapshot_file=$(create_pre_restore_snapshot); then
        log_warn "Failed to create pre-restore snapshot - continue anyway? (yes/no)"
        read -rp "> " continue_without_snapshot

        if [ "$continue_without_snapshot" != "yes" ]; then
            log_info "Restore cancelled"
            cleanup
            exit 0
        fi
    else
        log_info "Rollback snapshot: $snapshot_file"
    fi

    # Perform restore
    if ! perform_restore "$restore_dir"; then
        log_error "Restore failed"

        if [ -n "$snapshot_file" ]; then
            log_info "Rollback snapshot available at: $snapshot_file"
        fi

        cleanup
        exit 1
    fi

    # Verify restore
    if ! verify_restore; then
        log_error "Restore verification failed"

        if [ -n "$snapshot_file" ]; then
            log_info "Rollback snapshot available at: $snapshot_file"
        fi

        cleanup
        exit 1
    fi

    cleanup

    log_info "========================================="
    log_success "MongoDB Restore Complete"
    log_info "Backup: $backup_file"
    log_info "Rollback snapshot: ${snapshot_file:-none}"
    log_info "========================================="

    exit 0
}

trap cleanup EXIT INT TERM

main "$@"
