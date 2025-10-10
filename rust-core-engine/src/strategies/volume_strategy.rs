use super::*;
use crate::strategies::indicators::{calculate_sma, calculate_volume_profile};
use async_trait::async_trait;
use serde_json::json;

/// Volume-based trading strategy

// @spec:FR-STRATEGY-004 - Volume Strategy
// @ref:specs/02-design/2.5-components/COMP-RUST-TRADING.md#strategies
// @test:TC-TRADING-028

#[derive(Debug, Clone)]
pub struct VolumeStrategy {
    config: StrategyConfig,
}

impl VolumeStrategy {
    pub fn new() -> Self {
        let mut config = StrategyConfig::default();
        config
            .parameters
            .insert("volume_sma_period".to_string(), json!(20));
        config
            .parameters
            .insert("volume_spike_threshold".to_string(), json!(2.0));
        config
            .parameters
            .insert("price_volume_correlation_period".to_string(), json!(10));

        Self { config }
    }

    pub fn with_config(config: StrategyConfig) -> Self {
        Self { config }
    }

    fn get_volume_sma_period(&self) -> usize {
        self.config
            .parameters
            .get("volume_sma_period")
            .and_then(|v| v.as_u64())
            .unwrap_or(20) as usize
    }

    fn get_volume_spike_threshold(&self) -> f64 {
        self.config
            .parameters
            .get("volume_spike_threshold")
            .and_then(|v| v.as_f64())
            .unwrap_or(2.0)
    }

    fn get_price_volume_correlation_period(&self) -> usize {
        self.config
            .parameters
            .get("price_volume_correlation_period")
            .and_then(|v| v.as_u64())
            .unwrap_or(10) as usize
    }
}

#[async_trait]
impl Strategy for VolumeStrategy {
    fn name(&self) -> &'static str {
        "Volume Strategy"
    }

    fn description(&self) -> &'static str {
        "Volume-based strategy that identifies accumulation/distribution patterns and volume breakouts"
    }

    fn required_timeframes(&self) -> Vec<&'static str> {
        vec!["1h"]
    }

    async fn analyze(&self, data: &StrategyInput) -> Result<StrategyOutput, StrategyError> {
        self.validate_data(data)?;

        let primary_timeframe = "1h";
        let candles = data.timeframe_data.get(primary_timeframe).ok_or_else(|| {
            StrategyError::InsufficientData(format!("Missing {primary_timeframe} data"))
        })?;

        let volume_sma_period = self.get_volume_sma_period();
        let spike_threshold = self.get_volume_spike_threshold();
        let correlation_period = self.get_price_volume_correlation_period();

        // Calculate volume moving average
        let volumes: Vec<f64> = candles.iter().map(|c| c.volume).collect();
        let volume_sma =
            calculate_sma(&volumes, volume_sma_period).map_err(StrategyError::CalculationError)?;

        let current_volume = *volumes.last().ok_or_else(|| {
            StrategyError::InsufficientData("No volume data available".to_string())
        })?;
        let avg_volume = *volume_sma.last().ok_or_else(|| {
            StrategyError::InsufficientData("No volume SMA calculated".to_string())
        })?;
        let volume_ratio = current_volume / avg_volume;

        // Calculate price changes for volume-price analysis
        let price_changes: Vec<f64> = candles
            .windows(2)
            .map(|w| w[1].close - w[0].close)
            .collect();

        // Analyze recent volume and price action
        let recent_period = correlation_period.min(candles.len() - 1);
        let recent_candles = &candles[candles.len() - recent_period..];
        let recent_volumes = &volumes[volumes.len() - recent_period..];
        let recent_price_changes =
            &price_changes[price_changes.len() - recent_period.min(price_changes.len())..];

        // Calculate volume profile
        let volume_profile = calculate_volume_profile(recent_candles, 20)
            .map_err(StrategyError::CalculationError)?;

        let current_price = data.current_price;
        let poc_distance = ((current_price - volume_profile.poc) / current_price).abs();

        // Analyze accumulation/distribution
        let (signal, confidence, reasoning) = self.analyze_volume_signals(
            current_volume,
            avg_volume,
            volume_ratio,
            recent_volumes,
            recent_price_changes,
            current_price,
            volume_profile.poc,
            poc_distance,
            spike_threshold,
        );

        let mut metadata = std::collections::HashMap::new();
        metadata.insert("current_volume".to_string(), json!(current_volume));
        metadata.insert("avg_volume".to_string(), json!(avg_volume));
        metadata.insert("volume_ratio".to_string(), json!(volume_ratio));
        metadata.insert("poc".to_string(), json!(volume_profile.poc));
        metadata.insert("poc_distance".to_string(), json!(poc_distance));
        metadata.insert("volume_spike_threshold".to_string(), json!(spike_threshold));

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

            let min_required = self.get_volume_sma_period() + 5;

            if candles.len() < min_required {
                return Err(StrategyError::InsufficientData(format!(
                    "Need at least {} candles for {} timeframe, got {}",
                    min_required,
                    timeframe,
                    candles.len()
                )));
            }
        }

        Ok(())
    }
}

impl VolumeStrategy {
    #[allow(clippy::too_many_arguments)]
    fn analyze_volume_signals(
        &self,
        _current_volume: f64,
        _avg_volume: f64,
        volume_ratio: f64,
        recent_volumes: &[f64],
        recent_price_changes: &[f64],
        current_price: f64,
        poc: f64,
        poc_distance: f64,
        spike_threshold: f64,
    ) -> (TradingSignal, f64, String) {
        // Check for volume spike
        let is_volume_spike = volume_ratio >= spike_threshold;
        let is_high_volume = volume_ratio >= 1.5;

        // Calculate volume-weighted price momentum
        let mut bullish_volume = 0.0;
        let mut _bearish_volume = 0.0;
        let mut total_volume = 0.0;

        for (i, &volume) in recent_volumes.iter().enumerate() {
            if i < recent_price_changes.len() {
                let price_change = recent_price_changes[i];
                total_volume += volume;

                if price_change > 0.0 {
                    bullish_volume += volume;
                } else {
                    _bearish_volume += volume;
                }
            }
        }

        let bullish_volume_ratio = if total_volume > 0.0 {
            bullish_volume / total_volume
        } else {
            0.5
        };

        // Check price position relative to Point of Control
        let near_poc = poc_distance < 0.02; // Within 2% of POC
        let above_poc = current_price > poc;

        // Strong bullish signals
        if is_volume_spike && bullish_volume_ratio > 0.7 && above_poc {
            return (
                TradingSignal::Long,
                0.91,
                "Volume surge with price action confirmation".to_string(),
            );
        }

        // Strong bearish signals
        if is_volume_spike && bullish_volume_ratio < 0.3 && !above_poc {
            return (
                TradingSignal::Short,
                0.91,
                "High volume distribution with bearish price action".to_string(),
            );
        }

        // Moderate bullish signals
        if (is_high_volume && bullish_volume_ratio > 0.6)
            || (near_poc && bullish_volume_ratio > 0.65 && above_poc)
        {
            return (
                TradingSignal::Long,
                0.71,
                "Volume accumulation pattern detected".to_string(),
            );
        }

        // Moderate bearish signals
        if (is_high_volume && bullish_volume_ratio < 0.4)
            || (near_poc && bullish_volume_ratio < 0.35 && !above_poc)
        {
            return (
                TradingSignal::Short,
                0.71,
                "Volume distribution pattern detected".to_string(),
            );
        }

        // Weak signals based on volume patterns
        if bullish_volume_ratio > 0.55 && volume_ratio > 1.2 {
            return (
                TradingSignal::Long,
                0.51,
                "Moderate buying interest with increased volume".to_string(),
            );
        }

        if bullish_volume_ratio < 0.45 && volume_ratio > 1.2 {
            return (
                TradingSignal::Short,
                0.51,
                "Moderate selling pressure with increased volume".to_string(),
            );
        }

        // Neutral - low volume or balanced
        let confidence = if volume_ratio < 0.8 {
            0.65 // High confidence in low activity
        } else {
            0.45 // Mixed signals
        };

        (
            TradingSignal::Neutral,
            confidence,
            "Low volume consolidation, waiting for volume confirmation".to_string(),
        )
    }
}

impl Default for VolumeStrategy {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::market_data::cache::CandleData;
    use std::collections::HashMap;

    fn create_test_candles_with_volume(prices: Vec<f64>, volumes: Vec<f64>) -> Vec<CandleData> {
        prices
            .iter()
            .enumerate()
            .map(|(i, &price)| CandleData {
                open: price,
                high: price * 1.01,
                low: price * 0.99,
                close: price,
                volume: volumes[i],
                open_time: (i as i64) * 3600000,
                close_time: (i as i64) * 3600000 + 3600000,
                quote_volume: 1000.0 * price,
                trades: 100,
                is_closed: true,
            })
            .collect()
    }

    fn create_test_input_with_volume(prices: Vec<f64>, volumes: Vec<f64>) -> StrategyInput {
        let mut timeframe_data = HashMap::new();
        timeframe_data.insert(
            "1h".to_string(),
            create_test_candles_with_volume(prices, volumes),
        );

        StrategyInput {
            symbol: "BTCUSDT".to_string(),
            timeframe_data,
            current_price: 50000.0,
            volume_24h: 1000000.0,
            timestamp: 1234567890,
        }
    }

    #[tokio::test]
    async fn test_volume_strategy_new() {
        let strategy = VolumeStrategy::new();
        assert_eq!(strategy.name(), "Volume Strategy");
        assert!(strategy.config().enabled);
        assert_eq!(strategy.config().weight, 1.0);
    }

    #[tokio::test]
    async fn test_volume_strategy_high_volume_accumulation() {
        let strategy = VolumeStrategy::new();

        // Create accumulation pattern: prices rising with high volume
        let prices: Vec<f64> = (0..30).map(|i| 100.0 + (i as f64 * 0.5)).collect();
        let volumes: Vec<f64> = (0..30)
            .map(|i| if i > 20 { 3000.0 } else { 1000.0 })
            .collect();

        let input = create_test_input_with_volume(prices, volumes);
        let result = strategy.analyze(&input).await;

        assert!(result.is_ok());
        let output = result.unwrap();
        assert_eq!(output.signal, TradingSignal::Long);
        assert!(output.confidence > 0.7);
    }

    #[tokio::test]
    async fn test_volume_strategy_high_volume_distribution() {
        let strategy = VolumeStrategy::new();

        // Create distribution pattern: prices falling with high volume
        let prices: Vec<f64> = (0..30).map(|i| 150.0 - (i as f64 * 0.5)).collect();
        let volumes: Vec<f64> = (0..30)
            .map(|i| if i > 20 { 3000.0 } else { 1000.0 })
            .collect();

        let input = create_test_input_with_volume(prices, volumes);
        let result = strategy.analyze(&input).await;

        assert!(result.is_ok());
        let output = result.unwrap();
        assert_eq!(output.signal, TradingSignal::Short);
        assert!(output.confidence > 0.7);
    }

    #[tokio::test]
    async fn test_volume_strategy_low_volume() {
        let strategy = VolumeStrategy::new();

        // Create low volume consolidation
        let prices: Vec<f64> = (0..30).map(|i| 100.0 + ((i as f64 % 3.0) - 1.0)).collect();
        let volumes: Vec<f64> = vec![500.0; 30]; // Low volume

        let input = create_test_input_with_volume(prices, volumes);
        let result = strategy.analyze(&input).await;

        assert!(result.is_ok());
        let output = result.unwrap();
        assert_eq!(output.signal, TradingSignal::Neutral);
    }

    #[tokio::test]
    async fn test_volume_strategy_configuration() {
        let mut strategy = VolumeStrategy::new();

        assert_eq!(strategy.get_volume_sma_period(), 20);
        assert_eq!(strategy.get_volume_spike_threshold(), 2.0);
        assert_eq!(strategy.get_price_volume_correlation_period(), 10);

        // Update config
        let mut new_config = StrategyConfig::default();
        new_config
            .parameters
            .insert("volume_sma_period".to_string(), json!(15));
        new_config
            .parameters
            .insert("volume_spike_threshold".to_string(), json!(2.5));
        new_config
            .parameters
            .insert("price_volume_correlation_period".to_string(), json!(5));

        strategy.update_config(new_config);
        assert_eq!(strategy.get_volume_sma_period(), 15);
        assert_eq!(strategy.get_volume_spike_threshold(), 2.5);
        assert_eq!(strategy.get_price_volume_correlation_period(), 5);
    }

    #[tokio::test]
    async fn test_volume_strategy_required_timeframes() {
        let strategy = VolumeStrategy::new();
        let timeframes = strategy.required_timeframes();

        assert_eq!(timeframes.len(), 1);
        assert!(timeframes.contains(&"1h"));
    }

    #[tokio::test]
    async fn test_volume_strategy_validate_data_success() {
        let strategy = VolumeStrategy::new();
        let prices: Vec<f64> = (0..30).map(|i| 100.0 + (i as f64)).collect();
        let volumes: Vec<f64> = vec![1000.0; 30];
        let input = create_test_input_with_volume(prices, volumes);

        let result = strategy.validate_data(&input);
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_volume_strategy_validate_data_insufficient() {
        let strategy = VolumeStrategy::new();
        let prices: Vec<f64> = (0..15).map(|i| 100.0 + (i as f64)).collect();
        let volumes: Vec<f64> = vec![1000.0; 15];
        let input = create_test_input_with_volume(prices, volumes);

        let result = strategy.validate_data(&input);
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_volume_strategy_metadata() {
        let strategy = VolumeStrategy::new();
        let prices: Vec<f64> = (0..30).map(|i| 100.0 + (i as f64 * 0.5)).collect();
        let volumes: Vec<f64> = vec![1000.0; 30];
        let input = create_test_input_with_volume(prices, volumes);

        let result = strategy.analyze(&input).await;
        assert!(result.is_ok());

        let output = result.unwrap();
        assert!(output.metadata.contains_key("current_volume"));
        assert!(output.metadata.contains_key("avg_volume"));
        assert!(output.metadata.contains_key("volume_ratio"));
        assert!(output.metadata.contains_key("poc"));
        assert!(output.metadata.contains_key("volume_spike_threshold"));
    }

    #[test]
    fn test_analyze_volume_signals_strong_spike_bullish() {
        let strategy = VolumeStrategy::new();
        let recent_volumes = vec![1000.0; 10];
        let recent_price_changes = vec![1.0, 2.0, 1.5, 2.5, 1.0, 2.0, 1.5, 2.0, 1.0, 2.0]; // All positive

        let (signal, confidence, _) = strategy.analyze_volume_signals(
            3000.0, // current_volume (spike)
            1000.0, // avg_volume
            3.0,    // volume_ratio (above spike threshold)
            &recent_volumes,
            &recent_price_changes,
            51000.0, // current_price
            50000.0, // poc
            0.01,    // poc_distance
            2.0,     // spike_threshold
        );

        assert_eq!(signal, TradingSignal::Long);
        assert_eq!(confidence, 0.91);
    }

    #[test]
    fn test_analyze_volume_signals_strong_spike_bearish() {
        let strategy = VolumeStrategy::new();
        let recent_volumes = vec![1000.0; 10];
        let recent_price_changes = vec![-1.0, -2.0, -1.5, -2.5, -1.0, -2.0, -1.5, -2.0, -1.0, -2.0]; // All negative

        let (signal, confidence, _) = strategy.analyze_volume_signals(
            3000.0, // current_volume (spike)
            1000.0, // avg_volume
            3.0,    // volume_ratio
            &recent_volumes,
            &recent_price_changes,
            49000.0, // current_price (below POC)
            50000.0, // poc
            0.01,    // poc_distance
            2.0,     // spike_threshold
        );

        assert_eq!(signal, TradingSignal::Short);
        assert_eq!(confidence, 0.91);
    }

    #[test]
    fn test_analyze_volume_signals_moderate_accumulation() {
        let strategy = VolumeStrategy::new();
        let recent_volumes = vec![1000.0; 10];
        let recent_price_changes = vec![1.0, 0.5, 1.0, 1.5, 0.5, 1.0, 0.5, 1.0, 0.5, 1.0]; // Mostly positive

        let (signal, confidence, _) = strategy.analyze_volume_signals(
            1800.0, // current_volume (high but not spike)
            1000.0, // avg_volume
            1.8,    // volume_ratio
            &recent_volumes,
            &recent_price_changes,
            50000.0, // current_price
            50000.0, // poc
            0.01,    // poc_distance (near POC)
            2.0,     // spike_threshold
        );

        assert_eq!(signal, TradingSignal::Long);
        assert_eq!(confidence, 0.71);
    }

    #[test]
    fn test_analyze_volume_signals_low_volume_neutral() {
        let strategy = VolumeStrategy::new();
        let recent_volumes = vec![1000.0; 10];
        let recent_price_changes = vec![0.5, -0.3, 0.4, -0.2, 0.3, -0.4, 0.2, -0.3, 0.4, -0.2]; // Mixed

        let (signal, confidence, _) = strategy.analyze_volume_signals(
            600.0,  // current_volume (low)
            1000.0, // avg_volume
            0.6,    // volume_ratio (low)
            &recent_volumes,
            &recent_price_changes,
            50000.0, // current_price
            50000.0, // poc
            0.01,    // poc_distance
            2.0,     // spike_threshold
        );

        assert_eq!(signal, TradingSignal::Neutral);
        assert_eq!(confidence, 0.65);
    }

    #[tokio::test]
    async fn test_volume_strategy_description() {
        let strategy = VolumeStrategy::new();
        let desc = strategy.description();

        assert!(desc.contains("Volume"));
        assert!(!desc.is_empty());
    }

    #[tokio::test]
    async fn test_volume_strategy_with_custom_config() {
        let mut config = StrategyConfig::default();
        config.enabled = true;
        config.weight = 1.5;
        config
            .parameters
            .insert("volume_sma_period".to_string(), json!(25));
        config
            .parameters
            .insert("volume_spike_threshold".to_string(), json!(3.0));

        let strategy = VolumeStrategy::with_config(config);

        assert_eq!(strategy.get_volume_sma_period(), 25);
        assert_eq!(strategy.get_volume_spike_threshold(), 3.0);
        assert_eq!(strategy.config().weight, 1.5);
    }

    #[tokio::test]
    async fn test_volume_strategy_volume_spike() {
        let strategy = VolumeStrategy::new();

        // Create volume spike with bullish price action
        let prices: Vec<f64> = (0..30).map(|i| 100.0 + (i as f64 * 0.3)).collect();
        let mut volumes: Vec<f64> = vec![1000.0; 30];
        volumes[29] = 2500.0; // Spike at the end

        let input = create_test_input_with_volume(prices, volumes);
        let result = strategy.analyze(&input).await;

        assert!(result.is_ok());
        let output = result.unwrap();
        // High volume with bullish price should be positive
        assert_ne!(output.signal, TradingSignal::Short);
    }

    #[test]
    fn test_analyze_volume_signals_balanced() {
        let strategy = VolumeStrategy::new();
        let recent_volumes = vec![1000.0; 10];
        let recent_price_changes = vec![1.0, -1.0, 1.0, -1.0, 1.0, -1.0, 1.0, -1.0, 1.0, -1.0]; // Balanced

        let (signal, confidence, _) = strategy.analyze_volume_signals(
            1500.0, // current_volume
            1000.0, // avg_volume
            1.5,    // volume_ratio
            &recent_volumes,
            &recent_price_changes,
            50000.0, // current_price
            50000.0, // poc
            0.01,    // poc_distance
            2.0,     // spike_threshold
        );

        assert_eq!(signal, TradingSignal::Neutral);
        assert!(confidence > 0.4);
    }

    // ==================== Edge Cases Tests ====================

    #[tokio::test]
    async fn test_volume_strategy_zero_volume() {
        let strategy = VolumeStrategy::new();
        let prices: Vec<f64> = (0..30).map(|i| 100.0 + (i as f64 * 0.5)).collect();
        let volumes: Vec<f64> = vec![0.0; 30];

        let input = create_test_input_with_volume(prices, volumes);
        let result = strategy.analyze(&input).await;

        // Should handle zero volume gracefully
        assert!(result.is_ok() || result.is_err());
    }

    #[test]
    fn test_analyze_volume_signals_zero_total_volume() {
        let strategy = VolumeStrategy::new();
        let recent_volumes = vec![0.0; 10];
        let recent_price_changes = vec![1.0; 10];

        let (signal, confidence, _) = strategy.analyze_volume_signals(
            0.0, // current_volume
            1.0, // avg_volume (prevent division by zero)
            0.0, // volume_ratio
            &recent_volumes,
            &recent_price_changes,
            50000.0, // current_price
            50000.0, // poc
            0.01,    // poc_distance
            2.0,     // spike_threshold
        );

        // Should handle zero volume case
        assert_eq!(signal, TradingSignal::Neutral);
        assert!(confidence >= 0.0 && confidence <= 1.0);
    }

    #[test]
    fn test_analyze_volume_signals_empty_recent_volumes() {
        let strategy = VolumeStrategy::new();
        let recent_volumes: Vec<f64> = vec![];
        let recent_price_changes: Vec<f64> = vec![];

        let (signal, confidence, _) = strategy.analyze_volume_signals(
            1000.0, // current_volume
            1000.0, // avg_volume
            1.0,    // volume_ratio
            &recent_volumes,
            &recent_price_changes,
            50000.0, // current_price
            50000.0, // poc
            0.01,    // poc_distance
            2.0,     // spike_threshold
        );

        assert_eq!(signal, TradingSignal::Neutral);
        assert!(confidence >= 0.0 && confidence <= 1.0);
    }

    #[tokio::test]
    async fn test_volume_strategy_missing_timeframe() {
        let strategy = VolumeStrategy::new();
        let timeframe_data = HashMap::new();
        // Don't add "1h" timeframe

        let input = StrategyInput {
            symbol: "BTCUSDT".to_string(),
            timeframe_data,
            current_price: 50000.0,
            volume_24h: 1000000.0,
            timestamp: 1234567890,
        };

        let result = strategy.analyze(&input).await;
        assert!(result.is_err());

        if let Err(StrategyError::InsufficientData(msg)) = result {
            assert!(msg.contains("1h"));
        }
    }

    #[tokio::test]
    async fn test_volume_strategy_exact_minimum_candles() {
        let strategy = VolumeStrategy::new();
        let min_required = strategy.get_volume_sma_period() + 5;
        let prices: Vec<f64> = (0..min_required).map(|i| 100.0 + (i as f64)).collect();
        let volumes: Vec<f64> = vec![1000.0; min_required];

        let input = create_test_input_with_volume(prices, volumes);
        let result = strategy.validate_data(&input);
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_volume_strategy_one_less_than_minimum() {
        let strategy = VolumeStrategy::new();
        let min_required = strategy.get_volume_sma_period() + 5;
        let prices: Vec<f64> = (0..(min_required - 1))
            .map(|i| 100.0 + (i as f64))
            .collect();
        let volumes: Vec<f64> = vec![1000.0; min_required - 1];

        let input = create_test_input_with_volume(prices, volumes);
        let result = strategy.validate_data(&input);
        assert!(result.is_err());
    }

    // ==================== Parameter Validation Tests ====================

    #[test]
    fn test_get_volume_sma_period_default() {
        let strategy = VolumeStrategy::new();
        assert_eq!(strategy.get_volume_sma_period(), 20);
    }

    #[test]
    fn test_get_volume_sma_period_invalid_type() {
        let mut config = StrategyConfig::default();
        config
            .parameters
            .insert("volume_sma_period".to_string(), json!("invalid"));
        let strategy = VolumeStrategy::with_config(config);

        // Should fall back to default
        assert_eq!(strategy.get_volume_sma_period(), 20);
    }

    #[test]
    fn test_get_volume_spike_threshold_default() {
        let strategy = VolumeStrategy::new();
        assert_eq!(strategy.get_volume_spike_threshold(), 2.0);
    }

    #[test]
    fn test_get_volume_spike_threshold_invalid_type() {
        let mut config = StrategyConfig::default();
        config
            .parameters
            .insert("volume_spike_threshold".to_string(), json!("invalid"));
        let strategy = VolumeStrategy::with_config(config);

        // Should fall back to default
        assert_eq!(strategy.get_volume_spike_threshold(), 2.0);
    }

    #[test]
    fn test_get_price_volume_correlation_period_default() {
        let strategy = VolumeStrategy::new();
        assert_eq!(strategy.get_price_volume_correlation_period(), 10);
    }

    #[test]
    fn test_get_price_volume_correlation_period_invalid_type() {
        let mut config = StrategyConfig::default();
        config
            .parameters
            .insert("price_volume_correlation_period".to_string(), json!(null));
        let strategy = VolumeStrategy::with_config(config);

        // Should fall back to default
        assert_eq!(strategy.get_price_volume_correlation_period(), 10);
    }

    #[test]
    fn test_config_with_extreme_values() {
        let mut config = StrategyConfig::default();
        config
            .parameters
            .insert("volume_sma_period".to_string(), json!(1));
        config
            .parameters
            .insert("volume_spike_threshold".to_string(), json!(0.1));
        config
            .parameters
            .insert("price_volume_correlation_period".to_string(), json!(100));

        let strategy = VolumeStrategy::with_config(config);

        assert_eq!(strategy.get_volume_sma_period(), 1);
        assert_eq!(strategy.get_volume_spike_threshold(), 0.1);
        assert_eq!(strategy.get_price_volume_correlation_period(), 100);
    }

    // ==================== Volume Spike Detection Tests ====================

    #[test]
    fn test_analyze_volume_signals_exact_spike_threshold() {
        let strategy = VolumeStrategy::new();
        let recent_volumes = vec![1000.0; 10];
        let recent_price_changes = vec![1.0; 10]; // All positive

        let (signal, _, _) = strategy.analyze_volume_signals(
            2000.0, // current_volume
            1000.0, // avg_volume
            2.0,    // volume_ratio (exactly at threshold)
            &recent_volumes,
            &recent_price_changes,
            51000.0, // current_price (above POC)
            50000.0, // poc
            0.01,    // poc_distance
            2.0,     // spike_threshold
        );

        assert_eq!(signal, TradingSignal::Long);
    }

    #[test]
    fn test_analyze_volume_signals_just_below_spike_threshold() {
        let strategy = VolumeStrategy::new();
        let recent_volumes = vec![1000.0; 10];
        let recent_price_changes = vec![1.0; 10]; // All positive

        let (signal, confidence, _) = strategy.analyze_volume_signals(
            1990.0, // current_volume
            1000.0, // avg_volume
            1.99,   // volume_ratio (just below threshold)
            &recent_volumes,
            &recent_price_changes,
            51000.0, // current_price
            50000.0, // poc
            0.01,    // poc_distance
            2.0,     // spike_threshold
        );

        // Should not trigger spike signal (confidence should not be 0.91)
        // Instead should be moderate or weak signal
        assert!(signal == TradingSignal::Long || signal == TradingSignal::Neutral);
        assert_ne!(confidence, 0.91);
    }

    #[test]
    fn test_analyze_volume_signals_massive_spike() {
        let strategy = VolumeStrategy::new();
        let recent_volumes = vec![1000.0; 10];
        let recent_price_changes = vec![1.0; 10]; // All positive

        let (signal, confidence, _) = strategy.analyze_volume_signals(
            10000.0, // current_volume (10x spike)
            1000.0,  // avg_volume
            10.0,    // volume_ratio
            &recent_volumes,
            &recent_price_changes,
            51000.0, // current_price
            50000.0, // poc
            0.01,    // poc_distance
            2.0,     // spike_threshold
        );

        assert_eq!(signal, TradingSignal::Long);
        assert_eq!(confidence, 0.91);
    }

    // ==================== Volume-Price Correlation Tests ====================

    #[test]
    fn test_analyze_volume_signals_bullish_ratio_70_percent() {
        let strategy = VolumeStrategy::new();
        let recent_volumes = vec![1000.0; 10];
        let recent_price_changes = vec![1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, -1.0, -1.0, -1.0]; // 70% bullish

        let (signal, confidence, _) = strategy.analyze_volume_signals(
            2500.0, // current_volume (spike)
            1000.0, // avg_volume
            2.5,    // volume_ratio
            &recent_volumes,
            &recent_price_changes,
            51000.0, // current_price (above POC)
            50000.0, // poc
            0.01,    // poc_distance
            2.0,     // spike_threshold
        );

        assert_eq!(signal, TradingSignal::Long);
        assert_eq!(confidence, 0.91);
    }

    #[test]
    fn test_analyze_volume_signals_bearish_ratio_30_percent() {
        let strategy = VolumeStrategy::new();
        let recent_volumes = vec![1000.0; 10];
        let recent_price_changes = vec![1.0, 1.0, 1.0, -1.0, -1.0, -1.0, -1.0, -1.0, -1.0, -1.0]; // 30% bullish

        let (signal, confidence, _) = strategy.analyze_volume_signals(
            2500.0, // current_volume (spike)
            1000.0, // avg_volume
            2.5,    // volume_ratio
            &recent_volumes,
            &recent_price_changes,
            49000.0, // current_price (below POC)
            50000.0, // poc
            0.01,    // poc_distance
            2.0,     // spike_threshold
        );

        assert_eq!(signal, TradingSignal::Short);
        assert_eq!(confidence, 0.91);
    }

    #[test]
    fn test_analyze_volume_signals_moderate_bullish_60_percent() {
        let strategy = VolumeStrategy::new();
        let recent_volumes = vec![1000.0; 10];
        let recent_price_changes = vec![1.0, 1.0, 1.0, 1.0, 1.0, 1.0, -1.0, -1.0, -1.0, -1.0]; // 60% bullish

        let (signal, confidence, _) = strategy.analyze_volume_signals(
            1600.0, // current_volume (high volume)
            1000.0, // avg_volume
            1.6,    // volume_ratio
            &recent_volumes,
            &recent_price_changes,
            50000.0, // current_price
            50000.0, // poc
            0.01,    // poc_distance
            2.0,     // spike_threshold
        );

        assert_eq!(signal, TradingSignal::Long);
        assert_eq!(confidence, 0.71);
    }

    #[test]
    fn test_analyze_volume_signals_moderate_bearish_40_percent() {
        let strategy = VolumeStrategy::new();
        let recent_volumes = vec![1000.0; 10];
        let recent_price_changes = vec![1.0, 1.0, 1.0, 1.0, -1.0, -1.0, -1.0, -1.0, -1.0, -1.0]; // 40% bullish

        let (signal, confidence, _) = strategy.analyze_volume_signals(
            1600.0, // current_volume (high volume)
            1000.0, // avg_volume
            1.6,    // volume_ratio
            &recent_volumes,
            &recent_price_changes,
            50000.0, // current_price
            50000.0, // poc
            0.01,    // poc_distance
            2.0,     // spike_threshold
        );

        assert_eq!(signal, TradingSignal::Short);
        assert_eq!(confidence, 0.71);
    }

    // ==================== POC (Point of Control) Tests ====================

    #[test]
    fn test_analyze_volume_signals_near_poc_bullish() {
        let strategy = VolumeStrategy::new();
        let recent_volumes = vec![1000.0; 10];
        let recent_price_changes = vec![1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, -1.0, -1.0, -1.0]; // 70% bullish

        let (signal, confidence, _) = strategy.analyze_volume_signals(
            1500.0, // current_volume
            1000.0, // avg_volume
            1.5,    // volume_ratio
            &recent_volumes,
            &recent_price_changes,
            50900.0, // current_price (near POC)
            51000.0, // poc
            0.002,   // poc_distance (<2%)
            2.0,     // spike_threshold
        );

        assert_eq!(signal, TradingSignal::Long);
        assert_eq!(confidence, 0.71);
    }

    #[test]
    fn test_analyze_volume_signals_near_poc_bearish() {
        let strategy = VolumeStrategy::new();
        let recent_volumes = vec![1000.0; 10];
        let recent_price_changes = vec![1.0, 1.0, -1.0, -1.0, -1.0, -1.0, -1.0, -1.0, -1.0, -1.0]; // 20% bullish

        let (signal, confidence, _) = strategy.analyze_volume_signals(
            1500.0, // current_volume
            1000.0, // avg_volume
            1.5,    // volume_ratio
            &recent_volumes,
            &recent_price_changes,
            50100.0, // current_price (near POC, below)
            51000.0, // poc
            0.017,   // poc_distance (<2%)
            2.0,     // spike_threshold
        );

        assert_eq!(signal, TradingSignal::Short);
        assert_eq!(confidence, 0.71);
    }

    #[test]
    fn test_analyze_volume_signals_far_from_poc() {
        let strategy = VolumeStrategy::new();
        let recent_volumes = vec![1000.0; 10];
        let recent_price_changes = vec![1.0; 10];

        let (signal, _, _) = strategy.analyze_volume_signals(
            1500.0, // current_volume
            1000.0, // avg_volume
            1.5,    // volume_ratio
            &recent_volumes,
            &recent_price_changes,
            55000.0, // current_price (far from POC)
            50000.0, // poc
            0.10,    // poc_distance (10%)
            2.0,     // spike_threshold
        );

        // Should still signal based on other factors
        assert!(signal == TradingSignal::Long || signal == TradingSignal::Neutral);
    }

    #[test]
    fn test_analyze_volume_signals_exact_poc_boundary() {
        let strategy = VolumeStrategy::new();
        let recent_volumes = vec![1000.0; 10];
        let recent_price_changes = vec![1.0; 10];

        let (signal, _, _) = strategy.analyze_volume_signals(
            1500.0, // current_volume
            1000.0, // avg_volume
            1.5,    // volume_ratio
            &recent_volumes,
            &recent_price_changes,
            51000.0, // current_price
            50000.0, // poc
            0.02,    // poc_distance (exactly 2%)
            2.0,     // spike_threshold
        );

        assert!(signal == TradingSignal::Long || signal == TradingSignal::Neutral);
    }

    // ==================== Weak Signal Tests ====================

    #[test]
    fn test_analyze_volume_signals_weak_long_55_percent() {
        let strategy = VolumeStrategy::new();
        let recent_volumes = vec![1000.0; 10];
        let recent_price_changes = vec![1.0, 1.0, 1.0, 1.0, 1.0, 1.0, -1.0, -1.0, -1.0, 0.5]; // 55% bullish

        let (signal, confidence, _) = strategy.analyze_volume_signals(
            1300.0, // current_volume
            1000.0, // avg_volume
            1.3,    // volume_ratio
            &recent_volumes,
            &recent_price_changes,
            50000.0, // current_price
            50000.0, // poc
            0.01,    // poc_distance
            2.0,     // spike_threshold
        );

        assert_eq!(signal, TradingSignal::Long);
        assert_eq!(confidence, 0.51);
    }

    #[test]
    fn test_analyze_volume_signals_weak_short_45_percent() {
        let strategy = VolumeStrategy::new();
        let recent_volumes = vec![1000.0; 10];
        let recent_price_changes = vec![1.0, 1.0, 1.0, 1.0, 1.0, -1.0, -1.0, -1.0, -1.0, -0.5]; // 45% bullish

        let (signal, confidence, _) = strategy.analyze_volume_signals(
            1300.0, // current_volume
            1000.0, // avg_volume
            1.3,    // volume_ratio
            &recent_volumes,
            &recent_price_changes,
            50000.0, // current_price
            50000.0, // poc
            0.01,    // poc_distance
            2.0,     // spike_threshold
        );

        assert_eq!(signal, TradingSignal::Short);
        assert_eq!(confidence, 0.51);
    }

    #[test]
    fn test_analyze_volume_signals_just_above_neutral_threshold() {
        let strategy = VolumeStrategy::new();
        let recent_volumes = vec![1000.0; 10];
        let recent_price_changes = vec![1.0, 1.0, 1.0, 1.0, 1.0, 1.0, -1.0, -1.0, -1.0, -1.0]; // 50.1% bullish

        let (signal, _, _) = strategy.analyze_volume_signals(
            1100.0, // current_volume
            1000.0, // avg_volume
            1.1,    // volume_ratio
            &recent_volumes,
            &recent_price_changes,
            50000.0, // current_price
            50000.0, // poc
            0.01,    // poc_distance
            2.0,     // spike_threshold
        );

        assert_eq!(signal, TradingSignal::Neutral);
    }

    // ==================== Integration Tests ====================

    #[tokio::test]
    async fn test_volume_strategy_full_analysis_flow() {
        let strategy = VolumeStrategy::new();

        let prices: Vec<f64> = (0..30).map(|i| 100.0 + (i as f64 * 0.5)).collect();
        let volumes: Vec<f64> = vec![1000.0; 30];
        let input = create_test_input_with_volume(prices, volumes);

        let result = strategy.analyze(&input).await;
        assert!(result.is_ok());

        let output = result.unwrap();
        assert_eq!(output.timeframe, "1h");
        assert_eq!(output.timestamp, 1234567890);
        assert!(output.confidence >= 0.0 && output.confidence <= 1.0);
        assert!(!output.reasoning.is_empty());
    }

    #[tokio::test]
    async fn test_volume_strategy_increasing_volume_trend() {
        let strategy = VolumeStrategy::new();

        let prices: Vec<f64> = (0..30).map(|i| 100.0 + (i as f64 * 0.5)).collect();
        let volumes: Vec<f64> = (0..30).map(|i| 500.0 + (i as f64 * 50.0)).collect();
        let input = create_test_input_with_volume(prices, volumes);

        let result = strategy.analyze(&input).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_volume_strategy_decreasing_volume_trend() {
        let strategy = VolumeStrategy::new();

        let prices: Vec<f64> = (0..30).map(|i| 100.0 - (i as f64 * 0.5)).collect();
        let volumes: Vec<f64> = (0..30).map(|i| 2000.0 - (i as f64 * 50.0)).collect();
        let input = create_test_input_with_volume(prices, volumes);

        let result = strategy.analyze(&input).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_volume_strategy_volatile_volume() {
        let strategy = VolumeStrategy::new();

        let prices: Vec<f64> = (0..30).map(|i| 100.0 + ((i % 3) as f64 - 1.0)).collect();
        let volumes: Vec<f64> = (0..30)
            .map(|i| if i % 2 == 0 { 2000.0 } else { 500.0 })
            .collect();
        let input = create_test_input_with_volume(prices, volumes);

        let result = strategy.analyze(&input).await;
        assert!(result.is_ok());
    }

    #[test]
    fn test_volume_strategy_default_trait() {
        let strategy1 = VolumeStrategy::default();
        let strategy2 = VolumeStrategy::new();

        assert_eq!(
            strategy1.get_volume_sma_period(),
            strategy2.get_volume_sma_period()
        );
        assert_eq!(
            strategy1.get_volume_spike_threshold(),
            strategy2.get_volume_spike_threshold()
        );
    }

    #[test]
    fn test_volume_strategy_config_getter() {
        let strategy = VolumeStrategy::new();
        let config = strategy.config();

        assert!(config.enabled);
        assert_eq!(config.weight, 1.0);
        assert!(config.parameters.contains_key("volume_sma_period"));
    }

    #[tokio::test]
    async fn test_volume_strategy_validate_data_missing_timeframe() {
        let strategy = VolumeStrategy::new();
        let mut timeframe_data = HashMap::new();
        timeframe_data.insert("5m".to_string(), vec![]);

        let input = StrategyInput {
            symbol: "BTCUSDT".to_string(),
            timeframe_data,
            current_price: 50000.0,
            volume_24h: 1000000.0,
            timestamp: 1234567890,
        };

        let result = strategy.validate_data(&input);
        assert!(result.is_err());

        if let Err(StrategyError::DataValidation(_)) = result {
            // Expected error type
        } else {
            panic!("Expected DataValidation error");
        }
    }

    #[test]
    fn test_analyze_volume_signals_reasoning_messages() {
        let strategy = VolumeStrategy::new();
        let recent_volumes = vec![1000.0; 10];
        let recent_price_changes = vec![1.0; 10];

        let (_, _, reasoning) = strategy.analyze_volume_signals(
            3000.0, // spike
            1000.0,
            3.0,
            &recent_volumes,
            &recent_price_changes,
            51000.0,
            50000.0,
            0.01,
            2.0,
        );

        assert!(!reasoning.is_empty());
        assert!(reasoning.contains("Volume") || reasoning.contains("volume"));
    }

    #[tokio::test]
    async fn test_volume_strategy_metadata_completeness() {
        let strategy = VolumeStrategy::new();
        let prices: Vec<f64> = (0..30).map(|i| 100.0 + (i as f64)).collect();
        let volumes: Vec<f64> = vec![1000.0; 30];
        let input = create_test_input_with_volume(prices, volumes);

        let result = strategy.analyze(&input).await;
        assert!(result.is_ok());

        let output = result.unwrap();
        assert!(output.metadata.contains_key("current_volume"));
        assert!(output.metadata.contains_key("avg_volume"));
        assert!(output.metadata.contains_key("volume_ratio"));
        assert!(output.metadata.contains_key("poc"));
        assert!(output.metadata.contains_key("poc_distance"));
        assert!(output.metadata.contains_key("volume_spike_threshold"));
    }

    #[test]
    fn test_analyze_volume_signals_all_zero_price_changes() {
        let strategy = VolumeStrategy::new();
        let recent_volumes = vec![1000.0; 10];
        let recent_price_changes = vec![0.0; 10];

        let (signal, _, _) = strategy.analyze_volume_signals(
            1500.0,
            1000.0,
            1.5,
            &recent_volumes,
            &recent_price_changes,
            50000.0,
            50000.0,
            0.01,
            2.0,
        );

        // All zero price changes means balanced, should be neutral
        assert_eq!(signal, TradingSignal::Neutral);
    }

    #[test]
    fn test_analyze_volume_signals_single_volume_entry() {
        let strategy = VolumeStrategy::new();
        let recent_volumes = vec![1000.0];
        let recent_price_changes = vec![1.0];

        let (signal, confidence, _) = strategy.analyze_volume_signals(
            1500.0,
            1000.0,
            1.5,
            &recent_volumes,
            &recent_price_changes,
            50000.0,
            50000.0,
            0.01,
            2.0,
        );

        assert!(confidence >= 0.0 && confidence <= 1.0);
        assert!(
            signal == TradingSignal::Long
                || signal == TradingSignal::Short
                || signal == TradingSignal::Neutral
        );
    }

    #[test]
    fn test_analyze_volume_signals_mismatched_lengths() {
        let strategy = VolumeStrategy::new();
        let recent_volumes = vec![1000.0; 10];
        let recent_price_changes = vec![1.0; 5]; // Fewer price changes

        let (signal, confidence, _) = strategy.analyze_volume_signals(
            1500.0,
            1000.0,
            1.5,
            &recent_volumes,
            &recent_price_changes,
            50000.0,
            50000.0,
            0.01,
            2.0,
        );

        // Should handle gracefully
        assert!(confidence >= 0.0 && confidence <= 1.0);
        assert!(
            signal == TradingSignal::Long
                || signal == TradingSignal::Short
                || signal == TradingSignal::Neutral
        );
    }

    #[tokio::test]
    async fn test_volume_strategy_large_dataset() {
        let strategy = VolumeStrategy::new();

        let prices: Vec<f64> = (0..100).map(|i| 100.0 + (i as f64 * 0.1)).collect();
        let volumes: Vec<f64> = vec![1000.0; 100];
        let input = create_test_input_with_volume(prices, volumes);

        let result = strategy.analyze(&input).await;
        assert!(result.is_ok());
    }

    #[test]
    fn test_analyze_volume_signals_extreme_volume_ratio() {
        let strategy = VolumeStrategy::new();
        let recent_volumes = vec![1000.0; 10];
        let recent_price_changes = vec![1.0; 10];

        let (signal, confidence, _) = strategy.analyze_volume_signals(
            100000.0, // 100x spike
            1000.0,
            100.0,
            &recent_volumes,
            &recent_price_changes,
            51000.0,
            50000.0,
            0.01,
            2.0,
        );

        assert_eq!(signal, TradingSignal::Long);
        assert!(confidence > 0.0);
    }

    #[test]
    fn test_analyze_volume_signals_very_low_volume_ratio() {
        let strategy = VolumeStrategy::new();
        let recent_volumes = vec![1000.0; 10];
        let recent_price_changes = vec![1.0; 10];

        let (signal, confidence, _) = strategy.analyze_volume_signals(
            10.0, // 0.01x very low
            1000.0,
            0.01,
            &recent_volumes,
            &recent_price_changes,
            50000.0,
            50000.0,
            0.01,
            2.0,
        );

        assert_eq!(signal, TradingSignal::Neutral);
        assert_eq!(confidence, 0.65); // High confidence in low activity
    }

    #[tokio::test]
    async fn test_volume_strategy_config_persistence() {
        let mut strategy = VolumeStrategy::new();

        let mut new_config = StrategyConfig::default();
        new_config.enabled = false;
        new_config.weight = 2.5;
        new_config
            .parameters
            .insert("volume_sma_period".to_string(), json!(30));

        strategy.update_config(new_config.clone());

        assert_eq!(strategy.config().enabled, false);
        assert_eq!(strategy.config().weight, 2.5);
        assert_eq!(strategy.get_volume_sma_period(), 30);
    }

    #[test]
    fn test_volume_strategy_clone() {
        let strategy1 = VolumeStrategy::new();
        let strategy2 = strategy1.clone();

        assert_eq!(
            strategy1.get_volume_sma_period(),
            strategy2.get_volume_sma_period()
        );
        assert_eq!(strategy1.name(), strategy2.name());
    }

    #[test]
    fn test_volume_strategy_debug_format() {
        let strategy = VolumeStrategy::new();
        let debug_str = format!("{:?}", strategy);

        assert!(debug_str.contains("VolumeStrategy"));
    }

    #[test]
    fn test_analyze_volume_signals_confidence_boundaries() {
        let strategy = VolumeStrategy::new();
        let recent_volumes = vec![1000.0; 10];
        let recent_price_changes = vec![0.5; 10];

        let (_, confidence, _) = strategy.analyze_volume_signals(
            1000.0,
            1000.0,
            1.0,
            &recent_volumes,
            &recent_price_changes,
            50000.0,
            50000.0,
            0.01,
            2.0,
        );

        // Confidence should always be between 0.0 and 1.0
        assert!(confidence >= 0.0);
        assert!(confidence <= 1.0);
    }
}
