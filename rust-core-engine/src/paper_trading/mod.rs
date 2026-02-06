#![allow(dead_code)]

pub mod engine;
pub mod portfolio;
pub mod settings;
pub mod strategy_optimizer;
pub mod trade;

pub use engine::PaperTradingEngine;
pub use portfolio::PaperPortfolio;
pub use settings::PaperTradingSettings;
pub use trade::{CloseReason, PaperTrade, TradeType};
// ManualOrderParams is exported directly from this module (defined below)

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

/// Parameters for manual order execution
/// Groups multiple order parameters to reduce function argument count
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ManualOrderParams {
    /// Trading symbol (e.g., "BTCUSDT")
    pub symbol: String,
    /// Order side: "buy" or "sell"
    pub side: String,
    /// Order type: "market", "limit", or "stop-limit"
    pub order_type: String,
    /// Quantity to trade
    pub quantity: f64,
    /// Limit price (required for limit and stop-limit orders)
    pub price: Option<f64>,
    /// Stop price (required for stop-limit orders)
    pub stop_price: Option<f64>,
    /// Leverage (optional, defaults to settings)
    pub leverage: Option<u8>,
    /// Stop loss percentage (optional)
    pub stop_loss_pct: Option<f64>,
    /// Take profit percentage (optional)
    pub take_profit_pct: Option<f64>,
}

/// Order type for manual orders
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum OrderType {
    Market,
    Limit,
    StopLimit,
}

impl std::fmt::Display for OrderType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            OrderType::Market => write!(f, "market"),
            OrderType::Limit => write!(f, "limit"),
            OrderType::StopLimit => write!(f, "stop-limit"),
        }
    }
}

/// Order status for pending orders
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum OrderStatus {
    /// Order is pending, waiting for stop price to be hit
    Pending,
    /// Stop price was hit, order is now active (for stop-limit)
    Triggered,
    /// Order has been fully executed
    Filled,
    /// Order was cancelled by user
    Cancelled,
    /// Order expired (if TTL was set)
    Expired,
}

impl std::fmt::Display for OrderStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            OrderStatus::Pending => write!(f, "pending"),
            OrderStatus::Triggered => write!(f, "triggered"),
            OrderStatus::Filled => write!(f, "filled"),
            OrderStatus::Cancelled => write!(f, "cancelled"),
            OrderStatus::Expired => write!(f, "expired"),
        }
    }
}

/// @spec:FR-PAPER-003 - Stop-Limit Order
/// A pending order that waits for stop price to be hit before executing at limit price
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StopLimitOrder {
    /// Unique order ID
    pub id: String,
    /// Trading symbol (e.g., "BTCUSDT")
    pub symbol: String,
    /// Order side: "buy" or "sell"
    pub side: String,
    /// Order type
    pub order_type: OrderType,
    /// Quantity to trade
    pub quantity: f64,
    /// Stop price - when market price hits this, order becomes active
    pub stop_price: f64,
    /// Limit price - the price at which order will be executed after triggered
    pub limit_price: f64,
    /// Leverage for the trade
    pub leverage: u8,
    /// Stop loss percentage (optional)
    pub stop_loss_pct: Option<f64>,
    /// Take profit percentage (optional)
    pub take_profit_pct: Option<f64>,
    /// Current order status
    pub status: OrderStatus,
    /// When the order was created
    pub created_at: DateTime<Utc>,
    /// When the order was triggered (if applicable)
    pub triggered_at: Option<DateTime<Utc>>,
    /// When the order was filled (if applicable)
    pub filled_at: Option<DateTime<Utc>>,
    /// Error message if order failed
    pub error_message: Option<String>,
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ai_trading_signal_creation() {
        let signal = AITradingSignal {
            id: "signal-123".to_string(),
            symbol: "BTCUSDT".to_string(),
            signal_type: crate::strategies::TradingSignal::Long,
            confidence: 0.85,
            reasoning: "Strong uptrend detected".to_string(),
            entry_price: 50000.0,
            suggested_stop_loss: Some(48000.0),
            suggested_take_profit: Some(55000.0),
            suggested_leverage: Some(10),
            market_analysis: MarketAnalysisData {
                trend_direction: "Bullish".to_string(),
                trend_strength: 0.8,
                volatility: 0.3,
                support_levels: vec![48000.0, 46000.0],
                resistance_levels: vec![52000.0, 55000.0],
                volume_analysis: "High volume on breakout".to_string(),
                risk_score: 0.4,
            },
            timestamp: Utc::now(),
        };

        assert_eq!(signal.symbol, "BTCUSDT");
        assert_eq!(signal.confidence, 0.85);
        assert_eq!(signal.suggested_leverage, Some(10));
    }

    #[test]
    fn test_market_analysis_data() {
        let analysis = MarketAnalysisData {
            trend_direction: "Bearish".to_string(),
            trend_strength: 0.7,
            volatility: 0.5,
            support_levels: vec![45000.0, 44000.0, 43000.0],
            resistance_levels: vec![48000.0, 50000.0],
            volume_analysis: "Declining volume".to_string(),
            risk_score: 0.6,
        };

        assert_eq!(analysis.trend_direction, "Bearish");
        assert_eq!(analysis.support_levels.len(), 3);
        assert_eq!(analysis.resistance_levels.len(), 2);
        assert!(analysis.risk_score > 0.0 && analysis.risk_score <= 1.0);
    }

    #[test]
    fn test_price_update() {
        let update = PriceUpdate {
            symbol: "ETHUSDT".to_string(),
            price: 3000.0,
            volume: 1000000.0,
            timestamp: Utc::now(),
        };

        assert_eq!(update.symbol, "ETHUSDT");
        assert_eq!(update.price, 3000.0);
        assert!(update.volume > 0.0);
    }

    #[test]
    fn test_trade_execution_result_success() {
        let result = TradeExecutionResult {
            success: true,
            trade_id: Some("trade-456".to_string()),
            error_message: None,
            execution_price: Some(50000.0),
            fees_paid: Some(2.5),
        };

        assert!(result.success);
        assert!(result.trade_id.is_some());
        assert!(result.error_message.is_none());
    }

    #[test]
    fn test_trade_execution_result_failure() {
        let result = TradeExecutionResult {
            success: false,
            trade_id: None,
            error_message: Some("Insufficient margin".to_string()),
            execution_price: None,
            fees_paid: None,
        };

        assert!(!result.success);
        assert!(result.trade_id.is_none());
        assert!(result.error_message.is_some());
    }

    #[test]
    fn test_performance_summary() {
        let summary = PerformanceSummary {
            total_trades: 100,
            win_rate: 65.0,
            total_pnl: 5000.0,
            total_pnl_percentage: 50.0,
            max_drawdown: 500.0,
            max_drawdown_percentage: 5.0,
            sharpe_ratio: 2.5,
            profit_factor: 2.0,
            average_win: 150.0,
            average_loss: 75.0,
            largest_win: 500.0,
            largest_loss: 200.0,
            current_balance: 15000.0,
            equity: 15000.0,
            margin_used: 1000.0,
            free_margin: 14000.0,
        };

        assert_eq!(summary.total_trades, 100);
        assert_eq!(summary.win_rate, 65.0);
        assert!(summary.profit_factor > 1.0);
        assert!(summary.sharpe_ratio > 0.0);
    }

    #[test]
    fn test_paper_trading_event() {
        let event = PaperTradingEvent {
            event_type: "TradeOpened".to_string(),
            data: serde_json::json!({
                "trade_id": "trade-789",
                "symbol": "BTCUSDT",
                "type": "Long"
            }),
            timestamp: Utc::now(),
        };

        assert_eq!(event.event_type, "TradeOpened");
        assert!(event.data.is_object());
    }

    #[test]
    fn test_multiple_support_resistance_levels() {
        let analysis = MarketAnalysisData {
            trend_direction: "Neutral".to_string(),
            trend_strength: 0.3,
            volatility: 0.4,
            support_levels: vec![49000.0, 48500.0, 48000.0, 47500.0],
            resistance_levels: vec![50500.0, 51000.0, 51500.0],
            volume_analysis: "Average volume".to_string(),
            risk_score: 0.5,
        };

        assert!(analysis.support_levels.len() >= 3);
        assert!(analysis.resistance_levels.len() >= 2);

        // Support levels should be below resistance levels
        if let (Some(highest_support), Some(lowest_resistance)) = (
            analysis.support_levels.first(),
            analysis.resistance_levels.first(),
        ) {
            assert!(highest_support < lowest_resistance);
        }
    }

    #[test]
    fn test_high_confidence_signal() {
        let signal = AITradingSignal {
            id: "signal-high-conf".to_string(),
            symbol: "BTCUSDT".to_string(),
            signal_type: crate::strategies::TradingSignal::Long,
            confidence: 0.95,
            reasoning: "Multiple technical indicators align".to_string(),
            entry_price: 50000.0,
            suggested_stop_loss: Some(49000.0),
            suggested_take_profit: Some(55000.0),
            suggested_leverage: Some(5),
            market_analysis: MarketAnalysisData {
                trend_direction: "Bullish".to_string(),
                trend_strength: 0.9,
                volatility: 0.2,
                support_levels: vec![49000.0],
                resistance_levels: vec![55000.0],
                volume_analysis: "Very high volume".to_string(),
                risk_score: 0.2,
            },
            timestamp: Utc::now(),
        };

        assert!(signal.confidence >= 0.9);
        assert!(signal.market_analysis.risk_score <= 0.3);
        assert!(signal.suggested_leverage.unwrap() <= 10);
    }

    #[test]
    fn test_low_confidence_signal() {
        let signal = AITradingSignal {
            id: "signal-low-conf".to_string(),
            symbol: "ETHUSDT".to_string(),
            signal_type: crate::strategies::TradingSignal::Short,
            confidence: 0.55,
            reasoning: "Mixed signals from indicators".to_string(),
            entry_price: 3000.0,
            suggested_stop_loss: Some(3100.0),
            suggested_take_profit: Some(2850.0),
            suggested_leverage: Some(3),
            market_analysis: MarketAnalysisData {
                trend_direction: "Uncertain".to_string(),
                trend_strength: 0.4,
                volatility: 0.7,
                support_levels: vec![2850.0, 2800.0],
                resistance_levels: vec![3100.0, 3200.0],
                volume_analysis: "Low volume".to_string(),
                risk_score: 0.7,
            },
            timestamp: Utc::now(),
        };

        assert!(signal.confidence < 0.7);
        assert!(signal.market_analysis.risk_score >= 0.6);
        assert!(signal.suggested_leverage.unwrap() <= 5);
    }

    #[test]
    fn test_zero_fees_execution() {
        let result = TradeExecutionResult {
            success: true,
            trade_id: Some("trade-zero-fee".to_string()),
            error_message: None,
            execution_price: Some(50000.0),
            fees_paid: Some(0.0),
        };

        assert_eq!(result.fees_paid, Some(0.0));
    }

    #[test]
    fn test_performance_summary_all_losses() {
        let summary = PerformanceSummary {
            total_trades: 20,
            win_rate: 0.0,
            total_pnl: -1000.0,
            total_pnl_percentage: -10.0,
            max_drawdown: 1000.0,
            max_drawdown_percentage: 10.0,
            sharpe_ratio: -1.5,
            profit_factor: 0.0,
            average_win: 0.0,
            average_loss: 50.0,
            largest_win: 0.0,
            largest_loss: 150.0,
            current_balance: 9000.0,
            equity: 9000.0,
            margin_used: 0.0,
            free_margin: 9000.0,
        };

        assert_eq!(summary.win_rate, 0.0);
        assert!(summary.total_pnl < 0.0);
        assert_eq!(summary.profit_factor, 0.0);
    }

    #[test]
    fn test_extreme_volatility_market() {
        let analysis = MarketAnalysisData {
            trend_direction: "Volatile".to_string(),
            trend_strength: 0.5,
            volatility: 0.95,
            support_levels: vec![40000.0, 35000.0],
            resistance_levels: vec![60000.0, 65000.0],
            volume_analysis: "Extreme volume spikes".to_string(),
            risk_score: 0.9,
        };

        assert!(analysis.volatility >= 0.8);
        assert!(analysis.risk_score >= 0.8);
    }
}
