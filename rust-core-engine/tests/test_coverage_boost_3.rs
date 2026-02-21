// Coverage Boost 3: Comprehensive tests for remaining uncovered lines
// Target files:
// - api/mod.rs (88.15%, 225 uncovered)
// - api/settings.rs (85.51%, 134 uncovered)
// - api/notifications.rs (88.10%, 85 uncovered)
// - strategies/bollinger_strategy.rs (remaining)
// - strategies/stochastic_strategy.rs (82.93%, 63 uncovered)
// - strategies/ml_trend_predictor.rs (78.20%, 63 uncovered)
// - ai/mod.rs (83.25%, 65 uncovered)
// - ai/client.rs (86.88%, 127 uncovered)

mod common;

use binance_trading_bot::market_data::cache::CandleData;
use binance_trading_bot::strategies::bollinger_strategy::BollingerStrategy;
use binance_trading_bot::strategies::ml_trend_predictor::{
    MLPredictorConfig, MLTrendPrediction, MLTrendPredictor,
};
use binance_trading_bot::strategies::stochastic_strategy::StochasticStrategy;
use binance_trading_bot::strategies::trend_filter::TrendDirection;
use binance_trading_bot::strategies::{Strategy, StrategyConfig, StrategyInput};
use serde_json::json;
use std::collections::HashMap;

// ========== HELPER FUNCTIONS ==========

fn create_test_candles(prices: Vec<f64>) -> Vec<CandleData> {
    prices
        .iter()
        .enumerate()
        .map(|(i, &price)| CandleData {
            open: price,
            high: price * 1.02,
            low: price * 0.98,
            close: price,
            volume: 1000.0,
            open_time: (i as i64) * 3600000,
            close_time: (i as i64) * 3600000 + 3600000,
            quote_volume: 1000.0 * price,
            trades: 100,
            is_closed: true,
        })
        .collect()
}

fn create_test_input(
    prices_1h: Vec<f64>,
    prices_4h: Vec<f64>,
    current_price: f64,
) -> StrategyInput {
    let mut timeframe_data = HashMap::new();
    timeframe_data.insert("5m".to_string(), create_test_candles(prices_1h));
    timeframe_data.insert("15m".to_string(), create_test_candles(prices_4h));

    StrategyInput {
        symbol: "BTCUSDT".to_string(),
        timeframe_data,
        current_price,
        volume_24h: 1000000.0,
        timestamp: 1234567890,
    }
}

// ========== BOLLINGER STRATEGY COV3 TESTS ==========

#[tokio::test]
async fn test_cov3_bollinger_missing_5m_data() {
    let strategy = BollingerStrategy::new();

    let mut timeframe_data = HashMap::new();
    // Only 15m data, missing 5m
    timeframe_data.insert("15m".to_string(), create_test_candles(vec![100.0; 30]));

    let input = StrategyInput {
        symbol: "BTCUSDT".to_string(),
        timeframe_data,
        current_price: 100.0,
        volume_24h: 1000000.0,
        timestamp: 1234567890,
    };

    let result = strategy.analyze(&input).await;
    assert!(result.is_err());
    if let Err(e) = result {
        assert!(format!("{:?}", e).contains("Missing 5m"));
    }
}

#[tokio::test]
async fn test_cov3_bollinger_missing_15m_data() {
    let strategy = BollingerStrategy::new();

    let mut timeframe_data = HashMap::new();
    // Only 5m data, missing 15m
    timeframe_data.insert("5m".to_string(), create_test_candles(vec![100.0; 30]));

    let input = StrategyInput {
        symbol: "BTCUSDT".to_string(),
        timeframe_data,
        current_price: 100.0,
        volume_24h: 1000000.0,
        timestamp: 1234567890,
    };

    let result = strategy.analyze(&input).await;
    assert!(result.is_err());
}

#[tokio::test]
async fn test_cov3_bollinger_empty_bb_values() {
    let strategy = BollingerStrategy::new();

    // Too few data points to calculate BB
    let prices = vec![100.0; 5];
    let input = create_test_input(prices.clone(), prices, 100.0);

    let result = strategy.analyze(&input).await;
    assert!(result.is_err());
}

#[tokio::test]
async fn test_cov3_bollinger_price_above_upper_band_no_squeeze() {
    let strategy = BollingerStrategy::new();

    // Create prices with price above upper band but no squeeze
    let prices_1h: Vec<f64> = (0..30).map(|i| 100.0 + (i as f64 * 2.0)).collect();
    let prices_4h: Vec<f64> = (0..30).map(|i| 100.0 + (i as f64 * 1.5)).collect();
    let current_price = 165.0;

    let input = create_test_input(prices_1h, prices_4h, current_price);
    let result = strategy.analyze(&input).await;

    assert!(result.is_ok());
    let output = result.unwrap();
    assert!(output.metadata.contains_key("bb_upper_5m"));
}

#[tokio::test]
async fn test_cov3_bollinger_with_config_updates() {
    let mut strategy = BollingerStrategy::new();

    // Test config updates
    let mut new_config = StrategyConfig::default();
    new_config.enabled = false;
    new_config.weight = 2.5;
    new_config
        .parameters
        .insert("bb_period".to_string(), json!(30));
    new_config
        .parameters
        .insert("bb_multiplier".to_string(), json!(3.0));
    new_config
        .parameters
        .insert("squeeze_threshold".to_string(), json!(0.025));

    strategy.update_config(new_config.clone());

    assert_eq!(strategy.config().enabled, false);
    assert_eq!(strategy.config().weight, 2.5);
}

// ========== STOCHASTIC STRATEGY COV3 TESTS ==========

#[tokio::test]
async fn test_cov3_stochastic_missing_5m_data() {
    let strategy = StochasticStrategy::new();

    let mut timeframe_data = HashMap::new();
    timeframe_data.insert("15m".to_string(), create_test_candles(vec![100.0; 30]));

    let input = StrategyInput {
        symbol: "ETHUSDT".to_string(),
        timeframe_data,
        current_price: 100.0,
        volume_24h: 1000000.0,
        timestamp: 1234567890,
    };

    let result = strategy.analyze(&input).await;
    assert!(result.is_err());
}

#[tokio::test]
async fn test_cov3_stochastic_missing_15m_data() {
    let strategy = StochasticStrategy::new();

    let mut timeframe_data = HashMap::new();
    timeframe_data.insert("5m".to_string(), create_test_candles(vec![100.0; 30]));

    let input = StrategyInput {
        symbol: "ETHUSDT".to_string(),
        timeframe_data,
        current_price: 100.0,
        volume_24h: 1000000.0,
        timestamp: 1234567890,
    };

    let result = strategy.analyze(&input).await;
    assert!(result.is_err());
}

#[tokio::test]
async fn test_cov3_stochastic_insufficient_candles() {
    let strategy = StochasticStrategy::new();

    // Need at least k_period + d_period + 5 = 14 + 3 + 5 = 22 candles
    let prices = vec![100.0; 15];
    let mut timeframe_data = HashMap::new();
    timeframe_data.insert("5m".to_string(), create_test_candles(prices.clone()));
    timeframe_data.insert("15m".to_string(), create_test_candles(prices));

    let input = StrategyInput {
        symbol: "BTCUSDT".to_string(),
        timeframe_data,
        current_price: 100.0,
        volume_24h: 1000000.0,
        timestamp: 1234567890,
    };

    let result = strategy.validate_data(&input);
    assert!(result.is_err());
    if let Err(e) = result {
        assert!(format!("{:?}", e).contains("Need at least"));
    }
}

// Stochastic strategy tests that test through the public interface

#[tokio::test]
async fn test_cov3_stochastic_with_custom_config() {
    let mut config = StrategyConfig::default();
    config.enabled = true;
    config.weight = 1.5;
    config.parameters.insert("k_period".to_string(), json!(10));
    config.parameters.insert("d_period".to_string(), json!(5));
    config
        .parameters
        .insert("oversold_threshold".to_string(), json!(25.0));
    config
        .parameters
        .insert("overbought_threshold".to_string(), json!(75.0));
    config
        .parameters
        .insert("extreme_oversold".to_string(), json!(15.0));
    config
        .parameters
        .insert("extreme_overbought".to_string(), json!(85.0));

    let strategy = StochasticStrategy::with_config(config.clone());

    assert_eq!(strategy.config().weight, 1.5);
    assert!(strategy.config().enabled);
}

#[tokio::test]
async fn test_cov3_stochastic_default_trait() {
    let strategy = StochasticStrategy::default();
    assert_eq!(strategy.name(), "Stochastic Strategy");
}

#[tokio::test]
async fn test_cov3_stochastic_description() {
    let strategy = StochasticStrategy::new();
    let desc = strategy.description();
    assert!(desc.contains("Stochastic"));
    assert!(desc.contains("overbought"));
    assert!(desc.contains("oversold"));
}

// ========== ML TREND PREDICTOR COV3 TESTS ==========

#[tokio::test]
async fn test_cov3_ml_predictor_new() {
    let config = MLPredictorConfig::default();
    let predictor = MLTrendPredictor::new(config.clone());

    assert_eq!(predictor.config().service_url, config.service_url);
    assert_eq!(predictor.config().timeout_ms, config.timeout_ms);
}

#[tokio::test]
async fn test_cov3_ml_predictor_custom_config() {
    let config = MLPredictorConfig {
        service_url: "http://custom-ml:9000".to_string(),
        timeout_ms: 3500,
        min_confidence: 0.7,
        fallback_on_error: false,
    };

    let predictor = MLTrendPredictor::new(config.clone());

    assert_eq!(predictor.config().service_url, "http://custom-ml:9000");
    assert_eq!(predictor.config().timeout_ms, 3500);
    assert_eq!(predictor.config().min_confidence, 0.7);
    assert!(!predictor.config().fallback_on_error);
}

#[test]
fn test_cov3_ml_prediction_creation_uptrend() {
    let prediction = MLTrendPrediction {
        trend: TrendDirection::Uptrend,
        confidence: 0.82,
        model: "LSTM".to_string(),
        timestamp: 1234567890,
    };

    assert_eq!(prediction.trend, TrendDirection::Uptrend);
    assert_eq!(prediction.confidence, 0.82);
    assert_eq!(prediction.model, "LSTM");
    assert_eq!(prediction.timestamp, 1234567890);
}

#[test]
fn test_cov3_ml_prediction_creation_downtrend() {
    let prediction = MLTrendPrediction {
        trend: TrendDirection::Downtrend,
        confidence: 0.76,
        model: "GRU".to_string(),
        timestamp: 9876543210,
    };

    assert_eq!(prediction.trend, TrendDirection::Downtrend);
    assert_eq!(prediction.confidence, 0.76);
}

#[test]
fn test_cov3_ml_prediction_creation_neutral() {
    let prediction = MLTrendPrediction {
        trend: TrendDirection::Neutral,
        confidence: 0.55,
        model: "Ensemble".to_string(),
        timestamp: 1111111111,
    };

    assert_eq!(prediction.trend, TrendDirection::Neutral);
    assert_eq!(prediction.confidence, 0.55);
}

#[test]
fn test_cov3_ml_prediction_serialization() {
    let prediction = MLTrendPrediction {
        trend: TrendDirection::Uptrend,
        confidence: 0.88,
        model: "Transformer".to_string(),
        timestamp: 1600000000,
    };

    let json = serde_json::to_string(&prediction).unwrap();
    assert!(json.contains("Uptrend"));
    assert!(json.contains("0.88"));
    assert!(json.contains("Transformer"));
    assert!(json.contains("1600000000"));
}

#[test]
fn test_cov3_ml_prediction_deserialization() {
    let json = r#"{
        "trend": "Downtrend",
        "confidence": 0.73,
        "model": "XGBoost",
        "timestamp": 1700000000
    }"#;

    let prediction: MLTrendPrediction = serde_json::from_str(json).unwrap();

    assert_eq!(prediction.trend, TrendDirection::Downtrend);
    assert_eq!(prediction.confidence, 0.73);
    assert_eq!(prediction.model, "XGBoost");
    assert_eq!(prediction.timestamp, 1700000000);
}

#[test]
fn test_cov3_ml_config_serialization() {
    let config = MLPredictorConfig {
        service_url: "http://ml-backend:8080".to_string(),
        timeout_ms: 4000,
        min_confidence: 0.8,
        fallback_on_error: true,
    };

    let json = serde_json::to_string(&config).unwrap();
    assert!(json.contains("ml-backend:8080"));
    assert!(json.contains("4000"));
    assert!(json.contains("0.8"));
}

#[test]
fn test_cov3_ml_config_deserialization() {
    let json = r#"{
        "service_url": "https://prod-ml.example.com",
        "timeout_ms": 5500,
        "min_confidence": 0.72,
        "fallback_on_error": false
    }"#;

    let config: MLPredictorConfig = serde_json::from_str(json).unwrap();

    assert_eq!(config.service_url, "https://prod-ml.example.com");
    assert_eq!(config.timeout_ms, 5500);
    assert_eq!(config.min_confidence, 0.72);
    assert!(!config.fallback_on_error);
}

#[test]
fn test_cov3_ml_prediction_debug_display() {
    let prediction = MLTrendPrediction {
        trend: TrendDirection::Uptrend,
        confidence: 0.91,
        model: "LSTM-Attention".to_string(),
        timestamp: 1234567890,
    };

    let debug_str = format!("{:?}", prediction);
    assert!(debug_str.contains("MLTrendPrediction"));
    assert!(debug_str.contains("Uptrend"));
    assert!(debug_str.contains("0.91"));
    assert!(debug_str.contains("LSTM-Attention"));
}

#[test]
fn test_cov3_ml_config_debug_display() {
    let config = MLPredictorConfig::default();
    let debug_str = format!("{:?}", config);

    assert!(debug_str.contains("MLPredictorConfig"));
    assert!(debug_str.contains("localhost:8000"));
}

#[test]
fn test_cov3_ml_prediction_clone() {
    let pred1 = MLTrendPrediction {
        trend: TrendDirection::Downtrend,
        confidence: 0.79,
        model: "RandomForest".to_string(),
        timestamp: 1500000000,
    };

    let pred2 = pred1.clone();

    assert_eq!(pred1.trend, pred2.trend);
    assert_eq!(pred1.confidence, pred2.confidence);
    assert_eq!(pred1.model, pred2.model);
    assert_eq!(pred1.timestamp, pred2.timestamp);
}

#[test]
fn test_cov3_ml_config_clone() {
    let config1 = MLPredictorConfig {
        service_url: "http://test:9999".to_string(),
        timeout_ms: 1500,
        min_confidence: 0.6,
        fallback_on_error: false,
    };

    let config2 = config1.clone();

    assert_eq!(config1.service_url, config2.service_url);
    assert_eq!(config1.timeout_ms, config2.timeout_ms);
    assert_eq!(config1.min_confidence, config2.min_confidence);
    assert_eq!(config1.fallback_on_error, config2.fallback_on_error);
}

#[test]
fn test_cov3_ml_prediction_confidence_range() {
    let low_confidence = MLTrendPrediction {
        trend: TrendDirection::Neutral,
        confidence: 0.1,
        model: "Test".to_string(),
        timestamp: 123,
    };

    let high_confidence = MLTrendPrediction {
        trend: TrendDirection::Uptrend,
        confidence: 0.99,
        model: "Test".to_string(),
        timestamp: 456,
    };

    assert!(low_confidence.confidence < 0.5);
    assert!(high_confidence.confidence > 0.9);
}

#[test]
fn test_cov3_ml_config_various_timeouts() {
    let configs = vec![
        MLPredictorConfig {
            timeout_ms: 500,
            ..Default::default()
        },
        MLPredictorConfig {
            timeout_ms: 1000,
            ..Default::default()
        },
        MLPredictorConfig {
            timeout_ms: 5000,
            ..Default::default()
        },
        MLPredictorConfig {
            timeout_ms: 10000,
            ..Default::default()
        },
    ];

    for (i, config) in configs.iter().enumerate() {
        let expected = match i {
            0 => 500,
            1 => 1000,
            2 => 5000,
            3 => 10000,
            _ => 0,
        };
        assert_eq!(config.timeout_ms, expected);
    }
}

#[test]
fn test_cov3_ml_config_various_confidence_thresholds() {
    let thresholds = vec![0.5, 0.6, 0.7, 0.8, 0.9];

    for threshold in thresholds {
        let config = MLPredictorConfig {
            min_confidence: threshold,
            ..Default::default()
        };

        assert_eq!(config.min_confidence, threshold);
    }
}

#[test]
fn test_cov3_ml_prediction_all_trend_types() {
    let trends = vec![
        (TrendDirection::Uptrend, "Uptrend"),
        (TrendDirection::Downtrend, "Downtrend"),
        (TrendDirection::Neutral, "Neutral"),
    ];

    for (trend, name) in trends {
        let prediction = MLTrendPrediction {
            trend: trend.clone(),
            confidence: 0.75,
            model: name.to_string(),
            timestamp: 123456,
        };

        assert_eq!(prediction.trend, trend);
        assert_eq!(prediction.model, name);
    }
}

#[test]
fn test_cov3_ml_config_different_urls() {
    let urls = vec![
        "http://localhost:8000",
        "http://ml-service:9000",
        "https://production-ml.example.com",
        "http://192.168.1.100:7777",
    ];

    for url in urls {
        let config = MLPredictorConfig {
            service_url: url.to_string(),
            ..Default::default()
        };

        assert_eq!(config.service_url, url);
    }
}

#[test]
fn test_cov3_ml_prediction_edge_case_timestamps() {
    let predictions = vec![
        MLTrendPrediction {
            trend: TrendDirection::Uptrend,
            confidence: 0.8,
            model: "Test".to_string(),
            timestamp: 0,
        },
        MLTrendPrediction {
            trend: TrendDirection::Downtrend,
            confidence: 0.75,
            model: "Test".to_string(),
            timestamp: i64::MAX,
        },
    ];

    assert_eq!(predictions[0].timestamp, 0);
    assert_eq!(predictions[1].timestamp, i64::MAX);
}

#[test]
fn test_cov3_ml_config_fallback_modes() {
    let with_fallback = MLPredictorConfig {
        fallback_on_error: true,
        ..Default::default()
    };

    let without_fallback = MLPredictorConfig {
        fallback_on_error: false,
        ..Default::default()
    };

    assert!(with_fallback.fallback_on_error);
    assert!(!without_fallback.fallback_on_error);
}

#[test]
fn test_cov3_ml_prediction_model_names() {
    let models = vec![
        "LSTM",
        "GRU",
        "Transformer",
        "Ensemble",
        "XGBoost",
        "RandomForest",
        "CNN-LSTM",
    ];

    for model in models {
        let prediction = MLTrendPrediction {
            trend: TrendDirection::Uptrend,
            confidence: 0.8,
            model: model.to_string(),
            timestamp: 123456,
        };

        assert_eq!(prediction.model, model);
    }
}

// ========== ADDITIONAL STRATEGY EDGE CASE TESTS ==========

#[tokio::test]
async fn test_cov3_bollinger_with_single_price_variation() {
    let strategy = BollingerStrategy::new();

    // Create prices with minimal variation (will trigger squeeze)
    let mut prices = vec![100.0; 25];
    prices.push(100.1);
    prices.push(99.9);
    prices.push(100.05);
    prices.push(99.95);
    prices.push(100.0);

    let input = create_test_input(prices.clone(), prices, 100.0);
    let result = strategy.analyze(&input).await;

    assert!(result.is_ok());
    if let Ok(output) = result {
        assert!(output.metadata.contains_key("is_squeeze_5m"));
    }
}

#[tokio::test]
async fn test_cov3_stochastic_with_oscillating_prices() {
    let strategy = StochasticStrategy::new();

    // Create oscillating prices
    let prices: Vec<f64> = (0..30)
        .map(|i| {
            let base = 100.0;
            let oscillation = if i % 2 == 0 { 5.0 } else { -5.0 };
            base + oscillation
        })
        .collect();

    let mut timeframe_data = HashMap::new();
    timeframe_data.insert("5m".to_string(), create_test_candles(prices.clone()));
    timeframe_data.insert("15m".to_string(), create_test_candles(prices));

    let input = StrategyInput {
        symbol: "BTCUSDT".to_string(),
        timeframe_data,
        current_price: 100.0,
        volume_24h: 1000000.0,
        timestamp: 1234567890,
    };

    let result = strategy.analyze(&input).await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_cov3_strategies_with_large_price_movements() {
    let bollinger = BollingerStrategy::new();
    let stochastic = StochasticStrategy::new();

    // Create prices with large movements
    let prices: Vec<f64> = (0..30).map(|i| 100.0 + (i as f64 * 10.0)).collect();
    let input = create_test_input(prices.clone(), prices, 400.0);

    let bb_result = bollinger.analyze(&input).await;
    let stoch_result = stochastic.analyze(&input).await;

    assert!(bb_result.is_ok());
    assert!(stoch_result.is_ok());
}
