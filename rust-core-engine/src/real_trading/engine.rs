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
    BalanceUpdate, ExecutionReport, OrderSide, OutboundAccountPosition, SpotOrderRequest,
    SpotOrderResponse, SpotOrderType, TimeInForce,
};
use crate::binance::user_data_stream::{UserDataStreamEvent, UserDataStreamManager};
use crate::binance::BinanceClient;
use crate::config::TradingMode;
use crate::trading::risk_manager::RiskManager;

use super::config::RealTradingConfig;
use super::order::{OrderState, RealOrder};
use super::position::{PositionSide, RealPosition};
use super::risk::RealTradingRiskManager;

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

        // Create UserDataStreamManager
        let user_data_stream = UserDataStreamManager::new(binance_client.clone());

        // Create real trading risk manager
        let real_risk_manager = RealTradingRiskManager::new(config.clone());

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
        })
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

        // 2. Subscribe to events and spawn handler
        let event_rx = {
            let stream = self.user_data_stream.read().await;
            stream.subscribe()
        };

        // Clone self for the spawned task
        let engine = self.clone();
        tokio::spawn(async move {
            engine.process_user_data_events(event_rx).await;
        });
        info!("UserDataStream event handler spawned");

        // 3. Load initial balances and perform initial sync
        self.initial_sync().await?;

        // 4. Set running flag
        {
            let mut is_running = self.is_running.write().await;
            *is_running = true;
        }

        // 5. Spawn reconciliation loop
        let engine_for_reconciliation = self.clone();
        tokio::spawn(async move {
            engine_for_reconciliation.reconciliation_loop().await;
        });
        info!("Reconciliation loop spawned");

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
        self.check_risk_limits(symbol, side, quantity, price)
            .await?;

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
            side_str,
            order_type_str,
            quantity,
            price,
            stop_price,
            position_id,
            is_entry,
        );

        // Store order in pending state
        self.orders.insert(client_order_id.clone(), order.clone());

        // Build request
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

        // Submit to exchange
        match self.binance_client.place_spot_order(request).await {
            Ok(response) => {
                // Update order with exchange response
                self.update_order_from_response(&client_order_id, &response)
                    .await;

                let updated_order = self.orders.get(&client_order_id).map(|o| o.clone());

                if let Some(order) = updated_order {
                    info!(
                        "Order placed: {} {} {} @ {:?}",
                        order.side, order.symbol, order.original_quantity, order.price
                    );

                    // Record success
                    self.circuit_breaker.write().await.record_success();

                    self.emit_event(RealTradingEvent::OrderPlaced(order.clone()));
                    Ok(order)
                } else {
                    Err(anyhow!("Order not found after placement"))
                }
            },
            Err(e) => {
                // Record error
                let error_msg = format!("Order placement failed: {}", e);
                let mut cb = self.circuit_breaker.write().await;
                let config = self.config.read().await;

                if cb.record_error(&error_msg, config.circuit_breaker_errors) {
                    error!("Circuit breaker opened after order error");
                    self.emit_event(RealTradingEvent::CircuitBreakerOpened(error_msg.clone()));
                }

                // Update order to rejected
                if let Some(mut order) = self.orders.get_mut(&client_order_id) {
                    order.state = OrderState::Rejected;
                    order.reject_reason = Some(e.to_string());
                }

                Err(anyhow!(error_msg))
            },
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

        match self
            .binance_client
            .cancel_spot_order(
                &order.symbol,
                Some(order.exchange_order_id),
                Some(client_order_id),
            )
            .await
        {
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
                let position = RealPosition::new(
                    format!("pos_{}", Uuid::new_v4()),
                    symbol.clone(),
                    side,
                    order.executed_quantity,
                    order.average_fill_price,
                    order.client_order_id.clone(),
                    None,
                    None,
                );
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

    /// Refresh balances from exchange
    pub async fn refresh_balances(&self) -> Result<()> {
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

                debug!("Refreshed {} balances", balances.len());
                Ok(())
            },
            Err(e) => {
                error!("Failed to refresh balances: {}", e);
                Err(anyhow!("Balance refresh failed: {}", e))
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

        // 4. Clean up old terminal orders (sync function, no await)
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
        let account = self.binance_client.get_account_info().await?;
        let mut discrepancies = 0;

        let mut local_balances = self.balances.write().await;

        for balance in account.balances {
            let exchange_free: f64 = balance.free.parse().unwrap_or(0.0);
            let exchange_locked: f64 = balance.locked.parse().unwrap_or(0.0);

            // Skip zero balances
            if exchange_free <= 0.0 && exchange_locked <= 0.0 {
                continue;
            }

            let local_balance = local_balances.get(&balance.asset);
            let local_free = local_balance.map(|b| b.free).unwrap_or(0.0);

            // Check for significant difference (>0.01% or absolute difference > 0.0001)
            let diff = (exchange_free - local_free).abs();
            let threshold = (exchange_free * 0.0001).max(0.0001);

            if diff > threshold {
                warn!(
                    "Balance mismatch for {}: local={:.8}, exchange={:.8} (diff={:.8})",
                    balance.asset, local_free, exchange_free, diff
                );

                // Update local balance
                local_balances.insert(
                    balance.asset.clone(),
                    Balance {
                        asset: balance.asset.clone(),
                        free: exchange_free,
                        locked: exchange_locked,
                    },
                );

                discrepancies += 1;

                // Emit event
                let _ = self.event_tx.send(RealTradingEvent::BalanceUpdated {
                    asset: balance.asset,
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

    /// Get current configuration
    pub async fn get_config(&self) -> RealTradingConfig {
        self.config.read().await.clone()
    }

    /// Update configuration
    pub async fn update_config(&self, config: RealTradingConfig) -> Result<()> {
        config
            .validate()
            .map_err(|e| anyhow!("Invalid config: {}", e.join(", ")))?;
        *self.config.write().await = config;
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
