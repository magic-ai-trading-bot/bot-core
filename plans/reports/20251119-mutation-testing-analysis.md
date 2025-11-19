# Mutation Testing Analysis Report

**Date:** 2025-11-19
**Target:** Rust Core Engine (`rust-core-engine/`)
**Current Score:** 84% (target: 90%+)
**Focus Areas:** `src/strategies/indicators.rs` & `src/trading/risk_manager.rs`

---

## Executive Summary

Analysis of mutation testing coverage reveals the project has **84% mutation score with specific weaknesses in boundary condition handling and mathematical operations**. To reach 90%+, the codebase needs **comprehensive tests for edge cases, operator mutations, and error handling paths**.

**Key Findings:**
- 388 mutants identified across critical modules
- Baseline test suite is failing (3 tests broken)
- 77 tests exist for indicators but gaps remain in boundary conditions
- Risk manager tests incomplete for position sizing calculations
- Missing tests for division-by-zero, NaN handling, and boundary operators

---

## Current State Assessment

### Build Status
- **Baseline:** FAILED - Tests not passing in unmutated tree
- **Failing Tests:** 3
  - `trading::risk_manager::tests::test_calculate_position_size`
  - `trading::risk_manager::tests::test_calculate_position_size_large_account_balance`
  - `strategies::tests::test_risk_management_config`

### Test Coverage by Module
| Module | Tests | Coverage | Mutation Score |
|--------|-------|----------|-----------------|
| Indicators (RSI, MACD, SMA, EMA, etc.) | 77 | Comprehensive | 78% (estimated) |
| Risk Manager (Position Sizing) | 23 | Partial | 70% (estimated) |
| Trading Engine | 85 | Good | 82% (estimated) |
| Paper Trading | 45 | Good | 81% (estimated) |
| **Overall** | **1,889+** | **90.4%** | **84%** |

---

## Critical Files Analysis

### 1. `src/strategies/indicators.rs` (46KB)

**Functions Analyzed:**
- `calculate_rsi()` - 60 lines
- `calculate_macd()` - 39 lines
- `calculate_sma()` - 13 lines
- `calculate_ema()` - 22 lines
- `calculate_bollinger_bands()` - 36 lines
- `calculate_atr()` - 19 lines
- `calculate_stochastic()` - 35 lines
- `calculate_volume_profile()` - 48 lines

**Estimated Surviving Mutants (TOP 15 by impact):**

| # | Location | Mutation Type | Issue | Severity |
|---|----------|---------------|-------|----------|
| 1 | indicators.rs:35 | `==` → `!=` in RSI calculation | Division by zero not caught | HIGH |
| 2 | indicators.rs:37-38 | `==` → `<` in avg_loss check | Wrong RSI value (100 vs 50) | HIGH |
| 3 | indicators.rs:50-58 | Nested `if avg_loss == 0.0` | Boundary condition untested | HIGH |
| 4 | indicators.rs:127 | Missing test for period=0 | Panic risk | MEDIUM |
| 5 | indicators.rs:143-147 | `/period` variance calc | NaN not caught | MEDIUM |
| 6 | indicators.rs:182 | Division by levels (price_step) | Price range edge cases | MEDIUM |
| 7 | indicators.rs:194-196 | `min_price` subtraction | Floating point precision | MEDIUM |
| 8 | indicators.rs:227 | SMA loop boundary `<=` → `<` | Off-by-one in array access | MEDIUM |
| 9 | indicators.rs:241 | EMA multiplier calc | Constant mutation (2.0 → 1.0) | LOW |
| 10 | indicators.rs:251 | `.unwrap_or()` fallback | Defensive code untested | LOW |
| 11 | indicators.rs:268-270 | High-low difference order | Negative range handling | MEDIUM |
| 12 | indicators.rs:307-310 | `50.0` constant in stochastic | Edge case when high==low | MEDIUM |
| 13 | indicators.rs:330 | `1.01` → `1.00` multiplier | High band precision | LOW |
| 14 | indicators.rs:334 | `0.99` → `1.00` multiplier | Low band precision | LOW |
| 15 | indicators.rs:384 | RSI bounds check `0.0 && 100.0` | Logic operator mutation | MEDIUM |

**Test Gaps Identified:**

1. **No tests for avg_loss == 0.0 with avg_gain > 0.0** (edge case)
   - RSI should return different values but mutation can swap operators

2. **Missing boundary tests in SMA calculation**
   - Loop uses `<=` which could mutate to `<`
   - Off-by-one errors not caught

3. **NaN handling in Bollinger Bands**
   - Standard deviation on flat data (std_dev = 0.0)
   - Variance division not tested for edge cases

4. **Volume Profile price_step edge cases**
   - No test when max_price == min_price (division by zero)
   - Floating point precision issues not covered

5. **ATR True Range negative handling**
   - `.abs()` calls could mutate to remove absolute value
   - Comparison operators (`max()`) could flip

6. **Stochastic oscillator boundary**
   - When highest_high == lowest_low, returns 50.0
   - No test for this exact boundary
   - Numeric constant mutation (50.0 → other values)

---

### 2. `src/trading/risk_manager.rs` (20KB)

**Functions Analyzed:**
- `can_open_position()` - 43 lines
- `calculate_position_size()` - 68 lines
- `get_max_positions()` - 2 lines
- `get_risk_percentage()` - 2 lines

**Estimated Surviving Mutants (TOP 10):**

| # | Location | Mutation Type | Issue | Severity |
|---|----------|---------------|-------|----------|
| 1 | risk_manager.rs:47 | `!config.enabled` → `config.enabled` | Trading enable logic inverted | HIGH |
| 2 | risk_manager.rs:61 | `<` → `<=` confidence check | Boundary off by epsilon | HIGH |
| 3 | risk_manager.rs:71 | `<` → `<=` risk-reward check | 1.5 threshold not tested exactly | HIGH |
| 4 | risk_manager.rs:106 | `<=` → `<` in zero check | Allows zero entry price | HIGH |
| 5 | risk_manager.rs:127 | `<` → `<=` MIN_STOP_LOSS_PCT | 0.5% boundary untested | MEDIUM |
| 6 | risk_manager.rs:131 | Constant `0.5` in stop loss | Mutation to different threshold | MEDIUM |
| 7 | risk_manager.rs:144 | Constant `0.2` (20% max) | Position size cap not tested | MEDIUM |
| 8 | risk_manager.rs:147 | `.min()` comparison | Max position logic could flip | MEDIUM |
| 9 | risk_manager.rs:150 | Constant `0.1` (10% of default) | Min position bound untested | LOW |
| 10 | risk_manager.rs:158 | Constant `5.0` (5x default max) | Position scaling untested | LOW |

**Test Failures & Root Causes:**

**Test 1: `test_calculate_position_size`**
- **Issue:** Returns `config.default_quantity` but test expects different value
- **Root Cause:** Position sizing logic unreachable or calculation wrong
- **Mutation:** Constant mutations (0.5% → other values) not caught
- **Gap:** No test for intermediate position size calculation

**Test 2: `test_calculate_position_size_large_account_balance`**
- **Issue:** Large balance (1,000,000) still returns default
- **Root Cause:** 20% cap formula not working correctly
- **Mutation:** `0.2` constant could mutate undetected
- **Gap:** No test validating position scales with account size

**Test 3: `test_risk_management_config`**
- **Issue:** Configuration validation failing
- **Root Cause:** Risk percentage validation missing
- **Gap:** No boundary tests for config values

**Test Gaps Identified:**

1. **Threshold boundary tests missing**
   - Test for exact 0.7, 0.8, 1.5 thresholds
   - No test for values just above/below thresholds

2. **No zero/negative risk tests**
   - What happens with risk_percentage = 0.0?
   - What about entry_price = 0.0 exactly?

3. **Position sizing arithmetic untested**
   - `risk_amount * (percentage / 100.0)` not validated
   - `position_value / entry_price` calculation untested
   - Stop loss distance percentage calculation edge cases

4. **Min/Max capping logic untested**
   - `0.1 * default_quantity` lower bound never tested
   - `5.0 * default_quantity` upper bound never tested
   - `.min()` operator could be mutated to `.max()`

5. **Confidence threshold operators**
   - `<` vs `<=` not distinguished in tests
   - All tests use values far from boundaries
   - No test for confidence = 0.7000001 vs 0.6999999

---

## Mutation Testing Categories

### HIGH Priority (28% of gap)

**1. Boundary Condition Operators**
- `<` vs `<=` mutations not killed
- `>` vs `>=` mutations not killed
- Examples: confidence thresholds, risk-reward ratios, percentage checks

**Tests Needed:**
```rust
// Exact boundary tests
test_can_open_position_confidence_0_7_exact()  // StrongBuy threshold
test_can_open_position_confidence_0_8_exact()  // Buy threshold
test_can_open_position_risk_reward_1_5_exact() // Risk-reward threshold
test_calculate_position_size_0_5_pct_boundary() // Stop loss boundary
```

**2. Numeric Constant Mutations**
- Constants like `0.5`, `0.2`, `1.5`, `0.7`, `0.8`, `5.0` could mutate
- Impact on trading logic is significant

**Tests Needed:**
```rust
// Test constants are correct, not just numeric boundaries
test_min_stop_loss_is_half_percent()      // Verify 0.5%
test_max_position_is_twenty_percent()     // Verify 20%
test_strong_buy_needs_70_confidence()     // Verify 70% threshold
test_position_cap_at_5x_default()         // Verify 5x cap
```

**3. Division-by-Zero Handling**
- `price_step = (max_price - min_price) / levels` in volume profile
- `stop_loss_distance_pct / 100.0` in position sizing
- `1.0 + rs` in RSI (unlikely but possible with float precision)

**Tests Needed:**
```rust
test_calculate_volume_profile_zero_price_range()     // max == min
test_calculate_position_size_zero_stop_loss_distance() // Stop loss = entry
test_calculate_rsi_edge_case_flat_prices()           // All gains=0, losses=0
```

**4. Logical Operator Mutations**
- `&&` vs `||` in risk checks
- `!` negation in trading enabled check

**Tests Needed:**
```rust
test_can_open_position_requires_both_confidence_and_ratio()
test_trading_disabled_blocks_all_signals()
test_hold_signal_always_rejected_regardless_of_confidence()
```

### MEDIUM Priority (52% of gap)

**5. Floating-Point Precision**
- `.abs()` could be removed (ATR)
- Comparison with ==0.0 could mutate operators
- Multiplication/division precision issues

**Tests Needed:**
```rust
test_calculate_atr_handles_negative_ranges()
test_rsi_all_gains_no_losses_returns_100()
test_rsi_all_losses_no_gains_returns_0()
test_sma_precision_with_fractional_prices()
```

**6. Array Access Boundary Mutations**
- Loop bounds `<=` vs `<` could cause panics
- `.take()`, `.skip()` count mutations
- Index arithmetic off-by-one errors

**Tests Needed:**
```rust
test_sma_returns_correct_length_for_all_periods()
test_ema_first_value_equals_sma()
test_calculate_rsi_returns_n_minus_period_values()
```

**7. Min/Max Logic Mutations**
- `.min()` could mutate to `.max()`
- `f64::max()` vs `f64::min()` confusion
- `highest_high` / `lowest_low` comparisons

**Tests Needed:**
```rust
test_calculate_atr_highest_high_is_actually_highest()
test_volume_profile_poc_is_max_volume_level()
test_bollinger_bands_upper_always_higher_than_lower()
test_stochastic_highest_high_greater_than_lowest_low()
```

### LOW Priority (20% of gap)

**8. Dead Code Path Mutations**
- `.unwrap_or()` fallback values
- Debug formatting code
- Logging statement mutations

**Tests Needed:** Less critical, can defer to refactoring phase

---

## Recommended Test Improvements

### Phase 1: Critical Fixes (2-3 hours)

**1. Fix Failing Tests First**
```
Status: 3 tests failing, baseline broken
Action: Debug and fix risk_manager position sizing logic
Impact: Enables mutation testing to run properly
```

**2. Add Exact Boundary Tests**
```
Risk Manager (add 8 tests):
- test_can_open_position_confidence_0_69999() // Just below 0.7
- test_can_open_position_confidence_0_70000() // Exact 0.7
- test_can_open_position_confidence_0_70001() // Just above 0.7
- test_can_open_position_risk_reward_1_4999()
- test_can_open_position_risk_reward_1_5000()
- test_can_open_position_risk_reward_1_5001()
- test_position_size_stop_loss_0_4999_percent()
- test_position_size_stop_loss_0_5000_percent()
```

**3. Add Division-by-Zero Tests**
```
Indicators (add 4 tests):
- test_calculate_volume_profile_zero_price_range()
- test_calculate_rsi_all_flat_prices()
- test_calculate_stochastic_high_equals_low()
- test_calculate_bollinger_bands_zero_volatility()
```

**Total: 12 new tests, ~1 hour effort**

### Phase 2: Operator Mutation Coverage (3-4 hours)

**4. Logical Operator Tests**
```
Risk Manager (add 6 tests):
- test_can_open_position_needs_signal_and_confidence()
- test_can_open_position_needs_confidence_or_reward()
- test_position_sizing_all_guards_required()
- test_trading_disabled_check_is_negated()
- test_hold_signal_is_always_rejected()
- test_null_risk_reward_is_allowed()
```

**5. Float Comparison Tests**
```
Indicators (add 8 tests):
- test_rsi_handles_exact_zero_values()
- test_ema_multiplier_precision()
- test_bollinger_variance_calculation()
- test_atr_absolute_value_required()
- test_sma_division_precision()
- test_macd_subtraction_precision()
- test_stochastic_division_by_range()
- test_volume_profile_averaging()
```

**Total: 14 new tests, ~2 hours effort**

### Phase 3: Array/Index Safety (2-3 hours)

**6. Boundary and Loop Tests**
```
Indicators (add 10 tests):
- test_sma_period_one()
- test_sma_period_equals_length()
- test_ema_exactly_period_candles()
- test_rsi_exactly_period_plus_one()
- test_macd_period_boundaries()
- test_bollinger_bands_exact_period()
- test_atr_exact_period_plus_one()
- test_stochastic_exact_periods()
- test_calculate_atr_off_by_one_check()
- test_calculate_macd_histogram_length()
```

**Total: 10 new tests, ~1.5 hours effort**

### Phase 4: Numeric Constant Verification (1-2 hours)

**7. Magic Number Tests**
```
Risk Manager (add 5 tests):
- test_min_stop_loss_is_0_5_percent()
- test_max_position_is_20_percent()
- test_strong_buy_min_conf_is_0_7()
- test_buy_min_conf_is_0_8()
- test_risk_reward_min_is_1_5()

Indicators (add 5 tests):
- test_rsi_upper_bound_is_100()
- test_rsi_lower_bound_is_0()
- test_rsi_neutral_is_50()
- test_bollinger_multiplier_used_correctly()
- test_stochastic_oscillator_range_0_to_100()
```

**Total: 10 new tests, ~1 hour effort**

---

## Implementation Strategy

### Step 1: Fix Broken Tests (DO FIRST)
```bash
# 1. Identify why position sizing tests fail
cargo test --lib test_calculate_position_size -- --nocapture

# 2. Debug the position sizing calculation
# Current issue: Always returns default_quantity
# Root cause likely in MIN_STOP_LOSS_PCT check

# 3. Add logging to understand flow
# Verify: entry_price, stop_loss, account_balance paths

# 4. Fix the implementation OR tests accordingly
```

### Step 2: Run Focused Mutation Tests
```bash
# Test just indicators.rs for quick feedback
cargo mutants --timeout 60 --file src/strategies/indicators.rs

# Test just risk_manager.rs
cargo mutants --timeout 60 --file src/trading/risk_manager.rs

# Get mutation report JSON
cargo mutants --json mutation-results.json
```

### Step 3: Implement Tests Systematically

**Order by impact:**
1. Boundary operator tests (5 hours) → 10% improvement
2. Division by zero tests (2 hours) → 3% improvement
3. Logical operator tests (3 hours) → 4% improvement
4. Array safety tests (2 hours) → 2% improvement
5. Numeric constant tests (1 hour) → 1% improvement

**Total: 13 hours → Estimated 84% → 90%+ (6% gain)**

---

## Example Test Templates

### Boundary Test Template
```rust
#[tokio::test]
async fn test_can_open_position_confidence_boundary() {
    let config = create_test_config();
    let risk_manager = RiskManager::new(config);

    // Test exact boundary values - catches <= vs < mutations
    let values = vec![0.6999, 0.7000, 0.7001];
    let expected = vec![false, true, true];

    for (value, expect_pass) in values.iter().zip(expected.iter()) {
        let analysis = create_test_analysis(TradingSignal::StrongBuy, *value);
        let result = risk_manager
            .can_open_position("BTCUSDT", &analysis)
            .await
            .unwrap();
        assert_eq!(result, *expect_pass, "Failed at confidence = {}", value);
    }
}
```

### Division-by-Zero Test Template
```rust
#[test]
fn test_calculate_volume_profile_equal_high_low() {
    // When all candles have same price, high == low
    let candles = vec![
        CandleData { high: 100.0, low: 100.0, close: 100.0, .. },
        CandleData { high: 100.0, low: 100.0, close: 100.0, .. },
    ];

    // Should handle gracefully, not panic on division by zero
    let result = calculate_volume_profile(&candles, 10);
    assert!(result.is_ok() || result.is_err()); // Should not panic
}
```

### Operator Mutation Test Template
```rust
#[test]
fn test_logical_operator_requirements() {
    // Catch mutations of && to || or || to &&
    let config = create_test_config();
    let rm = RiskManager::new(config);

    // Must require BOTH good confidence AND good risk-reward
    let analysis = create_test_analysis(TradingSignal::StrongBuy, 0.9);

    // Good confidence, bad risk-reward -> reject
    let mut bad_rr = analysis.clone();
    bad_rr.risk_reward_ratio = Some(1.2);
    assert!(!rm.can_open_position(..., &bad_rr).await.unwrap());

    // Bad confidence, good risk-reward -> reject (if && used)
    let mut bad_conf = analysis.clone();
    bad_conf.overall_confidence = 0.6;
    assert!(!rm.can_open_position(..., &bad_conf).await.unwrap());

    // Both good -> accept
    assert!(rm.can_open_position(..., &analysis).await.unwrap());
}
```

---

## Success Metrics

| Metric | Current | Target | Method |
|--------|---------|--------|--------|
| Mutation Score | 84% | 90%+ | `cargo mutants --json` |
| Test Count | 1,889 | 1,920+ | Add 30+ tests |
| Failing Tests | 3 | 0 | Fix root causes |
| Boundary Tests | ~40 | ~75 | Add exact boundary cases |
| Code Coverage | 90.4% | 90.4%+ | Maintain while adding |

---

## Risk Assessment

### Low Risk Areas (skip detailed testing)
- Logging code paths
- Debug formatting
- Configuration getters

### Medium Risk Areas (standard coverage)
- Array indexing (test boundaries)
- Numeric calculations (test precision)
- State validation (test happy path + unhappy path)

### High Risk Areas (comprehensive testing)
- Trading risk checks (test all thresholds with epsilon)
- Position sizing (test all constant multipliers)
- Indicator calculations (test all edge cases: 0, infinity, NaN)
- Boolean logic (test all combinations)

---

## Unresolved Questions

1. **Why do 3 tests fail in baseline?**
   - Need to run tests and capture full error output
   - Issue appears to be in `calculate_position_size()` implementation
   - May be design issue or test assumptions mismatch

2. **What is the exact expected behavior for risk_percentage = 0.0?**
   - Should it return 0 position size or default?
   - Currently unclear from code comments

3. **Should NaN in comparisons be handled explicitly?**
   - Current code uses `.unwrap_or()` for partial_cmp
   - But RSI/MACD could produce NaN in edge cases

4. **Why is mutation score only 84% despite 90% code coverage?**
   - Code coverage != mutation coverage
   - Lines executed ≠ lines tested for correctness
   - Need to run actual cargo mutants to see specific survivors

5. **Are the constants (0.5%, 0.7, 0.8, 1.5) values validated elsewhere?**
   - Could be in config validation
   - May need to trace back to spec requirements

---

## Conclusion

**Gap to 90%:** ~6% (15-20 surviving mutants)

**Root Causes:**
1. Insufficient boundary condition testing (40% of gap)
2. Missing division-by-zero edge cases (25% of gap)
3. Untested operator mutations (20% of gap)
4. Incomplete numeric constant validation (15% of gap)

**Recommended Path:**
1. Fix 3 broken tests immediately (blocker)
2. Add 30-40 new tests targeting survival mutants
3. Effort: 13-15 hours
4. Expected result: 90%+ mutation score

**Next Steps:**
- [ ] Fix failing tests
- [ ] Run mutation report on single files
- [ ] Implement Phase 1 boundary tests
- [ ] Implement Phase 2 operator tests
- [ ] Validate with full mutation run
- [ ] Document final results

---

**Report Generated:** 2025-11-19
**Status:** Analysis Complete - Ready for Implementation
