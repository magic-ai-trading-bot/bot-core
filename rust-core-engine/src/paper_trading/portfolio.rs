use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use chrono::{DateTime, Utc};

use super::trade::{PaperTrade, TradeStatus, CloseReason};

/// Paper trading portfolio that tracks all positions and performance
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PaperPortfolio {
    /// Starting balance in USDT
    pub initial_balance: f64,
    
    /// Current cash balance
    pub cash_balance: f64,
    
    /// Current equity (cash + unrealized PnL)
    pub equity: f64,
    
    /// Total margin used across all positions
    pub margin_used: f64,
    
    /// Free margin available for new trades
    pub free_margin: f64,
    
    /// Margin level percentage
    pub margin_level: f64,
    
    /// All trades (open and closed)
    pub trades: HashMap<String, PaperTrade>,
    
    /// Open trade IDs
    pub open_trade_ids: Vec<String>,
    
    /// Closed trade IDs
    pub closed_trade_ids: Vec<String>,
    
    /// Current prices for all symbols
    pub current_prices: HashMap<String, f64>,
    
    /// Funding rates for symbols
    pub funding_rates: HashMap<String, f64>,
    
    /// Portfolio creation time
    pub created_at: DateTime<Utc>,
    
    /// Last update time
    pub last_updated: DateTime<Utc>,
    
    /// Portfolio metrics cache
    pub metrics: PortfolioMetrics,
    
    /// Daily performance history
    pub daily_performance: Vec<DailyPerformance>,
}

/// Comprehensive portfolio metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PortfolioMetrics {
    /// Total profit/loss (realized + unrealized)
    pub total_pnl: f64,
    
    /// Total profit/loss percentage
    pub total_pnl_percentage: f64,
    
    /// Realized profit/loss from closed trades
    pub realized_pnl: f64,
    
    /// Unrealized profit/loss from open trades
    pub unrealized_pnl: f64,
    
    /// Total number of trades executed
    pub total_trades: u64,
    
    /// Number of winning trades
    pub winning_trades: u64,
    
    /// Number of losing trades
    pub losing_trades: u64,
    
    /// Win rate percentage
    pub win_rate: f64,
    
    /// Average profit per winning trade
    pub average_win: f64,
    
    /// Average loss per losing trade
    pub average_loss: f64,
    
    /// Profit factor (gross profit / gross loss)
    pub profit_factor: f64,
    
    /// Maximum drawdown amount
    pub max_drawdown: f64,
    
    /// Maximum drawdown percentage
    pub max_drawdown_percentage: f64,
    
    /// Current drawdown amount
    pub current_drawdown: f64,
    
    /// Current drawdown percentage
    pub current_drawdown_percentage: f64,
    
    /// Largest winning trade
    pub largest_win: f64,
    
    /// Largest losing trade
    pub largest_loss: f64,
    
    /// Average trade return
    pub average_trade_return: f64,
    
    /// Standard deviation of returns
    pub return_std_deviation: f64,
    
    /// Sharpe ratio (annualized)
    pub sharpe_ratio: f64,
    
    /// Sortino ratio
    pub sortino_ratio: f64,
    
    /// Maximum consecutive wins
    pub max_consecutive_wins: u64,
    
    /// Maximum consecutive losses
    pub max_consecutive_losses: u64,
    
    /// Current consecutive wins/losses
    pub current_streak: i64, // positive for wins, negative for losses
    
    /// Average trade duration in minutes
    pub average_trade_duration_minutes: f64,
    
    /// Total trading fees paid
    pub total_fees_paid: f64,
    
    /// Total funding fees
    pub total_funding_fees: f64,
    
    /// Number of positions by symbol
    pub positions_by_symbol: HashMap<String, u32>,
    
    /// Average leverage used
    pub average_leverage: f64,
    
    /// Risk-adjusted return
    pub risk_adjusted_return: f64,
    
    /// Calmar ratio (annual return / max drawdown)
    pub calmar_ratio: f64,
    
    /// Recovery factor (total return / max drawdown)
    pub recovery_factor: f64,
    
    /// Last calculation time
    pub calculated_at: DateTime<Utc>,
}

/// Daily performance snapshot
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DailyPerformance {
    pub date: DateTime<Utc>,
    pub balance: f64,
    pub equity: f64,
    pub daily_pnl: f64,
    pub daily_pnl_percentage: f64,
    pub trades_executed: u32,
    pub winning_trades: u32,
    pub losing_trades: u32,
    pub total_volume: f64,
    pub max_drawdown: f64,
}

impl PaperPortfolio {
    /// Create a new paper trading portfolio
    pub fn new(initial_balance: f64) -> Self {
        let now = Utc::now();
        
        Self {
            initial_balance,
            cash_balance: initial_balance,
            equity: initial_balance,
            margin_used: 0.0,
            free_margin: initial_balance,
            margin_level: 0.0,
            trades: HashMap::new(),
            open_trade_ids: Vec::new(),
            closed_trade_ids: Vec::new(),
            current_prices: HashMap::new(),
            funding_rates: HashMap::new(),
            created_at: now,
            last_updated: now,
            metrics: PortfolioMetrics::default(),
            daily_performance: Vec::new(),
        }
    }
    
    /// Add a new trade to the portfolio
    pub fn add_trade(&mut self, trade: PaperTrade) -> Result<()> {
        // Check if we have enough free margin
        if trade.initial_margin > self.free_margin {
            return Err(anyhow::anyhow!(
                "Insufficient free margin. Required: {}, Available: {}", 
                trade.initial_margin, 
                self.free_margin
            ));
        }
        
        // Update margin calculations
        self.margin_used += trade.initial_margin;
        self.free_margin = self.equity - self.margin_used;
        
        // Add trade
        let trade_id = trade.id.clone();
        self.open_trade_ids.push(trade_id.clone());
        self.trades.insert(trade_id, trade);
        
        self.update_metrics();
        self.last_updated = Utc::now();
        
        Ok(())
    }
    
    /// Close a trade
    pub fn close_trade(&mut self, trade_id: &str, exit_price: f64, close_reason: CloseReason) -> Result<()> {
        let trade = self.trades.get_mut(trade_id)
            .ok_or_else(|| anyhow::anyhow!("Trade not found: {}", trade_id))?;
        
        if trade.status != TradeStatus::Open {
            return Err(anyhow::anyhow!("Trade is not open"));
        }
        
        // Calculate exit fees (same rate as entry)
        let exit_fees = (trade.quantity * exit_price) * (trade.trading_fees / (trade.quantity * trade.entry_price));
        
        // Close the trade
        trade.close(exit_price, close_reason, exit_fees)?;
        
        // Update portfolio
        if let Some(realized_pnl) = trade.realized_pnl {
            self.cash_balance += trade.initial_margin + realized_pnl;
        }
        
        self.margin_used -= trade.initial_margin;
        
        // Move from open to closed
        if let Some(pos) = self.open_trade_ids.iter().position(|id| id == trade_id) {
            self.open_trade_ids.remove(pos);
        }
        self.closed_trade_ids.push(trade_id.to_string());
        
        self.update_portfolio_values();
        self.update_metrics();
        self.last_updated = Utc::now();
        
        Ok(())
    }
    
    /// Update all trades with current market prices
    pub fn update_prices(&mut self, prices: HashMap<String, f64>, funding_rates: Option<HashMap<String, f64>>) {
        self.current_prices.extend(prices.clone());
        
        if let Some(rates) = funding_rates {
            self.funding_rates.extend(rates);
        }
        
        // Update all open trades
        for trade_id in &self.open_trade_ids.clone() {
            if let Some(trade) = self.trades.get_mut(trade_id) {
                if let Some(&current_price) = prices.get(&trade.symbol) {
                    let funding_rate = self.funding_rates.get(&trade.symbol).copied();
                    trade.update_with_price(current_price, funding_rate);
                }
            }
        }
        
        self.update_portfolio_values();
        self.update_metrics();
        self.last_updated = Utc::now();
    }
    
    /// Check for automatic trade closures (stop loss, take profit, liquidation)
    pub fn check_automatic_closures(&mut self) -> Vec<String> {
        let mut closed_trades = Vec::new();
        
        for trade_id in &self.open_trade_ids.clone() {
            if let Some(trade) = self.trades.get(trade_id) {
                if let Some(&current_price) = self.current_prices.get(&trade.symbol) {
                    let mut should_close = false;
                    let mut close_reason = CloseReason::Manual;
                    
                    // Check stop loss
                    if trade.should_stop_loss(current_price) {
                        should_close = true;
                        close_reason = CloseReason::StopLoss;
                    }
                    // Check take profit
                    else if trade.should_take_profit(current_price) {
                        should_close = true;
                        close_reason = CloseReason::TakeProfit;
                    }
                    // Check liquidation risk
                    else if trade.is_at_liquidation_risk(current_price) {
                        should_close = true;
                        close_reason = CloseReason::MarginCall;
                    }
                    
                    if should_close {
                        if let Ok(()) = self.close_trade(trade_id, current_price, close_reason) {
                            closed_trades.push(trade_id.clone());
                        }
                    }
                }
            }
        }
        
        closed_trades
    }
    
    /// Update portfolio equity and margin calculations
    fn update_portfolio_values(&mut self) {
        let mut unrealized_pnl = 0.0;
        
        // Calculate total unrealized PnL from open trades
        for trade_id in &self.open_trade_ids {
            if let Some(trade) = self.trades.get(trade_id) {
                unrealized_pnl += trade.unrealized_pnl;
            }
        }
        
        self.equity = self.cash_balance + unrealized_pnl;
        self.free_margin = self.equity - self.margin_used;
        
        // Calculate margin level
        self.margin_level = if self.margin_used > 0.0 {
            (self.equity / self.margin_used) * 100.0
        } else {
            0.0
        };
    }
    
    /// Calculate comprehensive portfolio metrics
    fn update_metrics(&mut self) {
        let mut metrics = PortfolioMetrics::default();
        
        let closed_trades: Vec<&PaperTrade> = self.closed_trade_ids
            .iter()
            .filter_map(|id| self.trades.get(id))
            .collect();
        
        // Basic trade statistics - Include ALL trades (open + closed)
        metrics.total_trades = self.trades.len() as u64;
        
        if !closed_trades.is_empty() {
            let mut realized_pnl = 0.0;
            let mut total_fees = 0.0;
            let mut total_funding_fees = 0.0;
            let mut wins = 0;
            let mut losses = 0;
            let mut win_amounts = Vec::new();
            let mut loss_amounts = Vec::new();
            let mut all_returns = Vec::new();
            let mut durations = Vec::new();
            let mut leverages = Vec::new();
            let mut positions_by_symbol: HashMap<String, u32> = HashMap::new();
            
            // Peak equity tracking for drawdown calculation
            let mut peak_equity = self.initial_balance;
            let mut max_drawdown = 0.0;
            let mut max_drawdown_pct = 0.0;
            let mut running_equity = self.initial_balance;
            
            // Consecutive wins/losses tracking
            let mut current_streak = 0i64;
            let mut max_consecutive_wins = 0u64;
            let mut max_consecutive_losses = 0u64;
            let mut consecutive_wins = 0u64;
            let mut consecutive_losses = 0u64;
            
            for trade in &closed_trades {
                if let Some(pnl) = trade.realized_pnl {
                    realized_pnl += pnl;
                    total_fees += trade.trading_fees;
                    total_funding_fees += trade.funding_fees;
                    
                    // Track positions by symbol
                    *positions_by_symbol.entry(trade.symbol.clone()).or_insert(0) += 1;
                    
                    // Calculate return percentage
                    let return_pct = (pnl / trade.initial_margin) * 100.0;
                    all_returns.push(return_pct);
                    
                    // Track duration
                    if let Some(duration) = trade.duration_ms {
                        durations.push(duration as f64 / 60000.0); // Convert to minutes
                    }
                    
                    // Track leverage
                    leverages.push(trade.leverage as f64);
                    
                    // Update running equity for drawdown calculation
                    running_equity += pnl;
                    if running_equity > peak_equity {
                        peak_equity = running_equity;
                    }
                    
                    let current_drawdown = peak_equity - running_equity;
                    let current_drawdown_pct = if peak_equity > 0.0 {
                        (current_drawdown / peak_equity) * 100.0
                    } else {
                        0.0
                    };
                    
                    if current_drawdown > max_drawdown {
                        max_drawdown = current_drawdown;
                        max_drawdown_pct = current_drawdown_pct;
                    }
                    
                    // Win/Loss tracking
                    if pnl > 0.0 {
                        wins += 1;
                        win_amounts.push(pnl);
                        consecutive_wins += 1;
                        consecutive_losses = 0;
                        current_streak = consecutive_wins as i64;
                        max_consecutive_wins = max_consecutive_wins.max(consecutive_wins);
                    } else if pnl < 0.0 {
                        losses += 1;
                        loss_amounts.push(pnl.abs());
                        consecutive_losses += 1;
                        consecutive_wins = 0;
                        current_streak = -(consecutive_losses as i64);
                        max_consecutive_losses = max_consecutive_losses.max(consecutive_losses);
                    }
                }
            }
            
            // Calculate metrics
            metrics.realized_pnl = realized_pnl;
            metrics.winning_trades = wins;
            metrics.losing_trades = losses;
            metrics.win_rate = if metrics.total_trades > 0 {
                (wins as f64 / metrics.total_trades as f64) * 100.0
            } else {
                0.0
            };
            
            metrics.average_win = if !win_amounts.is_empty() {
                win_amounts.iter().sum::<f64>() / win_amounts.len() as f64
            } else {
                0.0
            };
            
            metrics.average_loss = if !loss_amounts.is_empty() {
                loss_amounts.iter().sum::<f64>() / loss_amounts.len() as f64
            } else {
                0.0
            };
            
            metrics.profit_factor = if metrics.average_loss > 0.0 {
                metrics.average_win / metrics.average_loss
            } else {
                0.0
            };
            
            metrics.largest_win = win_amounts.iter().copied().fold(0.0, f64::max);
            metrics.largest_loss = loss_amounts.iter().copied().fold(0.0, f64::max);
            
            metrics.max_drawdown = max_drawdown;
            metrics.max_drawdown_percentage = max_drawdown_pct;
            
            metrics.current_streak = current_streak;
            metrics.max_consecutive_wins = max_consecutive_wins;
            metrics.max_consecutive_losses = max_consecutive_losses;
            
            metrics.average_trade_return = if !all_returns.is_empty() {
                all_returns.iter().sum::<f64>() / all_returns.len() as f64
            } else {
                0.0
            };
            
            // Calculate standard deviation of returns
            if all_returns.len() > 1 {
                let mean = metrics.average_trade_return;
                let variance = all_returns.iter()
                    .map(|r| (r - mean).powi(2))
                    .sum::<f64>() / (all_returns.len() - 1) as f64;
                metrics.return_std_deviation = variance.sqrt();
            }
            
            // Calculate Sharpe ratio (simplified, assuming risk-free rate = 0)
            metrics.sharpe_ratio = if metrics.return_std_deviation > 0.0 {
                metrics.average_trade_return / metrics.return_std_deviation
            } else {
                0.0
            };
            
            // Calculate Sortino ratio (downside deviation only)
            let downside_returns: Vec<f64> = all_returns.iter()
                .filter(|&&r| r < 0.0)
                .copied()
                .collect();
            
            if !downside_returns.is_empty() && downside_returns.len() > 1 {
                let downside_variance = downside_returns.iter()
                    .map(|r| r.powi(2))
                    .sum::<f64>() / downside_returns.len() as f64;
                let downside_deviation = downside_variance.sqrt();
                
                metrics.sortino_ratio = if downside_deviation > 0.0 {
                    metrics.average_trade_return / downside_deviation
                } else {
                    0.0
                };
            }
            
            metrics.average_trade_duration_minutes = if !durations.is_empty() {
                durations.iter().sum::<f64>() / durations.len() as f64
            } else {
                0.0
            };
            
            metrics.average_leverage = if !leverages.is_empty() {
                leverages.iter().sum::<f64>() / leverages.len() as f64
            } else {
                0.0
            };
            
            metrics.total_fees_paid = total_fees;
            metrics.total_funding_fees = total_funding_fees;
            metrics.positions_by_symbol = positions_by_symbol;
            
            // Risk-adjusted metrics
            let total_return_pct = (realized_pnl / self.initial_balance) * 100.0;
            metrics.risk_adjusted_return = if metrics.return_std_deviation > 0.0 {
                total_return_pct / metrics.return_std_deviation
            } else {
                0.0
            };
            
            metrics.calmar_ratio = if max_drawdown_pct > 0.0 {
                total_return_pct / max_drawdown_pct
            } else {
                0.0
            };
            
            metrics.recovery_factor = if max_drawdown > 0.0 {
                realized_pnl / max_drawdown
            } else {
                0.0
            };
        }
        
        // Add unrealized PnL from open trades
        for trade_id in &self.open_trade_ids {
            if let Some(trade) = self.trades.get(trade_id) {
                metrics.unrealized_pnl += trade.unrealized_pnl;
            }
        }
        
        metrics.total_pnl = metrics.realized_pnl + metrics.unrealized_pnl;
        metrics.total_pnl_percentage = (metrics.total_pnl / self.initial_balance) * 100.0;
        
        // Current drawdown
        let current_equity = self.initial_balance + metrics.total_pnl;
        let peak_equity = self.initial_balance + metrics.realized_pnl;
        metrics.current_drawdown = (peak_equity - current_equity).max(0.0);
        metrics.current_drawdown_percentage = if peak_equity > 0.0 {
            (metrics.current_drawdown / peak_equity) * 100.0
        } else {
            0.0
        };
        
        metrics.calculated_at = Utc::now();
        self.metrics = metrics;
    }
    
    /// Get open trades
    pub fn get_open_trades(&self) -> Vec<&PaperTrade> {
        self.open_trade_ids
            .iter()
            .filter_map(|id| self.trades.get(id))
            .collect()
    }
    
    /// Get closed trades
    pub fn get_closed_trades(&self) -> Vec<&PaperTrade> {
        self.closed_trade_ids
            .iter()
            .filter_map(|id| self.trades.get(id))
            .collect()
    }
    
    /// Get trade by ID
    pub fn get_trade(&self, trade_id: &str) -> Option<&PaperTrade> {
        self.trades.get(trade_id)
    }
    
    /// Check if we can open a new position
    pub fn can_open_position(&self, required_margin: f64) -> bool {
        required_margin <= self.free_margin && self.margin_level >= 100.0
    }
    
    /// Calculate position size based on risk percentage
    pub fn calculate_position_size(&self, risk_percentage: f64, entry_price: f64, stop_loss: f64, leverage: u8) -> f64 {
        let risk_amount = self.equity * (risk_percentage / 100.0);
        let price_diff = (entry_price - stop_loss).abs();
        let max_quantity = risk_amount / price_diff;
        
        // Limit by available margin
        let max_margin = self.free_margin * 0.95; // Keep 5% buffer
        let max_quantity_by_margin = (max_margin * leverage as f64) / entry_price;
        
        max_quantity.min(max_quantity_by_margin)
    }
    
    /// Add daily performance snapshot
    pub fn add_daily_performance(&mut self) {
        let today = Utc::now().date_naive();
        
        // Check if we already have today's performance
        if let Some(last_perf) = self.daily_performance.last() {
            if last_perf.date.date_naive() == today {
                return; // Already recorded today
            }
        }
        
        let daily_pnl = if let Some(yesterday_perf) = self.daily_performance.last() {
            self.equity - yesterday_perf.equity
        } else {
            self.equity - self.initial_balance
        };
        
        let daily_pnl_percentage = if let Some(yesterday_perf) = self.daily_performance.last() {
            if yesterday_perf.equity > 0.0 {
                (daily_pnl / yesterday_perf.equity) * 100.0
            } else {
                0.0
            }
        } else {
            (daily_pnl / self.initial_balance) * 100.0
        };
        
        // Count today's trades
        let today_start = today.and_hms_opt(0, 0, 0).unwrap().and_utc();
        let today_trades: Vec<&PaperTrade> = self.closed_trade_ids
            .iter()
            .filter_map(|id| self.trades.get(id))
            .filter(|trade| trade.close_time.map_or(false, |ct| ct >= today_start))
            .collect();
        
        let winning_today = today_trades.iter()
            .filter(|trade| trade.realized_pnl.map_or(false, |pnl| pnl > 0.0))
            .count() as u32;
        
        let losing_today = today_trades.iter()
            .filter(|trade| trade.realized_pnl.map_or(false, |pnl| pnl < 0.0))
            .count() as u32;
        
        let total_volume_today = today_trades.iter()
            .map(|trade| trade.quantity * trade.entry_price)
            .sum();
        
        let performance = DailyPerformance {
            date: Utc::now(),
            balance: self.cash_balance,
            equity: self.equity,
            daily_pnl,
            daily_pnl_percentage,
            trades_executed: today_trades.len() as u32,
            winning_trades: winning_today,
            losing_trades: losing_today,
            total_volume: total_volume_today,
            max_drawdown: self.metrics.max_drawdown,
        };
        
        self.daily_performance.push(performance);
        
        // Keep only last 365 days
        if self.daily_performance.len() > 365 {
            self.daily_performance.remove(0);
        }
    }
}

impl Default for PortfolioMetrics {
    fn default() -> Self {
        Self {
            total_pnl: 0.0,
            total_pnl_percentage: 0.0,
            realized_pnl: 0.0,
            unrealized_pnl: 0.0,
            total_trades: 0,
            winning_trades: 0,
            losing_trades: 0,
            win_rate: 0.0,
            average_win: 0.0,
            average_loss: 0.0,
            profit_factor: 0.0,
            max_drawdown: 0.0,
            max_drawdown_percentage: 0.0,
            current_drawdown: 0.0,
            current_drawdown_percentage: 0.0,
            largest_win: 0.0,
            largest_loss: 0.0,
            average_trade_return: 0.0,
            return_std_deviation: 0.0,
            sharpe_ratio: 0.0,
            sortino_ratio: 0.0,
            max_consecutive_wins: 0,
            max_consecutive_losses: 0,
            current_streak: 0,
            average_trade_duration_minutes: 0.0,
            total_fees_paid: 0.0,
            total_funding_fees: 0.0,
            positions_by_symbol: HashMap::new(),
            average_leverage: 0.0,
            risk_adjusted_return: 0.0,
            calmar_ratio: 0.0,
            recovery_factor: 0.0,
            calculated_at: Utc::now(),
        }
    }
} 