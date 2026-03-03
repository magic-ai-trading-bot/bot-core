# FR Specification Accuracy Review

**Reviewer**: code-reviewer
**Date**: 2026-03-03
**Scope**: 6 FR specs vs actual source code
**Verdict**: Multiple specs need updates -- line numbers, default values, and API paths are stale

---

## FR-PAPER-TRADING.md

**Status**: NEEDS UPDATE

**Issues Found**:

1. **Risk parameter defaults stale** (High)
   - Spec (implied via CLAUDE.md) says: `daily_loss_limit_pct = 5%`, `max_consecutive_losses = 5`
   - Actual code (`settings.rs:749-750`): `daily_loss_limit_pct = 3.0` (optimized down from 5%), `max_consecutive_losses = 3` (optimized down from 5)
   - `cool_down_minutes = 60` and `correlation_limit = 0.7` are correct

2. **Function line numbers in CLAUDE.md are stale** (Medium -- not in spec itself but CLAUDE.md references)
   - CLAUDE.md says `engine.rs:509-560` for `process_trading_signal()` -- actual: line 1242
   - CLAUDE.md says `engine.rs:738-845` for execution simulation -- actual: slippage at 1677, market impact at 1717, partial fills at 1758
   - CLAUDE.md says `engine.rs:847-1039` for risk management -- actual: daily loss at 1977, cooldown check at 2025, correlation at 2087
   - CLAUDE.md says `engine.rs:1041-1197` for `execute_trade()` -- actual: line 2624
   - CLAUDE.md says `engine.rs:1425-1452` for `close_trade()` -- actual: line 3344

3. **File size has grown massively** (Info)
   - Engine file is now 12,793 lines (includes extensive inline tests)

4. **Portfolio fields match spec reasonably well** (OK)
   - `cool_down_until` at `portfolio.rs:82` and `consecutive_losses` at `portfolio.rs:79` confirmed

**Fixes Needed**:
- Update CLAUDE.md line number references for paper trading functions
- Update risk defaults: `daily_loss_limit_pct` is now 3.0%, `max_consecutive_losses` is now 3

---

## FR-AUTH.md

**Status**: NEEDS UPDATE

**Issues Found**:

1. **JWT Algorithm: Spec and code agree (HS256), but CLAUDE.md says RS256** (Critical)
   - FR-AUTH.md line 90-95: correctly says HS256
   - `jwt.rs:69`: `Header::new(Algorithm::HS256)` -- confirmed HS256
   - CLAUDE.md Feature Location Map says "RS256 JWT" -- this is WRONG, should be HS256
   - This is a documentation error in CLAUDE.md, not in the spec itself

2. **Auth endpoints: CLAUDE.md references non-existent routes** (High)
   - CLAUDE.md says endpoints: `/api/auth/login`, `/api/auth/register`, `/api/auth/refresh`, `GET /api/auth/me`
   - Actual routes in `handlers.rs:57`: `auth/register`, `auth/login`, `auth/verify`, `auth/profile`
   - NO `/api/auth/refresh` endpoint exists (spec marks refresh as unimplemented `[ ]`)
   - NO `/api/auth/me` endpoint -- it is `/api/auth/profile`
   - FR-AUTH.md correctly lists: `POST /api/auth/register`, `POST /api/auth/login`, `GET /api/auth/profile`
   - The spec itself is accurate; CLAUDE.md references are wrong

3. **Code location line numbers are close but shifted** (Medium)
   - Spec says `jwt.rs:31-51` for token generation -- actual: `generate_token` at line 44, `generate_token_with_session` at line 50-76
   - Spec says `jwt.rs:8-15` for Claims -- actual: lines 12-21 (has extra `session_id` field)
   - Spec says `jwt.rs:17-29` for JwtService init -- actual: lines 24-35
   - Spec says `jwt.rs:53-62` for verify -- actual: lines 82-91
   - Spec says `jwt.rs:64-66` for header extraction -- actual: lines 93-95
   - Spec says `jwt.rs:69-82` for PasswordService -- actual: lines 102-114
   - Spec says `handlers.rs:85-200` for registration -- actual: `handle_register` at line 97

4. **Claims struct has `session_id` field not documented in spec** (Low)
   - `jwt.rs:20`: `pub session_id: Option<String>` -- added for session tracking (`@spec:FR-AUTH-015`)

**Fixes Needed**:
- CLAUDE.md: Change "RS256 JWT" to "HS256 JWT"
- CLAUDE.md: Fix auth endpoints to `/api/auth/login`, `/api/auth/register`, `/api/auth/verify`, `/api/auth/profile`
- FR-AUTH.md: Update code location line numbers throughout (shifted by ~10-20 lines)
- FR-AUTH.md: Document `session_id` field in Claims struct (FR-AUTH-015 exists but code locations need updating)

---

## FR-STRATEGIES.md

**Status**: ACCURATE (Minor Issues)

**Issues Found**:

1. **Strategy list is correct** (OK)
   - RSI: `rsi_strategy.rs` -- confirmed
   - MACD: `macd_strategy.rs` -- confirmed
   - Bollinger: `bollinger_strategy.rs` -- confirmed
   - Volume: `volume_strategy.rs` -- confirmed
   - Stochastic: `stochastic_strategy.rs` -- confirmed
   - All 5 strategies exist with correct struct names

2. **Additional strategy modules exist beyond spec** (Info)
   - `hybrid_filter.rs` -- not in spec (hybrid filter/trend filter system)
   - `ml_trend_predictor.rs` -- not in spec (ML trend prediction)
   - `trend_filter.rs` -- not in spec (trend filtering)
   - These are implementation details not requiring spec additions

3. **CLAUDE.md says "Combined" strategy but spec says "Stochastic"** (Low)
   - CLAUDE.md Feature Location Map lists: "RSI, MACD, Bollinger, Volume, strategy_engine, indicators"
   - Spec correctly lists 5 strategies (RSI, MACD, Bollinger, Volume, Stochastic) plus Strategy Engine

4. **Performance numbers unverifiable** (Info)
   - Spec says 65% combined win rate, 1.5% avg profit, Sharpe 1.6
   - These are likely aspirational/historical; no automated way to verify

**Fixes Needed**:
- CLAUDE.md: Mention Stochastic strategy explicitly in the strategies list

---

## FR-WEBSOCKET.md

**Status**: NEEDS UPDATE

**Issues Found**:

1. **CLAUDE.md references `src/websocket/` directory that does not exist** (High)
   - CLAUDE.md says: Code at `rust-core-engine/src/websocket/`
   - No such directory exists. WebSocket handling is in:
     - `rust-core-engine/src/api/mod.rs:557` (`handle_websocket` function)
     - `rust-core-engine/src/binance/websocket.rs` (Binance WebSocket client)
   - FR-WEBSOCKET.md correctly references `src/api/mod.rs:handle_websocket()` -- spec is accurate here

2. **Message types in spec match code patterns** (OK)
   - MarketData, ChartUpdate, PositionUpdate, TradeExecuted, AISignalReceived, BotStatusUpdate, Error, Pong -- all referenced in codebase

3. **Broadcast channel capacity 1000 correct** (OK)
   - `api/mod.rs` and `engine.rs` both use `broadcast::channel(1000)`

4. **Client hook location correct** (OK)
   - `nextjs-ui-dashboard/src/hooks/useWebSocket.ts` -- confirmed

5. **Event types mentioned in CLAUDE.md** (Medium)
   - CLAUDE.md says events: `price_update, signal_generated, trade_executed, portfolio_update, risk_event`
   - Spec uses different names: `MarketData, AISignalReceived, TradeExecuted, PositionUpdate, BotStatusUpdate`
   - CLAUDE.md names are simplified/informal; spec is more accurate

**Fixes Needed**:
- CLAUDE.md: Change `src/websocket/` to `src/api/mod.rs` (handle_websocket function)
- CLAUDE.md: Update WebSocket event names to match spec message types

---

## FR-AI.md

**Status**: NEEDS UPDATE

**Issues Found**:

1. **Code location line numbers are significantly stale** (High)
   - Spec says `POST /ai/analyze` at `main.py:1750-1889` -- actual: line 3001
   - Spec says `AIAnalysisRequest` at `main.py:402-413` -- actual: line 789
   - Spec says `AISignalResponse` at `main.py:441-451` -- actual: line 828
   - The `main.py` file has grown to 4,657+ lines; all line references are off

2. **Model types confirmed** (OK)
   - LSTM: `models/lstm_model.py` -- confirmed
   - GRU: `models/gru_model.py` -- confirmed
   - Transformer: `models/transformer_model.py` -- confirmed
   - Ensemble: NOT a separate model file. Only referenced in migration data. The "Ensemble" approach is a combination strategy within `model_manager.py`, not a standalone model class

3. **No dedicated Ensemble model** (Medium)
   - CLAUDE.md says "Ensemble 72%" as a model type
   - No `ensemble_model.py` exists
   - `model_manager.py` coordinates multiple models but there is no standalone Ensemble class
   - The spec does not claim an Ensemble model either -- CLAUDE.md is the one making this claim

4. **API endpoints differ from CLAUDE.md claims** (High)
   - CLAUDE.md says endpoints: `POST /predict`, `/analyze`, `/sentiment`, `/train`
   - Actual endpoints in code:
     - `POST /ai/analyze` -- confirmed
     - `POST /predict-trend` -- not `/predict`
     - `POST /ai/strategy-recommendations`
     - `POST /ai/market-condition`
     - `POST /ai/feedback`
     - `POST /ai/analyze-trade`
     - `GET /ai/info`, `GET /ai/strategies`, `GET /ai/performance`
     - `GET /ai/storage/stats`, `POST /ai/storage/clear`
     - `GET /health`
   - No `/sentiment` or `/train` endpoints exist
   - FR-AI.md correctly lists `/ai/analyze` as primary endpoint

5. **Model accuracy numbers from CLAUDE.md** (Info)
   - CLAUDE.md: "LSTM 68%, GRU 65%, Transformer 70%, Ensemble 72%"
   - These are not verifiable from code -- likely historical/aspirational

**Fixes Needed**:
- FR-AI.md: Update all `main.py` line number references (all shifted by ~1000+ lines)
- CLAUDE.md: Fix AI endpoints (`/predict` -> `/predict-trend`, remove `/sentiment` and `/train`)
- CLAUDE.md: Clarify "Ensemble" is a combination approach, not a standalone model

---

## FR-REAL-TRADING.md

**Status**: NEEDS UPDATE

**Issues Found**:

1. **Code location line numbers are significantly wrong** (Critical)
   - `engine.rs` is 10,137 lines (spec references assume ~1800 lines)
   - Spec says `mod.rs:1-50` for init -- `RealTradingEngine` struct and impl start at `engine.rs:319`
   - Spec says `client.rs:442-465` for `place_market_order` -- actual: line 550-557
   - Spec says `client.rs:467-493` for `place_limit_order` -- actual: line 562-570
   - Spec says `client.rs:481-505` for `place_stop_loss_order` -- actual: line 575-584 (named `place_stop_loss_limit_order`)
   - Spec says `client.rs:495-520` for `place_take_profit_order` -- actual: line 589-598 (named `place_take_profit_limit_order`)
   - Spec says `client.rs:518-550` for `cancel_order` -- actual: line 427 (`cancel_order` on futures) and line 603 (`cancel_spot_order`)
   - Spec says `client.rs:583-615` for `query_order` -- NO `query_order` function found
   - Spec says `types.rs:331-400` for `SpotOrderRequest` -- actual: line 549
   - Spec says `types.rs:596-700` for `SpotOrderResponse` -- actual: line 674
   - Spec says `engine.rs:1092-1150` for `handle_balance_update` -- actual: line 1455
   - Spec says `engine.rs:1209-1290` for `sync_initial_state` -- NO `sync_initial_state` function found
   - Spec says `engine.rs:1294-1350` for `start_reconciliation_loop` -- NO `start_reconciliation_loop` found; reconciliation loop is inlined elsewhere
   - Spec says `engine.rs:1378-1410` for `run_reconciliation` -- actual: line 1805
   - Spec says `engine.rs:1411-1470` for `reconcile_balance` -- actual: `reconcile_balances` at line 1842
   - Spec says `engine.rs:1472-1620` for `reconcile_orders` -- actual: line 2066
   - Spec says `engine.rs:1622-1710` for `cleanup_stale_orders` -- actual: line 2216
   - Spec says `engine.rs:1713-1735` for `handle_disconnect` -- NO `handle_disconnect` function found
   - Spec says `engine.rs:1738-1800` for `emergency_stop` -- actual: line 2332

2. **Config struct differs from spec** (High)
   - Spec shows simplified `RealTradingConfig` with 6 fields (testnet, max_open_positions, daily_loss_limit_percent, etc.)
   - Actual config (`config.rs:10-202`) has 40+ fields including ATR sizing, Kelly criterion, regime filters, signal reversal
   - Field names differ: spec says `testnet: bool` -- actual: `use_testnet: bool`
   - Spec says `max_open_positions: usize` -- actual: `max_positions: u32`
   - Spec says `daily_loss_limit_percent: f64` -- actual: `max_daily_loss_usdt: f64` (absolute value, not percentage)
   - Spec says `circuit_breaker_threshold: u32` -- actual: `circuit_breaker_errors: u32`
   - Default `circuit_breaker_errors = 3` (spec says 5)

3. **API routes mostly accurate but format differs** (Medium)
   - Spec says `POST /api/real-trading/order/market` -- actual routes use different pattern
   - Actual: `POST /api/real-trading/orders` (unified order placement, not per-type endpoints)
   - `DELETE /api/real-trading/orders/:id` -- confirmed
   - `GET /api/real-trading/orders` -- confirmed
   - `GET /api/real-trading/positions` -- actual: `GET /api/real-trading/trades/open`
   - Additional routes not in spec: `PUT /api/real-trading/positions/:symbol/sltp`, `DELETE /api/real-trading/orders` (cancel all)

4. **File count and structure** (Medium)
   - Spec says "9 files" in `src/real_trading/` -- actual: 6 files (config.rs, engine.rs, mod.rs, order.rs, position.rs, risk.rs)
   - Spec says "5 files" in `src/binance/` -- actual: 5 files (correct)
   - `order.rs` is 372 lines (spec says 1-200)
   - `position.rs` is 782 lines (spec says 1-300)
   - `risk.rs` is 1,167 lines (spec says 1-200)
   - `config.rs` is 748 lines (spec says 1-150)
   - `api/real_trading.rs` is 7,187 lines (spec says 1-500)

5. **Risk manager function names differ** (Medium)
   - Spec says `validate_trade` at `risk.rs:1-50` -- actual: `validate_order` at line 169
   - Spec says `calculate_position_size` at `risk.rs:50-150` -- actual: at line 295

6. **Missing functions referenced in spec** (High)
   - `sync_initial_state` -- not found as a function
   - `start_reconciliation_loop` -- not found as a function
   - `handle_disconnect` -- not found as a function
   - `query_order` -- not found in `client.rs`

**Fixes Needed**:
- Complete rewrite of all code location references (every single one is wrong)
- Update `RealTradingConfig` data model to match actual struct
- Update API route documentation
- Remove references to non-existent functions
- Update file line counts

---

## Summary Table

| Spec File | Status | Critical Issues | High Issues | Medium Issues | Low Issues |
|-----------|--------|-----------------|-------------|---------------|------------|
| FR-PAPER-TRADING.md | Needs Update | 0 | 1 (risk defaults) | 1 (CLAUDE.md lines) | 0 |
| FR-AUTH.md | Needs Update | 1 (CLAUDE.md RS256) | 1 (CLAUDE.md routes) | 1 (line numbers) | 1 |
| FR-STRATEGIES.md | Mostly Accurate | 0 | 0 | 0 | 1 |
| FR-WEBSOCKET.md | Needs Update | 0 | 1 (CLAUDE.md dir path) | 1 (event names) | 0 |
| FR-AI.md | Needs Update | 0 | 2 (line numbers, endpoints) | 1 (Ensemble claim) | 0 |
| FR-REAL-TRADING.md | Needs Major Update | 1 (all line refs wrong) | 3 (config, missing fns, routes) | 3 (file counts, names) | 0 |

---

## CLAUDE.md Specific Errors

These errors are in `CLAUDE.md` (the hub navigation file), not in the FR specs themselves:

1. **RS256 -> HS256**: Auth section says "RS256 JWT" but code uses HS256
2. **Auth endpoints**: Lists `/api/auth/refresh` and `/api/auth/me` which don't exist; missing `/api/auth/verify`
3. **Paper trading line numbers**: All 5 line ranges for `engine.rs` are wrong (file grew from ~1400 to 12,793 lines)
4. **WebSocket code path**: Says `src/websocket/` but directory doesn't exist; actual code is in `src/api/mod.rs`
5. **AI endpoints**: Lists `/predict`, `/sentiment`, `/train` but actual endpoints are `/predict-trend`, `/ai/analyze`, etc.
6. **AI models**: Claims "Ensemble 72%" as a model but no standalone Ensemble model exists
7. **WebSocket events**: Uses informal names not matching actual message type strings

---

## Recommended Priority

1. **P0**: Fix CLAUDE.md errors (RS256->HS256, wrong endpoints, non-existent paths) -- these actively mislead developers and AI agents
2. **P1**: Rewrite FR-REAL-TRADING.md code locations (every reference is wrong)
3. **P2**: Update FR-AI.md line numbers (shifted by 1000+ lines)
4. **P3**: Update FR-AUTH.md line numbers (shifted by 10-20 lines)
5. **P4**: Update paper trading risk defaults in CLAUDE.md (3% not 5%, 3 not 5 losses)

---

## Unresolved Questions

- Should specs include line numbers at all given how quickly they become stale? Consider using function/struct names as anchors instead.
- Are the performance numbers (win rates, Sharpe ratios) still relevant or should they be removed/updated?
- The FR-REAL-TRADING.md references functions (`sync_initial_state`, `handle_disconnect`) that may have been refactored into other patterns -- were these intentionally removed or renamed?
