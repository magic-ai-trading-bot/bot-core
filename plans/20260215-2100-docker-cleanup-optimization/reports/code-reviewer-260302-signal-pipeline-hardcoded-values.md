# Code Review: Hardcoded Signal Pipeline Values

**Date**: 2026-03-02
**Scope**: Rust codebase audit for hardcoded values that should reference `SignalPipelineSettings`
**Files reviewed**: `engine.rs`, `strategies/` (all files), `api/paper_trading.rs`

---

## Code Review Summary

### Scope
- Files reviewed: 11 files
  - `rust-core-engine/src/paper_trading/engine.rs` (4,096 production lines)
  - `rust-core-engine/src/paper_trading/settings.rs`
  - `rust-core-engine/src/api/paper_trading.rs` (2,346 production lines)
  - `rust-core-engine/src/strategies/strategy_engine.rs` (633 production lines)
  - `rust-core-engine/src/strategies/rsi_strategy.rs` (327 production lines)
  - `rust-core-engine/src/strategies/bollinger_strategy.rs` (376 production lines)
  - `rust-core-engine/src/strategies/stochastic_strategy.rs` (381 production lines)
  - `rust-core-engine/src/strategies/volume_strategy.rs` (315 production lines)
  - `rust-core-engine/src/strategies/macd_strategy.rs` (365 production lines)
  - `rust-core-engine/src/strategies/trend_filter.rs` (189 production lines)
  - `rust-core-engine/src/strategies/hybrid_filter.rs` (280 production lines)
- Lines of code analyzed: ~9,000 production lines
- Review focus: Hardcoded values matching `SignalPipelineSettings` fields

### Overall Assessment

The signal pipeline architecture is **correctly split**: Python (`main.py`) runs the full weighted-voting pipeline and correctly reads all `SignalPipelineSettings` values via `get_pipeline_*()` accessors. The Rust strategies are independent indicator-based strategies that do NOT run the same pipeline — they use their own internal logic. **`SignalPipelineSettings` is correctly exposed via API and stored/validated in `settings.rs`, but never consumed by Rust strategies.** This is the intended architecture.

The hardcoded values found in Rust strategy files are **not violations** of `SignalPipelineSettings` because:
1. Rust strategies are separate sub-engines (RSI, BB, MACD, etc.) with their own configurable thresholds via `StrategyConfig.parameters` (JSON map)
2. The Python pipeline thresholds (`rsi_bull_threshold`, `bb_bull_threshold`, etc.) control how the Python AI service classifies indicators before weighted voting — different from what Rust strategies use
3. The `SignalPipelineSettings` struct exists to pass pipeline config from Rust settings to Python, not to configure Rust strategy internals

However, several findings warrant attention:

---

## Critical Issues

None.

---

## High Priority Findings

### H1 — `engine.rs` line 374 and 739: Hardcoded timeframes ignore `analysis_timeframes`

**File**: `rust-core-engine/src/paper_trading/engine.rs`

```rust
// Line 374 — start_strategy_signal_loop, cache update loop
for timeframe in &["5m", "15m"] {

// Line 739 — build_strategy_input
for timeframe in &["5m", "15m", "1h"] {

// Lines 1449, 1550 — warmup / preload
const REQUIRED_TIMEFRAMES: &[&str] = &["5m", "15m"];
const REQUIRED_TIMEFRAMES: &[&str] = &["5m", "15m", "1h"];
```

**Problem**: `SignalPipelineSettings.analysis_timeframes` (default: `["15m", "30m", "1h"]`) is stored and exposed via API, but the Rust engine ignores it and uses hardcoded timeframe lists instead. If a user updates `analysis_timeframes` via the settings API, the Rust cache update loop and strategy input builder won't reflect the change.

**Impact**: Changing `analysis_timeframes` via the settings API has no effect on what candle data Rust fetches. The Python pipeline side reads `get_pipeline_analysis_timeframes()` correctly, but the Rust side still hardcodes `["5m", "15m"]` and `["5m", "15m", "1h"]`. This breaks the intended configurability.

**Fix**: Read `analysis_timeframes` from settings and merge with strategy-required timeframes:
```rust
let settings = engine.settings.read().await;
let pipeline_tfs = settings.signal_pipeline.analysis_timeframes.clone();
// Merge with strategy-required: always include "5m" for Rust strategies
let mut timeframes = vec!["5m".to_string()];
timeframes.extend(pipeline_tfs);
timeframes.dedup();
drop(settings);

for timeframe in &timeframes {
    // ...fetch klines...
}
```

---

### H2 — `engine.rs` line 2175: AI reversal decision uses hardcoded thresholds

**File**: `rust-core-engine/src/paper_trading/engine.rs`

```rust
// Line 2175
let conditions_met = ai_accuracy >= 0.65 && win_rate >= 0.55 && consecutive.wins >= 3
    || (consecutive.losses == 0 && win_rate >= 0.60) && volatility < 0.6;
```

**Problem**: `0.65` exactly matches `SignalPipelineSettings.counter_trend_confidence_max` (default 0.65). The `0.55` and `0.60` win rate thresholds and `0.6` volatility cap are unrelated to `SignalPipelineSettings` but are completely non-configurable.

**Assessment**: The `0.65` here is used as an AI accuracy threshold for enabling counter-trend reversals — semantically different from `counter_trend_confidence_max` (which caps signal confidence, not accuracy). These thresholds belong to a new settings group (e.g., `RiskSettings` or a new `AIDecisionSettings`). Not a `SignalPipelineSettings` issue, but they are genuinely non-configurable thresholds that could affect trading decisions.

**Recommendation**: Extract to a dedicated settings field, e.g.:
```rust
settings.ai.min_accuracy_for_reversal  // currently 0.65
settings.ai.min_win_rate_for_reversal  // currently 0.55 / 0.60
```

---

### H3 — `strategy_engine.rs` line 612: `min_confidence_threshold: 0.65` is a separate (non-pipeline) hardcoded default

**File**: `rust-core-engine/src/strategies/strategy_engine.rs`

```rust
// Line 612 — StrategyEngineConfig::default()
min_confidence_threshold: 0.65,
```

**Problem**: This controls which strategy results are accepted into the consensus (Rust strategies only). It is `0.65` — same value as `counter_trend_confidence_max` — but semantically independent. It is configurable at runtime only if a `StrategyEngineConfig` is injected; the `Default` impl hardcodes it. There is no `PaperTradingSettings` field that maps to this threshold, meaning changes to `signal_pipeline.counter_trend_confidence_max` won't affect it.

**Assessment**: Not a `SignalPipelineSettings` concern, but it should be wired to `PaperTradingSettings` for consistency.

---

## Medium Priority Improvements

### M1 — `rsi_strategy.rs` lines 280, 294: `55.0` / `45.0` RSI thresholds hardcoded

**File**: `rust-core-engine/src/strategies/rsi_strategy.rs`

```rust
// Line 280 — trend-following bullish signal
if rsi_1h > 55.0

// Line 294 — trend-following bearish signal
if rsi_1h < 45.0
```

**Assessment**: These exactly match `SignalPipelineSettings.rsi_bull_threshold` (55.0) and `rsi_bear_threshold` (45.0). However, `rsi_strategy.rs` uses its own `StrategyConfig.parameters` map (oversold=25, overbought=75), and these `55.0`/`45.0` values are for a separate "momentum trend-following" band — not the extreme oversold/overbought thresholds. They are architecturally similar to `signal_pipeline.rsi_bull_threshold`/`rsi_bear_threshold` but live in a different layer.

**Verdict**: True hardcoded values that semantically match `rsi_bull_threshold`/`rsi_bear_threshold`. If a user changes those via the API, these Rust thresholds remain 55/45. Consider pulling from `SignalPipelineSettings` or adding to `StrategyConfig.parameters`.

Line 309: `0.65` confidence return for neutral RSI — this is a strategy output confidence, not a pipeline threshold. Low risk.

---

### M2 — `bollinger_strategy.rs` lines 311, 319, 328, 336: `0.65` / `0.35` BB position thresholds

**File**: `rust-core-engine/src/strategies/bollinger_strategy.rs`

```rust
// Line 311
if bb_position_1h > 0.65 && bb_position_4h > 0.55 && bb_expanding_1h {

// Line 319
if bb_position_1h < 0.35 && bb_position_4h < 0.45 && bb_expanding_1h {

// Line 328
if bb_position_1h < 0.35 && current_price > middle_4h {

// Line 336
if bb_position_1h > 0.65 && current_price < middle_4h {
```

**Assessment**: `0.65`/`0.35` exactly match `SignalPipelineSettings.bb_bear_threshold` (0.7) and `bb_bull_threshold` (0.3), though these Rust values are slightly looser (0.65/0.35 vs 0.7/0.3). These are Bollinger Band position thresholds for trend-continuation signals in the Rust Bollinger strategy — semantically the same concept as `bb_bull_threshold`/`bb_bear_threshold`. If a user changes those pipeline settings, these Rust thresholds remain fixed.

**Verdict**: True match with `bb_bull_threshold`/`bb_bear_threshold` semantics but different values (0.35/0.65 vs 0.3/0.7). The discrepancy itself is a potential consistency bug — the Python pipeline uses 0.3/0.7 while Rust uses 0.35/0.65.

---

### M3 — `stochastic_strategy.rs` lines 69/77: `unwrap_or(20.0)` / `unwrap_or(80.0)` fallback defaults

**File**: `rust-core-engine/src/strategies/stochastic_strategy.rs`

```rust
// Line 69
.unwrap_or(20.0)  // oversold_threshold fallback

// Line 77
.unwrap_or(80.0)  // overbought_threshold fallback
```

**Assessment**: These match `SignalPipelineSettings.stoch_oversold` (20.0) and `stoch_overbought` (80.0) by value. However, these are fallbacks for the `StrategyConfig.parameters` JSON map — if the key is missing, fall back to 20/80. The Rust stochastic strategy has its own parameter system distinct from `SignalPipelineSettings`. Low risk since the values come from config; the `unwrap_or` is just a safety fallback.

Lines 284, 302: `0.85` confidence for "extreme oversold/overbought with momentum" signals — these are output confidence values for extreme conditions, not pipeline threshold settings.

---

### M4 — `volume_strategy.rs` lines 258, 269, 279, 287: Hardcoded ratio thresholds

**File**: `rust-core-engine/src/strategies/volume_strategy.rs`

```rust
// Line 258
|| (near_poc && bullish_volume_ratio >= 0.65 && above_poc)

// Line 269
|| (near_poc && bullish_volume_ratio <= 0.35 && !above_poc)

// Line 279
if bullish_volume_ratio >= 0.55 && volume_ratio > 1.2 {

// Line 287
if bullish_volume_ratio <= 0.45 && volume_ratio > 1.2 {
```

**Assessment**: `1.2` matches `SignalPipelineSettings.volume_confirm_multiplier` (1.2). The volume ratio comparison `volume_ratio > 1.2` at lines 279 and 287 is semantically identical to the pipeline's `volume_confirm_multiplier`. If a user changes `volume_confirm_multiplier` via the API, this Rust check remains at `1.2`.

`0.65`/`0.35` are bullish/bearish volume ratio thresholds — not directly mapped to any `SignalPipelineSettings` field (different concept from `bb_bull/bear_threshold`).

**Verdict**: Line 279 and 287 `> 1.2` is a genuine duplicate of `volume_confirm_multiplier`.

---

### M5 — `hybrid_filter.rs` lines 158, 216: `adjusted_confidence *= 0.85` hardcoded multiplier

**File**: `rust-core-engine/src/strategies/hybrid_filter.rs`

```rust
// Line 158 — ML neutral signal penalty for LONG
adjusted_confidence *= 0.85;

// Line 216 — ML neutral signal penalty for SHORT
adjusted_confidence *= 0.85;
```

**Assessment**: `0.85` appears as a confidence reduction factor when ML is neutral. This is not a `SignalPipelineSettings` field — it's a `HybridFilter`-specific penalty. The `confidence_max: 0.85` in `SignalPipelineSettings` is a cap, not a multiplier. Different semantics. Not a `SignalPipelineSettings` concern but still non-configurable.

---

### M6 — `trend_filter.rs` lines 163, 169: Alignment scores `0.85` / `0.65`

**File**: `rust-core-engine/src/strategies/trend_filter.rs`

```rust
// Line 163 — Strong alignment (Daily + 4H agree)
| (TrendDirection::Downtrend, TrendDirection::Downtrend, _) => 0.85,

// Line 169 — Moderate alignment
| (_, TrendDirection::Downtrend, TrendDirection::Downtrend) => 0.65,
```

**Assessment**: These are trend alignment score constants in a match expression — they determine how strongly timeframe trends agree. `0.85` and `0.65` coincidentally match `confidence_max` and `counter_trend_confidence_max` by value but are semantically unrelated (trend alignment scores, not confidence caps or multipliers). Not a `SignalPipelineSettings` concern.

---

## Low Priority Suggestions

### L1 — `api/paper_trading.rs` lines 1128-1155: Strategy config RSI/Stoch thresholds hardcoded in API response

**File**: `rust-core-engine/src/api/paper_trading.rs`

```rust
// Lines 1128-1129 — get_strategy_settings API handler
extreme_oversold: 20.0,
extreme_overbought: 80.0,

// Lines 1154-1155
oversold_threshold: 20.0,
overbought_threshold: 80.0,
```

**Problem**: The `get_strategy_settings` API handler hardcodes RSI and Stochastic thresholds instead of reading from `engine_settings`. The handler reads `engine_settings` for risk settings but ignores strategy-specific config. These values never reflect actual runtime configuration.

**Fix**: Read from `engine_settings.indicators` or the actual `StrategyConfig` if accessible.

---

### L2 — `python/main.py` line 1292: `volume_ratio > 1.2` in a non-pipeline function

**File**: `python-ai-service/main.py`

```python
# Line 1292 — in a context-building function (not the classify_timeframe pipeline)
if volume_ratio > 1.2:
    context["volume_trend"] = "increasing"
```

**Assessment**: This `1.2` is in a different function (`_build_market_context` or similar) from the pipeline — it does NOT use `get_pipeline_volume_confirm_multiplier()`. Semantically equivalent to `volume_confirm_multiplier` but in a separate code path. Should call `get_pipeline_volume_confirm_multiplier()` for consistency.

---

### L3 — `python/main.py` line 2463: `confidence * 0.85` in GPT fallback parser

**File**: `python-ai-service/main.py`

```python
# Line 2463 — in GPT response text parser fallback (not the main pipeline)
"Stochastic Strategy": confidence * 0.85,
```

**Assessment**: This is in a GPT text-response fallback parser that creates mock `strategy_scores`. The `0.85` is an arbitrary discount factor, not related to `confidence_max`. Not a `SignalPipelineSettings` concern. Low impact since this is a fallback only used when GPT returns unparseable text.

---

### L4 — `python/main.py` lines 3251, 0.20 as MACD weight in ML direction function

**File**: `python-ai-service/main.py`

```python
# Lines 3248-3257 — _compute_direction_score function
direction = (
    0.25 * ema_score
    + 0.20 * macd_score   # ← matches counter_trend_multiplier by coincidence
    ...
)
```

**Assessment**: `0.20` is a component weight in an internal ML scoring function — not related to `counter_trend_multiplier`. The `_compute_direction_score` function is a separate internal utility for ML model input preparation, not the signal pipeline. No fix needed.

---

## Positive Observations

1. **Python pipeline is fully parameterized**: All 15 `SignalPipelineSettings` fields are correctly read via `get_pipeline_*()` accessor functions in `main.py`. The main signal pipeline (`_classify_timeframe`, weighted voting, confidence scoring, counter-trend logic) uses zero hardcoded values.

2. **API layer is complete**: `api/paper_trading.rs` correctly maps all `SignalPipelineSettings` fields to the API response (lines 1970-1989) and correctly writes all fields on update (lines 2046-2067). No fields are missing.

3. **Settings validation is thorough**: `settings.rs` validates all bounds for `SignalPipelineSettings` including threshold ordering, ranges, and valid counter-trend mode values.

4. **Architecture separation is intentional**: Rust strategies (RSI, BB, MACD, etc.) and the Python signal pipeline are parallel subsystems. The Python pipeline consumes raw Binance data; Rust strategies consume cached candle data. They use different threshold systems by design.

5. **Test-only hardcoded values are numerous but harmless**: The many `0.85`, `0.65`, `confidence: 0.85` values in test code are test fixtures, not production configuration.

---

## Recommended Actions

1. **[High] Fix `engine.rs` timeframe lists to use `analysis_timeframes` from settings** (lines 374, 739, 1449, 1550):
   - In `start_strategy_signal_loop`: merge `settings.signal_pipeline.analysis_timeframes` with `["5m"]` (always required for Rust strategies)
   - In `build_strategy_input`: same merge
   - In `preload_historical_data`: same merge

2. **[Medium] Fix `rsi_strategy.rs` lines 280/294** to read `55.0`/`45.0` from `SignalPipelineSettings` or add `rsi_bull_threshold`/`rsi_bear_threshold` to `StrategyConfig.parameters`. Currently these Rust thresholds are out of sync with the Python pipeline thresholds.

3. **[Medium] Fix `volume_strategy.rs` lines 279/287** to read `1.2` from `SignalPipelineSettings.volume_confirm_multiplier`. This is the clearest case of a duplicated constant — it's exactly the same semantic check as the Python pipeline.

4. **[Medium] Fix `bollinger_strategy.rs` lines 311/319/328/336** — either align `0.65`/`0.35` with `bb_bear_threshold`/`bb_bull_threshold` (0.7/0.3 from settings) or document the intentional discrepancy (Rust uses a looser band for trend-continuation vs. Python's stricter reversal thresholds).

5. **[Medium] Fix `python/main.py` line 1292** — replace `1.2` with `get_pipeline_volume_confirm_multiplier()`.

6. **[Low] Fix `api/paper_trading.rs` lines 1128-1155** — `get_strategy_settings` should read actual indicator config from settings, not hardcode extreme_oversold=20.0, overbought=80.0 etc.

7. **[Low] Wire `strategy_engine.rs:612` `min_confidence_threshold: 0.65` to `PaperTradingSettings`** so it can be configured.

---

## Definitive Hardcoded Values That Should Reference `SignalPipelineSettings`

| File | Line | Hardcoded Value | Should Reference |
|------|------|-----------------|-----------------|
| `engine.rs` | 374 | `&["5m", "15m"]` (loop) | `settings.signal_pipeline.analysis_timeframes` (+ "5m") |
| `engine.rs` | 739 | `&["5m", "15m", "1h"]` (build input) | `settings.signal_pipeline.analysis_timeframes` (+ "5m") |
| `engine.rs` | 1449 | `&["5m", "15m"]` (warmup check) | `settings.signal_pipeline.analysis_timeframes` (+ "5m") |
| `engine.rs` | 1550 | `&["5m", "15m", "1h"]` (preload) | `settings.signal_pipeline.analysis_timeframes` (+ "5m") |
| `rsi_strategy.rs` | 280 | `55.0` (RSI bull momentum) | `settings.signal_pipeline.rsi_bull_threshold` |
| `rsi_strategy.rs` | 294 | `45.0` (RSI bear momentum) | `settings.signal_pipeline.rsi_bear_threshold` |
| `bollinger_strategy.rs` | 311, 336 | `0.65` (BB position upper) | `settings.signal_pipeline.bb_bear_threshold` |
| `bollinger_strategy.rs` | 319, 328 | `0.35` (BB position lower) | `settings.signal_pipeline.bb_bull_threshold` |
| `volume_strategy.rs` | 279, 287 | `1.2` (volume_ratio threshold) | `settings.signal_pipeline.volume_confirm_multiplier` |
| `python-ai-service/main.py` | 1292 | `1.2` (volume_ratio trend check) | `get_pipeline_volume_confirm_multiplier()` |

## Not `SignalPipelineSettings` Violations (Clarification)

| File | Line | Value | Reason Not a Violation |
|------|------|-------|----------------------|
| `engine.rs` | 2175 | `0.65` | AI accuracy threshold for reversal, not pipeline confidence cap |
| `strategy_engine.rs` | 612 | `0.65` | Strategy filter threshold, separate concern from pipeline |
| `stochastic_strategy.rs` | 69/77 | `20.0`/`80.0` | `unwrap_or` fallbacks for own StrategyConfig parameters |
| `stochastic_strategy.rs` | 284/302 | `0.85` | Output confidence for extreme conditions, not pipeline setting |
| `trend_filter.rs` | 163/169 | `0.85`/`0.65` | Alignment score constants, semantically unrelated |
| `hybrid_filter.rs` | 158/216 | `0.85` | ML neutral penalty multiplier, not pipeline setting |
| `macd_strategy.rs` | 347 | `0.65` | Neutral consolidation confidence output |
| `volume_strategy.rs` | 258/269 | `0.65`/`0.35` | Volume ratio thresholds (bullish_volume_ratio, not bb position) |
| `api/paper_trading.rs` | 1128-1155 | `20.0`/`80.0` | Strategy indicator config, not signal_pipeline (see L1) |
| `python/main.py` | 2463 | `0.85` | GPT fallback discount factor |
| `python/main.py` | 3251 | `0.20` | ML scoring component weight |

---

## Metrics
- Production files reviewed: 11
- True `SignalPipelineSettings` violations found: **10** (4 timeframe list sites + 2 RSI + 2 BB + 2 volume)
- Values correctly using `SignalPipelineSettings`: All 15 fields in Python pipeline
- Values that coincidentally match but are semantically distinct: 9
- Test-only false positives filtered: ~40+ occurrences
