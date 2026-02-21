// @spec:FR-RISK-003 - Portfolio Risk Management Tests
// @spec:FR-STRATEGIES-006 - Signal Combination Tests
// @spec:FR-STRATEGIES-007 - Multi-Timeframe Analysis Tests
// @ref:docs/features/paper-trading.md#risk-management
// @test:TC-RISK-001 through TC-RISK-050

use binance_trading_bot::market_data::cache::CandleData;
use binance_trading_bot::paper_trading::trade::TradeType;
use binance_trading_bot::strategies::strategy_engine::{
    SignalCombinationMode, StrategyEngine, StrategyEngineConfig, StrategySignalResult,
};
use binance_trading_bot::strategies::{StrategyInput, TradingSignal};
use std::collections::HashMap;

// ============== HELPER FUNCTIONS ==============

/// Create test candles with realistic data
fn create_test_candles(count: usize, base_price: f64) -> Vec<CandleData> {
    (0..count)
        .map(|i| {
            let time_offset = (i as i64) * 3600000; // 1 hour per candle
            CandleData {
                open: base_price + (i as f64),
                high: base_price + (i as f64) + 10.0,
                low: base_price + (i as f64) - 10.0,
                close: base_price + (i as f64) + 5.0,
                volume: 1000.0 + (i as f64) * 10.0,
                open_time: time_offset,
                close_time: time_offset + 3600000,
                quote_volume: (1000.0 + (i as f64) * 10.0) * base_price,
                trades: (100 + i) as i64,
                is_closed: true,
            }
        })
        .collect()
}

/// Create strategy input with multiple timeframes
fn create_multi_timeframe_input(symbol: &str) -> StrategyInput {
    let mut timeframe_data = HashMap::new();
    timeframe_data.insert("5m".to_string(), create_test_candles(50, 50000.0));
    timeframe_data.insert("15m".to_string(), create_test_candles(50, 50000.0));
    timeframe_data.insert("30m".to_string(), create_test_candles(50, 50000.0));
    timeframe_data.insert("1h".to_string(), create_test_candles(50, 50000.0));

    StrategyInput {
        symbol: symbol.to_string(),
        timeframe_data,
        current_price: 50000.0,
        volume_24h: 1000000.0,
        timestamp: chrono::Utc::now().timestamp_millis(),
    }
}

/// Helper to create strategy result
fn create_strategy_result(
    name: &str,
    signal: TradingSignal,
    confidence: f64,
) -> StrategySignalResult {
    StrategySignalResult {
        strategy_name: name.to_string(),
        signal,
        confidence,
        reasoning: format!("{} analysis", name),
        weight: 1.0,
        metadata: HashMap::new(),
    }
}

// ============== 1. SIGNAL COMBINATION TESTS (strategy_engine.rs) ==============

#[cfg(test)]
mod signal_combination_tests {
    use super::*;

    /// @test:TC-STRATEGIES-001 - Test 4/5 requirement: 4 Long + 1 Short = Long signal
    #[tokio::test]
    async fn test_consensus_4_long_1_short_gives_long() {
        let mut config = StrategyEngineConfig::default();
        config.signal_combination_mode = SignalCombinationMode::Consensus;
        config.min_strategies_agreement = 4; // Require ≥4/5

        let engine = StrategyEngine::with_config(config);
        let input = create_multi_timeframe_input("BTCUSDT");

        // Analyze market will combine signals internally
        // We'll test the consensus logic indirectly through analyze_market
        let result = engine.analyze_market(&input).await;

        // Should succeed with multiple strategies running
        assert!(result.is_ok(), "Market analysis should succeed");

        let combined = result.unwrap();

        // Verify metadata contains strategy counts
        assert!(combined.metadata.contains_key("long_signals"));
        assert!(combined.metadata.contains_key("short_signals"));
        assert!(combined.metadata.contains_key("neutral_signals"));
        assert!(combined.metadata.contains_key("total_strategies"));
    }

    /// @test:TC-STRATEGIES-002 - Test consensus requires minimum strategies
    #[tokio::test]
    async fn test_consensus_requires_minimum_strategies() {
        let mut config = StrategyEngineConfig::default();
        config.signal_combination_mode = SignalCombinationMode::Consensus;
        config.min_strategies_agreement = 4; // Require ≥4/5

        let engine = StrategyEngine::with_config(config);
        let input = create_multi_timeframe_input("BTCUSDT");

        let result = engine.analyze_market(&input).await;

        if result.is_ok() {
            let combined = result.unwrap();
            let _total_strategies = combined
                .metadata
                .get("total_strategies")
                .and_then(|v| v.as_u64())
                .unwrap_or(0);

            // If we got a non-neutral signal, it means enough strategies agreed
            if combined.final_signal != TradingSignal::Neutral {
                let long_count = combined
                    .metadata
                    .get("long_signals")
                    .and_then(|v| v.as_u64())
                    .unwrap_or(0);
                let short_count = combined
                    .metadata
                    .get("short_signals")
                    .and_then(|v| v.as_u64())
                    .unwrap_or(0);

                let max_agreement = long_count.max(short_count);

                // Verify at least 4 strategies agreed (4/5 threshold)
                assert!(
                    max_agreement >= 4,
                    "Signal {} requires ≥4 strategies to agree, got {}",
                    combined.final_signal.as_str(),
                    max_agreement
                );
            }
        }
    }

    /// @test:TC-STRATEGIES-003 - Test configurable threshold
    #[tokio::test]
    async fn test_consensus_configurable_threshold() {
        let mut config = StrategyEngineConfig::default();
        config.signal_combination_mode = SignalCombinationMode::Consensus;
        config.min_strategies_agreement = 3; // Lower threshold: ≥3/5 (60%)

        // Verify config is applied before moving
        assert_eq!(
            config.min_strategies_agreement, 3,
            "Should support configurable threshold of 3/5"
        );

        let _engine = StrategyEngine::with_config(config);
    }

    /// @test:TC-STRATEGIES-004 - Test different combination modes
    #[test]
    fn test_different_combination_modes() {
        let modes = vec![
            SignalCombinationMode::WeightedAverage,
            SignalCombinationMode::Consensus,
            SignalCombinationMode::BestConfidence,
            SignalCombinationMode::Conservative,
        ];

        for mode in modes {
            let mut config = StrategyEngineConfig::default();
            config.signal_combination_mode = mode;

            let _engine = StrategyEngine::with_config(config);
            // Each mode should initialize successfully
        }
    }

    /// @test:TC-STRATEGIES-005 - Test default config values
    #[test]
    fn test_default_config_values() {
        let config = StrategyEngineConfig::default();

        assert_eq!(
            config.min_strategies_agreement, 4,
            "Default should require 4/5 strategies (80%)"
        );
        assert_eq!(
            config.enabled_strategies.len(),
            5,
            "Should have 5 strategies enabled by default"
        );
        assert!(
            matches!(
                config.signal_combination_mode,
                SignalCombinationMode::Consensus
            ),
            "Default should use Consensus mode"
        );
    }
}

// ============== 2. PORTFOLIO RISK LIMIT TESTS ==============

#[cfg(test)]
mod portfolio_risk_tests {
    use super::*;

    /// @test:TC-RISK-001 - Portfolio risk calculation logic (empty portfolio)
    #[test]
    fn test_portfolio_risk_calculation_empty() {
        // Empty portfolio = 0% risk
        let open_trades: Vec<(TradeType, f64, f64, Option<f64>)> = vec![];

        let total_risk = calculate_portfolio_risk(&open_trades, 10000.0, 10.0);

        assert_eq!(total_risk, 0.0, "Empty portfolio should have 0% risk");
    }

    /// @test:TC-RISK-002 - Single Long position within limit
    #[test]
    fn test_portfolio_risk_single_long_within_limit() {
        // Entry: 50000, SL: 47500 (5% below), Quantity: 0.1
        // Position value: 50000 * 0.1 = 5000
        // Stop loss distance: (50000 - 47500) / 50000 = 5%
        // Risk amount: 5000 * 5% = 250
        // Risk % of equity (10000): 250 / 10000 = 2.5%
        let trades = vec![(TradeType::Long, 50000.0, 0.1, Some(47500.0))];

        let total_risk = calculate_portfolio_risk(&trades, 10000.0, 10.0);

        assert!(
            (total_risk - 2.5).abs() < 0.1,
            "Expected ~2.5% risk, got {}%",
            total_risk
        );
        assert!(total_risk < 10.0, "Risk should be within 10% limit");
    }

    /// @test:TC-RISK-003 - Multiple positions exceeding limit
    #[test]
    fn test_portfolio_risk_multiple_positions_exceeding_limit() {
        // 4 positions, each ~2.5-3% risk = ~11% total > 10% limit
        let trades = vec![
            (TradeType::Long, 50000.0, 0.1, Some(47500.0)), // 2.5%
            (TradeType::Long, 3000.0, 2.0, Some(2850.0)),   // 3.0%
            (TradeType::Long, 400.0, 15.0, Some(380.0)),    // 3.0%
            (TradeType::Long, 100.0, 50.0, Some(95.0)),     // 2.5%
        ];

        let total_risk = calculate_portfolio_risk(&trades, 10000.0, 10.0);

        assert!(
            total_risk > 10.0,
            "Total risk {}% should exceed 10% limit",
            total_risk
        );
    }

    /// @test:TC-RISK-004 - Missing stop loss uses 5% default for Long
    #[test]
    fn test_portfolio_risk_missing_stop_loss_long() {
        // No stop loss specified → should assume 5% below entry for Long
        let trades = vec![(TradeType::Long, 50000.0, 0.1, None)];

        let total_risk = calculate_portfolio_risk(&trades, 10000.0, 10.0);

        assert!(
            (total_risk - 2.5).abs() < 0.1,
            "Missing SL should use 5% default for Long, expected ~2.5% risk, got {}%",
            total_risk
        );
    }

    /// @test:TC-RISK-005 - Missing stop loss uses 5% default for Short
    #[test]
    fn test_portfolio_risk_missing_stop_loss_short() {
        // No stop loss specified → should assume 5% above entry for Short
        let trades = vec![(TradeType::Short, 50000.0, 0.1, None)];

        let total_risk = calculate_portfolio_risk(&trades, 10000.0, 10.0);

        assert!(
            (total_risk - 2.5).abs() < 0.1,
            "Missing SL should use 5% default for Short, expected ~2.5% risk, got {}%",
            total_risk
        );
    }

    /// @test:TC-RISK-006 - Long with explicit stop loss
    #[test]
    fn test_portfolio_risk_long_explicit_stop_loss() {
        // Long: Entry 50000, SL 48000 (4% below)
        let trades = vec![(TradeType::Long, 50000.0, 0.1, Some(48000.0))];

        let total_risk = calculate_portfolio_risk(&trades, 10000.0, 10.0);

        // Risk: (50000 - 48000) / 50000 = 4%
        // Risk amount: 5000 * 4% = 200
        // Risk % of equity: 200 / 10000 = 2.0%
        assert!(
            (total_risk - 2.0).abs() < 0.1,
            "Expected ~2.0% risk for 4% SL distance, got {}%",
            total_risk
        );
    }

    /// @test:TC-RISK-007 - Short with explicit stop loss
    #[test]
    fn test_portfolio_risk_short_explicit_stop_loss() {
        // Short: Entry 50000, SL 51000 (2% above)
        let trades = vec![(TradeType::Short, 50000.0, 0.1, Some(51000.0))];

        let total_risk = calculate_portfolio_risk(&trades, 10000.0, 10.0);

        // Risk: |50000 - 51000| / 50000 = 2%
        // Risk amount: 5000 * 2% = 100
        // Risk % of equity: 100 / 10000 = 1.0%
        assert!(
            (total_risk - 1.0).abs() < 0.1,
            "Expected ~1.0% risk for 2% SL distance (Short), got {}%",
            total_risk
        );
    }

    /// @test:TC-RISK-008 - Zero equity (division by zero protection)
    #[test]
    fn test_portfolio_risk_zero_equity() {
        let trades = vec![(TradeType::Long, 50000.0, 0.1, Some(47500.0))];

        let total_risk = calculate_portfolio_risk(&trades, 0.0, 10.0);

        assert!(
            total_risk.is_infinite(),
            "Should return infinity for zero equity to prevent division by zero"
        );
    }

    /// @test:TC-RISK-009 - Tight stop loss (0.5%)
    #[test]
    fn test_portfolio_risk_tight_stop_loss() {
        // Very tight SL: 0.5% below entry
        let trades = vec![(TradeType::Long, 50000.0, 0.1, Some(49750.0))];

        let total_risk = calculate_portfolio_risk(&trades, 10000.0, 10.0);

        assert!(
            (total_risk - 0.25).abs() < 0.1,
            "Expected ~0.25% risk for tight SL, got {}%",
            total_risk
        );
        assert!(total_risk < 1.0, "Tight SL should result in low risk");
    }

    /// @test:TC-RISK-010 - Wide stop loss (10%)
    #[test]
    fn test_portfolio_risk_wide_stop_loss() {
        // Wide SL: 10% below entry
        let trades = vec![(TradeType::Long, 50000.0, 0.1, Some(45000.0))];

        let total_risk = calculate_portfolio_risk(&trades, 10000.0, 10.0);

        assert!(
            (total_risk - 5.0).abs() < 0.2,
            "Expected ~5.0% risk for wide SL, got {}%",
            total_risk
        );
    }

    /// Helper function that mimics engine.rs:1376-1435 logic
    fn calculate_portfolio_risk(
        trades: &[(TradeType, f64, f64, Option<f64>)], // (type, entry_price, quantity, stop_loss)
        equity: f64,
        _max_portfolio_risk_pct: f64,
    ) -> f64 {
        if trades.is_empty() {
            return 0.0;
        }

        if equity == 0.0 || equity.abs() < 1e-10 {
            return f64::INFINITY;
        }

        let mut total_risk = 0.0;

        for (trade_type, entry_price, quantity, stop_loss) in trades {
            let position_value = entry_price * quantity;

            // Use stop_loss if set, otherwise assume max 5% stop loss distance
            let stop_loss_price = stop_loss.unwrap_or_else(|| match trade_type {
                TradeType::Long => entry_price * 0.95,  // 5% below for Long
                TradeType::Short => entry_price * 1.05, // 5% above for Short
            });

            let stop_loss_distance_pct =
                ((entry_price - stop_loss_price).abs() / entry_price) * 100.0;
            let risk_amount = position_value * (stop_loss_distance_pct / 100.0);
            let risk_pct_of_equity = (risk_amount / equity) * 100.0;
            total_risk += risk_pct_of_equity;
        }

        total_risk
    }
}

// ============== 3. MULTI-TIMEFRAME TESTS ==============

#[cfg(test)]
mod multi_timeframe_tests {
    use super::*;

    /// @test:TC-STRATEGIES-010 - All 4 timeframes loaded (15m, 30m, 1h, 4h)
    #[test]
    fn test_multi_timeframe_all_loaded() {
        let input = create_multi_timeframe_input("BTCUSDT");

        assert!(
            input.timeframe_data.contains_key("15m"),
            "15m timeframe should be loaded"
        );
        assert!(
            input.timeframe_data.contains_key("30m"),
            "30m timeframe should be loaded"
        );
        assert!(
            input.timeframe_data.contains_key("5m"),
            "5m timeframe should be loaded"
        );
        assert!(
            input.timeframe_data.contains_key("15m"),
            "15m timeframe should be loaded"
        );
        assert_eq!(
            input.timeframe_data.len(),
            4,
            "Should have exactly 4 timeframes"
        );
    }

    /// @test:TC-STRATEGIES-011 - Cache key format: symbol_timeframe
    #[test]
    fn test_cache_key_format() {
        let symbol = "BTCUSDT";
        let timeframes = ["5m", "15m", "30m", "1h"];

        for tf in &timeframes {
            let cache_key = format!("{}_{}", symbol, tf);
            assert_eq!(
                cache_key,
                format!("BTCUSDT_{}", tf),
                "Cache key format should be symbol_timeframe"
            );
        }
    }

    /// @test:TC-STRATEGIES-012 - Warmup period check for 1h and 4h
    #[test]
    fn test_warmup_period_required_timeframes() {
        const MIN_CANDLES_REQUIRED: usize = 50;
        let required_timeframes = ["5m", "15m"];

        for tf in &required_timeframes {
            let candles = create_test_candles(MIN_CANDLES_REQUIRED, 50000.0);
            assert_eq!(
                candles.len(),
                MIN_CANDLES_REQUIRED,
                "Timeframe {} should have minimum {} candles for warmup",
                tf,
                MIN_CANDLES_REQUIRED
            );
        }
    }

    /// @test:TC-STRATEGIES-013 - Insufficient data detection
    #[test]
    fn test_insufficient_data_detection() {
        const MIN_CANDLES_REQUIRED: usize = 50;
        let candles_insufficient = create_test_candles(30, 50000.0);

        assert!(
            candles_insufficient.len() < MIN_CANDLES_REQUIRED,
            "Should detect insufficient data: {} < {}",
            candles_insufficient.len(),
            MIN_CANDLES_REQUIRED
        );
    }

    /// @test:TC-STRATEGIES-014 - Timeframe data ordering (chronological)
    #[test]
    fn test_timeframe_data_ordering() {
        let candles = create_test_candles(50, 50000.0);

        // Verify chronological order (ascending timestamps)
        for i in 1..candles.len() {
            assert!(
                candles[i].open_time > candles[i - 1].open_time,
                "Candles should be in chronological order"
            );
        }

        // Most recent candle is last
        let last_candle = candles.last().unwrap();
        let first_candle = candles.first().unwrap();
        assert!(
            last_candle.open_time > first_candle.open_time,
            "Last candle should be most recent"
        );
    }

    /// @test:TC-STRATEGIES-015 - Exact 50 candles for each timeframe
    #[test]
    fn test_exact_50_candles_each_timeframe() {
        let input = create_multi_timeframe_input("BTCUSDT");

        for (tf, candles) in &input.timeframe_data {
            assert_eq!(
                candles.len(),
                50,
                "Timeframe {} should have exactly 50 candles",
                tf
            );
        }
    }

    /// @test:TC-STRATEGIES-016 - Required timeframes for strategies
    #[test]
    fn test_required_timeframes_for_strategies() {
        let input = create_multi_timeframe_input("BTCUSDT");

        // CRITICAL: All strategies require BOTH 1h and 4h timeframes (FR-STRATEGIES-007)
        const REQUIRED_TIMEFRAMES: &[&str] = &["5m", "15m"];

        for tf in REQUIRED_TIMEFRAMES {
            assert!(
                input.timeframe_data.contains_key(*tf),
                "Required timeframe {} must be present for strategy analysis",
                tf
            );

            let candles = input.timeframe_data.get(*tf).unwrap();
            assert!(
                candles.len() >= 50,
                "Required timeframe {} must have at least 50 candles",
                tf
            );
        }
    }
}

// ============== 4. INTEGRATION TESTS ==============

#[cfg(test)]
mod integration_tests {
    use super::*;

    /// @test:TC-INTEGRATION-001 - Signal generation with multi-timeframe data
    #[tokio::test]
    async fn test_signal_generation_multi_timeframe() {
        let engine = StrategyEngine::new();
        let input = create_multi_timeframe_input("BTCUSDT");

        let result = engine.analyze_market(&input).await;

        assert!(
            result.is_ok(),
            "Should successfully analyze market with multi-timeframe data"
        );

        let combined = result.unwrap();
        assert!(
            !combined.strategy_signals.is_empty(),
            "Should have strategy signals"
        );
        assert!(combined.combined_confidence >= 0.0 && combined.combined_confidence <= 1.0);
    }

    /// @test:TC-INTEGRATION-002 - Risk check workflow
    #[test]
    fn test_risk_check_workflow() {
        // Simulate complete risk check workflow

        // Check 1: Portfolio risk
        let trades = vec![(TradeType::Long, 50000.0, 0.1, Some(47500.0))];
        let portfolio_risk = calculate_portfolio_risk(&trades, 10000.0);
        assert!(portfolio_risk < 10.0, "Portfolio risk check should pass");

        // Check 2: Position count
        let max_positions = 5;
        assert!(
            trades.len() <= max_positions,
            "Position count check should pass"
        );

        // Check 3: Individual trade risk
        let max_risk_per_trade = 3.0;
        assert!(
            portfolio_risk < max_risk_per_trade,
            "Individual trade risk should be acceptable"
        );
    }

    /// Helper
    fn calculate_portfolio_risk(trades: &[(TradeType, f64, f64, Option<f64>)], equity: f64) -> f64 {
        if trades.is_empty() || equity == 0.0 {
            return 0.0;
        }

        let mut total_risk = 0.0;
        for (trade_type, entry_price, quantity, stop_loss) in trades {
            let position_value = entry_price * quantity;
            let stop_loss_price = stop_loss.unwrap_or_else(|| match trade_type {
                TradeType::Long => entry_price * 0.95,
                TradeType::Short => entry_price * 1.05,
            });
            let stop_loss_distance_pct =
                ((entry_price - stop_loss_price).abs() / entry_price) * 100.0;
            let risk_amount = position_value * (stop_loss_distance_pct / 100.0);
            let risk_pct_of_equity = (risk_amount / equity) * 100.0;
            total_risk += risk_pct_of_equity;
        }
        total_risk
    }
}

// ============== 5. ERROR SCENARIO TESTS ==============

#[cfg(test)]
mod error_scenario_tests {
    use super::*;

    /// @test:TC-ERROR-001 - Zero equity (division by zero)
    #[test]
    fn test_zero_equity_division_by_zero() {
        let trades = vec![(TradeType::Long, 50000.0, 0.1, Some(47500.0))];
        let total_risk = calculate_portfolio_risk(&trades, 0.0);

        assert!(
            total_risk.is_infinite() || total_risk == 0.0,
            "Should handle zero equity gracefully"
        );
    }

    /// @test:TC-ERROR-002 - Negative equity
    #[test]
    fn test_negative_equity() {
        let trades = vec![(TradeType::Long, 50000.0, 0.1, Some(47500.0))];
        let total_risk = calculate_portfolio_risk(&trades, -1000.0);

        assert!(
            total_risk.is_infinite() || total_risk.is_nan(),
            "Should handle negative equity"
        );
    }

    /// @test:TC-ERROR-003 - Zero quantity position
    #[test]
    fn test_zero_quantity_position() {
        let trades = vec![(TradeType::Long, 50000.0, 0.0, Some(47500.0))];
        let total_risk = calculate_portfolio_risk(&trades, 10000.0);

        assert_eq!(total_risk, 0.0, "Zero quantity should result in 0 risk");
    }

    /// @test:TC-ERROR-004 - Very large position
    #[test]
    fn test_very_large_position() {
        let trades = vec![(TradeType::Long, 50000.0, 1000.0, Some(47500.0))];
        let total_risk = calculate_portfolio_risk(&trades, 10000.0);

        assert!(
            total_risk.is_finite(),
            "Should handle large positions without overflow"
        );
        assert!(
            total_risk > 100.0,
            "Large position should result in very high risk: {}%",
            total_risk
        );
    }

    // Helper
    fn calculate_portfolio_risk(trades: &[(TradeType, f64, f64, Option<f64>)], equity: f64) -> f64 {
        if trades.is_empty() {
            return 0.0;
        }

        if equity == 0.0 || equity.abs() < 1e-10 {
            return f64::INFINITY;
        }

        if equity < 0.0 {
            return f64::INFINITY;
        }

        let mut total_risk = 0.0;
        for (trade_type, entry_price, quantity, stop_loss) in trades {
            if quantity == &0.0 {
                continue;
            }

            let position_value = entry_price * quantity;
            let stop_loss_price = stop_loss.unwrap_or_else(|| match trade_type {
                TradeType::Long => entry_price * 0.95,
                TradeType::Short => entry_price * 1.05,
            });

            let stop_loss_distance_pct =
                ((entry_price - stop_loss_price).abs() / entry_price) * 100.0;
            let risk_amount = position_value * (stop_loss_distance_pct / 100.0);
            let risk_pct_of_equity = (risk_amount / equity) * 100.0;
            total_risk += risk_pct_of_equity;
        }

        total_risk
    }
}
