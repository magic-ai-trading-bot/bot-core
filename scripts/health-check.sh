#!/bin/bash

# Health Check Script for Trading Bot Services
# Comprehensive health monitoring for all services
# @spec:FR-OPS-003 - Service Health Monitoring
# @ref:specs/05-operations/5.1-monitoring.md

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
CYAN='\033[0;36m'
NC='\033[0m' # No Color

# Configuration
RUST_PORT=${RUST_PORT:-8080}
PYTHON_PORT=${PYTHON_PORT:-8000}
FRONTEND_PORT=${FRONTEND_PORT:-3000}
MONGO_PORT=${MONGO_PORT:-27017}
REDIS_PORT=${REDIS_PORT:-6379}

# Timeouts
HTTP_TIMEOUT=10
WS_TIMEOUT=5

# Counters
HEALTHY=0
UNHEALTHY=0
DEGRADED=0
TOTAL=0

# Report file
REPORT_FILE="${1:-/tmp/health-check-report.txt}"

# Function to log to both console and file
log() {
    echo -e "$1" | tee -a "$REPORT_FILE"
}

# Function to print section header
print_section() {
    log ""
    log "${CYAN}========================================${NC}"
    log "${CYAN}$1${NC}"
    log "${CYAN}========================================${NC}"
}

# Initialize report
cat > "$REPORT_FILE" << EOF
Service Health Check Report
Generated: $(date)
Hostname: $(hostname)

========================================
HEALTH CHECK RESULTS
========================================

EOF

print_section "TRADING BOT HEALTH CHECK"
log "Report will be saved to: $REPORT_FILE"
log ""

# Function to check HTTP endpoint
check_http_endpoint() {
    local name=$1
    local url=$2
    local expected_status=${3:-200}

    TOTAL=$((TOTAL + 1))

    if ! command -v curl > /dev/null 2>&1; then
        log "${YELLOW}⚠ curl not installed - cannot check $name${NC}"
        echo "SKIPPED: $name - curl not installed" >> "$REPORT_FILE"
        return 2
    fi

    log "Checking $name: $url"

    # Make the request
    HTTP_CODE=$(curl -s -o /dev/null -w "%{http_code}" --connect-timeout $HTTP_TIMEOUT "$url" 2>/dev/null || echo "000")

    if [ "$HTTP_CODE" = "$expected_status" ]; then
        log "${GREEN}✓ $name is healthy (HTTP $HTTP_CODE)${NC}"
        echo "HEALTHY: $name - HTTP $HTTP_CODE" >> "$REPORT_FILE"
        HEALTHY=$((HEALTHY + 1))
        return 0
    elif [ "$HTTP_CODE" = "000" ]; then
        log "${RED}✗ $name is unreachable${NC}"
        echo "UNHEALTHY: $name - unreachable" >> "$REPORT_FILE"
        UNHEALTHY=$((UNHEALTHY + 1))
        return 1
    else
        log "${YELLOW}⚠ $name returned unexpected status (HTTP $HTTP_CODE)${NC}"
        echo "DEGRADED: $name - HTTP $HTTP_CODE (expected $expected_status)" >> "$REPORT_FILE"
        DEGRADED=$((DEGRADED + 1))
        return 2
    fi
}

# Function to check API endpoint with response
check_api_health() {
    local name=$1
    local url=$2
    local expected_field=${3:-"status"}

    TOTAL=$((TOTAL + 1))

    log "Checking $name API: $url"

    # Make the request and get response
    RESPONSE=$(curl -s --connect-timeout $HTTP_TIMEOUT "$url" 2>/dev/null || echo "{}")
    HTTP_CODE=$(curl -s -o /dev/null -w "%{http_code}" --connect-timeout $HTTP_TIMEOUT "$url" 2>/dev/null || echo "000")

    if [ "$HTTP_CODE" = "200" ]; then
        # Check if response contains expected field
        if echo "$RESPONSE" | grep -q "$expected_field"; then
            log "${GREEN}✓ $name API is healthy${NC}"
            log "  Response: $RESPONSE"
            echo "HEALTHY: $name - API responding correctly" >> "$REPORT_FILE"
            HEALTHY=$((HEALTHY + 1))
            return 0
        else
            log "${YELLOW}⚠ $name API returned unexpected response${NC}"
            log "  Response: $RESPONSE"
            echo "DEGRADED: $name - unexpected response" >> "$REPORT_FILE"
            DEGRADED=$((DEGRADED + 1))
            return 2
        fi
    elif [ "$HTTP_CODE" = "000" ]; then
        log "${RED}✗ $name API is unreachable${NC}"
        echo "UNHEALTHY: $name - unreachable" >> "$REPORT_FILE"
        UNHEALTHY=$((UNHEALTHY + 1))
        return 1
    else
        log "${YELLOW}⚠ $name API returned HTTP $HTTP_CODE${NC}"
        echo "DEGRADED: $name - HTTP $HTTP_CODE" >> "$REPORT_FILE"
        DEGRADED=$((DEGRADED + 1))
        return 2
    fi
}

# Function to check database
check_database() {
    local name=$1
    local host=${2:-localhost}
    local port=${3:-27017}

    TOTAL=$((TOTAL + 1))

    log "Checking $name database: $host:$port"

    if command -v mongosh > /dev/null 2>&1; then
        if [ ! -z "$DATABASE_URL" ]; then
            if mongosh "$DATABASE_URL" --eval "db.adminCommand('ping')" > /dev/null 2>&1; then
                log "${GREEN}✓ $name database is healthy${NC}"

                # Get database stats
                DB_SIZE=$(mongosh "$DATABASE_URL" --eval "db.stats().dataSize" --quiet 2>/dev/null || echo "0")
                COLLECTIONS=$(mongosh "$DATABASE_URL" --eval "db.getCollectionNames().length" --quiet 2>/dev/null || echo "0")

                log "  Collections: $COLLECTIONS"
                echo "HEALTHY: $name - $COLLECTIONS collections" >> "$REPORT_FILE"
                HEALTHY=$((HEALTHY + 1))
                return 0
            else
                log "${RED}✗ $name database connection failed${NC}"
                echo "UNHEALTHY: $name - connection failed" >> "$REPORT_FILE"
                UNHEALTHY=$((UNHEALTHY + 1))
                return 1
            fi
        else
            # Try simple TCP connection
            if timeout 5 bash -c "cat < /dev/null > /dev/tcp/$host/$port" 2>/dev/null; then
                log "${GREEN}✓ $name database port is open${NC}"
                echo "HEALTHY: $name - port open" >> "$REPORT_FILE"
                HEALTHY=$((HEALTHY + 1))
                return 0
            else
                log "${RED}✗ $name database port is not accessible${NC}"
                echo "UNHEALTHY: $name - port not accessible" >> "$REPORT_FILE"
                UNHEALTHY=$((UNHEALTHY + 1))
                return 1
            fi
        fi
    else
        # Try simple TCP connection
        if timeout 5 bash -c "cat < /dev/null > /dev/tcp/$host/$port" 2>/dev/null; then
            log "${GREEN}✓ $name database port is open${NC}"
            echo "HEALTHY: $name - port open" >> "$REPORT_FILE"
            HEALTHY=$((HEALTHY + 1))
            return 0
        else
            log "${RED}✗ $name database port is not accessible${NC}"
            echo "UNHEALTHY: $name - port not accessible" >> "$REPORT_FILE"
            UNHEALTHY=$((UNHEALTHY + 1))
            return 1
        fi
    fi
}

# Function to check Redis
check_redis() {
    local host=${1:-localhost}
    local port=${2:-6379}

    TOTAL=$((TOTAL + 1))

    log "Checking Redis: $host:$port"

    if command -v redis-cli > /dev/null 2>&1; then
        if redis-cli -h "$host" -p "$port" ping > /dev/null 2>&1; then
            log "${GREEN}✓ Redis is healthy${NC}"

            # Get Redis info
            CONNECTED_CLIENTS=$(redis-cli -h "$host" -p "$port" INFO clients | grep connected_clients | cut -d: -f2 | tr -d '\r')
            USED_MEMORY=$(redis-cli -h "$host" -p "$port" INFO memory | grep used_memory_human | cut -d: -f2 | tr -d '\r')

            log "  Connected clients: $CONNECTED_CLIENTS"
            log "  Used memory: $USED_MEMORY"
            echo "HEALTHY: Redis - $CONNECTED_CLIENTS clients, $USED_MEMORY memory" >> "$REPORT_FILE"
            HEALTHY=$((HEALTHY + 1))
            return 0
        else
            log "${RED}✗ Redis connection failed${NC}"
            echo "UNHEALTHY: Redis - connection failed" >> "$REPORT_FILE"
            UNHEALTHY=$((UNHEALTHY + 1))
            return 1
        fi
    else
        # Try simple TCP connection
        if timeout 5 bash -c "cat < /dev/null > /dev/tcp/$host/$port" 2>/dev/null; then
            log "${GREEN}✓ Redis port is open${NC}"
            echo "HEALTHY: Redis - port open" >> "$REPORT_FILE"
            HEALTHY=$((HEALTHY + 1))
            return 0
        else
            log "${YELLOW}⚠ Redis is not running (optional service)${NC}"
            echo "SKIPPED: Redis - not running" >> "$REPORT_FILE"
            return 2
        fi
    fi
}

# Function to check Docker container
check_docker_container() {
    local container_name=$1

    if ! command -v docker > /dev/null 2>&1; then
        return 2
    fi

    if docker ps --format "{{.Names}}" | grep -q "^${container_name}$"; then
        # Get container status
        STATUS=$(docker inspect --format='{{.State.Status}}' "$container_name" 2>/dev/null)
        HEALTH=$(docker inspect --format='{{.State.Health.Status}}' "$container_name" 2>/dev/null || echo "none")

        if [ "$STATUS" = "running" ]; then
            if [ "$HEALTH" = "healthy" ] || [ "$HEALTH" = "none" ]; then
                log "  Container '$container_name': ${GREEN}running${NC}"
                return 0
            else
                log "  Container '$container_name': ${YELLOW}$HEALTH${NC}"
                return 2
            fi
        else
            log "  Container '$container_name': ${RED}$STATUS${NC}"
            return 1
        fi
    else
        return 2
    fi
}

# 1. Rust Core Engine
print_section "1. Rust Core Engine (Port $RUST_PORT)"

check_api_health "Rust Health" "http://localhost:$RUST_PORT/api/health" "status"
check_http_endpoint "Rust API" "http://localhost:$RUST_PORT/api/ping" 200
check_docker_container "rust-core-engine"

# Check WebSocket (if wscat is installed)
if command -v wscat > /dev/null 2>&1; then
    TOTAL=$((TOTAL + 1))
    log "Checking Rust WebSocket..."

    if timeout $WS_TIMEOUT wscat -c "ws://localhost:$RUST_PORT/ws" -x "ping" > /dev/null 2>&1; then
        log "${GREEN}✓ Rust WebSocket is healthy${NC}"
        echo "HEALTHY: Rust WebSocket" >> "$REPORT_FILE"
        HEALTHY=$((HEALTHY + 1))
    else
        log "${YELLOW}⚠ Rust WebSocket test failed${NC}"
        echo "DEGRADED: Rust WebSocket" >> "$REPORT_FILE"
        DEGRADED=$((DEGRADED + 1))
    fi
fi

# 2. Python AI Service
print_section "2. Python AI Service (Port $PYTHON_PORT)"

check_api_health "Python Health" "http://localhost:$PYTHON_PORT/health" "status"
check_http_endpoint "Python API" "http://localhost:$PYTHON_PORT/docs" 200
check_docker_container "python-ai-service"

# 3. Frontend Dashboard
print_section "3. Frontend Dashboard (Port $FRONTEND_PORT)"

check_http_endpoint "Frontend" "http://localhost:$FRONTEND_PORT" 200
check_docker_container "nextjs-ui-dashboard"

# 4. MongoDB Database
print_section "4. MongoDB Database (Port $MONGO_PORT)"

check_database "MongoDB" "localhost" "$MONGO_PORT"
check_docker_container "mongodb"

# 5. Redis (Optional)
print_section "5. Redis Cache (Port $REDIS_PORT) [Optional]"

check_redis "localhost" "$REDIS_PORT"
check_docker_container "redis"

# 6. System Resources
print_section "6. System Resources"

# CPU usage
TOTAL=$((TOTAL + 1))
if command -v top > /dev/null 2>&1; then
    CPU_USAGE=$(top -bn1 | grep "Cpu(s)" | awk '{print $2}' | cut -d% -f1 2>/dev/null || echo "0")
    if [ -z "$CPU_USAGE" ]; then
        CPU_USAGE=$(top -l 1 | grep "CPU usage" | awk '{print $3}' | cut -d% -f1 2>/dev/null || echo "0")
    fi

    if (( $(echo "$CPU_USAGE < 80" | bc -l 2>/dev/null || echo "1") )); then
        log "${GREEN}✓ CPU usage is normal (${CPU_USAGE}%)${NC}"
        echo "HEALTHY: CPU - ${CPU_USAGE}%" >> "$REPORT_FILE"
        HEALTHY=$((HEALTHY + 1))
    else
        log "${YELLOW}⚠ High CPU usage (${CPU_USAGE}%)${NC}"
        echo "DEGRADED: CPU - ${CPU_USAGE}%" >> "$REPORT_FILE"
        DEGRADED=$((DEGRADED + 1))
    fi
else
    log "${YELLOW}⚠ Cannot check CPU usage${NC}"
fi

# Memory usage
TOTAL=$((TOTAL + 1))
if command -v free > /dev/null 2>&1; then
    MEMORY_USED_PCT=$(free | grep Mem | awk '{print int($3/$2 * 100)}')

    if [ "$MEMORY_USED_PCT" -lt 90 ]; then
        log "${GREEN}✓ Memory usage is normal (${MEMORY_USED_PCT}%)${NC}"
        echo "HEALTHY: Memory - ${MEMORY_USED_PCT}%" >> "$REPORT_FILE"
        HEALTHY=$((HEALTHY + 1))
    else
        log "${YELLOW}⚠ High memory usage (${MEMORY_USED_PCT}%)${NC}"
        echo "DEGRADED: Memory - ${MEMORY_USED_PCT}%" >> "$REPORT_FILE"
        DEGRADED=$((DEGRADED + 1))
    fi
elif command -v vm_stat > /dev/null 2>&1; then
    # macOS
    FREE_PAGES=$(vm_stat | grep "Pages free" | awk '{print $3}' | sed 's/\.//')
    TOTAL_PAGES=$(sysctl -n hw.memsize | awk '{print $1/4096}')
    MEMORY_USED_PCT=$(echo "scale=0; 100 - ($FREE_PAGES * 100 / $TOTAL_PAGES)" | bc)

    if [ "$MEMORY_USED_PCT" -lt 90 ]; then
        log "${GREEN}✓ Memory usage is normal (~${MEMORY_USED_PCT}%)${NC}"
        echo "HEALTHY: Memory - ${MEMORY_USED_PCT}%" >> "$REPORT_FILE"
        HEALTHY=$((HEALTHY + 1))
    else
        log "${YELLOW}⚠ High memory usage (~${MEMORY_USED_PCT}%)${NC}"
        echo "DEGRADED: Memory - ${MEMORY_USED_PCT}%" >> "$REPORT_FILE"
        DEGRADED=$((DEGRADED + 1))
    fi
else
    log "${YELLOW}⚠ Cannot check memory usage${NC}"
fi

# Disk usage
TOTAL=$((TOTAL + 1))
if command -v df > /dev/null 2>&1; then
    DISK_USED_PCT=$(df -h . | tail -1 | awk '{print $5}' | sed 's/%//')

    if [ "$DISK_USED_PCT" -lt 90 ]; then
        log "${GREEN}✓ Disk usage is normal (${DISK_USED_PCT}%)${NC}"
        echo "HEALTHY: Disk - ${DISK_USED_PCT}%" >> "$REPORT_FILE"
        HEALTHY=$((HEALTHY + 1))
    else
        log "${YELLOW}⚠ High disk usage (${DISK_USED_PCT}%)${NC}"
        echo "DEGRADED: Disk - ${DISK_USED_PCT}%" >> "$REPORT_FILE"
        DEGRADED=$((DEGRADED + 1))
    fi
else
    log "${YELLOW}⚠ Cannot check disk usage${NC}"
fi

# Summary
print_section "HEALTH CHECK SUMMARY"

log "Total Checks: $TOTAL"
log "Healthy: ${GREEN}$HEALTHY${NC}"
log "Degraded: ${YELLOW}$DEGRADED${NC}"
log "Unhealthy: ${RED}$UNHEALTHY${NC}"
log ""

# Calculate health percentage
if [ $TOTAL -gt 0 ]; then
    HEALTH_PCT=$(( (HEALTHY * 100) / TOTAL ))
else
    HEALTH_PCT=0
fi

log "Overall Health: $HEALTH_PCT%"

# Write summary to report
cat >> "$REPORT_FILE" << EOF

========================================
SUMMARY
========================================
Total Checks: $TOTAL
Healthy: $HEALTHY
Degraded: $DEGRADED
Unhealthy: $UNHEALTHY
Overall Health: $HEALTH_PCT%

EOF

# Final verdict
if [ $UNHEALTHY -eq 0 ] && [ $DEGRADED -eq 0 ]; then
    log "${GREEN}✓ ALL SERVICES ARE HEALTHY${NC}"
    echo "STATUS: HEALTHY - All services operational" >> "$REPORT_FILE"
    exit 0
elif [ $UNHEALTHY -eq 0 ]; then
    log "${YELLOW}⚠ SOME SERVICES ARE DEGRADED${NC}"
    log "${YELLOW}  $DEGRADED service(s) need attention${NC}"
    echo "STATUS: DEGRADED - Some services need attention" >> "$REPORT_FILE"
    exit 2
else
    log "${RED}✗ SOME SERVICES ARE UNHEALTHY${NC}"
    log "${RED}  $UNHEALTHY service(s) are down${NC}"
    log ""
    log "Troubleshooting:"
    log "  1. Check service logs: ./scripts/bot.sh logs"
    log "  2. Restart services: ./scripts/bot.sh restart"
    log "  3. Check Docker: docker ps -a"
    echo "STATUS: UNHEALTHY - Services down" >> "$REPORT_FILE"
    exit 1
fi
