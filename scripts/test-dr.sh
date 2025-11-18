#!/usr/bin/env bash

# @spec:NFR-OPS-018 - Disaster Recovery Testing
# @ref:specs/05-operations/5.3-disaster-recovery/DR-PLAN.md

set -euo pipefail

# Script: test-dr.sh
# Purpose: Automated DR drill and testing
# Usage: ./test-dr.sh [--automated]
# Version: 1.0.0

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
LOG_FILE="${LOG_DIR:-/var/log/backups}/dr-test-$(date +%Y%m%d_%H%M%S).log"
REPORT_FILE="/tmp/dr-test-report-$(date +%Y%m%d_%H%M%S).txt"
AUTOMATED_MODE="${1:-}"

log_info() {
    echo "[$(date '+%Y-%m-%d %H:%M:%S')] [INFO] $*" | tee -a "$LOG_FILE"
}

log_success() {
    echo "[$(date '+%Y-%m-%d %H:%M:%S')] [SUCCESS] $*" | tee -a "$LOG_FILE"
}

log_error() {
    echo "[$(date '+%Y-%m-%d %H:%M:%S')] [ERROR] $*" | tee -a "$LOG_FILE"
}

mkdir -p "$(dirname "$LOG_FILE")"

# Initialize report
cat > "$REPORT_FILE" <<EOF
================================================================================
DISASTER RECOVERY DRILL REPORT
Date: $(date '+%Y-%m-%d %H:%M:%S')
Type: ${AUTOMATED_MODE:+Automated}${AUTOMATED_MODE:-Manual}
================================================================================

EOF

start_time=$(date +%s)
tests_passed=0
tests_failed=0

run_test() {
    local test_name="$1"
    local test_command="$2"

    log_info "Running test: $test_name"
    echo "TEST: $test_name" >> "$REPORT_FILE"

    if eval "$test_command" &>> "$REPORT_FILE"; then
        log_success "$test_name: PASSED"
        echo "  Result: ✓ PASSED" >> "$REPORT_FILE"
        ((tests_passed++))
    else
        log_error "$test_name: FAILED"
        echo "  Result: ✗ FAILED" >> "$REPORT_FILE"
        ((tests_failed++))
    fi

    echo "" >> "$REPORT_FILE"
}

log_info "========================================="
log_info "DR Drill Started"
log_info "========================================="

# Test 1: Verify backup existence
run_test "Backup Existence Check" \
    "[ -n \"\$(find ${BACKUP_DIR:-/backups}/mongodb -name 'mongodb_full_*.tar.gz*' -type f | head -1)\" ]"

# Test 2: Backup integrity verification
run_test "Backup Integrity Verification" \
    "bash $SCRIPT_DIR/verify-backups.sh"

# Test 3: Test restore (dry-run)
run_test "Test Restore (Dry-Run)" \
    "bash $SCRIPT_DIR/verify-backups.sh --test-restore"

# Test 4: Disk space check
run_test "Sufficient Disk Space" \
    "[ \$(df -BG ${BACKUP_DIR:-/backups} | awk 'NR==2 {print \$4}' | sed 's/G//') -gt 10 ]"

# Test 5: Cloud backup accessibility (if enabled)
if [ "${ENABLE_S3_BACKUP:-false}" = "true" ]; then
    run_test "S3 Backup Accessibility" \
        "aws s3 ls ${S3_BACKUP_BUCKET:-s3://bot-core-backups}/ --region ${AWS_REGION:-us-east-1} > /dev/null 2>&1"
fi

# Test 6: MongoDB container health
run_test "MongoDB Container Health" \
    "docker ps | grep -q mongodb"

# Test 7: Backup script execution
run_test "Backup Script Executable" \
    "[ -x $SCRIPT_DIR/backup/backup-mongodb.sh ]"

# Test 8: Restore script execution
run_test "Restore Script Executable" \
    "[ -x $SCRIPT_DIR/restore/restore-mongodb.sh ]"

# Calculate duration
end_time=$(date +%s)
duration=$((end_time - start_time))

# Generate summary
cat >> "$REPORT_FILE" <<EOF
================================================================================
SUMMARY
================================================================================
Duration: ${duration}s
Tests Passed: $tests_passed
Tests Failed: $tests_failed

Overall Result: $([ $tests_failed -eq 0 ] && echo "✓ ALL TESTS PASSED" || echo "✗ SOME TESTS FAILED")

EOF

if [ $tests_failed -eq 0 ]; then
    echo "Status: ✓ READY FOR DISASTER RECOVERY" >> "$REPORT_FILE"
else
    echo "Status: ✗ DR READINESS ISSUES FOUND" >> "$REPORT_FILE"
    echo "Action Required: Review failed tests and remediate" >> "$REPORT_FILE"
fi

cat >> "$REPORT_FILE" <<EOF

Next DR Drill: $(date -d '+1 month' '+%Y-%m-%d' 2>/dev/null || date -v+1m '+%Y-%m-%d')

================================================================================
End of Report
================================================================================
EOF

# Display report
cat "$REPORT_FILE"

# Send notification
if [ "${AUTOMATED_MODE}" = "--automated" ]; then
    if [ -n "${SLACK_WEBHOOK:-}" ]; then
        curl -X POST "$SLACK_WEBHOOK" \
            -H 'Content-Type: application/json' \
            -d "{
                \"text\": \"DR Drill Complete: $tests_passed passed, $tests_failed failed\",
                \"attachments\": [{
                    \"color\": \"$([ $tests_failed -eq 0 ] && echo 'good' || echo 'danger')\",
                    \"text\": \"Full report: $REPORT_FILE\"
                }]
            }" 2>&1 | tee -a "$LOG_FILE" || true
    fi
fi

# Save report
cp "$REPORT_FILE" "${LOG_DIR:-/var/log/backups}/dr-test-reports/dr-report-$(date +%Y%m%d).txt" 2>/dev/null || true

log_info "========================================="
log_info "DR Drill Complete"
log_info "Report: $REPORT_FILE"
log_info "========================================="

[ $tests_failed -eq 0 ] && exit 0 || exit 1
