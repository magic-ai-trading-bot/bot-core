pub mod bollinger_strategy;
pub mod indicators;
pub mod macd_strategy;
pub mod rsi_strategy;
pub mod strategy_engine;
pub mod types;
pub mod volume_strategy;

use crate::market_data::cache::CandleData;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

// Re-export key types

/// Core trait for all trading strategies
#[async_trait]
pub trait Strategy: Send + Sync {
    /// Unique identifier for the strategy
    fn name(&self) -> &'static str;

    /// Strategy description
    fn description(&self) -> &'static str;

    /// Required timeframes for this strategy
    fn required_timeframes(&self) -> Vec<&'static str>;

    /// Analyze market data and generate trading signal
    async fn analyze(&self, data: &StrategyInput) -> Result<StrategyOutput, StrategyError>;

    /// Get strategy configuration
    fn config(&self) -> &StrategyConfig;

    /// Update strategy configuration
    fn update_config(&mut self, config: StrategyConfig);

    /// Validate if strategy can analyze the given data
    fn validate_data(&self, data: &StrategyInput) -> Result<(), StrategyError>;
}

/// Input data for strategy analysis
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StrategyInput {
    pub symbol: String,
    pub timeframe_data: HashMap<String, Vec<CandleData>>,
    pub current_price: f64,
    pub volume_24h: f64,
    pub timestamp: i64,
}

/// Output from strategy analysis
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StrategyOutput {
    pub signal: TradingSignal,
    pub confidence: f64,
    pub reasoning: String,
    pub timeframe: String,
    pub timestamp: i64,
    pub metadata: HashMap<String, serde_json::Value>,
}

/// Trading signal types
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
pub enum TradingSignal {
    Long,
    Short,
    Neutral,
}

/// Strategy configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StrategyConfig {
    pub enabled: bool,
    pub weight: f64,
    pub parameters: HashMap<String, serde_json::Value>,
}

/// Strategy errors
#[derive(Debug, thiserror::Error)]
pub enum StrategyError {
    #[error("Insufficient data: {0}")]
    InsufficientData(String),

    #[error("Invalid configuration: {0}")]
    InvalidConfiguration(String),

    #[error("Calculation error: {0}")]
    CalculationError(String),

    #[error("Data validation error: {0}")]
    DataValidation(String),
}

impl Default for StrategyConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            weight: 1.0,
            parameters: HashMap::new(),
        }
    }
}

impl TradingSignal {
    pub fn as_str(&self) -> &'static str {
        match self {
            TradingSignal::Long => "LONG",
            TradingSignal::Short => "SHORT",
            TradingSignal::Neutral => "NEUTRAL",
        }
    }

    pub fn from_string(s: &str) -> Option<Self> {
        match s.to_uppercase().as_str() {
            "LONG" => Some(TradingSignal::Long),
            "SHORT" => Some(TradingSignal::Short),
            "NEUTRAL" => Some(TradingSignal::Neutral),
            _ => None,
        }
    }
}
