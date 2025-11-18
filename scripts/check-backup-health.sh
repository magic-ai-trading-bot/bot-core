#!/usr/bin/env bash

# @spec:NFR-OPS-016 - Backup Health Monitoring
# @ref:specs/05-operations/5.3-disaster-recovery/DR-PLAN.md

set -euo pipefail

BACKUP_DIR="${BACKUP_DIR:-/backups}"
LOG_FILE="${LOG_DIR:-/var/log/backups}/health-check-$(date +%Y%m%d).log"
ALERT_EMAIL="${ALERT_EMAIL:-ops@botcore.app}"
SLACK_WEBHOOK="${SLACK_WEBHOOK:-}"

log_info() {
    echo "[$(date '+%Y-%m-%d %H:%M:%S')] [INFO] $*" | tee -a "$LOG_FILE"
}

log_warn() {
    echo "[$(date '+%Y-%m-%d %H:%M:%S')] [WARN] $*" | tee -a "$LOG_FILE"
}

log_error() {
    echo "[$(date '+%Y-%m-%d %H:%M:%S')] [ERROR] $*" | tee -a "$LOG_FILE"
}

mkdir -p "$(dirname "$LOG_FILE")"

send_alert() {
    local severity="$1"
    local message="$2"

    # Send email
    if [ -n "$ALERT_EMAIL" ]; then
        echo "$message" | mail -s "[$severity] Bot-Core Backup Alert" "$ALERT_EMAIL"
    fi

    # Send Slack notification
    if [ -n "$SLACK_WEBHOOK" ]; then
        local color="warning"
        [ "$severity" = "CRITICAL" ] && color="danger"

        curl -X POST "$SLACK_WEBHOOK" \
            -H 'Content-Type: application/json' \
            -d "{
                \"attachments\": [{
                    \"color\": \"$color\",
                    \"title\": \"[$severity] Backup Health Alert\",
                    \"text\": \"$message\",
                    \"footer\": \"Bot-Core Monitoring\",
                    \"ts\": $(date +%s)
                }]
            }" 2>&1 | tee -a "$LOG_FILE" || true
    fi
}

log_info "Checking backup health..."

issues_found=0

# Check 1: Last backup age
latest_backup=$(find "$BACKUP_DIR/mongodb" -name "mongodb_full_*.tar.gz*" -type f ! -name "*.sha256" | sort -r | head -1)

if [ -n "$latest_backup" ]; then
    backup_age_hours=$(( ($(date +%s) - $(stat -f%m "$latest_backup" 2>/dev/null || stat -c%Y "$latest_backup")) / 3600 ))

    if [ "$backup_age_hours" -gt 25 ]; then
        log_error "Last backup is too old: ${backup_age_hours}h"
        send_alert "CRITICAL" "Last MongoDB backup is ${backup_age_hours} hours old (threshold: 25h)"
        ((issues_found++))
    else
        log_info "Last backup age: ${backup_age_hours}h (OK)"
    fi
else
    log_error "No backups found!"
    send_alert "CRITICAL" "No MongoDB backups found in $BACKUP_DIR/mongodb"
    ((issues_found++))
fi

# Check 2: Disk space
available_gb=$(df -BG "$BACKUP_DIR" | awk 'NR==2 {print $4}' | sed 's/G//')

if [ "$available_gb" -lt 10 ]; then
    log_error "Low disk space: ${available_gb}GB available"
    send_alert "WARNING" "Backup disk space low: ${available_gb}GB available (threshold: 10GB)"
    ((issues_found++))
else
    log_info "Disk space: ${available_gb}GB (OK)"
fi

# Check 3: Backup count
backup_count=$(find "$BACKUP_DIR/mongodb" -name "mongodb_full_*.tar.gz*" -type f ! -name "*.sha256" -mtime -7 | wc -l)

if [ "$backup_count" -lt 7 ]; then
    log_warn "Only $backup_count backups in last 7 days (expected: 7)"
    send_alert "WARNING" "Only $backup_count MongoDB backups in last 7 days (expected: 7)"
    ((issues_found++))
else
    log_info "Backup count (7 days): $backup_count (OK)"
fi

# Check 4: Failed backup logs
failed_backups=$(grep -c "ERROR" "$LOG_DIR/mongodb-backup-$(date +%Y%m%d).log" 2>/dev/null || echo 0)

if [ "$failed_backups" -gt 0 ]; then
    log_error "Found $failed_backups errors in today's backup log"
    send_alert "WARNING" "Found $failed_backups errors in MongoDB backup log"
    ((issues_found++))
fi

# Summary
if [ $issues_found -eq 0 ]; then
    log_info "All backup health checks passed"
    exit 0
else
    log_error "$issues_found backup health issues found"
    exit 1
fi
