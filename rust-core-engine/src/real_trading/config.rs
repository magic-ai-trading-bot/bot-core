// @spec:FR-REAL-012 - Real Trading Configuration
// @ref:specs/01-requirements/1.1-functional-requirements/FR-RISK.md
// @test:TC-REAL-020, TC-REAL-021

use serde::{Deserialize, Serialize};

/// Configuration for real trading engine
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct RealTradingConfig {
    // ============ Risk Limits ============
    /// Maximum value for a single position in USDT
    pub max_position_size_usdt: f64,

    /// Maximum total exposure across all positions in USDT
    pub max_total_exposure_usdt: f64,

    /// Maximum daily loss in USDT before trading halts
    pub max_daily_loss_usdt: f64,

    /// Maximum concurrent open positions
    pub max_positions: u32,

    /// Risk percentage per trade (% of balance)
    pub risk_per_trade_percent: f64,

    /// Minimum required balance to trade
    pub min_balance_usdt: f64,

    // ============ Circuit Breaker ============
    /// Number of consecutive errors before circuit opens
    pub circuit_breaker_errors: u32,

    /// Cooldown period in seconds after circuit opens
    pub circuit_breaker_cooldown_secs: u64,

    /// Whether to automatically close positions when circuit opens
    pub circuit_breaker_close_positions: bool,

    // ============ Reconciliation ============
    /// Interval for REST API reconciliation in seconds
    pub reconciliation_interval_secs: u64,

    /// Timeout for considering an order stale (cancel after)
    pub stale_order_timeout_secs: u64,

    /// Maximum discrepancy before emergency stop (in USDT)
    pub max_reconciliation_discrepancy_usdt: f64,

    // ============ Order Settings ============
    /// Default slippage tolerance for limit orders (%)
    pub default_slippage_percent: f64,

    /// Order timeout - cancel if not filled after N seconds
    pub order_timeout_secs: u64,

    /// Use post-only orders when possible (maker orders)
    pub prefer_maker_orders: bool,

    /// Minimum order value in USDT
    pub min_order_value_usdt: f64,

    // ============ Stop Loss / Take Profit ============
    /// Default stop loss percentage
    pub default_stop_loss_percent: f64,

    /// Default take profit percentage
    pub default_take_profit_percent: f64,

    /// Enable trailing stop by default
    pub enable_trailing_stop: bool,

    /// Trailing stop activation (% profit before activating)
    pub trailing_stop_activation_percent: f64,

    /// Trailing stop distance (%)
    pub trailing_stop_percent: f64,

    // ============ Trading Mode ============
    /// Use testnet instead of mainnet
    pub use_testnet: bool,

    /// Symbols allowed for trading (empty = all)
    pub allowed_symbols: Vec<String>,

    /// Maximum leverage (1 = no leverage, spot only)
    pub max_leverage: u32,

    // ============ Logging & Monitoring ============
    /// Log all order events
    pub log_order_events: bool,

    /// Log all position updates
    pub log_position_updates: bool,

    /// Log execution reports from WebSocket
    pub log_execution_reports: bool,

    /// Send alerts on errors
    pub send_error_alerts: bool,
}

impl Default for RealTradingConfig {
    fn default() -> Self {
        Self {
            // Conservative risk limits for safety
            max_position_size_usdt: 1000.0,
            max_total_exposure_usdt: 5000.0,
            max_daily_loss_usdt: 500.0,
            max_positions: 5,
            risk_per_trade_percent: 2.0,
            min_balance_usdt: 100.0,

            // Circuit breaker
            circuit_breaker_errors: 3,
            circuit_breaker_cooldown_secs: 300, // 5 minutes
            circuit_breaker_close_positions: false,

            // Reconciliation
            reconciliation_interval_secs: 300, // 5 minutes
            stale_order_timeout_secs: 300,     // 5 minutes
            max_reconciliation_discrepancy_usdt: 100.0,

            // Order settings
            default_slippage_percent: 0.1,
            order_timeout_secs: 60,
            prefer_maker_orders: false,
            min_order_value_usdt: 10.0,

            // SL/TP
            default_stop_loss_percent: 2.0,
            default_take_profit_percent: 4.0,
            enable_trailing_stop: false,
            trailing_stop_activation_percent: 1.5,
            trailing_stop_percent: 1.0,

            // Trading mode - default to testnet for safety
            use_testnet: true,
            allowed_symbols: vec![],
            max_leverage: 1,

            // Logging
            log_order_events: true,
            log_position_updates: true,
            log_execution_reports: true,
            send_error_alerts: true,
        }
    }
}

impl RealTradingConfig {
    /// Create config for testnet testing
    pub fn testnet_default() -> Self {
        Self {
            use_testnet: true,
            max_position_size_usdt: 100.0,
            max_total_exposure_usdt: 500.0,
            max_daily_loss_usdt: 50.0,
            ..Default::default()
        }
    }

    /// Create config for production (more conservative)
    pub fn production_default() -> Self {
        Self {
            use_testnet: false,
            circuit_breaker_errors: 2,             // More sensitive
            circuit_breaker_close_positions: true, // Close on circuit break
            reconciliation_interval_secs: 60,      // More frequent
            send_error_alerts: true,
            ..Default::default()
        }
    }

    /// Validate configuration
    pub fn validate(&self) -> Result<(), Vec<String>> {
        let mut errors = Vec::new();

        if self.max_position_size_usdt <= 0.0 {
            errors.push("max_position_size_usdt must be positive".to_string());
        }

        if self.max_total_exposure_usdt < self.max_position_size_usdt {
            errors.push("max_total_exposure_usdt must be >= max_position_size_usdt".to_string());
        }

        if self.risk_per_trade_percent <= 0.0 || self.risk_per_trade_percent > 100.0 {
            errors.push("risk_per_trade_percent must be between 0 and 100".to_string());
        }

        if self.circuit_breaker_errors == 0 {
            errors.push("circuit_breaker_errors must be at least 1".to_string());
        }

        if self.default_slippage_percent < 0.0 || self.default_slippage_percent > 10.0 {
            errors.push("default_slippage_percent should be between 0 and 10".to_string());
        }

        if self.default_stop_loss_percent <= 0.0 || self.default_stop_loss_percent > 50.0 {
            errors.push("default_stop_loss_percent should be between 0 and 50".to_string());
        }

        if self.max_leverage == 0 || self.max_leverage > 125 {
            errors.push("max_leverage must be between 1 and 125".to_string());
        }

        if errors.is_empty() {
            Ok(())
        } else {
            Err(errors)
        }
    }

    /// Check if a symbol is allowed for trading
    pub fn is_symbol_allowed(&self, symbol: &str) -> bool {
        if self.allowed_symbols.is_empty() {
            true
        } else {
            self.allowed_symbols.contains(&symbol.to_string())
        }
    }

    /// Calculate maximum position size based on balance
    pub fn calculate_max_position_size(&self, balance: f64) -> f64 {
        let risk_based = balance * (self.risk_per_trade_percent / 100.0);
        risk_based.min(self.max_position_size_usdt)
    }

    /// Calculate stop loss price from entry
    pub fn calculate_stop_loss(&self, entry_price: f64, is_long: bool) -> f64 {
        if is_long {
            entry_price * (1.0 - self.default_stop_loss_percent / 100.0)
        } else {
            entry_price * (1.0 + self.default_stop_loss_percent / 100.0)
        }
    }

    /// Calculate take profit price from entry
    pub fn calculate_take_profit(&self, entry_price: f64, is_long: bool) -> f64 {
        if is_long {
            entry_price * (1.0 + self.default_take_profit_percent / 100.0)
        } else {
            entry_price * (1.0 - self.default_take_profit_percent / 100.0)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = RealTradingConfig::default();

        assert!(config.use_testnet); // Default to testnet for safety
        assert_eq!(config.max_positions, 5);
        assert_eq!(config.circuit_breaker_errors, 3);
    }

    #[test]
    fn test_config_validation() {
        let mut config = RealTradingConfig::default();

        // Valid config
        assert!(config.validate().is_ok());

        // Invalid: negative position size
        config.max_position_size_usdt = -100.0;
        assert!(config.validate().is_err());

        config.max_position_size_usdt = 1000.0;

        // Invalid: exposure less than position size
        config.max_total_exposure_usdt = 500.0;
        assert!(config.validate().is_err());
    }

    #[test]
    fn test_symbol_allowed() {
        let mut config = RealTradingConfig::default();

        // Empty list = all allowed
        assert!(config.is_symbol_allowed("BTCUSDT"));
        assert!(config.is_symbol_allowed("ETHUSDT"));

        // Restricted list
        config.allowed_symbols = vec!["BTCUSDT".to_string(), "ETHUSDT".to_string()];
        assert!(config.is_symbol_allowed("BTCUSDT"));
        assert!(!config.is_symbol_allowed("DOGEUSDT"));
    }

    #[test]
    fn test_calculate_max_position_size() {
        let config = RealTradingConfig {
            max_position_size_usdt: 1000.0,
            risk_per_trade_percent: 5.0,
            ..Default::default()
        };

        // 5% of 10000 = 500, which is less than max 1000
        assert!((config.calculate_max_position_size(10000.0) - 500.0).abs() < 0.01);

        // 5% of 30000 = 1500, but capped at max 1000
        assert!((config.calculate_max_position_size(30000.0) - 1000.0).abs() < 0.01);
    }

    #[test]
    fn test_calculate_stop_loss() {
        let config = RealTradingConfig {
            default_stop_loss_percent: 2.0,
            ..Default::default()
        };

        // Long: SL below entry
        let sl_long = config.calculate_stop_loss(50000.0, true);
        assert!((sl_long - 49000.0).abs() < 0.01); // 50000 * 0.98

        // Short: SL above entry
        let sl_short = config.calculate_stop_loss(50000.0, false);
        assert!((sl_short - 51000.0).abs() < 0.01); // 50000 * 1.02
    }

    #[test]
    fn test_calculate_take_profit() {
        let config = RealTradingConfig {
            default_take_profit_percent: 4.0,
            ..Default::default()
        };

        // Long: TP above entry
        let tp_long = config.calculate_take_profit(50000.0, true);
        assert!((tp_long - 52000.0).abs() < 0.01); // 50000 * 1.04

        // Short: TP below entry
        let tp_short = config.calculate_take_profit(50000.0, false);
        assert!((tp_short - 48000.0).abs() < 0.01); // 50000 * 0.96
    }

    #[test]
    fn test_testnet_config() {
        let config = RealTradingConfig::testnet_default();

        assert!(config.use_testnet);
        assert_eq!(config.max_position_size_usdt, 100.0); // Smaller for testing
    }

    #[test]
    fn test_production_config() {
        let config = RealTradingConfig::production_default();

        assert!(!config.use_testnet);
        assert_eq!(config.circuit_breaker_errors, 2); // More sensitive
        assert!(config.circuit_breaker_close_positions);
        assert_eq!(config.reconciliation_interval_secs, 60); // More frequent
    }

    // === COV8 TESTS: Additional coverage for real_trading/config.rs (91.80% → 95%+) ===

    #[test]
    fn test_cov8_validation_zero_circuit_breaker_errors() {
        let mut config = RealTradingConfig::default();
        config.circuit_breaker_errors = 0;
        let result = config.validate();
        assert!(result.is_err());
        let errors = result.unwrap_err();
        assert!(errors.iter().any(|e| e.contains("circuit_breaker_errors")));
    }

    #[test]
    fn test_cov8_validation_invalid_slippage_negative() {
        let mut config = RealTradingConfig::default();
        config.default_slippage_percent = -1.0;
        let result = config.validate();
        assert!(result.is_err());
        let errors = result.unwrap_err();
        assert!(errors.iter().any(|e| e.contains("slippage")));
    }

    #[test]
    fn test_cov8_validation_invalid_slippage_too_high() {
        let mut config = RealTradingConfig::default();
        config.default_slippage_percent = 15.0;
        let result = config.validate();
        assert!(result.is_err());
        let errors = result.unwrap_err();
        assert!(errors.iter().any(|e| e.contains("slippage")));
    }

    #[test]
    fn test_cov8_validation_zero_stop_loss() {
        let mut config = RealTradingConfig::default();
        config.default_stop_loss_percent = 0.0;
        let result = config.validate();
        assert!(result.is_err());
        let errors = result.unwrap_err();
        assert!(errors.iter().any(|e| e.contains("stop_loss")));
    }

    #[test]
    fn test_cov8_validation_stop_loss_too_high() {
        let mut config = RealTradingConfig::default();
        config.default_stop_loss_percent = 60.0;
        let result = config.validate();
        assert!(result.is_err());
        let errors = result.unwrap_err();
        assert!(errors.iter().any(|e| e.contains("stop_loss")));
    }

    #[test]
    fn test_cov8_validation_zero_leverage() {
        let mut config = RealTradingConfig::default();
        config.max_leverage = 0;
        let result = config.validate();
        assert!(result.is_err());
        let errors = result.unwrap_err();
        assert!(errors.iter().any(|e| e.contains("leverage")));
    }

    #[test]
    fn test_cov8_validation_leverage_too_high() {
        let mut config = RealTradingConfig::default();
        config.max_leverage = 126;
        let result = config.validate();
        assert!(result.is_err());
        let errors = result.unwrap_err();
        assert!(errors.iter().any(|e| e.contains("leverage")));
    }

    #[test]
    fn test_cov8_validation_multiple_errors() {
        let mut config = RealTradingConfig::default();
        config.max_position_size_usdt = -100.0;
        config.circuit_breaker_errors = 0;
        config.default_slippage_percent = -5.0;
        let result = config.validate();
        assert!(result.is_err());
        let errors = result.unwrap_err();
        assert!(errors.len() >= 3);
    }

    #[test]
    fn test_cov8_is_symbol_allowed_not_in_list() {
        let mut config = RealTradingConfig::default();
        config.allowed_symbols = vec!["BTCUSDT".to_string()];
        assert!(!config.is_symbol_allowed("ETHUSDT"));
        assert!(!config.is_symbol_allowed("SOLUSDT"));
    }

    #[test]
    fn test_cov8_calculate_max_position_risk_based_larger() {
        let config = RealTradingConfig {
            max_position_size_usdt: 10000.0,
            risk_per_trade_percent: 1.0,
            ..Default::default()
        };
        let balance = 50000.0;
        let max_size = config.calculate_max_position_size(balance);
        assert_eq!(max_size, 500.0); // 1% of 50000
    }

    #[test]
    fn test_cov8_calculate_max_position_capped() {
        let config = RealTradingConfig {
            max_position_size_usdt: 1000.0,
            risk_per_trade_percent: 10.0,
            ..Default::default()
        };
        let balance = 50000.0;
        let max_size = config.calculate_max_position_size(balance);
        assert_eq!(max_size, 1000.0); // Capped at max
    }

    #[test]
    fn test_cov8_calculate_stop_loss_short() {
        let config = RealTradingConfig {
            default_stop_loss_percent: 3.0,
            ..Default::default()
        };
        let entry = 50000.0;
        let sl = config.calculate_stop_loss(entry, false);
        assert!((sl - 51500.0).abs() < 0.01); // Short: SL above entry
    }

    #[test]
    fn test_cov8_calculate_take_profit_short() {
        let config = RealTradingConfig {
            default_take_profit_percent: 5.0,
            ..Default::default()
        };
        let entry = 50000.0;
        let tp = config.calculate_take_profit(entry, false);
        assert!((tp - 47500.0).abs() < 0.01); // Short: TP below entry
    }

    #[test]
    fn test_cov8_serde_default_annotation() {
        // Test that #[serde(default)] works correctly
        let json = "{}";
        let config: RealTradingConfig = serde_json::from_str(json).unwrap();
        assert_eq!(config.max_position_size_usdt, 1000.0);
        assert_eq!(config.use_testnet, true);
    }

    #[test]
    fn test_cov8_config_clone() {
        let config = RealTradingConfig::default();
        let cloned = config.clone();
        assert_eq!(config.max_position_size_usdt, cloned.max_position_size_usdt);
        assert_eq!(config.use_testnet, cloned.use_testnet);
    }
}
