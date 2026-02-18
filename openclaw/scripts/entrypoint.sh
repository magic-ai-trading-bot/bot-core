#!/bin/sh
# @spec:FR-MCP-017 - OpenClaw Entrypoint with Cron Registration
# @ref:plans/20260215-1900-openclaw-mcp-integration/phases/phase-05-cron-jobs-automation.md

set -e

echo "=== BotCore OpenClaw Gateway Starting ==="
echo "Timestamp: $(date -u +%Y-%m-%dT%H:%M:%SZ)"

# ===========================================================================
# CONFIG SYNC: staging mounts → named volume
# ===========================================================================
# Docker mounts git-tracked files to /config-source and /workspace-source
# (read-only). OpenClaw's live directory is a named volume at ~/.openclaw.
# We copy on every container start so config updates take effect on restart,
# but host-side git operations (pull, reset) NEVER affect the running container.
# This prevents the recurring issue where git ops trigger OpenClaw's config
# change detection → SIGUSR1 restart → in-memory cron jobs wiped.
# ===========================================================================

OPENCLAW_HOME="/home/node/.openclaw"
CONFIG_SRC="/config-source"
WORKSPACE_SRC="/workspace-source"

# Ensure required directories exist in the named volume
mkdir -p "$OPENCLAW_HOME/agents/main/sessions" \
         "$OPENCLAW_HOME/credentials" \
         "$OPENCLAW_HOME/cron" \
         "$OPENCLAW_HOME/workspace"

# Sync openclaw.json — use production config if NODE_ENV=production
if [ "$NODE_ENV" = "production" ] && [ -f "$CONFIG_SRC/openclaw.production.json" ]; then
  echo "Syncing production config to $OPENCLAW_HOME/openclaw.json"
  cp "$CONFIG_SRC/openclaw.production.json" "$OPENCLAW_HOME/openclaw.json"
elif [ -f "$CONFIG_SRC/openclaw.json" ]; then
  echo "Syncing dev config to $OPENCLAW_HOME/openclaw.json"
  cp "$CONFIG_SRC/openclaw.json" "$OPENCLAW_HOME/openclaw.json"
fi
chmod 600 "$OPENCLAW_HOME/openclaw.json" 2>/dev/null || true

# Sync cron job definitions (always overwrite so git changes take effect on restart)
if [ -d "$CONFIG_SRC/cron" ]; then
  echo "Syncing cron configs..."
  for f in "$CONFIG_SRC/cron"/*.json; do
    [ -f "$f" ] || continue
    cp "$f" "$OPENCLAW_HOME/cron/"
  done
fi

# Sync workspace (skills, docs — always overwrite for updates)
if [ -d "$WORKSPACE_SRC" ]; then
  echo "Syncing workspace..."
  cp -r "$WORKSPACE_SRC/"* "$OPENCLAW_HOME/workspace/" 2>/dev/null || true
fi

# Symlink --dev profile → default profile so cron sub-agents (which use
# default ~/.openclaw/) share the same pairing data as the gateway
# (which runs with --dev → uses ~/.openclaw-dev/).
ln -sfn "$OPENCLAW_HOME" /home/node/.openclaw-dev

echo "Config sync complete."

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

# --- Cron registration function ---
register_cron_jobs() {
  GATEWAY_URL="ws://localhost:18789"
  GATEWAY_TOKEN="${OPENCLAW_GATEWAY_TOKEN:-default-token}"
  CRON_DIR="$OPENCLAW_HOME/cron"

  [ -d "$CRON_DIR" ] || return 0

  echo "Registering cron jobs..."
  REGISTERED=0
  FAILED=0

  for f in "$CRON_DIR"/*.json; do
    [ -f "$f" ] || continue

    JOB_NAME=$(basename "$f" .json)
    [ "$JOB_NAME" = "jobs" ] && continue
    [ "$JOB_NAME" = "jobs.json" ] && continue

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

    # Use printf '%s' (not echo) to avoid /bin/sh interpreting \n in JSON strings
    CRON_EXPR=$(printf '%s' "$JOB_DATA" | node -e "const d=JSON.parse(require('fs').readFileSync(0,'utf8')); console.log(d.schedule)")
    CRON_MSG=$(printf '%s' "$JOB_DATA" | node -e "const d=JSON.parse(require('fs').readFileSync(0,'utf8')); console.log(d.prompt)")
    CRON_TIMEOUT=$(printf '%s' "$JOB_DATA" | node -e "const d=JSON.parse(require('fs').readFileSync(0,'utf8')); console.log(d.timeout)")
    CRON_NO_DELIVER=$(printf '%s' "$JOB_DATA" | node -e "const d=JSON.parse(require('fs').readFileSync(0,'utf8')); console.log(d.noDeliver?'yes':'')")

    if [ -z "$CRON_EXPR" ] || [ -z "$CRON_MSG" ]; then
      echo "  SKIP $JOB_NAME (missing schedule or prompt)"
      FAILED=$((FAILED + 1))
      continue
    fi

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
}

# --- Register cron jobs ---
if [ "$GATEWAY_READY" = "true" ]; then
  echo "Gateway is ready."

  # Wait for WebSocket handlers + potential config-triggered restart.
  # OpenClaw may auto-modify openclaw.json (e.g. adding plugins section),
  # which triggers a SIGUSR1 self-restart that wipes in-memory cron jobs.
  # We wait 15s after startup to let any config migration happen first.
  echo "Waiting 15s for config stabilization..."
  sleep 15

  # Verify gateway is still responsive after potential restart
  if curl -sf "http://localhost:18789/__openclaw__/canvas/" > /dev/null 2>&1; then
    register_cron_jobs
  else
    echo "Gateway restarted during stabilization, waiting for it to come back..."
    RESTAB=0
    until curl -sf "http://localhost:18789/__openclaw__/canvas/" > /dev/null 2>&1; do
      RESTAB=$((RESTAB + 1))
      if [ $RESTAB -gt 60 ]; then
        echo "ERROR: Gateway did not recover after restart. Cron NOT registered."
        break
      fi
      sleep 3
    done
    if [ $RESTAB -le 60 ]; then
      echo "Gateway recovered. Waiting 5s for full init..."
      sleep 5
      register_cron_jobs
    fi
  fi
fi

# Wait for gateway process (keep container alive)
echo "=== Gateway running (PID $GATEWAY_PID) ==="
wait $GATEWAY_PID
