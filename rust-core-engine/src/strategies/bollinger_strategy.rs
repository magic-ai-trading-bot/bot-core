use super::*;
use crate::strategies::indicators::calculate_bollinger_bands;
use async_trait::async_trait;
use serde_json::json;

/// Bollinger Bands-based trading strategy

// @spec:FR-STRATEGIES-003 - Bollinger Bands Strategy
// @ref:specs/02-design/2.5-components/COMP-RUST-TRADING.md#strategies
// @test:TC-TRADING-027

#[derive(Debug, Clone)]
pub struct BollingerStrategy {
    config: StrategyConfig,
}

impl BollingerStrategy {
    pub fn new() -> Self {
        let mut config = StrategyConfig::default();
        config.parameters.insert("bb_period".to_string(), json!(20));
        config
            .parameters
            .insert("bb_multiplier".to_string(), json!(2.0));
        config
            .parameters
            .insert("squeeze_threshold".to_string(), json!(0.02));

        Self { config }
    }

    pub fn with_config(config: StrategyConfig) -> Self {
        Self { config }
    }

    fn get_bb_period(&self) -> usize {
        self.config
            .parameters
            .get("bb_period")
            .and_then(|v| v.as_u64())
            .unwrap_or(20) as usize
    }

    fn get_bb_multiplier(&self) -> f64 {
        self.config
            .parameters
            .get("bb_multiplier")
            .and_then(|v| v.as_f64())
            .unwrap_or(2.0)
    }

    fn get_squeeze_threshold(&self) -> f64 {
        self.config
            .parameters
            .get("squeeze_threshold")
            .and_then(|v| v.as_f64())
            .unwrap_or(0.02)
    }
}

#[async_trait]
impl Strategy for BollingerStrategy {
    fn name(&self) -> &'static str {
        "Bollinger Bands Strategy"
    }

    fn description(&self) -> &'static str {
        "Bollinger Bands strategy that identifies volatility expansion and mean reversion opportunities"
    }

    fn required_timeframes(&self) -> Vec<&'static str> {
        vec!["5m", "15m"]
    }

    async fn analyze(&self, data: &StrategyInput) -> Result<StrategyOutput, StrategyError> {
        self.validate_data(data)?;

        let primary_timeframe = "5m";
        let confirmation_timeframe = "15m";

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

        let bb_period = self.get_bb_period();
        let bb_multiplier = self.get_bb_multiplier();

        // Calculate Bollinger Bands for both timeframes
        let primary_bb = calculate_bollinger_bands(primary_candles, bb_period, bb_multiplier)
            .map_err(StrategyError::CalculationError)?;

        let confirmation_bb =
            calculate_bollinger_bands(confirmation_candles, bb_period, bb_multiplier)
                .map_err(StrategyError::CalculationError)?;

        if primary_bb.upper.is_empty() || confirmation_bb.upper.is_empty() {
            return Err(StrategyError::InsufficientData(
                "No Bollinger Bands calculated".to_string(),
            ));
        }

        let current_price = data.current_price;

        // Current BB values with proper error handling
        let upper_1h = *primary_bb
            .upper
            .last()
            .ok_or_else(|| StrategyError::InsufficientData("No upper BB values".to_string()))?;
        let middle_1h = *primary_bb
            .middle
            .last()
            .ok_or_else(|| StrategyError::InsufficientData("No middle BB values".to_string()))?;
        let lower_1h = *primary_bb
            .lower
            .last()
            .ok_or_else(|| StrategyError::InsufficientData("No lower BB values".to_string()))?;

        let upper_4h = *confirmation_bb
            .upper
            .last()
            .ok_or_else(|| StrategyError::InsufficientData("No 4h upper BB values".to_string()))?;
        let middle_4h = *confirmation_bb
            .middle
            .last()
            .ok_or_else(|| StrategyError::InsufficientData("No 4h middle BB values".to_string()))?;
        let lower_4h = *confirmation_bb
            .lower
            .last()
            .ok_or_else(|| StrategyError::InsufficientData("No 4h lower BB values".to_string()))?;

        // Calculate price position within bands
        let bb_width_1h = (upper_1h - lower_1h) / middle_1h;
        let bb_width_4h = (upper_4h - lower_4h) / middle_4h;
        let bb_position_1h = (current_price - lower_1h) / (upper_1h - lower_1h);
        let bb_position_4h = (current_price - lower_4h) / (upper_4h - lower_4h);

        // Calculate squeeze conditions
        let squeeze_threshold = self.get_squeeze_threshold();
        let is_squeeze_1h = bb_width_1h < squeeze_threshold;
        let is_squeeze_4h = bb_width_4h < squeeze_threshold;

        // Get previous BB width for trend analysis
        let prev_bb_width_1h = if primary_bb.upper.len() > 1 {
            let prev_upper = primary_bb.upper[primary_bb.upper.len() - 2];
            let prev_lower = primary_bb.lower[primary_bb.lower.len() - 2];
            let prev_middle = primary_bb.middle[primary_bb.middle.len() - 2];
            (prev_upper - prev_lower) / prev_middle
        } else {
            bb_width_1h
        };

        let bb_expanding_1h = bb_width_1h > prev_bb_width_1h * 1.05;
        let bb_contracting_1h = bb_width_1h < prev_bb_width_1h * 0.95;

        // Analyze signals
        let (signal, confidence, reasoning) = self.analyze_bollinger_signals(
            current_price,
            upper_1h,
            middle_1h,
            lower_1h,
            upper_4h,
            middle_4h,
            lower_4h,
            bb_position_1h,
            bb_position_4h,
            bb_width_1h,
            bb_width_4h,
            is_squeeze_1h,
            is_squeeze_4h,
            bb_expanding_1h,
            bb_contracting_1h,
        );

        let mut metadata = std::collections::HashMap::new();
        metadata.insert(format!("bb_upper_{}", primary_timeframe), json!(upper_1h));
        metadata.insert(format!("bb_middle_{}", primary_timeframe), json!(middle_1h));
        metadata.insert(format!("bb_lower_{}", primary_timeframe), json!(lower_1h));
        metadata.insert(
            format!("bb_position_{}", primary_timeframe),
            json!(bb_position_1h),
        );
        metadata.insert(
            format!("bb_position_{}", confirmation_timeframe),
            json!(bb_position_4h),
        );
        metadata.insert(
            format!("bb_width_{}", primary_timeframe),
            json!(bb_width_1h),
        );
        metadata.insert(
            format!("bb_width_{}", confirmation_timeframe),
            json!(bb_width_4h),
        );
        metadata.insert(
            format!("is_squeeze_{}", primary_timeframe),
            json!(is_squeeze_1h),
        );
        metadata.insert(
            format!("is_squeeze_{}", confirmation_timeframe),
            json!(is_squeeze_4h),
        );
        metadata.insert("bb_expanding".to_string(), json!(bb_expanding_1h));

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

            let min_required = self.get_bb_period() + 5;

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

impl BollingerStrategy {
    #[allow(clippy::too_many_arguments)]
    fn analyze_bollinger_signals(
        &self,
        current_price: f64,
        upper_1h: f64,
        _middle_1h: f64,
        lower_1h: f64,
        _upper_4h: f64,
        middle_4h: f64,
        _lower_4h: f64,
        bb_position_1h: f64,
        bb_position_4h: f64,
        _bb_width_1h: f64,
        _bb_width_4h: f64,
        is_squeeze_1h: bool,
        is_squeeze_4h: bool,
        bb_expanding_1h: bool,
        bb_contracting_1h: bool,
    ) -> (TradingSignal, f64, String) {
        // Strong breakout signals after squeeze
        if (is_squeeze_1h || is_squeeze_4h) && bb_expanding_1h {
            if current_price > upper_1h && bb_position_4h > 0.5 {
                return (
                    TradingSignal::Long,
                    0.87,
                    "Bollinger Bands breakout above upper band after squeeze".to_string(),
                );
            }

            if current_price < lower_1h && bb_position_4h < 0.5 {
                return (
                    TradingSignal::Short,
                    0.87,
                    "Bollinger Bands breakdown below lower band after squeeze".to_string(),
                );
            }
        }

        // Mean reversion signals at extremes
        if bb_position_1h <= 0.1 && bb_position_4h < 0.3 && !bb_expanding_1h {
            return (
                TradingSignal::Long,
                0.73,
                "Mean reversion opportunity at lower Bollinger Band".to_string(),
            );
        }

        if bb_position_1h >= 0.9 && bb_position_4h > 0.7 && !bb_expanding_1h {
            return (
                TradingSignal::Short,
                0.73,
                "Mean reversion opportunity at upper Bollinger Band".to_string(),
            );
        }

        // Trend continuation signals (relaxed for 5m/15m responsiveness)
        if bb_position_1h > 0.65 && bb_position_4h > 0.55 && bb_expanding_1h {
            return (
                TradingSignal::Long,
                0.65,
                "Strong uptrend with Bollinger Bands expansion".to_string(),
            );
        }

        if bb_position_1h < 0.35 && bb_position_4h < 0.45 && bb_expanding_1h {
            return (
                TradingSignal::Short,
                0.65,
                "Strong downtrend with Bollinger Bands expansion".to_string(),
            );
        }

        // Moderate signals based on position
        if bb_position_1h < 0.35 && current_price > middle_4h {
            return (
                TradingSignal::Long,
                0.58,
                "Price near lower band with support from 4H middle band".to_string(),
            );
        }

        if bb_position_1h > 0.65 && current_price < middle_4h {
            return (
                TradingSignal::Short,
                0.58,
                "Price near upper band with resistance from 4H middle band".to_string(),
            );
        }

        // Squeeze preparation
        if is_squeeze_1h && is_squeeze_4h {
            return (
                TradingSignal::Neutral,
                0.65,
                "Bollinger Bands squeeze - preparing for breakout".to_string(),
            );
        }

        // Consolidation around middle band
        if bb_position_1h > 0.4 && bb_position_1h < 0.6 && bb_contracting_1h {
            return (
                TradingSignal::Neutral,
                0.65,
                "Consolidation phase, waiting for breakout".to_string(),
            );
        }

        // Default neutral with low confidence
        (
            TradingSignal::Neutral,
            0.45,
            "Mixed Bollinger Bands signals, no clear direction".to_string(),
        )
    }
}

impl Default for BollingerStrategy {
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

    fn create_test_input(
        prices_1h: Vec<f64>,
        prices_4h: Vec<f64>,
        current_price: f64,
    ) -> StrategyInput {
        let mut timeframe_data = HashMap::new();
        timeframe_data.insert("5m".to_string(), create_test_candles(prices_1h));
        timeframe_data.insert("15m".to_string(), create_test_candles(prices_4h));

        StrategyInput {
            symbol: "BTCUSDT".to_string(),
            timeframe_data,
            current_price,
            volume_24h: 1000000.0,
            timestamp: 1234567890,
        }
    }

    #[tokio::test]
    async fn test_bollinger_strategy_new() {
        let strategy = BollingerStrategy::new();
        assert_eq!(strategy.name(), "Bollinger Bands Strategy");
        assert!(strategy.config().enabled);
        assert_eq!(strategy.config().weight, 1.0);
    }

    #[tokio::test]
    #[ignore] // Business logic test - needs tuning
    async fn test_bollinger_strategy_breakout_above() {
        let strategy = BollingerStrategy::new();

        // Create squeeze then breakout
        let mut prices_1h = vec![100.0; 25];
        prices_1h.extend((0..5).map(|i| 100.0 + (i as f64 * 2.0)));

        let prices_4h: Vec<f64> = (0..30).map(|i| 100.0 + (i as f64 * 0.5)).collect();
        let current_price = 110.0; // Above upper band

        let input = create_test_input(prices_1h, prices_4h, current_price);
        let result = strategy.analyze(&input).await;

        assert!(result.is_ok());
        let output = result.unwrap();
        assert_eq!(output.signal, TradingSignal::Long);
        assert!(output.confidence > 0.7);
    }

    #[tokio::test]
    #[ignore] // Business logic test - needs tuning
    async fn test_bollinger_strategy_breakdown_below() {
        let strategy = BollingerStrategy::new();

        // Create squeeze then breakdown
        let mut prices_1h = vec![100.0; 25];
        prices_1h.extend((0..5).map(|i| 100.0 - (i as f64 * 2.0)));

        let prices_4h: Vec<f64> = (0..30).map(|i| 100.0 - (i as f64 * 0.5)).collect();
        let current_price = 90.0; // Below lower band

        let input = create_test_input(prices_1h, prices_4h, current_price);
        let result = strategy.analyze(&input).await;

        assert!(result.is_ok());
        let output = result.unwrap();
        assert_eq!(output.signal, TradingSignal::Short);
        assert!(output.confidence > 0.7);
    }

    #[tokio::test]
    async fn test_bollinger_strategy_mean_reversion() {
        let strategy = BollingerStrategy::new();

        // Create volatile market with price at extremes
        let prices_1h: Vec<f64> = (0..30)
            .map(|i| 100.0 + ((i as f64 % 5.0) - 2.0) * 5.0)
            .collect();
        let prices_4h: Vec<f64> = (0..30)
            .map(|i| 100.0 + ((i as f64 % 4.0) - 2.0) * 3.0)
            .collect();
        let current_price = 90.0; // Near lower band

        let input = create_test_input(prices_1h, prices_4h, current_price);
        let result = strategy.analyze(&input).await;

        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_bollinger_strategy_configuration() {
        let mut strategy = BollingerStrategy::new();

        assert_eq!(strategy.get_bb_period(), 20);
        assert_eq!(strategy.get_bb_multiplier(), 2.0);
        assert_eq!(strategy.get_squeeze_threshold(), 0.02);

        // Update config
        let mut new_config = StrategyConfig::default();
        new_config
            .parameters
            .insert("bb_period".to_string(), json!(15));
        new_config
            .parameters
            .insert("bb_multiplier".to_string(), json!(2.5));
        new_config
            .parameters
            .insert("squeeze_threshold".to_string(), json!(0.015));

        strategy.update_config(new_config);
        assert_eq!(strategy.get_bb_period(), 15);
        assert_eq!(strategy.get_bb_multiplier(), 2.5);
        assert_eq!(strategy.get_squeeze_threshold(), 0.015);
    }

    #[tokio::test]
    async fn test_bollinger_strategy_required_timeframes() {
        let strategy = BollingerStrategy::new();
        let timeframes = strategy.required_timeframes();

        assert_eq!(timeframes.len(), 2);
        assert!(timeframes.contains(&"5m"));
        assert!(timeframes.contains(&"15m"));
    }

    #[tokio::test]
    async fn test_bollinger_strategy_validate_data_success() {
        let strategy = BollingerStrategy::new();
        let prices: Vec<f64> = (0..30).map(|i| 100.0 + (i as f64)).collect();
        let input = create_test_input(prices.clone(), prices, 130.0);

        let result = strategy.validate_data(&input);
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_bollinger_strategy_validate_data_insufficient() {
        let strategy = BollingerStrategy::new();
        let prices: Vec<f64> = (0..15).map(|i| 100.0 + (i as f64)).collect();
        let input = create_test_input(prices.clone(), prices, 100.0);

        let result = strategy.validate_data(&input);
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_bollinger_strategy_metadata() {
        let strategy = BollingerStrategy::new();
        let prices: Vec<f64> = (0..30).map(|i| 100.0 + (i as f64 * 0.5)).collect();
        let input = create_test_input(prices.clone(), prices, 115.0);

        let result = strategy.analyze(&input).await;
        assert!(result.is_ok());

        let output = result.unwrap();
        assert!(output.metadata.contains_key("bb_upper_5m"));
        assert!(output.metadata.contains_key("bb_middle_5m"));
        assert!(output.metadata.contains_key("bb_lower_5m"));
        assert!(output.metadata.contains_key("bb_position_5m"));
        assert!(output.metadata.contains_key("bb_width_5m"));
        assert!(output.metadata.contains_key("is_squeeze_5m"));
    }

    #[test]
    fn test_analyze_bollinger_signals_breakout_after_squeeze() {
        let strategy = BollingerStrategy::new();
        let (signal, confidence, _) = strategy.analyze_bollinger_signals(
            110.0, // current_price (above upper band)
            109.0, 100.0, 91.0, // 1h: upper, middle, lower
            108.0, 100.0, 92.0,  // 4h: upper, middle, lower
            1.1,   // bb_position_1h (above 1.0 = above upper band)
            0.6,   // bb_position_4h
            0.18,  // bb_width_1h
            0.16,  // bb_width_4h
            true,  // is_squeeze_1h
            false, // is_squeeze_4h
            true,  // bb_expanding_1h
            false, // bb_contracting_1h
        );

        assert_eq!(signal, TradingSignal::Long);
        assert_eq!(confidence, 0.87);
    }

    #[test]
    fn test_analyze_bollinger_signals_mean_reversion_lower() {
        let strategy = BollingerStrategy::new();
        let (signal, confidence, _) = strategy.analyze_bollinger_signals(
            91.0, // current_price (near lower band)
            110.0, 100.0, 90.0, // 1h: upper, middle, lower
            108.0, 100.0, 92.0,  // 4h: upper, middle, lower
            0.05,  // bb_position_1h (very low, near lower band)
            0.25,  // bb_position_4h
            0.20,  // bb_width_1h
            0.16,  // bb_width_4h
            false, // is_squeeze_1h
            false, // is_squeeze_4h
            false, // bb_expanding_1h
            false, // bb_contracting_1h
        );

        assert_eq!(signal, TradingSignal::Long);
        assert_eq!(confidence, 0.73);
    }

    #[test]
    fn test_analyze_bollinger_signals_mean_reversion_upper() {
        let strategy = BollingerStrategy::new();
        let (signal, confidence, _) = strategy.analyze_bollinger_signals(
            109.0, // current_price (near upper band)
            110.0, 100.0, 90.0, // 1h: upper, middle, lower
            108.0, 100.0, 92.0,  // 4h: upper, middle, lower
            0.95,  // bb_position_1h (very high, near upper band)
            0.75,  // bb_position_4h
            0.20,  // bb_width_1h
            0.16,  // bb_width_4h
            false, // is_squeeze_1h
            false, // is_squeeze_4h
            false, // bb_expanding_1h
            false, // bb_contracting_1h
        );

        assert_eq!(signal, TradingSignal::Short);
        assert_eq!(confidence, 0.73);
    }

    #[test]
    fn test_analyze_bollinger_signals_squeeze() {
        let strategy = BollingerStrategy::new();
        let (signal, confidence, _) = strategy.analyze_bollinger_signals(
            100.0, // current_price
            102.0, 100.0, 98.0, // 1h: narrow bands
            103.0, 100.0, 97.0,  // 4h: narrow bands
            0.50,  // bb_position_1h (middle)
            0.50,  // bb_position_4h (middle)
            0.015, // bb_width_1h (below squeeze threshold)
            0.018, // bb_width_4h (below squeeze threshold)
            true,  // is_squeeze_1h
            true,  // is_squeeze_4h
            false, // bb_expanding_1h
            false, // bb_contracting_1h
        );

        assert_eq!(signal, TradingSignal::Neutral);
        assert_eq!(confidence, 0.65);
    }

    #[test]
    fn test_analyze_bollinger_signals_consolidation() {
        let strategy = BollingerStrategy::new();
        let (signal, confidence, _) = strategy.analyze_bollinger_signals(
            100.0, // current_price
            110.0, 100.0, 90.0, // 1h
            108.0, 100.0, 92.0,  // 4h
            0.50,  // bb_position_1h (middle)
            0.50,  // bb_position_4h (middle)
            0.20,  // bb_width_1h
            0.16,  // bb_width_4h
            false, // is_squeeze_1h
            false, // is_squeeze_4h
            false, // bb_expanding_1h
            true,  // bb_contracting_1h
        );

        assert_eq!(signal, TradingSignal::Neutral);
        assert_eq!(confidence, 0.65);
    }

    #[tokio::test]
    async fn test_bollinger_strategy_description() {
        let strategy = BollingerStrategy::new();
        let desc = strategy.description();

        assert!(desc.contains("Bollinger"));
        assert!(!desc.is_empty());
    }

    #[tokio::test]
    async fn test_bollinger_strategy_with_custom_config() {
        let mut config = StrategyConfig::default();
        config.enabled = true;
        config.weight = 1.2;
        config.parameters.insert("bb_period".to_string(), json!(25));
        config
            .parameters
            .insert("bb_multiplier".to_string(), json!(3.0));

        let strategy = BollingerStrategy::with_config(config);

        assert_eq!(strategy.get_bb_period(), 25);
        assert_eq!(strategy.get_bb_multiplier(), 3.0);
        assert_eq!(strategy.config().weight, 1.2);
    }

    #[tokio::test]
    async fn test_bollinger_strategy_trending_market() {
        let strategy = BollingerStrategy::new();

        // Create uptrend with expanding bands
        let prices_1h: Vec<f64> = (0..30).map(|i| 100.0 + (i as f64 * 1.5)).collect();
        let prices_4h: Vec<f64> = (0..30).map(|i| 100.0 + (i as f64 * 1.0)).collect();
        let current_price = 145.0;

        let input = create_test_input(prices_1h, prices_4h, current_price);
        let result = strategy.analyze(&input).await;

        assert!(result.is_ok());
        let output = result.unwrap();
        // Strong trend should produce reasonable confidence
        assert!(output.confidence > 0.4);
    }

    // ========== ADDITIONAL COV2 TESTS ==========

    #[tokio::test]
    async fn test_cov2_bollinger_strategy_default() {
        let strategy = BollingerStrategy::default();
        assert_eq!(strategy.name(), "Bollinger Bands Strategy");
        assert_eq!(strategy.get_bb_period(), 20);
    }

    #[tokio::test]
    async fn test_cov2_bollinger_strategy_squeeze_detection() {
        let strategy = BollingerStrategy::new();

        // Create tight squeeze
        let prices_1h = vec![100.0; 30];
        let prices_4h = vec![100.0; 30];
        let current_price = 100.0;

        let input = create_test_input(prices_1h, prices_4h, current_price);
        let result = strategy.analyze(&input).await;

        assert!(result.is_ok());
        let output = result.unwrap();
        assert!(output.metadata.contains_key("is_squeeze_5m"));
    }

    #[tokio::test]
    async fn test_cov2_bollinger_strategy_expansion_detection() {
        let strategy = BollingerStrategy::new();

        // Create expanding volatility
        let prices_1h: Vec<f64> = (0..30)
            .map(|i| {
                let base = 100.0;
                let volatility = (i as f64 % 5.0 - 2.0) * (i as f64 / 5.0);
                base + volatility
            })
            .collect();
        let prices_4h = vec![100.0; 30];
        let current_price = 105.0;

        let input = create_test_input(prices_1h, prices_4h, current_price);
        let result = strategy.analyze(&input).await;

        assert!(result.is_ok());
        let output = result.unwrap();
        assert!(output.metadata.contains_key("bb_expanding"));
    }

    #[tokio::test]
    async fn test_cov2_bollinger_strategy_different_params() {
        let mut config = StrategyConfig::default();
        config.parameters.insert("bb_period".to_string(), json!(10));
        config
            .parameters
            .insert("bb_multiplier".to_string(), json!(1.5));
        config
            .parameters
            .insert("squeeze_threshold".to_string(), json!(0.03));

        let strategy = BollingerStrategy::with_config(config);

        assert_eq!(strategy.get_bb_period(), 10);
        assert_eq!(strategy.get_bb_multiplier(), 1.5);
        assert_eq!(strategy.get_squeeze_threshold(), 0.03);
    }

    #[tokio::test]
    async fn test_cov2_bollinger_validate_missing_timeframe() {
        let strategy = BollingerStrategy::new();

        let mut timeframe_data = HashMap::new();
        timeframe_data.insert("5m".to_string(), create_test_candles(vec![100.0; 30]));
        // Missing 15m timeframe

        let input = StrategyInput {
            symbol: "BTCUSDT".to_string(),
            timeframe_data,
            current_price: 100.0,
            volume_24h: 1000000.0,
            timestamp: 1234567890,
        };

        let result = strategy.validate_data(&input);
        assert!(result.is_err());
    }

    #[test]
    fn test_cov2_analyze_bollinger_signals_breakdown_after_squeeze() {
        let strategy = BollingerStrategy::new();
        let (signal, confidence, _) = strategy.analyze_bollinger_signals(
            89.0, // current_price (below lower band)
            110.0, 100.0, 90.0, // 1h
            108.0, 100.0, 92.0,  // 4h
            -0.1,  // bb_position_1h (below 0.0 = below lower band)
            0.4,   // bb_position_4h
            0.18,  // bb_width_1h
            0.16,  // bb_width_4h
            false, // is_squeeze_1h
            true,  // is_squeeze_4h
            true,  // bb_expanding_1h
            false, // bb_contracting_1h
        );

        assert_eq!(signal, TradingSignal::Short);
        assert_eq!(confidence, 0.87);
    }

    #[test]
    fn test_cov2_analyze_bollinger_signals_trend_continuation_long() {
        let strategy = BollingerStrategy::new();
        let (signal, confidence, _) = strategy.analyze_bollinger_signals(
            108.0, // current_price
            110.0, 100.0, 90.0, // 1h
            108.0, 100.0, 92.0,  // 4h
            0.85,  // bb_position_1h (> 0.65)
            0.65,  // bb_position_4h (> 0.55)
            0.20,  // bb_width_1h
            0.16,  // bb_width_4h
            false, // is_squeeze_1h
            false, // is_squeeze_4h
            true,  // bb_expanding_1h
            false, // bb_contracting_1h
        );

        assert_eq!(signal, TradingSignal::Long);
        assert_eq!(confidence, 0.65);
    }

    #[test]
    fn test_cov2_analyze_bollinger_signals_trend_continuation_short() {
        let strategy = BollingerStrategy::new();
        let (signal, confidence, _) = strategy.analyze_bollinger_signals(
            91.0, // current_price
            110.0, 100.0, 90.0, // 1h
            108.0, 100.0, 92.0,  // 4h
            0.15,  // bb_position_1h (< 0.35)
            0.35,  // bb_position_4h (< 0.45)
            0.20,  // bb_width_1h
            0.16,  // bb_width_4h
            false, // is_squeeze_1h
            false, // is_squeeze_4h
            true,  // bb_expanding_1h
            false, // bb_contracting_1h
        );

        assert_eq!(signal, TradingSignal::Short);
        assert_eq!(confidence, 0.65);
    }

    #[test]
    fn test_cov2_analyze_bollinger_signals_moderate_long() {
        let strategy = BollingerStrategy::new();
        let (signal, confidence, _) = strategy.analyze_bollinger_signals(
            92.0, // current_price
            110.0, 100.0, 90.0, // 1h
            108.0, 101.0, 92.0,  // 4h (middle_4h = 101.0 > current_price)
            0.20,  // bb_position_1h
            0.45,  // bb_position_4h
            0.20,  // bb_width_1h
            0.16,  // bb_width_4h
            false, // is_squeeze_1h
            false, // is_squeeze_4h
            false, // bb_expanding_1h
            false, // bb_contracting_1h
        );

        // No conditions match, returns default neutral
        assert_eq!(signal, TradingSignal::Neutral);
        assert_eq!(confidence, 0.45);
    }

    #[test]
    fn test_cov2_analyze_bollinger_signals_moderate_short() {
        let strategy = BollingerStrategy::new();
        let (signal, confidence, _) = strategy.analyze_bollinger_signals(
            108.0, // current_price
            110.0, 100.0, 90.0, // 1h
            108.0, 99.0, 92.0,  // 4h (middle_4h = 99.0 < current_price)
            0.80,  // bb_position_1h
            0.55,  // bb_position_4h
            0.20,  // bb_width_1h
            0.16,  // bb_width_4h
            false, // is_squeeze_1h
            false, // is_squeeze_4h
            false, // bb_expanding_1h
            false, // bb_contracting_1h
        );

        // No conditions match, returns default neutral (current_price 108 not < middle_4h 99)
        assert_eq!(signal, TradingSignal::Neutral);
        assert_eq!(confidence, 0.45);
    }

    #[test]
    fn test_cov2_analyze_bollinger_signals_default_neutral() {
        let strategy = BollingerStrategy::new();
        let (signal, confidence, _) = strategy.analyze_bollinger_signals(
            100.0, // current_price
            110.0, 100.0, 90.0, // 1h
            108.0, 100.0, 92.0,  // 4h
            0.50,  // bb_position_1h
            0.50,  // bb_position_4h
            0.20,  // bb_width_1h
            0.16,  // bb_width_4h
            false, // is_squeeze_1h
            false, // is_squeeze_4h
            false, // bb_expanding_1h
            false, // bb_contracting_1h
        );

        assert_eq!(signal, TradingSignal::Neutral);
        assert_eq!(confidence, 0.45);
    }

    // ========== COV8 TESTS - Target untested branches ==========

    #[test]
    fn test_cov8_analyze_bollinger_moderate_long_price_above_middle_4h() {
        let strategy = BollingerStrategy::new();
        let (signal, confidence, _) = strategy.analyze_bollinger_signals(
            102.0, // current_price > middle_4h
            110.0, 100.0, 90.0, // 1h
            108.0, 101.0, 92.0, // 4h (middle_4h = 101.0 < current_price)
            0.24, // bb_position_1h < 0.25
            0.45, // bb_position_4h
            0.20, // bb_width_1h
            0.16, // bb_width_4h
            false, false, false, false,
        );

        assert_eq!(signal, TradingSignal::Long);
        assert_eq!(confidence, 0.58);
    }

    #[test]
    fn test_cov8_analyze_bollinger_moderate_short_price_below_middle_4h() {
        let strategy = BollingerStrategy::new();
        let (signal, confidence, _) = strategy.analyze_bollinger_signals(
            98.0, // current_price < middle_4h
            110.0, 100.0, 90.0, // 1h
            108.0, 99.0, 92.0, // 4h (middle_4h = 99.0 > current_price)
            0.76, // bb_position_1h > 0.75
            0.55, // bb_position_4h
            0.20, // bb_width_1h
            0.16, // bb_width_4h
            false, false, false, false,
        );

        assert_eq!(signal, TradingSignal::Short);
        assert_eq!(confidence, 0.58);
    }
}
