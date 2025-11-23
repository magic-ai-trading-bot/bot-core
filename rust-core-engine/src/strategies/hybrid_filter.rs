use crate::market_data::cache::CandleData;
use crate::strategies::ml_trend_predictor::{MLTrendPrediction, MLTrendPredictor};
use crate::strategies::trend_filter::{TrendAlignment, TrendDirection, TrendFilter};
use crate::strategies::{StrategyOutput, TradingSignal};
use std::sync::Arc;

/// Hybrid filter configuration
#[derive(Debug, Clone)]
pub struct HybridFilterConfig {
    pub enabled: bool,
    pub use_ml: bool,
    pub ml_weight: f64,  // Weight for ML prediction (0.0 - 1.0)
    pub mtf_weight: f64, // Weight for MTF alignment (0.0 - 1.0)
    pub block_counter_trend: bool,
}

impl Default for HybridFilterConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            use_ml: true,
            ml_weight: 0.4,
            mtf_weight: 0.6,
            block_counter_trend: true,
        }
    }
}

/// Hybrid filter result
#[derive(Debug, Clone)]
pub struct FilterResult {
    pub should_block: bool,
    pub adjusted_confidence: f64,
    pub reasoning: String,
    pub mtf_alignment: Option<TrendAlignment>,
    pub ml_prediction: Option<MLTrendPrediction>,
}

/// Hybrid trend filter combining MTF + ML
pub struct HybridFilter {
    config: HybridFilterConfig,
    trend_filter: Arc<TrendFilter>,
    ml_predictor: Option<Arc<MLTrendPredictor>>,
}

impl HybridFilter {
    pub fn new(
        config: HybridFilterConfig,
        trend_filter: Arc<TrendFilter>,
        ml_predictor: Option<Arc<MLTrendPredictor>>,
    ) -> Self {
        Self {
            config,
            trend_filter,
            ml_predictor,
        }
    }

    /// Apply hybrid filter to strategy output
    pub async fn apply_filter(
        &self,
        signal: TradingSignal,
        confidence: f64,
        symbol: &str,
        candles_1d: Option<&[CandleData]>,
        candles_4h: &[CandleData],
        candles_1h: &[CandleData],
    ) -> Result<FilterResult, String> {
        if !self.config.enabled {
            return Ok(FilterResult {
                should_block: false,
                adjusted_confidence: confidence,
                reasoning: "Filter disabled".to_string(),
                mtf_alignment: None,
                ml_prediction: None,
            });
        }

        // Step 1: Check multi-timeframe alignment
        let mtf_alignment = self
            .trend_filter
            .check_alignment(candles_1d, candles_4h, candles_1h)?;

        // Step 2: Get ML prediction if enabled
        let ml_prediction = if self.config.use_ml && self.ml_predictor.is_some() {
            self.ml_predictor
                .as_ref()
                .unwrap()
                .predict_trend_with_fallback(symbol, "4h")
                .await
        } else {
            None
        };

        // Step 3: Combine signals
        let filter_result =
            self.combine_signals(signal, confidence, &mtf_alignment, ml_prediction.as_ref());

        Ok(filter_result)
    }

    /// Combine MTF and ML signals to make final decision
    fn combine_signals(
        &self,
        signal: TradingSignal,
        original_confidence: f64,
        mtf_alignment: &TrendAlignment,
        ml_prediction: Option<&MLTrendPrediction>,
    ) -> FilterResult {
        // Determine if we should block the signal
        let mut should_block = false;
        let mut adjusted_confidence = original_confidence;
        let mut reasoning_parts = Vec::new();

        match signal {
            TradingSignal::Long => {
                // Check MTF alignment for LONG
                if !mtf_alignment.is_long_aligned() {
                    if self.config.block_counter_trend {
                        should_block = true;
                        reasoning_parts.push(format!(
                            "MTF not aligned for LONG (daily: {}, 4h: {})",
                            mtf_alignment.daily, mtf_alignment.four_hour
                        ));
                    } else {
                        // Reduce confidence significantly
                        adjusted_confidence *= 0.3;
                        reasoning_parts.push("MTF weak for LONG - confidence reduced".to_string());
                    }
                }

                // Check ML prediction if available
                if let Some(ml) = ml_prediction {
                    match ml.trend {
                        TrendDirection::Uptrend => {
                            // ML confirms LONG - boost confidence
                            adjusted_confidence = (adjusted_confidence * self.config.mtf_weight)
                                + (ml.confidence * self.config.ml_weight);
                            adjusted_confidence = adjusted_confidence.min(0.95);
                            reasoning_parts.push(format!(
                                "ML confirms Uptrend ({:.0}% confidence)",
                                ml.confidence * 100.0
                            ));
                        },
                        TrendDirection::Downtrend => {
                            // ML predicts opposite direction
                            if self.config.block_counter_trend {
                                should_block = true;
                                reasoning_parts.push(format!(
                                    "ML predicts Downtrend ({:.0}% confidence) - blocking LONG",
                                    ml.confidence * 100.0
                                ));
                            } else {
                                adjusted_confidence *= 0.2;
                                reasoning_parts
                                    .push("ML conflict - confidence penalized".to_string());
                            }
                        },
                        TrendDirection::Neutral => {
                            // ML is neutral - slight penalty
                            adjusted_confidence *= 0.85;
                            reasoning_parts
                                .push("ML neutral - minor confidence reduction".to_string());
                        },
                    }
                } else {
                    // No ML prediction - rely on MTF only
                    adjusted_confidence *= mtf_alignment.alignment_score;
                    reasoning_parts.push(format!(
                        "MTF alignment score: {:.0}%",
                        mtf_alignment.alignment_score * 100.0
                    ));
                }
            },

            TradingSignal::Short => {
                // Check MTF alignment for SHORT
                if !mtf_alignment.is_short_aligned() {
                    if self.config.block_counter_trend {
                        should_block = true;
                        reasoning_parts.push(format!(
                            "MTF not aligned for SHORT (daily: {}, 4h: {})",
                            mtf_alignment.daily, mtf_alignment.four_hour
                        ));
                    } else {
                        adjusted_confidence *= 0.3;
                        reasoning_parts.push("MTF weak for SHORT - confidence reduced".to_string());
                    }
                }

                // Check ML prediction
                if let Some(ml) = ml_prediction {
                    match ml.trend {
                        TrendDirection::Downtrend => {
                            // ML confirms SHORT - boost confidence
                            adjusted_confidence = (adjusted_confidence * self.config.mtf_weight)
                                + (ml.confidence * self.config.ml_weight);
                            adjusted_confidence = adjusted_confidence.min(0.95);
                            reasoning_parts.push(format!(
                                "ML confirms Downtrend ({:.0}% confidence)",
                                ml.confidence * 100.0
                            ));
                        },
                        TrendDirection::Uptrend => {
                            // ML predicts opposite direction
                            if self.config.block_counter_trend {
                                should_block = true;
                                reasoning_parts.push(format!(
                                    "ML predicts Uptrend ({:.0}% confidence) - blocking SHORT",
                                    ml.confidence * 100.0
                                ));
                            } else {
                                adjusted_confidence *= 0.2;
                                reasoning_parts
                                    .push("ML conflict - confidence penalized".to_string());
                            }
                        },
                        TrendDirection::Neutral => {
                            adjusted_confidence *= 0.85;
                            reasoning_parts
                                .push("ML neutral - minor confidence reduction".to_string());
                        },
                    }
                } else {
                    adjusted_confidence *= mtf_alignment.alignment_score;
                    reasoning_parts.push(format!(
                        "MTF alignment score: {:.0}%",
                        mtf_alignment.alignment_score * 100.0
                    ));
                }
            },

            TradingSignal::Neutral => {
                // No filtering for neutral signals
                reasoning_parts.push("Neutral signal - no filtering applied".to_string());
            },
        }

        FilterResult {
            should_block,
            adjusted_confidence,
            reasoning: reasoning_parts.join("; "),
            mtf_alignment: Some(mtf_alignment.clone()),
            ml_prediction: ml_prediction.cloned(),
        }
    }

    /// Apply filter result to strategy output
    pub fn apply_to_output(&self, output: StrategyOutput, filter: FilterResult) -> StrategyOutput {
        if filter.should_block {
            StrategyOutput {
                signal: TradingSignal::Neutral,
                confidence: 0.2,
                reasoning: format!(
                    "BLOCKED: {}. Original: {}",
                    filter.reasoning, output.reasoning
                ),
                timeframe: output.timeframe,
                timestamp: output.timestamp,
                metadata: output.metadata,
            }
        } else {
            StrategyOutput {
                signal: output.signal,
                confidence: filter.adjusted_confidence,
                reasoning: if output.confidence != filter.adjusted_confidence {
                    format!("{}. Filter: {}", output.reasoning, filter.reasoning)
                } else {
                    output.reasoning
                },
                timeframe: output.timeframe,
                timestamp: output.timestamp,
                metadata: output.metadata,
            }
        }
    }

    /// Get configuration
    pub fn config(&self) -> &HybridFilterConfig {
        &self.config
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::strategies::trend_filter::TrendFilterConfig;

    fn create_test_filter() -> HybridFilter {
        let trend_filter_config = TrendFilterConfig {
            ema_period: 20,
            ..Default::default()
        };
        let trend_filter = Arc::new(TrendFilter::new(trend_filter_config));

        HybridFilter::new(
            HybridFilterConfig::default(),
            trend_filter,
            None, // No ML predictor in tests
        )
    }

    #[test]
    fn test_hybrid_filter_config_default() {
        let config = HybridFilterConfig::default();
        assert!(config.enabled);
        assert!(config.use_ml);
        assert_eq!(config.ml_weight, 0.4);
        assert_eq!(config.mtf_weight, 0.6);
        assert!(config.block_counter_trend);
    }

    #[test]
    fn test_filter_result_creation() {
        let result = FilterResult {
            should_block: false,
            adjusted_confidence: 0.85,
            reasoning: "Test reasoning".to_string(),
            mtf_alignment: None,
            ml_prediction: None,
        };

        assert!(!result.should_block);
        assert_eq!(result.adjusted_confidence, 0.85);
    }

    #[test]
    fn test_combine_signals_long_with_aligned_mtf() {
        let filter = create_test_filter();

        let alignment = TrendAlignment {
            daily: TrendDirection::Uptrend,
            four_hour: TrendDirection::Uptrend,
            one_hour: TrendDirection::Uptrend,
            alignment_score: 1.0,
            is_aligned: true,
        };

        let result = filter.combine_signals(TradingSignal::Long, 0.75, &alignment, None);

        assert!(!result.should_block);
        assert!(result.adjusted_confidence > 0.5);
    }

    #[test]
    fn test_combine_signals_long_with_downtrend_mtf() {
        let filter = create_test_filter();

        let alignment = TrendAlignment {
            daily: TrendDirection::Downtrend,
            four_hour: TrendDirection::Downtrend,
            one_hour: TrendDirection::Neutral,
            alignment_score: 0.2,
            is_aligned: false,
        };

        let result = filter.combine_signals(TradingSignal::Long, 0.75, &alignment, None);

        assert!(result.should_block); // Should block counter-trend
    }

    #[test]
    fn test_combine_signals_neutral() {
        let filter = create_test_filter();

        let alignment = TrendAlignment {
            daily: TrendDirection::Neutral,
            four_hour: TrendDirection::Neutral,
            one_hour: TrendDirection::Neutral,
            alignment_score: 0.5,
            is_aligned: false,
        };

        let result = filter.combine_signals(TradingSignal::Neutral, 0.5, &alignment, None);

        assert!(!result.should_block);
        assert_eq!(result.adjusted_confidence, 0.5);
    }

    #[test]
    fn test_apply_to_output_not_blocked() {
        let filter = create_test_filter();

        let output = StrategyOutput {
            signal: TradingSignal::Long,
            confidence: 0.75,
            reasoning: "Strong uptrend".to_string(),
            timeframe: "1h".to_string(),
            timestamp: 1234567890,
            metadata: Default::default(),
        };

        let filter_result = FilterResult {
            should_block: false,
            adjusted_confidence: 0.85,
            reasoning: "MTF aligned".to_string(),
            mtf_alignment: None,
            ml_prediction: None,
        };

        let result = filter.apply_to_output(output.clone(), filter_result);
        assert_eq!(result.signal, TradingSignal::Long);
        assert_eq!(result.confidence, 0.85);
    }

    #[test]
    fn test_apply_to_output_blocked() {
        let filter = create_test_filter();

        let output = StrategyOutput {
            signal: TradingSignal::Long,
            confidence: 0.75,
            reasoning: "Strong uptrend".to_string(),
            timeframe: "1h".to_string(),
            timestamp: 1234567890,
            metadata: Default::default(),
        };

        let filter_result = FilterResult {
            should_block: true,
            adjusted_confidence: 0.2,
            reasoning: "Counter-trend".to_string(),
            mtf_alignment: None,
            ml_prediction: None,
        };

        let result = filter.apply_to_output(output, filter_result);
        assert_eq!(result.signal, TradingSignal::Neutral);
        assert_eq!(result.confidence, 0.2);
        assert!(result.reasoning.contains("BLOCKED"));
    }
}
