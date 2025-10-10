use super::*;
use crate::strategies::indicators::calculate_rsi;
use async_trait::async_trait;
use serde_json::json;

/// RSI-based trading strategy

// @spec:FR-STRATEGY-001 - RSI Strategy
// @ref:specs/02-design/2.5-components/COMP-RUST-TRADING.md#strategies
// @test:TC-TRADING-022, TC-TRADING-023

#[derive(Debug, Clone)]
pub struct RsiStrategy {
    config: StrategyConfig,
}

impl RsiStrategy {
    pub fn new() -> Self {
        let mut config = StrategyConfig::default();
        config
            .parameters
            .insert("rsi_period".to_string(), json!(14));
        config
            .parameters
            .insert("oversold_threshold".to_string(), json!(30.0));
        config
            .parameters
            .insert("overbought_threshold".to_string(), json!(70.0));
        config
            .parameters
            .insert("extreme_oversold".to_string(), json!(20.0));
        config
            .parameters
            .insert("extreme_overbought".to_string(), json!(80.0));

        Self { config }
    }

    pub fn with_config(config: StrategyConfig) -> Self {
        Self { config }
    }

    fn get_rsi_period(&self) -> usize {
        self.config
            .parameters
            .get("rsi_period")
            .and_then(|v| v.as_u64())
            .unwrap_or(14) as usize
    }

    fn get_oversold_threshold(&self) -> f64 {
        self.config
            .parameters
            .get("oversold_threshold")
            .and_then(|v| v.as_f64())
            .unwrap_or(30.0)
    }

    fn get_overbought_threshold(&self) -> f64 {
        self.config
            .parameters
            .get("overbought_threshold")
            .and_then(|v| v.as_f64())
            .unwrap_or(70.0)
    }

    fn get_extreme_oversold(&self) -> f64 {
        self.config
            .parameters
            .get("extreme_oversold")
            .and_then(|v| v.as_f64())
            .unwrap_or(20.0)
    }

    fn get_extreme_overbought(&self) -> f64 {
        self.config
            .parameters
            .get("extreme_overbought")
            .and_then(|v| v.as_f64())
            .unwrap_or(80.0)
    }
}

#[async_trait]
impl Strategy for RsiStrategy {
    fn name(&self) -> &'static str {
        "RSI Strategy"
    }

    fn description(&self) -> &'static str {
        "RSI-based strategy that identifies oversold/overbought conditions for reversal trading"
    }

    fn required_timeframes(&self) -> Vec<&'static str> {
        vec!["1h", "4h"]
    }

    async fn analyze(&self, data: &StrategyInput) -> Result<StrategyOutput, StrategyError> {
        self.validate_data(data)?;

        let primary_timeframe = "1h";
        let confirmation_timeframe = "4h";

        let primary_candles = data.timeframe_data.get(primary_timeframe).ok_or_else(|| {
            StrategyError::InsufficientData(format!("Missing {primary_timeframe} data"))
        })?;

        let confirmation_candles =
            data.timeframe_data
                .get(confirmation_timeframe)
                .ok_or_else(|| {
                    StrategyError::InsufficientData(format!(
                        "Missing {confirmation_timeframe} data"
                    ))
                })?;

        let rsi_period = self.get_rsi_period();

        // Calculate RSI for both timeframes
        let primary_rsi =
            calculate_rsi(primary_candles, rsi_period).map_err(StrategyError::CalculationError)?;

        let confirmation_rsi = calculate_rsi(confirmation_candles, rsi_period)
            .map_err(StrategyError::CalculationError)?;

        let current_rsi_1h = *primary_rsi.last().ok_or_else(|| {
            StrategyError::InsufficientData("No RSI values calculated for 1h".to_string())
        })?;
        let current_rsi_4h = *confirmation_rsi.last().ok_or_else(|| {
            StrategyError::InsufficientData("No RSI values calculated for 4h".to_string())
        })?;

        // Get previous RSI values for trend analysis
        let prev_rsi_1h = if primary_rsi.len() > 1 {
            primary_rsi[primary_rsi.len() - 2]
        } else {
            current_rsi_1h
        };
        let prev_rsi_4h = if confirmation_rsi.len() > 1 {
            confirmation_rsi[confirmation_rsi.len() - 2]
        } else {
            current_rsi_4h
        };

        let oversold = self.get_oversold_threshold();
        let overbought = self.get_overbought_threshold();
        let extreme_oversold = self.get_extreme_oversold();
        let extreme_overbought = self.get_extreme_overbought();

        // Determine signal and confidence
        let (signal, confidence, reasoning) = self.analyze_rsi_signals(
            current_rsi_1h,
            current_rsi_4h,
            prev_rsi_1h,
            prev_rsi_4h,
            oversold,
            overbought,
            extreme_oversold,
            extreme_overbought,
        );

        let mut metadata = std::collections::HashMap::new();
        metadata.insert("rsi_1h".to_string(), json!(current_rsi_1h));
        metadata.insert("rsi_4h".to_string(), json!(current_rsi_4h));
        metadata.insert("prev_rsi_1h".to_string(), json!(prev_rsi_1h));
        metadata.insert("prev_rsi_4h".to_string(), json!(prev_rsi_4h));
        metadata.insert("oversold_threshold".to_string(), json!(oversold));
        metadata.insert("overbought_threshold".to_string(), json!(overbought));

        Ok(StrategyOutput {
            signal,
            confidence,
            reasoning,
            timeframe: primary_timeframe.to_string(),
            timestamp: data.timestamp,
            metadata,
        })
    }

    fn config(&self) -> &StrategyConfig {
        &self.config
    }

    fn update_config(&mut self, config: StrategyConfig) {
        self.config = config;
    }

    fn validate_data(&self, data: &StrategyInput) -> Result<(), StrategyError> {
        let required_timeframes = self.required_timeframes();

        for timeframe in required_timeframes {
            let candles = data.timeframe_data.get(timeframe).ok_or_else(|| {
                StrategyError::DataValidation(format!("Missing {timeframe} timeframe data"))
            })?;

            let min_required = self.get_rsi_period() + 5; // RSI period + buffer

            if candles.len() < min_required {
                let candles_len = candles.len();
                return Err(StrategyError::InsufficientData(format!(
                    "Need at least {min_required} candles for {timeframe} timeframe, got {candles_len}"
                )));
            }
        }

        Ok(())
    }
}

impl RsiStrategy {
    #[allow(clippy::too_many_arguments)]
    fn analyze_rsi_signals(
        &self,
        rsi_1h: f64,
        rsi_4h: f64,
        prev_rsi_1h: f64,
        _prev_rsi_4h: f64,
        oversold: f64,
        overbought: f64,
        extreme_oversold: f64,
        extreme_overbought: f64,
    ) -> (TradingSignal, f64, String) {
        // Strong bullish signals
        if rsi_1h <= extreme_oversold && rsi_4h <= oversold && prev_rsi_1h < rsi_1h {
            return (
                TradingSignal::Long,
                0.87,
                "Strong bullish momentum with RSI oversold recovery".to_string(),
            );
        }

        // Strong bearish signals
        if rsi_1h >= extreme_overbought && rsi_4h >= overbought && prev_rsi_1h > rsi_1h {
            return (
                TradingSignal::Short,
                0.87,
                "Strong bearish momentum with RSI overbought breakdown".to_string(),
            );
        }

        // Moderate bullish signals
        if rsi_1h <= oversold && rsi_4h < 50.0 && prev_rsi_1h < rsi_1h {
            return (
                TradingSignal::Long,
                0.73,
                "Bullish divergence with RSI recovery from oversold".to_string(),
            );
        }

        // Moderate bearish signals
        if rsi_1h >= overbought && rsi_4h > 50.0 && prev_rsi_1h > rsi_1h {
            return (
                TradingSignal::Short,
                0.73,
                "Bearish divergence detected on 1H timeframe".to_string(),
            );
        }

        // Weak bullish signals
        if rsi_1h > oversold && rsi_1h < 50.0 && prev_rsi_1h < rsi_1h && rsi_4h < 50.0 {
            return (
                TradingSignal::Long,
                0.51,
                "Weak bullish momentum with RSI rising from low levels".to_string(),
            );
        }

        // Weak bearish signals
        if rsi_1h < overbought && rsi_1h > 50.0 && prev_rsi_1h > rsi_1h && rsi_4h > 50.0 {
            return (
                TradingSignal::Short,
                0.51,
                "Weak bearish momentum with RSI declining from high levels".to_string(),
            );
        }

        // Neutral/consolidation
        let confidence = if (rsi_1h - 50.0).abs() < 10.0 && (rsi_4h - 50.0).abs() < 15.0 {
            0.65
        } else {
            0.45
        };

        (
            TradingSignal::Neutral,
            confidence,
            "Consolidation phase, waiting for breakout".to_string(),
        )
    }
}

impl Default for RsiStrategy {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::market_data::cache::CandleData;
    use std::collections::HashMap;

    fn create_test_candles(prices: Vec<f64>) -> Vec<CandleData> {
        prices
            .iter()
            .enumerate()
            .map(|(i, &price)| CandleData {
                open: price,
                high: price * 1.01,
                low: price * 0.99,
                close: price,
                volume: 1000.0,
                open_time: (i as i64) * 3600000,
                close_time: (i as i64) * 3600000 + 3600000,
                quote_volume: 1000.0 * price,
                trades: 100,
                is_closed: true,
            })
            .collect()
    }

    fn create_test_input(prices_1h: Vec<f64>, prices_4h: Vec<f64>) -> StrategyInput {
        let mut timeframe_data = HashMap::new();
        timeframe_data.insert("1h".to_string(), create_test_candles(prices_1h));
        timeframe_data.insert("4h".to_string(), create_test_candles(prices_4h));

        StrategyInput {
            symbol: "BTCUSDT".to_string(),
            timeframe_data,
            current_price: 50000.0,
            volume_24h: 1000000.0,
            timestamp: 1234567890,
        }
    }

    #[tokio::test]
    async fn test_rsi_strategy_new() {
        let strategy = RsiStrategy::new();
        assert_eq!(strategy.name(), "RSI Strategy");
        assert!(strategy.config().enabled);
        assert_eq!(strategy.config().weight, 1.0);
    }

    #[tokio::test]
    async fn test_rsi_strategy_strong_bullish_signal() {
        let strategy = RsiStrategy::new();

        // Create oversold conditions
        let prices_1h: Vec<f64> = (0..20).map(|i| 100.0 - (i as f64 * 2.0)).collect();
        let mut recovery_prices = prices_1h.clone();
        recovery_prices.extend((0..5).map(|i| 60.0 + (i as f64)));

        let prices_4h: Vec<f64> = (0..20).map(|i| 100.0 - (i as f64 * 1.5)).collect();

        let input = create_test_input(recovery_prices, prices_4h);
        let result = strategy.analyze(&input).await;

        assert!(result.is_ok());
        let output = result.unwrap();
        assert_eq!(output.signal, TradingSignal::Long);
        assert!(output.confidence > 0.5);
    }

    #[tokio::test]
    async fn test_rsi_strategy_strong_bearish_signal() {
        let strategy = RsiStrategy::new();

        // Create overbought conditions
        let prices_1h: Vec<f64> = (0..20).map(|i| 100.0 + (i as f64 * 2.0)).collect();
        let mut decline_prices = prices_1h.clone();
        decline_prices.extend((0..5).map(|i| 140.0 - (i as f64)));

        let prices_4h: Vec<f64> = (0..20).map(|i| 100.0 + (i as f64 * 1.5)).collect();

        let input = create_test_input(decline_prices, prices_4h);
        let result = strategy.analyze(&input).await;

        assert!(result.is_ok());
        let output = result.unwrap();
        assert_eq!(output.signal, TradingSignal::Short);
        assert!(output.confidence > 0.5);
    }

    #[tokio::test]
    async fn test_rsi_strategy_neutral_signal() {
        let strategy = RsiStrategy::new();

        // Create neutral conditions with prices around 100
        let prices_1h: Vec<f64> = (0..25).map(|i| 100.0 + ((i as f64 % 3.0) - 1.0)).collect();
        let prices_4h: Vec<f64> = (0..25).map(|i| 100.0 + ((i as f64 % 2.0) - 0.5)).collect();

        let input = create_test_input(prices_1h, prices_4h);
        let result = strategy.analyze(&input).await;

        assert!(result.is_ok());
        let output = result.unwrap();
        assert_eq!(output.signal, TradingSignal::Neutral);
    }

    #[tokio::test]
    async fn test_rsi_strategy_configuration() {
        let mut strategy = RsiStrategy::new();

        assert_eq!(strategy.get_rsi_period(), 14);
        assert_eq!(strategy.get_oversold_threshold(), 30.0);
        assert_eq!(strategy.get_overbought_threshold(), 70.0);

        // Update config
        let mut new_config = StrategyConfig::default();
        new_config
            .parameters
            .insert("rsi_period".to_string(), json!(10));
        new_config
            .parameters
            .insert("oversold_threshold".to_string(), json!(25.0));

        strategy.update_config(new_config);
        assert_eq!(strategy.get_rsi_period(), 10);
        assert_eq!(strategy.get_oversold_threshold(), 25.0);
    }

    #[tokio::test]
    async fn test_rsi_strategy_required_timeframes() {
        let strategy = RsiStrategy::new();
        let timeframes = strategy.required_timeframes();

        assert_eq!(timeframes.len(), 2);
        assert!(timeframes.contains(&"1h"));
        assert!(timeframes.contains(&"4h"));
    }

    #[tokio::test]
    async fn test_rsi_strategy_validate_data_success() {
        let strategy = RsiStrategy::new();
        let prices: Vec<f64> = (0..30).map(|i| 100.0 + (i as f64)).collect();
        let input = create_test_input(prices.clone(), prices);

        let result = strategy.validate_data(&input);
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_rsi_strategy_validate_data_insufficient() {
        let strategy = RsiStrategy::new();
        let prices: Vec<f64> = (0..10).map(|i| 100.0 + (i as f64)).collect();
        let input = create_test_input(prices.clone(), prices);

        let result = strategy.validate_data(&input);
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_rsi_strategy_validate_data_missing_timeframe() {
        let strategy = RsiStrategy::new();
        let mut timeframe_data = HashMap::new();
        let prices: Vec<f64> = (0..30).map(|i| 100.0 + (i as f64)).collect();
        timeframe_data.insert("1h".to_string(), create_test_candles(prices));

        let input = StrategyInput {
            symbol: "BTCUSDT".to_string(),
            timeframe_data,
            current_price: 50000.0,
            volume_24h: 1000000.0,
            timestamp: 1234567890,
        };

        let result = strategy.validate_data(&input);
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_rsi_strategy_metadata() {
        let strategy = RsiStrategy::new();
        let prices: Vec<f64> = (0..25).map(|i| 100.0 + (i as f64 * 0.5)).collect();
        let input = create_test_input(prices.clone(), prices);

        let result = strategy.analyze(&input).await;
        assert!(result.is_ok());

        let output = result.unwrap();
        assert!(output.metadata.contains_key("rsi_1h"));
        assert!(output.metadata.contains_key("rsi_4h"));
        assert!(output.metadata.contains_key("prev_rsi_1h"));
        assert!(output.metadata.contains_key("prev_rsi_4h"));
        assert!(output.metadata.contains_key("oversold_threshold"));
        assert!(output.metadata.contains_key("overbought_threshold"));
    }

    #[test]
    fn test_analyze_rsi_signals_extreme_oversold() {
        let strategy = RsiStrategy::new();
        let (signal, confidence, _) =
            strategy.analyze_rsi_signals(15.0, 25.0, 10.0, 20.0, 30.0, 70.0, 20.0, 80.0);

        assert_eq!(signal, TradingSignal::Long);
        assert_eq!(confidence, 0.87);
    }

    #[test]
    fn test_analyze_rsi_signals_extreme_overbought() {
        let strategy = RsiStrategy::new();
        let (signal, confidence, _) =
            strategy.analyze_rsi_signals(85.0, 75.0, 90.0, 80.0, 30.0, 70.0, 20.0, 80.0);

        assert_eq!(signal, TradingSignal::Short);
        assert_eq!(confidence, 0.87);
    }

    #[test]
    fn test_analyze_rsi_signals_moderate_bullish() {
        let strategy = RsiStrategy::new();
        let (signal, confidence, _) =
            strategy.analyze_rsi_signals(28.0, 45.0, 25.0, 40.0, 30.0, 70.0, 20.0, 80.0);

        assert_eq!(signal, TradingSignal::Long);
        assert_eq!(confidence, 0.73);
    }

    #[test]
    fn test_analyze_rsi_signals_moderate_bearish() {
        let strategy = RsiStrategy::new();
        let (signal, confidence, _) =
            strategy.analyze_rsi_signals(72.0, 55.0, 75.0, 60.0, 30.0, 70.0, 20.0, 80.0);

        assert_eq!(signal, TradingSignal::Short);
        assert_eq!(confidence, 0.73);
    }

    #[test]
    fn test_analyze_rsi_signals_neutral() {
        let strategy = RsiStrategy::new();
        let (signal, confidence, _) =
            strategy.analyze_rsi_signals(50.0, 50.0, 48.0, 52.0, 30.0, 70.0, 20.0, 80.0);

        assert_eq!(signal, TradingSignal::Neutral);
        assert!(confidence > 0.4);
    }

    #[tokio::test]
    async fn test_rsi_strategy_description() {
        let strategy = RsiStrategy::new();
        let desc = strategy.description();

        assert!(desc.contains("RSI"));
        assert!(!desc.is_empty());
    }

    #[tokio::test]
    async fn test_rsi_strategy_with_custom_config() {
        let mut config = StrategyConfig::default();
        config.enabled = true;
        config.weight = 1.5;
        config
            .parameters
            .insert("rsi_period".to_string(), json!(21));
        config
            .parameters
            .insert("oversold_threshold".to_string(), json!(35.0));
        config
            .parameters
            .insert("overbought_threshold".to_string(), json!(65.0));

        let strategy = RsiStrategy::with_config(config);

        assert_eq!(strategy.get_rsi_period(), 21);
        assert_eq!(strategy.get_oversold_threshold(), 35.0);
        assert_eq!(strategy.get_overbought_threshold(), 65.0);
        assert_eq!(strategy.config().weight, 1.5);
    }

    #[test]
    fn test_rsi_strategy_default() {
        let strategy = RsiStrategy::default();
        assert_eq!(strategy.name(), "RSI Strategy");
        assert_eq!(strategy.get_rsi_period(), 14);
    }

    #[test]
    fn test_get_rsi_period_default() {
        let strategy = RsiStrategy::new();
        assert_eq!(strategy.get_rsi_period(), 14);
    }

    #[test]
    fn test_get_rsi_period_custom() {
        let mut config = StrategyConfig::default();
        config.parameters.insert("rsi_period".to_string(), json!(7));
        let strategy = RsiStrategy::with_config(config);
        assert_eq!(strategy.get_rsi_period(), 7);
    }

    #[test]
    fn test_get_rsi_period_missing_fallback() {
        let config = StrategyConfig::default();
        let strategy = RsiStrategy::with_config(config);
        assert_eq!(strategy.get_rsi_period(), 14);
    }

    #[test]
    fn test_get_oversold_threshold_default() {
        let strategy = RsiStrategy::new();
        assert_eq!(strategy.get_oversold_threshold(), 30.0);
    }

    #[test]
    fn test_get_oversold_threshold_custom() {
        let mut config = StrategyConfig::default();
        config
            .parameters
            .insert("oversold_threshold".to_string(), json!(25.0));
        let strategy = RsiStrategy::with_config(config);
        assert_eq!(strategy.get_oversold_threshold(), 25.0);
    }

    #[test]
    fn test_get_oversold_threshold_missing_fallback() {
        let config = StrategyConfig::default();
        let strategy = RsiStrategy::with_config(config);
        assert_eq!(strategy.get_oversold_threshold(), 30.0);
    }

    #[test]
    fn test_get_overbought_threshold_default() {
        let strategy = RsiStrategy::new();
        assert_eq!(strategy.get_overbought_threshold(), 70.0);
    }

    #[test]
    fn test_get_overbought_threshold_custom() {
        let mut config = StrategyConfig::default();
        config
            .parameters
            .insert("overbought_threshold".to_string(), json!(75.0));
        let strategy = RsiStrategy::with_config(config);
        assert_eq!(strategy.get_overbought_threshold(), 75.0);
    }

    #[test]
    fn test_get_overbought_threshold_missing_fallback() {
        let config = StrategyConfig::default();
        let strategy = RsiStrategy::with_config(config);
        assert_eq!(strategy.get_overbought_threshold(), 70.0);
    }

    #[test]
    fn test_get_extreme_oversold_default() {
        let strategy = RsiStrategy::new();
        assert_eq!(strategy.get_extreme_oversold(), 20.0);
    }

    #[test]
    fn test_get_extreme_oversold_custom() {
        let mut config = StrategyConfig::default();
        config
            .parameters
            .insert("extreme_oversold".to_string(), json!(15.0));
        let strategy = RsiStrategy::with_config(config);
        assert_eq!(strategy.get_extreme_oversold(), 15.0);
    }

    #[test]
    fn test_get_extreme_oversold_missing_fallback() {
        let config = StrategyConfig::default();
        let strategy = RsiStrategy::with_config(config);
        assert_eq!(strategy.get_extreme_oversold(), 20.0);
    }

    #[test]
    fn test_get_extreme_overbought_default() {
        let strategy = RsiStrategy::new();
        assert_eq!(strategy.get_extreme_overbought(), 80.0);
    }

    #[test]
    fn test_get_extreme_overbought_custom() {
        let mut config = StrategyConfig::default();
        config
            .parameters
            .insert("extreme_overbought".to_string(), json!(85.0));
        let strategy = RsiStrategy::with_config(config);
        assert_eq!(strategy.get_extreme_overbought(), 85.0);
    }

    #[test]
    fn test_get_extreme_overbought_missing_fallback() {
        let config = StrategyConfig::default();
        let strategy = RsiStrategy::with_config(config);
        assert_eq!(strategy.get_extreme_overbought(), 80.0);
    }

    #[test]
    fn test_rsi_strategy_config_getter() {
        let strategy = RsiStrategy::new();
        let config = strategy.config();
        assert!(config.enabled);
        assert_eq!(config.weight, 1.0);
        assert!(config.parameters.contains_key("rsi_period"));
    }

    #[test]
    fn test_rsi_strategy_update_config() {
        let mut strategy = RsiStrategy::new();
        let mut new_config = StrategyConfig::default();
        new_config.enabled = false;
        new_config.weight = 2.5;
        new_config
            .parameters
            .insert("rsi_period".to_string(), json!(20));

        strategy.update_config(new_config.clone());
        assert_eq!(strategy.config().enabled, false);
        assert_eq!(strategy.config().weight, 2.5);
        assert_eq!(strategy.get_rsi_period(), 20);
    }

    #[test]
    fn test_analyze_rsi_signals_weak_bullish() {
        let strategy = RsiStrategy::new();
        let (signal, confidence, _) =
            strategy.analyze_rsi_signals(35.0, 45.0, 33.0, 43.0, 30.0, 70.0, 20.0, 80.0);

        assert_eq!(signal, TradingSignal::Long);
        assert_eq!(confidence, 0.51);
    }

    #[test]
    fn test_analyze_rsi_signals_weak_bearish() {
        let strategy = RsiStrategy::new();
        let (signal, confidence, _) =
            strategy.analyze_rsi_signals(65.0, 55.0, 68.0, 58.0, 30.0, 70.0, 20.0, 80.0);

        assert_eq!(signal, TradingSignal::Short);
        assert_eq!(confidence, 0.51);
    }

    #[test]
    fn test_analyze_rsi_signals_neutral_high_confidence() {
        let strategy = RsiStrategy::new();
        let (signal, confidence, _) =
            strategy.analyze_rsi_signals(48.0, 52.0, 47.0, 51.0, 30.0, 70.0, 20.0, 80.0);

        assert_eq!(signal, TradingSignal::Neutral);
        assert_eq!(confidence, 0.65);
    }

    #[test]
    fn test_analyze_rsi_signals_neutral_low_confidence() {
        let strategy = RsiStrategy::new();
        let (signal, confidence, _) =
            strategy.analyze_rsi_signals(40.0, 65.0, 42.0, 63.0, 30.0, 70.0, 20.0, 80.0);

        assert_eq!(signal, TradingSignal::Neutral);
        assert_eq!(confidence, 0.45);
    }

    #[test]
    fn test_analyze_rsi_signals_edge_case_exact_oversold() {
        let strategy = RsiStrategy::new();
        let (signal, confidence, _) =
            strategy.analyze_rsi_signals(30.0, 45.0, 28.0, 43.0, 30.0, 70.0, 20.0, 80.0);

        assert_eq!(signal, TradingSignal::Long);
        assert_eq!(confidence, 0.73);
    }

    #[test]
    fn test_analyze_rsi_signals_edge_case_exact_overbought() {
        let strategy = RsiStrategy::new();
        let (signal, confidence, _) =
            strategy.analyze_rsi_signals(70.0, 55.0, 72.0, 58.0, 30.0, 70.0, 20.0, 80.0);

        assert_eq!(signal, TradingSignal::Short);
        assert_eq!(confidence, 0.73);
    }

    #[test]
    fn test_analyze_rsi_signals_edge_case_exact_extreme_oversold() {
        let strategy = RsiStrategy::new();
        let (signal, confidence, _) =
            strategy.analyze_rsi_signals(20.0, 30.0, 18.0, 28.0, 30.0, 70.0, 20.0, 80.0);

        assert_eq!(signal, TradingSignal::Long);
        assert_eq!(confidence, 0.87);
    }

    #[test]
    fn test_analyze_rsi_signals_edge_case_exact_extreme_overbought() {
        let strategy = RsiStrategy::new();
        let (signal, confidence, _) =
            strategy.analyze_rsi_signals(80.0, 70.0, 82.0, 72.0, 30.0, 70.0, 20.0, 80.0);

        assert_eq!(signal, TradingSignal::Short);
        assert_eq!(confidence, 0.87);
    }

    #[tokio::test]
    async fn test_rsi_strategy_empty_candles() {
        let strategy = RsiStrategy::new();
        let input = create_test_input(vec![], vec![]);

        let result = strategy.validate_data(&input);
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_rsi_strategy_single_candle() {
        let strategy = RsiStrategy::new();
        let input = create_test_input(vec![100.0], vec![100.0]);

        let result = strategy.validate_data(&input);
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_rsi_strategy_exact_minimum_candles() {
        let strategy = RsiStrategy::new();
        let prices: Vec<f64> = (0..19).map(|i| 100.0 + (i as f64)).collect();
        let input = create_test_input(prices.clone(), prices);

        let result = strategy.validate_data(&input);
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_rsi_strategy_analyze_missing_1h_data() {
        let strategy = RsiStrategy::new();
        let mut timeframe_data = HashMap::new();
        let prices: Vec<f64> = (0..30).map(|i| 100.0 + (i as f64)).collect();
        timeframe_data.insert("4h".to_string(), create_test_candles(prices));

        let input = StrategyInput {
            symbol: "BTCUSDT".to_string(),
            timeframe_data,
            current_price: 50000.0,
            volume_24h: 1000000.0,
            timestamp: 1234567890,
        };

        let result = strategy.analyze(&input).await;
        assert!(result.is_err());
        match result {
            Err(StrategyError::InsufficientData(msg)) => {
                assert!(msg.contains("1h"));
            },
            _ => panic!("Expected InsufficientData error"),
        }
    }

    #[tokio::test]
    async fn test_rsi_strategy_analyze_missing_4h_data() {
        let strategy = RsiStrategy::new();
        let mut timeframe_data = HashMap::new();
        let prices: Vec<f64> = (0..30).map(|i| 100.0 + (i as f64)).collect();
        timeframe_data.insert("1h".to_string(), create_test_candles(prices));

        let input = StrategyInput {
            symbol: "BTCUSDT".to_string(),
            timeframe_data,
            current_price: 50000.0,
            volume_24h: 1000000.0,
            timestamp: 1234567890,
        };

        let result = strategy.analyze(&input).await;
        assert!(result.is_err());
        match result {
            Err(StrategyError::InsufficientData(msg)) => {
                assert!(msg.contains("4h"));
            },
            _ => panic!("Expected InsufficientData error"),
        }
    }

    #[tokio::test]
    async fn test_rsi_strategy_output_timestamp() {
        let strategy = RsiStrategy::new();
        let prices: Vec<f64> = (0..25).map(|i| 100.0 + (i as f64 * 0.5)).collect();
        let mut input = create_test_input(prices.clone(), prices);
        input.timestamp = 9876543210;

        let result = strategy.analyze(&input).await;
        assert!(result.is_ok());

        let output = result.unwrap();
        assert_eq!(output.timestamp, 9876543210);
    }

    #[tokio::test]
    async fn test_rsi_strategy_output_timeframe() {
        let strategy = RsiStrategy::new();
        let prices: Vec<f64> = (0..25).map(|i| 100.0 + (i as f64 * 0.5)).collect();
        let input = create_test_input(prices.clone(), prices);

        let result = strategy.analyze(&input).await;
        assert!(result.is_ok());

        let output = result.unwrap();
        assert_eq!(output.timeframe, "1h");
    }

    #[test]
    fn test_analyze_rsi_signals_reasoning_extreme_oversold() {
        let strategy = RsiStrategy::new();
        let (_, _, reasoning) =
            strategy.analyze_rsi_signals(15.0, 25.0, 10.0, 20.0, 30.0, 70.0, 20.0, 80.0);

        assert!(reasoning.contains("bullish"));
        assert!(reasoning.contains("oversold"));
    }

    #[test]
    fn test_analyze_rsi_signals_reasoning_extreme_overbought() {
        let strategy = RsiStrategy::new();
        let (_, _, reasoning) =
            strategy.analyze_rsi_signals(85.0, 75.0, 90.0, 80.0, 30.0, 70.0, 20.0, 80.0);

        assert!(reasoning.contains("bearish"));
        assert!(reasoning.contains("overbought"));
    }

    #[test]
    fn test_analyze_rsi_signals_reasoning_neutral() {
        let strategy = RsiStrategy::new();
        let (_, _, reasoning) =
            strategy.analyze_rsi_signals(50.0, 50.0, 48.0, 52.0, 30.0, 70.0, 20.0, 80.0);

        assert!(reasoning.contains("Consolidation"));
    }

    #[tokio::test]
    async fn test_rsi_strategy_high_volatility_bullish() {
        let strategy = RsiStrategy::new();

        let mut prices_1h: Vec<f64> = (0..15).map(|i| 100.0 - (i as f64 * 3.0)).collect();
        prices_1h.extend((0..10).map(|i| 55.0 + (i as f64 * 2.0)));

        let prices_4h: Vec<f64> = (0..25).map(|i| 100.0 - (i as f64 * 1.0)).collect();

        let input = create_test_input(prices_1h, prices_4h);
        let result = strategy.analyze(&input).await;

        assert!(result.is_ok());
        let output = result.unwrap();
        assert!(output.signal == TradingSignal::Long || output.signal == TradingSignal::Neutral);
    }

    #[tokio::test]
    async fn test_rsi_strategy_high_volatility_bearish() {
        let strategy = RsiStrategy::new();

        let mut prices_1h: Vec<f64> = (0..15).map(|i| 100.0 + (i as f64 * 3.0)).collect();
        prices_1h.extend((0..10).map(|i| 145.0 - (i as f64 * 2.0)));

        let prices_4h: Vec<f64> = (0..25).map(|i| 100.0 + (i as f64 * 1.0)).collect();

        let input = create_test_input(prices_1h, prices_4h);
        let result = strategy.analyze(&input).await;

        assert!(result.is_ok());
        let output = result.unwrap();
        assert!(output.signal == TradingSignal::Short || output.signal == TradingSignal::Neutral);
    }

    #[tokio::test]
    async fn test_rsi_strategy_sideways_market() {
        let strategy = RsiStrategy::new();

        let prices_1h: Vec<f64> = (0..30)
            .map(|i| 100.0 + (((i as f64) * 0.5).sin() * 2.0))
            .collect();
        let prices_4h: Vec<f64> = (0..30)
            .map(|i| 100.0 + (((i as f64) * 0.3).sin() * 1.5))
            .collect();

        let input = create_test_input(prices_1h, prices_4h);
        let result = strategy.analyze(&input).await;

        assert!(result.is_ok());
        let output = result.unwrap();
        assert_eq!(output.signal, TradingSignal::Neutral);
    }

    #[tokio::test]
    async fn test_rsi_strategy_extreme_price_values() {
        let strategy = RsiStrategy::new();

        let prices_1h: Vec<f64> = (0..25).map(|i| 100000.0 + (i as f64 * 100.0)).collect();
        let prices_4h: Vec<f64> = (0..25).map(|i| 100000.0 + (i as f64 * 50.0)).collect();

        let input = create_test_input(prices_1h, prices_4h);
        let result = strategy.analyze(&input).await;

        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_rsi_strategy_minimal_price_values() {
        let strategy = RsiStrategy::new();

        let prices_1h: Vec<f64> = (0..25).map(|i| 0.001 + (i as f64 * 0.0001)).collect();
        let prices_4h: Vec<f64> = (0..25).map(|i| 0.001 + (i as f64 * 0.00005)).collect();

        let input = create_test_input(prices_1h, prices_4h);
        let result = strategy.analyze(&input).await;

        assert!(result.is_ok());
    }

    #[test]
    fn test_analyze_rsi_signals_all_parameters_boundary() {
        let strategy = RsiStrategy::new();

        let (signal, _, _) =
            strategy.analyze_rsi_signals(30.0, 30.0, 30.0, 30.0, 30.0, 70.0, 20.0, 80.0);
        assert!(signal == TradingSignal::Long || signal == TradingSignal::Neutral);

        let (signal, _, _) =
            strategy.analyze_rsi_signals(70.0, 70.0, 70.0, 70.0, 30.0, 70.0, 20.0, 80.0);
        assert!(signal == TradingSignal::Short || signal == TradingSignal::Neutral);
    }

    #[test]
    fn test_analyze_rsi_signals_zero_values() {
        let strategy = RsiStrategy::new();
        let (signal, confidence, _) =
            strategy.analyze_rsi_signals(0.0, 0.0, 0.0, 0.0, 30.0, 70.0, 20.0, 80.0);

        assert_eq!(signal, TradingSignal::Long);
        assert_eq!(confidence, 0.87);
    }

    #[test]
    fn test_analyze_rsi_signals_hundred_values() {
        let strategy = RsiStrategy::new();
        let (signal, confidence, _) =
            strategy.analyze_rsi_signals(100.0, 100.0, 100.0, 100.0, 30.0, 70.0, 20.0, 80.0);

        assert_eq!(signal, TradingSignal::Short);
        assert_eq!(confidence, 0.87);
    }

    #[tokio::test]
    async fn test_rsi_strategy_consistent_uptrend() {
        let strategy = RsiStrategy::new();

        let prices_1h: Vec<f64> = (0..25).map(|i| 100.0 + (i as f64 * 5.0)).collect();
        let prices_4h: Vec<f64> = (0..25).map(|i| 100.0 + (i as f64 * 3.0)).collect();

        let input = create_test_input(prices_1h, prices_4h);
        let result = strategy.analyze(&input).await;

        assert!(result.is_ok());
        let output = result.unwrap();
        assert!(output.confidence >= 0.0 && output.confidence <= 1.0);
    }

    #[tokio::test]
    async fn test_rsi_strategy_consistent_downtrend() {
        let strategy = RsiStrategy::new();

        let prices_1h: Vec<f64> = (0..25).map(|i| 200.0 - (i as f64 * 5.0)).collect();
        let prices_4h: Vec<f64> = (0..25).map(|i| 200.0 - (i as f64 * 3.0)).collect();

        let input = create_test_input(prices_1h, prices_4h);
        let result = strategy.analyze(&input).await;

        assert!(result.is_ok());
        let output = result.unwrap();
        assert!(output.confidence >= 0.0 && output.confidence <= 1.0);
    }

    #[tokio::test]
    async fn test_rsi_strategy_name_consistency() {
        let strategy1 = RsiStrategy::new();
        let strategy2 = RsiStrategy::default();
        assert_eq!(strategy1.name(), strategy2.name());
    }

    #[tokio::test]
    async fn test_rsi_strategy_description_not_empty() {
        let strategy = RsiStrategy::new();
        assert!(!strategy.description().is_empty());
        assert!(strategy.description().len() > 10);
    }

    #[test]
    fn test_rsi_strategy_clone() {
        let strategy1 = RsiStrategy::new();
        let strategy2 = strategy1.clone();

        assert_eq!(strategy1.name(), strategy2.name());
        assert_eq!(strategy1.get_rsi_period(), strategy2.get_rsi_period());
        assert_eq!(
            strategy1.get_oversold_threshold(),
            strategy2.get_oversold_threshold()
        );
    }

    #[tokio::test]
    async fn test_rsi_strategy_metadata_values_valid() {
        let strategy = RsiStrategy::new();
        let prices: Vec<f64> = (0..25).map(|i| 100.0 + (i as f64 * 0.5)).collect();
        let input = create_test_input(prices.clone(), prices);

        let result = strategy.analyze(&input).await;
        assert!(result.is_ok());

        let output = result.unwrap();
        let rsi_1h = output.metadata.get("rsi_1h").unwrap().as_f64().unwrap();
        let rsi_4h = output.metadata.get("rsi_4h").unwrap().as_f64().unwrap();

        assert!(rsi_1h >= 0.0 && rsi_1h <= 100.0);
        assert!(rsi_4h >= 0.0 && rsi_4h <= 100.0);
    }

    #[test]
    fn test_analyze_rsi_signals_confidence_bounds() {
        let strategy = RsiStrategy::new();

        let test_cases = vec![
            (15.0, 25.0, 10.0, 20.0),
            (85.0, 75.0, 90.0, 80.0),
            (28.0, 45.0, 25.0, 40.0),
            (72.0, 55.0, 75.0, 60.0),
            (50.0, 50.0, 48.0, 52.0),
            (35.0, 45.0, 33.0, 43.0),
        ];

        for (rsi_1h, rsi_4h, prev_1h, prev_4h) in test_cases {
            let (_, confidence, _) = strategy
                .analyze_rsi_signals(rsi_1h, rsi_4h, prev_1h, prev_4h, 30.0, 70.0, 20.0, 80.0);
            assert!(confidence >= 0.0 && confidence <= 1.0);
        }
    }
}
