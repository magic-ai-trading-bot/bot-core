#!/bin/bash
# RabbitMQ Initialization Script
# This script sets up RabbitMQ with proper vhosts and permissions

set -e

echo "🐰 Initializing RabbitMQ..."

# Wait for RabbitMQ to be ready
sleep 10

echo "1. Creating vhosts..."
# Create default vhost (required for Management UI)
rabbitmqctl add_vhost / || echo "Vhost / already exists"

# Create bot-core vhost (already created by docker-compose, but ensure it exists)
rabbitmqctl add_vhost bot-core || echo "Vhost bot-core already exists"

echo "2. Setting up admin user permissions..."
# Set permissions for admin user on both vhosts
rabbitmqctl set_permissions -p / admin ".*" ".*" ".*"
rabbitmqctl set_permissions -p bot-core admin ".*" ".*" ".*"

# Ensure admin has proper tags
rabbitmqctl set_user_tags admin administrator management

echo "3. Creating additional management user for easy access..."
# Create a simple user for Management UI access
rabbitmqctl add_user mgmt admin123 || echo "User mgmt already exists"
rabbitmqctl set_user_tags mgmt administrator management
rabbitmqctl set_permissions -p / mgmt ".*" ".*" ".*"
rabbitmqctl set_permissions -p bot-core mgmt ".*" ".*" ".*"

echo "4. Listing users and permissions..."
rabbitmqctl list_users
echo ""
rabbitmqctl list_permissions -p /
echo ""
rabbitmqctl list_permissions -p bot-core

echo "✅ RabbitMQ initialization complete!"
echo ""
echo "Management UI Access:"
echo "  URL: http://localhost:15672"
echo "  User 1: admin / <password from .env>"
echo "  User 2: mgmt / admin123"
