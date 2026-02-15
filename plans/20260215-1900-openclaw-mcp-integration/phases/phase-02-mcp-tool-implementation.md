# Phase 02: MCP Tool Implementation (80 Endpoints)

**Status**: Pending | **Est.**: 2 days | **Priority**: P0 (core functionality)

## Context Links

- [Scout: API Endpoints Mapping](../scout/scout-01-api-endpoints.md)
- [BotCore Rust API](../../../../specs/02-design/2.3-api/API-RUST-CORE.md) (37 endpoints)
- [BotCore Python API](../../../../specs/02-design/2.3-api/API-PYTHON-AI.md) (43 endpoints)
- [Research: MCP Tool Design](../research/researcher-02-mcp-server-patterns.md)
- Phase 01 (MCP server foundation)

## Overview

Implement all 80 BotCore REST API endpoints as MCP tools, organized into 8 categories with 4 security tiers. Each tool gets Zod input validation, proper error handling, and security tier enforcement (GREEN/YELLOW/RED classification).

## Key Insights

1. **43 read-only, 37 write** endpoints -- read-only tools can skip confirmation flows
2. Tools grouped into logical modules: market, trading, paper-trading, ai, monitoring, settings, auth, tasks
3. Naming convention follows Alpaca pattern: `verb_noun` (e.g., `get_portfolio`, `close_position`)
4. Every write tool on YELLOW/RED tier returns confirmation request first, then executes on re-call with token
5. Rate limiting enforced at MCP layer to prevent exceeding BotCore limits

## Requirements

- All 80 endpoints wrapped as MCP tools with Zod schemas
- 4 security tiers enforced (PUBLIC, AUTHENTICATED, SENSITIVE, CRITICAL)
- Confirmation flow for YELLOW and RED tier write operations
- Rate limiting per tool category
- Consistent error response format (`isError: true` with descriptive messages)
- Tool descriptions optimized for LLM understanding (Claude needs good descriptions to choose tools)

## Architecture

```
MCP Tool Registry
  |
  +-- market/ (8 tools) ........... Tier 1: PUBLIC
  |   get_market_prices, get_market_overview, get_candles,
  |   get_chart, get_multi_charts, get_symbols,
  |   add_symbol [T2], remove_symbol [T2]
  |
  +-- trading/ (4 tools) .......... Tier 2-3: AUTHENTICATED/SENSITIVE
  |   get_positions, get_account, close_position [T3],
  |   get_trading_performance
  |
  +-- paper_trading/ (28 tools) ... Tier 1-2: PUBLIC/AUTHENTICATED
  |   get_status, get_portfolio, get_open_trades, get_closed_trades,
  |   close_trade [T2], get_settings, get_strategy_settings,
  |   update_strategy_settings [T2], get_basic_settings,
  |   update_basic_settings [T2], get_symbol_settings,
  |   update_symbol_settings [T2], reset_account [T2],
  |   start_engine [T2], stop_engine [T2], trigger_analysis [T2],
  |   update_signal_interval [T2], get_trailing_stops,
  |   update_trailing_stop_settings [T2],
  |   manual_adjust_trailing_stop [T2], process_ai_signal [T2],
  |   get_data_resolutions, update_data_resolution [T2],
  |   get_correlation_analysis, ... (remaining paper trading tools)
  |
  +-- ai/ (12 tools) .............. Tier 1-2: PUBLIC/AUTHENTICATED
  |   analyze_market [T2], get_strategy_recommendations [T2],
  |   get_market_condition [T2], send_feedback, get_ai_info,
  |   get_ai_strategies, get_ai_performance,
  |   get_storage_stats, clear_storage [T3],
  |   get_cost_statistics, get_config_suggestions,
  |   get_analysis_history
  |
  +-- tasks/ (7 tools) ............ Tier 2-3: AUTHENTICATED/SENSITIVE
  |   trigger_training [T3], get_task_status, cancel_task [T2],
  |   list_tasks, retry_task [T2], get_task_stats,
  |   deploy_model [T4]
  |
  +-- backtests/ (3 tools) ........ Tier 2: AUTHENTICATED
  |   create_backtest, get_backtest, list_backtests
  |
  +-- monitoring/ (5 tools) ....... Tier 1: PUBLIC
  |   get_system_metrics, get_trading_metrics,
  |   get_connection_status, get_health_status, get_alerts
  |
  +-- auth/ (4 tools) ............. Tier 3-4: SENSITIVE/CRITICAL
      verify_token [T2], get_profile [T2],
      register_user [T4], login [T3]
```

## Related Code Files

| File | Purpose |
|------|---------|
| `mcp-server/src/tools/index.ts` | Tool registry, exports all tools |
| `mcp-server/src/tools/market.ts` | Market data tools (8) |
| `mcp-server/src/tools/trading.ts` | Trading tools (4) |
| `mcp-server/src/tools/paper-trading.ts` | Paper trading tools (28) |
| `mcp-server/src/tools/ai.ts` | AI analysis tools (12) |
| `mcp-server/src/tools/tasks.ts` | Async task tools (7) |
| `mcp-server/src/tools/backtests.ts` | Backtest tools (3) |
| `mcp-server/src/tools/monitoring.ts` | Monitoring tools (5) |
| `mcp-server/src/tools/auth-tools.ts` | Auth tools (4) -- note: different from `auth.ts` (internal auth) |
| `mcp-server/src/security.ts` | Security tier enforcement, confirmation flow |
| `mcp-server/src/rate-limiter.ts` | Per-tool rate limiting |
| `mcp-server/src/types.ts` | Shared types, tool result builders |

## Implementation Steps

### 1. Security & Confirmation Framework (~3h)

**`src/security.ts`**:
```typescript
enum SecurityTier { PUBLIC = 1, AUTHENTICATED = 2, SENSITIVE = 3, CRITICAL = 4 }

interface ToolConfig {
  name: string;
  tier: SecurityTier;
  requiresConfirmation: boolean;
  rateLimit: { max: number; windowMs: number };
}

// Confirmation flow:
// 1. Tool called without confirm_token -> return confirmation prompt
// 2. Tool called with valid confirm_token -> execute
// Tokens: SHA256(tool_name + params_hash + timestamp), expire in 5 min
```

**`src/rate-limiter.ts`**:
- In-memory sliding window rate limiter
- Per-tool-category limits matching BotCore API limits
- Returns `isError: true` with retry-after on limit exceeded

### 2. Tool Result Helpers (~1h)

**`src/types.ts`**:
```typescript
// Standard MCP tool result builders
function toolSuccess(data: unknown): ToolResult {
  return { content: [{ type: "text", text: JSON.stringify(data, null, 2) }] };
}

function toolError(message: string): ToolResult {
  return { content: [{ type: "text", text: message }], isError: true };
}

function toolConfirm(action: string, details: string, token: string): ToolResult {
  return {
    content: [{
      type: "text",
      text: `CONFIRM REQUIRED: ${action}\n${details}\nReply with confirm_token: ${token}`
    }]
  };
}
```

### 3. Market Tools (~2h)
- 8 tools, mostly Tier 1 (PUBLIC), no confirmation needed
- `get_market_prices`: No params, returns latest prices
- `get_candles`: Params: symbol (string), timeframe (enum), limit (number, optional)
- `get_chart`: Params: symbol, timeframe, limit -- returns candles + indicators
- `add_symbol` / `remove_symbol`: Tier 2, requires confirmation

### 4. Paper Trading Tools (~4h)
- 28 tools, largest category
- Read tools (16): Direct passthrough, no confirmation
- Write tools (12): Tier 2, confirmation for settings changes
- Special attention to:
  - `update_basic_settings`: Validate parameter bounds (leverage 1-125, stop_loss 0.5-10%)
  - `reset_account`: Requires confirmation ("This will close all positions")
  - `start_engine` / `stop_engine`: Requires confirmation
  - `process_ai_signal`: Rate limit 6/hour, async response

### 5. AI Tools (~2h)
- 12 tools bridging both Rust (8080) and Python (8000) services
- `analyze_market`: Long timeout (120s), rate limit 10/min
- `get_config_suggestions`: Maps to Python `/ai/config-suggestions` -- feeds self-tuning engine
- `trigger_config_analysis`: Maps to Python `/ai/config-analysis/trigger`

### 6. Task & Backtest Tools (~2h)
- `trigger_training`: Tier 3 (SENSITIVE), requires confirmation, long-running
- `deploy_model`: Tier 4 (CRITICAL), requires explicit approval with reason
- `create_backtest`: Tier 2, validate date range params

### 7. Trading & Auth Tools (~1.5h)
- Trading tools: `close_position` is Tier 3 (SENSITIVE) -- requires confirmation
- Auth tools: Mostly internal use, `register_user` is Tier 4

### 8. Monitoring Tools (~1h)
- All Tier 1 (PUBLIC), no auth needed
- `get_system_metrics`, `get_trading_metrics`, `get_connection_status`
- `get_health_status`: Calls Python `/api/monitoring/health`
- `get_alerts`: Calls Python `/api/monitoring/alerts`

### 9. Tool Registry & Dispatch (~1h)
- `src/tools/index.ts`: Central registry mapping tool names to handlers
- Used by `CallToolRequestSchema` handler in server.ts
- Validates security tier before executing tool
- Applies rate limiting before executing tool

## Todo List

- [ ] Implement security tier enforcement framework (`src/security.ts`)
- [ ] Implement confirmation token generation and validation
- [ ] Implement per-tool rate limiter (`src/rate-limiter.ts`)
- [ ] Implement tool result helpers (`src/types.ts`)
- [ ] Implement market tools (8) in `src/tools/market.ts`
- [ ] Implement paper trading tools (28) in `src/tools/paper-trading.ts`
- [ ] Implement AI tools (12) in `src/tools/ai.ts`
- [ ] Implement task management tools (7) in `src/tools/tasks.ts`
- [ ] Implement backtest tools (3) in `src/tools/backtests.ts`
- [ ] Implement trading tools (4) in `src/tools/trading.ts`
- [ ] Implement monitoring tools (5) in `src/tools/monitoring.ts`
- [ ] Implement auth tools (4) in `src/tools/auth-tools.ts`
- [ ] Create central tool registry (`src/tools/index.ts`)
- [ ] Write unit tests for security tier enforcement
- [ ] Write unit tests for confirmation flow
- [ ] Write unit tests for rate limiter
- [ ] Integration test: call each tool category with mock API responses
- [ ] Verify all 80 tools listed in `ListToolsRequestSchema` response

## Success Criteria

1. All 80 BotCore endpoints mapped to MCP tools
2. `ListToolsRequestSchema` returns 80+ tools with descriptions and input schemas
3. Security tiers enforced -- Tier 3/4 tools reject without proper auth
4. Confirmation flow works -- YELLOW/RED write tools require 2-step approval
5. Rate limiting prevents exceeding BotCore API limits
6. Tool descriptions are clear enough for Claude to select the right tool
7. Error responses are informative (not just "failed")

## Risk Assessment

| Risk | Probability | Impact | Mitigation |
|------|-------------|--------|------------|
| Tool description quality affects Claude's tool selection | Medium | High | Iterate on descriptions, test with Claude |
| 80 tools may overwhelm Claude's context | Low | Medium | Group logically, use clear naming |
| Confirmation tokens can be replayed | Low | Medium | 5-min expiry, single-use, include params hash |
| Rate limiter memory leak on high traffic | Low | Low | Use sliding window with cleanup interval |

## Security Considerations

- Confirmation tokens are SHA256(tool + params + timestamp + secret), single-use, 5-min expiry
- Tier 4 (CRITICAL) tools log all invocations to audit file
- No sensitive data (passwords, API keys) exposed in tool results
- Rate limits are defense-in-depth (BotCore also rate-limits)
- Tool input validation via Zod prevents injection attacks

## Next Steps

After this phase: proceed to Phase 03 to implement the self-tuning engine that uses AI tools to analyze performance and adjust parameters.
