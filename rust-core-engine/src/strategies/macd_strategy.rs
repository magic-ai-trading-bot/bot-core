use super::*;
use crate::strategies::indicators::calculate_macd;
use async_trait::async_trait;
use serde_json::json;

/// MACD-based trading strategy
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

        if primary_macd.histogram.is_empty() || confirmation_macd.histogram.is_empty() {
            return Err(StrategyError::InsufficientData(
                "No MACD values calculated".to_string(),
            ));
        }

        // Get current and previous values
        let current_macd_1h = *primary_macd.macd_line.last().unwrap();
        let current_signal_1h = *primary_macd.signal_line.last().unwrap();
        let current_histogram_1h = *primary_macd.histogram.last().unwrap();

        let current_macd_4h = *confirmation_macd.macd_line.last().unwrap();
        let current_signal_4h = *confirmation_macd.signal_line.last().unwrap();
        let current_histogram_4h = *confirmation_macd.histogram.last().unwrap();

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
                StrategyError::DataValidation(format!("Missing {timeframe} timeframe data"))
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
            || (histogram_above_zero_1h && histogram_increasing_1h && !histogram_below_zero_4h)
        {
            return (
                TradingSignal::Long,
                0.71,
                "Bullish MACD momentum building".to_string(),
            );
        }

        // Moderate bearish signals
        if (bearish_crossover_1h && histogram_decreasing_4h)
            || (histogram_below_zero_1h && histogram_decreasing_1h && !histogram_above_zero_4h)
        {
            return (
                TradingSignal::Short,
                0.71,
                "Bearish MACD momentum building".to_string(),
            );
        }

        // Weak bullish signals
        if histogram_increasing_1h && macd_1h > signal_1h && histogram_1h > prev_histogram_1h * 1.1
        {
            return (
                TradingSignal::Long,
                0.55,
                "Weak bullish momentum with MACD above signal line".to_string(),
            );
        }

        // Weak bearish signals
        if histogram_decreasing_1h && macd_1h < signal_1h && histogram_1h < prev_histogram_1h * 1.1
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
