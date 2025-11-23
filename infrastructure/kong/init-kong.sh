#!/bin/bash
# Kong API Gateway Initialization Script
# This script automatically configures Kong services and routes

set -e

echo "👑 Initializing Kong API Gateway..."

# Wait for Kong to be ready
echo "Waiting for Kong to be ready..."
until curl -s http://localhost:8001/status > /dev/null 2>&1; do
    echo "Waiting for Kong..."
    sleep 2
done

echo "✅ Kong is ready!"

# Function to create or update service
create_or_update_service() {
    local service_name=$1
    local service_url=$2

    echo "Setting up service: $service_name"

    # Check if service exists
    if curl -s http://localhost:8001/services/$service_name | grep -q "\"id\""; then
        echo "  Service $service_name already exists, skipping..."
    else
        curl -s -X POST http://localhost:8001/services \
          -H "Content-Type: application/json" \
          -d "{\"name\": \"$service_name\", \"url\": \"$service_url\"}" > /dev/null
        echo "  ✅ Service $service_name created"
    fi
}

# Function to create or update route
create_or_update_route() {
    local service_name=$1
    local route_name=$2
    local path=$3
    local strip_path=$4

    echo "Setting up route: $route_name"

    # Check if route exists
    if curl -s http://localhost:8001/routes/$route_name | grep -q "\"id\""; then
        echo "  Route $route_name already exists, skipping..."
    else
        curl -s -X POST http://localhost:8001/services/$service_name/routes \
          -H "Content-Type: application/json" \
          -d "{\"name\": \"$route_name\", \"paths\": [\"$path\"], \"strip_path\": $strip_path}" > /dev/null
        echo "  ✅ Route $route_name created"
    fi
}

echo ""
echo "=== Configuring Services ==="

# 1. Rust Core Engine API
create_or_update_service "rust-core-api" "http://rust-core-engine-dev:8080"
create_or_update_route "rust-core-api" "rust-core-route" "/api" "false"

# 2. Python AI Service
create_or_update_service "python-ai-api" "http://python-ai-service-dev:8000"
create_or_update_route "python-ai-api" "python-ai-route" "/ai" "true"

# 3. Next.js Dashboard
create_or_update_service "nextjs-dashboard" "http://nextjs-ui-dashboard-dev:3000"
create_or_update_route "nextjs-dashboard" "nextjs-dashboard-route" "/dashboard" "true"

# 4. Welcome route (lowest priority - matches root)
create_or_update_service "kong-welcome" "http://localhost:8001"
create_or_update_route "kong-welcome" "welcome-route" "/" "false"

echo ""
echo "=== Kong Configuration Summary ==="
curl -s http://localhost:8001/services | python3 -c "import sys, json; d=json.load(sys.stdin); print(f'Total Services: {len(d[\"data\"])}'); [print(f'  - {s[\"name\"]}: {s[\"protocol\"]}://{s[\"host\"]}:{s[\"port\"]}') for s in d['data']]"

echo ""
curl -s http://localhost:8001/routes | python3 -c "import sys, json; d=json.load(sys.stdin); print(f'Total Routes: {len(d[\"data\"])}'); [print(f'  - {r[\"name\"]}: {r[\"paths\"]} (strip_path: {r[\"strip_path\"]})') for r in d['data']]"

echo ""
echo "✅ Kong initialization complete!"
echo ""
echo "Kong Access:"
echo "  Admin API: http://localhost:8001"
echo "  Proxy:     http://localhost:8100"
echo ""
echo "Available endpoints via proxy:"
echo "  - Rust API:    http://localhost:8100/api/health"
echo "  - Python AI:   http://localhost:8100/ai/health"
echo "  - Dashboard:   http://localhost:8100/dashboard/ (dev: use http://localhost:3000)"
echo "  - Welcome:     http://localhost:8100/"
