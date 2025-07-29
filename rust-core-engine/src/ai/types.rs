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
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AIAnalysisStatus {
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
        write!(f, "{}", s)
    }
}

impl Default for AIAnalysisStatus {
    fn default() -> Self {
        AIAnalysisStatus::Pending
    }
}
