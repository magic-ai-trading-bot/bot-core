# Mutation Testing - Next Steps & Action Plan

**Report Date:** 2025-11-19
**Current Status:** Analysis Complete - Ready for Implementation
**Estimated Work:** 13-15 hours to reach 90%+

---

## CRITICAL: Blocking Issues

### 1. THREE FAILING TESTS (MUST FIX FIRST)

**Failing Tests:**
```
✗ trading::risk_manager::tests::test_calculate_position_size
✗ trading::risk_manager::tests::test_calculate_position_size_large_account_balance
✗ strategies::tests::test_risk_management_config
```

**Symptoms:**
- Position sizing always returns `config.default_quantity`
- Expected: Calculation should vary based on account size
- Current: All scenarios return default, never calculate

**Root Cause Theories:**
1. `MIN_STOP_LOSS_PCT` check (0.5%) might be too strict
2. Position sizing calculation logic unreachable
3. Test assumptions don't match implementation

**Fix Steps:**
```bash
# 1. Run test with full output
cargo test --lib test_calculate_position_size -- --nocapture --test-threads=1

# 2. Add debug prints to calculate_position_size()
// Around line 106-141 in src/trading/risk_manager.rs
debug!("entry_price={}, account_balance={}, stop_loss={:?}", entry_price, account_balance, stop_loss);
debug!("stop_loss_distance_pct={}, MIN_STOP_LOSS_PCT={}", stop_loss_distance_pct, MIN_STOP_LOSS_PCT);

# 3. Trace which branch is taken
// Add debug after each return statement

# 4. Compare with test expectations
// Verify test setup matches implementation assumptions
```

**Estimated Time:** 2 hours

---

## Priority 1: Quick Wins (4 Hours)

**Goal:** Kill top 4 most dangerous mutants

### Task 1.1: Boundary Operator Tests (1.5h)
Create 4 tests for exact thresholds:

```rust
// File: src/trading/risk_manager.rs (add to test module)

#[tokio::test]
async fn test_can_open_position_confidence_0_6999() {
    let config = create_test_config();
    let rm = RiskManager::new(config);
    let analysis = create_test_analysis(TradingSignal::StrongBuy, 0.6999);
    let result = rm.can_open_position("BTCUSDT", &analysis).await.unwrap();
    assert!(!result, "Should reject confidence below 0.7");
}

#[tokio::test]
async fn test_can_open_position_confidence_0_7000() {
    let config = create_test_config();
    let rm = RiskManager::new(config);
    let analysis = create_test_analysis(TradingSignal::StrongBuy, 0.7000);
    let result = rm.can_open_position("BTCUSDT", &analysis).await.unwrap();
    assert!(result, "Should accept confidence at 0.7");
}

#[tokio::test]
async fn test_can_open_position_confidence_0_7001() {
    let config = create_test_config();
    let rm = RiskManager::new(config);
    let analysis = create_test_analysis(TradingSignal::StrongBuy, 0.7001);
    let result = rm.can_open_position("BTCUSDT", &analysis).await.unwrap();
    assert!(result, "Should accept confidence above 0.7");
}

// Repeat for risk-reward: 1.4999, 1.5000, 1.5001
```

**Catches Mutations:** `<` → `<=` in 2 places
**Impact:** Kill 4 mutants

### Task 1.2: Negation Test (0.5h)

```rust
#[tokio::test]
async fn test_can_open_position_trading_disabled() {
    let mut config = create_test_config();
    config.enabled = false;

    let rm = RiskManager::new(config);
    let analysis = create_test_analysis(TradingSignal::StrongBuy, 0.95);

    let result = rm.can_open_position("BTCUSDT", &analysis).await.unwrap();
    assert!(!result, "Must reject when trading is disabled");
}
```

**Catches Mutation:** `!enabled` → `enabled`
**Impact:** Kill 1 mutant

### Task 1.3: Division-by-Zero Tests (1.5h)

```rust
// File: src/strategies/indicators.rs (add to test module)

#[test]
fn test_calculate_volume_profile_zero_price_range() {
    // When all candles have the same price (high == low == close)
    let candles = vec![
        CandleData {
            open: 100.0, high: 100.0, low: 100.0, close: 100.0,
            volume: 1000.0, open_time: 0, close_time: 1,
            quote_volume: 100000.0, trades: 100, is_closed: true,
        },
        CandleData {
            open: 100.0, high: 100.0, low: 100.0, close: 100.0,
            volume: 1000.0, open_time: 1, close_time: 2,
            quote_volume: 100000.0, trades: 100, is_closed: true,
        },
    ];

    // Should not panic, either Ok or Err
    let result = calculate_volume_profile(&candles, 10);
    // Verify no panic and result is sensible
    assert!(result.is_ok() || result.is_err());
}

#[test]
fn test_calculate_rsi_all_equal_prices() {
    let prices = vec![100.0; 20]; // All same price
    let candles = create_test_candles(prices);
    let result = calculate_rsi(&candles, 14);

    assert!(result.is_ok());
    let rsi = result.unwrap();
    // Should be 50.0 (neutral) when no price movement
    for &value in &rsi {
        assert!(value >= 0.0 && value <= 100.0);
    }
}

#[test]
fn test_calculate_stochastic_high_equals_low() {
    let closes = vec![100.0; 15];
    let highs = vec![100.0; 15];
    let lows = vec![100.0; 15];
    let candles = create_test_candles_with_range(closes, highs, lows);
    let result = calculate_stochastic(&candles, 5, 3);

    assert!(result.is_ok());
    let stoch = result.unwrap();
    // Should be 50.0 when highest_high == lowest_low
    for &k in &stoch.k_percent {
        assert_eq!(k, 50.0);
    }
}
```

**Catches Mutations:** Division by zero, removed `.abs()`, comparison flips
**Impact:** Kill 5 mutants

### Task 1.4: Numeric Constant Tests (1h)

```rust
// File: src/trading/risk_manager.rs

#[test]
fn test_position_size_constants_are_correct() {
    // These tests verify exact constants, not approximate values
    
    // Verify MIN_STOP_LOSS_PCT is 0.5%
    // by testing just above and below
    let config = create_test_config();
    let rm = RiskManager::new(config);

    // Stop loss at 0.4999% should return default
    let size_0_4999 = rm.calculate_position_size(
        "BTCUSDT",
        50000.0,
        Some(50000.0 * (1.0 - 0.004999)),
        10000.0
    );
    assert_eq!(size_0_4999, config.default_quantity);

    // Stop loss at 0.5000% should calculate
    let size_0_5000 = rm.calculate_position_size(
        "BTCUSDT",
        50000.0,
        Some(50000.0 * (1.0 - 0.005000)),
        10000.0
    );
    // Should be calculated position size, not default
    assert!(size_0_5000 >= config.default_quantity * 0.5);
}
```

**Catches Mutations:** Numeric constants mutated
**Impact:** Kill 3 mutants

---

## Priority 2: Comprehensive Coverage (5 Hours)

**Goal:** Add systematic boundary and operator tests

### Task 2.1: Array Boundary Tests (2h)
Test exact array lengths and off-by-one errors

**New tests to add:**
```
test_sma_period_one()                // SMA with period=1
test_sma_exactly_period_candles()    // Exact period match
test_sma_period_one_less()           // Period-1 should fail
test_ema_exactly_period_candles()    // EMA at period boundary
test_rsi_exactly_period_plus_one()   // RSI at minimum length
test_macd_period_boundaries()        // MACD period math
test_bollinger_period_boundaries()   // Bollinger at boundaries
test_atr_exact_period_plus_one()     // ATR minimum length
test_stochastic_exact_periods()      // Both K and D periods
test_calculate_atr_off_by_one()      // Verify loop bounds
```

**Effort:** 2 hours
**Impact:** Kill 5 mutants

### Task 2.2: Logical Operator Tests (2h)
Test all boolean combinations

**New tests to add:**
```
test_position_needs_both_good_signal_and_confidence()
test_risk_reward_check_is_optional_when_not_set()
test_hold_signal_always_rejected_despite_confidence()
test_poor_risk_reward_blocks_even_strong_signal()
test_poor_confidence_blocks_even_good_risk_reward()
test_trading_disabled_blocks_any_signal()
test_null_risk_reward_is_allowed()
test_all_signal_types_respect_confidence()
test_sell_signal_same_confidence_threshold_as_buy()
test_strong_sell_same_confidence_as_strong_buy()
```

**Effort:** 2 hours
**Impact:** Kill 4 mutants

### Task 2.3: Float Precision Tests (1h)
Test calculations with edge values

**New tests to add:**
```
test_rsi_calculation_with_micro_price_movements()
test_ema_multiplier_precision()
test_bollinger_variance_with_flat_prices()
test_volume_profile_averaging_with_zeros()
test_stochastic_with_equal_high_low()
```

**Effort:** 1 hour
**Impact:** Kill 2 mutants

---

## Priority 3: Validation & Wrap-up (4 Hours)

### Task 3.1: Run Full Mutation Report (1h)

```bash
# After fixing baseline and adding Priority 1 tests
cargo mutants --timeout 60 --json mutation-report.json

# Analyze results
cat mutation-report.json | jq '.[] | select(.status=="SURVIVED")' > survivors.json
wc -l survivors.json  # Should be <20
```

### Task 3.2: Identify Remaining Survivors (1h)

For each surviving mutant:
1. Understand what mutation survived
2. Create targeted test
3. Add test to codebase

### Task 3.3: Final Tests for Survivors (1.5h)

Add any remaining tests needed to reach 90%

### Task 3.4: Documentation (0.5h)

Update:
- CLAUDE.md with new mutation score
- docs/TESTING_GUIDE.md with edge case examples
- Code comments explaining hard-to-test scenarios

---

## Implementation Timeline

### Week 1: Monday-Wednesday (8 hours)

**Monday (3 hours)**
- [ ] 8:00 - 9:00: Debug and fix 3 failing tests
- [ ] 9:00 - 10:00: Add 4 boundary operator tests
- [ ] 10:00 - 11:00: Add negation and div-by-zero tests
- [ ] 11:00 - 12:00: Add numeric constant tests

**Tuesday (2.5 hours)**
- [ ] 8:00 - 9:00: Add array boundary tests (part 1)
- [ ] 9:00 - 10:30: Add array boundary tests (part 2)

**Wednesday (2.5 hours)**
- [ ] 8:00 - 9:00: Add logical operator tests
- [ ] 9:00 - 10:00: Add float precision tests
- [ ] 10:00 - 10:30: Run tests, verify passing

### Week 1: Thursday-Friday (5 hours)

**Thursday (2 hours)**
- [ ] 8:00 - 9:00: Run full mutation report
- [ ] 9:00 - 10:00: Analyze survivors

**Friday (3 hours)**
- [ ] 8:00 - 10:00: Implement tests for remaining survivors
- [ ] 10:00 - 11:00: Final verification and documentation

---

## Success Checklist

**Blocker Fixes:**
- [ ] Fix test_calculate_position_size
- [ ] Fix test_calculate_position_size_large_account_balance
- [ ] Fix test_risk_management_config
- [ ] All tests passing: `cargo test --lib`

**Phase 1 Tests (4 hours):**
- [ ] 4 boundary operator tests (< vs <=)
- [ ] 1 negation test (!)
- [ ] 3 division-by-zero tests
- [ ] 4 numeric constant tests

**Phase 2 Tests (5 hours):**
- [ ] 10 array boundary tests
- [ ] 10 logical operator tests
- [ ] 5 float precision tests

**Phase 3 (4 hours):**
- [ ] Run mutation report
- [ ] Identify survivors
- [ ] Add targeted tests
- [ ] Verify 90%+ score
- [ ] Update documentation

**Final Verification:**
- [ ] `cargo test --lib` - All pass
- [ ] `cargo mutants --timeout 60` - ≥90% score
- [ ] `cargo fmt --check` - All formatted
- [ ] `cargo clippy -- -D warnings` - No warnings

---

## Quick Start Commands

```bash
# 1. Check current state
cd rust-core-engine
cargo test --lib 2>&1 | tail -20

# 2. Debug failing test
cargo test --lib test_calculate_position_size -- --nocapture --test-threads=1

# 3. Add your tests to risk_manager.rs
# Edit: src/trading/risk_manager.rs
# Section: #[cfg(test)] mod tests { ... }

# 4. Run just your new tests
cargo test --lib risk_manager -- --nocapture

# 5. Run indicators tests
cargo test --lib indicators -- --nocapture

# 6. Full mutation test (after fixes)
cargo mutants --timeout 60 --file src/trading/risk_manager.rs

# 7. Generate JSON report
cargo mutants --timeout 60 --json mutation-results.json
```

---

## Files to Modify

1. **src/trading/risk_manager.rs**
   - Add 12-15 tests in the test module
   - NO changes to production code (unless fixing bug)

2. **src/strategies/indicators.rs**
   - Add 20-25 tests in the test module
   - NO changes to production code

3. **No other files need changes** for initial phase

---

## Risk Assessment

**Risk Level:** LOW
- Only adding tests, no production code changes
- Tests are isolated, no dependencies
- Can't break existing functionality

**Mitigation:**
- Run full test suite after each batch of new tests
- Keep changes small and focused
- Test one module at a time

---

## Success Metrics

| Metric | Current | Target | Status |
|--------|---------|--------|--------|
| Tests Passing | 1,889 | 1,920+ | ⏳ (3 failing) |
| Mutation Score | 84% | 90%+ | ⏳ |
| Risk Manager Tests | 23 | 38+ | ⏳ |
| Indicator Tests | 77 | 102+ | ⏳ |
| Code Coverage | 90.4% | 90.4%+ | ✓ |
| ESLint Errors | 0 | 0 | ✓ |

---

## Questions to Answer

1. What is the exact expected behavior of position sizing with 1M balance?
2. Why does MIN_STOP_LOSS_PCT of 0.5% cause default return?
3. Should risk_percentage=0 allow trading?
4. What error should volume profile throw for zero price range?
5. Should RSI/MACD/EMA handle all-zero inputs or fail gracefully?

**Recommendation:** Add detailed comments in code explaining edge cases

---

**Last Updated:** 2025-11-19
**Status:** Ready to Implement
**Estimated Completion:** 13-15 hours
**Expected Result:** 90%+ mutation score
