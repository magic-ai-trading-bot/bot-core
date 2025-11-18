#!/usr/bin/env bash

# @spec:NFR-OPS-017 - Daily Backup Status Report
# @ref:specs/05-operations/5.3-disaster-recovery/DR-PLAN.md

set -euo pipefail

BACKUP_DIR="${BACKUP_DIR:-/backups}"
REPORT_EMAIL="${REPORT_EMAIL:-ops@botcore.app}"

# Generate report
REPORT=$(cat <<EOF
Daily Backup Status Report
Generated: $(date '+%Y-%m-%d %H:%M:%S')

===================================
BACKUP SUMMARY (Last 24 hours)
===================================

MongoDB Backups:
$(find "$BACKUP_DIR/mongodb" -name "*.tar.gz*" -type f -mtime -1 ! -name "*.sha256" -exec ls -lh {} \; | awk '{print "  " $9 " (" $5 ")"}')

Volume Backups:
$(find "$BACKUP_DIR/volumes" -name "*.tar.gz" -type f -mtime -1 ! -name "*.sha256" -exec ls -lh {} \; | awk '{print "  " $9 " (" $5 ")"}' | head -5)

===================================
STORAGE STATUS
===================================

Total backup size: $(du -sh "$BACKUP_DIR" | awk '{print $1}')
Available space: $(df -h "$BACKUP_DIR" | awk 'NR==2 {print $4}')
Disk usage: $(df -h "$BACKUP_DIR" | awk 'NR==2 {print $5}')

===================================
LAST BACKUP
===================================

$(ls -lth "$BACKUP_DIR/mongodb/"*/mongodb_full_*.tar.gz* 2>/dev/null | head -1)

===================================
ISSUES (if any)
===================================

$(bash "$(dirname "$0")/check-backup-health.sh" 2>&1 | grep -E "ERROR|WARN" || echo "None")

===================================
End of Report
===================================
EOF
)

# Send email
echo "$REPORT" | mail -s "Bot-Core Daily Backup Report - $(date +%Y-%m-%d)" "$REPORT_EMAIL"

# Also log
echo "$REPORT" >> "${LOG_DIR:-/var/log/backups}/daily-reports-$(date +%Y%m).log"

echo "Report sent to $REPORT_EMAIL"
