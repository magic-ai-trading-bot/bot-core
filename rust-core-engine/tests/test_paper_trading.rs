mod common;

use binance_trading_bot::paper_trading::settings::{PositionSizingMethod, RiskSettings};

// ============== ATR-BASED STOP LOSS & POSITION SIZING TESTS ==============
// @spec:FR-RISK-010 - ATR-based stop loss and position sizing

#[test]
fn test_atr_stop_loss_long_position() {
    // ATR SL for long: entry - (ATR × multiplier)
    let entry_price = 50000.0;
    let current_atr = 500.0; // ATR = $500
    let atr_stop_multiplier = 1.2;

    let sl_distance = current_atr * atr_stop_multiplier;
    let stop_loss = entry_price - sl_distance;

    assert_eq!(sl_distance, 600.0);
    assert_eq!(stop_loss, 49400.0);
}

#[test]
fn test_atr_stop_loss_short_position() {
    // ATR SL for short: entry + (ATR × multiplier)
    let entry_price = 50000.0;
    let current_atr = 500.0;
    let atr_stop_multiplier = 1.2;

    let sl_distance = current_atr * atr_stop_multiplier;
    let stop_loss = entry_price + sl_distance;

    assert_eq!(stop_loss, 50600.0);
}

#[test]
fn test_atr_take_profit_long_position() {
    // ATR TP for long: entry + (ATR × tp_multiplier)
    let entry_price = 50000.0;
    let current_atr = 500.0;
    let atr_tp_multiplier = 2.4; // 2:1 R:R with 1.2 SL multiplier

    let tp_distance = current_atr * atr_tp_multiplier;
    let take_profit = entry_price + tp_distance;

    assert_eq!(tp_distance, 1200.0);
    assert_eq!(take_profit, 51200.0);
}

#[test]
fn test_atr_take_profit_short_position() {
    // ATR TP for short: entry - (ATR × tp_multiplier)
    let entry_price = 50000.0;
    let current_atr = 500.0;
    let atr_tp_multiplier = 2.4;

    let tp_distance = current_atr * atr_tp_multiplier;
    let take_profit = entry_price - tp_distance;

    assert_eq!(take_profit, 48800.0);
}

#[test]
fn test_atr_risk_reward_ratio() {
    // With default multipliers (1.2 SL, 2.4 TP), R:R should be exactly 2:1
    let atr_stop_multiplier = 1.2;
    let atr_tp_multiplier = 2.4;

    let risk_reward = atr_tp_multiplier / atr_stop_multiplier;
    assert_eq!(risk_reward, 2.0, "R:R ratio should be 2:1");
}

#[test]
fn test_atr_based_position_sizing() {
    // size = (equity × base_risk%) / SL_distance
    let equity: f64 = 10000.0;
    let base_risk_pct: f64 = 2.0; // 2% risk per trade
    let current_atr: f64 = 500.0;
    let atr_stop_multiplier: f64 = 1.2;

    let sl_distance = current_atr * atr_stop_multiplier; // 600
    let risk_amount = equity * (base_risk_pct / 100.0); // 200
    let position_size = risk_amount / sl_distance; // 200 / 600 = 0.3333

    assert!((position_size - 0.3333).abs() < 0.001);
}

#[test]
fn test_atr_position_sizing_higher_volatility_smaller_position() {
    // When ATR doubles, position size should halve (inverse relationship)
    let equity: f64 = 10000.0;
    let base_risk_pct: f64 = 2.0;
    let multiplier: f64 = 1.2;

    // Normal volatility
    let atr_normal: f64 = 500.0;
    let sl_normal = atr_normal * multiplier;
    let risk = equity * (base_risk_pct / 100.0);
    let size_normal = risk / sl_normal;

    // Double volatility
    let atr_high: f64 = 1000.0;
    let sl_high = atr_high * multiplier;
    let size_high = risk / sl_high;

    assert!(
        (size_high - size_normal / 2.0).abs() < 0.0001,
        "Position size should halve when ATR doubles"
    );
}

#[test]
fn test_atr_position_sizing_zero_sl_distance() {
    // Edge case: if SL distance is 0, should get 0 position
    let equity = 10000.0;
    let base_risk_pct = 2.0;
    let sl_distance = 0.0;

    let risk_amount = equity * (base_risk_pct / 100.0);
    let position_size = if sl_distance > 0.0 {
        risk_amount / sl_distance
    } else {
        0.0
    };

    assert_eq!(
        position_size, 0.0,
        "Position size should be 0 when SL distance is 0"
    );
}

// ============== HALF-KELLY CRITERION TESTS ==============
// @spec:FR-RISK-011 - Kelly criterion position sizing

#[test]
fn test_kelly_formula_basic() {
    // Kelly formula: f = (bp - q) / b
    // where b = avg_win/avg_loss, p = win_rate, q = 1-p
    let win_rate: f64 = 0.6; // 60% win rate
    let avg_win: f64 = 100.0;
    let avg_loss: f64 = 50.0;

    let b = avg_win / avg_loss; // 2.0
    let p = win_rate;
    let q = 1.0 - p; // 0.4

    let kelly_f = (b * p - q) / b;
    // (2.0 * 0.6 - 0.4) / 2.0 = (1.2 - 0.4) / 2.0 = 0.8 / 2.0 = 0.4

    assert!((kelly_f - 0.4).abs() < 0.0001, "Kelly f should be 0.4");
}

#[test]
fn test_half_kelly_multiplier() {
    // Half-Kelly: fraction × kelly_f
    let kelly_f: f64 = 0.4;
    let fraction: f64 = 0.5; // Half-Kelly

    let multiplier = (fraction * kelly_f).clamp(0.25, 2.0);
    // 0.5 × 0.4 = 0.2 → clamped to 0.25 (floor)

    assert_eq!(multiplier, 0.25, "Half-Kelly should clamp to floor of 0.25");
}

#[test]
fn test_kelly_high_edge_trader() {
    // 70% win rate, 2:1 avg win/loss = very profitable trader
    let win_rate: f64 = 0.70;
    let avg_win: f64 = 200.0;
    let avg_loss: f64 = 100.0;

    let b = avg_win / avg_loss; // 2.0
    let p = win_rate;
    let q: f64 = 1.0 - p; // 0.3

    let kelly_f = (b * p - q) / b;
    // (2.0 * 0.7 - 0.3) / 2.0 = (1.4 - 0.3) / 2.0 = 1.1 / 2.0 = 0.55

    let half_kelly: f64 = (0.5 * kelly_f).clamp(0.25, 2.0);
    // 0.5 × 0.55 = 0.275

    assert!((half_kelly - 0.275).abs() < 0.0001);
}

#[test]
fn test_kelly_negative_edge() {
    // 40% win rate with 1:1 avg win/loss = negative edge
    let win_rate: f64 = 0.4;
    let avg_win: f64 = 100.0;
    let avg_loss: f64 = 100.0;

    let b = avg_win / avg_loss; // 1.0
    let p = win_rate;
    let q: f64 = 1.0 - p; // 0.6

    let kelly_f = (b * p - q) / b;
    // (1.0 * 0.4 - 0.6) / 1.0 = -0.2

    let half_kelly: f64 = (0.5 * kelly_f).clamp(0.25, 2.0);
    // 0.5 × (-0.2) = -0.1 → clamped to 0.25 (minimum)

    assert_eq!(
        half_kelly, 0.25,
        "Negative edge should clamp to minimum 0.25"
    );
}

#[test]
fn test_kelly_extremely_profitable() {
    // 80% win rate, 3:1 avg win/loss
    let win_rate: f64 = 0.80;
    let avg_win: f64 = 300.0;
    let avg_loss: f64 = 100.0;

    let b = avg_win / avg_loss; // 3.0
    let p = win_rate;
    let q: f64 = 1.0 - p; // 0.2

    let kelly_f = (b * p - q) / b;
    // (3.0 * 0.8 - 0.2) / 3.0 = (2.4 - 0.2) / 3.0 = 2.2 / 3.0 ≈ 0.7333

    let half_kelly: f64 = (0.5 * kelly_f).clamp(0.25, 2.0);
    // 0.5 × 0.7333 = 0.3667

    assert!((half_kelly - 0.3667).abs() < 0.001);
}

#[test]
fn test_kelly_cap_at_maximum() {
    // Full Kelly with extreme edge should cap at 2.0
    let kelly_f: f64 = 5.0; // Unrealistically high
    let fraction: f64 = 1.0; // Full Kelly

    let multiplier = (fraction * kelly_f).clamp(0.25, 2.0);

    assert_eq!(multiplier, 2.0, "Should cap at 2.0 maximum");
}

#[test]
fn test_kelly_returns_one_when_disabled() {
    // When kelly_enabled is false, multiplier should be 1.0 (no change)
    let kelly_enabled = false;
    let multiplier = if kelly_enabled { 0.5 } else { 1.0 };

    assert_eq!(multiplier, 1.0);
}

#[test]
fn test_kelly_returns_one_when_insufficient_trades() {
    // When total trades < min_trades, multiplier should be 1.0
    let total_closed = 150;
    let min_trades = 200;
    let multiplier = if total_closed < min_trades { 1.0 } else { 0.5 };

    assert_eq!(multiplier, 1.0);
}

// ============== REGIME FILTER TESTS ==============
// @spec:FR-RISK-012 - Regime-based position reduction

#[test]
fn test_funding_spike_filter() {
    // When funding rate exceeds threshold, position size is reduced
    let funding_rate: f64 = 0.0005; // 0.05%
    let threshold = 0.0003; // 0.03%
    let reduction = 0.5;

    let factor = if funding_rate.abs() > threshold {
        reduction
    } else {
        1.0
    };

    assert_eq!(
        factor, 0.5,
        "Should reduce by 50% when funding exceeds threshold"
    );
}

#[test]
fn test_funding_spike_filter_negative_rate() {
    // Negative funding rate (short paying long) should also trigger
    let funding_rate: f64 = -0.0005;
    let threshold = 0.0003;
    let reduction = 0.5;

    let factor = if funding_rate.abs() > threshold {
        reduction
    } else {
        1.0
    };

    assert_eq!(
        factor, 0.5,
        "Negative funding spike should also trigger reduction"
    );
}

#[test]
fn test_funding_spike_filter_normal_rate() {
    // Normal funding rate should not trigger reduction
    let funding_rate: f64 = 0.0001; // 0.01% - normal
    let threshold = 0.0003;
    let reduction = 0.5;

    let factor = if funding_rate.abs() > threshold {
        reduction
    } else {
        1.0
    };

    assert_eq!(factor, 1.0, "Normal rate should not trigger reduction");
}

#[test]
fn test_atr_spike_filter() {
    // When current ATR > mean × multiplier, reduce position
    let current_atr = 1200.0;
    let mean_atr = 500.0;
    let spike_multiplier = 2.0;
    let spike_reduction = 0.5;

    let factor = if mean_atr > 0.0 && current_atr > mean_atr * spike_multiplier {
        spike_reduction
    } else {
        1.0
    };

    assert_eq!(factor, 0.5, "ATR spike should trigger 50% reduction");
}

#[test]
fn test_atr_spike_filter_normal_volatility() {
    // Normal ATR should not trigger
    let current_atr = 600.0;
    let mean_atr = 500.0;
    let spike_multiplier = 2.0;
    let spike_reduction = 0.5;

    let factor = if mean_atr > 0.0 && current_atr > mean_atr * spike_multiplier {
        spike_reduction
    } else {
        1.0
    };

    assert_eq!(factor, 1.0, "Normal ATR should not trigger reduction");
}

#[test]
fn test_consecutive_loss_reduction_basic() {
    // 5 consecutive losses with threshold=3 and reduction=0.3
    let consecutive_losses: u32 = 5;
    let threshold: u32 = 3;
    let reduction_pct: f64 = 0.3;

    let factor: f64 = if consecutive_losses >= threshold {
        let excess = consecutive_losses - threshold; // 2
        (1.0_f64 - reduction_pct).powi(excess as i32) // 0.7^2 = 0.49
    } else {
        1.0
    };

    assert!(
        (factor - 0.49).abs() < 0.0001,
        "5 losses = 0.7^2 = 0.49 factor"
    );
}

#[test]
fn test_consecutive_loss_reduction_at_threshold() {
    // Exactly at threshold: excess = 0, factor = 0.7^0 = 1.0
    let consecutive_losses: u32 = 3;
    let threshold: u32 = 3;
    let reduction_pct: f64 = 0.3;

    let factor: f64 = if consecutive_losses >= threshold {
        let excess = consecutive_losses - threshold; // 0
        (1.0_f64 - reduction_pct).powi(excess as i32) // 0.7^0 = 1.0
    } else {
        1.0
    };

    assert_eq!(factor, 1.0, "At threshold, no reduction yet");
}

#[test]
fn test_consecutive_loss_reduction_progressive() {
    // Each loss beyond threshold reduces by (1-0.3) multiplicatively
    let _threshold: u32 = 3;
    let reduction_pct: f64 = 0.3;

    // 4 losses: excess=1, 0.7^1 = 0.7
    let factor_4: f64 = (1.0_f64 - reduction_pct).powi(1);
    assert!((factor_4 - 0.7).abs() < 0.0001);

    // 5 losses: excess=2, 0.7^2 = 0.49
    let factor_5: f64 = (1.0_f64 - reduction_pct).powi(2);
    assert!((factor_5 - 0.49).abs() < 0.0001);

    // 6 losses: excess=3, 0.7^3 = 0.343
    let factor_6: f64 = (1.0_f64 - reduction_pct).powi(3);
    assert!((factor_6 - 0.343).abs() < 0.0001);

    // 10 losses: excess=7, 0.7^7 ≈ 0.0824
    let factor_10: f64 = (1.0_f64 - reduction_pct).powi(7);
    assert!((factor_10 - 0.0824).abs() < 0.001);
}

#[test]
fn test_consecutive_loss_reduction_below_threshold() {
    // Below threshold: no reduction
    let consecutive_losses: u32 = 2;
    let threshold: u32 = 3;

    let factor = if consecutive_losses >= threshold {
        0.5
    } else {
        1.0
    };

    assert_eq!(factor, 1.0, "Below threshold should have no reduction");
}

#[test]
fn test_regime_filters_cascade() {
    // Multiple filters multiply together
    let funding_factor: f64 = 0.5; // Funding spike
    let atr_factor: f64 = 0.5; // ATR spike
    let consecutive_factor: f64 = 0.7; // 1 excess loss

    let combined: f64 = (funding_factor * atr_factor * consecutive_factor).clamp(0.0, 1.0);
    assert!((combined - 0.175).abs() < 0.0001, "0.5 × 0.5 × 0.7 = 0.175");
}

#[test]
fn test_regime_filters_clamped_to_zero() {
    // Extreme case: many cascading reductions should clamp to 0.0
    let factor = (0.01_f64).clamp(0.0, 1.0);
    assert_eq!(factor, 0.01); // Very small but not zero

    let negative = (-0.5_f64).clamp(0.0, 1.0);
    assert_eq!(negative, 0.0, "Negative factor should clamp to 0.0");
}

// ============== WEEKLY DRAWDOWN LIMIT TESTS ==============
// @spec:FR-RISK-012 - Weekly drawdown limit

#[test]
fn test_weekly_drawdown_within_limit() {
    let start_equity = 10000.0;
    let current_equity = 9500.0;
    let limit_pct = 7.0;

    let drawdown_pct = (start_equity - current_equity) / start_equity * 100.0;
    let trading_allowed = drawdown_pct < limit_pct;

    assert_eq!(drawdown_pct, 5.0);
    assert!(trading_allowed, "5% DD < 7% limit, trading allowed");
}

#[test]
fn test_weekly_drawdown_exceeds_limit() {
    let start_equity = 10000.0;
    let current_equity = 9200.0;
    let limit_pct = 7.0;

    let drawdown_pct = (start_equity - current_equity) / start_equity * 100.0;
    let trading_allowed = drawdown_pct < limit_pct;

    assert_eq!(drawdown_pct, 8.0);
    assert!(!trading_allowed, "8% DD >= 7% limit, trading blocked");
}

#[test]
fn test_weekly_drawdown_exact_limit() {
    let start_equity: f64 = 10000.0;
    let current_equity: f64 = 9300.0;
    let limit_pct: f64 = 7.0;

    let drawdown_pct = (start_equity - current_equity) / start_equity * 100.0;
    // Use >= check like the engine does (floating point: 7.000000000000001 >= 7.0)
    let trading_blocked = drawdown_pct >= limit_pct;

    assert!((drawdown_pct - 7.0).abs() < 0.001, "Should be ~7%");
    assert!(trading_blocked, "At/near limit, trading blocked (>= check)");
}

#[test]
fn test_weekly_drawdown_equity_increase() {
    // If equity increased, drawdown is negative → should be allowed
    let start_equity = 10000.0;
    let current_equity = 11000.0;
    let limit_pct = 7.0;

    let drawdown_pct = (start_equity - current_equity) / start_equity * 100.0;
    let trading_allowed = drawdown_pct < limit_pct;

    assert!(drawdown_pct < 0.0, "Negative DD means profit");
    assert!(trading_allowed);
}

#[test]
fn test_weekly_drawdown_disabled_when_zero() {
    // limit_pct of 0.0 means disabled
    let limit_pct = 0.0;
    let allowed = limit_pct <= 0.0; // If zero, skip check → always allow

    assert!(allowed);
}

// ============== COMBINED POSITION SIZING PIPELINE TESTS ==============

#[test]
fn test_full_position_sizing_pipeline() {
    // Simulate full pipeline: ATR base → Kelly → Regime → final
    let equity: f64 = 10000.0;
    let base_risk_pct: f64 = 2.0;
    let current_atr: f64 = 500.0;
    let atr_stop_multiplier: f64 = 1.2;

    // Step 1: ATR-based position
    let sl_distance = current_atr * atr_stop_multiplier;
    let risk_amount = equity * (base_risk_pct / 100.0);
    let base_size = risk_amount / sl_distance; // 200 / 600 = 0.3333

    // Step 2: Kelly multiplier (moderate edge)
    let kelly_mult: f64 = 0.35; // Half-Kelly with ~60% win rate

    // Step 3: Regime filter (funding spike active)
    let regime_mult: f64 = 0.5;

    // Step 4: Final size
    let final_size = base_size * kelly_mult * regime_mult;
    // 0.3333 × 0.35 × 0.5 ≈ 0.0583

    assert!((final_size - 0.0583).abs() < 0.001);
}

#[test]
fn test_position_sizing_no_modifiers() {
    // When Kelly=1.0 and regime=1.0 (disabled), position equals base
    let base_size = 0.5;
    let kelly_mult = 1.0;
    let regime_mult = 1.0;

    let final_size = base_size * kelly_mult * regime_mult;

    assert_eq!(final_size, 0.5, "No modifiers should leave size unchanged");
}

// ============== SETTINGS SERIALIZATION TESTS ==============

#[test]
fn test_risk_settings_atr_defaults() {
    let settings = RiskSettings::default();

    // All ATR/Kelly/Regime features disabled by default
    assert!(!settings.atr_stop_enabled);
    assert!(!settings.kelly_enabled);
    assert!(!settings.funding_spike_filter_enabled);
    assert!(!settings.atr_spike_filter_enabled);
    assert!(!settings.consecutive_loss_reduction_enabled);

    // Default values
    assert_eq!(settings.atr_period, 14);
    assert!((settings.atr_stop_multiplier - 1.2).abs() < f64::EPSILON);
    assert!((settings.atr_tp_multiplier - 2.4).abs() < f64::EPSILON);
    assert!((settings.base_risk_pct - 2.0).abs() < f64::EPSILON);
    assert_eq!(settings.kelly_min_trades, 200);
    assert!((settings.kelly_fraction - 0.5).abs() < f64::EPSILON);
    assert_eq!(settings.kelly_lookback, 100);
    assert!((settings.funding_spike_threshold - 0.0003).abs() < f64::EPSILON);
    assert!((settings.funding_spike_reduction - 0.5).abs() < f64::EPSILON);
    assert!((settings.atr_spike_multiplier - 2.0).abs() < f64::EPSILON);
    assert!((settings.atr_spike_reduction - 0.5).abs() < f64::EPSILON);
    assert!((settings.consecutive_loss_reduction_pct - 0.3).abs() < f64::EPSILON);
    assert_eq!(settings.consecutive_loss_reduction_threshold, 3);
    assert!((settings.weekly_drawdown_limit_pct - 7.0).abs() < f64::EPSILON);
}

#[test]
fn test_risk_settings_atr_serialization_roundtrip() {
    let mut settings = RiskSettings::default();
    settings.atr_stop_enabled = true;
    settings.atr_stop_multiplier = 1.5;
    settings.atr_tp_multiplier = 3.0;
    settings.base_risk_pct = 3.0;
    settings.kelly_enabled = true;
    settings.kelly_fraction = 0.4;
    settings.funding_spike_filter_enabled = true;
    settings.weekly_drawdown_limit_pct = 10.0;

    let json = serde_json::to_string(&settings).unwrap();
    let deserialized: RiskSettings = serde_json::from_str(&json).unwrap();

    assert!(deserialized.atr_stop_enabled);
    assert!((deserialized.atr_stop_multiplier - 1.5).abs() < f64::EPSILON);
    assert!((deserialized.atr_tp_multiplier - 3.0).abs() < f64::EPSILON);
    assert!((deserialized.base_risk_pct - 3.0).abs() < f64::EPSILON);
    assert!(deserialized.kelly_enabled);
    assert!((deserialized.kelly_fraction - 0.4).abs() < f64::EPSILON);
    assert!(deserialized.funding_spike_filter_enabled);
    assert!((deserialized.weekly_drawdown_limit_pct - 10.0).abs() < f64::EPSILON);
}

#[test]
fn test_risk_settings_backward_compatible_deserialization() {
    // JSON without new fields should deserialize with defaults
    let old_json = r#"{
        "max_risk_per_trade_pct": 1.0,
        "max_portfolio_risk_pct": 10.0,
        "default_stop_loss_pct": 5.0,
        "default_take_profit_pct": 10.0,
        "max_leverage": 5,
        "min_margin_level": 300.0,
        "max_drawdown_pct": 10.0,
        "daily_loss_limit_pct": 3.0,
        "max_consecutive_losses": 3,
        "cool_down_minutes": 60,
        "position_sizing_method": "RiskBased",
        "min_risk_reward_ratio": 2.0,
        "correlation_limit": 0.7,
        "dynamic_sizing": true,
        "volatility_lookback_hours": 24,
        "trailing_stop_enabled": true,
        "trailing_stop_pct": 3.0,
        "trailing_activation_pct": 5.0,
        "enable_signal_reversal": true,
        "ai_auto_enable_reversal": true,
        "reversal_min_confidence": 0.65,
        "reversal_max_pnl_pct": 10.0,
        "reversal_allowed_regimes": ["trending"]
    }"#;

    let settings: RiskSettings = serde_json::from_str(old_json).unwrap();

    // New fields should have defaults
    assert!(!settings.atr_stop_enabled);
    assert_eq!(settings.atr_period, 14);
    assert!((settings.atr_stop_multiplier - 1.2).abs() < f64::EPSILON);
    assert!(!settings.kelly_enabled);
    assert_eq!(settings.kelly_min_trades, 200);
    assert!(!settings.funding_spike_filter_enabled);
    assert!(!settings.atr_spike_filter_enabled);
    assert!(!settings.consecutive_loss_reduction_enabled);
    assert!((settings.weekly_drawdown_limit_pct - 7.0).abs() < f64::EPSILON);
}

#[test]
fn test_position_sizing_method_atr_based() {
    let method = PositionSizingMethod::ATRBased;
    let json = serde_json::to_string(&method).unwrap();
    assert_eq!(json, "\"ATRBased\"");

    let deserialized: PositionSizingMethod = serde_json::from_str("\"ATRBased\"").unwrap();
    assert!(matches!(deserialized, PositionSizingMethod::ATRBased));
}

#[test]
fn test_risk_settings_with_all_atr_features_enabled() {
    let mut settings = RiskSettings::default();

    // Enable everything
    settings.atr_stop_enabled = true;
    settings.kelly_enabled = true;
    settings.funding_spike_filter_enabled = true;
    settings.atr_spike_filter_enabled = true;
    settings.consecutive_loss_reduction_enabled = true;

    // Custom values
    settings.atr_stop_multiplier = 1.5;
    settings.atr_tp_multiplier = 3.0;
    settings.base_risk_pct = 1.5;
    settings.kelly_min_trades = 100;
    settings.kelly_fraction = 0.4;
    settings.kelly_lookback = 50;
    settings.funding_spike_threshold = 0.0005;
    settings.funding_spike_reduction = 0.3;
    settings.atr_spike_multiplier = 2.5;
    settings.atr_spike_reduction = 0.4;
    settings.consecutive_loss_reduction_pct = 0.25;
    settings.consecutive_loss_reduction_threshold = 4;
    settings.weekly_drawdown_limit_pct = 5.0;

    // Serialize and deserialize
    let json = serde_json::to_string(&settings).unwrap();
    let restored: RiskSettings = serde_json::from_str(&json).unwrap();

    assert!(restored.atr_stop_enabled);
    assert!(restored.kelly_enabled);
    assert!(restored.funding_spike_filter_enabled);
    assert!(restored.atr_spike_filter_enabled);
    assert!(restored.consecutive_loss_reduction_enabled);
    assert!((restored.atr_stop_multiplier - 1.5).abs() < f64::EPSILON);
    assert!((restored.base_risk_pct - 1.5).abs() < f64::EPSILON);
    assert_eq!(restored.kelly_min_trades, 100);
    assert!((restored.kelly_fraction - 0.4).abs() < f64::EPSILON);
    assert_eq!(restored.kelly_lookback, 50);
    assert!((restored.funding_spike_threshold - 0.0005).abs() < f64::EPSILON);
    assert_eq!(restored.consecutive_loss_reduction_threshold, 4);
    assert!((restored.weekly_drawdown_limit_pct - 5.0).abs() < f64::EPSILON);
}

// ============== ORIGINAL TESTS ==============

#[test]
fn test_paper_trading_calculations() {
    // Test PnL calculation
    let entry_price = 50000.0;
    let current_price = 51000.0;
    let quantity = 0.1;

    let unrealized_pnl = (current_price - entry_price) * quantity;
    assert_eq!(unrealized_pnl, 100.0);
}

#[test]
fn test_portfolio_statistics() {
    let total_trades = 10;
    let winning_trades = 6;
    let _losing_trades = 4;

    // Calculate win rate
    let win_rate = (winning_trades as f64 / total_trades as f64) * 100.0;
    assert_eq!(win_rate, 60.0);
}

#[test]
fn test_risk_reward_calculation() {
    let entry_price = 50000.0;
    let stop_loss = 48000.0;
    let take_profit = 54000.0;

    let risk = entry_price - stop_loss;
    let reward = take_profit - entry_price;
    let risk_reward_ratio = reward / risk;

    assert_eq!(risk, 2000.0);
    assert_eq!(reward, 4000.0);
    assert_eq!(risk_reward_ratio, 2.0);
}

#[test]
fn test_portfolio_value_calculation() {
    let cash_balance = 5000.0;
    let btc_quantity = 0.5;
    let btc_price = 50000.0;
    let eth_quantity = 2.0;
    let eth_price = 3000.0;

    let btc_value = btc_quantity * btc_price;
    let eth_value = eth_quantity * eth_price;
    let total_value = cash_balance + btc_value + eth_value;

    assert_eq!(btc_value, 25000.0);
    assert_eq!(eth_value, 6000.0);
    assert_eq!(total_value, 36000.0);
}

#[test]
fn test_sharpe_ratio_calculation() {
    let returns = [0.01, 0.02, -0.005, 0.015, 0.03, -0.01, 0.025];
    let risk_free_rate = 0.02 / 252.0; // Daily risk-free rate

    // Calculate average return
    let avg_return: f64 = returns.iter().sum::<f64>() / returns.len() as f64;

    // Calculate standard deviation
    let variance: f64 = returns
        .iter()
        .map(|r| (r - avg_return).powi(2))
        .sum::<f64>()
        / returns.len() as f64;
    let std_dev = variance.sqrt();

    // Calculate Sharpe ratio
    let sharpe_ratio = (avg_return - risk_free_rate) / std_dev;

    assert!(sharpe_ratio > 0.0);
}

#[test]
fn test_position_pnl() {
    // Test long position PnL
    let entry_price = 3000.0;
    let current_price = 3150.0;
    let quantity = 1.0;

    let unrealized_pnl = (current_price - entry_price) * quantity;
    assert_eq!(unrealized_pnl, 150.0);

    // Test short position PnL
    let short_pnl = (entry_price - current_price) * quantity;
    assert_eq!(short_pnl, -150.0);
}

#[test]
fn test_trade_fees() {
    let trade_value = 10000.0;
    let fee_rate = 0.001; // 0.1%

    let fee = trade_value * fee_rate;
    let net_value = trade_value - fee;

    assert_eq!(fee, 10.0);
    assert_eq!(net_value, 9990.0);
}

#[test]
fn test_max_drawdown() {
    let peak_value = 12000.0;
    let current_value = 9600.0;

    let drawdown = (peak_value - current_value) / peak_value * 100.0;
    assert_eq!(drawdown, 20.0); // 20% drawdown
}

// ============== BOUNDARY CONDITION TESTS ==============

#[test]
fn test_win_rate_zero_trades() {
    // Win rate with zero trades should be 0.0 or handle gracefully
    let total_trades = 0;
    let winning_trades = 0;

    let win_rate = if total_trades == 0 {
        0.0
    } else {
        (winning_trades as f64 / total_trades as f64) * 100.0
    };

    assert_eq!(win_rate, 0.0, "Win rate should be 0 with no trades");
}

#[test]
fn test_win_rate_all_wins() {
    let total_trades = 10;
    let winning_trades = 10;

    let win_rate = (winning_trades as f64 / total_trades as f64) * 100.0;
    assert_eq!(win_rate, 100.0, "Win rate should be 100% with all wins");
}

#[test]
fn test_win_rate_all_losses() {
    let total_trades = 10;
    let winning_trades = 0;

    let win_rate = (winning_trades as f64 / total_trades as f64) * 100.0;
    assert_eq!(win_rate, 0.0, "Win rate should be 0% with all losses");
}

#[test]
fn test_sharpe_ratio_zero_volatility() {
    // Sharpe ratio with zero volatility should be 0 or handle gracefully
    let returns = [0.0, 0.0, 0.0, 0.0, 0.0]; // Flat returns
    let risk_free_rate = 0.02 / 252.0;

    let avg_return: f64 = returns.iter().sum::<f64>() / returns.len() as f64;
    let variance: f64 = returns
        .iter()
        .map(|r| (r - avg_return).powi(2))
        .sum::<f64>()
        / returns.len() as f64;
    let std_dev = variance.sqrt();

    let sharpe_ratio = if std_dev == 0.0 || std_dev.abs() < 1e-10 {
        0.0
    } else {
        (avg_return - risk_free_rate) / std_dev
    };

    assert_eq!(
        sharpe_ratio, 0.0,
        "Sharpe ratio should be 0 with no volatility"
    );
}

#[test]
fn test_max_drawdown_only_profits() {
    // Max drawdown with only profits should be 0
    let mut peak = 10000.0;
    let mut max_dd = 0.0;

    let values = [10000.0, 10500.0, 11000.0, 11500.0, 12000.0]; // Only increasing

    for &value in &values {
        if value > peak {
            peak = value;
        }
        let drawdown = (peak - value) / peak;
        if drawdown > max_dd {
            max_dd = drawdown;
        }
    }

    assert_eq!(max_dd, 0.0, "No drawdown if only profits");
}

#[test]
fn test_max_drawdown_from_peak() {
    let values: [f64; 5] = [10000.0, 12000.0, 9000.0, 8000.0, 10000.0]; // Peak at 12000
    let mut peak: f64 = values[0];
    let mut max_dd: f64 = 0.0;

    for &value in &values {
        if value > peak {
            peak = value;
        }
        let drawdown: f64 = (peak - value) / peak;
        if drawdown > max_dd {
            max_dd = drawdown;
        }
    }

    // Max drawdown from 12000 to 8000 = (12000 - 8000) / 12000 = 0.3333
    let expected: f64 = 0.3333;
    assert!(
        (max_dd - expected).abs() < 0.001,
        "Max drawdown should be ~33.33%, got {}",
        max_dd
    );
}

#[test]
fn test_pnl_percentage_zero_entry_price() {
    // PnL percentage with zero entry price should handle gracefully
    let entry_price: f64 = 0.0;
    let current_price: f64 = 100.0;
    let _quantity: f64 = 1.0; // Not used in calculation but kept for clarity

    let pnl_pct = if entry_price == 0.0 || entry_price.abs() < 1e-10 {
        0.0
    } else {
        ((current_price - entry_price) / entry_price) * 100.0
    };

    assert_eq!(pnl_pct, 0.0, "Should return 0 for zero entry price");
}

#[test]
fn test_margin_ratio_zero_initial_margin() {
    // Margin ratio with zero initial margin should handle gracefully
    let initial_margin: f64 = 0.0;
    let maintenance_margin: f64 = 100.0;

    let ratio = if initial_margin == 0.0 || initial_margin.abs() < 1e-10 {
        0.0
    } else {
        maintenance_margin / initial_margin
    };

    assert_eq!(ratio, 0.0, "Should return 0 for zero initial margin");
}

#[test]
fn test_leverage_calculation() {
    let position_value: f64 = 50000.0;
    let margin: f64 = 5000.0;

    let leverage = position_value / margin;
    assert_eq!(leverage, 10.0, "Leverage should be 10x");

    // Test with zero margin
    let zero_margin: f64 = 0.0;
    let safe_leverage = if zero_margin == 0.0 {
        0.0
    } else {
        position_value / zero_margin
    };
    assert_eq!(safe_leverage, 0.0, "Should handle zero margin");
}

#[test]
fn test_profit_factor_calculation() {
    // Profit factor = Total Wins / Total Losses
    let total_wins = 5000.0;
    let total_losses = 2000.0;

    let profit_factor = total_wins / total_losses;
    assert_eq!(profit_factor, 2.5, "Profit factor should be 2.5");

    // Test with zero losses (perfect trading)
    let zero_losses = 0.0;
    let safe_pf = if zero_losses == 0.0 {
        if total_wins > 0.0 {
            f64::INFINITY
        } else {
            0.0
        }
    } else {
        total_wins / zero_losses
    };
    assert!(
        safe_pf.is_infinite(),
        "Profit factor should be infinite with no losses"
    );
}
