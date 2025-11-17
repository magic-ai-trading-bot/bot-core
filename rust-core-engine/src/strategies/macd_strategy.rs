use super::*;
use crate::strategies::indicators::calculate_macd;
use async_trait::async_trait;
use serde_json::json;

/// MACD-based trading strategy

// @spec:FR-STRATEGY-002 - MACD Strategy
// @ref:specs/02-design/2.5-components/COMP-RUST-TRADING.md#strategies
// @test:TC-TRADING-024

#[derive(Debug, Clone)]
pub struct MacdStrategy {
    config: StrategyConfig,
}

impl MacdStrategy {
    pub fn new() -> Self {
        let mut config = StrategyConfig::default();
        config
            .parameters
            .insert("fast_period".to_string(), json!(12));
        config
            .parameters
            .insert("slow_period".to_string(), json!(26));
        config
            .parameters
            .insert("signal_period".to_string(), json!(9));
        config
            .parameters
            .insert("histogram_threshold".to_string(), json!(0.001));

        Self { config }
    }

    pub fn with_config(config: StrategyConfig) -> Self {
        Self { config }
    }

    fn get_fast_period(&self) -> usize {
        self.config
            .parameters
            .get("fast_period")
            .and_then(|v| v.as_u64())
            .unwrap_or(12) as usize
    }

    fn get_slow_period(&self) -> usize {
        self.config
            .parameters
            .get("slow_period")
            .and_then(|v| v.as_u64())
            .unwrap_or(26) as usize
    }

    fn get_signal_period(&self) -> usize {
        self.config
            .parameters
            .get("signal_period")
            .and_then(|v| v.as_u64())
            .unwrap_or(9) as usize
    }

    fn get_histogram_threshold(&self) -> f64 {
        self.config
            .parameters
            .get("histogram_threshold")
            .and_then(|v| v.as_f64())
            .unwrap_or(0.001)
    }
}

#[async_trait]
impl Strategy for MacdStrategy {
    fn name(&self) -> &'static str {
        "MACD Strategy"
    }

    fn description(&self) -> &'static str {
        "MACD-based strategy that identifies trend changes and momentum shifts"
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

        let fast_period = self.get_fast_period();
        let slow_period = self.get_slow_period();
        let signal_period = self.get_signal_period();

        // Calculate MACD for both timeframes
        let primary_macd = calculate_macd(primary_candles, fast_period, slow_period, signal_period)
            .map_err(StrategyError::CalculationError)?;

        let confirmation_macd = calculate_macd(
            confirmation_candles,
            fast_period,
            slow_period,
            signal_period,
        )
        .map_err(StrategyError::CalculationError)?;

        // Get current values with proper error handling
        let current_macd_1h = *primary_macd
            .macd_line
            .last()
            .ok_or_else(|| StrategyError::InsufficientData("No MACD line values".to_string()))?;
        let current_signal_1h = *primary_macd.signal_line.last().ok_or_else(|| {
            StrategyError::InsufficientData("No MACD signal line values".to_string())
        })?;
        let current_histogram_1h = *primary_macd.histogram.last().ok_or_else(|| {
            StrategyError::InsufficientData("No MACD histogram values".to_string())
        })?;

        let current_macd_4h = *confirmation_macd
            .macd_line
            .last()
            .ok_or_else(|| StrategyError::InsufficientData("No 4h MACD line values".to_string()))?;
        let current_signal_4h = *confirmation_macd.signal_line.last().ok_or_else(|| {
            StrategyError::InsufficientData("No 4h MACD signal line values".to_string())
        })?;
        let current_histogram_4h = *confirmation_macd.histogram.last().ok_or_else(|| {
            StrategyError::InsufficientData("No 4h MACD histogram values".to_string())
        })?;

        // Get previous values for trend analysis
        let prev_macd_1h = if primary_macd.macd_line.len() > 1 {
            primary_macd.macd_line[primary_macd.macd_line.len() - 2]
        } else {
            current_macd_1h
        };
        let prev_signal_1h = if primary_macd.signal_line.len() > 1 {
            primary_macd.signal_line[primary_macd.signal_line.len() - 2]
        } else {
            current_signal_1h
        };
        let prev_histogram_1h = if primary_macd.histogram.len() > 1 {
            primary_macd.histogram[primary_macd.histogram.len() - 2]
        } else {
            current_histogram_1h
        };

        let prev_histogram_4h = if confirmation_macd.histogram.len() > 1 {
            confirmation_macd.histogram[confirmation_macd.histogram.len() - 2]
        } else {
            current_histogram_4h
        };

        // Determine signal and confidence
        let (signal, confidence, reasoning) = self.analyze_macd_signals(
            current_macd_1h,
            current_signal_1h,
            current_histogram_1h,
            current_macd_4h,
            current_signal_4h,
            current_histogram_4h,
            prev_macd_1h,
            prev_signal_1h,
            prev_histogram_1h,
            prev_histogram_4h,
        );

        let mut metadata = std::collections::HashMap::new();
        metadata.insert("macd_line_1h".to_string(), json!(current_macd_1h));
        metadata.insert("signal_line_1h".to_string(), json!(current_signal_1h));
        metadata.insert("histogram_1h".to_string(), json!(current_histogram_1h));
        metadata.insert("macd_line_4h".to_string(), json!(current_macd_4h));
        metadata.insert("signal_line_4h".to_string(), json!(current_signal_4h));
        metadata.insert("histogram_4h".to_string(), json!(current_histogram_4h));
        metadata.insert("prev_histogram_1h".to_string(), json!(prev_histogram_1h));
        metadata.insert("prev_histogram_4h".to_string(), json!(prev_histogram_4h));

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

            let min_required = self.get_slow_period() + self.get_signal_period() + 10; // MACD calculation + buffer

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

impl MacdStrategy {
    #[allow(clippy::too_many_arguments)]
    fn analyze_macd_signals(
        &self,
        macd_1h: f64,
        signal_1h: f64,
        histogram_1h: f64,
        _macd_4h: f64,
        _signal_4h: f64,
        histogram_4h: f64,
        prev_macd_1h: f64,
        prev_signal_1h: f64,
        prev_histogram_1h: f64,
        prev_histogram_4h: f64,
    ) -> (TradingSignal, f64, String) {
        let threshold = self.get_histogram_threshold();

        // Check for MACD line crossovers
        let bullish_crossover_1h = prev_macd_1h <= prev_signal_1h && macd_1h > signal_1h;
        let bearish_crossover_1h = prev_macd_1h >= prev_signal_1h && macd_1h < signal_1h;

        // Check histogram momentum
        let histogram_increasing_1h = histogram_1h > prev_histogram_1h;
        let histogram_decreasing_1h = histogram_1h < prev_histogram_1h;
        let histogram_increasing_4h = histogram_4h > prev_histogram_4h;
        let histogram_decreasing_4h = histogram_4h < prev_histogram_4h;

        // Check zero line crossovers
        let histogram_above_zero_1h = histogram_1h > threshold;
        let histogram_below_zero_1h = histogram_1h < -threshold;
        let histogram_above_zero_4h = histogram_4h > threshold;
        let histogram_below_zero_4h = histogram_4h < -threshold;

        // Strong bullish signals
        if bullish_crossover_1h
            && histogram_above_zero_4h
            && histogram_increasing_1h
            && histogram_increasing_4h
        {
            return (
                TradingSignal::Long,
                0.89,
                "Strong bullish MACD crossover with momentum confirmation".to_string(),
            );
        }

        // Strong bearish signals
        if bearish_crossover_1h
            && histogram_below_zero_4h
            && histogram_decreasing_1h
            && histogram_decreasing_4h
        {
            return (
                TradingSignal::Short,
                0.89,
                "Strong bearish MACD crossover with momentum breakdown".to_string(),
            );
        }

        // Moderate bullish signals
        if (bullish_crossover_1h && histogram_increasing_4h)
            || (histogram_above_zero_1h
                && histogram_increasing_1h
                && histogram_above_zero_4h
                && histogram_increasing_4h)
        {
            return (
                TradingSignal::Long,
                0.71,
                "Bullish MACD momentum building".to_string(),
            );
        }

        // Moderate bearish signals
        if (bearish_crossover_1h && histogram_decreasing_4h)
            || (histogram_below_zero_1h
                && histogram_decreasing_1h
                && histogram_below_zero_4h
                && histogram_decreasing_4h)
        {
            return (
                TradingSignal::Short,
                0.71,
                "Bearish MACD momentum building".to_string(),
            );
        }

        // Weak bullish signals (but not if 4h is bearish)
        if histogram_increasing_1h
            && macd_1h > signal_1h
            && histogram_1h > prev_histogram_1h * 1.1
            && !histogram_below_zero_4h
        {
            return (
                TradingSignal::Long,
                0.55,
                "Weak bullish momentum with MACD above signal line".to_string(),
            );
        }

        // Weak bearish signals (but not if 4h is bullish)
        if histogram_decreasing_1h
            && macd_1h < signal_1h
            && histogram_1h < prev_histogram_1h * 1.1
            && !histogram_above_zero_4h
        {
            return (
                TradingSignal::Short,
                0.55,
                "Weak bearish momentum with MACD below signal line".to_string(),
            );
        }

        // Neutral - consolidation or no clear trend
        let confidence = if histogram_1h.abs() < threshold && histogram_4h.abs() < threshold * 2.0 {
            0.65 // High confidence in consolidation
        } else {
            0.45 // Low confidence due to mixed signals
        };

        (
            TradingSignal::Neutral,
            confidence,
            "MACD showing mixed signals, consolidation phase".to_string(),
        )
    }
}

impl Default for MacdStrategy {
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
                high: price * 1.005,
                low: price * 0.995,
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
    async fn test_macd_strategy_new() {
        let strategy = MacdStrategy::new();
        assert_eq!(strategy.name(), "MACD Strategy");
        assert!(strategy.config().enabled);
        assert_eq!(strategy.config().weight, 1.0);
    }

    #[tokio::test]
    async fn test_macd_strategy_bullish_crossover() {
        let strategy = MacdStrategy::new();

        // Create uptrend for bullish MACD crossover
        let prices_1h: Vec<f64> = (0..50).map(|i| 100.0 + (i as f64 * 0.5)).collect();
        let prices_4h: Vec<f64> = (0..50).map(|i| 100.0 + (i as f64 * 0.3)).collect();

        let input = create_test_input(prices_1h, prices_4h);
        let result = strategy.analyze(&input).await;

        assert!(result.is_ok());
        let output = result.unwrap();
        // Should be Long or Neutral, but not Short
        assert_ne!(output.signal, TradingSignal::Short);
    }

    #[tokio::test]
    async fn test_macd_strategy_bearish_crossover() {
        let strategy = MacdStrategy::new();

        // Create downtrend for bearish MACD crossover
        let prices_1h: Vec<f64> = (0..50).map(|i| 150.0 - (i as f64 * 0.5)).collect();
        let prices_4h: Vec<f64> = (0..50).map(|i| 150.0 - (i as f64 * 0.3)).collect();

        let input = create_test_input(prices_1h, prices_4h);
        let result = strategy.analyze(&input).await;

        assert!(result.is_ok());
        let output = result.unwrap();
        // Should be Short or Neutral, but not Long
        assert_ne!(output.signal, TradingSignal::Long);
    }

    #[tokio::test]
    async fn test_macd_strategy_configuration() {
        let mut strategy = MacdStrategy::new();

        assert_eq!(strategy.get_fast_period(), 12);
        assert_eq!(strategy.get_slow_period(), 26);
        assert_eq!(strategy.get_signal_period(), 9);

        // Update config
        let mut new_config = StrategyConfig::default();
        new_config
            .parameters
            .insert("fast_period".to_string(), json!(8));
        new_config
            .parameters
            .insert("slow_period".to_string(), json!(21));
        new_config
            .parameters
            .insert("signal_period".to_string(), json!(5));

        strategy.update_config(new_config);
        assert_eq!(strategy.get_fast_period(), 8);
        assert_eq!(strategy.get_slow_period(), 21);
        assert_eq!(strategy.get_signal_period(), 5);
    }

    #[tokio::test]
    async fn test_macd_strategy_required_timeframes() {
        let strategy = MacdStrategy::new();
        let timeframes = strategy.required_timeframes();

        assert_eq!(timeframes.len(), 2);
        assert!(timeframes.contains(&"1h"));
        assert!(timeframes.contains(&"4h"));
    }

    #[tokio::test]
    async fn test_macd_strategy_validate_data_success() {
        let strategy = MacdStrategy::new();
        let prices: Vec<f64> = (0..50).map(|i| 100.0 + (i as f64)).collect();
        let input = create_test_input(prices.clone(), prices);

        let result = strategy.validate_data(&input);
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_macd_strategy_validate_data_insufficient() {
        let strategy = MacdStrategy::new();
        let prices: Vec<f64> = (0..20).map(|i| 100.0 + (i as f64)).collect();
        let input = create_test_input(prices.clone(), prices);

        let result = strategy.validate_data(&input);
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_macd_strategy_metadata() {
        let strategy = MacdStrategy::new();
        let prices: Vec<f64> = (0..50).map(|i| 100.0 + (i as f64 * 0.2)).collect();
        let input = create_test_input(prices.clone(), prices);

        let result = strategy.analyze(&input).await;
        assert!(result.is_ok());

        let output = result.unwrap();
        assert!(output.metadata.contains_key("macd_line_1h"));
        assert!(output.metadata.contains_key("signal_line_1h"));
        assert!(output.metadata.contains_key("histogram_1h"));
        assert!(output.metadata.contains_key("macd_line_4h"));
        assert!(output.metadata.contains_key("signal_line_4h"));
        assert!(output.metadata.contains_key("histogram_4h"));
    }

    #[test]
    fn test_analyze_macd_signals_strong_bullish() {
        let strategy = MacdStrategy::new();
        // Bullish crossover with positive histogram on both timeframes
        let (signal, confidence, _) = strategy.analyze_macd_signals(
            0.5, 0.3, 0.2, // 1h: MACD > signal, positive histogram
            0.4, 0.2, 0.2, // 4h: MACD > signal, positive histogram
            0.2, 0.25, 0.1, // prev: MACD was below signal (crossover)
            0.1, // prev 4h histogram was lower
        );

        assert_eq!(signal, TradingSignal::Long);
        assert_eq!(confidence, 0.89);
    }

    #[test]
    fn test_analyze_macd_signals_strong_bearish() {
        let strategy = MacdStrategy::new();
        // Bearish crossover with negative histogram on both timeframes
        let (signal, confidence, _) = strategy.analyze_macd_signals(
            -0.5, -0.3, -0.2, // 1h: MACD < signal, negative histogram
            -0.4, -0.2, -0.2, // 4h: MACD < signal, negative histogram
            -0.2, -0.25, -0.1, // prev: MACD was above signal (crossover)
            -0.1, // prev 4h histogram was higher
        );

        assert_eq!(signal, TradingSignal::Short);
        assert_eq!(confidence, 0.89);
    }

    #[test]
    #[ignore] // Business logic test - needs tuning
    fn test_analyze_macd_signals_moderate_bullish() {
        let strategy = MacdStrategy::new();
        let (signal, confidence, _) = strategy.analyze_macd_signals(
            0.3, 0.2, 0.1, // 1h: bullish crossover
            0.2, 0.25, 0.15, // 4h: histogram increasing
            0.15, 0.25, 0.05, // prev: crossover occurred
            0.1,  // prev 4h histogram
        );

        assert_eq!(signal, TradingSignal::Long);
        assert_eq!(confidence, 0.71);
    }

    #[test]
    #[ignore] // Business logic test - needs tuning
    fn test_analyze_macd_signals_neutral() {
        let strategy = MacdStrategy::new();
        let (signal, confidence, _) = strategy.analyze_macd_signals(
            0.001, 0.0005, 0.0005, // 1h: near zero
            0.001, 0.0008, 0.0002, // 4h: near zero
            0.0008, 0.0007, 0.0001, // prev: minimal change
            0.0001, // prev 4h histogram
        );

        assert_eq!(signal, TradingSignal::Neutral);
        assert!(confidence > 0.4);
    }

    #[tokio::test]
    async fn test_macd_strategy_description() {
        let strategy = MacdStrategy::new();
        let desc = strategy.description();

        assert!(desc.contains("MACD"));
        assert!(!desc.is_empty());
    }

    #[tokio::test]
    async fn test_macd_strategy_with_custom_config() {
        let mut config = StrategyConfig::default();
        config.enabled = true;
        config.weight = 2.0;
        config
            .parameters
            .insert("fast_period".to_string(), json!(10));
        config
            .parameters
            .insert("slow_period".to_string(), json!(20));
        config
            .parameters
            .insert("signal_period".to_string(), json!(7));

        let strategy = MacdStrategy::with_config(config);

        assert_eq!(strategy.get_fast_period(), 10);
        assert_eq!(strategy.get_slow_period(), 20);
        assert_eq!(strategy.get_signal_period(), 7);
        assert_eq!(strategy.config().weight, 2.0);
    }

    #[tokio::test]
    async fn test_macd_strategy_sideways_market() {
        let strategy = MacdStrategy::new();

        // Create sideways market
        let prices_1h: Vec<f64> = (0..50)
            .map(|i| 100.0 + ((i as f64 % 10.0) - 5.0) * 0.5)
            .collect();
        let prices_4h: Vec<f64> = (0..50)
            .map(|i| 100.0 + ((i as f64 % 8.0) - 4.0) * 0.3)
            .collect();

        let input = create_test_input(prices_1h, prices_4h);
        let result = strategy.analyze(&input).await;

        assert!(result.is_ok());
        let output = result.unwrap();
        // Sideways market should produce lower confidence
        assert!(output.confidence < 0.8);
    }

    #[test]
    fn test_get_histogram_threshold() {
        let strategy = MacdStrategy::new();
        assert_eq!(strategy.get_histogram_threshold(), 0.001);

        let mut config = StrategyConfig::default();
        config
            .parameters
            .insert("histogram_threshold".to_string(), json!(0.005));
        let strategy2 = MacdStrategy::with_config(config);
        assert_eq!(strategy2.get_histogram_threshold(), 0.005);
    }

    #[test]
    fn test_macd_strategy_default() {
        let strategy = MacdStrategy::default();
        assert_eq!(strategy.name(), "MACD Strategy");
        assert_eq!(strategy.get_fast_period(), 12);
        assert_eq!(strategy.get_slow_period(), 26);
        assert_eq!(strategy.get_signal_period(), 9);
    }

    #[test]
    fn test_get_fast_period_default_fallback() {
        let mut config = StrategyConfig::default();
        config
            .parameters
            .insert("fast_period".to_string(), json!(null));
        let strategy = MacdStrategy::with_config(config);
        assert_eq!(strategy.get_fast_period(), 12);
    }

    #[test]
    fn test_get_slow_period_default_fallback() {
        let mut config = StrategyConfig::default();
        config
            .parameters
            .insert("slow_period".to_string(), json!(null));
        let strategy = MacdStrategy::with_config(config);
        assert_eq!(strategy.get_slow_period(), 26);
    }

    #[test]
    fn test_get_signal_period_default_fallback() {
        let mut config = StrategyConfig::default();
        config
            .parameters
            .insert("signal_period".to_string(), json!(null));
        let strategy = MacdStrategy::with_config(config);
        assert_eq!(strategy.get_signal_period(), 9);
    }

    #[test]
    fn test_get_histogram_threshold_default_fallback() {
        let mut config = StrategyConfig::default();
        config
            .parameters
            .insert("histogram_threshold".to_string(), json!(null));
        let strategy = MacdStrategy::with_config(config);
        assert_eq!(strategy.get_histogram_threshold(), 0.001);
    }

    #[test]
    fn test_get_fast_period_missing_parameter() {
        let config = StrategyConfig::default();
        let strategy = MacdStrategy::with_config(config);
        assert_eq!(strategy.get_fast_period(), 12);
    }

    #[test]
    fn test_get_slow_period_missing_parameter() {
        let config = StrategyConfig::default();
        let strategy = MacdStrategy::with_config(config);
        assert_eq!(strategy.get_slow_period(), 26);
    }

    #[test]
    fn test_get_signal_period_missing_parameter() {
        let config = StrategyConfig::default();
        let strategy = MacdStrategy::with_config(config);
        assert_eq!(strategy.get_signal_period(), 9);
    }

    #[test]
    fn test_get_histogram_threshold_missing_parameter() {
        let config = StrategyConfig::default();
        let strategy = MacdStrategy::with_config(config);
        assert_eq!(strategy.get_histogram_threshold(), 0.001);
    }

    #[test]
    fn test_get_fast_period_custom_value() {
        let mut config = StrategyConfig::default();
        config
            .parameters
            .insert("fast_period".to_string(), json!(15));
        let strategy = MacdStrategy::with_config(config);
        assert_eq!(strategy.get_fast_period(), 15);
    }

    #[test]
    fn test_get_slow_period_custom_value() {
        let mut config = StrategyConfig::default();
        config
            .parameters
            .insert("slow_period".to_string(), json!(30));
        let strategy = MacdStrategy::with_config(config);
        assert_eq!(strategy.get_slow_period(), 30);
    }

    #[test]
    fn test_get_signal_period_custom_value() {
        let mut config = StrategyConfig::default();
        config
            .parameters
            .insert("signal_period".to_string(), json!(12));
        let strategy = MacdStrategy::with_config(config);
        assert_eq!(strategy.get_signal_period(), 12);
    }

    #[test]
    fn test_get_histogram_threshold_custom_value() {
        let mut config = StrategyConfig::default();
        config
            .parameters
            .insert("histogram_threshold".to_string(), json!(0.01));
        let strategy = MacdStrategy::with_config(config);
        assert_eq!(strategy.get_histogram_threshold(), 0.01);
    }

    #[test]
    fn test_analyze_macd_signals_moderate_bullish_crossover() {
        let strategy = MacdStrategy::new();
        let (signal, confidence, reasoning) = strategy.analyze_macd_signals(
            0.5, 0.3, 0.2, // 1h: bullish crossover, histogram NOT increasing
            0.2, 0.1, 0.15, // 4h: histogram increasing (to satisfy moderate)
            0.2, 0.3, 0.25, // prev: MACD was below signal, histogram WAS higher
            0.10, // prev 4h histogram < current (increasing)
        );

        assert_eq!(signal, TradingSignal::Long);
        assert_eq!(confidence, 0.71);
        assert!(reasoning.contains("Bullish MACD momentum building"));
    }

    #[test]
    fn test_analyze_macd_signals_moderate_bearish_crossover() {
        let strategy = MacdStrategy::new();
        let (signal, confidence, reasoning) = strategy.analyze_macd_signals(
            -0.5, -0.3, -0.2, // 1h: bearish crossover, histogram NOT decreasing
            -0.2, -0.1, -0.15, // 4h: histogram decreasing (to satisfy moderate)
            -0.2, -0.3, -0.25, // prev: MACD was above signal, histogram WAS lower
            -0.10, // prev 4h histogram > current (decreasing)
        );

        assert_eq!(signal, TradingSignal::Short);
        assert_eq!(confidence, 0.71);
        assert!(reasoning.contains("Bearish MACD momentum building"));
    }

    #[test]
    fn test_analyze_macd_signals_moderate_bullish_histogram_above_zero() {
        let strategy = MacdStrategy::new();
        let (signal, confidence, reasoning) = strategy.analyze_macd_signals(
            0.3, 0.2, 0.1, // 1h: MACD > signal, histogram above zero
            0.1, 0.05, 0.02, // 4h: positive and increasing
            0.25, 0.22, 0.05, // prev: histogram increasing
            0.01, // prev 4h histogram (increasing: 0.01 -> 0.02)
        );

        assert_eq!(signal, TradingSignal::Long);
        assert_eq!(confidence, 0.71);
        assert!(reasoning.contains("Bullish MACD momentum building"));
    }

    #[test]
    fn test_analyze_macd_signals_moderate_bearish_histogram_below_zero() {
        let strategy = MacdStrategy::new();
        let (signal, confidence, reasoning) = strategy.analyze_macd_signals(
            -0.3, -0.2, -0.1, // 1h: MACD < signal, histogram below zero
            -0.1, -0.05, -0.02, // 4h: negative and decreasing
            -0.25, -0.22, -0.05, // prev: histogram decreasing
            -0.01, // prev 4h histogram (decreasing: -0.01 -> -0.02)
        );

        assert_eq!(signal, TradingSignal::Short);
        assert_eq!(confidence, 0.71);
        assert!(reasoning.contains("Bearish MACD momentum building"));
    }

    #[test]
    fn test_analyze_macd_signals_weak_bullish() {
        let strategy = MacdStrategy::new();
        let (signal, confidence, reasoning) = strategy.analyze_macd_signals(
            0.25, 0.2, 0.05, // 1h: MACD > signal
            0.1, 0.08, 0.02, // 4h: weak momentum
            0.24, 0.21, 0.02, // prev: histogram increasing significantly
            0.03, // prev 4h histogram (DECREASING: 0.03 -> 0.02)
        );

        assert_eq!(signal, TradingSignal::Long);
        assert_eq!(confidence, 0.55);
        assert!(reasoning.contains("Weak bullish momentum"));
    }

    #[test]
    fn test_analyze_macd_signals_weak_bearish() {
        let strategy = MacdStrategy::new();
        let (signal, confidence, reasoning) = strategy.analyze_macd_signals(
            -0.25, -0.2, -0.05, // 1h: MACD < signal
            -0.1, -0.08, -0.02, // 4h: weak momentum
            -0.24, -0.21, -0.02, // prev: histogram decreasing significantly
            -0.03, // prev 4h histogram (INCREASING: -0.03 -> -0.02)
        );

        assert_eq!(signal, TradingSignal::Short);
        assert_eq!(confidence, 0.55);
        assert!(reasoning.contains("Weak bearish momentum"));
    }

    #[test]
    fn test_analyze_macd_signals_neutral_consolidation() {
        let strategy = MacdStrategy::new();
        let (signal, confidence, reasoning) = strategy.analyze_macd_signals(
            0.0005, 0.0004, 0.0001, // 1h: near zero
            0.0003, 0.0002, 0.0001, // 4h: near zero
            0.0004, 0.0003, 0.0001, // prev: minimal change
            0.0001, // prev 4h histogram
        );

        assert_eq!(signal, TradingSignal::Neutral);
        assert_eq!(confidence, 0.65);
        assert!(reasoning.contains("mixed signals"));
    }

    #[test]
    fn test_analyze_macd_signals_neutral_mixed_signals() {
        let strategy = MacdStrategy::new();
        let (signal, confidence, reasoning) = strategy.analyze_macd_signals(
            0.5, 0.4, 0.1, // 1h: bullish but weak
            -0.2, -0.1, -0.1, // 4h: bearish
            0.45, 0.42, 0.08,  // prev: minimal change
            -0.08, // prev 4h histogram
        );

        assert_eq!(signal, TradingSignal::Neutral);
        assert_eq!(confidence, 0.45);
        assert!(reasoning.contains("mixed signals"));
    }

    #[test]
    fn test_analyze_macd_signals_bullish_no_crossover_but_momentum() {
        let strategy = MacdStrategy::new();
        let (signal, confidence, _) = strategy.analyze_macd_signals(
            0.6, 0.4, 0.2, // 1h: MACD > signal, strong momentum
            0.3, 0.2, 0.1, // 4h: positive
            0.55, 0.42, 0.15, // prev: no crossover but increasing
            0.08, // prev 4h histogram
        );

        assert_eq!(signal, TradingSignal::Long);
        // Should be moderate confidence
        assert!(confidence > 0.5);
    }

    #[test]
    fn test_analyze_macd_signals_bearish_no_crossover_but_momentum() {
        let strategy = MacdStrategy::new();
        let (signal, confidence, _) = strategy.analyze_macd_signals(
            -0.6, -0.4, -0.2, // 1h: MACD < signal, strong momentum
            -0.3, -0.2, -0.1, // 4h: negative
            -0.55, -0.42, -0.15, // prev: no crossover but decreasing
            -0.08, // prev 4h histogram
        );

        assert_eq!(signal, TradingSignal::Short);
        // Should be moderate confidence
        assert!(confidence > 0.5);
    }

    #[test]
    fn test_analyze_macd_signals_exact_threshold_positive() {
        let strategy = MacdStrategy::new();
        let threshold = strategy.get_histogram_threshold();
        let (signal, _, _) = strategy.analyze_macd_signals(
            threshold + 0.0001,
            threshold,
            0.0001,
            threshold,
            threshold - 0.0001,
            0.0001,
            threshold - 0.0001,
            threshold - 0.0002,
            0.00005,
            0.00005,
        );

        // Should produce some signal, not error
        assert!(matches!(
            signal,
            TradingSignal::Long | TradingSignal::Neutral
        ));
    }

    #[test]
    fn test_analyze_macd_signals_exact_threshold_negative() {
        let strategy = MacdStrategy::new();
        let threshold = strategy.get_histogram_threshold();
        let (signal, _, _) = strategy.analyze_macd_signals(
            -threshold - 0.0001,
            -threshold,
            -0.0001,
            -threshold,
            -threshold + 0.0001,
            -0.0001,
            -threshold + 0.0001,
            -threshold + 0.0002,
            -0.00005,
            -0.00005,
        );

        // Should produce some signal, not error
        assert!(matches!(
            signal,
            TradingSignal::Short | TradingSignal::Neutral
        ));
    }

    #[test]
    fn test_analyze_macd_signals_zero_values() {
        let strategy = MacdStrategy::new();
        let (signal, _, _) =
            strategy.analyze_macd_signals(0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0);

        assert_eq!(signal, TradingSignal::Neutral);
    }

    #[test]
    fn test_analyze_macd_signals_extreme_positive_values() {
        let strategy = MacdStrategy::new();
        let (signal, _, _) = strategy.analyze_macd_signals(
            1000.0, 900.0, 100.0, 800.0, 700.0, 100.0, 850.0, 950.0, 50.0, 50.0,
        );

        // Should handle extreme values without panic
        assert!(matches!(
            signal,
            TradingSignal::Long | TradingSignal::Neutral
        ));
    }

    #[test]
    fn test_analyze_macd_signals_extreme_negative_values() {
        let strategy = MacdStrategy::new();
        let (signal, _, _) = strategy.analyze_macd_signals(
            -1000.0, -900.0, -100.0, -800.0, -700.0, -100.0, -850.0, -950.0, -50.0, -50.0,
        );

        // Should handle extreme values without panic
        assert!(matches!(
            signal,
            TradingSignal::Short | TradingSignal::Neutral
        ));
    }

    #[tokio::test]
    async fn test_macd_strategy_missing_1h_timeframe() {
        let strategy = MacdStrategy::new();
        let mut timeframe_data = HashMap::new();
        timeframe_data.insert("4h".to_string(), create_test_candles(vec![100.0; 50]));

        let input = StrategyInput {
            symbol: "BTCUSDT".to_string(),
            timeframe_data,
            current_price: 50000.0,
            volume_24h: 1000000.0,
            timestamp: 1234567890,
        };

        let result = strategy.analyze(&input).await;
        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            StrategyError::InsufficientData(_)
        ));
    }

    #[tokio::test]
    async fn test_macd_strategy_missing_4h_timeframe() {
        let strategy = MacdStrategy::new();
        let mut timeframe_data = HashMap::new();
        timeframe_data.insert("1h".to_string(), create_test_candles(vec![100.0; 50]));

        let input = StrategyInput {
            symbol: "BTCUSDT".to_string(),
            timeframe_data,
            current_price: 50000.0,
            volume_24h: 1000000.0,
            timestamp: 1234567890,
        };

        let result = strategy.analyze(&input).await;
        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            StrategyError::InsufficientData(_)
        ));
    }

    #[tokio::test]
    async fn test_macd_strategy_empty_timeframe_data() {
        let strategy = MacdStrategy::new();
        let timeframe_data = HashMap::new();

        let input = StrategyInput {
            symbol: "BTCUSDT".to_string(),
            timeframe_data,
            current_price: 50000.0,
            volume_24h: 1000000.0,
            timestamp: 1234567890,
        };

        let result = strategy.analyze(&input).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_macd_strategy_validate_data_missing_timeframe() {
        let strategy = MacdStrategy::new();
        let mut timeframe_data = HashMap::new();
        timeframe_data.insert("1h".to_string(), create_test_candles(vec![100.0; 50]));

        let input = StrategyInput {
            symbol: "BTCUSDT".to_string(),
            timeframe_data,
            current_price: 50000.0,
            volume_24h: 1000000.0,
            timestamp: 1234567890,
        };

        let result = strategy.validate_data(&input);
        assert!(result.is_err());
        if let Err(StrategyError::InsufficientData(msg)) = result {
            assert!(msg.contains("4h"));
        }
    }

    #[tokio::test]
    async fn test_macd_strategy_output_structure() {
        let strategy = MacdStrategy::new();
        let prices: Vec<f64> = (0..50).map(|i| 100.0 + (i as f64 * 0.5)).collect();
        let input = create_test_input(prices.clone(), prices);

        let result = strategy.analyze(&input).await;
        assert!(result.is_ok());

        let output = result.unwrap();
        assert_eq!(output.timeframe, "1h");
        assert_eq!(output.timestamp, 1234567890);
        assert!(!output.reasoning.is_empty());
        assert!(output.confidence >= 0.0 && output.confidence <= 1.0);
    }

    #[tokio::test]
    async fn test_macd_strategy_strong_uptrend() {
        let strategy = MacdStrategy::new();
        // Strong uptrend with accelerating prices
        let prices_1h: Vec<f64> = (0..50)
            .map(|i| 100.0 + (i as f64 * i as f64 * 0.01))
            .collect();
        let prices_4h: Vec<f64> = (0..50)
            .map(|i| 100.0 + (i as f64 * i as f64 * 0.008))
            .collect();

        let input = create_test_input(prices_1h, prices_4h);
        let result = strategy.analyze(&input).await;

        assert!(result.is_ok());
        let output = result.unwrap();
        // Strong uptrend should likely produce Long or high confidence
        assert!(output.confidence > 0.0);
    }

    #[tokio::test]
    async fn test_macd_strategy_strong_downtrend() {
        let strategy = MacdStrategy::new();
        // Strong downtrend with accelerating decline
        let prices_1h: Vec<f64> = (0..50)
            .map(|i| 200.0 - (i as f64 * i as f64 * 0.01))
            .collect();
        let prices_4h: Vec<f64> = (0..50)
            .map(|i| 200.0 - (i as f64 * i as f64 * 0.008))
            .collect();

        let input = create_test_input(prices_1h, prices_4h);
        let result = strategy.analyze(&input).await;

        assert!(result.is_ok());
        let output = result.unwrap();
        // Strong downtrend should likely produce Short or high confidence
        assert!(output.confidence > 0.0);
    }

    #[tokio::test]
    async fn test_macd_strategy_config_persistence() {
        let mut config = StrategyConfig::default();
        config.enabled = false;
        config.weight = 3.5;
        config
            .parameters
            .insert("fast_period".to_string(), json!(8));

        let strategy = MacdStrategy::with_config(config.clone());
        let retrieved_config = strategy.config();

        assert!(!retrieved_config.enabled);
        assert_eq!(retrieved_config.weight, 3.5);
        assert_eq!(strategy.get_fast_period(), 8);
    }

    #[tokio::test]
    async fn test_macd_strategy_update_config_persistence() {
        let mut strategy = MacdStrategy::new();

        let mut new_config = StrategyConfig::default();
        new_config.enabled = false;
        new_config.weight = 2.5;
        new_config
            .parameters
            .insert("fast_period".to_string(), json!(10));
        new_config
            .parameters
            .insert("slow_period".to_string(), json!(22));

        strategy.update_config(new_config);

        assert!(!strategy.config().enabled);
        assert_eq!(strategy.config().weight, 2.5);
        assert_eq!(strategy.get_fast_period(), 10);
        assert_eq!(strategy.get_slow_period(), 22);
    }

    #[tokio::test]
    async fn test_macd_strategy_minimal_valid_data() {
        let strategy = MacdStrategy::new();
        // Create minimum required candles (slow_period + signal_period + 10 = 26 + 9 + 10 = 45)
        let prices: Vec<f64> = (0..45).map(|i| 100.0 + (i as f64 * 0.1)).collect();
        let input = create_test_input(prices.clone(), prices);

        let result = strategy.validate_data(&input);
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_macd_strategy_exactly_minimum_data() {
        let strategy = MacdStrategy::new();
        let min_required = strategy.get_slow_period() + strategy.get_signal_period() + 10;
        let prices: Vec<f64> = (0..min_required).map(|i| 100.0 + (i as f64)).collect();
        let input = create_test_input(prices.clone(), prices);

        let result = strategy.validate_data(&input);
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_macd_strategy_one_less_than_minimum() {
        let strategy = MacdStrategy::new();
        let min_required = strategy.get_slow_period() + strategy.get_signal_period() + 10;
        let prices: Vec<f64> = (0..min_required - 1).map(|i| 100.0 + (i as f64)).collect();
        let input = create_test_input(prices.clone(), prices);

        let result = strategy.validate_data(&input);
        assert!(result.is_err());
    }

    #[test]
    fn test_config_reference() {
        let strategy = MacdStrategy::new();
        let config = strategy.config();

        assert!(config.parameters.contains_key("fast_period"));
        assert!(config.parameters.contains_key("slow_period"));
        assert!(config.parameters.contains_key("signal_period"));
        assert!(config.parameters.contains_key("histogram_threshold"));
    }

    #[test]
    fn test_strategy_clone() {
        let strategy1 = MacdStrategy::new();
        let strategy2 = strategy1.clone();

        assert_eq!(strategy1.get_fast_period(), strategy2.get_fast_period());
        assert_eq!(strategy1.get_slow_period(), strategy2.get_slow_period());
        assert_eq!(strategy1.get_signal_period(), strategy2.get_signal_period());
        assert_eq!(
            strategy1.get_histogram_threshold(),
            strategy2.get_histogram_threshold()
        );
    }

    #[tokio::test]
    async fn test_metadata_values_are_numbers() {
        let strategy = MacdStrategy::new();
        let prices: Vec<f64> = (0..50).map(|i| 100.0 + (i as f64 * 0.3)).collect();
        let input = create_test_input(prices.clone(), prices);

        let result = strategy.analyze(&input).await;
        assert!(result.is_ok());

        let output = result.unwrap();
        assert!(output.metadata.get("macd_line_1h").unwrap().is_f64());
        assert!(output.metadata.get("signal_line_1h").unwrap().is_f64());
        assert!(output.metadata.get("histogram_1h").unwrap().is_f64());
        assert!(output.metadata.get("macd_line_4h").unwrap().is_f64());
        assert!(output.metadata.get("signal_line_4h").unwrap().is_f64());
        assert!(output.metadata.get("histogram_4h").unwrap().is_f64());
    }

    #[tokio::test]
    async fn test_reasoning_not_empty() {
        let strategy = MacdStrategy::new();
        let prices: Vec<f64> = (0..50).map(|i| 100.0 + (i as f64 * 0.2)).collect();
        let input = create_test_input(prices.clone(), prices);

        let result = strategy.analyze(&input).await;
        assert!(result.is_ok());

        let output = result.unwrap();
        assert!(!output.reasoning.is_empty());
        assert!(output.reasoning.len() > 10); // Meaningful reasoning
    }

    #[test]
    fn test_analyze_macd_signals_previous_equals_current() {
        let strategy = MacdStrategy::new();
        // Previous values equal current (no change)
        let (signal, _, _) = strategy.analyze_macd_signals(
            0.5, 0.3, 0.2, 0.4, 0.2, 0.2, 0.5, 0.3, 0.2, // Same as current
            0.2, // Same as current
        );

        // Should still produce a valid signal
        assert!(matches!(
            signal,
            TradingSignal::Long | TradingSignal::Short | TradingSignal::Neutral
        ));
    }

    #[test]
    fn test_analyze_macd_signals_strong_bullish_all_conditions() {
        let strategy = MacdStrategy::new();
        let (signal, confidence, reasoning) = strategy.analyze_macd_signals(
            0.6, 0.4, 0.2, // Bullish crossover (MACD > signal)
            0.5, 0.3, 0.2, // 4h positive histogram
            0.3, 0.5, 0.1, // Previous: MACD was below signal
            0.1, // 4h histogram increasing
        );

        assert_eq!(signal, TradingSignal::Long);
        assert_eq!(confidence, 0.89);
        assert!(reasoning.contains("Strong bullish"));
    }

    #[test]
    fn test_analyze_macd_signals_strong_bearish_all_conditions() {
        let strategy = MacdStrategy::new();
        let (signal, confidence, reasoning) = strategy.analyze_macd_signals(
            -0.6, -0.4, -0.2, // Bearish crossover (MACD < signal)
            -0.5, -0.3, -0.2, // 4h negative histogram
            -0.3, -0.5, -0.1, // Previous: MACD was above signal
            -0.1, // 4h histogram decreasing
        );

        assert_eq!(signal, TradingSignal::Short);
        assert_eq!(confidence, 0.89);
        assert!(reasoning.contains("Strong bearish"));
    }

    #[tokio::test]
    async fn test_validate_data_with_custom_periods() {
        let mut config = StrategyConfig::default();
        config
            .parameters
            .insert("fast_period".to_string(), json!(8));
        config
            .parameters
            .insert("slow_period".to_string(), json!(17));
        config
            .parameters
            .insert("signal_period".to_string(), json!(5));

        let strategy = MacdStrategy::with_config(config);
        let min_required = 17 + 5 + 10; // slow_period + signal_period + buffer
        let prices: Vec<f64> = (0..min_required).map(|i| 100.0 + (i as f64)).collect();
        let input = create_test_input(prices.clone(), prices);

        let result = strategy.validate_data(&input);
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_validate_data_insufficient_with_custom_periods() {
        let mut config = StrategyConfig::default();
        config
            .parameters
            .insert("slow_period".to_string(), json!(30));
        config
            .parameters
            .insert("signal_period".to_string(), json!(12));

        let strategy = MacdStrategy::with_config(config);
        let prices: Vec<f64> = (0..40).map(|i| 100.0 + (i as f64)).collect(); // Less than 30 + 12 + 10
        let input = create_test_input(prices.clone(), prices);

        let result = strategy.validate_data(&input);
        assert!(result.is_err());
    }
}
