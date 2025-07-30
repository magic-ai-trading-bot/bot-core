use super::*;
use crate::strategies::indicators::calculate_bollinger_bands;
use async_trait::async_trait;
use serde_json::json;

/// Bollinger Bands-based trading strategy
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

        // Current BB values
        let upper_1h = *primary_bb.upper.last().unwrap();
        let middle_1h = *primary_bb.middle.last().unwrap();
        let lower_1h = *primary_bb.lower.last().unwrap();

        let upper_4h = *confirmation_bb.upper.last().unwrap();
        let middle_4h = *confirmation_bb.middle.last().unwrap();
        let lower_4h = *confirmation_bb.lower.last().unwrap();

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
        metadata.insert("bb_upper_1h".to_string(), json!(upper_1h));
        metadata.insert("bb_middle_1h".to_string(), json!(middle_1h));
        metadata.insert("bb_lower_1h".to_string(), json!(lower_1h));
        metadata.insert("bb_position_1h".to_string(), json!(bb_position_1h));
        metadata.insert("bb_position_4h".to_string(), json!(bb_position_4h));
        metadata.insert("bb_width_1h".to_string(), json!(bb_width_1h));
        metadata.insert("bb_width_4h".to_string(), json!(bb_width_4h));
        metadata.insert("is_squeeze_1h".to_string(), json!(is_squeeze_1h));
        metadata.insert("is_squeeze_4h".to_string(), json!(is_squeeze_4h));
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
                StrategyError::DataValidation(format!("Missing {} timeframe data", timeframe))
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

        // Trend continuation signals
        if bb_position_1h > 0.8 && bb_position_4h > 0.6 && bb_expanding_1h {
            return (
                TradingSignal::Long,
                0.69,
                "Strong uptrend with Bollinger Bands expansion".to_string(),
            );
        }

        if bb_position_1h < 0.2 && bb_position_4h < 0.4 && bb_expanding_1h {
            return (
                TradingSignal::Short,
                0.69,
                "Strong downtrend with Bollinger Bands expansion".to_string(),
            );
        }

        // Moderate signals based on position
        if bb_position_1h < 0.25 && current_price > middle_4h {
            return (
                TradingSignal::Long,
                0.58,
                "Price near lower band with support from 4H middle band".to_string(),
            );
        }

        if bb_position_1h > 0.75 && current_price < middle_4h {
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
