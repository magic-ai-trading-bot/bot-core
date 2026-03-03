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

    /// Trading type: "spot" or "futures" (USDT-M)
    pub trading_type: String,

    /// Symbols allowed for trading (empty = all)
    pub allowed_symbols: Vec<String>,

    /// Maximum leverage (1 = no leverage, spot only)
    pub max_leverage: u32,

    // ============ Auto-Trading (Strategy Signal Automation) ============
    /// Enable automatic trading from strategy signals (SAFETY: default false)
    pub auto_trading_enabled: bool,

    /// Minimum signal confidence to execute (0.0-1.0)
    pub min_signal_confidence: f64,

    /// Maximum consecutive losses before cool-down
    pub max_consecutive_losses: u32,

    /// Cool-down period in minutes after max consecutive losses
    pub cool_down_minutes: u32,

    /// Maximum directional exposure ratio (e.g., 0.70 = 70% in one direction)
    pub correlation_limit: f64,

    /// Maximum total portfolio risk as % of equity
    pub max_portfolio_risk_pct: f64,

    /// Only allow short positions (no longs)
    pub short_only_mode: bool,

    /// Only allow long positions (no shorts)
    pub long_only_mode: bool,

    /// Symbols to auto-trade (empty = use allowed_symbols)
    pub auto_trade_symbols: Vec<String>,

    // ============ ATR-Based Sizing ============
    /// Enable ATR-based stop loss and take profit (overrides default_stop_loss/take_profit)
    pub atr_stop_enabled: bool,

    /// ATR calculation period (number of candles)
    pub atr_period: usize,

    /// ATR multiplier for stop loss distance (SL = entry +/- ATR * multiplier)
    pub atr_stop_multiplier: f64,

    /// ATR multiplier for take profit distance (TP = entry +/- ATR * multiplier)
    pub atr_tp_multiplier: f64,

    /// Base risk percentage per trade when using ATR sizing (% of equity risked)
    pub base_risk_pct: f64,

    // ============ Kelly Criterion ============
    /// Enable Kelly criterion position sizing
    pub kelly_enabled: bool,

    /// Minimum closed trades before Kelly activates
    pub kelly_min_trades: u64,

    /// Kelly fraction (0.5 = Half-Kelly for safety)
    pub kelly_fraction: f64,

    /// Number of recent trades to use for Kelly calculation
    pub kelly_lookback: u64,

    // ============ Regime Filters ============
    /// Enable funding rate spike filter (reduces size when funding is extreme)
    pub funding_spike_filter_enabled: bool,

    /// Funding rate threshold above which filter activates (absolute value)
    pub funding_spike_threshold: f64,

    /// Position size multiplier when funding spike detected (0.0-1.0)
    pub funding_spike_reduction: f64,

    /// Enable ATR spike filter (reduces size when volatility is extreme)
    pub atr_spike_filter_enabled: bool,

    /// ATR spike threshold: current_atr > mean_atr * multiplier triggers filter
    pub atr_spike_multiplier: f64,

    /// Position size multiplier when ATR spike detected (0.0-1.0)
    pub atr_spike_reduction: f64,

    /// Enable gradual position reduction after consecutive losses
    pub consecutive_loss_reduction_enabled: bool,

    /// Reduction factor per consecutive loss beyond threshold (0.0-1.0)
    pub consecutive_loss_reduction_pct: f64,

    /// Number of consecutive losses before reduction kicks in
    pub consecutive_loss_reduction_threshold: u32,

    // ============ Signal Reversal ============
    /// Enable advanced signal reversal logic (requires confidence + regime check)
    pub enable_signal_reversal: bool,

    /// Minimum signal confidence required for reversal
    pub reversal_min_confidence: f64,

    /// Maximum PnL% on existing position to allow reversal (safety cap)
    pub reversal_max_pnl_pct: f64,

    /// Market regimes where reversal is allowed (e.g., ["trending"])
    pub reversal_allowed_regimes: Vec<String>,

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
            trading_type: "futures".to_string(),
            allowed_symbols: vec![],
            max_leverage: 1,

            // Auto-trading — DISABLED by default for safety
            auto_trading_enabled: false,
            min_signal_confidence: 0.65,
            max_consecutive_losses: 3,
            cool_down_minutes: 60,
            correlation_limit: 0.70,
            max_portfolio_risk_pct: 10.0,
            short_only_mode: false,
            long_only_mode: false,
            auto_trade_symbols: vec![],

            // ATR-Based Sizing — DISABLED by default
            atr_stop_enabled: false,
            atr_period: 14,
            atr_stop_multiplier: 1.2,
            atr_tp_multiplier: 2.4,
            base_risk_pct: 2.0,

            // Kelly Criterion — DISABLED by default
            kelly_enabled: false,
            kelly_min_trades: 200,
            kelly_fraction: 0.5,
            kelly_lookback: 100,

            // Regime Filters — all DISABLED by default
            funding_spike_filter_enabled: false,
            funding_spike_threshold: 0.0003,
            funding_spike_reduction: 0.5,
            atr_spike_filter_enabled: false,
            atr_spike_multiplier: 2.0,
            atr_spike_reduction: 0.5,
            consecutive_loss_reduction_enabled: false,
            consecutive_loss_reduction_pct: 0.3,
            consecutive_loss_reduction_threshold: 3,

            // Signal Reversal — DISABLED by default
            enable_signal_reversal: false,
            reversal_min_confidence: 0.75,
            reversal_max_pnl_pct: 5.0,
            reversal_allowed_regimes: vec!["trending".to_string()],

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

        if self.min_signal_confidence < 0.0 || self.min_signal_confidence > 1.0 {
            errors.push("min_signal_confidence must be between 0.0 and 1.0".to_string());
        }

        if self.correlation_limit < 0.0 || self.correlation_limit > 1.0 {
            errors.push("correlation_limit must be between 0.0 and 1.0".to_string());
        }

        if self.max_portfolio_risk_pct <= 0.0 || self.max_portfolio_risk_pct > 100.0 {
            errors.push("max_portfolio_risk_pct must be between 0 and 100".to_string());
        }

        if self.short_only_mode && self.long_only_mode {
            errors.push("short_only_mode and long_only_mode cannot both be true".to_string());
        }

        // ATR-based sizing validation (only when enabled)
        if self.atr_stop_enabled {
            if self.atr_period < 2 || self.atr_period > 100 {
                errors.push("atr_period must be between 2 and 100".to_string());
            }
            if self.atr_stop_multiplier <= 0.0 || self.atr_stop_multiplier > 10.0 {
                errors.push("atr_stop_multiplier must be between 0 and 10".to_string());
            }
            if self.atr_tp_multiplier <= 0.0 || self.atr_tp_multiplier > 20.0 {
                errors.push("atr_tp_multiplier must be between 0 and 20".to_string());
            }
            if self.base_risk_pct <= 0.0 || self.base_risk_pct > 20.0 {
                errors.push("base_risk_pct must be between 0 and 20".to_string());
            }
        }

        // Kelly criterion validation (only when enabled)
        if self.kelly_enabled {
            if self.kelly_min_trades < 10 {
                errors.push("kelly_min_trades must be at least 10".to_string());
            }
            if self.kelly_fraction <= 0.0 || self.kelly_fraction > 1.0 {
                errors.push("kelly_fraction must be between 0 and 1".to_string());
            }
            if self.kelly_lookback < 10 || self.kelly_lookback > 1000 {
                errors.push("kelly_lookback must be between 10 and 1000".to_string());
            }
        }

        // Regime filter validation (only when enabled)
        if self.funding_spike_filter_enabled {
            if self.funding_spike_threshold <= 0.0 {
                errors.push("funding_spike_threshold must be positive".to_string());
            }
            if self.funding_spike_reduction < 0.0 || self.funding_spike_reduction > 1.0 {
                errors.push("funding_spike_reduction must be between 0 and 1".to_string());
            }
        }
        if self.atr_spike_filter_enabled {
            if self.atr_spike_multiplier <= 1.0 {
                errors.push("atr_spike_multiplier must be greater than 1".to_string());
            }
            if self.atr_spike_reduction < 0.0 || self.atr_spike_reduction > 1.0 {
                errors.push("atr_spike_reduction must be between 0 and 1".to_string());
            }
        }
        if self.consecutive_loss_reduction_enabled {
            if self.consecutive_loss_reduction_pct < 0.0
                || self.consecutive_loss_reduction_pct > 1.0
            {
                errors.push("consecutive_loss_reduction_pct must be between 0 and 1".to_string());
            }
            if self.consecutive_loss_reduction_threshold == 0 {
                errors.push("consecutive_loss_reduction_threshold must be at least 1".to_string());
            }
        }

        // Signal reversal validation (only when enabled)
        if self.enable_signal_reversal {
            if self.reversal_min_confidence < 0.0 || self.reversal_min_confidence > 1.0 {
                errors.push("reversal_min_confidence must be between 0 and 1".to_string());
            }
            if self.reversal_max_pnl_pct <= 0.0 {
                errors.push("reversal_max_pnl_pct must be positive".to_string());
            }
        }

        if errors.is_empty() {
            Ok(())
        } else {
            Err(errors)
        }
    }

    /// Check if trading type is futures
    pub fn is_futures(&self) -> bool {
        self.trading_type.to_lowercase() == "futures"
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
        // PnL-based SL adjusted for leverage (matching paper trading)
        // With leverage, price_change = pnl_pct / leverage
        let lev = self.max_leverage.max(1) as f64;
        if is_long {
            entry_price * (1.0 - self.default_stop_loss_percent / (lev * 100.0))
        } else {
            entry_price * (1.0 + self.default_stop_loss_percent / (lev * 100.0))
        }
    }

    /// Calculate take profit price from entry
    pub fn calculate_take_profit(&self, entry_price: f64, is_long: bool) -> f64 {
        // PnL-based TP adjusted for leverage (matching paper trading)
        let lev = self.max_leverage.max(1) as f64;
        if is_long {
            entry_price * (1.0 + self.default_take_profit_percent / (lev * 100.0))
        } else {
            entry_price * (1.0 - self.default_take_profit_percent / (lev * 100.0))
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

    // ============ ATR Validation Branch Coverage ============

    #[test]
    fn test_validate_atr_enabled_invalid_period_too_low() {
        let mut config = RealTradingConfig::default();
        config.atr_stop_enabled = true;
        config.atr_period = 1; // < 2
        let result = config.validate();
        assert!(result.is_err());
        let errs = result.unwrap_err();
        assert!(errs.iter().any(|e| e.contains("atr_period")));
    }

    #[test]
    fn test_validate_atr_enabled_invalid_period_too_high() {
        let mut config = RealTradingConfig::default();
        config.atr_stop_enabled = true;
        config.atr_period = 101; // > 100
        let result = config.validate();
        assert!(result.is_err());
        let errs = result.unwrap_err();
        assert!(errs.iter().any(|e| e.contains("atr_period")));
    }

    #[test]
    fn test_validate_atr_enabled_invalid_stop_multiplier_zero() {
        let mut config = RealTradingConfig::default();
        config.atr_stop_enabled = true;
        config.atr_stop_multiplier = 0.0; // <= 0
        let result = config.validate();
        assert!(result.is_err());
        let errs = result.unwrap_err();
        assert!(errs.iter().any(|e| e.contains("atr_stop_multiplier")));
    }

    #[test]
    fn test_validate_atr_enabled_invalid_stop_multiplier_too_high() {
        let mut config = RealTradingConfig::default();
        config.atr_stop_enabled = true;
        config.atr_stop_multiplier = 11.0; // > 10
        let result = config.validate();
        assert!(result.is_err());
        let errs = result.unwrap_err();
        assert!(errs.iter().any(|e| e.contains("atr_stop_multiplier")));
    }

    #[test]
    fn test_validate_atr_enabled_invalid_tp_multiplier_zero() {
        let mut config = RealTradingConfig::default();
        config.atr_stop_enabled = true;
        config.atr_tp_multiplier = 0.0; // <= 0
        let result = config.validate();
        assert!(result.is_err());
        let errs = result.unwrap_err();
        assert!(errs.iter().any(|e| e.contains("atr_tp_multiplier")));
    }

    #[test]
    fn test_validate_atr_enabled_invalid_tp_multiplier_too_high() {
        let mut config = RealTradingConfig::default();
        config.atr_stop_enabled = true;
        config.atr_tp_multiplier = 21.0; // > 20
        let result = config.validate();
        assert!(result.is_err());
        let errs = result.unwrap_err();
        assert!(errs.iter().any(|e| e.contains("atr_tp_multiplier")));
    }

    #[test]
    fn test_validate_atr_enabled_invalid_base_risk_pct_zero() {
        let mut config = RealTradingConfig::default();
        config.atr_stop_enabled = true;
        config.base_risk_pct = 0.0; // <= 0
        let result = config.validate();
        assert!(result.is_err());
        let errs = result.unwrap_err();
        assert!(errs.iter().any(|e| e.contains("base_risk_pct")));
    }

    #[test]
    fn test_validate_atr_enabled_invalid_base_risk_pct_too_high() {
        let mut config = RealTradingConfig::default();
        config.atr_stop_enabled = true;
        config.base_risk_pct = 21.0; // > 20
        let result = config.validate();
        assert!(result.is_err());
        let errs = result.unwrap_err();
        assert!(errs.iter().any(|e| e.contains("base_risk_pct")));
    }

    #[test]
    fn test_validate_atr_enabled_valid() {
        let mut config = RealTradingConfig::default();
        config.atr_stop_enabled = true;
        config.atr_period = 14; // valid
        config.atr_stop_multiplier = 1.5; // valid
        config.atr_tp_multiplier = 3.0; // valid
        config.base_risk_pct = 2.0; // valid
        let result = config.validate();
        assert!(result.is_ok());
    }

    // ============ Kelly Validation Branch Coverage ============

    #[test]
    fn test_validate_kelly_enabled_min_trades_too_low() {
        let mut config = RealTradingConfig::default();
        config.kelly_enabled = true;
        config.kelly_min_trades = 9; // < 10
        let result = config.validate();
        assert!(result.is_err());
        let errs = result.unwrap_err();
        assert!(errs.iter().any(|e| e.contains("kelly_min_trades")));
    }

    #[test]
    fn test_validate_kelly_enabled_fraction_zero() {
        let mut config = RealTradingConfig::default();
        config.kelly_enabled = true;
        config.kelly_fraction = 0.0; // <= 0
        let result = config.validate();
        assert!(result.is_err());
        let errs = result.unwrap_err();
        assert!(errs.iter().any(|e| e.contains("kelly_fraction")));
    }

    #[test]
    fn test_validate_kelly_enabled_fraction_above_one() {
        let mut config = RealTradingConfig::default();
        config.kelly_enabled = true;
        config.kelly_fraction = 1.5; // > 1
        let result = config.validate();
        assert!(result.is_err());
        let errs = result.unwrap_err();
        assert!(errs.iter().any(|e| e.contains("kelly_fraction")));
    }

    #[test]
    fn test_validate_kelly_enabled_lookback_too_low() {
        let mut config = RealTradingConfig::default();
        config.kelly_enabled = true;
        config.kelly_lookback = 9; // < 10
        let result = config.validate();
        assert!(result.is_err());
        let errs = result.unwrap_err();
        assert!(errs.iter().any(|e| e.contains("kelly_lookback")));
    }

    #[test]
    fn test_validate_kelly_enabled_lookback_too_high() {
        let mut config = RealTradingConfig::default();
        config.kelly_enabled = true;
        config.kelly_lookback = 1001; // > 1000
        let result = config.validate();
        assert!(result.is_err());
        let errs = result.unwrap_err();
        assert!(errs.iter().any(|e| e.contains("kelly_lookback")));
    }

    #[test]
    fn test_validate_kelly_enabled_valid() {
        let mut config = RealTradingConfig::default();
        config.kelly_enabled = true;
        config.kelly_min_trades = 50; // valid
        config.kelly_fraction = 0.5; // valid
        config.kelly_lookback = 100; // valid
        let result = config.validate();
        assert!(result.is_ok());
    }

    // ============ Regime Filter Validation Branch Coverage ============

    #[test]
    fn test_validate_funding_spike_filter_threshold_zero() {
        let mut config = RealTradingConfig::default();
        config.funding_spike_filter_enabled = true;
        config.funding_spike_threshold = 0.0; // <= 0
        let result = config.validate();
        assert!(result.is_err());
        let errs = result.unwrap_err();
        assert!(errs.iter().any(|e| e.contains("funding_spike_threshold")));
    }

    #[test]
    fn test_validate_funding_spike_filter_reduction_negative() {
        let mut config = RealTradingConfig::default();
        config.funding_spike_filter_enabled = true;
        config.funding_spike_reduction = -0.1; // < 0
        let result = config.validate();
        assert!(result.is_err());
        let errs = result.unwrap_err();
        assert!(errs.iter().any(|e| e.contains("funding_spike_reduction")));
    }

    #[test]
    fn test_validate_funding_spike_filter_reduction_above_one() {
        let mut config = RealTradingConfig::default();
        config.funding_spike_filter_enabled = true;
        config.funding_spike_reduction = 1.5; // > 1
        let result = config.validate();
        assert!(result.is_err());
        let errs = result.unwrap_err();
        assert!(errs.iter().any(|e| e.contains("funding_spike_reduction")));
    }

    #[test]
    fn test_validate_funding_spike_filter_valid() {
        let mut config = RealTradingConfig::default();
        config.funding_spike_filter_enabled = true;
        config.funding_spike_threshold = 0.001; // valid
        config.funding_spike_reduction = 0.5; // valid
        let result = config.validate();
        assert!(result.is_ok());
    }

    #[test]
    fn test_validate_atr_spike_filter_multiplier_too_low() {
        let mut config = RealTradingConfig::default();
        config.atr_spike_filter_enabled = true;
        config.atr_spike_multiplier = 1.0; // <= 1 (must be > 1)
        let result = config.validate();
        assert!(result.is_err());
        let errs = result.unwrap_err();
        assert!(errs.iter().any(|e| e.contains("atr_spike_multiplier")));
    }

    #[test]
    fn test_validate_atr_spike_filter_reduction_negative() {
        let mut config = RealTradingConfig::default();
        config.atr_spike_filter_enabled = true;
        config.atr_spike_reduction = -0.5; // < 0
        let result = config.validate();
        assert!(result.is_err());
        let errs = result.unwrap_err();
        assert!(errs.iter().any(|e| e.contains("atr_spike_reduction")));
    }

    #[test]
    fn test_validate_atr_spike_filter_reduction_above_one() {
        let mut config = RealTradingConfig::default();
        config.atr_spike_filter_enabled = true;
        config.atr_spike_reduction = 1.5; // > 1
        let result = config.validate();
        assert!(result.is_err());
        let errs = result.unwrap_err();
        assert!(errs.iter().any(|e| e.contains("atr_spike_reduction")));
    }

    #[test]
    fn test_validate_atr_spike_filter_valid() {
        let mut config = RealTradingConfig::default();
        config.atr_spike_filter_enabled = true;
        config.atr_spike_multiplier = 2.0; // > 1, valid
        config.atr_spike_reduction = 0.5; // valid
        let result = config.validate();
        assert!(result.is_ok());
    }

    #[test]
    fn test_validate_consecutive_loss_reduction_pct_negative() {
        let mut config = RealTradingConfig::default();
        config.consecutive_loss_reduction_enabled = true;
        config.consecutive_loss_reduction_pct = -0.1; // < 0
        let result = config.validate();
        assert!(result.is_err());
        let errs = result.unwrap_err();
        assert!(errs
            .iter()
            .any(|e| e.contains("consecutive_loss_reduction_pct")));
    }

    #[test]
    fn test_validate_consecutive_loss_reduction_pct_above_one() {
        let mut config = RealTradingConfig::default();
        config.consecutive_loss_reduction_enabled = true;
        config.consecutive_loss_reduction_pct = 1.5; // > 1
        let result = config.validate();
        assert!(result.is_err());
        let errs = result.unwrap_err();
        assert!(errs
            .iter()
            .any(|e| e.contains("consecutive_loss_reduction_pct")));
    }

    #[test]
    fn test_validate_consecutive_loss_reduction_threshold_zero() {
        let mut config = RealTradingConfig::default();
        config.consecutive_loss_reduction_enabled = true;
        config.consecutive_loss_reduction_threshold = 0; // must be >= 1
        let result = config.validate();
        assert!(result.is_err());
        let errs = result.unwrap_err();
        assert!(errs
            .iter()
            .any(|e| e.contains("consecutive_loss_reduction_threshold")));
    }

    #[test]
    fn test_validate_consecutive_loss_reduction_valid() {
        let mut config = RealTradingConfig::default();
        config.consecutive_loss_reduction_enabled = true;
        config.consecutive_loss_reduction_pct = 0.3; // valid
        config.consecutive_loss_reduction_threshold = 3; // valid
        let result = config.validate();
        assert!(result.is_ok());
    }

    // ============ Signal Reversal Validation Branch Coverage ============

    #[test]
    fn test_validate_signal_reversal_confidence_negative() {
        let mut config = RealTradingConfig::default();
        config.enable_signal_reversal = true;
        config.reversal_min_confidence = -0.1; // < 0
        let result = config.validate();
        assert!(result.is_err());
        let errs = result.unwrap_err();
        assert!(errs.iter().any(|e| e.contains("reversal_min_confidence")));
    }

    #[test]
    fn test_validate_signal_reversal_confidence_above_one() {
        let mut config = RealTradingConfig::default();
        config.enable_signal_reversal = true;
        config.reversal_min_confidence = 1.5; // > 1
        let result = config.validate();
        assert!(result.is_err());
        let errs = result.unwrap_err();
        assert!(errs.iter().any(|e| e.contains("reversal_min_confidence")));
    }

    #[test]
    fn test_validate_signal_reversal_max_pnl_pct_zero() {
        let mut config = RealTradingConfig::default();
        config.enable_signal_reversal = true;
        config.reversal_max_pnl_pct = 0.0; // <= 0
        let result = config.validate();
        assert!(result.is_err());
        let errs = result.unwrap_err();
        assert!(errs.iter().any(|e| e.contains("reversal_max_pnl_pct")));
    }

    #[test]
    fn test_validate_signal_reversal_valid() {
        let mut config = RealTradingConfig::default();
        config.enable_signal_reversal = true;
        config.reversal_min_confidence = 0.75; // valid
        config.reversal_max_pnl_pct = 5.0; // valid
        let result = config.validate();
        assert!(result.is_ok());
    }

    // ============ Short/Long Only Conflict ============

    #[test]
    fn test_validate_short_only_and_long_only_conflict() {
        let mut config = RealTradingConfig::default();
        config.short_only_mode = true;
        config.long_only_mode = true;
        let result = config.validate();
        assert!(result.is_err());
        let errs = result.unwrap_err();
        assert!(errs.iter().any(|e| e.contains("short_only_mode")));
    }

    // ============ is_futures ============

    #[test]
    fn test_is_futures_spot() {
        let config = RealTradingConfig {
            trading_type: "spot".to_string(),
            ..Default::default()
        };
        assert!(!config.is_futures());
    }

    #[test]
    fn test_is_futures_futures() {
        let config = RealTradingConfig {
            trading_type: "futures".to_string(),
            ..Default::default()
        };
        assert!(config.is_futures());
    }

    #[test]
    fn test_is_futures_uppercase() {
        let config = RealTradingConfig {
            trading_type: "FUTURES".to_string(),
            ..Default::default()
        };
        assert!(config.is_futures());
    }
}
