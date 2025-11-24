use anyhow::Result;
use chrono::{DateTime, Utc};
use rand::Rng;
use serde_json;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::{broadcast, Mutex, RwLock};
use tokio::time::{interval, Duration};
use tracing::{debug, error, info, warn};
use uuid::Uuid;

use crate::ai::AIService;
use crate::binance::BinanceClient;
use crate::storage::Storage;
use crate::strategies::TradingSignal;

use super::{
    // @spec:FR-TRADING-015 - Paper Trading Engine
    // @ref:specs/02-design/2.5-components/COMP-RUST-TRADING.md#paper-trading
    // @test:TC-INTEGRATION-025, TC-INTEGRATION-026
    portfolio::PaperPortfolio,
    settings::PaperTradingSettings,
    strategy_optimizer::StrategyOptimizer,
    trade::{CloseReason, PaperTrade, TradeType},
    AITradingSignal,
    PaperTradingEvent,
    PerformanceSummary,
    TradeExecutionResult,
};

/// Main paper trading engine
#[derive(Clone)]
pub struct PaperTradingEngine {
    /// Paper trading portfolio
    portfolio: Arc<RwLock<PaperPortfolio>>,

    /// Configuration settings
    settings: Arc<RwLock<PaperTradingSettings>>,

    /// Strategy optimizer
    optimizer: Arc<RwLock<StrategyOptimizer>>,

    /// Binance client for real market data
    binance_client: BinanceClient,

    /// AI service for signals
    ai_service: AIService,

    /// Storage for persistence
    storage: Storage,

    /// WebSocket broadcaster for real-time updates
    event_broadcaster: broadcast::Sender<PaperTradingEvent>,

    /// Current market prices
    current_prices: Arc<RwLock<HashMap<String, f64>>>,

    /// Engine state
    is_running: Arc<RwLock<bool>>,

    /// Trade execution queue
    execution_queue: Arc<RwLock<Vec<PendingTrade>>>,

    /// Mutex to prevent concurrent trade execution (race condition fix)
    trade_execution_lock: Arc<Mutex<()>>,

    /// Historical kline data cache for warmup period validation
    /// Key: symbol (e.g., "BTCUSDT"), Value: Vec of recent klines
    /// Pre-loaded at startup to enable immediate trading without waiting
    historical_data_cache: Arc<RwLock<HashMap<String, Vec<crate::binance::types::Kline>>>>,
}

/// Pending trade for execution
#[derive(Debug, Clone)]
pub struct PendingTrade {
    pub signal: AITradingSignal,
    pub calculated_quantity: f64,
    pub calculated_leverage: u8,
    pub stop_loss: f64,
    pub take_profit: f64,
    pub timestamp: DateTime<Utc>,
}

impl PaperTradingEngine {
    /// Create a new paper trading engine
    pub async fn new(
        default_settings: PaperTradingSettings,
        binance_client: BinanceClient,
        ai_service: AIService,
        storage: Storage,
        event_broadcaster: broadcast::Sender<PaperTradingEvent>,
    ) -> Result<Self> {
        // Try to load saved settings from database, fallback to defaults
        let settings = match storage.load_paper_trading_settings().await {
            Ok(Some(saved_settings)) => {
                info!("✅ Loaded saved paper trading settings from database");
                saved_settings
            },
            Ok(None) => {
                info!("📝 No saved settings found, using defaults");
                default_settings
            },
            Err(e) => {
                warn!(
                    "⚠️ Failed to load settings from database, using defaults: {}",
                    e
                );
                default_settings
            },
        };

        let portfolio = Arc::new(RwLock::new(PaperPortfolio::new(
            settings.basic.initial_balance,
        )));
        let optimizer = Arc::new(RwLock::new(StrategyOptimizer::new(
            super::strategy_optimizer::OptimizationConfig::default(),
        )));

        Ok(Self {
            portfolio,
            settings: Arc::new(RwLock::new(settings)),
            optimizer,
            binance_client,
            ai_service,
            storage,
            event_broadcaster,
            current_prices: Arc::new(RwLock::new(HashMap::new())),
            is_running: Arc::new(RwLock::new(false)),
            execution_queue: Arc::new(RwLock::new(Vec::new())),
            trade_execution_lock: Arc::new(Mutex::new(())),
            historical_data_cache: Arc::new(RwLock::new(HashMap::new())),
        })
    }

    /// Start the paper trading engine
    pub async fn start(&self) -> Result<()> {
        {
            let mut running = self.is_running.write().await;
            if *running {
                return Err(anyhow::anyhow!("Paper trading engine is already running"));
            }
            *running = true;
        }

        info!("Starting Paper Trading Engine");

        // Load portfolio from storage if exists
        if let Err(e) = self.load_portfolio_from_storage().await {
            warn!("Failed to load portfolio from storage: {}", e);
        }

        // Pre-load historical data for instant warmup (no more 12.5 hour wait!)
        info!("📊 Pre-loading historical data for all symbols...");
        if let Err(e) = self.preload_historical_data().await {
            warn!(
                "⚠️ Failed to preload historical data: {}. Warmup will use API queries instead.",
                e
            );
        }

        // Start background tasks
        let price_update_handle = self.start_price_updates();
        let signal_processing_handle = self.start_signal_processing();
        let trade_monitoring_handle = self.start_trade_monitoring();
        let performance_tracking_handle = self.start_performance_tracking();
        let optimization_handle = self.start_optimization_loop();
        let daily_metrics_handle = self.start_daily_metrics_save();

        // Broadcast start event
        let _ = self.event_broadcaster.send(PaperTradingEvent {
            event_type: "engine_started".to_string(),
            data: serde_json::json!({ "timestamp": Utc::now() }),
            timestamp: Utc::now(),
        });

        info!("Paper Trading Engine started successfully");

        // Wait for all background tasks
        let (
            _price_result,
            _signal_result,
            _trade_result,
            _perf_result,
            _opt_result,
            _metrics_result,
        ) = tokio::try_join!(
            price_update_handle,
            signal_processing_handle,
            trade_monitoring_handle,
            performance_tracking_handle,
            optimization_handle,
            daily_metrics_handle,
        )?;

        Ok(())
    }

    /// Stop the paper trading engine
    pub async fn stop(&self) -> Result<()> {
        let mut running = self.is_running.write().await;
        *running = false;

        // Save portfolio to storage
        self.save_portfolio_to_storage().await?;

        // Broadcast stop event
        let _ = self.event_broadcaster.send(PaperTradingEvent {
            event_type: "engine_stopped".to_string(),
            data: serde_json::json!({ "timestamp": Utc::now() }),
            timestamp: Utc::now(),
        });

        info!("Paper Trading Engine stopped");
        Ok(())
    }

    /// Start price update loop
    fn start_price_updates(&self) -> tokio::task::JoinHandle<Result<()>> {
        let engine = self.clone();

        tokio::spawn(async move {
            let mut interval = interval(Duration::from_secs(1)); // Update every second

            while *engine.is_running.read().await {
                interval.tick().await;

                if let Err(e) = engine.update_market_prices().await {
                    error!("Failed to update market prices: {}", e);
                }
            }

            Ok(())
        })
    }

    /// Start AI signal processing loop
    fn start_signal_processing(&self) -> tokio::task::JoinHandle<Result<()>> {
        let engine = self.clone();

        tokio::spawn(async move {
            let settings = engine.settings.read().await;
            let signal_interval = settings.ai.signal_refresh_interval_minutes;
            drop(settings);

            let mut interval = interval(Duration::from_secs(signal_interval as u64 * 60));

            while *engine.is_running.read().await {
                interval.tick().await;

                if let Err(e) = engine.process_ai_signals().await {
                    error!("Failed to process AI signals: {}", e);
                }
            }

            Ok(())
        })
    }

    /// Start trade monitoring loop
    fn start_trade_monitoring(&self) -> tokio::task::JoinHandle<Result<()>> {
        let engine = self.clone();

        tokio::spawn(async move {
            let mut interval = interval(Duration::from_secs(5)); // Check every 5 seconds

            while *engine.is_running.read().await {
                interval.tick().await;

                if let Err(e) = engine.monitor_open_trades().await {
                    error!("Failed to monitor open trades: {}", e);
                }

                if let Err(e) = engine.execute_pending_trades().await {
                    error!("Failed to execute pending trades: {}", e);
                }
            }

            Ok(())
        })
    }

    /// Start performance tracking loop
    fn start_performance_tracking(&self) -> tokio::task::JoinHandle<Result<()>> {
        let engine = self.clone();

        tokio::spawn(async move {
            let mut interval = interval(Duration::from_secs(300)); // Every 5 minutes

            while *engine.is_running.read().await {
                interval.tick().await;

                if let Err(e) = engine.update_performance_metrics().await {
                    error!("Failed to update performance metrics: {}", e);
                }
            }

            Ok(())
        })
    }

    /// Start optimization loop
    fn start_optimization_loop(&self) -> tokio::task::JoinHandle<Result<()>> {
        let engine = self.clone();

        tokio::spawn(async move {
            let mut interval = interval(Duration::from_secs(3600)); // Every hour

            while *engine.is_running.read().await {
                interval.tick().await;

                if let Err(e) = engine.run_optimization_analysis().await {
                    error!("Failed to run optimization analysis: {}", e);
                }
            }

            Ok(())
        })
    }

    /// Start daily metrics save loop
    fn start_daily_metrics_save(&self) -> tokio::task::JoinHandle<Result<()>> {
        let engine = self.clone();

        tokio::spawn(async move {
            let mut interval = interval(Duration::from_secs(86400)); // Every 24 hours
            let mut last_equity = 0.0;

            while *engine.is_running.read().await {
                interval.tick().await;

                let portfolio = engine.portfolio.read().await;
                let current_equity = portfolio.equity;
                let daily_pnl = current_equity - last_equity;

                if let Err(e) = engine
                    .storage
                    .save_daily_metrics(&portfolio, daily_pnl)
                    .await
                {
                    error!("Failed to save daily metrics: {}", e);
                } else {
                    info!(
                        "Saved daily metrics: PnL = {:.2}, Total Trades = {}",
                        daily_pnl, portfolio.metrics.total_trades
                    );
                }

                last_equity = current_equity;
            }

            Ok(())
        })
    }

    /// Update market prices from Binance
    async fn update_market_prices(&self) -> Result<()> {
        let settings = self.settings.read().await;
        let symbols: Vec<String> = settings.symbols.keys().cloned().collect();
        drop(settings);

        let mut new_prices = HashMap::new();
        let mut funding_rates = HashMap::new();

        // Get current prices for all symbols
        for symbol in &symbols {
            match self.binance_client.get_symbol_price(symbol).await {
                Ok(price_info) => {
                    let price: f64 = price_info.price.parse().unwrap_or(0.0);
                    new_prices.insert(symbol.clone(), price);
                },
                Err(e) => {
                    warn!("Failed to get price for {}: {}", symbol, e);
                },
            }

            // Get funding rate for futures
            match self.binance_client.get_funding_rate(symbol).await {
                Ok(funding_info) => {
                    if let Ok(rate) = funding_info.funding_rate.parse::<f64>() {
                        funding_rates.insert(symbol.clone(), rate);
                    }
                },
                Err(_) => {
                    // Funding rate not available, use default
                    funding_rates.insert(symbol.clone(), 0.0);
                },
            }
        }

        // Update portfolio with new prices
        {
            let mut portfolio = self.portfolio.write().await;
            portfolio.update_prices(new_prices.clone(), Some(funding_rates));

            // Update trailing stops for open trades if enabled
            // @spec:FR-RISK-007 - Trailing Stop Loss for Long Positions
            // @spec:FR-RISK-008 - Trailing Stop Loss for Short Positions
            // @ref:specs/01-requirements/1.1-functional-requirements/FR-RISK.md#fr-risk-007
            // @ref:specs/01-requirements/1.1-functional-requirements/FR-RISK.md#fr-risk-008
            // @ref:specs/02-design/2.5-components/COMP-RUST-TRADING.md#trailing-stop-component
            // @test:TC-TRADING-015
            let settings = self.settings.read().await;
            if settings.risk.trailing_stop_enabled {
                let trailing_pct = settings.risk.trailing_stop_pct;
                let activation_pct = settings.risk.trailing_activation_pct;

                // Update trailing stops for all open trades
                for trade_id in &portfolio.open_trade_ids.clone() {
                    if let Some(trade) = portfolio.trades.get_mut(trade_id) {
                        if let Some(current_price) = new_prices.get(&trade.symbol) {
                            trade.update_trailing_stop(
                                *current_price,
                                trailing_pct,
                                activation_pct,
                            );
                        }
                    }
                }
            }
        }

        // Update cached prices
        {
            let mut prices = self.current_prices.write().await;
            prices.extend(new_prices.clone());
        }

        // Log price updates for monitoring
        debug!(
            "💰 Market prices updated: BTCUSDT=${:.2}, ETHUSDT=${:.2}, BNBUSDT=${:.2}, SOLUSDT=${:.2}",
            new_prices.get("BTCUSDT").unwrap_or(&0.0),
            new_prices.get("ETHUSDT").unwrap_or(&0.0),
            new_prices.get("BNBUSDT").unwrap_or(&0.0),
            new_prices.get("SOLUSDT").unwrap_or(&0.0)
        );

        // Broadcast price update
        let _ = self.event_broadcaster.send(PaperTradingEvent {
            event_type: "price_update".to_string(),
            data: serde_json::to_value(&new_prices)?,
            timestamp: Utc::now(),
        });

        Ok(())
    }

    /// Process AI signals and generate trade decisions
    async fn process_ai_signals(&self) -> Result<()> {
        let settings = self.settings.read().await;
        let symbols: Vec<String> = settings.symbols.keys().cloned().collect();
        let min_confidence = settings.strategy.min_ai_confidence;
        drop(settings);

        for symbol in symbols {
            match self.get_ai_signal_for_symbol(&symbol).await {
                Ok(signal) => {
                    // Save AI signal to database
                    let executed = signal.confidence >= min_confidence;
                    let trade_id = if executed {
                        // This will be set after trade execution
                        None
                    } else {
                        None
                    };

                    if let Err(e) = self
                        .storage
                        .save_ai_signal(&signal, executed, trade_id)
                        .await
                    {
                        error!("Failed to save AI signal to database: {}", e);
                    }

                    // Broadcast AI signal via WebSocket regardless of confidence
                    let _ = self.event_broadcaster.send(PaperTradingEvent {
                        event_type: "AISignalReceived".to_string(),
                        data: serde_json::json!({
                            "symbol": signal.symbol,
                            "signal": format!("{:?}", signal.signal_type).to_lowercase(),
                            "confidence": signal.confidence,
                            "timestamp": signal.timestamp,
                            "reasoning": signal.reasoning,
                            "entry_price": signal.entry_price,
                            "trend_direction": signal.market_analysis.trend_direction
                        }),
                        timestamp: Utc::now(),
                    });

                    if signal.confidence >= min_confidence {
                        match self.process_trading_signal(signal.clone()).await {
                            Ok(result) => {
                                if result.success {
                                    // Update AI signal record with trade ID
                                    if let Some(trade_id) = result.trade_id {
                                        // Update the signal record to mark as executed with trade ID
                                        info!(
                                            "Trade executed for signal {}: {}",
                                            signal.id, trade_id
                                        );
                                    }
                                }
                            },
                            Err(e) => {
                                error!("Failed to process trading signal for {}: {}", symbol, e);
                            },
                        }
                    } else {
                        debug!(
                            "Signal confidence {} below threshold {} for {}",
                            signal.confidence, min_confidence, symbol
                        );
                    }
                },
                Err(e) => {
                    // In paper trading mode, this is expected - we skip automatic signal generation
                    // and wait for signals from the frontend via API/WebSocket
                    debug!(
                        "Skipped automatic AI signal for {} (expected in paper trading): {}",
                        symbol, e
                    );
                },
            }
        }

        Ok(())
    }

    /// Get AI signal for a specific symbol
    ///
    /// Note: In paper trading mode, this method is intentionally disabled to avoid calling Binance API.
    /// Paper trading should rely on AI signals generated by the frontend and sent via API/WebSocket.
    /// The frontend generates mock data and calls the AI service, then sends signals to this engine.
    async fn get_ai_signal_for_symbol(&self, symbol: &str) -> Result<AITradingSignal> {
        // @spec:FR-TRADING-015 - Paper Trading Simulation
        // Paper trading should NOT call Binance API (even testnet).
        // It should only react to signals from the frontend.

        debug!(
            "Skipping automated AI signal generation for {} in paper trading mode. \
            Paper trading waits for frontend signals via API/WebSocket.",
            symbol
        );

        // Return error to skip this symbol in the automatic processing loop.
        // This is expected behavior - the loop will continue to the next symbol.
        Err(anyhow::anyhow!(
            "Paper trading mode: skipping automatic signal generation for {}. \
            Use frontend AI analysis or manual API calls instead.",
            symbol
        ))
    }

    /// Process a trading signal and potentially execute a trade
    async fn process_trading_signal(
        &self,
        signal: AITradingSignal,
    ) -> Result<TradeExecutionResult> {
        // 🔒 CRITICAL: Acquire lock to prevent race condition (duplicate orders)
        // This ensures only ONE signal can be processed at a time
        let _lock = self.trade_execution_lock.lock().await;

        info!("🔒 Acquired trade execution lock for {}", signal.symbol);

        // ========== PHASE 1: WARMUP PERIOD CHECK ==========
        // Ensure sufficient historical data for accurate indicator calculations

        let settings = self.settings.read().await;
        let timeframe = &settings.strategy.backtesting.data_resolution; // Use configured timeframe (default: 15m)
        let timeframe_str = timeframe.clone();
        drop(settings);

        if !self
            .check_warmup_period(&signal.symbol, &timeframe_str)
            .await?
        {
            return Ok(TradeExecutionResult {
                success: false,
                trade_id: None,
                error_message: Some(format!(
                    "Warmup period incomplete - insufficient historical data for {} on {} timeframe. \
                    Need 50 candles (12.5 hours for 15m). Please wait for more data accumulation.",
                    signal.symbol, timeframe_str
                )),
                execution_price: None,
                fees_paid: None,
            });
        }

        // ========== PHASE 2: RISK MANAGEMENT CHECKS ==========

        // 1. Check daily loss limit
        if !self.check_daily_loss_limit().await? {
            return Ok(TradeExecutionResult {
                success: false,
                trade_id: None,
                error_message: Some("Daily loss limit reached - trading disabled".to_string()),
                execution_price: None,
                fees_paid: None,
            });
        }

        // 2. Check cool-down period
        if self.is_in_cooldown().await {
            return Ok(TradeExecutionResult {
                success: false,
                trade_id: None,
                error_message: Some("In cool-down period after consecutive losses".to_string()),
                execution_price: None,
                fees_paid: None,
            });
        }

        // 3. Check position correlation limits
        let trade_type = match signal.signal_type {
            crate::strategies::TradingSignal::Long => TradeType::Long,
            crate::strategies::TradingSignal::Short => TradeType::Short,
            _ => {
                return Ok(TradeExecutionResult {
                    success: false,
                    trade_id: None,
                    error_message: Some("Neutral signal cannot be executed".to_string()),
                    execution_price: None,
                    fees_paid: None,
                })
            },
        };

        if !self.check_position_correlation(trade_type).await? {
            return Ok(TradeExecutionResult {
                success: false,
                trade_id: None,
                error_message: Some("Position correlation limit exceeded".to_string()),
                execution_price: None,
                fees_paid: None,
            });
        }

        // ========== EXISTING CHECKS ==========

        // Check if we can trade this symbol
        let settings = self.settings.read().await;
        let symbol_settings = settings.get_symbol_settings(&signal.symbol);

        if !symbol_settings.enabled {
            return Ok(TradeExecutionResult {
                success: false,
                trade_id: None,
                error_message: Some("Symbol trading disabled".to_string()),
                execution_price: None,
                fees_paid: None,
            });
        }

        // Check if we already have a position for this symbol
        let portfolio = self.portfolio.read().await;
        let existing_positions = portfolio
            .get_open_trades()
            .iter()
            .filter(|trade| trade.symbol == signal.symbol)
            .count();

        if existing_positions >= symbol_settings.max_positions as usize {
            debug!("Maximum positions reached for {}", signal.symbol);
            return Ok(TradeExecutionResult {
                success: false,
                trade_id: None,
                error_message: Some("Maximum positions reached".to_string()),
                execution_price: None,
                fees_paid: None,
            });
        }

        // Calculate position parameters
        let leverage = symbol_settings.leverage;

        // Get REAL current price from Binance instead of using signal.entry_price
        let entry_price = self
            .current_prices
            .read()
            .await
            .get(&signal.symbol)
            .copied()
            .unwrap_or_else(|| {
                warn!(
                    "No current price for {}, using signal price as fallback",
                    signal.symbol
                );
                signal.entry_price
            });

        // @spec:FR-RISK-002 - Fixed Percentage Stop Loss (SIMPLIFIED)
        // REMOVED ATR: Always use fixed percentage from settings for predictability
        // ATR was causing 46%+ stop loss instead of 5% for volatile assets like BTC
        // Fixed percentage ensures 100% respect for user settings
        let stop_loss = signal
            .suggested_stop_loss
            .unwrap_or_else(|| match signal.signal_type {
                crate::strategies::TradingSignal::Long => {
                    entry_price * (1.0 - symbol_settings.stop_loss_pct / 100.0)
                },
                crate::strategies::TradingSignal::Short => {
                    entry_price * (1.0 + symbol_settings.stop_loss_pct / 100.0)
                },
                _ => entry_price,
            });

        let take_profit = signal.suggested_take_profit.unwrap_or_else(|| {
            match signal.signal_type {
                crate::strategies::TradingSignal::Long => {
                    entry_price * (1.0 + symbol_settings.take_profit_pct / 100.0)
                },
                crate::strategies::TradingSignal::Short => {
                    entry_price * (1.0 - symbol_settings.take_profit_pct / 100.0)
                },
                _ => entry_price, // Neutral signal
            }
        });

        // Calculate position size with PROPER risk-based formula
        // @spec:FR-RISK-001 - Position Size Calculation (FIXED)
        let risk_amount = portfolio.equity * (symbol_settings.position_size_pct / 100.0);

        // Calculate stop loss percentage
        let stop_loss_pct = ((entry_price - stop_loss).abs() / entry_price) * 100.0;

        // Calculate max position value based on risk
        let max_position_value = if stop_loss_pct > 0.0 {
            risk_amount / (stop_loss_pct / 100.0)
        } else {
            risk_amount * 10.0 // Default to 10% SL if none set
        };

        // Apply leverage to position value
        let max_position_value_with_leverage = max_position_value * leverage as f64;

        // Limit by available margin (keep 5% buffer)
        let available_for_position = portfolio.free_margin * 0.95;
        let actual_position_value = max_position_value_with_leverage.min(available_for_position);

        // Calculate quantity
        let max_quantity = actual_position_value / entry_price;

        // Additional safety: limit to max 20% of account per trade
        let safety_limit = portfolio.equity * 0.2 / entry_price;
        let quantity = max_quantity.min(safety_limit);

        drop(portfolio);
        drop(settings);

        if quantity <= 0.0 {
            debug!("Insufficient margin for trade on {}", signal.symbol);
            return Ok(TradeExecutionResult {
                success: false,
                trade_id: None,
                error_message: Some("Insufficient margin".to_string()),
                execution_price: None,
                fees_paid: None,
            });
        }

        // Create pending trade
        let pending_trade = PendingTrade {
            signal: signal.clone(),
            calculated_quantity: quantity,
            calculated_leverage: leverage,
            stop_loss,
            take_profit,
            timestamp: Utc::now(),
        };

        // Add to execution queue
        {
            let mut queue = self.execution_queue.write().await;
            queue.push(pending_trade);
        }

        // Broadcast signal event
        let _ = self.event_broadcaster.send(PaperTradingEvent {
            event_type: "ai_signal_received".to_string(),
            data: serde_json::to_value(&signal)?,
            timestamp: Utc::now(),
        });

        // Execute the trade
        self.execute_pending_trades().await?;

        Ok(TradeExecutionResult {
            success: true,
            trade_id: None, // Will be set by execute_trade
            error_message: None,
            execution_price: Some(signal.entry_price),
            fees_paid: None,
        })
    }

    /// Execute pending trades
    async fn execute_pending_trades(&self) -> Result<()> {
        let mut queue = self.execution_queue.write().await;
        let pending_trades = queue.drain(..).collect::<Vec<_>>();
        drop(queue);

        for pending_trade in pending_trades {
            if let Err(e) = self.execute_trade(pending_trade).await {
                error!("Failed to execute trade: {}", e);
            }
        }

        Ok(())
    }

    /// Apply slippage to execution price
    /// Simulates real market conditions where orders don't execute at exact prices
    /// @doc:docs/features/paper-trading.md#execution-simulation
    /// @spec:FR-TRADING-015 - Execution Realism
    async fn apply_slippage(&self, price: f64, trade_type: TradeType) -> f64 {
        let settings = self.settings.read().await;

        if !settings.execution.simulate_slippage {
            return price;
        }

        let max_slippage = settings.execution.max_slippage_pct;
        drop(settings);

        // Random slippage between 0 and max_slippage_pct
        let mut rng = rand::thread_rng();
        let slippage_pct = rng.gen::<f64>() * max_slippage;

        let slipped_price = match trade_type {
            TradeType::Long => price * (1.0 + slippage_pct / 100.0), // Buy at higher price
            TradeType::Short => price * (1.0 - slippage_pct / 100.0), // Sell at lower price
        };

        debug!(
            "💸 Slippage applied: {} -> {} ({:.4}% {} slippage)",
            price,
            slipped_price,
            slippage_pct,
            match trade_type {
                TradeType::Long => "positive",
                TradeType::Short => "negative",
            }
        );

        slipped_price
    }

    /// Calculate market impact based on order size
    /// Large orders move the market and get worse execution prices
    /// @doc:docs/features/paper-trading.md#execution-simulation
    /// @spec:FR-TRADING-015 - Execution Realism
    async fn calculate_market_impact(&self, symbol: &str, quantity: f64, price: f64) -> f64 {
        let settings = self.settings.read().await;

        if !settings.execution.simulate_market_impact {
            return 0.0;
        }

        let impact_factor = settings.execution.market_impact_factor;
        drop(settings);

        // Typical 1-hour volumes for major pairs (in USD)
        let typical_volumes: HashMap<&str, f64> = [
            ("BTCUSDT", 50_000_000.0),
            ("ETHUSDT", 20_000_000.0),
            ("BNBUSDT", 10_000_000.0),
            ("SOLUSDT", 5_000_000.0),
        ]
        .iter()
        .cloned()
        .collect();

        let order_value = quantity * price;
        let typical_volume = typical_volumes.get(symbol).unwrap_or(&10_000_000.0);

        // Market impact = (order_value / typical_volume) * impact_factor
        // Capped at 1% maximum
        let impact_pct = ((order_value / typical_volume) * impact_factor).min(1.0);

        if impact_pct > 0.001 {
            debug!(
                "📊 Market impact for {} order of ${:.2}: {:.4}%",
                symbol,
                order_value,
                impact_pct * 100.0
            );
        }

        impact_pct
    }

    /// Simulate partial fills
    /// Real Binance orders sometimes fill only partially, especially in volatile markets
    /// @doc:docs/features/paper-trading.md#execution-simulation
    /// @spec:FR-TRADING-015 - Execution Realism
    async fn simulate_partial_fill(&self, quantity: f64) -> (f64, bool) {
        let settings = self.settings.read().await;

        if !settings.execution.simulate_partial_fills {
            return (quantity, false); // Full fill
        }

        let partial_prob = settings.execution.partial_fill_probability;
        drop(settings);

        let mut rng = rand::thread_rng();

        if rng.gen::<f64>() < partial_prob {
            // Partial fill: 30-90% of requested quantity
            let fill_pct = 0.3 + (rng.gen::<f64>() * 0.6);
            let filled_qty = quantity * fill_pct;

            warn!(
                "⚠️ Partial fill: requested {:.6}, filled {:.6} ({:.1}%)",
                quantity,
                filled_qty,
                fill_pct * 100.0
            );

            (filled_qty, true) // Partial fill occurred
        } else {
            (quantity, false) // Full fill
        }
    }

    /// Check if enough historical data is available for trading (warmup period)
    /// Indicators like RSI, MACD, Bollinger Bands need sufficient candles to calculate accurately
    /// For 15m timeframe: 50 candles = 12.5 hours of data required
    /// @doc:docs/features/paper-trading.md#warmup-period
    /// @spec:FR-TRADING-015 - Warmup Period Check
    async fn check_warmup_period(&self, symbol: &str, timeframe: &str) -> Result<bool> {
        // Minimum candles required for indicators:
        // - RSI (14): 15 candles minimum
        // - MACD (26,12,9): 35 candles minimum
        // - Bollinger Bands (20): 20 candles minimum
        // - Stochastic (14,3): 17 candles minimum
        // Safe minimum: 50 candles for all strategies
        const MIN_CANDLES_REQUIRED: usize = 50;

        // STEP 1: Check cache first (instant, no API call needed!)
        {
            let cache = self.historical_data_cache.read().await;
            if let Some(klines) = cache.get(symbol) {
                let candle_count = klines.len();

                if candle_count >= MIN_CANDLES_REQUIRED {
                    // Cache has sufficient data - instant warmup complete!
                    debug!(
                        "✅ Warmup complete (cached): {} has {} candles for {} timeframe",
                        symbol, candle_count, timeframe
                    );
                    return Ok(true);
                } else {
                    warn!(
                        "⏳ Warmup pending (cached): {} only has {}/{} candles",
                        symbol, candle_count, MIN_CANDLES_REQUIRED
                    );
                    return Ok(false);
                }
            }
        }

        // STEP 2: Cache miss - query Binance API (fallback)
        debug!("📡 Cache miss for {}, querying Binance API...", symbol);
        match self
            .binance_client
            .get_klines(symbol, timeframe, Some(MIN_CANDLES_REQUIRED as u16))
            .await
        {
            Ok(klines) => {
                let candle_count = klines.len();

                // Update cache with fresh data
                {
                    let mut cache = self.historical_data_cache.write().await;
                    cache.insert(symbol.to_string(), klines);
                }

                if candle_count < MIN_CANDLES_REQUIRED {
                    warn!(
                        "⏳ Warmup period: {} only has {}/{} candles for {} timeframe",
                        symbol, candle_count, MIN_CANDLES_REQUIRED, timeframe
                    );
                    return Ok(false);
                }

                info!(
                    "✅ Warmup complete (API): {} has sufficient data ({} candles)",
                    symbol, candle_count
                );
                Ok(true)
            },
            Err(e) => {
                error!("❌ Failed to check warmup status for {}: {}", symbol, e);
                // If we can't verify data availability, assume warmup incomplete (safer approach)
                Ok(false)
            },
        }
    }

    /// Pre-load historical data for all trading symbols at startup
    /// This eliminates the 12.5 hour warmup wait by fetching data immediately
    /// WebSocket will then keep cache updated with real-time data
    /// @doc:docs/features/paper-trading.md#instant-warmup
    async fn preload_historical_data(&self) -> Result<()> {
        let settings = self.settings.read().await;
        let timeframe = settings.strategy.backtesting.data_resolution.clone();

        // Get list of symbols to trade (from settings or default list)
        // TODO: Make this configurable via settings
        let symbols = vec!["BTCUSDT", "ETHUSDT", "BNBUSDT", "SOLUSDT"];
        drop(settings);

        const MIN_CANDLES: u32 = 50;
        let mut total_loaded = 0;
        let mut failed = 0;

        for symbol in &symbols {
            match self
                .binance_client
                .get_klines(symbol, &timeframe, Some(MIN_CANDLES as u16))
                .await
            {
                Ok(klines) => {
                    let count = klines.len();

                    // Store in cache
                    let mut cache = self.historical_data_cache.write().await;
                    cache.insert(symbol.to_string(), klines);
                    drop(cache);

                    total_loaded += count;
                    info!(
                        "   ✅ Pre-loaded {} candles for {} ({})",
                        count, symbol, timeframe
                    );
                },
                Err(e) => {
                    warn!("   ⚠️ Failed to preload data for {}: {}", symbol, e);
                    failed += 1;
                },
            }
        }

        if failed == 0 {
            info!(
                "🎉 Successfully pre-loaded {} candles for {} symbols! Trading ready immediately.",
                total_loaded,
                symbols.len()
            );
        } else {
            warn!(
                "⚠️ Pre-loaded {}/{} symbols successfully ({} failed)",
                symbols.len() - failed,
                symbols.len(),
                failed
            );
        }

        Ok(())
    }

    /// Check daily loss limit
    /// Prevents catastrophic losses by stopping trading if daily loss exceeds limit
    /// @doc:docs/features/paper-trading.md#risk-management
    /// @spec:FR-RISK-001 - Daily Loss Limit
    async fn check_daily_loss_limit(&self) -> Result<bool> {
        let settings = self.settings.read().await;
        let daily_limit_pct = settings.risk.daily_loss_limit_pct;
        drop(settings);

        let portfolio = self.portfolio.read().await;

        // Get today's starting equity (use equity from last daily performance entry)
        let today_start_equity = portfolio
            .daily_performance
            .last()
            .map(|d| d.equity)
            .unwrap_or(portfolio.initial_balance);

        let current_equity = portfolio.equity;
        let daily_loss = today_start_equity - current_equity;
        let daily_loss_pct = (daily_loss / today_start_equity) * 100.0;

        drop(portfolio);

        if daily_loss_pct >= daily_limit_pct {
            error!(
                "🛑 DAILY LOSS LIMIT REACHED: {:.2}% (limit: {:.2}%) - Trading disabled for today",
                daily_loss_pct, daily_limit_pct
            );

            // Broadcast risk event
            let _ = self.event_broadcaster.send(PaperTradingEvent {
                event_type: "daily_loss_limit_reached".to_string(),
                data: serde_json::json!({
                    "daily_loss_pct": daily_loss_pct,
                    "daily_limit_pct": daily_limit_pct,
                    "daily_loss_usd": daily_loss,
                }),
                timestamp: Utc::now(),
            });

            return Ok(false); // Block new trades
        }

        Ok(true) // Allow trading
    }

    /// Check if in cool-down period
    /// After consecutive losses, bot pauses to prevent emotional trading
    async fn is_in_cooldown(&self) -> bool {
        let portfolio = self.portfolio.read().await;

        if let Some(cool_down_until) = portfolio.cool_down_until {
            if Utc::now() < cool_down_until {
                let remaining = (cool_down_until - Utc::now()).num_minutes();
                warn!(
                    "🧊 Cool-down active: {} minutes remaining (consecutive losses: {})",
                    remaining, portfolio.consecutive_losses
                );
                return true;
            }
        }

        false
    }

    /// Update consecutive losses and trigger cool-down if needed
    /// Call this after closing a trade
    async fn update_consecutive_losses(&self, pnl: f64) {
        let mut portfolio = self.portfolio.write().await;
        let settings = self.settings.read().await;

        if pnl < 0.0 {
            portfolio.consecutive_losses += 1;

            info!(
                "📉 Consecutive losses: {} (max: {})",
                portfolio.consecutive_losses, settings.risk.max_consecutive_losses
            );

            if portfolio.consecutive_losses >= settings.risk.max_consecutive_losses {
                let cool_down = settings.risk.cool_down_minutes;
                portfolio.cool_down_until =
                    Some(Utc::now() + chrono::Duration::minutes(cool_down as i64));

                error!(
                    "🛑 {} consecutive losses reached. Cool-down for {} minutes.",
                    portfolio.consecutive_losses, cool_down
                );

                // Broadcast cool-down event
                let _ = self.event_broadcaster.send(PaperTradingEvent {
                    event_type: "cooldown_activated".to_string(),
                    data: serde_json::json!({
                        "consecutive_losses": portfolio.consecutive_losses,
                        "cool_down_minutes": cool_down,
                        "cool_down_until": portfolio.cool_down_until,
                    }),
                    timestamp: Utc::now(),
                });
            }
        } else {
            // Reset on profitable trade
            if portfolio.consecutive_losses > 0 {
                info!(
                    "✅ Profitable trade - resetting consecutive losses counter (was {})",
                    portfolio.consecutive_losses
                );
            }
            portfolio.consecutive_losses = 0;
            portfolio.cool_down_until = None;
        }
    }

    /// Check position correlation limits
    /// Prevents too many positions in the same direction (risk concentration)
    async fn check_position_correlation(&self, new_type: TradeType) -> Result<bool> {
        let settings = self.settings.read().await;
        let correlation_limit = settings.risk.correlation_limit;
        drop(settings);

        let portfolio = self.portfolio.read().await;
        let open_trades = portfolio.get_open_trades();

        if open_trades.is_empty() {
            return Ok(true); // First position always OK
        }

        // Count positions by direction
        let mut long_exposure = 0.0;
        let mut short_exposure = 0.0;

        for trade in open_trades {
            let position_value = trade.quantity * trade.entry_price;
            match trade.trade_type {
                TradeType::Long => long_exposure += position_value,
                TradeType::Short => short_exposure += position_value,
            }
        }

        let total_exposure = long_exposure + short_exposure;

        if total_exposure == 0.0 {
            return Ok(true);
        }

        // Calculate directional exposure ratios
        let long_ratio = long_exposure / total_exposure;
        let short_ratio = short_exposure / total_exposure;

        // Check if new position would exceed correlation limit
        match new_type {
            TradeType::Long if long_ratio > correlation_limit => {
                warn!(
                    "⚠️ Position correlation limit: {:.1}% long exposure exceeds {:.0}% limit",
                    long_ratio * 100.0,
                    correlation_limit * 100.0
                );

                // Broadcast correlation warning
                let _ = self.event_broadcaster.send(PaperTradingEvent {
                    event_type: "correlation_limit_exceeded".to_string(),
                    data: serde_json::json!({
                        "direction": "long",
                        "current_ratio": long_ratio,
                        "limit": correlation_limit,
                    }),
                    timestamp: Utc::now(),
                });

                Ok(false)
            },
            TradeType::Short if short_ratio > correlation_limit => {
                warn!(
                    "⚠️ Position correlation limit: {:.1}% short exposure exceeds {:.0}% limit",
                    short_ratio * 100.0,
                    correlation_limit * 100.0
                );

                // Broadcast correlation warning
                let _ = self.event_broadcaster.send(PaperTradingEvent {
                    event_type: "correlation_limit_exceeded".to_string(),
                    data: serde_json::json!({
                        "direction": "short",
                        "current_ratio": short_ratio,
                        "limit": correlation_limit,
                    }),
                    timestamp: Utc::now(),
                });

                Ok(false)
            },
            _ => Ok(true),
        }
    }

    /// Execute a single trade
    /// Includes full execution simulation (delay, slippage, market impact, partial fills)
    /// and performance tracking (latency metrics)
    /// @doc:docs/features/paper-trading.md#execution-simulation
    /// @spec:FR-TRADING-015 - Execution Realism (Phase 1)
    /// @spec:FR-TRADING-015 - Performance Metrics (Phase 4)
    async fn execute_trade(&self, pending_trade: PendingTrade) -> Result<TradeExecutionResult> {
        let signal = &pending_trade.signal;

        // Determine trade type
        let trade_type = match signal.signal_type {
            crate::strategies::TradingSignal::Long => TradeType::Long,
            crate::strategies::TradingSignal::Short => TradeType::Short,
            _ => {
                return Ok(TradeExecutionResult {
                    success: false,
                    trade_id: None,
                    error_message: Some("Neutral signal cannot be executed".to_string()),
                    execution_price: None,
                    fees_paid: None,
                })
            },
        };

        // ========== PHASE 1: EXECUTION REALISM SIMULATION ==========

        // 1. Simulate execution delay (network latency)
        let settings = self.settings.read().await;
        let execution_delay_ms = settings.execution.execution_delay_ms;
        let trading_fee_rate = settings.basic.trading_fee_rate;
        drop(settings);

        if execution_delay_ms > 0 {
            debug!("⏳ Simulating execution delay: {}ms", execution_delay_ms);
            tokio::time::sleep(Duration::from_millis(execution_delay_ms as u64)).await;
        }

        // 2. Re-fetch current price after delay (price may have moved!)
        let current_price = self
            .current_prices
            .read()
            .await
            .get(&signal.symbol)
            .copied()
            .unwrap_or(signal.entry_price);

        // 3. Calculate market impact based on order size
        let market_impact_pct = self
            .calculate_market_impact(
                &signal.symbol,
                pending_trade.calculated_quantity,
                current_price,
            )
            .await;

        // 4. Apply market impact to price
        let price_with_impact = current_price * (1.0 + market_impact_pct / 100.0);

        // 5. Apply slippage simulation
        let execution_price = self.apply_slippage(price_with_impact, trade_type).await;

        // 6. Simulate partial fills
        let (filled_quantity, _is_partial) = self
            .simulate_partial_fill(pending_trade.calculated_quantity)
            .await;

        info!(
            "🎯 Execution simulation complete for {}: base={:.2}, impact={:.4}%, slippage applied, fill={:.1}%",
            signal.symbol,
            current_price,
            market_impact_pct * 100.0,
            (filled_quantity / pending_trade.calculated_quantity) * 100.0
        );

        // ========== CREATE PAPER TRADE WITH REALISTIC EXECUTION ==========

        // Create paper trade with realistic execution price
        let mut paper_trade = PaperTrade::new(
            signal.symbol.clone(),
            trade_type,
            execution_price, // Use realistic execution price (not signal price!)
            filled_quantity, // Use actual filled quantity (may be partial)
            pending_trade.calculated_leverage,
            trading_fee_rate,
            Some(signal.id.clone()),
            Some(signal.confidence),
            Some(signal.reasoning.clone()),
        );

        // ========== PHASE 4: PERFORMANCE METRICS ==========

        // Set signal timestamp and calculate execution latency
        paper_trade.signal_timestamp = Some(signal.timestamp);
        paper_trade.execution_timestamp = Utc::now();

        if let Some(signal_ts) = paper_trade.signal_timestamp {
            let latency = (paper_trade.execution_timestamp - signal_ts)
                .num_milliseconds()
                .max(0) as u64;
            paper_trade.execution_latency_ms = Some(latency);

            debug!(
                "⚡ Execution latency: {}ms (signal: {}, execution: {})",
                latency,
                signal_ts.format("%H:%M:%S%.3f"),
                paper_trade.execution_timestamp.format("%H:%M:%S%.3f")
            );
        }

        // Set stop loss and take profit
        if let Err(e) = paper_trade.set_stop_loss(pending_trade.stop_loss) {
            warn!("Failed to set stop loss for {}: {}", signal.symbol, e);
        }

        if let Err(e) = paper_trade.set_take_profit(pending_trade.take_profit) {
            warn!("Failed to set take profit for {}: {}", signal.symbol, e);
        }

        let trade_id = paper_trade.id.clone();
        let fees_paid = paper_trade.trading_fees;

        // Add trade to portfolio
        {
            let mut portfolio = self.portfolio.write().await;
            portfolio.add_trade(paper_trade.clone())?;
        }

        // Save trade to database with full audit trail
        {
            // Create settings snapshot at trade creation time
            let settings = self.settings.read().await;
            let settings_snapshot = Some(crate::storage::EffectiveSettingsSnapshot::from_settings(
                &settings,
                &signal.symbol,
            ));
            drop(settings);

            // Extract AI suggestions for audit trail
            let ai_suggestions = Some((
                signal.suggested_stop_loss,
                signal.suggested_take_profit,
                None, // TODO: Add suggested_quantity to AITradingSignal struct
                signal.suggested_leverage,
            ));

            // Persist trade with complete audit trail
            if let Err(e) = self
                .storage
                .save_paper_trade(&paper_trade, settings_snapshot, ai_suggestions)
                .await
            {
                error!("Failed to save paper trade to database: {}", e);
            }
        }

        // Save portfolio snapshot
        {
            let portfolio = self.portfolio.read().await;
            if let Err(e) = self.storage.save_portfolio_snapshot(&portfolio).await {
                error!("Failed to save portfolio snapshot: {}", e);
            }
        }

        // Broadcast trade execution event
        let _ = self.event_broadcaster.send(PaperTradingEvent {
            event_type: "trade_executed".to_string(),
            data: serde_json::json!({
                "trade_id": trade_id,
                "symbol": signal.symbol,
                "type": trade_type.to_string(),
                "quantity": filled_quantity,  // FIXED: Use actual filled quantity
                "entry_price": execution_price,  // FIXED: Use actual execution price after simulation
                "signal_price": signal.entry_price,  // Original signal price for reference
                "leverage": pending_trade.calculated_leverage,
                "slippage_pct": ((execution_price - signal.entry_price) / signal.entry_price * 100.0).abs(),
            }),
            timestamp: Utc::now(),
        });

        info!(
            "Executed paper trade: {} {} {} @ {:.2} (signal: {:.2}) with {}x leverage",
            trade_type.to_string(),
            filled_quantity,
            signal.symbol,
            execution_price,  // FIXED: Show actual execution price after simulation
            signal.entry_price,  // Show original signal price for comparison
            pending_trade.calculated_leverage
        );

        Ok(TradeExecutionResult {
            success: true,
            trade_id: Some(trade_id),
            error_message: None,
            execution_price: Some(execution_price),  // FIXED: Return actual execution price
            fees_paid: Some(fees_paid),
        })
    }

    /// Monitor open trades for stop loss/take profit
    async fn monitor_open_trades(&self) -> Result<()> {
        let mut portfolio = self.portfolio.write().await;
        let closed_trades = portfolio.check_automatic_closures();
        drop(portfolio);

        for trade_id in closed_trades {
            // Broadcast trade closure event
            let _ = self.event_broadcaster.send(PaperTradingEvent {
                event_type: "trade_closed".to_string(),
                data: serde_json::json!({
                    "trade_id": trade_id,
                    "reason": "automatic",
                }),
                timestamp: Utc::now(),
            });
        }

        Ok(())
    }

    /// Update performance metrics and broadcast updates
    async fn update_performance_metrics(&self) -> Result<()> {
        let portfolio = self.portfolio.read().await;
        let metrics = portfolio.metrics.clone();
        drop(portfolio);

        // Broadcast performance update
        let _ = self.event_broadcaster.send(PaperTradingEvent {
            event_type: "performance_update".to_string(),
            data: serde_json::to_value(&metrics)?,
            timestamp: Utc::now(),
        });

        Ok(())
    }

    /// Run optimization analysis
    async fn run_optimization_analysis(&self) -> Result<()> {
        // This would integrate with the strategy optimizer
        // to provide recommendations for parameter adjustments
        debug!("Running optimization analysis");
        Ok(())
    }

    /// Load portfolio from storage
    async fn load_portfolio_from_storage(&self) -> Result<()> {
        // Implementation would load portfolio state from database
        debug!("Loading portfolio from storage");
        Ok(())
    }

    /// Save portfolio to storage
    async fn save_portfolio_to_storage(&self) -> Result<()> {
        // Implementation would save portfolio state to database
        debug!("Saving portfolio to storage");
        Ok(())
    }

    /// Check if engine is running
    pub async fn is_running(&self) -> bool {
        *self.is_running.read().await
    }

    /// Start engine asynchronously (for API calls)
    pub async fn start_async(&self) -> Result<()> {
        {
            let mut running = self.is_running.write().await;
            if *running {
                return Ok(()); // Already running
            }
            *running = true;
        }

        info!("Starting Paper Trading Engine (async)");

        // Load portfolio from storage if exists
        if let Err(e) = self.load_portfolio_from_storage().await {
            warn!("Failed to load portfolio from storage: {}", e);
        }

        // Start background tasks
        let engine = self.clone();
        tokio::spawn(async move {
            let price_update_handle = engine.start_price_updates();
            let signal_processing_handle = engine.start_signal_processing();
            let trade_monitoring_handle = engine.start_trade_monitoring();
            let performance_tracking_handle = engine.start_performance_tracking();
            let optimization_handle = engine.start_optimization_loop();
            let daily_metrics_handle = engine.start_daily_metrics_save();

            // Wait for all background tasks or until stopped
            let (
                _price_result,
                _signal_result,
                _trade_result,
                _perf_result,
                _opt_result,
                _metrics_result,
            ) = tokio::join!(
                price_update_handle,
                signal_processing_handle,
                trade_monitoring_handle,
                performance_tracking_handle,
                optimization_handle,
                daily_metrics_handle,
            );

            info!("Paper Trading Engine background tasks completed");
        });

        // Broadcast start event
        let _ = self.event_broadcaster.send(PaperTradingEvent {
            event_type: "engine_started".to_string(),
            data: serde_json::json!({ "timestamp": Utc::now() }),
            timestamp: Utc::now(),
        });

        info!("Paper Trading Engine started successfully (async)");
        Ok(())
    }

    /// Get current portfolio status
    pub async fn get_portfolio_status(&self) -> PerformanceSummary {
        let portfolio = self.portfolio.read().await;
        let metrics = &portfolio.metrics;

        PerformanceSummary {
            total_trades: metrics.total_trades,
            win_rate: metrics.win_rate,
            total_pnl: metrics.total_pnl,
            total_pnl_percentage: metrics.total_pnl_percentage,
            max_drawdown: metrics.max_drawdown,
            max_drawdown_percentage: metrics.max_drawdown_percentage,
            sharpe_ratio: metrics.sharpe_ratio,
            profit_factor: metrics.profit_factor,
            average_win: metrics.average_win,
            average_loss: metrics.average_loss,
            largest_win: metrics.largest_win,
            largest_loss: metrics.largest_loss,
            current_balance: portfolio.cash_balance,
            equity: portfolio.equity,
            margin_used: portfolio.margin_used,
            free_margin: portfolio.free_margin,
        }
    }

    /// Get open trades
    pub async fn get_open_trades(&self) -> Vec<super::trade::TradeSummary> {
        let portfolio = self.portfolio.read().await;
        portfolio
            .get_open_trades()
            .iter()
            .map(|trade| trade.get_summary())
            .collect()
    }

    /// Get closed trades
    pub async fn get_closed_trades(&self) -> Vec<super::trade::TradeSummary> {
        let portfolio = self.portfolio.read().await;
        portfolio
            .get_closed_trades()
            .iter()
            .map(|trade| trade.get_summary())
            .collect()
    }

    /// Manually close a trade
    pub async fn close_trade(&self, trade_id: &str) -> Result<()> {
        let current_price = {
            let portfolio = self.portfolio.read().await;
            if let Some(trade) = portfolio.get_trade(trade_id) {
                self.current_prices
                    .read()
                    .await
                    .get(&trade.symbol)
                    .copied()
                    .unwrap_or(trade.entry_price)
            } else {
                return Err(anyhow::anyhow!("Trade not found"));
            }
        };

        let mut portfolio = self.portfolio.write().await;
        portfolio.close_trade(trade_id, current_price, CloseReason::Manual)?;

        // Get the closed trade PnL for consecutive loss tracking
        let trade_pnl = portfolio
            .get_trade(trade_id)
            .and_then(|t| t.realized_pnl)
            .unwrap_or(0.0);

        // Get the closed trade and update in database
        if let Some(trade) = portfolio.get_trade(trade_id) {
            if let Err(e) = self.storage.update_paper_trade(trade).await {
                error!("Failed to update paper trade in database: {}", e);
            }

            // Save portfolio snapshot after trade closure
            if let Err(e) = self.storage.save_portfolio_snapshot(&portfolio).await {
                error!(
                    "Failed to save portfolio snapshot after trade closure: {}",
                    e
                );
            }
        }

        drop(portfolio); // Release lock before calling update_consecutive_losses

        // Update consecutive losses counter and check cool-down
        self.update_consecutive_losses(trade_pnl).await;

        // Broadcast trade closure event
        let _ = self.event_broadcaster.send(PaperTradingEvent {
            event_type: "trade_closed".to_string(),
            data: serde_json::json!({
                "trade_id": trade_id,
                "reason": "manual",
            }),
            timestamp: Utc::now(),
        });

        Ok(())
    }

    /// Update settings
    pub async fn update_settings(&self, new_settings: PaperTradingSettings) -> Result<()> {
        new_settings.validate()?;

        let mut settings = self.settings.write().await;
        *settings = new_settings;

        // Save updated settings to database
        if let Err(e) = self.storage.save_paper_trading_settings(&settings).await {
            error!("❌ Failed to save settings to database: {}", e);
            // Continue anyway - settings are still updated in memory
        }

        // Broadcast settings update
        let _ = self.event_broadcaster.send(PaperTradingEvent {
            event_type: "settings_updated".to_string(),
            data: serde_json::json!({ "timestamp": Utc::now() }),
            timestamp: Utc::now(),
        });

        info!("✅ Settings updated and saved to database");
        Ok(())
    }

    /// Get current settings
    pub async fn get_settings(&self) -> PaperTradingSettings {
        self.settings.read().await.clone()
    }

    /// Reset portfolio
    pub async fn reset_portfolio(&self) -> Result<()> {
        let settings = self.settings.read().await;
        let initial_balance = settings.basic.initial_balance;
        drop(settings);

        let mut portfolio = self.portfolio.write().await;
        *portfolio = PaperPortfolio::new(initial_balance);

        // Broadcast reset event
        let _ = self.event_broadcaster.send(PaperTradingEvent {
            event_type: "portfolio_reset".to_string(),
            data: serde_json::json!({ "timestamp": Utc::now() }),
            timestamp: Utc::now(),
        });

        info!("Portfolio reset to initial balance: ${}", initial_balance);
        Ok(())
    }

    /// Update confidence threshold for AI signals
    pub async fn update_confidence_threshold(&self, threshold: f64) -> Result<()> {
        if !(0.0..=1.0).contains(&threshold) {
            return Err(anyhow::anyhow!(
                "Confidence threshold must be between 0.0 and 1.0"
            ));
        }

        let mut settings = self.settings.write().await;

        // Update the AI confidence threshold
        settings.strategy.min_ai_confidence = threshold;

        // Save updated settings to database
        if let Err(e) = self.storage.save_paper_trading_settings(&settings).await {
            error!("❌ Failed to save settings to database: {}", e);
            // Continue anyway - settings are still updated in memory
        }

        // Broadcast settings update
        let _ = self.event_broadcaster.send(PaperTradingEvent {
            event_type: "settings_updated".to_string(),
            data: serde_json::json!({
                "confidence_threshold": threshold,
                "timestamp": Utc::now()
            }),
            timestamp: Utc::now(),
        });

        info!(
            "✅ Confidence threshold updated to: {:.1}% and saved to database",
            threshold * 100.0
        );
        Ok(())
    }

    /// Update signal refresh interval in minutes
    pub async fn update_signal_refresh_interval(&self, interval_minutes: u32) -> Result<()> {
        if interval_minutes == 0 || interval_minutes > 1440 {
            return Err(anyhow::anyhow!(
                "Signal refresh interval must be between 1 and 1440 minutes"
            ));
        }

        let mut settings = self.settings.write().await;

        // Update the signal refresh interval
        settings.ai.signal_refresh_interval_minutes = interval_minutes;

        // Save updated settings to database
        if let Err(e) = self.storage.save_paper_trading_settings(&settings).await {
            error!("❌ Failed to save settings to database: {}", e);
            // Continue anyway - settings are still updated in memory
        }

        // Broadcast settings update
        let _ = self.event_broadcaster.send(PaperTradingEvent {
            event_type: "settings_updated".to_string(),
            data: serde_json::json!({
                "signal_refresh_interval_minutes": interval_minutes,
                "timestamp": Utc::now()
            }),
            timestamp: Utc::now(),
        });

        info!(
            "✅ Signal refresh interval updated to: {} minutes and saved to database",
            interval_minutes
        );
        Ok(())
    }

    /// Update data resolution/timeframe for trading signals
    pub async fn update_data_resolution(&self, timeframe: String) -> Result<()> {
        // Validate timeframe
        let valid_timeframes = ["1m", "3m", "5m", "15m", "30m", "1h", "4h", "1d"];
        if !valid_timeframes.contains(&timeframe.as_str()) {
            return Err(anyhow::anyhow!(
                "Invalid timeframe. Must be one of: {}",
                valid_timeframes.join(", ")
            ));
        }

        let mut settings = self.settings.write().await;

        // Update the data resolution in backtesting settings
        settings.strategy.backtesting.data_resolution = timeframe.clone();

        // Save updated settings to database
        if let Err(e) = self.storage.save_paper_trading_settings(&settings).await {
            error!("❌ Failed to save settings to database: {}", e);
            // Continue anyway - settings are still updated in memory
        }

        // Broadcast settings update
        let _ = self.event_broadcaster.send(PaperTradingEvent {
            event_type: "settings_updated".to_string(),
            data: serde_json::json!({
                "data_resolution": timeframe,
                "timestamp": Utc::now()
            }),
            timestamp: Utc::now(),
        });

        info!(
            "✅ Data resolution/timeframe updated to: {} and saved to database",
            timeframe
        );
        Ok(())
    }

    /// Trigger manual AI analysis and trade execution
    pub async fn trigger_manual_analysis(&self) -> Result<()> {
        if !*self.is_running.read().await {
            return Err(anyhow::anyhow!("Engine is not running"));
        }

        info!("🔧 Manual AI analysis triggered");

        // Force process AI signals immediately
        match self.process_ai_signals().await {
            Ok(_) => {
                info!("✅ Manual AI analysis completed successfully");
                Ok(())
            },
            Err(e) => {
                error!("❌ Manual AI analysis failed: {}", e);
                Err(e)
            },
        }
    }

    /// Process an external AI signal (from frontend or API)
    /// This method allows paper trading to receive AI signals from external sources
    /// without needing to fetch market data from Binance.
    #[allow(clippy::too_many_arguments)]
    pub async fn process_external_ai_signal(
        &self,
        symbol: String,
        signal_type: TradingSignal,
        confidence: f64,
        reasoning: String,
        entry_price: f64,
        stop_loss: Option<f64>,
        take_profit: Option<f64>,
    ) -> Result<()> {
        if !*self.is_running.read().await {
            return Err(anyhow::anyhow!("Engine is not running"));
        }

        info!(
            "📥 Received external AI signal: {} {} with {}% confidence",
            symbol,
            match signal_type {
                TradingSignal::Long => "LONG",
                TradingSignal::Short => "SHORT",
                TradingSignal::Neutral => "NEUTRAL",
            },
            (confidence * 100.0) as i32
        );

        // Create AI trading signal
        let ai_signal = AITradingSignal {
            id: Uuid::new_v4().to_string(),
            symbol: symbol.clone(),
            signal_type,
            confidence,
            reasoning,
            entry_price,
            suggested_stop_loss: stop_loss,
            suggested_take_profit: take_profit,
            suggested_leverage: None,
            market_analysis: crate::paper_trading::MarketAnalysisData {
                trend_direction: match signal_type {
                    TradingSignal::Long => "Bullish".to_string(),
                    TradingSignal::Short => "Bearish".to_string(),
                    TradingSignal::Neutral => "Neutral".to_string(),
                },
                trend_strength: confidence,
                volatility: 0.0,
                support_levels: vec![],
                resistance_levels: vec![],
                volume_analysis: "External signal".to_string(),
                risk_score: 1.0 - confidence,
            },
            timestamp: Utc::now(),
        };

        // Save signal to database
        let settings = self.settings.read().await;
        let min_confidence = settings.strategy.min_ai_confidence;
        drop(settings);

        let executed = confidence >= min_confidence;
        if let Err(e) = self
            .storage
            .save_ai_signal(&ai_signal, executed, None)
            .await
        {
            error!("Failed to save external AI signal to database: {}", e);
        }

        // Broadcast signal via WebSocket
        let _ = self.event_broadcaster.send(PaperTradingEvent {
            event_type: "AISignalReceived".to_string(),
            data: serde_json::json!({
                "symbol": ai_signal.symbol,
                "signal": format!("{:?}", ai_signal.signal_type).to_lowercase(),
                "confidence": ai_signal.confidence,
                "timestamp": ai_signal.timestamp,
                "reasoning": ai_signal.reasoning,
                "entry_price": ai_signal.entry_price,
                "source": "external",
            }),
            timestamp: Utc::now(),
        });

        // Process trading signal if confidence is high enough
        if confidence >= min_confidence {
            info!(
                "✅ External signal confidence {:.1}% >= threshold {:.1}%, executing trade",
                confidence * 100.0,
                min_confidence * 100.0
            );

            match self.process_trading_signal(ai_signal.clone()).await {
                Ok(result) => {
                    if result.success {
                        info!("🎯 Successfully executed trade for external signal");
                        Ok(())
                    } else {
                        let error_msg = result
                            .error_message
                            .unwrap_or_else(|| "Unknown error".to_string());
                        warn!("⚠️ Trade execution failed: {}", error_msg);
                        Err(anyhow::anyhow!("Trade execution failed: {}", error_msg))
                    }
                },
                Err(e) => {
                    error!("❌ Failed to process external trading signal: {}", e);
                    Err(e)
                },
            }
        } else {
            info!(
                "ℹ️ External signal confidence {:.1}% below threshold {:.1}%, not executing",
                confidence * 100.0,
                min_confidence * 100.0
            );
            Ok(())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::paper_trading::settings::{
        AISettings, BasicSettings, ExecutionSettings, NotificationSettings, RiskSettings,
        StrategySettings, SymbolSettings,
    };
    use crate::paper_trading::MarketAnalysisData;
    use std::sync::Arc;
    use tokio::sync::broadcast;

    // Mock implementations for testing
    async fn create_mock_storage() -> Storage {
        use crate::config::DatabaseConfig;
        // Use in-memory storage for tests (no MongoDB connection required)
        // By using a non-MongoDB URL, Storage will use in-memory fallback
        let config = DatabaseConfig {
            url: "memory://test".to_string(),
            database_name: Some("test_db".to_string()),
            max_connections: 10,
            enable_logging: false,
        };
        Storage::new(&config).await.unwrap()
    }

    fn create_mock_binance_client() -> BinanceClient {
        use crate::config::BinanceConfig;
        let config = BinanceConfig {
            api_key: "test_api_key".to_string(),
            secret_key: "test_secret".to_string(),
            testnet: true,
            base_url: "https://testnet.binance.vision".to_string(),
            ws_url: "wss://testnet.binance.vision/ws".to_string(),
            futures_base_url: "https://testnet.binancefuture.com".to_string(),
            futures_ws_url: "wss://stream.binancefuture.com/ws".to_string(),
        };
        BinanceClient::new(config).expect("Failed to create mock binance client")
    }

    fn create_mock_ai_service() -> AIService {
        use crate::ai::AIServiceConfig;
        let config = AIServiceConfig {
            python_service_url: "http://localhost:8000".to_string(),
            request_timeout_seconds: 30,
            max_retries: 3,
            enable_caching: false,
            cache_ttl_seconds: 300,
        };
        AIService::new(config)
    }

    fn create_test_settings() -> PaperTradingSettings {
        PaperTradingSettings {
            basic: BasicSettings {
                initial_balance: 10000.0,
                max_positions: 5,
                default_position_size_pct: 5.0,
                default_leverage: 10,
                trading_fee_rate: 0.0004,
                funding_fee_rate: 0.0001,
                slippage_pct: 0.01,
                enabled: true,
                auto_restart: false,
            },
            risk: RiskSettings::default(),
            strategy: StrategySettings::default(),
            symbols: HashMap::new(),
            ai: AISettings::default(),
            execution: ExecutionSettings::default(),
            notifications: NotificationSettings::default(),
        }
    }

    fn create_event_broadcaster() -> broadcast::Sender<PaperTradingEvent> {
        let (tx, _) = broadcast::channel(100);
        tx
    }

    // Tests for PaperTradingEngine::new()
    #[tokio::test]
    async fn test_engine_new_with_default_settings() {
        let settings = create_test_settings();
        let binance_client = create_mock_binance_client();
        let ai_service = create_mock_ai_service();
        let storage = create_mock_storage().await;
        let broadcaster = create_event_broadcaster();

        let engine = PaperTradingEngine::new(
            settings.clone(),
            binance_client,
            ai_service,
            storage,
            broadcaster,
        )
        .await;

        assert!(engine.is_ok());
        let engine = engine.unwrap();

        let loaded_settings = engine.get_settings().await;
        assert_eq!(
            loaded_settings.basic.initial_balance,
            settings.basic.initial_balance
        );
    }

    #[tokio::test]
    async fn test_engine_new_initializes_portfolio() {
        let settings = create_test_settings();
        let binance_client = create_mock_binance_client();
        let ai_service = create_mock_ai_service();
        let storage = create_mock_storage().await;
        let broadcaster = create_event_broadcaster();

        let engine = PaperTradingEngine::new(
            settings.clone(),
            binance_client,
            ai_service,
            storage,
            broadcaster,
        )
        .await
        .unwrap();

        let portfolio_status = engine.get_portfolio_status().await;
        assert_eq!(
            portfolio_status.current_balance,
            settings.basic.initial_balance
        );
        assert_eq!(portfolio_status.equity, settings.basic.initial_balance);
    }

    #[tokio::test]
    async fn test_engine_new_initializes_empty_state() {
        let settings = create_test_settings();
        let binance_client = create_mock_binance_client();
        let ai_service = create_mock_ai_service();
        let storage = create_mock_storage().await;
        let broadcaster = create_event_broadcaster();

        let engine =
            PaperTradingEngine::new(settings, binance_client, ai_service, storage, broadcaster)
                .await
                .unwrap();

        assert!(!engine.is_running().await);
        let open_trades = engine.get_open_trades().await;
        assert_eq!(open_trades.len(), 0);
    }

    #[tokio::test]
    async fn test_engine_new_initializes_current_prices() {
        let settings = create_test_settings();
        let binance_client = create_mock_binance_client();
        let ai_service = create_mock_ai_service();
        let storage = create_mock_storage().await;
        let broadcaster = create_event_broadcaster();

        let engine =
            PaperTradingEngine::new(settings, binance_client, ai_service, storage, broadcaster)
                .await
                .unwrap();

        let prices = engine.current_prices.read().await;
        assert_eq!(prices.len(), 0);
    }

    // Tests for is_running()
    #[tokio::test]
    async fn test_is_running_initially_false() {
        let settings = create_test_settings();
        let binance_client = create_mock_binance_client();
        let ai_service = create_mock_ai_service();
        let storage = create_mock_storage().await;
        let broadcaster = create_event_broadcaster();

        let engine =
            PaperTradingEngine::new(settings, binance_client, ai_service, storage, broadcaster)
                .await
                .unwrap();

        assert!(!engine.is_running().await);
    }

    // Tests for get_settings()
    #[tokio::test]
    async fn test_get_settings_returns_correct_values() {
        let settings = create_test_settings();
        let binance_client = create_mock_binance_client();
        let ai_service = create_mock_ai_service();
        let storage = create_mock_storage().await;
        let broadcaster = create_event_broadcaster();

        let engine = PaperTradingEngine::new(
            settings.clone(),
            binance_client,
            ai_service,
            storage,
            broadcaster,
        )
        .await
        .unwrap();

        let retrieved_settings = engine.get_settings().await;
        assert_eq!(
            retrieved_settings.basic.initial_balance,
            settings.basic.initial_balance
        );
        assert_eq!(
            retrieved_settings.basic.max_positions,
            settings.basic.max_positions
        );
        assert_eq!(
            retrieved_settings.basic.default_leverage,
            settings.basic.default_leverage
        );
    }

    #[tokio::test]
    async fn test_get_settings_returns_cloned_copy() {
        let settings = create_test_settings();
        let binance_client = create_mock_binance_client();
        let ai_service = create_mock_ai_service();
        let storage = create_mock_storage().await;
        let broadcaster = create_event_broadcaster();

        let engine =
            PaperTradingEngine::new(settings, binance_client, ai_service, storage, broadcaster)
                .await
                .unwrap();

        let settings1 = engine.get_settings().await;
        let settings2 = engine.get_settings().await;

        assert_eq!(
            settings1.basic.initial_balance,
            settings2.basic.initial_balance
        );
    }

    // Tests for update_settings()
    #[tokio::test]
    async fn test_update_settings_valid() {
        let settings = create_test_settings();
        let binance_client = create_mock_binance_client();
        let ai_service = create_mock_ai_service();
        let storage = create_mock_storage().await;
        let broadcaster = create_event_broadcaster();

        let engine =
            PaperTradingEngine::new(settings, binance_client, ai_service, storage, broadcaster)
                .await
                .unwrap();

        let mut new_settings = create_test_settings();
        new_settings.basic.initial_balance = 20000.0;
        new_settings.basic.max_positions = 8;

        let result = engine.update_settings(new_settings.clone()).await;
        assert!(result.is_ok());

        let updated = engine.get_settings().await;
        assert_eq!(updated.basic.initial_balance, 20000.0);
        assert_eq!(updated.basic.max_positions, 8);
    }

    #[tokio::test]
    async fn test_update_settings_invalid_fails() {
        let settings = create_test_settings();
        let binance_client = create_mock_binance_client();
        let ai_service = create_mock_ai_service();
        let storage = create_mock_storage().await;
        let broadcaster = create_event_broadcaster();

        let engine =
            PaperTradingEngine::new(settings, binance_client, ai_service, storage, broadcaster)
                .await
                .unwrap();

        let mut invalid_settings = create_test_settings();
        invalid_settings.basic.initial_balance = -1000.0;

        let result = engine.update_settings(invalid_settings).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_update_settings_does_not_change_on_validation_failure() {
        let settings = create_test_settings();
        let binance_client = create_mock_binance_client();
        let ai_service = create_mock_ai_service();
        let storage = create_mock_storage().await;
        let broadcaster = create_event_broadcaster();

        let engine = PaperTradingEngine::new(
            settings.clone(),
            binance_client,
            ai_service,
            storage,
            broadcaster,
        )
        .await
        .unwrap();

        let original_balance = settings.basic.initial_balance;

        let mut invalid_settings = create_test_settings();
        invalid_settings.basic.initial_balance = -1000.0;

        let _ = engine.update_settings(invalid_settings).await;

        let current_settings = engine.get_settings().await;
        assert_eq!(current_settings.basic.initial_balance, original_balance);
    }

    // Tests for update_confidence_threshold()
    #[tokio::test]
    async fn test_update_confidence_threshold_valid() {
        let settings = create_test_settings();
        let binance_client = create_mock_binance_client();
        let ai_service = create_mock_ai_service();
        let storage = create_mock_storage().await;
        let broadcaster = create_event_broadcaster();

        let engine =
            PaperTradingEngine::new(settings, binance_client, ai_service, storage, broadcaster)
                .await
                .unwrap();

        let result = engine.update_confidence_threshold(0.85).await;
        assert!(result.is_ok());

        let updated_settings = engine.get_settings().await;
        assert_eq!(updated_settings.strategy.min_ai_confidence, 0.85);
    }

    #[tokio::test]
    async fn test_update_confidence_threshold_lower_bound() {
        let settings = create_test_settings();
        let binance_client = create_mock_binance_client();
        let ai_service = create_mock_ai_service();
        let storage = create_mock_storage().await;
        let broadcaster = create_event_broadcaster();

        let engine =
            PaperTradingEngine::new(settings, binance_client, ai_service, storage, broadcaster)
                .await
                .unwrap();

        let result = engine.update_confidence_threshold(0.0).await;
        assert!(result.is_ok());

        let updated_settings = engine.get_settings().await;
        assert_eq!(updated_settings.strategy.min_ai_confidence, 0.0);
    }

    #[tokio::test]
    async fn test_update_confidence_threshold_upper_bound() {
        let settings = create_test_settings();
        let binance_client = create_mock_binance_client();
        let ai_service = create_mock_ai_service();
        let storage = create_mock_storage().await;
        let broadcaster = create_event_broadcaster();

        let engine =
            PaperTradingEngine::new(settings, binance_client, ai_service, storage, broadcaster)
                .await
                .unwrap();

        let result = engine.update_confidence_threshold(1.0).await;
        assert!(result.is_ok());

        let updated_settings = engine.get_settings().await;
        assert_eq!(updated_settings.strategy.min_ai_confidence, 1.0);
    }

    #[tokio::test]
    async fn test_update_confidence_threshold_below_zero_fails() {
        let settings = create_test_settings();
        let binance_client = create_mock_binance_client();
        let ai_service = create_mock_ai_service();
        let storage = create_mock_storage().await;
        let broadcaster = create_event_broadcaster();

        let engine =
            PaperTradingEngine::new(settings, binance_client, ai_service, storage, broadcaster)
                .await
                .unwrap();

        let result = engine.update_confidence_threshold(-0.1).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_update_confidence_threshold_above_one_fails() {
        let settings = create_test_settings();
        let binance_client = create_mock_binance_client();
        let ai_service = create_mock_ai_service();
        let storage = create_mock_storage().await;
        let broadcaster = create_event_broadcaster();

        let engine =
            PaperTradingEngine::new(settings, binance_client, ai_service, storage, broadcaster)
                .await
                .unwrap();

        let result = engine.update_confidence_threshold(1.5).await;
        assert!(result.is_err());
    }

    // Tests for update_signal_refresh_interval()
    #[tokio::test]
    async fn test_update_signal_refresh_interval_valid() {
        let settings = create_test_settings();
        let binance_client = create_mock_binance_client();
        let ai_service = create_mock_ai_service();
        let storage = create_mock_storage().await;
        let broadcaster = create_event_broadcaster();

        let engine =
            PaperTradingEngine::new(settings, binance_client, ai_service, storage, broadcaster)
                .await
                .unwrap();

        let result = engine.update_signal_refresh_interval(15).await;
        assert!(result.is_ok());

        let updated_settings = engine.get_settings().await;
        assert_eq!(updated_settings.ai.signal_refresh_interval_minutes, 15);
    }

    #[tokio::test]
    async fn test_update_signal_refresh_interval_minimum() {
        let settings = create_test_settings();
        let binance_client = create_mock_binance_client();
        let ai_service = create_mock_ai_service();
        let storage = create_mock_storage().await;
        let broadcaster = create_event_broadcaster();

        let engine =
            PaperTradingEngine::new(settings, binance_client, ai_service, storage, broadcaster)
                .await
                .unwrap();

        let result = engine.update_signal_refresh_interval(1).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_update_signal_refresh_interval_maximum() {
        let settings = create_test_settings();
        let binance_client = create_mock_binance_client();
        let ai_service = create_mock_ai_service();
        let storage = create_mock_storage().await;
        let broadcaster = create_event_broadcaster();

        let engine =
            PaperTradingEngine::new(settings, binance_client, ai_service, storage, broadcaster)
                .await
                .unwrap();

        let result = engine.update_signal_refresh_interval(1440).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_update_signal_refresh_interval_zero_fails() {
        let settings = create_test_settings();
        let binance_client = create_mock_binance_client();
        let ai_service = create_mock_ai_service();
        let storage = create_mock_storage().await;
        let broadcaster = create_event_broadcaster();

        let engine =
            PaperTradingEngine::new(settings, binance_client, ai_service, storage, broadcaster)
                .await
                .unwrap();

        let result = engine.update_signal_refresh_interval(0).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_update_signal_refresh_interval_above_max_fails() {
        let settings = create_test_settings();
        let binance_client = create_mock_binance_client();
        let ai_service = create_mock_ai_service();
        let storage = create_mock_storage().await;
        let broadcaster = create_event_broadcaster();

        let engine =
            PaperTradingEngine::new(settings, binance_client, ai_service, storage, broadcaster)
                .await
                .unwrap();

        let result = engine.update_signal_refresh_interval(1441).await;
        assert!(result.is_err());
    }

    // Tests for get_portfolio_status()
    #[tokio::test]
    async fn test_get_portfolio_status_initial_state() {
        let settings = create_test_settings();
        let binance_client = create_mock_binance_client();
        let ai_service = create_mock_ai_service();
        let storage = create_mock_storage().await;
        let broadcaster = create_event_broadcaster();

        let engine = PaperTradingEngine::new(
            settings.clone(),
            binance_client,
            ai_service,
            storage,
            broadcaster,
        )
        .await
        .unwrap();

        let status = engine.get_portfolio_status().await;

        assert_eq!(status.current_balance, settings.basic.initial_balance);
        assert_eq!(status.equity, settings.basic.initial_balance);
        assert_eq!(status.total_trades, 0);
        assert_eq!(status.win_rate, 0.0);
        assert_eq!(status.total_pnl, 0.0);
    }

    #[tokio::test]
    async fn test_get_portfolio_status_fields() {
        let settings = create_test_settings();
        let binance_client = create_mock_binance_client();
        let ai_service = create_mock_ai_service();
        let storage = create_mock_storage().await;
        let broadcaster = create_event_broadcaster();

        let engine =
            PaperTradingEngine::new(settings, binance_client, ai_service, storage, broadcaster)
                .await
                .unwrap();

        let status = engine.get_portfolio_status().await;

        // Verify all fields exist
        let _ = status.total_trades;
        let _ = status.win_rate;
        let _ = status.total_pnl;
        let _ = status.total_pnl_percentage;
        let _ = status.max_drawdown;
        let _ = status.max_drawdown_percentage;
        let _ = status.sharpe_ratio;
        let _ = status.profit_factor;
        let _ = status.average_win;
        let _ = status.average_loss;
        let _ = status.largest_win;
        let _ = status.largest_loss;
        let _ = status.current_balance;
        let _ = status.equity;
        let _ = status.margin_used;
        let _ = status.free_margin;
    }

    // Tests for get_open_trades()
    #[tokio::test]
    async fn test_get_open_trades_initially_empty() {
        let settings = create_test_settings();
        let binance_client = create_mock_binance_client();
        let ai_service = create_mock_ai_service();
        let storage = create_mock_storage().await;
        let broadcaster = create_event_broadcaster();

        let engine =
            PaperTradingEngine::new(settings, binance_client, ai_service, storage, broadcaster)
                .await
                .unwrap();

        let open_trades = engine.get_open_trades().await;
        assert_eq!(open_trades.len(), 0);
    }

    // Tests for get_closed_trades()
    #[tokio::test]
    async fn test_get_closed_trades_initially_empty() {
        let settings = create_test_settings();
        let binance_client = create_mock_binance_client();
        let ai_service = create_mock_ai_service();
        let storage = create_mock_storage().await;
        let broadcaster = create_event_broadcaster();

        let engine =
            PaperTradingEngine::new(settings, binance_client, ai_service, storage, broadcaster)
                .await
                .unwrap();

        let closed_trades = engine.get_closed_trades().await;
        assert_eq!(closed_trades.len(), 0);
    }

    // Tests for reset_portfolio()
    #[tokio::test]
    async fn test_reset_portfolio_restores_initial_balance() {
        let settings = create_test_settings();
        let binance_client = create_mock_binance_client();
        let ai_service = create_mock_ai_service();
        let storage = create_mock_storage().await;
        let broadcaster = create_event_broadcaster();

        let engine = PaperTradingEngine::new(
            settings.clone(),
            binance_client,
            ai_service,
            storage,
            broadcaster,
        )
        .await
        .unwrap();

        let result = engine.reset_portfolio().await;
        assert!(result.is_ok());

        let status = engine.get_portfolio_status().await;
        assert_eq!(status.current_balance, settings.basic.initial_balance);
        assert_eq!(status.equity, settings.basic.initial_balance);
    }

    #[tokio::test]
    async fn test_reset_portfolio_clears_trades() {
        let settings = create_test_settings();
        let binance_client = create_mock_binance_client();
        let ai_service = create_mock_ai_service();
        let storage = create_mock_storage().await;
        let broadcaster = create_event_broadcaster();

        let engine =
            PaperTradingEngine::new(settings, binance_client, ai_service, storage, broadcaster)
                .await
                .unwrap();

        let result = engine.reset_portfolio().await;
        assert!(result.is_ok());

        let open_trades = engine.get_open_trades().await;
        let closed_trades = engine.get_closed_trades().await;

        assert_eq!(open_trades.len(), 0);
        assert_eq!(closed_trades.len(), 0);
    }

    #[tokio::test]
    async fn test_reset_portfolio_resets_metrics() {
        let settings = create_test_settings();
        let binance_client = create_mock_binance_client();
        let ai_service = create_mock_ai_service();
        let storage = create_mock_storage().await;
        let broadcaster = create_event_broadcaster();

        let engine =
            PaperTradingEngine::new(settings, binance_client, ai_service, storage, broadcaster)
                .await
                .unwrap();

        let result = engine.reset_portfolio().await;
        assert!(result.is_ok());

        let status = engine.get_portfolio_status().await;
        assert_eq!(status.total_trades, 0);
        assert_eq!(status.total_pnl, 0.0);
        assert_eq!(status.win_rate, 0.0);
    }

    // Tests for PendingTrade structure
    #[test]
    fn test_pending_trade_creation() {
        let signal = AITradingSignal {
            id: "signal-123".to_string(),
            symbol: "BTCUSDT".to_string(),
            signal_type: crate::strategies::TradingSignal::Long,
            confidence: 0.85,
            reasoning: "Test signal".to_string(),
            entry_price: 50000.0,
            suggested_stop_loss: Some(48000.0),
            suggested_take_profit: Some(55000.0),
            suggested_leverage: Some(10),
            market_analysis: MarketAnalysisData {
                trend_direction: "Bullish".to_string(),
                trend_strength: 0.8,
                volatility: 0.3,
                support_levels: vec![48000.0],
                resistance_levels: vec![52000.0],
                volume_analysis: "High".to_string(),
                risk_score: 0.4,
            },
            timestamp: Utc::now(),
        };

        let pending = PendingTrade {
            signal: signal.clone(),
            calculated_quantity: 0.5,
            calculated_leverage: 10,
            stop_loss: 48000.0,
            take_profit: 55000.0,
            timestamp: Utc::now(),
        };

        assert_eq!(pending.signal.symbol, "BTCUSDT");
        assert_eq!(pending.calculated_quantity, 0.5);
        assert_eq!(pending.calculated_leverage, 10);
        assert_eq!(pending.stop_loss, 48000.0);
        assert_eq!(pending.take_profit, 55000.0);
    }

    #[test]
    fn test_pending_trade_clone() {
        let signal = AITradingSignal {
            id: "signal-456".to_string(),
            symbol: "ETHUSDT".to_string(),
            signal_type: crate::strategies::TradingSignal::Short,
            confidence: 0.75,
            reasoning: "Test signal 2".to_string(),
            entry_price: 3000.0,
            suggested_stop_loss: Some(3100.0),
            suggested_take_profit: Some(2800.0),
            suggested_leverage: Some(5),
            market_analysis: MarketAnalysisData {
                trend_direction: "Bearish".to_string(),
                trend_strength: 0.7,
                volatility: 0.4,
                support_levels: vec![2800.0],
                resistance_levels: vec![3100.0],
                volume_analysis: "Moderate".to_string(),
                risk_score: 0.5,
            },
            timestamp: Utc::now(),
        };

        let pending1 = PendingTrade {
            signal: signal.clone(),
            calculated_quantity: 1.0,
            calculated_leverage: 5,
            stop_loss: 3100.0,
            take_profit: 2800.0,
            timestamp: Utc::now(),
        };

        let pending2 = pending1.clone();

        assert_eq!(pending1.signal.symbol, pending2.signal.symbol);
        assert_eq!(pending1.calculated_quantity, pending2.calculated_quantity);
        assert_eq!(pending1.calculated_leverage, pending2.calculated_leverage);
    }

    // Tests for engine cloning
    #[tokio::test]
    async fn test_engine_clone() {
        let settings = create_test_settings();
        let binance_client = create_mock_binance_client();
        let ai_service = create_mock_ai_service();
        let storage = create_mock_storage().await;
        let broadcaster = create_event_broadcaster();

        let engine1 =
            PaperTradingEngine::new(settings, binance_client, ai_service, storage, broadcaster)
                .await
                .unwrap();

        let engine2 = engine1.clone();

        // Both engines should share the same state
        let settings1 = engine1.get_settings().await;
        let settings2 = engine2.get_settings().await;

        assert_eq!(
            settings1.basic.initial_balance,
            settings2.basic.initial_balance
        );
    }

    // Tests for configuration with symbol-specific settings
    #[tokio::test]
    async fn test_engine_with_symbol_specific_settings() {
        let mut settings = create_test_settings();

        let btc_settings = SymbolSettings {
            enabled: true,
            leverage: Some(15),
            position_size_pct: Some(8.0),
            stop_loss_pct: Some(2.5),
            take_profit_pct: Some(5.0),
            trading_hours: None,
            min_price_movement_pct: Some(0.3),
            max_positions: Some(2),
            custom_params: HashMap::new(),
        };

        settings.symbols.insert("BTCUSDT".to_string(), btc_settings);

        let binance_client = create_mock_binance_client();
        let ai_service = create_mock_ai_service();
        let storage = create_mock_storage().await;
        let broadcaster = create_event_broadcaster();

        let engine = PaperTradingEngine::new(
            settings.clone(),
            binance_client,
            ai_service,
            storage,
            broadcaster,
        )
        .await
        .unwrap();

        let loaded_settings = engine.get_settings().await;
        assert!(loaded_settings.symbols.contains_key("BTCUSDT"));

        let effective = loaded_settings.get_symbol_settings("BTCUSDT");
        assert_eq!(effective.leverage, 15);
        assert_eq!(effective.position_size_pct, 8.0);
    }

    // Tests for multiple settings updates
    #[tokio::test]
    async fn test_multiple_settings_updates() {
        let settings = create_test_settings();
        let binance_client = create_mock_binance_client();
        let ai_service = create_mock_ai_service();
        let storage = create_mock_storage().await;
        let broadcaster = create_event_broadcaster();

        let engine =
            PaperTradingEngine::new(settings, binance_client, ai_service, storage, broadcaster)
                .await
                .unwrap();

        // First update
        let result1 = engine.update_confidence_threshold(0.75).await;
        assert!(result1.is_ok());

        // Second update
        let result2 = engine.update_signal_refresh_interval(10).await;
        assert!(result2.is_ok());

        let final_settings = engine.get_settings().await;
        assert_eq!(final_settings.strategy.min_ai_confidence, 0.75);
        assert_eq!(final_settings.ai.signal_refresh_interval_minutes, 10);
    }

    // Tests for concurrent access
    #[tokio::test]
    async fn test_concurrent_settings_read() {
        let settings = create_test_settings();
        let binance_client = create_mock_binance_client();
        let ai_service = create_mock_ai_service();
        let storage = create_mock_storage().await;
        let broadcaster = create_event_broadcaster();

        let engine = Arc::new(
            PaperTradingEngine::new(settings, binance_client, ai_service, storage, broadcaster)
                .await
                .unwrap(),
        );

        let engine1 = Arc::clone(&engine);
        let engine2 = Arc::clone(&engine);

        let handle1 = tokio::spawn(async move { engine1.get_settings().await });

        let handle2 = tokio::spawn(async move { engine2.get_settings().await });

        let (settings1, settings2) = tokio::join!(handle1, handle2);

        assert!(settings1.is_ok());
        assert!(settings2.is_ok());
    }

    // Tests for state consistency
    #[tokio::test]
    async fn test_portfolio_state_consistency() {
        let settings = create_test_settings();
        let binance_client = create_mock_binance_client();
        let ai_service = create_mock_ai_service();
        let storage = create_mock_storage().await;
        let broadcaster = create_event_broadcaster();

        let engine = PaperTradingEngine::new(
            settings.clone(),
            binance_client,
            ai_service,
            storage,
            broadcaster,
        )
        .await
        .unwrap();

        let status1 = engine.get_portfolio_status().await;
        let status2 = engine.get_portfolio_status().await;

        assert_eq!(status1.current_balance, status2.current_balance);
        assert_eq!(status1.equity, status2.equity);
        assert_eq!(status1.total_trades, status2.total_trades);
    }

    // Tests for edge cases in threshold updates
    #[tokio::test]
    async fn test_confidence_threshold_precise_boundaries() {
        let settings = create_test_settings();
        let binance_client = create_mock_binance_client();
        let ai_service = create_mock_ai_service();
        let storage = create_mock_storage().await;
        let broadcaster = create_event_broadcaster();

        let engine =
            PaperTradingEngine::new(settings, binance_client, ai_service, storage, broadcaster)
                .await
                .unwrap();

        // Test very small positive value
        assert!(engine.update_confidence_threshold(0.0001).await.is_ok());

        // Test very close to 1.0
        assert!(engine.update_confidence_threshold(0.9999).await.is_ok());

        // Test just over 1.0
        assert!(engine.update_confidence_threshold(1.0001).await.is_err());

        // Test negative very close to 0
        assert!(engine.update_confidence_threshold(-0.0001).await.is_err());
    }

    #[tokio::test]
    async fn test_signal_refresh_interval_boundaries() {
        let settings = create_test_settings();
        let binance_client = create_mock_binance_client();
        let ai_service = create_mock_ai_service();
        let storage = create_mock_storage().await;
        let broadcaster = create_event_broadcaster();

        let engine =
            PaperTradingEngine::new(settings, binance_client, ai_service, storage, broadcaster)
                .await
                .unwrap();

        // Test minimum valid value
        assert!(engine.update_signal_refresh_interval(1).await.is_ok());

        // Test maximum valid value
        assert!(engine.update_signal_refresh_interval(1440).await.is_ok());

        // Test just above maximum
        assert!(engine.update_signal_refresh_interval(1441).await.is_err());
    }

    // Test initialization with various balances
    #[tokio::test]
    async fn test_engine_with_different_initial_balances() {
        let test_balances = vec![1000.0, 5000.0, 10000.0, 50000.0, 100000.0];

        for balance in test_balances {
            let mut settings = create_test_settings();
            settings.basic.initial_balance = balance;

            let binance_client = create_mock_binance_client();
            let ai_service = create_mock_ai_service();
            let storage = create_mock_storage().await;
            let broadcaster = create_event_broadcaster();

            let engine =
                PaperTradingEngine::new(settings, binance_client, ai_service, storage, broadcaster)
                    .await
                    .unwrap();

            let status = engine.get_portfolio_status().await;
            assert_eq!(status.current_balance, balance);
            assert_eq!(status.equity, balance);
        }
    }

    // Test settings validation through engine
    #[tokio::test]
    async fn test_engine_rejects_invalid_leverage_settings() {
        let mut settings = create_test_settings();
        settings.basic.default_leverage = 150; // Invalid leverage

        let binance_client = create_mock_binance_client();
        let ai_service = create_mock_ai_service();
        let storage = create_mock_storage().await;
        let broadcaster = create_event_broadcaster();

        let engine =
            PaperTradingEngine::new(settings, binance_client, ai_service, storage, broadcaster)
                .await
                .unwrap();

        // Engine should be created but validation should fail on update
        let mut new_settings = create_test_settings();
        new_settings.basic.default_leverage = 200;

        let result = engine.update_settings(new_settings).await;
        assert!(result.is_err());
    }
}
