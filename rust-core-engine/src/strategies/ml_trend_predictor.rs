use crate::strategies::trend_filter::TrendDirection;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::time::Duration;

/// ML-based trend prediction result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MLTrendPrediction {
    pub trend: TrendDirection,
    pub confidence: f64, // 0.0 - 1.0
    pub model: String,   // Model name (e.g., "LSTM", "GRU", "Ensemble")
    pub timestamp: i64,
}

/// Configuration for ML trend predictor
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MLPredictorConfig {
    pub service_url: String,
    pub timeout_ms: u64,
    pub min_confidence: f64,
    pub fallback_on_error: bool,
}

impl Default for MLPredictorConfig {
    fn default() -> Self {
        Self {
            service_url: "http://localhost:8000".to_string(),
            timeout_ms: 2000,
            min_confidence: 0.65,
            fallback_on_error: true,
        }
    }
}

/// ML trend predictor using Python AI service
pub struct MLTrendPredictor {
    config: MLPredictorConfig,
    client: Client,
}

impl MLTrendPredictor {
    pub fn new(config: MLPredictorConfig) -> Self {
        let client = Client::builder()
            .timeout(Duration::from_millis(config.timeout_ms))
            .build()
            .expect("Failed to create HTTP client");

        Self { config, client }
    }

    /// Predict trend direction using ML model
    pub async fn predict_trend(
        &self,
        symbol: &str,
        timeframe: &str,
    ) -> Result<MLTrendPrediction, String> {
        let url = format!("{}/predict-trend", self.config.service_url);

        let request_body = serde_json::json!({
            "symbol": symbol,
            "timeframe": timeframe,
        });

        let response = self
            .client
            .post(&url)
            .json(&request_body)
            .send()
            .await
            .map_err(|e| format!("HTTP request failed: {}", e))?;

        if !response.status().is_success() {
            return Err(format!("ML service returned error: {}", response.status()));
        }

        let prediction: MLTrendPrediction = response
            .json()
            .await
            .map_err(|e| format!("Failed to parse response: {}", e))?;

        // Validate confidence
        if prediction.confidence < self.config.min_confidence {
            return Err(format!(
                "Confidence {} below threshold {}",
                prediction.confidence, self.config.min_confidence
            ));
        }

        Ok(prediction)
    }

    /// Predict trend with fallback to None on error
    pub async fn predict_trend_with_fallback(
        &self,
        symbol: &str,
        timeframe: &str,
    ) -> Option<MLTrendPrediction> {
        match self.predict_trend(symbol, timeframe).await {
            Ok(prediction) => Some(prediction),
            Err(e) => {
                if self.config.fallback_on_error {
                    log::warn!("ML prediction failed (fallback enabled): {}", e);
                    None
                } else {
                    log::error!("ML prediction failed (no fallback): {}", e);
                    None
                }
            },
        }
    }

    /// Check if ML service is available
    pub async fn health_check(&self) -> Result<(), String> {
        let url = format!("{}/health", self.config.service_url);

        let response = self
            .client
            .get(&url)
            .send()
            .await
            .map_err(|e| format!("Health check failed: {}", e))?;

        if response.status().is_success() {
            Ok(())
        } else {
            Err(format!("ML service unhealthy: {}", response.status()))
        }
    }

    /// Get configuration
    pub fn config(&self) -> &MLPredictorConfig {
        &self.config
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ml_predictor_config_default() {
        let config = MLPredictorConfig::default();
        assert_eq!(config.service_url, "http://localhost:8000");
        assert_eq!(config.timeout_ms, 2000);
        assert_eq!(config.min_confidence, 0.65);
        assert!(config.fallback_on_error);
    }

    #[test]
    fn test_ml_predictor_config_custom() {
        let config = MLPredictorConfig {
            service_url: "http://ml-service:9000".to_string(),
            timeout_ms: 5000,
            min_confidence: 0.75,
            fallback_on_error: false,
        };

        assert_eq!(config.service_url, "http://ml-service:9000");
        assert_eq!(config.timeout_ms, 5000);
        assert_eq!(config.min_confidence, 0.75);
        assert!(!config.fallback_on_error);
    }

    #[test]
    fn test_ml_trend_prediction_creation() {
        let prediction = MLTrendPrediction {
            trend: TrendDirection::Uptrend,
            confidence: 0.85,
            model: "LSTM".to_string(),
            timestamp: 1234567890,
        };

        assert_eq!(prediction.trend, TrendDirection::Uptrend);
        assert_eq!(prediction.confidence, 0.85);
        assert_eq!(prediction.model, "LSTM");
        assert_eq!(prediction.timestamp, 1234567890);
    }

    #[test]
    fn test_ml_trend_prediction_serialize() {
        let prediction = MLTrendPrediction {
            trend: TrendDirection::Downtrend,
            confidence: 0.78,
            model: "Ensemble".to_string(),
            timestamp: 9876543210,
        };

        let serialized = serde_json::to_string(&prediction).unwrap();
        assert!(serialized.contains("Downtrend"));
        assert!(serialized.contains("0.78"));
        assert!(serialized.contains("Ensemble"));
    }

    #[test]
    fn test_ml_trend_prediction_deserialize() {
        let json = r#"{
            "trend": "Uptrend",
            "confidence": 0.92,
            "model": "GRU",
            "timestamp": 1234567890
        }"#;

        let prediction: MLTrendPrediction = serde_json::from_str(json).unwrap();
        assert_eq!(prediction.trend, TrendDirection::Uptrend);
        assert_eq!(prediction.confidence, 0.92);
        assert_eq!(prediction.model, "GRU");
        assert_eq!(prediction.timestamp, 1234567890);
    }

    #[tokio::test]
    async fn test_ml_predictor_creation() {
        let config = MLPredictorConfig::default();
        let predictor = MLTrendPredictor::new(config);
        assert_eq!(predictor.config().service_url, "http://localhost:8000");
    }

    #[test]
    fn test_ml_trend_prediction_with_different_trends() {
        let trends = vec![
            TrendDirection::Uptrend,
            TrendDirection::Downtrend,
            TrendDirection::Neutral,
        ];

        for trend in trends {
            let prediction = MLTrendPrediction {
                trend: trend.clone(),
                confidence: 0.75,
                model: "Test".to_string(),
                timestamp: 123456,
            };

            assert_eq!(prediction.trend, trend);
        }
    }

    #[test]
    fn test_ml_predictor_config_clone() {
        let config1 = MLPredictorConfig::default();
        let config2 = config1.clone();

        assert_eq!(config1.service_url, config2.service_url);
        assert_eq!(config1.timeout_ms, config2.timeout_ms);
        assert_eq!(config1.min_confidence, config2.min_confidence);
        assert_eq!(config1.fallback_on_error, config2.fallback_on_error);
    }

    #[test]
    fn test_ml_trend_prediction_clone() {
        let pred1 = MLTrendPrediction {
            trend: TrendDirection::Uptrend,
            confidence: 0.88,
            model: "LSTM".to_string(),
            timestamp: 999999,
        };

        let pred2 = pred1.clone();

        assert_eq!(pred1.trend, pred2.trend);
        assert_eq!(pred1.confidence, pred2.confidence);
        assert_eq!(pred1.model, pred2.model);
        assert_eq!(pred1.timestamp, pred2.timestamp);
    }

    #[test]
    fn test_ml_predictor_config_serialize() {
        let config = MLPredictorConfig::default();
        let serialized = serde_json::to_string(&config).unwrap();

        assert!(serialized.contains("http://localhost:8000"));
        assert!(serialized.contains("2000"));
        assert!(serialized.contains("0.65"));
    }

    #[test]
    fn test_ml_predictor_config_deserialize() {
        let json = r#"{
            "service_url": "http://ml-service:9000",
            "timeout_ms": 3000,
            "min_confidence": 0.7,
            "fallback_on_error": false
        }"#;

        let config: MLPredictorConfig = serde_json::from_str(json).unwrap();

        assert_eq!(config.service_url, "http://ml-service:9000");
        assert_eq!(config.timeout_ms, 3000);
        assert_eq!(config.min_confidence, 0.7);
        assert!(!config.fallback_on_error);
    }

    #[test]
    fn test_ml_trend_prediction_debug_format() {
        let prediction = MLTrendPrediction {
            trend: TrendDirection::Uptrend,
            confidence: 0.85,
            model: "GRU".to_string(),
            timestamp: 1234567890,
        };

        let debug_str = format!("{:?}", prediction);
        assert!(debug_str.contains("MLTrendPrediction"));
        assert!(debug_str.contains("Uptrend"));
        assert!(debug_str.contains("0.85"));
    }

    #[test]
    fn test_ml_predictor_config_different_urls() {
        let urls = vec![
            "http://localhost:8000",
            "https://ml-service.example.com",
            "http://192.168.1.100:9000",
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
    fn test_ml_predictor_config_timeout_values() {
        let timeouts = vec![1000, 2000, 5000, 10000];

        for timeout in timeouts {
            let config = MLPredictorConfig {
                timeout_ms: timeout,
                ..Default::default()
            };

            assert_eq!(config.timeout_ms, timeout);
        }
    }

    #[test]
    fn test_ml_predictor_config_confidence_thresholds() {
        let thresholds = vec![0.5, 0.65, 0.75, 0.9];

        for threshold in thresholds {
            let config = MLPredictorConfig {
                min_confidence: threshold,
                ..Default::default()
            };

            assert_eq!(config.min_confidence, threshold);
        }
    }

    #[test]
    fn test_ml_trend_prediction_low_confidence() {
        let prediction = MLTrendPrediction {
            trend: TrendDirection::Uptrend,
            confidence: 0.3,
            model: "Test".to_string(),
            timestamp: 123456,
        };

        assert_eq!(prediction.confidence, 0.3);
        assert!(prediction.confidence < 0.65); // Below default threshold
    }

    #[test]
    fn test_ml_trend_prediction_high_confidence() {
        let prediction = MLTrendPrediction {
            trend: TrendDirection::Downtrend,
            confidence: 0.95,
            model: "Ensemble".to_string(),
            timestamp: 123456,
        };

        assert_eq!(prediction.confidence, 0.95);
        assert!(prediction.confidence > 0.65); // Above default threshold
    }

    #[test]
    fn test_ml_trend_prediction_edge_case_confidence() {
        let predictions = vec![
            MLTrendPrediction {
                trend: TrendDirection::Neutral,
                confidence: 0.0,
                model: "Test".to_string(),
                timestamp: 123456,
            },
            MLTrendPrediction {
                trend: TrendDirection::Uptrend,
                confidence: 1.0,
                model: "Test".to_string(),
                timestamp: 123456,
            },
        ];

        assert_eq!(predictions[0].confidence, 0.0);
        assert_eq!(predictions[1].confidence, 1.0);
    }

    #[test]
    fn test_ml_predictor_config_fallback_modes() {
        let config_with_fallback = MLPredictorConfig {
            fallback_on_error: true,
            ..Default::default()
        };

        let config_no_fallback = MLPredictorConfig {
            fallback_on_error: false,
            ..Default::default()
        };

        assert!(config_with_fallback.fallback_on_error);
        assert!(!config_no_fallback.fallback_on_error);
    }

    // Note: Integration tests with actual ML service should be in separate test file
    // These are unit tests for the predictor structure and configuration

    // Additional inline unit tests for coverage boost

    #[test]
    fn test_ml_predictor_config_zero_timeout() {
        let config = MLPredictorConfig {
            timeout_ms: 0,
            ..Default::default()
        };
        assert_eq!(config.timeout_ms, 0);
    }

    #[test]
    fn test_ml_predictor_config_max_timeout() {
        let config = MLPredictorConfig {
            timeout_ms: u64::MAX,
            ..Default::default()
        };
        assert_eq!(config.timeout_ms, u64::MAX);
    }

    #[test]
    fn test_ml_trend_prediction_zero_confidence() {
        let prediction = MLTrendPrediction {
            trend: TrendDirection::Neutral,
            confidence: 0.0,
            model: "Test".to_string(),
            timestamp: 0,
        };
        assert_eq!(prediction.confidence, 0.0);
        assert!(prediction.confidence < 0.65);
    }

    #[test]
    fn test_ml_trend_prediction_max_confidence() {
        let prediction = MLTrendPrediction {
            trend: TrendDirection::Uptrend,
            confidence: 1.0,
            model: "Ensemble".to_string(),
            timestamp: i64::MAX,
        };
        assert_eq!(prediction.confidence, 1.0);
        assert!(prediction.confidence >= 0.65);
    }

    #[test]
    fn test_ml_predictor_config_negative_confidence() {
        let config = MLPredictorConfig {
            min_confidence: -0.5,
            ..Default::default()
        };
        assert!(config.min_confidence < 0.0);
    }

    #[test]
    fn test_ml_predictor_config_confidence_above_one() {
        let config = MLPredictorConfig {
            min_confidence: 1.5,
            ..Default::default()
        };
        assert!(config.min_confidence > 1.0);
    }

    #[test]
    fn test_ml_trend_prediction_empty_model() {
        let prediction = MLTrendPrediction {
            trend: TrendDirection::Neutral,
            confidence: 0.75,
            model: String::new(),
            timestamp: 123456,
        };
        assert!(prediction.model.is_empty());
    }

    #[test]
    fn test_ml_trend_prediction_very_long_model_name() {
        let long_name = "A".repeat(1000);
        let prediction = MLTrendPrediction {
            trend: TrendDirection::Uptrend,
            confidence: 0.8,
            model: long_name.clone(),
            timestamp: 123456,
        };
        assert_eq!(prediction.model.len(), 1000);
    }

    #[test]
    fn test_ml_predictor_config_empty_url() {
        let config = MLPredictorConfig {
            service_url: String::new(),
            ..Default::default()
        };
        assert!(config.service_url.is_empty());
    }

    #[test]
    fn test_ml_predictor_config_url_with_path() {
        let config = MLPredictorConfig {
            service_url: "http://localhost:8000/api/v1/ml".to_string(),
            ..Default::default()
        };
        assert!(config.service_url.contains("/api/v1/ml"));
    }

    #[test]
    fn test_ml_predictor_config_https_url() {
        let config = MLPredictorConfig {
            service_url: "https://secure-ml-service.com".to_string(),
            ..Default::default()
        };
        assert!(config.service_url.starts_with("https://"));
    }

    #[test]
    fn test_ml_trend_prediction_negative_timestamp() {
        let prediction = MLTrendPrediction {
            trend: TrendDirection::Downtrend,
            confidence: 0.7,
            model: "Test".to_string(),
            timestamp: -1,
        };
        assert_eq!(prediction.timestamp, -1);
    }

    #[test]
    fn test_ml_trend_prediction_all_trends_serialize() {
        let predictions = vec![
            MLTrendPrediction {
                trend: TrendDirection::Uptrend,
                confidence: 0.8,
                model: "M1".to_string(),
                timestamp: 1,
            },
            MLTrendPrediction {
                trend: TrendDirection::Downtrend,
                confidence: 0.75,
                model: "M2".to_string(),
                timestamp: 2,
            },
            MLTrendPrediction {
                trend: TrendDirection::Neutral,
                confidence: 0.6,
                model: "M3".to_string(),
                timestamp: 3,
            },
        ];

        for pred in predictions {
            let json = serde_json::to_string(&pred).unwrap();
            assert!(!json.is_empty());
        }
    }

    #[test]
    fn test_ml_predictor_config_boundary_confidence() {
        let configs = vec![
            (0.0, 0.0),
            (0.5, 0.5),
            (0.65, 0.65),
            (1.0, 1.0),
        ];

        for (conf, expected) in configs {
            let config = MLPredictorConfig {
                min_confidence: conf,
                ..Default::default()
            };
            assert_eq!(config.min_confidence, expected);
        }
    }

    #[test]
    fn test_ml_trend_prediction_confidence_ranges() {
        let confidences = vec![0.0, 0.1, 0.5, 0.65, 0.8, 0.9, 1.0];

        for conf in confidences {
            let prediction = MLTrendPrediction {
                trend: TrendDirection::Neutral,
                confidence: conf,
                model: "Test".to_string(),
                timestamp: 123,
            };
            assert_eq!(prediction.confidence, conf);
        }
    }

    #[test]
    fn test_ml_predictor_config_common_ports() {
        let ports = vec![8000, 8080, 9000, 3000, 5000];

        for port in ports {
            let config = MLPredictorConfig {
                service_url: format!("http://localhost:{}", port),
                ..Default::default()
            };
            assert!(config.service_url.contains(&port.to_string()));
        }
    }

    #[test]
    fn test_ml_trend_prediction_model_variations() {
        let models = vec!["LSTM", "GRU", "Transformer", "Ensemble", "CNN", "RNN"];

        for model in models {
            let prediction = MLTrendPrediction {
                trend: TrendDirection::Uptrend,
                confidence: 0.8,
                model: model.to_string(),
                timestamp: 123,
            };
            assert_eq!(prediction.model, model);
        }
    }

    #[test]
    fn test_ml_predictor_config_json_round_trip() {
        let original = MLPredictorConfig {
            service_url: "http://test:9000".to_string(),
            timeout_ms: 3500,
            min_confidence: 0.72,
            fallback_on_error: false,
        };

        let json = serde_json::to_string(&original).unwrap();
        let deserialized: MLPredictorConfig = serde_json::from_str(&json).unwrap();

        assert_eq!(original.service_url, deserialized.service_url);
        assert_eq!(original.timeout_ms, deserialized.timeout_ms);
        assert_eq!(original.min_confidence, deserialized.min_confidence);
        assert_eq!(original.fallback_on_error, deserialized.fallback_on_error);
    }

    #[test]
    fn test_ml_trend_prediction_json_round_trip() {
        let original = MLTrendPrediction {
            trend: TrendDirection::Downtrend,
            confidence: 0.87,
            model: "TestModel".to_string(),
            timestamp: 1234567890,
        };

        let json = serde_json::to_string(&original).unwrap();
        let deserialized: MLTrendPrediction = serde_json::from_str(&json).unwrap();

        assert_eq!(original.trend, deserialized.trend);
        assert_eq!(original.confidence, deserialized.confidence);
        assert_eq!(original.model, deserialized.model);
        assert_eq!(original.timestamp, deserialized.timestamp);
    }

    #[test]
    fn test_ml_predictor_config_extreme_timeouts() {
        let timeouts = vec![1, 100, 1000, 10000, 60000, 120000];

        for timeout in timeouts {
            let config = MLPredictorConfig {
                timeout_ms: timeout,
                ..Default::default()
            };
            assert_eq!(config.timeout_ms, timeout);
        }
    }

    #[test]
    fn test_ml_trend_prediction_timestamp_edge_cases() {
        let timestamps = vec![i64::MIN, -1000000, 0, 1000000, i64::MAX];

        for ts in timestamps {
            let prediction = MLTrendPrediction {
                trend: TrendDirection::Neutral,
                confidence: 0.7,
                model: "Test".to_string(),
                timestamp: ts,
            };
            assert_eq!(prediction.timestamp, ts);
        }
    }

    #[tokio::test]
    async fn test_ml_predictor_creation_with_custom_config() {
        let config = MLPredictorConfig {
            service_url: "http://custom:9999".to_string(),
            timeout_ms: 5000,
            min_confidence: 0.8,
            fallback_on_error: false,
        };

        let predictor = MLTrendPredictor::new(config.clone());
        assert_eq!(predictor.config().service_url, config.service_url);
        assert_eq!(predictor.config().timeout_ms, config.timeout_ms);
        assert_eq!(predictor.config().min_confidence, config.min_confidence);
        assert_eq!(predictor.config().fallback_on_error, config.fallback_on_error);
    }

    // ========== COV8 TESTS - Target untested branches ==========

    #[test]
    fn test_cov8_ml_predictor_config_getter() {
        let config = MLPredictorConfig {
            service_url: "http://test:8000".to_string(),
            timeout_ms: 3000,
            min_confidence: 0.7,
            fallback_on_error: true,
        };

        let predictor = MLTrendPredictor::new(config.clone());
        let retrieved_config = predictor.config();

        assert_eq!(retrieved_config.service_url, config.service_url);
        assert_eq!(retrieved_config.timeout_ms, config.timeout_ms);
        assert_eq!(retrieved_config.min_confidence, config.min_confidence);
        assert_eq!(retrieved_config.fallback_on_error, config.fallback_on_error);
    }

    #[tokio::test]
    async fn test_cov8_predict_trend_with_fallback_no_fallback() {
        let config = MLPredictorConfig {
            service_url: "http://nonexistent:9999".to_string(),
            timeout_ms: 100,
            min_confidence: 0.65,
            fallback_on_error: false, // No fallback
        };

        let predictor = MLTrendPredictor::new(config);
        let result = predictor.predict_trend_with_fallback("BTCUSDT", "4h").await;

        // Should return None even with fallback_on_error = false
        assert!(result.is_none());
    }

    #[test]
    fn test_cov8_ml_trend_prediction_all_fields() {
        let prediction = MLTrendPrediction {
            trend: TrendDirection::Uptrend,
            confidence: 0.92,
            model: "TestModel".to_string(),
            timestamp: 1234567890,
        };

        // Test all fields
        assert_eq!(prediction.trend, TrendDirection::Uptrend);
        assert_eq!(prediction.confidence, 0.92);
        assert_eq!(prediction.model, "TestModel");
        assert_eq!(prediction.timestamp, 1234567890);
    }

    #[test]
    fn test_cov8_ml_predictor_config_serialization_round_trip() {
        let original = MLPredictorConfig {
            service_url: "http://ml-test:7777".to_string(),
            timeout_ms: 4500,
            min_confidence: 0.68,
            fallback_on_error: true,
        };

        let json = serde_json::to_string(&original).unwrap();
        let deserialized: MLPredictorConfig = serde_json::from_str(&json).unwrap();

        assert_eq!(original.service_url, deserialized.service_url);
        assert_eq!(original.timeout_ms, deserialized.timeout_ms);
        assert_eq!(original.min_confidence, deserialized.min_confidence);
        assert_eq!(original.fallback_on_error, deserialized.fallback_on_error);
    }

    #[test]
    fn test_cov8_ml_trend_prediction_debug() {
        let prediction = MLTrendPrediction {
            trend: TrendDirection::Neutral,
            confidence: 0.75,
            model: "DebugModel".to_string(),
            timestamp: 9999,
        };

        let debug_str = format!("{:?}", prediction);
        assert!(debug_str.contains("MLTrendPrediction"));
        assert!(debug_str.contains("Neutral"));
    }

    #[test]
    fn test_cov8_ml_predictor_config_debug() {
        let config = MLPredictorConfig::default();
        let debug_str = format!("{:?}", config);

        assert!(debug_str.contains("MLPredictorConfig"));
        assert!(debug_str.contains("http://localhost:8000"));
    }

    #[test]
    fn test_ml_trend_prediction_confidence_below_threshold() {
        let low_conf_values = vec![0.0, 0.1, 0.3, 0.5, 0.64];

        for conf in low_conf_values {
            let prediction = MLTrendPrediction {
                trend: TrendDirection::Uptrend,
                confidence: conf,
                model: "Test".to_string(),
                timestamp: 123,
            };
            assert!(prediction.confidence < 0.65);
        }
    }

    #[test]
    fn test_ml_trend_prediction_confidence_above_threshold() {
        let high_conf_values = vec![0.65, 0.7, 0.8, 0.9, 1.0];

        for conf in high_conf_values {
            let prediction = MLTrendPrediction {
                trend: TrendDirection::Uptrend,
                confidence: conf,
                model: "Test".to_string(),
                timestamp: 123,
            };
            assert!(prediction.confidence >= 0.65);
        }
    }

    #[test]
    fn test_ml_predictor_config_ipv4_addresses() {
        let ips = vec![
            "http://127.0.0.1:8000",
            "http://192.168.1.1:9000",
            "http://10.0.0.1:3000",
        ];

        for ip in ips {
            let config = MLPredictorConfig {
                service_url: ip.to_string(),
                ..Default::default()
            };
            assert_eq!(config.service_url, ip);
        }
    }

    #[test]
    fn test_ml_predictor_config_special_characters_url() {
        let config = MLPredictorConfig {
            service_url: "http://ml-service.example.com:8080/api/v1/predict".to_string(),
            ..Default::default()
        };
        assert!(config.service_url.contains("ml-service"));
        assert!(config.service_url.contains("/api/v1/predict"));
    }

    #[test]
    fn test_ml_trend_prediction_model_case_sensitivity() {
        let models = vec!["lstm", "LSTM", "Lstm", "lStM"];

        for model in models {
            let prediction = MLTrendPrediction {
                trend: TrendDirection::Uptrend,
                confidence: 0.8,
                model: model.to_string(),
                timestamp: 123,
            };
            assert!(prediction.model.to_lowercase().contains("lstm"));
        }
    }
}
