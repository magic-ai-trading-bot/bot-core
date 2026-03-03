# Feature Documentation Accuracy Review

**Reviewer**: code-reviewer
**Date**: 2026-03-03
**Scope**: 7 feature docs in `specifications/06-features/` vs actual source code
**Verdict**: Multiple files have significant inaccuracies, primarily stale line numbers and outdated AI provider references

---

## 1. paper-trading.md

**Status**: Needs Update

### Line Number Drift (CRITICAL - all wrong)

The engine.rs file is now 12,793 lines. Every line reference in the doc is stale:

| Doc Reference | Doc Says | Actual Line | Drift |
|---|---|---|---|
| `apply_slippage()` | 738-845 | 1677 | +939 |
| `calculate_market_impact()` | 738-845 | 1714 | +976 |
| `simulate_partial_fill()` | 738-845 | 1758 | +1020 |
| `check_daily_loss_limit()` | 847 | 1977 | +1130 |
| `is_in_cooldown()` | 892 | 2022 | +1130 |
| `check_position_correlation()` | 982 | 2089 | +1107 |
| `process_trading_signal()` | 509-560 | 1242 | +733 |
| `execute_trade()` | 1041-1197 | 2624 | +1583 |
| `close_trade()` | 1425-1452 | 3344 | +1919 |

What the doc says is at lines 738-845 (execution simulation) is actually `build_strategy_input()` (line 742), `start_trade_monitoring()` (line 796), `start_performance_tracking()` (line 818), and `start_optimization_loop()` (line 837) -- none of which are execution simulation methods.

### portfolio.rs Lines

| Doc Reference | Doc Says | Actual |
|---|---|---|
| Cool-down fields | Lines 77-81 | Lines 78-82 (close, off by 1) |
| Initialization | Lines 223-224 | Lines 229-231 (consecutive_losses, cool_down_until init) |

These are close but slightly shifted due to added fields (e.g., `week_start_equity` at line 87).

### trade.rs Lines

| Doc Reference | Doc Says | Actual |
|---|---|---|
| Latency fields | Lines 145-152 | Lines 146-153 (off by 1) |
| After latency fields | - | New field: `highest_price_achieved` at line 158 (trailing stop, not documented) |
| Initialization | Lines 223-225 | ~Lines 218+ (shifted) |

### Features Missing from Doc

- **Signal Confirmation**: Requires 2 consecutive same-direction signals within 10 min (line 532-559)
- **Choppy Market Detection**: 4+ direction flips in 15 min blocks trades (line 510-530)
- **AI Bias Check**: Validates signal alignment with AI market bias (line 562+)
- **ATR-based Stop Loss**: FR-RISK-010 (line 1022+)
- **Half-Kelly Position Sizing**: FR-RISK-011 (line 1053+)
- **Regime Filters**: Funding spike, ATR spike, consecutive loss reduction (line 1132+)
- **Weekly Drawdown Limit**: FR-RISK-012 (line 1190+)
- **Trailing Stop Loss**: FR-RISK-007/008 (portfolio update section, line 960+)
- **Pending Stop-Limit Orders**: FR-PAPER-003 (line 1008+)

### Fixes Needed
1. All line numbers need complete update
2. Add documentation for 9+ new features/risk management methods
3. Add `highest_price_achieved` field to trade.rs documentation
4. Update "Phase" numbering -- doc jumps from Phase 2 to Phase 4

---

## 2. ai-integration.md

**Status**: Wrong (Major rewrite needed)

### AI Provider Changed (CRITICAL)

The entire AI service has been rewritten from OpenAI/GPT-4 to **xAI Grok**:

| Doc Claims | Actual Code |
|---|---|
| `DirectOpenAIClient` class | Does NOT exist. Actual: `GrokClient` (line 1358) |
| `GPTTradingAnalyzer` class | Does NOT exist. Actual: `GrokTradingAnalyzer` (line 1522) |
| Model: `gpt-4o-mini` | Model: `grok-4-1-fast-non-reasoning` (env `AI_MODEL`) |
| `OPENAI_API_KEY` primary | `XAI_API_KEY` primary, `OPENAI_API_KEY` fallback |
| Pricing: $0.15/1M input | xAI Grok pricing (different) |

### Endpoint Changes

| Doc Claims | Actual |
|---|---|
| `POST /api/ai/analyze` | `POST /ai/analyze` (no `/api` prefix) |
| `GET /api/ai/signals/{symbol}` | Does NOT exist as endpoint |

### Additional Endpoints Not Documented

The service has many more endpoints than documented:
- `POST /ai/strategy-recommendations`
- `POST /ai/market-condition`
- `POST /ai/feedback`
- `POST /predict-trend`
- `GET /ai/info`, `/ai/strategies`, `/ai/performance`
- `GET /ai/cost/statistics`, `/ai/storage/stats`
- `POST /ai/config-analysis/trigger`, `GET /ai/config-suggestions`
- `GET /ai/gpt4-analysis-history`
- `POST /ai/analyze-trade`
- `POST /api/chat/project`, `GET /api/chat/project/suggestions`

### What IS Still Accurate

- `TechnicalAnalyzer` class exists (line 1008)
- ML models (LSTM, GRU, Transformer, ModelManager) still exist but unused -- correct
- `GET /health` endpoint exists (line 2729)
- `features/technical_indicators.py` referenced correctly

### Fixes Needed
1. Replace all GPT-4/OpenAI references with Grok/xAI
2. Update class names: `DirectOpenAIClient` -> `GrokClient`, `GPTTradingAnalyzer` -> `GrokTradingAnalyzer`
3. Fix endpoint path: `/api/ai/analyze` -> `/ai/analyze`
4. Remove non-existent `/api/ai/signals/{symbol}` endpoint
5. Update environment variables section
6. Add documentation for 15+ undocumented endpoints
7. Update model name and pricing

---

## 3. trading-strategies.md

**Status**: Needs Update

### Strategy Files - Partially Accurate

Doc lists 6 files. Actual directory has 11 files:

| Documented | Exists? |
|---|---|
| `rsi_strategy.rs` | Yes |
| `macd_strategy.rs` | Yes |
| `bollinger_strategy.rs` | Yes |
| `volume_strategy.rs` | Yes |
| `strategy_engine.rs` | Yes |
| `indicators.rs` | Yes |

| NOT Documented | Exists |
|---|---|
| `stochastic_strategy.rs` | Yes (new strategy) |
| `trend_filter.rs` | Yes |
| `hybrid_filter.rs` | Yes |
| `ml_trend_predictor.rs` | Yes |
| `types.rs` | Yes |
| `tests.rs` | Yes |
| `mod.rs` | Yes |

### Indicator Functions - Accurate

All four documented indicator functions exist at expected locations in `indicators.rs`:
- `calculate_rsi()` (line 4)
- `calculate_macd()` (line 74)
- `calculate_bollinger_bands()` (line 123)
- `calculate_ema()` (line 236)

Additionally undocumented: `calculate_atr()` is also available (used by paper trading risk management).

### Fixes Needed
1. Add `stochastic_strategy.rs` as 5th strategy
2. Document `trend_filter.rs`, `hybrid_filter.rs`, `ml_trend_predictor.rs`
3. Add `calculate_atr()` to indicators list
4. Update "Total Strategies: 4 active" to at least 5
5. Add `types.rs` and `tests.rs` to file tree

---

## 4. authentication.md

**Status**: Needs Update

### JWT Algorithm (CRITICAL)

| Doc Claims | Actual Code |
|---|---|
| RS256 (asymmetric) | **HS256** (symmetric) - `jwt.rs` line 69 |
| `secret_key = "path/to/private_key.pem"` | Uses `secret: String` (HMAC secret, not file path) |
| `public_key = "path/to/public_key.pem"` | Not used (HS256 uses shared secret) |
| Access token: 15 minutes | **7 days** (`24 * 7` hours, `handlers.rs` line 32) |

### Function Names - Partially Wrong

| Doc Claims | Actual |
|---|---|
| `validate_token()` in jwt.rs | Does NOT exist by that name. Actual: `validate_claims()` via `decode()` |
| `refresh_token()` in jwt.rs | Does NOT exist. No refresh token endpoint |
| `find_user_by_email()` in database.rs | Actual name: `find_by_email()` (line 62) |
| `jwt_auth_middleware()` in middleware.rs | Actual: `with_auth()` (line 19) |
| `extract_user_from_token()` in middleware.rs | Actual: `authorize()` (line 43) + `extract_user_id()` in security_handlers.rs |

### Routes - Partially Wrong

| Doc Claims | Actual |
|---|---|
| `POST /api/auth/login` | `POST /auth/login` (via warp path) |
| `POST /api/auth/logout` | Does NOT exist |
| `POST /api/auth/register` | `POST /auth/register` |
| `POST /api/auth/refresh` | Does NOT exist |
| `GET /api/auth/me` | `GET /auth/profile` (different name) + `GET /auth/verify` |

### Password Hashing

| Doc Claims | Actual |
|---|---|
| bcrypt with cost 12 | bcrypt with `DEFAULT_COST` (which is 12 in the bcrypt crate) |

This is accurate, but the doc implies explicit config while code uses the library default.

### Missing from Doc

- `models.rs` - User, LoginRequest, LoginResponse, RegisterRequest models
- `security_handlers.rs` - Additional security endpoints (change password, etc.)
- `with_optional_auth()` and `with_admin_auth()` middleware variants

### Fixes Needed
1. Change RS256 to HS256
2. Remove RSA key pair generation instructions
3. Fix token expiry: 15 min -> 7 days
4. Fix function names to match actual code
5. Remove logout and refresh endpoints (don't exist)
6. Add verify and profile endpoints
7. Add `models.rs` and `security_handlers.rs` to file tree

---

## 5. websocket-realtime.md

**Status**: Needs Update

### WebSocket Directory Structure - Wrong

| Doc Claims | Actual |
|---|---|
| `websocket/server.rs` | Directory `rust-core-engine/src/websocket/` does NOT exist |
| `websocket/broadcaster.rs` | Does NOT exist |
| `websocket/handlers.rs` | Does NOT exist |
| `market_data/processor.rs` | Exists (correct) |

The WebSocket functionality appears to be embedded in `binance/websocket.rs` and the paper trading event system, not in a separate `websocket/` directory.

### Binance WebSocket Functions

| Doc Claims | Actual |
|---|---|
| `connect_to_binance()` | Actual: `connect_and_run()` (line 123) |
| `subscribe_to_streams()` | Actual: `subscribe_symbol()` (line 70) |
| `handle_message()` | Exists (line 292) - correct |

### Frontend Files - Accurate

- `useWebSocket.ts` exists in `nextjs-ui-dashboard/src/hooks/`
- `WebSocketContext.tsx` exists in `nextjs-ui-dashboard/src/contexts/`

### Fixes Needed
1. Remove non-existent `websocket/` directory from code locations
2. Fix function names: `connect_to_binance` -> `connect_and_run`, `subscribe_to_streams` -> `subscribe_symbol`
3. Add note about event broadcasting being part of paper trading engine (not separate module)
4. Clarify actual WebSocket architecture

---

## 6. mcp-server.md

**Status**: Needs Update

### SDK Version - Wrong

| Doc Claims | Actual |
|---|---|
| `@modelcontextprotocol/sdk` v1.26.0 | `^1.12.1` in package.json (line 16) |

### Tool Counts - Multiple Mismatches

Actual `server.registerTool()` call counts per file (114 total):

| Category | Doc Says | Actual Count | Delta |
|---|---|---|---|
| Paper Trading | 28 | 39 | +11 |
| AI/ML | 4 | 12 | +8 |
| Real Trading | 8 | 14 | +6 |
| Settings | varies | 10 | (was unspecified) |
| Tasks | 4 | 7 | +3 |
| Market Data | 4 | 8 | +4 |
| Monitoring | 3 | 4 | +1 |
| Trading/Strategies | 3 | 4 | +1 |
| Self-Tuning | 11 | 8 | -3 |
| Auth | 4 | 4 | Correct |
| Health | 3 | 3 | Correct |
| Notification | varies | 1 | (was unspecified) |

Doc header says "103+ tools" but actual is 114 tools across 12 files.

### Test Count

| Doc Claims | Actual |
|---|---|
| 89 tests | 85 `test()`/`it()` calls across 4 test files |

Note: The count difference is minor and could vary by how nested describes are counted.

### Other Accurate Items
- Port 8090: Correct
- Protocol MCP v2024-11-05: Likely correct
- Transport: Streamable HTTP with Express: Correct
- Per-session server model: Correct
- Implementation details about `transport.sessionId`, `req.body`, Zod schema: Correct

### Tool File - Missing from Doc
- `notification.ts` and `settings.ts` exist but are listed as "varies" in doc

### Fixes Needed
1. Update SDK version: v1.26.0 -> ^1.12.1
2. Update all tool counts to match actual (114 total)
3. Update test count
4. Specify actual counts for Settings (10) and Notification (1)

---

## 7. openclaw.md

**Status**: Mostly Accurate

### Verified Correct
- Gateway port 18789: Confirmed in `openclaw.production.json` line 34
- AI Model `xai/grok-4-1-fast`: Confirmed line 11
- WebSocket gateway with token auth: Confirmed
- Bridge script `botcore-bridge.mjs`: Confirmed with correct features (30s timeout, 2 retries)
- Telegram channel config structure: Confirmed
- Node >= 22 requirement: Confirmed in Dockerfile reference
- Entrypoint script behavior: Confirmed (waits for MCP, syncs config, registers cron)
- Cron gotchas: Confirmed (no --file flag, --dev required)

### Minor Issues

| Doc Claims | Actual |
|---|---|
| Auth: `CLAUDE_AI_SESSION_KEY` | NOT found anywhere in codebase. OpenClaw uses xAI/Grok, not Claude.ai session keys |
| `ln -sfn ~/.openclaw ~/.openclaw-dev` in entrypoint.sh | Not found in current entrypoint.sh (may have been replaced by config sync approach at lines 20-59) |

### Fixes Needed
1. Remove `CLAUDE_AI_SESSION_KEY` reference -- this is inaccurate. Auth uses xAI API keys
2. Update or remove the `ln -sfn` symlink reference -- entrypoint.sh now uses a staging/named-volume config sync approach instead

---

## Summary Table

| File | Status | Critical Issues | Total Issues |
|---|---|---|---|
| paper-trading.md | Needs Update | All line numbers wrong (~+1000 drift) | 12 |
| ai-integration.md | Wrong | AI provider changed (GPT-4 -> Grok), classes renamed, endpoints wrong | 15+ |
| trading-strategies.md | Needs Update | 5+ undocumented strategy files | 5 |
| authentication.md | Needs Update | JWT algorithm wrong (RS256 -> HS256), token expiry wrong, missing/wrong endpoints | 10 |
| websocket-realtime.md | Needs Update | websocket/ directory doesn't exist, wrong function names | 5 |
| mcp-server.md | Needs Update | SDK version wrong, most tool counts wrong | 6 |
| openclaw.md | Mostly Accurate | Auth method reference wrong | 2 |

### Priority Order for Fixes
1. **ai-integration.md** -- Most wrong; entire GPT-4 narrative is obsolete
2. **authentication.md** -- Security-relevant: wrong algorithm claim, wrong expiry
3. **paper-trading.md** -- All line references are useless; missing 9 features
4. **websocket-realtime.md** -- References non-existent directory
5. **mcp-server.md** -- Tool counts and SDK version stale
6. **trading-strategies.md** -- Missing strategies and files
7. **openclaw.md** -- Minor auth reference fix

---

### Unresolved Questions
- Is there a plan to add refresh token functionality to auth? The doc describes it but code doesn't implement it.
- The `gpt4-analysis-history` endpoint name still references GPT-4 in the Python service. Is this intentional for backward compatibility?
- The `debug/gpt4` endpoint (line 2929 in main.py) still has GPT-4 naming. Should this be renamed?
