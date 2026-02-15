#!/bin/sh
# @spec:FR-MCP-017 - OpenClaw Entrypoint with Cron Registration
# @ref:plans/20260215-1900-openclaw-mcp-integration/phases/phase-05-cron-jobs-automation.md

set -e

echo "=== BotCore OpenClaw Gateway Starting ==="
echo "Timestamp: $(date -u +%Y-%m-%dT%H:%M:%SZ)"

# Copy mounted config to writable location (Docker bind-mount files don't support atomic rename)
if [ -f /config/openclaw.json ]; then
  echo "Copying config to writable location..."
  cp /config/openclaw.json /home/node/.openclaw/openclaw.json
  chmod 600 /home/node/.openclaw/openclaw.json
fi

# Wait for MCP server to be ready
MCP_HEALTH_URL="${MCP_URL:-http://mcp-server:8090}/health"
echo "Waiting for MCP server at $MCP_HEALTH_URL ..."
until curl -sf "$MCP_HEALTH_URL" > /dev/null 2>&1; do
  echo "  MCP server not ready, retrying in 5s..."
  sleep 5
done
echo "MCP server is healthy."

# Register cron jobs first (before gateway blocks)
CRON_DIR="/home/node/.openclaw/cron"
if [ -d "$CRON_DIR" ]; then
  echo "Registering cron jobs..."
  for f in "$CRON_DIR"/*.json; do
    if [ -f "$f" ]; then
      JOB_NAME=$(basename "$f" .json)
      echo "  Registering: $JOB_NAME"
      openclaw cron add --file "$f" 2>/dev/null || echo "    (already registered or skipped)"
    fi
  done
  echo "Cron job registration complete."
fi

echo "=== Starting OpenClaw Gateway ==="

# Ensure required directories exist
mkdir -p /home/node/.openclaw/agents/main/sessions /home/node/.openclaw/credentials
chmod 700 /home/node/.openclaw

# Start gateway with explicit flags (--dev skips device pairing requirement)
exec openclaw gateway --dev --port 18789 --bind lan --token "${OPENCLAW_GATEWAY_TOKEN:-default-token}"
