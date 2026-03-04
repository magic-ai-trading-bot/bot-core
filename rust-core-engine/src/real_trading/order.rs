// @spec:FR-REAL-010 - Real Order Tracking
// @ref:specs/01-requirements/1.1-functional-requirements/FR-TRADING.md
// @test:TC-REAL-001, TC-REAL-002, TC-REAL-003

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::binance::types::ExecutionReport;

/// Order state machine for tracking order lifecycle
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum OrderState {
    /// Order submitted, awaiting exchange confirmation
    Pending,
    /// Confirmed by exchange, waiting for fills
    New,
    /// Some quantity has been filled
    PartiallyFilled,
    /// Order completely filled
    Filled,
    /// Cancelled by user or system
    Cancelled,
    /// Rejected by exchange
    Rejected,
    /// Order expired (GTX/GTC timeout)
    Expired,
}

impl OrderState {
    /// Check if order is in an active (non-terminal) state
    pub fn is_active(&self) -> bool {
        matches!(
            self,
            OrderState::Pending | OrderState::New | OrderState::PartiallyFilled
        )
    }

    /// Check if order is in a terminal state
    pub fn is_terminal(&self) -> bool {
        matches!(
            self,
            OrderState::Filled | OrderState::Cancelled | OrderState::Rejected | OrderState::Expired
        )
    }

    /// Convert from Binance order status string
    pub fn from_binance_status(status: &str) -> Self {
        match status {
            "NEW" => OrderState::New,
            "PARTIALLY_FILLED" => OrderState::PartiallyFilled,
            "FILLED" => OrderState::Filled,
            "CANCELED" => OrderState::Cancelled,
            "PENDING_CANCEL" => OrderState::Cancelled,
            "REJECTED" => OrderState::Rejected,
            "EXPIRED" => OrderState::Expired,
            "EXPIRED_IN_MATCH" => OrderState::Expired,
            _ => OrderState::Pending,
        }
    }
}

/// Individual fill record from ExecutionReport
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrderFill {
    /// Trade ID from exchange
    pub trade_id: i64,
    /// Fill price
    pub price: f64,
    /// Fill quantity
    pub quantity: f64,
    /// Commission paid
    pub commission: f64,
    /// Commission asset (e.g., "BNB", "USDT")
    pub commission_asset: String,
    /// Fill timestamp
    pub timestamp: DateTime<Utc>,
}

/// Real order tracking with full lifecycle management
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RealOrder {
    /// Client-generated order ID
    pub client_order_id: String,
    /// Exchange-assigned order ID
    pub exchange_order_id: i64,
    /// Trading symbol (e.g., "BTCUSDT")
    pub symbol: String,
    /// Order side: "BUY" or "SELL"
    pub side: String,
    /// Order type: "MARKET", "LIMIT", "STOP_LOSS_LIMIT", etc.
    pub order_type: String,
    /// Original requested quantity
    pub original_quantity: f64,
    /// Quantity filled so far
    pub executed_quantity: f64,
    /// Quantity remaining to fill
    pub remaining_quantity: f64,
    /// Limit price (None for market orders)
    pub price: Option<f64>,
    /// Stop trigger price (for stop orders)
    pub stop_price: Option<f64>,
    /// Volume-weighted average fill price
    pub average_fill_price: f64,
    /// Current order state
    pub state: OrderState,
    /// Order creation time
    pub created_at: DateTime<Utc>,
    /// Last update time
    pub updated_at: DateTime<Utc>,
    /// All fills for this order
    pub fills: Vec<OrderFill>,
    /// Position ID this order belongs to (if any)
    pub position_id: Option<String>,
    /// Whether this is an entry or exit order
    pub is_entry: bool,
    /// Error message if rejected
    pub reject_reason: Option<String>,
}

impl RealOrder {
    /// Create a new order in Pending state
    /// Note: Orders require multiple fields for complete specification in trading systems
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        client_order_id: String,
        symbol: String,
        side: String,
        order_type: String,
        quantity: f64,
        price: Option<f64>,
        stop_price: Option<f64>,
        position_id: Option<String>,
        is_entry: bool,
    ) -> Self {
        let now = Utc::now();
        Self {
            client_order_id,
            exchange_order_id: 0,
            symbol,
            side,
            order_type,
            original_quantity: quantity,
            executed_quantity: 0.0,
            remaining_quantity: quantity,
            price,
            stop_price,
            average_fill_price: 0.0,
            state: OrderState::Pending,
            created_at: now,
            updated_at: now,
            fills: Vec::new(),
            position_id,
            is_entry,
            reject_reason: None,
        }
    }

    /// Update order state from Binance ExecutionReport
    pub fn update_from_execution_report(&mut self, report: &ExecutionReport) {
        self.exchange_order_id = report.order_id;
        self.state = OrderState::from_binance_status(&report.order_status);
        self.updated_at = Utc::now();

        // Parse executed quantity
        if let Ok(exec_qty) = report.cumulative_filled_quantity.parse::<f64>() {
            self.executed_quantity = exec_qty;
            self.remaining_quantity = self.original_quantity - exec_qty;
        }

        // Parse average price (cumulative quote / cumulative qty)
        if let (Ok(quote_qty), Ok(exec_qty)) = (
            report.cumulative_quote_qty.parse::<f64>(),
            report.cumulative_filled_quantity.parse::<f64>(),
        ) {
            if exec_qty > 0.0 {
                self.average_fill_price = quote_qty / exec_qty;
            }
        }

        // If this is a TRADE execution, record the fill
        if report.execution_type == "TRADE" {
            if let (Ok(fill_price), Ok(fill_qty)) = (
                report.last_executed_price.parse::<f64>(),
                report.last_executed_quantity.parse::<f64>(),
            ) {
                let (commission, commission_asset) = if let (Ok(comm), asset) = (
                    report.commission_amount.parse::<f64>(),
                    &report.commission_asset,
                ) {
                    (comm, asset.clone().unwrap_or_else(|| "UNKNOWN".to_string()))
                } else {
                    (0.0, "UNKNOWN".to_string())
                };

                self.fills.push(OrderFill {
                    trade_id: report.trade_id,
                    price: fill_price,
                    quantity: fill_qty,
                    commission,
                    commission_asset,
                    timestamp: Utc::now(),
                });
            }
        }

        // Handle rejection
        if self.state == OrderState::Rejected {
            self.reject_reason = Some(report.order_reject_reason.clone());
        }
    }

    /// Check if order is still active (not terminal)
    pub fn is_active(&self) -> bool {
        self.state.is_active()
    }

    /// Check if order is in terminal state
    pub fn is_terminal(&self) -> bool {
        self.state.is_terminal()
    }

    /// Calculate total commission paid across all fills
    pub fn total_commission(&self) -> f64 {
        self.fills.iter().map(|f| f.commission).sum()
    }

    /// Get fill percentage (0.0 to 1.0)
    pub fn fill_percentage(&self) -> f64 {
        if self.original_quantity > 0.0 {
            self.executed_quantity / self.original_quantity
        } else {
            0.0
        }
    }

    /// Check if order is fully filled
    pub fn is_filled(&self) -> bool {
        self.state == OrderState::Filled
    }

    /// Get order value in quote currency
    pub fn order_value(&self) -> f64 {
        if self.average_fill_price > 0.0 {
            self.executed_quantity * self.average_fill_price
        } else if let Some(price) = self.price {
            self.original_quantity * price
        } else {
            0.0
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_order() -> RealOrder {
        RealOrder::new(
            "test-order-123".to_string(),
            "BTCUSDT".to_string(),
            "BUY".to_string(),
            "MARKET".to_string(),
            0.001,
            None,
            None,
            None,
            true,
        )
    }

    #[test]
    fn test_order_state_transitions() {
        // Pending -> New -> Filled
        assert!(OrderState::Pending.is_active());
        assert!(!OrderState::Pending.is_terminal());

        assert!(OrderState::New.is_active());
        assert!(!OrderState::New.is_terminal());

        assert!(!OrderState::Filled.is_active());
        assert!(OrderState::Filled.is_terminal());
    }

    #[test]
    fn test_order_state_from_binance() {
        assert_eq!(OrderState::from_binance_status("NEW"), OrderState::New);
        assert_eq!(
            OrderState::from_binance_status("FILLED"),
            OrderState::Filled
        );
        assert_eq!(
            OrderState::from_binance_status("CANCELED"),
            OrderState::Cancelled
        );
        assert_eq!(
            OrderState::from_binance_status("PARTIALLY_FILLED"),
            OrderState::PartiallyFilled
        );
        assert_eq!(
            OrderState::from_binance_status("REJECTED"),
            OrderState::Rejected
        );
        assert_eq!(
            OrderState::from_binance_status("EXPIRED"),
            OrderState::Expired
        );
    }

    #[test]
    fn test_new_order_initial_state() {
        let order = create_test_order();

        assert_eq!(order.state, OrderState::Pending);
        assert_eq!(order.executed_quantity, 0.0);
        assert_eq!(order.remaining_quantity, 0.001);
        assert!(order.is_active());
        assert!(!order.is_terminal());
        assert!(order.fills.is_empty());
    }

    #[test]
    fn test_order_fill_percentage() {
        let mut order = create_test_order();
        order.original_quantity = 1.0;

        // No fills
        assert_eq!(order.fill_percentage(), 0.0);

        // 50% filled
        order.executed_quantity = 0.5;
        assert!((order.fill_percentage() - 0.5).abs() < 0.0001);

        // 100% filled
        order.executed_quantity = 1.0;
        assert!((order.fill_percentage() - 1.0).abs() < 0.0001);
    }

    #[test]
    fn test_order_total_commission() {
        let mut order = create_test_order();

        // Add fills with commissions
        order.fills.push(OrderFill {
            trade_id: 1,
            price: 50000.0,
            quantity: 0.0005,
            commission: 0.001,
            commission_asset: "BNB".to_string(),
            timestamp: Utc::now(),
        });

        order.fills.push(OrderFill {
            trade_id: 2,
            price: 50100.0,
            quantity: 0.0005,
            commission: 0.001,
            commission_asset: "BNB".to_string(),
            timestamp: Utc::now(),
        });

        assert!((order.total_commission() - 0.002).abs() < 0.0001);
    }

    #[test]
    fn test_order_value_calculation() {
        let mut order = create_test_order();
        order.executed_quantity = 0.001;
        order.average_fill_price = 50000.0;

        assert!((order.order_value() - 50.0).abs() < 0.01);
    }

    // === COV36 TESTS: Cover lines 232, 245-246, 248 ===

    #[test]
    fn test_cov36_fill_percentage_zero_quantity() {
        // Covers line 232: else { 0.0 } when original_quantity == 0.0
        let mut order = create_test_order();
        order.original_quantity = 0.0;
        order.executed_quantity = 0.0;
        assert_eq!(order.fill_percentage(), 0.0);
    }

    #[test]
    fn test_cov36_order_value_with_limit_price_no_fill() {
        // Covers lines 245-246: else if let Some(price) branch in order_value()
        // when average_fill_price == 0.0 but price is set
        let order = RealOrder::new(
            "limit-order-123".to_string(),
            "BTCUSDT".to_string(),
            "BUY".to_string(),
            "LIMIT".to_string(),
            0.001,
            Some(50000.0), // limit price
            None,
            None,
            true,
        );
        // average_fill_price = 0.0, price = Some(50000.0)
        // order_value = original_quantity * price = 0.001 * 50000.0 = 50.0
        assert!((order.order_value() - 50.0).abs() < 0.01);
    }

    #[test]
    fn test_cov36_order_value_no_price_no_fill() {
        // Covers line 248: else { 0.0 } when no fill and no limit price
        let order = create_test_order(); // market order, no price, no fills
                                         // average_fill_price = 0.0, price = None
        assert_eq!(order.order_value(), 0.0);
    }

    // === COV43 TESTS ===

    /// Create a minimal ExecutionReport for testing
    fn create_test_execution_report(
        execution_type: &str,
        last_qty: &str,
        last_price: &str,
        commission: &str,
    ) -> ExecutionReport {
        ExecutionReport {
            event_type: "executionReport".to_string(),
            event_time: 1000000,
            symbol: "BTCUSDT".to_string(),
            client_order_id: "test-client-id".to_string(),
            side: "BUY".to_string(),
            order_type: "MARKET".to_string(),
            time_in_force: "GTC".to_string(),
            order_quantity: "0.001".to_string(),
            order_price: "0".to_string(),
            stop_price: "0".to_string(),
            iceberg_quantity: "0".to_string(),
            original_client_order_id: String::new(),
            execution_type: execution_type.to_string(),
            order_status: "FILLED".to_string(),
            order_reject_reason: "NONE".to_string(),
            order_id: 12345,
            last_executed_quantity: last_qty.to_string(),
            cumulative_filled_quantity: last_qty.to_string(),
            last_executed_price: last_price.to_string(),
            commission_amount: commission.to_string(),
            commission_asset: None,
            transaction_time: 1000000,
            trade_id: 67890,
            is_on_book: false,
            is_maker: false,
            order_creation_time: 999000,
            cumulative_quote_qty: "25.0".to_string(),
            last_quote_qty: "25.0".to_string(),
            quote_order_qty: "0".to_string(),
        }
    }

    /// Test update_from_execution_report with invalid commission amount (line 192)
    /// When commission_amount cannot be parsed as f64, fallback (0.0, "UNKNOWN") is used
    #[test]
    fn test_cov43_update_from_execution_report_invalid_commission() {
        let mut order = create_test_order();

        // Create TRADE execution report with valid fill but invalid commission
        let report = create_test_execution_report("TRADE", "0.001", "50000.0", "not_a_number");
        order.update_from_execution_report(&report);

        // Should have recorded the fill with fallback commission values
        assert_eq!(order.fills.len(), 1);
        assert_eq!(order.fills[0].commission, 0.0); // fallback
        assert_eq!(order.fills[0].commission_asset, "UNKNOWN"); // fallback
    }

    /// Test update_from_execution_report with non-TRADE execution type (line 203)
    /// The fills block is skipped when execution_type != "TRADE"
    #[test]
    fn test_cov43_update_from_execution_report_non_trade_type() {
        let mut order = create_test_order();

        // NEW execution type - no fill should be recorded
        let report = create_test_execution_report("NEW", "0.0", "0.0", "0");
        order.update_from_execution_report(&report);

        assert_eq!(order.fills.len(), 0);
        assert_eq!(order.state, OrderState::Filled); // Status updated
    }
}
