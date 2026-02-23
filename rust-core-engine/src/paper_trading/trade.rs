use anyhow::Result;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::fmt;
use uuid::Uuid;

/// Trade types supported
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
pub enum TradeType {
    Long,
    Short,
}

impl fmt::Display for TradeType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            TradeType::Long => write!(f, "Long"),
            TradeType::Short => write!(f, "Short"),
        }
    }
}

/// Trade status
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
pub enum TradeStatus {
    Open,
    Closed,
    Cancelled,
}

/// Reason for closing a trade
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum CloseReason {
    TakeProfit,
    StopLoss,
    Manual,
    AISignal,
    RiskManagement,
    MarginCall,
    TimeBasedExit,
}

/// Paper trading position that simulates Binance Futures
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PaperTrade {
    /// Unique trade identifier
    pub id: String,

    /// Trading symbol (e.g., BTCUSDT)
    pub symbol: String,

    /// Trade type (Long/Short)
    pub trade_type: TradeType,

    /// Current trade status
    pub status: TradeStatus,

    /// Entry price
    pub entry_price: f64,

    /// Exit price (if closed)
    pub exit_price: Option<f64>,

    /// Quantity in base asset
    pub quantity: f64,

    /// Leverage used (1-125x for Binance Futures)
    pub leverage: u8,

    /// Stop loss price
    pub stop_loss: Option<f64>,

    /// Take profit price
    pub take_profit: Option<f64>,

    /// Current unrealized PnL
    pub unrealized_pnl: f64,

    /// Realized PnL (if closed)
    pub realized_pnl: Option<f64>,

    /// PnL percentage based on margin
    pub pnl_percentage: f64,

    /// Trading fees paid
    pub trading_fees: f64,

    /// Funding fees accumulated
    pub funding_fees: f64,

    /// Initial margin required
    pub initial_margin: f64,

    /// Maintenance margin required
    pub maintenance_margin: f64,

    /// Current margin used
    pub margin_used: f64,

    /// Margin ratio (equity / margin)
    pub margin_ratio: f64,

    /// Trade opening timestamp
    pub open_time: DateTime<Utc>,

    /// Trade closing timestamp
    pub close_time: Option<DateTime<Utc>>,

    /// Duration in milliseconds
    pub duration_ms: Option<i64>,

    /// AI signal that triggered this trade
    pub ai_signal_id: Option<String>,

    /// AI confidence when trade was opened
    pub ai_confidence: Option<f64>,

    /// AI reasoning for the trade
    pub ai_reasoning: Option<String>,

    /// Strategy used for this trade
    pub strategy_name: Option<String>,

    /// Close reason
    pub close_reason: Option<CloseReason>,

    /// Risk score at time of entry
    pub risk_score: f64,

    /// Market regime when trade was opened
    pub market_regime: Option<String>,

    /// Volatility at time of entry
    pub entry_volatility: f64,

    /// Maximum favorable excursion
    pub max_favorable_excursion: f64,

    /// Maximum adverse excursion
    pub max_adverse_excursion: f64,

    /// Slippage experienced
    pub slippage: f64,

    /// Signal timestamp (when AI signal was generated)
    pub signal_timestamp: Option<DateTime<Utc>>,

    /// Execution timestamp (when trade was actually executed)
    pub execution_timestamp: DateTime<Utc>,

    /// Execution latency in milliseconds (signal to execution time)
    pub execution_latency_ms: Option<u64>,

    /// Highest price achieved (for trailing stop calculation)
    /// For Long: tracks highest price reached
    /// For Short: tracks lowest price reached
    pub highest_price_achieved: Option<f64>,

    /// Trailing stop activated flag
    /// True once profit threshold is met and trailing begins
    pub trailing_stop_active: bool,

    /// Custom metadata
    pub metadata: std::collections::HashMap<String, serde_json::Value>,
}

impl PaperTrade {
    /// Create a new paper trade
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        symbol: String,
        trade_type: TradeType,
        entry_price: f64,
        quantity: f64,
        leverage: u8,
        trading_fee_rate: f64,
        ai_signal_id: Option<String>,
        ai_confidence: Option<f64>,
        ai_reasoning: Option<String>,
    ) -> Self {
        let id = Uuid::new_v4().to_string();
        let notional_value = quantity * entry_price;
        let initial_margin = notional_value / leverage as f64;
        let trading_fees = notional_value * trading_fee_rate;

        // Binance Futures maintenance margin rates (simplified)
        let maintenance_margin_rate = match leverage {
            1..=5 => 0.01,     // 1%
            6..=10 => 0.025,   // 2.5%
            11..=20 => 0.05,   // 5%
            21..=50 => 0.1,    // 10%
            51..=100 => 0.125, // 12.5%
            _ => 0.15,         // 15%
        };

        let maintenance_margin = notional_value * maintenance_margin_rate;

        Self {
            id,
            symbol,
            trade_type,
            status: TradeStatus::Open,
            entry_price,
            exit_price: None,
            quantity,
            leverage,
            stop_loss: None,
            take_profit: None,
            unrealized_pnl: 0.0,
            realized_pnl: None,
            pnl_percentage: 0.0,
            trading_fees,
            funding_fees: 0.0,
            initial_margin,
            maintenance_margin,
            margin_used: initial_margin,
            margin_ratio: 1.0,
            open_time: Utc::now(),
            close_time: None,
            duration_ms: None,
            ai_signal_id,
            ai_confidence,
            ai_reasoning,
            strategy_name: None,
            close_reason: None,
            risk_score: 0.5,
            market_regime: None,
            entry_volatility: 0.0,
            max_favorable_excursion: 0.0,
            max_adverse_excursion: 0.0,
            slippage: 0.0,
            signal_timestamp: None,
            execution_timestamp: Utc::now(),
            execution_latency_ms: None,
            highest_price_achieved: None,
            trailing_stop_active: false,
            metadata: std::collections::HashMap::new(),
        }
    }

    /// Update trade with current market price
    pub fn update_with_price(&mut self, current_price: f64, funding_rate: Option<f64>) {
        if self.status != TradeStatus::Open {
            return;
        }

        // Calculate unrealized PnL
        let price_diff = match self.trade_type {
            TradeType::Long => current_price - self.entry_price,
            TradeType::Short => self.entry_price - current_price,
        };

        self.unrealized_pnl = price_diff * self.quantity - self.trading_fees - self.funding_fees;

        // Calculate PnL percentage based on position value (not margin)
        // This gives users the traditional crypto exchange view of P&L%
        // E.g., 10x leverage with -$256 loss on $10,000 position = -2.56% (not -25.6%)
        let position_value = self.entry_price * self.quantity;
        self.pnl_percentage = if position_value > 0.0 {
            (self.unrealized_pnl / position_value) * 100.0
        } else {
            0.0
        };

        // Update margin ratio (with division-by-zero protection)
        let equity = self.initial_margin + self.unrealized_pnl;
        self.margin_ratio = if self.margin_used > 0.0 {
            equity / self.margin_used
        } else {
            1.0
        };

        // Update max favorable/adverse excursion
        let excursion = price_diff * self.quantity;
        if excursion > 0.0 {
            self.max_favorable_excursion = self.max_favorable_excursion.max(excursion);
        } else {
            self.max_adverse_excursion = self.max_adverse_excursion.min(excursion);
        }

        // Add funding fees if provided (Binance Futures funding every 8 hours)
        if let Some(rate) = funding_rate {
            let notional_value = self.quantity * current_price;
            let funding_fee = notional_value * rate;

            // For long positions, we pay funding if rate is positive
            // For short positions, we pay funding if rate is negative
            match self.trade_type {
                TradeType::Long => self.funding_fees += funding_fee,
                TradeType::Short => self.funding_fees -= funding_fee,
            }
        }
    }

    /// Check if trade should be closed due to stop loss
    pub fn should_stop_loss(&self, current_price: f64) -> bool {
        if let Some(stop_loss) = self.stop_loss {
            match self.trade_type {
                TradeType::Long => current_price <= stop_loss,
                TradeType::Short => current_price >= stop_loss,
            }
        } else {
            false
        }
    }

    /// Check if trade should be closed due to take profit
    pub fn should_take_profit(&self, current_price: f64) -> bool {
        if let Some(take_profit) = self.take_profit {
            match self.trade_type {
                TradeType::Long => current_price >= take_profit,
                TradeType::Short => current_price <= take_profit,
            }
        } else {
            false
        }
    }

    /// Update trailing stop based on current price
    ///
    /// This method implements a trailing stop-loss that moves with price in favorable direction
    /// but never moves back. It only activates after a minimum profit threshold is reached.
    ///
    /// # Arguments
    /// * `current_price` - Current market price
    /// * `trailing_pct` - Percentage to trail behind high/low (e.g., 3.0 = 3%)
    /// * `activation_pct` - Minimum profit % before trailing activates (e.g., 5.0 = 5%)
    ///
    /// # Behavior
    /// - For Long: Stop trails below highest price achieved
    /// - For Short: Stop trails above lowest price achieved
    /// - Stop only moves in favorable direction (never moves back)
    /// - Activates only after profit >= activation_pct
    ///
    /// @spec:FR-RISK-007 - Trailing Stop Loss for Long Positions
    /// @spec:FR-RISK-008 - Trailing Stop Loss for Short Positions
    /// @ref:specs/01-requirements/1.1-functional-requirements/FR-RISK.md#fr-risk-007
    /// @ref:specs/01-requirements/1.1-functional-requirements/FR-RISK.md#fr-risk-008
    /// @ref:specs/02-design/2.5-components/COMP-RUST-TRADING.md#trailing-stop-component
    /// @test:TC-TRADING-015
    pub fn update_trailing_stop(
        &mut self,
        current_price: f64,
        trailing_pct: f64,
        activation_pct: f64,
    ) {
        // Only for open trades
        if self.status != TradeStatus::Open {
            return;
        }

        // Calculate profit percentage (price-based)
        let profit_pct = match self.trade_type {
            TradeType::Long => ((current_price - self.entry_price) / self.entry_price) * 100.0,
            TradeType::Short => ((self.entry_price - current_price) / self.entry_price) * 100.0,
        };

        // PnL-based activation: activation_pct is PnL%, so multiply price change by leverage
        let pnl_pct = profit_pct * self.leverage as f64;

        // Check if PnL threshold met to activate trailing
        if !self.trailing_stop_active && pnl_pct >= activation_pct {
            self.trailing_stop_active = true;
            self.highest_price_achieved = Some(current_price);
            tracing::info!(
                "🎯 Trailing stop ACTIVATED for {} at ${:.2} (+{:.2}%)",
                self.symbol,
                current_price,
                profit_pct
            );
        }

        // Update highest/lowest price achieved
        match self.trade_type {
            TradeType::Long => {
                // Track highest price for long positions
                if let Some(highest) = self.highest_price_achieved {
                    if current_price > highest {
                        self.highest_price_achieved = Some(current_price);
                    }
                }
            },
            TradeType::Short => {
                // Track lowest price for short positions
                if let Some(lowest) = self.highest_price_achieved {
                    if current_price < lowest {
                        self.highest_price_achieved = Some(current_price);
                    }
                }
            },
        }

        // Update stop loss if trailing is active
        if self.trailing_stop_active {
            if let Some(best_price) = self.highest_price_achieved {
                let new_stop = match self.trade_type {
                    TradeType::Long => {
                        // Stop trails below high by trailing_pct
                        let trail_stop = best_price * (1.0 - trailing_pct / 100.0);

                        // Only move stop UP, never down
                        if let Some(current_stop) = self.stop_loss {
                            if trail_stop > current_stop {
                                Some(trail_stop)
                            } else {
                                Some(current_stop) // Keep current
                            }
                        } else {
                            Some(trail_stop) // Set initial trailing stop
                        }
                    },
                    TradeType::Short => {
                        // Stop trails above low by trailing_pct
                        let trail_stop = best_price * (1.0 + trailing_pct / 100.0);

                        // Only move stop DOWN, never up
                        if let Some(current_stop) = self.stop_loss {
                            if trail_stop < current_stop {
                                Some(trail_stop)
                            } else {
                                Some(current_stop) // Keep current
                            }
                        } else {
                            Some(trail_stop) // Set initial trailing stop
                        }
                    },
                };

                // Update if changed
                if new_stop != self.stop_loss {
                    let old_stop = self.stop_loss.unwrap_or(0.0);
                    self.stop_loss = new_stop;
                    tracing::info!(
                        "📈 Trailing SL updated: {} ${:.2} → ${:.2} (best: ${:.2})",
                        self.symbol,
                        old_stop,
                        new_stop.unwrap_or(0.0),
                        best_price
                    );
                }
            }
        }
    }

    /// Check if trade is at risk of liquidation
    pub fn is_at_liquidation_risk(&self, current_price: f64) -> bool {
        // Binance uses a more complex liquidation calculation, but this is a simplified version
        let bankruptcy_price = match self.trade_type {
            TradeType::Long => self.entry_price * (1.0 - 1.0 / self.leverage as f64),
            TradeType::Short => self.entry_price * (1.0 + 1.0 / self.leverage as f64),
        };

        match self.trade_type {
            TradeType::Long => current_price <= bankruptcy_price * 1.05, // 5% margin
            TradeType::Short => current_price >= bankruptcy_price * 0.95, // 5% margin
        }
    }

    /// Close the trade
    pub fn close(
        &mut self,
        exit_price: f64,
        close_reason: CloseReason,
        additional_fees: f64,
    ) -> Result<()> {
        if self.status != TradeStatus::Open {
            return Err(anyhow::anyhow!("Trade is not open"));
        }

        self.exit_price = Some(exit_price);
        self.status = TradeStatus::Closed;
        self.close_reason = Some(close_reason);
        self.close_time = Some(Utc::now());

        // Calculate final PnL
        let price_diff = match self.trade_type {
            TradeType::Long => exit_price - self.entry_price,
            TradeType::Short => self.entry_price - exit_price,
        };

        self.realized_pnl = Some(
            price_diff * self.quantity - self.trading_fees - self.funding_fees - additional_fees,
        );

        // Calculate duration
        if let Some(close_time) = self.close_time {
            self.duration_ms = Some((close_time - self.open_time).num_milliseconds());
        }

        Ok(())
    }

    /// Cancel the trade
    pub fn cancel(&mut self, reason: String) -> Result<()> {
        if self.status != TradeStatus::Open {
            return Err(anyhow::anyhow!("Trade is not open"));
        }

        self.status = TradeStatus::Cancelled;
        self.close_time = Some(Utc::now());
        self.close_reason = Some(CloseReason::Manual);
        self.metadata.insert(
            "cancel_reason".to_string(),
            serde_json::Value::String(reason),
        );

        Ok(())
    }

    /// Set stop loss
    pub fn set_stop_loss(&mut self, stop_loss: f64) -> Result<()> {
        // Validate stop loss makes sense
        match self.trade_type {
            TradeType::Long => {
                if stop_loss >= self.entry_price {
                    return Err(anyhow::anyhow!(
                        "Stop loss must be below entry price for long trades"
                    ));
                }
            },
            TradeType::Short => {
                if stop_loss <= self.entry_price {
                    return Err(anyhow::anyhow!(
                        "Stop loss must be above entry price for short trades"
                    ));
                }
            },
        }

        self.stop_loss = Some(stop_loss);
        Ok(())
    }

    /// Set take profit
    pub fn set_take_profit(&mut self, take_profit: f64) -> Result<()> {
        // Validate take profit makes sense
        match self.trade_type {
            TradeType::Long => {
                if take_profit <= self.entry_price {
                    return Err(anyhow::anyhow!(
                        "Take profit must be above entry price for long trades"
                    ));
                }
            },
            TradeType::Short => {
                if take_profit >= self.entry_price {
                    return Err(anyhow::anyhow!(
                        "Take profit must be below entry price for short trades"
                    ));
                }
            },
        }

        self.take_profit = Some(take_profit);
        Ok(())
    }

    /// Get trade summary for display
    pub fn get_summary(&self) -> TradeSummary {
        TradeSummary {
            id: self.id.clone(),
            symbol: self.symbol.clone(),
            trade_type: self.trade_type,
            status: self.status,
            entry_price: self.entry_price,
            exit_price: self.exit_price,
            quantity: self.quantity,
            leverage: self.leverage,
            stop_loss: self.stop_loss,
            take_profit: self.take_profit,
            pnl: if self.status == TradeStatus::Closed {
                self.realized_pnl
            } else {
                Some(self.unrealized_pnl)
            },
            pnl_percentage: self.pnl_percentage,
            duration_ms: self.duration_ms,
            open_time: self.open_time,
            close_time: self.close_time,
            close_reason: self.close_reason.clone(),
        }
    }
}

/// Simplified trade summary for API responses
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TradeSummary {
    pub id: String,
    pub symbol: String,
    pub trade_type: TradeType,
    pub status: TradeStatus,
    pub entry_price: f64,
    pub exit_price: Option<f64>,
    pub quantity: f64,
    pub leverage: u8,
    pub stop_loss: Option<f64>,
    pub take_profit: Option<f64>,
    pub pnl: Option<f64>,
    pub pnl_percentage: f64,
    pub duration_ms: Option<i64>,
    pub open_time: DateTime<Utc>,
    pub close_time: Option<DateTime<Utc>>,
    pub close_reason: Option<CloseReason>,
}

impl TradeType {
    pub fn as_str(&self) -> &'static str {
        match self {
            TradeType::Long => "Long",
            TradeType::Short => "Short",
        }
    }

    pub fn from_string(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "long" | "buy" => Some(TradeType::Long),
            "short" | "sell" => Some(TradeType::Short),
            _ => None,
        }
    }
}

impl TradeStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            TradeStatus::Open => "Open",
            TradeStatus::Closed => "Closed",
            TradeStatus::Cancelled => "Cancelled",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_trade_creation() {
        let trade = PaperTrade::new(
            "BTCUSDT".to_string(),
            TradeType::Long,
            50000.0,
            0.1,
            10,
            0.0004,
            Some("signal123".to_string()),
            Some(0.85),
            Some("Test reasoning".to_string()),
        );

        assert_eq!(trade.symbol, "BTCUSDT");
        assert_eq!(trade.trade_type, TradeType::Long);
        assert_eq!(trade.entry_price, 50000.0);
        assert_eq!(trade.quantity, 0.1);
        assert_eq!(trade.leverage, 10);
        assert_eq!(trade.status, TradeStatus::Open);
        assert_eq!(trade.initial_margin, 500.0); // (0.1 * 50000) / 10
        assert_eq!(trade.trading_fees, 2.0); // 0.1 * 50000 * 0.0004
    }

    #[test]
    fn test_long_pnl_calculation_profit() {
        let mut trade = PaperTrade::new(
            "BTCUSDT".to_string(),
            TradeType::Long,
            50000.0,
            0.1,
            10,
            0.0004,
            None,
            None,
            None,
        );

        // Price goes up 10%
        trade.update_with_price(55000.0, None);

        // Expected: (55000 - 50000) * 0.1 - fees = 500 - 2 = 498
        assert!((trade.unrealized_pnl - 498.0).abs() < 0.01);
        // PnL % is now based on position_value (not margin): 498/5000 * 100 = 9.96%
        assert!((trade.pnl_percentage - 9.96).abs() < 0.1);
    }

    #[test]
    fn test_long_pnl_calculation_loss() {
        let mut trade = PaperTrade::new(
            "BTCUSDT".to_string(),
            TradeType::Long,
            50000.0,
            0.1,
            10,
            0.0004,
            None,
            None,
            None,
        );

        // Price goes down 5%
        trade.update_with_price(47500.0, None);

        // Expected: (47500 - 50000) * 0.1 - fees = -250 - 2 = -252
        assert!((trade.unrealized_pnl - (-252.0)).abs() < 0.01);
        // PnL % is now based on position_value (not margin): -252/5000 * 100 = -5.04%
        assert!((trade.pnl_percentage - (-5.04)).abs() < 0.1);
    }

    #[test]
    fn test_short_pnl_calculation_profit() {
        let mut trade = PaperTrade::new(
            "BTCUSDT".to_string(),
            TradeType::Short,
            50000.0,
            0.1,
            10,
            0.0004,
            None,
            None,
            None,
        );

        // Price goes down 10%
        trade.update_with_price(45000.0, None);

        // Expected: (50000 - 45000) * 0.1 - fees = 500 - 2 = 498
        assert!((trade.unrealized_pnl - 498.0).abs() < 0.01);
    }

    #[test]
    fn test_short_pnl_calculation_loss() {
        let mut trade = PaperTrade::new(
            "BTCUSDT".to_string(),
            TradeType::Short,
            50000.0,
            0.1,
            10,
            0.0004,
            None,
            None,
            None,
        );

        // Price goes up 5%
        trade.update_with_price(52500.0, None);

        // Expected: (50000 - 52500) * 0.1 - fees = -250 - 2 = -252
        assert!((trade.unrealized_pnl - (-252.0)).abs() < 0.01);
    }

    #[test]
    fn test_stop_loss_long_triggered() {
        let mut trade = PaperTrade::new(
            "BTCUSDT".to_string(),
            TradeType::Long,
            50000.0,
            0.1,
            10,
            0.0004,
            None,
            None,
            None,
        );

        trade.set_stop_loss(48000.0).unwrap();

        // Price at stop loss
        assert!(trade.should_stop_loss(48000.0));
        // Price below stop loss
        assert!(trade.should_stop_loss(47000.0));
        // Price above stop loss
        assert!(!trade.should_stop_loss(49000.0));
    }

    #[test]
    fn test_stop_loss_short_triggered() {
        let mut trade = PaperTrade::new(
            "BTCUSDT".to_string(),
            TradeType::Short,
            50000.0,
            0.1,
            10,
            0.0004,
            None,
            None,
            None,
        );

        trade.set_stop_loss(52000.0).unwrap();

        // Price at stop loss
        assert!(trade.should_stop_loss(52000.0));
        // Price above stop loss
        assert!(trade.should_stop_loss(53000.0));
        // Price below stop loss
        assert!(!trade.should_stop_loss(51000.0));
    }

    #[test]
    fn test_take_profit_long_triggered() {
        let mut trade = PaperTrade::new(
            "BTCUSDT".to_string(),
            TradeType::Long,
            50000.0,
            0.1,
            10,
            0.0004,
            None,
            None,
            None,
        );

        trade.set_take_profit(55000.0).unwrap();

        // Price at take profit
        assert!(trade.should_take_profit(55000.0));
        // Price above take profit
        assert!(trade.should_take_profit(56000.0));
        // Price below take profit
        assert!(!trade.should_take_profit(54000.0));
    }

    #[test]
    fn test_take_profit_short_triggered() {
        let mut trade = PaperTrade::new(
            "BTCUSDT".to_string(),
            TradeType::Short,
            50000.0,
            0.1,
            10,
            0.0004,
            None,
            None,
            None,
        );

        trade.set_take_profit(45000.0).unwrap();

        // Price at take profit
        assert!(trade.should_take_profit(45000.0));
        // Price below take profit
        assert!(trade.should_take_profit(44000.0));
        // Price above take profit
        assert!(!trade.should_take_profit(46000.0));
    }

    #[test]
    fn test_invalid_stop_loss_long() {
        let mut trade = PaperTrade::new(
            "BTCUSDT".to_string(),
            TradeType::Long,
            50000.0,
            0.1,
            10,
            0.0004,
            None,
            None,
            None,
        );

        // Stop loss above entry for long should fail
        assert!(trade.set_stop_loss(51000.0).is_err());
        // Stop loss equal to entry should fail
        assert!(trade.set_stop_loss(50000.0).is_err());
        // Stop loss below entry should succeed
        assert!(trade.set_stop_loss(48000.0).is_ok());
    }

    #[test]
    fn test_invalid_stop_loss_short() {
        let mut trade = PaperTrade::new(
            "BTCUSDT".to_string(),
            TradeType::Short,
            50000.0,
            0.1,
            10,
            0.0004,
            None,
            None,
            None,
        );

        // Stop loss below entry for short should fail
        assert!(trade.set_stop_loss(49000.0).is_err());
        // Stop loss equal to entry should fail
        assert!(trade.set_stop_loss(50000.0).is_err());
        // Stop loss above entry should succeed
        assert!(trade.set_stop_loss(52000.0).is_ok());
    }

    #[test]
    fn test_invalid_take_profit_long() {
        let mut trade = PaperTrade::new(
            "BTCUSDT".to_string(),
            TradeType::Long,
            50000.0,
            0.1,
            10,
            0.0004,
            None,
            None,
            None,
        );

        // Take profit below entry for long should fail
        assert!(trade.set_take_profit(49000.0).is_err());
        // Take profit equal to entry should fail
        assert!(trade.set_take_profit(50000.0).is_err());
        // Take profit above entry should succeed
        assert!(trade.set_take_profit(55000.0).is_ok());
    }

    #[test]
    fn test_invalid_take_profit_short() {
        let mut trade = PaperTrade::new(
            "BTCUSDT".to_string(),
            TradeType::Short,
            50000.0,
            0.1,
            10,
            0.0004,
            None,
            None,
            None,
        );

        // Take profit above entry for short should fail
        assert!(trade.set_take_profit(51000.0).is_err());
        // Take profit equal to entry should fail
        assert!(trade.set_take_profit(50000.0).is_err());
        // Take profit below entry should succeed
        assert!(trade.set_take_profit(45000.0).is_ok());
    }

    #[test]
    fn test_close_trade() {
        let mut trade = PaperTrade::new(
            "BTCUSDT".to_string(),
            TradeType::Long,
            50000.0,
            0.1,
            10,
            0.0004,
            None,
            None,
            None,
        );

        let exit_price = 55000.0;
        let exit_fees = 2.2;

        let result = trade.close(exit_price, CloseReason::TakeProfit, exit_fees);
        assert!(result.is_ok());

        assert_eq!(trade.status, TradeStatus::Closed);
        assert_eq!(trade.exit_price, Some(55000.0));
        assert_eq!(trade.close_reason, Some(CloseReason::TakeProfit));
        assert!(trade.close_time.is_some());
        assert!(trade.duration_ms.is_some());

        // PnL: (55000 - 50000) * 0.1 - entry_fees - exit_fees = 500 - 2 - 2.2 = 495.8
        let expected_pnl = 495.8;
        assert!((trade.realized_pnl.unwrap() - expected_pnl).abs() < 0.01);
    }

    #[test]
    fn test_close_already_closed_trade() {
        let mut trade = PaperTrade::new(
            "BTCUSDT".to_string(),
            TradeType::Long,
            50000.0,
            0.1,
            10,
            0.0004,
            None,
            None,
            None,
        );

        // Close the trade
        trade.close(55000.0, CloseReason::Manual, 2.0).unwrap();

        // Try to close again
        let result = trade.close(56000.0, CloseReason::Manual, 2.0);
        assert!(result.is_err());
    }

    #[test]
    fn test_liquidation_risk_long() {
        let trade = PaperTrade::new(
            "BTCUSDT".to_string(),
            TradeType::Long,
            50000.0,
            0.1,
            10,
            0.0004,
            None,
            None,
            None,
        );

        // Bankruptcy price for long at 10x: 50000 * (1 - 1/10) = 45000
        // Liquidation risk at 45000 * 1.05 = 47250
        assert!(trade.is_at_liquidation_risk(47000.0));
        assert!(trade.is_at_liquidation_risk(45000.0));
        assert!(!trade.is_at_liquidation_risk(48000.0));
    }

    #[test]
    fn test_liquidation_risk_short() {
        let trade = PaperTrade::new(
            "BTCUSDT".to_string(),
            TradeType::Short,
            50000.0,
            0.1,
            10,
            0.0004,
            None,
            None,
            None,
        );

        // Bankruptcy price for short at 10x: 50000 * (1 + 1/10) = 55000
        // Liquidation risk at 55000 * 0.95 = 52250
        assert!(trade.is_at_liquidation_risk(53000.0));
        assert!(trade.is_at_liquidation_risk(55000.0));
        assert!(!trade.is_at_liquidation_risk(52000.0));
    }

    #[test]
    fn test_extreme_leverage() {
        let trade = PaperTrade::new(
            "BTCUSDT".to_string(),
            TradeType::Long,
            50000.0,
            0.1,
            125,
            0.0004,
            None,
            None,
            None,
        );

        // Initial margin at 125x: (0.1 * 50000) / 125 = 40
        assert!((trade.initial_margin - 40.0).abs() < 0.01);
        assert_eq!(trade.maintenance_margin, 750.0); // 5000 * 0.15
    }

    #[test]
    fn test_funding_fees_long() {
        let mut trade = PaperTrade::new(
            "BTCUSDT".to_string(),
            TradeType::Long,
            50000.0,
            0.1,
            10,
            0.0004,
            None,
            None,
            None,
        );

        // Positive funding rate (longs pay shorts)
        trade.update_with_price(50000.0, Some(0.0001));

        // Funding fee: 0.1 * 50000 * 0.0001 = 0.5
        assert!((trade.funding_fees - 0.5).abs() < 0.01);
    }

    #[test]
    fn test_funding_fees_short() {
        let mut trade = PaperTrade::new(
            "BTCUSDT".to_string(),
            TradeType::Short,
            50000.0,
            0.1,
            10,
            0.0004,
            None,
            None,
            None,
        );

        // Positive funding rate (shorts receive from longs)
        trade.update_with_price(50000.0, Some(0.0001));

        // Funding fee: -(0.1 * 50000 * 0.0001) = -0.5 (negative means we receive)
        assert!((trade.funding_fees - (-0.5)).abs() < 0.01);
    }

    #[test]
    fn test_max_favorable_excursion() {
        let mut trade = PaperTrade::new(
            "BTCUSDT".to_string(),
            TradeType::Long,
            50000.0,
            0.1,
            10,
            0.0004,
            None,
            None,
            None,
        );

        trade.update_with_price(55000.0, None);
        assert!((trade.max_favorable_excursion - 500.0).abs() < 0.01);

        trade.update_with_price(52000.0, None);
        // MFE should remain at peak
        assert!((trade.max_favorable_excursion - 500.0).abs() < 0.01);
    }

    #[test]
    fn test_max_adverse_excursion() {
        let mut trade = PaperTrade::new(
            "BTCUSDT".to_string(),
            TradeType::Long,
            50000.0,
            0.1,
            10,
            0.0004,
            None,
            None,
            None,
        );

        trade.update_with_price(48000.0, None);
        assert!((trade.max_adverse_excursion - (-200.0)).abs() < 0.01);

        trade.update_with_price(49000.0, None);
        // MAE should remain at worst
        assert!((trade.max_adverse_excursion - (-200.0)).abs() < 0.01);
    }

    #[test]
    fn test_zero_quantity() {
        let trade = PaperTrade::new(
            "BTCUSDT".to_string(),
            TradeType::Long,
            50000.0,
            0.0,
            10,
            0.0004,
            None,
            None,
            None,
        );

        assert_eq!(trade.initial_margin, 0.0);
        assert_eq!(trade.trading_fees, 0.0);
    }

    #[test]
    fn test_cancel_trade() {
        let mut trade = PaperTrade::new(
            "BTCUSDT".to_string(),
            TradeType::Long,
            50000.0,
            0.1,
            10,
            0.0004,
            None,
            None,
            None,
        );

        let result = trade.cancel("User requested cancellation".to_string());
        assert!(result.is_ok());
        assert_eq!(trade.status, TradeStatus::Cancelled);
        assert!(trade.close_time.is_some());
    }

    #[test]
    fn test_trade_type_conversion() {
        assert_eq!(TradeType::Long.as_str(), "Long");
        assert_eq!(TradeType::Short.as_str(), "Short");

        assert_eq!(TradeType::from_string("long"), Some(TradeType::Long));
        assert_eq!(TradeType::from_string("buy"), Some(TradeType::Long));
        assert_eq!(TradeType::from_string("short"), Some(TradeType::Short));
        assert_eq!(TradeType::from_string("sell"), Some(TradeType::Short));
        assert_eq!(TradeType::from_string("invalid"), None);
    }

    #[test]
    fn test_margin_ratio_calculation() {
        let mut trade = PaperTrade::new(
            "BTCUSDT".to_string(),
            TradeType::Long,
            50000.0,
            0.1,
            10,
            0.0004,
            None,
            None,
            None,
        );

        // Initial margin ratio should be 1.0
        assert!((trade.margin_ratio - 1.0).abs() < 0.01);

        // Update with profit
        trade.update_with_price(55000.0, None);
        // Equity: 500 + 498 (PnL) = 998
        // Margin ratio: 998 / 500 ≈ 1.996
        assert!(trade.margin_ratio > 1.5);

        // Update with loss
        trade.update_with_price(45000.0, None);
        // Should have lower margin ratio
        assert!(trade.margin_ratio < 1.0);
    }

    #[test]
    fn test_trade_summary_open_trade() {
        let mut trade = PaperTrade::new(
            "ETHUSDT".to_string(),
            TradeType::Long,
            3000.0,
            1.0,
            5,
            0.0004,
            Some("signal456".to_string()),
            Some(0.75),
            Some("AI prediction".to_string()),
        );

        trade.update_with_price(3100.0, None);
        let summary = trade.get_summary();

        assert_eq!(summary.symbol, "ETHUSDT");
        assert_eq!(summary.trade_type, TradeType::Long);
        assert_eq!(summary.status, TradeStatus::Open);
        assert_eq!(summary.entry_price, 3000.0);
        assert!(summary.exit_price.is_none());
        assert!(summary.pnl.is_some());
        assert!(summary.close_time.is_none());
    }

    #[test]
    fn test_trade_summary_closed_trade() {
        let mut trade = PaperTrade::new(
            "ETHUSDT".to_string(),
            TradeType::Long,
            3000.0,
            1.0,
            5,
            0.0004,
            None,
            None,
            None,
        );

        trade.close(3200.0, CloseReason::TakeProfit, 1.2).unwrap();
        let summary = trade.get_summary();

        assert_eq!(summary.status, TradeStatus::Closed);
        assert_eq!(summary.exit_price, Some(3200.0));
        assert!(summary.pnl.is_some());
        assert!(summary.close_time.is_some());
        assert!(summary.duration_ms.is_some());
    }

    #[test]
    fn test_maintenance_margin_rates() {
        // Test different leverage tiers for maintenance margin calculation
        let trade_1x = PaperTrade::new(
            "BTCUSDT".to_string(),
            TradeType::Long,
            50000.0,
            0.1,
            1,
            0.0004,
            None,
            None,
            None,
        );
        assert_eq!(trade_1x.maintenance_margin, 5000.0 * 0.01); // 1%

        let trade_5x = PaperTrade::new(
            "BTCUSDT".to_string(),
            TradeType::Long,
            50000.0,
            0.1,
            5,
            0.0004,
            None,
            None,
            None,
        );
        assert_eq!(trade_5x.maintenance_margin, 5000.0 * 0.01); // 1%

        let trade_10x = PaperTrade::new(
            "BTCUSDT".to_string(),
            TradeType::Long,
            50000.0,
            0.1,
            10,
            0.0004,
            None,
            None,
            None,
        );
        assert_eq!(trade_10x.maintenance_margin, 5000.0 * 0.025); // 2.5%

        let trade_20x = PaperTrade::new(
            "BTCUSDT".to_string(),
            TradeType::Long,
            50000.0,
            0.1,
            20,
            0.0004,
            None,
            None,
            None,
        );
        assert_eq!(trade_20x.maintenance_margin, 5000.0 * 0.05); // 5%

        let trade_50x = PaperTrade::new(
            "BTCUSDT".to_string(),
            TradeType::Long,
            50000.0,
            0.1,
            50,
            0.0004,
            None,
            None,
            None,
        );
        assert_eq!(trade_50x.maintenance_margin, 5000.0 * 0.1); // 10%

        let trade_100x = PaperTrade::new(
            "BTCUSDT".to_string(),
            TradeType::Long,
            50000.0,
            0.1,
            100,
            0.0004,
            None,
            None,
            None,
        );
        assert_eq!(trade_100x.maintenance_margin, 5000.0 * 0.125); // 12.5%

        let trade_125x = PaperTrade::new(
            "BTCUSDT".to_string(),
            TradeType::Long,
            50000.0,
            0.1,
            125,
            0.0004,
            None,
            None,
            None,
        );
        assert_eq!(trade_125x.maintenance_margin, 5000.0 * 0.15); // 15%
    }

    #[test]
    fn test_update_closed_trade_has_no_effect() {
        let mut trade = PaperTrade::new(
            "BTCUSDT".to_string(),
            TradeType::Long,
            50000.0,
            0.1,
            10,
            0.0004,
            None,
            None,
            None,
        );

        trade.close(55000.0, CloseReason::TakeProfit, 2.0).unwrap();
        let pnl_after_close = trade.realized_pnl;

        // Update should have no effect on closed trade
        trade.update_with_price(60000.0, None);
        assert_eq!(trade.realized_pnl, pnl_after_close);
        assert_eq!(trade.unrealized_pnl, 0.0);
    }

    #[test]
    fn test_update_cancelled_trade_has_no_effect() {
        let mut trade = PaperTrade::new(
            "BTCUSDT".to_string(),
            TradeType::Long,
            50000.0,
            0.1,
            10,
            0.0004,
            None,
            None,
            None,
        );

        trade.cancel("Test cancellation".to_string()).unwrap();

        // Update should have no effect on cancelled trade
        trade.update_with_price(60000.0, None);
        assert_eq!(trade.unrealized_pnl, 0.0);
    }

    #[test]
    fn test_cancel_already_cancelled_trade() {
        let mut trade = PaperTrade::new(
            "BTCUSDT".to_string(),
            TradeType::Long,
            50000.0,
            0.1,
            10,
            0.0004,
            None,
            None,
            None,
        );

        trade.cancel("First cancellation".to_string()).unwrap();
        let result = trade.cancel("Second cancellation".to_string());
        assert!(result.is_err());
    }

    #[test]
    fn test_cancel_already_closed_trade() {
        let mut trade = PaperTrade::new(
            "BTCUSDT".to_string(),
            TradeType::Long,
            50000.0,
            0.1,
            10,
            0.0004,
            None,
            None,
            None,
        );

        trade.close(55000.0, CloseReason::Manual, 2.0).unwrap();
        let result = trade.cancel("Trying to cancel".to_string());
        assert!(result.is_err());
    }

    #[test]
    fn test_extreme_price_long_massive_profit() {
        let mut trade = PaperTrade::new(
            "BTCUSDT".to_string(),
            TradeType::Long,
            50000.0,
            0.1,
            10,
            0.0004,
            None,
            None,
            None,
        );

        // Price doubles (100% increase)
        trade.update_with_price(100000.0, None);

        // PnL: (100000 - 50000) * 0.1 - fees = 5000 - 2 = 4998
        assert!((trade.unrealized_pnl - 4998.0).abs() < 0.01);
        // PnL % is now based on position_value: 4998/5000 * 100 = 99.96%
        assert!(trade.pnl_percentage > 95.0);
    }

    #[test]
    fn test_extreme_price_short_massive_profit() {
        let mut trade = PaperTrade::new(
            "BTCUSDT".to_string(),
            TradeType::Short,
            50000.0,
            0.1,
            10,
            0.0004,
            None,
            None,
            None,
        );

        // Price halves (50% decrease)
        trade.update_with_price(25000.0, None);

        // PnL: (50000 - 25000) * 0.1 - fees = 2500 - 2 = 2498
        assert!((trade.unrealized_pnl - 2498.0).abs() < 0.01);
        // PnL % is now based on position_value: 2498/5000 * 100 = 49.96%
        assert!(trade.pnl_percentage > 45.0);
    }

    #[test]
    fn test_extreme_price_near_zero() {
        let mut trade = PaperTrade::new(
            "BTCUSDT".to_string(),
            TradeType::Long,
            50000.0,
            0.1,
            10,
            0.0004,
            None,
            None,
            None,
        );

        // Price crashes near zero
        trade.update_with_price(0.01, None);

        // Massive loss
        assert!(trade.unrealized_pnl < -4999.0);
        assert!(trade.is_at_liquidation_risk(0.01));
    }

    #[test]
    fn test_negative_funding_rate_long() {
        let mut trade = PaperTrade::new(
            "BTCUSDT".to_string(),
            TradeType::Long,
            50000.0,
            0.1,
            10,
            0.0004,
            None,
            None,
            None,
        );

        // Negative funding rate (longs receive from shorts)
        trade.update_with_price(50000.0, Some(-0.0001));

        // Funding fee: 0.1 * 50000 * (-0.0001) = -0.5
        assert!((trade.funding_fees - (-0.5)).abs() < 0.01);
    }

    #[test]
    fn test_negative_funding_rate_short() {
        let mut trade = PaperTrade::new(
            "BTCUSDT".to_string(),
            TradeType::Short,
            50000.0,
            0.1,
            10,
            0.0004,
            None,
            None,
            None,
        );

        // Negative funding rate (shorts pay longs)
        trade.update_with_price(50000.0, Some(-0.0001));

        // Funding fee: -(0.1 * 50000 * (-0.0001)) = 0.5
        assert!((trade.funding_fees - 0.5).abs() < 0.01);
    }

    #[test]
    fn test_accumulated_funding_fees() {
        let mut trade = PaperTrade::new(
            "BTCUSDT".to_string(),
            TradeType::Long,
            50000.0,
            0.1,
            10,
            0.0004,
            None,
            None,
            None,
        );

        // Multiple funding periods
        trade.update_with_price(50000.0, Some(0.0001));
        trade.update_with_price(51000.0, Some(0.0001));
        trade.update_with_price(52000.0, Some(0.0001));

        // First: 5000 * 0.0001 = 0.5
        // Second: 5100 * 0.0001 = 0.51
        // Third: 5200 * 0.0001 = 0.52
        // Total ≈ 1.53
        assert!((trade.funding_fees - 1.53).abs() < 0.01);
    }

    #[test]
    fn test_margin_ratio_with_zero_margin_used() {
        let mut trade = PaperTrade::new(
            "BTCUSDT".to_string(),
            TradeType::Long,
            50000.0,
            0.1,
            10,
            0.0004,
            None,
            None,
            None,
        );

        // Set margin_used to 0 to test edge case
        trade.margin_used = 0.0;
        trade.update_with_price(55000.0, None);

        // Should default to 1.0 when margin_used is 0
        assert!((trade.margin_ratio - 1.0).abs() < 0.01);
    }

    #[test]
    fn test_close_with_different_reasons() {
        let reasons = vec![
            CloseReason::TakeProfit,
            CloseReason::StopLoss,
            CloseReason::Manual,
            CloseReason::AISignal,
            CloseReason::RiskManagement,
            CloseReason::MarginCall,
            CloseReason::TimeBasedExit,
        ];

        for reason in reasons {
            let mut trade = PaperTrade::new(
                "BTCUSDT".to_string(),
                TradeType::Long,
                50000.0,
                0.1,
                10,
                0.0004,
                None,
                None,
                None,
            );

            trade.close(55000.0, reason.clone(), 2.0).unwrap();
            assert_eq!(trade.close_reason, Some(reason));
        }
    }

    #[test]
    fn test_trade_type_display() {
        let long = TradeType::Long;
        let short = TradeType::Short;

        assert_eq!(format!("{}", long), "Long");
        assert_eq!(format!("{}", short), "Short");
    }

    #[test]
    fn test_trade_type_case_insensitive_parsing() {
        assert_eq!(TradeType::from_string("LONG"), Some(TradeType::Long));
        assert_eq!(TradeType::from_string("Long"), Some(TradeType::Long));
        assert_eq!(TradeType::from_string("BUY"), Some(TradeType::Long));
        assert_eq!(TradeType::from_string("Buy"), Some(TradeType::Long));
        assert_eq!(TradeType::from_string("SHORT"), Some(TradeType::Short));
        assert_eq!(TradeType::from_string("Short"), Some(TradeType::Short));
        assert_eq!(TradeType::from_string("SELL"), Some(TradeType::Short));
        assert_eq!(TradeType::from_string("Sell"), Some(TradeType::Short));
    }

    #[test]
    fn test_trade_status_as_str() {
        assert_eq!(TradeStatus::Open.as_str(), "Open");
        assert_eq!(TradeStatus::Closed.as_str(), "Closed");
        assert_eq!(TradeStatus::Cancelled.as_str(), "Cancelled");
    }

    #[test]
    fn test_serialization_deserialization() {
        let trade = PaperTrade::new(
            "BTCUSDT".to_string(),
            TradeType::Long,
            50000.0,
            0.1,
            10,
            0.0004,
            Some("signal789".to_string()),
            Some(0.9),
            Some("Strong buy signal".to_string()),
        );

        let serialized = serde_json::to_string(&trade).unwrap();
        let deserialized: PaperTrade = serde_json::from_str(&serialized).unwrap();

        assert_eq!(trade.id, deserialized.id);
        assert_eq!(trade.symbol, deserialized.symbol);
        assert_eq!(trade.trade_type, deserialized.trade_type);
        assert_eq!(trade.entry_price, deserialized.entry_price);
        assert_eq!(trade.quantity, deserialized.quantity);
        assert_eq!(trade.leverage, deserialized.leverage);
    }

    #[test]
    fn test_trade_summary_serialization() {
        let trade = PaperTrade::new(
            "ETHUSDT".to_string(),
            TradeType::Short,
            3000.0,
            1.0,
            5,
            0.0004,
            None,
            None,
            None,
        );

        let summary = trade.get_summary();
        let serialized = serde_json::to_string(&summary).unwrap();
        let deserialized: TradeSummary = serde_json::from_str(&serialized).unwrap();

        assert_eq!(summary.symbol, deserialized.symbol);
        assert_eq!(summary.trade_type, deserialized.trade_type);
        assert_eq!(summary.status, deserialized.status);
    }

    #[test]
    fn test_very_high_leverage_liquidation() {
        let trade = PaperTrade::new(
            "BTCUSDT".to_string(),
            TradeType::Long,
            50000.0,
            0.1,
            100,
            0.0004,
            None,
            None,
            None,
        );

        // At 100x, bankruptcy price: 50000 * (1 - 1/100) = 49500
        // Liquidation risk at 49500 * 1.05 = 51975
        assert!(trade.is_at_liquidation_risk(51000.0));
        assert!(trade.is_at_liquidation_risk(49500.0));
    }

    #[test]
    fn test_low_leverage_liquidation_resistance() {
        let trade = PaperTrade::new(
            "BTCUSDT".to_string(),
            TradeType::Long,
            50000.0,
            0.1,
            2,
            0.0004,
            None,
            None,
            None,
        );

        // At 2x, bankruptcy price: 50000 * (1 - 1/2) = 25000
        // Liquidation risk at 25000 * 1.05 = 26250
        assert!(!trade.is_at_liquidation_risk(30000.0));
        assert!(trade.is_at_liquidation_risk(26000.0));
    }

    #[test]
    fn test_stop_loss_and_take_profit_both_set() {
        let mut trade = PaperTrade::new(
            "BTCUSDT".to_string(),
            TradeType::Long,
            50000.0,
            0.1,
            10,
            0.0004,
            None,
            None,
            None,
        );

        trade.set_stop_loss(48000.0).unwrap();
        trade.set_take_profit(55000.0).unwrap();

        assert_eq!(trade.stop_loss, Some(48000.0));
        assert_eq!(trade.take_profit, Some(55000.0));

        // Test both conditions
        assert!(trade.should_stop_loss(47500.0));
        assert!(!trade.should_take_profit(47500.0));

        assert!(!trade.should_stop_loss(56000.0));
        assert!(trade.should_take_profit(56000.0));
    }

    #[test]
    fn test_close_with_funding_fees() {
        let mut trade = PaperTrade::new(
            "BTCUSDT".to_string(),
            TradeType::Long,
            50000.0,
            0.1,
            10,
            0.0004,
            None,
            None,
            None,
        );

        // Accumulate funding fees
        trade.update_with_price(50000.0, Some(0.0001));
        trade.update_with_price(51000.0, Some(0.0001));

        let exit_price = 52000.0;
        let exit_fees = 2.08; // 52000 * 0.1 * 0.0004

        trade
            .close(exit_price, CloseReason::TakeProfit, exit_fees)
            .unwrap();

        // PnL: (52000 - 50000) * 0.1 - entry_fees - funding_fees - exit_fees
        // = 200 - 2 - ~1.01 - 2.08 ≈ 194.91
        assert!(trade.realized_pnl.unwrap() < 195.0);
        assert!(trade.realized_pnl.unwrap() > 194.0);
    }

    #[test]
    fn test_metadata_on_cancel() {
        let mut trade = PaperTrade::new(
            "BTCUSDT".to_string(),
            TradeType::Long,
            50000.0,
            0.1,
            10,
            0.0004,
            None,
            None,
            None,
        );

        let cancel_reason = "Risk limit exceeded".to_string();
        trade.cancel(cancel_reason.clone()).unwrap();

        assert!(trade.metadata.contains_key("cancel_reason"));
        assert_eq!(
            trade
                .metadata
                .get("cancel_reason")
                .unwrap()
                .as_str()
                .unwrap(),
            cancel_reason
        );
    }

    #[test]
    fn test_no_stop_loss_never_triggers() {
        let trade = PaperTrade::new(
            "BTCUSDT".to_string(),
            TradeType::Long,
            50000.0,
            0.1,
            10,
            0.0004,
            None,
            None,
            None,
        );

        // No stop loss set, should never trigger
        assert!(!trade.should_stop_loss(0.0));
        assert!(!trade.should_stop_loss(1000000.0));
    }

    #[test]
    fn test_no_take_profit_never_triggers() {
        let trade = PaperTrade::new(
            "BTCUSDT".to_string(),
            TradeType::Long,
            50000.0,
            0.1,
            10,
            0.0004,
            None,
            None,
            None,
        );

        // No take profit set, should never trigger
        assert!(!trade.should_take_profit(0.0));
        assert!(!trade.should_take_profit(1000000.0));
    }

    #[test]
    fn test_ai_fields_preservation() {
        let ai_signal_id = "ai_signal_12345".to_string();
        let ai_confidence = 0.95;
        let ai_reasoning = "Strong bullish momentum detected".to_string();

        let trade = PaperTrade::new(
            "BTCUSDT".to_string(),
            TradeType::Long,
            50000.0,
            0.1,
            10,
            0.0004,
            Some(ai_signal_id.clone()),
            Some(ai_confidence),
            Some(ai_reasoning.clone()),
        );

        assert_eq!(trade.ai_signal_id, Some(ai_signal_id));
        assert_eq!(trade.ai_confidence, Some(ai_confidence));
        assert_eq!(trade.ai_reasoning, Some(ai_reasoning));
    }

    #[test]
    fn test_uuid_uniqueness() {
        let trade1 = PaperTrade::new(
            "BTCUSDT".to_string(),
            TradeType::Long,
            50000.0,
            0.1,
            10,
            0.0004,
            None,
            None,
            None,
        );
        let trade2 = PaperTrade::new(
            "BTCUSDT".to_string(),
            TradeType::Long,
            50000.0,
            0.1,
            10,
            0.0004,
            None,
            None,
            None,
        );

        assert_ne!(trade1.id, trade2.id);
    }

    #[test]
    fn test_extreme_small_quantity() {
        let trade = PaperTrade::new(
            "BTCUSDT".to_string(),
            TradeType::Long,
            50000.0,
            0.00001, // Very small quantity
            10,
            0.0004,
            None,
            None,
            None,
        );

        // Notional: 50000 * 0.00001 = 0.5
        // Initial margin: 0.5 / 10 = 0.05
        assert!((trade.initial_margin - 0.05).abs() < 0.001);
    }

    #[test]
    fn test_price_at_entry_no_pnl_change() {
        let mut trade = PaperTrade::new(
            "BTCUSDT".to_string(),
            TradeType::Long,
            50000.0,
            0.1,
            10,
            0.0004,
            None,
            None,
            None,
        );

        // Update with same price as entry
        trade.update_with_price(50000.0, None);

        // PnL should be negative only due to fees
        assert_eq!(trade.unrealized_pnl, -2.0); // Only trading fees
    }

    // ===== Division-by-Zero Protection Tests =====

    #[test]
    fn test_division_by_zero_protection_zero_initial_margin() {
        let mut trade = PaperTrade::new(
            "BTCUSDT".to_string(),
            TradeType::Long,
            50000.0,
            0.0, // Zero quantity leads to zero initial margin
            10,
            0.0004,
            None,
            None,
            None,
        );

        // Update with price - should not panic with division by zero
        trade.update_with_price(55000.0, None);

        // PnL percentage should be 0.0 when initial margin is zero
        assert_eq!(trade.pnl_percentage, 0.0);
        assert_eq!(trade.unrealized_pnl, 0.0);
    }

    #[test]
    fn test_division_by_zero_protection_zero_margin_used() {
        let mut trade = PaperTrade::new(
            "BTCUSDT".to_string(),
            TradeType::Long,
            50000.0,
            0.1,
            10,
            0.0004,
            None,
            None,
            None,
        );

        // Manually set margin_used to 0 to test protection
        trade.margin_used = 0.0;

        // Update with price - should not panic
        trade.update_with_price(55000.0, None);

        // Margin ratio should default to 1.0 when margin_used is 0
        assert_eq!(trade.margin_ratio, 1.0);
    }

    #[test]
    fn test_division_by_zero_protection_negative_margin_scenario() {
        let mut trade = PaperTrade::new(
            "BTCUSDT".to_string(),
            TradeType::Long,
            50000.0,
            0.1,
            10,
            0.0004,
            None,
            None,
            None,
        );

        // Massive loss scenario (80% price drop)
        trade.update_with_price(10000.0, None);

        // Even with extreme losses, should not panic
        assert!(trade.margin_ratio < 1.0);
        // PnL % is now based on position_value: -4002/5000 * 100 = -80.04%
        assert!(trade.pnl_percentage < -70.0);
    }

    #[test]
    fn test_pnl_percentage_calculation_with_small_margin() {
        let mut trade = PaperTrade::new(
            "BTCUSDT".to_string(),
            TradeType::Long,
            50000.0,
            0.00001, // Very small quantity
            125,     // High leverage
            0.0004,
            None,
            None,
            None,
        );

        // Small initial margin: 0.5 / 125 = 0.004
        assert!(trade.initial_margin > 0.0);
        assert!(trade.initial_margin < 0.01);

        // Price increases by 10%
        trade.update_with_price(55000.0, None);

        // Should calculate PnL percentage without issues
        assert!(trade.pnl_percentage != 0.0);
        assert!(trade.pnl_percentage.is_finite());
    }

    #[test]
    fn test_margin_ratio_calculation_extreme_profit() {
        let mut trade = PaperTrade::new(
            "BTCUSDT".to_string(),
            TradeType::Long,
            50000.0,
            0.1,
            10,
            0.0004,
            None,
            None,
            None,
        );

        // Price increases 500%
        trade.update_with_price(250000.0, None);

        // Margin ratio should be very high (profitable)
        assert!(trade.margin_ratio > 10.0);
        assert!(trade.margin_ratio.is_finite());
        assert!(!trade.margin_ratio.is_nan());
    }

    #[test]
    fn test_margin_ratio_calculation_extreme_loss() {
        let mut trade = PaperTrade::new(
            "BTCUSDT".to_string(),
            TradeType::Long,
            50000.0,
            0.1,
            10,
            0.0004,
            None,
            None,
            None,
        );

        // Price drops 80%
        trade.update_with_price(10000.0, None);

        // Margin ratio should be very low (near liquidation)
        assert!(trade.margin_ratio < 1.0);
        assert!(trade.margin_ratio.is_finite());
        assert!(!trade.margin_ratio.is_nan());
    }

    #[test]
    fn test_pnl_calculation_precision_no_rounding_errors() {
        let mut trade = PaperTrade::new(
            "BTCUSDT".to_string(),
            TradeType::Long,
            50000.0,
            0.1,
            10,
            0.0004,
            None,
            None,
            None,
        );

        // Repeated updates should maintain consistency
        for _ in 0..100 {
            trade.update_with_price(51000.0, None);
        }

        // Values should remain consistent (not accumulate rounding errors)
        assert!((trade.unrealized_pnl - 98.0).abs() < 0.01);
        assert!(trade.pnl_percentage.is_finite());
    }

    #[test]
    fn test_funding_fees_with_zero_quantity() {
        let mut trade = PaperTrade::new(
            "BTCUSDT".to_string(),
            TradeType::Long,
            50000.0,
            0.0, // Zero quantity
            10,
            0.0004,
            None,
            None,
            None,
        );

        // Update with funding rate
        trade.update_with_price(50000.0, Some(0.0001));

        // Funding fees should be zero for zero quantity
        assert_eq!(trade.funding_fees, 0.0);
    }

    #[test]
    fn test_liquidation_calculation_no_overflow() {
        let trade = PaperTrade::new(
            "BTCUSDT".to_string(),
            TradeType::Long,
            50000.0,
            0.1,
            10, // 10x leverage for safer testing
            0.0004,
            None,
            None,
            None,
        );

        // Should calculate liquidation risk without overflow
        // At entry price, should not be at liquidation risk
        assert!(!trade.is_at_liquidation_risk(50000.0));

        // At bankruptcy price calculation: 50000 * (1 - 1/10) = 45000
        // Risk threshold: 45000 * 1.05 = 47250
        assert!(trade.is_at_liquidation_risk(47000.0));
        assert!(trade.is_at_liquidation_risk(45000.0));

        // Test with maximum leverage separately
        let high_leverage_trade = PaperTrade::new(
            "BTCUSDT".to_string(),
            TradeType::Long,
            50000.0,
            0.1,
            125,
            0.0004,
            None,
            None,
            None,
        );

        // At 125x, bankruptcy: 50000 * (1 - 1/125) = 49600
        // Risk threshold: 49600 * 1.05 = 52080
        // At entry price with 125x, close to liquidation
        assert!(high_leverage_trade.is_at_liquidation_risk(49800.0));
    }

    #[test]
    fn test_concurrent_updates_thread_safety() {
        use std::sync::{Arc, Mutex};
        use std::thread;

        let trade = Arc::new(Mutex::new(PaperTrade::new(
            "BTCUSDT".to_string(),
            TradeType::Long,
            50000.0,
            0.1,
            10,
            0.0004,
            None,
            None,
            None,
        )));

        let mut handles = vec![];

        // Spawn multiple threads updating the trade
        for i in 0..10 {
            let trade_clone = Arc::clone(&trade);
            let handle = thread::spawn(move || {
                let mut t = trade_clone.lock().unwrap();
                t.update_with_price(50000.0 + (i as f64 * 100.0), None);
            });
            handles.push(handle);
        }

        // Wait for all threads
        for handle in handles {
            handle.join().unwrap();
        }

        // Trade should be in valid state
        let final_trade = trade.lock().unwrap();
        assert!(final_trade.margin_ratio.is_finite());
        assert!(final_trade.pnl_percentage.is_finite());
    }
}
