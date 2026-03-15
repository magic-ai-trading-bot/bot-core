// Integration tests for AI module (stub interface)
//
// Python AI service was removed. AIClient and AIService are now no-op stubs.
// These tests verify the stub behavior: correct return values, no HTTP calls,
// type serialization, and interface compatibility.

use binance_trading_bot::ai::*;
use binance_trading_bot::market_data::cache::CandleData;
use binance_trading_bot::strategies::{StrategyInput, TradingSignal};
use serde_json::json;
use std::collections::HashMap;

// ============================================================================
// Helpers
// ============================================================================

fn make_candle(timestamp: i64, close: f64) -> CandleData {
    CandleData {
        open_time: timestamp,
        close_time: timestamp + 60000,
        open: close - 10.0,
        high: close + 20.0,
        low: close - 30.0,
        close,
        volume: 1000.0,
        quote_volume: close * 1000.0,
        trades: 100,
        is_closed: true,
    }
}

fn make_strategy_input() -> StrategyInput {
    let mut timeframe_data = HashMap::new();
    timeframe_data.insert(
        "1m".to_string(),
        vec![
            make_candle(1700000000000, 45000.0),
            make_candle(1700000060000, 45100.0),
        ],
    );
    StrategyInput {
        symbol: "BTCUSDT".to_string(),
        timeframe_data,
        current_price: 45100.0,
        volume_24h: 1_000_000.0,
        timestamp: 1700000060000,
    }
}

fn make_ai_service() -> AIService {
    AIService::new(AIServiceConfig::default())
}

fn make_ai_client() -> AIClient {
    AIClient::new("http://127.0.0.1:19999", 1)
}

// ============================================================================
// AIClient stub tests
// ============================================================================

#[tokio::test]
async fn test_ai_client_creation() {
    let client = AIClient::new("http://localhost:8000", 30);
    assert!(format!("{:?}", client).contains("AIClient"));
}

#[tokio::test]
async fn test_ai_client_trailing_slash_trimmed() {
    let client = AIClient::new("http://localhost:8000/", 30);
    assert_eq!(client.base_url(), "http://localhost:8000");
}

#[tokio::test]
async fn test_ai_client_analyze_trading_signals_returns_err() {
    let client = make_ai_client();
    let request = AIAnalysisRequest {
        symbol: "BTCUSDT".to_string(),
        timeframe_data: HashMap::new(),
        current_price: 45000.0,
        volume_24h: 1_000_000.0,
        timestamp: 1700000000000,
        strategy_context: AIStrategyContext::default(),
    };
    let result = client.analyze_trading_signals(&request).await;
    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("disabled"));
}

#[tokio::test]
async fn test_ai_client_get_strategy_recommendations_returns_empty() {
    let client = make_ai_client();
    let request = StrategyRecommendationRequest {
        symbol: "BTCUSDT".to_string(),
        timeframe_data: HashMap::new(),
        current_price: 45000.0,
        available_strategies: vec!["RSI".to_string()],
        timestamp: 1700000000000,
    };
    let result = client.get_strategy_recommendations(&request).await;
    assert!(result.is_ok());
    assert!(result.unwrap().is_empty());
}

#[tokio::test]
async fn test_ai_client_analyze_market_condition_returns_err() {
    let client = make_ai_client();
    let request = MarketConditionRequest {
        symbol: "BTCUSDT".to_string(),
        timeframe_data: HashMap::new(),
        current_price: 45000.0,
        volume_24h: 1_000_000.0,
        timestamp: 1700000000000,
    };
    let result = client.analyze_market_condition(&request).await;
    assert!(result.is_err());
}

#[tokio::test]
async fn test_ai_client_send_performance_feedback_returns_ok() {
    let client = make_ai_client();
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
async fn test_ai_client_get_service_info_returns_err() {
    let client = make_ai_client();
    let result = client.get_service_info().await;
    assert!(result.is_err());
}

#[tokio::test]
async fn test_ai_client_get_supported_strategies_returns_ok() {
    let client = make_ai_client();
    let result = client.get_supported_strategies().await;
    assert!(result.is_ok());
    let strategies = result.unwrap();
    assert!(!strategies.strategies.is_empty());
    assert!(strategies.strategies.iter().any(|s| s.contains("RSI")));
    assert!(strategies.strategies.iter().any(|s| s.contains("MACD")));
}

#[tokio::test]
async fn test_ai_client_request_trade_analysis_returns_ok() {
    use binance_trading_bot::ai::client::TradeAnalysisRequest;
    let client = make_ai_client();
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
    let result = client.request_trade_analysis(&request).await;
    assert!(result.is_ok());
}

// ============================================================================
// AIService stub tests
// ============================================================================

#[tokio::test]
async fn test_ai_service_config_default() {
    let config = AIServiceConfig::default();
    assert_eq!(config.python_service_url, "http://localhost:8000");
    assert_eq!(config.request_timeout_seconds, 30);
    assert_eq!(config.max_retries, 3);
    assert!(config.enable_caching);
    assert_eq!(config.cache_ttl_seconds, 300);
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
    let service = AIService::new(config);
    assert!(format!("{:?}", service).contains("AIService"));
}

#[tokio::test]
async fn test_ai_service_analyze_for_trading_signal_returns_err() {
    let service = make_ai_service();
    let input = make_strategy_input();
    let result = service
        .analyze_for_trading_signal(&input, AIStrategyContext::default())
        .await;
    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("disabled"));
}

#[tokio::test]
async fn test_ai_service_get_strategy_recommendations_returns_empty() {
    let service = make_ai_service();
    let input = make_strategy_input();
    let result = service
        .get_strategy_recommendations(&input, vec!["RSI Strategy".to_string()])
        .await;
    assert!(result.is_ok());
    assert!(result.unwrap().is_empty());
}

#[tokio::test]
async fn test_ai_service_analyze_market_condition_returns_err() {
    let service = make_ai_service();
    let input = make_strategy_input();
    let result = service.analyze_market_condition(&input).await;
    assert!(result.is_err());
}

#[tokio::test]
async fn test_ai_service_send_performance_feedback_returns_ok() {
    let service = make_ai_service();
    let feedback = PerformanceFeedback {
        signal_id: "sig_456".to_string(),
        symbol: "ETHUSDT".to_string(),
        predicted_signal: TradingSignal::Short,
        actual_outcome: "failure".to_string(),
        profit_loss: -100.0,
        confidence_was_accurate: false,
        feedback_notes: Some("Market reversed".to_string()),
        timestamp: 1700000120000,
    };
    let result = service.send_performance_feedback(feedback).await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_ai_service_get_service_info_returns_err() {
    let service = make_ai_service();
    let result = service.get_service_info().await;
    assert!(result.is_err());
}

#[tokio::test]
async fn test_ai_service_get_supported_strategies_returns_ok() {
    let service = make_ai_service();
    let result = service.get_supported_strategies().await;
    assert!(result.is_ok());
    let strategies = result.unwrap();
    assert!(!strategies.strategies.is_empty());
    assert!(strategies.strategies.iter().any(|s| s.contains("RSI")));
}

// ============================================================================
// Type / serialization tests
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
    assert!(context.user_preferences.contains_key("max_risk"));
    assert!(context.technical_indicators.contains_key("rsi"));
}

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
    assert_eq!(signal.confidence, 0.75);
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

#[tokio::test]
async fn test_trading_signal_as_str() {
    assert_eq!(TradingSignal::Long.as_str(), "LONG");
    assert_eq!(TradingSignal::Short.as_str(), "SHORT");
    assert_eq!(TradingSignal::Neutral.as_str(), "NEUTRAL");
}

#[tokio::test]
async fn test_ai_analysis_request_serialization() {
    let request = AIAnalysisRequest {
        symbol: "BTCUSDT".to_string(),
        timeframe_data: HashMap::new(),
        current_price: 45200.0,
        volume_24h: 1_000_000.0,
        timestamp: 1700000120000,
        strategy_context: AIStrategyContext::default(),
    };

    let json = serde_json::to_value(&request).unwrap();
    assert_eq!(json["symbol"], "BTCUSDT");
    assert_eq!(json["current_price"], 45200.0);
    assert_eq!(json["volume_24h"], 1_000_000.0);
    assert!(json["timeframe_data"].is_object());
    assert!(json["strategy_context"].is_object());
}

#[tokio::test]
async fn test_strategy_recommendation_request_serialization() {
    let request = StrategyRecommendationRequest {
        symbol: "BTCUSDT".to_string(),
        timeframe_data: HashMap::new(),
        current_price: 45200.0,
        available_strategies: vec!["RSI".to_string(), "MACD".to_string()],
        timestamp: 1700000120000,
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
    assert_eq!(json["profit_loss"], 150.0);
}

#[tokio::test]
async fn test_candle_data_fields() {
    let candle = make_candle(1700000000000, 45000.0);
    assert_eq!(candle.open_time, 1700000000000);
    assert_eq!(candle.close_time, 1700000000000 + 60000);
    assert_eq!(candle.close, 45000.0);
    assert_eq!(candle.volume, 1000.0);
    assert!(candle.is_closed);
}

// ============================================================================
// Concurrent stub calls (no HTTP, so these run fast)
// ============================================================================

#[tokio::test]
async fn test_concurrent_stub_calls() {
    use std::sync::Arc;

    let service = Arc::new(make_ai_service());
    let mut handles = vec![];

    for _ in 0..5 {
        let svc = service.clone();
        let handle = tokio::spawn(async move {
            let input = make_strategy_input();
            svc.analyze_for_trading_signal(&input, AIStrategyContext::default())
                .await
        });
        handles.push(handle);
    }

    for handle in handles {
        let result = handle.await.unwrap();
        // All should return err since stub is disabled
        assert!(result.is_err());
    }
}

// ============================================================================
// Edge cases
// ============================================================================

#[tokio::test]
async fn test_empty_timeframe_data_in_request() {
    let client = make_ai_client();
    let request = AIAnalysisRequest {
        symbol: "BTCUSDT".to_string(),
        timeframe_data: HashMap::new(),
        current_price: 45000.0,
        volume_24h: 1_000_000.0,
        timestamp: 1700000000000,
        strategy_context: AIStrategyContext::default(),
    };
    // Stub returns error regardless of request content
    let result = client.analyze_trading_signals(&request).await;
    assert!(result.is_err());
}

#[tokio::test]
async fn test_multiple_timeframes_in_request() {
    let client = make_ai_client();
    let mut timeframe_data = HashMap::new();
    timeframe_data.insert("1m".to_string(), vec![make_candle(1700000000000, 45000.0)]);
    timeframe_data.insert("5m".to_string(), vec![make_candle(1700000000000, 45050.0)]);
    timeframe_data.insert("1h".to_string(), vec![make_candle(1700000000000, 45100.0)]);

    let request = AIAnalysisRequest {
        symbol: "BTCUSDT".to_string(),
        timeframe_data,
        current_price: 45100.0,
        volume_24h: 1_000_000.0,
        timestamp: 1700000000000,
        strategy_context: AIStrategyContext::default(),
    };
    // Stub returns error regardless
    let result = client.analyze_trading_signals(&request).await;
    assert!(result.is_err());
}

#[tokio::test]
async fn test_performance_feedback_no_notes() {
    let client = make_ai_client();
    let feedback = PerformanceFeedback {
        signal_id: "sig_000".to_string(),
        symbol: "BTCUSDT".to_string(),
        predicted_signal: TradingSignal::Neutral,
        actual_outcome: "neutral".to_string(),
        profit_loss: 0.0,
        confidence_was_accurate: false,
        feedback_notes: None,
        timestamp: 1700000000000,
    };
    let result = client.send_performance_feedback(&feedback).await;
    assert!(result.is_ok());
}
