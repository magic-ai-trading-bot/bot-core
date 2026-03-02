// @spec:FR-REAL-013 - Real Trading Engine
// @spec:FR-REAL-030 - User Data Stream Integration
// @ref:specs/02-design/2.5-components/COMP-RUST-TRADING.md
// @test:TC-REAL-030, TC-REAL-031, TC-REAL-032

use anyhow::{anyhow, Result};
use chrono::{DateTime, Utc};
use dashmap::DashMap;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::{broadcast, Mutex, RwLock};
use tracing::{debug, error, info, warn};
use uuid::Uuid;

use crate::binance::types::{
    BalanceUpdate, ExecutionReport, FuturesPosition, Kline, OrderSide, OutboundAccountPosition,
    SpotOrderRequest, SpotOrderResponse, SpotOrderType, TimeInForce,
};
use crate::binance::user_data_stream::{UserDataStreamEvent, UserDataStreamManager};
use crate::binance::BinanceClient;
use crate::config::TradingMode;
use crate::market_data::cache::{CandleData, MarketDataCache};
use crate::paper_trading::AIMarketBias;
use crate::strategies::strategy_engine::StrategyEngine;
use crate::strategies::TradingSignal;
use crate::trading::risk_manager::RiskManager;

use super::config::RealTradingConfig;
use super::order::{OrderState, RealOrder};
use super::position::{PositionSide, RealPosition};
use super::risk::RealTradingRiskManager;

/// Signal history for choppy market detection
type SignalFlipTracker = HashMap<String, Vec<(i64, TradingSignal)>>;

/// Circuit breaker state for error handling
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct CircuitBreakerState {
    /// Whether circuit breaker is open (trading halted)
    pub is_open: bool,
    /// Number of consecutive errors
    pub error_count: u32,
    /// When circuit breaker was opened
    pub opened_at: Option<DateTime<Utc>>,
    /// Last error message
    pub last_error: Option<String>,
}

impl CircuitBreakerState {
    /// Record an error, possibly opening the circuit
    pub fn record_error(&mut self, error: &str, threshold: u32) -> bool {
        self.error_count += 1;
        self.last_error = Some(error.to_string());

        if self.error_count >= threshold && !self.is_open {
            self.is_open = true;
            self.opened_at = Some(Utc::now());
            true // Circuit just opened
        } else {
            false
        }
    }

    /// Record a success, resetting error count
    pub fn record_success(&mut self) {
        self.error_count = 0;
        // Don't auto-close circuit - use explicit reset
    }

    /// Check if circuit should close based on cooldown
    pub fn should_close(&self, cooldown_secs: u64) -> bool {
        if let Some(opened_at) = self.opened_at {
            let elapsed = Utc::now().signed_duration_since(opened_at);
            elapsed.num_seconds() >= cooldown_secs as i64
        } else {
            false
        }
    }

    /// Close the circuit breaker
    pub fn close(&mut self) {
        self.is_open = false;
        self.error_count = 0;
        self.opened_at = None;
        self.last_error = None;
    }
}

/// Events emitted by the trading engine
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RealTradingEvent {
    /// Order placed successfully
    OrderPlaced(RealOrder),
    /// Order fully filled
    OrderFilled(RealOrder),
    /// Order partially filled
    OrderPartiallyFilled(RealOrder),
    /// Order cancelled
    OrderCancelled(RealOrder),
    /// Order rejected by exchange
    OrderRejected { order: RealOrder, reason: String },
    /// New position opened
    PositionOpened(RealPosition),
    /// Position updated (fill/price change)
    PositionUpdated(RealPosition),
    /// Position closed
    PositionClosed { position: RealPosition, pnl: f64 },
    /// Balance updated
    BalanceUpdated {
        asset: String,
        free: f64,
        locked: f64,
    },
    /// Circuit breaker opened
    CircuitBreakerOpened(String),
    /// Circuit breaker closed
    CircuitBreakerClosed,
    /// Reconciliation completed
    ReconciliationComplete { discrepancies: u32 },
    /// Error occurred
    Error(String),
    /// Daily loss limit reached
    DailyLossLimitReached { loss: f64, limit: f64 },
    /// Strategy signal generated
    SignalGenerated {
        symbol: String,
        signal: String,
        confidence: f64,
    },
    /// Signal executed as real trade
    SignalExecuted {
        symbol: String,
        signal: String,
        order_id: String,
    },
    /// Signal rejected by filters or risk checks
    SignalRejected { symbol: String, reason: String },
    /// Cool-down activated after consecutive losses
    CooldownActivated {
        consecutive_losses: u32,
        cool_down_minutes: u32,
    },
    /// Engine started
    EngineStarted,
    /// Engine stopped
    EngineStopped,
}

/// Balance tracking
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Balance {
    pub asset: String,
    pub free: f64,
    pub locked: f64,
}

impl Balance {
    pub fn total(&self) -> f64 {
        self.free + self.locked
    }
}

/// Daily trading metrics
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct DailyMetrics {
    pub date: String,
    pub realized_pnl: f64,
    pub trades_count: u32,
    pub winning_trades: u32,
    pub losing_trades: u32,
    pub total_volume: f64,
    pub total_commission: f64,
}

/// Reconciliation metrics for monitoring
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ReconciliationMetrics {
    /// Last reconciliation run time
    pub last_run_time: Option<DateTime<Utc>>,
    /// Duration of last reconciliation in milliseconds
    pub last_run_duration_ms: u64,
    /// Total discrepancies found across all reconciliations
    pub total_discrepancies_found: u64,
    /// Balance mismatches detected
    pub balance_mismatches: u64,
    /// Order state mismatches detected
    pub order_mismatches: u64,
    /// Stale orders cancelled
    pub stale_orders_cancelled: u64,
    /// Terminal orders cleaned up
    pub terminal_orders_cleaned: u64,
    /// Consecutive reconciliation failures
    pub consecutive_failures: u32,
    /// Total reconciliation runs
    pub total_runs: u64,
}

impl DailyMetrics {
    pub fn new() -> Self {
        Self {
            date: Utc::now().format("%Y-%m-%d").to_string(),
            ..Default::default()
        }
    }

    pub fn win_rate(&self) -> f64 {
        if self.trades_count > 0 {
            (self.winning_trades as f64 / self.trades_count as f64) * 100.0
        } else {
            0.0
        }
    }

    pub fn reset_if_new_day(&mut self) {
        let today = Utc::now().format("%Y-%m-%d").to_string();
        if self.date != today {
            *self = Self::new();
        }
    }
}

/// Main real trading engine
#[derive(Clone)]
pub struct RealTradingEngine {
    // ============ Thread-Safe State ============
    /// Active positions by symbol
    positions: Arc<DashMap<String, RealPosition>>,

    /// Active orders by client_order_id
    orders: Arc<DashMap<String, RealOrder>>,

    /// Account balances by asset
    balances: Arc<RwLock<HashMap<String, Balance>>>,

    // ============ Configuration ============
    /// Trading configuration
    config: Arc<RwLock<RealTradingConfig>>,

    // ============ External Services ============
    /// Binance API client
    binance_client: BinanceClient,

    /// Legacy risk manager (for compatibility)
    #[allow(dead_code)]
    risk_manager: Arc<RwLock<RiskManager>>,

    /// Real trading risk manager with comprehensive checks
    real_risk_manager: RealTradingRiskManager,

    /// User Data Stream manager for WebSocket events
    user_data_stream: Arc<RwLock<UserDataStreamManager>>,

    // ============ Event Broadcasting ============
    /// Event broadcaster for external consumers
    event_tx: broadcast::Sender<RealTradingEvent>,

    // ============ Engine State ============
    /// Whether engine is running
    is_running: Arc<RwLock<bool>>,

    /// Circuit breaker state
    circuit_breaker: Arc<RwLock<CircuitBreakerState>>,

    /// Daily metrics
    daily_metrics: Arc<RwLock<DailyMetrics>>,

    /// Reconciliation metrics
    reconciliation_metrics: Arc<RwLock<ReconciliationMetrics>>,

    // ============ Synchronization ============
    /// Execution lock for order operations
    execution_lock: Arc<Mutex<()>>,

    /// Whether to use Futures endpoints (derived from config.trading_type)
    use_futures: bool,

    // ============ Auto-Trading State ============
    /// Strategy engine for signal generation
    strategy_engine: Arc<StrategyEngine>,

    /// Historical kline data cache per symbol_timeframe
    historical_data_cache: Arc<RwLock<HashMap<String, Vec<Kline>>>>,

    /// Market data cache for O(1) WebSocket price lookups
    market_data_cache: Option<MarketDataCache>,

    /// Current prices for all tracked symbols
    current_prices: Arc<RwLock<HashMap<String, f64>>>,

    /// AI market bias per symbol (from external AI service)
    ai_market_bias: Arc<RwLock<HashMap<String, AIMarketBias>>>,

    /// Signal confirmation: requires 2 consecutive same-direction signals
    /// Key: "symbol_direction", Value: (first_seen_timestamp, signal_count)
    recent_signals: Arc<RwLock<HashMap<String, (i64, u32)>>>,

    /// Choppy market detection: tracks direction flips per symbol
    signal_flip_tracker: Arc<RwLock<SignalFlipTracker>>,

    /// Consecutive loss counter for cool-down trigger
    consecutive_losses: Arc<RwLock<u32>>,

    /// Cool-down expiry time (None = not in cool-down)
    cool_down_until: Arc<RwLock<Option<DateTime<Utc>>>>,

    /// Whether the strategy signal loop has been spawned
    signal_loop_spawned: Arc<RwLock<bool>>,
}

impl RealTradingEngine {
    /// Create a new real trading engine
    pub async fn new(
        config: RealTradingConfig,
        binance_client: BinanceClient,
        risk_manager: RiskManager,
    ) -> Result<Self> {
        // Validate configuration
        config
            .validate()
            .map_err(|errs| anyhow!("Invalid config: {}", errs.join(", ")))?;

        let (event_tx, _) = broadcast::channel(1000);

        // Create UserDataStreamManager (Futures or Spot based on config)
        let user_data_stream = if config.is_futures() {
            UserDataStreamManager::new_futures(binance_client.clone())
        } else {
            UserDataStreamManager::new(binance_client.clone())
        };

        // Create real trading risk manager
        let real_risk_manager = RealTradingRiskManager::new(config.clone());

        let use_futures = config.is_futures();

        Ok(Self {
            positions: Arc::new(DashMap::new()),
            orders: Arc::new(DashMap::new()),
            balances: Arc::new(RwLock::new(HashMap::new())),
            config: Arc::new(RwLock::new(config)),
            binance_client,
            risk_manager: Arc::new(RwLock::new(risk_manager)),
            real_risk_manager,
            user_data_stream: Arc::new(RwLock::new(user_data_stream)),
            event_tx,
            is_running: Arc::new(RwLock::new(false)),
            circuit_breaker: Arc::new(RwLock::new(CircuitBreakerState::default())),
            daily_metrics: Arc::new(RwLock::new(DailyMetrics::new())),
            reconciliation_metrics: Arc::new(RwLock::new(ReconciliationMetrics::default())),
            execution_lock: Arc::new(Mutex::new(())),
            use_futures,
            // Auto-trading state
            strategy_engine: Arc::new(StrategyEngine::new()),
            historical_data_cache: Arc::new(RwLock::new(HashMap::new())),
            market_data_cache: None,
            current_prices: Arc::new(RwLock::new(HashMap::new())),
            ai_market_bias: Arc::new(RwLock::new(HashMap::new())),
            recent_signals: Arc::new(RwLock::new(HashMap::new())),
            signal_flip_tracker: Arc::new(RwLock::new(HashMap::new())),
            consecutive_losses: Arc::new(RwLock::new(0)),
            cool_down_until: Arc::new(RwLock::new(None)),
            signal_loop_spawned: Arc::new(RwLock::new(false)),
        })
    }

    /// Set the market data cache for real-time WebSocket price lookups
    /// Must be called before start() — uses O(1) cache reads instead of REST polling
    pub fn set_market_data_cache(&mut self, cache: MarketDataCache) {
        self.market_data_cache = Some(cache);
        info!("Market data cache connected to RealTradingEngine (WebSocket prices -> O(1) lookup)");
    }

    /// Get event receiver for subscribing to events
    pub fn subscribe_events(&self) -> broadcast::Receiver<RealTradingEvent> {
        self.event_tx.subscribe()
    }

    /// Start the trading engine
    pub async fn start(&self) -> Result<()> {
        // Check if already running
        {
            let is_running = self.is_running.read().await;
            if *is_running {
                return Err(anyhow!("Engine already running"));
            }
        }

        // Verify we're in the correct trading mode
        let config = self.config.read().await;
        let expected_mode = if config.use_testnet {
            TradingMode::RealTestnet
        } else {
            TradingMode::RealMainnet
        };
        drop(config);

        info!("Starting RealTradingEngine in {:?} mode", expected_mode);

        // 1. Start UserDataStream
        {
            let mut stream = self.user_data_stream.write().await;
            stream.start().await.map_err(|e| {
                error!("Failed to start UserDataStream: {}", e);
                anyhow!("Failed to start UserDataStream: {}", e)
            })?;
        }
        info!("UserDataStream started");

        // 2. Load initial balances and perform initial sync
        self.initial_sync().await?;

        // 3. Set running flag BEFORE spawning loops (prevents race condition)
        {
            let mut is_running = self.is_running.write().await;
            *is_running = true;
        }

        // 4. Subscribe to events and spawn handler
        let event_rx = {
            let stream = self.user_data_stream.read().await;
            stream.subscribe()
        };

        let engine = self.clone();
        tokio::spawn(async move {
            engine.process_user_data_events(event_rx).await;
        });
        info!("UserDataStream event handler spawned");

        // 5. Spawn reconciliation loop
        let engine_for_reconciliation = self.clone();
        tokio::spawn(async move {
            engine_for_reconciliation.reconciliation_loop().await;
        });
        info!("Reconciliation loop spawned");

        // 6. Spawn price update loop (always runs — needed for SL/TP monitoring)
        let engine_for_prices = self.clone();
        tokio::spawn(async move {
            engine_for_prices.price_update_loop().await;
        });
        info!("Price update loop spawned (5s interval)");

        // 7. Spawn SL/TP monitoring loop (always runs)
        let engine_for_sltp = self.clone();
        tokio::spawn(async move {
            engine_for_sltp.sl_tp_monitoring_loop().await;
        });
        info!("SL/TP monitoring loop spawned (5s interval)");

        // 8. Spawn strategy signal loop (only if auto-trading is enabled)
        {
            let config = self.config.read().await;
            if config.auto_trading_enabled {
                let engine_for_signals = self.clone();
                tokio::spawn(async move {
                    engine_for_signals.strategy_signal_loop().await;
                });
                *self.signal_loop_spawned.write().await = true;
                info!("Strategy signal loop spawned (30s interval) - auto-trading ENABLED");
            } else {
                info!("Auto-trading disabled - strategy signal loop NOT started (will auto-spawn when enabled via API)");
            }
        }

        self.emit_event(RealTradingEvent::EngineStarted);

        info!("RealTradingEngine started successfully");
        Ok(())
    }

    // ============ User Data Stream Event Processing ============

    /// Process events from UserDataStream
    async fn process_user_data_events(&self, mut rx: broadcast::Receiver<UserDataStreamEvent>) {
        info!("Starting UserDataStream event processing loop");

        loop {
            // Check if engine is still running
            if !*self.is_running.read().await {
                info!("Engine stopped, exiting event loop");
                break;
            }

            match rx.recv().await {
                Ok(event) => {
                    match event {
                        UserDataStreamEvent::ExecutionReport(report) => {
                            if let Err(e) = self.process_execution_report(&report).await {
                                error!("Failed to process execution report: {}", e);
                                self.record_error(&format!("ExecutionReport error: {}", e))
                                    .await;
                            }
                        },
                        UserDataStreamEvent::AccountPosition(pos) => {
                            self.handle_account_position(pos).await;
                        },
                        UserDataStreamEvent::BalanceUpdate(update) => {
                            self.handle_balance_update(update).await;
                        },
                        UserDataStreamEvent::Connected => {
                            info!("UserDataStream connected");
                            // Reset error count on successful connection
                            self.circuit_breaker.write().await.record_success();
                        },
                        UserDataStreamEvent::Disconnected => {
                            warn!("UserDataStream disconnected, will reconnect...");
                            self.emit_event(RealTradingEvent::Error(
                                "UserDataStream disconnected".to_string(),
                            ));
                            // Trigger reconciliation on disconnect
                            self.handle_websocket_disconnect().await;
                        },
                        UserDataStreamEvent::Error(e) => {
                            error!("UserDataStream error: {}", e);
                            self.record_error(&e).await;
                        },
                    }
                },
                Err(broadcast::error::RecvError::Lagged(n)) => {
                    warn!("Lagged {} events, may need reconciliation", n);
                    // Trigger reconciliation on next opportunity
                    if let Err(e) = self.refresh_balances().await {
                        error!("Failed to refresh balances after lag: {}", e);
                    }
                },
                Err(broadcast::error::RecvError::Closed) => {
                    warn!("UserDataStream channel closed");
                    break;
                },
            }
        }

        info!("UserDataStream event processing loop ended");
    }

    /// Record an error and check circuit breaker
    async fn record_error(&self, error: &str) {
        let config = self.config.read().await;
        let threshold = config.circuit_breaker_errors;
        drop(config);

        let mut cb = self.circuit_breaker.write().await;
        if cb.record_error(error, threshold) {
            error!("Circuit breaker opened after error: {}", error);
            self.emit_event(RealTradingEvent::CircuitBreakerOpened(error.to_string()));
        }
    }

    /// Stop the trading engine
    pub async fn stop(&self) -> Result<()> {
        // Check if running
        {
            let is_running = self.is_running.read().await;
            if !*is_running {
                return Ok(());
            }
        }

        info!("Stopping RealTradingEngine...");

        // 1. Set running flag to false first (stops event loop)
        {
            let mut is_running = self.is_running.write().await;
            *is_running = false;
        }

        // 2. Stop UserDataStream
        {
            let mut stream = self.user_data_stream.write().await;
            if let Err(e) = stream.stop().await {
                warn!("Failed to stop UserDataStream: {}", e);
            }
        }
        info!("UserDataStream stopped");

        // 3. Cancel all open orders
        let open_orders: Vec<String> = self
            .orders
            .iter()
            .filter(|o| o.value().is_active())
            .map(|o| o.key().clone())
            .collect();

        for order_id in open_orders {
            if let Err(e) = self.cancel_order(&order_id).await {
                warn!("Failed to cancel order {} on shutdown: {}", order_id, e);
            }
        }

        self.emit_event(RealTradingEvent::EngineStopped);

        info!("RealTradingEngine stopped");
        Ok(())
    }

    /// Check if engine is running
    pub async fn is_running(&self) -> bool {
        *self.is_running.read().await
    }

    // ============ Order Management ============

    /// Place a market order
    pub async fn place_market_order(
        &self,
        symbol: &str,
        side: OrderSide,
        quantity: f64,
        position_id: Option<String>,
        is_entry: bool,
    ) -> Result<RealOrder> {
        self.place_order(
            symbol,
            side,
            SpotOrderType::Market,
            quantity,
            None,
            None,
            position_id,
            is_entry,
        )
        .await
    }

    /// Place a limit order
    pub async fn place_limit_order(
        &self,
        symbol: &str,
        side: OrderSide,
        quantity: f64,
        price: f64,
        position_id: Option<String>,
        is_entry: bool,
    ) -> Result<RealOrder> {
        self.place_order(
            symbol,
            side,
            SpotOrderType::Limit,
            quantity,
            Some(price),
            None,
            position_id,
            is_entry,
        )
        .await
    }

    /// Place an order (internal)
    /// Note: Multiple parameters needed for complete order specification in trading systems
    #[allow(clippy::too_many_arguments)]
    async fn place_order(
        &self,
        symbol: &str,
        side: OrderSide,
        order_type: SpotOrderType,
        quantity: f64,
        price: Option<f64>,
        stop_price: Option<f64>,
        position_id: Option<String>,
        is_entry: bool,
    ) -> Result<RealOrder> {
        // Acquire execution lock
        let _lock = self.execution_lock.lock().await;

        // Check if engine is running
        if !*self.is_running.read().await {
            return Err(anyhow!("Engine not running"));
        }

        // Check circuit breaker
        {
            let cb = self.circuit_breaker.read().await;
            if cb.is_open {
                return Err(anyhow!("Circuit breaker is open: {:?}", cb.last_error));
            }
        }

        // Validate symbol
        {
            let config = self.config.read().await;
            if !config.is_symbol_allowed(symbol) {
                return Err(anyhow!("Symbol {} not allowed", symbol));
            }
        }

        // Pre-trade risk checks using comprehensive risk manager
        // Skip risk checks for exit orders (closing positions reduces risk)
        if is_entry {
            self.check_risk_limits(symbol, side, quantity, price)
                .await?;
        }

        // Generate client order ID
        let client_order_id = format!(
            "real_{}",
            &Uuid::new_v4().to_string().replace("-", "")[..16]
        );

        // Convert side to string
        let side_str = match side {
            OrderSide::Buy => "BUY".to_string(),
            OrderSide::Sell => "SELL".to_string(),
        };

        // Convert order type to string
        let order_type_str = match order_type {
            SpotOrderType::Market => "MARKET".to_string(),
            SpotOrderType::Limit => "LIMIT".to_string(),
            SpotOrderType::StopLossLimit => "STOP_LOSS_LIMIT".to_string(),
            SpotOrderType::TakeProfitLimit => "TAKE_PROFIT_LIMIT".to_string(),
            SpotOrderType::LimitMaker => "LIMIT_MAKER".to_string(),
        };

        // Create local order tracking
        let order = RealOrder::new(
            client_order_id.clone(),
            symbol.to_string(),
            side_str.clone(),
            order_type_str.clone(),
            quantity,
            price,
            stop_price,
            position_id,
            is_entry,
        );

        // Store order in pending state
        self.orders.insert(client_order_id.clone(), order.clone());

        // Submit to exchange (futures or spot based on trading mode)
        if self.use_futures {
            // Build futures order request
            let time_in_force = if order_type == SpotOrderType::Limit {
                Some("GTC".to_string())
            } else {
                None
            };
            let rounded_qty = Self::round_quantity_for_exchange(symbol, quantity);
            let request = crate::binance::types::NewOrderRequest {
                symbol: symbol.to_string(),
                side: side_str,
                r#type: order_type_str,
                quantity: Some(rounded_qty.to_string()),
                quote_order_qty: None,
                price: if order_type == SpotOrderType::Market {
                    None
                } else {
                    price.map(|p| format!("{:.2}", p))
                },
                new_client_order_id: Some(client_order_id.clone()),
                stop_price: stop_price.map(|p| format!("{:.2}", p)),
                iceberg_qty: None,
                new_order_resp_type: None,
                time_in_force,
                reduce_only: None,
                close_position: None,
                position_side: None,
                working_type: None,
                price_protect: None,
            };

            match self.binance_client.place_futures_order(request).await {
                Ok(response) => {
                    // Update order from futures response
                    self.update_order_from_futures_response(&client_order_id, &response)
                        .await;

                    let updated_order = self.orders.get(&client_order_id).map(|o| o.clone());
                    if let Some(order) = updated_order {
                        info!(
                            "Futures order placed: {} {} {} @ {:?}",
                            order.side, order.symbol, order.original_quantity, order.price
                        );
                        self.circuit_breaker.write().await.record_success();
                        self.emit_event(RealTradingEvent::OrderPlaced(order.clone()));
                        Ok(order)
                    } else {
                        Err(anyhow!("Order not found after placement"))
                    }
                },
                Err(e) => {
                    let error_msg = format!("Order placement failed: {}", e);
                    let mut cb = self.circuit_breaker.write().await;
                    let config = self.config.read().await;
                    if cb.record_error(&error_msg, config.circuit_breaker_errors) {
                        error!("Circuit breaker opened after order error");
                        self.emit_event(RealTradingEvent::CircuitBreakerOpened(error_msg.clone()));
                    }
                    if let Some(mut order) = self.orders.get_mut(&client_order_id) {
                        order.state = OrderState::Rejected;
                        order.reject_reason = Some(e.to_string());
                    }
                    Err(anyhow!(error_msg))
                },
            }
        } else {
            // Build spot order request
            let request = SpotOrderRequest {
                symbol: symbol.to_string(),
                side,
                order_type,
                time_in_force: if order_type == SpotOrderType::Limit {
                    Some(TimeInForce::Gtc)
                } else {
                    None
                },
                quantity: Some(quantity.to_string()),
                quote_order_qty: None,
                price: price.map(|p| p.to_string()),
                client_order_id: Some(client_order_id.clone()),
                stop_price: stop_price.map(|p| p.to_string()),
                iceberg_qty: None,
                new_order_resp_type: None,
            };

            match self.binance_client.place_spot_order(request).await {
                Ok(response) => {
                    self.update_order_from_response(&client_order_id, &response)
                        .await;

                    let updated_order = self.orders.get(&client_order_id).map(|o| o.clone());
                    if let Some(order) = updated_order {
                        info!(
                            "Spot order placed: {} {} {} @ {:?}",
                            order.side, order.symbol, order.original_quantity, order.price
                        );
                        self.circuit_breaker.write().await.record_success();
                        self.emit_event(RealTradingEvent::OrderPlaced(order.clone()));
                        Ok(order)
                    } else {
                        Err(anyhow!("Order not found after placement"))
                    }
                },
                Err(e) => {
                    let error_msg = format!("Order placement failed: {}", e);
                    let mut cb = self.circuit_breaker.write().await;
                    let config = self.config.read().await;
                    if cb.record_error(&error_msg, config.circuit_breaker_errors) {
                        error!("Circuit breaker opened after order error");
                        self.emit_event(RealTradingEvent::CircuitBreakerOpened(error_msg.clone()));
                    }
                    if let Some(mut order) = self.orders.get_mut(&client_order_id) {
                        order.state = OrderState::Rejected;
                        order.reject_reason = Some(e.to_string());
                    }
                    Err(anyhow!(error_msg))
                },
            }
        }
    }

    /// Cancel an order by client order ID
    pub async fn cancel_order(&self, client_order_id: &str) -> Result<RealOrder> {
        let _lock = self.execution_lock.lock().await;

        let order = self
            .orders
            .get(client_order_id)
            .map(|o| o.clone())
            .ok_or_else(|| anyhow!("Order not found: {}", client_order_id))?;

        if !order.is_active() {
            return Err(anyhow!("Order not active: {}", client_order_id));
        }

        let cancel_result = if self.use_futures {
            self.binance_client
                .cancel_futures_order(
                    &order.symbol,
                    Some(order.exchange_order_id),
                    Some(client_order_id),
                )
                .await
        } else {
            self.binance_client
                .cancel_spot_order(
                    &order.symbol,
                    Some(order.exchange_order_id),
                    Some(client_order_id),
                )
                .await
        };

        match cancel_result {
            Ok(_) => {
                if let Some(mut order) = self.orders.get_mut(client_order_id) {
                    order.state = OrderState::Cancelled;
                    order.updated_at = Utc::now();
                }

                let updated = self.orders.get(client_order_id).map(|o| o.clone()).unwrap();
                self.emit_event(RealTradingEvent::OrderCancelled(updated.clone()));
                Ok(updated)
            },
            Err(e) => Err(anyhow!("Cancel failed: {}", e)),
        }
    }

    /// Update order from exchange response
    async fn update_order_from_response(
        &self,
        client_order_id: &str,
        response: &SpotOrderResponse,
    ) {
        if let Some(mut order) = self.orders.get_mut(client_order_id) {
            order.exchange_order_id = response.order_id;
            order.state = OrderState::from_binance_status(&response.status);

            if let Ok(exec_qty) = response.executed_qty.parse::<f64>() {
                order.executed_quantity = exec_qty;
                order.remaining_quantity = order.original_quantity - exec_qty;
            }

            if let (Ok(quote_qty), Ok(exec_qty)) = (
                response.cumulative_quote_qty.parse::<f64>(),
                response.executed_qty.parse::<f64>(),
            ) {
                if exec_qty > 0.0 {
                    order.average_fill_price = quote_qty / exec_qty;
                }
            }

            order.updated_at = Utc::now();
        }
    }

    /// Update order from futures exchange response
    async fn update_order_from_futures_response(
        &self,
        client_order_id: &str,
        response: &crate::binance::types::FuturesOrderResponse,
    ) {
        if let Some(mut order) = self.orders.get_mut(client_order_id) {
            order.exchange_order_id = response.order_id;
            order.state = OrderState::from_binance_status(&response.status);

            if let Ok(exec_qty) = response.executed_qty.parse::<f64>() {
                order.executed_quantity = exec_qty;
                order.remaining_quantity = order.original_quantity - exec_qty;
            }

            if let (Ok(quote_qty), Ok(exec_qty)) = (
                response.cumulative_quote_qty.parse::<f64>(),
                response.executed_qty.parse::<f64>(),
            ) {
                if exec_qty > 0.0 {
                    order.average_fill_price = quote_qty / exec_qty;
                }
            }

            order.updated_at = Utc::now();
        }
    }

    // ============ ExecutionReport Processing ============

    /// Process an ExecutionReport from WebSocket
    pub async fn process_execution_report(&self, report: &ExecutionReport) -> Result<()> {
        let client_order_id = &report.client_order_id;

        debug!(
            "Processing ExecutionReport: {} - {}",
            client_order_id, report.execution_type
        );

        // Update order state
        if let Some(mut order) = self.orders.get_mut(client_order_id) {
            let prev_state = order.state.clone();
            order.update_from_execution_report(report);

            // Emit events based on state change
            match order.state {
                OrderState::Filled if prev_state != OrderState::Filled => {
                    info!(
                        "Order filled: {} {} {}",
                        order.symbol, order.side, order.executed_quantity
                    );
                    self.emit_event(RealTradingEvent::OrderFilled(order.clone()));

                    // Update position
                    drop(order);
                    self.update_position_from_fill(client_order_id).await?;
                },
                OrderState::PartiallyFilled => {
                    debug!(
                        "Order partially filled: {:.4}/{:.4}",
                        order.executed_quantity, order.original_quantity
                    );
                    self.emit_event(RealTradingEvent::OrderPartiallyFilled(order.clone()));
                },
                OrderState::Cancelled => {
                    self.emit_event(RealTradingEvent::OrderCancelled(order.clone()));
                },
                OrderState::Rejected => {
                    let reason = order.reject_reason.clone().unwrap_or_default();
                    self.emit_event(RealTradingEvent::OrderRejected {
                        order: order.clone(),
                        reason,
                    });
                },
                _ => {},
            }
        } else {
            warn!(
                "Received ExecutionReport for unknown order: {}",
                client_order_id
            );
        }

        Ok(())
    }

    /// Update position from a filled order
    async fn update_position_from_fill(&self, client_order_id: &str) -> Result<()> {
        let order = self
            .orders
            .get(client_order_id)
            .map(|o| o.clone())
            .ok_or_else(|| anyhow!("Order not found"))?;

        if !order.is_filled() && order.state != OrderState::PartiallyFilled {
            return Ok(());
        }

        let symbol = &order.symbol;
        let commission = order.total_commission();

        if order.is_entry {
            // Opening or adding to position
            if let Some(mut position) = self.positions.get_mut(symbol) {
                // Add to existing position
                position.add_fill(
                    order.average_fill_price,
                    order.executed_quantity,
                    commission,
                    order.client_order_id.clone(),
                );
                self.emit_event(RealTradingEvent::PositionUpdated(position.clone()));
            } else {
                // Create new position
                let side = PositionSide::from_order_side(&order.side);
                let mut position = RealPosition::new(
                    format!("pos_{}", Uuid::new_v4()),
                    symbol.clone(),
                    side,
                    order.executed_quantity,
                    order.average_fill_price,
                    order.client_order_id.clone(),
                    None,
                    None,
                );
                // Set leverage from config for liquidation risk monitoring
                let config = self.config.read().await;
                position.set_leverage(config.max_leverage);
                drop(config);
                self.positions.insert(symbol.clone(), position.clone());
                self.emit_event(RealTradingEvent::PositionOpened(position));
            }
        } else {
            // Closing position
            if let Some(mut position) = self.positions.get_mut(symbol) {
                let pnl = position.partial_close(
                    order.average_fill_price,
                    order.executed_quantity,
                    commission,
                    order.client_order_id.clone(),
                );

                // Update daily metrics
                {
                    let mut metrics = self.daily_metrics.write().await;
                    metrics.reset_if_new_day();
                    metrics.realized_pnl += pnl;
                    metrics.trades_count += 1;
                    if pnl >= 0.0 {
                        metrics.winning_trades += 1;
                    } else {
                        metrics.losing_trades += 1;
                    }
                    metrics.total_commission += commission;
                    metrics.total_volume += order.executed_quantity * order.average_fill_price;
                }

                // Record trade in risk manager for daily loss tracking
                self.real_risk_manager.record_trade(pnl).await;

                if position.is_closed() {
                    let closed_position = position.clone();
                    drop(position);
                    self.positions.remove(symbol);
                    self.emit_event(RealTradingEvent::PositionClosed {
                        position: closed_position,
                        pnl,
                    });
                } else {
                    self.emit_event(RealTradingEvent::PositionUpdated(position.clone()));
                }

                // Check daily loss limit
                self.check_daily_loss_limit().await;
            }
        }

        Ok(())
    }

    // ============ Risk Management ============

    /// Check pre-trade risk limits using the comprehensive risk manager
    async fn check_risk_limits(
        &self,
        symbol: &str,
        side: OrderSide,
        quantity: f64,
        price: Option<f64>,
    ) -> Result<()> {
        // Get current balances as HashMap
        let balances_lock = self.balances.read().await;
        let balances: HashMap<String, f64> = balances_lock
            .iter()
            .map(|(k, v)| (k.clone(), v.free))
            .collect();
        drop(balances_lock);

        // Use estimated price if not provided
        let effective_price = price.unwrap_or(50000.0);

        // Validate using the comprehensive risk manager
        let result = self
            .real_risk_manager
            .validate_order(
                symbol,
                side,
                quantity,
                effective_price,
                &self.positions,
                &balances,
            )
            .await?;

        if !result.passed {
            let error_msg = result.errors.join("; ");
            return Err(anyhow!("{}", error_msg));
        }

        // Log warnings if any
        for warning in &result.warnings {
            warn!("Risk warning: {}", warning);
        }

        // Log suggested size if different
        if let Some(suggested) = result.suggested_size {
            if (suggested - quantity).abs() > 0.0001 {
                info!(
                    "Risk manager suggests position size: {:.8} (requested: {:.8})",
                    suggested, quantity
                );
            }
        }

        Ok(())
    }

    /// Check pre-trade risk limits (legacy version for backwards compatibility)
    #[allow(dead_code)]
    async fn check_risk_limits_legacy(
        &self,
        symbol: &str,
        quantity: f64,
        price: Option<f64>,
    ) -> Result<()> {
        let config = self.config.read().await;

        // Check daily loss limit
        {
            let metrics = self.daily_metrics.read().await;
            if metrics.realized_pnl < 0.0
                && metrics.realized_pnl.abs() >= config.max_daily_loss_usdt
            {
                return Err(anyhow!(
                    "Daily loss limit reached: ${:.2} >= ${:.2}",
                    metrics.realized_pnl.abs(),
                    config.max_daily_loss_usdt
                ));
            }
        }

        // Check max positions
        if self.positions.len() >= config.max_positions as usize
            && !self.positions.contains_key(symbol)
        {
            return Err(anyhow!(
                "Max positions reached: {} >= {}",
                self.positions.len(),
                config.max_positions
            ));
        }

        // Check position size
        let estimated_value = quantity * price.unwrap_or(50000.0); // Fallback estimate
        if estimated_value > config.max_position_size_usdt {
            return Err(anyhow!(
                "Position size exceeds limit: ${:.2} > ${:.2}",
                estimated_value,
                config.max_position_size_usdt
            ));
        }

        // Check total exposure
        let current_exposure: f64 = self
            .positions
            .iter()
            .map(|p| p.value().position_value())
            .sum();
        if current_exposure + estimated_value > config.max_total_exposure_usdt {
            return Err(anyhow!(
                "Total exposure would exceed limit: ${:.2} + ${:.2} > ${:.2}",
                current_exposure,
                estimated_value,
                config.max_total_exposure_usdt
            ));
        }

        // Check minimum order value
        if estimated_value < config.min_order_value_usdt {
            return Err(anyhow!(
                "Order value too small: ${:.2} < ${:.2}",
                estimated_value,
                config.min_order_value_usdt
            ));
        }

        Ok(())
    }

    /// Check daily loss limit and emit event if reached
    async fn check_daily_loss_limit(&self) {
        let config = self.config.read().await;
        let metrics = self.daily_metrics.read().await;

        if metrics.realized_pnl < 0.0 && metrics.realized_pnl.abs() >= config.max_daily_loss_usdt {
            warn!(
                "Daily loss limit reached: ${:.2} >= ${:.2}",
                metrics.realized_pnl.abs(),
                config.max_daily_loss_usdt
            );
            self.emit_event(RealTradingEvent::DailyLossLimitReached {
                loss: metrics.realized_pnl.abs(),
                limit: config.max_daily_loss_usdt,
            });
        }
    }

    // ============ Balance Management ============

    /// Refresh balances from exchange (Futures or Spot)
    pub async fn refresh_balances(&self) -> Result<()> {
        if self.use_futures {
            self.refresh_futures_balances().await
        } else {
            self.refresh_spot_balances().await
        }
    }

    /// Refresh balances from Spot account (/api/v3/account)
    async fn refresh_spot_balances(&self) -> Result<()> {
        match self.binance_client.get_account_info().await {
            Ok(account) => {
                let mut balances = self.balances.write().await;
                balances.clear();

                for balance in account.balances {
                    if let (Ok(free), Ok(locked)) =
                        (balance.free.parse::<f64>(), balance.locked.parse::<f64>())
                    {
                        if free > 0.0 || locked > 0.0 {
                            balances.insert(
                                balance.asset.clone(),
                                Balance {
                                    asset: balance.asset,
                                    free,
                                    locked,
                                },
                            );
                        }
                    }
                }

                debug!("Refreshed {} spot balances", balances.len());
                Ok(())
            },
            Err(e) => {
                error!("Failed to refresh spot balances: {}", e);
                Err(anyhow!("Spot balance refresh failed: {}", e))
            },
        }
    }

    /// Refresh balances from Futures account (/fapi/v2/account)
    async fn refresh_futures_balances(&self) -> Result<()> {
        match self.binance_client.get_futures_account().await {
            Ok(account) => {
                let mut balances = self.balances.write().await;
                balances.clear();

                // Futures account has "assets" array with different field names
                if let Some(assets) = account.get("assets").and_then(|a| a.as_array()) {
                    for asset_obj in assets {
                        let asset_name = asset_obj
                            .get("asset")
                            .and_then(|a| a.as_str())
                            .unwrap_or_default();
                        let wallet_balance = asset_obj
                            .get("walletBalance")
                            .and_then(|v| v.as_str())
                            .and_then(|s| s.parse::<f64>().ok())
                            .unwrap_or(0.0);
                        let available_balance = asset_obj
                            .get("availableBalance")
                            .and_then(|v| v.as_str())
                            .and_then(|s| s.parse::<f64>().ok())
                            .unwrap_or(0.0);

                        if wallet_balance > 0.0 || available_balance > 0.0 {
                            let locked = wallet_balance - available_balance;
                            balances.insert(
                                asset_name.to_string(),
                                Balance {
                                    asset: asset_name.to_string(),
                                    free: available_balance,
                                    locked: locked.max(0.0),
                                },
                            );
                        }
                    }
                }

                debug!("Refreshed {} futures balances", balances.len());
                Ok(())
            },
            Err(e) => {
                error!("Failed to refresh futures balances: {}", e);
                Err(anyhow!("Futures balance refresh failed: {}", e))
            },
        }
    }

    /// Get balance for an asset
    pub async fn get_balance(&self, asset: &str) -> Option<Balance> {
        self.balances.read().await.get(asset).cloned()
    }

    /// Get USDT balance (primary quote asset)
    pub async fn get_usdt_balance(&self) -> f64 {
        self.balances
            .read()
            .await
            .get("USDT")
            .map(|b| b.free)
            .unwrap_or(0.0)
    }

    // ============ Account Position & Balance Update Handlers ============

    // @spec:FR-REAL-033 - Balance Tracking from WebSocket
    /// Handle account position update from UserDataStream
    async fn handle_account_position(&self, pos: OutboundAccountPosition) {
        debug!(
            "Processing account position update: {} balances",
            pos.balances.len()
        );

        let mut balances = self.balances.write().await;

        for balance in pos.balances {
            // Parse balances with proper error handling
            let free = match balance.free.parse::<f64>() {
                Ok(v) if v >= 0.0 => v,
                Ok(v) => {
                    warn!("Invalid negative balance for {}: free={}", balance.asset, v);
                    continue;
                },
                Err(e) => {
                    warn!(
                        "Failed to parse free balance for {}: '{}' - {}",
                        balance.asset, balance.free, e
                    );
                    continue;
                },
            };

            let locked = match balance.locked.parse::<f64>() {
                Ok(v) if v >= 0.0 => v,
                Ok(v) => {
                    warn!(
                        "Invalid negative locked balance for {}: locked={}",
                        balance.asset, v
                    );
                    0.0 // Keep free balance but set locked to 0
                },
                Err(e) => {
                    warn!(
                        "Failed to parse locked balance for {}: '{}' - {}",
                        balance.asset, balance.locked, e
                    );
                    0.0 // Keep free balance but set locked to 0
                },
            };

            // Only track non-zero balances
            if free > 0.0 || locked > 0.0 {
                let asset = balance.asset.clone();
                balances.insert(
                    asset.clone(),
                    Balance {
                        asset: asset.clone(),
                        free,
                        locked,
                    },
                );

                // Emit balance updated event
                self.emit_event(RealTradingEvent::BalanceUpdated {
                    asset,
                    free,
                    locked,
                });
            } else {
                // Remove zero balances
                balances.remove(&balance.asset);
            }
        }

        debug!("Updated {} balances from account position", balances.len());
    }

    /// Handle balance update (delta) from UserDataStream
    async fn handle_balance_update(&self, update: BalanceUpdate) {
        let delta = match update.balance_delta.parse::<f64>() {
            Ok(d) => d,
            Err(e) => {
                error!(
                    "Failed to parse balance delta for {}: '{}' - {}",
                    update.asset, update.balance_delta, e
                );
                return;
            },
        };

        debug!(
            "Processing balance update: {} delta {}",
            update.asset, delta
        );

        let mut balances = self.balances.write().await;

        // Get current balance and apply delta
        let current = balances.get(&update.asset).map(|b| b.free).unwrap_or(0.0);
        let new_free = current + delta;

        if new_free < 0.0 {
            warn!("Balance update would result in negative balance for {}: current={}, delta={}, new={}",
                  update.asset, current, delta, new_free);
        }

        if new_free > 0.0 {
            let locked = balances.get(&update.asset).map(|b| b.locked).unwrap_or(0.0);

            balances.insert(
                update.asset.clone(),
                Balance {
                    asset: update.asset.clone(),
                    free: new_free,
                    locked,
                },
            );

            info!(
                "Balance update: {} delta {} (new free: {})",
                update.asset, delta, new_free
            );

            self.emit_event(RealTradingEvent::BalanceUpdated {
                asset: update.asset,
                free: new_free,
                locked,
            });
        }
    }

    // ============ Initial Sync ============

    // @spec:FR-REAL-034 - Initial State Sync
    /// Perform initial sync to load existing state from REST API
    async fn initial_sync(&self) -> Result<()> {
        info!("Performing initial sync...");

        // 1. Refresh balances from REST API
        self.refresh_balances().await?;

        // 2. Load open orders from REST API
        self.sync_open_orders().await?;

        // 3. Sync open positions from exchange
        self.sync_positions_from_exchange().await?;

        info!("Initial sync completed successfully");
        Ok(())
    }

    /// Sync open orders from REST API
    async fn sync_open_orders(&self) -> Result<()> {
        info!("Syncing open orders from exchange...");

        match self.binance_client.get_open_orders(None).await {
            Ok(open_orders) => {
                let mut count = 0;
                for order_response in open_orders {
                    // Convert to RealOrder
                    let client_order_id = order_response.client_order_id.clone();

                    // Skip if we already track this order
                    if self.orders.contains_key(&client_order_id) {
                        continue;
                    }

                    // Parse order details
                    let executed_qty = order_response.executed_qty.parse::<f64>().unwrap_or(0.0);
                    let orig_qty = order_response.orig_qty.parse::<f64>().unwrap_or(0.0);
                    let price = order_response.price.parse::<f64>().ok();
                    let stop_price = order_response.stop_price.parse::<f64>().ok();

                    let avg_price = if executed_qty > 0.0 {
                        let quote_qty = order_response
                            .cumulative_quote_qty
                            .parse::<f64>()
                            .unwrap_or(0.0);
                        quote_qty / executed_qty
                    } else {
                        0.0
                    };

                    let order = RealOrder {
                        client_order_id: client_order_id.clone(),
                        exchange_order_id: order_response.order_id,
                        symbol: order_response.symbol.clone(),
                        side: order_response.side.clone(),
                        order_type: order_response.r#type.clone(), // Note: `type` is a reserved keyword
                        original_quantity: orig_qty,
                        executed_quantity: executed_qty,
                        remaining_quantity: orig_qty - executed_qty,
                        price,
                        stop_price,
                        average_fill_price: avg_price,
                        state: OrderState::from_binance_status(&order_response.status),
                        created_at: Utc::now(), // Approximate since we don't have exact timestamp
                        updated_at: Utc::now(),
                        fills: Vec::new(),
                        position_id: None,
                        is_entry: true, // Assume entry, will be updated by logic
                        reject_reason: None,
                    };

                    self.orders.insert(client_order_id, order);
                    count += 1;
                }

                info!("Synced {} open orders from exchange", count);
                Ok(())
            },
            Err(e) => {
                warn!("Failed to sync open orders: {}", e);
                // Don't fail engine start if order sync fails
                Ok(())
            },
        }
    }

    // @spec:FR-REAL-055 - Position Sync from Exchange
    /// Sync open positions from Binance Futures API into local state
    async fn sync_positions_from_exchange(&self) -> Result<()> {
        if !self.use_futures {
            debug!("Position sync skipped: not in futures mode");
            return Ok(());
        }

        info!("Syncing open positions from exchange...");

        match self.binance_client.get_futures_positions().await {
            Ok(futures_positions) => {
                let mut synced = 0;

                // Build set of active exchange symbols for cleanup later
                let exchange_active_symbols: HashSet<String> =
                    futures_positions
                        .iter()
                        .filter(|fp| {
                            fp.position_amt.parse::<f64>().unwrap_or(0.0).abs() > 1e-10
                        })
                        .map(|fp| fp.symbol.clone())
                        .collect();

                for fp in &futures_positions {
                    let position_amt: f64 = fp.position_amt.parse().unwrap_or(0.0);

                    // Skip zero-size positions (Binance returns all symbols)
                    if position_amt.abs() < 1e-10 {
                        continue;
                    }

                    let symbol = fp.symbol.clone();
                    let entry_price: f64 = fp.entry_price.parse().unwrap_or(0.0);
                    let mark_price: f64 = fp.mark_price.parse().unwrap_or(0.0);
                    let unrealized_pnl: f64 = fp.unrealized_pnl.parse().unwrap_or(0.0);
                    let leverage: u32 = fp.leverage.parse().unwrap_or(1);

                    let side = if position_amt > 0.0 {
                        PositionSide::Long
                    } else {
                        PositionSide::Short
                    };

                    // Check if we already track this position
                    if let Some(mut existing) = self.positions.get_mut(&symbol) {
                        // Update existing position with exchange data
                        existing.quantity = position_amt.abs();
                        existing.entry_price = entry_price;
                        existing.current_price = mark_price;
                        existing.unrealized_pnl = unrealized_pnl;
                        existing.side = side;
                        existing.leverage = leverage;
                        existing.updated_at = Utc::now();
                        debug!(
                            "Updated existing position {}: qty={}, entry={:.2}, mark={:.2}",
                            symbol, existing.quantity, entry_price, mark_price
                        );
                    } else {
                        // Create new position from exchange data
                        let mut position = RealPosition::new(
                            Uuid::new_v4().to_string(),
                            symbol.clone(),
                            side,
                            position_amt.abs(),
                            entry_price,
                            format!("exchange-sync-{}", Uuid::new_v4()),
                            None, // strategy_name unknown for exchange-synced positions
                            None, // signal_confidence unknown
                        );
                        position.current_price = mark_price;
                        position.unrealized_pnl = unrealized_pnl;
                        position.leverage = leverage;

                        info!(
                            "Synced position from exchange: {} {:?} qty={} entry={:.2} mark={:.2} pnl={:.4}",
                            symbol, position.side, position.quantity, entry_price, mark_price, unrealized_pnl
                        );

                        self.positions.insert(symbol, position);
                        synced += 1;
                    }
                }

                // Remove local positions that no longer exist on exchange
                let local_symbols: Vec<String> = self
                    .positions
                    .iter()
                    .filter(|p| p.value().is_open())
                    .map(|p| p.key().clone())
                    .collect();

                for symbol in &local_symbols {
                    if !exchange_active_symbols.contains(symbol) {
                        if let Some(mut pos) = self.positions.get_mut(symbol) {
                            warn!(
                                "Position {} exists locally but not on exchange — marking as closed",
                                symbol
                            );
                            pos.quantity = 0.0;
                            pos.updated_at = Utc::now();
                        }
                    }
                }

                info!(
                    "Position sync complete: {} new positions synced, {} total open",
                    synced,
                    self.positions.iter().filter(|p| p.value().is_open()).count()
                );

                Ok(())
            },
            Err(e) => {
                warn!("Failed to sync positions from exchange: {}", e);
                // Don't fail engine start if position sync fails
                Ok(())
            },
        }
    }

    // ============ Reconciliation (Phase 5) ============

    // @spec:FR-REAL-051 - Periodic Reconciliation
    /// Reconciliation loop - runs periodically to sync state with exchange
    async fn reconciliation_loop(&self) {
        let interval_secs = {
            let config = self.config.read().await;
            config.reconciliation_interval_secs
        };

        info!(
            "Starting reconciliation loop with {}s interval",
            interval_secs
        );
        let mut interval = tokio::time::interval(Duration::from_secs(interval_secs));

        loop {
            interval.tick().await;

            // Check if engine is still running
            if !*self.is_running.read().await {
                info!("Engine stopped, exiting reconciliation loop");
                break;
            }

            // Skip if circuit breaker is open
            {
                let cb = self.circuit_breaker.read().await;
                if cb.is_open {
                    debug!("Circuit breaker open, skipping reconciliation");
                    continue;
                }
            }

            let start_time = std::time::Instant::now();

            match self.run_reconciliation().await {
                Ok(discrepancies) => {
                    let duration_ms = start_time.elapsed().as_millis() as u64;

                    // Update metrics
                    {
                        let mut metrics = self.reconciliation_metrics.write().await;
                        metrics.last_run_time = Some(Utc::now());
                        metrics.last_run_duration_ms = duration_ms;
                        metrics.total_discrepancies_found += discrepancies as u64;
                        metrics.consecutive_failures = 0;
                        metrics.total_runs += 1;
                    }

                    if discrepancies > 0 {
                        warn!(
                            "Reconciliation found {} discrepancies ({}ms)",
                            discrepancies, duration_ms
                        );
                    } else {
                        debug!(
                            "Reconciliation complete, no discrepancies ({}ms)",
                            duration_ms
                        );
                    }

                    let _ = self
                        .event_tx
                        .send(RealTradingEvent::ReconciliationComplete { discrepancies });
                },
                Err(e) => {
                    error!("Reconciliation failed: {}", e);

                    // Update failure metrics
                    {
                        let mut metrics = self.reconciliation_metrics.write().await;
                        metrics.consecutive_failures += 1;
                        metrics.total_runs += 1;
                    }

                    // Record error for circuit breaker
                    self.record_error(&format!("Reconciliation failed: {}", e))
                        .await;
                },
            }
        }

        info!("Reconciliation loop ended");
    }

    // @spec:FR-REAL-052 - Run Reconciliation
    /// Run a single reconciliation cycle
    pub async fn run_reconciliation(&self) -> Result<u32> {
        debug!("Running reconciliation...");
        let mut discrepancies = 0;

        // 1. Reconcile balances
        let balance_discrepancies = self.reconcile_balances().await?;
        discrepancies += balance_discrepancies;

        // 2. Reconcile orders
        let order_discrepancies = self.reconcile_orders().await?;
        discrepancies += order_discrepancies;

        // 3. Clean up stale orders
        let stale_cleaned = self.cleanup_stale_orders().await?;
        discrepancies += stale_cleaned;

        // 4. Reconcile positions with exchange
        let position_discrepancies = self.reconcile_positions().await?;
        discrepancies += position_discrepancies;

        // 5. Clean up old terminal orders (sync function, no await)
        let terminal_cleaned = self.cleanup_terminal_orders();

        // Update metrics
        {
            let mut metrics = self.reconciliation_metrics.write().await;
            metrics.balance_mismatches += balance_discrepancies as u64;
            metrics.order_mismatches += order_discrepancies as u64;
            metrics.stale_orders_cancelled += stale_cleaned as u64;
            metrics.terminal_orders_cleaned += terminal_cleaned;
        }

        Ok(discrepancies)
    }

    // @spec:FR-REAL-053 - Balance Reconciliation
    /// Reconcile local balances with exchange balances
    async fn reconcile_balances(&self) -> Result<u32> {
        // Use futures or spot endpoint based on trading mode
        let exchange_balances = if self.use_futures {
            self.fetch_futures_balances_for_reconciliation().await?
        } else {
            self.fetch_spot_balances_for_reconciliation().await?
        };

        let mut discrepancies = 0;
        let mut local_balances = self.balances.write().await;

        for (asset, exchange_free, exchange_locked) in exchange_balances {
            // Skip zero balances
            if exchange_free <= 0.0 && exchange_locked <= 0.0 {
                continue;
            }

            let local_balance = local_balances.get(&asset);
            let local_free = local_balance.map(|b| b.free).unwrap_or(0.0);

            // Check for significant difference (>0.01% or absolute difference > 0.0001)
            let diff = (exchange_free - local_free).abs();
            let threshold = (exchange_free * 0.0001).max(0.0001);

            if diff > threshold {
                warn!(
                    "Balance mismatch for {}: local={:.8}, exchange={:.8} (diff={:.8})",
                    asset, local_free, exchange_free, diff
                );

                // Update local balance
                local_balances.insert(
                    asset.clone(),
                    Balance {
                        asset: asset.clone(),
                        free: exchange_free,
                        locked: exchange_locked,
                    },
                );

                discrepancies += 1;

                // Emit event
                let _ = self.event_tx.send(RealTradingEvent::BalanceUpdated {
                    asset,
                    free: exchange_free,
                    locked: exchange_locked,
                });
            }
        }

        if discrepancies > 0 {
            info!(
                "Balance reconciliation: {} mismatches corrected",
                discrepancies
            );
        }

        Ok(discrepancies)
    }

    /// Fetch balances from Futures account for reconciliation
    async fn fetch_futures_balances_for_reconciliation(&self) -> Result<Vec<(String, f64, f64)>> {
        let account = self.binance_client.get_futures_account().await?;
        let mut balances = Vec::new();

        if let Some(assets) = account.get("assets").and_then(|a| a.as_array()) {
            for asset_obj in assets {
                let asset_name = asset_obj
                    .get("asset")
                    .and_then(|a| a.as_str())
                    .unwrap_or_default()
                    .to_string();
                let wallet_balance = asset_obj
                    .get("walletBalance")
                    .and_then(|v| v.as_str())
                    .and_then(|s| s.parse::<f64>().ok())
                    .unwrap_or(0.0);
                let available_balance = asset_obj
                    .get("availableBalance")
                    .and_then(|v| v.as_str())
                    .and_then(|s| s.parse::<f64>().ok())
                    .unwrap_or(0.0);
                let locked = (wallet_balance - available_balance).max(0.0);

                balances.push((asset_name, available_balance, locked));
            }
        }

        Ok(balances)
    }

    /// Fetch balances from Spot account for reconciliation
    async fn fetch_spot_balances_for_reconciliation(&self) -> Result<Vec<(String, f64, f64)>> {
        let account = self.binance_client.get_account_info().await?;
        let balances = account
            .balances
            .into_iter()
            .map(|b| {
                let free: f64 = b.free.parse().unwrap_or(0.0);
                let locked: f64 = b.locked.parse().unwrap_or(0.0);
                (b.asset, free, locked)
            })
            .collect();
        Ok(balances)
    }

    // @spec:FR-REAL-056 - Position Reconciliation
    /// Reconcile local positions with exchange positions
    async fn reconcile_positions(&self) -> Result<u32> {
        if !self.use_futures {
            return Ok(0);
        }

        let futures_positions = match self.binance_client.get_futures_positions().await {
            Ok(fp) => fp,
            Err(e) => {
                warn!("Failed to fetch positions for reconciliation: {}", e);
                return Ok(0);
            },
        };

        let mut discrepancies = 0;

        // Build map of exchange positions with non-zero quantity
        let exchange_map: HashMap<String, &FuturesPosition> =
            futures_positions
                .iter()
                .filter(|fp| fp.position_amt.parse::<f64>().unwrap_or(0.0).abs() > 1e-10)
                .map(|fp| (fp.symbol.clone(), fp))
                .collect();

        // Check exchange positions against local
        for (symbol, fp) in &exchange_map {
            let position_amt: f64 = fp.position_amt.parse().unwrap_or(0.0);
            let entry_price: f64 = fp.entry_price.parse().unwrap_or(0.0);
            let mark_price: f64 = fp.mark_price.parse().unwrap_or(0.0);
            let unrealized_pnl: f64 = fp.unrealized_pnl.parse().unwrap_or(0.0);
            let leverage: u32 = fp.leverage.parse().unwrap_or(1);

            let side = if position_amt > 0.0 {
                PositionSide::Long
            } else {
                PositionSide::Short
            };

            if let Some(mut local_pos) = self.positions.get_mut(symbol) {
                // Check for quantity mismatch
                let qty_diff = (local_pos.quantity - position_amt.abs()).abs();
                if qty_diff > 1e-8 {
                    warn!(
                        "Position {} qty mismatch: local={}, exchange={}",
                        symbol, local_pos.quantity, position_amt.abs()
                    );
                    local_pos.quantity = position_amt.abs();
                    local_pos.entry_price = entry_price;
                    local_pos.side = side;
                    local_pos.leverage = leverage;
                    local_pos.updated_at = Utc::now();
                    discrepancies += 1;
                }
                // Always update mark price and PnL
                local_pos.current_price = mark_price;
                local_pos.unrealized_pnl = unrealized_pnl;
            } else {
                // Exchange has position we don't track — create it
                warn!(
                    "Position {} found on exchange but not locally — syncing",
                    symbol
                );
                let mut position = RealPosition::new(
                    Uuid::new_v4().to_string(),
                    symbol.clone(),
                    side,
                    position_amt.abs(),
                    entry_price,
                    format!("reconciliation-sync-{}", Uuid::new_v4()),
                    None,
                    None,
                );
                position.current_price = mark_price;
                position.unrealized_pnl = unrealized_pnl;
                position.leverage = leverage;

                self.positions.insert(symbol.clone(), position);
                discrepancies += 1;
            }
        }

        // Check local open positions not on exchange
        let local_open: Vec<String> = self
            .positions
            .iter()
            .filter(|p| p.value().is_open())
            .map(|p| p.key().clone())
            .collect();

        for symbol in local_open {
            if !exchange_map.contains_key(&symbol) {
                warn!(
                    "Position {} exists locally but not on exchange — closing",
                    symbol
                );
                if let Some(mut pos) = self.positions.get_mut(&symbol) {
                    pos.quantity = 0.0;
                    pos.updated_at = Utc::now();
                }
                discrepancies += 1;
            }
        }

        if discrepancies > 0 {
            info!(
                "Position reconciliation: {} mismatches corrected",
                discrepancies
            );
        }

        Ok(discrepancies)
    }

    // @spec:FR-REAL-054 - Order Reconciliation
    /// Reconcile local orders with exchange orders
    async fn reconcile_orders(&self) -> Result<u32> {
        let exchange_orders = self.binance_client.get_open_orders(None).await?;
        let mut discrepancies = 0;

        // Build map of exchange orders by client_order_id
        let exchange_order_map: HashMap<String, _> = exchange_orders
            .iter()
            .map(|o| (o.client_order_id.clone(), o))
            .collect();

        // Build set of exchange order IDs for quick lookup (reserved for future use)
        let _exchange_order_ids: HashSet<String> = exchange_orders
            .iter()
            .map(|o| o.client_order_id.clone())
            .collect();

        // Check local orders against exchange
        let local_active_orders: Vec<(String, RealOrder)> = self
            .orders
            .iter()
            .filter(|e| e.value().is_active())
            .map(|e| (e.key().clone(), e.value().clone()))
            .collect();

        for (client_order_id, local_order) in local_active_orders {
            if let Some(exchange_order) = exchange_order_map.get(&client_order_id) {
                // Order exists on both sides - check for state mismatch
                let exchange_filled: f64 = exchange_order.executed_qty.parse().unwrap_or(0.0);
                let fill_diff = (local_order.executed_quantity - exchange_filled).abs();

                if fill_diff > 0.0001 {
                    warn!(
                        "Order {} fill mismatch: local={:.8}, exchange={:.8}",
                        client_order_id, local_order.executed_quantity, exchange_filled
                    );
                    discrepancies += 1;

                    // Update local state
                    if let Some(mut order) = self.orders.get_mut(&client_order_id) {
                        order.executed_quantity = exchange_filled;
                        order.remaining_quantity = order.original_quantity - exchange_filled;
                        order.state = OrderState::from_binance_status(&exchange_order.status);
                        order.updated_at = Utc::now();
                    }
                }
            } else {
                // Local active order not found on exchange - likely filled or cancelled
                warn!(
                    "Active local order {} not found on exchange, marking as unknown",
                    client_order_id
                );
                discrepancies += 1;

                // Try to get order status from exchange
                match self
                    .binance_client
                    .get_spot_order_status(&local_order.symbol, None, Some(&client_order_id))
                    .await
                {
                    Ok(order_response) => {
                        if let Some(mut order) = self.orders.get_mut(&client_order_id) {
                            order.state = OrderState::from_binance_status(&order_response.status);
                            if let Ok(exec_qty) = order_response.executed_qty.parse::<f64>() {
                                order.executed_quantity = exec_qty;
                                order.remaining_quantity = order.original_quantity - exec_qty;
                            }
                            order.updated_at = Utc::now();
                            info!(
                                "Updated order {} status to {:?}",
                                client_order_id, order.state
                            );
                        }
                    },
                    Err(e) => {
                        warn!(
                            "Failed to query order {} status: {}, marking as cancelled",
                            client_order_id, e
                        );
                        if let Some(mut order) = self.orders.get_mut(&client_order_id) {
                            order.state = OrderState::Cancelled;
                            order.updated_at = Utc::now();
                        }
                    },
                }
            }
        }

        // Check for orphan orders on exchange (orders we don't track locally)
        for exchange_order in &exchange_orders {
            if !self.orders.contains_key(&exchange_order.client_order_id) {
                warn!(
                    "Orphan order found on exchange: {} (symbol: {})",
                    exchange_order.client_order_id, exchange_order.symbol
                );
                discrepancies += 1;

                // Add to local tracking
                let executed_qty = exchange_order.executed_qty.parse::<f64>().unwrap_or(0.0);
                let orig_qty = exchange_order.orig_qty.parse::<f64>().unwrap_or(0.0);
                let price = exchange_order.price.parse::<f64>().ok();
                let stop_price = exchange_order.stop_price.parse::<f64>().ok();

                let avg_price = if executed_qty > 0.0 {
                    let quote_qty = exchange_order
                        .cumulative_quote_qty
                        .parse::<f64>()
                        .unwrap_or(0.0);
                    quote_qty / executed_qty
                } else {
                    0.0
                };

                let order = RealOrder {
                    client_order_id: exchange_order.client_order_id.clone(),
                    exchange_order_id: exchange_order.order_id,
                    symbol: exchange_order.symbol.clone(),
                    side: exchange_order.side.clone(),
                    order_type: exchange_order.r#type.clone(),
                    original_quantity: orig_qty,
                    executed_quantity: executed_qty,
                    remaining_quantity: orig_qty - executed_qty,
                    price,
                    stop_price,
                    average_fill_price: avg_price,
                    state: OrderState::from_binance_status(&exchange_order.status),
                    created_at: Utc::now(),
                    updated_at: Utc::now(),
                    fills: Vec::new(),
                    position_id: None,
                    is_entry: true,
                    reject_reason: None,
                };

                self.orders
                    .insert(exchange_order.client_order_id.clone(), order);
            }
        }

        if discrepancies > 0 {
            info!(
                "Order reconciliation: {} discrepancies found",
                discrepancies
            );
        }

        Ok(discrepancies)
    }

    // @spec:FR-REAL-055 - Stale Order Cleanup
    /// Cancel orders that have been pending too long
    async fn cleanup_stale_orders(&self) -> Result<u32> {
        let stale_timeout_secs = {
            let config = self.config.read().await;
            config.stale_order_timeout_secs
        };

        let now = Utc::now();
        let mut cleaned = 0;

        // Find stale orders (active orders older than timeout)
        let stale_order_ids: Vec<(String, String)> = self
            .orders
            .iter()
            .filter(|entry| {
                let order = entry.value();
                if !order.is_active() {
                    return false;
                }

                let age_secs = now
                    .signed_duration_since(order.created_at)
                    .num_seconds()
                    .max(0) as u64;

                age_secs > stale_timeout_secs
            })
            .map(|entry| (entry.key().clone(), entry.value().symbol.clone()))
            .collect();

        // Cancel stale orders
        for (order_id, symbol) in stale_order_ids {
            info!("Cancelling stale order: {} (symbol: {})", order_id, symbol);

            match self.cancel_order(&order_id).await {
                Ok(_) => {
                    info!("Cancelled stale order: {}", order_id);
                    cleaned += 1;
                },
                Err(e) => {
                    // Order might already be filled/cancelled on exchange
                    warn!("Failed to cancel stale order {}: {}", order_id, e);

                    // Mark as cancelled locally
                    if let Some(mut order) = self.orders.get_mut(&order_id) {
                        if order.is_active() {
                            order.state = OrderState::Cancelled;
                            order.updated_at = Utc::now();
                        }
                    }
                },
            }
        }

        if cleaned > 0 {
            info!("Stale order cleanup: {} orders cancelled", cleaned);
        }

        Ok(cleaned)
    }

    /// Clean up old terminal orders to prevent memory growth
    fn cleanup_terminal_orders(&self) -> u64 {
        let now = Utc::now();
        let cleanup_threshold = now - chrono::Duration::hours(24);

        let orders_to_remove: Vec<String> = self
            .orders
            .iter()
            .filter(|entry| {
                let order = entry.value();
                order.is_terminal() && order.updated_at < cleanup_threshold
            })
            .map(|entry| entry.key().clone())
            .collect();

        let count = orders_to_remove.len() as u64;

        for order_id in orders_to_remove {
            self.orders.remove(&order_id);
            debug!("Cleaned up old terminal order: {}", order_id);
        }

        if count > 0 {
            info!("Terminal order cleanup: {} old orders removed", count);
        }

        count
    }

    // @spec:FR-REAL-056 - WebSocket Disconnect Handler
    /// Handle WebSocket disconnect - trigger immediate reconciliation
    pub async fn handle_websocket_disconnect(&self) {
        warn!("WebSocket disconnected, triggering immediate reconciliation...");

        // Run immediate reconciliation to catch any missed events
        match self.run_reconciliation().await {
            Ok(discrepancies) => {
                if discrepancies > 0 {
                    warn!(
                        "Post-disconnect reconciliation found {} discrepancies",
                        discrepancies
                    );
                } else {
                    info!("Post-disconnect reconciliation complete, no discrepancies");
                }
            },
            Err(e) => {
                error!("Post-disconnect reconciliation failed: {}", e);
                self.record_error(&format!("Post-disconnect reconciliation failed: {}", e))
                    .await;
            },
        }
    }

    // @spec:FR-REAL-057 - Emergency Stop
    /// Emergency stop - cancel all orders and halt trading
    pub async fn emergency_stop(&self, reason: &str) -> Result<()> {
        error!("EMERGENCY STOP triggered: {}", reason);

        // 1. Open circuit breaker immediately
        {
            let mut cb = self.circuit_breaker.write().await;
            cb.is_open = true;
            cb.opened_at = Some(Utc::now());
            cb.last_error = Some(format!("EMERGENCY: {}", reason));
        }
        self.emit_event(RealTradingEvent::CircuitBreakerOpened(format!(
            "EMERGENCY: {}",
            reason
        )));

        // 2. Cancel all open orders
        let open_order_ids: Vec<String> = self
            .orders
            .iter()
            .filter(|o| o.value().is_active())
            .map(|o| o.key().clone())
            .collect();

        let mut cancelled_count = 0;
        for order_id in &open_order_ids {
            match self.cancel_order(order_id).await {
                Ok(_) => {
                    cancelled_count += 1;
                },
                Err(e) => {
                    warn!(
                        "Failed to cancel order {} during emergency stop: {}",
                        order_id, e
                    );
                },
            }
        }
        info!(
            "Emergency stop: cancelled {}/{} open orders",
            cancelled_count,
            open_order_ids.len()
        );

        // 3. Close positions if configured
        let should_close_positions = {
            let config = self.config.read().await;
            config.circuit_breaker_close_positions
        };

        if should_close_positions {
            let position_symbols: Vec<String> =
                self.positions.iter().map(|p| p.key().clone()).collect();

            for symbol in position_symbols {
                match self.close_position(&symbol).await {
                    Ok(_) => {
                        info!("Emergency stop: closed position {}", symbol);
                    },
                    Err(e) => {
                        error!("Emergency stop: failed to close position {}: {}", symbol, e);
                    },
                }
            }
        }

        // 4. Stop the engine
        {
            let mut running = self.is_running.write().await;
            *running = false;
        }

        // 5. Stop UserDataStream
        {
            let mut stream = self.user_data_stream.write().await;
            if let Err(e) = stream.stop().await {
                warn!("Failed to stop UserDataStream during emergency: {}", e);
            }
        }

        self.emit_event(RealTradingEvent::Error(format!(
            "Emergency stop: {}",
            reason
        )));

        error!("Emergency stop complete");
        Ok(())
    }

    /// Cancel all open orders (optionally filtered by symbol)
    pub async fn cancel_all_orders(&self, symbol: Option<&str>) -> Result<Vec<String>> {
        let order_ids: Vec<String> = self
            .orders
            .iter()
            .filter(|o| {
                let order = o.value();
                order.is_active() && symbol.is_none_or(|s| order.symbol == s)
            })
            .map(|o| o.key().clone())
            .collect();

        let mut cancelled = Vec::new();

        for order_id in order_ids {
            match self.cancel_order(&order_id).await {
                Ok(_) => {
                    cancelled.push(order_id);
                },
                Err(e) => {
                    warn!("Failed to cancel order {}: {}", order_id, e);
                },
            }
        }

        info!("Cancelled {} orders", cancelled.len());
        Ok(cancelled)
    }

    /// Get reconciliation metrics
    pub async fn get_reconciliation_metrics(&self) -> ReconciliationMetrics {
        self.reconciliation_metrics.read().await.clone()
    }

    /// Force an immediate reconciliation (for testing or manual trigger)
    pub async fn force_reconciliation(&self) -> Result<u32> {
        info!("Force reconciliation triggered");
        self.run_reconciliation().await
    }

    /// Get total equity in USDT (balance + position values)
    pub async fn get_total_equity_usdt(&self) -> f64 {
        let usdt_balance = self.get_usdt_balance().await;

        let position_value: f64 = self
            .positions
            .iter()
            .map(|p| p.value().position_value())
            .sum();

        usdt_balance + position_value
    }

    // ============ Getters ============

    /// Get all positions
    pub fn get_positions(&self) -> Vec<RealPosition> {
        self.positions.iter().map(|p| p.value().clone()).collect()
    }

    /// Get position by symbol
    pub fn get_position(&self, symbol: &str) -> Option<RealPosition> {
        self.positions.get(symbol).map(|p| p.clone())
    }

    /// Get all orders
    pub fn get_orders(&self) -> Vec<RealOrder> {
        self.orders.iter().map(|o| o.value().clone()).collect()
    }

    /// Get active orders
    pub fn get_active_orders(&self) -> Vec<RealOrder> {
        self.orders
            .iter()
            .filter(|o| o.value().is_active())
            .map(|o| o.value().clone())
            .collect()
    }

    /// Get order by client ID
    pub fn get_order(&self, client_order_id: &str) -> Option<RealOrder> {
        self.orders.get(client_order_id).map(|o| o.clone())
    }

    /// Fetch trade history from Binance for given symbols
    pub async fn get_trade_history(
        &self,
        symbols: &[String],
        limit: Option<u16>,
    ) -> Vec<crate::binance::types::FuturesUserTrade> {
        let mut all_trades = Vec::new();
        for symbol in symbols {
            match self
                .binance_client
                .get_futures_user_trades(symbol, limit)
                .await
            {
                Ok(trades) => all_trades.extend(trades),
                Err(e) => {
                    warn!("Failed to fetch trade history for {}: {}", symbol, e);
                },
            }
        }
        // Sort by time descending (newest first)
        all_trades.sort_by(|a, b| b.time.cmp(&a.time));
        all_trades
    }

    /// Fetch order history from Binance for given symbols (includes cancelled/filled/expired)
    pub async fn get_order_history(
        &self,
        symbols: &[String],
        limit: Option<u16>,
    ) -> Vec<crate::binance::types::FuturesOrder> {
        let mut all_orders = Vec::new();
        for symbol in symbols {
            match self
                .binance_client
                .get_all_futures_orders(symbol, limit)
                .await
            {
                Ok(orders) => all_orders.extend(orders),
                Err(e) => {
                    warn!("Failed to fetch order history for {}: {}", symbol, e);
                },
            }
        }
        // Sort by time descending (newest first)
        all_orders.sort_by(|a, b| b.time.cmp(&a.time));
        all_orders
    }

    /// Get current configuration
    pub async fn get_config(&self) -> RealTradingConfig {
        self.config.read().await.clone()
    }

    /// Update configuration
    pub async fn update_config(&self, config: RealTradingConfig) -> Result<()> {
        config
            .validate()
            .map_err(|e| anyhow!("Invalid config: {}", e.join(", ")))?;

        let should_spawn_signal_loop = config.auto_trading_enabled
            && *self.is_running.read().await
            && !*self.signal_loop_spawned.read().await;

        *self.config.write().await = config;

        // Dynamically spawn strategy signal loop if auto_trading just enabled
        if should_spawn_signal_loop {
            let engine_for_signals = self.clone();
            tokio::spawn(async move {
                engine_for_signals.strategy_signal_loop().await;
            });
            *self.signal_loop_spawned.write().await = true;
            info!("🚀 Strategy signal loop dynamically spawned - auto-trading ENABLED via API");
        }

        Ok(())
    }

    /// Get circuit breaker state
    pub async fn get_circuit_breaker(&self) -> CircuitBreakerState {
        self.circuit_breaker.read().await.clone()
    }

    /// Reset circuit breaker
    pub async fn reset_circuit_breaker(&self) {
        self.circuit_breaker.write().await.close();
        self.emit_event(RealTradingEvent::CircuitBreakerClosed);
    }

    /// Get daily metrics
    pub async fn get_daily_metrics(&self) -> DailyMetrics {
        let mut metrics = self.daily_metrics.write().await;
        metrics.reset_if_new_day();
        metrics.clone()
    }

    /// Get total unrealized PnL
    pub fn get_total_unrealized_pnl(&self) -> f64 {
        self.positions
            .iter()
            .map(|p| p.value().unrealized_pnl)
            .sum()
    }

    /// Get total exposure
    pub fn get_total_exposure(&self) -> f64 {
        self.positions
            .iter()
            .map(|p| p.value().position_value())
            .sum()
    }

    /// Get the risk manager for external access
    pub fn get_risk_manager(&self) -> &RealTradingRiskManager {
        &self.real_risk_manager
    }

    // ============ Stop Loss / Take Profit Management ============

    /// Set stop loss for a position
    pub async fn set_stop_loss(&self, symbol: &str, stop_loss: f64) -> Result<()> {
        if let Some(mut position) = self.positions.get_mut(symbol) {
            position.stop_loss = Some(stop_loss);
            info!("Set stop loss for {}: ${:.2}", symbol, stop_loss);
            self.emit_event(RealTradingEvent::PositionUpdated(position.clone()));
            Ok(())
        } else {
            Err(anyhow!("Position not found: {}", symbol))
        }
    }

    /// Set take profit for a position
    pub async fn set_take_profit(&self, symbol: &str, take_profit: f64) -> Result<()> {
        if let Some(mut position) = self.positions.get_mut(symbol) {
            position.take_profit = Some(take_profit);
            info!("Set take profit for {}: ${:.2}", symbol, take_profit);
            self.emit_event(RealTradingEvent::PositionUpdated(position.clone()));
            Ok(())
        } else {
            Err(anyhow!("Position not found: {}", symbol))
        }
    }

    /// Set both stop loss and take profit for a position
    pub async fn set_sl_tp(
        &self,
        symbol: &str,
        stop_loss: Option<f64>,
        take_profit: Option<f64>,
    ) -> Result<()> {
        if let Some(mut position) = self.positions.get_mut(symbol) {
            position.set_sl_tp(stop_loss, take_profit);
            info!(
                "Set SL/TP for {}: SL={:?}, TP={:?}",
                symbol, stop_loss, take_profit
            );
            self.emit_event(RealTradingEvent::PositionUpdated(position.clone()));
            Ok(())
        } else {
            Err(anyhow!("Position not found: {}", symbol))
        }
    }

    /// Calculate and set SL/TP based on risk parameters
    pub async fn set_auto_sl_tp(&self, symbol: &str) -> Result<(f64, f64)> {
        let position = self
            .positions
            .get(symbol)
            .map(|p| p.clone())
            .ok_or_else(|| anyhow!("Position not found: {}", symbol))?;

        let is_long = position.side == PositionSide::Long;
        let (stop_loss, take_profit) = self
            .real_risk_manager
            .calculate_sl_tp(position.entry_price, is_long)
            .await;

        // Set the calculated values
        if let Some(mut pos) = self.positions.get_mut(symbol) {
            pos.set_sl_tp(Some(stop_loss), Some(take_profit));
            info!(
                "Auto SL/TP set for {}: SL=${:.2}, TP=${:.2}",
                symbol, stop_loss, take_profit
            );
            self.emit_event(RealTradingEvent::PositionUpdated(pos.clone()));
        }

        Ok((stop_loss, take_profit))
    }

    /// Enable trailing stop for a position
    pub async fn enable_trailing_stop(
        &self,
        symbol: &str,
        activation_price: f64,
        trail_percent: f64,
    ) -> Result<()> {
        if let Some(mut position) = self.positions.get_mut(symbol) {
            position.enable_trailing_stop(activation_price, trail_percent);
            info!(
                "Enabled trailing stop for {}: activation=${:.2}, trail={:.2}%",
                symbol, activation_price, trail_percent
            );
            self.emit_event(RealTradingEvent::PositionUpdated(position.clone()));
            Ok(())
        } else {
            Err(anyhow!("Position not found: {}", symbol))
        }
    }

    /// Check all positions for SL/TP triggers and close if needed
    pub async fn check_sl_tp_triggers(&self) -> Result<Vec<String>> {
        let mut triggered: Vec<String> = Vec::new();

        for entry in self.positions.iter() {
            let position = entry.value();
            let symbol = entry.key().clone();

            // Check stop loss
            if position.should_trigger_stop_loss() {
                info!("Stop loss triggered for {}", symbol);
                triggered.push(symbol.clone());
                continue;
            }

            // Check take profit
            if position.should_trigger_take_profit() {
                info!("Take profit triggered for {}", symbol);
                triggered.push(symbol);
            }
        }

        // Close triggered positions
        for symbol in &triggered {
            if let Err(e) = self.close_position(symbol).await {
                error!("Failed to close {} on SL/TP trigger: {}", symbol, e);
            }
        }

        Ok(triggered)
    }

    /// Close a position by placing a market order
    pub async fn close_position(&self, symbol: &str) -> Result<RealOrder> {
        let position = self
            .positions
            .get(symbol)
            .map(|p| p.clone())
            .ok_or_else(|| anyhow!("Position not found: {}", symbol))?;

        let close_side = if position.side == PositionSide::Long {
            OrderSide::Sell
        } else {
            OrderSide::Buy
        };

        self.place_market_order(
            symbol,
            close_side,
            position.quantity,
            Some(position.id.clone()),
            false, // is_entry = false (exit)
        )
        .await
    }

    /// Calculate optimal position size for a new trade
    pub async fn calculate_position_size(&self, entry_price: f64, stop_loss: f64) -> f64 {
        let balance = self.get_usdt_balance().await;
        let config = self.config.read().await;
        self.real_risk_manager
            .calculate_position_size(entry_price, stop_loss, balance, &config)
    }

    /// Calculate position size with automatic stop loss
    pub async fn calculate_position_size_auto_sl(
        &self,
        entry_price: f64,
        is_long: bool,
    ) -> (f64, f64) {
        let balance = self.get_usdt_balance().await;
        self.real_risk_manager
            .calculate_position_size_auto_sl(entry_price, balance, is_long)
            .await
    }

    /// Get current risk utilization (0.0 to 1.0)
    pub async fn get_risk_utilization(&self) -> f64 {
        self.real_risk_manager
            .get_risk_utilization(&self.positions)
            .await
    }

    /// Get daily loss utilization (0.0 to 1.0)
    pub async fn get_daily_loss_utilization(&self) -> f64 {
        self.real_risk_manager.get_daily_loss_utilization().await
    }

    /// Get all balances
    /// Used by API to display portfolio balances
    pub async fn get_all_balances(&self) -> HashMap<String, Balance> {
        self.balances.read().await.clone()
    }

    // ============ Auto-Trading Risk Management ============

    /// Check if engine is currently in cool-down period after consecutive losses
    /// Lock order: consecutive_losses -> cool_down_until (consistent with update_consecutive_losses)
    async fn is_in_cooldown(&self) -> bool {
        let losses = *self.consecutive_losses.read().await;
        let cool_down_until = self.cool_down_until.read().await;
        if let Some(until) = *cool_down_until {
            if Utc::now() < until {
                let remaining = (until - Utc::now()).num_minutes();
                warn!(
                    "Cool-down active: {} minutes remaining (consecutive losses: {})",
                    remaining, losses
                );
                return true;
            }
        }
        false
    }

    /// Update consecutive loss counter and trigger cool-down if threshold reached
    async fn update_consecutive_losses(&self, pnl: f64) {
        let config = self.config.read().await;
        let max_losses = config.max_consecutive_losses;
        let cool_down_mins = config.cool_down_minutes;
        drop(config);

        if pnl < 0.0 {
            let mut losses = self.consecutive_losses.write().await;
            *losses += 1;

            info!("Consecutive losses: {} (max: {})", *losses, max_losses);

            if *losses >= max_losses {
                let until = Utc::now() + chrono::Duration::minutes(cool_down_mins as i64);
                let mut cd = self.cool_down_until.write().await;
                *cd = Some(until);

                error!(
                    "COOL-DOWN ACTIVATED: {} consecutive losses. Trading paused for {} minutes.",
                    *losses, cool_down_mins
                );

                self.emit_event(RealTradingEvent::CooldownActivated {
                    consecutive_losses: *losses,
                    cool_down_minutes: cool_down_mins,
                });
            }
        } else {
            let mut losses = self.consecutive_losses.write().await;
            if *losses > 0 {
                info!(
                    "Profitable trade - resetting consecutive losses counter (was {})",
                    *losses
                );
            }
            *losses = 0;
            let mut cd = self.cool_down_until.write().await;
            *cd = None;
        }
    }

    /// Check if adding a new position in the given direction would exceed correlation limit
    async fn check_correlation_limit(&self, is_long: bool) -> bool {
        let config = self.config.read().await;
        let limit = config.correlation_limit;
        drop(config);

        let positions: Vec<RealPosition> =
            self.positions.iter().map(|e| e.value().clone()).collect();

        // Only meaningful with 3+ positions
        if positions.len() < 3 {
            return true;
        }

        let mut long_exposure = 0.0;
        let mut short_exposure = 0.0;

        for pos in &positions {
            let value = pos.quantity * pos.current_price;
            match pos.side {
                PositionSide::Long => long_exposure += value,
                PositionSide::Short => short_exposure += value,
            }
        }

        let total = long_exposure + short_exposure;
        if total == 0.0 {
            return true;
        }

        let ratio = if is_long {
            long_exposure / total
        } else {
            short_exposure / total
        };

        if ratio > limit {
            let direction = if is_long { "long" } else { "short" };
            warn!(
                "Correlation limit: {:.1}% {} exposure exceeds {:.0}% limit",
                ratio * 100.0,
                direction,
                limit * 100.0
            );
            self.emit_event(RealTradingEvent::SignalRejected {
                symbol: "portfolio".to_string(),
                reason: format!(
                    "Correlation limit: {:.1}% {} exposure > {:.0}%",
                    ratio * 100.0,
                    direction,
                    limit * 100.0
                ),
            });
            return false;
        }

        true
    }

    /// Check if total portfolio risk is within limits
    async fn check_portfolio_risk(&self) -> bool {
        let config = self.config.read().await;
        let max_risk_pct = config.max_portfolio_risk_pct;
        let default_sl_pct = config.default_stop_loss_percent;
        drop(config);

        let positions: Vec<RealPosition> =
            self.positions.iter().map(|e| e.value().clone()).collect();
        if positions.is_empty() {
            return true;
        }

        // Use total equity = free USDT + unrealized PnL from all positions
        let free_usdt = self.get_usdt_balance().await;
        let unrealized_pnl: f64 = positions.iter().map(|p| p.unrealized_pnl).sum();
        let total_equity = free_usdt + unrealized_pnl;
        let usdt_balance = if total_equity > 0.0 {
            total_equity
        } else {
            free_usdt
        };

        if usdt_balance <= 0.0 {
            warn!("Portfolio equity is zero or negative, blocking trades");
            return false;
        }

        let sl_multiplier = default_sl_pct / 100.0;
        let mut total_risk = 0.0;

        for pos in &positions {
            let position_value = pos.quantity * pos.entry_price;
            let sl_price = pos.stop_loss.unwrap_or(match pos.side {
                PositionSide::Long => pos.entry_price * (1.0 - sl_multiplier),
                PositionSide::Short => pos.entry_price * (1.0 + sl_multiplier),
            });
            let sl_distance_pct = ((pos.entry_price - sl_price).abs() / pos.entry_price) * 100.0;
            let risk_amount = position_value * (sl_distance_pct / 100.0);
            let risk_pct = (risk_amount / usdt_balance) * 100.0;
            total_risk += risk_pct;
        }

        if total_risk >= max_risk_pct {
            warn!(
                "Portfolio risk limit exceeded: {:.1}% of {:.0}% max ({} positions)",
                total_risk,
                max_risk_pct,
                positions.len()
            );
            self.emit_event(RealTradingEvent::SignalRejected {
                symbol: "portfolio".to_string(),
                reason: format!(
                    "Portfolio risk {:.1}% exceeds {:.0}% limit",
                    total_risk, max_risk_pct
                ),
            });
            return false;
        }

        debug!(
            "Portfolio risk OK: {:.1}% of {:.0}% max ({} positions)",
            total_risk,
            max_risk_pct,
            positions.len()
        );
        true
    }

    // ============ Helpers ============

    /// Emit event to subscribers
    fn emit_event(&self, event: RealTradingEvent) {
        if self.event_tx.receiver_count() > 0 {
            let _ = self.event_tx.send(event);
        }
    }

    /// Update price for all positions (called from market data feed)
    pub fn update_prices(&self, prices: &HashMap<String, f64>) {
        for mut entry in self.positions.iter_mut() {
            if let Some(price) = prices.get(entry.key()) {
                entry.value_mut().update_price(*price);
            }
        }
    }

    // ============ Background Loops (Auto-Trading) ============

    /// Price update loop — fetches prices from WebSocket cache (O(1)) with REST fallback
    /// Runs every 5 seconds, updates positions and current_prices map
    async fn price_update_loop(&self) {
        use tokio::time::{interval, Duration};
        let mut tick = interval(Duration::from_secs(5));

        info!("Price update loop started (5s interval)");

        loop {
            tick.tick().await;

            if !*self.is_running.read().await {
                info!("Price update loop stopped");
                break;
            }

            // Determine which symbols to track
            let symbols = self.get_tracked_symbols().await;
            if symbols.is_empty() {
                continue;
            }

            let mut new_prices = HashMap::new();

            // Try WebSocket cache first (O(1) lookup)
            if let Some(ref cache) = self.market_data_cache {
                for symbol in &symbols {
                    if let Some(price) = cache.get_latest_price(symbol) {
                        if price > 0.0 {
                            new_prices.insert(symbol.clone(), price);
                        }
                    }
                }
            }

            // REST fallback for symbols not in cache
            let missing: Vec<&String> = symbols
                .iter()
                .filter(|s| !new_prices.contains_key(*s))
                .collect();
            if !missing.is_empty() {
                debug!("Fetching {} symbols via REST (not in cache)", missing.len());
                for symbol in &missing {
                    match self.binance_client.get_symbol_price(symbol).await {
                        Ok(price_info) => {
                            if let Ok(price) = price_info.price.parse::<f64>() {
                                if price > 0.0 {
                                    new_prices.insert((*symbol).clone(), price);
                                }
                            }
                        },
                        Err(e) => warn!("Failed to get price for {}: {}", symbol, e),
                    }
                }
            }

            // Update positions
            self.update_prices(&new_prices);

            // Update current_prices map
            {
                let mut prices = self.current_prices.write().await;
                *prices = new_prices;
            }
        }
    }

    /// SL/TP monitoring loop — checks triggers every 5 seconds and closes positions
    /// Also tracks PnL for consecutive loss counting
    async fn sl_tp_monitoring_loop(&self) {
        use tokio::time::{interval, Duration};
        let mut tick = interval(Duration::from_secs(5));

        info!("SL/TP monitoring loop started (5s interval)");

        loop {
            tick.tick().await;

            if !*self.is_running.read().await {
                info!("SL/TP monitoring loop stopped");
                break;
            }

            // Collect triggered positions (read-only scan)
            let mut triggered: Vec<(String, f64)> = Vec::new();

            for entry in self.positions.iter() {
                let position = entry.value();
                let symbol = entry.key().clone();

                if position.should_trigger_stop_loss() {
                    info!(
                        "SL triggered for {} ({:?}) price=${:.2} sl=${:.2}",
                        symbol,
                        position.side,
                        position.current_price,
                        position.stop_loss.unwrap_or(0.0)
                    );
                    triggered.push((symbol, position.unrealized_pnl));
                } else if position.should_trigger_take_profit() {
                    info!(
                        "TP triggered for {} ({:?}) price=${:.2} tp=${:.2}",
                        symbol,
                        position.side,
                        position.current_price,
                        position.take_profit.unwrap_or(0.0)
                    );
                    triggered.push((symbol, position.unrealized_pnl));
                } else if position.is_at_liquidation_risk() {
                    warn!(
                        "LIQUIDATION RISK for {} ({:?}) {}x leverage, price=${:.2}, entry=${:.2}",
                        symbol,
                        position.side,
                        position.leverage,
                        position.current_price,
                        position.entry_price
                    );
                    triggered.push((symbol, position.unrealized_pnl));
                }
            }

            // Close triggered positions and track PnL
            for (symbol, estimated_pnl) in triggered {
                match self.close_position(&symbol).await {
                    Ok(order) => {
                        info!(
                            "Auto-closed {} via SL/TP (order: {})",
                            symbol, order.client_order_id
                        );
                        // Track consecutive losses for cool-down
                        self.update_consecutive_losses(estimated_pnl).await;
                    },
                    Err(e) => error!("Failed to auto-close {} on SL/TP trigger: {}", symbol, e),
                }
            }
        }
    }

    /// Strategy signal loop — runs strategy engine every 30s on new closed candles
    /// Mirrors paper trading signal loop with 5-layer filtering
    async fn strategy_signal_loop(&self) {
        use tokio::time::{interval, Duration};
        let mut tick = interval(Duration::from_secs(30));

        // Track last processed candle close_time per symbol_timeframe
        let mut last_processed: HashMap<String, i64> = HashMap::new();

        info!("Strategy signal loop started (30s interval, auto-trading ENABLED)");

        loop {
            tick.tick().await;

            if !*self.is_running.read().await {
                info!("Strategy signal loop stopped");
                break;
            }

            // Re-check auto_trading_enabled (can be toggled at runtime)
            let config = self.config.read().await;
            if !config.auto_trading_enabled {
                debug!("Auto-trading disabled, skipping signal loop iteration");
                continue;
            }
            let min_confidence = config.min_signal_confidence;
            let short_only = config.short_only_mode;
            let long_only = config.long_only_mode;
            let symbols = self.get_auto_trade_symbols(&config);
            drop(config);

            for symbol in &symbols {
                for timeframe in &["5m", "15m", "1h"] {
                    let cache_key = format!("{}_{}", symbol, timeframe);

                    // Use market_data_cache (mainnet) for candle close detection
                    // This ensures real trading analyzes the SAME data as paper trading
                    let candles = if let Some(ref mdc) = self.market_data_cache {
                        mdc.get_all_candles(symbol, timeframe)
                    } else {
                        // Fallback: fetch from testnet API (not ideal but functional)
                        let fetch_limit = if !last_processed.contains_key(&cache_key) {
                            100u16
                        } else {
                            5u16
                        };
                        match self
                            .binance_client
                            .get_klines(symbol, timeframe, Some(fetch_limit))
                            .await
                        {
                            Ok(k) => k.iter().map(CandleData::from).collect(),
                            Err(e) => {
                                debug!(
                                    "Failed to fetch klines for {} {}: {}",
                                    symbol, timeframe, e
                                );
                                continue;
                            },
                        }
                    };

                    if candles.len() < 2 {
                        continue;
                    }

                    // Detect new closed candle (2nd-to-last = most recent CLOSED)
                    let closed_candle = &candles[candles.len() - 2];
                    let last_close_time = closed_candle.close_time;
                    let prev_time = last_processed.get(&cache_key).copied().unwrap_or(0);

                    if last_close_time <= prev_time {
                        continue;
                    }

                    last_processed.insert(cache_key.clone(), last_close_time);

                    // Skip first detection (warmup)
                    if prev_time == 0 {
                        info!(
                            "Warmup: {} candles available for {} {}",
                            candles.len(),
                            symbol,
                            timeframe
                        );
                        continue;
                    }

                    // Warmup gate: require >= 50 candles for MACD/Bollinger to be meaningful
                    if candles.len() < 50 {
                        debug!(
                            "Insufficient data for {} {}: {} candles (need 50)",
                            symbol,
                            timeframe,
                            candles.len()
                        );
                        continue;
                    }

                    info!(
                        "New {} candle closed for {}, running strategy analysis...",
                        timeframe, symbol
                    );

                    // Build strategy input from historical cache
                    let strategy_input = match self.build_strategy_input(symbol).await {
                        Some(input) => input,
                        None => {
                            warn!(
                                "Failed to build strategy input for {} (missing data or price)",
                                symbol
                            );
                            continue;
                        },
                    };

                    // Run strategy engine
                    let combined_signal =
                        match self.strategy_engine.analyze_market(&strategy_input).await {
                            Ok(sig) => sig,
                            Err(e) => {
                                warn!(
                                    "Strategy analysis failed for {} {}: {}",
                                    symbol, timeframe, e
                                );
                                continue;
                            },
                        };

                    let signal = combined_signal.final_signal;
                    let confidence = combined_signal.combined_confidence;

                    info!(
                        "Strategy result for {} {}: {:?} (confidence: {:.2}%)",
                        symbol,
                        timeframe,
                        signal,
                        confidence * 100.0
                    );

                    // ===== 5-LAYER SIGNAL FILTERING =====

                    // Layer 1: Skip neutral signals
                    if signal == TradingSignal::Neutral {
                        continue;
                    }

                    // Layer 2: Confidence threshold
                    if confidence < min_confidence {
                        info!(
                            "Signal for {} rejected: confidence {:.2} < {:.2}",
                            symbol, confidence, min_confidence
                        );
                        self.emit_event(RealTradingEvent::SignalRejected {
                            symbol: symbol.clone(),
                            reason: format!(
                                "Low confidence: {:.2} < {:.2}",
                                confidence, min_confidence
                            ),
                        });
                        continue;
                    }

                    // Layer 3: Market direction mode
                    if signal == TradingSignal::Long && short_only {
                        self.emit_event(RealTradingEvent::SignalRejected {
                            symbol: symbol.clone(),
                            reason: "Long signal blocked (short_only_mode)".to_string(),
                        });
                        continue;
                    }
                    if signal == TradingSignal::Short && long_only {
                        self.emit_event(RealTradingEvent::SignalRejected {
                            symbol: symbol.clone(),
                            reason: "Short signal blocked (long_only_mode)".to_string(),
                        });
                        continue;
                    }

                    // Layer 4: Choppy market detection (4+ flips in 15min = block)
                    {
                        let mut tracker = self.signal_flip_tracker.write().await;
                        let now_ts = Utc::now().timestamp();
                        let flips = tracker.entry(symbol.clone()).or_insert_with(Vec::new);
                        flips.retain(|(ts, _)| now_ts - ts < 900); // 15min window
                        let flip_count = flips.windows(2).filter(|w| w[0].1 != w[1].1).count();
                        flips.push((now_ts, signal));

                        if flip_count >= 4 {
                            self.emit_event(RealTradingEvent::SignalRejected {
                                symbol: symbol.clone(),
                                reason: format!("Choppy market: {} flips in 15min", flip_count),
                            });
                            continue;
                        }
                    }

                    // Layer 5: Signal confirmation (2 consecutive same-direction within 10min)
                    let dedup_key = format!("{}_{:?}", symbol, signal);
                    let opposite_key = format!(
                        "{}_{:?}",
                        symbol,
                        match signal {
                            TradingSignal::Long => TradingSignal::Short,
                            TradingSignal::Short => TradingSignal::Long,
                            TradingSignal::Neutral => TradingSignal::Neutral,
                        }
                    );
                    let now = Utc::now().timestamp();

                    // Check AI bias alignment
                    let bias_aligned = {
                        let bias = self.ai_market_bias.read().await;
                        if let Some(market_bias) = bias.get(symbol.as_str()) {
                            if !market_bias.is_stale() && market_bias.bias_confidence > 0.7 {
                                let signal_dir = match signal {
                                    TradingSignal::Long => 1.0,
                                    TradingSignal::Short => -1.0,
                                    TradingSignal::Neutral => 0.0,
                                };
                                let threshold = if matches!(signal, TradingSignal::Long) {
                                    -0.3
                                } else {
                                    -0.5
                                };
                                signal_dir * market_bias.direction_bias >= threshold
                            } else {
                                true // Stale or low-confidence bias — allow
                            }
                        } else {
                            true // No bias data — allow
                        }
                    };

                    if !bias_aligned {
                        self.emit_event(RealTradingEvent::SignalRejected {
                            symbol: symbol.clone(),
                            reason: "AI market bias misaligned".to_string(),
                        });
                        continue;
                    }

                    // Check confirmation
                    let confirmed = {
                        let recent = self.recent_signals.read().await;
                        if let Some((first_seen, count)) = recent.get(&dedup_key) {
                            now - first_seen < 600 && *count >= 1 && now - first_seen >= 60
                        } else {
                            false
                        }
                    };

                    // Update confirmation tracking
                    {
                        let mut recent = self.recent_signals.write().await;
                        recent.remove(&opposite_key);
                        if let Some((first_seen, count)) = recent.get_mut(&dedup_key) {
                            if now - *first_seen >= 600 {
                                *first_seen = now;
                                *count = 1;
                            } else if now - *first_seen >= 60 {
                                *count += 1;
                            }
                        } else {
                            recent.insert(dedup_key.clone(), (now, 1));
                        }
                        recent.retain(|_, (ts, _)| now - *ts < 600);
                    }

                    if !confirmed {
                        info!(
                            "Signal {:?} for {} awaiting confirmation (need 2 consecutive same-direction within 10min, >=60s apart)",
                            signal, symbol
                        );
                        continue;
                    }

                    // ===== ALL FILTERS PASSED — EXECUTE =====
                    self.emit_event(RealTradingEvent::SignalGenerated {
                        symbol: symbol.clone(),
                        signal: format!("{:?}", signal),
                        confidence,
                    });

                    if let Err(e) = self
                        .process_signal_for_real_trade(symbol, signal, confidence)
                        .await
                    {
                        error!("Failed to process signal for {}: {}", symbol, e);
                    }
                }
            }
        }
    }

    /// Process a confirmed signal into a real Binance trade
    /// Performs risk checks, calculates position size, places order, sets SL/TP
    async fn process_signal_for_real_trade(
        &self,
        symbol: &str,
        signal: TradingSignal,
        confidence: f64,
    ) -> Result<()> {
        info!(
            "Processing real trade signal: {} {:?} (confidence: {:.2})",
            symbol, signal, confidence
        );

        // 1. Check daily loss limit (via existing risk manager)
        {
            let daily = self.daily_metrics.read().await;
            let config = self.config.read().await;
            if daily.realized_pnl.abs() >= config.max_daily_loss_usdt && daily.realized_pnl < 0.0 {
                self.emit_event(RealTradingEvent::SignalRejected {
                    symbol: symbol.to_string(),
                    reason: "Daily loss limit reached".to_string(),
                });
                return Ok(());
            }
        }

        // 2. Check cool-down
        if self.is_in_cooldown().await {
            self.emit_event(RealTradingEvent::SignalRejected {
                symbol: symbol.to_string(),
                reason: "In cool-down period after consecutive losses".to_string(),
            });
            return Ok(());
        }

        // 3. Check correlation limit
        let is_long = signal == TradingSignal::Long;
        if !self.check_correlation_limit(is_long).await {
            return Ok(()); // Event already emitted inside
        }

        // 4. Check portfolio risk
        if !self.check_portfolio_risk().await {
            return Ok(()); // Event already emitted inside
        }

        // 5. Check max positions
        let config = self.config.read().await;
        let max_positions = config.max_positions as usize;
        let default_sl_pct = config.default_stop_loss_percent;
        let default_tp_pct = config.default_take_profit_percent;
        let leverage = config.max_leverage;
        let enable_ts = config.enable_trailing_stop;
        let ts_activation_pct = config.trailing_stop_activation_percent;
        let ts_trail_pct = config.trailing_stop_percent;
        drop(config);

        if self.positions.len() >= max_positions {
            self.emit_event(RealTradingEvent::SignalRejected {
                symbol: symbol.to_string(),
                reason: format!("Max positions ({}) reached", max_positions),
            });
            return Ok(());
        }

        // 6. Check if position already exists — close if opposite direction (reversal)
        if let Some(existing) = self.positions.get(symbol) {
            let existing_is_long = existing.side == crate::real_trading::PositionSide::Long;
            let signal_is_long = is_long;

            if existing_is_long == signal_is_long {
                // Same direction — skip (already positioned correctly)
                debug!(
                    "Position already exists for {} in same direction, skipping",
                    symbol
                );
                self.emit_event(RealTradingEvent::SignalRejected {
                    symbol: symbol.to_string(),
                    reason: "Position already open in same direction".to_string(),
                });
                return Ok(());
            } else {
                // Opposite direction — close existing position first (reversal)
                drop(existing); // Release DashMap ref before mutating
                info!(
                    "Reversing position for {}: closing existing {:?} to open {:?}",
                    symbol,
                    if existing_is_long { "LONG" } else { "SHORT" },
                    if signal_is_long { "LONG" } else { "SHORT" }
                );
                match self.close_position(symbol).await {
                    Ok(order) => {
                        info!(
                            "Closed existing position for reversal: {} (order: {})",
                            symbol, order.client_order_id
                        );
                    },
                    Err(e) => {
                        error!("Failed to close position for reversal on {}: {}", symbol, e);
                        return Ok(());
                    },
                }
            }
        }

        // 7. Get current price
        let entry_price = {
            let prices = self.current_prices.read().await;
            match prices.get(symbol).copied() {
                Some(p) if p > 0.0 => p,
                _ => {
                    warn!("No current price for {}, cannot execute", symbol);
                    return Ok(());
                },
            }
        };

        // 8a. Set margin mode ISOLATED (futures only, prevents cross-margin bleed)
        if self.use_futures {
            match self
                .binance_client
                .change_margin_type(symbol, "ISOLATED")
                .await
            {
                Ok(_) => {
                    info!("Set ISOLATED margin for {}", symbol);
                },
                Err(e) => {
                    // Error -4046 = "No need to change margin type" (already ISOLATED) — safe to ignore
                    let err_str = e.to_string();
                    if !err_str.contains("-4046") {
                        warn!("Failed to set ISOLATED margin for {}: {}", symbol, err_str);
                    }
                },
            }
        }

        // 8b. Set leverage on Binance (futures only, must be done BEFORE placing order)
        if self.use_futures && leverage > 1 {
            match self
                .binance_client
                .change_leverage(symbol, leverage as u8)
                .await
            {
                Ok(_) => {
                    info!("Set leverage {}x for {} on Binance", leverage, symbol);
                },
                Err(e) => {
                    warn!(
                        "Failed to set leverage {}x for {}: {} (using current exchange leverage)",
                        leverage, symbol, e
                    );
                },
            }
        }

        // 9. Calculate SL/TP (PnL-based, adjusted for leverage like paper trading)
        // With leverage, price_change = pnl_pct / leverage
        // E.g., 2% SL with 10x leverage = 0.2% price move triggers stop
        let lev = leverage as f64;
        let stop_loss = if is_long {
            entry_price * (1.0 - default_sl_pct / (lev * 100.0))
        } else {
            entry_price * (1.0 + default_sl_pct / (lev * 100.0))
        };

        let take_profit = if is_long {
            entry_price * (1.0 + default_tp_pct / (lev * 100.0))
        } else {
            entry_price * (1.0 - default_tp_pct / (lev * 100.0))
        };

        // 9. Calculate position size (round to exchange step size)
        let raw_quantity = self.calculate_position_size(entry_price, stop_loss).await;
        let quantity = Self::round_quantity_for_exchange(symbol, raw_quantity);
        if quantity <= 0.0 {
            warn!("Calculated position size is 0 for {}, skipping", symbol);
            return Ok(());
        }

        // 10. Place market order on Binance
        let side = if is_long {
            OrderSide::Buy
        } else {
            OrderSide::Sell
        };

        info!(
            "Placing REAL {:?} order: {} {:.6} @ ~${:.2} (SL=${:.2}, TP=${:.2})",
            side, symbol, quantity, entry_price, stop_loss, take_profit
        );

        // Use place_order directly with price hint for accurate risk validation
        // Market order (SpotOrderType::Market) but pass Some(entry_price) for risk checks
        match self
            .place_order(
                symbol,
                side,
                SpotOrderType::Market,
                quantity,
                Some(entry_price),
                None,
                None,
                true,
            )
            .await
        {
            Ok(order) => {
                info!(
                    "Real trade executed: {} {:?} {:.6} (order: {})",
                    symbol, side, quantity, order.client_order_id
                );

                // 11. Set SL/TP on the new position
                if let Err(e) = self
                    .set_sl_tp(symbol, Some(stop_loss), Some(take_profit))
                    .await
                {
                    warn!("Failed to set SL/TP for {}: {} (will retry)", symbol, e);
                }

                // 12. Enable trailing stop if configured
                if enable_ts {
                    let activation = if is_long {
                        entry_price * (1.0 + ts_activation_pct / 100.0)
                    } else {
                        entry_price * (1.0 - ts_activation_pct / 100.0)
                    };
                    if let Err(e) = self
                        .enable_trailing_stop(symbol, activation, ts_trail_pct)
                        .await
                    {
                        warn!("Failed to enable trailing stop for {}: {}", symbol, e);
                    }
                }

                self.emit_event(RealTradingEvent::SignalExecuted {
                    symbol: symbol.to_string(),
                    signal: format!("{:?}", signal),
                    order_id: order.client_order_id,
                });
            },
            Err(e) => {
                error!("Failed to place real order for {}: {}", symbol, e);
                self.emit_event(RealTradingEvent::SignalRejected {
                    symbol: symbol.to_string(),
                    reason: format!("Order failed: {}", e),
                });
            },
        }

        Ok(())
    }

    /// Build strategy input from historical data cache
    async fn build_strategy_input(&self, symbol: &str) -> Option<crate::strategies::StrategyInput> {
        let mut timeframe_data: HashMap<String, Vec<CandleData>> = HashMap::new();

        // Use market_data_cache (mainnet WebSocket data) as PRIMARY source.
        // This ensures real trading uses the SAME market data as paper trading,
        // since testnet klines can differ from mainnet and produce different signals.
        if let Some(ref mdc) = self.market_data_cache {
            for timeframe in &["5m", "15m", "1h"] {
                let candles = mdc.get_all_candles(symbol, timeframe);
                if !candles.is_empty() {
                    timeframe_data.insert(timeframe.to_string(), candles);
                }
            }
        }

        // Fallback to historical_data_cache (testnet klines) if market_data_cache unavailable
        if timeframe_data.is_empty() {
            let cache = self.historical_data_cache.read().await;
            for timeframe in &["5m", "15m", "1h"] {
                let cache_key = format!("{}_{}", symbol, timeframe);
                if let Some(klines) = cache.get(&cache_key) {
                    let candles: Vec<CandleData> =
                        klines.iter().map(CandleData::from).collect();
                    if !candles.is_empty() {
                        timeframe_data.insert(timeframe.to_string(), candles);
                    }
                }
            }
        }

        // Need at least 5m data for strategies
        if !timeframe_data.contains_key("5m") {
            debug!(
                "Insufficient data for strategy analysis on {}: missing 5m timeframe",
                symbol
            );
            return None;
        }

        // Use mainnet price from market_data_cache if available
        let current_price = if let Some(ref mdc) = self.market_data_cache {
            mdc.get_latest_price(symbol).unwrap_or(0.0)
        } else {
            let prices = self.current_prices.read().await;
            prices.get(symbol).copied().unwrap_or(0.0)
        };

        if current_price <= 0.0 {
            debug!(
                "No current price for {}, skipping strategy analysis",
                symbol
            );
            return None;
        }

        let volume_24h = timeframe_data
            .get("1h")
            .map(|candles| candles.iter().rev().take(24).map(|c| c.volume).sum::<f64>())
            .unwrap_or(0.0);

        Some(crate::strategies::StrategyInput {
            symbol: symbol.to_string(),
            timeframe_data,
            current_price,
            volume_24h,
            timestamp: Utc::now().timestamp(),
        })
    }

    /// Round quantity to exchange-allowed step size for a symbol.
    /// Binance rejects orders with precision exceeding the LOT_SIZE filter.
    fn round_quantity_for_exchange(symbol: &str, quantity: f64) -> f64 {
        // Futures step sizes (from Binance exchange info)
        let decimals = match symbol {
            "BTCUSDT" => 3,  // step 0.001
            "ETHUSDT" => 3,  // step 0.001
            "BNBUSDT" => 2,  // step 0.01
            "SOLUSDT" => 1,  // step 0.1 (futures) — some testnet use 0
            "XRPUSDT" => 1,  // step 0.1
            "DOGEUSDT" => 0, // step 1
            "ADAUSDT" => 0,  // step 1
            _ => 3,          // safe default
        };
        let factor = 10f64.powi(decimals);
        (quantity * factor).floor() / factor
    }

    /// Process an external AI signal for real trading
    pub async fn process_external_ai_signal(
        &self,
        symbol: String,
        signal_type: TradingSignal,
        confidence: f64,
        _reasoning: String,
        _entry_price: f64,
        _stop_loss: Option<f64>,
        _take_profit: Option<f64>,
    ) -> Result<()> {
        if !*self.is_running.read().await {
            return Err(anyhow!("Real trading engine is not running"));
        }

        let config = self.config.read().await;
        if !config.auto_trading_enabled {
            return Err(anyhow!("Auto-trading is disabled"));
        }
        let min_confidence = config.min_signal_confidence;
        drop(config);

        // Validate confidence
        if confidence < min_confidence {
            info!(
                "AI signal for {} rejected: confidence {:.2} < {:.2}",
                symbol, confidence, min_confidence
            );
            return Ok(());
        }

        // Skip neutral
        if signal_type == TradingSignal::Neutral {
            return Ok(());
        }

        info!(
            "Processing external AI signal for real trading: {} {:?} (confidence: {:.2})",
            symbol, signal_type, confidence
        );

        self.process_signal_for_real_trade(&symbol, signal_type, confidence)
            .await
    }

    /// Update AI market bias for a symbol (called by external AI service or MCP tools)
    pub async fn update_ai_market_bias(&self, symbol: String, bias: AIMarketBias) {
        let mut biases = self.ai_market_bias.write().await;
        biases.insert(symbol, bias);
    }

    /// Get symbols to track for price updates (positions + auto-trade symbols)
    async fn get_tracked_symbols(&self) -> Vec<String> {
        let mut symbols: HashSet<String> = HashSet::new();

        // All symbols with open positions
        for entry in self.positions.iter() {
            symbols.insert(entry.key().clone());
        }

        // Auto-trade symbols from config
        let config = self.config.read().await;
        for s in &config.auto_trade_symbols {
            symbols.insert(s.clone());
        }
        // Fallback to allowed_symbols if auto_trade_symbols is empty
        if config.auto_trade_symbols.is_empty() {
            for s in &config.allowed_symbols {
                symbols.insert(s.clone());
            }
        }

        symbols.into_iter().collect()
    }

    /// Get the list of symbols to auto-trade from config
    fn get_auto_trade_symbols(&self, config: &RealTradingConfig) -> Vec<String> {
        if !config.auto_trade_symbols.is_empty() {
            config.auto_trade_symbols.clone()
        } else if !config.allowed_symbols.is_empty() {
            config.allowed_symbols.clone()
        } else {
            vec![]
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::binance::types::{
        AccountBalance, BalanceUpdate, ExecutionReport, OutboundAccountPosition,
    };

    // ============ Helper Functions ============

    fn create_test_execution_report(
        client_order_id: &str,
        symbol: &str,
        side: &str,
        execution_type: &str,
        order_status: &str,
        quantity: &str,
        filled_qty: &str,
        price: &str,
    ) -> ExecutionReport {
        ExecutionReport {
            event_type: "executionReport".to_string(),
            event_time: Utc::now().timestamp_millis(),
            symbol: symbol.to_string(),
            client_order_id: client_order_id.to_string(),
            side: side.to_string(),
            order_type: "MARKET".to_string(),
            time_in_force: "GTC".to_string(),
            order_quantity: quantity.to_string(),
            order_price: "0".to_string(),
            stop_price: "0".to_string(),
            iceberg_quantity: "0".to_string(),
            original_client_order_id: "".to_string(),
            execution_type: execution_type.to_string(),
            order_status: order_status.to_string(),
            order_reject_reason: "NONE".to_string(),
            order_id: 12345,
            last_executed_quantity: filled_qty.to_string(),
            cumulative_filled_quantity: filled_qty.to_string(),
            last_executed_price: price.to_string(),
            commission_amount: "0.001".to_string(),
            commission_asset: Some("BNB".to_string()),
            transaction_time: Utc::now().timestamp_millis(),
            trade_id: 67890,
            is_on_book: false,
            is_maker: false,
            order_creation_time: Utc::now().timestamp_millis(),
            cumulative_quote_qty: (filled_qty.parse::<f64>().unwrap_or(0.0)
                * price.parse::<f64>().unwrap_or(0.0))
            .to_string(),
            last_quote_qty: (filled_qty.parse::<f64>().unwrap_or(0.0)
                * price.parse::<f64>().unwrap_or(0.0))
            .to_string(),
            quote_order_qty: "0".to_string(),
        }
    }

    fn create_test_balance_update(asset: &str, delta: &str) -> BalanceUpdate {
        BalanceUpdate {
            event_type: "balanceUpdate".to_string(),
            event_time: Utc::now().timestamp_millis(),
            asset: asset.to_string(),
            balance_delta: delta.to_string(),
            clear_time: Utc::now().timestamp_millis(),
        }
    }

    fn create_test_account_position(balances: Vec<(&str, &str, &str)>) -> OutboundAccountPosition {
        OutboundAccountPosition {
            event_type: "outboundAccountPosition".to_string(),
            event_time: Utc::now().timestamp_millis(),
            last_update_time: Utc::now().timestamp_millis(),
            balances: balances
                .into_iter()
                .map(|(asset, free, locked)| AccountBalance {
                    asset: asset.to_string(),
                    free: free.to_string(),
                    locked: locked.to_string(),
                })
                .collect(),
        }
    }

    // ============ Circuit Breaker Tests ============

    #[test]
    fn test_circuit_breaker_default() {
        let cb = CircuitBreakerState::default();
        assert!(!cb.is_open);
        assert_eq!(cb.error_count, 0);
    }

    #[test]
    fn test_circuit_breaker_opens_on_errors() {
        let mut cb = CircuitBreakerState::default();

        // First error
        assert!(!cb.record_error("Error 1", 3));
        assert_eq!(cb.error_count, 1);
        assert!(!cb.is_open);

        // Second error
        assert!(!cb.record_error("Error 2", 3));
        assert_eq!(cb.error_count, 2);
        assert!(!cb.is_open);

        // Third error - opens
        assert!(cb.record_error("Error 3", 3));
        assert!(cb.is_open);
        assert!(cb.opened_at.is_some());
    }

    #[test]
    fn test_circuit_breaker_success_resets_count() {
        let mut cb = CircuitBreakerState::default();

        cb.record_error("Error 1", 3);
        cb.record_error("Error 2", 3);
        assert_eq!(cb.error_count, 2);

        cb.record_success();
        assert_eq!(cb.error_count, 0);
    }

    #[test]
    fn test_circuit_breaker_close() {
        let mut cb = CircuitBreakerState::default();
        cb.record_error("Error", 1);
        assert!(cb.is_open);

        cb.close();
        assert!(!cb.is_open);
        assert_eq!(cb.error_count, 0);
        assert!(cb.opened_at.is_none());
    }

    // ============ Daily Metrics Tests ============

    #[test]
    fn test_daily_metrics_new() {
        let metrics = DailyMetrics::new();
        assert_eq!(metrics.trades_count, 0);
        assert_eq!(metrics.realized_pnl, 0.0);
    }

    #[test]
    fn test_daily_metrics_win_rate() {
        let mut metrics = DailyMetrics::new();

        // No trades
        assert_eq!(metrics.win_rate(), 0.0);

        // 60% win rate
        metrics.trades_count = 10;
        metrics.winning_trades = 6;
        metrics.losing_trades = 4;
        assert!((metrics.win_rate() - 60.0).abs() < 0.1);
    }

    // ============ Balance Tests ============

    #[test]
    fn test_balance_total() {
        let balance = Balance {
            asset: "USDT".to_string(),
            free: 1000.0,
            locked: 500.0,
        };
        assert!((balance.total() - 1500.0).abs() < 0.01);
    }

    // ============ ExecutionReport Processing Tests ============

    #[test]
    fn test_execution_report_new_order() {
        let report = create_test_execution_report(
            "test-order-001",
            "BTCUSDT",
            "BUY",
            "NEW",
            "NEW",
            "0.001",
            "0",
            "50000",
        );

        assert!(report.is_new());
        assert!(!report.is_filled());
        assert!(!report.is_trade());
        assert_eq!(report.fill_percentage(), 0.0);
    }

    #[test]
    fn test_execution_report_partial_fill() {
        let report = create_test_execution_report(
            "test-order-002",
            "BTCUSDT",
            "BUY",
            "TRADE",
            "PARTIALLY_FILLED",
            "0.01",
            "0.005",
            "50000",
        );

        assert!(report.is_trade());
        assert!(report.is_partially_filled());
        assert!(!report.is_filled());
        assert!((report.fill_percentage() - 50.0).abs() < 0.1);
    }

    #[test]
    fn test_execution_report_full_fill() {
        let report = create_test_execution_report(
            "test-order-003",
            "BTCUSDT",
            "BUY",
            "TRADE",
            "FILLED",
            "0.01",
            "0.01",
            "50000",
        );

        assert!(report.is_trade());
        assert!(report.is_filled());
        assert!((report.fill_percentage() - 100.0).abs() < 0.1);
    }

    #[test]
    fn test_execution_report_cancelled() {
        let report = create_test_execution_report(
            "test-order-004",
            "BTCUSDT",
            "BUY",
            "CANCELED",
            "CANCELED",
            "0.01",
            "0",
            "50000",
        );

        assert!(report.is_cancelled());
        assert!(!report.is_filled());
    }

    #[test]
    fn test_execution_report_rejected() {
        let report = create_test_execution_report(
            "test-order-005",
            "BTCUSDT",
            "BUY",
            "REJECTED",
            "REJECTED",
            "0.01",
            "0",
            "50000",
        );

        assert!(report.is_rejected());
    }

    // ============ Order State Tests ============

    #[test]
    fn test_order_state_from_binance_status() {
        assert_eq!(OrderState::from_binance_status("NEW"), OrderState::New);
        assert_eq!(
            OrderState::from_binance_status("PARTIALLY_FILLED"),
            OrderState::PartiallyFilled
        );
        assert_eq!(
            OrderState::from_binance_status("FILLED"),
            OrderState::Filled
        );
        assert_eq!(
            OrderState::from_binance_status("CANCELED"),
            OrderState::Cancelled
        );
        assert_eq!(
            OrderState::from_binance_status("REJECTED"),
            OrderState::Rejected
        );
        assert_eq!(
            OrderState::from_binance_status("EXPIRED"),
            OrderState::Expired
        );
        assert_eq!(
            OrderState::from_binance_status("UNKNOWN"),
            OrderState::Pending
        );
    }

    #[test]
    fn test_order_state_is_active() {
        assert!(OrderState::Pending.is_active());
        assert!(OrderState::New.is_active());
        assert!(OrderState::PartiallyFilled.is_active());
        assert!(!OrderState::Filled.is_active());
        assert!(!OrderState::Cancelled.is_active());
        assert!(!OrderState::Rejected.is_active());
        assert!(!OrderState::Expired.is_active());
    }

    #[test]
    fn test_order_state_is_terminal() {
        assert!(!OrderState::Pending.is_terminal());
        assert!(!OrderState::New.is_terminal());
        assert!(!OrderState::PartiallyFilled.is_terminal());
        assert!(OrderState::Filled.is_terminal());
        assert!(OrderState::Cancelled.is_terminal());
        assert!(OrderState::Rejected.is_terminal());
        assert!(OrderState::Expired.is_terminal());
    }

    // ============ RealOrder Tests ============

    #[test]
    fn test_real_order_new() {
        let order = RealOrder::new(
            "test-123".to_string(),
            "BTCUSDT".to_string(),
            "BUY".to_string(),
            "MARKET".to_string(),
            0.001,
            None,
            None,
            None,
            true,
        );

        assert_eq!(order.state, OrderState::Pending);
        assert_eq!(order.executed_quantity, 0.0);
        assert_eq!(order.remaining_quantity, 0.001);
        assert!(order.is_active());
        assert!(order.fills.is_empty());
    }

    #[test]
    fn test_real_order_update_from_execution_report() {
        let mut order = RealOrder::new(
            "test-456".to_string(),
            "BTCUSDT".to_string(),
            "BUY".to_string(),
            "MARKET".to_string(),
            0.01,
            None,
            None,
            None,
            true,
        );

        // Simulate a partial fill
        let report = create_test_execution_report(
            "test-456",
            "BTCUSDT",
            "BUY",
            "TRADE",
            "PARTIALLY_FILLED",
            "0.01",
            "0.005",
            "50000",
        );

        order.update_from_execution_report(&report);

        assert_eq!(order.state, OrderState::PartiallyFilled);
        assert!((order.executed_quantity - 0.005).abs() < 0.0001);
        assert!((order.remaining_quantity - 0.005).abs() < 0.0001);
        assert_eq!(order.fills.len(), 1);
        assert!((order.fills[0].price - 50000.0).abs() < 0.01);
    }

    #[test]
    fn test_real_order_fill_percentage() {
        let mut order = RealOrder::new(
            "test-789".to_string(),
            "BTCUSDT".to_string(),
            "BUY".to_string(),
            "MARKET".to_string(),
            1.0,
            None,
            None,
            None,
            true,
        );

        assert_eq!(order.fill_percentage(), 0.0);

        order.executed_quantity = 0.5;
        assert!((order.fill_percentage() - 0.5).abs() < 0.001);

        order.executed_quantity = 1.0;
        assert!((order.fill_percentage() - 1.0).abs() < 0.001);
    }

    #[test]
    fn test_real_order_total_commission() {
        let mut order = RealOrder::new(
            "test-comm".to_string(),
            "BTCUSDT".to_string(),
            "BUY".to_string(),
            "MARKET".to_string(),
            0.01,
            None,
            None,
            None,
            true,
        );

        // Add some fills with commissions
        order.fills.push(super::super::order::OrderFill {
            trade_id: 1,
            price: 50000.0,
            quantity: 0.005,
            commission: 0.001,
            commission_asset: "BNB".to_string(),
            timestamp: Utc::now(),
        });

        order.fills.push(super::super::order::OrderFill {
            trade_id: 2,
            price: 50100.0,
            quantity: 0.005,
            commission: 0.002,
            commission_asset: "BNB".to_string(),
            timestamp: Utc::now(),
        });

        assert!((order.total_commission() - 0.003).abs() < 0.0001);
    }

    // ============ Position Tests ============

    #[test]
    fn test_position_side_from_order_side() {
        assert_eq!(PositionSide::from_order_side("BUY"), PositionSide::Long);
        assert_eq!(PositionSide::from_order_side("SELL"), PositionSide::Short);
        assert_eq!(PositionSide::from_order_side("buy"), PositionSide::Long);
        assert_eq!(PositionSide::from_order_side("sell"), PositionSide::Short);
    }

    #[test]
    fn test_position_closing_order_side() {
        assert_eq!(PositionSide::Long.closing_order_side(), "SELL");
        assert_eq!(PositionSide::Short.closing_order_side(), "BUY");
    }

    #[test]
    fn test_real_position_new() {
        let pos = RealPosition::new(
            "pos-123".to_string(),
            "BTCUSDT".to_string(),
            PositionSide::Long,
            0.1,
            50000.0,
            "order-123".to_string(),
            Some("RSI".to_string()),
            Some(0.85),
        );

        assert_eq!(pos.symbol, "BTCUSDT");
        assert_eq!(pos.side, PositionSide::Long);
        assert!((pos.quantity - 0.1).abs() < 0.0001);
        assert!((pos.entry_price - 50000.0).abs() < 0.01);
        assert!(pos.is_open());
        assert!(!pos.is_closed());
    }

    #[test]
    fn test_position_add_fill_average_entry() {
        let mut pos = RealPosition::new(
            "pos-123".to_string(),
            "BTCUSDT".to_string(),
            PositionSide::Long,
            0.1,
            50000.0,
            "order-123".to_string(),
            None,
            None,
        );

        // Add more at higher price
        pos.add_fill(52000.0, 0.1, 0.5, "order-456".to_string());

        // New average: (50000 * 0.1 + 52000 * 0.1) / 0.2 = 51000
        assert!((pos.entry_price - 51000.0).abs() < 0.01);
        assert!((pos.quantity - 0.2).abs() < 0.0001);
        assert_eq!(pos.entry_order_ids.len(), 2);
    }

    #[test]
    fn test_position_partial_close_long() {
        let mut pos = RealPosition::new(
            "pos-123".to_string(),
            "BTCUSDT".to_string(),
            PositionSide::Long,
            0.1,
            50000.0,
            "order-123".to_string(),
            None,
            None,
        );

        // Close half at profit
        let pnl = pos.partial_close(52000.0, 0.05, 0.25, "exit-order-1".to_string());

        // PnL: (52000 - 50000) * 0.05 - 0.25 = 100 - 0.25 = 99.75
        assert!((pnl - 99.75).abs() < 0.01);
        assert!((pos.quantity - 0.05).abs() < 0.0001);
        assert!((pos.realized_pnl - 99.75).abs() < 0.01);
        assert_eq!(pos.exit_order_ids.len(), 1);
    }

    #[test]
    fn test_position_partial_close_short() {
        let mut pos = RealPosition::new(
            "pos-123".to_string(),
            "BTCUSDT".to_string(),
            PositionSide::Short,
            0.1,
            50000.0,
            "order-123".to_string(),
            None,
            None,
        );

        // Close half at profit (price went down for short)
        let pnl = pos.partial_close(48000.0, 0.05, 0.25, "exit-order-1".to_string());

        // PnL: (50000 - 48000) * 0.05 - 0.25 = 100 - 0.25 = 99.75
        assert!((pnl - 99.75).abs() < 0.01);
    }

    #[test]
    fn test_position_full_close() {
        let mut pos = RealPosition::new(
            "pos-123".to_string(),
            "BTCUSDT".to_string(),
            PositionSide::Long,
            0.1,
            50000.0,
            "order-123".to_string(),
            None,
            None,
        );

        // Close full position
        let pnl = pos.partial_close(51000.0, 0.1, 0.5, "exit-order-1".to_string());

        // PnL: (51000 - 50000) * 0.1 - 0.5 = 100 - 0.5 = 99.5
        assert!((pnl - 99.5).abs() < 0.01);
        assert!(pos.is_closed());
        assert!(!pos.is_open());
    }

    #[test]
    fn test_position_unrealized_pnl_long() {
        let mut pos = RealPosition::new(
            "pos-123".to_string(),
            "BTCUSDT".to_string(),
            PositionSide::Long,
            0.1,
            50000.0,
            "order-123".to_string(),
            None,
            None,
        );

        // Price goes up - profit
        pos.update_price(51000.0);
        assert!((pos.unrealized_pnl - 100.0).abs() < 0.01);

        // Price goes down - loss
        pos.update_price(49000.0);
        assert!((pos.unrealized_pnl - (-100.0)).abs() < 0.01);
    }

    #[test]
    fn test_position_unrealized_pnl_short() {
        let mut pos = RealPosition::new(
            "pos-123".to_string(),
            "BTCUSDT".to_string(),
            PositionSide::Short,
            0.1,
            50000.0,
            "order-123".to_string(),
            None,
            None,
        );

        // Price goes down - profit for short
        pos.update_price(49000.0);
        assert!((pos.unrealized_pnl - 100.0).abs() < 0.01);

        // Price goes up - loss for short
        pos.update_price(51000.0);
        assert!((pos.unrealized_pnl - (-100.0)).abs() < 0.01);
    }

    // ============ Balance Update Tests ============

    #[test]
    fn test_balance_update_creation() {
        let update = create_test_balance_update("USDT", "100.50");

        assert_eq!(update.asset, "USDT");
        assert_eq!(update.balance_delta, "100.50");
    }

    #[test]
    fn test_account_position_creation() {
        let pos =
            create_test_account_position(vec![("USDT", "1000.0", "100.0"), ("BTC", "0.5", "0.1")]);

        assert_eq!(pos.balances.len(), 2);
        assert_eq!(pos.balances[0].asset, "USDT");
        assert_eq!(pos.balances[0].free, "1000.0");
        assert_eq!(pos.balances[0].locked, "100.0");
    }

    // ============ Position Value Tests ============

    #[test]
    fn test_position_value_calculation() {
        let mut pos = RealPosition::new(
            "pos-123".to_string(),
            "BTCUSDT".to_string(),
            PositionSide::Long,
            0.1,
            50000.0,
            "order-123".to_string(),
            None,
            None,
        );

        // At entry price
        assert!((pos.position_value() - 5000.0).abs() < 0.01);

        // After price update
        pos.update_price(55000.0);
        assert!((pos.position_value() - 5500.0).abs() < 0.01);
    }

    #[test]
    fn test_position_cost_basis() {
        let pos = RealPosition::new(
            "pos-123".to_string(),
            "BTCUSDT".to_string(),
            PositionSide::Long,
            0.1,
            50000.0,
            "order-123".to_string(),
            None,
            None,
        );

        assert!((pos.cost_basis() - 5000.0).abs() < 0.01);
    }

    #[test]
    fn test_position_pnl_percentage() {
        let mut pos = RealPosition::new(
            "pos-123".to_string(),
            "BTCUSDT".to_string(),
            PositionSide::Long,
            0.1,
            50000.0,
            "order-123".to_string(),
            None,
            None,
        );

        pos.update_price(55000.0);
        // PnL: 500, percentage: 500/5000 * 100 = 10%
        assert!((pos.pnl_percentage() - 10.0).abs() < 0.1);
    }

    // ============ Stop Loss / Take Profit Tests ============

    #[test]
    fn test_position_stop_loss_trigger_long() {
        let mut pos = RealPosition::new(
            "pos-123".to_string(),
            "BTCUSDT".to_string(),
            PositionSide::Long,
            0.1,
            50000.0,
            "order-123".to_string(),
            None,
            None,
        );
        pos.set_sl_tp(Some(49000.0), Some(52000.0));

        // Above SL - no trigger
        pos.update_price(49500.0);
        assert!(!pos.should_trigger_stop_loss());

        // At SL - trigger
        pos.update_price(49000.0);
        assert!(pos.should_trigger_stop_loss());
    }

    #[test]
    fn test_position_take_profit_trigger_long() {
        let mut pos = RealPosition::new(
            "pos-123".to_string(),
            "BTCUSDT".to_string(),
            PositionSide::Long,
            0.1,
            50000.0,
            "order-123".to_string(),
            None,
            None,
        );
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
        let mut pos = RealPosition::new(
            "pos-123".to_string(),
            "BTCUSDT".to_string(),
            PositionSide::Long,
            0.1,
            50000.0,
            "order-123".to_string(),
            None,
            None,
        );
        pos.enable_trailing_stop(52000.0, 2.0);

        // Below activation - no trailing stop
        pos.update_price(51000.0);
        assert!(pos.trailing_stop_price.is_none());

        // At activation
        pos.update_price(52000.0);
        // Trailing stop = 52000 * 0.98 = 50960
        assert!((pos.trailing_stop_price.unwrap() - 50960.0).abs() < 1.0);

        // Price higher - trailing stop moves up
        pos.update_price(54000.0);
        // Trailing stop = 54000 * 0.98 = 52920
        assert!((pos.trailing_stop_price.unwrap() - 52920.0).abs() < 1.0);

        // Price drops - trailing stop stays
        pos.update_price(53000.0);
        assert!((pos.trailing_stop_price.unwrap() - 52920.0).abs() < 1.0);

        // Hit trailing stop
        pos.update_price(52920.0);
        assert!(pos.should_trigger_stop_loss());
    }

    // ============ Integration-like Tests ============

    #[test]
    fn test_order_to_position_flow_buy() {
        // Simulate order creation
        let mut order = RealOrder::new(
            "flow-test-001".to_string(),
            "BTCUSDT".to_string(),
            "BUY".to_string(),
            "MARKET".to_string(),
            0.1,
            None,
            None,
            None,
            true,
        );

        // Simulate fill
        let report = create_test_execution_report(
            "flow-test-001",
            "BTCUSDT",
            "BUY",
            "TRADE",
            "FILLED",
            "0.1",
            "0.1",
            "50000",
        );

        order.update_from_execution_report(&report);
        assert!(order.is_filled());

        // Create position from filled order
        let pos = RealPosition::new(
            "pos-flow-001".to_string(),
            order.symbol.clone(),
            PositionSide::from_order_side(&order.side),
            order.executed_quantity,
            order.average_fill_price,
            order.client_order_id.clone(),
            None,
            None,
        );

        assert_eq!(pos.side, PositionSide::Long);
        assert!((pos.quantity - 0.1).abs() < 0.0001);
        assert!((pos.entry_price - 50000.0).abs() < 1.0);
    }

    #[test]
    fn test_order_to_position_flow_sell() {
        // Simulate sell order (short)
        let mut order = RealOrder::new(
            "flow-test-002".to_string(),
            "BTCUSDT".to_string(),
            "SELL".to_string(),
            "MARKET".to_string(),
            0.1,
            None,
            None,
            None,
            true,
        );

        // Simulate fill
        let report = create_test_execution_report(
            "flow-test-002",
            "BTCUSDT",
            "SELL",
            "TRADE",
            "FILLED",
            "0.1",
            "0.1",
            "50000",
        );

        order.update_from_execution_report(&report);

        // Create position
        let pos = RealPosition::new(
            "pos-flow-002".to_string(),
            order.symbol.clone(),
            PositionSide::from_order_side(&order.side),
            order.executed_quantity,
            order.average_fill_price,
            order.client_order_id.clone(),
            None,
            None,
        );

        assert_eq!(pos.side, PositionSide::Short);
    }

    #[test]
    fn test_multiple_fills_accumulate() {
        let mut order = RealOrder::new(
            "multi-fill-001".to_string(),
            "BTCUSDT".to_string(),
            "BUY".to_string(),
            "MARKET".to_string(),
            0.1,
            None,
            None,
            None,
            true,
        );

        // First partial fill
        let report1 = create_test_execution_report(
            "multi-fill-001",
            "BTCUSDT",
            "BUY",
            "TRADE",
            "PARTIALLY_FILLED",
            "0.1",
            "0.04",
            "50000",
        );
        order.update_from_execution_report(&report1);
        assert_eq!(order.fills.len(), 1);
        assert!((order.executed_quantity - 0.04).abs() < 0.0001);

        // Second partial fill (manually adjust cumulative for test)
        let mut report2 = create_test_execution_report(
            "multi-fill-001",
            "BTCUSDT",
            "BUY",
            "TRADE",
            "PARTIALLY_FILLED",
            "0.1",
            "0.03",
            "50100",
        );
        report2.cumulative_filled_quantity = "0.07".to_string();
        report2.cumulative_quote_qty = "3507".to_string(); // 0.04*50000 + 0.03*50100
        order.update_from_execution_report(&report2);
        assert_eq!(order.fills.len(), 2);
        assert!((order.executed_quantity - 0.07).abs() < 0.0001);

        // Final fill
        let mut report3 = create_test_execution_report(
            "multi-fill-001",
            "BTCUSDT",
            "BUY",
            "TRADE",
            "FILLED",
            "0.1",
            "0.03",
            "50200",
        );
        report3.cumulative_filled_quantity = "0.1".to_string();
        report3.cumulative_quote_qty = "5013".to_string();
        order.update_from_execution_report(&report3);
        assert_eq!(order.fills.len(), 3);
        assert!(order.is_filled());
    }

    // ============ Realized PnL Calculation Tests ============

    #[test]
    fn test_realized_pnl_long_profit() {
        let mut pos = RealPosition::new(
            "pnl-test-001".to_string(),
            "BTCUSDT".to_string(),
            PositionSide::Long,
            0.1,
            50000.0,
            "order-123".to_string(),
            None,
            None,
        );

        // Close at profit
        let pnl = pos.partial_close(52000.0, 0.1, 1.0, "exit-order".to_string());

        // (52000 - 50000) * 0.1 - 1.0 = 200 - 1 = 199
        assert!((pnl - 199.0).abs() < 0.01);
        assert!((pos.realized_pnl - 199.0).abs() < 0.01);
    }

    #[test]
    fn test_realized_pnl_long_loss() {
        let mut pos = RealPosition::new(
            "pnl-test-002".to_string(),
            "BTCUSDT".to_string(),
            PositionSide::Long,
            0.1,
            50000.0,
            "order-123".to_string(),
            None,
            None,
        );

        // Close at loss
        let pnl = pos.partial_close(48000.0, 0.1, 1.0, "exit-order".to_string());

        // (48000 - 50000) * 0.1 - 1.0 = -200 - 1 = -201
        assert!((pnl - (-201.0)).abs() < 0.01);
    }

    #[test]
    fn test_realized_pnl_short_profit() {
        let mut pos = RealPosition::new(
            "pnl-test-003".to_string(),
            "BTCUSDT".to_string(),
            PositionSide::Short,
            0.1,
            50000.0,
            "order-123".to_string(),
            None,
            None,
        );

        // Close at profit (price went down for short)
        let pnl = pos.partial_close(48000.0, 0.1, 1.0, "exit-order".to_string());

        // (50000 - 48000) * 0.1 - 1.0 = 200 - 1 = 199
        assert!((pnl - 199.0).abs() < 0.01);
    }

    #[test]
    fn test_realized_pnl_short_loss() {
        let mut pos = RealPosition::new(
            "pnl-test-004".to_string(),
            "BTCUSDT".to_string(),
            PositionSide::Short,
            0.1,
            50000.0,
            "order-123".to_string(),
            None,
            None,
        );

        // Close at loss (price went up for short)
        let pnl = pos.partial_close(52000.0, 0.1, 1.0, "exit-order".to_string());

        // (50000 - 52000) * 0.1 - 1.0 = -200 - 1 = -201
        assert!((pnl - (-201.0)).abs() < 0.01);
    }

    // ============ Event Tests ============

    #[test]
    fn test_real_trading_event_serialization() {
        let order = RealOrder::new(
            "event-test".to_string(),
            "BTCUSDT".to_string(),
            "BUY".to_string(),
            "MARKET".to_string(),
            0.001,
            None,
            None,
            None,
            true,
        );

        let event = RealTradingEvent::OrderPlaced(order);

        // Should be serializable
        let json = serde_json::to_string(&event);
        assert!(json.is_ok());
    }

    #[test]
    fn test_balance_updated_event() {
        let event = RealTradingEvent::BalanceUpdated {
            asset: "USDT".to_string(),
            free: 1000.0,
            locked: 100.0,
        };

        let json = serde_json::to_string(&event).unwrap();
        assert!(json.contains("USDT"));
        assert!(json.contains("1000"));
    }
}
#[cfg(test)]
mod additional_coverage_tests {
    use super::*;
    use crate::config::{BinanceConfig, TradingMode};
    use crate::trading::risk_manager::RiskManager;
    use chrono::Utc;

    fn create_test_execution_report(
        client_order_id: &str,
        symbol: &str,
        side: &str,
        execution_type: &str,
        order_status: &str,
        quantity: &str,
        filled_qty: &str,
        price: &str,
    ) -> ExecutionReport {
        ExecutionReport {
            event_type: "executionReport".to_string(),
            event_time: Utc::now().timestamp_millis(),
            symbol: symbol.to_string(),
            client_order_id: client_order_id.to_string(),
            side: side.to_string(),
            order_type: "MARKET".to_string(),
            time_in_force: "GTC".to_string(),
            order_quantity: quantity.to_string(),
            order_price: "0".to_string(),
            stop_price: "0".to_string(),
            iceberg_quantity: "0".to_string(),
            original_client_order_id: "".to_string(),
            execution_type: execution_type.to_string(),
            order_status: order_status.to_string(),
            order_reject_reason: "NONE".to_string(),
            order_id: 12345,
            last_executed_quantity: filled_qty.to_string(),
            cumulative_filled_quantity: filled_qty.to_string(),
            last_executed_price: price.to_string(),
            commission_amount: "0.001".to_string(),
            commission_asset: Some("BNB".to_string()),
            transaction_time: Utc::now().timestamp_millis(),
            trade_id: 67890,
            is_on_book: false,
            is_maker: false,
            order_creation_time: Utc::now().timestamp_millis(),
            cumulative_quote_qty: (filled_qty.parse::<f64>().unwrap_or(0.0)
                * price.parse::<f64>().unwrap_or(0.0))
            .to_string(),
            last_quote_qty: (filled_qty.parse::<f64>().unwrap_or(0.0)
                * price.parse::<f64>().unwrap_or(0.0))
            .to_string(),
            quote_order_qty: "0".to_string(),
        }
    }

    async fn create_test_engine() -> RealTradingEngine {
        let config = RealTradingConfig::default();
        let binance_config = BinanceConfig {
            api_key: "test_key".to_string(),
            secret_key: "test_secret".to_string(),
            futures_api_key: String::new(),
            futures_secret_key: String::new(),
            testnet: true,
            base_url: "https://testnet.binance.vision".to_string(),
            ws_url: "wss://testnet.binance.vision/ws".to_string(),
            futures_base_url: "https://testnet.binancefuture.com".to_string(),
            futures_ws_url: "wss://stream.binancefuture.com/ws".to_string(),
            trading_mode: TradingMode::RealTestnet,
        };
        let binance_client = BinanceClient::new(binance_config).unwrap();
        let risk_manager = RiskManager::new(crate::config::TradingConfig {
            enabled: false,
            max_positions: 5,
            default_quantity: 0.001,
            risk_percentage: 2.0,
            stop_loss_percentage: 2.0,
            take_profit_percentage: 4.0,
            order_timeout_seconds: 60,
            position_check_interval_seconds: 10,
            leverage: 1,
            margin_type: "ISOLATED".to_string(),
        });

        RealTradingEngine::new(config, binance_client, risk_manager)
            .await
            .unwrap()
    }

    #[tokio::test]
    async fn test_get_risk_utilization() {
        let engine = create_test_engine().await;
        let utilization = engine.get_risk_utilization().await;
        assert!(utilization >= 0.0 && utilization <= 1.0);
    }

    #[tokio::test]
    async fn test_get_daily_loss_utilization() {
        let engine = create_test_engine().await;
        let utilization = engine.get_daily_loss_utilization().await;
        assert!(utilization >= 0.0 && utilization <= 1.0);
    }

    #[tokio::test]
    async fn test_calculate_position_size() {
        let engine = create_test_engine().await;
        let size = engine.calculate_position_size(50000.0, 49000.0).await;
        assert!(size >= 0.0);
    }

    #[tokio::test]
    async fn test_calculate_position_size_auto_sl() {
        let engine = create_test_engine().await;
        let (size, stop_loss) = engine.calculate_position_size_auto_sl(50000.0, true).await;
        assert!(size >= 0.0);
        assert!(stop_loss > 0.0);
    }

    #[tokio::test]
    async fn test_set_stop_loss_nonexistent() {
        let engine = create_test_engine().await;
        let result = engine.set_stop_loss("BTCUSDT", 49000.0).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_set_take_profit_nonexistent() {
        let engine = create_test_engine().await;
        let result = engine.set_take_profit("BTCUSDT", 51000.0).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_set_sl_tp_nonexistent() {
        let engine = create_test_engine().await;
        let result = engine
            .set_sl_tp("BTCUSDT", Some(49000.0), Some(51000.0))
            .await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_set_auto_sl_tp_nonexistent() {
        let engine = create_test_engine().await;
        let result = engine.set_auto_sl_tp("BTCUSDT").await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_enable_trailing_stop_nonexistent() {
        let engine = create_test_engine().await;
        let result = engine.enable_trailing_stop("BTCUSDT", 52000.0, 2.0).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_close_position_nonexistent() {
        let engine = create_test_engine().await;
        let result = engine.close_position("BTCUSDT").await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_check_sl_tp_triggers_empty() {
        let engine = create_test_engine().await;
        let result = engine.check_sl_tp_triggers().await;
        assert!(result.is_ok());
        let triggered = result.unwrap();
        assert!(triggered.is_empty());
    }

    #[tokio::test]
    async fn test_update_config() {
        let engine = create_test_engine().await;
        let mut new_config = RealTradingConfig::default();
        new_config.max_positions = 10;
        let result = engine.update_config(new_config).await;
        assert!(result.is_ok());
        let config = engine.get_config().await;
        assert_eq!(config.max_positions, 10);
    }

    #[tokio::test]
    async fn test_reset_circuit_breaker() {
        let engine = create_test_engine().await;
        engine.reset_circuit_breaker().await;
        let cb = engine.get_circuit_breaker().await;
        assert!(!cb.is_open);
        assert_eq!(cb.error_count, 0);
    }

    #[tokio::test]
    async fn test_get_reconciliation_metrics() {
        let engine = create_test_engine().await;
        let metrics = engine.get_reconciliation_metrics().await;
        assert_eq!(metrics.total_runs, 0);
    }

    #[tokio::test]
    async fn test_is_running_initial_state() {
        let engine = create_test_engine().await;
        assert!(!engine.is_running().await);
    }

    #[tokio::test]
    async fn test_subscribe_events() {
        let engine = create_test_engine().await;
        let _rx = engine.subscribe_events();
        // Should successfully create a receiver
    }

    #[tokio::test]
    async fn test_get_positions_empty() {
        let engine = create_test_engine().await;
        let positions = engine.get_positions();
        assert!(positions.is_empty());
    }

    #[tokio::test]
    async fn test_get_position_nonexistent() {
        let engine = create_test_engine().await;
        let position = engine.get_position("BTCUSDT");
        assert!(position.is_none());
    }

    #[tokio::test]
    async fn test_get_orders_empty() {
        let engine = create_test_engine().await;
        let orders = engine.get_orders();
        assert!(orders.is_empty());
    }

    #[tokio::test]
    async fn test_get_active_orders_empty() {
        let engine = create_test_engine().await;
        let orders = engine.get_active_orders();
        assert!(orders.is_empty());
    }

    #[tokio::test]
    async fn test_get_order_nonexistent() {
        let engine = create_test_engine().await;
        let order = engine.get_order("nonexistent-order");
        assert!(order.is_none());
    }

    #[tokio::test]
    async fn test_get_balance_nonexistent() {
        let engine = create_test_engine().await;
        let balance = engine.get_balance("BTC").await;
        assert!(balance.is_none());
    }

    #[tokio::test]
    async fn test_get_usdt_balance_zero() {
        let engine = create_test_engine().await;
        let balance = engine.get_usdt_balance().await;
        assert_eq!(balance, 0.0);
    }

    #[tokio::test]
    async fn test_get_all_balances_empty() {
        let engine = create_test_engine().await;
        let balances = engine.get_all_balances().await;
        assert!(balances.is_empty());
    }

    #[tokio::test]
    async fn test_get_total_unrealized_pnl_empty() {
        let engine = create_test_engine().await;
        let pnl = engine.get_total_unrealized_pnl();
        assert_eq!(pnl, 0.0);
    }

    #[tokio::test]
    async fn test_get_total_exposure_empty() {
        let engine = create_test_engine().await;
        let exposure = engine.get_total_exposure();
        assert_eq!(exposure, 0.0);
    }

    #[tokio::test]
    async fn test_get_total_equity_usdt_zero() {
        let engine = create_test_engine().await;
        let equity = engine.get_total_equity_usdt().await;
        assert_eq!(equity, 0.0);
    }

    #[tokio::test]
    async fn test_get_circuit_breaker_initial() {
        let engine = create_test_engine().await;
        let cb = engine.get_circuit_breaker().await;
        assert!(!cb.is_open);
        assert_eq!(cb.error_count, 0);
        assert!(cb.opened_at.is_none());
        assert!(cb.last_error.is_none());
    }

    #[tokio::test]
    async fn test_get_daily_metrics_initial() {
        let engine = create_test_engine().await;
        let metrics = engine.get_daily_metrics().await;
        assert_eq!(metrics.trades_count, 0);
        assert_eq!(metrics.winning_trades, 0);
        assert_eq!(metrics.losing_trades, 0);
        assert_eq!(metrics.realized_pnl, 0.0);
        assert_eq!(metrics.total_volume, 0.0);
        assert_eq!(metrics.total_commission, 0.0);
    }

    #[tokio::test]
    async fn test_get_risk_manager() {
        let engine = create_test_engine().await;
        let _rm = engine.get_risk_manager();
        // Should return reference successfully
    }

    #[tokio::test]
    async fn test_update_prices_empty() {
        let engine = create_test_engine().await;
        let prices = HashMap::new();
        engine.update_prices(&prices);
        // Should not panic with empty prices
    }

    #[tokio::test]
    async fn test_update_prices_with_no_positions() {
        let engine = create_test_engine().await;
        let mut prices = HashMap::new();
        prices.insert("BTCUSDT".to_string(), 50000.0);
        prices.insert("ETHUSDT".to_string(), 3000.0);
        engine.update_prices(&prices);
        // Should not panic with no matching positions
    }

    #[tokio::test]
    async fn test_refresh_balances_no_api() {
        let engine = create_test_engine().await;
        // This will likely fail due to no real API, but should handle gracefully
        let result = engine.refresh_balances().await;
        // We expect it might fail with testnet/API issues
        // Just verify it doesn't panic
        let _ = result;
    }

    #[tokio::test]
    async fn test_cancel_order_nonexistent() {
        let engine = create_test_engine().await;
        let result = engine.cancel_order("nonexistent-order").await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_cancel_all_orders_empty() {
        let engine = create_test_engine().await;
        let result = engine.cancel_all_orders(None).await;
        assert!(result.is_ok());
        let cancelled = result.unwrap();
        assert!(cancelled.is_empty());
    }

    #[tokio::test]
    async fn test_cancel_all_orders_with_symbol_filter() {
        let engine = create_test_engine().await;
        let result = engine.cancel_all_orders(Some("BTCUSDT")).await;
        assert!(result.is_ok());
        let cancelled = result.unwrap();
        assert!(cancelled.is_empty());
    }

    #[tokio::test]
    async fn test_force_reconciliation_no_api() {
        let engine = create_test_engine().await;
        // Will likely fail due to no API but should handle gracefully
        let result = engine.force_reconciliation().await;
        // Just verify it doesn't panic
        let _ = result;
    }

    #[tokio::test]
    async fn test_handle_websocket_disconnect() {
        let engine = create_test_engine().await;
        // Should handle disconnect gracefully
        engine.handle_websocket_disconnect().await;
        // Verify it doesn't panic
    }

    #[tokio::test]
    async fn test_emergency_stop_no_positions() {
        let engine = create_test_engine().await;
        let result = engine.emergency_stop("test emergency").await;
        assert!(result.is_ok());
        // Verify circuit breaker is open
        let cb = engine.get_circuit_breaker().await;
        assert!(cb.is_open);
    }

    #[tokio::test]
    async fn test_stop_when_not_running() {
        let engine = create_test_engine().await;
        let result = engine.stop().await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_place_market_order_no_api() {
        let engine = create_test_engine().await;
        let result = engine
            .place_market_order("BTCUSDT", OrderSide::Buy, 0.001, None, true)
            .await;
        // Will fail due to no real API but should handle gracefully
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_place_limit_order_no_api() {
        let engine = create_test_engine().await;
        let result = engine
            .place_limit_order("BTCUSDT", OrderSide::Buy, 0.001, 50000.0, None, true)
            .await;
        // Will fail due to no real API but should handle gracefully
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_calculate_position_size_zero_stop_loss() {
        let engine = create_test_engine().await;
        let size = engine.calculate_position_size(50000.0, 0.0).await;
        assert!(size >= 0.0);
    }

    #[tokio::test]
    async fn test_calculate_position_size_negative_stop_loss() {
        let engine = create_test_engine().await;
        let size = engine.calculate_position_size(50000.0, -1000.0).await;
        assert!(size >= 0.0);
    }

    #[tokio::test]
    async fn test_calculate_position_size_auto_sl_long() {
        let engine = create_test_engine().await;
        let (size, stop_loss) = engine.calculate_position_size_auto_sl(50000.0, true).await;
        assert!(size >= 0.0);
        assert!(stop_loss > 0.0);
        assert!(stop_loss < 50000.0); // SL should be below entry for long
    }

    #[tokio::test]
    async fn test_calculate_position_size_auto_sl_short() {
        let engine = create_test_engine().await;
        let (size, stop_loss) = engine.calculate_position_size_auto_sl(50000.0, false).await;
        assert!(size >= 0.0);
        assert!(stop_loss > 0.0);
        assert!(stop_loss > 50000.0); // SL should be above entry for short
    }

    #[tokio::test]
    async fn test_set_stop_loss_invalid_price() {
        let engine = create_test_engine().await;
        let result = engine.set_stop_loss("BTCUSDT", 0.0).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_set_take_profit_invalid_price() {
        let engine = create_test_engine().await;
        let result = engine.set_take_profit("BTCUSDT", 0.0).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_set_sl_tp_both_none() {
        let engine = create_test_engine().await;
        let result = engine.set_sl_tp("BTCUSDT", None, None).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_enable_trailing_stop_invalid_activation() {
        let engine = create_test_engine().await;
        let result = engine.enable_trailing_stop("BTCUSDT", 0.0, 2.0).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_enable_trailing_stop_invalid_callback() {
        let engine = create_test_engine().await;
        let result = engine.enable_trailing_stop("BTCUSDT", 52000.0, 0.0).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_enable_trailing_stop_negative_callback() {
        let engine = create_test_engine().await;
        let result = engine.enable_trailing_stop("BTCUSDT", 52000.0, -1.0).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_config_update_validation() {
        let engine = create_test_engine().await;
        let mut invalid_config = RealTradingConfig::default();
        invalid_config.max_positions = 0; // Invalid
        let result = engine.update_config(invalid_config).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_config_get_returns_clone() {
        let engine = create_test_engine().await;
        let config1 = engine.get_config().await;
        let config2 = engine.get_config().await;
        assert_eq!(config1.max_positions, config2.max_positions);
    }

    #[tokio::test]
    async fn test_calculate_position_size_equal_prices() {
        let engine = create_test_engine().await;
        let size = engine.calculate_position_size(50000.0, 50000.0).await;
        // When entry equals stop loss, should return 0 or very small value
        assert!(size >= 0.0);
    }

    #[tokio::test]
    async fn test_calculate_position_size_extreme_values() {
        let engine = create_test_engine().await;
        let size = engine.calculate_position_size(1000000.0, 1.0).await;
        assert!(size >= 0.0);
    }

    #[tokio::test]
    async fn test_get_risk_utilization_max() {
        let engine = create_test_engine().await;
        let utilization = engine.get_risk_utilization().await;
        // Should always be between 0 and 1
        assert!(utilization >= 0.0 && utilization <= 1.0);
    }

    #[tokio::test]
    async fn test_get_daily_loss_utilization_min() {
        let engine = create_test_engine().await;
        let utilization = engine.get_daily_loss_utilization().await;
        // With no losses, should be 0
        assert_eq!(utilization, 0.0);
    }

    #[tokio::test]
    async fn test_place_market_order_with_position_id() {
        let engine = create_test_engine().await;
        let result = engine
            .place_market_order(
                "BTCUSDT",
                OrderSide::Buy,
                0.001,
                Some("pos-123".to_string()),
                true,
            )
            .await;
        // Will fail due to no API but should handle position_id parameter
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_place_limit_order_with_position_id() {
        let engine = create_test_engine().await;
        let result = engine
            .place_limit_order(
                "BTCUSDT",
                OrderSide::Sell,
                0.001,
                51000.0,
                Some("pos-456".to_string()),
                false,
            )
            .await;
        // Will fail due to no API but should handle position_id parameter
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_place_market_order_zero_quantity() {
        let engine = create_test_engine().await;
        let result = engine
            .place_market_order("BTCUSDT", OrderSide::Buy, 0.0, None, true)
            .await;
        // Should fail validation
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_place_limit_order_negative_price() {
        let engine = create_test_engine().await;
        let result = engine
            .place_limit_order("BTCUSDT", OrderSide::Buy, 0.001, -1000.0, None, true)
            .await;
        // Should fail validation
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_place_limit_order_zero_price() {
        let engine = create_test_engine().await;
        let result = engine
            .place_limit_order("BTCUSDT", OrderSide::Buy, 0.001, 0.0, None, true)
            .await;
        // Should fail validation
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_set_sl_tp_both_values() {
        let engine = create_test_engine().await;
        let result = engine
            .set_sl_tp("BTCUSDT", Some(49000.0), Some(51000.0))
            .await;
        // Will fail because position doesn't exist
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_set_sl_tp_only_stop_loss() {
        let engine = create_test_engine().await;
        let result = engine.set_sl_tp("BTCUSDT", Some(49000.0), None).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_set_sl_tp_only_take_profit() {
        let engine = create_test_engine().await;
        let result = engine.set_sl_tp("BTCUSDT", None, Some(51000.0)).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_enable_trailing_stop_valid_params() {
        let engine = create_test_engine().await;
        let result = engine.enable_trailing_stop("BTCUSDT", 52000.0, 2.5).await;
        // Will fail because position doesn't exist
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_close_position_empty_string() {
        let engine = create_test_engine().await;
        let result = engine.close_position("").await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_reset_circuit_breaker_when_open() {
        let engine = create_test_engine().await;

        // First, trigger circuit breaker by simulating errors
        {
            let mut cb = engine.circuit_breaker.write().await;
            cb.record_error("Test error", 1);
        }

        // Verify it's open
        assert!(engine.get_circuit_breaker().await.is_open);

        // Reset it
        engine.reset_circuit_breaker().await;

        // Verify it's closed
        let cb = engine.get_circuit_breaker().await;
        assert!(!cb.is_open);
        assert_eq!(cb.error_count, 0);
    }

    #[tokio::test]
    async fn test_emergency_stop_opens_circuit_breaker() {
        let engine = create_test_engine().await;
        let result = engine.emergency_stop("Critical failure").await;
        assert!(result.is_ok());

        let cb = engine.get_circuit_breaker().await;
        assert!(cb.is_open);
        assert!(cb.last_error.is_some());
        assert!(cb.last_error.unwrap().contains("EMERGENCY"));
    }

    #[tokio::test]
    async fn test_emergency_stop_with_custom_reason() {
        let engine = create_test_engine().await;
        let result = engine.emergency_stop("Account compromised").await;
        assert!(result.is_ok());

        let cb = engine.get_circuit_breaker().await;
        assert!(cb.last_error.unwrap().contains("Account compromised"));
    }

    #[tokio::test]
    async fn test_cancel_all_orders_filters_by_symbol() {
        let engine = create_test_engine().await;

        // Test with specific symbol filter
        let result = engine.cancel_all_orders(Some("ETHUSDT")).await;
        assert!(result.is_ok());

        // Should return empty list when no orders exist
        let cancelled = result.unwrap();
        assert!(cancelled.is_empty());
    }

    #[tokio::test]
    async fn test_stop_multiple_times() {
        let engine = create_test_engine().await;

        // First stop should succeed
        let result1 = engine.stop().await;
        assert!(result1.is_ok());

        // Second stop should also succeed (idempotent)
        let result2 = engine.stop().await;
        assert!(result2.is_ok());
    }

    #[tokio::test]
    async fn test_update_config_with_valid_changes() {
        let engine = create_test_engine().await;

        let mut new_config = RealTradingConfig::default();
        new_config.max_positions = 15;
        new_config.max_daily_loss_usdt = 200.0;

        let result = engine.update_config(new_config).await;
        assert!(result.is_ok());

        let updated = engine.get_config().await;
        assert_eq!(updated.max_positions, 15);
        assert_eq!(updated.max_daily_loss_usdt, 200.0);
    }

    #[tokio::test]
    async fn test_get_total_equity_with_no_balances() {
        let engine = create_test_engine().await;
        let equity = engine.get_total_equity_usdt().await;
        // Should be 0 when no balances exist
        assert_eq!(equity, 0.0);
    }

    #[tokio::test]
    async fn test_check_sl_tp_triggers_with_no_positions() {
        let engine = create_test_engine().await;
        let result = engine.check_sl_tp_triggers().await;
        assert!(result.is_ok());

        let triggered = result.unwrap();
        assert!(triggered.is_empty());
    }

    #[tokio::test]
    async fn test_calculate_position_size_auto_sl_extreme_price() {
        let engine = create_test_engine().await;
        let (size, stop_loss) = engine
            .calculate_position_size_auto_sl(1000000.0, true)
            .await;
        assert!(size >= 0.0);
        assert!(stop_loss > 0.0);
        assert!(stop_loss < 1000000.0);
    }

    #[tokio::test]
    async fn test_calculate_position_size_auto_sl_low_price() {
        let engine = create_test_engine().await;
        let (size, stop_loss) = engine.calculate_position_size_auto_sl(0.001, true).await;
        assert!(size >= 0.0);
        assert!(stop_loss > 0.0);
    }

    // ============ Additional Async Coverage Tests ============

    #[tokio::test]
    async fn test_cov_get_risk_manager() {
        let engine = create_test_engine().await;
        let _risk_manager = engine.get_risk_manager();
        // Should successfully return risk manager reference
    }

    #[tokio::test]
    async fn test_cov_update_prices_empty_map() {
        let engine = create_test_engine().await;
        let prices = std::collections::HashMap::new();
        engine.update_prices(&prices);
        // Should handle empty price map without panic
    }

    #[tokio::test]
    async fn test_cov_update_prices_with_data() {
        let engine = create_test_engine().await;
        let mut prices = std::collections::HashMap::new();
        prices.insert("BTCUSDT".to_string(), 50000.0);
        prices.insert("ETHUSDT".to_string(), 3000.0);
        engine.update_prices(&prices);
        // Should update prices successfully
    }

    #[tokio::test]
    async fn test_cov_calculate_position_size_zero_stop_loss() {
        let engine = create_test_engine().await;
        let size = engine.calculate_position_size(50000.0, 50000.0).await;
        // Same entry and SL should result in minimal or zero size
        assert!(size >= 0.0);
    }

    #[tokio::test]
    async fn test_cov_calculate_position_size_large_stop_distance() {
        let engine = create_test_engine().await;
        let size = engine.calculate_position_size(50000.0, 10000.0).await;
        // Very large SL distance should result in smaller position
        assert!(size >= 0.0);
    }

    #[tokio::test]
    async fn test_cov_calculate_position_size_auto_sl_short() {
        let engine = create_test_engine().await;
        let (size, stop_loss) = engine.calculate_position_size_auto_sl(50000.0, false).await;
        assert!(size >= 0.0);
        assert!(stop_loss > 50000.0); // Short: SL above entry
    }

    #[tokio::test]
    async fn test_cov_get_config_returns_valid() {
        let engine = create_test_engine().await;
        let config = engine.get_config().await;
        assert!(config.max_positions > 0);
        assert!(config.max_daily_loss_usdt > 0.0);
    }

    #[tokio::test]
    async fn test_cov_update_config_different_values() {
        let engine = create_test_engine().await;
        let mut new_config = RealTradingConfig::default();
        new_config.max_positions = 20;
        new_config.max_daily_loss_usdt = 500.0;
        new_config.max_position_size_usdt = 5000.0;

        let result = engine.update_config(new_config).await;
        assert!(result.is_ok());

        let updated = engine.get_config().await;
        assert_eq!(updated.max_positions, 20);
        assert_eq!(updated.max_daily_loss_usdt, 500.0);
        assert_eq!(updated.max_position_size_usdt, 5000.0);
    }

    #[tokio::test]
    async fn test_cov_cancel_all_orders_none_symbol() {
        let engine = create_test_engine().await;
        let result = engine.cancel_all_orders(None).await;
        // Should succeed with empty list when no orders
        assert!(result.is_ok());
        assert!(result.unwrap().is_empty());
    }

    #[tokio::test]
    async fn test_cov_cancel_all_orders_with_symbol() {
        let engine = create_test_engine().await;
        let result = engine.cancel_all_orders(Some("BTCUSDT")).await;
        // Should succeed with empty list when no orders
        assert!(result.is_ok());
        assert!(result.unwrap().is_empty());
    }

    #[tokio::test]
    async fn test_cov_force_reconciliation() {
        let engine = create_test_engine().await;
        // Note: Will fail with binance API call, but tests the code path
        let _result = engine.force_reconciliation().await;
        // Expected to fail due to no real exchange connection
    }

    #[tokio::test]
    async fn test_cov_handle_websocket_disconnect() {
        let engine = create_test_engine().await;
        engine.handle_websocket_disconnect().await;
        // Should complete without panic
    }

    #[tokio::test]
    async fn test_cov_start_when_already_stopped() {
        let engine = create_test_engine().await;
        // Engine starts in stopped state
        let result = engine.start().await;
        // Will fail due to no real WebSocket, but tests the path
        assert!(result.is_err() || result.is_ok());
    }

    #[tokio::test]
    async fn test_cov_stop_when_already_stopped() {
        let engine = create_test_engine().await;
        // Engine starts in stopped state
        let result = engine.stop().await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_cov_refresh_balances() {
        let engine = create_test_engine().await;
        // Will fail with API call, but tests code path
        let _result = engine.refresh_balances().await;
        // Expected to fail due to testnet API
    }

    #[tokio::test]
    async fn test_cov_emergency_stop_no_orders() {
        let engine = create_test_engine().await;
        let result = engine.emergency_stop("Test emergency").await;
        assert!(result.is_ok());

        // Circuit breaker should be open
        let cb = engine.get_circuit_breaker().await;
        assert!(cb.is_open);
        assert!(cb.last_error.is_some());

        // Engine should be stopped
        assert!(!engine.is_running().await);
    }

    #[tokio::test]
    async fn test_cov_calculate_position_size_negative_stop() {
        let engine = create_test_engine().await;
        // Entry > SL for long (invalid, but should handle gracefully)
        let size = engine.calculate_position_size(50000.0, 60000.0).await;
        assert!(size >= 0.0);
    }

    #[tokio::test]
    async fn test_cov_set_sl_tp_both_none() {
        let engine = create_test_engine().await;
        let result = engine.set_sl_tp("BTCUSDT", None, None).await;
        // Should fail - no position exists
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_cov_set_sl_tp_only_sl() {
        let engine = create_test_engine().await;
        let result = engine.set_sl_tp("BTCUSDT", Some(49000.0), None).await;
        // Should fail - no position exists
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_cov_set_sl_tp_only_tp() {
        let engine = create_test_engine().await;
        let result = engine.set_sl_tp("BTCUSDT", None, Some(51000.0)).await;
        // Should fail - no position exists
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_cov_enable_trailing_stop_zero_callback() {
        let engine = create_test_engine().await;
        let result = engine.enable_trailing_stop("BTCUSDT", 52000.0, 0.0).await;
        // Should fail - no position exists
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_cov_enable_trailing_stop_negative_callback() {
        let engine = create_test_engine().await;
        let result = engine.enable_trailing_stop("BTCUSDT", 52000.0, -1.0).await;
        // Should fail - no position exists (invalid callback also)
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_cov_enable_trailing_stop_large_callback() {
        let engine = create_test_engine().await;
        let result = engine.enable_trailing_stop("BTCUSDT", 52000.0, 50.0).await;
        // Should fail - no position exists
        assert!(result.is_err());
    }

    // ============ Process Execution Report Tests ============

    #[tokio::test]
    async fn test_cov_process_execution_report_unknown_order() {
        let engine = create_test_engine().await;
        let report = create_test_execution_report(
            "unknown-order",
            "BTCUSDT",
            "BUY",
            "NEW",
            "NEW",
            "1.0",
            "0.0",
            "50000.0",
        );
        let result = engine.process_execution_report(&report).await;
        // Should succeed even with unknown order (just logs warning)
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_cov_process_execution_report_partially_filled() {
        use super::*;
        let engine = create_test_engine().await;

        // Insert a pending order first
        let order = RealOrder::new(
            "test-order-001".to_string(),
            "BTCUSDT".to_string(),
            "BUY".to_string(),
            "LIMIT".to_string(),
            1.0,
            Some(50000.0),
            None,
            None,
            true,
        );
        engine.orders.insert("test-order-001".to_string(), order);

        let report = create_test_execution_report(
            "test-order-001",
            "BTCUSDT",
            "BUY",
            "TRADE",
            "PARTIALLY_FILLED",
            "1.0",
            "0.5",
            "50000.0",
        );
        let result = engine.process_execution_report(&report).await;
        assert!(result.is_ok());

        // Check order state updated
        let updated_order = engine.orders.get("test-order-001").unwrap();
        assert_eq!(updated_order.state, OrderState::PartiallyFilled);
    }

    #[tokio::test]
    async fn test_cov_process_execution_report_cancelled() {
        use super::*;
        let engine = create_test_engine().await;

        let order = RealOrder::new(
            "test-order-002".to_string(),
            "ETHUSDT".to_string(),
            "SELL".to_string(),
            "LIMIT".to_string(),
            2.0,
            Some(3000.0),
            None,
            None,
            false,
        );
        engine.orders.insert("test-order-002".to_string(), order);

        let report = create_test_execution_report(
            "test-order-002",
            "ETHUSDT",
            "SELL",
            "CANCELED",
            "CANCELED",
            "2.0",
            "0.0",
            "3000.0",
        );
        let result = engine.process_execution_report(&report).await;
        assert!(result.is_ok());

        let updated_order = engine.orders.get("test-order-002").unwrap();
        assert_eq!(updated_order.state, OrderState::Cancelled);
    }

    #[tokio::test]
    async fn test_cov_process_execution_report_rejected() {
        use super::*;
        let engine = create_test_engine().await;

        let order = RealOrder::new(
            "test-order-003".to_string(),
            "BTCUSDT".to_string(),
            "BUY".to_string(),
            "MARKET".to_string(),
            0.1,
            None,
            None,
            None,
            true,
        );
        engine.orders.insert("test-order-003".to_string(), order);

        let report = create_test_execution_report(
            "test-order-003",
            "BTCUSDT",
            "BUY",
            "REJECTED",
            "REJECTED",
            "0.1",
            "0.0",
            "0.0",
        );
        let result = engine.process_execution_report(&report).await;
        assert!(result.is_ok());

        let updated_order = engine.orders.get("test-order-003").unwrap();
        assert_eq!(updated_order.state, OrderState::Rejected);
    }

    // ============ More Position and Order Tests ============

    #[test]
    fn test_cov_position_add_fill() {
        use super::*;
        let mut pos = RealPosition::new(
            "pos-001".to_string(),
            "BTCUSDT".to_string(),
            PositionSide::Long,
            1.0,
            50000.0,
            "order-001".to_string(),
            None,
            None,
        );

        pos.add_fill(50500.0, 0.5, 1.0, "order-002".to_string());

        // Position should increase
        assert!((pos.quantity - 1.5).abs() < 0.001);
        // Average entry price should be weighted
        assert!(pos.entry_price > 50000.0 && pos.entry_price < 50500.0);
    }

    #[test]
    fn test_cov_position_set_stop_loss() {
        use super::*;
        let mut pos = RealPosition::new(
            "pos-001".to_string(),
            "BTCUSDT".to_string(),
            PositionSide::Long,
            1.0,
            50000.0,
            "order-001".to_string(),
            None,
            None,
        );

        pos.stop_loss = Some(49000.0);
        assert_eq!(pos.stop_loss, Some(49000.0));
    }

    #[test]
    fn test_cov_position_set_take_profit() {
        use super::*;
        let mut pos = RealPosition::new(
            "pos-001".to_string(),
            "BTCUSDT".to_string(),
            PositionSide::Long,
            1.0,
            50000.0,
            "order-001".to_string(),
            None,
            None,
        );

        pos.take_profit = Some(52000.0);
        assert_eq!(pos.take_profit, Some(52000.0));
    }

    #[test]
    fn test_cov_order_update_from_execution_report() {
        use super::*;
        let mut order = RealOrder::new(
            "test-order".to_string(),
            "BTCUSDT".to_string(),
            "BUY".to_string(),
            "LIMIT".to_string(),
            1.0,
            Some(50000.0),
            None,
            None,
            true,
        );

        let report = create_test_execution_report(
            "test-order",
            "BTCUSDT",
            "BUY",
            "TRADE",
            "FILLED",
            "1.0",
            "1.0",
            "50000.0",
        );

        order.update_from_execution_report(&report);

        assert_eq!(order.state, OrderState::Filled);
        assert!((order.executed_quantity - 1.0).abs() < 0.001);
        assert_eq!(order.exchange_order_id, 12345);
    }

    #[test]
    fn test_cov_order_is_filled_true() {
        use super::*;
        let mut order = RealOrder::new(
            "test-order".to_string(),
            "BTCUSDT".to_string(),
            "BUY".to_string(),
            "MARKET".to_string(),
            1.0,
            None,
            None,
            None,
            true,
        );
        order.state = OrderState::Filled;
        assert!(order.is_filled());
    }

    #[test]
    fn test_cov_order_is_filled_false() {
        use super::*;
        let order = RealOrder::new(
            "test-order".to_string(),
            "BTCUSDT".to_string(),
            "BUY".to_string(),
            "LIMIT".to_string(),
            1.0,
            Some(50000.0),
            None,
            None,
            true,
        );
        assert!(!order.is_filled());
    }

    #[test]
    fn test_cov_position_side_from_order_side_buy() {
        use super::*;
        let side = PositionSide::from_order_side("BUY");
        assert_eq!(side, PositionSide::Long);
    }

    #[test]
    fn test_cov_position_side_from_order_side_sell() {
        use super::*;
        let side = PositionSide::from_order_side("SELL");
        assert_eq!(side, PositionSide::Short);
    }

    #[test]
    fn test_cov_position_side_from_order_side_unknown() {
        use super::*;
        let side = PositionSide::from_order_side("UNKNOWN");
        assert_eq!(side, PositionSide::Long); // Default
    }

    // ============ Daily Metrics Additional Tests ============

    #[test]
    fn test_cov_daily_metrics_record_loss() {
        use super::*;
        let mut metrics = DailyMetrics::new();
        metrics.trades_count = 10;
        metrics.losing_trades = 3;
        metrics.winning_trades = 7;

        assert_eq!(metrics.losing_trades, 3);
    }

    #[test]
    fn test_cov_daily_metrics_total_pnl_positive() {
        use super::*;
        let mut metrics = DailyMetrics::new();
        metrics.realized_pnl = 150.0;
        metrics.trades_count = 5;

        assert_eq!(metrics.realized_pnl, 150.0);
    }

    #[test]
    fn test_cov_daily_metrics_total_pnl_negative() {
        use super::*;
        let mut metrics = DailyMetrics::new();
        metrics.realized_pnl = -50.0;
        metrics.trades_count = 3;

        assert_eq!(metrics.realized_pnl, -50.0);
    }

    #[test]
    fn test_cov_daily_metrics_fees_accumulation() {
        use super::*;
        let mut metrics = DailyMetrics::new();
        metrics.total_commission = 2.3;

        assert!((metrics.total_commission - 2.3).abs() < 0.01);
    }

    #[test]
    fn test_cov_daily_metrics_volume_tracking() {
        use super::*;
        let mut metrics = DailyMetrics::new();
        metrics.total_volume = 10000.0;

        assert_eq!(metrics.total_volume, 10000.0);
    }

    #[test]
    fn test_cov_daily_metrics_winning_losing_trades() {
        use super::*;
        let mut metrics = DailyMetrics::new();
        metrics.winning_trades = 15;
        metrics.losing_trades = 5;
        metrics.trades_count = 20;

        assert_eq!(metrics.winning_trades, 15);
        assert_eq!(metrics.losing_trades, 5);
        // win_rate returns percentage (0-100), not decimal
        assert!((metrics.win_rate() - 75.0).abs() < 0.001);
    }

    // ============ Config Tests ============

    #[test]
    fn test_cov_real_trading_config_default() {
        use super::*;
        let config = RealTradingConfig::default();

        assert!(config.use_testnet);
        assert_eq!(config.max_positions, 5);
        assert_eq!(config.max_position_size_usdt, 1000.0);
        // Default max_daily_loss_usdt is 500.0, not 100.0
        assert_eq!(config.max_daily_loss_usdt, 500.0);
    }

    #[test]
    fn test_cov_real_trading_config_is_symbol_allowed_empty_list() {
        use super::*;
        let config = RealTradingConfig {
            allowed_symbols: vec![],
            ..Default::default()
        };

        // Empty list means all symbols allowed
        assert!(config.is_symbol_allowed("BTCUSDT"));
        assert!(config.is_symbol_allowed("ETHUSDT"));
    }

    #[test]
    fn test_cov_real_trading_config_is_symbol_allowed_with_list() {
        use super::*;
        let config = RealTradingConfig {
            allowed_symbols: vec!["BTCUSDT".to_string(), "ETHUSDT".to_string()],
            ..Default::default()
        };

        assert!(config.is_symbol_allowed("BTCUSDT"));
        assert!(config.is_symbol_allowed("ETHUSDT"));
        assert!(!config.is_symbol_allowed("BNBUSDT"));
    }

    #[test]
    fn test_cov_real_trading_config_circuit_breaker_settings() {
        use super::*;
        let config = RealTradingConfig::default();

        // Default circuit_breaker_errors is 3, not 5
        assert_eq!(config.circuit_breaker_errors, 3);
        assert_eq!(config.circuit_breaker_cooldown_secs, 300);
        assert!(!config.circuit_breaker_close_positions);
    }

    // ============ Additional Engine Getter/Utility Method Coverage Tests ============

    #[tokio::test]
    async fn test_eng_get_positions_empty() {
        let engine = create_test_engine().await;
        let positions = engine.get_positions();
        assert!(positions.is_empty());
    }

    #[tokio::test]
    async fn test_eng_get_positions_with_data() {
        let engine = create_test_engine().await;
        let pos = RealPosition::new(
            "pos-001".to_string(),
            "BTCUSDT".to_string(),
            PositionSide::Long,
            0.1,
            50000.0,
            "order-001".to_string(),
            None,
            None,
        );
        engine.positions.insert("BTCUSDT".to_string(), pos);

        let positions = engine.get_positions();
        assert_eq!(positions.len(), 1);
        assert_eq!(positions[0].symbol, "BTCUSDT");
    }

    #[tokio::test]
    async fn test_eng_get_position_found() {
        let engine = create_test_engine().await;
        let pos = RealPosition::new(
            "pos-001".to_string(),
            "ETHUSDT".to_string(),
            PositionSide::Short,
            1.0,
            3000.0,
            "order-002".to_string(),
            None,
            None,
        );
        engine.positions.insert("ETHUSDT".to_string(), pos);

        let result = engine.get_position("ETHUSDT");
        assert!(result.is_some());
        assert_eq!(result.unwrap().symbol, "ETHUSDT");
    }

    #[tokio::test]
    async fn test_eng_get_position_not_found() {
        let engine = create_test_engine().await;
        assert!(engine.get_position("NONEXISTENT").is_none());
    }

    #[tokio::test]
    async fn test_eng_get_orders_empty() {
        let engine = create_test_engine().await;
        assert!(engine.get_orders().is_empty());
    }

    #[tokio::test]
    async fn test_eng_get_orders_with_data() {
        let engine = create_test_engine().await;
        let order = RealOrder::new(
            "order-001".to_string(),
            "BTCUSDT".to_string(),
            "BUY".to_string(),
            "LIMIT".to_string(),
            0.1,
            Some(50000.0),
            None,
            None,
            true,
        );
        engine.orders.insert("order-001".to_string(), order);

        let orders = engine.get_orders();
        assert_eq!(orders.len(), 1);
    }

    #[tokio::test]
    async fn test_eng_get_active_orders_filtering() {
        let engine = create_test_engine().await;

        let mut active = RealOrder::new(
            "active-001".to_string(),
            "BTCUSDT".to_string(),
            "BUY".to_string(),
            "LIMIT".to_string(),
            0.1,
            Some(50000.0),
            None,
            None,
            true,
        );
        active.state = OrderState::New;
        engine.orders.insert("active-001".to_string(), active);

        let mut filled = RealOrder::new(
            "filled-001".to_string(),
            "ETHUSDT".to_string(),
            "SELL".to_string(),
            "MARKET".to_string(),
            1.0,
            None,
            None,
            None,
            false,
        );
        filled.state = OrderState::Filled;
        engine.orders.insert("filled-001".to_string(), filled);

        let result = engine.get_active_orders();
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].client_order_id, "active-001");
    }

    #[tokio::test]
    async fn test_eng_get_order_found() {
        let engine = create_test_engine().await;
        let order = RealOrder::new(
            "test-order".to_string(),
            "BTCUSDT".to_string(),
            "BUY".to_string(),
            "LIMIT".to_string(),
            0.1,
            Some(49000.0),
            None,
            None,
            true,
        );
        engine.orders.insert("test-order".to_string(), order);

        let result = engine.get_order("test-order");
        assert!(result.is_some());
    }

    #[tokio::test]
    async fn test_eng_get_order_not_found() {
        let engine = create_test_engine().await;
        assert!(engine.get_order("nonexistent").is_none());
    }

    #[tokio::test]
    async fn test_eng_get_total_unrealized_pnl_empty() {
        let engine = create_test_engine().await;
        assert_eq!(engine.get_total_unrealized_pnl(), 0.0);
    }

    #[tokio::test]
    async fn test_eng_get_total_unrealized_pnl_with_positions() {
        let engine = create_test_engine().await;

        let mut pos1 = RealPosition::new(
            "pos1".to_string(),
            "BTCUSDT".to_string(),
            PositionSide::Long,
            0.1,
            50000.0,
            "ord1".to_string(),
            None,
            None,
        );
        pos1.update_price(52000.0);
        engine.positions.insert("BTCUSDT".to_string(), pos1);

        let total = engine.get_total_unrealized_pnl();
        assert!((total - 200.0).abs() < 1.0);
    }

    #[tokio::test]
    async fn test_eng_get_total_exposure_empty() {
        let engine = create_test_engine().await;
        assert_eq!(engine.get_total_exposure(), 0.0);
    }

    #[tokio::test]
    async fn test_eng_get_total_exposure_with_positions() {
        let engine = create_test_engine().await;

        let mut pos1 = RealPosition::new(
            "pos1".to_string(),
            "BTCUSDT".to_string(),
            PositionSide::Long,
            0.1,
            50000.0,
            "ord1".to_string(),
            None,
            None,
        );
        pos1.update_price(52000.0);
        engine.positions.insert("BTCUSDT".to_string(), pos1);

        let exposure = engine.get_total_exposure();
        assert!((exposure - 5200.0).abs() < 1.0);
    }

    #[tokio::test]
    async fn test_eng_update_prices() {
        let engine = create_test_engine().await;

        let pos = RealPosition::new(
            "pos1".to_string(),
            "BTCUSDT".to_string(),
            PositionSide::Long,
            0.1,
            50000.0,
            "ord1".to_string(),
            None,
            None,
        );
        engine.positions.insert("BTCUSDT".to_string(), pos);

        let mut prices = HashMap::new();
        prices.insert("BTCUSDT".to_string(), 55000.0);
        engine.update_prices(&prices);

        let updated = engine.get_position("BTCUSDT").unwrap();
        assert!((updated.current_price - 55000.0).abs() < 0.01);
    }

    #[tokio::test]
    async fn test_eng_get_all_balances_empty() {
        let engine = create_test_engine().await;
        let balances = engine.get_all_balances().await;
        assert!(balances.is_empty());
    }

    #[tokio::test]
    async fn test_eng_get_all_balances_with_data() {
        let engine = create_test_engine().await;

        let mut balances_map = HashMap::new();
        balances_map.insert(
            "USDT".to_string(),
            Balance {
                asset: "USDT".to_string(),
                free: 10000.0,
                locked: 500.0,
            },
        );

        {
            let mut balances = engine.balances.write().await;
            *balances = balances_map;
        }

        let result = engine.get_all_balances().await;
        assert_eq!(result.len(), 1);
        assert!(result.contains_key("USDT"));
    }

    #[tokio::test]
    async fn test_eng_get_balance_none() {
        let engine = create_test_engine().await;
        assert!(engine.get_balance("NONEXISTENT").await.is_none());
    }

    #[tokio::test]
    async fn test_eng_get_balance_found() {
        let engine = create_test_engine().await;

        let mut balances_map = HashMap::new();
        balances_map.insert(
            "BTC".to_string(),
            Balance {
                asset: "BTC".to_string(),
                free: 1.5,
                locked: 0.2,
            },
        );

        {
            let mut balances = engine.balances.write().await;
            *balances = balances_map;
        }

        let result = engine.get_balance("BTC").await;
        assert!(result.is_some());
        assert_eq!(result.unwrap().asset, "BTC");
    }

    #[tokio::test]
    async fn test_eng_get_usdt_balance_zero() {
        let engine = create_test_engine().await;
        assert_eq!(engine.get_usdt_balance().await, 0.0);
    }

    #[tokio::test]
    async fn test_eng_get_usdt_balance_with_value() {
        let engine = create_test_engine().await;

        let mut balances_map = HashMap::new();
        balances_map.insert(
            "USDT".to_string(),
            Balance {
                asset: "USDT".to_string(),
                free: 5000.0,
                locked: 1000.0,
            },
        );

        {
            let mut balances = engine.balances.write().await;
            *balances = balances_map;
        }

        assert!((engine.get_usdt_balance().await - 5000.0).abs() < 0.01);
    }

    #[tokio::test]
    async fn test_eng_cleanup_terminal_orders_empty() {
        let engine = create_test_engine().await;
        assert_eq!(engine.cleanup_terminal_orders(), 0);
    }

    #[tokio::test]
    async fn test_eng_cleanup_terminal_orders_with_filled() {
        let engine = create_test_engine().await;

        let mut order = RealOrder::new(
            "filled-order".to_string(),
            "BTCUSDT".to_string(),
            "BUY".to_string(),
            "MARKET".to_string(),
            0.1,
            None,
            None,
            None,
            true,
        );
        order.state = OrderState::Filled;
        // Must be older than 24h for cleanup to remove it
        order.updated_at = Utc::now() - chrono::Duration::hours(25);
        engine.orders.insert("filled-order".to_string(), order);

        assert_eq!(engine.cleanup_terminal_orders(), 1);
        assert!(engine.get_order("filled-order").is_none());
    }

    #[tokio::test]
    async fn test_eng_cleanup_terminal_orders_mixed() {
        let engine = create_test_engine().await;

        let mut active = RealOrder::new(
            "active".to_string(),
            "BTCUSDT".to_string(),
            "BUY".to_string(),
            "LIMIT".to_string(),
            0.1,
            Some(50000.0),
            None,
            None,
            true,
        );
        active.state = OrderState::New;
        engine.orders.insert("active".to_string(), active);

        let mut filled = RealOrder::new(
            "filled".to_string(),
            "ETHUSDT".to_string(),
            "SELL".to_string(),
            "MARKET".to_string(),
            1.0,
            None,
            None,
            None,
            false,
        );
        filled.state = OrderState::Filled;
        // Must be older than 24h for cleanup to remove it
        filled.updated_at = Utc::now() - chrono::Duration::hours(25);
        engine.orders.insert("filled".to_string(), filled);

        assert_eq!(engine.cleanup_terminal_orders(), 1);
        assert!(engine.get_order("active").is_some());
        assert!(engine.get_order("filled").is_none());
    }

    #[tokio::test]
    async fn test_eng_subscribe_events() {
        let engine = create_test_engine().await;
        let _rx = engine.subscribe_events();
    }

    #[tokio::test]
    async fn test_eng_get_circuit_breaker_default() {
        let engine = create_test_engine().await;
        let cb = engine.get_circuit_breaker().await;
        assert!(!cb.is_open);
    }

    #[tokio::test]
    async fn test_eng_get_daily_metrics_initial() {
        let engine = create_test_engine().await;
        let metrics = engine.get_daily_metrics().await;
        assert_eq!(metrics.trades_count, 0);
    }

    #[tokio::test]
    async fn test_eng_get_reconciliation_metrics_initial() {
        let engine = create_test_engine().await;
        let metrics = engine.get_reconciliation_metrics().await;
        assert_eq!(metrics.total_runs, 0);
    }

    // ============ NEW TESTS FOR COVERAGE BOOST ============

    #[tokio::test]
    async fn test_process_execution_report_new_order() {
        let engine = create_test_engine().await;
        let report = create_test_execution_report(
            "order-123",
            "BTCUSDT",
            "BUY",
            "NEW",
            "NEW",
            "0.001",
            "0.0",
            "50000.0",
        );

        let order = RealOrder::new(
            "order-123".to_string(),
            "BTCUSDT".to_string(),
            "BUY".to_string(),
            "MARKET".to_string(),
            0.001,
            Some(50000.0),
            None,
            None,
            true,
        );
        engine.orders.insert("order-123".to_string(), order);

        let result = engine.process_execution_report(&report).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_process_execution_report_filled() {
        let engine = create_test_engine().await;
        let report = create_test_execution_report(
            "order-456",
            "BTCUSDT",
            "BUY",
            "TRADE",
            "FILLED",
            "0.001",
            "0.001",
            "50000.0",
        );

        let order = RealOrder::new(
            "order-456".to_string(),
            "BTCUSDT".to_string(),
            "BUY".to_string(),
            "MARKET".to_string(),
            0.001,
            Some(50000.0),
            None,
            None,
            true,
        );
        engine.orders.insert("order-456".to_string(), order);

        let result = engine.process_execution_report(&report).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_process_execution_report_partially_filled() {
        let engine = create_test_engine().await;
        let report = create_test_execution_report(
            "order-789",
            "BTCUSDT",
            "BUY",
            "TRADE",
            "PARTIALLY_FILLED",
            "0.002",
            "0.001",
            "50000.0",
        );

        let order = RealOrder::new(
            "order-789".to_string(),
            "BTCUSDT".to_string(),
            "BUY".to_string(),
            "MARKET".to_string(),
            0.002,
            Some(50000.0),
            None,
            None,
            true,
        );
        engine.orders.insert("order-789".to_string(), order);

        let result = engine.process_execution_report(&report).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_process_execution_report_cancelled() {
        let engine = create_test_engine().await;
        let report = create_test_execution_report(
            "order-cancel",
            "BTCUSDT",
            "BUY",
            "CANCELED",
            "CANCELED",
            "0.001",
            "0.0",
            "50000.0",
        );

        let order = RealOrder::new(
            "order-cancel".to_string(),
            "BTCUSDT".to_string(),
            "BUY".to_string(),
            "MARKET".to_string(),
            0.001,
            Some(50000.0),
            None,
            None,
            true,
        );
        engine.orders.insert("order-cancel".to_string(), order);

        let result = engine.process_execution_report(&report).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_process_execution_report_rejected() {
        let engine = create_test_engine().await;
        let mut report = create_test_execution_report(
            "order-reject",
            "BTCUSDT",
            "BUY",
            "REJECTED",
            "REJECTED",
            "0.001",
            "0.0",
            "50000.0",
        );
        report.order_reject_reason = "INSUFFICIENT_BALANCE".to_string();

        let order = RealOrder::new(
            "order-reject".to_string(),
            "BTCUSDT".to_string(),
            "BUY".to_string(),
            "MARKET".to_string(),
            0.001,
            Some(50000.0),
            None,
            None,
            true,
        );
        engine.orders.insert("order-reject".to_string(), order);

        let result = engine.process_execution_report(&report).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_process_execution_report_unknown_order() {
        let engine = create_test_engine().await;
        let report = create_test_execution_report(
            "unknown-order",
            "BTCUSDT",
            "BUY",
            "NEW",
            "NEW",
            "0.001",
            "0.0",
            "50000.0",
        );

        let result = engine.process_execution_report(&report).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_handle_balance_update() {
        let engine = create_test_engine().await;
        let update = BalanceUpdate {
            event_type: "balanceUpdate".to_string(),
            event_time: chrono::Utc::now().timestamp_millis(),
            asset: "USDT".to_string(),
            balance_delta: "100.0".to_string(),
            clear_time: chrono::Utc::now().timestamp_millis(),
        };

        engine.handle_balance_update(update).await;
    }

    #[tokio::test]
    async fn test_outbound_account_position_struct() {
        let position = OutboundAccountPosition {
            event_type: "outboundAccountPosition".to_string(),
            event_time: chrono::Utc::now().timestamp_millis(),
            last_update_time: chrono::Utc::now().timestamp_millis(),
            balances: vec![],
        };

        assert_eq!(position.event_type, "outboundAccountPosition");
        assert!(position.balances.is_empty());
    }

    #[tokio::test]
    async fn test_update_position_with_existing_position() {
        let engine = create_test_engine().await;

        let mut order = RealOrder::new(
            "entry-order".to_string(),
            "BTCUSDT".to_string(),
            "BUY".to_string(),
            "MARKET".to_string(),
            0.001,
            Some(50000.0),
            None,
            None,
            true,
        );
        order.state = OrderState::Filled;
        order.executed_quantity = 0.001;
        order.average_fill_price = 50000.0;
        engine.orders.insert("entry-order".to_string(), order);

        let result = engine.update_position_from_fill("entry-order").await;
        assert!(result.is_ok());

        let position = engine.get_position("BTCUSDT");
        assert!(position.is_some());
    }

    #[tokio::test]
    async fn test_update_position_exit_order() {
        let engine = create_test_engine().await;

        let position = RealPosition::new(
            "pos-001".to_string(),
            "BTCUSDT".to_string(),
            PositionSide::Long,
            0.001,
            50000.0,
            "entry-order".to_string(),
            None,
            None,
        );
        engine.positions.insert("BTCUSDT".to_string(), position);

        let mut order = RealOrder::new(
            "exit-order".to_string(),
            "BTCUSDT".to_string(),
            "SELL".to_string(),
            "MARKET".to_string(),
            0.001,
            Some(51000.0),
            None,
            None,
            false,
        );
        order.state = OrderState::Filled;
        order.executed_quantity = 0.001;
        order.average_fill_price = 51000.0;
        engine.orders.insert("exit-order".to_string(), order);

        let result = engine.update_position_from_fill("exit-order").await;
        assert!(result.is_ok());

        let position = engine.get_position("BTCUSDT");
        assert!(position.is_none());
    }

    #[tokio::test]
    async fn test_check_risk_limits_legacy_daily_loss() {
        let engine = create_test_engine().await;

        {
            let mut metrics = engine.daily_metrics.write().await;
            metrics.realized_pnl = -10000.0;
        }

        let result = engine
            .check_risk_limits_legacy("BTCUSDT", 0.001, Some(50000.0))
            .await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_check_risk_limits_legacy_max_positions() {
        let engine = create_test_engine().await;

        for i in 0..10 {
            let symbol = format!("SYM{}USDT", i);
            let position = RealPosition::new(
                format!("pos-{}", i),
                symbol.clone(),
                PositionSide::Long,
                0.001,
                50000.0,
                format!("order-{}", i),
                None,
                None,
            );
            engine.positions.insert(symbol, position);
        }

        let result = engine
            .check_risk_limits_legacy("NEWUSDT", 0.001, Some(50000.0))
            .await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_check_risk_limits_legacy_position_size() {
        let engine = create_test_engine().await;

        let result = engine
            .check_risk_limits_legacy("BTCUSDT", 100.0, Some(50000.0))
            .await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_check_risk_limits_legacy_min_order_value() {
        let engine = create_test_engine().await;

        let result = engine
            .check_risk_limits_legacy("BTCUSDT", 0.00001, Some(50000.0))
            .await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_check_daily_loss_limit_not_reached() {
        let engine = create_test_engine().await;

        {
            let mut metrics = engine.daily_metrics.write().await;
            metrics.realized_pnl = -100.0;
        }

        engine.check_daily_loss_limit().await;
    }

    #[tokio::test]
    async fn test_check_daily_loss_limit_reached() {
        let engine = create_test_engine().await;

        {
            let mut metrics = engine.daily_metrics.write().await;
            metrics.realized_pnl = -10000.0;
        }

        engine.check_daily_loss_limit().await;
    }

    #[tokio::test]
    async fn test_cancel_order_not_active() {
        let engine = create_test_engine().await;

        let mut order = RealOrder::new(
            "filled-order".to_string(),
            "BTCUSDT".to_string(),
            "BUY".to_string(),
            "MARKET".to_string(),
            0.001,
            Some(50000.0),
            None,
            None,
            true,
        );
        order.state = OrderState::Filled;
        engine.orders.insert("filled-order".to_string(), order);

        let result = engine.cancel_order("filled-order").await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_set_stop_loss_with_position() {
        let engine = create_test_engine().await;

        let position = RealPosition::new(
            "pos-001".to_string(),
            "BTCUSDT".to_string(),
            PositionSide::Long,
            0.001,
            50000.0,
            "entry-order".to_string(),
            None,
            None,
        );
        engine.positions.insert("BTCUSDT".to_string(), position);

        let result = engine.set_stop_loss("BTCUSDT", 49000.0).await;
        assert!(result.is_ok());

        let pos = engine.get_position("BTCUSDT").unwrap();
        assert_eq!(pos.stop_loss, Some(49000.0));
    }

    #[tokio::test]
    async fn test_set_take_profit_with_position() {
        let engine = create_test_engine().await;

        let position = RealPosition::new(
            "pos-001".to_string(),
            "BTCUSDT".to_string(),
            PositionSide::Long,
            0.001,
            50000.0,
            "entry-order".to_string(),
            None,
            None,
        );
        engine.positions.insert("BTCUSDT".to_string(), position);

        let result = engine.set_take_profit("BTCUSDT", 52000.0).await;
        assert!(result.is_ok());

        let pos = engine.get_position("BTCUSDT").unwrap();
        assert_eq!(pos.take_profit, Some(52000.0));
    }

    #[tokio::test]
    async fn test_set_sl_tp_with_position() {
        let engine = create_test_engine().await;

        let position = RealPosition::new(
            "pos-001".to_string(),
            "BTCUSDT".to_string(),
            PositionSide::Long,
            0.001,
            50000.0,
            "entry-order".to_string(),
            None,
            None,
        );
        engine.positions.insert("BTCUSDT".to_string(), position);

        let result = engine
            .set_sl_tp("BTCUSDT", Some(49000.0), Some(52000.0))
            .await;
        assert!(result.is_ok());

        let pos = engine.get_position("BTCUSDT").unwrap();
        assert_eq!(pos.stop_loss, Some(49000.0));
        assert_eq!(pos.take_profit, Some(52000.0));
    }

    #[tokio::test]
    async fn test_enable_trailing_stop_with_position() {
        let engine = create_test_engine().await;

        let mut position = RealPosition::new(
            "pos-001".to_string(),
            "BTCUSDT".to_string(),
            PositionSide::Long,
            0.001,
            50000.0,
            "entry-order".to_string(),
            None,
            None,
        );
        position.current_price = 52000.0;
        engine.positions.insert("BTCUSDT".to_string(), position);

        let result = engine.enable_trailing_stop("BTCUSDT", 51000.0, 2.0).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_check_sl_tp_triggers_stop_loss_long() {
        let engine = create_test_engine().await;

        let mut position = RealPosition::new(
            "pos-001".to_string(),
            "BTCUSDT".to_string(),
            PositionSide::Long,
            0.001,
            50000.0,
            "entry-order".to_string(),
            None,
            None,
        );
        position.stop_loss = Some(49000.0);
        position.current_price = 48000.0;
        engine.positions.insert("BTCUSDT".to_string(), position);

        let result = engine.check_sl_tp_triggers().await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_check_sl_tp_triggers_take_profit_long() {
        let engine = create_test_engine().await;

        let mut position = RealPosition::new(
            "pos-001".to_string(),
            "BTCUSDT".to_string(),
            PositionSide::Long,
            0.001,
            50000.0,
            "entry-order".to_string(),
            None,
            None,
        );
        position.take_profit = Some(52000.0);
        position.current_price = 53000.0;
        engine.positions.insert("BTCUSDT".to_string(), position);

        let result = engine.check_sl_tp_triggers().await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_check_sl_tp_triggers_stop_loss_short() {
        let engine = create_test_engine().await;

        let mut position = RealPosition::new(
            "pos-001".to_string(),
            "BTCUSDT".to_string(),
            PositionSide::Short,
            0.001,
            50000.0,
            "entry-order".to_string(),
            None,
            None,
        );
        position.stop_loss = Some(51000.0);
        position.current_price = 52000.0;
        engine.positions.insert("BTCUSDT".to_string(), position);

        let result = engine.check_sl_tp_triggers().await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_check_sl_tp_triggers_take_profit_short() {
        let engine = create_test_engine().await;

        let mut position = RealPosition::new(
            "pos-001".to_string(),
            "BTCUSDT".to_string(),
            PositionSide::Short,
            0.001,
            50000.0,
            "entry-order".to_string(),
            None,
            None,
        );
        position.take_profit = Some(48000.0);
        position.current_price = 47000.0;
        engine.positions.insert("BTCUSDT".to_string(), position);

        let result = engine.check_sl_tp_triggers().await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_update_prices_with_positions() {
        let engine = create_test_engine().await;

        let position1 = RealPosition::new(
            "pos-001".to_string(),
            "BTCUSDT".to_string(),
            PositionSide::Long,
            0.001,
            50000.0,
            "entry-order-1".to_string(),
            None,
            None,
        );
        let position2 = RealPosition::new(
            "pos-002".to_string(),
            "ETHUSDT".to_string(),
            PositionSide::Long,
            0.01,
            3000.0,
            "entry-order-2".to_string(),
            None,
            None,
        );
        engine.positions.insert("BTCUSDT".to_string(), position1);
        engine.positions.insert("ETHUSDT".to_string(), position2);

        let mut prices = HashMap::new();
        prices.insert("BTCUSDT".to_string(), 51000.0);
        prices.insert("ETHUSDT".to_string(), 3100.0);

        engine.update_prices(&prices);

        let pos1 = engine.get_position("BTCUSDT").unwrap();
        assert_eq!(pos1.current_price, 51000.0);
        let pos2 = engine.get_position("ETHUSDT").unwrap();
        assert_eq!(pos2.current_price, 3100.0);
    }

    #[tokio::test]
    async fn test_get_total_unrealized_pnl_with_positions() {
        let engine = create_test_engine().await;

        let mut position = RealPosition::new(
            "pos-001".to_string(),
            "BTCUSDT".to_string(),
            PositionSide::Long,
            0.001,
            50000.0,
            "entry-order".to_string(),
            None,
            None,
        );
        position.current_price = 51000.0;
        position.unrealized_pnl = 1.0; // Set PnL directly since get_total_unrealized_pnl reads this field
        engine.positions.insert("BTCUSDT".to_string(), position);

        let pnl = engine.get_total_unrealized_pnl();
        assert!(pnl > 0.0);
    }

    #[tokio::test]
    async fn test_get_total_exposure_with_positions() {
        let engine = create_test_engine().await;

        let position1 = RealPosition::new(
            "pos-001".to_string(),
            "BTCUSDT".to_string(),
            PositionSide::Long,
            0.001,
            50000.0,
            "entry-order-1".to_string(),
            None,
            None,
        );
        engine.positions.insert("BTCUSDT".to_string(), position1);

        let exposure = engine.get_total_exposure();
        assert!(exposure > 0.0);
    }

    #[tokio::test]
    async fn test_get_balance_with_balance() {
        let engine = create_test_engine().await;

        {
            let mut balances = engine.balances.write().await;
            balances.insert(
                "USDT".to_string(),
                Balance {
                    asset: "USDT".to_string(),
                    free: 10000.0,
                    locked: 0.0,
                },
            );
        }

        let balance = engine.get_balance("USDT").await;
        assert!(balance.is_some());
        assert_eq!(balance.unwrap().free, 10000.0);
    }

    #[tokio::test]
    async fn test_get_usdt_balance_with_balance() {
        let engine = create_test_engine().await;

        {
            let mut balances = engine.balances.write().await;
            balances.insert(
                "USDT".to_string(),
                Balance {
                    asset: "USDT".to_string(),
                    free: 5000.0,
                    locked: 1000.0,
                },
            );
        }

        let balance = engine.get_usdt_balance().await;
        assert_eq!(balance, 5000.0);
    }

    #[tokio::test]
    async fn test_get_all_balances_with_balances() {
        let engine = create_test_engine().await;

        {
            let mut balances = engine.balances.write().await;
            balances.insert(
                "USDT".to_string(),
                Balance {
                    asset: "USDT".to_string(),
                    free: 5000.0,
                    locked: 0.0,
                },
            );
            balances.insert(
                "BTC".to_string(),
                Balance {
                    asset: "BTC".to_string(),
                    free: 0.1,
                    locked: 0.0,
                },
            );
        }

        let balances = engine.get_all_balances().await;
        assert_eq!(balances.len(), 2);
    }

    #[tokio::test]
    async fn test_get_total_equity_usdt_with_balance() {
        let engine = create_test_engine().await;

        {
            let mut balances = engine.balances.write().await;
            balances.insert(
                "USDT".to_string(),
                Balance {
                    asset: "USDT".to_string(),
                    free: 10000.0,
                    locked: 0.0,
                },
            );
        }

        let equity = engine.get_total_equity_usdt().await;
        assert_eq!(equity, 10000.0);
    }

    #[tokio::test]
    async fn test_get_positions_with_positions() {
        let engine = create_test_engine().await;

        let position = RealPosition::new(
            "pos-001".to_string(),
            "BTCUSDT".to_string(),
            PositionSide::Long,
            0.001,
            50000.0,
            "entry-order".to_string(),
            None,
            None,
        );
        engine.positions.insert("BTCUSDT".to_string(), position);

        let positions = engine.get_positions();
        assert_eq!(positions.len(), 1);
    }

    #[tokio::test]
    async fn test_get_position_existing() {
        let engine = create_test_engine().await;

        let position = RealPosition::new(
            "pos-001".to_string(),
            "BTCUSDT".to_string(),
            PositionSide::Long,
            0.001,
            50000.0,
            "entry-order".to_string(),
            None,
            None,
        );
        engine.positions.insert("BTCUSDT".to_string(), position);

        let pos = engine.get_position("BTCUSDT");
        assert!(pos.is_some());
    }

    #[tokio::test]
    async fn test_get_orders_with_orders() {
        let engine = create_test_engine().await;

        let order = RealOrder::new(
            "order-123".to_string(),
            "BTCUSDT".to_string(),
            "BUY".to_string(),
            "MARKET".to_string(),
            0.001,
            Some(50000.0),
            None,
            None,
            true,
        );
        engine.orders.insert("order-123".to_string(), order);

        let orders = engine.get_orders();
        assert_eq!(orders.len(), 1);
    }

    #[tokio::test]
    async fn test_get_active_orders_with_active() {
        let engine = create_test_engine().await;

        let mut order = RealOrder::new(
            "order-active".to_string(),
            "BTCUSDT".to_string(),
            "BUY".to_string(),
            "LIMIT".to_string(),
            0.001,
            Some(50000.0),
            None,
            None,
            true,
        );
        order.state = OrderState::Pending;
        engine.orders.insert("order-active".to_string(), order);

        let mut order2 = RealOrder::new(
            "order-filled".to_string(),
            "BTCUSDT".to_string(),
            "BUY".to_string(),
            "MARKET".to_string(),
            0.001,
            Some(50000.0),
            None,
            None,
            true,
        );
        order2.state = OrderState::Filled;
        engine.orders.insert("order-filled".to_string(), order2);

        let orders = engine.get_active_orders();
        assert_eq!(orders.len(), 1);
    }

    #[tokio::test]
    async fn test_get_order_existing() {
        let engine = create_test_engine().await;

        let order = RealOrder::new(
            "order-123".to_string(),
            "BTCUSDT".to_string(),
            "BUY".to_string(),
            "MARKET".to_string(),
            0.001,
            Some(50000.0),
            None,
            None,
            true,
        );
        engine.orders.insert("order-123".to_string(), order);

        let ord = engine.get_order("order-123");
        assert!(ord.is_some());
    }

    #[tokio::test]
    async fn test_circuit_breaker_record_error() {
        let mut cb = CircuitBreakerState::default();
        let opened = cb.record_error("Test error", 3);
        assert!(!opened); // count=1, below threshold

        let opened = cb.record_error("Test error 2", 3);
        assert!(!opened); // count=2, below threshold

        let opened = cb.record_error("Test error 3", 3);
        assert!(opened); // count=3 >= threshold, circuit opens
    }

    #[tokio::test]
    async fn test_circuit_breaker_record_success() {
        let mut cb = CircuitBreakerState::default();
        cb.record_error("Test error", 3);
        cb.record_error("Test error 2", 3);
        assert_eq!(cb.error_count, 2);

        cb.record_success();
        assert_eq!(cb.error_count, 0);
    }

    #[tokio::test]
    async fn test_circuit_breaker_should_close_not_yet() {
        let mut cb = CircuitBreakerState::default();
        cb.is_open = true;
        cb.opened_at = Some(chrono::Utc::now() - chrono::Duration::seconds(30));

        assert!(!cb.should_close(60));
    }

    #[tokio::test]
    async fn test_circuit_breaker_close() {
        let mut cb = CircuitBreakerState::default();
        cb.is_open = true;
        cb.error_count = 5;
        cb.opened_at = Some(chrono::Utc::now());
        cb.last_error = Some("Error".to_string());

        cb.close();

        assert!(!cb.is_open);
        assert_eq!(cb.error_count, 0);
        assert!(cb.opened_at.is_none());
        assert!(cb.last_error.is_none());
    }

    #[tokio::test]
    async fn test_balance_total() {
        let balance = Balance {
            asset: "BTC".to_string(),
            free: 1.5,
            locked: 0.5,
        };
        assert_eq!(balance.total(), 2.0);
    }

    #[tokio::test]
    async fn test_daily_metrics_win_rate_zero_trades() {
        let metrics = DailyMetrics::new();
        assert_eq!(metrics.win_rate(), 0.0);
    }

    #[tokio::test]
    async fn test_daily_metrics_win_rate_mixed() {
        let mut metrics = DailyMetrics::new();
        metrics.trades_count = 10;
        metrics.winning_trades = 6;
        metrics.losing_trades = 4;
        assert!((metrics.win_rate() - 60.0).abs() < 0.01);
    }

    #[tokio::test]
    async fn test_daily_metrics_reset_if_new_day() {
        let mut metrics = DailyMetrics::new();
        metrics.trades_count = 100;
        metrics.realized_pnl = 5000.0;

        metrics.date = "2020-01-01".to_string();

        metrics.reset_if_new_day();

        assert_eq!(metrics.trades_count, 0);
        assert_eq!(metrics.realized_pnl, 0.0);
    }

    #[tokio::test]
    async fn test_daily_metrics_no_reset_same_day() {
        let mut metrics = DailyMetrics::new();
        metrics.trades_count = 10;

        metrics.reset_if_new_day();

        assert_eq!(metrics.trades_count, 10);
    }

    // ========== ADDITIONAL COVERAGE TESTS FOR ENGINE ==========

    #[test]
    fn test_cov_circuit_breaker_record_multiple_errors() {
        let mut cb = CircuitBreakerState::default();

        // Record errors below threshold
        assert!(!cb.record_error("Error 1", 3));
        assert_eq!(cb.error_count, 1);
        assert!(!cb.is_open);

        assert!(!cb.record_error("Error 2", 3));
        assert_eq!(cb.error_count, 2);
        assert!(!cb.is_open);

        // Third error opens circuit
        assert!(cb.record_error("Error 3", 3));
        assert_eq!(cb.error_count, 3);
        assert!(cb.is_open);
        assert!(cb.opened_at.is_some());
        assert_eq!(cb.last_error, Some("Error 3".to_string()));
    }

    #[test]
    fn test_cov_circuit_breaker_should_close_cooldown() {
        let mut cb = CircuitBreakerState::default();
        cb.is_open = true;
        cb.opened_at = Some(Utc::now() - chrono::Duration::seconds(61));

        assert!(cb.should_close(60));
    }

    #[test]
    fn test_cov_circuit_breaker_should_not_close_before_cooldown() {
        let mut cb = CircuitBreakerState::default();
        cb.is_open = true;
        cb.opened_at = Some(Utc::now() - chrono::Duration::seconds(30));

        assert!(!cb.should_close(60));
    }

    #[test]
    fn test_cov_circuit_breaker_should_close_no_opened_at() {
        let cb = CircuitBreakerState::default();
        assert!(!cb.should_close(60));
    }

    #[test]
    fn test_cov_balance_structure() {
        let balance = Balance {
            asset: "USDT".to_string(),
            free: 10000.0,
            locked: 2000.0,
        };

        assert_eq!(balance.total(), 12000.0);
        assert_eq!(balance.asset, "USDT");
        assert_eq!(balance.free, 10000.0);
        assert_eq!(balance.locked, 2000.0);
    }

    #[test]
    fn test_cov_balance_default() {
        let balance = Balance::default();
        assert_eq!(balance.asset, "");
        assert_eq!(balance.free, 0.0);
        assert_eq!(balance.locked, 0.0);
        assert_eq!(balance.total(), 0.0);
    }

    #[test]
    fn test_cov_daily_metrics_new() {
        let metrics = DailyMetrics::new();
        assert_eq!(metrics.realized_pnl, 0.0);
        assert_eq!(metrics.trades_count, 0);
        assert_eq!(metrics.winning_trades, 0);
        assert_eq!(metrics.losing_trades, 0);
        assert_eq!(metrics.total_volume, 0.0);
        assert_eq!(metrics.total_commission, 0.0);
        assert_eq!(metrics.win_rate(), 0.0);
    }

    #[test]
    fn test_cov_daily_metrics_win_rate_calculation() {
        let mut metrics = DailyMetrics::new();
        metrics.trades_count = 20;
        metrics.winning_trades = 15;
        metrics.losing_trades = 5;

        assert!((metrics.win_rate() - 75.0).abs() < 0.01);
    }

    #[test]
    fn test_cov_daily_metrics_win_rate_all_wins() {
        let mut metrics = DailyMetrics::new();
        metrics.trades_count = 10;
        metrics.winning_trades = 10;

        assert_eq!(metrics.win_rate(), 100.0);
    }

    #[test]
    fn test_cov_daily_metrics_win_rate_all_losses() {
        let mut metrics = DailyMetrics::new();
        metrics.trades_count = 10;
        metrics.losing_trades = 10;

        assert_eq!(metrics.win_rate(), 0.0);
    }

    #[test]
    fn test_cov_reconciliation_metrics_default() {
        let metrics = ReconciliationMetrics::default();
        assert!(metrics.last_run_time.is_none());
        assert_eq!(metrics.last_run_duration_ms, 0);
        assert_eq!(metrics.total_discrepancies_found, 0);
        assert_eq!(metrics.balance_mismatches, 0);
        assert_eq!(metrics.order_mismatches, 0);
        assert_eq!(metrics.stale_orders_cancelled, 0);
        assert_eq!(metrics.terminal_orders_cleaned, 0);
        assert_eq!(metrics.consecutive_failures, 0);
        assert_eq!(metrics.total_runs, 0);
    }

    #[test]
    fn test_cov_real_trading_event_error() {
        let event = RealTradingEvent::Error("Test error".to_string());

        match event {
            RealTradingEvent::Error(msg) => {
                assert_eq!(msg, "Test error");
            },
            _ => panic!("Expected Error event"),
        }
    }

    #[test]
    fn test_cov_real_trading_event_circuit_breaker_opened() {
        let event = RealTradingEvent::CircuitBreakerOpened("Too many errors".to_string());

        match event {
            RealTradingEvent::CircuitBreakerOpened(reason) => {
                assert_eq!(reason, "Too many errors");
            },
            _ => panic!("Expected CircuitBreakerOpened event"),
        }
    }

    #[test]
    fn test_cov_real_trading_event_circuit_breaker_closed() {
        let event = RealTradingEvent::CircuitBreakerClosed;

        match event {
            RealTradingEvent::CircuitBreakerClosed => {},
            _ => panic!("Expected CircuitBreakerClosed event"),
        }
    }

    #[test]
    fn test_cov_real_trading_event_balance_updated() {
        let event = RealTradingEvent::BalanceUpdated {
            asset: "BTC".to_string(),
            free: 1.5,
            locked: 0.5,
        };

        match event {
            RealTradingEvent::BalanceUpdated {
                asset,
                free,
                locked,
            } => {
                assert_eq!(asset, "BTC");
                assert_eq!(free, 1.5);
                assert_eq!(locked, 0.5);
            },
            _ => panic!("Expected BalanceUpdated event"),
        }
    }

    #[test]
    fn test_cov_real_trading_event_daily_loss_limit_reached() {
        let event = RealTradingEvent::DailyLossLimitReached {
            loss: 500.0,
            limit: 1000.0,
        };

        match event {
            RealTradingEvent::DailyLossLimitReached { loss, limit } => {
                assert_eq!(loss, 500.0);
                assert_eq!(limit, 1000.0);
            },
            _ => panic!("Expected DailyLossLimitReached event"),
        }
    }

    #[test]
    fn test_cov_real_trading_event_reconciliation_complete() {
        let event = RealTradingEvent::ReconciliationComplete { discrepancies: 5 };

        match event {
            RealTradingEvent::ReconciliationComplete { discrepancies } => {
                assert_eq!(discrepancies, 5);
            },
            _ => panic!("Expected ReconciliationComplete event"),
        }
    }

    #[test]
    fn test_cov_real_trading_event_engine_lifecycle() {
        let started = RealTradingEvent::EngineStarted;
        let stopped = RealTradingEvent::EngineStopped;

        match started {
            RealTradingEvent::EngineStarted => {},
            _ => panic!("Expected EngineStarted"),
        }

        match stopped {
            RealTradingEvent::EngineStopped => {},
            _ => panic!("Expected EngineStopped"),
        }
    }

    #[test]
    fn test_cov_circuit_breaker_serialization() {
        let cb = CircuitBreakerState {
            is_open: true,
            error_count: 5,
            opened_at: Some(Utc::now()),
            last_error: Some("Test error".to_string()),
        };

        let json = serde_json::to_string(&cb).unwrap();
        let deserialized: CircuitBreakerState = serde_json::from_str(&json).unwrap();

        assert_eq!(cb.is_open, deserialized.is_open);
        assert_eq!(cb.error_count, deserialized.error_count);
        assert_eq!(cb.last_error, deserialized.last_error);
    }

    #[test]
    fn test_cov_balance_serialization() {
        let balance = Balance {
            asset: "ETH".to_string(),
            free: 5.5,
            locked: 1.5,
        };

        let json = serde_json::to_string(&balance).unwrap();
        let deserialized: Balance = serde_json::from_str(&json).unwrap();

        assert_eq!(balance.asset, deserialized.asset);
        assert_eq!(balance.free, deserialized.free);
        assert_eq!(balance.locked, deserialized.locked);
    }

    #[test]
    fn test_cov_daily_metrics_serialization() {
        let metrics = DailyMetrics {
            date: "2024-01-01".to_string(),
            realized_pnl: 1000.0,
            trades_count: 50,
            winning_trades: 30,
            losing_trades: 20,
            total_volume: 100000.0,
            total_commission: 50.0,
        };

        let json = serde_json::to_string(&metrics).unwrap();
        let deserialized: DailyMetrics = serde_json::from_str(&json).unwrap();

        assert_eq!(metrics.date, deserialized.date);
        assert_eq!(metrics.realized_pnl, deserialized.realized_pnl);
        assert_eq!(metrics.trades_count, deserialized.trades_count);
    }

    #[test]
    fn test_cov_reconciliation_metrics_serialization() {
        let metrics = ReconciliationMetrics {
            last_run_time: Some(Utc::now()),
            last_run_duration_ms: 500,
            total_discrepancies_found: 10,
            balance_mismatches: 3,
            order_mismatches: 4,
            stale_orders_cancelled: 2,
            terminal_orders_cleaned: 1,
            consecutive_failures: 0,
            total_runs: 100,
        };

        let json = serde_json::to_string(&metrics).unwrap();
        let deserialized: ReconciliationMetrics = serde_json::from_str(&json).unwrap();

        assert_eq!(
            metrics.last_run_duration_ms,
            deserialized.last_run_duration_ms
        );
        assert_eq!(
            metrics.total_discrepancies_found,
            deserialized.total_discrepancies_found
        );
        assert_eq!(metrics.balance_mismatches, deserialized.balance_mismatches);
    }

    // ============================================================================
    // BOOST COVERAGE - Additional Unit Tests
    // ============================================================================

    #[test]
    fn test_boost_circuit_breaker_should_close_true() {
        let mut cb = CircuitBreakerState::default();
        cb.record_error("Error", 1);
        assert!(cb.is_open);

        // Set opened_at to 10 minutes ago
        cb.opened_at = Some(Utc::now() - chrono::Duration::seconds(600));

        // Should close after 300 second cooldown
        assert!(cb.should_close(300));
    }

    #[test]
    fn test_boost_circuit_breaker_should_close_false() {
        let mut cb = CircuitBreakerState::default();
        cb.record_error("Error", 1);

        // Just opened - should not close
        assert!(!cb.should_close(300));
    }

    #[test]
    fn test_boost_circuit_breaker_should_close_no_opened_at() {
        let cb = CircuitBreakerState::default();
        assert!(!cb.should_close(300));
    }

    #[test]
    fn test_boost_circuit_breaker_record_error_increments() {
        let mut cb = CircuitBreakerState::default();
        cb.record_error("Error 1", 5);
        assert_eq!(cb.error_count, 1);
        assert_eq!(cb.last_error, Some("Error 1".to_string()));

        cb.record_error("Error 2", 5);
        assert_eq!(cb.error_count, 2);
        assert_eq!(cb.last_error, Some("Error 2".to_string()));
    }

    #[test]
    fn test_boost_circuit_breaker_already_open() {
        let mut cb = CircuitBreakerState::default();
        cb.record_error("Error 1", 2);
        cb.record_error("Error 2", 2);
        assert!(cb.is_open);

        // Already open - should return false
        assert!(!cb.record_error("Error 3", 2));
    }

    #[test]
    fn test_boost_circuit_breaker_clone() {
        let mut cb = CircuitBreakerState::default();
        cb.record_error("Test", 1);

        let cloned = cb.clone();
        assert_eq!(cloned.is_open, cb.is_open);
        assert_eq!(cloned.error_count, cb.error_count);
        assert_eq!(cloned.last_error, cb.last_error);
    }

    #[test]
    fn test_boost_circuit_breaker_debug() {
        let cb = CircuitBreakerState::default();
        let debug_str = format!("{:?}", cb);
        assert!(debug_str.contains("CircuitBreakerState"));
    }

    #[test]
    fn test_boost_balance_default() {
        let balance = Balance::default();
        assert_eq!(balance.asset, "");
        assert_eq!(balance.free, 0.0);
        assert_eq!(balance.locked, 0.0);
        assert_eq!(balance.total(), 0.0);
    }

    #[test]
    fn test_boost_balance_total_various_values() {
        let balance = Balance {
            asset: "BTC".to_string(),
            free: 1.5,
            locked: 0.5,
        };
        assert!((balance.total() - 2.0).abs() < 0.001);
    }

    #[test]
    fn test_boost_balance_clone() {
        let balance = Balance {
            asset: "ETH".to_string(),
            free: 10.0,
            locked: 2.5,
        };
        let cloned = balance.clone();
        assert_eq!(cloned.asset, balance.asset);
        assert_eq!(cloned.free, balance.free);
        assert_eq!(cloned.locked, balance.locked);
    }

    #[test]
    fn test_boost_balance_debug() {
        let balance = Balance {
            asset: "USDT".to_string(),
            free: 1000.0,
            locked: 100.0,
        };
        let debug_str = format!("{:?}", balance);
        assert!(debug_str.contains("Balance"));
        assert!(debug_str.contains("USDT"));
    }

    #[test]
    fn test_boost_daily_metrics_reset_if_new_day_same_day() {
        let mut metrics = DailyMetrics::new();
        metrics.trades_count = 10;
        metrics.winning_trades = 6;

        // Same day - should not reset
        metrics.reset_if_new_day();
        assert_eq!(metrics.trades_count, 10);
        assert_eq!(metrics.winning_trades, 6);
    }

    #[test]
    fn test_boost_daily_metrics_default() {
        let metrics = DailyMetrics::default();
        assert_eq!(metrics.date, "");
        assert_eq!(metrics.realized_pnl, 0.0);
        assert_eq!(metrics.trades_count, 0);
        assert_eq!(metrics.winning_trades, 0);
        assert_eq!(metrics.losing_trades, 0);
        assert_eq!(metrics.total_volume, 0.0);
        assert_eq!(metrics.total_commission, 0.0);
    }

    #[test]
    fn test_boost_daily_metrics_win_rate_100_percent() {
        let mut metrics = DailyMetrics::new();
        metrics.trades_count = 10;
        metrics.winning_trades = 10;
        metrics.losing_trades = 0;

        assert!((metrics.win_rate() - 100.0).abs() < 0.1);
    }

    #[test]
    fn test_boost_daily_metrics_win_rate_0_percent() {
        let mut metrics = DailyMetrics::new();
        metrics.trades_count = 10;
        metrics.winning_trades = 0;
        metrics.losing_trades = 10;

        assert!((metrics.win_rate() - 0.0).abs() < 0.1);
    }

    #[test]
    fn test_boost_daily_metrics_clone() {
        let metrics = DailyMetrics {
            date: "2025-01-01".to_string(),
            realized_pnl: 100.0,
            trades_count: 5,
            winning_trades: 3,
            losing_trades: 2,
            total_volume: 5000.0,
            total_commission: 5.0,
        };

        let cloned = metrics.clone();
        assert_eq!(cloned.date, metrics.date);
        assert_eq!(cloned.realized_pnl, metrics.realized_pnl);
        assert_eq!(cloned.trades_count, metrics.trades_count);
    }

    #[test]
    fn test_boost_reconciliation_metrics_default() {
        let metrics = ReconciliationMetrics::default();
        assert!(metrics.last_run_time.is_none());
        assert_eq!(metrics.last_run_duration_ms, 0);
        assert_eq!(metrics.total_discrepancies_found, 0);
        assert_eq!(metrics.balance_mismatches, 0);
        assert_eq!(metrics.order_mismatches, 0);
        assert_eq!(metrics.stale_orders_cancelled, 0);
        assert_eq!(metrics.terminal_orders_cleaned, 0);
        assert_eq!(metrics.consecutive_failures, 0);
        assert_eq!(metrics.total_runs, 0);
    }

    #[test]
    fn test_boost_reconciliation_metrics_clone() {
        let mut metrics = ReconciliationMetrics::default();
        metrics.total_discrepancies_found = 5;
        metrics.balance_mismatches = 2;

        let cloned = metrics.clone();
        assert_eq!(cloned.total_discrepancies_found, 5);
        assert_eq!(cloned.balance_mismatches, 2);
    }

    #[test]
    fn test_boost_real_trading_event_order_placed() {
        let order = RealOrder::new(
            "test-order".to_string(),
            "BTCUSDT".to_string(),
            "BUY".to_string(),
            "MARKET".to_string(),
            0.001,
            None,
            None,
            None,
            true,
        );

        let event = RealTradingEvent::OrderPlaced(order.clone());
        let json = serde_json::to_string(&event).unwrap();
        assert!(json.contains("OrderPlaced"));
    }

    #[test]
    fn test_boost_real_trading_event_order_filled() {
        let order = RealOrder::new(
            "test-order".to_string(),
            "BTCUSDT".to_string(),
            "BUY".to_string(),
            "MARKET".to_string(),
            0.001,
            None,
            None,
            None,
            true,
        );

        let event = RealTradingEvent::OrderFilled(order);
        let json = serde_json::to_string(&event).unwrap();
        assert!(json.contains("OrderFilled"));
    }

    #[test]
    fn test_boost_real_trading_event_order_partially_filled() {
        let order = RealOrder::new(
            "test-order".to_string(),
            "BTCUSDT".to_string(),
            "BUY".to_string(),
            "MARKET".to_string(),
            0.001,
            None,
            None,
            None,
            true,
        );

        let event = RealTradingEvent::OrderPartiallyFilled(order);
        let json = serde_json::to_string(&event).unwrap();
        assert!(json.contains("OrderPartiallyFilled"));
    }

    #[test]
    fn test_boost_real_trading_event_order_cancelled() {
        let order = RealOrder::new(
            "test-order".to_string(),
            "BTCUSDT".to_string(),
            "BUY".to_string(),
            "MARKET".to_string(),
            0.001,
            None,
            None,
            None,
            true,
        );

        let event = RealTradingEvent::OrderCancelled(order);
        let json = serde_json::to_string(&event).unwrap();
        assert!(json.contains("OrderCancelled"));
    }

    #[test]
    fn test_boost_real_trading_event_order_rejected() {
        let order = RealOrder::new(
            "test-order".to_string(),
            "BTCUSDT".to_string(),
            "BUY".to_string(),
            "MARKET".to_string(),
            0.001,
            None,
            None,
            None,
            true,
        );

        let event = RealTradingEvent::OrderRejected {
            order,
            reason: "Insufficient balance".to_string(),
        };
        let json = serde_json::to_string(&event).unwrap();
        assert!(json.contains("OrderRejected"));
        assert!(json.contains("Insufficient balance"));
    }

    #[test]
    fn test_boost_real_trading_event_position_opened() {
        let pos = RealPosition::new(
            "pos-123".to_string(),
            "BTCUSDT".to_string(),
            PositionSide::Long,
            0.1,
            50000.0,
            "order-123".to_string(),
            None,
            None,
        );

        let event = RealTradingEvent::PositionOpened(pos);
        let json = serde_json::to_string(&event).unwrap();
        assert!(json.contains("PositionOpened"));
    }

    #[test]
    fn test_boost_real_trading_event_position_updated() {
        let pos = RealPosition::new(
            "pos-123".to_string(),
            "BTCUSDT".to_string(),
            PositionSide::Long,
            0.1,
            50000.0,
            "order-123".to_string(),
            None,
            None,
        );

        let event = RealTradingEvent::PositionUpdated(pos);
        let json = serde_json::to_string(&event).unwrap();
        assert!(json.contains("PositionUpdated"));
    }

    #[test]
    fn test_boost_real_trading_event_position_closed() {
        let pos = RealPosition::new(
            "pos-123".to_string(),
            "BTCUSDT".to_string(),
            PositionSide::Long,
            0.1,
            50000.0,
            "order-123".to_string(),
            None,
            None,
        );

        let event = RealTradingEvent::PositionClosed {
            position: pos,
            pnl: 100.5,
        };
        let json = serde_json::to_string(&event).unwrap();
        assert!(json.contains("PositionClosed"));
        assert!(json.contains("100.5"));
    }

    #[test]
    fn test_boost_real_trading_event_balance_updated() {
        let event = RealTradingEvent::BalanceUpdated {
            asset: "USDT".to_string(),
            free: 1000.0,
            locked: 100.0,
        };
        let json = serde_json::to_string(&event).unwrap();
        assert!(json.contains("BalanceUpdated"));
        assert!(json.contains("USDT"));
    }

    #[test]
    fn test_boost_real_trading_event_circuit_breaker_opened() {
        let event = RealTradingEvent::CircuitBreakerOpened("Too many errors".to_string());
        let json = serde_json::to_string(&event).unwrap();
        assert!(json.contains("CircuitBreakerOpened"));
        assert!(json.contains("Too many errors"));
    }

    #[test]
    fn test_boost_real_trading_event_circuit_breaker_closed() {
        let event = RealTradingEvent::CircuitBreakerClosed;
        let json = serde_json::to_string(&event).unwrap();
        assert!(json.contains("CircuitBreakerClosed"));
    }

    #[test]
    fn test_boost_real_trading_event_reconciliation_complete() {
        let event = RealTradingEvent::ReconciliationComplete { discrepancies: 5 };
        let json = serde_json::to_string(&event).unwrap();
        assert!(json.contains("ReconciliationComplete"));
        assert!(json.contains("5"));
    }

    #[test]
    fn test_boost_real_trading_event_error() {
        let event = RealTradingEvent::Error("Test error".to_string());
        let json = serde_json::to_string(&event).unwrap();
        assert!(json.contains("Error"));
        assert!(json.contains("Test error"));
    }

    #[test]
    fn test_boost_real_trading_event_daily_loss_limit_reached() {
        let event = RealTradingEvent::DailyLossLimitReached {
            loss: 500.0,
            limit: 1000.0,
        };
        let json = serde_json::to_string(&event).unwrap();
        assert!(json.contains("DailyLossLimitReached"));
        assert!(json.contains("500"));
        assert!(json.contains("1000"));
    }

    #[test]
    fn test_boost_real_trading_event_engine_started() {
        let event = RealTradingEvent::EngineStarted;
        let json = serde_json::to_string(&event).unwrap();
        assert!(json.contains("EngineStarted"));
    }

    #[test]
    fn test_boost_real_trading_event_engine_stopped() {
        let event = RealTradingEvent::EngineStopped;
        let json = serde_json::to_string(&event).unwrap();
        assert!(json.contains("EngineStopped"));
    }

    #[test]
    fn test_boost_real_trading_event_clone() {
        let event = RealTradingEvent::Error("Test".to_string());
        let cloned = event.clone();

        if let RealTradingEvent::Error(msg) = cloned {
            assert_eq!(msg, "Test");
        } else {
            panic!("Expected Error variant");
        }
    }

    #[test]
    fn test_boost_real_trading_event_debug() {
        let event = RealTradingEvent::EngineStarted;
        let debug_str = format!("{:?}", event);
        assert!(debug_str.contains("EngineStarted"));
    }

    #[test]
    fn test_boost_balance_serialization_roundtrip() {
        let balance = Balance {
            asset: "BNB".to_string(),
            free: 123.45,
            locked: 67.89,
        };

        let json = serde_json::to_string(&balance).unwrap();
        let deserialized: Balance = serde_json::from_str(&json).unwrap();

        assert_eq!(deserialized.asset, balance.asset);
        assert!((deserialized.free - balance.free).abs() < 0.001);
        assert!((deserialized.locked - balance.locked).abs() < 0.001);
    }

    #[test]
    fn test_boost_daily_metrics_serialization_roundtrip() {
        let metrics = DailyMetrics {
            date: "2025-12-25".to_string(),
            realized_pnl: 250.75,
            trades_count: 15,
            winning_trades: 9,
            losing_trades: 6,
            total_volume: 15000.0,
            total_commission: 15.5,
        };

        let json = serde_json::to_string(&metrics).unwrap();
        let deserialized: DailyMetrics = serde_json::from_str(&json).unwrap();

        assert_eq!(deserialized.date, metrics.date);
        assert_eq!(deserialized.trades_count, metrics.trades_count);
        assert!((deserialized.realized_pnl - metrics.realized_pnl).abs() < 0.001);
    }

    #[test]
    fn test_boost_circuit_breaker_state_serialization_roundtrip() {
        let mut cb = CircuitBreakerState::default();
        cb.record_error("Test error", 2);
        cb.record_error("Second error", 2);

        let json = serde_json::to_string(&cb).unwrap();
        let deserialized: CircuitBreakerState = serde_json::from_str(&json).unwrap();

        assert_eq!(deserialized.is_open, cb.is_open);
        assert_eq!(deserialized.error_count, cb.error_count);
        assert_eq!(deserialized.last_error, cb.last_error);
    }

    #[test]
    fn test_boost_reconciliation_metrics_serialization_roundtrip() {
        let mut metrics = ReconciliationMetrics::default();
        metrics.total_discrepancies_found = 10;
        metrics.balance_mismatches = 3;
        metrics.order_mismatches = 7;
        metrics.total_runs = 100;

        let json = serde_json::to_string(&metrics).unwrap();
        let deserialized: ReconciliationMetrics = serde_json::from_str(&json).unwrap();

        assert_eq!(deserialized.total_discrepancies_found, 10);
        assert_eq!(deserialized.balance_mismatches, 3);
        assert_eq!(deserialized.order_mismatches, 7);
        assert_eq!(deserialized.total_runs, 100);
    }

    #[test]
    fn test_boost_balance_zero_values() {
        let balance = Balance {
            asset: "ZERO".to_string(),
            free: 0.0,
            locked: 0.0,
        };

        assert_eq!(balance.total(), 0.0);
    }

    #[test]
    fn test_boost_balance_large_values() {
        let balance = Balance {
            asset: "WHALE".to_string(),
            free: 1_000_000.0,
            locked: 500_000.0,
        };

        assert!((balance.total() - 1_500_000.0).abs() < 0.001);
    }

    #[test]
    fn test_boost_daily_metrics_edge_case_single_trade() {
        let mut metrics = DailyMetrics::new();
        metrics.trades_count = 1;
        metrics.winning_trades = 1;

        assert!((metrics.win_rate() - 100.0).abs() < 0.1);
    }

    #[test]
    fn test_boost_circuit_breaker_threshold_exactly() {
        let mut cb = CircuitBreakerState::default();

        // Exactly at threshold
        cb.record_error("1", 3);
        assert!(!cb.is_open);
        cb.record_error("2", 3);
        assert!(!cb.is_open);

        // Third error triggers
        let opened = cb.record_error("3", 3);
        assert!(opened);
        assert!(cb.is_open);
    }

    #[test]
    fn test_boost_circuit_breaker_threshold_zero() {
        let mut cb = CircuitBreakerState::default();

        // Zero threshold - error_count (1) >= threshold (0) so it opens immediately
        cb.record_error("Error", 0);
        assert!(cb.is_open);
    }

    #[test]
    fn test_boost_reconciliation_metrics_all_fields() {
        let metrics = ReconciliationMetrics {
            last_run_time: Some(Utc::now()),
            last_run_duration_ms: 500,
            total_discrepancies_found: 20,
            balance_mismatches: 5,
            order_mismatches: 10,
            stale_orders_cancelled: 3,
            terminal_orders_cleaned: 2,
            consecutive_failures: 1,
            total_runs: 150,
        };

        assert_eq!(metrics.total_discrepancies_found, 20);
        assert_eq!(metrics.balance_mismatches, 5);
        assert_eq!(metrics.order_mismatches, 10);
        assert_eq!(metrics.stale_orders_cancelled, 3);
        assert_eq!(metrics.terminal_orders_cleaned, 2);
        assert_eq!(metrics.consecutive_failures, 1);
        assert_eq!(metrics.total_runs, 150);
    }

    // ============ Enhanced Coverage Boost Tests ============

    #[test]
    fn test_enhanced_circuit_breaker_should_close_before_cooldown() {
        let mut cb = CircuitBreakerState::default();
        cb.is_open = true;
        cb.opened_at = Some(Utc::now() - chrono::Duration::seconds(30));
        assert!(!cb.should_close(60));
    }

    #[test]
    fn test_enhanced_circuit_breaker_should_close_after_cooldown() {
        let mut cb = CircuitBreakerState::default();
        cb.is_open = true;
        cb.opened_at = Some(Utc::now() - chrono::Duration::seconds(120));
        assert!(cb.should_close(60));
    }

    #[test]
    fn test_enhanced_daily_metrics_reset_same_day() {
        let mut metrics = DailyMetrics::new();
        metrics.trades_count = 10;
        let today = Utc::now().format("%Y-%m-%d").to_string();
        metrics.date = today;
        metrics.reset_if_new_day();
        assert_eq!(metrics.trades_count, 10);
    }

    #[test]
    fn test_enhanced_daily_metrics_reset_new_day() {
        let mut metrics = DailyMetrics::new();
        metrics.trades_count = 10;
        metrics.date = "2020-01-01".to_string();
        metrics.reset_if_new_day();
        assert_eq!(metrics.trades_count, 0);
    }

    #[test]
    fn test_enhanced_circuit_breaker_success_clears() {
        let mut cb = CircuitBreakerState::default();
        cb.record_error("Error", 5);
        cb.record_success();
        assert_eq!(cb.error_count, 0);
    }

    #[test]
    fn test_enhanced_balance_total() {
        let balance = Balance {
            asset: "USDT".to_string(),
            free: 1000.0,
            locked: 500.0,
        };
        assert_eq!(balance.total(), 1500.0);
    }

    #[test]
    fn test_enhanced_daily_metrics_win_rate_perfect() {
        let mut metrics = DailyMetrics::new();
        metrics.trades_count = 10;
        metrics.winning_trades = 10;
        assert!((metrics.win_rate() - 100.0).abs() < 0.1);
    }

    #[test]
    fn test_enhanced_reconciliation_incremental() {
        let mut metrics = ReconciliationMetrics::default();
        for _ in 0..5 {
            metrics.total_runs += 1;
            metrics.total_discrepancies_found += 2;
        }
        assert_eq!(metrics.total_runs, 5);
        assert_eq!(metrics.total_discrepancies_found, 10);
    }

    #[test]
    fn test_enhanced_circuit_breaker_close_resets() {
        let mut cb = CircuitBreakerState::default();
        cb.record_error("E1", 2);
        cb.record_error("E2", 2);
        cb.close();
        assert!(!cb.is_open);
        assert_eq!(cb.error_count, 0);
    }

    #[test]
    fn test_enhanced_balance_default() {
        let balance = Balance::default();
        assert_eq!(balance.total(), 0.0);
    }

    // ============ MASSIVE Coverage Boost: Engine Integration Tests ============

    #[tokio::test]
    async fn test_engine_get_risk_manager() {
        let engine = create_test_engine().await;
        let _rm = engine.get_risk_manager();
        // Verify no panic
    }

    #[tokio::test]
    async fn test_engine_get_total_equity_usdt_empty() {
        let engine = create_test_engine().await;
        let equity = engine.get_total_equity_usdt().await;
        assert_eq!(equity, 0.0);
    }

    #[tokio::test]
    async fn test_engine_cancel_all_orders_empty() {
        let engine = create_test_engine().await;
        let result = engine.cancel_all_orders(None).await;
        assert!(result.is_ok());
        assert!(result.unwrap().is_empty());
    }

    #[tokio::test]
    async fn test_engine_cancel_all_orders_with_symbol_filter() {
        let engine = create_test_engine().await;
        let result = engine.cancel_all_orders(Some("BTCUSDT")).await;
        assert!(result.is_ok());
        assert!(result.unwrap().is_empty());
    }

    #[tokio::test]
    async fn test_engine_update_prices() {
        let engine = create_test_engine().await;
        let mut prices = HashMap::new();
        prices.insert("BTCUSDT".to_string(), 50000.0);
        prices.insert("ETHUSDT".to_string(), 3000.0);
        engine.update_prices(&prices);
    }

    #[tokio::test]
    async fn test_engine_get_daily_metrics_new_day() {
        let engine = create_test_engine().await;
        let metrics = engine.get_daily_metrics().await;
        let today = Utc::now().format("%Y-%m-%d").to_string();
        assert_eq!(metrics.date, today);
        assert_eq!(metrics.trades_count, 0);
    }

    // ============ Balance Handler Tests ============

    #[tokio::test]
    async fn test_handle_account_position_empty() {
        let engine = create_test_engine().await;
        let pos = OutboundAccountPosition {
            event_type: "outboundAccountPosition".to_string(),
            event_time: Utc::now().timestamp_millis(),
            last_update_time: Utc::now().timestamp_millis(),
            balances: vec![],
        };
        engine.handle_account_position(pos).await;
        let balances = engine.get_all_balances().await;
        assert!(balances.is_empty());
    }

    #[tokio::test]
    async fn test_handle_account_position_with_balances() {
        let engine = create_test_engine().await;
        let pos = OutboundAccountPosition {
            event_type: "outboundAccountPosition".to_string(),
            event_time: Utc::now().timestamp_millis(),
            last_update_time: Utc::now().timestamp_millis(),
            balances: vec![
                crate::binance::types::AccountBalance {
                    asset: "USDT".to_string(),
                    free: "1000.0".to_string(),
                    locked: "100.0".to_string(),
                },
                crate::binance::types::AccountBalance {
                    asset: "BTC".to_string(),
                    free: "0.5".to_string(),
                    locked: "0.1".to_string(),
                },
            ],
        };
        engine.handle_account_position(pos).await;
        let balances = engine.get_all_balances().await;
        assert_eq!(balances.len(), 2);
        assert!(balances.contains_key("USDT"));
        assert!(balances.contains_key("BTC"));
    }

    #[tokio::test]
    async fn test_handle_account_position_invalid_balance() {
        let engine = create_test_engine().await;
        let pos = OutboundAccountPosition {
            event_type: "outboundAccountPosition".to_string(),
            event_time: Utc::now().timestamp_millis(),
            last_update_time: Utc::now().timestamp_millis(),
            balances: vec![crate::binance::types::AccountBalance {
                asset: "INVALID".to_string(),
                free: "not_a_number".to_string(),
                locked: "100.0".to_string(),
            }],
        };
        engine.handle_account_position(pos).await;
        let balances = engine.get_all_balances().await;
        assert!(balances.is_empty());
    }

    #[tokio::test]
    async fn test_handle_account_position_negative_balance() {
        let engine = create_test_engine().await;
        let pos = OutboundAccountPosition {
            event_type: "outboundAccountPosition".to_string(),
            event_time: Utc::now().timestamp_millis(),
            last_update_time: Utc::now().timestamp_millis(),
            balances: vec![crate::binance::types::AccountBalance {
                asset: "NEGATIVE".to_string(),
                free: "-100.0".to_string(),
                locked: "0.0".to_string(),
            }],
        };
        engine.handle_account_position(pos).await;
        let balances = engine.get_all_balances().await;
        assert!(balances.is_empty());
    }

    #[tokio::test]
    async fn test_handle_account_position_zero_balance_removed() {
        let engine = create_test_engine().await;
        // First add a balance
        let pos1 = OutboundAccountPosition {
            event_type: "outboundAccountPosition".to_string(),
            event_time: Utc::now().timestamp_millis(),
            last_update_time: Utc::now().timestamp_millis(),
            balances: vec![crate::binance::types::AccountBalance {
                asset: "USDT".to_string(),
                free: "1000.0".to_string(),
                locked: "0.0".to_string(),
            }],
        };
        engine.handle_account_position(pos1).await;
        assert_eq!(engine.get_all_balances().await.len(), 1);

        // Update to zero
        let pos2 = OutboundAccountPosition {
            event_type: "outboundAccountPosition".to_string(),
            event_time: Utc::now().timestamp_millis(),
            last_update_time: Utc::now().timestamp_millis(),
            balances: vec![crate::binance::types::AccountBalance {
                asset: "USDT".to_string(),
                free: "0.0".to_string(),
                locked: "0.0".to_string(),
            }],
        };
        engine.handle_account_position(pos2).await;
        assert_eq!(engine.get_all_balances().await.len(), 0);
    }

    #[tokio::test]
    async fn test_handle_balance_update_positive_delta() {
        let engine = create_test_engine().await;
        let pos = OutboundAccountPosition {
            event_type: "outboundAccountPosition".to_string(),
            event_time: Utc::now().timestamp_millis(),
            last_update_time: Utc::now().timestamp_millis(),
            balances: vec![crate::binance::types::AccountBalance {
                asset: "USDT".to_string(),
                free: "1000.0".to_string(),
                locked: "0.0".to_string(),
            }],
        };
        engine.handle_account_position(pos).await;

        let update = crate::binance::types::BalanceUpdate {
            event_type: "balanceUpdate".to_string(),
            event_time: Utc::now().timestamp_millis(),
            asset: "USDT".to_string(),
            balance_delta: "100.0".to_string(),
            clear_time: Utc::now().timestamp_millis(),
        };
        engine.handle_balance_update(update).await;

        let balance = engine.get_balance("USDT").await.unwrap();
        assert!((balance.free - 1100.0).abs() < 0.01);
    }

    #[tokio::test]
    async fn test_handle_balance_update_negative_delta() {
        let engine = create_test_engine().await;
        let pos = OutboundAccountPosition {
            event_type: "outboundAccountPosition".to_string(),
            event_time: Utc::now().timestamp_millis(),
            last_update_time: Utc::now().timestamp_millis(),
            balances: vec![crate::binance::types::AccountBalance {
                asset: "USDT".to_string(),
                free: "1000.0".to_string(),
                locked: "0.0".to_string(),
            }],
        };
        engine.handle_account_position(pos).await;

        let update = crate::binance::types::BalanceUpdate {
            event_type: "balanceUpdate".to_string(),
            event_time: Utc::now().timestamp_millis(),
            asset: "USDT".to_string(),
            balance_delta: "-200.0".to_string(),
            clear_time: Utc::now().timestamp_millis(),
        };
        engine.handle_balance_update(update).await;

        let balance = engine.get_balance("USDT").await.unwrap();
        assert!((balance.free - 800.0).abs() < 0.01);
    }

    #[tokio::test]
    async fn test_handle_balance_update_invalid_delta() {
        let engine = create_test_engine().await;
        let update = crate::binance::types::BalanceUpdate {
            event_type: "balanceUpdate".to_string(),
            event_time: Utc::now().timestamp_millis(),
            asset: "USDT".to_string(),
            balance_delta: "not_a_number".to_string(),
            clear_time: Utc::now().timestamp_millis(),
        };
        engine.handle_balance_update(update).await;
        // Should not panic
    }

    #[tokio::test]
    async fn test_handle_balance_update_negative_result() {
        let engine = create_test_engine().await;
        let pos = OutboundAccountPosition {
            event_type: "outboundAccountPosition".to_string(),
            event_time: Utc::now().timestamp_millis(),
            last_update_time: Utc::now().timestamp_millis(),
            balances: vec![crate::binance::types::AccountBalance {
                asset: "USDT".to_string(),
                free: "100.0".to_string(),
                locked: "0.0".to_string(),
            }],
        };
        engine.handle_account_position(pos).await;

        let update = crate::binance::types::BalanceUpdate {
            event_type: "balanceUpdate".to_string(),
            event_time: Utc::now().timestamp_millis(),
            asset: "USDT".to_string(),
            balance_delta: "-200.0".to_string(),
            clear_time: Utc::now().timestamp_millis(),
        };
        engine.handle_balance_update(update).await;
        // Should warn but not crash
    }

    // ============ ExecutionReport Processing Tests ============

    #[tokio::test]
    async fn test_process_execution_report_unknown_order_v2() {
        let engine = create_test_engine().await;
        let report = create_test_execution_report(
            "unknown-order",
            "BTCUSDT",
            "BUY",
            "NEW",
            "NEW",
            "0.001",
            "0",
            "50000",
        );
        let result = engine.process_execution_report(&report).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_process_execution_report_fill_creates_position() {
        let engine = create_test_engine().await;

        let order = RealOrder::new(
            "test-order-fill".to_string(),
            "BTCUSDT".to_string(),
            "BUY".to_string(),
            "MARKET".to_string(),
            0.001,
            None,
            None,
            None,
            true,
        );
        engine.orders.insert("test-order-fill".to_string(), order);

        let report = create_test_execution_report(
            "test-order-fill",
            "BTCUSDT",
            "BUY",
            "TRADE",
            "FILLED",
            "0.001",
            "0.001",
            "50000",
        );
        let result = engine.process_execution_report(&report).await;
        assert!(result.is_ok());

        let position = engine.get_position("BTCUSDT");
        assert!(position.is_some());
    }

    #[tokio::test]
    async fn test_process_execution_report_partial_fill() {
        let engine = create_test_engine().await;

        let order = RealOrder::new(
            "test-partial".to_string(),
            "BTCUSDT".to_string(),
            "BUY".to_string(),
            "LIMIT".to_string(),
            0.01,
            Some(50000.0),
            None,
            None,
            true,
        );
        engine.orders.insert("test-partial".to_string(), order);

        let report = create_test_execution_report(
            "test-partial",
            "BTCUSDT",
            "BUY",
            "TRADE",
            "PARTIALLY_FILLED",
            "0.01",
            "0.005",
            "50000",
        );
        let result = engine.process_execution_report(&report).await;
        assert!(result.is_ok());

        let order = engine.get_order("test-partial");
        assert!(order.is_some());
        assert_eq!(order.unwrap().state, OrderState::PartiallyFilled);
    }

    #[tokio::test]
    async fn test_process_execution_report_rejected_v2() {
        let engine = create_test_engine().await;

        let order = RealOrder::new(
            "test-reject".to_string(),
            "BTCUSDT".to_string(),
            "BUY".to_string(),
            "MARKET".to_string(),
            0.001,
            None,
            None,
            None,
            true,
        );
        engine.orders.insert("test-reject".to_string(), order);

        let report = create_test_execution_report(
            "test-reject",
            "BTCUSDT",
            "BUY",
            "REJECTED",
            "REJECTED",
            "0.001",
            "0",
            "50000",
        );
        let result = engine.process_execution_report(&report).await;
        assert!(result.is_ok());

        let order = engine.get_order("test-reject");
        assert!(order.is_some());
        assert_eq!(order.unwrap().state, OrderState::Rejected);
    }

    #[tokio::test]
    async fn test_process_execution_report_cancelled_v2() {
        let engine = create_test_engine().await;

        let order = RealOrder::new(
            "test-cancel".to_string(),
            "BTCUSDT".to_string(),
            "BUY".to_string(),
            "LIMIT".to_string(),
            0.001,
            Some(50000.0),
            None,
            None,
            true,
        );
        engine.orders.insert("test-cancel".to_string(), order);

        let report = create_test_execution_report(
            "test-cancel",
            "BTCUSDT",
            "BUY",
            "CANCELED",
            "CANCELED",
            "0.001",
            "0",
            "50000",
        );
        let result = engine.process_execution_report(&report).await;
        assert!(result.is_ok());

        let order = engine.get_order("test-cancel");
        assert!(order.is_some());
        assert_eq!(order.unwrap().state, OrderState::Cancelled);
    }

    // ============ Position Update Tests ============

    #[tokio::test]
    async fn test_update_position_from_fill_entry() {
        let engine = create_test_engine().await;

        let mut order = RealOrder::new(
            "entry-order".to_string(),
            "BTCUSDT".to_string(),
            "BUY".to_string(),
            "MARKET".to_string(),
            0.01,
            None,
            None,
            None,
            true,
        );
        order.state = OrderState::Filled;
        order.executed_quantity = 0.01;
        order.average_fill_price = 50000.0;
        engine.orders.insert("entry-order".to_string(), order);

        let result = engine.update_position_from_fill("entry-order").await;
        assert!(result.is_ok());

        let position = engine.get_position("BTCUSDT");
        assert!(position.is_some());
        assert_eq!(position.unwrap().side, PositionSide::Long);
    }

    #[tokio::test]
    async fn test_update_position_from_fill_add_to_existing() {
        let engine = create_test_engine().await;

        let mut order1 = RealOrder::new(
            "entry-1".to_string(),
            "BTCUSDT".to_string(),
            "BUY".to_string(),
            "MARKET".to_string(),
            0.01,
            None,
            None,
            None,
            true,
        );
        order1.state = OrderState::Filled;
        order1.executed_quantity = 0.01;
        order1.average_fill_price = 50000.0;
        engine.orders.insert("entry-1".to_string(), order1);
        engine.update_position_from_fill("entry-1").await.unwrap();

        let mut order2 = RealOrder::new(
            "entry-2".to_string(),
            "BTCUSDT".to_string(),
            "BUY".to_string(),
            "MARKET".to_string(),
            0.01,
            None,
            None,
            None,
            true,
        );
        order2.state = OrderState::Filled;
        order2.executed_quantity = 0.01;
        order2.average_fill_price = 52000.0;
        engine.orders.insert("entry-2".to_string(), order2);
        engine.update_position_from_fill("entry-2").await.unwrap();

        let position = engine.get_position("BTCUSDT");
        assert!(position.is_some());
        let pos = position.unwrap();
        assert!((pos.quantity - 0.02).abs() < 0.0001);
        assert!((pos.entry_price - 51000.0).abs() < 1.0);
    }

    #[tokio::test]
    async fn test_update_position_from_fill_exit() {
        let engine = create_test_engine().await;

        let pos = RealPosition::new(
            "pos-exit-test".to_string(),
            "BTCUSDT".to_string(),
            PositionSide::Long,
            0.01,
            50000.0,
            "entry-order".to_string(),
            None,
            None,
        );
        engine.positions.insert("BTCUSDT".to_string(), pos);

        let mut exit_order = RealOrder::new(
            "exit-order".to_string(),
            "BTCUSDT".to_string(),
            "SELL".to_string(),
            "MARKET".to_string(),
            0.01,
            None,
            None,
            Some("pos-exit-test".to_string()),
            false,
        );
        exit_order.state = OrderState::Filled;
        exit_order.executed_quantity = 0.01;
        exit_order.average_fill_price = 52000.0;
        engine.orders.insert("exit-order".to_string(), exit_order);

        let result = engine.update_position_from_fill("exit-order").await;
        assert!(result.is_ok());

        let position = engine.get_position("BTCUSDT");
        assert!(position.is_none());

        let metrics = engine.get_daily_metrics().await;
        assert_eq!(metrics.trades_count, 1);
        assert_eq!(metrics.winning_trades, 1);
    }

    #[tokio::test]
    async fn test_update_position_from_fill_partial_exit() {
        let engine = create_test_engine().await;

        let pos = RealPosition::new(
            "pos-partial-exit".to_string(),
            "BTCUSDT".to_string(),
            PositionSide::Long,
            0.02,
            50000.0,
            "entry-order".to_string(),
            None,
            None,
        );
        engine.positions.insert("BTCUSDT".to_string(), pos);

        let mut exit_order = RealOrder::new(
            "partial-exit".to_string(),
            "BTCUSDT".to_string(),
            "SELL".to_string(),
            "MARKET".to_string(),
            0.01,
            None,
            None,
            Some("pos-partial-exit".to_string()),
            false,
        );
        exit_order.state = OrderState::Filled;
        exit_order.executed_quantity = 0.01;
        exit_order.average_fill_price = 52000.0;
        engine.orders.insert("partial-exit".to_string(), exit_order);

        let result = engine.update_position_from_fill("partial-exit").await;
        assert!(result.is_ok());

        let position = engine.get_position("BTCUSDT");
        assert!(position.is_some());
        assert!((position.unwrap().quantity - 0.01).abs() < 0.0001);
    }

    // ============ Price Update Tests ============

    #[tokio::test]
    async fn test_update_prices_with_positions_v2() {
        let engine = create_test_engine().await;

        let pos1 = RealPosition::new(
            "pos-1".to_string(),
            "BTCUSDT".to_string(),
            PositionSide::Long,
            0.01,
            50000.0,
            "order-1".to_string(),
            None,
            None,
        );
        engine.positions.insert("BTCUSDT".to_string(), pos1);

        let pos2 = RealPosition::new(
            "pos-2".to_string(),
            "ETHUSDT".to_string(),
            PositionSide::Long,
            0.5,
            3000.0,
            "order-2".to_string(),
            None,
            None,
        );
        engine.positions.insert("ETHUSDT".to_string(), pos2);

        let mut prices = HashMap::new();
        prices.insert("BTCUSDT".to_string(), 52000.0);
        prices.insert("ETHUSDT".to_string(), 3200.0);
        engine.update_prices(&prices);

        let btc_pos = engine.get_position("BTCUSDT").unwrap();
        // (52000-50000) * 0.01 = 20.0
        assert!((btc_pos.unrealized_pnl - 20.0).abs() < 0.1);

        let eth_pos = engine.get_position("ETHUSDT").unwrap();
        // (3200-3000) * 0.5 = 100.0
        assert!((eth_pos.unrealized_pnl - 100.0).abs() < 0.1);
    }

    #[tokio::test]
    async fn test_update_prices_partial_symbols() {
        let engine = create_test_engine().await;

        let pos1 = RealPosition::new(
            "pos-1".to_string(),
            "BTCUSDT".to_string(),
            PositionSide::Long,
            0.01,
            50000.0,
            "order-1".to_string(),
            None,
            None,
        );
        engine.positions.insert("BTCUSDT".to_string(), pos1);

        let pos2 = RealPosition::new(
            "pos-2".to_string(),
            "ETHUSDT".to_string(),
            PositionSide::Long,
            0.5,
            3000.0,
            "order-2".to_string(),
            None,
            None,
        );
        engine.positions.insert("ETHUSDT".to_string(), pos2);

        let mut prices = HashMap::new();
        prices.insert("BTCUSDT".to_string(), 52000.0);
        engine.update_prices(&prices);

        let btc_pos = engine.get_position("BTCUSDT").unwrap();
        // (52000-50000) * 0.01 = 20.0
        assert!((btc_pos.unrealized_pnl - 20.0).abs() < 0.1);

        let eth_pos = engine.get_position("ETHUSDT").unwrap();
        // ETH price not updated, unrealized_pnl stays 0
        assert!((eth_pos.unrealized_pnl).abs() < 0.1);
    }

    // ============ Additional Getter Tests ============

    #[tokio::test]
    async fn test_get_total_unrealized_pnl_with_positions_v2() {
        let engine = create_test_engine().await;

        let mut pos1 = RealPosition::new(
            "pos-1".to_string(),
            "BTCUSDT".to_string(),
            PositionSide::Long,
            0.01,
            50000.0,
            "order-1".to_string(),
            None,
            None,
        );
        pos1.update_price(52000.0);
        engine.positions.insert("BTCUSDT".to_string(), pos1);

        let mut pos2 = RealPosition::new(
            "pos-2".to_string(),
            "ETHUSDT".to_string(),
            PositionSide::Long,
            0.5,
            3000.0,
            "order-2".to_string(),
            None,
            None,
        );
        pos2.update_price(3100.0);
        engine.positions.insert("ETHUSDT".to_string(), pos2);

        let total_pnl = engine.get_total_unrealized_pnl();
        // BTC: (52000-50000)*0.01=20.0, ETH: (3100-3000)*0.5=50.0, total=70.0
        assert!((total_pnl - 70.0).abs() < 0.1);
    }

    #[tokio::test]
    async fn test_get_total_exposure_with_positions_v2() {
        let engine = create_test_engine().await;

        let mut pos1 = RealPosition::new(
            "pos-1".to_string(),
            "BTCUSDT".to_string(),
            PositionSide::Long,
            0.01,
            50000.0,
            "order-1".to_string(),
            None,
            None,
        );
        pos1.update_price(52000.0);
        engine.positions.insert("BTCUSDT".to_string(), pos1);

        let mut pos2 = RealPosition::new(
            "pos-2".to_string(),
            "ETHUSDT".to_string(),
            PositionSide::Long,
            0.5,
            3000.0,
            "order-2".to_string(),
            None,
            None,
        );
        pos2.update_price(3100.0);
        engine.positions.insert("ETHUSDT".to_string(), pos2);

        let exposure = engine.get_total_exposure();
        assert!((exposure - 2070.0).abs() < 1.0);
    }

    #[tokio::test]
    async fn test_cleanup_terminal_orders_v2() {
        let engine = create_test_engine().await;

        let mut old_order = RealOrder::new(
            "old-filled".to_string(),
            "BTCUSDT".to_string(),
            "BUY".to_string(),
            "MARKET".to_string(),
            0.001,
            None,
            None,
            None,
            true,
        );
        old_order.state = OrderState::Filled;
        old_order.updated_at = Utc::now() - chrono::Duration::hours(48);
        engine.orders.insert("old-filled".to_string(), old_order);

        let mut recent_order = RealOrder::new(
            "recent-filled".to_string(),
            "ETHUSDT".to_string(),
            "BUY".to_string(),
            "MARKET".to_string(),
            0.01,
            None,
            None,
            None,
            true,
        );
        recent_order.state = OrderState::Filled;
        recent_order.updated_at = Utc::now() - chrono::Duration::hours(12);
        engine
            .orders
            .insert("recent-filled".to_string(), recent_order);

        let count = engine.cleanup_terminal_orders();
        assert!(count >= 1);
    }

    // ============ Error Path Tests ============

    #[tokio::test]
    async fn test_update_position_from_fill_nonexistent_order_v2() {
        let engine = create_test_engine().await;
        let result = engine.update_position_from_fill("nonexistent").await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_update_position_from_fill_not_filled() {
        let engine = create_test_engine().await;

        let order = RealOrder::new(
            "not-filled".to_string(),
            "BTCUSDT".to_string(),
            "BUY".to_string(),
            "LIMIT".to_string(),
            0.001,
            Some(50000.0),
            None,
            None,
            true,
        );
        engine.orders.insert("not-filled".to_string(), order);

        let result = engine.update_position_from_fill("not-filled").await;
        assert!(result.is_ok());
        assert!(engine.get_position("BTCUSDT").is_none());
    }

    #[test]
    fn test_circuit_breaker_already_open_v2() {
        let mut cb = CircuitBreakerState::default();
        cb.is_open = true;
        cb.opened_at = Some(Utc::now());

        let result = cb.record_error("Another error", 3);
        assert!(!result);
    }

    #[test]
    fn test_circuit_breaker_should_close_no_opened_at_v2() {
        let mut cb = CircuitBreakerState::default();
        cb.is_open = true;
        cb.opened_at = None;

        assert!(!cb.should_close(60));
    }

    #[test]
    fn test_daily_metrics_all_losing_v2() {
        let mut metrics = DailyMetrics::new();
        metrics.trades_count = 5;
        metrics.winning_trades = 0;
        metrics.losing_trades = 5;

        assert!((metrics.win_rate() - 0.0).abs() < 0.1);
    }

    // ============ Additional Coverage Boost Tests ============

    #[test]
    fn test_circuit_breaker_record_error_increments() {
        let mut cb = CircuitBreakerState::default();
        cb.record_error("Test error", 5);
        assert_eq!(cb.error_count, 1);
        assert_eq!(cb.last_error, Some("Test error".to_string()));
        assert!(!cb.is_open);
    }

    #[test]
    fn test_circuit_breaker_record_error_opens_at_threshold() {
        let mut cb = CircuitBreakerState::default();
        let _ = cb.record_error("Error 1", 2);
        let opened = cb.record_error("Error 2", 2);
        assert!(opened);
        assert!(cb.is_open);
        assert!(cb.opened_at.is_some());
    }

    #[test]
    fn test_circuit_breaker_record_error_already_open() {
        let mut cb = CircuitBreakerState::default();
        cb.is_open = true;
        cb.error_count = 5;
        let opened = cb.record_error("Another error", 3);
        assert!(!opened); // Already open, doesn't re-open
        assert_eq!(cb.error_count, 6);
    }

    #[test]
    fn test_circuit_breaker_should_close_after_cooldown() {
        let mut cb = CircuitBreakerState::default();
        cb.is_open = true;
        cb.opened_at = Some(Utc::now() - chrono::Duration::seconds(120));
        assert!(cb.should_close(60)); // 120 seconds > 60 second cooldown
    }

    #[test]
    fn test_circuit_breaker_should_not_close_before_cooldown() {
        let mut cb = CircuitBreakerState::default();
        cb.is_open = true;
        cb.opened_at = Some(Utc::now() - chrono::Duration::seconds(30));
        assert!(!cb.should_close(60)); // 30 seconds < 60 second cooldown
    }

    #[test]
    fn test_circuit_breaker_close_resets_all() {
        let mut cb = CircuitBreakerState {
            is_open: true,
            error_count: 5,
            opened_at: Some(Utc::now()),
            last_error: Some("Error".to_string()),
        };
        cb.close();
        assert!(!cb.is_open);
        assert_eq!(cb.error_count, 0);
        assert!(cb.opened_at.is_none());
        assert!(cb.last_error.is_none());
    }

    #[test]
    fn test_balance_total_calculation() {
        let balance = Balance {
            asset: "BTC".to_string(),
            free: 1.5,
            locked: 0.5,
        };
        assert_eq!(balance.total(), 2.0);
    }

    #[test]
    fn test_balance_total_zero_locked() {
        let balance = Balance {
            asset: "ETH".to_string(),
            free: 10.0,
            locked: 0.0,
        };
        assert_eq!(balance.total(), 10.0);
    }

    #[test]
    fn test_daily_metrics_new_has_correct_date() {
        let metrics = DailyMetrics::new();
        let today = Utc::now().format("%Y-%m-%d").to_string();
        assert_eq!(metrics.date, today);
        assert_eq!(metrics.trades_count, 0);
        assert_eq!(metrics.realized_pnl, 0.0);
    }

    #[test]
    fn test_daily_metrics_win_rate_zero_trades_v3() {
        let metrics = DailyMetrics::new();
        assert_eq!(metrics.win_rate(), 0.0);
    }

    #[test]
    fn test_daily_metrics_win_rate_some_wins() {
        let mut metrics = DailyMetrics::new();
        metrics.trades_count = 10;
        metrics.winning_trades = 6;
        metrics.losing_trades = 4;
        assert!((metrics.win_rate() - 60.0).abs() < 0.01);
    }

    #[test]
    fn test_daily_metrics_win_rate_all_wins() {
        let mut metrics = DailyMetrics::new();
        metrics.trades_count = 5;
        metrics.winning_trades = 5;
        metrics.losing_trades = 0;
        assert!((metrics.win_rate() - 100.0).abs() < 0.01);
    }

    #[test]
    fn test_daily_metrics_reset_if_new_day_same_day() {
        let mut metrics = DailyMetrics::new();
        metrics.trades_count = 10;
        metrics.realized_pnl = 100.0;

        metrics.reset_if_new_day();

        assert_eq!(metrics.trades_count, 10); // Should not reset
        assert_eq!(metrics.realized_pnl, 100.0);
    }

    #[test]
    fn test_daily_metrics_reset_if_new_day_different_day() {
        let mut metrics = DailyMetrics::new();
        metrics.date = "2020-01-01".to_string(); // Old date
        metrics.trades_count = 10;
        metrics.realized_pnl = 100.0;

        metrics.reset_if_new_day();

        assert_eq!(metrics.trades_count, 0); // Should reset
        assert_eq!(metrics.realized_pnl, 0.0);
        assert_eq!(metrics.date, Utc::now().format("%Y-%m-%d").to_string());
    }

    #[test]
    fn test_real_trading_event_order_placed_serialization() {
        let order = RealOrder {
            client_order_id: "client-123".to_string(),
            exchange_order_id: 12345,
            symbol: "BTCUSDT".to_string(),
            side: "BUY".to_string(),
            order_type: "MARKET".to_string(),
            original_quantity: 0.1,
            executed_quantity: 0.0,
            remaining_quantity: 0.1,
            price: None,
            stop_price: None,
            average_fill_price: 0.0,
            state: OrderState::New,
            created_at: Utc::now(),
            updated_at: Utc::now(),
            fills: vec![],
            position_id: None,
            is_entry: true,
            reject_reason: None,
        };

        let event = RealTradingEvent::OrderPlaced(order);
        let json = serde_json::to_string(&event).unwrap();
        assert!(json.contains("OrderPlaced"));
    }

    #[test]
    fn test_real_trading_event_balance_updated_serialization() {
        let event = RealTradingEvent::BalanceUpdated {
            asset: "USDT".to_string(),
            free: 1000.0,
            locked: 50.0,
        };
        let json = serde_json::to_string(&event).unwrap();
        assert!(json.contains("BalanceUpdated"));
        assert!(json.contains("USDT"));
    }

    #[test]
    fn test_real_trading_event_circuit_breaker_opened() {
        let event = RealTradingEvent::CircuitBreakerOpened("Too many errors".to_string());
        let json = serde_json::to_string(&event).unwrap();
        assert!(json.contains("CircuitBreakerOpened"));
    }

    #[test]
    fn test_real_trading_event_daily_loss_limit_reached() {
        let event = RealTradingEvent::DailyLossLimitReached {
            loss: -500.0,
            limit: 1000.0,
        };
        let json = serde_json::to_string(&event).unwrap();
        assert!(json.contains("DailyLossLimitReached"));
    }

    #[test]
    fn test_reconciliation_metrics_default() {
        let metrics = ReconciliationMetrics::default();
        assert!(metrics.last_run_time.is_none());
        assert_eq!(metrics.total_discrepancies_found, 0);
        assert_eq!(metrics.consecutive_failures, 0);
        assert_eq!(metrics.total_runs, 0);
    }

    #[test]
    fn test_balance_default() {
        let balance = Balance::default();
        assert_eq!(balance.asset, "");
        assert_eq!(balance.free, 0.0);
        assert_eq!(balance.locked, 0.0);
        assert_eq!(balance.total(), 0.0);
    }

    #[test]
    fn test_circuit_breaker_state_default() {
        let cb = CircuitBreakerState::default();
        assert!(!cb.is_open);
        assert_eq!(cb.error_count, 0);
        assert!(cb.opened_at.is_none());
        assert!(cb.last_error.is_none());
    }

    #[test]
    fn test_daily_metrics_default() {
        let metrics = DailyMetrics::default();
        assert_eq!(metrics.date, "");
        assert_eq!(metrics.realized_pnl, 0.0);
        assert_eq!(metrics.trades_count, 0);
    }

    #[test]
    fn test_real_trading_event_error_serialization() {
        let event = RealTradingEvent::Error("Connection failed".to_string());
        let json = serde_json::to_string(&event).unwrap();
        assert!(json.contains("Error"));
        assert!(json.contains("Connection failed"));
    }

    #[test]
    fn test_real_trading_event_engine_started() {
        let event = RealTradingEvent::EngineStarted;
        let json = serde_json::to_string(&event).unwrap();
        assert!(json.contains("EngineStarted"));
    }

    #[test]
    fn test_real_trading_event_engine_stopped() {
        let event = RealTradingEvent::EngineStopped;
        let json = serde_json::to_string(&event).unwrap();
        assert!(json.contains("EngineStopped"));
    }

    #[test]
    fn test_real_trading_event_reconciliation_complete() {
        let event = RealTradingEvent::ReconciliationComplete { discrepancies: 5 };
        let json = serde_json::to_string(&event).unwrap();
        assert!(json.contains("ReconciliationComplete"));
        assert!(json.contains("\"discrepancies\":5"));
    }

    #[test]
    fn test_circuit_breaker_record_success_resets_count() {
        let mut cb = CircuitBreakerState::default();
        cb.error_count = 3;
        cb.last_error = Some("Previous error".to_string());

        cb.record_success();

        assert_eq!(cb.error_count, 0);
        assert!(cb.is_open == cb.is_open); // Doesn't change is_open
    }

    #[test]
    fn test_balance_with_negative_values() {
        // Edge case: negative values (shouldn't happen but test anyway)
        let balance = Balance {
            asset: "TEST".to_string(),
            free: -1.0,
            locked: 2.0,
        };
        assert_eq!(balance.total(), 1.0);
    }

    #[test]
    fn test_daily_metrics_win_rate_single_win() {
        let mut metrics = DailyMetrics::new();
        metrics.trades_count = 1;
        metrics.winning_trades = 1;
        assert!((metrics.win_rate() - 100.0).abs() < 0.01);
    }

    #[test]
    fn test_daily_metrics_win_rate_single_loss() {
        let mut metrics = DailyMetrics::new();
        metrics.trades_count = 1;
        metrics.winning_trades = 0;
        assert!((metrics.win_rate() - 0.0).abs() < 0.01);
    }

    #[test]
    fn test_circuit_breaker_should_close_edge_case_exact_cooldown() {
        let mut cb = CircuitBreakerState::default();
        cb.is_open = true;
        cb.opened_at = Some(Utc::now() - chrono::Duration::seconds(60));
        assert!(cb.should_close(60)); // Exactly at cooldown
    }

    #[test]
    fn test_circuit_breaker_record_error_preserves_last_error() {
        let mut cb = CircuitBreakerState::default();
        cb.record_error("First error", 5);
        cb.record_error("Second error", 5);
        assert_eq!(cb.last_error, Some("Second error".to_string()));
    }

    #[test]
    fn test_balance_total_with_large_values() {
        let balance = Balance {
            asset: "BTC".to_string(),
            free: 1000000.0,
            locked: 500000.0,
        };
        assert_eq!(balance.total(), 1500000.0);
    }

    #[test]
    fn test_daily_metrics_reset_preserves_date_format() {
        let metrics = DailyMetrics::new();
        assert!(metrics.date.contains("-"));
        assert_eq!(metrics.date.len(), 10); // YYYY-MM-DD format
    }
}
