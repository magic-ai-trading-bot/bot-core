use anyhow::Result;
use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};
use uuid::Uuid;

/// Trade types supported
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
pub enum TradeType {
    Long,
    Short,
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
    
    /// Custom metadata
    pub metadata: std::collections::HashMap<String, serde_json::Value>,
}

impl PaperTrade {
    /// Create a new paper trade
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
        
        // Calculate PnL percentage based on margin
        self.pnl_percentage = (self.unrealized_pnl / self.initial_margin) * 100.0;
        
        // Update margin ratio
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
    
    /// Check if trade is at risk of liquidation
    pub fn is_at_liquidation_risk(&self, current_price: f64) -> bool {
        // Binance uses a more complex liquidation calculation, but this is a simplified version
        let bankruptcy_price = match self.trade_type {
            TradeType::Long => {
                self.entry_price * (1.0 - 1.0 / self.leverage as f64)
            },
            TradeType::Short => {
                self.entry_price * (1.0 + 1.0 / self.leverage as f64)
            },
        };
        
        match self.trade_type {
            TradeType::Long => current_price <= bankruptcy_price * 1.05, // 5% margin
            TradeType::Short => current_price >= bankruptcy_price * 0.95, // 5% margin
        }
    }
    
    /// Close the trade
    pub fn close(&mut self, exit_price: f64, close_reason: CloseReason, additional_fees: f64) -> Result<()> {
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
        
        self.realized_pnl = Some(price_diff * self.quantity - self.trading_fees - self.funding_fees - additional_fees);
        
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
        self.metadata.insert("cancel_reason".to_string(), serde_json::Value::String(reason));
        
        Ok(())
    }
    
    /// Set stop loss
    pub fn set_stop_loss(&mut self, stop_loss: f64) -> Result<()> {
        // Validate stop loss makes sense
        match self.trade_type {
            TradeType::Long => {
                if stop_loss >= self.entry_price {
                    return Err(anyhow::anyhow!("Stop loss must be below entry price for long trades"));
                }
            },
            TradeType::Short => {
                if stop_loss <= self.entry_price {
                    return Err(anyhow::anyhow!("Stop loss must be above entry price for short trades"));
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
                    return Err(anyhow::anyhow!("Take profit must be above entry price for long trades"));
                }
            },
            TradeType::Short => {
                if take_profit >= self.entry_price {
                    return Err(anyhow::anyhow!("Take profit must be below entry price for short trades"));
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
}

impl TradeType {
    pub fn to_string(&self) -> &'static str {
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
    pub fn to_string(&self) -> &'static str {
        match self {
            TradeStatus::Open => "Open",
            TradeStatus::Closed => "Closed",
            TradeStatus::Cancelled => "Cancelled",
        }
    }
} 