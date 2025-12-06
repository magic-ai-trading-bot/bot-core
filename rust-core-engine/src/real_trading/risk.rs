// @spec:FR-REAL-040 - Real Trading Risk Manager
// @spec:FR-REAL-041 - Pre-Trade Risk Validation
// @spec:FR-REAL-042 - Risk-Based Position Sizing
// @ref:specs/01-requirements/1.1-functional-requirements/FR-RISK.md
// @test:TC-REAL-040, TC-REAL-041, TC-REAL-042

//! Risk Management for Real Trading
//!
//! This module provides comprehensive risk management for real trading,
//! including pre-trade validation, position sizing, and daily loss tracking.

use anyhow::Result;
use chrono::{DateTime, Utc};
use dashmap::DashMap;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{debug, info, warn};

use super::config::RealTradingConfig;
use super::position::RealPosition;
use crate::binance::types::OrderSide;

/// Result of risk validation
#[derive(Debug, Clone)]
pub struct RiskValidationResult {
    /// Whether the validation passed
    pub passed: bool,
    /// List of validation errors if any
    pub errors: Vec<String>,
    /// List of warnings (non-blocking)
    pub warnings: Vec<String>,
    /// Suggested position size (if calculated)
    pub suggested_size: Option<f64>,
}

impl RiskValidationResult {
    pub fn success() -> Self {
        Self {
            passed: true,
            errors: Vec::new(),
            warnings: Vec::new(),
            suggested_size: None,
        }
    }

    pub fn failure(error: String) -> Self {
        Self {
            passed: false,
            errors: vec![error],
            warnings: Vec::new(),
            suggested_size: None,
        }
    }

    pub fn with_warning(mut self, warning: String) -> Self {
        self.warnings.push(warning);
        self
    }

    pub fn with_suggested_size(mut self, size: f64) -> Self {
        self.suggested_size = Some(size);
        self
    }
}

/// Risk Manager for Real Trading
///
/// Manages:
/// - Daily loss tracking and limits
/// - Pre-trade risk validation
/// - Position sizing based on risk parameters
/// - Daily counter resets
pub struct RealTradingRiskManager {
    /// Configuration for risk limits
    config: Arc<RwLock<RealTradingConfig>>,

    /// Daily loss accumulated
    daily_loss: Arc<RwLock<f64>>,

    /// Daily trade count
    daily_trades: Arc<RwLock<u32>>,

    /// Daily realized PnL (can be positive or negative)
    daily_pnl: Arc<RwLock<f64>>,

    /// Last reset timestamp
    last_reset: Arc<RwLock<DateTime<Utc>>>,
}

impl RealTradingRiskManager {
    /// Create a new risk manager
    pub fn new(config: RealTradingConfig) -> Self {
        Self {
            config: Arc::new(RwLock::new(config)),
            daily_loss: Arc::new(RwLock::new(0.0)),
            daily_trades: Arc::new(RwLock::new(0)),
            daily_pnl: Arc::new(RwLock::new(0.0)),
            last_reset: Arc::new(RwLock::new(Utc::now())),
        }
    }

    /// Check if new day and reset counters if needed
    pub async fn check_daily_reset(&self) {
        let last = *self.last_reset.read().await;
        let now = Utc::now();

        if now.date_naive() != last.date_naive() {
            *self.daily_loss.write().await = 0.0;
            *self.daily_trades.write().await = 0;
            *self.daily_pnl.write().await = 0.0;
            *self.last_reset.write().await = now;
            info!("Daily risk counters reset for new trading day");
        }
    }

    /// Record a trade result
    pub async fn record_trade(&self, pnl: f64) {
        // Update daily PnL
        let mut daily_pnl = self.daily_pnl.write().await;
        *daily_pnl += pnl;

        // Update daily loss (only count negative PnL)
        if pnl < 0.0 {
            let mut daily_loss = self.daily_loss.write().await;
            *daily_loss += pnl.abs();
        }

        // Increment trade count
        let mut trades = self.daily_trades.write().await;
        *trades += 1;

        debug!(
            "Recorded trade: PnL ${:.2}, Daily PnL ${:.2}, Daily Loss ${:.2}",
            pnl,
            *daily_pnl,
            *self.daily_loss.read().await
        );
    }

    /// Get current daily loss
    pub async fn get_daily_loss(&self) -> f64 {
        *self.daily_loss.read().await
    }

    /// Get current daily PnL
    pub async fn get_daily_pnl(&self) -> f64 {
        *self.daily_pnl.read().await
    }

    /// Get daily trade count
    pub async fn get_daily_trades(&self) -> u32 {
        *self.daily_trades.read().await
    }

    /// Check if daily loss limit has been reached
    pub async fn is_daily_loss_limit_reached(&self) -> bool {
        let config = self.config.read().await;
        let daily_loss = *self.daily_loss.read().await;
        daily_loss >= config.max_daily_loss_usdt
    }

    // ============ Pre-Trade Validation ============

    /// Validate an order against all risk limits
    ///
    /// This is the main pre-trade risk check that should be called
    /// before every order placement.
    pub async fn validate_order(
        &self,
        symbol: &str,
        side: OrderSide,
        quantity: f64,
        price: f64,
        current_positions: &DashMap<String, RealPosition>,
        balances: &HashMap<String, f64>,
    ) -> Result<RiskValidationResult> {
        self.check_daily_reset().await;

        let config = self.config.read().await;
        let mut result = RiskValidationResult::success();

        // 1. Check daily loss limit
        let daily_loss = *self.daily_loss.read().await;
        if daily_loss >= config.max_daily_loss_usdt {
            return Ok(RiskValidationResult::failure(format!(
                "Daily loss limit reached: ${:.2} >= ${:.2}",
                daily_loss, config.max_daily_loss_usdt
            )));
        }

        // Warning if approaching limit
        if daily_loss >= config.max_daily_loss_usdt * 0.8 {
            result = result.with_warning(format!(
                "Approaching daily loss limit: ${:.2} / ${:.2} (80%)",
                daily_loss, config.max_daily_loss_usdt
            ));
        }

        // 2. Check max positions
        let position_count = current_positions.len();
        let has_position = current_positions.contains_key(symbol);

        if !has_position && position_count >= config.max_positions as usize {
            return Ok(RiskValidationResult::failure(format!(
                "Max positions reached: {} >= {}",
                position_count, config.max_positions
            )));
        }

        // 3. Check position size limit
        let order_value = quantity * price;
        if order_value > config.max_position_size_usdt {
            return Ok(RiskValidationResult::failure(format!(
                "Order value ${:.2} exceeds max position size ${:.2}",
                order_value, config.max_position_size_usdt
            )));
        }

        // 4. Check total exposure limit
        let current_exposure: f64 = current_positions
            .iter()
            .map(|p| p.value().position_value())
            .sum();

        if current_exposure + order_value > config.max_total_exposure_usdt {
            return Ok(RiskValidationResult::failure(format!(
                "Total exposure ${:.2} + ${:.2} exceeds limit ${:.2}",
                current_exposure, order_value, config.max_total_exposure_usdt
            )));
        }

        // 5. Check available balance (for buy orders)
        let usdt_balance = balances.get("USDT").copied().unwrap_or(0.0);
        if side == OrderSide::Buy && order_value > usdt_balance {
            return Ok(RiskValidationResult::failure(format!(
                "Insufficient balance: need ${:.2}, have ${:.2}",
                order_value, usdt_balance
            )));
        }

        // 6. Check minimum balance requirement
        if usdt_balance < config.min_balance_usdt {
            return Ok(RiskValidationResult::failure(format!(
                "Balance ${:.2} below minimum required ${:.2}",
                usdt_balance, config.min_balance_usdt
            )));
        }

        // 7. Check risk per trade limit
        let max_risk_amount = usdt_balance * (config.risk_per_trade_percent / 100.0);
        let max_allowed_position = max_risk_amount * (100.0 / config.default_stop_loss_percent);

        if order_value > max_allowed_position {
            result = result.with_warning(format!(
                "Order value ${:.2} exceeds risk-based limit ${:.2}",
                order_value, max_allowed_position
            ));
            // Calculate suggested size
            let suggested = max_allowed_position / price;
            result = result.with_suggested_size(suggested);
        }

        // 8. Check minimum order value
        if order_value < config.min_order_value_usdt {
            return Ok(RiskValidationResult::failure(format!(
                "Order value ${:.2} below minimum ${:.2}",
                order_value, config.min_order_value_usdt
            )));
        }

        // 9. Check symbol allowlist
        if !config.is_symbol_allowed(symbol) {
            return Ok(RiskValidationResult::failure(format!(
                "Symbol {} not in allowed list",
                symbol
            )));
        }

        Ok(result)
    }

    // ============ Position Sizing ============

    /// Calculate optimal position size based on risk parameters
    ///
    /// Uses the formula: Position Size = Risk Amount / Stop Loss Distance
    pub fn calculate_position_size(
        &self,
        entry_price: f64,
        stop_loss: f64,
        account_balance: f64,
        config: &RealTradingConfig,
    ) -> f64 {
        // Validate inputs
        if entry_price <= 0.0 || account_balance <= 0.0 {
            debug!("Invalid inputs for position sizing");
            return 0.0;
        }

        // Calculate risk amount (risk_percentage of account balance)
        let risk_amount = account_balance * (config.risk_per_trade_percent / 100.0);

        // Calculate stop loss distance as decimal
        let stop_distance = (entry_price - stop_loss).abs() / entry_price;

        // Minimum stop distance to prevent huge positions (0.5%)
        const MIN_STOP_DISTANCE: f64 = 0.005;
        let effective_stop_distance = if stop_distance < MIN_STOP_DISTANCE {
            warn!(
                "Stop loss distance {:.2}% too tight, using minimum {:.2}%",
                stop_distance * 100.0,
                MIN_STOP_DISTANCE * 100.0
            );
            MIN_STOP_DISTANCE
        } else {
            stop_distance
        };

        // Calculate position value and size
        let position_value = risk_amount / effective_stop_distance;
        let position_size = position_value / entry_price;

        // Apply limits
        let max_size_by_position = config.max_position_size_usdt / entry_price;
        let max_size_by_exposure = config.max_total_exposure_usdt / entry_price;

        let final_size = position_size
            .min(max_size_by_position)
            .min(max_size_by_exposure);

        debug!(
            "Position size calculation: risk=${:.2}, stop_dist={:.4}, size={:.8}",
            risk_amount, effective_stop_distance, final_size
        );

        final_size
    }

    /// Calculate position size with automatic stop loss
    pub async fn calculate_position_size_auto_sl(
        &self,
        entry_price: f64,
        account_balance: f64,
        is_long: bool,
    ) -> (f64, f64) {
        let config = self.config.read().await;

        // Calculate stop loss based on default percentage
        let stop_loss = if is_long {
            entry_price * (1.0 - config.default_stop_loss_percent / 100.0)
        } else {
            entry_price * (1.0 + config.default_stop_loss_percent / 100.0)
        };

        let size = self.calculate_position_size(entry_price, stop_loss, account_balance, &config);

        (size, stop_loss)
    }

    // ============ Stop Loss / Take Profit Calculation ============

    /// Calculate stop loss price from entry
    pub async fn calculate_stop_loss(&self, entry_price: f64, is_long: bool) -> f64 {
        let config = self.config.read().await;
        config.calculate_stop_loss(entry_price, is_long)
    }

    /// Calculate take profit price from entry
    pub async fn calculate_take_profit(&self, entry_price: f64, is_long: bool) -> f64 {
        let config = self.config.read().await;
        config.calculate_take_profit(entry_price, is_long)
    }

    /// Calculate both SL and TP for a position
    pub async fn calculate_sl_tp(&self, entry_price: f64, is_long: bool) -> (f64, f64) {
        let config = self.config.read().await;
        (
            config.calculate_stop_loss(entry_price, is_long),
            config.calculate_take_profit(entry_price, is_long),
        )
    }

    // ============ Risk Metrics ============

    /// Get current risk utilization (0.0 to 1.0)
    pub async fn get_risk_utilization(
        &self,
        current_positions: &DashMap<String, RealPosition>,
    ) -> f64 {
        let config = self.config.read().await;

        let current_exposure: f64 = current_positions
            .iter()
            .map(|p| p.value().position_value())
            .sum();

        if config.max_total_exposure_usdt > 0.0 {
            current_exposure / config.max_total_exposure_usdt
        } else {
            0.0
        }
    }

    /// Get daily loss utilization (0.0 to 1.0)
    pub async fn get_daily_loss_utilization(&self) -> f64 {
        let config = self.config.read().await;
        let daily_loss = *self.daily_loss.read().await;

        if config.max_daily_loss_usdt > 0.0 {
            daily_loss / config.max_daily_loss_usdt
        } else {
            0.0
        }
    }

    /// Update configuration
    pub async fn update_config(&self, config: RealTradingConfig) {
        *self.config.write().await = config;
    }

    /// Get current configuration
    pub async fn get_config(&self) -> RealTradingConfig {
        self.config.read().await.clone()
    }
}

// Allow cloning for the risk manager
impl Clone for RealTradingRiskManager {
    fn clone(&self) -> Self {
        Self {
            config: Arc::clone(&self.config),
            daily_loss: Arc::clone(&self.daily_loss),
            daily_trades: Arc::clone(&self.daily_trades),
            daily_pnl: Arc::clone(&self.daily_pnl),
            last_reset: Arc::clone(&self.last_reset),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_config() -> RealTradingConfig {
        RealTradingConfig {
            max_position_size_usdt: 1000.0,
            max_total_exposure_usdt: 5000.0,
            max_daily_loss_usdt: 500.0,
            max_positions: 5,
            risk_per_trade_percent: 2.0,
            min_balance_usdt: 100.0,
            default_stop_loss_percent: 2.0,
            default_take_profit_percent: 4.0,
            min_order_value_usdt: 10.0,
            ..RealTradingConfig::default()
        }
    }

    #[tokio::test]
    async fn test_risk_manager_new() {
        let config = create_test_config();
        let rm = RealTradingRiskManager::new(config);

        assert_eq!(rm.get_daily_loss().await, 0.0);
        assert_eq!(rm.get_daily_trades().await, 0);
    }

    #[tokio::test]
    async fn test_record_trade_loss() {
        let rm = RealTradingRiskManager::new(create_test_config());

        rm.record_trade(-50.0).await;

        assert_eq!(rm.get_daily_loss().await, 50.0);
        assert_eq!(rm.get_daily_pnl().await, -50.0);
        assert_eq!(rm.get_daily_trades().await, 1);
    }

    #[tokio::test]
    async fn test_record_trade_profit() {
        let rm = RealTradingRiskManager::new(create_test_config());

        rm.record_trade(100.0).await;

        assert_eq!(rm.get_daily_loss().await, 0.0); // Loss not increased
        assert_eq!(rm.get_daily_pnl().await, 100.0);
        assert_eq!(rm.get_daily_trades().await, 1);
    }

    #[tokio::test]
    async fn test_daily_loss_limit_reached() {
        let rm = RealTradingRiskManager::new(create_test_config());

        // Record losses approaching limit
        rm.record_trade(-250.0).await;
        assert!(!rm.is_daily_loss_limit_reached().await);

        rm.record_trade(-250.0).await;
        assert!(rm.is_daily_loss_limit_reached().await);
    }

    #[tokio::test]
    async fn test_validate_order_success() {
        let rm = RealTradingRiskManager::new(create_test_config());
        let positions = DashMap::new();
        let mut balances = HashMap::new();
        balances.insert("USDT".to_string(), 10000.0);

        let result = rm
            .validate_order(
                "BTCUSDT",
                OrderSide::Buy,
                0.01,
                50000.0, // $500 order
                &positions,
                &balances,
            )
            .await
            .unwrap();

        assert!(result.passed);
        assert!(result.errors.is_empty());
    }

    #[tokio::test]
    async fn test_validate_order_daily_loss_limit() {
        let rm = RealTradingRiskManager::new(create_test_config());

        // Hit daily loss limit
        rm.record_trade(-500.0).await;

        let positions = DashMap::new();
        let mut balances = HashMap::new();
        balances.insert("USDT".to_string(), 10000.0);

        let result = rm
            .validate_order(
                "BTCUSDT",
                OrderSide::Buy,
                0.01,
                50000.0,
                &positions,
                &balances,
            )
            .await
            .unwrap();

        assert!(!result.passed);
        assert!(result.errors[0].contains("Daily loss limit"));
    }

    #[tokio::test]
    async fn test_validate_order_max_positions() {
        let rm = RealTradingRiskManager::new(create_test_config());
        let positions = DashMap::new();

        // Add 5 positions (max)
        for i in 0..5 {
            positions.insert(
                format!("SYMBOL{}", i),
                RealPosition::new(
                    format!("pos-{}", i),
                    format!("SYMBOL{}", i),
                    super::super::position::PositionSide::Long,
                    0.01,
                    50000.0,
                    "order".to_string(),
                    None,
                    None,
                ),
            );
        }

        let mut balances = HashMap::new();
        balances.insert("USDT".to_string(), 10000.0);

        let result = rm
            .validate_order(
                "NEWCOIN", // New symbol
                OrderSide::Buy,
                0.01,
                1000.0,
                &positions,
                &balances,
            )
            .await
            .unwrap();

        assert!(!result.passed);
        assert!(result.errors[0].contains("Max positions"));
    }

    #[tokio::test]
    async fn test_validate_order_position_size_limit() {
        let rm = RealTradingRiskManager::new(create_test_config());
        let positions = DashMap::new();
        let mut balances = HashMap::new();
        balances.insert("USDT".to_string(), 10000.0);

        let result = rm
            .validate_order(
                "BTCUSDT",
                OrderSide::Buy,
                0.1,
                50000.0, // $5000 order > $1000 limit
                &positions,
                &balances,
            )
            .await
            .unwrap();

        assert!(!result.passed);
        assert!(result.errors[0].contains("exceeds max position size"));
    }

    #[tokio::test]
    async fn test_validate_order_exposure_limit() {
        let rm = RealTradingRiskManager::new(create_test_config());
        let positions = DashMap::new();

        // Add existing position worth $4500
        let mut pos = RealPosition::new(
            "pos-1".to_string(),
            "ETHUSDT".to_string(),
            super::super::position::PositionSide::Long,
            1.5,
            3000.0,
            "order".to_string(),
            None,
            None,
        );
        pos.update_price(3000.0);
        positions.insert("ETHUSDT".to_string(), pos);

        let mut balances = HashMap::new();
        balances.insert("USDT".to_string(), 10000.0);

        let result = rm
            .validate_order(
                "BTCUSDT",
                OrderSide::Buy,
                0.02,
                50000.0, // $1000 order, total would be $5500 > $5000
                &positions,
                &balances,
            )
            .await
            .unwrap();

        assert!(!result.passed);
        assert!(result.errors[0].contains("exposure"));
    }

    #[tokio::test]
    async fn test_validate_order_insufficient_balance() {
        let rm = RealTradingRiskManager::new(create_test_config());
        let positions = DashMap::new();
        let mut balances = HashMap::new();
        balances.insert("USDT".to_string(), 100.0); // Only $100

        let result = rm
            .validate_order(
                "BTCUSDT",
                OrderSide::Buy,
                0.01,
                50000.0, // $500 order
                &positions,
                &balances,
            )
            .await
            .unwrap();

        assert!(!result.passed);
        assert!(result.errors[0].contains("Insufficient balance"));
    }

    #[tokio::test]
    async fn test_validate_order_min_balance() {
        let rm = RealTradingRiskManager::new(create_test_config());
        let positions = DashMap::new();
        let mut balances = HashMap::new();
        balances.insert("USDT".to_string(), 50.0); // Below min $100

        let result = rm
            .validate_order(
                "BTCUSDT",
                OrderSide::Buy,
                0.0001,
                50000.0, // $5 order
                &positions,
                &balances,
            )
            .await
            .unwrap();

        assert!(!result.passed);
        assert!(result.errors[0].contains("below minimum"));
    }

    #[tokio::test]
    async fn test_validate_order_min_order_value() {
        let rm = RealTradingRiskManager::new(create_test_config());
        let positions = DashMap::new();
        let mut balances = HashMap::new();
        balances.insert("USDT".to_string(), 1000.0);

        let result = rm
            .validate_order(
                "BTCUSDT",
                OrderSide::Buy,
                0.00001,
                50000.0, // $0.50 order < $10 min
                &positions,
                &balances,
            )
            .await
            .unwrap();

        assert!(!result.passed);
        assert!(result.errors[0].contains("below minimum"));
    }

    #[tokio::test]
    async fn test_validate_order_warning_approaching_loss() {
        let rm = RealTradingRiskManager::new(create_test_config());

        // Record 80% of daily loss limit
        rm.record_trade(-400.0).await;

        let positions = DashMap::new();
        let mut balances = HashMap::new();
        balances.insert("USDT".to_string(), 10000.0);

        let result = rm
            .validate_order(
                "BTCUSDT",
                OrderSide::Buy,
                0.01,
                50000.0,
                &positions,
                &balances,
            )
            .await
            .unwrap();

        assert!(result.passed);
        assert!(!result.warnings.is_empty());
        assert!(result.warnings[0].contains("Approaching daily loss"));
    }

    #[test]
    fn test_calculate_position_size() {
        let rm = RealTradingRiskManager::new(create_test_config());
        let config = create_test_config();

        // Entry at $50,000, SL at $49,000 (2%)
        let size = rm.calculate_position_size(50000.0, 49000.0, 10000.0, &config);

        // Risk: 10000 * 2% = 200
        // Stop distance: 1000/50000 = 0.02
        // Position value: 200 / 0.02 = 10000
        // Position size: 10000 / 50000 = 0.2, but capped at max_position_size/price
        // Max: 1000/50000 = 0.02
        assert!((size - 0.02).abs() < 0.001);
    }

    #[test]
    fn test_calculate_position_size_tight_stop() {
        let rm = RealTradingRiskManager::new(create_test_config());
        let config = create_test_config();

        // Very tight stop (0.1% distance)
        let size = rm.calculate_position_size(50000.0, 49950.0, 10000.0, &config);

        // Should use minimum 0.5% stop distance
        // Risk: 200, Stop dist: 0.005, Value: 40000, Size: 0.8
        // But capped at max_position_size/price = 0.02
        assert!(size > 0.0);
        assert!(size <= 0.02);
    }

    #[test]
    fn test_calculate_position_size_zero_inputs() {
        let rm = RealTradingRiskManager::new(create_test_config());
        let config = create_test_config();

        assert_eq!(
            rm.calculate_position_size(0.0, 49000.0, 10000.0, &config),
            0.0
        );
        assert_eq!(
            rm.calculate_position_size(50000.0, 49000.0, 0.0, &config),
            0.0
        );
    }

    #[tokio::test]
    async fn test_calculate_sl_tp_long() {
        let rm = RealTradingRiskManager::new(create_test_config());

        let (sl, tp) = rm.calculate_sl_tp(50000.0, true).await;

        // SL: 50000 * (1 - 0.02) = 49000
        assert!((sl - 49000.0).abs() < 0.01);
        // TP: 50000 * (1 + 0.04) = 52000
        assert!((tp - 52000.0).abs() < 0.01);
    }

    #[tokio::test]
    async fn test_calculate_sl_tp_short() {
        let rm = RealTradingRiskManager::new(create_test_config());

        let (sl, tp) = rm.calculate_sl_tp(50000.0, false).await;

        // SL: 50000 * (1 + 0.02) = 51000
        assert!((sl - 51000.0).abs() < 0.01);
        // TP: 50000 * (1 - 0.04) = 48000
        assert!((tp - 48000.0).abs() < 0.01);
    }

    #[tokio::test]
    async fn test_risk_utilization() {
        let rm = RealTradingRiskManager::new(create_test_config());
        let positions = DashMap::new();

        // Empty positions
        assert_eq!(rm.get_risk_utilization(&positions).await, 0.0);

        // Add position worth $2500 (50% of $5000 limit)
        let mut pos = RealPosition::new(
            "pos-1".to_string(),
            "BTCUSDT".to_string(),
            super::super::position::PositionSide::Long,
            0.05,
            50000.0,
            "order".to_string(),
            None,
            None,
        );
        pos.update_price(50000.0);
        positions.insert("BTCUSDT".to_string(), pos);

        let util = rm.get_risk_utilization(&positions).await;
        assert!((util - 0.5).abs() < 0.01);
    }

    #[tokio::test]
    async fn test_daily_loss_utilization() {
        let rm = RealTradingRiskManager::new(create_test_config());

        assert_eq!(rm.get_daily_loss_utilization().await, 0.0);

        rm.record_trade(-250.0).await; // 50% of $500 limit
        assert!((rm.get_daily_loss_utilization().await - 0.5).abs() < 0.01);

        rm.record_trade(-250.0).await; // Now at 100%
        assert!((rm.get_daily_loss_utilization().await - 1.0).abs() < 0.01);
    }

    #[tokio::test]
    async fn test_calculate_position_size_auto_sl() {
        let rm = RealTradingRiskManager::new(create_test_config());

        let (size, sl) = rm
            .calculate_position_size_auto_sl(50000.0, 10000.0, true)
            .await;

        // SL should be 2% below entry for long
        assert!((sl - 49000.0).abs() < 0.01);
        // Size should be positive
        assert!(size > 0.0);
    }
}
