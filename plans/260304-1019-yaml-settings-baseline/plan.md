---
title: "YAML-based settings baseline for paper trading engine"
description: "Replace DB-loaded settings with git-tracked YAML file as read-only source of truth on startup"
status: completed
priority: P1
effort: 3h
branch: feat/yaml-settings-baseline
tags: [paper-trading, settings, safety, config, rust]
created: 2026-03-04
---

# YAML-Based Settings Baseline for Paper Trading Engine

## Phases

| Phase | Name | Status | Link |
|-------|------|--------|------|
| 1 | Add serde_yml dependency | ✅ Done | [phase-1](#phase-1-add-serde_yml-dependency) |
| 2 | Create YAML defaults file | ✅ Done | [phase-2](#phase-2-create-yaml-defaults-file) |
| 3 | Add from_yaml method | ✅ Done | [phase-3](#phase-3-add-from_yaml-method-to-papertradingsettings) |

## Problem

Stale DB settings (leverage=10, SL=2%) persisted across restarts, causing all trades to lose money. The current flow:

```
Startup: load from DB (if exists) -> use defaults (if DB empty) -> trade
```

A startup migration guard was added to clamp stale per-symbol overrides, but the root problem remains: **DB is the source of truth, and DB can silently contain bad values**.

## Solution

Make a git-tracked YAML file the **read-only baseline**. On every startup, YAML always wins:

```
Startup: load from YAML -> validate -> write to DB (overwrite) -> trade
Runtime: API/self-tuning updates go to DB only (ephemeral, reset on restart)
```

This means: runtime tuning is a "session" concept. YAML is the "blessed" config.

---

## Key Design Decisions

| Decision | Choice | Rationale |
|---|---|---|
| Format | YAML (not keep TOML) | User request; YAML better for nested structures, comments, readability. Existing TOML file is unused at startup anyway. |
| Crate | `serde_yml` (not `serde_yaml`) | `serde_yaml` is deprecated/unmaintained. `serde_yml` is the active successor (same API). |
| File location | `rust-core-engine/config/paper-trading-defaults.yml` | Matches user spec. `config/` dir is new but conventional. |
| Startup behavior | YAML -> validate -> overwrite DB | Ensures DB always matches YAML on boot. Runtime tuning still works via DB. |
| Migration guard | Remove | No longer needed if YAML always overwrites DB on startup. The guard only existed to patch stale DB values. |
| Existing TOML file | Keep as reference, mark deprecated | `paper_trading_settings.toml` at project root is unused by code at startup. Keep for history, add deprecation note. |
| Symbols | Defined in YAML | Symbols section in YAML replaces hardcoded list in main.rs. User-added symbols from DB still appended. |
| Fallback | If YAML missing/invalid, FAIL HARD (don't start) | This is a finance system. Silent fallback to defaults caused the original bug. |

---

## Phase 1: Add `serde_yml` dependency

**Files to modify:**
- `rust-core-engine/Cargo.toml`

**Steps:**
1. Add `serde_yml = "0.0.12"` to `[dependencies]` (latest stable as of 2026-03)

**Todo:**
- [x] Add serde_yml dependency to Cargo.toml
- [x] Run `cargo check` to verify it compiles

---

## Phase 2: Create YAML defaults file

**Files to create:**
- `rust-core-engine/config/paper-trading-defaults.yml`

**Steps:**
1. Create `config/` directory under `rust-core-engine/`
2. Write YAML file with ALL settings matching current `Default` impl values

The YAML structure mirrors the `PaperTradingSettings` struct exactly:

```yaml
# Paper Trading Engine - Baseline Settings (Source of Truth)
# This file is git-tracked. Changes require code review.
# Runtime tuning via API/self-tuning writes to DB only (reset on restart).

basic:
  initial_balance: 10000.0
  max_positions: 5
  default_position_size_pct: 2.0
  default_leverage: 3
  trading_fee_rate: 0.0004
  funding_fee_rate: 0.0001
  slippage_pct: 0.01
  enabled: true
  auto_restart: false

risk:
  max_risk_per_trade_pct: 1.0
  max_portfolio_risk_pct: 10.0
  default_stop_loss_pct: 5.0
  default_take_profit_pct: 10.0
  max_leverage: 5
  min_margin_level: 300.0
  max_drawdown_pct: 10.0
  daily_loss_limit_pct: 3.0
  max_consecutive_losses: 3
  cool_down_minutes: 60
  position_sizing_method: RiskBased
  min_risk_reward_ratio: 2.0
  correlation_limit: 0.7
  dynamic_sizing: true
  volatility_lookback_hours: 24
  trailing_stop_enabled: true
  trailing_stop_pct: 3.0
  trailing_activation_pct: 5.0
  enable_signal_reversal: true
  ai_auto_enable_reversal: true
  reversal_min_confidence: 0.65
  reversal_max_pnl_pct: 10.0
  reversal_allowed_regimes: [trending, ranging, volatile]
  short_only_mode: false
  long_only_mode: false
  atr_stop_enabled: false
  atr_period: 14
  atr_stop_multiplier: 1.2
  atr_tp_multiplier: 2.4
  base_risk_pct: 2.0
  kelly_enabled: false
  kelly_min_trades: 200
  kelly_fraction: 0.5
  kelly_lookback: 100
  funding_spike_filter_enabled: false
  funding_spike_threshold: 0.0003
  funding_spike_reduction: 0.5
  atr_spike_filter_enabled: false
  atr_spike_multiplier: 2.0
  atr_spike_reduction: 0.5
  consecutive_loss_reduction_enabled: false
  consecutive_loss_reduction_pct: 0.3
  consecutive_loss_reduction_threshold: 3
  weekly_drawdown_limit_pct: 7.0

strategy:
  enabled_strategies:
    ai_ensemble: 1.0
  min_ai_confidence: 0.6
  combination_method: AIEnsemble
  enable_optimization: true
  optimization_period_days: 30
  min_trades_for_optimization: 50
  signal_timeout_minutes: 30
  enable_market_regime_detection: true
  regime_specific_params: {}
  backtesting:
    enabled: true
    period_days: 90
    data_resolution: "15m"
    min_trades: 20
    walk_forward_optimization: false
    out_of_sample_pct: 20.0
  market_preset: normal_volatility

symbols:
  BTCUSDT:
    enabled: true
    max_positions: 1
  ETHUSDT:
    enabled: true
    max_positions: 1
  BNBUSDT:
    enabled: true
    max_positions: 1
  SOLUSDT:
    enabled: true
    max_positions: 1

ai:
  service_url: "http://python-ai-service:8000"
  request_timeout_seconds: 30
  signal_refresh_interval_minutes: 15
  enable_realtime_signals: true
  confidence_thresholds:
    trending: 0.65
    ranging: 0.75
    volatile: 0.80
  enable_feedback_learning: true
  feedback_delay_hours: 4
  enable_strategy_recommendations: true
  track_model_performance: true

execution:
  auto_execution: true
  execution_delay_ms: 100
  simulate_partial_fills: false
  partial_fill_probability: 0.1
  order_expiration_minutes: 60
  simulate_slippage: true
  max_slippage_pct: 0.05
  simulate_market_impact: false
  market_impact_factor: 0.001
  price_update_frequency_seconds: 1

notifications:
  enable_trade_notifications: true
  enable_performance_notifications: true
  enable_risk_warnings: true
  daily_summary: true
  weekly_report: true
  channels: [WebSocket]
  min_pnl_notification: 10.0
  max_notifications_per_hour: 20

indicators:
  rsi_period: 14
  macd_fast: 12
  macd_slow: 26
  macd_signal: 9
  ema_periods: [9, 21, 50]
  bollinger_period: 20
  bollinger_std: 2.0
  volume_sma_period: 20
  stochastic_k_period: 14
  stochastic_d_period: 3

signal:
  trend_threshold_percent: 0.8
  min_required_timeframes: 3
  min_required_indicators: 4
  confidence_base: 0.5
  confidence_per_timeframe: 0.08

signal_pipeline:
  min_weighted_threshold: 60.0
  weight_15m: 0.5
  weight_30m: 1.0
  weight_1h: 2.0
  rsi_bull_threshold: 55.0
  rsi_bear_threshold: 45.0
  bb_bull_threshold: 0.3
  bb_bear_threshold: 0.7
  stoch_overbought: 80.0
  stoch_oversold: 20.0
  volume_confirm_multiplier: 1.2
  confidence_max: 0.85
  confidence_multiplier: 0.35
  counter_trend_confidence_max: 0.65
  counter_trend_multiplier: 0.20
  neutral_confidence: 0.40
  counter_trend_block_offset: 0.05
  counter_trend_enabled: true
  counter_trend_mode: block
  analysis_timeframes: ["15m", "30m", "1h"]
```

**Note on symbols:** Each symbol only specifies `enabled` and `max_positions`. All other fields (`leverage`, `stop_loss_pct`, etc.) are `Option<T>` and default to `None` — the engine already falls through to global defaults. This keeps the YAML clean.

**Todo:**
- [x] Create `rust-core-engine/config/` directory
- [x] Create `paper-trading-defaults.yml` with all settings
- [x] Verify YAML parses correctly with a quick test

---

## Phase 3: Add `from_yaml()` method to PaperTradingSettings

**Files to modify:**
- `rust-core-engine/src/paper_trading/settings.rs`

**Steps:**
1. Add `from_yaml()` method alongside existing `from_file()` (TOML):

```rust
/// Load settings from YAML baseline file (source of truth)
/// Panics-worthy if file is missing/invalid — this is a finance system.
pub fn from_yaml(path: &str) -> Result<Self> {
    let content = std::fs::read_to_string(path)
        .map_err(|e| anyhow::anyhow!("Failed to read YAML settings from {}: {}", path, e))?;
    let settings: Self = serde_yml::from_str(&content)
        .map_err(|e| anyhow::anyhow!("Failed to parse YAML settings from {}: {}", path, e))?;
    settings.validate()?;
    Ok(settings)
}
```

2. The method reads file, parses YAML, validates, and returns. Validation failure = startup abort.

**Todo:**
- [x] Add `use serde_yml;` (or keep it path-qualified)
- [x] Add `from_yaml()` method to `impl PaperTradingSettings`
- [x] Add unit test: `test_from_yaml_valid` (write temp file, parse, assert key values)
- [x] Add unit test: `test_from_yaml_invalid_fails` (bad YAML, expect error)
- [x] Add unit test: `test_from_yaml_missing_file_fails`

---

## Phase 4: Modify engine startup to use YAML baseline

**Files to modify:**
- `rust-core-engine/src/paper_trading/engine.rs` (lines ~139-226)
- `rust-core-engine/src/main.rs` (lines ~113-155)

### 4a. Change `PaperTradingEngine::new()` signature

Current signature:
```rust
pub async fn new(
    default_settings: PaperTradingSettings,
    binance_client: BinanceClient,
    ai_service: AIService,
    storage: Storage,
    event_broadcaster: broadcast::Sender<PaperTradingEvent>,
) -> Result<Self>
```

The `default_settings` param already exists. The change is in **how it's used inside**:

**Current behavior (lines 148-165):**
```rust
let mut settings = match storage.load_paper_trading_settings().await {
    Ok(Some(saved)) => saved,       // DB wins if present
    Ok(None) => default_settings,   // defaults only if DB empty
    Err(_) => default_settings,     // defaults on DB error
};
```

**New behavior:**
```rust
// YAML baseline always wins on startup — DB settings are ephemeral runtime overrides
let settings = default_settings; // Already loaded from YAML by caller

// Write baseline to DB (overwrite any stale runtime values)
if let Err(e) = storage.save_paper_trading_settings(&settings).await {
    warn!("Failed to write YAML baseline to DB: {}. Runtime tuning will be memory-only.", e);
}
info!("YAML baseline written to DB — runtime tuning resets on restart");
```

**Remove:** The entire startup migration guard block (lines 167-226) — no longer needed.

### 4b. Change `main.rs` to load from YAML

**Current (lines 113-155):**
```rust
let mut paper_trading_settings = PaperTradingSettings::default();
// ... hardcoded symbol setup ...
```

**New:**
```rust
// Load paper trading settings from YAML baseline (git-tracked source of truth)
let yaml_path = std::env::var("PAPER_TRADING_YAML")
    .unwrap_or_else(|_| "config/paper-trading-defaults.yml".to_string());
let mut paper_trading_settings = PaperTradingSettings::from_yaml(&yaml_path)
    .expect("FATAL: Cannot load paper trading YAML baseline. Fix the file and restart.");
```

**Remove:** The hardcoded symbol loop (lines 142-155) — symbols now come from YAML.

**Keep:** The user-added symbols from DB loading (lines 122-138) — these are dynamic additions:
```rust
// Append user-added symbols from DB (not in YAML baseline)
match storage.load_user_symbols().await {
    Ok(user_symbols) => {
        for symbol in user_symbols {
            if !paper_trading_settings.symbols.contains_key(&symbol) {
                info!("Loading user-added symbol: {}", symbol);
                paper_trading_settings.set_symbol_settings(
                    symbol,
                    paper_trading::settings::SymbolSettings {
                        enabled: true,
                        leverage: None,
                        position_size_pct: None,
                        stop_loss_pct: None,
                        take_profit_pct: None,
                        trading_hours: None,
                        min_price_movement_pct: None,
                        max_positions: Some(1),
                        custom_params: std::collections::HashMap::new(),
                    },
                );
            }
        }
    },
    Err(e) => info!("No user symbols found: {}", e),
}
```

**Todo:**
- [x] Modify `PaperTradingEngine::new()` to use YAML settings directly (remove DB load logic)
- [x] Remove startup migration guard block
- [x] Modify `main.rs` to call `PaperTradingSettings::from_yaml()`
- [x] Add `PAPER_TRADING_YAML` env var support with default path
- [x] Keep user-added symbols from DB append logic
- [x] Remove hardcoded symbol loop in main.rs
- [x] Run `cargo check` and `cargo test`

---

## Phase 5: Docker volume mount

**Files to modify:**
- `rust-core-engine/Dockerfile` (add COPY for config dir)
- `docker-compose-vps.yml` (add volume mount)

### 5a. Dockerfile

Add after line 53 (`COPY config.example.toml /app/config.toml`):
```dockerfile
# Copy paper trading YAML baseline
COPY config/ /app/config/
```

### 5b. docker-compose-vps.yml

Add to `rust-core-engine.volumes`:
```yaml
- ./rust-core-engine/config/paper-trading-defaults.yml:/app/config/paper-trading-defaults.yml:ro
```

The `:ro` mount means the container can't modify the file — reinforces "YAML is read-only".

**Todo:**
- [x] Update Dockerfile to COPY config/ directory
- [x] Add read-only volume mount in docker-compose-vps.yml
- [x] Verify path matches default in code (`config/paper-trading-defaults.yml`)

---

## Phase 6: Tests

**Files to modify:**
- `rust-core-engine/tests/test_paper_trading.rs` (if integration tests reference settings loading)
- `rust-core-engine/src/paper_trading/settings.rs` (unit tests in `mod tests`)

**New tests:**
1. `test_from_yaml_roundtrip` — serialize defaults to YAML, parse back, assert equality
2. `test_from_yaml_real_file` — load the actual `config/paper-trading-defaults.yml`, validate
3. `test_from_yaml_validation_catches_bad_values` — modify a parsed settings to have bad values, confirm validate() catches them
4. `test_yaml_file_matches_defaults` — load YAML, compare key fields to `Default` impl (catches drift)

**Todo:**
- [x] Add unit tests for `from_yaml()`
- [x] Add integration test that loads real YAML file
- [x] Add drift-detection test (YAML vs Default impl)
- [x] Run full test suite: `cargo test`
- [x] Verify coverage thresholds still met (95%)

---

## Phase 7: Cleanup and documentation

**Steps:**
1. Add deprecation header to `rust-core-engine/paper_trading_settings.toml`:
   ```
   # DEPRECATED: This file is no longer used at startup.
   # See config/paper-trading-defaults.yml for the active baseline.
   # Kept for historical reference only.
   ```
2. Update `CLAUDE.md` Quick Feature Location Map to mention YAML config path
3. Add comment in `from_file()` (TOML loader) marking it deprecated

**Todo:**
- [x] Mark old TOML file as deprecated
- [x] Update CLAUDE.md with YAML config reference
- [x] Mark `from_file()` as deprecated in code

---

## Runtime Flow (After Implementation)

```
STARTUP:
  1. main.rs reads PAPER_TRADING_YAML env var (default: config/paper-trading-defaults.yml)
  2. PaperTradingSettings::from_yaml() loads + validates (FATAL on failure)
  3. User-added symbols appended from DB
  4. PaperTradingEngine::new() receives validated settings
  5. Settings written to DB (overwrite stale values)
  6. Engine starts trading with YAML baseline

RUNTIME:
  7. API calls update_settings() -> writes to DB + in-memory
  8. Self-tuning adjusts parameters -> writes to DB + in-memory
  9. All runtime changes are ephemeral

RESTART:
  10. Go to step 1 (YAML baseline restored, runtime tuning lost)
```

---

## Risk Assessment

| Risk | Impact | Mitigation |
|---|---|---|
| YAML file missing in Docker | Engine won't start (by design) | Dockerfile COPYs file. Volume mount is backup. Both paths covered. |
| YAML has typo/bad values | Engine won't start (validate() catches) | CI can run `cargo test` which loads real YAML. |
| Self-tuning loses changes on restart | Expected behavior (feature, not bug) | Document clearly. If a tuned param proves good, update YAML via PR. |
| YAML drift from Default impl | Settings diverge silently | Drift-detection test catches this. |
| Enum serialization (PositionSizingMethod, etc.) | YAML `RiskBased` vs Rust enum variant | serde already handles this via derive. Verify with roundtrip test. |
| NotificationChannel enum variants with data (Email(String)) | YAML representation may be tricky | Only `WebSocket` (no data) used in defaults. Test tagged enum serialization. |

---

## Files Modified Summary

| File | Action |
|---|---|
| `rust-core-engine/Cargo.toml` | Add `serde_yml` dependency |
| `rust-core-engine/config/paper-trading-defaults.yml` | **NEW** - YAML baseline |
| `rust-core-engine/src/paper_trading/settings.rs` | Add `from_yaml()`, unit tests |
| `rust-core-engine/src/paper_trading/engine.rs` | Simplify `new()`, remove migration guard |
| `rust-core-engine/src/main.rs` | Load from YAML, remove hardcoded symbols |
| `rust-core-engine/Dockerfile` | COPY config/ directory |
| `docker-compose-vps.yml` | Add :ro volume mount |
| `rust-core-engine/paper_trading_settings.toml` | Add deprecation header |

---

## Success Criteria

- [x] Engine starts from YAML baseline (not DB)
- [x] Bad/missing YAML = engine refuses to start
- [x] Runtime tuning via API still works (writes to DB)
- [x] Restart resets to YAML baseline
- [x] All existing tests pass
- [x] Coverage >= 95%
- [x] Docker container correctly mounts YAML file
- [x] `cargo clippy` clean, `cargo fmt` clean

---

## Unresolved Questions

1. **serde_yml vs serde_yaml**: `serde_yaml` (0.9.x) is deprecated but widely used. `serde_yml` (0.0.x) is the successor but version < 1.0. Should we use `serde_yaml 0.9` for stability or `serde_yml` for futureproofing? **Recommendation: use `serde_yml`** — the API is identical and it's actively maintained.

2. **Should `from_file()` (TOML) be removed entirely?** It's unused at startup. Keeping it adds dead code. **Recommendation: deprecate with `#[deprecated]` attribute, remove in a future PR.**

3. **Should the `update_settings()` method in engine.rs log a warning that changes are ephemeral?** This would help operators understand the new behavior. **Recommendation: yes, add info log.**
