pub mod client;
pub mod types;

use crate::strategies::{StrategyInput, TradingSignal};
use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

// Re-export key types
pub use client::*;

/// AI analysis service for communicating with Python AI
#[derive(Debug, Clone)]
pub struct AIService {
    client: AIClient,
    config: AIServiceConfig,
}

/// Configuration for AI service
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AIServiceConfig {
    pub python_service_url: String,
    pub request_timeout_seconds: u64,
    pub max_retries: u32,
    pub enable_caching: bool,
    pub cache_ttl_seconds: u64,
}

impl AIService {
    pub fn new(config: AIServiceConfig) -> Self {
        let client = AIClient::new(&config.python_service_url, config.request_timeout_seconds);

        Self { client, config }
    }

    /// Analyze market data using AI and return trading signal
    pub async fn analyze_for_trading_signal(
        &self,
        data: &StrategyInput,
        strategy_context: AIStrategyContext,
    ) -> Result<AISignalResponse> {
        let request = AIAnalysisRequest {
            symbol: data.symbol.clone(),
            timeframe_data: data.timeframe_data.clone(),
            current_price: data.current_price,
            volume_24h: data.volume_24h,
            timestamp: data.timestamp,
            strategy_context,
        };

        let mut attempts = 0;
        let max_retries = self.config.max_retries;

        while attempts <= max_retries {
            match self.client.analyze_trading_signals(&request).await {
                Ok(response) => return Ok(response),
                Err(e) => {
                    attempts += 1;
                    if attempts > max_retries {
                        return Err(e);
                    }

                    // Exponential backoff
                    let delay = std::time::Duration::from_millis(100 * (2_u64.pow(attempts - 1)));
                    tokio::time::sleep(delay).await;

                    log::warn!("AI analysis attempt {attempts} failed, retrying: {e}");
                }
            }
        }

        Err(anyhow::anyhow!(
            "AI analysis failed after {} attempts",
            max_retries
        ))
    }

    /// Get AI recommendations for strategy selection
    pub async fn get_strategy_recommendations(
        &self,
        market_data: &StrategyInput,
        available_strategies: Vec<String>,
    ) -> Result<Vec<StrategyRecommendation>> {
        let request = StrategyRecommendationRequest {
            symbol: market_data.symbol.clone(),
            timeframe_data: market_data.timeframe_data.clone(),
            current_price: market_data.current_price,
            available_strategies,
            timestamp: market_data.timestamp,
        };

        self.client.get_strategy_recommendations(&request).await
    }

    /// Get market condition analysis
    pub async fn analyze_market_condition(
        &self,
        data: &StrategyInput,
    ) -> Result<MarketConditionAnalysis> {
        let request = MarketConditionRequest {
            symbol: data.symbol.clone(),
            timeframe_data: data.timeframe_data.clone(),
            current_price: data.current_price,
            volume_24h: data.volume_24h,
            timestamp: data.timestamp,
        };

        self.client.analyze_market_condition(&request).await
    }

    /// Send strategy performance feedback to AI for learning
    pub async fn send_performance_feedback(&self, feedback: PerformanceFeedback) -> Result<()> {
        self.client.send_performance_feedback(&feedback).await
    }

    /// Get AI service information
    pub async fn get_service_info(&self) -> Result<crate::ai::client::AIServiceInfo> {
        self.client.get_service_info().await
    }

    /// Get supported strategies
    pub async fn get_supported_strategies(
        &self,
    ) -> Result<crate::ai::client::SupportedStrategiesResponse> {
        self.client.get_supported_strategies().await
    }
}

impl Default for AIServiceConfig {
    fn default() -> Self {
        Self {
            python_service_url: "http://localhost:8000".to_string(),
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
    pub strategy_scores: HashMap<String, f64>,
    pub market_analysis: AIMarketAnalysis,
    pub risk_assessment: AIRiskAssessment,
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
    pub characteristics: Vec<String>,
    pub recommended_strategies: Vec<String>,
    pub market_phase: String,
}

/// Market condition request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MarketConditionRequest {
    pub symbol: String,
    pub timeframe_data: HashMap<String, Vec<crate::market_data::cache::CandleData>>,
    pub current_price: f64,
    pub volume_24h: f64,
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

        assert_eq!(config.python_service_url, "http://localhost:8000");
        assert_eq!(config.request_timeout_seconds, 30);
        assert_eq!(config.max_retries, 3);
        assert_eq!(config.enable_caching, true);
        assert_eq!(config.cache_ttl_seconds, 300);
    }

    #[test]
    fn test_ai_service_config_serialization() {
        let config = AIServiceConfig {
            python_service_url: "http://ai-service:8000".to_string(),
            request_timeout_seconds: 60,
            max_retries: 5,
            enable_caching: false,
            cache_ttl_seconds: 600,
        };

        let json = serde_json::to_string(&config).unwrap();
        let deserialized: AIServiceConfig = serde_json::from_str(&json).unwrap();

        assert_eq!(deserialized.python_service_url, "http://ai-service:8000");
        assert_eq!(deserialized.request_timeout_seconds, 60);
        assert_eq!(deserialized.max_retries, 5);
        assert_eq!(deserialized.enable_caching, false);
        assert_eq!(deserialized.cache_ttl_seconds, 600);
    }

    #[test]
    fn test_ai_service_new() {
        let config = AIServiceConfig::default();
        let service = AIService::new(config.clone());

        assert_eq!(service.config.python_service_url, config.python_service_url);
        assert_eq!(service.config.request_timeout_seconds, config.request_timeout_seconds);
        assert_eq!(service.config.max_retries, config.max_retries);
    }

    #[test]
    fn test_ai_service_new_with_custom_config() {
        let config = AIServiceConfig {
            python_service_url: "http://custom-ai:9000".to_string(),
            request_timeout_seconds: 45,
            max_retries: 2,
            enable_caching: true,
            cache_ttl_seconds: 180,
        };

        let service = AIService::new(config.clone());

        assert_eq!(service.config.python_service_url, "http://custom-ai:9000");
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
            characteristics: vec!["strong_momentum".to_string(), "high_volume".to_string()],
            recommended_strategies: vec!["trend_following".to_string()],
            market_phase: "accumulation".to_string(),
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
            python_service_url: "http://192.168.1.100:8888".to_string(),
            request_timeout_seconds: 120,
            max_retries: 10,
            enable_caching: true,
            cache_ttl_seconds: 1800,
        };

        assert_eq!(config.python_service_url, "http://192.168.1.100:8888");
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
}
