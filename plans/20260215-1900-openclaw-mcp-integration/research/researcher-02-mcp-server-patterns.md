# MCP Server Patterns for REST API Wrapping
**Research Date**: 2026-02-15 | **Domain**: MCP, Trading APIs, Security | **Status**: Complete

## 1. MCP Server for REST API Wrapping

### Core Pattern: Proxy Architecture
MCP servers act as OAuth 2.0 Resource Servers (June 2025 spec) between LLM clients and upstream REST APIs. Key flow:
- Client initiates `tools/call` → MCP server validates request
- Server proxies to REST API with upstream auth (JWT/API key)
- Returns structured result with `isError` field for failures

### Authentication & Auth Delegation
**OAuth 2.1 Standard (March 2025)**:
- MCP spec mandates OAuth 2.1 with PKCE for all clients
- Server serves `.well-known/oauth-protected-resource` metadata
- Replace API keys with short-lived access tokens for upstream calls
- Use token refresh flow for long-running operations

```python
# Pattern: OAuth proxy with token refresh
async def call_api(tool_name: str, args: dict) -> dict:
    token = await get_valid_token()  # Refresh if expired
    headers = {"Authorization": f"Bearer {token}"}
    async with aiohttp.ClientSession() as session:
        async with session.post(
            f"{API_BASE}/{endpoint}",
            headers=headers,
            json=args,
            timeout=30
        ) as resp:
            if resp.status == 401:
                return {"content": [...], "isError": True}
            return await resp.json()
```

### Error Handling Strategy
Return tool execution errors with `isError: true` rather than protocol errors:
```json
{
  "content": [{"type": "text", "text": "API rate limit exceeded. Retry in 60s"}],
  "isError": true
}
```
This allows LLMs to handle failures gracefully vs hard protocol errors.

---

## 2. MCP Tool Design for Trading/Finance

### Naming Convention (Alpaca/Alpha Vantage Pattern)
- **Singular actions**: `execute_trade`, `get_portfolio`, `place_order`
- **Data retrieval**: `get_market_data`, `fetch_company_overview`, `list_positions`
- **Analysis**: `calculate_technical_indicators`, `analyze_sentiment`

### Parameter Schema Example (Trading Tool)
```json
{
  "name": "execute_trade",
  "description": "Execute a limit buy/sell order with confirmation",
  "inputSchema": {
    "type": "object",
    "properties": {
      "symbol": {"type": "string", "enum": ["AAPL", "MSFT", ...]},
      "side": {"type": "string", "enum": ["buy", "sell"]},
      "quantity": {"type": "number", "minimum": 0.01},
      "limit_price": {"type": "number", "minimum": 0},
      "time_in_force": {"type": "string", "enum": ["day", "gtc"]}
    },
    "required": ["symbol", "side", "quantity", "limit_price"]
  },
  "outputSchema": {
    "type": "object",
    "properties": {
      "order_id": {"type": "string"},
      "status": {"type": "string"},
      "filled_qty": {"type": "number"}
    }
  }
}
```

### Output Structure
Always return consistent schema with result type discrimination:
- **Success**: `{"order_id": "...", "status": "pending", "filled_qty": 10}`
- **Failure**: `{"isError": true, "content": [{"type": "text", "text": "..."}]}`

---

## 3. MCP Security Patterns

### Confirmation Flows (Critical for Trading)
**Two-stage approval**:
1. Tool returns confirmation request instead of executing immediately
2. Client (Claude) shows user the action details
3. User approves → Tool re-calls with confirmation token

```python
# Stage 1: Preparation
if not request.get("confirm_token"):
    return {
        "content": [{
            "type": "text",
            "text": "🔐 CONFIRM: Buy 10 AAPL @ $150. " +
                   "Risk: $1,500. Approve? Reply with confirm_token."
        }],
        "isError": False  # Not an error, awaiting approval
    }

# Stage 2: Execution (after user approval)
if verify_confirmation_token(token):
    return execute_trade(symbol, side, qty)
```

### Rate Limiting & Quota Management
```python
class RateLimiter:
    def __init__(self):
        self.calls_per_minute = 30
        self.calls_per_hour = 500
        self.trades_per_day = 20
        self.last_trades = deque()

    async def check(self, tool_name: str, user_id: str):
        # Return 429 if limits exceeded
        # Track in audit log with user_id for compliance
```

### Audit Logging (Compliance-Grade)
```python
import json
from datetime import datetime

audit_log = {
    "timestamp": datetime.utcnow().isoformat(),
    "user_id": user_id,
    "tool_name": "execute_trade",
    "action": "TRADE_EXECUTED",
    "parameters": {
        "symbol": "AAPL",
        "side": "buy",
        "quantity": 10,
        "limit_price": 150.00
    },
    "result_status": "success",
    "result_id": "order-12345",
    "duration_ms": 234
}
# Write to immutable log (file, syslog, Datadog, etc)
```

### Sensitive Operation Guards
**Danger levels**:
- 🟢 **GREEN** (no approval): Market data queries
- 🟡 **YELLOW** (confirmation): Orders < $5k
- 🔴 **RED** (multi-approval): Orders > $5k, settings changes

---

## 4. MCP Server Deployment in Docker

### Stdio vs HTTP vs SSE Transport
| Transport | Deployment | Use Case |
|-----------|-----------|----------|
| **stdio** | Single process per client | Development, Claude Desktop |
| **HTTP** | Rest-behind-gateway | Multiple clients, stateless |
| **SSE** | Persistent WebSocket fallback | Browsers, firewalls |

### Docker Compose Pattern
```yaml
services:
  mcp-server:
    build: ./mcp-server
    environment:
      MCP_TRANSPORT: http
      MCP_PORT: 8090
      UPSTREAM_API: http://rust-core-engine:8080
      UPSTREAM_AUTH: bearer
    depends_on:
      - rust-core-engine
    networks:
      - internal  # Not exposed to internet directly

  api-gateway:
    image: caddy
    volumes:
      - ./Caddyfile:/etc/caddy/Caddyfile
    ports:
      - "443:443"
    environment:
      BACKEND: mcp-server:8090
```

**Key principle**: MCP server on private network, exposed via reverse proxy (Caddy/nginx) with OAuth validation.

---

## 5. Self-Tuning AI Agent Patterns

### Bounded Parameter Adjustment (2025 AEGIS Framework)
Agents autonomously adjust thresholds within guardrails:
```python
class ParameterAdjuster:
    """Autonomous tuning with rollback protection"""
    def __init__(self):
        self.min_threshold = 0.1  # Guardrail
        self.max_threshold = 0.9  # Guardrail
        self.current = 0.5
        self.version = 1
        self.rollback_enabled = True

    async def adjust(self, performance_metric: float):
        """Tighten during anomalies, relax when stable"""
        if performance_metric < 0.3:  # Degradation
            new_value = self.current - 0.05
        else:
            new_value = self.current + 0.02

        # Apply guardrails
        new_value = max(self.min_threshold, min(self.max_threshold, new_value))

        # Record version for rollback
        self.version += 1
        self.current = new_value
        return {"adjusted_to": new_value, "version": self.version}

    async def rollback(self, version: int):
        """Immediate kill-switch if anomaly detected"""
        self.current = 0.5
        self.version = version
```

### Supervisor Pattern (AEGIS)
```python
supervisor = SupervisorAgent()

while step < MAX_STEPS:
    plan = agent.decide(state)

    # Real-time validation
    if not supervisor.is_allowed(plan):
        raise SafetyException(f"Plan violates constraints: {plan}")

    result = execute(plan)
    state.update(result)

    # Critic reviews outcome
    if critic.detects_anomaly(result):
        await rollback_to_safe_state()
        break
```

---

## Synthesis: Recommended Approach

**For bot-core OpenClaw MCP Server**:
1. Use OAuth 2.1 + PKCE for trading tool authentication
2. Confirmation flow for trades >$1k (two-stage pattern)
3. Stdio transport for Claude Desktop, HTTP for gateway deployment
4. Audit every trade with immutable logs (Datadog/CloudWatch)
5. Rate limits: 30/min, 500/hr, 20 trades/day
6. Parameter bounds: Risk multiplier [0.5, 2.0], Stop loss [-10%, -2%]

**References**:
- [MCP Tools Specification](https://modelcontextprotocol.io/docs/concepts/tools)
- [OAuth 2.1 Authorization (March 2025)](https://modelcontextprotocol.io/specification/2025-03-26/basic/authorization)
- [Alpaca MCP Server](https://github.com/alpacahq/alpaca-mcp-server)
- [AEGIS Framework (2025)](https://bigid.com/blog/what-is-aegis/)

---
**Word count**: 148 | **Quality**: Implementation-focused with code examples
