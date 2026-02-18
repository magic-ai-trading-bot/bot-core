#!/bin/sh
# @spec:FR-MCP-017 - OpenClaw Entrypoint with Cron Registration
# @ref:plans/20260215-1900-openclaw-mcp-integration/phases/phase-05-cron-jobs-automation.md

set -e

echo "=== BotCore OpenClaw Gateway Starting ==="
echo "Timestamp: $(date -u +%Y-%m-%dT%H:%M:%SZ)"

# --- Config setup ---
# Docker mounts ./openclaw/config → /home/node/.openclaw (read-write)
# If a separate /config mount exists (legacy), copy it over
if [ -f /config/openclaw.json ] && [ "/config" != "/home/node/.openclaw" ]; then
  echo "Copying config from /config mount..."
  cp /config/openclaw.json /home/node/.openclaw/openclaw.json
  chmod 600 /home/node/.openclaw/openclaw.json
fi

# Ensure required directories exist
mkdir -p /home/node/.openclaw/agents/main/sessions \
         /home/node/.openclaw/credentials \
         /home/node/.openclaw/cron

# Symlink --dev profile → default profile so cron sub-agents (which use
# default ~/.openclaw/) share the same pairing data as the gateway
# (which runs with --dev → uses ~/.openclaw-dev/).
ln -sfn /home/node/.openclaw /home/node/.openclaw-dev

# --- Wait for MCP server ---
MCP_HEALTH_URL="${MCP_URL:-http://mcp-server:8090}/health"
echo "Waiting for MCP server at $MCP_HEALTH_URL ..."
MCP_RETRIES=0
MCP_MAX_RETRIES=60
until curl -sf "$MCP_HEALTH_URL" > /dev/null 2>&1; do
  MCP_RETRIES=$((MCP_RETRIES + 1))
  if [ $MCP_RETRIES -gt $MCP_MAX_RETRIES ]; then
    echo "ERROR: MCP server not ready after ${MCP_MAX_RETRIES}x5s. Exiting."
    exit 1
  fi
  echo "  MCP server not ready, retrying in 5s... ($MCP_RETRIES/$MCP_MAX_RETRIES)"
  sleep 5
done
echo "MCP server is healthy."

echo "=== Starting OpenClaw Gateway ==="

# Start gateway in background so we can register cron jobs after it's ready
openclaw gateway --dev --port 18789 --bind lan --token "${OPENCLAW_GATEWAY_TOKEN:-default-token}" &
GATEWAY_PID=$!

# Wait for gateway to be ready (canvas endpoint responds)
# Gateway can take 90-120s on first start (model loading, plugin init)
echo "Waiting for gateway to be ready..."
RETRIES=0
MAX_RETRIES=80
GATEWAY_READY=false
until curl -sf "http://localhost:18789/__openclaw__/canvas/" > /dev/null 2>&1; do
  # Check if gateway process died
  if ! kill -0 $GATEWAY_PID 2>/dev/null; then
    echo "ERROR: Gateway process died during startup"
    exit 1
  fi
  RETRIES=$((RETRIES + 1))
  if [ $RETRIES -gt $MAX_RETRIES ]; then
    echo "ERROR: Gateway did not start in time (${MAX_RETRIES}x3s = $((MAX_RETRIES * 3))s)"
    echo "  Container will keep running but cron jobs are NOT registered."
    echo "  Restart container to retry."
    break
  fi
  sleep 3
done

if [ $RETRIES -le $MAX_RETRIES ]; then
  GATEWAY_READY=true
fi

# --- Register cron jobs ---
GATEWAY_URL="ws://localhost:18789"
GATEWAY_TOKEN="${OPENCLAW_GATEWAY_TOKEN:-default-token}"
CRON_DIR="/home/node/.openclaw/cron"

if [ "$GATEWAY_READY" = "true" ] && [ -d "$CRON_DIR" ]; then
  echo "Gateway is ready. Registering cron jobs..."

  # Extra wait for WebSocket handlers to fully initialize
  sleep 5

  REGISTERED=0
  FAILED=0

  for f in "$CRON_DIR"/*.json; do
    [ -f "$f" ] || continue

    JOB_NAME=$(basename "$f" .json)

    # Skip registry file
    [ "$JOB_NAME" = "jobs" ] && continue

    # Extract all fields in one node call (efficient + atomic)
    JOB_DATA=$(node -e "
      try {
        const j = require('$f');
        const out = {
          schedule: j.schedule || '',
          prompt: j.prompt || '',
          timeout: j.timeout_seconds || 180,
          noDeliver: j.no_deliver === true
        };
        console.log(JSON.stringify(out));
      } catch(e) {
        console.error('Invalid JSON in $f: ' + e.message);
        process.exit(1);
      }
    " 2>&1) || { echo "  SKIP $JOB_NAME (invalid JSON)"; FAILED=$((FAILED + 1)); continue; }

    CRON_EXPR=$(echo "$JOB_DATA" | node -e "const d=JSON.parse(require('fs').readFileSync(0,'utf8')); console.log(d.schedule)")
    CRON_MSG=$(echo "$JOB_DATA" | node -e "const d=JSON.parse(require('fs').readFileSync(0,'utf8')); console.log(d.prompt)")
    CRON_TIMEOUT=$(echo "$JOB_DATA" | node -e "const d=JSON.parse(require('fs').readFileSync(0,'utf8')); console.log(d.timeout)")
    CRON_NO_DELIVER=$(echo "$JOB_DATA" | node -e "const d=JSON.parse(require('fs').readFileSync(0,'utf8')); console.log(d.noDeliver?'yes':'')")

    if [ -z "$CRON_EXPR" ] || [ -z "$CRON_MSG" ]; then
      echo "  SKIP $JOB_NAME (missing schedule or prompt)"
      FAILED=$((FAILED + 1))
      continue
    fi

    # --no-deliver for silent jobs (health-check, risk-monitor)
    # Report jobs (hourly-pnl, daily-portfolio, etc.) use auto-delivery
    DELIVER_FLAG=""
    if [ "$CRON_NO_DELIVER" = "yes" ]; then
      DELIVER_FLAG="--no-deliver"
    fi

    echo "  Registering: $JOB_NAME (${CRON_EXPR})${DELIVER_FLAG:+ [no-deliver]}"

    if openclaw --dev cron add \
      --url "$GATEWAY_URL" \
      --token "$GATEWAY_TOKEN" \
      --name "$JOB_NAME" \
      --cron "$CRON_EXPR" \
      --message "$CRON_MSG" \
      --timeout-seconds "$CRON_TIMEOUT" \
      $DELIVER_FLAG \
      2>&1 | tail -1; then
      REGISTERED=$((REGISTERED + 1))
    else
      echo "    FAILED to register $JOB_NAME"
      FAILED=$((FAILED + 1))
    fi
  done

  echo "Cron registration complete: $REGISTERED registered, $FAILED failed."
  if [ $FAILED -gt 0 ]; then
    echo "WARNING: Some cron jobs failed to register. Check logs above."
  fi
fi

# Wait for gateway process (keep container alive)
echo "=== Gateway running (PID $GATEWAY_PID) ==="
wait $GATEWAY_PID
