pub mod bollinger_strategy;
pub mod indicators;
pub mod macd_strategy;
pub mod rsi_strategy;
pub mod stochastic_strategy;
pub mod strategy_engine;
pub mod types;
pub mod volume_strategy;

#[cfg(test)]
mod tests;

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

#[cfg(test)]
mod mod_tests {
    use super::*;

    // ========================================
    // TradingSignal Tests
    // ========================================

    #[test]
    fn test_trading_signal_as_str_long() {
        assert_eq!(TradingSignal::Long.as_str(), "LONG");
    }

    #[test]
    fn test_trading_signal_as_str_short() {
        assert_eq!(TradingSignal::Short.as_str(), "SHORT");
    }

    #[test]
    fn test_trading_signal_as_str_neutral() {
        assert_eq!(TradingSignal::Neutral.as_str(), "NEUTRAL");
    }

    #[test]
    fn test_trading_signal_from_string_long() {
        assert_eq!(
            TradingSignal::from_string("LONG"),
            Some(TradingSignal::Long)
        );
    }

    #[test]
    fn test_trading_signal_from_string_short() {
        assert_eq!(
            TradingSignal::from_string("SHORT"),
            Some(TradingSignal::Short)
        );
    }

    #[test]
    fn test_trading_signal_from_string_neutral() {
        assert_eq!(
            TradingSignal::from_string("NEUTRAL"),
            Some(TradingSignal::Neutral)
        );
    }

    #[test]
    fn test_trading_signal_from_string_lowercase() {
        assert_eq!(
            TradingSignal::from_string("long"),
            Some(TradingSignal::Long)
        );
        assert_eq!(
            TradingSignal::from_string("short"),
            Some(TradingSignal::Short)
        );
        assert_eq!(
            TradingSignal::from_string("neutral"),
            Some(TradingSignal::Neutral)
        );
    }

    #[test]
    fn test_trading_signal_from_string_mixed_case() {
        assert_eq!(
            TradingSignal::from_string("LoNg"),
            Some(TradingSignal::Long)
        );
        assert_eq!(
            TradingSignal::from_string("ShOrT"),
            Some(TradingSignal::Short)
        );
        assert_eq!(
            TradingSignal::from_string("NeUtRaL"),
            Some(TradingSignal::Neutral)
        );
    }

    #[test]
    fn test_trading_signal_from_string_invalid() {
        assert_eq!(TradingSignal::from_string("INVALID"), None);
        assert_eq!(TradingSignal::from_string("BUY"), None);
        assert_eq!(TradingSignal::from_string("SELL"), None);
        assert_eq!(TradingSignal::from_string(""), None);
        assert_eq!(TradingSignal::from_string("   "), None);
    }

    #[test]
    fn test_trading_signal_partial_eq() {
        assert_eq!(TradingSignal::Long, TradingSignal::Long);
        assert_eq!(TradingSignal::Short, TradingSignal::Short);
        assert_eq!(TradingSignal::Neutral, TradingSignal::Neutral);
        assert_ne!(TradingSignal::Long, TradingSignal::Short);
        assert_ne!(TradingSignal::Long, TradingSignal::Neutral);
        assert_ne!(TradingSignal::Short, TradingSignal::Neutral);
    }

    #[test]
    fn test_trading_signal_clone() {
        let signal = TradingSignal::Long;
        let cloned = signal;
        assert_eq!(signal, cloned);
    }

    #[test]
    fn test_trading_signal_copy() {
        let signal = TradingSignal::Long;
        let copied = signal;
        assert_eq!(signal, copied);
    }

    // ========================================
    // StrategyConfig Tests
    // ========================================

    #[test]
    fn test_strategy_config_default_enabled() {
        let config = StrategyConfig::default();
        assert!(config.enabled);
    }

    #[test]
    fn test_strategy_config_default_weight() {
        let config = StrategyConfig::default();
        assert_eq!(config.weight, 1.0);
    }

    #[test]
    fn test_strategy_config_default_parameters_empty() {
        let config = StrategyConfig::default();
        assert!(config.parameters.is_empty());
    }

    #[test]
    fn test_strategy_config_clone() {
        let mut config = StrategyConfig::default();
        config.enabled = false;
        config.weight = 2.5;
        config
            .parameters
            .insert("test".to_string(), serde_json::json!(42));

        let cloned = config.clone();
        assert_eq!(config.enabled, cloned.enabled);
        assert_eq!(config.weight, cloned.weight);
        assert_eq!(config.parameters.len(), cloned.parameters.len());
    }

    #[test]
    fn test_strategy_config_custom_values() {
        let mut params = HashMap::new();
        params.insert("threshold".to_string(), serde_json::json!(0.75));
        params.insert("period".to_string(), serde_json::json!(14));

        let config = StrategyConfig {
            enabled: false,
            weight: 3.0,
            parameters: params,
        };

        assert!(!config.enabled);
        assert_eq!(config.weight, 3.0);
        assert_eq!(config.parameters.len(), 2);
    }

    // ========================================
    // StrategyInput Tests
    // ========================================

    #[test]
    fn test_strategy_input_creation() {
        let input = StrategyInput {
            symbol: "BTCUSDT".to_string(),
            timeframe_data: HashMap::new(),
            current_price: 50000.0,
            volume_24h: 1000000.0,
            timestamp: 1234567890,
        };

        assert_eq!(input.symbol, "BTCUSDT");
        assert_eq!(input.current_price, 50000.0);
        assert_eq!(input.volume_24h, 1000000.0);
        assert_eq!(input.timestamp, 1234567890);
    }

    #[test]
    fn test_strategy_input_clone() {
        let input = StrategyInput {
            symbol: "ETHUSDT".to_string(),
            timeframe_data: HashMap::new(),
            current_price: 3000.0,
            volume_24h: 500000.0,
            timestamp: 9876543210,
        };

        let cloned = input.clone();
        assert_eq!(input.symbol, cloned.symbol);
        assert_eq!(input.current_price, cloned.current_price);
    }

    // ========================================
    // StrategyOutput Tests
    // ========================================

    #[test]
    fn test_strategy_output_creation() {
        let output = StrategyOutput {
            signal: TradingSignal::Long,
            confidence: 0.85,
            reasoning: "Strong uptrend".to_string(),
            timeframe: "1h".to_string(),
            timestamp: 1234567890,
            metadata: HashMap::new(),
        };

        assert_eq!(output.signal, TradingSignal::Long);
        assert_eq!(output.confidence, 0.85);
        assert_eq!(output.reasoning, "Strong uptrend");
    }

    #[test]
    fn test_strategy_output_clone() {
        let mut metadata = HashMap::new();
        metadata.insert("rsi".to_string(), serde_json::json!(65.0));

        let output = StrategyOutput {
            signal: TradingSignal::Short,
            confidence: 0.72,
            reasoning: "Overbought condition".to_string(),
            timeframe: "4h".to_string(),
            timestamp: 9876543210,
            metadata,
        };

        let cloned = output.clone();
        assert_eq!(output.signal, cloned.signal);
        assert_eq!(output.confidence, cloned.confidence);
        assert_eq!(output.metadata.len(), cloned.metadata.len());
    }

    // ========================================
    // StrategyError Tests
    // ========================================

    #[test]
    fn test_strategy_error_insufficient_data() {
        let error = StrategyError::InsufficientData("Need 50 candles".to_string());
        assert!(error.to_string().contains("Insufficient data"));
        assert!(error.to_string().contains("Need 50 candles"));
    }

    #[test]
    fn test_strategy_error_invalid_configuration() {
        let error = StrategyError::InvalidConfiguration("Invalid period".to_string());
        assert!(error.to_string().contains("Invalid configuration"));
        assert!(error.to_string().contains("Invalid period"));
    }

    #[test]
    fn test_strategy_error_calculation_error() {
        let error = StrategyError::CalculationError("Division by zero".to_string());
        assert!(error.to_string().contains("Calculation error"));
        assert!(error.to_string().contains("Division by zero"));
    }

    #[test]
    fn test_strategy_error_data_validation() {
        let error = StrategyError::DataValidation("Price cannot be negative".to_string());
        assert!(error.to_string().contains("Data validation error"));
        assert!(error.to_string().contains("Price cannot be negative"));
    }

    // ========================================
    // Serialization/Deserialization Tests
    // ========================================

    #[test]
    fn test_trading_signal_serialize_deserialize() {
        let signal = TradingSignal::Long;
        let serialized = serde_json::to_string(&signal).unwrap();
        let deserialized: TradingSignal = serde_json::from_str(&serialized).unwrap();
        assert_eq!(signal, deserialized);
    }

    #[test]
    fn test_strategy_config_serialize_deserialize() {
        let config = StrategyConfig::default();
        let serialized = serde_json::to_string(&config).unwrap();
        let deserialized: StrategyConfig = serde_json::from_str(&serialized).unwrap();
        assert_eq!(config.enabled, deserialized.enabled);
        assert_eq!(config.weight, deserialized.weight);
    }

    #[test]
    fn test_strategy_input_serialize_deserialize() {
        let input = StrategyInput {
            symbol: "BTCUSDT".to_string(),
            timeframe_data: HashMap::new(),
            current_price: 50000.0,
            volume_24h: 1000000.0,
            timestamp: 1234567890,
        };

        let serialized = serde_json::to_string(&input).unwrap();
        let deserialized: StrategyInput = serde_json::from_str(&serialized).unwrap();
        assert_eq!(input.symbol, deserialized.symbol);
        assert_eq!(input.current_price, deserialized.current_price);
    }

    #[test]
    fn test_strategy_output_serialize_deserialize() {
        let output = StrategyOutput {
            signal: TradingSignal::Neutral,
            confidence: 0.5,
            reasoning: "No clear trend".to_string(),
            timeframe: "1d".to_string(),
            timestamp: 1234567890,
            metadata: HashMap::new(),
        };

        let serialized = serde_json::to_string(&output).unwrap();
        let deserialized: StrategyOutput = serde_json::from_str(&serialized).unwrap();
        assert_eq!(output.signal, deserialized.signal);
        assert_eq!(output.confidence, deserialized.confidence);
    }
}
