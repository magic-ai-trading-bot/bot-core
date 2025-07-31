use super::*;
use crate::strategies::indicators::calculate_rsi;
use async_trait::async_trait;
use serde_json::json;

/// RSI-based trading strategy
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

        if primary_rsi.is_empty() || confirmation_rsi.is_empty() {
            return Err(StrategyError::InsufficientData(
                "No RSI values calculated".to_string(),
            ));
        }

        let current_rsi_1h = *primary_rsi.last().unwrap();
        let current_rsi_4h = *confirmation_rsi.last().unwrap();

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
