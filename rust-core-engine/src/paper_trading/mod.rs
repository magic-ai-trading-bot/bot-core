#![allow(dead_code)]

pub mod engine;
pub mod portfolio;
pub mod settings;
pub mod strategy_optimizer;
pub mod trade;

pub use engine::PaperTradingEngine;
pub use portfolio::PaperPortfolio;
pub use settings::PaperTradingSettings;
pub use trade::PaperTrade;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// Paper trading signal from AI
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AITradingSignal {
    pub id: String,
    pub symbol: String,
    pub signal_type: crate::strategies::TradingSignal,
    pub confidence: f64,
    pub reasoning: String,
    pub entry_price: f64,
    pub suggested_stop_loss: Option<f64>,
    pub suggested_take_profit: Option<f64>,
    pub suggested_leverage: Option<u8>,
    pub market_analysis: MarketAnalysisData,
    pub timestamp: DateTime<Utc>,
}

/// Comprehensive market analysis from AI
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MarketAnalysisData {
    pub trend_direction: String,
    pub trend_strength: f64,
    pub volatility: f64,
    pub support_levels: Vec<f64>,
    pub resistance_levels: Vec<f64>,
    pub volume_analysis: String,
    pub risk_score: f64,
}

/// Paper trading event for WebSocket broadcasting
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PaperTradingEvent {
    pub event_type: String,
    pub data: serde_json::Value,
    pub timestamp: DateTime<Utc>,
}

/// Real-time price update
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PriceUpdate {
    pub symbol: String,
    pub price: f64,
    pub volume: f64,
    pub timestamp: DateTime<Utc>,
}

/// Trade execution result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TradeExecutionResult {
    pub success: bool,
    pub trade_id: Option<String>,
    pub error_message: Option<String>,
    pub execution_price: Option<f64>,
    pub fees_paid: Option<f64>,
}

/// Portfolio performance summary
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceSummary {
    pub total_trades: u64,
    pub win_rate: f64,
    pub total_pnl: f64,
    pub total_pnl_percentage: f64,
    pub max_drawdown: f64,
    pub max_drawdown_percentage: f64,
    pub sharpe_ratio: f64,
    pub profit_factor: f64,
    pub average_win: f64,
    pub average_loss: f64,
    pub largest_win: f64,
    pub largest_loss: f64,
    pub current_balance: f64,
    pub equity: f64,
    pub margin_used: f64,
    pub free_margin: f64,
}
