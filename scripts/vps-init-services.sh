#!/bin/bash
# =============================================================================
# VPS SERVICE INITIALIZATION SCRIPT
# =============================================================================
# Ensures all services are properly configured and running
# Run after deployment or when services have issues
# Usage: ./scripts/vps-init-services.sh
# =============================================================================

set -e

echo "=============================================="
echo "VPS Service Initialization - $(date)"
echo "=============================================="

cd /root/bot-core || { echo "Error: /root/bot-core not found"; exit 1; }

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m'

print_status() { echo -e "${GREEN}✅ $1${NC}"; }
print_warning() { echo -e "${YELLOW}⚠️  $1${NC}"; }
print_error() { echo -e "${RED}❌ $1${NC}"; }

# =============================================================================
# 1. Check core services
# =============================================================================
echo ""
echo "🚀 Checking core services..."

check_service_health() {
    local name=$1
    local url=$2

    if curl -sf "$url" > /dev/null 2>&1; then
        print_status "$name is healthy"
        return 0
    else
        print_error "$name is NOT healthy"
        return 1
    fi
}

check_service_health "Rust API" "http://localhost:8080/api/health"
check_service_health "Python AI" "http://localhost:8000/health"
check_service_health "Frontend" "http://localhost:3000"

# =============================================================================
# 2. Check MongoDB
# =============================================================================
echo ""
echo "🗄️ Checking MongoDB..."

if docker ps | grep -q "mongodb.*healthy"; then
    print_status "MongoDB is healthy"

    # Check if users collection exists
    user_count=$(docker exec mongodb mongosh "mongodb://admin:${MONGO_ROOT_PASSWORD:-BotCore2024Secure}@localhost:27017/bot_core?authSource=admin" --eval "db.users.countDocuments()" --quiet 2>/dev/null || echo "0")
    echo "   Users in database: $user_count"
else
    print_error "MongoDB is NOT healthy"
fi

# =============================================================================
# 3. Final status
# =============================================================================
echo ""
echo "=============================================="
echo "📋 Final Service Status"
echo "=============================================="
docker ps --format "table {{.Names}}\t{{.Status}}" | sort

echo ""
echo "=============================================="
echo "✅ Initialization complete - $(date)"
echo "=============================================="
