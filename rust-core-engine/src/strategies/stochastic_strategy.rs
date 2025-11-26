use super::*;
use crate::strategies::indicators::calculate_stochastic;
use async_trait::async_trait;
use serde_json::json;

/// Stochastic Oscillator-based trading strategy
///
/// Uses %K and %D lines to identify overbought/oversold conditions
/// and generate trading signals based on crossovers

// @spec:FR-STRATEGIES-005 - Stochastic Oscillator Trading Strategy
// @ref:specs/01-requirements/1.1-functional-requirements/FR-STRATEGIES.md#fr-strategies-005
// @ref:specs/02-design/2.5-components/COMP-RUST-TRADING.md#strategies
// @test:TC-AI-016

#[derive(Debug, Clone)]
pub struct StochasticStrategy {
    config: StrategyConfig,
}

impl StochasticStrategy {
    pub fn new() -> Self {
        let mut config = StrategyConfig::default();
        config.parameters.insert("k_period".to_string(), json!(14));
        config.parameters.insert("d_period".to_string(), json!(3));
        // @spec:FR-STRATEGIES-005 - Stochastic Strategy optimized thresholds
        // @ref:docs/features/how-it-works.md - Stochastic: "%K vùng oversold (<15)", "%K vùng overbought (>85)"
        config
            .parameters
            .insert("oversold_threshold".to_string(), json!(15.0));  // FIXED: Match docs - 15 (not 20)
        config
            .parameters
            .insert("overbought_threshold".to_string(), json!(85.0)); // FIXED: Match docs - 85 (not 80)
        config
            .parameters
            .insert("extreme_oversold".to_string(), json!(10.0));
        config
            .parameters
            .insert("extreme_overbought".to_string(), json!(90.0));

        Self { config }
    }

    pub fn with_config(config: StrategyConfig) -> Self {
        Self { config }
    }

    fn get_k_period(&self) -> usize {
        self.config
            .parameters
            .get("k_period")
            .and_then(|v| v.as_u64())
            .unwrap_or(14) as usize
    }

    fn get_d_period(&self) -> usize {
        self.config
            .parameters
            .get("d_period")
            .and_then(|v| v.as_u64())
            .unwrap_or(3) as usize
    }

    fn get_oversold_threshold(&self) -> f64 {
        self.config
            .parameters
            .get("oversold_threshold")
            .and_then(|v| v.as_f64())
            .unwrap_or(20.0)
    }

    fn get_overbought_threshold(&self) -> f64 {
        self.config
            .parameters
            .get("overbought_threshold")
            .and_then(|v| v.as_f64())
            .unwrap_or(80.0)
    }

    fn get_extreme_oversold(&self) -> f64 {
        self.config
            .parameters
            .get("extreme_oversold")
            .and_then(|v| v.as_f64())
            .unwrap_or(10.0)
    }

    fn get_extreme_overbought(&self) -> f64 {
        self.config
            .parameters
            .get("extreme_overbought")
            .and_then(|v| v.as_f64())
            .unwrap_or(90.0)
    }
}

#[async_trait]
impl Strategy for StochasticStrategy {
    fn name(&self) -> &'static str {
        "Stochastic Strategy"
    }

    fn description(&self) -> &'static str {
        "Stochastic Oscillator strategy that identifies overbought/oversold conditions and crossover signals"
    }

    fn required_timeframes(&self) -> Vec<&'static str> {
        vec!["1h", "4h"]
    }

    // @spec:FR-STRATEGIES-005 - Stochastic Oscillator Trading Strategy (Multi-Timeframe Analysis)
    // @ref:specs/01-requirements/1.1-functional-requirements/FR-STRATEGIES.md#fr-strategies-005
    // @test:TC-AI-016
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

        let k_period = self.get_k_period();
        let d_period = self.get_d_period();

        // Calculate Stochastic for both timeframes
        let primary_stoch = calculate_stochastic(primary_candles, k_period, d_period)
            .map_err(StrategyError::CalculationError)?;

        let confirmation_stoch = calculate_stochastic(confirmation_candles, k_period, d_period)
            .map_err(StrategyError::CalculationError)?;

        // Get current values
        let current_k_1h = *primary_stoch.k_percent.last().ok_or_else(|| {
            StrategyError::InsufficientData("No Stochastic K values calculated for 1h".to_string())
        })?;
        let current_d_1h = *primary_stoch.d_percent.last().ok_or_else(|| {
            StrategyError::InsufficientData("No Stochastic D values calculated for 1h".to_string())
        })?;

        let current_k_4h = *confirmation_stoch.k_percent.last().ok_or_else(|| {
            StrategyError::InsufficientData("No Stochastic K values calculated for 4h".to_string())
        })?;
        let current_d_4h = *confirmation_stoch.d_percent.last().ok_or_else(|| {
            StrategyError::InsufficientData("No Stochastic D values calculated for 4h".to_string())
        })?;

        // Get previous values for crossover detection
        let prev_k_1h = if primary_stoch.k_percent.len() > 1 {
            primary_stoch.k_percent[primary_stoch.k_percent.len() - 2]
        } else {
            current_k_1h
        };
        let prev_d_1h = if primary_stoch.d_percent.len() > 1 {
            primary_stoch.d_percent[primary_stoch.d_percent.len() - 2]
        } else {
            current_d_1h
        };

        let prev_k_4h = if confirmation_stoch.k_percent.len() > 1 {
            confirmation_stoch.k_percent[confirmation_stoch.k_percent.len() - 2]
        } else {
            current_k_4h
        };
        let prev_d_4h = if confirmation_stoch.d_percent.len() > 1 {
            confirmation_stoch.d_percent[confirmation_stoch.d_percent.len() - 2]
        } else {
            current_d_4h
        };

        let oversold = self.get_oversold_threshold();
        let overbought = self.get_overbought_threshold();
        let extreme_oversold = self.get_extreme_oversold();
        let extreme_overbought = self.get_extreme_overbought();

        // Determine signal and confidence
        let (signal, confidence, reasoning) = self.analyze_stochastic_signals(
            current_k_1h,
            current_d_1h,
            current_k_4h,
            current_d_4h,
            prev_k_1h,
            prev_d_1h,
            prev_k_4h,
            prev_d_4h,
            oversold,
            overbought,
            extreme_oversold,
            extreme_overbought,
        );

        let mut metadata = std::collections::HashMap::new();
        metadata.insert("stoch_k_1h".to_string(), json!(current_k_1h));
        metadata.insert("stoch_d_1h".to_string(), json!(current_d_1h));
        metadata.insert("stoch_k_4h".to_string(), json!(current_k_4h));
        metadata.insert("stoch_d_4h".to_string(), json!(current_d_4h));
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
                StrategyError::InsufficientData(format!("Missing {timeframe} timeframe data"))
            })?;

            let min_required = self.get_k_period() + self.get_d_period() + 5;

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

impl StochasticStrategy {
    #[allow(clippy::too_many_arguments)]
    fn analyze_stochastic_signals(
        &self,
        k_1h: f64,
        d_1h: f64,
        k_4h: f64,
        _d_4h: f64,
        prev_k_1h: f64,
        prev_d_1h: f64,
        _prev_k_4h: f64,
        _prev_d_4h: f64,
        oversold: f64,
        overbought: f64,
        extreme_oversold: f64,
        extreme_overbought: f64,
    ) -> (TradingSignal, f64, String) {
        // Detect crossovers
        let bullish_crossover_1h = prev_k_1h <= prev_d_1h && k_1h > d_1h;
        let bearish_crossover_1h = prev_k_1h >= prev_d_1h && k_1h < d_1h;

        // Strong bullish signals - bullish crossover in oversold zone
        if bullish_crossover_1h && k_1h <= oversold && k_4h <= oversold {
            return (
                TradingSignal::Long,
                0.89,
                "Strong bullish crossover in extreme oversold zone".to_string(),
            );
        }

        // Extreme bullish - very oversold on both timeframes
        if k_1h <= extreme_oversold && k_4h <= oversold && k_1h > d_1h {
            return (
                TradingSignal::Long,
                0.85,
                "Extreme oversold with bullish momentum".to_string(),
            );
        }

        // Strong bearish signals - bearish crossover in overbought zone
        if bearish_crossover_1h && k_1h >= overbought && k_4h >= overbought {
            return (
                TradingSignal::Short,
                0.89,
                "Strong bearish crossover in extreme overbought zone".to_string(),
            );
        }

        // Extreme bearish - very overbought on both timeframes
        if k_1h >= extreme_overbought && k_4h >= overbought && k_1h < d_1h {
            return (
                TradingSignal::Short,
                0.85,
                "Extreme overbought with bearish momentum".to_string(),
            );
        }

        // Moderate bullish signals - crossover near oversold
        if bullish_crossover_1h && k_1h <= oversold + 10.0 && k_4h < 50.0 {
            return (
                TradingSignal::Long,
                0.72,
                "Bullish crossover recovery from oversold".to_string(),
            );
        }

        // Moderate bearish signals - crossover near overbought
        if bearish_crossover_1h && k_1h >= overbought - 10.0 && k_4h > 50.0 {
            return (
                TradingSignal::Short,
                0.72,
                "Bearish crossover decline from overbought".to_string(),
            );
        }

        // Weak bullish signals - %K above %D in lower half
        if k_1h > d_1h && k_1h < 50.0 && k_4h < 50.0 && prev_k_1h < k_1h {
            return (
                TradingSignal::Long,
                0.52,
                "Weak bullish momentum with stochastic rising".to_string(),
            );
        }

        // Weak bearish signals - %K below %D in upper half
        if k_1h < d_1h && k_1h > 50.0 && k_4h > 50.0 && prev_k_1h > k_1h {
            return (
                TradingSignal::Short,
                0.52,
                "Weak bearish momentum with stochastic falling".to_string(),
            );
        }

        // Neutral/consolidation
        let confidence = if (k_1h - 50.0).abs() < 15.0 && (k_4h - 50.0).abs() < 20.0 {
            0.63
        } else {
            0.47
        };

        (
            TradingSignal::Neutral,
            confidence,
            "Consolidation phase, no clear crossover signals".to_string(),
        )
    }
}

impl Default for StochasticStrategy {
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
                high: price * 1.02,
                low: price * 0.98,
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
    async fn test_stochastic_strategy_new() {
        let strategy = StochasticStrategy::new();
        assert_eq!(strategy.name(), "Stochastic Strategy");
        assert!(strategy.config().enabled);
        assert_eq!(strategy.config().weight, 1.0);
    }

    #[tokio::test]
    async fn test_stochastic_strategy_strong_bullish_signal() {
        let strategy = StochasticStrategy::new();

        // Create strong downtrend then reversal
        let prices_1h: Vec<f64> = (0..30).map(|i| 100.0 - (i as f64 * 3.0)).collect();
        let mut recovery_prices = prices_1h.clone();
        recovery_prices.extend((0..5).map(|i| -90.0 + (i as f64 * 2.0)));

        let prices_4h: Vec<f64> = (0..25).map(|i| 100.0 - (i as f64 * 2.0)).collect();

        let input = create_test_input(recovery_prices, prices_4h);
        let result = strategy.analyze(&input).await;

        if let Err(e) = &result {
            eprintln!("Error in test: {:?}", e);
        }
        assert!(result.is_ok());
        let output = result.unwrap();
        assert!(matches!(
            output.signal,
            TradingSignal::Long | TradingSignal::Neutral
        ));
        assert!(output.confidence > 0.4);
    }

    #[tokio::test]
    async fn test_stochastic_strategy_strong_bearish_signal() {
        let strategy = StochasticStrategy::new();

        // Create strong uptrend then reversal
        let prices_1h: Vec<f64> = (0..30).map(|i| 100.0 + (i as f64 * 3.0)).collect();
        let mut decline_prices = prices_1h.clone();
        decline_prices.extend((0..5).map(|i| 190.0 - (i as f64 * 2.0)));

        let prices_4h: Vec<f64> = (0..25).map(|i| 100.0 + (i as f64 * 2.0)).collect();

        let input = create_test_input(decline_prices, prices_4h);
        let result = strategy.analyze(&input).await;

        if let Err(e) = &result {
            eprintln!("Error in test: {:?}", e);
        }
        assert!(result.is_ok());
        let output = result.unwrap();
        assert!(matches!(
            output.signal,
            TradingSignal::Short | TradingSignal::Neutral
        ));
        assert!(output.confidence > 0.4);
    }

    #[tokio::test]
    async fn test_stochastic_strategy_neutral_signal() {
        let strategy = StochasticStrategy::new();

        // Create sideways market
        let prices_1h: Vec<f64> = (0..30).map(|i| 100.0 + ((i as f64 % 3.0) - 1.0)).collect();
        let prices_4h: Vec<f64> = (0..30).map(|i| 100.0 + ((i as f64 % 2.0) - 0.5)).collect();

        let input = create_test_input(prices_1h, prices_4h);
        let result = strategy.analyze(&input).await;

        assert!(result.is_ok());
        let output = result.unwrap();
        assert_eq!(output.signal, TradingSignal::Neutral);
    }

    #[tokio::test]
    async fn test_stochastic_strategy_configuration() {
        let mut strategy = StochasticStrategy::new();

        assert_eq!(strategy.get_k_period(), 14);
        assert_eq!(strategy.get_d_period(), 3);
        assert_eq!(strategy.get_oversold_threshold(), 15.0);  // FIXED: Match docs - 15
        assert_eq!(strategy.get_overbought_threshold(), 85.0); // FIXED: Match docs - 85

        // Update config
        let mut new_config = StrategyConfig::default();
        new_config
            .parameters
            .insert("k_period".to_string(), json!(10));
        new_config
            .parameters
            .insert("oversold_threshold".to_string(), json!(15.0));

        strategy.update_config(new_config);
        assert_eq!(strategy.get_k_period(), 10);
        assert_eq!(strategy.get_oversold_threshold(), 15.0);
    }

    #[tokio::test]
    async fn test_stochastic_strategy_required_timeframes() {
        let strategy = StochasticStrategy::new();
        let timeframes = strategy.required_timeframes();

        assert_eq!(timeframes.len(), 2);
        assert!(timeframes.contains(&"1h"));
        assert!(timeframes.contains(&"4h"));
    }

    #[tokio::test]
    async fn test_stochastic_strategy_validate_data_success() {
        let strategy = StochasticStrategy::new();
        let prices: Vec<f64> = (0..30).map(|i| 100.0 + (i as f64)).collect();
        let input = create_test_input(prices.clone(), prices);

        let result = strategy.validate_data(&input);
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_stochastic_strategy_validate_data_insufficient() {
        let strategy = StochasticStrategy::new();
        let prices: Vec<f64> = (0..10).map(|i| 100.0 + (i as f64)).collect();
        let input = create_test_input(prices.clone(), prices);

        let result = strategy.validate_data(&input);
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_stochastic_strategy_metadata() {
        let strategy = StochasticStrategy::new();
        let prices: Vec<f64> = (0..30).map(|i| 100.0 + (i as f64 * 0.5)).collect();
        let input = create_test_input(prices.clone(), prices);

        let result = strategy.analyze(&input).await;
        assert!(result.is_ok());

        let output = result.unwrap();
        assert!(output.metadata.contains_key("stoch_k_1h"));
        assert!(output.metadata.contains_key("stoch_d_1h"));
        assert!(output.metadata.contains_key("stoch_k_4h"));
        assert!(output.metadata.contains_key("stoch_d_4h"));
    }

    #[test]
    fn test_stochastic_strategy_default() {
        let strategy = StochasticStrategy::default();
        assert_eq!(strategy.name(), "Stochastic Strategy");
        assert_eq!(strategy.get_k_period(), 14);
    }
}
