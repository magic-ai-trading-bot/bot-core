# Code Review: Real Trading Auto-Engine (Mirror Paper Trading)

**Date**: 2026-03-02
**Reviewer**: Code Review Agent
**Branch**: `main` (uncommitted changes)

---

## Code Review Summary

### Scope
- Files reviewed: 4 modified files
  - `rust-core-engine/src/real_trading/config.rs` (+28 lines)
  - `rust-core-engine/src/real_trading/engine.rs` (+970 lines)
  - `rust-core-engine/src/main.rs` (+3 lines)
  - `rust-core-engine/src/api/mod.rs` (+19 lines)
- Lines of code analyzed: ~1,020 net new lines
- Review focus: Safety, correctness, thread safety, missing error handling

### Overall Assessment

The implementation is well-structured and largely mirrors paper trading correctly. Defaults are safe (`auto_trading_enabled: false`). The 5-layer signal filter is sound. However, there are **3 high-priority bugs** that could cause real money loss or incorrect behavior in production, and **4 medium-priority issues** worth addressing before enabling auto-trading.

---

## Critical Issues

### CRIT-1: `update_consecutive_losses` uses ESTIMATED unrealized PnL, not actual realized PnL

**File**: `engine.rs` — `sl_tp_monitoring_loop` lines 2774, 2783

```rust
// BUG: unrealized_pnl is captured BEFORE close_position() is called
triggered.push((symbol, position.unrealized_pnl));

// ... later ...
self.update_consecutive_losses(estimated_pnl).await;
```

`position.unrealized_pnl` is a mark-to-market estimate at scan time. After `close_position()` is called, the actual realized PnL (net of fees/slippage) will differ — especially on illiquid pairs or fast-moving markets. If the position moves against you between the scan and the actual close (e.g. 100ms slippage), this estimated PnL could be positive when the actual PnL is negative, **causing a loss to not count toward the consecutive-loss counter**.

**Fix**: Capture the realized PnL from the closed order. The `close_position()` already returns `Result<RealOrder>`. Hook into the `PositionClosed` event or return the actual PnL from `close_position`:

```rust
for (symbol, _) in triggered {
    match self.close_position(&symbol).await {
        Ok(_order) => {
            // Get actual realized PnL from daily_metrics delta
            let actual_pnl = self.get_last_realized_pnl().await;
            self.update_consecutive_losses(actual_pnl).await;
        }
        Err(e) => error!("Failed to auto-close {} on SL/TP trigger: {}", symbol, e),
    }
}
```

Alternatively, listen to `RealTradingEvent::PositionClosed { pnl, .. }` which already carries the correct realized PnL (set in `handle_execution_report` at line 1082).

---

### CRIT-2: Missing warmup gate — strategy signals can fire with <5 candles

**File**: `engine.rs` — `strategy_signal_loop` line 2848

```rust
if klines.len() < 2 {
    continue;
}
```

Paper trading requires **50 candles minimum** across both 5m and 15m timeframes before firing signals (`check_warmup_period` in `paper_trading/engine.rs:1437`). The real trading loop only checks `len() < 2`. On boot, the cache starts empty. Each 30s iteration fetches only `Some(5)` candles. After the first candle closure is detected (skipped as warmup), the next closure will trigger strategy analysis with only ~5-10 candles — far fewer than MACD's required 35 or Bollinger Bands' 20.

Trading with 5 candles produces statistically meaningless indicator values. RSI on 5 bars is not RSI.

**Fix**: Add an explicit candle count check before running `analyze_market`:

```rust
// After build_strategy_input, before running strategy engine:
let cache = self.historical_data_cache.read().await;
let cache_key_5m = format!("{}_5m", symbol);
let cache_key_15m = format!("{}_15m", symbol);
let count_5m = cache.get(&cache_key_5m).map(|v| v.len()).unwrap_or(0);
let count_15m = cache.get(&cache_key_15m).map(|v| v.len()).unwrap_or(0);
drop(cache);

if count_5m < 50 || count_15m < 50 {
    debug!("Warmup incomplete for {} ({}/{} candles)", symbol, count_5m, count_15m);
    continue;
}
```

Also, consider fetching more candles on boot (e.g. `Some(100)`) instead of `Some(5)` to warm up faster.

---

### CRIT-3: SL/TP monitoring loop can double-close a position

**File**: `engine.rs` — `sl_tp_monitoring_loop` vs pre-existing `check_sl_tp_triggers()`

The new `sl_tp_monitoring_loop` runs every 5s and calls `close_position()`. The pre-existing public `check_sl_tp_triggers()` does the same thing and can be called from external API handlers. Neither uses a per-symbol close guard — they only share the `execution_lock` at order placement.

Race condition:
1. Loop scans positions, sees BTCUSDT triggered → adds to `triggered` list
2. External `check_sl_tp_triggers()` call sees same position → calls `close_position("BTCUSDT")`
3. External call closes position, removes from `positions` DashMap
4. Loop's `close_position("BTCUSDT")` also runs → position already gone → `Err("Position not found")` — this is safe, **but**
5. `update_consecutive_losses(estimated_pnl)` runs anyway on the loop side (even on Err, since the match block correctly only calls it on `Ok`)

Wait — re-reading the code, `update_consecutive_losses` is only called inside `Ok(order) => { ... }`. So if the second close fails, consecutive losses aren't double-counted. The error path correctly drops it. **This is safe as coded.**

However, there is a softer issue: if `check_sl_tp_triggers()` closes the position (no PnL tracking), the loop fails silently. Consecutive losses won't be counted for externally-triggered closes. Consider consolidating into one code path.

**Severity downgraded to High** (not Critical) — no money loss risk, but loss tracking can be incomplete.

---

## High Priority Findings

### HIGH-1: `ai_market_bias` is never populated in real trading engine

**File**: `engine.rs` — `strategy_signal_loop`, bias check at line 2975

```rust
let bias = self.ai_market_bias.read().await;
if let Some(market_bias) = bias.get(symbol.as_str()) {
    // This branch is NEVER reached — ai_market_bias is always empty
```

The `ai_market_bias` map is initialized empty (line 359) and **there is no setter method on `RealTradingEngine`**. In paper trading, `update_ai_market_bias()` is called via the `/api/paper-trading/market-bias` endpoint (api/mod.rs line 877). The real trading engine has no equivalent. The code silently falls through to `true // No bias data — allow`, meaning the AI bias filter is a no-op.

This is safe (trade is allowed), but the filter provides no protection and the code is misleading.

**Fix**: Either add `pub async fn update_ai_market_bias(...)` to `RealTradingEngine` and wire it in the API (routing to both engines), or remove the dead bias check from the real trading signal loop.

---

### HIGH-2: `check_portfolio_risk` uses free USDT balance as equity base, not total portfolio value

**File**: `engine.rs` — `check_portfolio_risk` line 2600

```rust
let usdt_balance = self.get_usdt_balance().await;
// get_usdt_balance() returns balances["USDT"].free — excludes locked USDT and all unrealized PnL
```

Paper trading uses `portfolio.equity` which includes all position values. The real trading version only uses free USDT. When positions are open, free USDT decreases while locked USDT and position value increase. This makes the risk calculation denominator **understate** the true portfolio value, causing the risk percentage to appear **higher than actual**, potentially blocking valid trades.

Example: $10,000 portfolio, $8,000 in open BTC positions. Free USDT = $2,000. A new $100 position with 1% SL = $1 risk. Paper trading: 1/10000 = 0.01%. Real trading: 1/2000 = 0.05% — 5x higher than actual.

**Fix**: Use total equity (free + locked + unrealized PnL across all positions):

```rust
let total_equity = self.get_usdt_balance().await
    + self.get_total_unrealized_pnl().await;
// get_total_unrealized_pnl() already exists at line 2270
```

---

### HIGH-3: Hardcoded $50,000 fallback price in `check_risk_limits`

**File**: `engine.rs` — `place_order` → `check_risk_limits`, line 1115 (pre-existing, now exercised)

```rust
let effective_price = price.unwrap_or(50000.0);
```

`process_signal_for_real_trade` calls `place_market_order(..., None, true)` — passing `None` for price. For assets other than BTC, $50,000 will produce wildly incorrect position value estimates during risk validation. For ETH at $3,000, a 1-unit order validates against $50,000 (16x overstatement), potentially blocking legitimate trades.

**Fix**: Pass the actual entry price calculated at step 7 of `process_signal_for_real_trade`:

```rust
// Step 10 — pass entry_price instead of None
match self
    .place_market_order(symbol, side, quantity, None, true)  // <-- current
    // should be:
    .place_order_with_price(symbol, side, quantity, Some(entry_price), None, true)
```

Or update the `place_market_order` signature to accept an optional price hint used only for risk estimation.

---

### HIGH-4: Lock ordering risk in `update_consecutive_losses`

**File**: `engine.rs` — `update_consecutive_losses` lines 2504, 2514

```rust
let mut losses = self.consecutive_losses.write().await;  // Lock A acquired
// ...
let mut cd = self.cool_down_until.write().await;          // Lock B acquired while A held
```

`is_in_cooldown` acquires locks in reverse order:
```rust
let cool_down_until = self.cool_down_until.read().await;  // Lock B acquired
// ...
let losses = *self.consecutive_losses.read().await;        // Lock A acquired while B held
```

This is a classic ABBA lock ordering pattern. With `tokio::sync::RwLock`, both tasks can simultaneously hold their first lock and wait for the other, causing a deadlock when `sl_tp_monitoring_loop` calls `update_consecutive_losses` at the same time `strategy_signal_loop` calls `is_in_cooldown`.

**Fix**: Always acquire locks in the same order. Since `consecutive_losses` and `cool_down_until` are logically coupled, consider combining them into one struct protected by one lock:

```rust
struct CooldownState {
    consecutive_losses: u32,
    cool_down_until: Option<DateTime<Utc>>,
}
cooldown_state: Arc<RwLock<CooldownState>>,
```

Alternatively, in `is_in_cooldown`, read `cool_down_until` and drop the lock before reading `consecutive_losses`:

```rust
async fn is_in_cooldown(&self) -> bool {
    let until = *self.cool_down_until.read().await;  // Lock B, then drop
    if let Some(until) = until {
        if Utc::now() < until {
            let remaining = (until - Utc::now()).num_minutes();
            let losses = *self.consecutive_losses.read().await;  // Now safe
            warn!("Cool-down active: {} minutes remaining (consecutive losses: {})", remaining, losses);
            return true;
        }
    }
    false
}
```

---

## Medium Priority Improvements

### MED-1: "1h" timeframe requested in `build_strategy_input` but never fetched

**File**: `engine.rs` — `build_strategy_input` line 3231 vs `strategy_signal_loop` line 2836

The signal loop only fetches `["5m", "15m"]` into `historical_data_cache`. But `build_strategy_input` also looks for `"1h"`:

```rust
for timeframe in &["5m", "15m", "1h"] {
    let cache_key = format!("{}_{}", symbol, timeframe);
    if let Some(klines) = cache.get(&cache_key) { ... }
}
```

Since "1h" is never populated, `volume_24h` is always 0.0 and the 1h data is absent from `StrategyInput`. If strategies use 1h data for trend confirmation, they'll silently receive no data.

**Fix**: Either add "1h" to the fetch loop, or remove it from `build_strategy_input`. Match what paper trading actually uses.

---

### MED-2: Signal confirmation window too tight relative to signal loop frequency

**File**: `engine.rs` — `strategy_signal_loop` signal confirmation logic, lines 3007-3031

The signal loop runs every 30s. The 5m timeframe produces a new closed candle every 5 minutes. The confirmation window requires `now - first_seen >= 60` seconds (1 minute). This means two consecutive 5m candle signals could confirm in as little as 60 seconds — which would require the strategy to fire twice within 60-600 seconds.

With a 30s loop and 5m candles, the minimum confirmation gap is ~5 minutes (one candle). The `now - first_seen >= 60` guard passes if there are two signals 60+ seconds apart. This is loose enough to allow confirmation within a single 5-minute candle period (if the price action triggers the strategy twice in that window via the 30s loop hitting the same candle twice).

Wait — the closed candle detection check (`last_close_time <= prev_time`) prevents double-processing the same closed candle. So confirmation requires signals from **two different closed candles** at minimum. Since 5m candles are 5 minutes apart and the loop is 30s, the minimum actual confirmation gap is ~5 minutes. The `>= 60` guard is redundant but harmless.

Actually this is fine — documenting for clarity only.

---

### MED-3: `set_market_data_cache` mutates `&mut self` before Arc wrapping

**File**: `main.rs` lines 233-237, `engine.rs` line 117

```rust
Ok(mut engine) => {
    engine.set_market_data_cache(market_data_processor.get_cache().clone());
    let engine = std::sync::Arc::new(engine);
```

`set_market_data_cache` takes `&mut self` and sets `self.market_data_cache = Some(cache)`. This is called before `Arc::new()`, so there's no concurrent access issue at this point. However, `market_data_cache` is `Option<MarketDataCache>` (not `Arc<RwLock<...>>`), meaning it can't be updated after `Arc::new()`. The comment says "Must be called before start()" — this is correct and the code enforces it.

Minor concern: `MarketDataCache` uses `Arc<DashMap<...>>` internally so cloning it is cheap (shared reference). This is fine.

**No fix needed** — just documenting the design is intentional.

---

### MED-4: `SL/TP monitoring loop` duplicates `check_sl_tp_triggers` without deduplication event

**File**: `engine.rs` — two separate SL/TP close paths

The new `sl_tp_monitoring_loop` (loop-based, internal) and the existing `check_sl_tp_triggers()` (public, callable externally via MCP tools or API) both do the same SL/TP check. If `check_sl_tp_triggers` is called by an MCP tool while the loop runs, the second caller will get `Err("Position not found")`. This is safe but produces confusing error logs.

More importantly: **external calls to `check_sl_tp_triggers` do NOT call `update_consecutive_losses`**, so losses from those closes don't count toward cool-down.

**Fix**: Consolidate SL/TP closing logic into one path, or have `check_sl_tp_triggers` delegate to an internal method that includes PnL tracking.

---

## Low Priority Suggestions

### LOW-1: `Kline` import used only in type annotation for a private field

The `Kline` import in the import block is used only for `historical_data_cache: Arc<RwLock<HashMap<String, Vec<Kline>>>>`. This is correct but could be replaced with `CandleData` (the processed form) to avoid coupling to the raw Binance type throughout the cache. This is an architectural preference, not a bug.

### LOW-2: `_reasoning`, `_entry_price`, `_stop_loss`, `_take_profit` parameters in `process_external_ai_signal` are unused

**File**: `engine.rs` line 964

```rust
pub async fn process_external_ai_signal(
    &self,
    symbol: String,
    signal_type: TradingSignal,
    confidence: f64,
    _reasoning: String,       // unused
    _entry_price: f64,        // unused — current price from current_prices map used instead
    _stop_loss: Option<f64>,  // unused — recalculated from config
    _take_profit: Option<f64>,// unused — recalculated from config
) -> Result<()> {
```

The AI's suggested SL/TP and entry price are completely ignored. The function recalculates everything from config. This may be intentional (config-driven, not AI-driven), but it means the AI's risk assessment output (`stop_loss_suggestion`, `take_profit_suggestion` from `AIRiskAssessment`) is never used for real trades, only paper trades.

If this is intentional, consider removing the unused parameters or documenting why they're ignored (e.g. "config SL/TP overrides AI suggestions for safety").

### LOW-3: `emit_event` on `SignalRejected` inside `check_correlation_limit` uses "portfolio" as symbol

```rust
self.emit_event(RealTradingEvent::SignalRejected {
    symbol: "portfolio".to_string(),  // misleading
    reason: format!(...),
});
```

The actual symbol being rejected is passed as a parameter to `check_correlation_limit` (via `is_long` direction only). The symbol is not available inside this function. Either add `symbol: &str` parameter to `check_correlation_limit` or use "correlation_limit" as the pseudo-symbol. Using "portfolio" is confusing in event streams.

---

## Positive Observations

1. **Safe-by-default config**: `auto_trading_enabled: false` in `Default` is correct. No accidental live trading.

2. **Mutual exclusion validation**: `short_only_mode && long_only_mode` conflict detection in `config.rs` validate() is correct and prevents misconfiguration.

3. **Config validation completeness**: All 4 new numeric fields have range validation (`min_signal_confidence`, `correlation_limit`, `max_portfolio_risk_pct`, conflict check).

4. **Correct closed-candle detection**: Using `klines[len()-2]` (second-to-last = last closed candle, last = open candle) is the correct Binance pattern. Paper trading uses the same approach.

5. **Warmup-on-first-detection skip**: `if prev_time == 0 { continue; }` correctly prevents firing on the first candle seen after boot (before a baseline is established).

6. **Explicit `execution_lock` in `place_order`**: Market orders are serialized through the execution lock, preventing duplicate orders.

7. **5-layer filter mirrors paper trading**: Layer 1 (neutral), Layer 2 (confidence), Layer 3 (direction mode), Layer 4 (choppy market), Layer 5 (confirmation) — this correctly ports the paper trading filter chain.

8. **Proper config drop before async work**: `drop(config)` called before `await` points to avoid holding `RwLock` across yield points — this is idiomatic and correct.

9. **Auto-trading check in signal loop at runtime**: Re-reading `config.auto_trading_enabled` each iteration allows runtime disable without restart.

10. **`market_data_cache` shared with paper trading**: Same WebSocket price cache → real trading sees prices within the same millisecond latency window as paper trading.

---

## Recommended Actions

Priority order (highest risk first):

1. **[CRITICAL] Fix consecutive-loss tracking to use actual realized PnL** — subscribe to `RealTradingEvent::PositionClosed { pnl }` instead of using `unrealized_pnl` estimate.

2. **[CRITICAL] Add 50-candle warmup gate** before running `strategy_engine.analyze_market()` in `strategy_signal_loop`. Fetch 100 candles on first boot instead of 5.

3. **[HIGH] Fix lock ordering in `update_consecutive_losses` / `is_in_cooldown`** — consolidate `consecutive_losses` + `cool_down_until` into a single `Arc<RwLock<CooldownState>>`.

4. **[HIGH] Fix equity base in `check_portfolio_risk`** — use `get_usdt_balance() + get_total_unrealized_pnl()` as the denominator.

5. **[HIGH] Pass actual price to `place_market_order`** for risk validation — avoid the hardcoded $50,000 fallback during pre-trade risk checks.

6. **[HIGH] Wire `ai_market_bias` to real trading engine** — add setter method + API route, or remove the dead filter code.

7. **[MED] Add "1h" timeframe fetch to signal loop** or remove it from `build_strategy_input`.

8. **[MED] Consolidate SL/TP closing paths** — have `check_sl_tp_triggers()` delegate to same internal method that calls `update_consecutive_losses`.

9. **[LOW] Remove unused parameters from `process_external_ai_signal`** or add documentation explaining they're intentionally ignored.

---

## Metrics

- Type Coverage: N/A (build environment toolchain mismatch prevented `cargo check`)
- Test Coverage: No new tests added for the ~970 new lines
- Linting Issues: Cannot run `cargo clippy` due to toolchain issue
- New tests needed: warmup gate, consecutive-loss tracking with actual PnL, cooldown deadlock scenario, portfolio risk with open positions

---

## Unresolved Questions

1. Should the real trading engine also consume AI market bias from the `/api/paper-trading/market-bias` endpoint, or have its own endpoint? The current code has the field but no writer.

2. Is the intent to use the AI's suggested `stop_loss_suggestion` / `take_profit_suggestion` from `AIRiskAssessment` for real trades, or always use config-driven SL/TP? The current code ignores AI suggestions entirely for real trades.

3. `check_sl_tp_triggers()` is a public method callable via MCP tools. Should it be deprecated now that `sl_tp_monitoring_loop` runs continuously? Or kept for manual override?

4. The `get_klines` call in `strategy_signal_loop` uses the `BinanceClient` directly (likely mainnet REST). Are rate limits accounted for? With 2 symbols × 2 timeframes = 4 REST calls per 30s iteration, this is 8 requests/minute — well within Binance's 1200 weight/minute limit, but should be monitored as symbol count grows.
