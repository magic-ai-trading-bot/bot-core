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

        self.client
            .get_strategy_recommendations(&request)
            .await
            .map_err(|e| e)
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

        self.client
            .analyze_market_condition(&request)
            .await
            .map_err(|e| e)
    }

    /// Send strategy performance feedback to AI for learning
    pub async fn send_performance_feedback(&self, feedback: PerformanceFeedback) -> Result<()> {
        self.client
            .send_performance_feedback(&feedback)
            .await
            .map_err(|e| e)
    }

    /// Get AI service information
    pub async fn get_service_info(&self) -> Result<crate::ai::client::AIServiceInfo> {
        self.client.get_service_info().await.map_err(|e| e)
    }

    /// Get supported strategies
    pub async fn get_supported_strategies(
        &self,
    ) -> Result<crate::ai::client::SupportedStrategiesResponse> {
        self.client.get_supported_strategies().await.map_err(|e| e)
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
