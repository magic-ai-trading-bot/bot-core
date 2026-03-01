# Code Review: Paper Trading vs Real Trading Gap Analysis

**Date:** 2026-03-02
**Focus:** Trading-logic gaps between paper and real trading engines
**Files reviewed:**
- `rust-core-engine/src/paper_trading/engine.rs` (execute_trade ~L1041-1300, process_trading_signal ~L500-760, monitor_open_trades ~L2472-2539, execute_pending_trades ~L2285-2468)
- `rust-core-engine/src/real_trading/engine.rs` (process_signal_for_real_trade ~L3119-3321, sl_tp_monitoring_loop ~L2770-2827, strategy_signal_loop ~L2831-3115)
- `rust-core-engine/src/real_trading/position.rs` (full)
- `rust-core-engine/src/real_trading/risk.rs` (calculate_position_size ~L288-358)
- `rust-core-engine/src/real_trading/config.rs` (calculate_stop_loss/take_profit ~L294-309)
- `rust-core-engine/src/paper_trading/trade.rs` (full)
- `rust-core-engine/src/binance/client.rs` (change_leverage ~L421-450)

---

## Overall Assessment

Real trading is missing **7 significant gaps** relative to paper trading. Two are critical (margin mode, SL/TP formula discrepancy). Four are high priority (position sizing ignores leverage, no liquidation monitoring, no AI reversal, no per-symbol config). One is medium (funding fees).

---

## Critical Issues

### GAP-1: Margin Mode Never Set — Real Trading Is Likely Using CROSS Margin

**Paper trading does:** Simulates isolated margin. `PaperTrade.initial_margin = notional_value / leverage` — the position is isolated with a fixed margin per trade. Liquidation check uses `entry_price * (1.0 - 1.0 / leverage)` (isolated formula).

**Real trading does:** `change_leverage()` is called before placing the order (L3205-3224), but `change_margin_type()` is **never called**. The `BinanceClient::change_margin_type()` function exists in `client.rs` but is not invoked anywhere in `engine.rs`.

**Impact:** If the Binance account defaults to CROSS margin, all real trades share a single margin pool. A single bad trade can drain margin from other positions. This contradicts the paper trading model (isolated), miscalculates liquidation risk, and can cause catastrophic cascading liquidations in real trading.

**Fix:** Call `change_margin_type(symbol, "ISOLATED")` immediately before `change_leverage()`, tolerating the `-4046` "already set" error (already handled in client).

```rust
// Before change_leverage(), add:
if self.use_futures {
    match self.binance_client.change_margin_type(symbol, "ISOLATED").await {
        Ok(_) => info!("Set ISOLATED margin for {}", symbol),
        Err(e) => warn!("Failed to set margin type for {}: {}", symbol, e),
    }
}
```

**Priority: CRITICAL**

---

### GAP-2: SL/TP Formula Inconsistency Between Config and Engine

**Paper trading does (engine.rs L1196-1219):**
```
stop_loss = entry_price * (1.0 - stop_loss_pct / (leverage * 100.0))
```
The SL/TP percentages are **PnL-based** (e.g., 5% SL = 5% loss on capital). With 10x leverage, that means 0.5% price move.

**Real trading does (engine.rs L3230-3240):** Same formula — correctly divides by `(lev * 100.0)`. This part is correct.

**But:** `RealTradingConfig::calculate_stop_loss()` (config.rs L294-299) uses a **different formula**:
```rust
entry_price * (1.0 - self.default_stop_loss_percent / 100.0)  // price-based, no leverage!
```
This method is called by `RealTradingRiskManager::calculate_stop_loss/take_profit` (risk.rs L364-382) and `calculate_position_size_auto_sl` (risk.rs L341-358).

**Impact:** Any code path that uses `config.calculate_stop_loss()` or `risk_manager.calculate_sl_tp()` produces SL prices that are **leverage * further** from entry than intended. For 10x leverage with 2% SL config: engine uses 0.2% price move, but config method uses 2.0% price move — a 10x difference. The `process_signal_for_real_trade` path is correct, but the `set_auto_sl_tp` API endpoint and any external callers will set wrong SL/TP.

**Fix:** Add leverage parameter to `config.calculate_stop_loss()`, or document clearly which formula each method uses and add a separate `calculate_stop_loss_with_leverage()`.

```rust
pub fn calculate_stop_loss_pnl_based(&self, entry_price: f64, is_long: bool, leverage: u32) -> f64 {
    let lev = leverage as f64;
    if is_long {
        entry_price * (1.0 - self.default_stop_loss_percent / (lev * 100.0))
    } else {
        entry_price * (1.0 + self.default_stop_loss_percent / (lev * 100.0))
    }
}
```

**Priority: CRITICAL**

---

## High Priority Findings

### GAP-3: Position Sizing Ignores Leverage in Real Trading

**Paper trading does (engine.rs L1221-1251):**
1. Computes `risk_amount = equity * position_size_pct / 100`
2. Computes `stop_loss_pct` (price distance)
3. `max_position_value = risk_amount / stop_loss_pct` (notional value at risk)
4. **Multiplies by leverage**: `max_position_value_with_leverage = max_position_value * leverage`
5. Caps by `free_margin * 0.95` and 20% equity safety limit
6. Final: `quantity = actual_position_value / entry_price`

**Real trading does (risk.rs L288-338):**
1. `risk_amount = account_balance * risk_per_trade_percent / 100`
2. `position_value = risk_amount / stop_distance`
3. `position_size = position_value / entry_price`
4. Caps by `max_position_size_usdt` and `max_total_exposure_usdt`
5. **No leverage multiplication at all**

**Impact:** For 10x leverage, real trading sizes positions at 1/10th of what paper trading would. With 10x leverage, the paper trade would buy 10x more contracts for the same capital at risk. Real trading is massively undersizing positions when leverage > 1x.

**Fix:**
```rust
// In calculate_position_size(), after computing position_value:
let leveraged_position_value = position_value * config.max_leverage as f64;
let position_size = leveraged_position_value / entry_price;
// Update caps to also consider leverage:
let max_size_by_margin = (account_balance * 0.95) * config.max_leverage as f64 / entry_price;
let final_size = position_size
    .min(max_size_by_position)
    .min(max_size_by_exposure)
    .min(max_size_by_margin);
```

**Priority: HIGH**

---

### GAP-4: No Liquidation Risk Monitoring in Real Trading

**Paper trading does (engine.rs L2514-2519, trade.rs L447-458):**
- Every price tick, checks `trade.is_at_liquidation_risk(current_price)` using bankruptcy price formula
- `bankruptcy_price = entry_price * (1.0 - 1.0 / leverage)` for longs
- Checks 5% buffer before bankruptcy price
- Closes with `CloseReason::MarginCall` if at risk

**Real trading does:** `sl_tp_monitoring_loop` (engine.rs L2770-2827) only checks `should_trigger_stop_loss()` and `should_trigger_take_profit()`. No liquidation/bankruptcy price check exists in `RealPosition`. The `check_sl_tp_triggers()` at L2398 also has no liquidation check.

**Impact:** If a fast price move blows through the SL order on Binance (e.g., during flash crash, low liquidity), and the engine-side SL hasn't been hit yet, the position could approach bankruptcy without any emergency close logic. Also, local SL monitoring is the fallback when Binance order placement fails.

**Fix:** Add `is_at_liquidation_risk()` to `RealPosition`, analogous to paper trading, and call it in `sl_tp_monitoring_loop`:
```rust
// In sl_tp_monitoring_loop, add:
} else if position.is_at_liquidation_risk() {
    warn!("Liquidation risk for {} - emergency close", symbol);
    triggered.push((symbol, position.unrealized_pnl));
}
```

**Priority: HIGH**

---

### GAP-5: No Position Reversal Logic in Real Trading

**Paper trading does (engine.rs L1133-1159):**
- When a new opposing signal arrives on an already-open position, checks `should_ai_enable_reversal()` or `settings.risk.enable_signal_reversal`
- If reversal enabled, calls `close_and_reverse_position()` — closes old trade and opens new one in opposite direction
- Supports AI-auto-reversal mode that decides based on accuracy, win rate, market regime, momentum, volatility

**Real trading does (engine.rs L3182-3190):**
```rust
if self.positions.contains_key(symbol) {
    // Just skip — no reversal logic at all
    return Ok(());
}
```

**Impact:** Real trading can never reverse a position on signal. When the strategy flips from Long to Short, real trading just ignores the Short signal while the Long position bleeds. Paper trading would reverse the position, potentially locking in gains and capturing the new trend.

**Fix:** Add reversal logic to `process_signal_for_real_trade`, optionally behind a `config.enable_signal_reversal` flag:
```rust
if self.positions.contains_key(symbol) {
    let config = self.config.read().await;
    if config.enable_signal_reversal {
        // Close existing and open opposite
        if let Err(e) = self.close_position(symbol).await {
            error!("Failed to close for reversal: {}", e);
        }
        // Continue to open new position below...
    } else {
        return Ok(()); // Original behavior
    }
}
```

**Priority: HIGH**

---

### GAP-6: No Per-Symbol Config in Real Trading (Flat Config Only)

**Paper trading does:** `settings.get_symbol_settings(symbol)` returns per-symbol overrides for `leverage`, `position_size_pct`, `stop_loss_pct`, `take_profit_pct`, `max_positions`, `enabled`. Each symbol can have completely different risk parameters.

**Real trading does:** Single `RealTradingConfig` applies to all symbols. No per-symbol leverage, SL/TP, or position size overrides exist. The `allowed_symbols` list exists but no per-symbol parameters.

**Impact:** Real trading cannot run BTCUSDT at 10x leverage and SOLUSDT at 3x leverage simultaneously. It cannot apply tighter SL% to more volatile altcoins. This makes the real engine less flexible than paper trading.

**Fix:** Add `symbol_overrides: HashMap<String, SymbolOverride>` to `RealTradingConfig`, and resolve in `process_signal_for_real_trade` using the same fallback pattern as paper trading's `get_symbol_settings()`.

**Priority: HIGH**

---

## Medium Priority Improvements

### GAP-7: Real Trading Has No Funding Fee Tracking

**Paper trading does:** `PaperTrade.update_with_price()` accepts `funding_rate: Option<f64>` and accumulates `funding_fees`. Funding fees are subtracted from final PnL at trade close (`realized_pnl = price_diff * quantity - trading_fees - funding_fees`).

**Real trading does:** `RealPosition` has no `funding_fees` field. The commission from fills is tracked via `total_commission`, but Binance Futures charges funding every 8 hours which is a separate P&L component. PnL calculated in `partial_close()` and `calculate_unrealized_pnl()` does not include funding.

**Impact:** On positions held for hours or days, funding fees (positive or negative) can be significant (0.01-0.1% per 8-hour period). PnL reporting will be overstated (or understated in negative funding environments). This also affects the daily loss limit comparison.

**Fix:** Add `funding_fees: f64` to `RealPosition`. Accumulate via `UserDataStreamEvent` funding fee updates or via REST API polling. Include in `calculate_unrealized_pnl()`.

**Priority: MEDIUM**

---

## Low Priority Suggestions

### GAP-8: Real Trading Missing Slippage and Market Impact Accounting

**Paper trading simulates:**
- Random slippage (0 to `max_slippage_pct`) applied to execution price
- Market impact based on order size vs typical volume
- Partial fill probability

**Real trading:** Market orders do experience real slippage, but the engine records the price from `entry_price` (pre-order price hint), not the actual fill price from the execution report. The `actual_fill_price` from Binance is available in `ExecutionReport.last_executed_price` and gets stored in `RealOrder.fills`, but `RealPosition.entry_price` may be set from the hint price rather than VWAP of fills.

**Recommendation:** Verify `update_position_from_fill()` (L1009) uses actual fill prices for `entry_price` calculation. Not a gap per se (real market handles this), but worth verifying for PnL accuracy.

**Priority: LOW**

---

## Positive Observations

1. **Leverage setting before order placement** (L3204-3224) is correct — calls `change_leverage()` before `place_order()`.
2. **SL/TP formula in process_signal_for_real_trade** (L3229-3240) correctly applies the PnL-based formula with leverage divisor — matches paper trading.
3. **5-layer signal filtering** in `strategy_signal_loop` mirrors paper trading exactly: neutral skip, confidence gate, direction mode, choppy market, signal confirmation.
4. **AI bias check** is properly ported to real trading with the same Long (-0.3) / Short (-0.5) asymmetric thresholds.
5. **Consecutive loss cool-down** tracks PnL from SL/TP triggers and calls `update_consecutive_losses()`.
6. **Trailing stop in RealPosition** correctly implements the ratchet mechanism (only moves in favorable direction).

---

## Recommended Actions (Prioritized)

1. **[CRITICAL]** Add `change_margin_type(symbol, "ISOLATED")` call in `process_signal_for_real_trade` before `change_leverage()`. File: `real_trading/engine.rs` ~L3204.

2. **[CRITICAL]** Fix `RealTradingConfig::calculate_stop_loss/take_profit` to be leverage-aware, or rename to `_price_based` to prevent misuse. File: `real_trading/config.rs` L294-309. Also fix `RealTradingRiskManager::calculate_sl_tp()` and `set_auto_sl_tp()`.

3. **[HIGH]** Apply leverage multiplier in `RealTradingRiskManager::calculate_position_size()`. File: `real_trading/risk.rs` L320-322.

4. **[HIGH]** Add `is_at_liquidation_risk()` to `RealPosition` and check it in `sl_tp_monitoring_loop`. File: `real_trading/position.rs` + `real_trading/engine.rs` ~L2790.

5. **[HIGH]** Add position reversal option to `process_signal_for_real_trade` behind `config.enable_signal_reversal` flag. File: `real_trading/engine.rs` ~L3182.

6. **[HIGH]** Add per-symbol config override support to `RealTradingConfig`. Files: `real_trading/config.rs` + `real_trading/engine.rs`.

7. **[MEDIUM]** Add `funding_fees` field to `RealPosition` and include in PnL calculation. File: `real_trading/position.rs`.

---

## Metrics

- Files reviewed: 7 source files + supporting test code
- Gaps found: 7 (2 critical, 4 high, 1 medium)
- SL/TP formula: CORRECT in main execution path, INCORRECT in config helper methods
- Leverage handling: Correctly SET on Binance, but NOT applied to position sizing
- Margin mode: `change_margin_type()` exists in client but never called in engine

---

## Unresolved Questions

1. Does the Binance testnet account default to ISOLATED or CROSS margin? (Affects urgency of GAP-1)
2. Is `set_auto_sl_tp()` (L2351) currently called anywhere in production flow? If so, GAP-2 is already causing wrong SL/TP prices.
3. For GAP-3 (position sizing with leverage): does `get_usdt_balance()` return the total account balance or available margin? If it returns only available margin, the leverage multiplier is already partially accounted for. Need to verify (see `get_usdt_balance()` L1345).
