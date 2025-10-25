// Comprehensive unit tests for AI modules
// Target: 90%+ coverage for client.rs and mod.rs
//
// Test Coverage:
// 1. AI client initialization and configuration
// 2. API communication with Python AI service
// 3. Request/response handling and serialization
// 4. Signal parsing and validation
// 5. Error handling and retries
// 6. Timeout handling
// 7. Health checks and service info

use anyhow::Result;
use binance_trading_bot::ai::*;
use binance_trading_bot::market_data::cache::CandleData;
use binance_trading_bot::strategies::{StrategyInput, TradingSignal};
use serde_json::json;
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::Mutex;
use warp::Filter;

// ============================================================================
// Mock HTTP Server Setup
// ============================================================================

/// Mock HTTP server that simulates the Python AI service
struct MockAIServer {
    port: u16,
    response_handler: Arc<Mutex<Box<dyn ResponseHandler + Send>>>,
}

trait ResponseHandler {
    fn handle(&self, path: &str, body: Option<serde_json::Value>) -> MockResponse;
}

struct MockResponse {
    status: u16,
    body: serde_json::Value,
    delay: Option<Duration>,
}

impl MockResponse {
    fn success(body: serde_json::Value) -> Self {
        Self {
            status: 200,
            body,
            delay: None,
        }
    }

    fn error(status: u16, message: &str) -> Self {
        Self {
            status,
            body: json!({ "error": message }),
            delay: None,
        }
    }

    fn with_delay(mut self, delay: Duration) -> Self {
        self.delay = Some(delay);
        self
    }
}

impl MockAIServer {
    async fn start(port: u16, handler: Box<dyn ResponseHandler + Send>) -> Result<Self> {
        let response_handler = Arc::new(Mutex::new(handler));
        let handler_clone = response_handler.clone();

        let routes = warp::any()
            .and(warp::path::full())
            .and(
                warp::body::json()
                    .or(warp::any().map(|| json!(null)))
                    .unify(),
            )
            .then(move |path: warp::path::FullPath, body: serde_json::Value| {
                let handler = handler_clone.clone();
                async move {
                    let handler = handler.lock().await;
                    let response = handler.handle(
                        path.as_str(),
                        if body.is_null() { None } else { Some(body) },
                    );

                    if let Some(delay) = response.delay {
                        tokio::time::sleep(delay).await;
                    }

                    warp::reply::with_status(
                        warp::reply::json(&response.body),
                        warp::http::StatusCode::from_u16(response.status).unwrap(),
                    )
                }
            });

        tokio::spawn(async move {
            warp::serve(routes).run(([127, 0, 0, 1], port)).await;
        });

        // Give server time to start
        tokio::time::sleep(Duration::from_millis(100)).await;

        Ok(Self {
            port,
            response_handler,
        })
    }

    fn base_url(&self) -> String {
        format!("http://127.0.0.1:{}", self.port)
    }

    async fn set_handler(&self, handler: Box<dyn ResponseHandler + Send>) {
        *self.response_handler.lock().await = handler;
    }
}

// ============================================================================
// Helper Functions and Fixtures
// ============================================================================

fn create_test_candle_data(timestamp: i64, close_price: f64) -> CandleData {
    CandleData {
        open_time: timestamp,
        close_time: timestamp + 60000,
        open: close_price - 10.0,
        high: close_price + 20.0,
        low: close_price - 30.0,
        close: close_price,
        volume: 1000.0,
        quote_volume: close_price * 1000.0,
        trades: 100,
        is_closed: true,
    }
}

fn create_test_strategy_input() -> StrategyInput {
    let mut timeframe_data = HashMap::new();
    let candles = vec![
        create_test_candle_data(1700000000000, 45000.0),
        create_test_candle_data(1700000060000, 45100.0),
        create_test_candle_data(1700000120000, 45200.0),
    ];
    timeframe_data.insert("1m".to_string(), candles);

    StrategyInput {
        symbol: "BTCUSDT".to_string(),
        timeframe_data,
        current_price: 45200.0,
        volume_24h: 1000000.0,
        timestamp: 1700000120000,
    }
}

fn create_test_ai_signal_response() -> serde_json::Value {
    json!({
        "signal": "Long",
        "confidence": 0.85,
        "reasoning": "Strong bullish momentum with RSI oversold and MACD crossover",
        "strategy_scores": {
            "RSI Strategy": 0.9,
            "MACD Strategy": 0.8
        },
        "market_analysis": {
            "trend_direction": "bullish",
            "trend_strength": 0.85,
            "support_levels": [44000.0, 43500.0],
            "resistance_levels": [46000.0, 47000.0],
            "volatility_level": "moderate",
            "volume_analysis": "increasing"
        },
        "risk_assessment": {
            "overall_risk": "moderate",
            "technical_risk": 0.4,
            "market_risk": 0.35,
            "recommended_position_size": 0.02,
            "stop_loss_suggestion": 44500.0,
            "take_profit_suggestion": 46500.0
        },
        "timestamp": 1700000120000i64
    })
}

fn create_test_strategy_recommendations() -> serde_json::Value {
    json!([
        {
            "strategy_name": "RSI Strategy",
            "suitability_score": 0.9,
            "reasoning": "Market shows oversold conditions ideal for RSI strategy",
            "recommended_config": {
                "rsi_period": 14,
                "oversold": 30,
                "overbought": 70
            }
        },
        {
            "strategy_name": "MACD Strategy",
            "suitability_score": 0.75,
            "reasoning": "Recent MACD crossover indicates potential trend",
            "recommended_config": {
                "fast_period": 12,
                "slow_period": 26,
                "signal_period": 9
            }
        }
    ])
}

fn create_test_market_condition() -> serde_json::Value {
    json!({
        "condition_type": "trending_bullish",
        "confidence": 0.8,
        "characteristics": ["strong_momentum", "high_volume", "breakout"],
        "recommended_strategies": ["Trend Following", "Breakout Strategy"],
        "market_phase": "expansion"
    })
}

// ============================================================================
// Default Response Handler for Success Cases
// ============================================================================

struct DefaultResponseHandler;

impl ResponseHandler for DefaultResponseHandler {
    fn handle(&self, path: &str, _body: Option<serde_json::Value>) -> MockResponse {
        match path {
            "/health" => MockResponse::success(json!({ "status": "healthy" })),
            "/ai/analyze" => MockResponse::success(create_test_ai_signal_response()),
            "/ai/strategy-recommendations" => {
                MockResponse::success(create_test_strategy_recommendations())
            },
            "/ai/market-condition" => MockResponse::success(create_test_market_condition()),
            "/ai/feedback" => MockResponse::success(json!({ "status": "received" })),
            "/ai/info" => MockResponse::success(json!({
                "service_name": "Python AI Service",
                "version": "1.0.0",
                "model_version": "v1.2.3",
                "supported_timeframes": ["1m", "5m", "15m", "1h", "4h", "1d"],
                "supported_symbols": ["BTCUSDT", "ETHUSDT", "BNBUSDT"],
                "capabilities": ["technical_analysis", "ml_prediction", "risk_assessment"],
                "last_trained": "2024-01-01T00:00:00Z"
            })),
            "/ai/strategies" => MockResponse::success(json!({
                "strategies": ["RSI Strategy", "MACD Strategy", "Bollinger Bands", "Volume Analysis"]
            })),
            "/ai/performance" => MockResponse::success(json!({
                "overall_accuracy": 0.78,
                "precision": 0.82,
                "recall": 0.75,
                "f1_score": 0.78,
                "predictions_made": 10000,
                "successful_predictions": 7800,
                "average_confidence": 0.72,
                "model_uptime": "30 days",
                "last_updated": "2024-01-15T10:00:00Z"
            })),
            _ => MockResponse::error(404, "Not found"),
        }
    }
}

// ============================================================================
// Tests: AIClient Initialization and Configuration
// ============================================================================

#[tokio::test]
async fn test_ai_client_creation() {
    let client = AIClient::new("http://localhost:8000", 30);
    // Client should be created successfully
    // We can't directly test internal fields, but we can test that it doesn't panic
    assert!(format!("{:?}", client).contains("AIClient"));
}

#[tokio::test]
async fn test_ai_client_trims_trailing_slash() {
    let server = MockAIServer::start(8091, Box::new(DefaultResponseHandler))
        .await
        .unwrap();

    let client1 = AIClient::new(&format!("{}/", server.base_url()), 30);
    let client2 = AIClient::new(&server.base_url(), 30);

    // Both should work identically
    let result1 = client1.health_check().await;
    let result2 = client2.health_check().await;

    assert!(result1.is_ok());
    assert!(result2.is_ok());
}

#[tokio::test]
async fn test_ai_service_creation_with_config() {
    let config = AIServiceConfig {
        python_service_url: "http://localhost:8000".to_string(),
        request_timeout_seconds: 30,
        max_retries: 3,
        enable_caching: true,
        cache_ttl_seconds: 300,
    };

    let service = AIService::new(config.clone());
    assert!(format!("{:?}", service).contains("AIService"));
}

#[tokio::test]
async fn test_ai_service_config_default() {
    let config = AIServiceConfig::default();
    assert_eq!(config.python_service_url, "http://localhost:8000");
    assert_eq!(config.request_timeout_seconds, 30);
    assert_eq!(config.max_retries, 3);
    assert!(config.enable_caching);
    assert_eq!(config.cache_ttl_seconds, 300);
}

// ============================================================================
// Tests: Health Check and Service Info
// ============================================================================

#[tokio::test]
async fn test_health_check_success() {
    let server = MockAIServer::start(8092, Box::new(DefaultResponseHandler))
        .await
        .unwrap();

    let client = AIClient::new(&server.base_url(), 5);
    let result = client.health_check().await;

    assert!(result.is_ok());
    assert!(result.unwrap());
}

#[tokio::test]
async fn test_health_check_failure() {
    struct HealthCheckFailureHandler;
    impl ResponseHandler for HealthCheckFailureHandler {
        fn handle(&self, path: &str, _body: Option<serde_json::Value>) -> MockResponse {
            if path == "/health" {
                MockResponse::error(503, "Service unavailable")
            } else {
                DefaultResponseHandler.handle(path, _body)
            }
        }
    }

    let server = MockAIServer::start(8093, Box::new(HealthCheckFailureHandler))
        .await
        .unwrap();

    let client = AIClient::new(&server.base_url(), 5);
    let result = client.health_check().await;

    assert!(result.is_ok());
    assert!(!result.unwrap()); // Should return false for non-success status
}

#[tokio::test]
async fn test_health_check_network_error() {
    // Use a port that's not listening
    let client = AIClient::new("http://127.0.0.1:9999", 1);
    let result = client.health_check().await;

    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("health check"));
}

#[tokio::test]
async fn test_get_service_info_success() {
    let server = MockAIServer::start(8094, Box::new(DefaultResponseHandler))
        .await
        .unwrap();

    let client = AIClient::new(&server.base_url(), 5);
    let result = client.get_service_info().await;

    assert!(result.is_ok());
    let info = result.unwrap();
    assert_eq!(info.service_name, "Python AI Service");
    assert_eq!(info.version, "1.0.0");
    assert_eq!(info.model_version, "v1.2.3");
    assert_eq!(info.supported_timeframes.len(), 6);
    assert_eq!(info.supported_symbols.len(), 3);
    assert!(info.supported_symbols.contains(&"BTCUSDT".to_string()));
}

#[tokio::test]
async fn test_get_service_info_error_response() {
    struct ServiceInfoErrorHandler;
    impl ResponseHandler for ServiceInfoErrorHandler {
        fn handle(&self, path: &str, _body: Option<serde_json::Value>) -> MockResponse {
            if path == "/ai/info" {
                MockResponse::error(500, "Internal server error")
            } else {
                DefaultResponseHandler.handle(path, _body)
            }
        }
    }

    let server = MockAIServer::start(8095, Box::new(ServiceInfoErrorHandler))
        .await
        .unwrap();

    let client = AIClient::new(&server.base_url(), 5);
    let result = client.get_service_info().await;

    assert!(result.is_err());
    let error = result.unwrap_err().to_string();
    assert!(error.contains("Service info request failed"));
    assert!(error.contains("500"));
}

#[tokio::test]
async fn test_get_supported_strategies_success() {
    let server = MockAIServer::start(8096, Box::new(DefaultResponseHandler))
        .await
        .unwrap();

    let client = AIClient::new(&server.base_url(), 5);
    let result = client.get_supported_strategies().await;

    assert!(result.is_ok());
    let strategies = result.unwrap();
    assert_eq!(strategies.strategies.len(), 4);
    assert!(strategies.strategies.contains(&"RSI Strategy".to_string()));
    assert!(strategies.strategies.contains(&"MACD Strategy".to_string()));
}

// ============================================================================
// Tests: AI Analysis Request/Response
// ============================================================================

#[tokio::test]
async fn test_analyze_trading_signals_success() {
    let server = MockAIServer::start(8097, Box::new(DefaultResponseHandler))
        .await
        .unwrap();

    let client = AIClient::new(&server.base_url(), 5);
    let strategy_input = create_test_strategy_input();

    let request = AIAnalysisRequest {
        symbol: strategy_input.symbol.clone(),
        timeframe_data: strategy_input.timeframe_data.clone(),
        current_price: strategy_input.current_price,
        volume_24h: strategy_input.volume_24h,
        timestamp: strategy_input.timestamp,
        strategy_context: AIStrategyContext::default(),
    };

    let result = client.analyze_trading_signals(&request).await;

    assert!(result.is_ok());
    let response = result.unwrap();
    assert_eq!(response.signal, TradingSignal::Long);
    assert_eq!(response.confidence, 0.85);
    assert!(response.reasoning.contains("bullish"));
    assert!(response.strategy_scores.contains_key("RSI Strategy"));
    assert_eq!(response.market_analysis.trend_direction, "bullish");
    assert_eq!(response.risk_assessment.overall_risk, "moderate");
}

#[tokio::test]
async fn test_analyze_trading_signals_with_context() {
    let server = MockAIServer::start(8098, Box::new(DefaultResponseHandler))
        .await
        .unwrap();

    let client = AIClient::new(&server.base_url(), 5);
    let strategy_input = create_test_strategy_input();

    let mut context = AIStrategyContext::default();
    context.selected_strategies = vec!["RSI Strategy".to_string()];
    context.market_condition = "bullish".to_string();
    context.risk_level = "low".to_string();

    let request = AIAnalysisRequest {
        symbol: strategy_input.symbol.clone(),
        timeframe_data: strategy_input.timeframe_data.clone(),
        current_price: strategy_input.current_price,
        volume_24h: strategy_input.volume_24h,
        timestamp: strategy_input.timestamp,
        strategy_context: context,
    };

    let result = client.analyze_trading_signals(&request).await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_analyze_trading_signals_error_response() {
    struct AnalysisErrorHandler;
    impl ResponseHandler for AnalysisErrorHandler {
        fn handle(&self, path: &str, _body: Option<serde_json::Value>) -> MockResponse {
            if path == "/ai/analyze" {
                MockResponse::error(400, "Invalid request data")
            } else {
                DefaultResponseHandler.handle(path, _body)
            }
        }
    }

    let server = MockAIServer::start(8099, Box::new(AnalysisErrorHandler))
        .await
        .unwrap();

    let client = AIClient::new(&server.base_url(), 5);
    let strategy_input = create_test_strategy_input();

    let request = AIAnalysisRequest {
        symbol: strategy_input.symbol.clone(),
        timeframe_data: strategy_input.timeframe_data.clone(),
        current_price: strategy_input.current_price,
        volume_24h: strategy_input.volume_24h,
        timestamp: strategy_input.timestamp,
        strategy_context: AIStrategyContext::default(),
    };

    let result = client.analyze_trading_signals(&request).await;

    assert!(result.is_err());
    let error = result.unwrap_err().to_string();
    assert!(error.contains("AI analysis request failed"));
    assert!(error.contains("400"));
}

#[tokio::test]
async fn test_analyze_trading_signals_invalid_json() {
    struct InvalidJsonHandler;
    impl ResponseHandler for InvalidJsonHandler {
        fn handle(&self, path: &str, _body: Option<serde_json::Value>) -> MockResponse {
            if path == "/ai/analyze" {
                MockResponse {
                    status: 200,
                    body: json!({ "invalid": "response" }),
                    delay: None,
                }
            } else {
                DefaultResponseHandler.handle(path, _body)
            }
        }
    }

    let server = MockAIServer::start(8100, Box::new(InvalidJsonHandler))
        .await
        .unwrap();

    let client = AIClient::new(&server.base_url(), 5);
    let strategy_input = create_test_strategy_input();

    let request = AIAnalysisRequest {
        symbol: strategy_input.symbol.clone(),
        timeframe_data: strategy_input.timeframe_data.clone(),
        current_price: strategy_input.current_price,
        volume_24h: strategy_input.volume_24h,
        timestamp: strategy_input.timestamp,
        strategy_context: AIStrategyContext::default(),
    };

    let result = client.analyze_trading_signals(&request).await;

    assert!(result.is_err());
    let error = result.unwrap_err().to_string();
    assert!(error.contains("Failed to parse AI analysis response"));
}

// ============================================================================
// Tests: Strategy Recommendations
// ============================================================================

#[tokio::test]
async fn test_get_strategy_recommendations_success() {
    let server = MockAIServer::start(8101, Box::new(DefaultResponseHandler))
        .await
        .unwrap();

    let client = AIClient::new(&server.base_url(), 5);
    let strategy_input = create_test_strategy_input();

    let request = StrategyRecommendationRequest {
        symbol: strategy_input.symbol.clone(),
        timeframe_data: strategy_input.timeframe_data.clone(),
        current_price: strategy_input.current_price,
        available_strategies: vec!["RSI Strategy".to_string(), "MACD Strategy".to_string()],
        timestamp: strategy_input.timestamp,
    };

    let result = client.get_strategy_recommendations(&request).await;

    assert!(result.is_ok());
    let recommendations = result.unwrap();
    assert_eq!(recommendations.len(), 2);
    assert_eq!(recommendations[0].strategy_name, "RSI Strategy");
    assert_eq!(recommendations[0].suitability_score, 0.9);
    assert!(recommendations[0].reasoning.contains("oversold"));
    assert!(recommendations[0]
        .recommended_config
        .contains_key("rsi_period"));
}

#[tokio::test]
async fn test_get_strategy_recommendations_error() {
    struct RecommendationErrorHandler;
    impl ResponseHandler for RecommendationErrorHandler {
        fn handle(&self, path: &str, _body: Option<serde_json::Value>) -> MockResponse {
            if path == "/ai/strategy-recommendations" {
                MockResponse::error(500, "Analysis failed")
            } else {
                DefaultResponseHandler.handle(path, _body)
            }
        }
    }

    let server = MockAIServer::start(8102, Box::new(RecommendationErrorHandler))
        .await
        .unwrap();

    let client = AIClient::new(&server.base_url(), 5);
    let strategy_input = create_test_strategy_input();

    let request = StrategyRecommendationRequest {
        symbol: strategy_input.symbol.clone(),
        timeframe_data: strategy_input.timeframe_data.clone(),
        current_price: strategy_input.current_price,
        available_strategies: vec!["RSI Strategy".to_string()],
        timestamp: strategy_input.timestamp,
    };

    let result = client.get_strategy_recommendations(&request).await;

    assert!(result.is_err());
    assert!(result
        .unwrap_err()
        .to_string()
        .contains("Strategy recommendation request failed"));
}

// ============================================================================
// Tests: Market Condition Analysis
// ============================================================================

#[tokio::test]
async fn test_analyze_market_condition_success() {
    let server = MockAIServer::start(8103, Box::new(DefaultResponseHandler))
        .await
        .unwrap();

    let client = AIClient::new(&server.base_url(), 5);
    let strategy_input = create_test_strategy_input();

    let request = MarketConditionRequest {
        symbol: strategy_input.symbol.clone(),
        timeframe_data: strategy_input.timeframe_data.clone(),
        current_price: strategy_input.current_price,
        volume_24h: strategy_input.volume_24h,
        timestamp: strategy_input.timestamp,
    };

    let result = client.analyze_market_condition(&request).await;

    assert!(result.is_ok());
    let analysis = result.unwrap();
    assert_eq!(analysis.condition_type, "trending_bullish");
    assert_eq!(analysis.confidence, 0.8);
    assert_eq!(analysis.characteristics.len(), 3);
    assert!(analysis
        .characteristics
        .contains(&"strong_momentum".to_string()));
    assert_eq!(analysis.market_phase, "expansion");
}

#[tokio::test]
async fn test_analyze_market_condition_error() {
    struct MarketConditionErrorHandler;
    impl ResponseHandler for MarketConditionErrorHandler {
        fn handle(&self, path: &str, _body: Option<serde_json::Value>) -> MockResponse {
            if path == "/ai/market-condition" {
                MockResponse::error(503, "Service temporarily unavailable")
            } else {
                DefaultResponseHandler.handle(path, _body)
            }
        }
    }

    let server = MockAIServer::start(8104, Box::new(MarketConditionErrorHandler))
        .await
        .unwrap();

    let client = AIClient::new(&server.base_url(), 5);
    let strategy_input = create_test_strategy_input();

    let request = MarketConditionRequest {
        symbol: strategy_input.symbol.clone(),
        timeframe_data: strategy_input.timeframe_data.clone(),
        current_price: strategy_input.current_price,
        volume_24h: strategy_input.volume_24h,
        timestamp: strategy_input.timestamp,
    };

    let result = client.analyze_market_condition(&request).await;

    assert!(result.is_err());
    let error = result.unwrap_err().to_string();
    assert!(error.contains("Market condition request failed"));
    assert!(error.contains("503"));
}

// ============================================================================
// Tests: Performance Feedback
// ============================================================================

#[tokio::test]
async fn test_send_performance_feedback_success() {
    let server = MockAIServer::start(8105, Box::new(DefaultResponseHandler))
        .await
        .unwrap();

    let client = AIClient::new(&server.base_url(), 5);

    let feedback = PerformanceFeedback {
        signal_id: "sig_123".to_string(),
        symbol: "BTCUSDT".to_string(),
        predicted_signal: TradingSignal::Long,
        actual_outcome: "success".to_string(),
        profit_loss: 250.50,
        confidence_was_accurate: true,
        feedback_notes: Some("Great prediction".to_string()),
        timestamp: 1700000120000,
    };

    let result = client.send_performance_feedback(&feedback).await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_send_performance_feedback_error() {
    struct FeedbackErrorHandler;
    impl ResponseHandler for FeedbackErrorHandler {
        fn handle(&self, path: &str, _body: Option<serde_json::Value>) -> MockResponse {
            if path == "/ai/feedback" {
                MockResponse::error(400, "Invalid feedback data")
            } else {
                DefaultResponseHandler.handle(path, _body)
            }
        }
    }

    let server = MockAIServer::start(8106, Box::new(FeedbackErrorHandler))
        .await
        .unwrap();

    let client = AIClient::new(&server.base_url(), 5);

    let feedback = PerformanceFeedback {
        signal_id: "sig_123".to_string(),
        symbol: "BTCUSDT".to_string(),
        predicted_signal: TradingSignal::Long,
        actual_outcome: "success".to_string(),
        profit_loss: 250.50,
        confidence_was_accurate: true,
        feedback_notes: None,
        timestamp: 1700000120000,
    };

    let result = client.send_performance_feedback(&feedback).await;

    assert!(result.is_err());
    assert!(result
        .unwrap_err()
        .to_string()
        .contains("Performance feedback request failed"));
}

// ============================================================================
// Tests: Model Performance Metrics
// ============================================================================

#[tokio::test]
async fn test_get_model_performance_success() {
    let server = MockAIServer::start(8107, Box::new(DefaultResponseHandler))
        .await
        .unwrap();

    let client = AIClient::new(&server.base_url(), 5);
    let result = client.get_model_performance().await;

    assert!(result.is_ok());
    let performance = result.unwrap();
    assert_eq!(performance.overall_accuracy, 0.78);
    assert_eq!(performance.precision, 0.82);
    assert_eq!(performance.recall, 0.75);
    assert_eq!(performance.f1_score, 0.78);
    assert_eq!(performance.predictions_made, 10000);
    assert_eq!(performance.successful_predictions, 7800);
    assert_eq!(performance.average_confidence, 0.72);
}

// ============================================================================
// Tests: Timeout Handling
// ============================================================================

#[tokio::test]
async fn test_analyze_signals_timeout() {
    struct TimeoutHandler;
    impl ResponseHandler for TimeoutHandler {
        fn handle(&self, path: &str, _body: Option<serde_json::Value>) -> MockResponse {
            if path == "/ai/analyze" {
                // Delay longer than client timeout
                MockResponse::success(create_test_ai_signal_response())
                    .with_delay(Duration::from_secs(3))
            } else {
                DefaultResponseHandler.handle(path, _body)
            }
        }
    }

    let server = MockAIServer::start(8108, Box::new(TimeoutHandler))
        .await
        .unwrap();

    let client = AIClient::new(&server.base_url(), 1); // 1 second timeout
    let strategy_input = create_test_strategy_input();

    let request = AIAnalysisRequest {
        symbol: strategy_input.symbol.clone(),
        timeframe_data: strategy_input.timeframe_data.clone(),
        current_price: strategy_input.current_price,
        volume_24h: strategy_input.volume_24h,
        timestamp: strategy_input.timestamp,
        strategy_context: AIStrategyContext::default(),
    };

    let result = client.analyze_trading_signals(&request).await;

    assert!(result.is_err());
    // Timeout errors come through as network errors
    assert!(result
        .unwrap_err()
        .to_string()
        .contains("Failed to send AI analysis request"));
}

// ============================================================================
// Tests: Retry Logic in AIService
// ============================================================================

#[tokio::test]
async fn test_ai_service_retry_logic_success_on_second_attempt() {
    let attempt_count = Arc::new(std::sync::Mutex::new(0));
    let attempt_clone = attempt_count.clone();

    struct RetryHandler {
        attempt_count: Arc<std::sync::Mutex<u32>>,
    }

    impl ResponseHandler for RetryHandler {
        fn handle(&self, path: &str, _body: Option<serde_json::Value>) -> MockResponse {
            if path == "/ai/analyze" {
                let mut count = self.attempt_count.lock().unwrap();
                *count += 1;

                if *count == 1 {
                    MockResponse::error(500, "Temporary error")
                } else {
                    MockResponse::success(create_test_ai_signal_response())
                }
            } else {
                DefaultResponseHandler.handle(path, _body)
            }
        }
    }

    let server = MockAIServer::start(
        8109,
        Box::new(RetryHandler {
            attempt_count: attempt_clone.clone(),
        }),
    )
    .await
    .unwrap();

    let config = AIServiceConfig {
        python_service_url: server.base_url(),
        request_timeout_seconds: 5,
        max_retries: 3,
        enable_caching: false,
        cache_ttl_seconds: 0,
    };

    let service = AIService::new(config);
    let strategy_input = create_test_strategy_input();

    let result = service
        .analyze_for_trading_signal(&strategy_input, AIStrategyContext::default())
        .await;

    assert!(result.is_ok());
    let final_count = *attempt_count.lock().unwrap();
    assert_eq!(final_count, 2); // Should have retried once
}

#[tokio::test]
async fn test_ai_service_retry_logic_exhausted() {
    struct AlwaysFailHandler;
    impl ResponseHandler for AlwaysFailHandler {
        fn handle(&self, path: &str, _body: Option<serde_json::Value>) -> MockResponse {
            if path == "/ai/analyze" {
                MockResponse::error(500, "Persistent error")
            } else {
                DefaultResponseHandler.handle(path, _body)
            }
        }
    }

    let server = MockAIServer::start(8110, Box::new(AlwaysFailHandler))
        .await
        .unwrap();

    let config = AIServiceConfig {
        python_service_url: server.base_url(),
        request_timeout_seconds: 5,
        max_retries: 2,
        enable_caching: false,
        cache_ttl_seconds: 0,
    };

    let service = AIService::new(config);
    let strategy_input = create_test_strategy_input();

    let result = service
        .analyze_for_trading_signal(&strategy_input, AIStrategyContext::default())
        .await;

    assert!(result.is_err());
    let error = result.unwrap_err().to_string();
    assert!(error.contains("500") || error.contains("failed"));
}

#[tokio::test]
async fn test_ai_service_retry_with_exponential_backoff() {
    let timestamps = Arc::new(std::sync::Mutex::new(Vec::new()));
    let timestamps_clone = timestamps.clone();

    struct BackoffTestHandler {
        timestamps: Arc<std::sync::Mutex<Vec<std::time::Instant>>>,
    }

    impl ResponseHandler for BackoffTestHandler {
        fn handle(&self, path: &str, _body: Option<serde_json::Value>) -> MockResponse {
            if path == "/ai/analyze" {
                let mut timestamps = self.timestamps.lock().unwrap();
                timestamps.push(std::time::Instant::now());
                let count = timestamps.len();

                if count < 3 {
                    MockResponse::error(500, "Temporary error")
                } else {
                    MockResponse::success(create_test_ai_signal_response())
                }
            } else {
                DefaultResponseHandler.handle(path, _body)
            }
        }
    }

    let server = MockAIServer::start(
        8111,
        Box::new(BackoffTestHandler {
            timestamps: timestamps_clone.clone(),
        }),
    )
    .await
    .unwrap();

    let config = AIServiceConfig {
        python_service_url: server.base_url(),
        request_timeout_seconds: 5,
        max_retries: 3,
        enable_caching: false,
        cache_ttl_seconds: 0,
    };

    let service = AIService::new(config);
    let strategy_input = create_test_strategy_input();

    let result = service
        .analyze_for_trading_signal(&strategy_input, AIStrategyContext::default())
        .await;

    assert!(result.is_ok());

    let times = timestamps.lock().unwrap();
    assert_eq!(times.len(), 3);

    // Check exponential backoff: 100ms, 200ms
    let delay1 = times[1].duration_since(times[0]).as_millis();
    let delay2 = times[2].duration_since(times[1]).as_millis();

    // Allow some tolerance for timing
    assert!(delay1 >= 80 && delay1 <= 150, "First delay: {}ms", delay1);
    assert!(delay2 >= 180 && delay2 <= 250, "Second delay: {}ms", delay2);
}

// ============================================================================
// Tests: AIService High-Level Methods
// ============================================================================

#[tokio::test]
async fn test_ai_service_analyze_for_trading_signal() {
    let server = MockAIServer::start(8112, Box::new(DefaultResponseHandler))
        .await
        .unwrap();

    let config = AIServiceConfig {
        python_service_url: server.base_url(),
        request_timeout_seconds: 5,
        max_retries: 3,
        enable_caching: false,
        cache_ttl_seconds: 0,
    };

    let service = AIService::new(config);
    let strategy_input = create_test_strategy_input();

    let result = service
        .analyze_for_trading_signal(&strategy_input, AIStrategyContext::default())
        .await;

    assert!(result.is_ok());
    let response = result.unwrap();
    assert_eq!(response.signal, TradingSignal::Long);
    assert_eq!(response.confidence, 0.85);
}

#[tokio::test]
async fn test_ai_service_get_strategy_recommendations() {
    let server = MockAIServer::start(8113, Box::new(DefaultResponseHandler))
        .await
        .unwrap();

    let config = AIServiceConfig {
        python_service_url: server.base_url(),
        request_timeout_seconds: 5,
        max_retries: 3,
        enable_caching: false,
        cache_ttl_seconds: 0,
    };

    let service = AIService::new(config);
    let strategy_input = create_test_strategy_input();

    let result = service
        .get_strategy_recommendations(
            &strategy_input,
            vec!["RSI Strategy".to_string(), "MACD Strategy".to_string()],
        )
        .await;

    assert!(result.is_ok());
    let recommendations = result.unwrap();
    assert_eq!(recommendations.len(), 2);
}

#[tokio::test]
async fn test_ai_service_analyze_market_condition() {
    let server = MockAIServer::start(8114, Box::new(DefaultResponseHandler))
        .await
        .unwrap();

    let config = AIServiceConfig {
        python_service_url: server.base_url(),
        request_timeout_seconds: 5,
        max_retries: 3,
        enable_caching: false,
        cache_ttl_seconds: 0,
    };

    let service = AIService::new(config);
    let strategy_input = create_test_strategy_input();

    let result = service.analyze_market_condition(&strategy_input).await;

    assert!(result.is_ok());
    let analysis = result.unwrap();
    assert_eq!(analysis.condition_type, "trending_bullish");
}

#[tokio::test]
async fn test_ai_service_send_performance_feedback() {
    let server = MockAIServer::start(8115, Box::new(DefaultResponseHandler))
        .await
        .unwrap();

    let config = AIServiceConfig {
        python_service_url: server.base_url(),
        request_timeout_seconds: 5,
        max_retries: 3,
        enable_caching: false,
        cache_ttl_seconds: 0,
    };

    let service = AIService::new(config);

    let feedback = PerformanceFeedback {
        signal_id: "sig_456".to_string(),
        symbol: "ETHUSDT".to_string(),
        predicted_signal: TradingSignal::Short,
        actual_outcome: "failure".to_string(),
        profit_loss: -100.0,
        confidence_was_accurate: false,
        feedback_notes: Some("Market reversed unexpectedly".to_string()),
        timestamp: 1700000120000,
    };

    let result = service.send_performance_feedback(feedback).await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_ai_service_get_service_info() {
    let server = MockAIServer::start(8116, Box::new(DefaultResponseHandler))
        .await
        .unwrap();

    let config = AIServiceConfig {
        python_service_url: server.base_url(),
        request_timeout_seconds: 5,
        max_retries: 3,
        enable_caching: false,
        cache_ttl_seconds: 0,
    };

    let service = AIService::new(config);
    let result = service.get_service_info().await;

    assert!(result.is_ok());
    let info = result.unwrap();
    assert_eq!(info.service_name, "Python AI Service");
}

#[tokio::test]
async fn test_ai_service_get_supported_strategies() {
    let server = MockAIServer::start(8117, Box::new(DefaultResponseHandler))
        .await
        .unwrap();

    let config = AIServiceConfig {
        python_service_url: server.base_url(),
        request_timeout_seconds: 5,
        max_retries: 3,
        enable_caching: false,
        cache_ttl_seconds: 0,
    };

    let service = AIService::new(config);
    let result = service.get_supported_strategies().await;

    assert!(result.is_ok());
    let strategies = result.unwrap();
    assert_eq!(strategies.strategies.len(), 4);
}

// ============================================================================
// Tests: Signal Parsing and Validation
// ============================================================================

#[tokio::test]
async fn test_trading_signal_parsing_long() {
    let response_json = json!({
        "signal": "Long",
        "confidence": 0.75,
        "reasoning": "Test",
        "strategy_scores": {},
        "market_analysis": {
            "trend_direction": "bullish",
            "trend_strength": 0.7,
            "support_levels": [],
            "resistance_levels": [],
            "volatility_level": "low",
            "volume_analysis": "normal"
        },
        "risk_assessment": {
            "overall_risk": "low",
            "technical_risk": 0.3,
            "market_risk": 0.2,
            "recommended_position_size": 0.01,
            "stop_loss_suggestion": null,
            "take_profit_suggestion": null
        },
        "timestamp": 1700000000000i64
    });

    let signal: AISignalResponse = serde_json::from_value(response_json).unwrap();
    assert_eq!(signal.signal, TradingSignal::Long);
}

#[tokio::test]
async fn test_trading_signal_parsing_short() {
    let response_json = json!({
        "signal": "Short",
        "confidence": 0.65,
        "reasoning": "Test",
        "strategy_scores": {},
        "market_analysis": {
            "trend_direction": "bearish",
            "trend_strength": 0.6,
            "support_levels": [],
            "resistance_levels": [],
            "volatility_level": "high",
            "volume_analysis": "decreasing"
        },
        "risk_assessment": {
            "overall_risk": "high",
            "technical_risk": 0.7,
            "market_risk": 0.6,
            "recommended_position_size": 0.005,
            "stop_loss_suggestion": null,
            "take_profit_suggestion": null
        },
        "timestamp": 1700000000000i64
    });

    let signal: AISignalResponse = serde_json::from_value(response_json).unwrap();
    assert_eq!(signal.signal, TradingSignal::Short);
}

#[tokio::test]
async fn test_trading_signal_parsing_neutral() {
    let response_json = json!({
        "signal": "Neutral",
        "confidence": 0.5,
        "reasoning": "Test",
        "strategy_scores": {},
        "market_analysis": {
            "trend_direction": "sideways",
            "trend_strength": 0.3,
            "support_levels": [],
            "resistance_levels": [],
            "volatility_level": "moderate",
            "volume_analysis": "stable"
        },
        "risk_assessment": {
            "overall_risk": "moderate",
            "technical_risk": 0.5,
            "market_risk": 0.5,
            "recommended_position_size": 0.01,
            "stop_loss_suggestion": null,
            "take_profit_suggestion": null
        },
        "timestamp": 1700000000000i64
    });

    let signal: AISignalResponse = serde_json::from_value(response_json).unwrap();
    assert_eq!(signal.signal, TradingSignal::Neutral);
}

// ============================================================================
// Tests: Request Serialization
// ============================================================================

#[tokio::test]
async fn test_ai_analysis_request_serialization() {
    let strategy_input = create_test_strategy_input();

    let request = AIAnalysisRequest {
        symbol: strategy_input.symbol.clone(),
        timeframe_data: strategy_input.timeframe_data.clone(),
        current_price: strategy_input.current_price,
        volume_24h: strategy_input.volume_24h,
        timestamp: strategy_input.timestamp,
        strategy_context: AIStrategyContext::default(),
    };

    let json = serde_json::to_value(&request).unwrap();

    assert_eq!(json["symbol"], "BTCUSDT");
    assert_eq!(json["current_price"], 45200.0);
    assert_eq!(json["volume_24h"], 1000000.0);
    assert!(json["timeframe_data"].is_object());
    assert!(json["strategy_context"].is_object());
}

#[tokio::test]
async fn test_strategy_recommendation_request_serialization() {
    let strategy_input = create_test_strategy_input();

    let request = StrategyRecommendationRequest {
        symbol: strategy_input.symbol.clone(),
        timeframe_data: strategy_input.timeframe_data.clone(),
        current_price: strategy_input.current_price,
        available_strategies: vec!["RSI".to_string(), "MACD".to_string()],
        timestamp: strategy_input.timestamp,
    };

    let json = serde_json::to_value(&request).unwrap();

    assert_eq!(json["symbol"], "BTCUSDT");
    assert_eq!(json["available_strategies"].as_array().unwrap().len(), 2);
}

#[tokio::test]
async fn test_performance_feedback_serialization() {
    let feedback = PerformanceFeedback {
        signal_id: "sig_789".to_string(),
        symbol: "BNBUSDT".to_string(),
        predicted_signal: TradingSignal::Long,
        actual_outcome: "success".to_string(),
        profit_loss: 150.0,
        confidence_was_accurate: true,
        feedback_notes: Some("Good call".to_string()),
        timestamp: 1700000120000,
    };

    let json = serde_json::to_value(&feedback).unwrap();

    assert_eq!(json["signal_id"], "sig_789");
    assert_eq!(json["symbol"], "BNBUSDT");
    assert_eq!(json["actual_outcome"], "success");
    assert_eq!(json["profit_loss"], 150.0);
}

// ============================================================================
// Tests: AIStrategyContext
// ============================================================================

#[tokio::test]
async fn test_ai_strategy_context_default() {
    let context = AIStrategyContext::default();

    assert_eq!(context.selected_strategies.len(), 2);
    assert_eq!(context.market_condition, "Unknown");
    assert_eq!(context.risk_level, "Moderate");
    assert!(context.user_preferences.is_empty());
    assert!(context.technical_indicators.is_empty());
}

#[tokio::test]
async fn test_ai_strategy_context_custom() {
    let mut user_prefs = HashMap::new();
    user_prefs.insert("max_risk".to_string(), json!(0.02));

    let mut indicators = HashMap::new();
    indicators.insert("rsi".to_string(), json!(35.5));

    let context = AIStrategyContext {
        selected_strategies: vec!["Bollinger Bands".to_string()],
        market_condition: "volatile".to_string(),
        risk_level: "high".to_string(),
        user_preferences: user_prefs,
        technical_indicators: indicators,
    };

    assert_eq!(context.selected_strategies.len(), 1);
    assert_eq!(context.market_condition, "volatile");
    assert_eq!(context.risk_level, "high");
    assert!(context.user_preferences.contains_key("max_risk"));
    assert!(context.technical_indicators.contains_key("rsi"));
}

// ============================================================================
// Tests: Edge Cases
// ============================================================================

#[tokio::test]
async fn test_empty_timeframe_data() {
    let server = MockAIServer::start(8118, Box::new(DefaultResponseHandler))
        .await
        .unwrap();

    let client = AIClient::new(&server.base_url(), 5);

    let request = AIAnalysisRequest {
        symbol: "BTCUSDT".to_string(),
        timeframe_data: HashMap::new(), // Empty
        current_price: 45000.0,
        volume_24h: 1000000.0,
        timestamp: 1700000000000,
        strategy_context: AIStrategyContext::default(),
    };

    // Should still send request (validation is Python side)
    let result = client.analyze_trading_signals(&request).await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_multiple_timeframes() {
    let server = MockAIServer::start(8119, Box::new(DefaultResponseHandler))
        .await
        .unwrap();

    let client = AIClient::new(&server.base_url(), 5);

    let mut timeframe_data = HashMap::new();
    timeframe_data.insert(
        "1m".to_string(),
        vec![create_test_candle_data(1700000000000, 45000.0)],
    );
    timeframe_data.insert(
        "5m".to_string(),
        vec![create_test_candle_data(1700000000000, 45050.0)],
    );
    timeframe_data.insert(
        "1h".to_string(),
        vec![create_test_candle_data(1700000000000, 45100.0)],
    );

    let request = AIAnalysisRequest {
        symbol: "BTCUSDT".to_string(),
        timeframe_data,
        current_price: 45100.0,
        volume_24h: 1000000.0,
        timestamp: 1700000000000,
        strategy_context: AIStrategyContext::default(),
    };

    let result = client.analyze_trading_signals(&request).await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_very_high_confidence() {
    struct HighConfidenceHandler;
    impl ResponseHandler for HighConfidenceHandler {
        fn handle(&self, path: &str, _body: Option<serde_json::Value>) -> MockResponse {
            if path == "/ai/analyze" {
                let mut response = create_test_ai_signal_response();
                if let Some(obj) = response.as_object_mut() {
                    obj.insert("confidence".to_string(), json!(0.98));
                }
                MockResponse::success(response)
            } else {
                DefaultResponseHandler.handle(path, _body)
            }
        }
    }

    let server = MockAIServer::start(8120, Box::new(HighConfidenceHandler))
        .await
        .unwrap();

    let client = AIClient::new(&server.base_url(), 5);
    let strategy_input = create_test_strategy_input();

    let request = AIAnalysisRequest {
        symbol: strategy_input.symbol.clone(),
        timeframe_data: strategy_input.timeframe_data.clone(),
        current_price: strategy_input.current_price,
        volume_24h: strategy_input.volume_24h,
        timestamp: strategy_input.timestamp,
        strategy_context: AIStrategyContext::default(),
    };

    let result = client.analyze_trading_signals(&request).await;
    assert!(result.is_ok());
    assert_eq!(result.unwrap().confidence, 0.98);
}

#[tokio::test]
async fn test_zero_confidence() {
    struct ZeroConfidenceHandler;
    impl ResponseHandler for ZeroConfidenceHandler {
        fn handle(&self, path: &str, _body: Option<serde_json::Value>) -> MockResponse {
            if path == "/ai/analyze" {
                let mut response = create_test_ai_signal_response();
                if let Some(obj) = response.as_object_mut() {
                    obj.insert("confidence".to_string(), json!(0.0));
                }
                MockResponse::success(response)
            } else {
                DefaultResponseHandler.handle(path, _body)
            }
        }
    }

    let server = MockAIServer::start(8121, Box::new(ZeroConfidenceHandler))
        .await
        .unwrap();

    let client = AIClient::new(&server.base_url(), 5);
    let strategy_input = create_test_strategy_input();

    let request = AIAnalysisRequest {
        symbol: strategy_input.symbol.clone(),
        timeframe_data: strategy_input.timeframe_data.clone(),
        current_price: strategy_input.current_price,
        volume_24h: strategy_input.volume_24h,
        timestamp: strategy_input.timestamp,
        strategy_context: AIStrategyContext::default(),
    };

    let result = client.analyze_trading_signals(&request).await;
    assert!(result.is_ok());
    assert_eq!(result.unwrap().confidence, 0.0);
}

// ============================================================================
// Tests: Data Model Conversions
// ============================================================================

#[tokio::test]
async fn test_candle_data_conversion() {
    let candle = create_test_candle_data(1700000000000, 45000.0);

    // Test that CandleData has all required fields
    assert_eq!(candle.open_time, 1700000000000);
    assert_eq!(candle.close_time, 1700000000000 + 60000);
    assert_eq!(candle.close, 45000.0);
    assert_eq!(candle.volume, 1000.0);
    assert!(candle.is_closed);
}

#[tokio::test]
async fn test_trading_signal_conversion() {
    assert_eq!(TradingSignal::Long.as_str(), "LONG");
    assert_eq!(TradingSignal::Short.as_str(), "SHORT");
    assert_eq!(TradingSignal::Neutral.as_str(), "NEUTRAL");
}

// ============================================================================
// Tests: Concurrent Requests
// ============================================================================

#[tokio::test]
async fn test_concurrent_ai_requests() {
    let server = MockAIServer::start(8122, Box::new(DefaultResponseHandler))
        .await
        .unwrap();

    let config = AIServiceConfig {
        python_service_url: server.base_url(),
        request_timeout_seconds: 10,
        max_retries: 3,
        enable_caching: false,
        cache_ttl_seconds: 0,
    };

    let service = Arc::new(AIService::new(config));
    let mut handles = vec![];

    // Create 5 concurrent requests
    for _i in 0..5 {
        let service_clone = service.clone();
        let handle = tokio::spawn(async move {
            let strategy_input = create_test_strategy_input();
            service_clone
                .analyze_for_trading_signal(&strategy_input, AIStrategyContext::default())
                .await
        });
        handles.push(handle);
    }

    // Wait for all to complete
    for handle in handles {
        let result = handle.await.unwrap();
        assert!(result.is_ok());
    }
}

// ============================================================================
// Tests: Network Errors
// ============================================================================

#[tokio::test]
async fn test_network_error_invalid_host() {
    let client = AIClient::new("http://invalid-host-that-does-not-exist.com", 2);
    let strategy_input = create_test_strategy_input();

    let request = AIAnalysisRequest {
        symbol: strategy_input.symbol.clone(),
        timeframe_data: strategy_input.timeframe_data.clone(),
        current_price: strategy_input.current_price,
        volume_24h: strategy_input.volume_24h,
        timestamp: strategy_input.timestamp,
        strategy_context: AIStrategyContext::default(),
    };

    let result = client.analyze_trading_signals(&request).await;
    assert!(result.is_err());
}

#[tokio::test]
async fn test_network_error_connection_refused() {
    // Use a port that's definitely not listening
    let client = AIClient::new("http://127.0.0.1:54321", 1);
    let strategy_input = create_test_strategy_input();

    let request = AIAnalysisRequest {
        symbol: strategy_input.symbol.clone(),
        timeframe_data: strategy_input.timeframe_data.clone(),
        current_price: strategy_input.current_price,
        volume_24h: strategy_input.volume_24h,
        timestamp: strategy_input.timestamp,
        strategy_context: AIStrategyContext::default(),
    };

    let result = client.analyze_trading_signals(&request).await;
    assert!(result.is_err());
}

// ============================================================================
// End of Tests
// ============================================================================
