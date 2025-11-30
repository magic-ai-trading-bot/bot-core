use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Complete paper trading configuration
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct PaperTradingSettings {
    /// Basic trading parameters
    pub basic: BasicSettings,

    /// Risk management settings
    pub risk: RiskSettings,

    /// Strategy configuration
    pub strategy: StrategySettings,

    /// Symbol-specific settings
    pub symbols: HashMap<String, SymbolSettings>,

    /// AI integration settings
    pub ai: AISettings,

    /// Execution settings
    pub execution: ExecutionSettings,

    /// Notification settings
    pub notifications: NotificationSettings,

    /// Technical indicator configuration (shared with Python AI service)
    /// @spec:FR-SETTINGS-001 - Unified indicator settings across services
    #[serde(default)]
    pub indicators: IndicatorSettings,

    /// Signal generation thresholds (shared with Python AI service)
    /// @spec:FR-SETTINGS-002 - Unified signal generation settings
    #[serde(default)]
    pub signal: SignalGenerationSettings,
}

/// Technical indicator configuration
/// These settings are shared between Rust trading engine and Python AI service
/// to ensure consistent indicator calculations across the system.
/// @spec:FR-SETTINGS-001 - Unified indicator configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IndicatorSettings {
    /// RSI (Relative Strength Index) period (default: 14)
    /// Valid range: 5-50
    pub rsi_period: u32,

    /// MACD fast EMA period (default: 12)
    /// Must be less than macd_slow
    pub macd_fast: u32,

    /// MACD slow EMA period (default: 26)
    /// Must be greater than macd_fast
    pub macd_slow: u32,

    /// MACD signal line period (default: 9)
    pub macd_signal: u32,

    /// EMA periods for trend analysis (default: [9, 21, 50])
    pub ema_periods: Vec<u32>,

    /// Bollinger Bands period (default: 20)
    pub bollinger_period: u32,

    /// Bollinger Bands standard deviation multiplier (default: 2.0)
    /// Valid range: 1.0-4.0
    pub bollinger_std: f64,

    /// Volume SMA period for volume analysis (default: 20)
    pub volume_sma_period: u32,

    /// Stochastic %K period (default: 14)
    pub stochastic_k_period: u32,

    /// Stochastic %D period (smoothing, default: 3)
    pub stochastic_d_period: u32,
}

/// Signal generation thresholds
/// These settings control how AI signals are generated and when to trade.
/// Shared between Rust trading engine and Python AI service.
/// @spec:FR-SETTINGS-002 - Signal generation configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SignalGenerationSettings {
    /// Trend threshold percentage (default: 0.8%)
    /// Price movement must exceed this % to qualify as bullish/bearish trend
    /// Lower = more signals (aggressive), Higher = fewer signals (conservative)
    /// Valid range: 0.1-10.0%
    pub trend_threshold_percent: f64,

    /// Minimum timeframes required to agree (default: 3 out of 4)
    /// Timeframes: 15M, 30M, 1H, 4H
    /// 2 = 50% agreement (aggressive), 3 = 75% (balanced), 4 = 100% (conservative)
    /// Valid range: 1-4
    pub min_required_timeframes: u32,

    /// Minimum indicators per timeframe (default: 4 out of 5)
    /// Indicators: MACD, RSI, Bollinger Bands, Stochastic, Volume
    /// 3 = 60% agreement (aggressive), 4 = 80% (balanced), 5 = 100% (conservative)
    /// Valid range: 1-5
    pub min_required_indicators: u32,

    /// Base confidence when signal is triggered (default: 0.5)
    /// This is the starting confidence level
    /// Valid range: 0.1-0.9
    pub confidence_base: f64,

    /// Confidence added per agreeing timeframe (default: 0.08)
    /// Max confidence with 4 timeframes = base + (4 * per_tf) = ~0.82
    /// Valid range: 0.01-0.2
    pub confidence_per_timeframe: f64,
}

impl Default for IndicatorSettings {
    fn default() -> Self {
        Self {
            rsi_period: 14,
            macd_fast: 12,
            macd_slow: 26,
            macd_signal: 9,
            ema_periods: vec![9, 21, 50],
            bollinger_period: 20,
            bollinger_std: 2.0,
            volume_sma_period: 20,
            stochastic_k_period: 14,
            stochastic_d_period: 3,
        }
    }
}

impl Default for SignalGenerationSettings {
    fn default() -> Self {
        Self {
            trend_threshold_percent: 0.8,
            min_required_timeframes: 3,
            min_required_indicators: 4,
            confidence_base: 0.5,
            confidence_per_timeframe: 0.08,
        }
    }
}

/// Basic trading settings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BasicSettings {
    /// Starting balance in USDT
    pub initial_balance: f64,

    /// Maximum number of concurrent positions
    pub max_positions: u32,

    /// Default position size as percentage of balance
    pub default_position_size_pct: f64,

    /// Default leverage (1-125)
    pub default_leverage: u8,

    /// Trading fee rate (e.g., 0.0004 for 0.04%)
    pub trading_fee_rate: f64,

    /// Funding fee rate (Binance Futures)
    pub funding_fee_rate: f64,

    /// Slippage simulation percentage
    pub slippage_pct: f64,

    /// Enable/disable paper trading
    pub enabled: bool,

    /// Auto-restart after reset
    pub auto_restart: bool,
}

/// Risk management settings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RiskSettings {
    /// Maximum risk per trade (percentage of balance)
    pub max_risk_per_trade_pct: f64,

    /// Maximum total portfolio risk (percentage)
    pub max_portfolio_risk_pct: f64,

    /// Default stop loss percentage
    pub default_stop_loss_pct: f64,

    /// Default take profit percentage
    pub default_take_profit_pct: f64,

    /// Maximum leverage allowed
    pub max_leverage: u8,

    /// Minimum margin level before liquidation warning
    pub min_margin_level: f64,

    /// Maximum drawdown before auto-stop (percentage)
    pub max_drawdown_pct: f64,

    /// Daily loss limit (percentage of balance)
    pub daily_loss_limit_pct: f64,

    /// Maximum consecutive losses before pause
    pub max_consecutive_losses: u32,

    /// Cool-down period after consecutive losses (minutes)
    pub cool_down_minutes: u32,

    /// Position sizing method
    pub position_sizing_method: PositionSizingMethod,

    /// Risk-reward ratio requirement
    pub min_risk_reward_ratio: f64,

    /// Correlation limit (max positions in correlated assets)
    pub correlation_limit: f64,

    /// Enable dynamic position sizing based on volatility
    pub dynamic_sizing: bool,

    /// Volatility lookback period for dynamic sizing
    pub volatility_lookback_hours: u32,

    /// Enable trailing stop-loss
    pub trailing_stop_enabled: bool,

    /// Trailing stop distance (percentage below high for long, above low for short)
    pub trailing_stop_pct: f64,

    /// Minimum profit before trailing activates (percentage)
    pub trailing_activation_pct: f64,

    /// Enable automatic position reversal on opposite signals
    pub enable_signal_reversal: bool,

    /// Let AI automatically decide when to enable/disable reversal based on conditions
    /// When true, AI analyzes: accuracy history, market regime, win rate, volatility
    /// Overrides enable_signal_reversal setting when conditions are favorable
    pub ai_auto_enable_reversal: bool,

    /// Minimum AI confidence required for signal reversal (0.0 - 1.0)
    pub reversal_min_confidence: f64,

    /// Maximum position P&L percentage before disabling reversal (use trailing stop instead)
    pub reversal_max_pnl_pct: f64,

    /// Allowed market regimes for reversal (e.g., ["trending"])
    pub reversal_allowed_regimes: Vec<String>,
}

/// Strategy configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StrategySettings {
    /// Enabled strategies with weights
    pub enabled_strategies: HashMap<String, f64>,

    /// Minimum AI confidence required for trade execution
    pub min_ai_confidence: f64,

    /// Strategy combination method
    pub combination_method: StrategyCombinationMethod,

    /// Enable strategy optimization
    pub enable_optimization: bool,

    /// Optimization period (days)
    pub optimization_period_days: u32,

    /// Minimum number of trades for optimization
    pub min_trades_for_optimization: u32,

    /// Strategy timeout (minutes) - cancel signal if too old
    pub signal_timeout_minutes: u32,

    /// Enable market regime detection
    pub enable_market_regime_detection: bool,

    /// Strategy parameters per market regime
    pub regime_specific_params: HashMap<String, HashMap<String, serde_json::Value>>,

    /// Backtesting settings
    pub backtesting: BacktestingSettings,

    /// Selected market preset (low_volatility, normal_volatility, high_volatility)
    /// Used by frontend to track which preset is selected
    #[serde(default = "default_market_preset")]
    pub market_preset: String,
}

fn default_market_preset() -> String {
    "normal_volatility".to_string()
}

/// AI integration settings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AISettings {
    /// Python AI service URL
    pub service_url: String,

    /// Request timeout (seconds)
    pub request_timeout_seconds: u32,

    /// Signal refresh interval (minutes)
    pub signal_refresh_interval_minutes: u32,

    /// Enable real-time signal updates
    pub enable_realtime_signals: bool,

    /// Confidence threshold for different market conditions
    pub confidence_thresholds: HashMap<String, f64>,

    /// Enable AI feedback learning
    pub enable_feedback_learning: bool,

    /// Feedback delay (hours) - time to wait before sending feedback
    pub feedback_delay_hours: u32,

    /// Enable strategy recommendations
    pub enable_strategy_recommendations: bool,

    /// Model performance tracking
    pub track_model_performance: bool,
}

/// Symbol-specific settings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SymbolSettings {
    /// Enable trading for this symbol
    pub enabled: bool,

    /// Symbol-specific leverage
    pub leverage: Option<u8>,

    /// Symbol-specific position size
    pub position_size_pct: Option<f64>,

    /// Symbol-specific stop loss
    pub stop_loss_pct: Option<f64>,

    /// Symbol-specific take profit
    pub take_profit_pct: Option<f64>,

    /// Trading session hours (UTC)
    pub trading_hours: Option<TradingHours>,

    /// Minimum price movement for trade entry
    pub min_price_movement_pct: Option<f64>,

    /// Maximum number of positions for this symbol
    pub max_positions: Option<u32>,

    /// Custom parameters for this symbol
    pub custom_params: HashMap<String, serde_json::Value>,
}

/// Execution settings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionSettings {
    /// Enable automatic trade execution
    pub auto_execution: bool,

    /// Execution delay simulation (milliseconds)
    pub execution_delay_ms: u32,

    /// Enable partial fills simulation
    pub simulate_partial_fills: bool,

    /// Partial fill probability (0.0 - 1.0)
    pub partial_fill_probability: f64,

    /// Order expiration time (minutes)
    pub order_expiration_minutes: u32,

    /// Enable slippage simulation
    pub simulate_slippage: bool,

    /// Maximum slippage percentage
    pub max_slippage_pct: f64,

    /// Enable market impact simulation
    pub simulate_market_impact: bool,

    /// Market impact factor
    pub market_impact_factor: f64,

    /// Price update frequency (seconds)
    pub price_update_frequency_seconds: u32,
}

/// Notification settings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NotificationSettings {
    /// Enable trade notifications
    pub enable_trade_notifications: bool,

    /// Enable performance notifications
    pub enable_performance_notifications: bool,

    /// Enable risk warnings
    pub enable_risk_warnings: bool,

    /// Daily summary notifications
    pub daily_summary: bool,

    /// Weekly performance report
    pub weekly_report: bool,

    /// Notification channels
    pub channels: Vec<NotificationChannel>,

    /// Minimum P&L for notification (absolute value)
    pub min_pnl_notification: f64,

    /// Notification frequency limits
    pub max_notifications_per_hour: u32,
}

/// Trading session hours
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TradingHours {
    pub start_hour: u8,
    pub start_minute: u8,
    pub end_hour: u8,
    pub end_minute: u8,
    pub timezone: String,
}

/// Backtesting configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BacktestingSettings {
    /// Enable automatic backtesting
    pub enabled: bool,

    /// Backtesting period (days)
    pub period_days: u32,

    /// Data resolution for backtesting
    pub data_resolution: String,

    /// Minimum number of trades required
    pub min_trades: u32,

    /// Enable walk-forward optimization
    pub walk_forward_optimization: bool,

    /// Out-of-sample percentage for validation
    pub out_of_sample_pct: f64,
}

/// Position sizing methods
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PositionSizingMethod {
    /// Fixed percentage of balance
    FixedPercentage,
    /// Based on risk amount (Kelly criterion inspired)
    RiskBased,
    /// Based on volatility (inverse volatility)
    VolatilityAdjusted,
    /// Based on AI confidence
    ConfidenceWeighted,
    /// Combination of multiple factors
    Composite,
}

/// Strategy combination methods
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum StrategyCombinationMethod {
    /// Weighted average of all strategies
    WeightedAverage,
    /// Only execute if majority agrees
    MajorityVoting,
    /// Only execute if all strategies agree
    Unanimous,
    /// Use the most confident strategy
    HighestConfidence,
    /// Use AI to combine strategies
    AIEnsemble,
}

/// Notification channels
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum NotificationChannel {
    WebSocket,
    Email(String),
    Telegram(String),
    Discord(String),
    Webhook(String),
}

impl Default for BasicSettings {
    fn default() -> Self {
        Self {
            initial_balance: 10000.0,
            max_positions: 5,               // OPTIMIZED: Down from 10 - better focus
            default_position_size_pct: 2.0, // OPTIMIZED: Down from 5% - conservative sizing
            default_leverage: 3,            // OPTIMIZED: Down from 10x - CRITICAL CHANGE!
            trading_fee_rate: 0.0004,       // 0.04% Binance Futures
            funding_fee_rate: 0.0001,       // 0.01% every 8 hours
            slippage_pct: 0.01,             // 0.01% average slippage
            enabled: true,
            auto_restart: false,
        }
    }
}

impl Default for RiskSettings {
    fn default() -> Self {
        Self {
            max_risk_per_trade_pct: 1.0, // OPTIMIZED: Down from 2% - max 1% loss/trade
            max_portfolio_risk_pct: 10.0, // OPTIMIZED: Down from 20% - safer limit
            default_stop_loss_pct: 5.0,  // OPTIMIZED: Up from 2% - avoid market noise!
            default_take_profit_pct: 10.0, // OPTIMIZED: Up from 4% - better R:R (2:1)
            max_leverage: 5,             // OPTIMIZED: Down from 50x - safety cap
            min_margin_level: 300.0,     // OPTIMIZED: Up from 200% - extra buffer
            max_drawdown_pct: 10.0,      // OPTIMIZED: Down from 15% - stop earlier
            daily_loss_limit_pct: 3.0,   // OPTIMIZED: Down from 5% - protect capital
            max_consecutive_losses: 3,   // OPTIMIZED: Down from 5 - stop faster
            // @spec:FR-RISK-006 - Cool-down period 60 minutes after consecutive losses
            // @ref:docs/features/how-it-works.md - Layer 6: "Nghỉ 60 phút sau thua lỗ"
            cool_down_minutes: 60, // FIXED: Match docs - 60 minutes cool-down
            position_sizing_method: PositionSizingMethod::RiskBased,
            min_risk_reward_ratio: 2.0, // OPTIMIZED: Up from 1.5 - quality trades only
            correlation_limit: 0.7,
            dynamic_sizing: true,
            volatility_lookback_hours: 24,
            trailing_stop_enabled: true,   // NEW: Enable trailing stops
            trailing_stop_pct: 3.0,        // NEW: Trail 3% below high/above low
            trailing_activation_pct: 5.0,  // NEW: Start after 5% profit
            enable_signal_reversal: true,  // ENABLED: Auto-close positions on reversal signals
            ai_auto_enable_reversal: true, // Let AI decide automatically ✨
            reversal_min_confidence: 0.65, // LOWERED: 65% minimum confidence (was 75%)
            reversal_max_pnl_pct: 10.0,    // NEW: 10% max profit before using trailing stop
            reversal_allowed_regimes: vec![
                "trending".to_string(),
                "ranging".to_string(),
                "volatile".to_string(),
            ], // Allow reversal in ALL market conditions
        }
    }
}

impl Default for StrategySettings {
    fn default() -> Self {
        let mut enabled_strategies = HashMap::new();
        enabled_strategies.insert("ai_ensemble".to_string(), 1.0);

        Self {
            enabled_strategies,
            min_ai_confidence: 0.5, // Lowered for testnet to get more trading activity
            combination_method: StrategyCombinationMethod::AIEnsemble,
            enable_optimization: true,
            optimization_period_days: 30,
            min_trades_for_optimization: 50,
            signal_timeout_minutes: 30,
            enable_market_regime_detection: true,
            regime_specific_params: HashMap::new(),
            backtesting: BacktestingSettings::default(),
            market_preset: "normal_volatility".to_string(),
        }
    }
}

impl Default for AISettings {
    fn default() -> Self {
        let mut confidence_thresholds = HashMap::new();
        confidence_thresholds.insert("trending".to_string(), 0.65);
        confidence_thresholds.insert("ranging".to_string(), 0.75);
        confidence_thresholds.insert("volatile".to_string(), 0.80);

        Self {
            service_url: "http://python-ai-service:8000".to_string(),
            request_timeout_seconds: 30,
            signal_refresh_interval_minutes: 15, // 15 minutes - Optimized for crypto day trading with 15m charts
            enable_realtime_signals: true,
            confidence_thresholds,
            enable_feedback_learning: true,
            feedback_delay_hours: 4,
            enable_strategy_recommendations: true,
            track_model_performance: true,
        }
    }
}

impl Default for ExecutionSettings {
    fn default() -> Self {
        Self {
            auto_execution: true,
            execution_delay_ms: 100,
            simulate_partial_fills: false,
            partial_fill_probability: 0.1,
            order_expiration_minutes: 60,
            simulate_slippage: true,
            max_slippage_pct: 0.05,
            simulate_market_impact: false,
            market_impact_factor: 0.001,
            price_update_frequency_seconds: 1,
        }
    }
}

impl Default for NotificationSettings {
    fn default() -> Self {
        Self {
            enable_trade_notifications: true,
            enable_performance_notifications: true,
            enable_risk_warnings: true,
            daily_summary: true,
            weekly_report: true,
            channels: vec![NotificationChannel::WebSocket],
            min_pnl_notification: 10.0,
            max_notifications_per_hour: 20,
        }
    }
}

impl Default for BacktestingSettings {
    fn default() -> Self {
        Self {
            enabled: true,
            period_days: 90,
            data_resolution: "15m".to_string(), // Changed from 1h to 15m for better crypto day trading
            min_trades: 20,
            walk_forward_optimization: false,
            out_of_sample_pct: 20.0,
        }
    }
}

impl PaperTradingSettings {
    /// Load settings from configuration file
    pub fn from_file(path: &str) -> Result<Self> {
        let content = std::fs::read_to_string(path)?;
        let settings: Self = toml::from_str(&content)?;
        Ok(settings)
    }

    /// Save settings to configuration file
    pub fn to_file(&self, path: &str) -> Result<()> {
        let content = toml::to_string_pretty(self)?;
        std::fs::write(path, content)?;
        Ok(())
    }

    /// Validate settings
    pub fn validate(&self) -> Result<()> {
        // Validate basic settings
        if self.basic.initial_balance <= 0.0 {
            return Err(anyhow::anyhow!("Initial balance must be positive"));
        }

        if self.basic.default_leverage == 0 || self.basic.default_leverage > 125 {
            return Err(anyhow::anyhow!(
                "Default leverage must be between 1 and 125"
            ));
        }

        if self.basic.trading_fee_rate < 0.0 || self.basic.trading_fee_rate > 0.01 {
            return Err(anyhow::anyhow!("Trading fee rate must be between 0 and 1%"));
        }

        // Validate risk settings
        if self.risk.max_risk_per_trade_pct <= 0.0 || self.risk.max_risk_per_trade_pct > 50.0 {
            return Err(anyhow::anyhow!(
                "Max risk per trade must be between 0% and 50%"
            ));
        }

        if self.risk.max_portfolio_risk_pct <= 0.0 || self.risk.max_portfolio_risk_pct > 100.0 {
            return Err(anyhow::anyhow!(
                "Max portfolio risk must be between 0% and 100%"
            ));
        }

        if self.risk.max_leverage > 125 {
            return Err(anyhow::anyhow!("Max leverage cannot exceed 125"));
        }

        if self.risk.min_margin_level < 100.0 {
            return Err(anyhow::anyhow!("Min margin level must be at least 100%"));
        }

        // Validate strategy settings
        if !(0.0..=1.0).contains(&self.strategy.min_ai_confidence) {
            return Err(anyhow::anyhow!("AI confidence must be between 0 and 1"));
        }

        // Validate AI settings
        if self.ai.request_timeout_seconds == 0 {
            return Err(anyhow::anyhow!("Request timeout must be positive"));
        }

        if self.ai.signal_refresh_interval_minutes == 0 {
            return Err(anyhow::anyhow!("Signal refresh interval must be positive"));
        }

        // Validate indicator settings
        // @spec:FR-SETTINGS-001 - Indicator validation rules
        if self.indicators.rsi_period < 5 || self.indicators.rsi_period > 50 {
            return Err(anyhow::anyhow!(
                "RSI period must be between 5 and 50, got {}",
                self.indicators.rsi_period
            ));
        }

        if self.indicators.macd_fast >= self.indicators.macd_slow {
            return Err(anyhow::anyhow!(
                "MACD fast period ({}) must be less than slow period ({})",
                self.indicators.macd_fast,
                self.indicators.macd_slow
            ));
        }

        if self.indicators.macd_signal == 0 || self.indicators.macd_signal > 20 {
            return Err(anyhow::anyhow!(
                "MACD signal period must be between 1 and 20, got {}",
                self.indicators.macd_signal
            ));
        }

        if self.indicators.ema_periods.is_empty() {
            return Err(anyhow::anyhow!("EMA periods cannot be empty"));
        }

        if self.indicators.bollinger_period < 5 || self.indicators.bollinger_period > 50 {
            return Err(anyhow::anyhow!(
                "Bollinger period must be between 5 and 50, got {}",
                self.indicators.bollinger_period
            ));
        }

        if self.indicators.bollinger_std < 1.0 || self.indicators.bollinger_std > 4.0 {
            return Err(anyhow::anyhow!(
                "Bollinger std must be between 1.0 and 4.0, got {}",
                self.indicators.bollinger_std
            ));
        }

        if self.indicators.stochastic_k_period < 5 || self.indicators.stochastic_k_period > 30 {
            return Err(anyhow::anyhow!(
                "Stochastic K period must be between 5 and 30, got {}",
                self.indicators.stochastic_k_period
            ));
        }

        // Validate signal generation settings
        // @spec:FR-SETTINGS-002 - Signal generation validation rules
        if self.signal.trend_threshold_percent <= 0.0 || self.signal.trend_threshold_percent > 10.0
        {
            return Err(anyhow::anyhow!(
                "Trend threshold must be between 0.1% and 10.0%, got {}%",
                self.signal.trend_threshold_percent
            ));
        }

        if self.signal.min_required_timeframes == 0 || self.signal.min_required_timeframes > 4 {
            return Err(anyhow::anyhow!(
                "Min required timeframes must be between 1 and 4, got {}",
                self.signal.min_required_timeframes
            ));
        }

        if self.signal.min_required_indicators == 0 || self.signal.min_required_indicators > 5 {
            return Err(anyhow::anyhow!(
                "Min required indicators must be between 1 and 5, got {}",
                self.signal.min_required_indicators
            ));
        }

        if self.signal.confidence_base < 0.1 || self.signal.confidence_base > 0.9 {
            return Err(anyhow::anyhow!(
                "Confidence base must be between 0.1 and 0.9, got {}",
                self.signal.confidence_base
            ));
        }

        if self.signal.confidence_per_timeframe < 0.01 || self.signal.confidence_per_timeframe > 0.2
        {
            return Err(anyhow::anyhow!(
                "Confidence per timeframe must be between 0.01 and 0.2, got {}",
                self.signal.confidence_per_timeframe
            ));
        }

        Ok(())
    }

    /// Get effective settings for a specific symbol
    pub fn get_symbol_settings(&self, symbol: &str) -> EffectiveSymbolSettings {
        let symbol_specific = self.symbols.get(symbol);

        EffectiveSymbolSettings {
            enabled: symbol_specific.is_none_or(|s| s.enabled),
            leverage: symbol_specific
                .and_then(|s| s.leverage)
                .unwrap_or(self.basic.default_leverage),
            position_size_pct: symbol_specific
                .and_then(|s| s.position_size_pct)
                .unwrap_or(self.basic.default_position_size_pct),
            stop_loss_pct: symbol_specific
                .and_then(|s| s.stop_loss_pct)
                .unwrap_or(self.risk.default_stop_loss_pct),
            take_profit_pct: symbol_specific
                .and_then(|s| s.take_profit_pct)
                .unwrap_or(self.risk.default_take_profit_pct),
            max_positions: symbol_specific.and_then(|s| s.max_positions).unwrap_or(1),
        }
    }

    /// Update settings at runtime
    pub fn update_basic(&mut self, basic: BasicSettings) -> Result<()> {
        // Validate before updating
        if basic.initial_balance <= 0.0 {
            return Err(anyhow::anyhow!("Initial balance must be positive"));
        }

        self.basic = basic;
        Ok(())
    }

    /// Update risk settings
    pub fn update_risk(&mut self, risk: RiskSettings) -> Result<()> {
        if risk.max_risk_per_trade_pct <= 0.0 || risk.max_risk_per_trade_pct > 50.0 {
            return Err(anyhow::anyhow!("Invalid risk per trade percentage"));
        }

        self.risk = risk;
        Ok(())
    }

    /// Add or update symbol-specific settings
    pub fn set_symbol_settings(&mut self, symbol: String, settings: SymbolSettings) {
        self.symbols.insert(symbol, settings);
    }

    /// Remove symbol-specific settings
    pub fn remove_symbol_settings(&mut self, symbol: &str) {
        self.symbols.remove(symbol);
    }

    /// Get all configured symbols
    pub fn get_configured_symbols(&self) -> Vec<String> {
        self.symbols.keys().cloned().collect()
    }
}

/// Effective settings for a symbol (merged with defaults)
#[derive(Debug, Clone)]
pub struct EffectiveSymbolSettings {
    pub enabled: bool,
    pub leverage: u8,
    pub position_size_pct: f64,
    pub stop_loss_pct: f64,
    pub take_profit_pct: f64,
    pub max_positions: u32,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_basic_settings() {
        let settings = BasicSettings::default();

        assert_eq!(settings.initial_balance, 10000.0);
        assert_eq!(settings.max_positions, 5); // FIXED: Down from 10 - better focus
        assert_eq!(settings.default_position_size_pct, 2.0); // FIXED: Down from 5% - conservative
        assert_eq!(settings.default_leverage, 3); // FIXED: Down from 10x - CRITICAL!
        assert_eq!(settings.trading_fee_rate, 0.0004);
        assert!(settings.enabled);
    }

    #[test]
    fn test_default_risk_settings() {
        let settings = RiskSettings::default();

        assert_eq!(settings.max_risk_per_trade_pct, 1.0); // FIXED: Down from 2% - max 1% loss/trade
        assert_eq!(settings.max_portfolio_risk_pct, 10.0); // FIXED: Down from 20% - safer limit
        assert_eq!(settings.default_stop_loss_pct, 5.0); // FIXED: Up from 2% - avoid market noise!
        assert_eq!(settings.default_take_profit_pct, 10.0); // FIXED: Up from 4% - better R:R (2:1)
        assert_eq!(settings.max_leverage, 5); // FIXED: Down from 50x - safety cap
        assert_eq!(settings.min_margin_level, 300.0); // FIXED: Up from 200% - extra buffer
        assert_eq!(settings.max_consecutive_losses, 3); // FIXED: Down from 5 - stop faster

        // NEW: Signal reversal defaults - ENABLED for better trade management
        assert!(settings.enable_signal_reversal); // Enabled by default
        assert!(settings.ai_auto_enable_reversal); // AI auto-enable enabled
        assert_eq!(settings.reversal_min_confidence, 0.65); // 65% minimum (lowered)
        assert_eq!(settings.reversal_max_pnl_pct, 10.0); // 10% max P&L
        assert_eq!(
            settings.reversal_allowed_regimes,
            vec!["trending", "ranging", "volatile"]
        ); // All market conditions
    }

    #[test]
    fn test_default_strategy_settings() {
        let settings = StrategySettings::default();

        assert_eq!(settings.min_ai_confidence, 0.5); // FIXED: Lowered from 0.7 for testnet activity
        assert!(settings.enable_optimization);
        assert_eq!(settings.optimization_period_days, 30);
        assert_eq!(settings.min_trades_for_optimization, 50);
    }

    #[test]
    fn test_default_ai_settings() {
        let settings = AISettings::default();

        assert_eq!(settings.service_url, "http://python-ai-service:8000");
        assert_eq!(settings.request_timeout_seconds, 30);
        assert_eq!(settings.signal_refresh_interval_minutes, 15); // Updated to reflect new 15-minute default for crypto day trading
        assert!(settings.enable_realtime_signals);
        assert!(settings.enable_feedback_learning);
    }

    #[test]
    fn test_default_execution_settings() {
        let settings = ExecutionSettings::default();

        assert!(settings.auto_execution);
        assert_eq!(settings.execution_delay_ms, 100);
        assert!(settings.simulate_slippage);
        assert!(!settings.simulate_partial_fills);
    }

    #[test]
    fn test_validate_basic_settings() {
        let mut settings = PaperTradingSettings::default();

        // Valid settings should pass
        assert!(settings.validate().is_ok());

        // Invalid balance
        settings.basic.initial_balance = -100.0;
        assert!(settings.validate().is_err());

        settings.basic.initial_balance = 10000.0;

        // Invalid leverage
        settings.basic.default_leverage = 0;
        assert!(settings.validate().is_err());

        settings.basic.default_leverage = 150;
        assert!(settings.validate().is_err());

        settings.basic.default_leverage = 3; // FIXED: Reset to new default

        // Invalid fee rate
        settings.basic.trading_fee_rate = -0.01;
        assert!(settings.validate().is_err());

        settings.basic.trading_fee_rate = 0.02;
        assert!(settings.validate().is_err());
    }

    #[test]
    fn test_validate_risk_settings() {
        let mut settings = PaperTradingSettings::default();

        // Invalid max risk per trade
        settings.risk.max_risk_per_trade_pct = 0.0;
        assert!(settings.validate().is_err());

        settings.risk.max_risk_per_trade_pct = 60.0;
        assert!(settings.validate().is_err());

        settings.risk.max_risk_per_trade_pct = 1.0; // FIXED: Reset to new default

        // Invalid max portfolio risk
        settings.risk.max_portfolio_risk_pct = 0.0;
        assert!(settings.validate().is_err());

        settings.risk.max_portfolio_risk_pct = 150.0;
        assert!(settings.validate().is_err());

        settings.risk.max_portfolio_risk_pct = 10.0; // FIXED: Reset to new default

        // Invalid max leverage
        settings.risk.max_leverage = 130;
        assert!(settings.validate().is_err());

        settings.risk.max_leverage = 5; // FIXED: Reset to new default

        // Invalid min margin level
        settings.risk.min_margin_level = 50.0;
        assert!(settings.validate().is_err());
    }

    #[test]
    fn test_validate_strategy_settings() {
        let mut settings = PaperTradingSettings::default();

        // Invalid AI confidence
        settings.strategy.min_ai_confidence = -0.1;
        assert!(settings.validate().is_err());

        settings.strategy.min_ai_confidence = 1.5;
        assert!(settings.validate().is_err());

        settings.strategy.min_ai_confidence = 0.7;
        assert!(settings.validate().is_ok());
    }

    #[test]
    fn test_validate_ai_settings() {
        let mut settings = PaperTradingSettings::default();

        // Invalid timeout
        settings.ai.request_timeout_seconds = 0;
        assert!(settings.validate().is_err());

        settings.ai.request_timeout_seconds = 30;

        // Invalid refresh interval
        settings.ai.signal_refresh_interval_minutes = 0;
        assert!(settings.validate().is_err());
    }

    #[test]
    fn test_get_symbol_settings_default() {
        let settings = PaperTradingSettings::default();

        let effective = settings.get_symbol_settings("BTCUSDT");

        assert!(effective.enabled);
        assert_eq!(effective.leverage, settings.basic.default_leverage);
        assert_eq!(
            effective.position_size_pct,
            settings.basic.default_position_size_pct
        );
        assert_eq!(effective.stop_loss_pct, settings.risk.default_stop_loss_pct);
        assert_eq!(
            effective.take_profit_pct,
            settings.risk.default_take_profit_pct
        );
    }

    #[test]
    fn test_get_symbol_settings_custom() {
        let mut settings = PaperTradingSettings::default();

        let symbol_settings = SymbolSettings {
            enabled: true,
            leverage: Some(20),
            position_size_pct: Some(10.0),
            stop_loss_pct: Some(3.0),
            take_profit_pct: Some(6.0),
            trading_hours: None,
            min_price_movement_pct: None,
            max_positions: Some(2),
            custom_params: HashMap::new(),
        };

        settings.set_symbol_settings("BTCUSDT".to_string(), symbol_settings);

        let effective = settings.get_symbol_settings("BTCUSDT");

        assert!(effective.enabled);
        assert_eq!(effective.leverage, 20);
        assert_eq!(effective.position_size_pct, 10.0);
        assert_eq!(effective.stop_loss_pct, 3.0);
        assert_eq!(effective.take_profit_pct, 6.0);
        assert_eq!(effective.max_positions, 2);
    }

    #[test]
    fn test_set_and_remove_symbol_settings() {
        let mut settings = PaperTradingSettings::default();

        let symbol_settings = SymbolSettings {
            enabled: true,
            leverage: Some(15),
            position_size_pct: None,
            stop_loss_pct: None,
            take_profit_pct: None,
            trading_hours: None,
            min_price_movement_pct: None,
            max_positions: None,
            custom_params: HashMap::new(),
        };

        settings.set_symbol_settings("ETHUSDT".to_string(), symbol_settings);

        assert_eq!(settings.get_configured_symbols().len(), 1);

        settings.remove_symbol_settings("ETHUSDT");

        assert_eq!(settings.get_configured_symbols().len(), 0);
    }

    #[test]
    fn test_update_basic_settings() {
        let mut settings = PaperTradingSettings::default();

        let new_basic = BasicSettings {
            initial_balance: 20000.0,
            max_positions: 5,
            default_position_size_pct: 3.0,
            default_leverage: 5,
            trading_fee_rate: 0.0002,
            funding_fee_rate: 0.0001,
            slippage_pct: 0.01,
            enabled: true,
            auto_restart: false,
        };

        let result = settings.update_basic(new_basic.clone());
        assert!(result.is_ok());
        assert_eq!(settings.basic.initial_balance, 20000.0);
        assert_eq!(settings.basic.max_positions, 5);
    }

    #[test]
    fn test_update_basic_settings_invalid() {
        let mut settings = PaperTradingSettings::default();

        let invalid_basic = BasicSettings {
            initial_balance: -1000.0,
            max_positions: 5,
            default_position_size_pct: 3.0,
            default_leverage: 5,
            trading_fee_rate: 0.0002,
            funding_fee_rate: 0.0001,
            slippage_pct: 0.01,
            enabled: true,
            auto_restart: false,
        };

        let result = settings.update_basic(invalid_basic);
        assert!(result.is_err());
    }

    #[test]
    fn test_update_risk_settings() {
        let mut settings = PaperTradingSettings::default();

        let new_risk = RiskSettings {
            max_risk_per_trade_pct: 1.0,
            max_portfolio_risk_pct: 10.0,
            default_stop_loss_pct: 1.5,
            default_take_profit_pct: 3.0,
            max_leverage: 25,
            min_margin_level: 150.0,
            max_drawdown_pct: 10.0,
            daily_loss_limit_pct: 3.0,
            max_consecutive_losses: 3,
            cool_down_minutes: 60, // FIXED: Match docs - 60 minutes
            position_sizing_method: PositionSizingMethod::FixedPercentage,
            min_risk_reward_ratio: 2.0,
            correlation_limit: 0.8,
            dynamic_sizing: false,
            volatility_lookback_hours: 12,
            trailing_stop_enabled: true,
            trailing_stop_pct: 3.0,
            trailing_activation_pct: 5.0,
            enable_signal_reversal: false,
            ai_auto_enable_reversal: false,
            reversal_min_confidence: 0.75,
            reversal_max_pnl_pct: 10.0,
            reversal_allowed_regimes: vec!["trending".to_string()],
        };

        let result = settings.update_risk(new_risk.clone());
        assert!(result.is_ok());
        assert_eq!(settings.risk.max_risk_per_trade_pct, 1.0);
    }

    #[test]
    fn test_update_risk_settings_invalid() {
        let mut settings = PaperTradingSettings::default();

        let mut invalid_risk = RiskSettings::default();
        invalid_risk.max_risk_per_trade_pct = 60.0;

        let result = settings.update_risk(invalid_risk);
        assert!(result.is_err());
    }

    #[test]
    fn test_position_sizing_method_variants() {
        let methods = vec![
            PositionSizingMethod::FixedPercentage,
            PositionSizingMethod::RiskBased,
            PositionSizingMethod::VolatilityAdjusted,
            PositionSizingMethod::ConfidenceWeighted,
            PositionSizingMethod::Composite,
        ];

        for method in methods {
            let mut settings = RiskSettings::default();
            settings.position_sizing_method = method;
            // Should be able to set any variant
        }
    }

    #[test]
    fn test_strategy_combination_method_variants() {
        let methods = vec![
            StrategyCombinationMethod::WeightedAverage,
            StrategyCombinationMethod::MajorityVoting,
            StrategyCombinationMethod::Unanimous,
            StrategyCombinationMethod::HighestConfidence,
            StrategyCombinationMethod::AIEnsemble,
        ];

        for method in methods {
            let mut settings = StrategySettings::default();
            settings.combination_method = method;
            // Should be able to set any variant
        }
    }

    #[test]
    fn test_notification_channel_variants() {
        let channels = vec![
            NotificationChannel::WebSocket,
            NotificationChannel::Email("test@example.com".to_string()),
            NotificationChannel::Telegram("@testuser".to_string()),
            NotificationChannel::Discord("webhook_url".to_string()),
            NotificationChannel::Webhook("https://example.com/hook".to_string()),
        ];

        let mut settings = NotificationSettings::default();
        settings.channels = channels;

        assert_eq!(settings.channels.len(), 5);
    }

    #[test]
    fn test_backtesting_settings_default() {
        let settings = BacktestingSettings::default();

        assert!(settings.enabled);
        assert_eq!(settings.period_days, 90);
        assert_eq!(settings.data_resolution, "15m"); // Changed from 1h to 15m
        assert_eq!(settings.min_trades, 20);
        assert!(!settings.walk_forward_optimization);
        assert_eq!(settings.out_of_sample_pct, 20.0);
    }

    #[test]
    fn test_trading_hours() {
        let hours = TradingHours {
            start_hour: 9,
            start_minute: 30,
            end_hour: 16,
            end_minute: 0,
            timezone: "UTC".to_string(),
        };

        assert_eq!(hours.start_hour, 9);
        assert_eq!(hours.end_hour, 16);
    }

    #[test]
    fn test_symbol_settings_disabled() {
        let mut settings = PaperTradingSettings::default();

        let symbol_settings = SymbolSettings {
            enabled: false,
            leverage: None,
            position_size_pct: None,
            stop_loss_pct: None,
            take_profit_pct: None,
            trading_hours: None,
            min_price_movement_pct: None,
            max_positions: None,
            custom_params: HashMap::new(),
        };

        settings.set_symbol_settings("BTCUSDT".to_string(), symbol_settings);

        let effective = settings.get_symbol_settings("BTCUSDT");
        assert!(!effective.enabled);
    }

    #[test]
    fn test_get_configured_symbols() {
        let mut settings = PaperTradingSettings::default();

        let symbol_settings = SymbolSettings {
            enabled: true,
            leverage: None,
            position_size_pct: None,
            stop_loss_pct: None,
            take_profit_pct: None,
            trading_hours: None,
            min_price_movement_pct: None,
            max_positions: None,
            custom_params: HashMap::new(),
        };

        settings.set_symbol_settings("BTCUSDT".to_string(), symbol_settings.clone());
        settings.set_symbol_settings("ETHUSDT".to_string(), symbol_settings.clone());
        settings.set_symbol_settings("BNBUSDT".to_string(), symbol_settings);

        let configured = settings.get_configured_symbols();
        assert_eq!(configured.len(), 3);
        assert!(configured.contains(&"BTCUSDT".to_string()));
        assert!(configured.contains(&"ETHUSDT".to_string()));
        assert!(configured.contains(&"BNBUSDT".to_string()));
    }

    #[test]
    fn test_extreme_leverage_validation() {
        let mut settings = PaperTradingSettings::default();

        // Test edge cases for leverage
        settings.basic.default_leverage = 1;
        assert!(settings.validate().is_ok());

        settings.basic.default_leverage = 125;
        assert!(settings.validate().is_ok());

        settings.basic.default_leverage = 126;
        assert!(settings.validate().is_err());
    }

    #[test]
    fn test_zero_values() {
        let mut settings = PaperTradingSettings::default();

        // Zero initial balance should fail
        settings.basic.initial_balance = 0.0;
        assert!(settings.validate().is_err());

        settings.basic.initial_balance = 10000.0;

        // Zero risk per trade should fail
        settings.risk.max_risk_per_trade_pct = 0.0;
        assert!(settings.validate().is_err());
    }

    #[test]
    fn test_default_notification_settings() {
        let settings = NotificationSettings::default();

        assert!(settings.enable_trade_notifications);
        assert!(settings.enable_performance_notifications);
        assert!(settings.enable_risk_warnings);
        assert!(settings.daily_summary);
        assert!(settings.weekly_report);
        assert_eq!(settings.min_pnl_notification, 10.0);
        assert_eq!(settings.max_notifications_per_hour, 20);
        assert_eq!(settings.channels.len(), 1);
    }

    #[test]
    fn test_default_paper_trading_settings() {
        let settings = PaperTradingSettings::default();

        assert!(settings.basic.enabled);
        assert_eq!(settings.basic.initial_balance, 10000.0);
        assert_eq!(settings.risk.max_leverage, 5); // FIXED: Down from 50x
        assert_eq!(settings.strategy.min_ai_confidence, 0.5); // FIXED: Down from 0.7
        assert!(settings.ai.enable_realtime_signals);
        assert!(settings.execution.auto_execution);
        assert!(settings.notifications.enable_trade_notifications);
        assert!(settings.symbols.is_empty());
    }

    #[test]
    fn test_serialization_basic_settings() {
        let settings = BasicSettings::default();
        let serialized = serde_json::to_string(&settings).unwrap();
        let deserialized: BasicSettings = serde_json::from_str(&serialized).unwrap();

        assert_eq!(settings.initial_balance, deserialized.initial_balance);
        assert_eq!(settings.max_positions, deserialized.max_positions);
        assert_eq!(settings.default_leverage, deserialized.default_leverage);
    }

    #[test]
    fn test_serialization_risk_settings() {
        let settings = RiskSettings::default();
        let serialized = serde_json::to_string(&settings).unwrap();
        let deserialized: RiskSettings = serde_json::from_str(&serialized).unwrap();

        assert_eq!(
            settings.max_risk_per_trade_pct,
            deserialized.max_risk_per_trade_pct
        );
        assert_eq!(settings.max_leverage, deserialized.max_leverage);
        assert_eq!(settings.min_margin_level, deserialized.min_margin_level);
    }

    #[test]
    fn test_serialization_strategy_settings() {
        let settings = StrategySettings::default();
        let serialized = serde_json::to_string(&settings).unwrap();
        let deserialized: StrategySettings = serde_json::from_str(&serialized).unwrap();

        assert_eq!(settings.min_ai_confidence, deserialized.min_ai_confidence);
        assert_eq!(
            settings.optimization_period_days,
            deserialized.optimization_period_days
        );
    }

    #[test]
    fn test_serialization_notification_channel() {
        let channels = vec![
            NotificationChannel::WebSocket,
            NotificationChannel::Email("test@test.com".to_string()),
            NotificationChannel::Telegram("@user".to_string()),
        ];

        let serialized = serde_json::to_string(&channels).unwrap();
        let deserialized: Vec<NotificationChannel> = serde_json::from_str(&serialized).unwrap();

        assert_eq!(channels.len(), deserialized.len());
    }

    #[test]
    fn test_boundary_max_risk_per_trade() {
        let mut settings = PaperTradingSettings::default();

        // Test lower boundary
        settings.risk.max_risk_per_trade_pct = 0.01;
        assert!(settings.validate().is_ok());

        // Test upper boundary
        settings.risk.max_risk_per_trade_pct = 50.0;
        assert!(settings.validate().is_ok());

        // Test just beyond upper boundary
        settings.risk.max_risk_per_trade_pct = 50.01;
        assert!(settings.validate().is_err());

        // Test negative value
        settings.risk.max_risk_per_trade_pct = -1.0;
        assert!(settings.validate().is_err());
    }

    #[test]
    fn test_boundary_max_portfolio_risk() {
        let mut settings = PaperTradingSettings::default();

        // Test lower boundary
        settings.risk.max_portfolio_risk_pct = 0.01;
        assert!(settings.validate().is_ok());

        // Test upper boundary
        settings.risk.max_portfolio_risk_pct = 100.0;
        assert!(settings.validate().is_ok());

        // Test just beyond upper boundary
        settings.risk.max_portfolio_risk_pct = 100.01;
        assert!(settings.validate().is_err());

        // Test zero
        settings.risk.max_portfolio_risk_pct = 0.0;
        assert!(settings.validate().is_err());
    }

    #[test]
    fn test_boundary_trading_fee_rate() {
        let mut settings = PaperTradingSettings::default();

        // Test zero (valid)
        settings.basic.trading_fee_rate = 0.0;
        assert!(settings.validate().is_ok());

        // Test upper boundary
        settings.basic.trading_fee_rate = 0.01;
        assert!(settings.validate().is_ok());

        // Test just beyond upper boundary
        settings.basic.trading_fee_rate = 0.0101;
        assert!(settings.validate().is_err());
    }

    #[test]
    fn test_boundary_ai_confidence() {
        let mut settings = PaperTradingSettings::default();

        // Test lower boundary
        settings.strategy.min_ai_confidence = 0.0;
        assert!(settings.validate().is_ok());

        // Test upper boundary
        settings.strategy.min_ai_confidence = 1.0;
        assert!(settings.validate().is_ok());

        // Test just beyond upper boundary
        settings.strategy.min_ai_confidence = 1.01;
        assert!(settings.validate().is_err());

        // Test negative
        settings.strategy.min_ai_confidence = -0.1;
        assert!(settings.validate().is_err());
    }

    #[test]
    fn test_boundary_min_margin_level() {
        let mut settings = PaperTradingSettings::default();

        // Test exactly at minimum
        settings.risk.min_margin_level = 100.0;
        assert!(settings.validate().is_ok());

        // Test above minimum
        settings.risk.min_margin_level = 150.0;
        assert!(settings.validate().is_ok());

        // Test below minimum
        settings.risk.min_margin_level = 99.9;
        assert!(settings.validate().is_err());
    }

    #[test]
    fn test_symbol_settings_partial_override() {
        let mut settings = PaperTradingSettings::default();

        // Only override leverage, rest should use defaults
        let symbol_settings = SymbolSettings {
            enabled: true,
            leverage: Some(15),
            position_size_pct: None,
            stop_loss_pct: None,
            take_profit_pct: None,
            trading_hours: None,
            min_price_movement_pct: None,
            max_positions: None,
            custom_params: HashMap::new(),
        };

        settings.set_symbol_settings("BTCUSDT".to_string(), symbol_settings);

        let effective = settings.get_symbol_settings("BTCUSDT");

        assert_eq!(effective.leverage, 15);
        assert_eq!(
            effective.position_size_pct,
            settings.basic.default_position_size_pct
        );
        assert_eq!(effective.stop_loss_pct, settings.risk.default_stop_loss_pct);
        assert_eq!(
            effective.take_profit_pct,
            settings.risk.default_take_profit_pct
        );
    }

    #[test]
    fn test_symbol_settings_with_trading_hours() {
        let trading_hours = TradingHours {
            start_hour: 8,
            start_minute: 30,
            end_hour: 17,
            end_minute: 0,
            timezone: "EST".to_string(),
        };

        let symbol_settings = SymbolSettings {
            enabled: true,
            leverage: None,
            position_size_pct: None,
            stop_loss_pct: None,
            take_profit_pct: None,
            trading_hours: Some(trading_hours.clone()),
            min_price_movement_pct: Some(0.5),
            max_positions: None,
            custom_params: HashMap::new(),
        };

        assert_eq!(symbol_settings.trading_hours.unwrap().start_hour, 8);
    }

    #[test]
    fn test_symbol_settings_custom_params() {
        let mut custom_params = HashMap::new();
        custom_params.insert("custom_param1".to_string(), serde_json::json!("value1"));
        custom_params.insert("custom_param2".to_string(), serde_json::json!(42));

        let symbol_settings = SymbolSettings {
            enabled: true,
            leverage: None,
            position_size_pct: None,
            stop_loss_pct: None,
            take_profit_pct: None,
            trading_hours: None,
            min_price_movement_pct: None,
            max_positions: None,
            custom_params: custom_params.clone(),
        };

        assert_eq!(symbol_settings.custom_params.len(), 2);
    }

    #[test]
    fn test_ai_settings_confidence_thresholds() {
        let settings = AISettings::default();

        assert_eq!(settings.confidence_thresholds.get("trending"), Some(&0.65));
        assert_eq!(settings.confidence_thresholds.get("ranging"), Some(&0.75));
        assert_eq!(settings.confidence_thresholds.get("volatile"), Some(&0.80));
    }

    #[test]
    fn test_ai_settings_custom_confidence_thresholds() {
        let mut settings = AISettings::default();

        settings
            .confidence_thresholds
            .insert("custom_regime".to_string(), 0.85);

        assert_eq!(settings.confidence_thresholds.len(), 4);
        assert_eq!(
            settings.confidence_thresholds.get("custom_regime"),
            Some(&0.85)
        );
    }

    #[test]
    fn test_execution_settings_boundaries() {
        let mut settings = ExecutionSettings::default();

        settings.partial_fill_probability = 0.0;
        assert!(settings.partial_fill_probability >= 0.0);

        settings.partial_fill_probability = 1.0;
        assert!(settings.partial_fill_probability <= 1.0);

        settings.max_slippage_pct = 0.0;
        assert!(settings.max_slippage_pct >= 0.0);
    }

    #[test]
    fn test_strategy_settings_enabled_strategies() {
        let settings = StrategySettings::default();

        assert_eq!(settings.enabled_strategies.len(), 1);
        assert_eq!(settings.enabled_strategies.get("ai_ensemble"), Some(&1.0));
    }

    #[test]
    fn test_strategy_settings_multiple_strategies() {
        let mut settings = StrategySettings::default();

        settings
            .enabled_strategies
            .insert("momentum".to_string(), 0.3);
        settings
            .enabled_strategies
            .insert("mean_reversion".to_string(), 0.4);
        settings
            .enabled_strategies
            .insert("breakout".to_string(), 0.3);

        assert_eq!(settings.enabled_strategies.len(), 4);

        // Sum of weights should equal 1.0 (approximately)
        let total_weight: f64 = settings.enabled_strategies.values().sum();
        assert!((total_weight - 2.0).abs() < 0.001); // ai_ensemble is 1.0 + 3 new ones
    }

    #[test]
    fn test_strategy_regime_specific_params() {
        let mut settings = StrategySettings::default();

        let mut trending_params = HashMap::new();
        trending_params.insert("threshold".to_string(), serde_json::json!(0.6));
        trending_params.insert("period".to_string(), serde_json::json!(20));

        settings
            .regime_specific_params
            .insert("trending".to_string(), trending_params);

        assert_eq!(settings.regime_specific_params.len(), 1);
    }

    #[test]
    fn test_update_basic_settings_valid() {
        let mut settings = PaperTradingSettings::default();

        let new_basic = BasicSettings {
            initial_balance: 50000.0,
            max_positions: 15,
            default_position_size_pct: 10.0,
            default_leverage: 20,
            trading_fee_rate: 0.0003,
            funding_fee_rate: 0.00015,
            slippage_pct: 0.02,
            enabled: false,
            auto_restart: true,
        };

        let result = settings.update_basic(new_basic.clone());
        assert!(result.is_ok());
        assert_eq!(settings.basic.initial_balance, 50000.0);
        assert_eq!(settings.basic.max_positions, 15);
        assert!(!settings.basic.enabled);
        assert!(settings.basic.auto_restart);
    }

    #[test]
    fn test_update_risk_settings_valid() {
        let mut settings = PaperTradingSettings::default();

        let new_risk = RiskSettings {
            max_risk_per_trade_pct: 3.0,
            max_portfolio_risk_pct: 25.0,
            default_stop_loss_pct: 2.5,
            default_take_profit_pct: 5.0,
            max_leverage: 75,
            min_margin_level: 180.0,
            max_drawdown_pct: 20.0,
            daily_loss_limit_pct: 8.0,
            max_consecutive_losses: 7,
            cool_down_minutes: 90,
            position_sizing_method: PositionSizingMethod::VolatilityAdjusted,
            min_risk_reward_ratio: 2.5,
            correlation_limit: 0.6,
            dynamic_sizing: false,
            volatility_lookback_hours: 48,
            trailing_stop_enabled: true,
            trailing_stop_pct: 3.0,
            trailing_activation_pct: 5.0,
            enable_signal_reversal: false,
            ai_auto_enable_reversal: false,
            reversal_min_confidence: 0.75,
            reversal_max_pnl_pct: 10.0,
            reversal_allowed_regimes: vec!["trending".to_string()],
        };

        let result = settings.update_risk(new_risk.clone());
        assert!(result.is_ok());
        assert_eq!(settings.risk.max_risk_per_trade_pct, 3.0);
        assert_eq!(settings.risk.max_consecutive_losses, 7);
        assert!(!settings.risk.dynamic_sizing);
    }

    #[test]
    fn test_update_risk_settings_boundary_values() {
        let mut settings = PaperTradingSettings::default();

        // Test at exact boundary (50%)
        let mut boundary_risk = RiskSettings::default();
        boundary_risk.max_risk_per_trade_pct = 50.0;
        assert!(settings.update_risk(boundary_risk).is_ok());

        // Test just over boundary
        let mut over_boundary_risk = RiskSettings::default();
        over_boundary_risk.max_risk_per_trade_pct = 50.1;
        assert!(settings.update_risk(over_boundary_risk).is_err());

        // Test at zero boundary
        let mut zero_risk = RiskSettings::default();
        zero_risk.max_risk_per_trade_pct = 0.0;
        assert!(settings.update_risk(zero_risk).is_err());
    }

    #[test]
    fn test_multiple_symbol_configurations() {
        let mut settings = PaperTradingSettings::default();

        let btc_settings = SymbolSettings {
            enabled: true,
            leverage: Some(10),
            position_size_pct: Some(8.0),
            stop_loss_pct: Some(2.5),
            take_profit_pct: Some(5.0),
            trading_hours: None,
            min_price_movement_pct: Some(0.3),
            max_positions: Some(3),
            custom_params: HashMap::new(),
        };

        let eth_settings = SymbolSettings {
            enabled: true,
            leverage: Some(15),
            position_size_pct: Some(6.0),
            stop_loss_pct: Some(3.0),
            take_profit_pct: Some(6.0),
            trading_hours: None,
            min_price_movement_pct: Some(0.4),
            max_positions: Some(2),
            custom_params: HashMap::new(),
        };

        let bnb_settings = SymbolSettings {
            enabled: false,
            leverage: None,
            position_size_pct: None,
            stop_loss_pct: None,
            take_profit_pct: None,
            trading_hours: None,
            min_price_movement_pct: None,
            max_positions: None,
            custom_params: HashMap::new(),
        };

        settings.set_symbol_settings("BTCUSDT".to_string(), btc_settings);
        settings.set_symbol_settings("ETHUSDT".to_string(), eth_settings);
        settings.set_symbol_settings("BNBUSDT".to_string(), bnb_settings);

        assert_eq!(settings.get_configured_symbols().len(), 3);

        let btc_effective = settings.get_symbol_settings("BTCUSDT");
        assert!(btc_effective.enabled);
        assert_eq!(btc_effective.leverage, 10);
        assert_eq!(btc_effective.position_size_pct, 8.0);
        assert_eq!(btc_effective.max_positions, 3);

        let eth_effective = settings.get_symbol_settings("ETHUSDT");
        assert_eq!(eth_effective.leverage, 15);
        assert_eq!(eth_effective.position_size_pct, 6.0);

        let bnb_effective = settings.get_symbol_settings("BNBUSDT");
        assert!(!bnb_effective.enabled);
    }

    #[test]
    fn test_get_symbol_settings_nonexistent() {
        let settings = PaperTradingSettings::default();

        // Get settings for a symbol that doesn't exist
        let effective = settings.get_symbol_settings("XRPUSDT");

        // Should return defaults
        assert!(effective.enabled);
        assert_eq!(effective.leverage, settings.basic.default_leverage);
        assert_eq!(
            effective.position_size_pct,
            settings.basic.default_position_size_pct
        );
    }

    #[test]
    fn test_remove_nonexistent_symbol() {
        let mut settings = PaperTradingSettings::default();

        // Remove a symbol that doesn't exist (should not panic)
        settings.remove_symbol_settings("NONEXISTENT");

        assert_eq!(settings.get_configured_symbols().len(), 0);
    }

    #[test]
    fn test_backtesting_settings_custom() {
        let backtesting = BacktestingSettings {
            enabled: false,
            period_days: 180,
            data_resolution: "15m".to_string(),
            min_trades: 100,
            walk_forward_optimization: true,
            out_of_sample_pct: 30.0,
        };

        assert!(!backtesting.enabled);
        assert_eq!(backtesting.period_days, 180);
        assert_eq!(backtesting.data_resolution, "15m");
        assert!(backtesting.walk_forward_optimization);
        assert_eq!(backtesting.out_of_sample_pct, 30.0);
    }

    #[test]
    fn test_notification_settings_custom_channels() {
        let mut settings = NotificationSettings::default();

        settings.channels = vec![
            NotificationChannel::WebSocket,
            NotificationChannel::Email("trader@example.com".to_string()),
            NotificationChannel::Telegram("@trader_bot".to_string()),
            NotificationChannel::Discord("https://discord.com/webhook/123".to_string()),
            NotificationChannel::Webhook("https://api.example.com/notifications".to_string()),
        ];

        settings.min_pnl_notification = 50.0;
        settings.max_notifications_per_hour = 10;

        assert_eq!(settings.channels.len(), 5);
        assert_eq!(settings.min_pnl_notification, 50.0);
        assert_eq!(settings.max_notifications_per_hour, 10);
    }

    #[test]
    fn test_effective_symbol_settings_all_fields() {
        let mut settings = PaperTradingSettings::default();

        let symbol_settings = SymbolSettings {
            enabled: true,
            leverage: Some(25),
            position_size_pct: Some(7.5),
            stop_loss_pct: Some(1.8),
            take_profit_pct: Some(3.6),
            trading_hours: None,
            min_price_movement_pct: Some(0.25),
            max_positions: Some(5),
            custom_params: HashMap::new(),
        };

        settings.set_symbol_settings("ADAUSDT".to_string(), symbol_settings);

        let effective = settings.get_symbol_settings("ADAUSDT");

        assert!(effective.enabled);
        assert_eq!(effective.leverage, 25);
        assert_eq!(effective.position_size_pct, 7.5);
        assert_eq!(effective.stop_loss_pct, 1.8);
        assert_eq!(effective.take_profit_pct, 3.6);
        assert_eq!(effective.max_positions, 5);
    }

    #[test]
    fn test_file_io_operations() {
        use std::fs;

        let temp_dir = std::env::temp_dir();
        let test_file = temp_dir.join("test_paper_trading_settings.toml");

        let settings = PaperTradingSettings::default();

        // Test saving to file
        let save_result = settings.to_file(test_file.to_str().unwrap());
        assert!(save_result.is_ok());

        // Test loading from file
        let load_result = PaperTradingSettings::from_file(test_file.to_str().unwrap());
        assert!(load_result.is_ok());

        let loaded_settings = load_result.unwrap();
        assert_eq!(
            loaded_settings.basic.initial_balance,
            settings.basic.initial_balance
        );
        assert_eq!(
            loaded_settings.risk.max_leverage,
            settings.risk.max_leverage
        );

        // Cleanup
        let _ = fs::remove_file(test_file);
    }

    #[test]
    fn test_file_io_nonexistent_file() {
        let result = PaperTradingSettings::from_file("/nonexistent/path/settings.toml");
        assert!(result.is_err());
    }

    #[test]
    fn test_file_io_invalid_toml() {
        use std::fs;

        let temp_dir = std::env::temp_dir();
        let test_file = temp_dir.join("invalid_settings.toml");

        // Write invalid TOML
        let _ = fs::write(&test_file, "this is not valid toml {[}]");

        let result = PaperTradingSettings::from_file(test_file.to_str().unwrap());
        assert!(result.is_err());

        // Cleanup
        let _ = fs::remove_file(test_file);
    }

    #[test]
    fn test_complete_validation_pass() {
        let settings = PaperTradingSettings::default();
        assert!(settings.validate().is_ok());
    }

    #[test]
    fn test_validation_multiple_failures() {
        let mut settings = PaperTradingSettings::default();

        // Set multiple invalid values
        settings.basic.initial_balance = -100.0;

        // Should fail on first error
        let result = settings.validate();
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("balance"));
    }

    #[test]
    fn test_position_sizing_method_serialization() {
        let methods = vec![
            PositionSizingMethod::FixedPercentage,
            PositionSizingMethod::RiskBased,
            PositionSizingMethod::VolatilityAdjusted,
            PositionSizingMethod::ConfidenceWeighted,
            PositionSizingMethod::Composite,
        ];

        for method in methods {
            let serialized = serde_json::to_string(&method).unwrap();
            let deserialized: PositionSizingMethod = serde_json::from_str(&serialized).unwrap();

            // Both should serialize/deserialize successfully
            let _ = serde_json::to_string(&deserialized).unwrap();
        }
    }

    #[test]
    fn test_strategy_combination_method_serialization() {
        let methods = vec![
            StrategyCombinationMethod::WeightedAverage,
            StrategyCombinationMethod::MajorityVoting,
            StrategyCombinationMethod::Unanimous,
            StrategyCombinationMethod::HighestConfidence,
            StrategyCombinationMethod::AIEnsemble,
        ];

        for method in methods {
            let serialized = serde_json::to_string(&method).unwrap();
            let deserialized: StrategyCombinationMethod =
                serde_json::from_str(&serialized).unwrap();

            // Both should serialize/deserialize successfully
            let _ = serde_json::to_string(&deserialized).unwrap();
        }
    }

    #[test]
    fn test_trading_hours_edge_cases() {
        // 24-hour trading
        let hours_24 = TradingHours {
            start_hour: 0,
            start_minute: 0,
            end_hour: 23,
            end_minute: 59,
            timezone: "UTC".to_string(),
        };

        assert_eq!(hours_24.start_hour, 0);
        assert_eq!(hours_24.end_hour, 23);

        // Same start and end (no trading window)
        let hours_same = TradingHours {
            start_hour: 10,
            start_minute: 0,
            end_hour: 10,
            end_minute: 0,
            timezone: "UTC".to_string(),
        };

        assert_eq!(hours_same.start_hour, hours_same.end_hour);
    }

    #[test]
    fn test_ai_settings_all_fields() {
        let mut settings = AISettings::default();

        settings.service_url = "http://custom-ai:9000".to_string();
        settings.request_timeout_seconds = 60;
        settings.signal_refresh_interval_minutes = 10;
        settings.enable_realtime_signals = false;
        settings.enable_feedback_learning = false;
        settings.feedback_delay_hours = 8;
        settings.enable_strategy_recommendations = false;
        settings.track_model_performance = false;

        assert_eq!(settings.service_url, "http://custom-ai:9000");
        assert_eq!(settings.request_timeout_seconds, 60);
        assert!(!settings.enable_realtime_signals);
        assert!(!settings.enable_feedback_learning);
    }

    #[test]
    fn test_execution_settings_all_combinations() {
        let mut settings = ExecutionSettings::default();

        settings.auto_execution = false;
        settings.simulate_partial_fills = true;
        settings.partial_fill_probability = 0.5;
        settings.simulate_slippage = false;
        settings.simulate_market_impact = true;
        settings.market_impact_factor = 0.002;

        assert!(!settings.auto_execution);
        assert!(settings.simulate_partial_fills);
        assert_eq!(settings.partial_fill_probability, 0.5);
        assert!(!settings.simulate_slippage);
        assert!(settings.simulate_market_impact);
    }

    #[test]
    fn test_risk_settings_all_fields_custom() {
        let settings = RiskSettings {
            max_risk_per_trade_pct: 1.5,
            max_portfolio_risk_pct: 15.0,
            default_stop_loss_pct: 1.8,
            default_take_profit_pct: 3.6,
            max_leverage: 30,
            min_margin_level: 175.0,
            max_drawdown_pct: 12.0,
            daily_loss_limit_pct: 4.0,
            max_consecutive_losses: 4,
            cool_down_minutes: 45,
            position_sizing_method: PositionSizingMethod::ConfidenceWeighted,
            min_risk_reward_ratio: 2.0,
            correlation_limit: 0.75,
            dynamic_sizing: true,
            volatility_lookback_hours: 36,
            trailing_stop_enabled: true,
            trailing_stop_pct: 3.0,
            trailing_activation_pct: 5.0,
            enable_signal_reversal: false,
            ai_auto_enable_reversal: false,
            reversal_min_confidence: 0.75,
            reversal_max_pnl_pct: 10.0,
            reversal_allowed_regimes: vec!["trending".to_string()],
        };

        assert_eq!(settings.max_risk_per_trade_pct, 1.5);
        assert_eq!(settings.max_portfolio_risk_pct, 15.0);
        assert_eq!(settings.max_consecutive_losses, 4);
        assert_eq!(settings.cool_down_minutes, 45);
        assert_eq!(settings.volatility_lookback_hours, 36);
    }

    #[test]
    fn test_basic_settings_all_fields_custom() {
        let settings = BasicSettings {
            initial_balance: 25000.0,
            max_positions: 8,
            default_position_size_pct: 4.5,
            default_leverage: 15,
            trading_fee_rate: 0.0005,
            funding_fee_rate: 0.00012,
            slippage_pct: 0.015,
            enabled: false,
            auto_restart: true,
        };

        assert_eq!(settings.initial_balance, 25000.0);
        assert_eq!(settings.max_positions, 8);
        assert_eq!(settings.trading_fee_rate, 0.0005);
        assert!(!settings.enabled);
        assert!(settings.auto_restart);
    }

    #[test]
    fn test_clone_all_settings() {
        let settings = PaperTradingSettings::default();
        let cloned = settings.clone();

        assert_eq!(settings.basic.initial_balance, cloned.basic.initial_balance);
        assert_eq!(settings.risk.max_leverage, cloned.risk.max_leverage);
        assert_eq!(
            settings.strategy.min_ai_confidence,
            cloned.strategy.min_ai_confidence
        );
    }
}
