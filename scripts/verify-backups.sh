#!/usr/bin/env bash

# @spec:NFR-OPS-013 - Backup Verification System
# @ref:specs/05-operations/5.3-disaster-recovery/DR-PLAN.md

set -euo pipefail

# Script: verify-backups.sh
# Purpose: Verify all backups integrity and generate report
# Usage: ./verify-backups.sh [--test-restore]
# Version: 1.0.0

BACKUP_DIR="${BACKUP_DIR:-/backups}"
LOG_FILE="${LOG_DIR:-/var/log/backups}/verify-$(date +%Y%m%d_%H%M%S).log"
REPORT_FILE="${BACKUP_DIR}/verification-report-$(date +%Y%m%d).txt"
TEST_RESTORE="${1:-}"

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

mkdir -p "$(dirname "$LOG_FILE")" "$(dirname "$REPORT_FILE")"

# Initialize counters
total_backups=0
verified_ok=0
verified_failed=0
missing_checksum=0

# Initialize report
cat > "$REPORT_FILE" <<EOF
================================================================================
BACKUP VERIFICATION REPORT
Generated: $(date '+%Y-%m-%d %H:%M:%S')
================================================================================

EOF

verify_file() {
    local file="$1"
    local checksum_file="${file}.sha256"

    ((total_backups++))

    log_info "Verifying: $(basename "$file")"

    # Check file exists and not empty
    if [ ! -f "$file" ]; then
        log_error "File not found: $file"
        echo "  [FAIL] Not found: $file" >> "$REPORT_FILE"
        ((verified_failed++))
        return 1
    fi

    if [ ! -s "$file" ]; then
        log_error "File is empty: $file"
        echo "  [FAIL] Empty file: $file" >> "$REPORT_FILE"
        ((verified_failed++))
        return 1
    fi

    # Verify checksum
    if [ -f "$checksum_file" ]; then
        if sha256sum -c "$checksum_file" &>/dev/null; then
            log_success "Checksum OK: $(basename "$file")"
            echo "  [OK] $file" >> "$REPORT_FILE"
            ((verified_ok++))
            return 0
        else
            log_error "Checksum mismatch: $file"
            echo "  [FAIL] Checksum mismatch: $file" >> "$REPORT_FILE"
            ((verified_failed++))
            return 1
        fi
    else
        log_warn "No checksum file: $(basename "$file")"
        echo "  [WARN] No checksum: $file" >> "$REPORT_FILE"
        ((missing_checksum++))
        ((verified_ok++))
        return 0
    fi
}

log_info "========================================="
log_info "Backup Verification Started"
log_info "========================================="

# Verify MongoDB backups
if [ -d "$BACKUP_DIR/mongodb" ]; then
    log_info "Verifying MongoDB backups..."
    echo -e "\nMONGODB BACKUPS:" >> "$REPORT_FILE"

    find "$BACKUP_DIR/mongodb" -name "*.tar.gz*" -type f ! -name "*.sha256" | sort -r | while read -r backup; do
        verify_file "$backup"
    done
else
    log_warn "MongoDB backup directory not found: $BACKUP_DIR/mongodb"
fi

# Verify volume backups
if [ -d "$BACKUP_DIR/volumes" ]; then
    log_info "Verifying volume backups..."
    echo -e "\nVOLUME BACKUPS:" >> "$REPORT_FILE"

    find "$BACKUP_DIR/volumes" -name "*.tar.gz" -type f ! -name "*.sha256" | sort -r | head -20 | while read -r backup; do
        verify_file "$backup"
    done
else
    log_warn "Volume backup directory not found: $BACKUP_DIR/volumes"
fi

# Verify config backups
if [ -d "$BACKUP_DIR/config" ]; then
    log_info "Verifying config backups..."
    echo -e "\nCONFIG BACKUPS:" >> "$REPORT_FILE"

    find "$BACKUP_DIR/config" -name "*.tar.gz" -type f ! -name "*.sha256" | sort -r | head -10 | while read -r backup; do
        verify_file "$backup"
    done
else
    log_warn "Config backup directory not found: $BACKUP_DIR/config"
fi

# Check backup age
log_info "Checking backup freshness..."
echo -e "\nBACKUP FRESHNESS:" >> "$REPORT_FILE"

latest_backup=$(find "$BACKUP_DIR" -name "mongodb_full_*.tar.gz*" -type f ! -name "*.sha256" | sort -r | head -1)

if [ -n "$latest_backup" ]; then
    backup_age_hours=$(( ($(date +%s) - $(stat -f%m "$latest_backup" 2>/dev/null || stat -c%Y "$latest_backup")) / 3600 ))

    echo "  Latest full backup: $(basename "$latest_backup")" >> "$REPORT_FILE"
    echo "  Age: ${backup_age_hours} hours" >> "$REPORT_FILE"

    if [ "$backup_age_hours" -lt 25 ]; then
        log_success "Latest backup is fresh (${backup_age_hours}h old)"
        echo "  Status: ✓ FRESH" >> "$REPORT_FILE"
    elif [ "$backup_age_hours" -lt 48 ]; then
        log_warn "Latest backup is getting old (${backup_age_hours}h old)"
        echo "  Status: ⚠ OLD" >> "$REPORT_FILE"
    else
        log_error "Latest backup is too old (${backup_age_hours}h old)"
        echo "  Status: ✗ TOO OLD" >> "$REPORT_FILE"
    fi
else
    log_error "No full backups found!"
    echo "  Status: ✗ NO BACKUPS FOUND" >> "$REPORT_FILE"
fi

# Test restore if requested
if [ "$TEST_RESTORE" = "--test-restore" ] && [ -n "$latest_backup" ]; then
    log_info "Performing test restore..."
    echo -e "\nTEST RESTORE:" >> "$REPORT_FILE"

    # Run restore in dry-run mode
    if bash "$(dirname "$0")/restore/restore-mongodb.sh" --dry-run "$latest_backup" &>/tmp/test-restore.log; then
        log_success "Test restore: PASSED"
        echo "  Status: ✓ PASSED" >> "$REPORT_FILE"
    else
        log_error "Test restore: FAILED"
        echo "  Status: ✗ FAILED" >> "$REPORT_FILE"
        echo "  Log: /tmp/test-restore.log" >> "$REPORT_FILE"
    fi
fi

# Disk space check
log_info "Checking disk space..."
echo -e "\nDISK SPACE:" >> "$REPORT_FILE"

backup_size=$(du -sh "$BACKUP_DIR" 2>/dev/null | awk '{print $1}')
available_space=$(df -h "$BACKUP_DIR" | awk 'NR==2 {print $4}')
usage_percent=$(df "$BACKUP_DIR" | awk 'NR==2 {print $5}')

echo "  Backup directory size: $backup_size" >> "$REPORT_FILE"
echo "  Available space: $available_space" >> "$REPORT_FILE"
echo "  Disk usage: $usage_percent" >> "$REPORT_FILE"

usage_num=$(echo "$usage_percent" | sed 's/%//')
if [ "$usage_num" -gt 90 ]; then
    log_error "Disk usage critical: $usage_percent"
    echo "  Status: ✗ CRITICAL" >> "$REPORT_FILE"
elif [ "$usage_num" -gt 80 ]; then
    log_warn "Disk usage high: $usage_percent"
    echo "  Status: ⚠ HIGH" >> "$REPORT_FILE"
else
    log_success "Disk usage normal: $usage_percent"
    echo "  Status: ✓ OK" >> "$REPORT_FILE"
fi

# Generate summary
cat >> "$REPORT_FILE" <<EOF

================================================================================
SUMMARY
================================================================================
Total backups checked:     $total_backups
Verified OK:               $verified_ok
Verification failed:       $verified_failed
Missing checksums:         $missing_checksum

EOF

if [ $verified_failed -eq 0 ]; then
    echo "Overall Status: ✓ ALL BACKUPS VERIFIED" >> "$REPORT_FILE"
else
    echo "Overall Status: ✗ SOME BACKUPS FAILED VERIFICATION" >> "$REPORT_FILE"
fi

echo -e "\n================================================================================\n" >> "$REPORT_FILE"

# Display report
cat "$REPORT_FILE"

log_info "========================================="
log_info "Backup Verification Complete"
log_info "Report: $REPORT_FILE"
log_info "========================================="

# Exit code
if [ $verified_failed -eq 0 ]; then
    exit 0
else
    exit 1
fi
