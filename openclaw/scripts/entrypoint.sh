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

# Wait for gateway to be ready (canvas endpoint responds)
# Gateway can take 90+ seconds on first start (model loading, plugin init)
echo "Waiting for gateway to be ready..."
RETRIES=0
MAX_RETRIES=60
until curl -sf "http://localhost:18789/__openclaw__/canvas/" > /dev/null 2>&1; do
  RETRIES=$((RETRIES + 1))
  if [ $RETRIES -gt $MAX_RETRIES ]; then
    echo "  Gateway did not start in time (${MAX_RETRIES}x3s), skipping cron registration"
    break
  fi
  sleep 3
done

# Register cron jobs AFTER gateway is running
# Uses openclaw --dev cron add with explicit --url and --token
GATEWAY_URL="ws://localhost:18789"
GATEWAY_TOKEN="${OPENCLAW_GATEWAY_TOKEN:-default-token}"
CRON_DIR="/home/node/.openclaw/cron"

if [ -d "$CRON_DIR" ] && [ $RETRIES -le $MAX_RETRIES ]; then
  echo "Gateway is ready. Registering cron jobs..."

  # Wait a few extra seconds for the gateway to fully initialize WebSocket handlers
  sleep 5

  for f in "$CRON_DIR"/*.json; do
    if [ -f "$f" ]; then
      JOB_NAME=$(basename "$f" .json)
      # Skip jobs.json (registry file, not a job definition)
      if [ "$JOB_NAME" = "jobs" ]; then
        continue
      fi

      # Extract fields from JSON config using node (available in container)
      CRON_EXPR=$(node -e "const j=require('$f'); console.log(j.schedule||'')")
      CRON_MSG=$(node -e "const j=require('$f'); console.log(j.prompt||'')")
      CRON_TIMEOUT=$(node -e "const j=require('$f'); console.log(j.timeout_seconds||180)")

      if [ -z "$CRON_EXPR" ] || [ -z "$CRON_MSG" ]; then
        echo "  Skipping $JOB_NAME (missing schedule or prompt)"
        continue
      fi

      echo "  Registering: $JOB_NAME (${CRON_EXPR})"

      # Always use --no-deliver: cron prompts handle Telegram delivery
      # via `botcore send_telegram_notification` when needed.
      # Without --no-deliver, OpenClaw delivers raw AI responses to
      # Telegram with "(error)" labels for short/silent responses.
      openclaw --dev cron add \
        --url "$GATEWAY_URL" \
        --token "$GATEWAY_TOKEN" \
        --name "$JOB_NAME" \
        --cron "$CRON_EXPR" \
        --message "$CRON_MSG" \
        --timeout-seconds "$CRON_TIMEOUT" \
        --no-deliver \
        2>&1 | tail -1 || echo "    (failed to register $JOB_NAME)"
    fi
  done
  echo "Cron job registration complete."
fi

# Wait for gateway process (keep container alive)
wait $GATEWAY_PID
