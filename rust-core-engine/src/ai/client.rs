use super::*;
use anyhow::{anyhow, Result};
use reqwest::{Client, RequestBuilder};
use serde_json;
use std::collections::HashMap;
use std::time::Duration;

// Helper structure for Python AI service (matches its expected format)
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
struct PythonCandleData {
    timestamp: i64,
    open: f64,
    high: f64,
    low: f64,
    close: f64,
    volume: f64,
}

// Helper structure for Python AI service request
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
struct PythonAIAnalysisRequest {
    symbol: String,
    timeframe_data: HashMap<String, Vec<PythonCandleData>>,
    current_price: f64,
    volume_24h: f64,
    timestamp: i64,
    strategy_context: AIStrategyContext,
}

// Helper structure for Python strategy recommendation request
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
struct PythonStrategyRecommendationRequest {
    symbol: String,
    timeframe_data: HashMap<String, Vec<PythonCandleData>>,
    current_price: f64,
    available_strategies: Vec<String>,
    timestamp: i64,
}

// Helper structure for Python market condition request
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
struct PythonMarketConditionRequest {
    symbol: String,
    timeframe_data: HashMap<String, Vec<PythonCandleData>>,
    current_price: f64,
    volume_24h: f64,
    timestamp: i64,
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

impl From<&AIAnalysisRequest> for PythonAIAnalysisRequest {
    fn from(request: &AIAnalysisRequest) -> Self {
        let mut python_timeframe_data = HashMap::new();
        
        for (timeframe, candles) in &request.timeframe_data {
            let python_candles: Vec<PythonCandleData> = candles
                .iter()
                .map(PythonCandleData::from)
                .collect();
            python_timeframe_data.insert(timeframe.clone(), python_candles);
        }
        
        Self {
            symbol: request.symbol.clone(),
            timeframe_data: python_timeframe_data,
            current_price: request.current_price,
            volume_24h: request.volume_24h,
            timestamp: request.timestamp,
            strategy_context: request.strategy_context.clone(),
        }
    }
}

impl From<&StrategyRecommendationRequest> for PythonStrategyRecommendationRequest {
    fn from(request: &StrategyRecommendationRequest) -> Self {
        let mut python_timeframe_data = HashMap::new();
        
        for (timeframe, candles) in &request.timeframe_data {
            let python_candles: Vec<PythonCandleData> = candles
                .iter()
                .map(PythonCandleData::from)
                .collect();
            python_timeframe_data.insert(timeframe.clone(), python_candles);
        }
        
        Self {
            symbol: request.symbol.clone(),
            timeframe_data: python_timeframe_data,
            current_price: request.current_price,
            available_strategies: request.available_strategies.clone(),
            timestamp: request.timestamp,
        }
    }
}

impl From<&MarketConditionRequest> for PythonMarketConditionRequest {
    fn from(request: &MarketConditionRequest) -> Self {
        let mut python_timeframe_data = HashMap::new();
        
        for (timeframe, candles) in &request.timeframe_data {
            let python_candles: Vec<PythonCandleData> = candles
                .iter()
                .map(PythonCandleData::from)
                .collect();
            python_timeframe_data.insert(timeframe.clone(), python_candles);
        }
        
        Self {
            symbol: request.symbol.clone(),
            timeframe_data: python_timeframe_data,
            current_price: request.current_price,
            volume_24h: request.volume_24h,
            timestamp: request.timestamp,
        }
    }
}

/// HTTP client for communicating with Python AI service
#[derive(Debug, Clone)]
pub struct AIClient {
    client: Client,
    base_url: String,
    timeout: Duration,
}

impl AIClient {
    pub fn new(base_url: &str, timeout_seconds: u64) -> Self {
        let client = Client::builder()
            .timeout(Duration::from_secs(timeout_seconds))
            .build()
            .expect("Failed to create HTTP client");
        
        Self {
            client,
            base_url: base_url.trim_end_matches('/').to_string(),
            timeout: Duration::from_secs(timeout_seconds),
        }
    }
    
    /// Analyze trading signals using AI
    pub async fn analyze_trading_signals(&self, request: &AIAnalysisRequest) -> Result<AISignalResponse> {
        let url = format!("{}/ai/analyze", self.base_url);
        
        // Transform the request to Python-expected format
        let python_request = PythonAIAnalysisRequest::from(request);
        
        let response = self.client
            .post(&url)
            .header("Content-Type", "application/json")
            .json(&python_request)
            .send()
            .await
            .map_err(|e| anyhow!("Failed to send AI analysis request: {}", e))?;
        
        if !response.status().is_success() {
            let status = response.status();
            let error_text = response.text().await.unwrap_or_else(|_| "Unknown error".to_string());
            return Err(anyhow!("AI analysis request failed with status {}: {}", status, error_text));
        }
        
        let ai_response: AISignalResponse = response
            .json()
            .await
            .map_err(|e| anyhow!("Failed to parse AI analysis response: {}", e))?;
        
        Ok(ai_response)
    }
    
    /// Get strategy recommendations from AI
    pub async fn get_strategy_recommendations(&self, request: &StrategyRecommendationRequest) -> Result<Vec<StrategyRecommendation>> {
        let url = format!("{}/ai/strategy-recommendations", self.base_url);
        
        // Transform the request to Python-expected format
        let python_request = PythonStrategyRecommendationRequest::from(request);
        
        let response = self.client
            .post(&url)
            .header("Content-Type", "application/json")
            .json(&python_request)
            .send()
            .await
            .map_err(|e| anyhow!("Failed to send strategy recommendation request: {}", e))?;
        
        if !response.status().is_success() {
            let status = response.status();
            let error_text = response.text().await.unwrap_or_else(|_| "Unknown error".to_string());
            return Err(anyhow!("Strategy recommendation request failed with status {}: {}", status, error_text));
        }
        
        let recommendations: Vec<StrategyRecommendation> = response
            .json()
            .await
            .map_err(|e| anyhow!("Failed to parse strategy recommendations response: {}", e))?;
        
        Ok(recommendations)
    }
    
    /// Analyze market condition using AI
    pub async fn analyze_market_condition(&self, request: &MarketConditionRequest) -> Result<MarketConditionAnalysis> {
        let url = format!("{}/ai/market-condition", self.base_url);
        
        // Transform the request to Python-expected format
        let python_request = PythonMarketConditionRequest::from(request);
        
        let response = self.client
            .post(&url)
            .header("Content-Type", "application/json")
            .json(&python_request)
            .send()
            .await
            .map_err(|e| anyhow!("Failed to send market condition request: {}", e))?;
        
        if !response.status().is_success() {
            let status = response.status();
            let error_text = response.text().await.unwrap_or_else(|_| "Unknown error".to_string());
            return Err(anyhow!("Market condition request failed with status {}: {}", status, error_text));
        }
        
        let analysis: MarketConditionAnalysis = response
            .json()
            .await
            .map_err(|e| anyhow!("Failed to parse market condition response: {}", e))?;
        
        Ok(analysis)
    }
    
    /// Send performance feedback to AI for learning
    pub async fn send_performance_feedback(&self, feedback: &PerformanceFeedback) -> Result<()> {
        let url = format!("{}/ai/feedback", self.base_url);
        
        let response = self.client
            .post(&url)
            .header("Content-Type", "application/json")
            .json(feedback)
            .send()
            .await
            .map_err(|e| anyhow!("Failed to send performance feedback: {}", e))?;
        
        if !response.status().is_success() {
            let status = response.status();
            let error_text = response.text().await.unwrap_or_else(|_| "Unknown error".to_string());
            return Err(anyhow!("Performance feedback request failed with status {}: {}", status, error_text));
        }
        
        Ok(())
    }
    
    /// Health check for AI service
    pub async fn health_check(&self) -> Result<bool> {
        let url = format!("{}/health", self.base_url);
        
        let response = self.client
            .get(&url)
            .send()
            .await
            .map_err(|e| anyhow!("Failed to perform health check: {}", e))?;
        
        Ok(response.status().is_success())
    }
    
    /// Get AI service information
    pub async fn get_service_info(&self) -> Result<AIServiceInfo> {
        let url = format!("{}/ai/info", self.base_url);
        
        let response = self.client
            .get(&url)
            .send()
            .await
            .map_err(|e| anyhow!("Failed to get service info: {}", e))?;
        
        if !response.status().is_success() {
            let status = response.status();
            let error_text = response.text().await.unwrap_or_else(|_| "Unknown error".to_string());
            return Err(anyhow!("Service info request failed with status {}: {}", status, error_text));
        }
        
        let info: AIServiceInfo = response
            .json()
            .await
            .map_err(|e| anyhow!("Failed to parse service info response: {}", e))?;
        
        Ok(info)
    }
    
    /// Get supported strategies from AI service
    pub async fn get_supported_strategies(&self) -> Result<SupportedStrategiesResponse> {
        let url = format!("{}/ai/strategies", self.base_url);
        
        let response = self.client
            .get(&url)
            .send()
            .await
            .map_err(|e| anyhow!("Failed to get supported strategies: {}", e))?;
        
        if !response.status().is_success() {
            let status = response.status();
            let error_text = response.text().await.unwrap_or_else(|_| "Unknown error".to_string());
            return Err(anyhow!("Supported strategies request failed with status {}: {}", status, error_text));
        }
        
        let strategies: SupportedStrategiesResponse = response
            .json()
            .await
            .map_err(|e| anyhow!("Failed to parse supported strategies response: {}", e))?;
        
        Ok(strategies)
    }
    
    /// Get AI model performance metrics
    pub async fn get_model_performance(&self) -> Result<AIModelPerformance> {
        let url = format!("{}/ai/performance", self.base_url);
        
        let response = self.client
            .get(&url)
            .send()
            .await
            .map_err(|e| anyhow!("Failed to get model performance: {}", e))?;
        
        if !response.status().is_success() {
            let status = response.status();
            let error_text = response.text().await.unwrap_or_else(|_| "Unknown error".to_string());
            return Err(anyhow!("Model performance request failed with status {}: {}", status, error_text));
        }
        
        let performance: AIModelPerformance = response
            .json()
            .await
            .map_err(|e| anyhow!("Failed to parse model performance response: {}", e))?;
        
        Ok(performance)
    }
}

/// AI service information
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

/// AI model performance metrics
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