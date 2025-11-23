#!/bin/bash
# Master Initialization Script for All Services
# This script runs all initialization scripts in the correct order

set -e

echo "================================================"
echo "🚀 Bot Core - Service Initialization"
echo "================================================"
echo ""

# Color codes
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
RED='\033[0;31m'
NC='\033[0m' # No Color

# Function to wait for service
wait_for_service() {
    local service=$1
    local port=$2
    local max_wait=$3

    echo -e "${YELLOW}Waiting for $service to be ready on port $port...${NC}"

    count=0
    until nc -z localhost $port 2>/dev/null || [ $count -eq $max_wait ]; do
        sleep 2
        count=$((count + 1))
        if [ $((count % 5)) -eq 0 ]; then
            echo "  Still waiting... ($count/${max_wait})"
        fi
    done

    if [ $count -eq $max_wait ]; then
        echo -e "${RED}❌ $service did not start in time${NC}"
        return 1
    fi

    echo -e "${GREEN}✅ $service is ready!${NC}"
    echo ""
}

# Function to run initialization script in container
run_init_in_container() {
    local container=$1
    local script_path=$2
    local script_name=$3

    echo "================================================"
    echo "🔧 Initializing: $script_name"
    echo "================================================"

    if docker exec $container sh -c "test -f $script_path"; then
        docker exec $container bash $script_path
        if [ $? -eq 0 ]; then
            echo -e "${GREEN}✅ $script_name completed successfully${NC}"
        else
            echo -e "${RED}❌ $script_name failed${NC}"
            return 1
        fi
    else
        echo -e "${YELLOW}⚠️  Script not found in container: $script_path${NC}"
        return 1
    fi

    echo ""
}

# Main initialization flow
echo "Step 1/4: Waiting for core services..."
wait_for_service "RabbitMQ" 5672 30
wait_for_service "Grafana" 3001 30
wait_for_service "Kong" 8001 30

echo "Step 2/4: Initializing RabbitMQ..."
if docker exec rabbitmq test -f /etc/rabbitmq/init-rabbitmq.sh; then
    docker exec rabbitmq bash /etc/rabbitmq/init-rabbitmq.sh
else
    echo -e "${YELLOW}⚠️  Running RabbitMQ init from host...${NC}"
    docker exec rabbitmq bash << 'EOF'
#!/bin/bash
set -e

echo "🐰 Setting up RabbitMQ..."

# Create vhosts
rabbitmqctl add_vhost / 2>/dev/null || true
rabbitmqctl add_vhost bot-core 2>/dev/null || true

# Set permissions for admin user
rabbitmqctl set_permissions -p / admin ".*" ".*" ".*"
rabbitmqctl set_permissions -p bot-core admin ".*" ".*" ".*"
rabbitmqctl set_user_tags admin administrator management

# Create management user
rabbitmqctl add_user mgmt admin123 2>/dev/null || true
rabbitmqctl set_user_tags mgmt administrator management
rabbitmqctl set_permissions -p / mgmt ".*" ".*" ".*"
rabbitmqctl set_permissions -p bot-core mgmt ".*" ".*" ".*"

echo "✅ RabbitMQ setup complete"
rabbitmqctl list_users
EOF
fi
echo ""

echo "Step 3/4: Initializing Grafana..."
if docker exec grafana test -f /etc/grafana/init-grafana.sh; then
    docker exec grafana bash /etc/grafana/init-grafana.sh
else
    echo -e "${YELLOW}⚠️  Running Grafana init from host...${NC}"
    GRAFANA_PASS=${GRAFANA_PASSWORD:-admin123}
    docker exec grafana grafana cli admin reset-admin-password "$GRAFANA_PASS"
    echo -e "${GREEN}✅ Grafana password set to: $GRAFANA_PASS${NC}"
fi
echo ""

echo "Step 4/4: Initializing Kong..."
if [ -f "./infrastructure/kong/init-kong.sh" ]; then
    bash ./infrastructure/kong/init-kong.sh
else
    echo -e "${RED}❌ Kong init script not found${NC}"
fi
echo ""

echo "================================================"
echo "✅ All Services Initialized Successfully!"
echo "================================================"
echo ""
echo "Service Access URLs:"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo ""
echo "📊 Monitoring & Management:"
echo "  - RabbitMQ:    http://localhost:15672 (mgmt/admin123)"
echo "  - Grafana:     http://localhost:3001 (admin/\$GRAFANA_PASSWORD)"
echo "  - Prometheus:  http://localhost:9090"
echo "  - Kong Admin:  http://localhost:8001"
echo "  - Flower:      http://localhost:5555"
echo ""
echo "🔀 Kong Proxy (http://localhost:8100):"
echo "  - Rust API:    http://localhost:8100/api/health"
echo "  - Python AI:   http://localhost:8100/ai/health"
echo "  - Dashboard:   http://localhost:8100/dashboard/"
echo ""
echo "💻 Direct Access:"
echo "  - Rust API:    http://localhost:8080/api/health"
echo "  - Python AI:   http://localhost:8000/health"
echo "  - Dashboard:   http://localhost:3000"
echo ""
echo "================================================"
