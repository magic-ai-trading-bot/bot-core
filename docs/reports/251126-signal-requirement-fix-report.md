# Signal Requirement Fix Report: 4/5 Strategy Agreement

**Date**: 2025-11-26
**Status**: COMPLETE - All components aligned with FR-STRATEGIES-006

---

## Executive Summary

This report documents the fix to align ALL signal generation systems with the **4/5 (80%) strategy agreement requirement** as specified in FR-STRATEGIES-006. Previously, the Python fallback analysis only required 2+ signals, creating a dangerous inconsistency with the Rust strategy engine.

---

## Issue Background

### Rust Strategy Engine (Correct)
- Location: `rust-core-engine/src/strategies/strategy_engine.rs:426-445`
- Config: `min_strategies_agreement = 4` (default)
- Behavior: Only generates trading signal when 4+ strategies agree

### Python AI Fallback (Was Incorrect)
- Location: `python-ai-service/main.py:1433-1447`
- Old behavior: Required only 2+ signals
- Risk: Could generate false signals when Rust would stay Neutral

---

## Fix Applied

### 1. Python Fallback Analysis (`main.py:1433-1447`)

**Before (INCORRECT):**
```python
if bullish_count >= 2:
    signal = "Long"
elif bearish_count >= 2:
    signal = "Short"
```

**After (CORRECT):**
```python
# @spec:FR-STRATEGIES-006 - Signal Combination requires ≥4/5 strategies agreement
MIN_REQUIRED_SIGNALS = 4  # Must have 4+ out of 5 indicators agree

if bullish_count >= MIN_REQUIRED_SIGNALS:
    signal = "Long"
    confidence = min(0.75, 0.50 + (bullish_count * 0.05))
elif bearish_count >= MIN_REQUIRED_SIGNALS:
    signal = "Short"
    confidence = min(0.75, 0.50 + (bearish_count * 0.05))
else:
    # Stay Neutral when consensus is weak (< 4/5 agreement)
    signal = "Neutral"
    confidence = 0.35
```

---

## Tests Updated

### Test Files Modified
- `python-ai-service/tests/test_main.py`

### Tests Changed

| Test Name | Old Behavior | New Behavior |
|-----------|--------------|--------------|
| `test_fallback_analysis_rsi_oversold` | Expected Long with 2 signals | Now provides 4 bullish signals (RSI + MACD + BB + Price trend) |
| `test_fallback_analysis_rsi_overbought` | Expected Short with 2 signals | Now provides 4 bearish signals (RSI + MACD + BB + Price trend) |
| `test_fallback_analysis_bollinger_bands_insufficient_signals` | Expected Long with 2 signals | Now expects Neutral (2 < 4 required) |
| `test_fallback_analysis_macd_bearish_neutral_signal` | Comment updated | Comment: "4/5 = 80% required" |
| `test_fallback_analysis_bb_upper_neutral` | Comment updated | Comment: "4/5 = 80% required" |

### Key Test Changes

**test_fallback_analysis_rsi_oversold**:
- Creates 100 candles with >1% price increase in last candle
- Triggers 4 bullish signals: RSI oversold + MACD bullish + BB lower + Strong upward movement
- Expects: `signal == "Long"`, `"Bullish: 4" in reasoning`

**test_fallback_analysis_bollinger_bands_insufficient_signals**:
- Provides only 2 bullish signals (BB + RSI)
- Expects: `signal == "Neutral"` (2 < 4 required)
- Validates safety-first approach

---

## Additional Fixes: Dummy Data Elimination

### Files Also Fixed in This Session

1. **`useAIAnalysis.ts`**: Replaced `generateSampleCandles()` with `fetchRealCandles()`
2. **`ml_tasks.py`**: Added `fetch_real_candles_sync()`, `fetch_current_price_sync()`
3. **`PerformanceChart.tsx`**: Now uses real `closedTrades` history
4. **`test_main.py`**: Replaced `generate_dummy_market_data` tests with `fetch_real_market_data` tests

---

## Test Results

```
119 passed, 2 failed (pre-existing issues), 5 skipped
```

**Pre-existing failures (not related to this fix):**
- `test_cost_statistics_with_usage` - Item count mismatch
- `test_periodic_analysis_symbol_level_exception` - Missing constant

**All signal-related tests: PASSING**

---

## Verification

### Signal Requirement Consistency Check

| Component | Required Signals | Status |
|-----------|-----------------|--------|
| Rust Strategy Engine | 4/5 (80%) | ✅ Correct |
| Python Fallback | 4/5 (80%) | ✅ Fixed |
| Tests | 4/5 (80%) | ✅ Updated |

### Code Tags Added/Updated

```python
# @spec:FR-STRATEGIES-006 - Signal Combination requires ≥4/5 strategies agreement
MIN_REQUIRED_SIGNALS = 4  # Must have 4+ out of 5 indicators agree
```

---

## Impact Analysis

### Safety Improvement
- **Before**: Python fallback could generate false Long/Short with only 2/5 indicator agreement
- **After**: Requires 80% consensus (4/5) before generating trading signal
- **Result**: Fewer false signals, better capital protection

### Behavior Change
- More signals will result in "Neutral" status
- Only strong consensus triggers trading actions
- Aligns with Rust engine for consistent behavior

---

## Files Modified

| File | Changes |
|------|---------|
| `python-ai-service/main.py` | Updated MIN_REQUIRED_SIGNALS from 2 to 4 |
| `python-ai-service/tests/test_main.py` | ~150 lines: Updated 5+ tests, added 3 new tests |
| `python-ai-service/tasks/ml_tasks.py` | Added real data fetching helpers |
| `nextjs-ui-dashboard/src/hooks/useAIAnalysis.ts` | Replaced dummy data with real API calls |

---

## Conclusion

The 4/5 strategy agreement requirement (FR-STRATEGIES-006) is now consistently enforced across:
- Rust Strategy Engine (primary)
- Python AI Fallback (fixed)
- All related tests (updated)

**This fix is CRITICAL for a trading system where inconsistent signals = potential money loss.**

---

**Report Generated**: 2025-11-26
**Author**: Claude Code AI
**Spec Reference**: FR-STRATEGIES-006
