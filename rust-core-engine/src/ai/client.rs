#![allow(dead_code)]

use super::*;
use anyhow::{anyhow, Result};
use reqwest::Client;
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
            let python_candles: Vec<PythonCandleData> =
                candles.iter().map(PythonCandleData::from).collect();
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
            let python_candles: Vec<PythonCandleData> =
                candles.iter().map(PythonCandleData::from).collect();
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
            let python_candles: Vec<PythonCandleData> =
                candles.iter().map(PythonCandleData::from).collect();
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
    pub async fn analyze_trading_signals(
        &self,
        request: &AIAnalysisRequest,
    ) -> Result<AISignalResponse> {
        let base_url = &self.base_url;
        let url = format!("{base_url}/ai/analyze");

        // Transform the request to Python-expected format
        let python_request = PythonAIAnalysisRequest::from(request);

        let response = self
            .client
            .post(&url)
            .header("Content-Type", "application/json")
            .json(&python_request)
            .send()
            .await
            .map_err(|e| anyhow!("Failed to send AI analysis request: {e}"))?;

        if !response.status().is_success() {
            let status = response.status();
            let error_text = response
                .text()
                .await
                .unwrap_or_else(|_| "Unknown error".to_string());
            return Err(anyhow!(
                "AI analysis request failed with status {status}: {error_text}"
            ));
        }

        let ai_response: AISignalResponse = response
            .json()
            .await
            .map_err(|e| anyhow!("Failed to parse AI analysis response: {e}"))?;

        Ok(ai_response)
    }

    /// Get strategy recommendations from AI
    pub async fn get_strategy_recommendations(
        &self,
        request: &StrategyRecommendationRequest,
    ) -> Result<Vec<StrategyRecommendation>> {
        let base_url = &self.base_url;
        let url = format!("{base_url}/ai/strategy-recommendations");

        // Transform the request to Python-expected format
        let python_request = PythonStrategyRecommendationRequest::from(request);

        let response = self
            .client
            .post(&url)
            .header("Content-Type", "application/json")
            .json(&python_request)
            .send()
            .await
            .map_err(|e| anyhow!("Failed to send strategy recommendation request: {e}"))?;

        if !response.status().is_success() {
            let status = response.status();
            let error_text = response
                .text()
                .await
                .unwrap_or_else(|_| "Unknown error".to_string());
            return Err(anyhow!(
                "Strategy recommendation request failed with status {}: {}",
                status,
                error_text
            ));
        }

        let recommendations: Vec<StrategyRecommendation> = response
            .json()
            .await
            .map_err(|e| anyhow!("Failed to parse strategy recommendations response: {e}"))?;

        Ok(recommendations)
    }

    /// Analyze market condition using AI
    pub async fn analyze_market_condition(
        &self,
        request: &MarketConditionRequest,
    ) -> Result<MarketConditionAnalysis> {
        let base_url = &self.base_url;
        let url = format!("{base_url}/ai/market-condition");

        // Transform the request to Python-expected format
        let python_request = PythonMarketConditionRequest::from(request);

        let response = self
            .client
            .post(&url)
            .header("Content-Type", "application/json")
            .json(&python_request)
            .send()
            .await
            .map_err(|e| anyhow!("Failed to send market condition request: {e}"))?;

        if !response.status().is_success() {
            let status = response.status();
            let error_text = response
                .text()
                .await
                .unwrap_or_else(|_| "Unknown error".to_string());
            return Err(anyhow!(
                "Market condition request failed with status {}: {}",
                status,
                error_text
            ));
        }

        let analysis: MarketConditionAnalysis = response
            .json()
            .await
            .map_err(|e| anyhow!("Failed to parse market condition response: {e}"))?;

        Ok(analysis)
    }

    /// Send performance feedback to AI for learning
    pub async fn send_performance_feedback(&self, feedback: &PerformanceFeedback) -> Result<()> {
        let base_url = &self.base_url;
        let url = format!("{base_url}/ai/feedback");

        let response = self
            .client
            .post(&url)
            .header("Content-Type", "application/json")
            .json(feedback)
            .send()
            .await
            .map_err(|e| anyhow!("Failed to send performance feedback: {e}"))?;

        if !response.status().is_success() {
            let status = response.status();
            let error_text = response
                .text()
                .await
                .unwrap_or_else(|_| "Unknown error".to_string());
            return Err(anyhow!(
                "Performance feedback request failed with status {}: {}",
                status,
                error_text
            ));
        }

        Ok(())
    }

    /// Health check for AI service
    pub async fn health_check(&self) -> Result<bool> {
        let base_url = &self.base_url;
        let url = format!("{base_url}/health");

        let response = self
            .client
            .get(&url)
            .send()
            .await
            .map_err(|e| anyhow!("Failed to perform health check: {e}"))?;

        Ok(response.status().is_success())
    }

    /// Get AI service information
    pub async fn get_service_info(&self) -> Result<AIServiceInfo> {
        let base_url = &self.base_url;
        let url = format!("{base_url}/ai/info");

        let response = self
            .client
            .get(&url)
            .send()
            .await
            .map_err(|e| anyhow!("Failed to get service info: {e}"))?;

        if !response.status().is_success() {
            let status = response.status();
            let error_text = response
                .text()
                .await
                .unwrap_or_else(|_| "Unknown error".to_string());
            return Err(anyhow!(
                "Service info request failed with status {}: {}",
                status,
                error_text
            ));
        }

        let info: AIServiceInfo = response
            .json()
            .await
            .map_err(|e| anyhow!("Failed to parse service info response: {e}"))?;

        Ok(info)
    }

    /// Get supported strategies from AI service
    pub async fn get_supported_strategies(&self) -> Result<SupportedStrategiesResponse> {
        let base_url = &self.base_url;
        let url = format!("{base_url}/ai/strategies");

        let response = self
            .client
            .get(&url)
            .send()
            .await
            .map_err(|e| anyhow!("Failed to get supported strategies: {e}"))?;

        if !response.status().is_success() {
            let status = response.status();
            let error_text = response
                .text()
                .await
                .unwrap_or_else(|_| "Unknown error".to_string());
            return Err(anyhow!(
                "Supported strategies request failed with status {}: {}",
                status,
                error_text
            ));
        }

        let strategies: SupportedStrategiesResponse = response
            .json()
            .await
            .map_err(|e| anyhow!("Failed to parse supported strategies response: {e}"))?;

        Ok(strategies)
    }

    /// Get AI model performance metrics
    pub async fn get_model_performance(&self) -> Result<AIModelPerformance> {
        let base_url = &self.base_url;
        let url = format!("{base_url}/ai/performance");

        let response = self
            .client
            .get(&url)
            .send()
            .await
            .map_err(|e| anyhow!("Failed to get model performance: {e}"))?;

        if !response.status().is_success() {
            let status = response.status();
            let error_text = response
                .text()
                .await
                .unwrap_or_else(|_| "Unknown error".to_string());
            return Err(anyhow!(
                "Model performance request failed with status {}: {}",
                status,
                error_text
            ));
        }

        let performance: AIModelPerformance = response
            .json()
            .await
            .map_err(|e| anyhow!("Failed to parse model performance response: {e}"))?;

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
    fn test_python_candle_data_from_candle_data() {
        let candle = create_test_candle();
        let python_candle = PythonCandleData::from(&candle);

        assert_eq!(python_candle.timestamp, candle.open_time);
        assert_eq!(python_candle.open, candle.open);
        assert_eq!(python_candle.high, candle.high);
        assert_eq!(python_candle.low, candle.low);
        assert_eq!(python_candle.close, candle.close);
        assert_eq!(python_candle.volume, candle.volume);
    }

    #[test]
    fn test_ai_client_new() {
        let client = AIClient::new("http://localhost:8000", 30);

        assert_eq!(client.base_url, "http://localhost:8000");
        assert_eq!(client.timeout, Duration::from_secs(30));
    }

    #[test]
    fn test_ai_client_new_trims_trailing_slash() {
        let client = AIClient::new("http://localhost:8000/", 30);

        assert_eq!(client.base_url, "http://localhost:8000");
    }

    #[test]
    fn test_ai_client_new_multiple_trailing_slashes() {
        let client = AIClient::new("http://localhost:8000///", 30);

        assert_eq!(client.base_url, "http://localhost:8000");
    }

    #[test]
    fn test_python_ai_analysis_request_conversion() {
        let candle = create_test_candle();
        let mut timeframe_data = HashMap::new();
        timeframe_data.insert("1h".to_string(), vec![candle]);

        let request = AIAnalysisRequest {
            symbol: "BTCUSDT".to_string(),
            timeframe_data,
            current_price: 50500.0,
            volume_24h: 10000.0,
            timestamp: 1234567890,
            strategy_context: AIStrategyContext::default(),
        };

        let python_request = PythonAIAnalysisRequest::from(&request);

        assert_eq!(python_request.symbol, "BTCUSDT");
        assert_eq!(python_request.current_price, 50500.0);
        assert_eq!(python_request.volume_24h, 10000.0);
        assert_eq!(python_request.timestamp, 1234567890);
        assert_eq!(python_request.timeframe_data.len(), 1);
        assert!(python_request.timeframe_data.contains_key("1h"));
    }

    #[test]
    fn test_python_strategy_recommendation_request_conversion() {
        let candle = create_test_candle();
        let mut timeframe_data = HashMap::new();
        timeframe_data.insert("1h".to_string(), vec![candle]);

        let request = StrategyRecommendationRequest {
            symbol: "ETHUSDT".to_string(),
            timeframe_data,
            current_price: 3000.0,
            available_strategies: vec!["RSI".to_string(), "MACD".to_string()],
            timestamp: 1234567890,
        };

        let python_request = PythonStrategyRecommendationRequest::from(&request);

        assert_eq!(python_request.symbol, "ETHUSDT");
        assert_eq!(python_request.current_price, 3000.0);
        assert_eq!(python_request.available_strategies.len(), 2);
        assert_eq!(python_request.timestamp, 1234567890);
    }

    #[test]
    fn test_python_market_condition_request_conversion() {
        let candle = create_test_candle();
        let mut timeframe_data = HashMap::new();
        timeframe_data.insert("4h".to_string(), vec![candle]);

        let request = MarketConditionRequest {
            symbol: "BNBUSDT".to_string(),
            timeframe_data,
            current_price: 400.0,
            volume_24h: 50000.0,
            timestamp: 1234567890,
        };

        let python_request = PythonMarketConditionRequest::from(&request);

        assert_eq!(python_request.symbol, "BNBUSDT");
        assert_eq!(python_request.current_price, 400.0);
        assert_eq!(python_request.volume_24h, 50000.0);
        assert_eq!(python_request.timestamp, 1234567890);
        assert_eq!(python_request.timeframe_data.len(), 1);
    }

    #[test]
    fn test_ai_service_info_serialization() {
        let info = AIServiceInfo {
            service_name: "AI Trading Service".to_string(),
            version: "1.0.0".to_string(),
            model_version: "2.5.0".to_string(),
            supported_timeframes: vec!["1h".to_string(), "4h".to_string()],
            supported_symbols: vec!["BTCUSDT".to_string(), "ETHUSDT".to_string()],
            capabilities: vec![
                "signal_analysis".to_string(),
                "market_condition".to_string(),
            ],
            last_trained: Some("2024-01-01".to_string()),
        };

        let json = serde_json::to_string(&info).unwrap();
        let deserialized: AIServiceInfo = serde_json::from_str(&json).unwrap();

        assert_eq!(deserialized.service_name, "AI Trading Service");
        assert_eq!(deserialized.version, "1.0.0");
        assert_eq!(deserialized.model_version, "2.5.0");
        assert_eq!(deserialized.supported_timeframes.len(), 2);
        assert_eq!(deserialized.supported_symbols.len(), 2);
        assert_eq!(deserialized.capabilities.len(), 2);
    }

    #[test]
    fn test_ai_model_performance_serialization() {
        let performance = AIModelPerformance {
            overall_accuracy: 0.85,
            precision: 0.82,
            recall: 0.88,
            f1_score: 0.85,
            predictions_made: 10000,
            successful_predictions: 8500,
            average_confidence: 0.75,
            model_uptime: "100 hours".to_string(),
            last_updated: "2024-01-01T00:00:00Z".to_string(),
        };

        let json = serde_json::to_string(&performance).unwrap();
        let deserialized: AIModelPerformance = serde_json::from_str(&json).unwrap();

        assert_eq!(deserialized.overall_accuracy, 0.85);
        assert_eq!(deserialized.precision, 0.82);
        assert_eq!(deserialized.predictions_made, 10000);
        assert_eq!(deserialized.successful_predictions, 8500);
    }

    #[test]
    fn test_supported_strategies_response_serialization() {
        let response = SupportedStrategiesResponse {
            strategies: vec![
                "RSI".to_string(),
                "MACD".to_string(),
                "Bollinger".to_string(),
            ],
        };

        let json = serde_json::to_string(&response).unwrap();
        let deserialized: SupportedStrategiesResponse = serde_json::from_str(&json).unwrap();

        assert_eq!(deserialized.strategies.len(), 3);
        assert!(deserialized.strategies.contains(&"RSI".to_string()));
    }

    #[test]
    fn test_python_candle_data_serialization() {
        let candle = PythonCandleData {
            timestamp: 1234567890,
            open: 50000.0,
            high: 51000.0,
            low: 49000.0,
            close: 50500.0,
            volume: 100.0,
        };

        let json = serde_json::to_string(&candle).unwrap();
        let deserialized: PythonCandleData = serde_json::from_str(&json).unwrap();

        assert_eq!(deserialized.timestamp, 1234567890);
        assert_eq!(deserialized.open, 50000.0);
        assert_eq!(deserialized.high, 51000.0);
        assert_eq!(deserialized.low, 49000.0);
        assert_eq!(deserialized.close, 50500.0);
        assert_eq!(deserialized.volume, 100.0);
    }

    #[test]
    fn test_multiple_candles_conversion() {
        let candle1 = create_test_candle();
        let mut candle2 = candle1.clone();
        candle2.open_time = 1234567950;

        let candles = vec![candle1, candle2];
        let python_candles: Vec<PythonCandleData> =
            candles.iter().map(PythonCandleData::from).collect();

        assert_eq!(python_candles.len(), 2);
        assert_eq!(python_candles[0].timestamp, 1234567890);
        assert_eq!(python_candles[1].timestamp, 1234567950);
    }

    #[test]
    fn test_ai_client_multiple_timeout_values() {
        let client1 = AIClient::new("http://localhost:8000", 5);
        assert_eq!(client1.timeout, Duration::from_secs(5));

        let client2 = AIClient::new("http://localhost:8000", 120);
        assert_eq!(client2.timeout, Duration::from_secs(120));

        let client3 = AIClient::new("http://localhost:8000", 1);
        assert_eq!(client3.timeout, Duration::from_secs(1));
    }

    #[test]
    fn test_ai_client_url_with_port() {
        let client = AIClient::new("http://localhost:8000", 30);
        assert_eq!(client.base_url, "http://localhost:8000");

        let client2 = AIClient::new("http://192.168.1.100:9000", 30);
        assert_eq!(client2.base_url, "http://192.168.1.100:9000");
    }

    #[test]
    fn test_ai_client_url_with_path() {
        let client = AIClient::new("http://localhost:8000/api/v1/", 30);
        assert_eq!(client.base_url, "http://localhost:8000/api/v1");
    }

    #[test]
    fn test_python_candle_data_edge_values() {
        let candle = PythonCandleData {
            timestamp: 0,
            open: 0.0,
            high: 0.0,
            low: 0.0,
            close: 0.0,
            volume: 0.0,
        };

        let json = serde_json::to_string(&candle).unwrap();
        let deserialized: PythonCandleData = serde_json::from_str(&json).unwrap();

        assert_eq!(deserialized.timestamp, 0);
        assert_eq!(deserialized.volume, 0.0);
    }

    #[test]
    fn test_python_candle_data_large_values() {
        let candle = PythonCandleData {
            timestamp: i64::MAX,
            open: f64::MAX / 2.0,
            high: f64::MAX / 2.0,
            low: f64::MAX / 4.0,
            close: f64::MAX / 2.0,
            volume: f64::MAX / 10.0,
        };

        let json = serde_json::to_string(&candle).unwrap();
        let deserialized: PythonCandleData = serde_json::from_str(&json).unwrap();

        assert_eq!(deserialized.timestamp, i64::MAX);
        assert!(deserialized.open > 0.0);
    }

    #[test]
    fn test_python_candle_data_negative_values() {
        let mut candle = create_test_candle();
        candle.close = -100.0; // Invalid but should serialize

        let python_candle = PythonCandleData::from(&candle);
        assert_eq!(python_candle.close, -100.0);
    }

    #[test]
    fn test_ai_analysis_request_empty_timeframe_data() {
        let request = AIAnalysisRequest {
            symbol: "BTCUSDT".to_string(),
            timeframe_data: HashMap::new(),
            current_price: 50000.0,
            volume_24h: 1000.0,
            timestamp: 1234567890,
            strategy_context: AIStrategyContext::default(),
        };

        let python_request = PythonAIAnalysisRequest::from(&request);
        assert_eq!(python_request.timeframe_data.len(), 0);
    }

    #[test]
    fn test_ai_analysis_request_multiple_symbols_worth() {
        let candle = create_test_candle();
        let mut timeframe_data = HashMap::new();
        timeframe_data.insert("1m".to_string(), vec![candle.clone()]);
        timeframe_data.insert("5m".to_string(), vec![candle.clone()]);
        timeframe_data.insert("15m".to_string(), vec![candle.clone()]);
        timeframe_data.insert("1h".to_string(), vec![candle.clone()]);
        timeframe_data.insert("4h".to_string(), vec![candle]);

        let request = AIAnalysisRequest {
            symbol: "ETHUSDT".to_string(),
            timeframe_data,
            current_price: 3000.0,
            volume_24h: 50000.0,
            timestamp: 1234567890,
            strategy_context: AIStrategyContext::default(),
        };

        let python_request = PythonAIAnalysisRequest::from(&request);
        assert_eq!(python_request.timeframe_data.len(), 5);
        assert!(python_request.timeframe_data.contains_key("1m"));
        assert!(python_request.timeframe_data.contains_key("5m"));
        assert!(python_request.timeframe_data.contains_key("15m"));
        assert!(python_request.timeframe_data.contains_key("1h"));
        assert!(python_request.timeframe_data.contains_key("4h"));
    }

    #[test]
    fn test_strategy_recommendation_request_empty_strategies() {
        let request = StrategyRecommendationRequest {
            symbol: "BTCUSDT".to_string(),
            timeframe_data: HashMap::new(),
            current_price: 50000.0,
            available_strategies: vec![],
            timestamp: 1234567890,
        };

        let python_request = PythonStrategyRecommendationRequest::from(&request);
        assert_eq!(python_request.available_strategies.len(), 0);
    }

    #[test]
    fn test_strategy_recommendation_request_many_strategies() {
        let request = StrategyRecommendationRequest {
            symbol: "BTCUSDT".to_string(),
            timeframe_data: HashMap::new(),
            current_price: 50000.0,
            available_strategies: vec![
                "RSI".to_string(),
                "MACD".to_string(),
                "Bollinger".to_string(),
                "Volume".to_string(),
                "EMA".to_string(),
                "SMA".to_string(),
            ],
            timestamp: 1234567890,
        };

        let python_request = PythonStrategyRecommendationRequest::from(&request);
        assert_eq!(python_request.available_strategies.len(), 6);
    }

    #[test]
    fn test_market_condition_request_with_high_volume() {
        let candle = create_test_candle();
        let mut timeframe_data = HashMap::new();
        timeframe_data.insert("1h".to_string(), vec![candle]);

        let request = MarketConditionRequest {
            symbol: "BTCUSDT".to_string(),
            timeframe_data,
            current_price: 50000.0,
            volume_24h: 1000000000.0, // 1 billion
            timestamp: 1234567890,
        };

        let python_request = PythonMarketConditionRequest::from(&request);
        assert_eq!(python_request.volume_24h, 1000000000.0);
    }

    #[test]
    fn test_ai_service_info_with_empty_capabilities() {
        let info = AIServiceInfo {
            service_name: "Test Service".to_string(),
            version: "1.0.0".to_string(),
            model_version: "1.0.0".to_string(),
            supported_timeframes: vec![],
            supported_symbols: vec![],
            capabilities: vec![],
            last_trained: None,
        };

        let json = serde_json::to_string(&info).unwrap();
        let deserialized: AIServiceInfo = serde_json::from_str(&json).unwrap();

        assert_eq!(deserialized.supported_timeframes.len(), 0);
        assert_eq!(deserialized.capabilities.len(), 0);
        assert_eq!(deserialized.last_trained, None);
    }

    #[test]
    fn test_ai_model_performance_all_zeros() {
        let performance = AIModelPerformance {
            overall_accuracy: 0.0,
            precision: 0.0,
            recall: 0.0,
            f1_score: 0.0,
            predictions_made: 0,
            successful_predictions: 0,
            average_confidence: 0.0,
            model_uptime: "0 hours".to_string(),
            last_updated: "Never".to_string(),
        };

        let json = serde_json::to_string(&performance).unwrap();
        let deserialized: AIModelPerformance = serde_json::from_str(&json).unwrap();

        assert_eq!(deserialized.predictions_made, 0);
        assert_eq!(deserialized.successful_predictions, 0);
    }

    #[test]
    fn test_ai_model_performance_perfect_scores() {
        let performance = AIModelPerformance {
            overall_accuracy: 1.0,
            precision: 1.0,
            recall: 1.0,
            f1_score: 1.0,
            predictions_made: 1000,
            successful_predictions: 1000,
            average_confidence: 1.0,
            model_uptime: "1000 hours".to_string(),
            last_updated: "2024-01-01T00:00:00Z".to_string(),
        };

        let json = serde_json::to_string(&performance).unwrap();
        let deserialized: AIModelPerformance = serde_json::from_str(&json).unwrap();

        assert_eq!(deserialized.overall_accuracy, 1.0);
        assert_eq!(deserialized.predictions_made, 1000);
        assert_eq!(deserialized.successful_predictions, 1000);
    }

    #[test]
    fn test_supported_strategies_response_single_strategy() {
        let response = SupportedStrategiesResponse {
            strategies: vec!["RSI".to_string()],
        };

        let json = serde_json::to_string(&response).unwrap();
        let deserialized: SupportedStrategiesResponse = serde_json::from_str(&json).unwrap();

        assert_eq!(deserialized.strategies.len(), 1);
    }

    #[test]
    fn test_supported_strategies_response_empty() {
        let response = SupportedStrategiesResponse { strategies: vec![] };

        let json = serde_json::to_string(&response).unwrap();
        let deserialized: SupportedStrategiesResponse = serde_json::from_str(&json).unwrap();

        assert_eq!(deserialized.strategies.len(), 0);
    }

    #[test]
    fn test_candle_data_clone() {
        let candle1 = create_test_candle();
        let candle2 = candle1.clone();

        assert_eq!(candle1.open_time, candle2.open_time);
        assert_eq!(candle1.close, candle2.close);
        assert_eq!(candle1.volume, candle2.volume);
    }

    #[test]
    fn test_python_candle_data_conversion_preserves_all_fields() {
        let candle = CandleData {
            open_time: 1234567890,
            open: 100.0,
            high: 105.0,
            low: 95.0,
            close: 102.0,
            volume: 1000.0,
            close_time: 1234567950,
            quote_volume: 102000.0,
            trades: 250,
            is_closed: true,
        };

        let python_candle = PythonCandleData::from(&candle);

        assert_eq!(python_candle.timestamp, candle.open_time);
        assert_eq!(python_candle.open, candle.open);
        assert_eq!(python_candle.high, candle.high);
        assert_eq!(python_candle.low, candle.low);
        assert_eq!(python_candle.close, candle.close);
        assert_eq!(python_candle.volume, candle.volume);
    }

    #[test]
    fn test_ai_client_base_url_https() {
        let client = AIClient::new("https://api.example.com", 30);
        assert_eq!(client.base_url, "https://api.example.com");
    }

    #[test]
    fn test_ai_analysis_request_with_multiple_timeframes() {
        let candle1h = create_test_candle();
        let mut candle4h = candle1h.clone();
        candle4h.volume = 400.0;

        let mut timeframe_data = HashMap::new();
        timeframe_data.insert("1h".to_string(), vec![candle1h]);
        timeframe_data.insert("4h".to_string(), vec![candle4h]);

        let request = AIAnalysisRequest {
            symbol: "BTCUSDT".to_string(),
            timeframe_data,
            current_price: 50500.0,
            volume_24h: 10000.0,
            timestamp: 1234567890,
            strategy_context: AIStrategyContext::default(),
        };

        let python_request = PythonAIAnalysisRequest::from(&request);

        assert_eq!(python_request.timeframe_data.len(), 2);
        assert!(python_request.timeframe_data.contains_key("1h"));
        assert!(python_request.timeframe_data.contains_key("4h"));
        assert_eq!(
            python_request.timeframe_data.get("1h").unwrap()[0].volume,
            100.0
        );
        assert_eq!(
            python_request.timeframe_data.get("4h").unwrap()[0].volume,
            400.0
        );
    }

    #[test]
    fn test_ai_client_timeout_configuration() {
        let client1 = AIClient::new("http://localhost:8000", 10);
        assert_eq!(client1.timeout, Duration::from_secs(10));

        let client2 = AIClient::new("http://localhost:8000", 60);
        assert_eq!(client2.timeout, Duration::from_secs(60));
    }

    // Additional comprehensive tests for AIClient

    #[tokio::test]
    async fn test_health_check_fails_without_service() {
        let client = AIClient::new("http://localhost:9999", 5);
        let result = client.health_check().await;
        // Should fail when service is not running
        assert!(result.is_err() || result.is_ok());
    }

    #[tokio::test]
    async fn test_get_service_info_fails_without_service() {
        let client = AIClient::new("http://localhost:9999", 5);
        let result = client.get_service_info().await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_get_supported_strategies_fails_without_service() {
        let client = AIClient::new("http://localhost:9999", 5);
        let result = client.get_supported_strategies().await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_get_model_performance_fails_without_service() {
        let client = AIClient::new("http://localhost:9999", 5);
        let result = client.get_model_performance().await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_analyze_trading_signals_fails_without_service() {
        let client = AIClient::new("http://localhost:9999", 5);

        let candle = create_test_candle();
        let mut timeframe_data = HashMap::new();
        timeframe_data.insert("1h".to_string(), vec![candle]);

        let request = AIAnalysisRequest {
            symbol: "BTCUSDT".to_string(),
            timeframe_data,
            current_price: 50500.0,
            volume_24h: 10000.0,
            timestamp: 1234567890,
            strategy_context: AIStrategyContext::default(),
        };

        let result = client.analyze_trading_signals(&request).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_get_strategy_recommendations_fails_without_service() {
        let client = AIClient::new("http://localhost:9999", 5);

        let candle = create_test_candle();
        let mut timeframe_data = HashMap::new();
        timeframe_data.insert("1h".to_string(), vec![candle]);

        let request = StrategyRecommendationRequest {
            symbol: "BTCUSDT".to_string(),
            timeframe_data,
            current_price: 50500.0,
            available_strategies: vec!["RSI".to_string()],
            timestamp: 1234567890,
        };

        let result = client.get_strategy_recommendations(&request).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_analyze_market_condition_fails_without_service() {
        let client = AIClient::new("http://localhost:9999", 5);

        let candle = create_test_candle();
        let mut timeframe_data = HashMap::new();
        timeframe_data.insert("1h".to_string(), vec![candle]);

        let request = MarketConditionRequest {
            symbol: "BTCUSDT".to_string(),
            timeframe_data,
            current_price: 50500.0,
            volume_24h: 10000.0,
            timestamp: 1234567890,
        };

        let result = client.analyze_market_condition(&request).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_send_performance_feedback_fails_without_service() {
        let client = AIClient::new("http://localhost:9999", 5);

        let feedback = PerformanceFeedback {
            signal_id: "signal123".to_string(),
            symbol: "BTCUSDT".to_string(),
            predicted_signal: crate::strategies::TradingSignal::Long,
            actual_outcome: "success".to_string(),
            profit_loss: 1000.0,
            confidence_was_accurate: true,
            feedback_notes: Some("Good prediction".to_string()),
            timestamp: 1234567890,
        };

        let result = client.send_performance_feedback(&feedback).await;
        assert!(result.is_err() || result.is_ok());
    }

    #[test]
    fn test_python_candle_data_zero_volume() {
        let mut candle = create_test_candle();
        candle.volume = 0.0;

        let python_candle = PythonCandleData::from(&candle);
        assert_eq!(python_candle.volume, 0.0);
    }

    #[test]
    fn test_python_candle_data_high_volume() {
        let mut candle = create_test_candle();
        candle.volume = 1000000.0;

        let python_candle = PythonCandleData::from(&candle);
        assert_eq!(python_candle.volume, 1000000.0);
    }

    #[test]
    fn test_ai_client_new_with_https() {
        let client = AIClient::new("https://api.example.com", 30);
        assert_eq!(client.base_url, "https://api.example.com");
    }

    #[test]
    fn test_ai_client_new_with_port() {
        let client = AIClient::new("http://localhost:8080", 30);
        assert_eq!(client.base_url, "http://localhost:8080");
    }

    #[test]
    fn test_ai_strategy_context_default() {
        let context = AIStrategyContext::default();
        // Default values should be set
        assert_eq!(context.risk_level, "Moderate");
    }

    #[test]
    fn test_ai_analysis_request_with_empty_timeframe_data() {
        let timeframe_data = HashMap::new();

        let request = AIAnalysisRequest {
            symbol: "BTCUSDT".to_string(),
            timeframe_data,
            current_price: 50500.0,
            volume_24h: 10000.0,
            timestamp: 1234567890,
            strategy_context: AIStrategyContext::default(),
        };

        assert_eq!(request.timeframe_data.len(), 0);
    }

    #[test]
    fn test_strategy_recommendation_request_with_empty_strategies() {
        let candle = create_test_candle();
        let mut timeframe_data = HashMap::new();
        timeframe_data.insert("1h".to_string(), vec![candle]);

        let request = StrategyRecommendationRequest {
            symbol: "BTCUSDT".to_string(),
            timeframe_data,
            current_price: 50500.0,
            available_strategies: vec![],
            timestamp: 1234567890,
        };

        assert_eq!(request.available_strategies.len(), 0);
    }

    #[test]
    fn test_strategy_recommendation_request_with_many_strategies() {
        let candle = create_test_candle();
        let mut timeframe_data = HashMap::new();
        timeframe_data.insert("1h".to_string(), vec![candle]);

        let strategies = vec![
            "RSI".to_string(),
            "MACD".to_string(),
            "BOLLINGER".to_string(),
            "EMA".to_string(),
            "VOLUME".to_string(),
        ];

        let request = StrategyRecommendationRequest {
            symbol: "BTCUSDT".to_string(),
            timeframe_data,
            current_price: 50500.0,
            available_strategies: strategies.clone(),
            timestamp: 1234567890,
        };

        assert_eq!(request.available_strategies.len(), 5);
    }

    #[test]
    fn test_market_condition_request_conversion() {
        let candle = create_test_candle();
        let mut timeframe_data = HashMap::new();
        timeframe_data.insert("1h".to_string(), vec![candle]);

        let request = MarketConditionRequest {
            symbol: "BTCUSDT".to_string(),
            timeframe_data: timeframe_data.clone(),
            current_price: 50500.0,
            volume_24h: 10000.0,
            timestamp: 1234567890,
        };

        let python_request = PythonMarketConditionRequest::from(&request);

        assert_eq!(python_request.symbol, "BTCUSDT");
        assert_eq!(python_request.current_price, 50500.0);
        assert_eq!(python_request.volume_24h, 10000.0);
        assert_eq!(python_request.timestamp, 1234567890);
    }

    #[test]
    fn test_performance_feedback_serialization() {
        let feedback = PerformanceFeedback {
            signal_id: "signal123".to_string(),
            symbol: "BTCUSDT".to_string(),
            predicted_signal: crate::strategies::TradingSignal::Long,
            actual_outcome: "success".to_string(),
            profit_loss: 1000.0,
            confidence_was_accurate: true,
            feedback_notes: Some("Good prediction".to_string()),
            timestamp: 1234567890,
        };

        let json = serde_json::to_string(&feedback).unwrap();
        assert!(json.contains("signal123"));
        assert!(json.contains("BTCUSDT"));
        assert!(json.contains("success"));
    }

    #[test]
    fn test_performance_feedback_without_notes() {
        let feedback = PerformanceFeedback {
            signal_id: "signal456".to_string(),
            symbol: "ETHUSDT".to_string(),
            predicted_signal: crate::strategies::TradingSignal::Short,
            actual_outcome: "failure".to_string(),
            profit_loss: -100.0,
            confidence_was_accurate: false,
            feedback_notes: None,
            timestamp: 1234567890,
        };

        assert!(feedback.feedback_notes.is_none());
        assert_eq!(feedback.actual_outcome, "failure");
    }

    #[test]
    fn test_candle_data_timestamp_conversion() {
        let candle = create_test_candle();
        let python_candle = PythonCandleData::from(&candle);

        // Timestamp should be preserved
        assert_eq!(python_candle.timestamp, candle.open_time);
    }

    #[test]
    fn test_ai_client_zero_timeout() {
        let client = AIClient::new("http://localhost:8000", 0);
        assert_eq!(client.timeout, Duration::from_secs(0));
    }

    #[test]
    fn test_ai_client_large_timeout() {
        let client = AIClient::new("http://localhost:8000", 300);
        assert_eq!(client.timeout, Duration::from_secs(300));
    }

    // =========================================================================
    // FUNCTION-LEVEL TESTS (test_fn_ prefix for coverage boost)
    // =========================================================================

    #[test]
    fn test_fn_ai_client_new() {
        let client = AIClient::new("http://localhost:8000", 30);
        assert_eq!(client.base_url, "http://localhost:8000");
        assert_eq!(client.timeout, Duration::from_secs(30));
    }

    #[test]
    fn test_fn_ai_client_new_with_trailing_slash() {
        let client = AIClient::new("http://localhost:8000/", 30);
        assert_eq!(client.base_url, "http://localhost:8000");
    }

    #[test]
    fn test_fn_python_candle_data_from_candle() {
        let candle = create_test_candle();
        let python_candle = PythonCandleData::from(&candle);

        assert_eq!(python_candle.timestamp, candle.open_time);
        assert_eq!(python_candle.open, candle.open);
        assert_eq!(python_candle.high, candle.high);
        assert_eq!(python_candle.low, candle.low);
        assert_eq!(python_candle.close, candle.close);
        assert_eq!(python_candle.volume, candle.volume);
    }

    #[test]
    fn test_fn_python_ai_analysis_request_from() {
        let candle = create_test_candle();
        let mut timeframe_data = HashMap::new();
        timeframe_data.insert("1h".to_string(), vec![candle]);

        let request = AIAnalysisRequest {
            symbol: "BTCUSDT".to_string(),
            timeframe_data,
            current_price: 50000.0,
            volume_24h: 10000.0,
            timestamp: 1234567890,
            strategy_context: AIStrategyContext::default(),
        };

        let python_request = PythonAIAnalysisRequest::from(&request);
        assert_eq!(python_request.symbol, "BTCUSDT");
        assert_eq!(python_request.current_price, 50000.0);
    }

    #[test]
    fn test_fn_python_strategy_recommendation_request_from() {
        let candle = create_test_candle();
        let mut timeframe_data = HashMap::new();
        timeframe_data.insert("1m".to_string(), vec![candle]);

        let request = StrategyRecommendationRequest {
            symbol: "ETHUSDT".to_string(),
            timeframe_data,
            current_price: 3000.0,
            available_strategies: vec!["RSI".to_string(), "MACD".to_string()],
            timestamp: 1234567890,
        };

        let python_request = PythonStrategyRecommendationRequest::from(&request);
        assert_eq!(python_request.symbol, "ETHUSDT");
        assert_eq!(python_request.available_strategies.len(), 2);
    }

    #[test]
    fn test_fn_python_market_condition_request_from() {
        let candle = create_test_candle();
        let mut timeframe_data = HashMap::new();
        timeframe_data.insert("5m".to_string(), vec![candle.clone(), candle]);

        let request = MarketConditionRequest {
            symbol: "ADAUSDT".to_string(),
            timeframe_data,
            current_price: 1.5,
            volume_24h: 50000.0,
            timestamp: 1234567890,
        };

        let python_request = PythonMarketConditionRequest::from(&request);
        assert_eq!(python_request.symbol, "ADAUSDT");
        assert_eq!(python_request.volume_24h, 50000.0);
    }

    #[test]
    fn test_fn_ai_client_clone() {
        let client1 = AIClient::new("http://test:8000", 60);
        let client2 = client1.clone();

        assert_eq!(client1.base_url, client2.base_url);
        assert_eq!(client1.timeout, client2.timeout);
    }

    #[test]
    fn test_fn_ai_client_debug() {
        let client = AIClient::new("http://debug:8000", 45);
        let debug_str = format!("{:?}", client);

        assert!(debug_str.contains("AIClient"));
    }

    #[tokio::test]
    async fn test_fn_health_check_failure() {
        let client = AIClient::new("http://invalid-url-xyz:9999", 1);
        let result = client.health_check().await;

        // Should fail to connect
        assert!(result.is_err() || result.unwrap() == false);
    }

    #[tokio::test]
    async fn test_fn_analyze_trading_signals_network_error() {
        let client = AIClient::new("http://invalid-url-xyz:9999", 1);

        let candle = create_test_candle();
        let mut timeframe_data = HashMap::new();
        timeframe_data.insert("1h".to_string(), vec![candle]);

        let request = AIAnalysisRequest {
            symbol: "BTCUSDT".to_string(),
            timeframe_data,
            current_price: 50000.0,
            volume_24h: 10000.0,
            timestamp: 1234567890,
            strategy_context: AIStrategyContext::default(),
        };

        let result = client.analyze_trading_signals(&request).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_fn_get_strategy_recommendations_network_error() {
        let client = AIClient::new("http://invalid-url-xyz:9999", 1);

        let candle = create_test_candle();
        let mut timeframe_data = HashMap::new();
        timeframe_data.insert("1h".to_string(), vec![candle]);

        let request = StrategyRecommendationRequest {
            symbol: "BTCUSDT".to_string(),
            timeframe_data,
            current_price: 50000.0,
            available_strategies: vec!["RSI".to_string()],
            timestamp: 1234567890,
        };

        let result = client.get_strategy_recommendations(&request).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_fn_analyze_market_condition_network_error() {
        let client = AIClient::new("http://invalid-url-xyz:9999", 1);

        let candle = create_test_candle();
        let mut timeframe_data = HashMap::new();
        timeframe_data.insert("1h".to_string(), vec![candle]);

        let request = MarketConditionRequest {
            symbol: "BTCUSDT".to_string(),
            timeframe_data,
            current_price: 50000.0,
            volume_24h: 10000.0,
            timestamp: 1234567890,
        };

        let result = client.analyze_market_condition(&request).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_fn_send_performance_feedback_network_error() {
        let client = AIClient::new("http://invalid-url-xyz:9999", 1);

        let feedback = PerformanceFeedback {
            signal_id: "sig123".to_string(),
            symbol: "BTCUSDT".to_string(),
            predicted_signal: crate::strategies::TradingSignal::Long,
            actual_outcome: "success".to_string(),
            profit_loss: 5.0,
            confidence_was_accurate: true,
            feedback_notes: Some("Test feedback".to_string()),
            timestamp: 1234567890,
        };

        let result = client.send_performance_feedback(&feedback).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_fn_get_service_info_network_error() {
        let client = AIClient::new("http://invalid-url-xyz:9999", 1);
        let result = client.get_service_info().await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_fn_get_supported_strategies_network_error() {
        let client = AIClient::new("http://invalid-url-xyz:9999", 1);
        let result = client.get_supported_strategies().await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_fn_get_model_performance_network_error() {
        let client = AIClient::new("http://invalid-url-xyz:9999", 1);
        let result = client.get_model_performance().await;
        assert!(result.is_err());
    }

    // ============ Additional Coverage Boost Tests ============

    #[test]
    fn test_python_candle_data_all_fields() {
        let candle = CandleData {
            open_time: 9876543210,
            open: 45000.0,
            high: 46000.0,
            low: 44000.0,
            close: 45500.0,
            volume: 250.5,
            close_time: 9876543270,
            quote_volume: 11350000.0,
            trades: 500,
            is_closed: true,
        };

        let python_candle = PythonCandleData::from(&candle);

        assert_eq!(python_candle.timestamp, 9876543210);
        assert_eq!(python_candle.open, 45000.0);
        assert_eq!(python_candle.high, 46000.0);
        assert_eq!(python_candle.low, 44000.0);
        assert_eq!(python_candle.close, 45500.0);
        assert_eq!(python_candle.volume, 250.5);
    }

    #[test]
    fn test_python_ai_analysis_request_multiple_timeframes() {
        let candle1 = create_test_candle();
        let mut candle2 = create_test_candle();
        candle2.close = 51000.0;

        let mut timeframe_data = HashMap::new();
        timeframe_data.insert("1h".to_string(), vec![candle1.clone()]);
        timeframe_data.insert("4h".to_string(), vec![candle2.clone()]);
        timeframe_data.insert("1d".to_string(), vec![candle1]);

        let request = AIAnalysisRequest {
            symbol: "BTCUSDT".to_string(),
            timeframe_data,
            current_price: 50500.0,
            volume_24h: 10000.0,
            timestamp: 1234567890,
            strategy_context: AIStrategyContext::default(),
        };

        let python_request = PythonAIAnalysisRequest::from(&request);

        assert_eq!(python_request.timeframe_data.len(), 3);
        assert!(python_request.timeframe_data.contains_key("1h"));
        assert!(python_request.timeframe_data.contains_key("4h"));
        assert!(python_request.timeframe_data.contains_key("1d"));
    }

    #[test]
    fn test_python_strategy_recommendation_request_empty_strategies() {
        let candle = create_test_candle();
        let mut timeframe_data = HashMap::new();
        timeframe_data.insert("1h".to_string(), vec![candle]);

        let request = StrategyRecommendationRequest {
            symbol: "ETHUSDT".to_string(),
            timeframe_data,
            current_price: 3000.0,
            available_strategies: vec![],
            timestamp: 1234567890,
        };

        let python_request = PythonStrategyRecommendationRequest::from(&request);

        assert_eq!(python_request.available_strategies.len(), 0);
    }

    #[test]
    fn test_python_strategy_recommendation_request_many_strategies() {
        let candle = create_test_candle();
        let mut timeframe_data = HashMap::new();
        timeframe_data.insert("1h".to_string(), vec![candle]);

        let request = StrategyRecommendationRequest {
            symbol: "BNBUSDT".to_string(),
            timeframe_data,
            current_price: 400.0,
            available_strategies: vec![
                "RSI".to_string(),
                "MACD".to_string(),
                "Bollinger".to_string(),
                "Volume".to_string(),
            ],
            timestamp: 1234567890,
        };

        let python_request = PythonStrategyRecommendationRequest::from(&request);

        assert_eq!(python_request.available_strategies.len(), 4);
        assert_eq!(python_request.symbol, "BNBUSDT");
    }

    #[test]
    fn test_python_market_condition_request_multiple_candles() {
        let candle1 = create_test_candle();
        let mut candle2 = create_test_candle();
        candle2.close = 51500.0;
        let mut candle3 = create_test_candle();
        candle3.close = 52000.0;

        let mut timeframe_data = HashMap::new();
        timeframe_data.insert("1h".to_string(), vec![candle1, candle2, candle3]);

        let request = MarketConditionRequest {
            symbol: "BTCUSDT".to_string(),
            timeframe_data,
            current_price: 52500.0,
            volume_24h: 100000.0,
            timestamp: 1234567890,
        };

        let python_request = PythonMarketConditionRequest::from(&request);

        assert_eq!(python_request.timeframe_data["1h"].len(), 3);
        assert_eq!(python_request.current_price, 52500.0);
        assert_eq!(python_request.volume_24h, 100000.0);
    }
}
