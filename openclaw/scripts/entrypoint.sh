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

# Ensure required directories exist
mkdir -p /home/node/.openclaw/agents/main/sessions /home/node/.openclaw/credentials
chmod 700 /home/node/.openclaw

echo "=== Starting OpenClaw Gateway ==="

# Start gateway in background so we can register cron jobs after it's ready
openclaw gateway --dev --port 18789 --bind lan --token "${OPENCLAW_GATEWAY_TOKEN:-default-token}" &
GATEWAY_PID=$!

# Wait for gateway to be ready (WebSocket listening)
echo "Waiting for gateway to be ready..."
RETRIES=0
until curl -sf "http://localhost:18789/__openclaw__/canvas/" > /dev/null 2>&1; do
  RETRIES=$((RETRIES + 1))
  if [ $RETRIES -gt 30 ]; then
    echo "  Gateway did not start in time, skipping cron registration"
    break
  fi
  sleep 2
done

# Register cron jobs AFTER gateway is running
CRON_DIR="/home/node/.openclaw/cron"
if [ -d "$CRON_DIR" ] && [ $RETRIES -le 30 ]; then
  echo "Registering cron jobs..."
  for f in "$CRON_DIR"/*.json; do
    if [ -f "$f" ]; then
      JOB_NAME=$(basename "$f" .json)
      # Skip jobs.json (registry file, not a job definition)
      if [ "$JOB_NAME" = "jobs" ]; then
        continue
      fi
      echo "  Registering: $JOB_NAME"
      openclaw cron add --file "$f" 2>&1 || echo "    (failed to register)"
    fi
  done
  echo "Cron job registration complete."
fi

# Wait for gateway process (keep container alive)
wait $GATEWAY_PID
