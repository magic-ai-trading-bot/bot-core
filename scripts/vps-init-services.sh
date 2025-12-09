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
# 1. Check and fix RabbitMQ
# =============================================================================
echo ""
echo "📨 Checking RabbitMQ..."

if docker ps | grep -q "rabbitmq.*healthy"; then
    print_status "RabbitMQ is healthy"

    # Check if vhost exists
    if docker exec rabbitmq rabbitmqctl list_vhosts 2>/dev/null | grep -q "bot-core"; then
        print_status "RabbitMQ vhost 'bot-core' exists"
    else
        print_warning "RabbitMQ vhost missing, creating..."
        docker exec rabbitmq rabbitmqctl add_vhost bot-core || true
        docker exec rabbitmq rabbitmqctl set_permissions -p bot-core admin ".*" ".*" ".*" || true
        print_status "RabbitMQ vhost created"
    fi

    # Check if user exists
    if docker exec rabbitmq rabbitmqctl list_users 2>/dev/null | grep -q "admin"; then
        print_status "RabbitMQ user 'admin' exists"
    else
        print_warning "RabbitMQ user missing, this should be auto-created on startup"
    fi
else
    print_warning "RabbitMQ not healthy, restarting..."
    docker compose --profile messaging restart rabbitmq
    sleep 30

    if docker ps | grep -q "rabbitmq.*healthy"; then
        print_status "RabbitMQ restarted successfully"
    else
        print_error "RabbitMQ still unhealthy, may need manual intervention"
    fi
fi

# =============================================================================
# 2. Check and fix Celery services
# =============================================================================
echo ""
echo "🔄 Checking Celery services..."

# Check celery-worker
if docker ps | grep -q "celery-worker"; then
    worker_status=$(docker inspect --format='{{.State.Health.Status}}' celery-worker 2>/dev/null || echo "unknown")
    if [ "$worker_status" = "healthy" ]; then
        print_status "Celery Worker is healthy"
    else
        print_warning "Celery Worker status: $worker_status, restarting..."
        docker compose --profile messaging restart celery-worker
        sleep 20
    fi
else
    print_warning "Celery Worker not running, starting..."
    docker compose --profile messaging up -d celery-worker
    sleep 20
fi

# Check celery-beat
if docker ps | grep -q "celery-beat"; then
    beat_status=$(docker inspect --format='{{.State.Health.Status}}' celery-beat 2>/dev/null || echo "unknown")
    if [ "$beat_status" = "healthy" ]; then
        print_status "Celery Beat is healthy"
    else
        print_warning "Celery Beat status: $beat_status (non-critical)"
    fi
else
    print_warning "Celery Beat not running, starting..."
    docker compose --profile messaging up -d celery-beat
fi

# =============================================================================
# 3. Check core services
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
# 4. Check MongoDB
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
# 5. Final status
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
