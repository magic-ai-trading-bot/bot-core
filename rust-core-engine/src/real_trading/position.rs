// @spec:FR-REAL-011 - Real Position Tracking
// @ref:specs/01-requirements/1.1-functional-requirements/FR-PORTFOLIO.md
// @test:TC-REAL-010, TC-REAL-011, TC-REAL-012

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// Position side (direction)
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum PositionSide {
    Long,
    Short,
}

impl PositionSide {
    pub fn as_str(&self) -> &'static str {
        match self {
            PositionSide::Long => "LONG",
            PositionSide::Short => "SHORT",
        }
    }

    pub fn from_order_side(side: &str) -> Self {
        match side.to_uppercase().as_str() {
            "BUY" => PositionSide::Long,
            "SELL" => PositionSide::Short,
            _ => PositionSide::Long,
        }
    }

    /// Get the opposite side for closing orders
    pub fn closing_order_side(&self) -> &'static str {
        match self {
            PositionSide::Long => "SELL",
            PositionSide::Short => "BUY",
        }
    }
}

/// Real position with complete tracking
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RealPosition {
    /// Unique position identifier
    pub id: String,
    /// Trading symbol (e.g., "BTCUSDT")
    pub symbol: String,
    /// Position direction
    pub side: PositionSide,
    /// Current position size (in base asset)
    pub quantity: f64,
    /// Volume-weighted average entry price
    pub entry_price: f64,
    /// Current market price
    pub current_price: f64,
    /// Unrealized PnL (mark-to-market)
    pub unrealized_pnl: f64,
    /// Realized PnL from partial closes
    pub realized_pnl: f64,
    /// Stop loss price (if set)
    pub stop_loss: Option<f64>,
    /// Take profit price (if set)
    pub take_profit: Option<f64>,
    /// Stop loss order ID (if placed)
    pub stop_loss_order_id: Option<String>,
    /// Take profit order ID (if placed)
    pub take_profit_order_id: Option<String>,
    /// Trailing stop activation price
    pub trailing_stop_activation: Option<f64>,
    /// Trailing stop distance (percentage)
    pub trailing_stop_percent: Option<f64>,
    /// Current trailing stop price (dynamic)
    pub trailing_stop_price: Option<f64>,
    /// Whether trailing stop has been activated (PnL-based activation)
    #[serde(default)]
    pub trailing_stop_active: bool,
    /// Best price since trailing stop activated (tracks high-water mark)
    #[serde(default)]
    pub best_price_since_trailing: Option<f64>,
    /// Order IDs that opened this position
    pub entry_order_ids: Vec<String>,
    /// Order IDs that closed (partially or fully)
    pub exit_order_ids: Vec<String>,
    /// Total commission paid
    pub total_commission: f64,
    /// Position open time
    pub created_at: DateTime<Utc>,
    /// Last update time
    pub updated_at: DateTime<Utc>,
    /// Strategy that generated this position
    pub strategy_name: Option<String>,
    /// Signal confidence (0.0 to 1.0)
    pub signal_confidence: Option<f64>,
    /// Leverage used for this position
    #[serde(default = "default_leverage")]
    pub leverage: u32,
}

fn default_leverage() -> u32 {
    1
}

impl RealPosition {
    /// Create a new position from an initial fill
    /// Note: Positions require multiple fields for complete tracking in trading systems
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        id: String,
        symbol: String,
        side: PositionSide,
        quantity: f64,
        entry_price: f64,
        entry_order_id: String,
        strategy_name: Option<String>,
        signal_confidence: Option<f64>,
    ) -> Self {
        let now = Utc::now();
        Self {
            id,
            symbol,
            side,
            quantity,
            entry_price,
            current_price: entry_price,
            unrealized_pnl: 0.0,
            realized_pnl: 0.0,
            stop_loss: None,
            take_profit: None,
            stop_loss_order_id: None,
            take_profit_order_id: None,
            trailing_stop_activation: None,
            trailing_stop_percent: None,
            trailing_stop_price: None,
            trailing_stop_active: false,
            best_price_since_trailing: None,
            entry_order_ids: vec![entry_order_id],
            exit_order_ids: Vec::new(),
            total_commission: 0.0,
            created_at: now,
            updated_at: now,
            strategy_name,
            signal_confidence,
            leverage: 1,
        }
    }

    /// Add a fill to the position (for scaling in)
    /// Returns true if this is an increase, false if decrease
    pub fn add_fill(
        &mut self,
        price: f64,
        quantity: f64,
        commission: f64,
        order_id: String,
    ) -> bool {
        // Calculate new average entry price (volume-weighted)
        let total_value = (self.entry_price * self.quantity) + (price * quantity);
        let new_quantity = self.quantity + quantity;

        if new_quantity > 0.0 {
            self.entry_price = total_value / new_quantity;
            self.quantity = new_quantity;
        }

        self.total_commission += commission;
        self.entry_order_ids.push(order_id);
        self.updated_at = Utc::now();

        true
    }

    /// Partially close the position
    /// Returns realized PnL for this partial close
    pub fn partial_close(
        &mut self,
        exit_price: f64,
        quantity: f64,
        commission: f64,
        order_id: String,
    ) -> f64 {
        let close_qty = quantity.min(self.quantity);

        // Calculate realized PnL for this portion
        let pnl = match self.side {
            PositionSide::Long => (exit_price - self.entry_price) * close_qty,
            PositionSide::Short => (self.entry_price - exit_price) * close_qty,
        };

        // Deduct commission from PnL
        let net_pnl = pnl - commission;

        self.quantity -= close_qty;
        self.realized_pnl += net_pnl;
        self.total_commission += commission;
        self.exit_order_ids.push(order_id);
        self.updated_at = Utc::now();

        // Recalculate unrealized PnL
        self.update_price(self.current_price);

        net_pnl
    }

    /// Update current price and recalculate unrealized PnL
    pub fn update_price(&mut self, price: f64) {
        self.current_price = price;
        self.unrealized_pnl = self.calculate_unrealized_pnl();
        self.updated_at = Utc::now();

        // Update trailing stop if active
        self.update_trailing_stop(price);
    }

    /// Calculate unrealized PnL at current price
    pub fn calculate_unrealized_pnl(&self) -> f64 {
        if self.quantity <= 0.0 {
            return 0.0;
        }

        match self.side {
            PositionSide::Long => (self.current_price - self.entry_price) * self.quantity,
            PositionSide::Short => (self.entry_price - self.current_price) * self.quantity,
        }
    }

    /// Calculate total PnL (realized + unrealized)
    pub fn total_pnl(&self) -> f64 {
        self.realized_pnl + self.unrealized_pnl
    }

    /// Calculate PnL percentage
    pub fn pnl_percentage(&self) -> f64 {
        let cost_basis = self.entry_price * self.quantity;
        if cost_basis > 0.0 {
            (self.total_pnl() / cost_basis) * 100.0
        } else {
            0.0
        }
    }

    /// Get position value in quote currency
    pub fn position_value(&self) -> f64 {
        self.quantity * self.current_price
    }

    /// Get cost basis (entry value)
    pub fn cost_basis(&self) -> f64 {
        self.quantity * self.entry_price
    }

    /// Check if position is still open
    pub fn is_open(&self) -> bool {
        self.quantity > 0.0
    }

    /// Check if position is closed
    pub fn is_closed(&self) -> bool {
        self.quantity <= 0.0
    }

    /// Set stop loss and take profit levels
    pub fn set_sl_tp(&mut self, stop_loss: Option<f64>, take_profit: Option<f64>) {
        self.stop_loss = stop_loss;
        self.take_profit = take_profit;
        self.updated_at = Utc::now();
    }

    /// Enable trailing stop
    pub fn enable_trailing_stop(&mut self, activation_price: f64, percent: f64) {
        self.trailing_stop_activation = Some(activation_price);
        self.trailing_stop_percent = Some(percent);
        self.updated_at = Utc::now();

        // Initialize if already past activation
        self.update_trailing_stop(self.current_price);
    }

    /// Update trailing stop price based on current price.
    /// Uses PnL-based activation: trailing stop activates when unrealized PnL%
    /// reaches the activation threshold, then tracks best price from that point.
    pub(crate) fn update_trailing_stop(&mut self, price: f64) {
        if let (Some(activation_pct), Some(trail_pct)) =
            (self.trailing_stop_activation, self.trailing_stop_percent)
        {
            // Check if trailing stop should activate based on PnL%
            if !self.trailing_stop_active {
                let pnl_pct = match self.side {
                    PositionSide::Long => (price - self.entry_price) / self.entry_price * 100.0,
                    PositionSide::Short => (self.entry_price - price) / self.entry_price * 100.0,
                };

                if pnl_pct >= activation_pct {
                    self.trailing_stop_active = true;
                    self.best_price_since_trailing = Some(price);
                }
            }

            // Once active, track best price and update trailing stop
            if self.trailing_stop_active {
                match self.side {
                    PositionSide::Long => {
                        // Update best price (highest since activation)
                        let best = self
                            .best_price_since_trailing
                            .map(|b| b.max(price))
                            .unwrap_or(price);
                        self.best_price_since_trailing = Some(best);

                        let new_stop = best * (1.0 - trail_pct / 100.0);
                        // Only move stop up, never down
                        if self.trailing_stop_price.is_none_or(|s| new_stop > s) {
                            self.trailing_stop_price = Some(new_stop);
                        }
                    },
                    PositionSide::Short => {
                        // Update best price (lowest since activation)
                        let best = self
                            .best_price_since_trailing
                            .map(|b| b.min(price))
                            .unwrap_or(price);
                        self.best_price_since_trailing = Some(best);

                        let new_stop = best * (1.0 + trail_pct / 100.0);
                        // Only move stop down, never up
                        if self.trailing_stop_price.is_none_or(|s| new_stop < s) {
                            self.trailing_stop_price = Some(new_stop);
                        }
                    },
                }
            }
        }
    }

    /// Check if stop loss should trigger
    pub fn should_trigger_stop_loss(&self) -> bool {
        // Check trailing stop first
        if let Some(trailing) = self.trailing_stop_price {
            match self.side {
                PositionSide::Long => {
                    if self.current_price <= trailing {
                        return true;
                    }
                },
                PositionSide::Short => {
                    if self.current_price >= trailing {
                        return true;
                    }
                },
            }
        }

        // Then check fixed stop loss
        if let Some(sl) = self.stop_loss {
            match self.side {
                PositionSide::Long => self.current_price <= sl,
                PositionSide::Short => self.current_price >= sl,
            }
        } else {
            false
        }
    }

    /// Check if take profit should trigger
    pub fn should_trigger_take_profit(&self) -> bool {
        if let Some(tp) = self.take_profit {
            match self.side {
                PositionSide::Long => self.current_price >= tp,
                PositionSide::Short => self.current_price <= tp,
            }
        } else {
            false
        }
    }

    /// Get effective stop loss price (trailing or fixed)
    pub fn effective_stop_loss(&self) -> Option<f64> {
        self.trailing_stop_price.or(self.stop_loss)
    }

    /// Set leverage for this position
    pub fn set_leverage(&mut self, leverage: u32) {
        self.leverage = leverage;
    }

    /// Check if position is at liquidation risk (matching paper trading logic)
    /// Uses simplified bankruptcy price with 5% safety buffer
    pub fn is_at_liquidation_risk(&self) -> bool {
        if self.leverage <= 1 {
            return false; // Spot positions can't be liquidated
        }
        let lev = self.leverage as f64;
        let bankruptcy_price = match self.side {
            PositionSide::Long => self.entry_price * (1.0 - 1.0 / lev),
            PositionSide::Short => self.entry_price * (1.0 + 1.0 / lev),
        };
        match self.side {
            PositionSide::Long => self.current_price <= bankruptcy_price * 1.05, // 5% buffer
            PositionSide::Short => self.current_price >= bankruptcy_price * 0.95, // 5% buffer
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_position(side: PositionSide) -> RealPosition {
        RealPosition::new(
            "pos-123".to_string(),
            "BTCUSDT".to_string(),
            side,
            0.1,
            50000.0,
            "order-123".to_string(),
            Some("RSI".to_string()),
            Some(0.85),
        )
    }

    #[test]
    fn test_position_new() {
        let pos = create_test_position(PositionSide::Long);

        assert_eq!(pos.symbol, "BTCUSDT");
        assert_eq!(pos.side, PositionSide::Long);
        assert_eq!(pos.quantity, 0.1);
        assert_eq!(pos.entry_price, 50000.0);
        assert!(pos.is_open());
        assert!(!pos.is_closed());
    }

    #[test]
    fn test_long_position_unrealized_pnl() {
        let mut pos = create_test_position(PositionSide::Long);

        // Price goes up - profit
        pos.update_price(51000.0);
        assert!((pos.unrealized_pnl - 100.0).abs() < 0.01); // (51000 - 50000) * 0.1 = 100

        // Price goes down - loss
        pos.update_price(49000.0);
        assert!((pos.unrealized_pnl - (-100.0)).abs() < 0.01); // (49000 - 50000) * 0.1 = -100
    }

    #[test]
    fn test_short_position_unrealized_pnl() {
        let mut pos = create_test_position(PositionSide::Short);

        // Price goes down - profit for short
        pos.update_price(49000.0);
        assert!((pos.unrealized_pnl - 100.0).abs() < 0.01); // (50000 - 49000) * 0.1 = 100

        // Price goes up - loss for short
        pos.update_price(51000.0);
        assert!((pos.unrealized_pnl - (-100.0)).abs() < 0.01); // (50000 - 51000) * 0.1 = -100
    }

    #[test]
    fn test_position_add_fill_average_entry() {
        let mut pos = create_test_position(PositionSide::Long);
        // Initial: 0.1 @ 50000

        // Add more at higher price
        pos.add_fill(52000.0, 0.1, 0.5, "order-456".to_string());

        // New average: (50000 * 0.1 + 52000 * 0.1) / 0.2 = 51000
        assert!((pos.entry_price - 51000.0).abs() < 0.01);
        assert!((pos.quantity - 0.2).abs() < 0.0001);
    }

    #[test]
    fn test_position_partial_close() {
        let mut pos = create_test_position(PositionSide::Long);
        // 0.1 BTC @ 50000

        // Close half at profit
        let pnl = pos.partial_close(52000.0, 0.05, 0.25, "exit-order-1".to_string());

        // PnL: (52000 - 50000) * 0.05 - 0.25 = 100 - 0.25 = 99.75
        assert!((pnl - 99.75).abs() < 0.01);
        assert!((pos.quantity - 0.05).abs() < 0.0001);
        assert!((pos.realized_pnl - 99.75).abs() < 0.01);
    }

    #[test]
    fn test_position_stop_loss_trigger_long() {
        let mut pos = create_test_position(PositionSide::Long);
        pos.set_sl_tp(Some(49000.0), Some(52000.0));

        // Above SL - no trigger
        pos.update_price(49500.0);
        assert!(!pos.should_trigger_stop_loss());

        // At SL - trigger
        pos.update_price(49000.0);
        assert!(pos.should_trigger_stop_loss());

        // Below SL - trigger
        pos.update_price(48000.0);
        assert!(pos.should_trigger_stop_loss());
    }

    #[test]
    fn test_position_take_profit_trigger_long() {
        let mut pos = create_test_position(PositionSide::Long);
        pos.set_sl_tp(Some(49000.0), Some(52000.0));

        // Below TP - no trigger
        pos.update_price(51000.0);
        assert!(!pos.should_trigger_take_profit());

        // At TP - trigger
        pos.update_price(52000.0);
        assert!(pos.should_trigger_take_profit());
    }

    #[test]
    fn test_position_trailing_stop_long() {
        let mut pos = create_test_position(PositionSide::Long);
        // Entry at 50000, activate trailing at 4% PnL (price ~52000), 2% trail
        pos.enable_trailing_stop(4.0, 2.0);

        // Price below activation PnL threshold - no trailing stop yet
        pos.update_price(51000.0); // 2% PnL, below 4% threshold
        assert!(pos.trailing_stop_price.is_none());
        assert!(!pos.trailing_stop_active);

        // Price hits activation threshold (4% PnL = 52000)
        pos.update_price(52000.0);
        assert!(pos.trailing_stop_active);
        // Trailing stop = 52000 * 0.98 = 50960
        assert!((pos.trailing_stop_price.unwrap() - 50960.0).abs() < 1.0);

        // Price goes higher - trailing stop moves up
        pos.update_price(54000.0);
        // Trailing stop = 54000 * 0.98 = 52920
        assert!((pos.trailing_stop_price.unwrap() - 52920.0).abs() < 1.0);

        // Price drops but trailing stop doesn't go down
        pos.update_price(53000.0);
        assert!((pos.trailing_stop_price.unwrap() - 52920.0).abs() < 1.0);

        // Check trigger
        pos.update_price(52920.0);
        assert!(pos.should_trigger_stop_loss());
    }

    #[test]
    fn test_position_pnl_percentage() {
        let mut pos = create_test_position(PositionSide::Long);
        // Cost basis: 50000 * 0.1 = 5000

        pos.update_price(55000.0);
        // PnL: 500, percentage: 500/5000 * 100 = 10%
        assert!((pos.pnl_percentage() - 10.0).abs() < 0.1);
    }

    #[test]
    fn test_position_side_conversion() {
        assert_eq!(PositionSide::from_order_side("BUY"), PositionSide::Long);
        assert_eq!(PositionSide::from_order_side("SELL"), PositionSide::Short);
        assert_eq!(PositionSide::from_order_side("buy"), PositionSide::Long);

        assert_eq!(PositionSide::Long.closing_order_side(), "SELL");
        assert_eq!(PositionSide::Short.closing_order_side(), "BUY");
    }

    #[test]
    fn test_position_side_as_str() {
        assert_eq!(PositionSide::Long.as_str(), "LONG");
        assert_eq!(PositionSide::Short.as_str(), "SHORT");
    }

    #[test]
    fn test_position_side_from_unknown() {
        // Unknown sides default to Long
        assert_eq!(PositionSide::from_order_side("UNKNOWN"), PositionSide::Long);
        assert_eq!(PositionSide::from_order_side(""), PositionSide::Long);
    }

    #[test]
    fn test_position_value_and_cost_basis() {
        let mut pos = create_test_position(PositionSide::Long);
        // Entry: 50000, Qty: 0.1

        assert!((pos.cost_basis() - 5000.0).abs() < 0.01); // 50000 * 0.1
        assert!((pos.position_value() - 5000.0).abs() < 0.01); // Same at entry

        pos.update_price(55000.0);
        assert!((pos.cost_basis() - 5000.0).abs() < 0.01); // Cost doesn't change
        assert!((pos.position_value() - 5500.0).abs() < 0.01); // Value updated
    }

    #[test]
    fn test_position_zero_quantity() {
        let mut pos = create_test_position(PositionSide::Long);

        // Close entire position
        pos.partial_close(51000.0, 0.1, 0.5, "close-all".to_string());

        assert!(pos.is_closed());
        assert!(!pos.is_open());
        assert_eq!(pos.calculate_unrealized_pnl(), 0.0);
    }

    #[test]
    fn test_position_partial_close_quantity_capped() {
        let mut pos = create_test_position(PositionSide::Long);
        // 0.1 BTC @ 50000

        // Try to close more than we have
        pos.partial_close(52000.0, 0.5, 0.25, "close-order".to_string());

        // Should only close 0.1 (max available)
        assert_eq!(pos.quantity, 0.0);
        assert!(pos.is_closed());
    }

    #[test]
    fn test_position_add_fill_zero_quantity() {
        let mut pos = create_test_position(PositionSide::Long);
        let original_qty = pos.quantity;
        let original_entry = pos.entry_price;

        // Add zero quantity
        pos.add_fill(52000.0, 0.0, 0.0, "order-zero".to_string());

        // Entry price and quantity shouldn't change
        assert_eq!(pos.quantity, original_qty);
        assert_eq!(pos.entry_price, original_entry);
    }

    #[test]
    fn test_position_trailing_stop_short() {
        let mut pos = create_test_position(PositionSide::Short);
        // Entry at 50000, activate trailing at 4% PnL (price ~48000 for short), 2% trail
        pos.enable_trailing_stop(4.0, 2.0);

        // Price above activation - PnL only 2%, below 4% threshold
        pos.update_price(49000.0);
        assert!(pos.trailing_stop_price.is_none());
        assert!(!pos.trailing_stop_active);

        // Price hits activation (4% PnL for short = 48000)
        pos.update_price(48000.0);
        assert!(pos.trailing_stop_active);
        // Trailing stop = 48000 * 1.02 = 48960
        assert!((pos.trailing_stop_price.unwrap() - 48960.0).abs() < 1.0);

        // Price goes lower - trailing stop moves down
        pos.update_price(46000.0);
        // Trailing stop = 46000 * 1.02 = 46920
        assert!((pos.trailing_stop_price.unwrap() - 46920.0).abs() < 1.0);

        // Price rises but trailing stop doesn't go up
        pos.update_price(47000.0);
        assert!((pos.trailing_stop_price.unwrap() - 46920.0).abs() < 1.0);

        // Check trigger when price rises to trailing stop
        pos.update_price(46920.0);
        assert!(pos.should_trigger_stop_loss());
    }

    #[test]
    fn test_position_stop_loss_trigger_short() {
        let mut pos = create_test_position(PositionSide::Short);
        pos.set_sl_tp(Some(51000.0), Some(48000.0));

        // Below SL - no trigger
        pos.update_price(50000.0);
        assert!(!pos.should_trigger_stop_loss());

        // At SL - trigger
        pos.update_price(51000.0);
        assert!(pos.should_trigger_stop_loss());

        // Above SL - trigger
        pos.update_price(52000.0);
        assert!(pos.should_trigger_stop_loss());
    }

    #[test]
    fn test_position_take_profit_trigger_short() {
        let mut pos = create_test_position(PositionSide::Short);
        pos.set_sl_tp(Some(51000.0), Some(48000.0));

        // Above TP - no trigger
        pos.update_price(49000.0);
        assert!(!pos.should_trigger_take_profit());

        // At TP - trigger
        pos.update_price(48000.0);
        assert!(pos.should_trigger_take_profit());

        // Below TP - trigger
        pos.update_price(47000.0);
        assert!(pos.should_trigger_take_profit());
    }

    #[test]
    fn test_position_effective_stop_loss() {
        let mut pos = create_test_position(PositionSide::Long);

        // No stops set
        assert!(pos.effective_stop_loss().is_none());

        // Set fixed SL
        pos.set_sl_tp(Some(49000.0), None);
        assert_eq!(pos.effective_stop_loss(), Some(49000.0));

        // Enable trailing stop (4% PnL activation, 2% trail)
        pos.enable_trailing_stop(4.0, 2.0);
        pos.update_price(52000.0); // 4% PnL -> activates trailing

        // Trailing stop should take precedence
        assert!(pos.trailing_stop_price.is_some());
        assert_eq!(pos.effective_stop_loss(), pos.trailing_stop_price);
    }

    #[test]
    fn test_position_short_unrealized_pnl() {
        let mut pos = create_test_position(PositionSide::Short);

        // Partial close at profit
        let pnl = pos.partial_close(48000.0, 0.05, 0.25, "exit-1".to_string());

        // PnL: (50000 - 48000) * 0.05 - 0.25 = 100 - 0.25 = 99.75
        assert!((pnl - 99.75).abs() < 0.01);
        assert!((pos.quantity - 0.05).abs() < 0.0001);
    }

    #[test]
    fn test_position_pnl_percentage_negative() {
        let mut pos = create_test_position(PositionSide::Long);

        pos.update_price(45000.0); // 10% loss
                                   // PnL: -500, percentage: -500/5000 * 100 = -10%
        assert!((pos.pnl_percentage() - (-10.0)).abs() < 0.1);
    }

    #[test]
    fn test_position_pnl_percentage_zero_cost() {
        let mut pos = create_test_position(PositionSide::Long);
        pos.quantity = 0.0; // Force zero cost basis

        assert_eq!(pos.pnl_percentage(), 0.0);
    }

    #[test]
    fn test_position_total_pnl_with_realized() {
        let mut pos = create_test_position(PositionSide::Long);

        // Partial close with profit
        pos.partial_close(52000.0, 0.05, 0.25, "exit-1".to_string());

        // Update price for unrealized PnL
        pos.update_price(51000.0);

        // Total PnL = realized + unrealized
        let total = pos.total_pnl();
        assert!((total - (pos.realized_pnl + pos.unrealized_pnl)).abs() < 0.01);
    }

    #[test]
    fn test_position_order_ids_tracking() {
        let pos = create_test_position(PositionSide::Long);

        assert_eq!(pos.entry_order_ids.len(), 1);
        assert_eq!(pos.entry_order_ids[0], "order-123");
        assert_eq!(pos.exit_order_ids.len(), 0);
    }

    #[test]
    fn test_position_commission_tracking() {
        let mut pos = create_test_position(PositionSide::Long);

        pos.add_fill(51000.0, 0.05, 2.5, "order-2".to_string());
        pos.partial_close(52000.0, 0.05, 1.25, "exit-1".to_string());

        // Total commission = 2.5 + 1.25 = 3.75
        assert!((pos.total_commission - 3.75).abs() < 0.01);
    }

    // ========== COV22 TESTS - Cover default_leverage, trailing stop short, liquidation risk ==========

    #[test]
    fn test_cov22_default_leverage_value() {
        // Covers lines 98-100: default_leverage() returns 1 for spot positions
        // Test by verifying serialized position with leverage field absent uses default
        let pos = create_test_position(PositionSide::Long);
        // New positions start without leverage set explicitly - leverage field in struct
        // The default_leverage() is called when serde encounters missing field
        // Test it by creating position via new() which sets leverage field to default (1)
        let json = serde_json::to_string(&pos).unwrap();
        let deserialized: RealPosition = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.leverage, pos.leverage);

        // Explicitly verify leverage defaults to 1 via serialization without the field
        // by serializing with leverage field present and re-deserializing
        let mut map: serde_json::Map<String, serde_json::Value> =
            serde_json::from_str(&json).unwrap();
        map.remove("leverage"); // Remove leverage field to trigger default
        let without_leverage = serde_json::to_string(&map).unwrap();
        let reloaded: RealPosition = serde_json::from_str(&without_leverage).unwrap();
        assert_eq!(reloaded.leverage, 1); // default_leverage() returns 1
    }

    #[test]
    fn test_cov22_trailing_stop_short_position_triggers() {
        // Covers line 341/346: trailing stop for Short position (current >= trailing)
        let mut pos = create_test_position(PositionSide::Short);
        pos.stop_loss = None;
        pos.trailing_stop_price = Some(51000.0);
        pos.current_price = 51500.0; // above trailing stop → hit for short
        assert!(
            pos.should_trigger_stop_loss(),
            "Short trailing stop should trigger when price >= trailing"
        );
    }

    #[test]
    fn test_cov22_trailing_stop_short_not_triggered() {
        // Trailing stop for Short not triggered when price below trailing
        let mut pos = create_test_position(PositionSide::Short);
        pos.stop_loss = None;
        pos.trailing_stop_price = Some(51000.0);
        pos.current_price = 49000.0; // below trailing → not hit
        assert!(!pos.should_trigger_stop_loss());
    }

    #[test]
    fn test_cov22_trailing_stop_long_triggered() {
        // Covers line 339: Long trailing stop when current <= trailing
        let mut pos = create_test_position(PositionSide::Long);
        pos.stop_loss = None;
        pos.trailing_stop_price = Some(49000.0);
        pos.current_price = 48500.0; // below trailing → hit
        assert!(
            pos.should_trigger_stop_loss(),
            "Long trailing stop should trigger when price <= trailing"
        );
    }

    #[test]
    fn test_cov22_is_at_liquidation_risk_long_high_leverage() {
        // Covers lines 386-399: liquidation risk for Long position
        let mut pos = create_test_position(PositionSide::Long);
        pos.leverage = 10;
        pos.entry_price = 50000.0;
        // bankruptcy_price = 50000 * (1 - 1/10) = 50000 * 0.9 = 45000
        // liquidation threshold = 45000 * 1.05 = 47250
        pos.current_price = 47000.0; // below threshold → at risk
        assert!(
            pos.is_at_liquidation_risk(),
            "Long high leverage at risk when near bankruptcy"
        );
    }

    #[test]
    fn test_cov22_is_at_liquidation_risk_long_safe() {
        // Long position safe (price well above threshold)
        let mut pos = create_test_position(PositionSide::Long);
        pos.leverage = 10;
        pos.entry_price = 50000.0;
        pos.current_price = 50000.0; // well above 47250 → safe
        assert!(!pos.is_at_liquidation_risk());
    }

    #[test]
    fn test_cov22_is_at_liquidation_risk_short_high_leverage() {
        // Covers Short branch of is_at_liquidation_risk (line 397)
        let mut pos = create_test_position(PositionSide::Short);
        pos.leverage = 10;
        pos.entry_price = 50000.0;
        // bankruptcy_price = 50000 * (1 + 1/10) = 55000
        // liquidation threshold = 55000 * 0.95 = 52250
        pos.current_price = 53000.0; // above threshold → at risk
        assert!(
            pos.is_at_liquidation_risk(),
            "Short high leverage at risk when near bankruptcy"
        );
    }

    #[test]
    fn test_cov22_is_at_liquidation_risk_short_safe() {
        let mut pos = create_test_position(PositionSide::Short);
        pos.leverage = 10;
        pos.entry_price = 50000.0;
        pos.current_price = 50000.0; // below 52250 → safe
        assert!(!pos.is_at_liquidation_risk());
    }

    #[test]
    fn test_cov22_is_at_liquidation_risk_spot() {
        // Spot position (leverage=1) → covers lines 387-388 (early return false)
        let mut pos = create_test_position(PositionSide::Long);
        pos.leverage = 1;
        assert!(
            !pos.is_at_liquidation_risk(),
            "Spot positions cannot be liquidated"
        );
    }

    #[test]
    fn test_cov22_set_leverage() {
        // Covers set_leverage() method (lines 380-382)
        let mut pos = create_test_position(PositionSide::Long);
        pos.set_leverage(20);
        assert_eq!(pos.leverage, 20);
    }
}
