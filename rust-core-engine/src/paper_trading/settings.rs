use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Complete paper trading configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
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

impl Default for PaperTradingSettings {
    fn default() -> Self {
        Self {
            basic: BasicSettings::default(),
            risk: RiskSettings::default(),
            strategy: StrategySettings::default(),
            symbols: HashMap::new(),
            ai: AISettings::default(),
            execution: ExecutionSettings::default(),
            notifications: NotificationSettings::default(),
        }
    }
}

impl Default for BasicSettings {
    fn default() -> Self {
        Self {
            initial_balance: 10000.0,
            max_positions: 10,
            default_position_size_pct: 5.0,
            default_leverage: 10,
            trading_fee_rate: 0.0004, // 0.04% Binance Futures
            funding_fee_rate: 0.0001, // 0.01% every 8 hours
            slippage_pct: 0.01,       // 0.01% average slippage
            enabled: true,
            auto_restart: false,
        }
    }
}

impl Default for RiskSettings {
    fn default() -> Self {
        Self {
            max_risk_per_trade_pct: 2.0,
            max_portfolio_risk_pct: 20.0,
            default_stop_loss_pct: 2.0,
            default_take_profit_pct: 4.0,
            max_leverage: 50,
            min_margin_level: 200.0,
            max_drawdown_pct: 15.0,
            daily_loss_limit_pct: 5.0,
            max_consecutive_losses: 5,
            cool_down_minutes: 60,
            position_sizing_method: PositionSizingMethod::RiskBased,
            min_risk_reward_ratio: 1.5,
            correlation_limit: 0.7,
            dynamic_sizing: true,
            volatility_lookback_hours: 24,
        }
    }
}

impl Default for StrategySettings {
    fn default() -> Self {
        let mut enabled_strategies = HashMap::new();
        enabled_strategies.insert("ai_ensemble".to_string(), 1.0);
        
        Self {
            enabled_strategies,
            min_ai_confidence: 0.7,
            combination_method: StrategyCombinationMethod::AIEnsemble,
            enable_optimization: true,
            optimization_period_days: 30,
            min_trades_for_optimization: 50,
            signal_timeout_minutes: 30,
            enable_market_regime_detection: true,
            regime_specific_params: HashMap::new(),
            backtesting: BacktestingSettings::default(),
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
            signal_refresh_interval_minutes: 5, // Changed from 30 to 5 minutes for faster signal processing
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
            data_resolution: "1h".to_string(),
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
            return Err(anyhow::anyhow!("Default leverage must be between 1 and 125"));
        }
        
        if self.basic.trading_fee_rate < 0.0 || self.basic.trading_fee_rate > 0.01 {
            return Err(anyhow::anyhow!("Trading fee rate must be between 0 and 1%"));
        }
        
        // Validate risk settings
        if self.risk.max_risk_per_trade_pct <= 0.0 || self.risk.max_risk_per_trade_pct > 50.0 {
            return Err(anyhow::anyhow!("Max risk per trade must be between 0% and 50%"));
        }
        
        if self.risk.max_portfolio_risk_pct <= 0.0 || self.risk.max_portfolio_risk_pct > 100.0 {
            return Err(anyhow::anyhow!("Max portfolio risk must be between 0% and 100%"));
        }
        
        if self.risk.max_leverage > 125 {
            return Err(anyhow::anyhow!("Max leverage cannot exceed 125"));
        }
        
        if self.risk.min_margin_level < 100.0 {
            return Err(anyhow::anyhow!("Min margin level must be at least 100%"));
        }
        
        // Validate strategy settings
        if self.strategy.min_ai_confidence < 0.0 || self.strategy.min_ai_confidence > 1.0 {
            return Err(anyhow::anyhow!("AI confidence must be between 0 and 1"));
        }
        
        // Validate AI settings
        if self.ai.request_timeout_seconds == 0 {
            return Err(anyhow::anyhow!("Request timeout must be positive"));
        }
        
        if self.ai.signal_refresh_interval_minutes == 0 {
            return Err(anyhow::anyhow!("Signal refresh interval must be positive"));
        }
        
        Ok(())
    }
    
    /// Get effective settings for a specific symbol
    pub fn get_symbol_settings(&self, symbol: &str) -> EffectiveSymbolSettings {
        let symbol_specific = self.symbols.get(symbol);
        
        EffectiveSymbolSettings {
            enabled: symbol_specific.map_or(true, |s| s.enabled),
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
            max_positions: symbol_specific
                .and_then(|s| s.max_positions)
                .unwrap_or(1),
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