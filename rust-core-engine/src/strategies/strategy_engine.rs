use super::*;
use crate::strategies::{
    rsi_strategy::RsiStrategy,
    macd_strategy::MacdStrategy,
    volume_strategy::VolumeStrategy,
    bollinger_strategy::BollingerStrategy,
};
use async_trait::async_trait;
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
    
    pub fn update_strategy_config(&mut self, name: &str, config: StrategyConfig) -> Result<(), StrategyError> {
        for strategy in &mut self.strategies {
            if strategy.name() == name {
                strategy.update_config(config);
                return Ok(());
            }
        }
        Err(StrategyError::InvalidConfiguration(format!("Strategy '{}' not found", name)))
    }
    
    pub async fn analyze_market(&self, data: &StrategyInput) -> Result<CombinedSignal, StrategyError> {
        let mut strategy_results = Vec::new();
        
        // Execute all enabled strategies
        for strategy in &self.strategies {
            let strategy_name = strategy.name().to_string();
            
            // Check if strategy is enabled
            if !self.config.enabled_strategies.is_empty() && 
               !self.config.enabled_strategies.contains(&strategy_name) {
                continue;
            }
            
            // Validate data for this strategy
            if let Err(e) = strategy.validate_data(data) {
                warn!("Strategy '{}' validation failed: {}", strategy_name, e);
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
                    
                    info!("Strategy '{}' signal: {:?} (confidence: {:.2})", 
                          strategy_name, output.signal, output.confidence);
                }
                Err(e) => {
                    warn!("Strategy '{}' analysis failed: {}", strategy_name, e);
                    continue;
                }
            }
        }
        
        if strategy_results.is_empty() {
            return Err(StrategyError::InsufficientData("No strategies produced valid signals".to_string()));
        }
        
        // Combine strategy signals
        let combined_signal = self.combine_signals(&strategy_results, data)?;
        
        // Store in history
        self.add_to_history(combined_signal.clone()).await;
        
        Ok(combined_signal)
    }
    
    fn combine_signals(&self, results: &[StrategySignalResult], data: &StrategyInput) -> Result<CombinedSignal, StrategyError> {
        let (final_signal, combined_confidence, reasoning) = match self.config.signal_combination_mode {
            SignalCombinationMode::WeightedAverage => self.combine_weighted_average(results),
            SignalCombinationMode::Consensus => self.combine_consensus(results),
            SignalCombinationMode::BestConfidence => self.combine_best_confidence(results),
            SignalCombinationMode::Conservative => self.combine_conservative(results),
        };
        
        // Create metadata with strategy summary
        let mut metadata = HashMap::new();
        metadata.insert("total_strategies".to_string(), json!(results.len()));
        metadata.insert("combination_mode".to_string(), json!(format!("{:?}", self.config.signal_combination_mode)));
        metadata.insert("min_confidence_threshold".to_string(), json!(self.config.min_confidence_threshold));
        
        // Add strategy breakdown
        let long_count = results.iter().filter(|r| r.signal == TradingSignal::Long).count();
        let short_count = results.iter().filter(|r| r.signal == TradingSignal::Short).count();
        let neutral_count = results.iter().filter(|r| r.signal == TradingSignal::Neutral).count();
        
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
    
    fn combine_weighted_average(&self, results: &[StrategySignalResult]) -> (TradingSignal, f64, String) {
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
        
        let avg_confidence = if total_weight > 0.0 { total_confidence / total_weight } else { 0.0 };
        
        let final_signal = if long_score > short_score && long_score > neutral_score {
            TradingSignal::Long
        } else if short_score > long_score && short_score > neutral_score {
            TradingSignal::Short
        } else {
            TradingSignal::Neutral
        };
        
        let reasoning = format!(
            "Weighted average: Long={:.2}, Short={:.2}, Neutral={:.2}",
            long_score, short_score, neutral_score
        );
        
        (final_signal, avg_confidence, reasoning)
    }
    
    fn combine_consensus(&self, results: &[StrategySignalResult]) -> (TradingSignal, f64, String) {
        let long_count = results.iter().filter(|r| r.signal == TradingSignal::Long).count();
        let short_count = results.iter().filter(|r| r.signal == TradingSignal::Short).count();
        let neutral_count = results.iter().filter(|r| r.signal == TradingSignal::Neutral).count();
        
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
        let agreeing_strategies: Vec<_> = results.iter()
            .filter(|r| r.signal == final_signal)
            .collect();
        
        let avg_confidence = if !agreeing_strategies.is_empty() {
            agreeing_strategies.iter().map(|r| r.confidence).sum::<f64>() / agreeing_strategies.len() as f64
        } else {
            0.5
        };
        
        let combined_confidence = avg_confidence * consensus_strength;
        
        let reasoning = format!(
            "Consensus: {}L/{}S/{}N (strength: {:.1}%)",
            long_count, short_count, neutral_count, consensus_strength * 100.0
        );
        
        (final_signal, combined_confidence, reasoning)
    }
    
    fn combine_best_confidence(&self, results: &[StrategySignalResult]) -> (TradingSignal, f64, String) {
        let best_result = results.iter()
            .max_by(|a, b| a.confidence.partial_cmp(&b.confidence).unwrap())
            .unwrap();
        
        let reasoning = format!(
            "Best confidence from {} ({:.1}%): {}",
            best_result.strategy_name, best_result.confidence * 100.0, best_result.reasoning
        );
        
        (best_result.signal.clone(), best_result.confidence, reasoning)
    }
    
    fn combine_conservative(&self, results: &[StrategySignalResult]) -> (TradingSignal, f64, String) {
        // Only signal if high confidence and multiple strategies agree
        let high_confidence_results: Vec<_> = results.iter()
            .filter(|r| r.confidence >= self.config.min_confidence_threshold)
            .collect();
        
        if high_confidence_results.len() < 2 {
            return (
                TradingSignal::Neutral,
                0.5,
                "Conservative: Insufficient high-confidence signals".to_string()
            );
        }
        
        // Check for agreement among high-confidence strategies
        let long_count = high_confidence_results.iter().filter(|r| r.signal == TradingSignal::Long).count();
        let short_count = high_confidence_results.iter().filter(|r| r.signal == TradingSignal::Short).count();
        
        if long_count >= 2 && short_count == 0 {
            let avg_confidence = high_confidence_results.iter()
                .filter(|r| r.signal == TradingSignal::Long)
                .map(|r| r.confidence)
                .sum::<f64>() / long_count as f64;
            
            return (
                TradingSignal::Long,
                avg_confidence * 0.9, // Conservative discount
                format!("Conservative: {} high-confidence LONG signals", long_count)
            );
        }
        
        if short_count >= 2 && long_count == 0 {
            let avg_confidence = high_confidence_results.iter()
                .filter(|r| r.signal == TradingSignal::Short)
                .map(|r| r.confidence)
                .sum::<f64>() / short_count as f64;
            
            return (
                TradingSignal::Short,
                avg_confidence * 0.9, // Conservative discount
                format!("Conservative: {} high-confidence SHORT signals", short_count)
            );
        }
        
        (
            TradingSignal::Neutral,
            0.6,
            "Conservative: Mixed signals among high-confidence strategies".to_string()
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