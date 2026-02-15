# OpenClaw MCP Integration Research Report
**Date**: 2026-02-15 | **Research Focus**: MCP support, Docker deployment, channel configs, automation

---

## 1. OpenClaw MCP Support & Configuration

**Status**: Limited native support, requires plugins/workarounds

**Connection Methods**:
- **HTTP Transport** (preferred): Remote MCP servers via HTTP with Streamable HTTP
- **stdio Transport**: Local MCP servers (less common in OpenClaw context)
- **MCP Plugin**: `openclaw-mcp-plugin` enables connecting to any MCP server

**Config Pattern** (JSON):
```json
{
  "mcp": {
    "servers": [
      {
        "name": "filesystem",
        "command": "npx",
        "args": ["@modelcontextprotocol/server-filesystem", "/home/user/docs"],
        "transport": "stdio"
      },
      {
        "name": "remote-api",
        "type": "http",
        "url": "https://mcp-server.example.com",
        "auth": {
          "type": "oauth",
          "clientId": "MCP_CLIENT_ID",
          "issuerUrl": "MCP_ISSUER_URL"
        }
      }
    ]
  }
}
```

**Key Limitations**:
- No native MCP client in core (requires plugin/community solution)
- Feature requests exist (#4834, #8188) but not yet merged
- PR #5121 attempted MCP server support for clawdbot

---

## 2. MCP TypeScript SDK Best Practices (2025/2026)

**SDK**: `@modelcontextprotocol/sdk` (official, v1.0+)

**Core Components**:
| Component | Purpose | Example |
|-----------|---------|---------|
| **Tools** | Actions LLM can invoke | `getWeather(location)`, `writefile(path, content)` |
| **Resources** | Read-only data surfaces | File contents, database snapshots, docs |
| **Prompts** | Reusable templates | System prompts, few-shot examples |

**Transport Tiers**:
1. **Streamable HTTP** (recommended 2025+): Multi-node, stateless, proxy-friendly
2. **stdio**: Local servers only (child process)
3. **SSE**: Deprecated in favor of Streamable HTTP

**Schema Validation**: Requires `zod` peer dependency (v3.25+, backwards compatible)

**Deployment Pattern**:
```bash
# Install
npm install @modelcontextprotocol/sdk zod

# Server setup
const server = new Server({
  name: "my-server",
  version: "1.0.0"
});

server.setRequestHandler(ListToolsRequestSchema, async () => ({
  tools: [
    {
      name: "analyze_portfolio",
      description: "Analyze trading portfolio",
      inputSchema: { /* zod schema */ }
    }
  ]
}));

server.setRequestHandler(CallToolRequestSchema, async (request) => {
  // Implementation
});

// Run on Streamable HTTP
const transport = new StdioServerTransport();
await server.connect(transport);
```

---

## 3. OpenClaw Docker Deployment

**Memory Requirements**:
- **Default sandbox**: 1GB memory + 2GB swap per container
- **Production recommended**: 2GB container limit, 1GB reserved
- **Resource-constrained**: 512MB possible (performance impact)
- **Model inference**: <8GB RAM = quantized models (75% memory reduction, 4-bit)

**Dockerfile Pattern** (minimal):
```dockerfile
FROM node:18-alpine
RUN npm install -g openclaw
ENV NODE_OPTIONS="--max-old-space-size=512"
EXPOSE 3000 5173
CMD ["openclaw", "start"]
```

**Environment Config**:
```bash
# docker run or docker-compose.env
NODE_OPTIONS=--max-old-space-size=512
OPENCLAW_DATA=/data  # Persistent volume
LOG_LEVEL=info
```

**Compose Example**:
```yaml
services:
  openclaw:
    image: node:18-alpine
    environment:
      NODE_OPTIONS: --max-old-space-size=1024
    volumes:
      - openclaw_data:/root/.openclaw
    ports:
      - "3000:3000"
      - "5173:5173"
    mem_limit: 2g
```

**Headless Production**: Can run without UI (daemon mode)

---

## 4. Telegram + WhatsApp (Baileys) Channel Setup

**Telegram** (token-based):
```bash
# ~/.openclaw/openclaw.json
{
  "channels": {
    "telegram": {
      "enabled": true,
      "token": "${TELEGRAM_BOT_TOKEN}"
    }
  }
}
```
Setup: `openclaw channels add telegram` → paste bot token

**WhatsApp** (Baileys library):
```json
{
  "channels": {
    "whatsapp": {
      "enabled": true,
      "dmPolicy": "allowlist",
      "allowFrom": ["1234567890@s.whatsapp.net"],
      "library": "@whiskeysockets/baileys"
    }
  }
}
```

**Baileys Library**:
- Uses WhatsApp Web protocol (not official API)
- WebSocket connection to WhatsApp servers
- Multi-file auth state persistence (`~/.openclaw/auth/whatsapp/`)
- **Limitation**: Offline phone = no messages

**Setup Process**: `openclaw channels add whatsapp` → QR code scan → auth persisted

**Policy Options**:
- `"open"`: Anyone can message
- `"pairing"`: Initial pairing required
- `"allowlist"`: Only numbers in `allowFrom` array

---

## 5. Cron Jobs & Automation

**Storage**: `~/.openclaw/cron/*.json` (persists across restarts)

**Job Structure**:
```json
{
  "id": "daily-market-analysis",
  "name": "Daily Market Analysis",
  "schedule": "0 9 * * 1-5",
  "prompt": "Analyze BTC/ETH 24h movement and provide trading signals",
  "mode": "independent"
}
```

**Schedule Patterns**:
| Pattern | Meaning |
|---------|---------|
| `0 9 * * 1` | 9 AM every Monday (cron format) |
| `*/30 * * * *` | Every 30 minutes |
| `0 0 * * *` | Daily at midnight |
| `0 */6 * * *` | Every 6 hours |

**Execution Modes**:
- `independent`: Separate context, no session interference
- `shared`: Uses main session context

**Error Handling**:
- Exponential backoff: 30s → 1m → 5m → 15m → 60m
- Auto-reset backoff on next success

**API**: `openclaw cron add|list|remove|run` CLI or REST API

**Use Cases**:
- Market analysis (daily)
- Portfolio rebalancing (weekly)
- ML model retraining (scheduled)
- Health checks (hourly)

---

## Architecture Summary

```
┌─────────────────────────────────────┐
│     OpenClaw (Node.js + AI)         │
│  ┌─────────────────────────────────┐│
│  │  MCP Plugin (optional)          ││
│  │  └─ Connect to MCP servers ────→││
│  └─────────────────────────────────┘│
│  ┌─────────────────────────────────┐│
│  │  Channels Layer                 ││
│  │  ├─ Telegram (token auth)       ││
│  │  ├─ WhatsApp (Baileys/QR)       ││
│  │  ├─ Discord, Slack, iMessage    ││
│  │  └─ HTTP webhooks               ││
│  └─────────────────────────────────┘│
│  ┌─────────────────────────────────┐│
│  │  Automation Layer               ││
│  │  ├─ Cron (scheduled tasks)      ││
│  │  ├─ Webhooks (event-driven)     ││
│  │  └─ Multi-step workflows        ││
│  └─────────────────────────────────┘│
└─────────────────────────────────────┘
         │             │
         ↓             ↓
    Docker       Persistent
    Container    Storage
   (1-2GB mem)  (~/.openclaw)
```

---

## Key Findings & Actionable Next Steps

✅ **MCP Integration**: Use `openclaw-mcp-plugin` or build HTTP client via TypeScript SDK (Streamable HTTP)

✅ **Docker Ready**: 2GB memory + persistent volume is production baseline

✅ **Multi-Channel**: Telegram (instant, token) → WhatsApp (more setup, Baileys) + others

✅ **Automation**: Cron jobs handle scheduled analysis; webhooks handle event-driven workflows

⚠️ **Limitations**:
- No native MCP in OpenClaw core (requires plugin or custom integration)
- WhatsApp requires phone online (Baileys limitation)
- Quantized models needed for <8GB RAM deployments

---

## Sources

- [OpenClaw MCP Issue #4834](https://github.com/openclaw/openclaw/issues/4834)
- [MCP TypeScript SDK Official](https://github.com/modelcontextprotocol/typescript-sdk)
- [OpenClaw Docker Docs](https://docs.openclaw.ai/install/docker)
- [OpenClaw WhatsApp Integration](https://docs.openclaw.ai/channels/whatsapp)
- [OpenClaw Cron Jobs](https://docs.openclaw.ai/automation/cron-jobs)
- [Cyanheads MCP Resources](https://github.com/cyanheads/model-context-protocol-resources)
