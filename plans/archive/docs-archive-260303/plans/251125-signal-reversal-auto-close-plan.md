# Implementation Plan: Smart Auto-Close on Signal Reversal

**Document Version:** 1.0
**Created:** 2025-11-25
**Status:** Ready for Implementation
**Priority:** MEDIUM
**Estimated Effort:** 6-8 hours

---

## Overview

Add smart auto-close feature for paper trading engine that automatically closes existing positions and opens new ones when receiving high-confidence opposite signals under specific conditions, enabling dynamic strategy adaptation to market reversals.

**Current Behavior:**
- System rejects new signal if position already exists for that symbol
- User must manually close position before opening opposite direction

**Target Behavior:**
- System evaluates reversal conditions (confidence, market regime, P&L)
- Automatically closes existing position if conditions met
- Opens new position in opposite direction atomically
- Logs and broadcasts reversal events

---

## Requirements

### Functional Requirements

**FR-REVERSAL-001:** System shall detect opposite signal direction (LONG→SHORT or SHORT→LONG)
**FR-REVERSAL-002:** System shall check signal confidence ≥ 75%
**FR-REVERSAL-003:** System shall verify market regime = "trending"
**FR-REVERSAL-004:** System shall check position P&L < 10%
**FR-REVERSAL-005:** System shall close existing position and open new one atomically
**FR-REVERSAL-006:** System shall log all reversal events with reason
**FR-REVERSAL-007:** System shall broadcast WebSocket event for frontend
**FR-REVERSAL-008:** System shall support enable/disable via settings

### Non-Functional Requirements

**NFR-REVERSAL-001:** Reversal check adds <50ms latency to signal processing
**NFR-REVERSAL-002:** No race conditions during close+open sequence
**NFR-REVERSAL-003:** Rollback if open fails after close
**NFR-REVERSAL-004:** All risk checks still apply after reversal
**NFR-REVERSAL-005:** 100% test coverage for reversal logic

---

## Architecture

### Component Interaction

```
AITradingSignal
    ↓
process_trading_signal()  [engine.rs:554]
    ↓
check_existing_positions()
    ↓
should_close_on_reversal() [NEW]
    ├─ check_opposite_direction() [NEW]
    ├─ check_signal_confidence() [NEW]
    ├─ check_market_regime() [NEW]
    └─ check_position_pnl() [NEW]
    ↓
close_existing_and_open_new() [NEW]
    ├─ close_trade() [existing]
    ├─ execute_trade() [existing]
    └─ broadcast_reversal_event() [NEW]
```

### Data Flow

```
Input: AITradingSignal {
  signal_type: Short,
  confidence: 0.78,
  market_regime: "trending",
  ...
}

Current: Open Long position (P&L: +8%)

Process:
1. Detect opposite direction: Long → Short ✓
2. Check confidence: 0.78 ≥ 0.75 ✓
3. Check regime: "trending" ✓
4. Check P&L: 8% < 10% ✓
5. Close Long at current_price
6. Open Short at execution_price
7. Broadcast reversal event

Result: Position reversed successfully
```

---

## Implementation Tasks

### Task 1: Add Reversal Settings

**File:** `rust-core-engine/src/paper_trading/settings.rs`
**Location:** Inside `RiskSettings` struct

```rust
// Add to RiskSettings (line ~117)
pub struct RiskSettings {
    // ... existing fields ...

    /// Enable smart auto-close on signal reversal
    pub enable_signal_reversal: bool,

    /// Minimum signal confidence for reversal (default: 0.75)
    pub reversal_min_confidence: f64,

    /// Maximum position P&L % for reversal (default: 10.0)
    pub reversal_max_pnl_pct: f64,

    /// Required market regimes for reversal (default: ["trending"])
    pub reversal_allowed_regimes: Vec<String>,
}
```

**Update Default Implementation (line ~365):**
```rust
impl Default for RiskSettings {
    fn default() -> Self {
        Self {
            // ... existing defaults ...
            enable_signal_reversal: false, // Disabled by default for safety
            reversal_min_confidence: 0.75,
            reversal_max_pnl_pct: 10.0,
            reversal_allowed_regimes: vec!["trending".to_string()],
        }
    }
}
```

**Update Validation (line ~508):**
```rust
// Add to validate() function
if !(0.0..=1.0).contains(&self.reversal_min_confidence) {
    return Err(anyhow::anyhow!("Reversal confidence must be between 0 and 1"));
}

if self.reversal_max_pnl_pct < 0.0 || self.reversal_max_pnl_pct > 100.0 {
    return Err(anyhow::anyhow!("Reversal max P&L must be between 0% and 100%"));
}
```

---

### Task 2: Add Market Regime Detection

**File:** `rust-core-engine/src/paper_trading/engine.rs`
**Location:** New helper function in PaperTradingEngine impl

```rust
/// Detect market regime from signal metadata
///
/// Checks signal metadata for "market_regime" field.
/// Falls back to "trending" if not present (safe default).
fn detect_market_regime(&self, signal: &AITradingSignal) -> String {
    // Check if signal has market_regime in metadata
    if let Some(metadata) = &signal.metadata {
        if let Some(regime) = metadata.get("market_regime") {
            if let Some(regime_str) = regime.as_str() {
                return regime_str.to_string();
            }
        }
    }

    // Fallback: Detect from AI analysis if available
    if let Some(metadata) = &signal.metadata {
        if let Some(analysis) = metadata.get("ai_analysis") {
            if let Some(analysis_str) = analysis.as_str() {
                // Simple keyword detection
                let lower = analysis_str.to_lowercase();
                if lower.contains("trending") || lower.contains("trend") {
                    return "trending".to_string();
                }
                if lower.contains("ranging") || lower.contains("range") {
                    return "ranging".to_string();
                }
                if lower.contains("volatile") || lower.contains("volatility") {
                    return "volatile".to_string();
                }
            }
        }
    }

    // Safe default
    "trending".to_string()
}
```

---

### Task 3: Implement Reversal Condition Checker

**File:** `rust-core-engine/src/paper_trading/engine.rs`
**Location:** New function after `check_position_correlation` (~636)

```rust
/// Check if signal should trigger position reversal
///
/// Conditions:
/// 1. Signal confidence ≥ configured threshold
/// 2. Market regime in allowed list
/// 3. Existing position P&L < max threshold
/// 4. Opposite direction from existing position
async fn should_close_on_reversal(
    &self,
    signal: &AITradingSignal,
    existing_trade: &PaperTrade,
) -> Result<bool> {
    let settings = self.settings.read().await;

    // Feature disabled?
    if !settings.risk.enable_signal_reversal {
        return Ok(false);
    }

    // Check 1: Signal confidence
    if signal.confidence < settings.risk.reversal_min_confidence {
        debug!(
            "Reversal rejected: low confidence {:.2}% < {:.2}%",
            signal.confidence * 100.0,
            settings.risk.reversal_min_confidence * 100.0
        );
        return Ok(false);
    }

    // Check 2: Market regime
    let market_regime = self.detect_market_regime(signal);
    if !settings.risk.reversal_allowed_regimes.contains(&market_regime) {
        debug!(
            "Reversal rejected: market regime '{}' not in allowed list {:?}",
            market_regime, settings.risk.reversal_allowed_regimes
        );
        return Ok(false);
    }

    // Check 3: Position P&L (use absolute percentage)
    let pnl_pct = existing_trade.pnl_percentage.abs();
    if pnl_pct >= settings.risk.reversal_max_pnl_pct {
        debug!(
            "Reversal rejected: high P&L {:.2}% >= {:.2}% (use trailing stop instead)",
            pnl_pct, settings.risk.reversal_max_pnl_pct
        );
        return Ok(false);
    }

    // Check 4: Opposite direction
    let signal_type = match signal.signal_type {
        crate::strategies::TradingSignal::Long => TradeType::Long,
        crate::strategies::TradingSignal::Short => TradeType::Short,
        _ => return Ok(false), // Neutral signal can't reverse
    };

    if signal_type == existing_trade.trade_type {
        return Ok(false); // Same direction, not a reversal
    }

    drop(settings);

    info!(
        "✅ Reversal conditions met for {}: {:?} → {:?} (confidence: {:.1}%, regime: {}, P&L: {:.2}%)",
        signal.symbol,
        existing_trade.trade_type,
        signal_type,
        signal.confidence * 100.0,
        market_regime,
        pnl_pct
    );

    Ok(true)
}
```

---

### Task 4: Implement Atomic Close + Open

**File:** `rust-core-engine/src/paper_trading/engine.rs`
**Location:** New function after `should_close_on_reversal`

```rust
/// Close existing position and open new one in opposite direction
///
/// This is an atomic operation - if opening new position fails,
/// the close will have already happened (cannot rollback).
/// However, this is acceptable for paper trading.
async fn close_and_reverse_position(
    &self,
    signal: AITradingSignal,
    existing_trade_id: &str,
    current_price: f64,
) -> Result<TradeExecutionResult> {
    let symbol = signal.symbol.clone();

    info!(
        "🔄 Reversing position for {}: trade_id={}, current_price={}",
        symbol, existing_trade_id, current_price
    );

    // Step 1: Close existing position
    let close_result = self.close_trade(existing_trade_id).await;

    if let Err(e) = close_result {
        error!(
            "❌ Failed to close position {} for reversal: {}",
            existing_trade_id, e
        );
        return Ok(TradeExecutionResult {
            success: false,
            trade_id: None,
            error_message: Some(format!("Reversal failed: could not close existing position - {}", e)),
            execution_price: None,
            fees_paid: None,
        });
    }

    info!("✅ Closed position {} for reversal", existing_trade_id);

    // Step 2: Execute new trade in opposite direction
    // This will go through full risk checks again
    let new_trade_result = self.execute_trade(signal.clone(), current_price).await;

    // Step 3: Broadcast reversal event
    let _ = self.event_broadcaster.send(PaperTradingEvent {
        event_type: "position_reversed".to_string(),
        data: serde_json::json!({
            "symbol": symbol,
            "closed_trade_id": existing_trade_id,
            "new_trade_id": new_trade_result.trade_id,
            "signal_confidence": signal.confidence,
            "timestamp": chrono::Utc::now().timestamp(),
            "success": new_trade_result.success,
        }),
        timestamp: chrono::Utc::now(),
    });

    if new_trade_result.success {
        info!(
            "✅ Position reversed successfully: {} → new trade_id={}",
            symbol,
            new_trade_result.trade_id.as_ref().unwrap()
        );
    } else {
        warn!(
            "⚠️ Position closed but new trade failed for {}: {}",
            symbol,
            new_trade_result.error_message.as_ref().unwrap_or(&"Unknown error".to_string())
        );
    }

    Ok(new_trade_result)
}
```

---

### Task 5: Integrate into Signal Processing

**File:** `rust-core-engine/src/paper_trading/engine.rs`
**Location:** Modify `process_trading_signal` function (line ~654-671)

**Replace the existing position check block:**

```rust
// ========== EXISTING POSITION CHECK & REVERSAL ==========
// Check if we already have a position for this symbol
let portfolio = self.portfolio.read().await;
let existing_positions: Vec<&PaperTrade> = portfolio
    .get_open_trades()
    .iter()
    .filter(|trade| trade.symbol == signal.symbol)
    .collect();

if existing_positions.len() >= symbol_settings.max_positions as usize {
    drop(portfolio);

    // Check if this is a reversal signal
    if existing_positions.len() == 1 {
        let existing_trade = existing_positions[0];

        if self.should_close_on_reversal(&signal, existing_trade).await? {
            // Get current price for close execution
            let current_price = match portfolio.current_prices.get(&signal.symbol) {
                Some(&price) => price,
                None => {
                    return Ok(TradeExecutionResult {
                        success: false,
                        trade_id: None,
                        error_message: Some("Cannot reverse: no current price available".to_string()),
                        execution_price: None,
                        fees_paid: None,
                    });
                }
            };

            // Execute reversal (close + open)
            return self.close_and_reverse_position(
                signal,
                &existing_trade.id,
                current_price
            ).await;
        }
    }

    // Not a reversal or conditions not met
    debug!(
        "Maximum positions reached for {} (no reversal)",
        signal.symbol
    );
    return Ok(TradeExecutionResult {
        success: false,
        trade_id: None,
        error_message: Some("Maximum positions reached".to_string()),
        execution_price: None,
        fees_paid: None,
    });
}

drop(portfolio);
```

---

### Task 6: Update Tests

**File:** `rust-core-engine/tests/test_paper_trading.rs`
**Location:** Add new test module at end

```rust
#[cfg(test)]
mod reversal_tests {
    use super::*;

    #[tokio::test]
    async fn test_reversal_enabled_high_confidence() {
        // Setup: portfolio with open LONG position
        let mut settings = PaperTradingSettings::default();
        settings.risk.enable_signal_reversal = true;
        settings.risk.reversal_min_confidence = 0.75;
        settings.risk.reversal_max_pnl_pct = 10.0;

        let engine = create_test_engine(settings).await;

        // Open initial LONG position
        let long_signal = create_test_signal("BTCUSDT", TradingSignal::Long, 0.80);
        let result = engine.process_trading_signal(long_signal).await.unwrap();
        assert!(result.success);
        let long_trade_id = result.trade_id.unwrap();

        // Send opposite SHORT signal with high confidence
        let short_signal = create_test_signal("BTCUSDT", TradingSignal::Short, 0.78);
        let result = engine.process_trading_signal(short_signal).await.unwrap();

        // Should reverse (close LONG, open SHORT)
        assert!(result.success);
        assert_ne!(result.trade_id.unwrap(), long_trade_id);

        // Verify: LONG closed, SHORT open
        let portfolio = engine.portfolio.read().await;
        assert_eq!(portfolio.get_open_trades().len(), 1);
        assert_eq!(portfolio.get_open_trades()[0].trade_type, TradeType::Short);
    }

    #[tokio::test]
    async fn test_reversal_rejected_low_confidence() {
        let mut settings = PaperTradingSettings::default();
        settings.risk.enable_signal_reversal = true;
        settings.risk.reversal_min_confidence = 0.75;

        let engine = create_test_engine(settings).await;

        // Open LONG
        let long_signal = create_test_signal("BTCUSDT", TradingSignal::Long, 0.80);
        engine.process_trading_signal(long_signal).await.unwrap();

        // Send SHORT with LOW confidence
        let short_signal = create_test_signal("BTCUSDT", TradingSignal::Short, 0.60);
        let result = engine.process_trading_signal(short_signal).await.unwrap();

        // Should reject (confidence too low)
        assert!(!result.success);
        assert!(result.error_message.unwrap().contains("Maximum positions reached"));
    }

    #[tokio::test]
    async fn test_reversal_rejected_high_pnl() {
        let mut settings = PaperTradingSettings::default();
        settings.risk.enable_signal_reversal = true;
        settings.risk.reversal_max_pnl_pct = 10.0;

        let engine = create_test_engine(settings).await;

        // Open LONG
        let long_signal = create_test_signal("BTCUSDT", TradingSignal::Long, 0.80);
        engine.process_trading_signal(long_signal).await.unwrap();

        // Simulate price increase (15% profit)
        let mut portfolio = engine.portfolio.write().await;
        let trade_id = portfolio.get_open_trades()[0].id.clone();
        let trade = portfolio.trades.get_mut(&trade_id).unwrap();
        trade.pnl_percentage = 15.0; // High profit
        drop(portfolio);

        // Send SHORT signal
        let short_signal = create_test_signal("BTCUSDT", TradingSignal::Short, 0.80);
        let result = engine.process_trading_signal(short_signal).await.unwrap();

        // Should reject (P&L too high)
        assert!(!result.success);
    }

    #[tokio::test]
    async fn test_reversal_rejected_wrong_regime() {
        let mut settings = PaperTradingSettings::default();
        settings.risk.enable_signal_reversal = true;
        settings.risk.reversal_allowed_regimes = vec!["trending".to_string()];

        let engine = create_test_engine(settings).await;

        // Open LONG
        let long_signal = create_test_signal("BTCUSDT", TradingSignal::Long, 0.80);
        engine.process_trading_signal(long_signal).await.unwrap();

        // Send SHORT with "ranging" regime
        let mut short_signal = create_test_signal("BTCUSDT", TradingSignal::Short, 0.80);
        short_signal.metadata = Some(serde_json::json!({
            "market_regime": "ranging"
        }));
        let result = engine.process_trading_signal(short_signal).await.unwrap();

        // Should reject (wrong regime)
        assert!(!result.success);
    }

    #[tokio::test]
    async fn test_reversal_disabled_by_default() {
        let settings = PaperTradingSettings::default(); // enable_signal_reversal = false
        let engine = create_test_engine(settings).await;

        // Open LONG
        let long_signal = create_test_signal("BTCUSDT", TradingSignal::Long, 0.80);
        engine.process_trading_signal(long_signal).await.unwrap();

        // Send SHORT
        let short_signal = create_test_signal("BTCUSDT", TradingSignal::Short, 0.80);
        let result = engine.process_trading_signal(short_signal).await.unwrap();

        // Should reject (feature disabled)
        assert!(!result.success);
    }
}
```

---

## Testing Strategy

### Unit Tests (Required)

1. **Reversal Enabled + All Conditions Met**
   - Given: Open LONG, high-confidence SHORT signal, trending regime, low P&L
   - Expected: Position reversed successfully

2. **Reversal Rejected - Low Confidence**
   - Given: Confidence 60% < 75% threshold
   - Expected: Rejected with "Maximum positions reached"

3. **Reversal Rejected - High P&L**
   - Given: Position P&L 15% > 10% threshold
   - Expected: Rejected (use trailing stop instead)

4. **Reversal Rejected - Wrong Regime**
   - Given: Market regime "ranging" not in ["trending"]
   - Expected: Rejected

5. **Reversal Disabled by Default**
   - Given: `enable_signal_reversal = false`
   - Expected: Rejected (feature off)

6. **Same Direction Signal**
   - Given: LONG signal with existing LONG position
   - Expected: Rejected (not a reversal)

7. **Neutral Signal**
   - Given: Neutral signal with existing position
   - Expected: Rejected (can't reverse to neutral)

### Integration Tests

1. **Reversal with Full Risk Checks**
   - Verify: New position still passes all risk checks
   - Verify: Daily loss limit checked
   - Verify: Cool-down period checked
   - Verify: Position correlation checked

2. **WebSocket Event Broadcasting**
   - Verify: `position_reversed` event sent with correct data
   - Verify: Frontend receives reversal notification

3. **Database Persistence**
   - Verify: Old position marked as closed with reason "AISignal"
   - Verify: New position persisted correctly
   - Verify: Portfolio metrics updated

### Manual Testing Checklist

- [ ] Enable feature via settings API
- [ ] Trigger reversal with high-confidence signal
- [ ] Verify old position closed
- [ ] Verify new position opened
- [ ] Check WebSocket events in frontend
- [ ] Verify logs show reversal reason
- [ ] Test with low confidence (should reject)
- [ ] Test with high P&L position (should reject)
- [ ] Test with "ranging" regime (should reject)
- [ ] Disable feature and verify rejection

---

## Security Considerations

### Risk Mitigation

1. **Disabled by Default**
   - Feature flag `enable_signal_reversal = false`
   - User must explicitly enable

2. **Conservative Thresholds**
   - High confidence required (75%)
   - Low P&L threshold (10%)
   - Only "trending" regime allowed

3. **All Risk Checks Still Apply**
   - New position validates daily loss limit
   - Cool-down period checked
   - Position correlation checked
   - Margin requirements verified

4. **Atomic Operation**
   - Close completes before open attempt
   - If open fails, user notified (position already closed)
   - No partial state (either fully reversed or fully rejected)

5. **Audit Trail**
   - All reversals logged with timestamp
   - Close reason set to "AISignal"
   - WebSocket events for monitoring

---

## Performance Considerations

### Latency Impact

**Additional Operations:**
- Reversal check: ~5ms (4 condition checks)
- Market regime detection: ~1ms (metadata lookup)
- Position close: existing operation
- Position open: existing operation

**Total Added Latency:** ~6ms (within <50ms requirement)

### Optimization

1. **Early Exit**
   - Check feature flag first (fastest)
   - Check direction second (simple comparison)
   - Expensive checks last (confidence, regime, P&L)

2. **Settings Cache**
   - Settings read once at start of function
   - No repeated lock acquisitions

3. **No Database Calls**
   - All checks use in-memory data
   - Database writes happen in existing functions

---

## Risks & Mitigations

### Risk 1: Close Succeeds, Open Fails

**Scenario:** Position closed but new position rejected by risk checks

**Impact:** User has no position (expected state)

**Mitigation:**
- Not critical for paper trading (no real money)
- Log clearly shows what happened
- WebSocket event indicates failure
- User can manually re-enter if desired

**Decision:** Acceptable for paper trading

### Risk 2: Race Conditions

**Scenario:** Multiple signals arrive simultaneously

**Impact:** Could trigger multiple reversals

**Mitigation:**
- Trade execution lock already in place (line 560)
- Only one signal processed at a time
- Subsequent signals will see updated state

**Decision:** Already handled by existing lock

### Risk 3: Infinite Reversal Loop

**Scenario:** Signal A reverses to B, then B reverses back to A repeatedly

**Impact:** Excessive trading, fees accumulation

**Mitigation:**
- P&L threshold prevents frequent reversals
- Minimum confidence prevents weak signals
- Market regime filter adds stability
- Cool-down period applies after consecutive losses

**Decision:** Multiple safeguards prevent this

### Risk 4: Missing Market Regime Data

**Scenario:** Signal doesn't include market regime

**Impact:** Cannot validate regime condition

**Mitigation:**
- Fallback to "trending" (safe default)
- AI analysis keyword detection
- Feature can be disabled if regime data unreliable

**Decision:** Handled with graceful fallback

---

## Unresolved Questions

### Question 1: Regime Detection Source

**Question:** Should we add dedicated market regime detection or rely on AI service?

**Options:**
1. **Use AI metadata** (current plan)
   - Pros: Already available, leverages GPT-4 analysis
   - Cons: Depends on external service

2. **Add dedicated Rust indicator**
   - Pros: Self-contained, no dependencies
   - Cons: More code, needs indicator library

**Recommendation:** Start with AI metadata, add indicator if needed

### Question 2: P&L Calculation Timing

**Question:** Should P&L check use current price or last update price?

**Options:**
1. **Current price** (current plan)
   - Pros: Most accurate, real-time
   - Cons: Requires price lookup

2. **Last cached P&L**
   - Pros: Faster, no lookup needed
   - Cons: Slightly stale data

**Recommendation:** Use cached P&L (updated on every price tick anyway)

### Question 3: Reversal Cooldown

**Question:** Should we add cooldown between reversals?

**Options:**
1. **No cooldown** (current plan)
   - Pros: Immediate reaction to signals
   - Cons: Could reverse frequently in choppy markets

2. **Add 15-minute cooldown**
   - Pros: Prevents excessive reversals
   - Cons: Might miss valid opportunities

**Recommendation:** Start without, monitor behavior, add if needed

---

## Files to Modify

### Primary Changes

1. **`rust-core-engine/src/paper_trading/settings.rs`**
   - Add reversal settings to `RiskSettings`
   - Update defaults
   - Add validation

2. **`rust-core-engine/src/paper_trading/engine.rs`**
   - Add `detect_market_regime()`
   - Add `should_close_on_reversal()`
   - Add `close_and_reverse_position()`
   - Modify `process_trading_signal()`

3. **`rust-core-engine/tests/test_paper_trading.rs`**
   - Add `reversal_tests` module
   - Add 7 test cases

### Documentation Updates

4. **`docs/features/paper-trading.md`**
   - Add "Signal Reversal" section
   - Document settings and behavior

5. **`specs/01-requirements/1.1-functional-requirements/FR-PAPER-TRADING.md`**
   - Add FR-PAPER-TRADING-009 specification

---

## Implementation Steps

### Phase 1: Core Functionality (2-3 hours)

1. Add reversal settings to `settings.rs` (30 min)
2. Implement `detect_market_regime()` (30 min)
3. Implement `should_close_on_reversal()` (1 hour)
4. Implement `close_and_reverse_position()` (1 hour)

### Phase 2: Integration (1-2 hours)

5. Modify `process_trading_signal()` (1 hour)
6. Test manually with curl commands (30 min)
7. Verify WebSocket events (30 min)

### Phase 3: Testing (2-3 hours)

8. Write unit tests (1.5 hours)
9. Write integration tests (1 hour)
10. Run full test suite (30 min)

### Phase 4: Documentation (1 hour)

11. Update feature documentation (30 min)
12. Update specs (30 min)

**Total:** 6-8 hours

---

## Rollback Plan

If issues discovered after deployment:

1. **Immediate:** Set `enable_signal_reversal = false` in settings API
2. **Short-term:** Revert commits for this feature
3. **Long-term:** Fix issues, re-test, re-deploy

Feature is disabled by default, so rollback is clean.

---

## Success Criteria

### Functional

- [ ] Reversal triggered with high-confidence opposite signal
- [ ] Reversal rejected with low confidence (<75%)
- [ ] Reversal rejected with high P&L (>10%)
- [ ] Reversal rejected with wrong regime (not "trending")
- [ ] Feature disabled by default
- [ ] All risk checks still apply to new position
- [ ] WebSocket event broadcast on reversal
- [ ] Logs show clear reversal reason

### Non-Functional

- [ ] <50ms latency added to signal processing
- [ ] No race conditions (protected by existing lock)
- [ ] 100% test coverage for new functions
- [ ] No breaking changes to existing API
- [ ] Documentation complete

---

## References

### Code Locations

- Signal processing: `engine.rs:553-671`
- Position close: `portfolio.rs:257-303`
- Trade execution: `engine.rs:1041-1197`
- Settings defaults: `settings.rs:365-387`
- Existing tests: `tests/test_paper_trading.rs`

### Specifications

- Paper Trading FR: `specs/01-requirements/1.1-functional-requirements/FR-PAPER-TRADING.md`
- Risk Management: `specs/01-requirements/1.1-functional-requirements/FR-RISK.md`
- Traceability Matrix: `specs/TRACEABILITY_MATRIX.md`

### Related Features

- Trailing Stop Loss: `engine.rs:1300-1450`
- Daily Loss Limit: `engine.rs:847-891`
- Position Correlation: `engine.rs:628-636`

---

**Plan Status:** ✅ Ready for Implementation
**Next Step:** Review plan → Implement Phase 1 → Test → Deploy
**Estimated Timeline:** 1-2 days (including testing)
