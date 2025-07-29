use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// AI analysis request to Python service
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AIAnalysisRequest {
    pub symbol: String,
    pub timeframe_data: HashMap<String, Vec<crate::market_data::cache::CandleData>>,
    pub current_price: f64,
    pub volume_24h: f64,
    pub timestamp: i64,
    pub strategy_context: StrategyContext,
}

/// Strategy context for AI analysis
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StrategyContext {
    pub selected_strategies: Vec<String>,
    pub market_condition: MarketCondition,
    pub risk_level: RiskLevel,
    pub technical_indicators: HashMap<String, serde_json::Value>,
}

/// Market condition assessment
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MarketCondition {
    Trending,
    Ranging,
    Volatile,
    LowVolume,
    Unknown,
}

/// Risk level for trading
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RiskLevel {
    Conservative,
    Moderate,
    Aggressive,
}

/// AI analysis response from Python service
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AIAnalysisResponse {
    pub signal: super::TradingSignal,
    pub confidence: f64,
    pub reasoning: String,
    pub strategy_scores: HashMap<String, f64>,
    pub market_analysis: MarketAnalysis,
    pub risk_assessment: RiskAssessment,
    pub timestamp: i64,
}

/// Detailed market analysis from AI
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MarketAnalysis {
    pub trend_direction: TrendDirection,
    pub trend_strength: f64,
    pub support_levels: Vec<f64>,
    pub resistance_levels: Vec<f64>,
    pub volatility_assessment: VolatilityAssessment,
    pub volume_analysis: VolumeAnalysis,
}

/// Trend direction assessment
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TrendDirection {
    Bullish,
    Bearish,
    Sideways,
    Uncertain,
}

/// Volatility assessment
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VolatilityAssessment {
    pub level: VolatilityLevel,
    pub trend: VolatilityTrend,
    pub percentile: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum VolatilityLevel {
    VeryLow,
    Low,
    Normal,
    High,
    VeryHigh,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum VolatilityTrend {
    Increasing,
    Decreasing,
    Stable,
}

/// Volume analysis
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VolumeAnalysis {
    pub relative_volume: f64,
    pub trend: VolumeTrend,
    pub accumulation_distribution: AccumulationDistribution,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum VolumeTrend {
    Increasing,
    Decreasing,
    Stable,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AccumulationDistribution {
    Accumulation,
    Distribution,
    Neutral,
}

/// Risk assessment from AI
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RiskAssessment {
    pub overall_risk: RiskLevel,
    pub technical_risk: f64,
    pub market_risk: f64,
    pub liquidity_risk: f64,
    pub recommended_position_size: f64,
    pub stop_loss_suggestion: Option<f64>,
    pub take_profit_suggestion: Option<f64>,
}

/// Strategy performance metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StrategyPerformance {
    pub strategy_name: String,
    pub total_signals: u64,
    pub successful_signals: u64,
    pub accuracy: f64,
    pub average_confidence: f64,
    pub profit_factor: f64,
    pub max_drawdown: f64,
    pub sharpe_ratio: f64,
    pub last_updated: i64,
}

/// Signal quality metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SignalQuality {
    pub accuracy_score: f64,
    pub consistency_score: f64,
    pub timeliness_score: f64,
    pub risk_adjusted_return: f64,
    pub overall_quality: SignalQualityRating,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SignalQualityRating {
    Excellent,
    Good,
    Average,
    Poor,
    VeryPoor,
}

impl Default for MarketCondition {
    fn default() -> Self {
        MarketCondition::Unknown
    }
}

impl Default for RiskLevel {
    fn default() -> Self {
        RiskLevel::Moderate
    }
}

impl Default for StrategyContext {
    fn default() -> Self {
        Self {
            selected_strategies: vec![
                "RSI Strategy".to_string(),
                "MACD Strategy".to_string(),
                "Volume Strategy".to_string(),
                "Bollinger Bands Strategy".to_string(),
            ],
            market_condition: MarketCondition::Unknown,
            risk_level: RiskLevel::Moderate,
            technical_indicators: HashMap::new(),
        }
    }
}

impl std::fmt::Display for super::TradingSignal {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.to_string())
    }
}

impl std::fmt::Display for MarketCondition {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            MarketCondition::Trending => "Trending",
            MarketCondition::Ranging => "Ranging",
            MarketCondition::Volatile => "Volatile",
            MarketCondition::LowVolume => "Low Volume",
            MarketCondition::Unknown => "Unknown",
        };
        write!(f, "{}", s)
    }
}

impl std::fmt::Display for RiskLevel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            RiskLevel::Conservative => "Conservative",
            RiskLevel::Moderate => "Moderate",
            RiskLevel::Aggressive => "Aggressive",
        };
        write!(f, "{}", s)
    }
}
