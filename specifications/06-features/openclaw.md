# OpenClaw Gateway (AI via Telegram)

## Quick Reference

### Code Locations
```
openclaw/
├── config/
│   ├── openclaw.json               Dev config (WebSocket gateway, model settings)
│   └── openclaw.production.json    Production with Telegram channels
├── scripts/
│   ├── entrypoint.sh               Docker entrypoint: waits for MCP, syncs config, registers cron
│   └── botcore-bridge.mjs          MCP client CLI bridge (Node.js script)
├── workspace/
│   └── SKILL.md                    Injected into AI system prompt
└── Dockerfile                      Node 22 base image
```

### Configuration
- **Gateway Port**: 18789 (WebSocket)
- **AI Model**: `xai/grok-4-1-fast` (configurable in `agents.defaults.model.primary`)
- **Auth**: xAI API key via `XAI_API_KEY` environment variable
- **Timezone**: TZ=Asia/Ho_Chi_Minh

## Architecture

### Bridge Pattern (NOT native MCP client)
OpenClaw does NOT support native MCP client (issue #4834 closed "not planned" Feb 2026). Instead uses a bridge approach:

```
Telegram -> OpenClaw AI -> exec tool -> botcore-bridge.mjs -> MCP Server (:8090)
                                            |
                                       Tool result -> AI response -> Telegram
```

### Bridge Script (botcore-bridge.mjs)
- Acts as MCP client CLI
- 30s timeout, 2 retries with exponential backoff
- Auto-wraps plain text for notifications
- Field normalization (text/content/body -> message)
- Usage: `botcore <tool_name> '{"param": "value"}'`

## Telegram Integration

### Channel Configuration
```json
{
  "channels": {
    "telegram": {
      "bot_token": "${TELEGRAM_BOT_TOKEN}",
      "user_id": "${TELEGRAM_USER_ID}"
    }
  }
}
```

### Security
- User ID filtering — only responds to authorized `TELEGRAM_USER_ID`
- Gateway token authentication for WebSocket connections (`OPENCLAW_GATEWAY_TOKEN`)
- LAN binding for gateway access

## Cron Jobs

Registered via entrypoint.sh using `openclaw --dev cron add`:
- **Health checks**: Periodic system health monitoring (`no_deliver: true`)
- **Risk monitoring**: Real-time risk assessment (`no_deliver: true`)
- **Reports**: Trading performance reports (auto-deliver to Telegram)

### Cron Gotchas
- NO `--file` flag exists — must use inline `--message`
- Must use `--dev` flag + explicit `--url`/`--token`
- Gateway needs 90+ seconds to start before cron registration
- `ln -sfn $OPENCLAW_HOME /home/node/.openclaw-dev` in entrypoint.sh — symlinks dev profile so cron sub-agents share pairing data

## Docker Deployment

### Environment Variables
```
TELEGRAM_BOT_TOKEN      Telegram bot API token
TELEGRAM_USER_ID        Authorized Telegram user ID
OPENCLAW_GATEWAY_TOKEN  WebSocket gateway auth token
XAI_API_KEY             xAI API key for Grok model
```

### Dependencies
- MCP Server (:8090) — must be running before OpenClaw starts
- Node >= 22 (NOT 18)
- Docker container in docker-compose-vps.yml

## Related Specs
- `FR-OPENCLAW.md` — Functional requirements
- `FR-MCP.md` — MCP Server requirements (dependency)

**Last Updated**: 2026-03-03
