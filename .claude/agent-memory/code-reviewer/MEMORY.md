# Code Reviewer Agent Memory

## Feature Doc Accuracy (2026-03-03)
- `specifications/06-features/` docs are heavily stale -- line numbers in paper-trading.md drifted ~1000+ lines
- AI service switched from GPT-4/OpenAI to Grok/xAI. Classes: `GrokClient`, `GrokTradingAnalyzer`, `TechnicalAnalyzer`
- JWT uses HS256 (not RS256 as docs claim), 7-day expiry (not 15 min)
- Auth routes: register, login, verify, profile. NO logout or refresh endpoints.
- `rust-core-engine/src/websocket/` directory does NOT exist. WS logic is in `binance/websocket.rs` + paper trading events.
- MCP SDK in package.json is `^1.12.1` (not v1.26.0). 114 tools total across 12 files.
- engine.rs is 12,793 lines. Key function locations:
  - `process_trading_signal()`: line 1242
  - `apply_slippage()`: line 1677
  - `check_daily_loss_limit()`: line 1977
  - `execute_trade()`: line 2624
  - `close_trade()`: line 3344

## Strategies Module
- 11 files total: rsi, macd, bollinger, volume, stochastic, trend_filter, hybrid_filter, ml_trend_predictor, strategy_engine, indicators, types, tests, mod
- Doc only mentions 4 strategies; stochastic is the 5th undocumented one

## Auth Module
- Files: jwt.rs, handlers.rs, middleware.rs, database.rs, models.rs, security_handlers.rs, mod.rs
- Middleware functions: `with_auth()`, `with_optional_auth()`, `with_admin_auth()`
- database.rs: `find_by_email()` (not `find_user_by_email`)
- bcrypt with DEFAULT_COST (library default = 12)

## Strategy Engine (Rust)
- Signal analysis via internal strategy engine
- AI endpoints in Rust (`/api/ai/*`) route to strategy engine, not external service
