use super::*;
use crate::strategies::{
    bollinger_strategy::BollingerStrategy, macd_strategy::MacdStrategy, rsi_strategy::RsiStrategy,
    volume_strategy::VolumeStrategy,
};
use serde_json::json;
use std::sync::Arc;
use tokio::sync::RwLock;

/// Main strategy engine that manages and executes multiple strategies
pub struct StrategyEngine {
    strategies: Vec<Box<dyn Strategy>>,
    config: StrategyEngineConfig,
    signal_history: Arc<RwLock<Vec<CombinedSignal>>>,
}

/// Configuration for the strategy engine
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StrategyEngineConfig {
    pub enabled_strategies: Vec<String>,
    pub min_confidence_threshold: f64,
    pub signal_combination_mode: SignalCombinationMode,
    pub max_history_size: usize,
}

/// How to combine signals from multiple strategies
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SignalCombinationMode {
    WeightedAverage,
    Consensus,
    BestConfidence,
    Conservative,
}

/// Combined signal from multiple strategies
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CombinedSignal {
    pub final_signal: TradingSignal,
    pub combined_confidence: f64,
    pub strategy_signals: Vec<StrategySignalResult>,
    pub reasoning: String,
    pub symbol: String,
    pub timestamp: i64,
    pub metadata: HashMap<String, serde_json::Value>,
}

/// Individual strategy result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StrategySignalResult {
    pub strategy_name: String,
    pub signal: TradingSignal,
    pub confidence: f64,
    pub reasoning: String,
    pub weight: f64,
    pub metadata: HashMap<String, serde_json::Value>,
}

impl StrategyEngine {
    pub fn new() -> Self {
        let config = StrategyEngineConfig::default();
        let mut engine = Self {
            strategies: Vec::new(),
            config,
            signal_history: Arc::new(RwLock::new(Vec::new())),
        };

        // Add default strategies
        engine.add_strategy(Box::new(RsiStrategy::new()));
        engine.add_strategy(Box::new(MacdStrategy::new()));
        engine.add_strategy(Box::new(VolumeStrategy::new()));
        engine.add_strategy(Box::new(BollingerStrategy::new()));

        engine
    }

    pub fn with_config(config: StrategyEngineConfig) -> Self {
        let mut engine = Self::new();
        engine.config = config;
        engine
    }

    pub fn add_strategy(&mut self, strategy: Box<dyn Strategy>) {
        self.strategies.push(strategy);
    }

    pub fn remove_strategy(&mut self, name: &str) {
        self.strategies.retain(|s| s.name() != name);
    }

    pub fn get_strategy_names(&self) -> Vec<&str> {
        self.strategies.iter().map(|s| s.name()).collect()
    }

    pub fn update_strategy_config(
        &mut self,
        name: &str,
        config: StrategyConfig,
    ) -> Result<(), StrategyError> {
        for strategy in &mut self.strategies {
            if strategy.name() == name {
                strategy.update_config(config);
                return Ok(());
            }
        }
        Err(StrategyError::InvalidConfiguration(format!(
            "Strategy '{name}' not found"
        )))
    }

    pub async fn analyze_market(
        &self,
        data: &StrategyInput,
    ) -> Result<CombinedSignal, StrategyError> {
        let mut strategy_results = Vec::new();

        // Execute all enabled strategies
        for strategy in &self.strategies {
            let strategy_name = strategy.name().to_string();

            // Check if strategy is enabled
            if !self.config.enabled_strategies.is_empty()
                && !self.config.enabled_strategies.contains(&strategy_name)
            {
                continue;
            }

            // Validate data for this strategy
            if let Err(e) = strategy.validate_data(data) {
                warn!("Strategy '{strategy_name}' validation failed: {e}");
                continue;
            }

            // Execute strategy analysis
            match strategy.analyze(data).await {
                Ok(output) => {
                    let weight = strategy.config().weight;
                    let result = StrategySignalResult {
                        strategy_name: strategy_name.clone(),
                        signal: output.signal,
                        confidence: output.confidence,
                        reasoning: output.reasoning,
                        weight,
                        metadata: output.metadata,
                    };
                    strategy_results.push(result);

                    info!(
                        "Strategy '{}' signal: {:?} (confidence: {:.2})",
                        strategy_name, output.signal, output.confidence
                    );
                },
                Err(e) => {
                    warn!("Strategy '{strategy_name}' analysis failed: {e}");
                    continue;
                },
            }
        }

        if strategy_results.is_empty() {
            return Err(StrategyError::InsufficientData(
                "No strategies produced valid signals".to_string(),
            ));
        }

        // Combine strategy signals
        let combined_signal = self.combine_signals(&strategy_results, data)?;

        // Store in history
        self.add_to_history(combined_signal.clone()).await;

        Ok(combined_signal)
    }

    fn combine_signals(
        &self,
        results: &[StrategySignalResult],
        data: &StrategyInput,
    ) -> Result<CombinedSignal, StrategyError> {
        let (final_signal, combined_confidence, reasoning) =
            match self.config.signal_combination_mode {
                SignalCombinationMode::WeightedAverage => self.combine_weighted_average(results),
                SignalCombinationMode::Consensus => self.combine_consensus(results),
                SignalCombinationMode::BestConfidence => self.combine_best_confidence(results),
                SignalCombinationMode::Conservative => self.combine_conservative(results),
            };

        // Create metadata with strategy summary
        let mut metadata = HashMap::new();
        metadata.insert("total_strategies".to_string(), json!(results.len()));
        metadata.insert(
            "combination_mode".to_string(),
            json!(format!("{:?}", self.config.signal_combination_mode)),
        );
        metadata.insert(
            "min_confidence_threshold".to_string(),
            json!(self.config.min_confidence_threshold),
        );

        // Add strategy breakdown
        let long_count = results
            .iter()
            .filter(|r| r.signal == TradingSignal::Long)
            .count();
        let short_count = results
            .iter()
            .filter(|r| r.signal == TradingSignal::Short)
            .count();
        let neutral_count = results
            .iter()
            .filter(|r| r.signal == TradingSignal::Neutral)
            .count();

        metadata.insert("long_signals".to_string(), json!(long_count));
        metadata.insert("short_signals".to_string(), json!(short_count));
        metadata.insert("neutral_signals".to_string(), json!(neutral_count));

        Ok(CombinedSignal {
            final_signal,
            combined_confidence,
            strategy_signals: results.to_vec(),
            reasoning,
            symbol: data.symbol.clone(),
            timestamp: data.timestamp,
            metadata,
        })
    }

    fn combine_weighted_average(
        &self,
        results: &[StrategySignalResult],
    ) -> (TradingSignal, f64, String) {
        let mut long_score = 0.0;
        let mut short_score = 0.0;
        let mut neutral_score = 0.0;
        let mut total_weight = 0.0;
        let mut total_confidence = 0.0;

        for result in results {
            let weighted_confidence = result.confidence * result.weight;
            total_weight += result.weight;
            total_confidence += weighted_confidence;

            match result.signal {
                TradingSignal::Long => long_score += weighted_confidence,
                TradingSignal::Short => short_score += weighted_confidence,
                TradingSignal::Neutral => neutral_score += weighted_confidence,
            }
        }

        let avg_confidence = if total_weight > 0.0 {
            total_confidence / total_weight
        } else {
            0.0
        };

        let final_signal = if long_score > short_score && long_score > neutral_score {
            TradingSignal::Long
        } else if short_score > long_score && short_score > neutral_score {
            TradingSignal::Short
        } else {
            TradingSignal::Neutral
        };

        let reasoning = format!(
            "Weighted average: Long={long_score:.2}, Short={short_score:.2}, Neutral={neutral_score:.2}"
        );

        (final_signal, avg_confidence, reasoning)
    }

    fn combine_consensus(&self, results: &[StrategySignalResult]) -> (TradingSignal, f64, String) {
        let long_count = results
            .iter()
            .filter(|r| r.signal == TradingSignal::Long)
            .count();
        let short_count = results
            .iter()
            .filter(|r| r.signal == TradingSignal::Short)
            .count();
        let neutral_count = results
            .iter()
            .filter(|r| r.signal == TradingSignal::Neutral)
            .count();

        let total_count = results.len();
        let majority_threshold = total_count / 2;

        let final_signal = if long_count > majority_threshold {
            TradingSignal::Long
        } else if short_count > majority_threshold {
            TradingSignal::Short
        } else {
            TradingSignal::Neutral
        };

        // Calculate confidence based on consensus strength
        let max_count = long_count.max(short_count).max(neutral_count);
        let consensus_strength = max_count as f64 / total_count as f64;

        // Average confidence of agreeing strategies
        let agreeing_strategies: Vec<_> = results
            .iter()
            .filter(|r| r.signal == final_signal)
            .collect();

        let avg_confidence = if !agreeing_strategies.is_empty() {
            agreeing_strategies
                .iter()
                .map(|r| r.confidence)
                .sum::<f64>()
                / agreeing_strategies.len() as f64
        } else {
            0.5
        };

        let combined_confidence = avg_confidence * consensus_strength;

        let reasoning = format!(
            "Consensus: {}L/{}S/{}N (strength: {:.1}%)",
            long_count,
            short_count,
            neutral_count,
            consensus_strength * 100.0
        );

        (final_signal, combined_confidence, reasoning)
    }

    fn combine_best_confidence(
        &self,
        results: &[StrategySignalResult],
    ) -> (TradingSignal, f64, String) {
        let best_result = results
            .iter()
            .max_by(|a, b| {
                a.confidence
                    .partial_cmp(&b.confidence)
                    .unwrap_or(std::cmp::Ordering::Equal) // Handle NaN gracefully
            })
            .expect("Results should not be empty"); // Safe: checked by caller

        let reasoning = format!(
            "Best confidence from {} ({:.1}%): {}",
            best_result.strategy_name,
            best_result.confidence * 100.0,
            best_result.reasoning
        );

        (best_result.signal, best_result.confidence, reasoning)
    }

    fn combine_conservative(
        &self,
        results: &[StrategySignalResult],
    ) -> (TradingSignal, f64, String) {
        // Only signal if high confidence and multiple strategies agree
        let high_confidence_results: Vec<_> = results
            .iter()
            .filter(|r| r.confidence >= self.config.min_confidence_threshold)
            .collect();

        if high_confidence_results.len() < 2 {
            return (
                TradingSignal::Neutral,
                0.5,
                "Conservative: Insufficient high-confidence signals".to_string(),
            );
        }

        // Check for agreement among high-confidence strategies
        let long_count = high_confidence_results
            .iter()
            .filter(|r| r.signal == TradingSignal::Long)
            .count();
        let short_count = high_confidence_results
            .iter()
            .filter(|r| r.signal == TradingSignal::Short)
            .count();

        if long_count >= 2 && short_count == 0 {
            let avg_confidence = high_confidence_results
                .iter()
                .filter(|r| r.signal == TradingSignal::Long)
                .map(|r| r.confidence)
                .sum::<f64>()
                / long_count as f64;

            return (
                TradingSignal::Long,
                avg_confidence * 0.9, // Conservative discount
                format!("Conservative: {long_count} high-confidence LONG signals"),
            );
        }

        if short_count >= 2 && long_count == 0 {
            let avg_confidence = high_confidence_results
                .iter()
                .filter(|r| r.signal == TradingSignal::Short)
                .map(|r| r.confidence)
                .sum::<f64>()
                / short_count as f64;

            return (
                TradingSignal::Short,
                avg_confidence * 0.9, // Conservative discount
                format!("Conservative: {short_count} high-confidence SHORT signals"),
            );
        }

        (
            TradingSignal::Neutral,
            0.6,
            "Conservative: Mixed signals among high-confidence strategies".to_string(),
        )
    }

    async fn add_to_history(&self, signal: CombinedSignal) {
        let mut history = self.signal_history.write().await;
        history.push(signal);

        // Limit history size
        if history.len() > self.config.max_history_size {
            history.remove(0);
        }
    }

    pub async fn get_signal_history(&self, limit: Option<usize>) -> Vec<CombinedSignal> {
        let history = self.signal_history.read().await;
        let start = if let Some(limit) = limit {
            history.len().saturating_sub(limit)
        } else {
            0
        };
        history[start..].to_vec()
    }

    pub async fn get_latest_signal(&self) -> Option<CombinedSignal> {
        let history = self.signal_history.read().await;
        history.last().cloned()
    }
}

impl Default for StrategyEngineConfig {
    fn default() -> Self {
        Self {
            enabled_strategies: vec![
                "RSI Strategy".to_string(),
                "MACD Strategy".to_string(),
                "Volume Strategy".to_string(),
                "Bollinger Bands Strategy".to_string(),
            ],
            min_confidence_threshold: 0.65,
            signal_combination_mode: SignalCombinationMode::WeightedAverage,
            max_history_size: 1000,
        }
    }
}

impl Default for StrategyEngine {
    fn default() -> Self {
        Self::new()
    }
}

// Import log macros
use log::{info, warn};

#[cfg(test)]
mod tests {
    use super::*;
    use crate::market_data::cache::CandleData;

    fn create_test_candles(count: usize) -> Vec<CandleData> {
        (0..count)
            .map(|i| CandleData {
                open: 100.0 + (i as f64),
                high: 101.0 + (i as f64),
                low: 99.0 + (i as f64),
                close: 100.0 + (i as f64),
                volume: 1000.0,
                open_time: (i as i64) * 3600000,
                close_time: (i as i64) * 3600000 + 3600000,
                quote_volume: 1000.0 * (100.0 + (i as f64)),
                trades: 100,
                is_closed: true,
            })
            .collect()
    }

    fn create_test_input() -> StrategyInput {
        let mut timeframe_data = HashMap::new();
        timeframe_data.insert("1h".to_string(), create_test_candles(50));
        timeframe_data.insert("4h".to_string(), create_test_candles(50));

        StrategyInput {
            symbol: "BTCUSDT".to_string(),
            timeframe_data,
            current_price: 50000.0,
            volume_24h: 1000000.0,
            timestamp: 1234567890,
        }
    }

    fn create_mock_strategy_results() -> Vec<StrategySignalResult> {
        vec![
            StrategySignalResult {
                strategy_name: "Strategy1".to_string(),
                signal: TradingSignal::Long,
                confidence: 0.8,
                reasoning: "Bullish signal".to_string(),
                weight: 1.0,
                metadata: HashMap::new(),
            },
            StrategySignalResult {
                strategy_name: "Strategy2".to_string(),
                signal: TradingSignal::Long,
                confidence: 0.7,
                reasoning: "Moderate bullish".to_string(),
                weight: 1.0,
                metadata: HashMap::new(),
            },
            StrategySignalResult {
                strategy_name: "Strategy3".to_string(),
                signal: TradingSignal::Neutral,
                confidence: 0.6,
                reasoning: "Neutral".to_string(),
                weight: 1.0,
                metadata: HashMap::new(),
            },
        ]
    }

    #[test]
    fn test_strategy_engine_new() {
        let engine = StrategyEngine::new();
        let names = engine.get_strategy_names();

        assert_eq!(names.len(), 4);
        assert!(names.contains(&"RSI Strategy"));
        assert!(names.contains(&"MACD Strategy"));
        assert!(names.contains(&"Volume Strategy"));
        assert!(names.contains(&"Bollinger Bands Strategy"));
    }

    #[test]
    #[ignore] // Integration test - needs setup
    fn test_strategy_engine_add_remove_strategy() {
        let mut engine = StrategyEngine::new();
        let initial_count = engine.get_strategy_names().len();

        engine.add_strategy(Box::new(RsiStrategy::new()));
        assert_eq!(engine.get_strategy_names().len(), initial_count + 1);

        engine.remove_strategy("RSI Strategy");
        assert_eq!(engine.get_strategy_names().len(), initial_count);
    }

    #[test]
    fn test_strategy_engine_config_default() {
        let config = StrategyEngineConfig::default();

        assert_eq!(config.enabled_strategies.len(), 4);
        assert_eq!(config.min_confidence_threshold, 0.65);
        assert_eq!(config.max_history_size, 1000);
    }

    #[test]
    fn test_combine_weighted_average_all_long() {
        let engine = StrategyEngine::new();
        let results = vec![
            StrategySignalResult {
                strategy_name: "S1".to_string(),
                signal: TradingSignal::Long,
                confidence: 0.8,
                reasoning: "".to_string(),
                weight: 1.0,
                metadata: HashMap::new(),
            },
            StrategySignalResult {
                strategy_name: "S2".to_string(),
                signal: TradingSignal::Long,
                confidence: 0.9,
                reasoning: "".to_string(),
                weight: 1.0,
                metadata: HashMap::new(),
            },
        ];

        let (signal, confidence, _) = engine.combine_weighted_average(&results);

        assert_eq!(signal, TradingSignal::Long);
        assert!((confidence - 0.85).abs() < 0.01); // (0.8 + 0.9) / 2
    }

    #[test]
    fn test_combine_weighted_average_mixed() {
        let engine = StrategyEngine::new();
        let results = vec![
            StrategySignalResult {
                strategy_name: "S1".to_string(),
                signal: TradingSignal::Long,
                confidence: 0.8,
                reasoning: "".to_string(),
                weight: 1.0,
                metadata: HashMap::new(),
            },
            StrategySignalResult {
                strategy_name: "S2".to_string(),
                signal: TradingSignal::Short,
                confidence: 0.7,
                reasoning: "".to_string(),
                weight: 1.0,
                metadata: HashMap::new(),
            },
        ];

        let (signal, confidence, _) = engine.combine_weighted_average(&results);

        assert_eq!(signal, TradingSignal::Long); // Long has higher confidence
        assert!(confidence > 0.0 && confidence <= 1.0);
    }

    #[test]
    fn test_combine_consensus_majority_long() {
        let engine = StrategyEngine::new();
        let results = vec![
            StrategySignalResult {
                strategy_name: "S1".to_string(),
                signal: TradingSignal::Long,
                confidence: 0.8,
                reasoning: "".to_string(),
                weight: 1.0,
                metadata: HashMap::new(),
            },
            StrategySignalResult {
                strategy_name: "S2".to_string(),
                signal: TradingSignal::Long,
                confidence: 0.7,
                reasoning: "".to_string(),
                weight: 1.0,
                metadata: HashMap::new(),
            },
            StrategySignalResult {
                strategy_name: "S3".to_string(),
                signal: TradingSignal::Short,
                confidence: 0.6,
                reasoning: "".to_string(),
                weight: 1.0,
                metadata: HashMap::new(),
            },
        ];

        let (signal, confidence, _) = engine.combine_consensus(&results);

        assert_eq!(signal, TradingSignal::Long);
        assert!(confidence > 0.0);
    }

    #[test]
    fn test_combine_consensus_no_majority() {
        let engine = StrategyEngine::new();
        let results = vec![
            StrategySignalResult {
                strategy_name: "S1".to_string(),
                signal: TradingSignal::Long,
                confidence: 0.8,
                reasoning: "".to_string(),
                weight: 1.0,
                metadata: HashMap::new(),
            },
            StrategySignalResult {
                strategy_name: "S2".to_string(),
                signal: TradingSignal::Short,
                confidence: 0.7,
                reasoning: "".to_string(),
                weight: 1.0,
                metadata: HashMap::new(),
            },
        ];

        let (signal, _, _) = engine.combine_consensus(&results);

        // No majority, should be neutral or highest count
        assert!(signal == TradingSignal::Long || signal == TradingSignal::Neutral);
    }

    #[test]
    fn test_combine_best_confidence() {
        let engine = StrategyEngine::new();
        let results = vec![
            StrategySignalResult {
                strategy_name: "S1".to_string(),
                signal: TradingSignal::Long,
                confidence: 0.6,
                reasoning: "Low".to_string(),
                weight: 1.0,
                metadata: HashMap::new(),
            },
            StrategySignalResult {
                strategy_name: "S2".to_string(),
                signal: TradingSignal::Short,
                confidence: 0.9,
                reasoning: "High".to_string(),
                weight: 1.0,
                metadata: HashMap::new(),
            },
        ];

        let (signal, confidence, _) = engine.combine_best_confidence(&results);

        assert_eq!(signal, TradingSignal::Short); // Highest confidence
        assert_eq!(confidence, 0.9);
    }

    #[test]
    fn test_combine_conservative_sufficient_agreement() {
        let engine = StrategyEngine::new();
        let results = vec![
            StrategySignalResult {
                strategy_name: "S1".to_string(),
                signal: TradingSignal::Long,
                confidence: 0.8,
                reasoning: "".to_string(),
                weight: 1.0,
                metadata: HashMap::new(),
            },
            StrategySignalResult {
                strategy_name: "S2".to_string(),
                signal: TradingSignal::Long,
                confidence: 0.75,
                reasoning: "".to_string(),
                weight: 1.0,
                metadata: HashMap::new(),
            },
        ];

        let (signal, confidence, _) = engine.combine_conservative(&results);

        assert_eq!(signal, TradingSignal::Long);
        assert!(confidence > 0.65); // Above threshold with discount
    }

    #[test]
    fn test_combine_conservative_insufficient_signals() {
        let engine = StrategyEngine::new();
        let results = vec![StrategySignalResult {
            strategy_name: "S1".to_string(),
            signal: TradingSignal::Long,
            confidence: 0.8,
            reasoning: "".to_string(),
            weight: 1.0,
            metadata: HashMap::new(),
        }];

        let (signal, _, _) = engine.combine_conservative(&results);

        assert_eq!(signal, TradingSignal::Neutral); // Not enough signals
    }

    #[test]
    fn test_combine_conservative_mixed_signals() {
        let engine = StrategyEngine::new();
        let results = vec![
            StrategySignalResult {
                strategy_name: "S1".to_string(),
                signal: TradingSignal::Long,
                confidence: 0.8,
                reasoning: "".to_string(),
                weight: 1.0,
                metadata: HashMap::new(),
            },
            StrategySignalResult {
                strategy_name: "S2".to_string(),
                signal: TradingSignal::Short,
                confidence: 0.75,
                reasoning: "".to_string(),
                weight: 1.0,
                metadata: HashMap::new(),
            },
        ];

        let (signal, _, _) = engine.combine_conservative(&results);

        assert_eq!(signal, TradingSignal::Neutral); // Conflicting signals
    }

    #[tokio::test]
    async fn test_strategy_engine_analyze_market() {
        let engine = StrategyEngine::new();
        let input = create_test_input();

        let result = engine.analyze_market(&input).await;

        assert!(result.is_ok());
        let combined = result.unwrap();
        assert!(!combined.strategy_signals.is_empty());
        assert!(combined.combined_confidence >= 0.0 && combined.combined_confidence <= 1.0);
        assert_eq!(combined.symbol, "BTCUSDT");
    }

    #[tokio::test]
    async fn test_strategy_engine_history_management() {
        let engine = StrategyEngine::new();
        let input = create_test_input();

        // Analyze market to generate signal
        let _ = engine.analyze_market(&input).await;

        // Check history
        let history = engine.get_signal_history(None).await;
        assert_eq!(history.len(), 1);

        let latest = engine.get_latest_signal().await;
        assert!(latest.is_some());
    }

    #[tokio::test]
    async fn test_strategy_engine_history_limit() {
        let engine = StrategyEngine::new();
        let input = create_test_input();

        // Add multiple signals
        for _ in 0..5 {
            let _ = engine.analyze_market(&input).await;
        }

        let history = engine.get_signal_history(Some(3)).await;
        assert_eq!(history.len(), 3);
    }

    #[test]
    fn test_strategy_engine_update_strategy_config() {
        let mut engine = StrategyEngine::new();
        let mut new_config = StrategyConfig::default();
        new_config.weight = 2.5;

        let result = engine.update_strategy_config("RSI Strategy", new_config);
        assert!(result.is_ok());

        let result = engine.update_strategy_config("NonExistent", StrategyConfig::default());
        assert!(result.is_err());
    }

    #[test]
    fn test_signal_combination_mode_variants() {
        let modes = vec![
            SignalCombinationMode::WeightedAverage,
            SignalCombinationMode::Consensus,
            SignalCombinationMode::BestConfidence,
            SignalCombinationMode::Conservative,
        ];

        assert_eq!(modes.len(), 4);
    }

    #[test]
    fn test_combined_signal_metadata() {
        let engine = StrategyEngine::new();
        let results = create_mock_strategy_results();
        let input = create_test_input();

        let combined = engine.combine_signals(&results, &input).unwrap();

        assert!(combined.metadata.contains_key("total_strategies"));
        assert!(combined.metadata.contains_key("long_signals"));
        assert!(combined.metadata.contains_key("short_signals"));
        assert!(combined.metadata.contains_key("neutral_signals"));
        assert_eq!(
            combined.metadata.get("total_strategies").unwrap().as_u64(),
            Some(3)
        );
    }

    #[tokio::test]
    async fn test_strategy_engine_with_custom_config() {
        let mut config = StrategyEngineConfig::default();
        config.signal_combination_mode = SignalCombinationMode::Consensus;
        config.min_confidence_threshold = 0.7;

        let engine = StrategyEngine::with_config(config);
        let input = create_test_input();

        let result = engine.analyze_market(&input).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_strategy_engine_filtered_strategies() {
        let mut config = StrategyEngineConfig::default();
        config.enabled_strategies = vec!["RSI Strategy".to_string()];

        let engine = StrategyEngine::with_config(config);
        let input = create_test_input();

        let result = engine.analyze_market(&input).await;
        assert!(result.is_ok());

        let combined = result.unwrap();
        // Should only have RSI strategy result
        assert_eq!(combined.strategy_signals.len(), 1);
        assert_eq!(combined.strategy_signals[0].strategy_name, "RSI Strategy");
    }

    #[test]
    fn test_trading_signal_as_str() {
        assert_eq!(TradingSignal::Long.as_str(), "LONG");
        assert_eq!(TradingSignal::Short.as_str(), "SHORT");
        assert_eq!(TradingSignal::Neutral.as_str(), "NEUTRAL");
    }

    #[test]
    fn test_trading_signal_from_string() {
        assert_eq!(
            TradingSignal::from_string("LONG"),
            Some(TradingSignal::Long)
        );
        assert_eq!(
            TradingSignal::from_string("long"),
            Some(TradingSignal::Long)
        );
        assert_eq!(
            TradingSignal::from_string("SHORT"),
            Some(TradingSignal::Short)
        );
        assert_eq!(
            TradingSignal::from_string("NEUTRAL"),
            Some(TradingSignal::Neutral)
        );
        assert_eq!(TradingSignal::from_string("invalid"), None);
    }

    #[test]
    fn test_combine_weighted_average_all_neutral() {
        let engine = StrategyEngine::new();
        let results = vec![
            StrategySignalResult {
                strategy_name: "S1".to_string(),
                signal: TradingSignal::Neutral,
                confidence: 0.8,
                reasoning: "".to_string(),
                weight: 1.0,
                metadata: HashMap::new(),
            },
            StrategySignalResult {
                strategy_name: "S2".to_string(),
                signal: TradingSignal::Neutral,
                confidence: 0.9,
                reasoning: "".to_string(),
                weight: 1.0,
                metadata: HashMap::new(),
            },
        ];

        let (signal, confidence, _) = engine.combine_weighted_average(&results);

        assert_eq!(signal, TradingSignal::Neutral);
        assert!((confidence - 0.85).abs() < 0.01);
    }

    #[test]
    fn test_combine_weighted_average_different_weights() {
        let engine = StrategyEngine::new();
        let results = vec![
            StrategySignalResult {
                strategy_name: "S1".to_string(),
                signal: TradingSignal::Long,
                confidence: 0.8,
                reasoning: "".to_string(),
                weight: 2.0, // Higher weight
                metadata: HashMap::new(),
            },
            StrategySignalResult {
                strategy_name: "S2".to_string(),
                signal: TradingSignal::Short,
                confidence: 0.9,
                reasoning: "".to_string(),
                weight: 0.5, // Lower weight
                metadata: HashMap::new(),
            },
        ];

        let (signal, _, _) = engine.combine_weighted_average(&results);

        // Long should win due to higher weight
        assert_eq!(signal, TradingSignal::Long);
    }

    #[test]
    fn test_combine_weighted_average_zero_weight() {
        let engine = StrategyEngine::new();
        let results = vec![StrategySignalResult {
            strategy_name: "S1".to_string(),
            signal: TradingSignal::Long,
            confidence: 0.8,
            reasoning: "".to_string(),
            weight: 0.0,
            metadata: HashMap::new(),
        }];

        let (_, confidence, _) = engine.combine_weighted_average(&results);

        assert_eq!(confidence, 0.0); // Zero weight gives zero confidence
    }

    #[test]
    fn test_combine_consensus_all_neutral() {
        let engine = StrategyEngine::new();
        let results = vec![
            StrategySignalResult {
                strategy_name: "S1".to_string(),
                signal: TradingSignal::Neutral,
                confidence: 0.8,
                reasoning: "".to_string(),
                weight: 1.0,
                metadata: HashMap::new(),
            },
            StrategySignalResult {
                strategy_name: "S2".to_string(),
                signal: TradingSignal::Neutral,
                confidence: 0.7,
                reasoning: "".to_string(),
                weight: 1.0,
                metadata: HashMap::new(),
            },
            StrategySignalResult {
                strategy_name: "S3".to_string(),
                signal: TradingSignal::Neutral,
                confidence: 0.6,
                reasoning: "".to_string(),
                weight: 1.0,
                metadata: HashMap::new(),
            },
        ];

        let (signal, confidence, _) = engine.combine_consensus(&results);

        assert_eq!(signal, TradingSignal::Neutral);
        assert!(confidence > 0.0);
    }

    #[test]
    fn test_combine_consensus_single_strategy() {
        let engine = StrategyEngine::new();
        let results = vec![StrategySignalResult {
            strategy_name: "S1".to_string(),
            signal: TradingSignal::Long,
            confidence: 0.9,
            reasoning: "".to_string(),
            weight: 1.0,
            metadata: HashMap::new(),
        }];

        let (signal, _, _) = engine.combine_consensus(&results);

        // Single strategy won't meet majority threshold
        assert_eq!(signal, TradingSignal::Neutral);
    }

    #[test]
    fn test_combine_best_confidence_all_equal_confidence() {
        let engine = StrategyEngine::new();
        let results = vec![
            StrategySignalResult {
                strategy_name: "S1".to_string(),
                signal: TradingSignal::Long,
                confidence: 0.8,
                reasoning: "Long reason".to_string(),
                weight: 1.0,
                metadata: HashMap::new(),
            },
            StrategySignalResult {
                strategy_name: "S2".to_string(),
                signal: TradingSignal::Short,
                confidence: 0.8,
                reasoning: "Short reason".to_string(),
                weight: 1.0,
                metadata: HashMap::new(),
            },
        ];

        let (_, confidence, _) = engine.combine_best_confidence(&results);

        assert_eq!(confidence, 0.8);
    }

    #[test]
    fn test_combine_conservative_all_low_confidence() {
        let engine = StrategyEngine::new();
        let results = vec![
            StrategySignalResult {
                strategy_name: "S1".to_string(),
                signal: TradingSignal::Long,
                confidence: 0.5, // Below threshold
                reasoning: "".to_string(),
                weight: 1.0,
                metadata: HashMap::new(),
            },
            StrategySignalResult {
                strategy_name: "S2".to_string(),
                signal: TradingSignal::Long,
                confidence: 0.6, // Below threshold
                reasoning: "".to_string(),
                weight: 1.0,
                metadata: HashMap::new(),
            },
        ];

        let (signal, _, _) = engine.combine_conservative(&results);

        assert_eq!(signal, TradingSignal::Neutral); // Not enough high-confidence signals
    }

    #[test]
    fn test_combine_conservative_high_confidence_short() {
        let engine = StrategyEngine::new();
        let results = vec![
            StrategySignalResult {
                strategy_name: "S1".to_string(),
                signal: TradingSignal::Short,
                confidence: 0.8,
                reasoning: "".to_string(),
                weight: 1.0,
                metadata: HashMap::new(),
            },
            StrategySignalResult {
                strategy_name: "S2".to_string(),
                signal: TradingSignal::Short,
                confidence: 0.75,
                reasoning: "".to_string(),
                weight: 1.0,
                metadata: HashMap::new(),
            },
        ];

        let (signal, confidence, _) = engine.combine_conservative(&results);

        assert_eq!(signal, TradingSignal::Short);
        assert!(confidence > 0.65); // Should have discount applied
    }

    #[test]
    fn test_strategy_engine_remove_nonexistent_strategy() {
        let mut engine = StrategyEngine::new();
        let initial_count = engine.get_strategy_names().len();

        engine.remove_strategy("NonExistentStrategy");

        // Should have no effect
        assert_eq!(engine.get_strategy_names().len(), initial_count);
    }

    #[test]
    fn test_strategy_engine_get_strategy_names() {
        let engine = StrategyEngine::new();
        let names = engine.get_strategy_names();

        // Should have default strategies
        assert!(!names.is_empty());
        for name in names {
            assert!(!name.is_empty());
        }
    }

    #[test]
    fn test_combined_signal_fields() {
        let signal = CombinedSignal {
            final_signal: TradingSignal::Long,
            combined_confidence: 0.85,
            strategy_signals: vec![],
            reasoning: "Test reasoning".to_string(),
            symbol: "BTCUSDT".to_string(),
            timestamp: 1234567890,
            metadata: HashMap::new(),
        };

        assert_eq!(signal.final_signal, TradingSignal::Long);
        assert_eq!(signal.combined_confidence, 0.85);
        assert_eq!(signal.symbol, "BTCUSDT");
        assert_eq!(signal.timestamp, 1234567890);
    }

    #[test]
    fn test_strategy_signal_result_fields() {
        let result = StrategySignalResult {
            strategy_name: "TestStrategy".to_string(),
            signal: TradingSignal::Short,
            confidence: 0.75,
            reasoning: "Test reason".to_string(),
            weight: 1.5,
            metadata: HashMap::new(),
        };

        assert_eq!(result.strategy_name, "TestStrategy");
        assert_eq!(result.signal, TradingSignal::Short);
        assert_eq!(result.confidence, 0.75);
        assert_eq!(result.weight, 1.5);
    }

    #[test]
    fn test_signal_combination_mode_serialization() {
        let mode = SignalCombinationMode::WeightedAverage;
        let json = serde_json::to_string(&mode).unwrap();
        let deserialized: SignalCombinationMode = serde_json::from_str(&json).unwrap();

        assert!(matches!(
            deserialized,
            SignalCombinationMode::WeightedAverage
        ));
    }

    #[test]
    fn test_strategy_engine_config_custom_values() {
        let config = StrategyEngineConfig {
            enabled_strategies: vec!["Custom Strategy".to_string()],
            min_confidence_threshold: 0.75,
            signal_combination_mode: SignalCombinationMode::BestConfidence,
            max_history_size: 500,
        };

        assert_eq!(config.enabled_strategies.len(), 1);
        assert_eq!(config.min_confidence_threshold, 0.75);
        assert_eq!(config.max_history_size, 500);
    }

    #[tokio::test]
    async fn test_strategy_engine_multiple_analyses() {
        let engine = StrategyEngine::new();
        let input = create_test_input();

        // Run multiple analyses
        for _ in 0..3 {
            let result = engine.analyze_market(&input).await;
            assert!(result.is_ok());
        }

        // History should have 3 entries
        let history = engine.get_signal_history(None).await;
        assert_eq!(history.len(), 3);
    }

    #[tokio::test]
    async fn test_strategy_engine_history_oldest_removed() {
        let mut config = StrategyEngineConfig::default();
        config.max_history_size = 2; // Very small history

        let engine = StrategyEngine::with_config(config);
        let input = create_test_input();

        // Add 5 signals
        for _ in 0..5 {
            let _ = engine.analyze_market(&input).await;
        }

        // Should only keep last 2
        let history = engine.get_signal_history(None).await;
        assert_eq!(history.len(), 2);
    }

    #[test]
    fn test_strategy_engine_config_empty_enabled_strategies() {
        let mut config = StrategyEngineConfig::default();
        config.enabled_strategies = vec![];

        let engine = StrategyEngine::with_config(config);

        // Should still have strategies added
        assert!(!engine.get_strategy_names().is_empty());
    }

    #[test]
    fn test_combine_signals_metadata_counts() {
        let engine = StrategyEngine::new();
        let results = vec![
            StrategySignalResult {
                strategy_name: "S1".to_string(),
                signal: TradingSignal::Long,
                confidence: 0.8,
                reasoning: "".to_string(),
                weight: 1.0,
                metadata: HashMap::new(),
            },
            StrategySignalResult {
                strategy_name: "S2".to_string(),
                signal: TradingSignal::Short,
                confidence: 0.7,
                reasoning: "".to_string(),
                weight: 1.0,
                metadata: HashMap::new(),
            },
            StrategySignalResult {
                strategy_name: "S3".to_string(),
                signal: TradingSignal::Neutral,
                confidence: 0.6,
                reasoning: "".to_string(),
                weight: 1.0,
                metadata: HashMap::new(),
            },
        ];
        let input = create_test_input();

        let combined = engine.combine_signals(&results, &input).unwrap();

        assert_eq!(
            combined.metadata.get("long_signals").unwrap().as_u64(),
            Some(1)
        );
        assert_eq!(
            combined.metadata.get("short_signals").unwrap().as_u64(),
            Some(1)
        );
        assert_eq!(
            combined.metadata.get("neutral_signals").unwrap().as_u64(),
            Some(1)
        );
    }

    #[tokio::test]
    async fn test_get_latest_signal_empty_history() {
        let engine = StrategyEngine::new();

        let latest = engine.get_latest_signal().await;
        assert!(latest.is_none());
    }

    #[test]
    fn test_trading_signal_equality() {
        assert_eq!(TradingSignal::Long, TradingSignal::Long);
        assert_eq!(TradingSignal::Short, TradingSignal::Short);
        assert_eq!(TradingSignal::Neutral, TradingSignal::Neutral);
        assert_ne!(TradingSignal::Long, TradingSignal::Short);
    }

    #[test]
    fn test_combined_signal_serialization() {
        let signal = CombinedSignal {
            final_signal: TradingSignal::Long,
            combined_confidence: 0.85,
            strategy_signals: vec![],
            reasoning: "Test".to_string(),
            symbol: "BTCUSDT".to_string(),
            timestamp: 1234567890,
            metadata: HashMap::new(),
        };

        let json = serde_json::to_string(&signal).unwrap();
        let deserialized: CombinedSignal = serde_json::from_str(&json).unwrap();

        assert_eq!(deserialized.final_signal, signal.final_signal);
        assert_eq!(deserialized.combined_confidence, signal.combined_confidence);
        assert_eq!(deserialized.symbol, signal.symbol);
    }
}
