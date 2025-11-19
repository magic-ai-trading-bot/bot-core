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

use super::{
    // @spec:FR-PAPER-001 - Paper Trading Engine
    // @ref:specs/02-design/2.5-components/COMP-RUST-TRADING.md#paper-trading
    // @test:TC-INTEGRATION-025, TC-INTEGRATION-026
    portfolio::PaperPortfolio,
    settings::PaperTradingSettings,
    strategy_optimizer::StrategyOptimizer,
    trade::{CloseReason, PaperTrade, TradeType},
    AITradingSignal,
    MarketAnalysisData,
    PaperTradingEvent,
    PerformanceSummary,
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
                    warn!("Failed to get AI signal for {}: {}", symbol, e);
                },
            }
        }

        Ok(())
    }

    /// Get AI signal for a specific symbol
    async fn get_ai_signal_for_symbol(&self, symbol: &str) -> Result<AITradingSignal> {
        // @spec:FR-STRATEGY-006 - Multi-Timeframe Analysis (FIXED)
        // Fetch multiple timeframes for better market context
        let mut timeframe_data = HashMap::new();
        let timeframes = vec![
            ("1h", 100), // 100 hourly candles (~ 4 days)
            ("4h", 60),  // 60 4-hour candles (~ 10 days)
            ("1d", 30),  // 30 daily candles (~ 1 month)
        ];

        let mut current_price = 0.0;
        let mut volume_24h = 0.0;

        // Fetch data for all timeframes
        for (timeframe, limit) in timeframes {
            let klines = self
                .binance_client
                .get_klines(symbol, timeframe, Some(limit))
                .await?;

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

            // Use 1h data for current price and volume
            if timeframe == "1h" {
                current_price = candles.last().map(|c| c.close).unwrap_or(0.0);
                volume_24h = candles.iter().map(|c| c.volume).sum();
            }

            timeframe_data.insert(timeframe.to_string(), candles);
        }

        // Execute technical analysis with StrategyEngine
        // @spec:FR-STRATEGY-005 - Strategy Engine Integration (FIXED)
        let settings_read = self.settings.read().await;
        let enabled_strategies: Vec<String> = settings_read
            .strategy
            .enabled_strategies
            .keys()
            .cloned()
            .collect();
        let min_confidence = settings_read.strategy.min_ai_confidence;
        drop(settings_read);

        // Create strategy engine
        let strategy_engine = crate::strategies::strategy_engine::StrategyEngine::with_config(
            crate::strategies::strategy_engine::StrategyEngineConfig {
                enabled_strategies: enabled_strategies.clone(),
                min_confidence_threshold: min_confidence,
                signal_combination_mode:
                    crate::strategies::strategy_engine::SignalCombinationMode::Consensus,
                max_history_size: 100,
            },
        );

        let strategy_input = crate::strategies::StrategyInput {
            symbol: symbol.to_string(),
            timeframe_data: timeframe_data.clone(),
            current_price,
            volume_24h,
            timestamp: chrono::Utc::now().timestamp_millis(),
        };

        // Get technical analysis
        let technical_analysis = strategy_engine.analyze_market(&strategy_input).await?;

        // Build technical indicators map for AI
        let mut technical_indicators = HashMap::new();
        for strategy_result in &technical_analysis.strategy_signals {
            technical_indicators.insert(
                strategy_result.strategy_name.clone(),
                serde_json::json!({
                    "signal": format!("{:?}", strategy_result.signal),
                    "confidence": strategy_result.confidence,
                }),
            );
        }

        // Determine market condition from strategy signals
        let long_count = technical_analysis
            .metadata
            .get("long_signals")
            .and_then(|v| v.as_u64())
            .unwrap_or(0);
        let short_count = technical_analysis
            .metadata
            .get("short_signals")
            .and_then(|v| v.as_u64())
            .unwrap_or(0);

        let market_condition = if long_count > short_count {
            "Bullish"
        } else if short_count > long_count {
            "Bearish"
        } else {
            "Neutral"
        };

        // Determine risk level from combined confidence
        let risk_level = if technical_analysis.combined_confidence < 0.5 {
            "High" // Low confidence = high uncertainty = high risk
        } else if technical_analysis.combined_confidence > 0.75 {
            "Low" // High confidence = low risk
        } else {
            "Moderate"
        };

        let ai_request = crate::ai::AIAnalysisRequest {
            symbol: symbol.to_string(),
            timeframe_data,
            current_price,
            volume_24h,
            timestamp: chrono::Utc::now().timestamp_millis(),
            strategy_context: crate::ai::AIStrategyContext {
                selected_strategies: enabled_strategies,
                market_condition: market_condition.to_string(),
                risk_level: risk_level.to_string(),
                user_preferences: HashMap::new(),
                technical_indicators,
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

        // @spec:FR-RISK-002 - Dynamic ATR-based Stop Loss (FIXED)
        // Calculate stop loss using ATR for dynamic volatility-adjusted risk management
        let stop_loss = signal.suggested_stop_loss.unwrap_or_else(|| {
            // Fetch recent candles for ATR calculation
            let atr_stop_loss = async {
                // Get recent 30 candles for ATR calculation
                match self
                    .binance_client
                    .get_klines(&signal.symbol, "1h", Some(30))
                    .await
                {
                    Ok(klines) => {
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

                        // Calculate ATR with 14-period
                        if let Ok(atr_values) =
                            crate::strategies::indicators::calculate_atr(&candles, 14)
                        {
                            if let Some(current_atr) = atr_values.last() {
                                // Use 2x ATR as stop loss distance (industry standard)
                                let atr_stop_distance = current_atr * 2.0;
                                return match signal.signal_type {
                                    crate::strategies::TradingSignal::Long => {
                                        entry_price - atr_stop_distance
                                    },
                                    crate::strategies::TradingSignal::Short => {
                                        entry_price + atr_stop_distance
                                    },
                                    _ => entry_price,
                                };
                            }
                        }
                    },
                    Err(e) => {
                        warn!("Failed to fetch candles for ATR calculation: {}", e);
                    },
                }

                // Fallback to fixed percentage if ATR calculation fails
                match signal.signal_type {
                    crate::strategies::TradingSignal::Long => {
                        entry_price * (1.0 - symbol_settings.stop_loss_pct / 100.0)
                    },
                    crate::strategies::TradingSignal::Short => {
                        entry_price * (1.0 + symbol_settings.stop_loss_pct / 100.0)
                    },
                    _ => entry_price,
                }
            };

            // Block on the async operation
            tokio::task::block_in_place(|| {
                tokio::runtime::Handle::current().block_on(atr_stop_loss)
            })
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
            },
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
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::paper_trading::settings::{
        AISettings, BasicSettings, ExecutionSettings, NotificationSettings, RiskSettings,
        StrategySettings, SymbolSettings,
    };
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
