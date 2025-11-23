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

    // Note: Integration tests with actual ML service should be in separate test file
    // These are unit tests for the predictor structure and configuration
}
