#!/bin/bash
# Seed admin user for MCP Server authentication
# This script creates the admin user if it doesn't exist
# Credentials come from .env: BOTCORE_EMAIL and BOTCORE_PASSWORD

set -e

# Default values (can be overridden by env vars)
API_URL="${RUST_API_URL:-http://localhost:8080}"
EMAIL="${BOTCORE_EMAIL:-admin@botcore.local}"
PASSWORD="${BOTCORE_PASSWORD:-changeme}"
NAME="${BOTCORE_ADMIN_NAME:-BotCore Admin}"

echo "Seeding admin user: $EMAIL"

# Try login first — if it works, user already exists
LOGIN_RESPONSE=$(curl -sf -X POST "$API_URL/api/auth/login" \
  -H "Content-Type: application/json" \
  -d "{\"email\":\"$EMAIL\",\"password\":\"$PASSWORD\"}" 2>/dev/null || true)

if echo "$LOGIN_RESPONSE" | grep -q '"success":true'; then
  echo "Admin user already exists — skipping"
  exit 0
fi

# Register new user
REGISTER_RESPONSE=$(curl -sf -X POST "$API_URL/api/auth/register" \
  -H "Content-Type: application/json" \
  -d "{\"email\":\"$EMAIL\",\"password\":\"$PASSWORD\",\"name\":\"$NAME\"}" 2>/dev/null || true)

if echo "$REGISTER_RESPONSE" | grep -q '"success":true'; then
  echo "Admin user created successfully"
else
  echo "ERROR: Failed to create admin user"
  echo "Response: $REGISTER_RESPONSE"
  exit 1
fi
