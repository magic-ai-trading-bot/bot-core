#!/bin/bash

# Rollback Automation Script
# Restore from the most recent backup
# @spec:FR-OPS-005 - Rollback Capability
# @ref:specs/05-operations/5.3-disaster-recovery.md

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
CYAN='\033[0;36m'
NC='\033[0m' # No Color

# Configuration
BACKUP_DIR="${BACKUP_DIR:-./backups}"
ROLLBACK_LOG="${ROLLBACK_LOG:-./logs/rollback.log}"

# Specific backup to restore (optional)
BACKUP_TO_RESTORE="${1:-}"

# Function to log to both console and file
log() {
    echo -e "$1" | tee -a "$ROLLBACK_LOG"
}

# Function to print section header
print_section() {
    log ""
    log "${CYAN}========================================${NC}"
    log "${CYAN}$1${NC}"
    log "${CYAN}========================================${NC}"
}

# Create log directory
mkdir -p "$(dirname "$ROLLBACK_LOG")"

# Initialize log
cat > "$ROLLBACK_LOG" << EOF
Rollback Log
Started: $(date)
User: $(whoami)
Hostname: $(hostname)

========================================
ROLLBACK PROCESS
========================================

EOF

print_section "ROLLBACK TO PREVIOUS VERSION"

# Step 1: Check if backups exist
print_section "1. Checking Available Backups"

if [ ! -d "$BACKUP_DIR" ] || [ -z "$(ls -A "$BACKUP_DIR")" ]; then
    log "${RED}✗ No backups found in $BACKUP_DIR${NC}"
    log ""
    log "Cannot rollback without a backup."
    log "Please restore manually or redeploy."
    exit 1
fi

# List available backups
log "Available backups:"
BACKUPS=($(ls -t "$BACKUP_DIR"))
for i in "${!BACKUPS[@]}"; do
    BACKUP="${BACKUPS[$i]}"
    METADATA_FILE="$BACKUP_DIR/$BACKUP/metadata.txt"

    if [ -f "$METADATA_FILE" ]; then
        BACKUP_DATE=$(grep "Backup Created:" "$METADATA_FILE" | cut -d: -f2- | xargs)
        log "  [$i] $BACKUP - Created: $BACKUP_DATE"
    else
        log "  [$i] $BACKUP"
    fi
done
log ""

# Step 2: Select backup to restore
if [ -z "$BACKUP_TO_RESTORE" ]; then
    # Use most recent backup
    BACKUP_TO_RESTORE="${BACKUPS[0]}"
    log "No backup specified, using most recent: $BACKUP_TO_RESTORE"
else
    # Validate specified backup exists
    if [ ! -d "$BACKUP_DIR/$BACKUP_TO_RESTORE" ]; then
        log "${RED}✗ Backup not found: $BACKUP_TO_RESTORE${NC}"
        log "Available backups: ${BACKUPS[@]}"
        exit 1
    fi
fi

BACKUP_PATH="$BACKUP_DIR/$BACKUP_TO_RESTORE"
log "${GREEN}✓ Selected backup: $BACKUP_TO_RESTORE${NC}"
log "  Path: $BACKUP_PATH"

# Show backup metadata
if [ -f "$BACKUP_PATH/metadata.txt" ]; then
    log ""
    log "Backup metadata:"
    cat "$BACKUP_PATH/metadata.txt" | while read line; do
        log "  $line"
    done
fi

# Confirmation
log ""
read -p "Proceed with rollback to this backup? (y/N) " -n 1 -r
echo
if [[ ! $REPLY =~ ^[Yy]$ ]]; then
    log "Rollback cancelled by user"
    exit 0
fi

# Step 3: Stop current services
print_section "2. Stopping Current Services"

log "Stopping all running services..."

if docker-compose ps | grep -q "Up"; then
    docker-compose down --timeout 30 || true
    log "${GREEN}✓ Services stopped${NC}"
else
    log "${YELLOW}⚠ No running services found${NC}"
fi

# Wait for services to fully stop
sleep 5

# Step 4: Restore environment
print_section "3. Restoring Environment"

# Restore .env file
if [ -f "$BACKUP_PATH/.env.backup" ]; then
    log "Restoring .env file..."
    cp "$BACKUP_PATH/.env.backup" .env
    log "${GREEN}✓ .env file restored${NC}"
else
    log "${YELLOW}⚠ No .env backup found${NC}"
fi

# Step 5: Restore git commit (if available)
if [ -f "$BACKUP_PATH/git-commit.txt" ] && [ -d ".git" ]; then
    print_section "4. Restoring Git State"

    BACKUP_COMMIT=$(cat "$BACKUP_PATH/git-commit.txt")
    CURRENT_COMMIT=$(git rev-parse HEAD 2>/dev/null || echo "")

    if [ "$BACKUP_COMMIT" != "$CURRENT_COMMIT" ]; then
        log "Restoring git commit: $BACKUP_COMMIT"

        # Stash current changes
        git stash save "rollback-stash-$(date +%Y%m%d-%H%M%S)" || true

        # Checkout backup commit
        if git checkout "$BACKUP_COMMIT" 2>&1 | tee -a "$ROLLBACK_LOG"; then
            log "${GREEN}✓ Git state restored to $BACKUP_COMMIT${NC}"
        else
            log "${RED}✗ Failed to restore git state${NC}"
            log "${YELLOW}  Continuing with current code...${NC}"
        fi
    else
        log "${GREEN}✓ Already at backup commit${NC}"
    fi
fi

# Step 6: Restore database (if available)
if [ -d "$BACKUP_PATH/mongodb-dump" ]; then
    print_section "5. Restoring Database"

    log "Starting MongoDB for restore..."
    docker-compose up -d mongodb || true
    sleep 10

    log "Restoring MongoDB database..."

    # Copy backup into container
    docker cp "$BACKUP_PATH/mongodb-dump" mongodb:/tmp/restore || handle_error 1 "Failed to copy backup to container"

    # Restore database
    if docker exec mongodb mongorestore --drop /tmp/restore 2>&1 | tee -a "$ROLLBACK_LOG"; then
        log "${GREEN}✓ Database restored${NC}"
    else
        log "${RED}✗ Database restore failed${NC}"
        log "${YELLOW}  You may need to restore manually${NC}"
    fi

    # Clean up
    docker exec mongodb rm -rf /tmp/restore || true
fi

# Step 7: Rebuild and start services
print_section "6. Rebuilding Services"

log "Rebuilding Docker images..."
if docker-compose build 2>&1 | tee -a "$ROLLBACK_LOG"; then
    log "${GREEN}✓ Build completed${NC}"
else
    log "${RED}✗ Build failed${NC}"
    exit 1
fi

# Step 8: Start services
print_section "7. Starting Services"

log "Starting services..."
if docker-compose up -d 2>&1 | tee -a "$ROLLBACK_LOG"; then
    log "${GREEN}✓ Services started${NC}"
else
    log "${RED}✗ Failed to start services${NC}"
    exit 1
fi

# Wait for services to initialize
log "Waiting for services to initialize (30 seconds)..."
sleep 30

# Step 9: Health checks
print_section "8. Running Health Checks"

HEALTH_RETRIES=3
HEALTH_PASSED=false

for i in $(seq 1 $HEALTH_RETRIES); do
    log "Health check attempt $i/$HEALTH_RETRIES..."

    if ./scripts/health-check.sh /tmp/rollback-health.log; then
        log "${GREEN}✓ All health checks passed${NC}"
        HEALTH_PASSED=true
        break
    else
        EXIT_CODE=$?
        if [ $EXIT_CODE -eq 2 ]; then
            log "${YELLOW}⚠ Some services are degraded${NC}"
            if [ $i -eq $HEALTH_RETRIES ]; then
                log "${YELLOW}  Proceeding with degraded services${NC}"
                HEALTH_PASSED=true
                break
            fi
        else
            log "${RED}✗ Health checks failed${NC}"
        fi

        if [ $i -lt $HEALTH_RETRIES ]; then
            log "Waiting 10s before retry..."
            sleep 10
        fi
    fi
done

if [ "$HEALTH_PASSED" != "true" ]; then
    log "${RED}✗ Health checks failed after $HEALTH_RETRIES attempts${NC}"
    log "${YELLOW}  Services may not be fully functional${NC}"
    log "${YELLOW}  Check logs: ./scripts/bot.sh logs${NC}"
fi

# Step 10: Log rollback
print_section "9. Finalizing Rollback"

ROLLBACK_ID="rollback-$(date +%Y%m%d-%H%M%S)"

cat > "./logs/$ROLLBACK_ID.log" << EOF
Rollback ID: $ROLLBACK_ID
Completed: $(date)
User: $(whoami)
Hostname: $(hostname)
Restored From: $BACKUP_TO_RESTORE
Status: $([ "$HEALTH_PASSED" = "true" ] && echo "SUCCESS" || echo "PARTIAL")

Services:
$(docker-compose ps)

Health Check:
See /tmp/rollback-health.log
EOF

log "${GREEN}✓ Rollback logged: ./logs/$ROLLBACK_ID.log${NC}"

# Summary
print_section "ROLLBACK SUMMARY"

if [ "$HEALTH_PASSED" = "true" ]; then
    log "${GREEN}✓ ROLLBACK COMPLETED SUCCESSFULLY${NC}"
else
    log "${YELLOW}⚠ ROLLBACK COMPLETED WITH WARNINGS${NC}"
fi

log ""
log "Rollback ID: $ROLLBACK_ID"
log "Restored From: $BACKUP_TO_RESTORE"
log "Log: $ROLLBACK_LOG"
log ""
log "Service URLs:"
log "  - Frontend:  http://localhost:3000"
log "  - Rust API:  http://localhost:8080/api/health"
log "  - Python AI: http://localhost:8000/health"
log ""
log "Next steps:"
log "  1. Verify functionality through dashboard"
log "  2. Monitor logs: ./scripts/bot.sh logs"
log "  3. Check health: ./scripts/health-check.sh"
log ""

exit 0
