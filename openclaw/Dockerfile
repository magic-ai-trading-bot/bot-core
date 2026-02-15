# OpenClaw Gateway - BotCore Trading Bot Assistant
# @spec:FR-MCP-015 - OpenClaw Deployment
# @ref:plans/20260215-1900-openclaw-mcp-integration/phases/phase-04-openclaw-deployment.md

FROM node:22-slim

# Install system dependencies
RUN apt-get update && apt-get install -y --no-install-recommends \
    git curl ca-certificates build-essential cmake python3 \
    && rm -rf /var/lib/apt/lists/*

# Install openclaw globally (node-llama-cpp needs build tools)
RUN npm install -g openclaw@latest || \
    npm install -g --ignore-scripts openclaw@latest

WORKDIR /home/node

# Copy botcore-bridge CLI (MCP client for tool calls)
COPY scripts/botcore-bridge.mjs /usr/local/lib/botcore-bridge.mjs
RUN chmod +x /usr/local/lib/botcore-bridge.mjs

# Create a wrapper script for easy CLI access
RUN printf '#!/bin/sh\nnode /usr/local/lib/botcore-bridge.mjs "$@"\n' > /usr/local/bin/botcore && \
    chmod +x /usr/local/bin/botcore

# Copy entrypoint script (starts gateway + registers cron jobs)
COPY scripts/entrypoint.sh /usr/local/bin/entrypoint.sh
RUN chmod +x /usr/local/bin/entrypoint.sh

# Memory limit for Node.js (1536MB leaves room for OS overhead within 2048MB container)
ENV NODE_OPTIONS="--max-old-space-size=1536"
ENV HOME=/home/node

# Gateway WebSocket port
EXPOSE 18789

# Healthcheck: verify gateway process is alive (port may not open without Claude credentials)
HEALTHCHECK --interval=30s --timeout=10s --retries=5 --start-period=60s \
  CMD pgrep -f "openclaw" > /dev/null || exit 1

# Use entrypoint that starts gateway and registers cron jobs
CMD ["/usr/local/bin/entrypoint.sh"]
