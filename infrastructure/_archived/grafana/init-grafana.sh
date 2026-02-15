#!/bin/bash
# Grafana Initialization Script
# This script resets Grafana admin password to match environment variable

set -e

echo "📊 Initializing Grafana..."

# Wait for Grafana to be ready
sleep 15

# Get password from environment variable or use default
GRAFANA_ADMIN_PASSWORD=${GRAFANA_PASSWORD:-admin123}

echo "Resetting admin password..."
grafana cli admin reset-admin-password "$GRAFANA_ADMIN_PASSWORD"

echo "✅ Grafana initialization complete!"
echo ""
echo "Grafana Access:"
echo "  URL: http://localhost:3001"
echo "  Username: admin"
echo "  Password: $GRAFANA_ADMIN_PASSWORD"
