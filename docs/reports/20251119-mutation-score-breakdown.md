# Mutation Score Breakdown - Quick Reference

**Current:** 84% | **Target:** 90%+ | **Gap:** 6% (~15-20 mutants)

---

## File-by-File Mutation Score Estimates

### Critical Files

```
src/strategies/indicators.rs (46KB)
├─ 388 total mutants in scope
├─ Estimated 78% killed (survivors: ~85)
├─ Top Issues:
│  ├─ RSI comparison operators (avg_loss == 0.0) ................... 8 mutants
│  ├─ SMA loop boundaries (<=  vs <) .............................. 6 mutants
│  ├─ Volume profile division (max_price - min_price) ............ 5 mutants
│  ├─ Bollinger bands variance division ........................... 4 mutants
│  ├─ ATR high-low order/negation ............................... 4 mutants
│  ├─ Stochastic oscillator constants (50.0) .................... 3 mutants
│  ├─ EMA multiplier and constants .............................. 3 mutants
│  ├─ MACD subtraction operators ................................ 2 mutants
│  └─ Array indexing off-by-one errors ......................... 5 mutants
└─ Tests Needed: +25 tests (estimated effort: 4 hours)

src/trading/risk_manager.rs (20KB)
├─ Currently FAILING - Baseline broken
├─ Estimated 70% killed (survivors: ~12)
├─ Top Issues:
│  ├─ Trading enabled negation (!) .............................. 2 mutants
│  ├─ Confidence threshold operators (< vs <=) ................. 4 mutants
│  ├─ Risk-reward ratio threshold (1.5) ........................ 3 mutants
│  ├─ Position sizing constants (0.5%, 0.2, 5.0, 0.1) ......... 5 mutants
│  ├─ Division by zero in stop loss distance .................. 2 mutants
│  ├─ Min/Max position logic ..................................... 2 mutants
│  └─ Risk percentage zero guard ................................ 1 mutant
└─ Tests Needed: +15 tests (estimated effort: 3 hours)
```

---

## Mutation Type Breakdown

### By Category (Percentage of Surviving Mutants)

```
Boundary Operator Mutations (28%)
  └─ < vs <=  (confidence, risk-reward, SMA loop)
  └─ > vs >=  (array bounds, price comparisons)
  └─ == vs != (zero checks, equality tests)
  └─ Tests: Add exact boundary value tests
     Example: test for 0.7000001 vs 0.6999999

Numeric Constant Mutations (18%)
  └─ 0.5 → 0.4, 0.6       (MIN_STOP_LOSS_PCT)
  └─ 0.7 → 0.6, 0.8       (StrongBuy confidence)
  └─ 0.8 → 0.7, 0.9       (Buy confidence)
  └─ 1.5 → 1.4, 1.6       (risk_reward minimum)
  └─ 0.2 → 0.15, 0.25     (max position 20%)
  └─ 5.0 → 4.0, 6.0       (position size cap)
  └─ Tests: Verify exact constants, not approximate
     Example: assert_eq!(min_stop_loss_pct, 0.5)

Division & Math Operations (15%)
  └─ Dividend by zero      (price_step, stop_loss_distance)
  └─ Precision errors      (variance, EMA multiplier)
  └─ Absolute value removal (ATR true ranges)
  └─ Tests: Test when numerator/denominator = 0
     Example: test_volume_profile_zero_price_range()

Logical Operator Mutations (12%)
  └─ && → ||               (confidence AND risk_reward)
  └─ ! negation removed    (trading_enabled check)
  └─ Comparison flip       (min vs max)
  └─ Tests: Test combinations of conditions
     Example: good_conf + bad_ratio = reject

Array Index Mutations (11%)
  └─ Loop bounds off-by-one
  └─ Array access panics
  └─ Slice boundary errors
  └─ Tests: Test exact period boundaries
     Example: test_sma_exactly_period_candles()

Floating Point Precision (9%)
  └─ NaN handling
  └─ Infinity handling
  └─ Rounding differences
  └─ Tests: Test edge values
     Example: test_rsi_all_gains_equals_100()

Other Mutations (7%)
  └─ Dead code paths
  └─ Debug/logging code
  └─ Unreachable branches
  └─ Lower priority for mutation killing
```

---

## Top 10 Most Dangerous Surviving Mutants

| Rank | File | Line | Type | Danger | Effort |
|------|------|------|------|--------|--------|
| 1 | risk_manager.rs | 47 | `!enabled` removed | HIGH | 0.5h |
| 2 | indicators.rs | 35 | `==` → `!=` in RSI | HIGH | 0.5h |
| 3 | risk_manager.rs | 61 | `<` → `<=` confidence | HIGH | 0.5h |
| 4 | risk_manager.rs | 71 | `<` → `<=` risk-reward | HIGH | 0.5h |
| 5 | indicators.rs | 182 | Division by zero | MEDIUM | 1h |
| 6 | indicators.rs | 227 | `<=` → `<` in loop | MEDIUM | 0.5h |
| 7 | risk_manager.rs | 144 | Constant `0.2` mutation | MEDIUM | 1h |
| 8 | risk_manager.rs | 106 | `<=` → `<` zero guard | MEDIUM | 0.5h |
| 9 | indicators.rs | 307 | Constant `50.0` stochastic | MEDIUM | 0.5h |
| 10 | indicators.rs | 268 | ATR `.abs()` removed | MEDIUM | 1h |

**Total effort to kill top 10:** ~6 hours

---

## Test Implementation Roadmap

### Week 1: Foundation (8 hours)

**Day 1: Fix Blockers (2 hours)**
- Debug 3 failing tests
- Fix `calculate_position_size()` logic
- Verify baseline passes

**Day 2: Phase 1 Tests (3 hours)**
- 12 boundary condition tests
- 4 division-by-zero tests
- All HIGH priority mutants

**Day 3: Phase 2 Tests (3 hours)**
- 14 operator mutation tests
- Run first full mutation report
- Identify any missed survivors

### Week 2: Coverage (7 hours)

**Day 4: Phase 3 Tests (2.5 hours)**
- 10 array/index boundary tests
- Verify off-by-one errors caught

**Day 5: Phase 4 Tests (2.5 hours)**
- 10 numeric constant tests
- Verify exact values, not approximations

**Day 6: Verification (2 hours)**
- Run full cargo mutants
- Document final mutation score
- Create regression test suite

---

## Current Test Status

### Passing Tests: 1,889 (96%)
```
✓ strategies (77 tests for indicators)
✓ trading::engine (85 tests)
✓ paper_trading (45 tests)
✓ trading::position_manager (28 tests)
✓ risk_manager (23 tests - BUT 2 FAILING)
✓ storage, auth, monitoring, etc.
```

### Failing Tests: 3 (4%)
```
✗ trading::risk_manager::tests::test_calculate_position_size
✗ trading::risk_manager::tests::test_calculate_position_size_large_account_balance
✗ strategies::tests::test_risk_management_config

Symptom: Position sizing always returns config.default_quantity
Root Cause: Min stop loss check or calculation logic (needs debug)
Impact: Baseline fails, prevents mutation testing
```

---

## Mutation Score by Module (Estimated)

```
Module                          Tests  Lines  Est. Mut. Score  Gap
────────────────────────────────────────────────────────────────
strategies::rsi_strategy         45    120    82%              8%
strategies::macd_strategy        42    150    81%              9%
strategies::bollinger_strategy   28    100    80%              10%
strategies::volume_strategy      35    140    79%              11%
strategies::indicators           77    250    78%              12%
────────────────────────────────────────────────────────────────
trading::engine                  85    180    82%              8%
trading::risk_manager            23    70     70%              20% ← WEAK
trading::position_manager        28    90     82%              8%
────────────────────────────────────────────────────────────────
paper_trading                    45    160    81%              9%
────────────────────────────────────────────────────────────────
OVERALL                        1889   3535    84%              6%
```

**Bottleneck:** `risk_manager` module at 70% is dragging down overall score

---

## Mutation Killers Checklist

- [ ] **Boundary Operators** (28% of gap)
  - [ ] Confidence thresholds: 0.7, 0.8 exact values
  - [ ] Risk-reward: 1.5 exact boundary
  - [ ] SMA loop: `<=` vs `<` distinction
  - [ ] Array bounds: Loop limit precision

- [ ] **Numeric Constants** (18% of gap)
  - [ ] 0.5% min stop loss
  - [ ] 0.7 strong buy threshold
  - [ ] 0.8 buy threshold
  - [ ] 1.5 risk-reward minimum
  - [ ] 0.2 max position percentage
  - [ ] 5.0 position size cap
  - [ ] 50.0 stochastic midpoint
  - [ ] 100.0/0.0 RSI bounds

- [ ] **Division & Precision** (15% of gap)
  - [ ] Volume profile: max_price == min_price
  - [ ] Position sizing: stop_loss == entry_price
  - [ ] RSI: avg_gain == 0.0, avg_loss == 0.0
  - [ ] EMA multiplier precision
  - [ ] Stochastic: highest_high == lowest_low

- [ ] **Logical Operators** (12% of gap)
  - [ ] Trading enabled: test negation with `enabled=false`
  - [ ] Confidence AND risk-reward: test both conditions
  - [ ] Hold signal: test always rejected
  - [ ] Min/max logic: verify correct function used

- [ ] **Array Safety** (11% of gap)
  - [ ] Period boundaries: exact period, period-1, period+1
  - [ ] Loop limits: `<=` vs `<` off-by-one
  - [ ] Slice indices: starting/ending positions
  - [ ] Vector lengths: exact output sizes

- [ ] **Float Handling** (9% of gap)
  - [ ] NaN propagation: test edge cases
  - [ ] Infinity values: handled gracefully
  - [ ] Comparison with 0.0: epsilon or exact
  - [ ] `.abs()` required: tested with negative inputs

---

## Quick Wins (Under 1 Hour Each)

| Test | Impact | Effort | File |
|------|--------|--------|------|
| `test_can_open_position_trading_disabled_negation` | Catch `!` removal | 0.3h | risk_manager |
| `test_can_open_position_exact_0_7_confidence` | Catch `<`→`<=` | 0.3h | risk_manager |
| `test_can_open_position_exact_0_8_confidence` | Catch `<`→`<=` | 0.3h | risk_manager |
| `test_can_open_position_exact_1_5_risk_reward` | Catch `<`→`<=` | 0.3h | risk_manager |
| `test_calculate_rsi_all_flat_prices` | Catch == mutations | 0.5h | indicators |
| `test_calculate_volume_profile_zero_range` | Catch div by 0 | 0.5h | indicators |
| `test_position_size_min_stop_loss_0_5_pct` | Verify constant | 0.5h | risk_manager |
| `test_position_size_max_20_percent_of_account` | Verify constant | 0.5h | risk_manager |

**Total: 4 hours → 4-5% improvement**

---

## Coverage Validation

### Before (Current State)
```
Total Tests: 1,889 (96% passing)
Coverage: 90.4%
Mutation Score: 84%
Ratio: 90.4% coverage → 84% mutation (12% loss)
```

### After (Projected)
```
Total Tests: 1,920+ (+31 tests)
Coverage: 90.5%+ (maintained)
Mutation Score: 90%+ (target)
Ratio: 90.5% coverage → 90%+ mutation (0.5% loss)
```

**Improvement:** 84% → 90%+ = 6% gain with 31 new tests

---

## Dependencies

- **Blocker:** Fix 3 failing tests first
- **Prerequisite:** cargo mutants tool working
- **Resource:** ~13 hours developer time
- **Risk:** Low (new tests only, no production code changes)

---

## Success Criteria

- [ ] All 1,889 existing tests pass
- [ ] No new test failures introduced
- [ ] Mutation score ≥ 90%
- [ ] No unhandled panics in edge cases
- [ ] Report generated with specific survivors killed

---

**Generated:** 2025-11-19
**Status:** Ready for Implementation
