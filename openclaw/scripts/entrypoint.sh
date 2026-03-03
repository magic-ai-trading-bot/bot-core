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

# Locate production config (support both mount paths for compatibility)
if [ "$NODE_ENV" = "production" ]; then
  if [ -f "$CONFIG_SRC/openclaw.production.json" ]; then
    echo "Syncing production config from $CONFIG_SRC/openclaw.production.json"
    cp "$CONFIG_SRC/openclaw.production.json" "$OPENCLAW_HOME/openclaw.json"
  elif [ -f "/config/openclaw.json" ]; then
    echo "Syncing production config from /config/openclaw.json"
    cp "/config/openclaw.json" "$OPENCLAW_HOME/openclaw.json"
  fi
elif [ -f "$CONFIG_SRC/openclaw.json" ]; then
  echo "Syncing dev config to $OPENCLAW_HOME/openclaw.json"
  cp "$CONFIG_SRC/openclaw.json" "$OPENCLAW_HOME/openclaw.json"
fi
chmod 600 "$OPENCLAW_HOME/openclaw.json" 2>/dev/null || true

# Sync cron job definitions (always overwrite so git changes take effect on restart)
if [ -d "$CONFIG_SRC/cron" ]; then
  echo "Syncing cron configs..."
  # Remove stale cron files that no longer exist in source
  for f in "$OPENCLAW_HOME/cron"/*.json; do
    [ -f "$f" ] || continue
    basename_f=$(basename "$f")
    [ "$basename_f" = "jobs.json" ] && continue
    [ "$basename_f" = "jobs.json.bak" ] && continue
    if [ ! -f "$CONFIG_SRC/cron/$basename_f" ]; then
      echo "  Removing stale cron config: $basename_f"
      rm -f "$f"
    fi
  done
  # Copy current configs
  for f in "$CONFIG_SRC/cron"/*.json; do
    [ -f "$f" ] || continue
    cp "$f" "$OPENCLAW_HOME/cron/"
  done
fi

# Clean stale session lock files from previous container runs
# (locks are not released when container is killed mid-session)
STALE_LOCKS=$(find "$OPENCLAW_HOME/agents" -name "*.lock" -type f 2>/dev/null | wc -l)
if [ "$STALE_LOCKS" -gt 0 ]; then
  echo "Removing $STALE_LOCKS stale session lock file(s)..."
  find "$OPENCLAW_HOME/agents" -name "*.lock" -type f -delete
fi

# Sync workspace (skills, docs — always overwrite for updates)
if [ -d "$WORKSPACE_SRC" ]; then
  echo "Syncing workspace..."
  cp -r "$WORKSPACE_SRC/"* "$OPENCLAW_HOME/workspace/" 2>/dev/null || true
fi

# Symlink profiles so all OpenClaw invocations share the same state directory
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

# ===========================================================================
# CRON JOB REGISTRATION: Write directly to jobs.json
# ===========================================================================
# The openclaw CLI's `cron add` requires device pairing (Ed25519 challenge),
# which doesn't work reliably in Docker. Instead, we write jobs directly to
# jobs.json in the format the gateway reads at startup.
# ===========================================================================

echo "Building cron jobs.json from config files..."
node -e "
  const fs = require('fs');
  const path = require('path');
  const crypto = require('crypto');
  const cronDir = '$OPENCLAW_HOME/cron';
  const jobsPath = path.join(cronDir, 'jobs.json');
  const tz = process.env.TZ || 'UTC';

  const files = fs.readdirSync(cronDir).filter(f =>
    f.endsWith('.json') && f !== 'jobs.json' && f !== 'jobs.json.bak' && !f.startsWith('.')
  );

  const jobs = [];
  for (const file of files) {
    try {
      const cfg = JSON.parse(fs.readFileSync(path.join(cronDir, file), 'utf8'));
      if (!cfg.schedule || !cfg.prompt) {
        console.log('  SKIP ' + file + ' (missing schedule or prompt)');
        continue;
      }
      const name = file.replace('.json', '');
      const noDeliver = cfg.no_deliver === true;
      const timeout = cfg.timeout_seconds || 180;

      const job = {
        jobId: crypto.randomUUID(),
        name: name,
        schedule: {
          kind: 'cron',
          expr: cfg.schedule,
          tz: tz
        },
        sessionTarget: 'isolated',
        wakeMode: 'now',
        payload: {
          kind: 'agentTurn',
          message: cfg.prompt
        },
        timeoutSeconds: timeout,
        enabled: true
      };

      if (!noDeliver) {
        job.delivery = { mode: 'announce' };
      }

      jobs.push(job);
      const deliverLabel = noDeliver ? ' [no-deliver]' : '';
      console.log('  Added: ' + name + ' (' + cfg.schedule + ')' + deliverLabel);
    } catch (e) {
      console.log('  SKIP ' + file + ': ' + e.message);
    }
  }

  const jobsData = { version: 1, jobs: jobs };
  fs.writeFileSync(jobsPath, JSON.stringify(jobsData, null, 2));
  console.log('Wrote ' + jobs.length + ' cron jobs to jobs.json');
" 2>&1

echo "=== Starting OpenClaw Gateway ==="

# Start gateway (reads jobs.json at startup for cron scheduling)
openclaw gateway --port 18789 --bind lan --token "${OPENCLAW_GATEWAY_TOKEN:-default-token}" &
GATEWAY_PID=$!

# Wait for gateway to be ready
echo "Waiting for gateway to be ready..."
RETRIES=0
MAX_RETRIES=80
until curl -sf "http://localhost:18789/" > /dev/null 2>&1; do
  if ! kill -0 $GATEWAY_PID 2>/dev/null; then
    echo "ERROR: Gateway process died during startup"
    exit 1
  fi
  RETRIES=$((RETRIES + 1))
  if [ $RETRIES -gt $MAX_RETRIES ]; then
    echo "ERROR: Gateway did not start in time (${MAX_RETRIES}x3s = $((MAX_RETRIES * 3))s)"
    echo "  Container will keep running but cron jobs may not be active."
    break
  fi
  sleep 3
done

if [ $RETRIES -le $MAX_RETRIES ]; then
  echo "Gateway is ready."
fi

# Wait for gateway process (keep container alive)
echo "=== Gateway running (PID $GATEWAY_PID) ==="
wait $GATEWAY_PID
