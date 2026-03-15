#![allow(dead_code)]

use super::*;
use anyhow::{anyhow, Result};

// Python AI service types kept for serialization compatibility (external consumers may still
// serialize/deserialize these even though the HTTP calls are disabled)
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
struct PythonCandleData {
    timestamp: i64,
    open: f64,
    high: f64,
    low: f64,
    close: f64,
    volume: f64,
}

impl From<&crate::market_data::cache::CandleData> for PythonCandleData {
    fn from(candle: &crate::market_data::cache::CandleData) -> Self {
        Self {
            timestamp: candle.open_time,
            open: candle.open,
            high: candle.high,
            low: candle.low,
            close: candle.close,
            volume: candle.volume,
        }
    }
}

/// No-op client stub — Python AI service removed; all methods return errors immediately.
/// Kept for interface compatibility. HTTP calls are disabled.
#[derive(Debug, Clone)]
pub struct AIClient {
    base_url: String,
}

impl AIClient {
    pub fn new(base_url: &str, _timeout_seconds: u64) -> Self {
        Self {
            base_url: base_url.trim_end_matches('/').to_string(),
        }
    }

    /// Disabled — Python AI service removed.
    pub async fn analyze_trading_signals(
        &self,
        _request: &AIAnalysisRequest,
    ) -> Result<AISignalResponse> {
        Err(anyhow!("Python AI service disabled"))
    }

    /// Disabled — Python AI service removed.
    pub async fn get_strategy_recommendations(
        &self,
        _request: &StrategyRecommendationRequest,
    ) -> Result<Vec<StrategyRecommendation>> {
        Ok(vec![])
    }

    /// Disabled — Python AI service removed.
    pub async fn analyze_market_condition(
        &self,
        _request: &MarketConditionRequest,
    ) -> Result<MarketConditionAnalysis> {
        Err(anyhow!("Python AI service disabled"))
    }

    /// Disabled — Python AI service removed.
    pub async fn send_performance_feedback(&self, _feedback: &PerformanceFeedback) -> Result<()> {
        Ok(())
    }

    /// Disabled — Python AI service removed.
    pub async fn get_service_info(&self) -> Result<AIServiceInfo> {
        Err(anyhow!("Python AI service disabled"))
    }

    /// Disabled — Python AI service removed.
    pub async fn get_supported_strategies(&self) -> Result<SupportedStrategiesResponse> {
        Ok(SupportedStrategiesResponse {
            strategies: vec![
                "RSI Strategy".to_string(),
                "MACD Strategy".to_string(),
                "Bollinger Bands Strategy".to_string(),
                "Volume Strategy".to_string(),
                "Stochastic Strategy".to_string(),
            ],
        })
    }

    /// Disabled — Python AI service removed.
    pub async fn request_trade_analysis(&self, _request: &TradeAnalysisRequest) -> Result<()> {
        Ok(())
    }

    /// Returns the configured base URL (kept for tests)
    pub fn base_url(&self) -> &str {
        &self.base_url
    }
}

/// Request data for trade analysis (kept for interface compatibility)
#[derive(Debug, Clone, serde::Serialize)]
pub struct TradeAnalysisRequest {
    pub trade_id: String,
    pub symbol: String,
    pub side: String,
    pub entry_price: f64,
    pub exit_price: f64,
    pub quantity: f64,
    pub leverage: u8,
    pub pnl_usdt: f64,
    pub pnl_percentage: f64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub duration_seconds: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub close_reason: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub open_time: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub close_time: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub strategy_name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ai_confidence: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ai_reasoning: Option<String>,
}

/// AI service information (stub response)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AIServiceInfo {
    pub service_name: String,
    pub version: String,
    pub model_version: String,
    pub supported_timeframes: Vec<String>,
    pub supported_symbols: Vec<String>,
    pub capabilities: Vec<String>,
    pub last_trained: Option<String>,
}

/// AI model performance metrics (kept for type compatibility)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AIModelPerformance {
    pub overall_accuracy: f64,
    pub precision: f64,
    pub recall: f64,
    pub f1_score: f64,
    pub predictions_made: u64,
    pub successful_predictions: u64,
    pub average_confidence: f64,
    pub model_uptime: String,
    pub last_updated: String,
}

/// Supported strategies response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SupportedStrategiesResponse {
    pub strategies: Vec<String>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::market_data::cache::CandleData;
    use std::collections::HashMap;

    fn create_test_candle() -> CandleData {
        CandleData {
            open_time: 1234567890,
            open: 50000.0,
            high: 51000.0,
            low: 49000.0,
            close: 50500.0,
            volume: 100.0,
            close_time: 1234567950,
            quote_volume: 5000000.0,
            trades: 100,
            is_closed: true,
        }
    }

    #[test]
    fn test_ai_client_new() {
        let client = AIClient::new("http://localhost:8000", 30);
        assert_eq!(client.base_url(), "http://localhost:8000");
    }

    #[test]
    fn test_ai_client_new_trims_trailing_slash() {
        let client = AIClient::new("http://localhost:8000/", 30);
        assert_eq!(client.base_url(), "http://localhost:8000");
    }

    #[test]
    fn test_ai_client_clone() {
        let client = AIClient::new("http://localhost:8000", 30);
        let cloned = client.clone();
        assert_eq!(client.base_url(), cloned.base_url());
    }

    #[test]
    fn test_python_candle_data_from_candle() {
        let candle = create_test_candle();
        let python_candle = PythonCandleData::from(&candle);
        assert_eq!(python_candle.timestamp, candle.open_time);
        assert_eq!(python_candle.open, candle.open);
        assert_eq!(python_candle.close, candle.close);
    }

    #[tokio::test]
    async fn test_analyze_trading_signals_returns_err() {
        let client = AIClient::new("http://127.0.0.1:19999", 1);
        let request = AIAnalysisRequest {
            symbol: "BTCUSDT".to_string(),
            timeframe_data: HashMap::new(),
            current_price: 50000.0,
            volume_24h: 1000000.0,
            timestamp: 1700000000000,
            strategy_context: AIStrategyContext::default(),
        };
        let result = client.analyze_trading_signals(&request).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_get_strategy_recommendations_returns_empty() {
        let client = AIClient::new("http://127.0.0.1:19999", 1);
        let request = StrategyRecommendationRequest {
            symbol: "BTCUSDT".to_string(),
            timeframe_data: HashMap::new(),
            current_price: 50000.0,
            available_strategies: vec!["RSI".to_string()],
            timestamp: 1700000000000,
        };
        let result = client.get_strategy_recommendations(&request).await;
        assert!(result.is_ok());
        assert!(result.unwrap().is_empty());
    }

    #[tokio::test]
    async fn test_analyze_market_condition_returns_err() {
        let client = AIClient::new("http://127.0.0.1:19999", 1);
        let request = MarketConditionRequest {
            symbol: "BTCUSDT".to_string(),
            timeframe_data: HashMap::new(),
            current_price: 50000.0,
            volume_24h: 1000000.0,
            timestamp: 1700000000000,
        };
        let result = client.analyze_market_condition(&request).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_send_performance_feedback_returns_ok() {
        let client = AIClient::new("http://127.0.0.1:19999", 1);
        use crate::strategies::TradingSignal;
        let feedback = PerformanceFeedback {
            signal_id: "sig-001".to_string(),
            symbol: "BTCUSDT".to_string(),
            predicted_signal: TradingSignal::Long,
            actual_outcome: "success".to_string(),
            profit_loss: 100.0,
            confidence_was_accurate: true,
            feedback_notes: None,
            timestamp: 1700000000000,
        };
        let result = client.send_performance_feedback(&feedback).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_get_service_info_returns_err() {
        let client = AIClient::new("http://127.0.0.1:19999", 1);
        let result = client.get_service_info().await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_get_supported_strategies_returns_ok() {
        let client = AIClient::new("http://127.0.0.1:19999", 1);
        let result = client.get_supported_strategies().await;
        assert!(result.is_ok());
        let strategies = result.unwrap();
        assert!(!strategies.strategies.is_empty());
    }

    #[tokio::test]
    async fn test_request_trade_analysis_returns_ok() {
        let client = AIClient::new("http://127.0.0.1:19999", 1);
        let request = TradeAnalysisRequest {
            trade_id: "trade-001".to_string(),
            symbol: "BTCUSDT".to_string(),
            side: "Long".to_string(),
            entry_price: 50000.0,
            exit_price: 51000.0,
            quantity: 0.01,
            leverage: 1,
            pnl_usdt: 100.0,
            pnl_percentage: 2.0,
            duration_seconds: Some(3600),
            close_reason: Some("Take Profit".to_string()),
            open_time: None,
            close_time: None,
            strategy_name: Some("RSI Strategy".to_string()),
            ai_confidence: None,
            ai_reasoning: None,
        };
        let result = client.request_trade_analysis(&request).await;
        assert!(result.is_ok());
    }

    #[test]
    fn test_ai_service_info_serialization() {
        let info = AIServiceInfo {
            service_name: "stub".to_string(),
            version: "0.0.0".to_string(),
            model_version: "none".to_string(),
            supported_timeframes: vec!["1m".to_string()],
            supported_symbols: vec!["BTCUSDT".to_string()],
            capabilities: vec![],
            last_trained: None,
        };
        let json = serde_json::to_string(&info).unwrap();
        let deserialized: AIServiceInfo = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.service_name, "stub");
    }

    #[test]
    fn test_ai_service_info_with_empty_capabilities() {
        let info = AIServiceInfo {
            service_name: "test".to_string(),
            version: "1.0".to_string(),
            model_version: "v1".to_string(),
            supported_timeframes: vec![],
            supported_symbols: vec![],
            capabilities: vec![],
            last_trained: None,
        };
        let json = serde_json::to_string(&info).unwrap();
        let deserialized: AIServiceInfo = serde_json::from_str(&json).unwrap();
        assert!(deserialized.capabilities.is_empty());
    }

    #[test]
    fn test_supported_strategies_response_serialization() {
        let resp = SupportedStrategiesResponse {
            strategies: vec!["RSI".to_string(), "MACD".to_string()],
        };
        let json = serde_json::to_string(&resp).unwrap();
        let deserialized: SupportedStrategiesResponse = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.strategies.len(), 2);
    }

    #[test]
    fn test_trade_analysis_request_serialization() {
        let request = TradeAnalysisRequest {
            trade_id: "t1".to_string(),
            symbol: "ETHUSDT".to_string(),
            side: "Short".to_string(),
            entry_price: 3000.0,
            exit_price: 2900.0,
            quantity: 0.1,
            leverage: 2,
            pnl_usdt: 200.0,
            pnl_percentage: 3.33,
            duration_seconds: None,
            close_reason: None,
            open_time: None,
            close_time: None,
            strategy_name: None,
            ai_confidence: None,
            ai_reasoning: None,
        };
        let json = serde_json::to_string(&request).unwrap();
        assert!(json.contains("ETHUSDT"));
        assert!(json.contains("3000.0"));
    }

    #[test]
    fn test_ai_model_performance_serialization() {
        let perf = AIModelPerformance {
            overall_accuracy: 0.72,
            precision: 0.75,
            recall: 0.70,
            f1_score: 0.72,
            predictions_made: 1000,
            successful_predictions: 720,
            average_confidence: 0.68,
            model_uptime: "72h".to_string(),
            last_updated: "2026-01-01".to_string(),
        };
        let json = serde_json::to_string(&perf).unwrap();
        let deserialized: AIModelPerformance = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.overall_accuracy, 0.72);
        assert_eq!(deserialized.predictions_made, 1000);
    }
}
