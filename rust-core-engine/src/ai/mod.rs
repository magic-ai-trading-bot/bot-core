pub mod client;
pub mod types;

use crate::strategies::{StrategyInput, TradingSignal};
use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

// Re-export key types
pub use client::*;

/// No-op stub for the AI service.
/// All HTTP calls removed — strategy engine (RSI/MACD/Bollinger/etc.) is the sole signal source.
#[derive(Debug, Clone)]
pub struct AIService {
    #[allow(dead_code)]
    config: AIServiceConfig,
}

/// Configuration for AI service (retained for interface compatibility)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AIServiceConfig {
    pub ai_service_url: String,
    pub request_timeout_seconds: u64,
    pub max_retries: u32,
    pub enable_caching: bool,
    pub cache_ttl_seconds: u64,
}

impl AIService {
    pub fn new(config: AIServiceConfig) -> Self {
        Self { config }
    }

    /// Disabled — returns error; strategy engine handles all signals.
    pub async fn analyze_for_trading_signal(
        &self,
        _data: &StrategyInput,
        _strategy_context: AIStrategyContext,
    ) -> Result<AISignalResponse> {
        Err(anyhow::anyhow!("AI service disabled"))
    }

    /// Disabled — returns empty vec; strategy engine provides strategy selection.
    pub async fn get_strategy_recommendations(
        &self,
        _market_data: &StrategyInput,
        _available_strategies: Vec<String>,
    ) -> Result<Vec<StrategyRecommendation>> {
        Ok(vec![])
    }

    /// Disabled — returns error; market analysis handled by strategy engine.
    pub async fn analyze_market_condition(
        &self,
        _data: &StrategyInput,
    ) -> Result<MarketConditionAnalysis> {
        Err(anyhow::anyhow!("AI service disabled"))
    }

    /// No-op — feedback loop removed with Python service.
    pub async fn send_performance_feedback(&self, _feedback: PerformanceFeedback) -> Result<()> {
        Ok(())
    }

    /// Disabled — returns error.
    pub async fn get_service_info(&self) -> Result<crate::ai::client::AIServiceInfo> {
        Err(anyhow::anyhow!("AI service disabled"))
    }

    /// Returns static list of Rust-native strategies.
    pub async fn get_supported_strategies(
        &self,
    ) -> Result<crate::ai::client::SupportedStrategiesResponse> {
        Ok(crate::ai::client::SupportedStrategiesResponse {
            strategies: vec![
                "RSI Strategy".to_string(),
                "MACD Strategy".to_string(),
                "Bollinger Bands Strategy".to_string(),
                "Volume Strategy".to_string(),
                "Stochastic Strategy".to_string(),
            ],
        })
    }

    /// No-op — trade analysis removed with Python service.
    pub async fn request_trade_analysis(
        &self,
        _request: &crate::ai::client::TradeAnalysisRequest,
    ) -> Result<()> {
        Ok(())
    }
}

impl Default for AIServiceConfig {
    fn default() -> Self {
        Self {
            ai_service_url: "http://localhost:8000".to_string(),
            request_timeout_seconds: 30,
            max_retries: 3,
            enable_caching: true,
            cache_ttl_seconds: 300, // 5 minutes
        }
    }
}

/// Strategy context for AI analysis
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AIStrategyContext {
    pub selected_strategies: Vec<String>,
    pub market_condition: String,
    pub risk_level: String,
    pub user_preferences: HashMap<String, serde_json::Value>,
    pub technical_indicators: HashMap<String, serde_json::Value>,
}

impl Default for AIStrategyContext {
    fn default() -> Self {
        Self {
            selected_strategies: vec!["RSI Strategy".to_string(), "MACD Strategy".to_string()],
            market_condition: "Unknown".to_string(),
            risk_level: "Moderate".to_string(),
            user_preferences: HashMap::new(),
            technical_indicators: HashMap::new(),
        }
    }
}

/// AI analysis request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AIAnalysisRequest {
    pub symbol: String,
    pub timeframe_data: HashMap<String, Vec<crate::market_data::cache::CandleData>>,
    pub current_price: f64,
    pub volume_24h: f64,
    pub timestamp: i64,
    pub strategy_context: AIStrategyContext,
}

/// AI signal response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AISignalResponse {
    pub signal: TradingSignal,
    pub confidence: f64,
    pub reasoning: String,
    #[serde(default)]
    pub strategy_scores: HashMap<String, f64>,
    pub market_analysis: AIMarketAnalysis,
    pub risk_assessment: AIRiskAssessment,
    #[serde(default)]
    pub timestamp: i64,
}

/// AI market analysis
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AIMarketAnalysis {
    pub trend_direction: String,
    pub trend_strength: f64,
    pub support_levels: Vec<f64>,
    pub resistance_levels: Vec<f64>,
    pub volatility_level: String,
    pub volume_analysis: String,
}

/// AI risk assessment
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AIRiskAssessment {
    pub overall_risk: String,
    pub technical_risk: f64,
    pub market_risk: f64,
    pub recommended_position_size: f64,
    pub stop_loss_suggestion: Option<f64>,
    pub take_profit_suggestion: Option<f64>,
}

/// Strategy recommendation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StrategyRecommendation {
    pub strategy_name: String,
    pub suitability_score: f64,
    pub reasoning: String,
    pub recommended_config: HashMap<String, serde_json::Value>,
}

/// Strategy recommendation request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StrategyRecommendationRequest {
    pub symbol: String,
    pub timeframe_data: HashMap<String, Vec<crate::market_data::cache::CandleData>>,
    pub current_price: f64,
    pub available_strategies: Vec<String>,
    pub timestamp: i64,
}

/// Market condition analysis
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MarketConditionAnalysis {
    pub condition_type: String,
    pub confidence: f64,
    #[serde(default)]
    pub direction: f64,
    #[serde(default)]
    pub trend_strength: f64,
    pub characteristics: Vec<String>,
    pub recommended_strategies: Vec<String>,
    pub market_phase: String,
    #[serde(default)]
    pub timeframe_analysis: HashMap<String, serde_json::Value>,
    #[serde(default)]
    pub indicators_summary: HashMap<String, serde_json::Value>,
}

/// Market condition request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MarketConditionRequest {
    pub symbol: String,
    #[serde(default)]
    pub timeframe_data: HashMap<String, Vec<crate::market_data::cache::CandleData>>,
    #[serde(default)]
    pub current_price: f64,
    #[serde(default)]
    pub volume_24h: f64,
    #[serde(default)]
    pub timestamp: i64,
}

/// Performance feedback for AI learning
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceFeedback {
    pub signal_id: String,
    pub symbol: String,
    pub predicted_signal: TradingSignal,
    pub actual_outcome: String, // "success", "failure", "neutral"
    pub profit_loss: f64,
    pub confidence_was_accurate: bool,
    pub feedback_notes: Option<String>,
    pub timestamp: i64,
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    #[test]
    fn test_ai_service_config_default() {
        let config = AIServiceConfig::default();

        assert_eq!(config.ai_service_url, "http://localhost:8000");
        assert_eq!(config.request_timeout_seconds, 30);
        assert_eq!(config.max_retries, 3);
        assert!(config.enable_caching);
        assert_eq!(config.cache_ttl_seconds, 300);
    }

    #[test]
    fn test_ai_service_config_serialization() {
        let config = AIServiceConfig {
            ai_service_url: "http://ai-service:8000".to_string(),
            request_timeout_seconds: 60,
            max_retries: 5,
            enable_caching: false,
            cache_ttl_seconds: 600,
        };

        let json = serde_json::to_string(&config).unwrap();
        let deserialized: AIServiceConfig = serde_json::from_str(&json).unwrap();

        assert_eq!(deserialized.ai_service_url, "http://ai-service:8000");
        assert_eq!(deserialized.request_timeout_seconds, 60);
        assert_eq!(deserialized.max_retries, 5);
        assert!(!deserialized.enable_caching);
        assert_eq!(deserialized.cache_ttl_seconds, 600);
    }

    #[test]
    fn test_ai_service_new() {
        let config = AIServiceConfig::default();
        let service = AIService::new(config.clone());

        assert_eq!(service.config.ai_service_url, config.ai_service_url);
        assert_eq!(
            service.config.request_timeout_seconds,
            config.request_timeout_seconds
        );
        assert_eq!(service.config.max_retries, config.max_retries);
    }

    #[test]
    fn test_ai_service_new_with_custom_config() {
        let config = AIServiceConfig {
            ai_service_url: "http://custom-ai:9000".to_string(),
            request_timeout_seconds: 45,
            max_retries: 2,
            enable_caching: true,
            cache_ttl_seconds: 180,
        };

        let service = AIService::new(config.clone());

        assert_eq!(service.config.ai_service_url, "http://custom-ai:9000");
        assert_eq!(service.config.request_timeout_seconds, 45);
        assert_eq!(service.config.max_retries, 2);
    }

    #[test]
    fn test_ai_strategy_context_default() {
        let context = AIStrategyContext::default();

        assert_eq!(context.selected_strategies.len(), 2);
        assert_eq!(context.selected_strategies[0], "RSI Strategy");
        assert_eq!(context.selected_strategies[1], "MACD Strategy");
        assert_eq!(context.market_condition, "Unknown");
        assert_eq!(context.risk_level, "Moderate");
        assert_eq!(context.user_preferences.len(), 0);
        assert_eq!(context.technical_indicators.len(), 0);
    }

    #[test]
    fn test_ai_strategy_context_serialization() {
        let mut user_prefs = HashMap::new();
        user_prefs.insert("risk_tolerance".to_string(), serde_json::json!("low"));

        let mut indicators = HashMap::new();
        indicators.insert("rsi".to_string(), serde_json::json!(45.5));

        let context = AIStrategyContext {
            selected_strategies: vec!["Bollinger".to_string()],
            market_condition: "Bullish".to_string(),
            risk_level: "High".to_string(),
            user_preferences: user_prefs,
            technical_indicators: indicators,
        };

        let json = serde_json::to_string(&context).unwrap();
        let deserialized: AIStrategyContext = serde_json::from_str(&json).unwrap();

        assert_eq!(deserialized.selected_strategies.len(), 1);
        assert_eq!(deserialized.market_condition, "Bullish");
        assert_eq!(deserialized.risk_level, "High");
        assert_eq!(deserialized.user_preferences.len(), 1);
        assert_eq!(deserialized.technical_indicators.len(), 1);
    }

    #[test]
    fn test_ai_analysis_request_serialization() {
        let request = AIAnalysisRequest {
            symbol: "BTCUSDT".to_string(),
            timeframe_data: HashMap::new(),
            current_price: 50000.0,
            volume_24h: 10000.0,
            timestamp: 1234567890,
            strategy_context: AIStrategyContext::default(),
        };

        let json = serde_json::to_string(&request).unwrap();
        let deserialized: AIAnalysisRequest = serde_json::from_str(&json).unwrap();

        assert_eq!(deserialized.symbol, "BTCUSDT");
        assert_eq!(deserialized.current_price, 50000.0);
        assert_eq!(deserialized.volume_24h, 10000.0);
        assert_eq!(deserialized.timestamp, 1234567890);
    }

    #[test]
    fn test_ai_signal_response_serialization() {
        let response = AISignalResponse {
            signal: TradingSignal::Long,
            confidence: 0.85,
            reasoning: "Strong uptrend detected".to_string(),
            strategy_scores: HashMap::new(),
            market_analysis: AIMarketAnalysis {
                trend_direction: "up".to_string(),
                trend_strength: 0.8,
                support_levels: vec![49000.0, 48000.0],
                resistance_levels: vec![51000.0, 52000.0],
                volatility_level: "medium".to_string(),
                volume_analysis: "increasing".to_string(),
            },
            risk_assessment: AIRiskAssessment {
                overall_risk: "moderate".to_string(),
                technical_risk: 0.3,
                market_risk: 0.4,
                recommended_position_size: 0.1,
                stop_loss_suggestion: Some(48000.0),
                take_profit_suggestion: Some(52000.0),
            },
            timestamp: 1234567890,
        };

        let json = serde_json::to_string(&response).unwrap();
        let deserialized: AISignalResponse = serde_json::from_str(&json).unwrap();

        assert!(matches!(deserialized.signal, TradingSignal::Long));
        assert_eq!(deserialized.confidence, 0.85);
        assert_eq!(deserialized.reasoning, "Strong uptrend detected");
    }

    #[test]
    fn test_ai_market_analysis_serialization() {
        let analysis = AIMarketAnalysis {
            trend_direction: "down".to_string(),
            trend_strength: 0.6,
            support_levels: vec![30000.0, 29000.0, 28000.0],
            resistance_levels: vec![32000.0, 33000.0],
            volatility_level: "high".to_string(),
            volume_analysis: "decreasing".to_string(),
        };

        let json = serde_json::to_string(&analysis).unwrap();
        let deserialized: AIMarketAnalysis = serde_json::from_str(&json).unwrap();

        assert_eq!(deserialized.trend_direction, "down");
        assert_eq!(deserialized.trend_strength, 0.6);
        assert_eq!(deserialized.support_levels.len(), 3);
        assert_eq!(deserialized.resistance_levels.len(), 2);
    }

    #[test]
    fn test_ai_risk_assessment_serialization() {
        let risk = AIRiskAssessment {
            overall_risk: "high".to_string(),
            technical_risk: 0.7,
            market_risk: 0.8,
            recommended_position_size: 0.05,
            stop_loss_suggestion: Some(45000.0),
            take_profit_suggestion: None,
        };

        let json = serde_json::to_string(&risk).unwrap();
        let deserialized: AIRiskAssessment = serde_json::from_str(&json).unwrap();

        assert_eq!(deserialized.overall_risk, "high");
        assert_eq!(deserialized.technical_risk, 0.7);
        assert_eq!(deserialized.recommended_position_size, 0.05);
        assert_eq!(deserialized.stop_loss_suggestion, Some(45000.0));
        assert_eq!(deserialized.take_profit_suggestion, None);
    }

    #[test]
    fn test_strategy_recommendation_serialization() {
        let mut recommended_config = HashMap::new();
        recommended_config.insert("period".to_string(), serde_json::json!(14));

        let recommendation = StrategyRecommendation {
            strategy_name: "RSI Strategy".to_string(),
            suitability_score: 0.9,
            reasoning: "Market conditions favor RSI".to_string(),
            recommended_config,
        };

        let json = serde_json::to_string(&recommendation).unwrap();
        let deserialized: StrategyRecommendation = serde_json::from_str(&json).unwrap();

        assert_eq!(deserialized.strategy_name, "RSI Strategy");
        assert_eq!(deserialized.suitability_score, 0.9);
        assert_eq!(deserialized.recommended_config.len(), 1);
    }

    #[test]
    fn test_strategy_recommendation_request_serialization() {
        let request = StrategyRecommendationRequest {
            symbol: "ETHUSDT".to_string(),
            timeframe_data: HashMap::new(),
            current_price: 3000.0,
            available_strategies: vec!["RSI".to_string(), "MACD".to_string()],
            timestamp: 1234567890,
        };

        let json = serde_json::to_string(&request).unwrap();
        let deserialized: StrategyRecommendationRequest = serde_json::from_str(&json).unwrap();

        assert_eq!(deserialized.symbol, "ETHUSDT");
        assert_eq!(deserialized.current_price, 3000.0);
        assert_eq!(deserialized.available_strategies.len(), 2);
    }

    #[test]
    fn test_market_condition_analysis_serialization() {
        let analysis = MarketConditionAnalysis {
            condition_type: "trending".to_string(),
            confidence: 0.88,
            direction: 0.65,
            trend_strength: 0.72,
            characteristics: vec!["strong_momentum".to_string(), "high_volume".to_string()],
            recommended_strategies: vec!["trend_following".to_string()],
            market_phase: "accumulation".to_string(),
            timeframe_analysis: HashMap::new(),
            indicators_summary: HashMap::new(),
        };

        let json = serde_json::to_string(&analysis).unwrap();
        let deserialized: MarketConditionAnalysis = serde_json::from_str(&json).unwrap();

        assert_eq!(deserialized.condition_type, "trending");
        assert_eq!(deserialized.confidence, 0.88);
        assert_eq!(deserialized.characteristics.len(), 2);
        assert_eq!(deserialized.market_phase, "accumulation");
    }

    #[test]
    fn test_market_condition_request_serialization() {
        let request = MarketConditionRequest {
            symbol: "BNBUSDT".to_string(),
            timeframe_data: HashMap::new(),
            current_price: 400.0,
            volume_24h: 50000.0,
            timestamp: 1234567890,
        };

        let json = serde_json::to_string(&request).unwrap();
        let deserialized: MarketConditionRequest = serde_json::from_str(&json).unwrap();

        assert_eq!(deserialized.symbol, "BNBUSDT");
        assert_eq!(deserialized.current_price, 400.0);
        assert_eq!(deserialized.volume_24h, 50000.0);
    }

    #[test]
    fn test_performance_feedback_serialization() {
        let feedback = PerformanceFeedback {
            signal_id: "signal_123".to_string(),
            symbol: "BTCUSDT".to_string(),
            predicted_signal: TradingSignal::Long,
            actual_outcome: "success".to_string(),
            profit_loss: 500.0,
            confidence_was_accurate: true,
            feedback_notes: Some("Great prediction".to_string()),
            timestamp: 1234567890,
        };

        let json = serde_json::to_string(&feedback).unwrap();
        let deserialized: PerformanceFeedback = serde_json::from_str(&json).unwrap();

        assert_eq!(deserialized.signal_id, "signal_123");
        assert_eq!(deserialized.symbol, "BTCUSDT");
        assert!(matches!(deserialized.predicted_signal, TradingSignal::Long));
        assert_eq!(deserialized.actual_outcome, "success");
        assert_eq!(deserialized.profit_loss, 500.0);
        assert!(deserialized.confidence_was_accurate);
    }

    #[test]
    fn test_performance_feedback_with_no_notes() {
        let feedback = PerformanceFeedback {
            signal_id: "signal_456".to_string(),
            symbol: "ETHUSDT".to_string(),
            predicted_signal: TradingSignal::Short,
            actual_outcome: "failure".to_string(),
            profit_loss: -200.0,
            confidence_was_accurate: false,
            feedback_notes: None,
            timestamp: 1234567890,
        };

        let json = serde_json::to_string(&feedback).unwrap();
        let deserialized: PerformanceFeedback = serde_json::from_str(&json).unwrap();

        assert_eq!(deserialized.feedback_notes, None);
        assert_eq!(deserialized.profit_loss, -200.0);
    }

    #[test]
    fn test_ai_service_config_custom_values() {
        let config = AIServiceConfig {
            ai_service_url: "http://192.168.1.100:8888".to_string(),
            request_timeout_seconds: 120,
            max_retries: 10,
            enable_caching: true,
            cache_ttl_seconds: 1800,
        };

        assert_eq!(config.ai_service_url, "http://192.168.1.100:8888");
        assert_eq!(config.request_timeout_seconds, 120);
        assert_eq!(config.max_retries, 10);
        assert!(config.enable_caching);
        assert_eq!(config.cache_ttl_seconds, 1800);
    }

    #[test]
    fn test_ai_strategy_context_with_multiple_strategies() {
        let context = AIStrategyContext {
            selected_strategies: vec![
                "RSI".to_string(),
                "MACD".to_string(),
                "Bollinger".to_string(),
                "EMA".to_string(),
            ],
            market_condition: "Volatile".to_string(),
            risk_level: "Low".to_string(),
            user_preferences: HashMap::new(),
            technical_indicators: HashMap::new(),
        };

        assert_eq!(context.selected_strategies.len(), 4);
        assert!(context.selected_strategies.contains(&"RSI".to_string()));
        assert!(context.selected_strategies.contains(&"MACD".to_string()));
    }

    #[test]
    fn test_ai_market_analysis_with_many_levels() {
        let analysis = AIMarketAnalysis {
            trend_direction: "sideways".to_string(),
            trend_strength: 0.3,
            support_levels: vec![40000.0, 39000.0, 38000.0, 37000.0],
            resistance_levels: vec![42000.0, 43000.0, 44000.0],
            volatility_level: "low".to_string(),
            volume_analysis: "stable".to_string(),
        };

        assert_eq!(analysis.support_levels.len(), 4);
        assert_eq!(analysis.resistance_levels.len(), 3);
        assert_eq!(analysis.trend_direction, "sideways");
    }

    // ============================================================================
    // Async stub tests — verify no-op behavior (no HTTP calls made)
    // ============================================================================

    fn create_test_strategy_input() -> StrategyInput {
        StrategyInput {
            symbol: "BTCUSDT".to_string(),
            timeframe_data: HashMap::new(),
            current_price: 50000.0,
            volume_24h: 1000000.0,
            timestamp: 1700000000000,
        }
    }

    #[tokio::test]
    async fn test_analyze_for_trading_signal_returns_err() {
        let service = AIService::new(AIServiceConfig::default());
        let data = create_test_strategy_input();
        let context = AIStrategyContext::default();
        let result = service.analyze_for_trading_signal(&data, context).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_get_strategy_recommendations_returns_empty() {
        let service = AIService::new(AIServiceConfig::default());
        let data = create_test_strategy_input();
        let result = service
            .get_strategy_recommendations(&data, vec!["RSI".to_string()])
            .await;
        assert!(result.is_ok());
        assert!(result.unwrap().is_empty());
    }

    #[tokio::test]
    async fn test_analyze_market_condition_returns_err() {
        let service = AIService::new(AIServiceConfig::default());
        let data = create_test_strategy_input();
        let result = service.analyze_market_condition(&data).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_send_performance_feedback_returns_ok() {
        let service = AIService::new(AIServiceConfig::default());
        let feedback = PerformanceFeedback {
            signal_id: "sig-001".to_string(),
            symbol: "BTCUSDT".to_string(),
            predicted_signal: TradingSignal::Long,
            actual_outcome: "success".to_string(),
            profit_loss: 150.0,
            confidence_was_accurate: true,
            feedback_notes: Some("Good signal".to_string()),
            timestamp: 1700000000000,
        };
        let result = service.send_performance_feedback(feedback).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_get_service_info_returns_err() {
        let service = AIService::new(AIServiceConfig::default());
        let result = service.get_service_info().await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_get_supported_strategies_returns_ok() {
        let service = AIService::new(AIServiceConfig::default());
        let result = service.get_supported_strategies().await;
        assert!(result.is_ok());
        let strategies = result.unwrap();
        assert!(!strategies.strategies.is_empty());
        assert!(strategies.strategies.iter().any(|s| s.contains("RSI")));
    }

    #[tokio::test]
    async fn test_request_trade_analysis_returns_ok() {
        use crate::ai::client::TradeAnalysisRequest;
        let service = AIService::new(AIServiceConfig::default());
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
            duration_seconds: None,
            close_reason: None,
            open_time: None,
            close_time: None,
            strategy_name: None,
            ai_confidence: None,
            ai_reasoning: None,
        };
        let result = service.request_trade_analysis(&request).await;
        assert!(result.is_ok());
    }
}
