use anyhow::Result;
use chrono::{DateTime, Utc};
use serde_json;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::{broadcast, RwLock};
use tokio::time::{interval, Duration};
use tracing::{debug, error, info, warn};

use crate::ai::AIService;
use crate::binance::BinanceClient;
use crate::storage::Storage;
use crate::strategies::{MarketCondition, RiskLevel};

use super::{
    portfolio::PaperPortfolio,
    settings::PaperTradingSettings,
    strategy_optimizer::StrategyOptimizer,
    trade::{CloseReason, PaperTrade, TradeType},
    AITradingSignal, MarketAnalysisData, PaperTradingEvent, PerformanceSummary,
    TradeExecutionResult,
};
use uuid;

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
            }
            Ok(None) => {
                info!("📝 No saved settings found, using defaults");
                default_settings
            }
            Err(e) => {
                warn!(
                    "⚠️ Failed to load settings from database, using defaults: {}",
                    e
                );
                default_settings
            }
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
                }
                Err(e) => {
                    warn!("Failed to get price for {}: {}", symbol, e);
                }
            }

            // Get funding rate for futures
            match self.binance_client.get_funding_rate(symbol).await {
                Ok(funding_info) => {
                    if let Ok(rate) = funding_info.funding_rate.parse::<f64>() {
                        funding_rates.insert(symbol.clone(), rate);
                    }
                }
                Err(_) => {
                    // Funding rate not available, use default
                    funding_rates.insert(symbol.clone(), 0.0);
                }
            }
        }

        // Update portfolio with new prices
        {
            let mut portfolio = self.portfolio.write().await;
            portfolio.update_prices(new_prices.clone(), Some(funding_rates));
        }

        // Update cached prices
        {
            let mut prices = self.current_prices.write().await;
            prices.extend(new_prices.clone());
        }

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
                            }
                            Err(e) => {
                                error!("Failed to process trading signal for {}: {}", symbol, e);
                            }
                        }
                    } else {
                        debug!(
                            "Signal confidence {} below threshold {} for {}",
                            signal.confidence, min_confidence, symbol
                        );
                    }
                }
                Err(e) => {
                    warn!("Failed to get AI signal for {}: {}", symbol, e);
                }
            }
        }

        Ok(())
    }

    /// Get AI signal for a specific symbol
    async fn get_ai_signal_for_symbol(&self, symbol: &str) -> Result<AITradingSignal> {
        // Get recent market data
        let klines = self
            .binance_client
            .get_klines(symbol, "1h", Some(100))
            .await?;

        // Convert to AI request format
        let mut timeframe_data = HashMap::new();
        let candles: Vec<crate::market_data::cache::CandleData> = klines
            .into_iter()
            .map(|kline| crate::market_data::cache::CandleData {
                open_time: kline.open_time,
                close_time: kline.close_time,
                open: kline.open.parse().unwrap_or(0.0),
                high: kline.high.parse().unwrap_or(0.0),
                low: kline.low.parse().unwrap_or(0.0),
                close: kline.close.parse().unwrap_or(0.0),
                volume: kline.volume.parse().unwrap_or(0.0),
                quote_volume: kline.quote_asset_volume.parse().unwrap_or(0.0),
                trades: kline.number_of_trades,
                is_closed: true,
            })
            .collect();

        timeframe_data.insert("1h".to_string(), candles.clone());

        let current_price = candles.last().map(|c| c.close).unwrap_or(0.0);
        let volume_24h = candles.iter().map(|c| c.volume).sum();

        let ai_request = crate::ai::AIAnalysisRequest {
            symbol: symbol.to_string(),
            timeframe_data,
            current_price,
            volume_24h,
            timestamp: chrono::Utc::now().timestamp_millis(),
            strategy_context: crate::ai::AIStrategyContext {
                selected_strategies: vec!["ai_ensemble".to_string()],
                market_condition: "Unknown".to_string(),
                risk_level: "Moderate".to_string(),
                user_preferences: HashMap::new(),
                technical_indicators: HashMap::new(),
            },
        };

        // Convert to StrategyInput for AI analysis
        let strategy_input = crate::strategies::StrategyInput {
            symbol: ai_request.symbol.clone(),
            timeframe_data: ai_request.timeframe_data.clone(),
            current_price: ai_request.current_price,
            volume_24h: ai_request.volume_24h,
            timestamp: ai_request.timestamp,
        };

        // Get AI analysis
        let ai_response = self
            .ai_service
            .analyze_for_trading_signal(&strategy_input, ai_request.strategy_context)
            .await?;

        // Convert to paper trading signal
        let signal = AITradingSignal {
            id: uuid::Uuid::new_v4().to_string(),
            symbol: symbol.to_string(),
            signal_type: ai_response.signal,
            confidence: ai_response.confidence,
            reasoning: ai_response.reasoning,
            entry_price: current_price,
            suggested_stop_loss: ai_response.risk_assessment.stop_loss_suggestion,
            suggested_take_profit: ai_response.risk_assessment.take_profit_suggestion,
            suggested_leverage: None, // Will be calculated based on settings
            market_analysis: MarketAnalysisData {
                trend_direction: format!("{:?}", ai_response.market_analysis.trend_direction),
                trend_strength: ai_response.market_analysis.trend_strength,
                volatility: 0.0, // Would be calculated from price data
                support_levels: ai_response.market_analysis.support_levels,
                resistance_levels: ai_response.market_analysis.resistance_levels,
                volume_analysis: format!("{:?}", ai_response.market_analysis.volume_analysis),
                risk_score: ai_response.risk_assessment.technical_risk,
            },
            timestamp: Utc::now(),
        };

        Ok(signal)
    }

    /// Process a trading signal and potentially execute a trade
    async fn process_trading_signal(
        &self,
        signal: AITradingSignal,
    ) -> Result<TradeExecutionResult> {
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
        let entry_price = signal.entry_price;

        // Calculate stop loss and take profit
        let stop_loss = signal.suggested_stop_loss.unwrap_or_else(|| {
            match signal.signal_type {
                crate::strategies::TradingSignal::Long => {
                    entry_price * (1.0 - symbol_settings.stop_loss_pct / 100.0)
                }
                crate::strategies::TradingSignal::Short => {
                    entry_price * (1.0 + symbol_settings.stop_loss_pct / 100.0)
                }
                _ => entry_price, // Neutral signal
            }
        });

        let take_profit = signal.suggested_take_profit.unwrap_or_else(|| {
            match signal.signal_type {
                crate::strategies::TradingSignal::Long => {
                    entry_price * (1.0 + symbol_settings.take_profit_pct / 100.0)
                }
                crate::strategies::TradingSignal::Short => {
                    entry_price * (1.0 - symbol_settings.take_profit_pct / 100.0)
                }
                _ => entry_price, // Neutral signal
            }
        });

        // Calculate position size
        let risk_amount = portfolio.equity * (symbol_settings.position_size_pct / 100.0);
        let price_diff = (entry_price - stop_loss).abs();
        let max_quantity = if price_diff > 0.0 {
            risk_amount / price_diff
        } else {
            0.0
        };

        // Limit by available margin
        let required_margin = (max_quantity * entry_price) / leverage as f64;
        let quantity = if required_margin <= portfolio.free_margin {
            max_quantity
        } else {
            (portfolio.free_margin * 0.95 * leverage as f64) / entry_price
        };

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

    /// Execute a single trade
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
            }
        };

        // Get current settings
        let settings = self.settings.read().await;
        let trading_fee_rate = settings.basic.trading_fee_rate;
        drop(settings);

        // Create paper trade
        let mut paper_trade = PaperTrade::new(
            signal.symbol.clone(),
            trade_type,
            signal.entry_price,
            pending_trade.calculated_quantity,
            pending_trade.calculated_leverage,
            trading_fee_rate,
            Some(signal.id.clone()),
            Some(signal.confidence),
            Some(signal.reasoning.clone()),
        );

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

        // Save trade to database
        if let Err(e) = self.storage.save_paper_trade(&paper_trade).await {
            error!("Failed to save paper trade to database: {}", e);
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
                "quantity": pending_trade.calculated_quantity,
                "entry_price": signal.entry_price,
                "leverage": pending_trade.calculated_leverage,
            }),
            timestamp: Utc::now(),
        });

        info!(
            "Executed paper trade: {} {} {} @ {} with {}x leverage",
            trade_type.to_string(),
            pending_trade.calculated_quantity,
            signal.symbol,
            signal.entry_price,
            pending_trade.calculated_leverage
        );

        Ok(TradeExecutionResult {
            success: true,
            trade_id: Some(trade_id),
            error_message: None,
            execution_price: Some(signal.entry_price),
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
        if threshold < 0.0 || threshold > 1.0 {
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
            }
            Err(e) => {
                error!("❌ Manual AI analysis failed: {}", e);
                Err(e)
            }
        }
    }
}
