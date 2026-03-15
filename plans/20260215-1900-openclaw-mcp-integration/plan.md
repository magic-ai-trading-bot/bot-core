# OpenClaw + BotCore MCP Integration Plan

**Created**: 2026-02-15 | **Status**: ✅ Complete | **Est. Effort**: 8-12 days

## Goal

Enable Claude (via $200/mo Max subscription) to control, monitor, report, and self-tune BotCore trading bot through Telegram/WhatsApp via OpenClaw + MCP Server.

## Architecture

```
User (Telegram/WhatsApp)
  -> OpenClaw (Node.js container, port 3000/5173)
    -> Claude API (Max subscription, $0 extra)
      -> MCP Server (TypeScript, Streamable HTTP, port 8090)
        -> Rust Core Engine (port 8080) + Python AI Service (port 8000)
```

All services on same VPS, internal Docker network. MCP server NOT exposed externally.

## Phases

| Phase | Name | Status | Link |
|-------|------|--------|------|
| 1 | MCP TypeScript Server Setup | ✅ Done | [phase-01](./phases/phase-01-mcp-server-setup.md) |
| 2 | MCP Tool Implementation | ✅ Done | [phase-02](./phases/phase-02-mcp-tool-implementation.md) |
| 3 | Self-Tuning Engine | ✅ Done | [phase-03](./phases/phase-03-self-tuning-engine.md) |
| 4 | OpenClaw Deployment | ✅ Done | [phase-04](./phases/phase-04-openclaw-deployment.md) |
| 5 | Cron Jobs & Automation | ✅ Done | [phase-05](./phases/phase-05-cron-jobs-automation.md) |
| 6 | Integration Testing | ✅ Done | [phase-06](./phases/phase-06-integration-testing.md) |

## Key Decisions

- **MCP Transport**: Streamable HTTP (stateless, multi-client, proxy-friendly)
- **Auth**: Internal bearer token between OpenClaw->MCP; MCP->BotCore uses JWT
- **Security Tiers**: GREEN (auto) / YELLOW (confirm) / RED (approve+MFA)
- **Self-Tuning**: Bounded parameter adjustment with AEGIS-style supervisor pattern
- **Memory Budget**: ~512MB for MCP server, ~1GB for OpenClaw (total ~1.5GB new)

## Dependencies

- OpenClaw Docker image (node:18-alpine base)
- `@modelcontextprotocol/sdk` v1.0+ (TypeScript)
- `zod` v3.25+ (schema validation)
- Telegram Bot Token (from @BotFather)
- WhatsApp via Baileys (QR code pairing)
- Existing BotCore services healthy on VPS

## Risk Summary

| Risk | Impact | Mitigation |
|------|--------|------------|
| VPS memory pressure (4GB total) | High | Memory limits per container, swap, monitoring |
| OpenClaw MCP plugin immaturity | Medium | Build custom HTTP client as fallback |
| WhatsApp Baileys disconnects | Low | Auto-reconnect, Telegram as primary channel |
| Self-tuning parameter drift | High | Hard guardrails, rollback, audit logging |
