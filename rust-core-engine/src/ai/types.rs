use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Error types for AI operations
#[allow(dead_code)]
#[derive(Debug, thiserror::Error)]
pub enum AIError {
    #[error("Network error: {0}")]
    Network(String),

    #[error("Service unavailable: {0}")]
    ServiceUnavailable(String),

    #[error("Invalid request: {0}")]
    InvalidRequest(String),

    #[error("Analysis failed: {0}")]
    AnalysisFailed(String),

    #[error("Timeout error: {0}")]
    Timeout(String),

    #[error("Parsing error: {0}")]
    Parsing(String),
}

/// AI analysis status
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub enum AIAnalysisStatus {
    #[default]
    Pending,
    Processing,
    Completed,
    Failed,
}

/// AI confidence levels
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AIConfidenceLevel {
    VeryLow,  // 0.0 - 0.2
    Low,      // 0.2 - 0.4
    Medium,   // 0.4 - 0.6
    High,     // 0.6 - 0.8
    VeryHigh, // 0.8 - 1.0
}

impl AIConfidenceLevel {
    pub fn from_score(score: f64) -> Self {
        match score {
            s if s < 0.2 => Self::VeryLow,
            s if s < 0.4 => Self::Low,
            s if s < 0.6 => Self::Medium,
            s if s < 0.8 => Self::High,
            _ => Self::VeryHigh,
        }
    }

    pub fn to_score_range(&self) -> (f64, f64) {
        match self {
            Self::VeryLow => (0.0, 0.2),
            Self::Low => (0.2, 0.4),
            Self::Medium => (0.4, 0.6),
            Self::High => (0.6, 0.8),
            Self::VeryHigh => (0.8, 1.0),
        }
    }
}

/// AI prediction metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AIPredictionMetadata {
    pub model_version: String,
    pub features_used: Vec<String>,
    pub confidence_level: AIConfidenceLevel,
    pub processing_time_ms: u64,
    pub data_quality_score: f64,
    pub prediction_id: String,
}

/// Real-time AI signal update
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AISignalUpdate {
    pub signal_id: String,
    pub symbol: String,
    pub signal: crate::strategies::TradingSignal,
    pub confidence: f64,
    pub updated_reasoning: String,
    pub metadata: AIPredictionMetadata,
    pub timestamp: i64,
}

/// AI learning feedback types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AIFeedbackType {
    SignalAccuracy,
    ConfidenceCalibration,
    StrategyPerformance,
    MarketConditionDetection,
    RiskAssessment,
}

/// Enhanced performance feedback
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnhancedPerformanceFeedback {
    pub feedback_type: AIFeedbackType,
    pub signal_id: String,
    pub symbol: String,
    pub timeframe: String,
    pub predicted_signal: crate::strategies::TradingSignal,
    pub predicted_confidence: f64,
    pub actual_outcome: String,
    pub profit_loss_percentage: f64,
    pub holding_period_hours: f64,
    pub market_conditions_during: HashMap<String, String>,
    pub user_rating: Option<u8>, // 1-5 rating from user
    pub feedback_notes: Option<String>,
    pub timestamp: i64,
}

/// AI model training status
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AIModelTrainingStatus {
    pub is_training: bool,
    pub training_progress: f64,
    pub estimated_completion: Option<i64>,
    pub current_epoch: Option<u32>,
    pub total_epochs: Option<u32>,
    pub validation_accuracy: Option<f64>,
    pub training_loss: Option<f64>,
}

/// AI service health metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AIServiceHealth {
    pub status: String,
    pub uptime_seconds: u64,
    pub requests_processed: u64,
    pub error_rate: f64,
    pub average_response_time_ms: f64,
    pub memory_usage_percent: f64,
    pub cpu_usage_percent: f64,
    pub model_loaded: bool,
    pub last_health_check: i64,
}

/// AI strategy optimization suggestion
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AIStrategyOptimization {
    pub strategy_name: String,
    pub current_parameters: HashMap<String, serde_json::Value>,
    pub suggested_parameters: HashMap<String, serde_json::Value>,
    pub expected_improvement: f64,
    pub confidence: f64,
    pub reasoning: String,
    pub backtesting_results: Option<HashMap<String, f64>>,
}

/// AI market regime detection
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AIMarketRegime {
    pub regime_type: String, // "bull", "bear", "sideways", "volatile", "low_volatility"
    pub confidence: f64,
    pub characteristics: Vec<String>,
    pub duration_estimate_hours: Option<f64>,
    pub suitable_strategies: Vec<String>,
    pub risk_factors: Vec<String>,
    pub detected_at: i64,
}

/// AI anomaly detection result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AIAnomalyDetection {
    pub anomaly_type: String,
    pub severity: String, // "low", "medium", "high", "critical"
    pub description: String,
    pub affected_symbols: Vec<String>,
    pub confidence: f64,
    pub recommended_actions: Vec<String>,
    pub detected_at: i64,
}

/// AI backtesting request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AIBacktestingRequest {
    pub strategy_name: String,
    pub strategy_parameters: HashMap<String, serde_json::Value>,
    pub symbol: String,
    pub start_date: String,
    pub end_date: String,
    pub initial_capital: f64,
    pub commission_rate: f64,
    pub slippage_rate: f64,
}

/// AI backtesting results
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AIBacktestingResults {
    pub strategy_name: String,
    pub symbol: String,
    pub period: String,
    pub total_return: f64,
    pub annualized_return: f64,
    pub max_drawdown: f64,
    pub sharpe_ratio: f64,
    pub win_rate: f64,
    pub profit_factor: f64,
    pub total_trades: u32,
    pub winning_trades: u32,
    pub losing_trades: u32,
    pub average_trade_return: f64,
    pub largest_winning_trade: f64,
    pub largest_losing_trade: f64,
    pub detailed_trades: Vec<AITradeResult>,
    pub equity_curve: Vec<AIEquityPoint>,
}

/// Individual trade result from backtesting
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AITradeResult {
    pub entry_time: i64,
    pub exit_time: i64,
    pub entry_price: f64,
    pub exit_price: f64,
    pub side: String, // "long" or "short"
    pub quantity: f64,
    pub profit_loss: f64,
    pub profit_loss_percentage: f64,
    pub commission_paid: f64,
    pub holding_period_hours: f64,
}

/// Equity curve point for backtesting
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AIEquityPoint {
    pub timestamp: i64,
    pub equity: f64,
    pub drawdown: f64,
    pub open_positions: u32,
}

impl std::fmt::Display for AIConfidenceLevel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            AIConfidenceLevel::VeryLow => "Very Low",
            AIConfidenceLevel::Low => "Low",
            AIConfidenceLevel::Medium => "Medium",
            AIConfidenceLevel::High => "High",
            AIConfidenceLevel::VeryHigh => "Very High",
        };
        write!(f, "{s}")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json;

    #[test]
    fn test_ai_confidence_level_from_score() {
        assert!(matches!(AIConfidenceLevel::from_score(0.1), AIConfidenceLevel::VeryLow));
        assert!(matches!(AIConfidenceLevel::from_score(0.3), AIConfidenceLevel::Low));
        assert!(matches!(AIConfidenceLevel::from_score(0.5), AIConfidenceLevel::Medium));
        assert!(matches!(AIConfidenceLevel::from_score(0.7), AIConfidenceLevel::High));
        assert!(matches!(AIConfidenceLevel::from_score(0.9), AIConfidenceLevel::VeryHigh));

        // Edge cases
        assert!(matches!(AIConfidenceLevel::from_score(0.0), AIConfidenceLevel::VeryLow));
        assert!(matches!(AIConfidenceLevel::from_score(0.2), AIConfidenceLevel::Low));
        assert!(matches!(AIConfidenceLevel::from_score(0.4), AIConfidenceLevel::Medium));
        assert!(matches!(AIConfidenceLevel::from_score(0.6), AIConfidenceLevel::High));
        assert!(matches!(AIConfidenceLevel::from_score(0.8), AIConfidenceLevel::VeryHigh));
        assert!(matches!(AIConfidenceLevel::from_score(1.0), AIConfidenceLevel::VeryHigh));
    }

    #[test]
    fn test_ai_confidence_level_to_score_range() {
        assert_eq!(AIConfidenceLevel::VeryLow.to_score_range(), (0.0, 0.2));
        assert_eq!(AIConfidenceLevel::Low.to_score_range(), (0.2, 0.4));
        assert_eq!(AIConfidenceLevel::Medium.to_score_range(), (0.4, 0.6));
        assert_eq!(AIConfidenceLevel::High.to_score_range(), (0.6, 0.8));
        assert_eq!(AIConfidenceLevel::VeryHigh.to_score_range(), (0.8, 1.0));
    }

    #[test]
    fn test_ai_confidence_level_display() {
        assert_eq!(format!("{}", AIConfidenceLevel::VeryLow), "Very Low");
        assert_eq!(format!("{}", AIConfidenceLevel::Low), "Low");
        assert_eq!(format!("{}", AIConfidenceLevel::Medium), "Medium");
        assert_eq!(format!("{}", AIConfidenceLevel::High), "High");
        assert_eq!(format!("{}", AIConfidenceLevel::VeryHigh), "Very High");
    }

    #[test]
    fn test_ai_error_display() {
        let network_error = AIError::Network("connection failed".to_string());
        assert_eq!(format!("{}", network_error), "Network error: connection failed");

        let service_error = AIError::ServiceUnavailable("service down".to_string());
        assert_eq!(format!("{}", service_error), "Service unavailable: service down");

        let invalid_error = AIError::InvalidRequest("bad params".to_string());
        assert_eq!(format!("{}", invalid_error), "Invalid request: bad params");

        let analysis_error = AIError::AnalysisFailed("no data".to_string());
        assert_eq!(format!("{}", analysis_error), "Analysis failed: no data");

        let timeout_error = AIError::Timeout("took too long".to_string());
        assert_eq!(format!("{}", timeout_error), "Timeout error: took too long");

        let parsing_error = AIError::Parsing("invalid json".to_string());
        assert_eq!(format!("{}", parsing_error), "Parsing error: invalid json");
    }

    #[test]
    fn test_ai_analysis_status_default() {
        let status: AIAnalysisStatus = Default::default();
        assert!(matches!(status, AIAnalysisStatus::Pending));
    }

    #[test]
    fn test_ai_analysis_status_serialization() {
        let pending = AIAnalysisStatus::Pending;
        let json = serde_json::to_string(&pending).unwrap();
        let deserialized: AIAnalysisStatus = serde_json::from_str(&json).unwrap();
        assert!(matches!(deserialized, AIAnalysisStatus::Pending));

        let processing = AIAnalysisStatus::Processing;
        let json = serde_json::to_string(&processing).unwrap();
        let deserialized: AIAnalysisStatus = serde_json::from_str(&json).unwrap();
        assert!(matches!(deserialized, AIAnalysisStatus::Processing));
    }

    #[test]
    fn test_ai_confidence_level_serialization() {
        let high = AIConfidenceLevel::High;
        let json = serde_json::to_string(&high).unwrap();
        let deserialized: AIConfidenceLevel = serde_json::from_str(&json).unwrap();
        assert!(matches!(deserialized, AIConfidenceLevel::High));
    }

    #[test]
    fn test_ai_prediction_metadata_serialization() {
        let metadata = AIPredictionMetadata {
            model_version: "1.0.0".to_string(),
            features_used: vec!["RSI".to_string(), "MACD".to_string()],
            confidence_level: AIConfidenceLevel::High,
            processing_time_ms: 150,
            data_quality_score: 0.95,
            prediction_id: "pred_123".to_string(),
        };

        let json = serde_json::to_string(&metadata).unwrap();
        let deserialized: AIPredictionMetadata = serde_json::from_str(&json).unwrap();

        assert_eq!(deserialized.model_version, "1.0.0");
        assert_eq!(deserialized.features_used.len(), 2);
        assert_eq!(deserialized.processing_time_ms, 150);
        assert_eq!(deserialized.data_quality_score, 0.95);
        assert_eq!(deserialized.prediction_id, "pred_123");
    }

    #[test]
    fn test_ai_service_health_serialization() {
        let health = AIServiceHealth {
            status: "healthy".to_string(),
            uptime_seconds: 3600,
            requests_processed: 1000,
            error_rate: 0.01,
            average_response_time_ms: 250.5,
            memory_usage_percent: 45.2,
            cpu_usage_percent: 30.8,
            model_loaded: true,
            last_health_check: 1234567890,
        };

        let json = serde_json::to_string(&health).unwrap();
        let deserialized: AIServiceHealth = serde_json::from_str(&json).unwrap();

        assert_eq!(deserialized.status, "healthy");
        assert_eq!(deserialized.uptime_seconds, 3600);
        assert_eq!(deserialized.requests_processed, 1000);
        assert_eq!(deserialized.error_rate, 0.01);
        assert!(deserialized.model_loaded);
    }

    #[test]
    fn test_ai_market_regime_serialization() {
        let regime = AIMarketRegime {
            regime_type: "bull".to_string(),
            confidence: 0.85,
            characteristics: vec!["uptrend".to_string(), "high_volume".to_string()],
            duration_estimate_hours: Some(24.0),
            suitable_strategies: vec!["trend_following".to_string()],
            risk_factors: vec!["overbought".to_string()],
            detected_at: 1234567890,
        };

        let json = serde_json::to_string(&regime).unwrap();
        let deserialized: AIMarketRegime = serde_json::from_str(&json).unwrap();

        assert_eq!(deserialized.regime_type, "bull");
        assert_eq!(deserialized.confidence, 0.85);
        assert_eq!(deserialized.characteristics.len(), 2);
        assert_eq!(deserialized.duration_estimate_hours, Some(24.0));
    }

    #[test]
    fn test_ai_anomaly_detection_serialization() {
        let anomaly = AIAnomalyDetection {
            anomaly_type: "sudden_price_drop".to_string(),
            severity: "high".to_string(),
            description: "Price dropped 10% in 5 minutes".to_string(),
            affected_symbols: vec!["BTCUSDT".to_string()],
            confidence: 0.92,
            recommended_actions: vec!["close_positions".to_string(), "wait".to_string()],
            detected_at: 1234567890,
        };

        let json = serde_json::to_string(&anomaly).unwrap();
        let deserialized: AIAnomalyDetection = serde_json::from_str(&json).unwrap();

        assert_eq!(deserialized.anomaly_type, "sudden_price_drop");
        assert_eq!(deserialized.severity, "high");
        assert_eq!(deserialized.confidence, 0.92);
        assert_eq!(deserialized.affected_symbols.len(), 1);
    }

    #[test]
    fn test_ai_backtesting_results_serialization() {
        let results = AIBacktestingResults {
            strategy_name: "RSI Strategy".to_string(),
            symbol: "BTCUSDT".to_string(),
            period: "2024-01-01 to 2024-12-31".to_string(),
            total_return: 0.25,
            annualized_return: 0.25,
            max_drawdown: -0.15,
            sharpe_ratio: 1.8,
            win_rate: 0.65,
            profit_factor: 2.1,
            total_trades: 100,
            winning_trades: 65,
            losing_trades: 35,
            average_trade_return: 0.0025,
            largest_winning_trade: 0.05,
            largest_losing_trade: -0.03,
            detailed_trades: vec![],
            equity_curve: vec![],
        };

        let json = serde_json::to_string(&results).unwrap();
        let deserialized: AIBacktestingResults = serde_json::from_str(&json).unwrap();

        assert_eq!(deserialized.strategy_name, "RSI Strategy");
        assert_eq!(deserialized.total_trades, 100);
        assert_eq!(deserialized.win_rate, 0.65);
        assert_eq!(deserialized.sharpe_ratio, 1.8);
    }

    #[test]
    fn test_ai_trade_result_serialization() {
        let trade = AITradeResult {
            entry_time: 1234567890,
            exit_time: 1234571490,
            entry_price: 50000.0,
            exit_price: 51000.0,
            side: "long".to_string(),
            quantity: 0.1,
            profit_loss: 100.0,
            profit_loss_percentage: 0.02,
            commission_paid: 5.0,
            holding_period_hours: 1.0,
        };

        let json = serde_json::to_string(&trade).unwrap();
        let deserialized: AITradeResult = serde_json::from_str(&json).unwrap();

        assert_eq!(deserialized.side, "long");
        assert_eq!(deserialized.profit_loss, 100.0);
        assert_eq!(deserialized.profit_loss_percentage, 0.02);
    }

    #[test]
    fn test_ai_equity_point_serialization() {
        let equity_point = AIEquityPoint {
            timestamp: 1234567890,
            equity: 10500.0,
            drawdown: -0.05,
            open_positions: 2,
        };

        let json = serde_json::to_string(&equity_point).unwrap();
        let deserialized: AIEquityPoint = serde_json::from_str(&json).unwrap();

        assert_eq!(deserialized.equity, 10500.0);
        assert_eq!(deserialized.drawdown, -0.05);
        assert_eq!(deserialized.open_positions, 2);
    }

    #[test]
    fn test_ai_model_training_status_serialization() {
        let status = AIModelTrainingStatus {
            is_training: true,
            training_progress: 0.75,
            estimated_completion: Some(1234567890),
            current_epoch: Some(75),
            total_epochs: Some(100),
            validation_accuracy: Some(0.85),
            training_loss: Some(0.12),
        };

        let json = serde_json::to_string(&status).unwrap();
        let deserialized: AIModelTrainingStatus = serde_json::from_str(&json).unwrap();

        assert!(deserialized.is_training);
        assert_eq!(deserialized.training_progress, 0.75);
        assert_eq!(deserialized.current_epoch, Some(75));
        assert_eq!(deserialized.total_epochs, Some(100));
    }

    #[test]
    fn test_ai_feedback_type_serialization() {
        let feedback_type = AIFeedbackType::SignalAccuracy;
        let json = serde_json::to_string(&feedback_type).unwrap();
        let deserialized: AIFeedbackType = serde_json::from_str(&json).unwrap();
        assert!(matches!(deserialized, AIFeedbackType::SignalAccuracy));
    }

    #[test]
    fn test_ai_strategy_optimization_serialization() {
        let mut current_params = HashMap::new();
        current_params.insert("period".to_string(), serde_json::json!(14));

        let mut suggested_params = HashMap::new();
        suggested_params.insert("period".to_string(), serde_json::json!(20));

        let optimization = AIStrategyOptimization {
            strategy_name: "RSI Strategy".to_string(),
            current_parameters: current_params,
            suggested_parameters: suggested_params,
            expected_improvement: 0.15,
            confidence: 0.8,
            reasoning: "Better performance in current market".to_string(),
            backtesting_results: None,
        };

        let json = serde_json::to_string(&optimization).unwrap();
        let deserialized: AIStrategyOptimization = serde_json::from_str(&json).unwrap();

        assert_eq!(deserialized.strategy_name, "RSI Strategy");
        assert_eq!(deserialized.expected_improvement, 0.15);
    }
}
