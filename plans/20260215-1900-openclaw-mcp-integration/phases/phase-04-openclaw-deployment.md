# Phase 04: OpenClaw Deployment & Channel Configuration

**Status**: Pending | **Est.**: 1.5 days | **Priority**: P0 (user-facing)

## Context Links

- [Research: OpenClaw Docker Deployment](../research/researcher-01-openclaw-mcp.md#3-openclaw-docker-deployment)
- [Research: Telegram + WhatsApp Setup](../research/researcher-01-openclaw-mcp.md#4-telegram--whatsapp-baileys-channel-setup)
- [VPS Docker Compose](../../../../docker-compose-vps.yml)
- Phase 01 (MCP server running)

## Overview

Deploy OpenClaw as a Docker container alongside existing BotCore services. Configure Telegram and WhatsApp channels. Connect OpenClaw to the MCP server via HTTP transport. Set up Claude Max subscription as the LLM backend (no extra LLM cost).

## Key Insights

1. OpenClaw runs as Node.js process, needs ~1GB memory. VPS has 4GB total, current usage ~3GB (Mongo 512MB + Python 1GB + Rust 1GB + Next.js 512MB). Need to optimize.
2. **Memory strategy**: Reduce Next.js dashboard to 256MB (it is lightweight), reduce MongoDB to 384MB. Frees ~384MB. OpenClaw gets 768MB, MCP server gets 384MB. Total: ~3.8GB with swap as buffer.
3. OpenClaw connects to Claude API using user's Max subscription API key -- no per-token cost.
4. MCP plugin config points to internal `http://mcp-server:8090` (Docker network).
5. Telegram is primary channel (simpler setup). WhatsApp is secondary (requires phone online for Baileys).
6. OpenClaw persistent data at `/data/openclaw/` volume mount (auth state, cron jobs, conversation history).

## Requirements

- OpenClaw container running headless (no UI needed)
- Claude Max API key configured as LLM provider
- MCP server connected via HTTP transport
- Telegram bot configured and responding to messages
- WhatsApp configured via Baileys with QR code pairing
- Channel security: allowlist mode (only owner's accounts)
- Persistent data survives container restarts
- Memory usage stays within VPS budget

## Architecture

```
                 Internet
                    |
        +-----------+-----------+
        |                       |
   Telegram API          WhatsApp (Baileys)
        |                       |
        +-----------+-----------+
                    |
        +-----------+-----------+
        |   OpenClaw Container  |
        |   (node:18-alpine)    |
        |   768MB memory limit  |
        |                       |
        |  LLM: Claude Max API  |
        |  MCP: http://mcp-     |
        |       server:8090     |
        |  Data: /data/openclaw |
        +-----------+-----------+
                    |
            Docker Network
            (bot-network)
                    |
        +-----------+-----------+
        |   MCP Server (:8090)  |
        |   384MB memory limit  |
        +-----------+-----------+
                    |
        +-----------+-----------+
        |                       |
   Rust :8080            Python :8000
```

## Related Code Files

| File | Purpose |
|------|---------|
| `openclaw/` (new) | OpenClaw configuration directory |
| `openclaw/Dockerfile` | Custom OpenClaw Docker image |
| `openclaw/config/openclaw.json` | Main OpenClaw configuration |
| `openclaw/config/mcp.json` | MCP server connection config |
| `openclaw/config/channels.json` | Telegram + WhatsApp channel config |
| `openclaw/config/system-prompt.md` | System prompt for Claude (trading bot context) |
| `docker-compose-vps.yml` | Updated with OpenClaw + adjusted memory limits |

## Implementation Steps

### 1. OpenClaw Docker Setup (~2h)

**`openclaw/Dockerfile`**:
```dockerfile
FROM node:18-alpine
RUN npm install -g openclaw@latest
WORKDIR /app
COPY config/ /root/.openclaw/
ENV NODE_OPTIONS="--max-old-space-size=640"
ENV OPENCLAW_MODE=headless
EXPOSE 3001
CMD ["openclaw", "start", "--headless"]
```

### 2. OpenClaw Configuration (~3h)

**`openclaw/config/openclaw.json`**:
```json
{
  "llm": {
    "provider": "anthropic",
    "model": "claude-sonnet-4-20250514",
    "apiKey": "${ANTHROPIC_API_KEY}",
    "maxTokens": 8192
  },
  "mcp": {
    "servers": [
      {
        "name": "botcore",
        "type": "http",
        "url": "http://mcp-server:8090",
        "auth": {
          "type": "bearer",
          "token": "${MCP_AUTH_TOKEN}"
        }
      }
    ]
  },
  "system_prompt_file": "/root/.openclaw/system-prompt.md",
  "data_dir": "/data/openclaw",
  "log_level": "info"
}
```

**`openclaw/config/system-prompt.md`**:
```markdown
You are a trading bot assistant for BotCore, a cryptocurrency trading system.
You have access to MCP tools that let you monitor, control, and tune the trading bot.

CAPABILITIES:
- Monitor market prices, positions, portfolio, and system health
- View and analyze trading performance
- Adjust trading parameters within safety bounds
- Trigger AI analysis for trading signals
- Manage paper trading (start/stop, settings)
- Run backtests and manage ML model training

SAFETY RULES:
- NEVER enable real trading without explicit user instruction
- ALWAYS show current values before proposing changes
- For parameter adjustments, explain your reasoning
- Respect the 3-tier security system (GREEN/YELLOW/RED)
- When in doubt, ask the user instead of acting

COMMUNICATION STYLE:
- Be concise but informative
- Use numbers and data to support recommendations
- Format tables for portfolio/performance data
- Alert immediately on risk events or significant losses
```

### 3. Telegram Channel Setup (~2h)

**Prerequisites**: Create Telegram bot via @BotFather, get token.

**`openclaw/config/channels.json`**:
```json
{
  "channels": {
    "telegram": {
      "enabled": true,
      "token": "${TELEGRAM_BOT_TOKEN}",
      "allowedUsers": ["${TELEGRAM_USER_ID}"],
      "dmPolicy": "allowlist"
    },
    "whatsapp": {
      "enabled": true,
      "library": "@whiskeysockets/baileys",
      "dmPolicy": "allowlist",
      "allowFrom": ["${WHATSAPP_PHONE_NUMBER}@s.whatsapp.net"],
      "authDir": "/data/openclaw/auth/whatsapp"
    }
  }
}
```

**Telegram Setup Process**:
1. Message @BotFather -> /newbot -> get token
2. Get user chat ID via @userinfobot
3. Set token and user ID in `.env` file
4. Deploy container -> bot responds to messages

### 4. WhatsApp Channel Setup (~2h)

**Setup Process**:
1. Deploy container with WhatsApp enabled
2. Check logs for QR code: `docker logs openclaw | grep "QR"`
3. Scan QR code with phone WhatsApp (linked devices)
4. Auth state persisted to `/data/openclaw/auth/whatsapp/`
5. Subsequent restarts use cached auth (no re-scan needed)

**Fallback**: If Baileys disconnects, Telegram remains as primary channel.

### 5. Docker Compose Integration (~2h)

Update `docker-compose-vps.yml`:

```yaml
services:
  # ... existing services with adjusted memory ...

  mongodb:
    deploy:
      resources:
        limits:
          memory: 384M   # Was 512M

  nextjs-ui-dashboard:
    deploy:
      resources:
        limits:
          memory: 256M   # Was 512M

  # New: OpenClaw
  openclaw:
    build:
      context: ./openclaw
      dockerfile: Dockerfile
    container_name: openclaw
    restart: unless-stopped
    environment:
      - ANTHROPIC_API_KEY=${ANTHROPIC_API_KEY}
      - MCP_AUTH_TOKEN=${MCP_AUTH_TOKEN}
      - TELEGRAM_BOT_TOKEN=${TELEGRAM_BOT_TOKEN}
      - NODE_ENV=production
    volumes:
      - openclaw_data:/data/openclaw
      - ./openclaw/config:/root/.openclaw:ro
    networks:
      - bot-network
    depends_on:
      mcp-server:
        condition: service_healthy
    deploy:
      resources:
        limits:
          memory: 768M
          cpus: "0.5"

  # New: MCP Server (from Phase 01)
  mcp-server:
    # ... (defined in Phase 01)

volumes:
  mongodb_data:
  openclaw_data:
```

### 6. Environment Variables (.env additions) (~30m)

```bash
# OpenClaw + MCP
ANTHROPIC_API_KEY=sk-ant-...           # Claude Max subscription key
MCP_AUTH_TOKEN=<generate-random-64>     # Internal auth token
TELEGRAM_BOT_TOKEN=<from-botfather>     # Telegram bot token
TELEGRAM_USER_ID=<your-telegram-id>     # Allowlist
WHATSAPP_PHONE_NUMBER=<your-phone>      # With country code, no +
BOTCORE_EMAIL=admin@botcore.local       # For MCP->BotCore JWT auth
BOTCORE_PASSWORD=<admin-password>       # For MCP->BotCore JWT auth
```

### 7. System Prompt Tuning (~1h)

Refine the system prompt based on initial testing:
- Add specific examples of common user queries
- Include formatting guidance for Telegram (markdown subset)
- Add context about BotCore's current state (paper trading only, 4 symbols)
- Include emergency procedures (e.g., "stop everything" = stop paper trading engine)

## Todo List

- [ ] Create `openclaw/` directory structure
- [ ] Write OpenClaw Dockerfile (node:18-alpine, headless mode)
- [ ] Write OpenClaw main config (`openclaw.json`)
- [ ] Write MCP connection config pointing to internal `mcp-server:8090`
- [ ] Write Telegram channel config with allowlist
- [ ] Write WhatsApp (Baileys) channel config with allowlist
- [ ] Write system prompt for trading bot context
- [ ] Create Telegram bot via @BotFather, record token
- [ ] Get Telegram user ID for allowlist
- [ ] Update `docker-compose-vps.yml` with openclaw + mcp-server services
- [ ] Adjust memory limits: MongoDB 384MB, Next.js 256MB
- [ ] Add all new env vars to `.env.example`
- [ ] Generate MCP_AUTH_TOKEN (random 64-char hex)
- [ ] Test: OpenClaw starts and connects to Claude API
- [ ] Test: Telegram bot receives and responds to messages
- [ ] Test: MCP tools accessible from OpenClaw (ask "what's the system health?")
- [ ] Test: WhatsApp QR code pairing and message exchange
- [ ] Test: Memory usage stays within budget (`docker stats`)
- [ ] Test: Container restart preserves WhatsApp auth state

## Success Criteria

1. OpenClaw container starts in headless mode on Docker network
2. Telegram bot responds to user messages with trading data
3. WhatsApp responds to user messages (after QR pairing)
4. Claude uses MCP tools to fetch real BotCore data in responses
5. Only allowlisted users can interact (others are ignored)
6. Total VPS memory usage stays under 3.8GB
7. Container restarts preserve conversation history and WhatsApp auth
8. System prompt correctly contextualizes Claude for trading operations

## Risk Assessment

| Risk | Probability | Impact | Mitigation |
|------|-------------|--------|------------|
| VPS runs out of memory | Medium | High | Swap file, aggressive mem limits, monitoring |
| OpenClaw npm package breaks | Low | High | Pin version, test locally before deploying |
| Telegram API rate limits | Low | Low | Max 30 msgs/sec, unlikely to hit |
| WhatsApp QR expires / Baileys disconnect | Medium | Medium | Telegram as fallback, auto-reconnect |
| Claude Max API key rotation | Low | Medium | Document rotation procedure |

## Security Considerations

- Anthropic API key stored as env var, never in config files committed to git
- Telegram/WhatsApp allowlist ensures only owner can interact
- MCP auth token is internal-only (never exposed externally)
- OpenClaw container has no port mapping (internal network only, except for channel APIs which it reaches out to)
- System prompt explicitly forbids enabling real trading
- WhatsApp auth state is sensitive (grants phone access) -- protect volume

## Next Steps

After this phase: proceed to Phase 05 to set up automated cron jobs for scheduled market analysis, portfolio reports, and performance reviews.
