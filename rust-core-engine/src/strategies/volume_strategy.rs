use super::*;
use crate::strategies::indicators::{calculate_sma, calculate_volume_profile};
use async_trait::async_trait;
use serde_json::json;

/// Volume-based trading strategy
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

        if volume_sma.is_empty() {
            return Err(StrategyError::InsufficientData(
                "No volume SMA calculated".to_string(),
            ));
        }

        let current_volume = *volumes.last().unwrap();
        let avg_volume = *volume_sma.last().unwrap();
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
