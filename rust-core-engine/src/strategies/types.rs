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
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub enum MarketCondition {
    Trending,
    Ranging,
    Volatile,
    LowVolume,
    #[default]
    Unknown,
}

/// Risk level for trading
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub enum RiskLevel {
    Conservative,
    #[default]
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
        write!(f, "{}", self.as_str())
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
        write!(f, "{s}")
    }
}

impl std::fmt::Display for RiskLevel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            RiskLevel::Conservative => "Conservative",
            RiskLevel::Moderate => "Moderate",
            RiskLevel::Aggressive => "Aggressive",
        };
        write!(f, "{s}")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json;

    // ==================== MarketCondition Tests ====================

    #[test]
    fn test_market_condition_default() {
        let mc: MarketCondition = Default::default();
        assert!(matches!(mc, MarketCondition::Unknown));
    }

    #[test]
    fn test_market_condition_display_trending() {
        let mc = MarketCondition::Trending;
        assert_eq!(format!("{}", mc), "Trending");
    }

    #[test]
    fn test_market_condition_display_ranging() {
        let mc = MarketCondition::Ranging;
        assert_eq!(format!("{}", mc), "Ranging");
    }

    #[test]
    fn test_market_condition_display_volatile() {
        let mc = MarketCondition::Volatile;
        assert_eq!(format!("{}", mc), "Volatile");
    }

    #[test]
    fn test_market_condition_display_low_volume() {
        let mc = MarketCondition::LowVolume;
        assert_eq!(format!("{}", mc), "Low Volume");
    }

    #[test]
    fn test_market_condition_display_unknown() {
        let mc = MarketCondition::Unknown;
        assert_eq!(format!("{}", mc), "Unknown");
    }

    #[test]
    fn test_market_condition_clone() {
        let mc1 = MarketCondition::Trending;
        let mc2 = mc1.clone();
        assert!(matches!(mc2, MarketCondition::Trending));
    }

    #[test]
    fn test_market_condition_serialize() {
        let mc = MarketCondition::Volatile;
        let json = serde_json::to_string(&mc).unwrap();
        assert_eq!(json, "\"Volatile\"");
    }

    #[test]
    fn test_market_condition_deserialize() {
        let json = "\"Trending\"";
        let mc: MarketCondition = serde_json::from_str(json).unwrap();
        assert!(matches!(mc, MarketCondition::Trending));
    }

    // ==================== RiskLevel Tests ====================

    #[test]
    fn test_risk_level_default() {
        let rl: RiskLevel = Default::default();
        assert!(matches!(rl, RiskLevel::Moderate));
    }

    #[test]
    fn test_risk_level_display_conservative() {
        let rl = RiskLevel::Conservative;
        assert_eq!(format!("{}", rl), "Conservative");
    }

    #[test]
    fn test_risk_level_display_moderate() {
        let rl = RiskLevel::Moderate;
        assert_eq!(format!("{}", rl), "Moderate");
    }

    #[test]
    fn test_risk_level_display_aggressive() {
        let rl = RiskLevel::Aggressive;
        assert_eq!(format!("{}", rl), "Aggressive");
    }

    #[test]
    fn test_risk_level_clone() {
        let rl1 = RiskLevel::Conservative;
        let rl2 = rl1.clone();
        assert!(matches!(rl2, RiskLevel::Conservative));
    }

    #[test]
    fn test_risk_level_serialize() {
        let rl = RiskLevel::Aggressive;
        let json = serde_json::to_string(&rl).unwrap();
        assert_eq!(json, "\"Aggressive\"");
    }

    #[test]
    fn test_risk_level_deserialize() {
        let json = "\"Conservative\"";
        let rl: RiskLevel = serde_json::from_str(json).unwrap();
        assert!(matches!(rl, RiskLevel::Conservative));
    }

    // ==================== TrendDirection Tests ====================

    #[test]
    fn test_trend_direction_variants() {
        let td1 = TrendDirection::Bullish;
        let td2 = TrendDirection::Bearish;
        let td3 = TrendDirection::Sideways;
        let td4 = TrendDirection::Uncertain;

        assert!(matches!(td1, TrendDirection::Bullish));
        assert!(matches!(td2, TrendDirection::Bearish));
        assert!(matches!(td3, TrendDirection::Sideways));
        assert!(matches!(td4, TrendDirection::Uncertain));
    }

    #[test]
    fn test_trend_direction_serialize() {
        let td = TrendDirection::Bullish;
        let json = serde_json::to_string(&td).unwrap();
        assert_eq!(json, "\"Bullish\"");
    }

    #[test]
    fn test_trend_direction_deserialize() {
        let json = "\"Bearish\"";
        let td: TrendDirection = serde_json::from_str(json).unwrap();
        assert!(matches!(td, TrendDirection::Bearish));
    }

    #[test]
    fn test_trend_direction_clone() {
        let td1 = TrendDirection::Sideways;
        let td2 = td1.clone();
        assert!(matches!(td2, TrendDirection::Sideways));
    }

    // ==================== VolatilityLevel Tests ====================

    #[test]
    fn test_volatility_level_variants() {
        let levels = vec![
            VolatilityLevel::VeryLow,
            VolatilityLevel::Low,
            VolatilityLevel::Normal,
            VolatilityLevel::High,
            VolatilityLevel::VeryHigh,
        ];
        assert_eq!(levels.len(), 5);
    }

    #[test]
    fn test_volatility_level_serialize() {
        let vl = VolatilityLevel::High;
        let json = serde_json::to_string(&vl).unwrap();
        assert_eq!(json, "\"High\"");
    }

    #[test]
    fn test_volatility_level_deserialize() {
        let json = "\"VeryLow\"";
        let vl: VolatilityLevel = serde_json::from_str(json).unwrap();
        assert!(matches!(vl, VolatilityLevel::VeryLow));
    }

    // ==================== VolatilityTrend Tests ====================

    #[test]
    fn test_volatility_trend_variants() {
        let vt1 = VolatilityTrend::Increasing;
        let vt2 = VolatilityTrend::Decreasing;
        let vt3 = VolatilityTrend::Stable;

        assert!(matches!(vt1, VolatilityTrend::Increasing));
        assert!(matches!(vt2, VolatilityTrend::Decreasing));
        assert!(matches!(vt3, VolatilityTrend::Stable));
    }

    #[test]
    fn test_volatility_trend_serialize() {
        let vt = VolatilityTrend::Stable;
        let json = serde_json::to_string(&vt).unwrap();
        assert_eq!(json, "\"Stable\"");
    }

    #[test]
    fn test_volatility_trend_deserialize() {
        let json = "\"Increasing\"";
        let vt: VolatilityTrend = serde_json::from_str(json).unwrap();
        assert!(matches!(vt, VolatilityTrend::Increasing));
    }

    // ==================== VolumeTrend Tests ====================

    #[test]
    fn test_volume_trend_variants() {
        let vt1 = VolumeTrend::Increasing;
        let vt2 = VolumeTrend::Decreasing;
        let vt3 = VolumeTrend::Stable;

        assert!(matches!(vt1, VolumeTrend::Increasing));
        assert!(matches!(vt2, VolumeTrend::Decreasing));
        assert!(matches!(vt3, VolumeTrend::Stable));
    }

    #[test]
    fn test_volume_trend_serialize() {
        let vt = VolumeTrend::Decreasing;
        let json = serde_json::to_string(&vt).unwrap();
        assert_eq!(json, "\"Decreasing\"");
    }

    #[test]
    fn test_volume_trend_deserialize() {
        let json = "\"Stable\"";
        let vt: VolumeTrend = serde_json::from_str(json).unwrap();
        assert!(matches!(vt, VolumeTrend::Stable));
    }

    // ==================== AccumulationDistribution Tests ====================

    #[test]
    fn test_accumulation_distribution_variants() {
        let ad1 = AccumulationDistribution::Accumulation;
        let ad2 = AccumulationDistribution::Distribution;
        let ad3 = AccumulationDistribution::Neutral;

        assert!(matches!(ad1, AccumulationDistribution::Accumulation));
        assert!(matches!(ad2, AccumulationDistribution::Distribution));
        assert!(matches!(ad3, AccumulationDistribution::Neutral));
    }

    #[test]
    fn test_accumulation_distribution_serialize() {
        let ad = AccumulationDistribution::Accumulation;
        let json = serde_json::to_string(&ad).unwrap();
        assert_eq!(json, "\"Accumulation\"");
    }

    #[test]
    fn test_accumulation_distribution_deserialize() {
        let json = "\"Distribution\"";
        let ad: AccumulationDistribution = serde_json::from_str(json).unwrap();
        assert!(matches!(ad, AccumulationDistribution::Distribution));
    }

    // ==================== SignalQualityRating Tests ====================

    #[test]
    fn test_signal_quality_rating_variants() {
        let ratings = vec![
            SignalQualityRating::Excellent,
            SignalQualityRating::Good,
            SignalQualityRating::Average,
            SignalQualityRating::Poor,
            SignalQualityRating::VeryPoor,
        ];
        assert_eq!(ratings.len(), 5);
    }

    #[test]
    fn test_signal_quality_rating_serialize() {
        let rating = SignalQualityRating::Excellent;
        let json = serde_json::to_string(&rating).unwrap();
        assert_eq!(json, "\"Excellent\"");
    }

    #[test]
    fn test_signal_quality_rating_deserialize() {
        let json = "\"Poor\"";
        let rating: SignalQualityRating = serde_json::from_str(json).unwrap();
        assert!(matches!(rating, SignalQualityRating::Poor));
    }

    // ==================== StrategyContext Tests ====================

    #[test]
    fn test_strategy_context_default() {
        let ctx = StrategyContext::default();
        assert_eq!(ctx.selected_strategies.len(), 4);
        assert!(ctx
            .selected_strategies
            .contains(&"RSI Strategy".to_string()));
        assert!(ctx
            .selected_strategies
            .contains(&"MACD Strategy".to_string()));
        assert!(ctx
            .selected_strategies
            .contains(&"Volume Strategy".to_string()));
        assert!(ctx
            .selected_strategies
            .contains(&"Bollinger Bands Strategy".to_string()));
        assert!(matches!(ctx.market_condition, MarketCondition::Unknown));
        assert!(matches!(ctx.risk_level, RiskLevel::Moderate));
        assert!(ctx.technical_indicators.is_empty());
    }

    #[test]
    fn test_strategy_context_clone() {
        let ctx1 = StrategyContext::default();
        let ctx2 = ctx1.clone();
        assert_eq!(ctx1.selected_strategies, ctx2.selected_strategies);
    }

    #[test]
    fn test_strategy_context_custom() {
        let mut indicators = HashMap::new();
        indicators.insert("RSI".to_string(), serde_json::json!(70.5));

        let ctx = StrategyContext {
            selected_strategies: vec!["Custom Strategy".to_string()],
            market_condition: MarketCondition::Trending,
            risk_level: RiskLevel::Aggressive,
            technical_indicators: indicators,
        };

        assert_eq!(ctx.selected_strategies.len(), 1);
        assert_eq!(ctx.selected_strategies[0], "Custom Strategy");
        assert!(matches!(ctx.market_condition, MarketCondition::Trending));
        assert!(matches!(ctx.risk_level, RiskLevel::Aggressive));
        assert_eq!(ctx.technical_indicators.len(), 1);
    }

    #[test]
    fn test_strategy_context_serialize_deserialize() {
        let ctx = StrategyContext::default();
        let json = serde_json::to_string(&ctx).unwrap();
        let deserialized: StrategyContext = serde_json::from_str(&json).unwrap();
        assert_eq!(ctx.selected_strategies, deserialized.selected_strategies);
    }

    // ==================== VolatilityAssessment Tests ====================

    #[test]
    fn test_volatility_assessment_creation() {
        let va = VolatilityAssessment {
            level: VolatilityLevel::High,
            trend: VolatilityTrend::Increasing,
            percentile: 85.5,
        };

        assert!(matches!(va.level, VolatilityLevel::High));
        assert!(matches!(va.trend, VolatilityTrend::Increasing));
        assert_eq!(va.percentile, 85.5);
    }

    #[test]
    fn test_volatility_assessment_clone() {
        let va1 = VolatilityAssessment {
            level: VolatilityLevel::Low,
            trend: VolatilityTrend::Stable,
            percentile: 25.0,
        };
        let va2 = va1.clone();
        assert_eq!(va1.percentile, va2.percentile);
    }

    #[test]
    fn test_volatility_assessment_serialize_deserialize() {
        let va = VolatilityAssessment {
            level: VolatilityLevel::Normal,
            trend: VolatilityTrend::Decreasing,
            percentile: 50.0,
        };

        let json = serde_json::to_string(&va).unwrap();
        let deserialized: VolatilityAssessment = serde_json::from_str(&json).unwrap();
        assert_eq!(va.percentile, deserialized.percentile);
    }

    // ==================== VolumeAnalysis Tests ====================

    #[test]
    fn test_volume_analysis_creation() {
        let va = VolumeAnalysis {
            relative_volume: 1.5,
            trend: VolumeTrend::Increasing,
            accumulation_distribution: AccumulationDistribution::Accumulation,
        };

        assert_eq!(va.relative_volume, 1.5);
        assert!(matches!(va.trend, VolumeTrend::Increasing));
        assert!(matches!(
            va.accumulation_distribution,
            AccumulationDistribution::Accumulation
        ));
    }

    #[test]
    fn test_volume_analysis_clone() {
        let va1 = VolumeAnalysis {
            relative_volume: 0.8,
            trend: VolumeTrend::Decreasing,
            accumulation_distribution: AccumulationDistribution::Distribution,
        };
        let va2 = va1.clone();
        assert_eq!(va1.relative_volume, va2.relative_volume);
    }

    #[test]
    fn test_volume_analysis_serialize_deserialize() {
        let va = VolumeAnalysis {
            relative_volume: 1.2,
            trend: VolumeTrend::Stable,
            accumulation_distribution: AccumulationDistribution::Neutral,
        };

        let json = serde_json::to_string(&va).unwrap();
        let deserialized: VolumeAnalysis = serde_json::from_str(&json).unwrap();
        assert_eq!(va.relative_volume, deserialized.relative_volume);
    }

    // ==================== MarketAnalysis Tests ====================

    #[test]
    fn test_market_analysis_creation() {
        let ma = MarketAnalysis {
            trend_direction: TrendDirection::Bullish,
            trend_strength: 0.85,
            support_levels: vec![50000.0, 49500.0],
            resistance_levels: vec![52000.0, 53000.0],
            volatility_assessment: VolatilityAssessment {
                level: VolatilityLevel::Normal,
                trend: VolatilityTrend::Stable,
                percentile: 50.0,
            },
            volume_analysis: VolumeAnalysis {
                relative_volume: 1.0,
                trend: VolumeTrend::Stable,
                accumulation_distribution: AccumulationDistribution::Neutral,
            },
        };

        assert!(matches!(ma.trend_direction, TrendDirection::Bullish));
        assert_eq!(ma.trend_strength, 0.85);
        assert_eq!(ma.support_levels.len(), 2);
        assert_eq!(ma.resistance_levels.len(), 2);
    }

    #[test]
    fn test_market_analysis_clone() {
        let ma1 = MarketAnalysis {
            trend_direction: TrendDirection::Bearish,
            trend_strength: 0.6,
            support_levels: vec![45000.0],
            resistance_levels: vec![48000.0],
            volatility_assessment: VolatilityAssessment {
                level: VolatilityLevel::High,
                trend: VolatilityTrend::Increasing,
                percentile: 75.0,
            },
            volume_analysis: VolumeAnalysis {
                relative_volume: 1.3,
                trend: VolumeTrend::Increasing,
                accumulation_distribution: AccumulationDistribution::Distribution,
            },
        };
        let ma2 = ma1.clone();
        assert_eq!(ma1.trend_strength, ma2.trend_strength);
        assert_eq!(ma1.support_levels, ma2.support_levels);
    }

    #[test]
    fn test_market_analysis_serialize_deserialize() {
        let ma = MarketAnalysis {
            trend_direction: TrendDirection::Sideways,
            trend_strength: 0.3,
            support_levels: vec![],
            resistance_levels: vec![],
            volatility_assessment: VolatilityAssessment {
                level: VolatilityLevel::Low,
                trend: VolatilityTrend::Decreasing,
                percentile: 20.0,
            },
            volume_analysis: VolumeAnalysis {
                relative_volume: 0.7,
                trend: VolumeTrend::Decreasing,
                accumulation_distribution: AccumulationDistribution::Neutral,
            },
        };

        let json = serde_json::to_string(&ma).unwrap();
        let deserialized: MarketAnalysis = serde_json::from_str(&json).unwrap();
        assert_eq!(ma.trend_strength, deserialized.trend_strength);
    }

    // ==================== RiskAssessment Tests ====================

    #[test]
    fn test_risk_assessment_creation() {
        let ra = RiskAssessment {
            overall_risk: RiskLevel::Moderate,
            technical_risk: 0.4,
            market_risk: 0.5,
            liquidity_risk: 0.2,
            recommended_position_size: 0.05,
            stop_loss_suggestion: Some(49000.0),
            take_profit_suggestion: Some(52000.0),
        };

        assert!(matches!(ra.overall_risk, RiskLevel::Moderate));
        assert_eq!(ra.technical_risk, 0.4);
        assert_eq!(ra.market_risk, 0.5);
        assert_eq!(ra.liquidity_risk, 0.2);
        assert_eq!(ra.recommended_position_size, 0.05);
        assert_eq!(ra.stop_loss_suggestion, Some(49000.0));
        assert_eq!(ra.take_profit_suggestion, Some(52000.0));
    }

    #[test]
    fn test_risk_assessment_without_suggestions() {
        let ra = RiskAssessment {
            overall_risk: RiskLevel::Conservative,
            technical_risk: 0.2,
            market_risk: 0.3,
            liquidity_risk: 0.1,
            recommended_position_size: 0.02,
            stop_loss_suggestion: None,
            take_profit_suggestion: None,
        };

        assert!(ra.stop_loss_suggestion.is_none());
        assert!(ra.take_profit_suggestion.is_none());
    }

    #[test]
    fn test_risk_assessment_serialize_deserialize() {
        let ra = RiskAssessment {
            overall_risk: RiskLevel::Aggressive,
            technical_risk: 0.7,
            market_risk: 0.8,
            liquidity_risk: 0.6,
            recommended_position_size: 0.1,
            stop_loss_suggestion: Some(48000.0),
            take_profit_suggestion: Some(55000.0),
        };

        let json = serde_json::to_string(&ra).unwrap();
        let deserialized: RiskAssessment = serde_json::from_str(&json).unwrap();
        assert_eq!(ra.technical_risk, deserialized.technical_risk);
        assert_eq!(ra.stop_loss_suggestion, deserialized.stop_loss_suggestion);
    }

    // ==================== StrategyPerformance Tests ====================

    #[test]
    fn test_strategy_performance_creation() {
        let sp = StrategyPerformance {
            strategy_name: "RSI Strategy".to_string(),
            total_signals: 100,
            successful_signals: 65,
            accuracy: 0.65,
            average_confidence: 0.75,
            profit_factor: 1.8,
            max_drawdown: 0.15,
            sharpe_ratio: 1.5,
            last_updated: 1234567890,
        };

        assert_eq!(sp.strategy_name, "RSI Strategy");
        assert_eq!(sp.total_signals, 100);
        assert_eq!(sp.successful_signals, 65);
        assert_eq!(sp.accuracy, 0.65);
    }

    #[test]
    fn test_strategy_performance_clone() {
        let sp1 = StrategyPerformance {
            strategy_name: "MACD Strategy".to_string(),
            total_signals: 50,
            successful_signals: 30,
            accuracy: 0.6,
            average_confidence: 0.7,
            profit_factor: 1.5,
            max_drawdown: 0.2,
            sharpe_ratio: 1.2,
            last_updated: 1234567890,
        };
        let sp2 = sp1.clone();
        assert_eq!(sp1.strategy_name, sp2.strategy_name);
        assert_eq!(sp1.total_signals, sp2.total_signals);
    }

    #[test]
    fn test_strategy_performance_serialize_deserialize() {
        let sp = StrategyPerformance {
            strategy_name: "Volume Strategy".to_string(),
            total_signals: 200,
            successful_signals: 140,
            accuracy: 0.7,
            average_confidence: 0.8,
            profit_factor: 2.0,
            max_drawdown: 0.1,
            sharpe_ratio: 1.8,
            last_updated: 1234567890,
        };

        let json = serde_json::to_string(&sp).unwrap();
        let deserialized: StrategyPerformance = serde_json::from_str(&json).unwrap();
        assert_eq!(sp.strategy_name, deserialized.strategy_name);
        assert_eq!(sp.total_signals, deserialized.total_signals);
    }

    // ==================== SignalQuality Tests ====================

    #[test]
    fn test_signal_quality_creation() {
        let sq = SignalQuality {
            accuracy_score: 0.85,
            consistency_score: 0.75,
            timeliness_score: 0.9,
            risk_adjusted_return: 1.5,
            overall_quality: SignalQualityRating::Excellent,
        };

        assert_eq!(sq.accuracy_score, 0.85);
        assert_eq!(sq.consistency_score, 0.75);
        assert_eq!(sq.timeliness_score, 0.9);
        assert!(matches!(sq.overall_quality, SignalQualityRating::Excellent));
    }

    #[test]
    fn test_signal_quality_clone() {
        let sq1 = SignalQuality {
            accuracy_score: 0.6,
            consistency_score: 0.5,
            timeliness_score: 0.7,
            risk_adjusted_return: 1.0,
            overall_quality: SignalQualityRating::Average,
        };
        let sq2 = sq1.clone();
        assert_eq!(sq1.accuracy_score, sq2.accuracy_score);
    }

    #[test]
    fn test_signal_quality_serialize_deserialize() {
        let sq = SignalQuality {
            accuracy_score: 0.4,
            consistency_score: 0.3,
            timeliness_score: 0.5,
            risk_adjusted_return: 0.8,
            overall_quality: SignalQualityRating::Poor,
        };

        let json = serde_json::to_string(&sq).unwrap();
        let deserialized: SignalQuality = serde_json::from_str(&json).unwrap();
        assert_eq!(sq.accuracy_score, deserialized.accuracy_score);
    }

    // ==================== AIAnalysisRequest Tests ====================

    #[test]
    fn test_ai_analysis_request_creation() {
        use crate::market_data::cache::CandleData;

        let mut timeframe_data = HashMap::new();
        let candles = vec![CandleData {
            open_time: 1000,
            close_time: 2000,
            open: 50000.0,
            high: 51000.0,
            low: 49500.0,
            close: 50500.0,
            volume: 100.0,
            quote_volume: 5000000.0,
            trades: 1000,
            is_closed: true,
        }];
        timeframe_data.insert("1h".to_string(), candles);

        let req = AIAnalysisRequest {
            symbol: "BTCUSDT".to_string(),
            timeframe_data,
            current_price: 50500.0,
            volume_24h: 1000000.0,
            timestamp: 1234567890,
            strategy_context: StrategyContext::default(),
        };

        assert_eq!(req.symbol, "BTCUSDT");
        assert_eq!(req.current_price, 50500.0);
        assert_eq!(req.volume_24h, 1000000.0);
        assert!(req.timeframe_data.contains_key("1h"));
    }

    #[test]
    fn test_ai_analysis_request_clone() {
        let req1 = AIAnalysisRequest {
            symbol: "ETHUSDT".to_string(),
            timeframe_data: HashMap::new(),
            current_price: 3000.0,
            volume_24h: 500000.0,
            timestamp: 1234567890,
            strategy_context: StrategyContext::default(),
        };
        let req2 = req1.clone();
        assert_eq!(req1.symbol, req2.symbol);
        assert_eq!(req1.current_price, req2.current_price);
    }

    // ==================== AIAnalysisResponse Tests ====================

    #[test]
    fn test_ai_analysis_response_creation() {
        let response = AIAnalysisResponse {
            signal: super::super::TradingSignal::Long,
            confidence: 0.85,
            reasoning: "Strong bullish momentum".to_string(),
            strategy_scores: {
                let mut scores = HashMap::new();
                scores.insert("RSI".to_string(), 0.8);
                scores.insert("MACD".to_string(), 0.9);
                scores
            },
            market_analysis: MarketAnalysis {
                trend_direction: TrendDirection::Bullish,
                trend_strength: 0.85,
                support_levels: vec![50000.0],
                resistance_levels: vec![52000.0],
                volatility_assessment: VolatilityAssessment {
                    level: VolatilityLevel::Normal,
                    trend: VolatilityTrend::Stable,
                    percentile: 50.0,
                },
                volume_analysis: VolumeAnalysis {
                    relative_volume: 1.0,
                    trend: VolumeTrend::Stable,
                    accumulation_distribution: AccumulationDistribution::Neutral,
                },
            },
            risk_assessment: RiskAssessment {
                overall_risk: RiskLevel::Moderate,
                technical_risk: 0.4,
                market_risk: 0.5,
                liquidity_risk: 0.2,
                recommended_position_size: 0.05,
                stop_loss_suggestion: Some(49000.0),
                take_profit_suggestion: Some(52000.0),
            },
            timestamp: 1234567890,
        };

        assert_eq!(response.confidence, 0.85);
        assert_eq!(response.reasoning, "Strong bullish momentum");
        assert_eq!(response.strategy_scores.len(), 2);
    }

    #[test]
    fn test_ai_analysis_response_clone() {
        let resp1 = AIAnalysisResponse {
            signal: super::super::TradingSignal::Short,
            confidence: 0.7,
            reasoning: "Bearish divergence".to_string(),
            strategy_scores: HashMap::new(),
            market_analysis: MarketAnalysis {
                trend_direction: TrendDirection::Bearish,
                trend_strength: 0.6,
                support_levels: vec![],
                resistance_levels: vec![],
                volatility_assessment: VolatilityAssessment {
                    level: VolatilityLevel::High,
                    trend: VolatilityTrend::Increasing,
                    percentile: 75.0,
                },
                volume_analysis: VolumeAnalysis {
                    relative_volume: 1.2,
                    trend: VolumeTrend::Increasing,
                    accumulation_distribution: AccumulationDistribution::Distribution,
                },
            },
            risk_assessment: RiskAssessment {
                overall_risk: RiskLevel::Aggressive,
                technical_risk: 0.7,
                market_risk: 0.8,
                liquidity_risk: 0.6,
                recommended_position_size: 0.03,
                stop_loss_suggestion: None,
                take_profit_suggestion: None,
            },
            timestamp: 1234567890,
        };
        let resp2 = resp1.clone();
        assert_eq!(resp1.confidence, resp2.confidence);
        assert_eq!(resp1.reasoning, resp2.reasoning);
    }

    #[test]
    fn test_ai_analysis_response_serialize_deserialize() {
        let response = AIAnalysisResponse {
            signal: super::super::TradingSignal::Neutral,
            confidence: 0.5,
            reasoning: "Mixed signals".to_string(),
            strategy_scores: HashMap::new(),
            market_analysis: MarketAnalysis {
                trend_direction: TrendDirection::Sideways,
                trend_strength: 0.3,
                support_levels: vec![],
                resistance_levels: vec![],
                volatility_assessment: VolatilityAssessment {
                    level: VolatilityLevel::Low,
                    trend: VolatilityTrend::Stable,
                    percentile: 30.0,
                },
                volume_analysis: VolumeAnalysis {
                    relative_volume: 0.8,
                    trend: VolumeTrend::Decreasing,
                    accumulation_distribution: AccumulationDistribution::Neutral,
                },
            },
            risk_assessment: RiskAssessment {
                overall_risk: RiskLevel::Conservative,
                technical_risk: 0.3,
                market_risk: 0.4,
                liquidity_risk: 0.2,
                recommended_position_size: 0.02,
                stop_loss_suggestion: None,
                take_profit_suggestion: None,
            },
            timestamp: 1234567890,
        };

        let json = serde_json::to_string(&response).unwrap();
        let deserialized: AIAnalysisResponse = serde_json::from_str(&json).unwrap();
        assert_eq!(response.confidence, deserialized.confidence);
    }

    // ==================== Edge Cases and Complex Scenarios ====================

    #[test]
    fn test_strategy_context_with_empty_strategies() {
        let ctx = StrategyContext {
            selected_strategies: vec![],
            market_condition: MarketCondition::Unknown,
            risk_level: RiskLevel::Conservative,
            technical_indicators: HashMap::new(),
        };

        assert_eq!(ctx.selected_strategies.len(), 0);
    }

    #[test]
    fn test_strategy_context_with_many_indicators() {
        let mut indicators = HashMap::new();
        indicators.insert("RSI".to_string(), serde_json::json!(70.5));
        indicators.insert(
            "MACD".to_string(),
            serde_json::json!({"macd": 0.5, "signal": 0.3}),
        );
        indicators.insert("BB_Upper".to_string(), serde_json::json!(51000.0));
        indicators.insert("BB_Lower".to_string(), serde_json::json!(49000.0));

        let ctx = StrategyContext {
            selected_strategies: vec!["Multi-Indicator".to_string()],
            market_condition: MarketCondition::Volatile,
            risk_level: RiskLevel::Conservative,
            technical_indicators: indicators,
        };

        assert_eq!(ctx.technical_indicators.len(), 4);
    }

    #[test]
    fn test_market_analysis_with_empty_levels() {
        let ma = MarketAnalysis {
            trend_direction: TrendDirection::Uncertain,
            trend_strength: 0.0,
            support_levels: vec![],
            resistance_levels: vec![],
            volatility_assessment: VolatilityAssessment {
                level: VolatilityLevel::VeryLow,
                trend: VolatilityTrend::Stable,
                percentile: 5.0,
            },
            volume_analysis: VolumeAnalysis {
                relative_volume: 0.5,
                trend: VolumeTrend::Decreasing,
                accumulation_distribution: AccumulationDistribution::Neutral,
            },
        };

        assert!(ma.support_levels.is_empty());
        assert!(ma.resistance_levels.is_empty());
        assert_eq!(ma.trend_strength, 0.0);
    }

    #[test]
    fn test_market_analysis_with_many_levels() {
        let ma = MarketAnalysis {
            trend_direction: TrendDirection::Bullish,
            trend_strength: 0.9,
            support_levels: vec![50000.0, 49500.0, 49000.0, 48500.0],
            resistance_levels: vec![52000.0, 53000.0, 54000.0, 55000.0],
            volatility_assessment: VolatilityAssessment {
                level: VolatilityLevel::VeryHigh,
                trend: VolatilityTrend::Increasing,
                percentile: 95.0,
            },
            volume_analysis: VolumeAnalysis {
                relative_volume: 2.5,
                trend: VolumeTrend::Increasing,
                accumulation_distribution: AccumulationDistribution::Accumulation,
            },
        };

        assert_eq!(ma.support_levels.len(), 4);
        assert_eq!(ma.resistance_levels.len(), 4);
    }

    #[test]
    fn test_volatility_assessment_edge_percentiles() {
        let va_min = VolatilityAssessment {
            level: VolatilityLevel::VeryLow,
            trend: VolatilityTrend::Decreasing,
            percentile: 0.0,
        };

        let va_max = VolatilityAssessment {
            level: VolatilityLevel::VeryHigh,
            trend: VolatilityTrend::Increasing,
            percentile: 100.0,
        };

        assert_eq!(va_min.percentile, 0.0);
        assert_eq!(va_max.percentile, 100.0);
    }

    #[test]
    fn test_volume_analysis_extreme_values() {
        let va_low = VolumeAnalysis {
            relative_volume: 0.1,
            trend: VolumeTrend::Decreasing,
            accumulation_distribution: AccumulationDistribution::Distribution,
        };

        let va_high = VolumeAnalysis {
            relative_volume: 10.0,
            trend: VolumeTrend::Increasing,
            accumulation_distribution: AccumulationDistribution::Accumulation,
        };

        assert_eq!(va_low.relative_volume, 0.1);
        assert_eq!(va_high.relative_volume, 10.0);
    }

    #[test]
    fn test_risk_assessment_extreme_values() {
        let ra = RiskAssessment {
            overall_risk: RiskLevel::Aggressive,
            technical_risk: 1.0,
            market_risk: 1.0,
            liquidity_risk: 1.0,
            recommended_position_size: 0.0,
            stop_loss_suggestion: Some(0.0),
            take_profit_suggestion: Some(1000000.0),
        };

        assert_eq!(ra.technical_risk, 1.0);
        assert_eq!(ra.market_risk, 1.0);
        assert_eq!(ra.liquidity_risk, 1.0);
        assert_eq!(ra.recommended_position_size, 0.0);
    }

    #[test]
    fn test_strategy_performance_zero_signals() {
        let sp = StrategyPerformance {
            strategy_name: "New Strategy".to_string(),
            total_signals: 0,
            successful_signals: 0,
            accuracy: 0.0,
            average_confidence: 0.0,
            profit_factor: 0.0,
            max_drawdown: 0.0,
            sharpe_ratio: 0.0,
            last_updated: 0,
        };

        assert_eq!(sp.total_signals, 0);
        assert_eq!(sp.successful_signals, 0);
        assert_eq!(sp.accuracy, 0.0);
    }

    #[test]
    fn test_signal_quality_all_zero() {
        let sq = SignalQuality {
            accuracy_score: 0.0,
            consistency_score: 0.0,
            timeliness_score: 0.0,
            risk_adjusted_return: 0.0,
            overall_quality: SignalQualityRating::VeryPoor,
        };

        assert_eq!(sq.accuracy_score, 0.0);
        assert!(matches!(sq.overall_quality, SignalQualityRating::VeryPoor));
    }

    #[test]
    fn test_signal_quality_all_max() {
        let sq = SignalQuality {
            accuracy_score: 1.0,
            consistency_score: 1.0,
            timeliness_score: 1.0,
            risk_adjusted_return: 5.0,
            overall_quality: SignalQualityRating::Excellent,
        };

        assert_eq!(sq.accuracy_score, 1.0);
        assert!(matches!(sq.overall_quality, SignalQualityRating::Excellent));
    }
}
