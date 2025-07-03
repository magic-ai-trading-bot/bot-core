use anyhow::Result;
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;
use tokio::time::{interval, sleep};
use tracing::{debug, error, info, warn};
use uuid::Uuid;

use crate::binance::{BinanceClient, NewOrderRequest};
use crate::config::{BinanceConfig, TradingConfig};
use crate::market_data::{MarketDataProcessor, TradingSignal};
use crate::storage::{Storage, TradeRecord};

use super::position_manager::{Position, PositionManager};
use super::risk_manager::RiskManager;

#[derive(Clone)]
pub struct TradingEngine {
    binance_config: BinanceConfig,
    config: TradingConfig,
    client: BinanceClient,
    market_data: MarketDataProcessor,
    position_manager: PositionManager,
    risk_manager: RiskManager,
    storage: Storage,
}

impl TradingEngine {
    pub async fn new(
        binance_config: BinanceConfig,
        config: TradingConfig,
        market_data: MarketDataProcessor,
        storage: Storage,
    ) -> Result<Self> {
        let client = BinanceClient::new(binance_config.clone());
        let position_manager = PositionManager::new();
        let risk_manager = RiskManager::new(config.clone());

        // Initialize leverage and margin type for configured symbols
        let symbols = market_data.get_supported_symbols();
        for symbol in &symbols {
            if let Err(e) = client.change_leverage(symbol, config.leverage).await {
                warn!("Failed to set leverage for {}: {}", symbol, e);
            }
            
            if let Err(e) = client.change_margin_type(symbol, &config.margin_type).await {
                warn!("Failed to set margin type for {}: {}", symbol, e);
            }
            
            sleep(Duration::from_millis(100)).await; // Rate limiting
        }

        Ok(Self {
            binance_config,
            config,
            client,
            market_data,
            position_manager,
            risk_manager,
            storage,
        })
    }

    pub async fn start(&self) -> Result<()> {
        if !self.config.enabled {
            info!("Trading is disabled in configuration");
            return Ok(());
        }

        info!("Starting Trading Engine");

        // Load existing positions from exchange
        self.sync_positions().await?;

        // Start main trading loop
        let trading_handle = self.start_trading_loop();
        
        // Start position monitoring
        let monitoring_handle = self.start_position_monitoring();

        // Wait for both tasks
        tokio::try_join!(
            async { trading_handle.await? },
            async { monitoring_handle.await? }
        )?;

        Ok(())
    }

    async fn sync_positions(&self) -> Result<()> {
        info!("Syncing positions with exchange");
        
        let positions = self.client.get_futures_positions().await?;
        let mut active_positions = 0;

        for binance_position in positions {
            let position_amt: f64 = binance_position.position_amt.parse().unwrap_or(0.0);
            
            if position_amt.abs() > 0.0 {
                let position = Position {
                    id: Uuid::new_v4().to_string(),
                    symbol: binance_position.symbol.clone(),
                    side: if position_amt > 0.0 { "BUY".to_string() } else { "SELL".to_string() },
                    size: position_amt.abs(),
                    entry_price: binance_position.entry_price.parse().unwrap_or(0.0),
                    current_price: binance_position.mark_price.parse().unwrap_or(0.0),
                    unrealized_pnl: binance_position.unrealized_pnl.parse().unwrap_or(0.0),
                    stop_loss: None,
                    take_profit: None,
                    timestamp: chrono::Utc::now().timestamp_millis(),
                };

                self.position_manager.add_position(position);
                active_positions += 1;
                
                info!("Synced position: {} {} {}", 
                      binance_position.symbol, 
                      if position_amt > 0.0 { "LONG" } else { "SHORT" },
                      position_amt.abs());
            }
        }

        info!("Synced {} active positions", active_positions);
        Ok(())
    }

    fn start_trading_loop(&self) -> tokio::task::JoinHandle<Result<()>> {
        let market_data = self.market_data.clone();
        let position_manager = self.position_manager.clone();
        let risk_manager = self.risk_manager.clone();
        let client = self.client.clone();
        let storage = self.storage.clone();
        let symbols = self.market_data.get_supported_symbols();

        tokio::spawn(async move {
            // Check for trading opportunities every minute
            let mut interval = interval(Duration::from_secs(60));

            loop {
                interval.tick().await;

                for symbol in &symbols {
                    if let Err(e) = Self::process_trading_opportunity(
                        &market_data,
                        &position_manager,
                        &risk_manager,
                        &client,
                        &storage,
                        symbol,
                    ).await {
                        error!("Error processing trading opportunity for {}: {}", symbol, e);
                    }
                }
            }
        })
    }

    async fn process_trading_opportunity(
        market_data: &MarketDataProcessor,
        position_manager: &PositionManager,
        risk_manager: &RiskManager,
        client: &BinanceClient,
        storage: &Storage,
        symbol: &str,
    ) -> Result<()> {
        // Check if we already have a position for this symbol
        if position_manager.has_position(symbol) {
            debug!("Already have position for {}, skipping", symbol);
            return Ok(());
        }

        // Get latest analysis
        let analysis = match market_data.get_latest_analysis(symbol).await {
            Ok(analysis) => analysis,
            Err(e) => {
                debug!("No analysis available for {}: {}", symbol, e);
                return Ok(());
            }
        };

        // Check if signal is strong enough and has good confidence
        let should_trade = match analysis.overall_signal {
            TradingSignal::StrongBuy | TradingSignal::StrongSell => analysis.overall_confidence >= 0.7,
            TradingSignal::Buy | TradingSignal::Sell => analysis.overall_confidence >= 0.8,
            TradingSignal::Hold => false,
        };

        if !should_trade {
            debug!("Signal not strong enough for {}: {:?} (confidence: {:.2})", 
                   symbol, analysis.overall_signal, analysis.overall_confidence);
            return Ok(());
        }

        // Risk check
        if !risk_manager.can_open_position(symbol, &analysis).await? {
            debug!("Risk manager rejected trade for {}", symbol);
            return Ok(());
        }

        // Execute trade
        match Self::execute_trade(client, storage, symbol, &analysis).await {
            Ok(trade_record) => {
                info!("Executed trade: {} {} {} @ {}", 
                      trade_record.symbol, trade_record.side, 
                      trade_record.quantity, trade_record.entry_price);
                
                // Create position record
                let position = Position {
                    id: Uuid::new_v4().to_string(),
                    symbol: symbol.to_string(),
                    side: trade_record.side.clone(),
                    size: trade_record.quantity,
                    entry_price: trade_record.entry_price,
                    current_price: trade_record.entry_price,
                    unrealized_pnl: 0.0,
                    stop_loss: trade_record.stop_loss,
                    take_profit: trade_record.take_profit,
                    timestamp: trade_record.entry_time,
                };

                position_manager.add_position(position);
            }
            Err(e) => {
                error!("Failed to execute trade for {}: {}", symbol, e);
            }
        }

        Ok(())
    }

    async fn execute_trade(
        client: &BinanceClient,
        storage: &Storage,
        symbol: &str,
        analysis: &crate::market_data::analyzer::MultiTimeframeAnalysis,
    ) -> Result<TradeRecord> {
        let side = match analysis.overall_signal {
            TradingSignal::Buy | TradingSignal::StrongBuy => "BUY",
            TradingSignal::Sell | TradingSignal::StrongSell => "SELL",
            _ => return Err(anyhow::anyhow!("Invalid signal for trading")),
        };

        // Calculate position size (for now, use fixed quantity)
        let quantity = 0.01; // This should be calculated based on risk management

        let order_request = NewOrderRequest {
            symbol: symbol.to_string(),
            side: side.to_string(),
            r#type: "MARKET".to_string(),
            quantity: Some(quantity.to_string()),
            quote_order_qty: None,
            price: None,
            new_client_order_id: Some(Uuid::new_v4().to_string()),
            stop_price: None,
            iceberg_qty: None,
            new_order_resp_type: Some("RESULT".to_string()),
            time_in_force: None,
            reduce_only: Some(false),
            close_position: Some(false),
            position_side: Some("BOTH".to_string()),
            working_type: None,
            price_protect: Some(false),
        };

        let order_response = client.place_futures_order(order_request).await?;

        let entry_price: f64 = order_response.price.parse().unwrap_or(0.0);
        let executed_qty: f64 = order_response.executed_qty.parse().unwrap_or(0.0);

        let trade_record = TradeRecord {
            id: None,
            symbol: symbol.to_string(),
            side: side.to_string(),
            quantity: executed_qty,
            entry_price,
            exit_price: None,
            stop_loss: analysis.stop_loss,
            take_profit: analysis.take_profit,
            entry_time: chrono::Utc::now().timestamp_millis(),
            exit_time: None,
            pnl: None,
            status: "open".to_string(),
            strategy_used: Some("multi_timeframe_analysis".to_string()),
        };

        storage.store_trade_record(&trade_record).await?;

        Ok(trade_record)
    }

    fn start_position_monitoring(&self) -> tokio::task::JoinHandle<Result<()>> {
        let position_manager = self.position_manager.clone();
        let client = self.client.clone();
        let storage = self.storage.clone();
        let market_data = self.market_data.clone();
        let check_interval = Duration::from_secs(self.config.position_check_interval_seconds);

        tokio::spawn(async move {
            let mut interval = interval(check_interval);

            loop {
                interval.tick().await;

                let positions = position_manager.get_all_positions();
                for position in positions {
                    if let Err(e) = Self::monitor_position(
                        &position_manager,
                        &client,
                        &storage,
                        &market_data,
                        &position,
                    ).await {
                        error!("Error monitoring position {}: {}", position.symbol, e);
                    }
                }
            }
        })
    }

    async fn monitor_position(
        position_manager: &PositionManager,
        client: &BinanceClient,
        storage: &Storage,
        market_data: &MarketDataProcessor,
        position: &Position,
    ) -> Result<()> {
        // Get current market price
        let current_price = market_data
            .get_cache()
            .get_latest_price(&position.symbol)
            .unwrap_or(position.current_price);

        // Update position with current price
        let mut updated_position = position.clone();
        updated_position.current_price = current_price;
        
        // Calculate unrealized PnL
        let price_diff = if position.side == "BUY" {
            current_price - position.entry_price
        } else {
            position.entry_price - current_price
        };
        updated_position.unrealized_pnl = price_diff * position.size;

        // Check for stop loss or take profit
        let should_close = if let Some(stop_loss) = position.stop_loss {
            if position.side == "BUY" && current_price <= stop_loss {
                true
            } else if position.side == "SELL" && current_price >= stop_loss {
                true
            } else {
                false
            }
        } else {
            false
        } || if let Some(take_profit) = position.take_profit {
            if position.side == "BUY" && current_price >= take_profit {
                true
            } else if position.side == "SELL" && current_price <= take_profit {
                true
            } else {
                false
            }
        } else {
            false
        };

        if should_close {
            info!("Closing position {} due to stop loss/take profit", position.symbol);
            
            match Self::close_position(client, storage, position).await {
                Ok(_) => {
                    position_manager.remove_position(&position.id);
                    info!("Successfully closed position {}", position.symbol);
                }
                Err(e) => {
                    error!("Failed to close position {}: {}", position.symbol, e);
                }
            }
        } else {
            // Update position in manager
            position_manager.update_position(updated_position);
        }

        Ok(())
    }

    async fn close_position(
        client: &BinanceClient,
        storage: &Storage,
        position: &Position,
    ) -> Result<()> {
        let close_side = if position.side == "BUY" { "SELL" } else { "BUY" };

        let order_request = NewOrderRequest {
            symbol: position.symbol.clone(),
            side: close_side.to_string(),
            r#type: "MARKET".to_string(),
            quantity: Some(position.size.to_string()),
            quote_order_qty: None,
            price: None,
            new_client_order_id: Some(Uuid::new_v4().to_string()),
            stop_price: None,
            iceberg_qty: None,
            new_order_resp_type: Some("RESULT".to_string()),
            time_in_force: None,
            reduce_only: Some(true),
            close_position: Some(false),
            position_side: Some("BOTH".to_string()),
            working_type: None,
            price_protect: Some(false),
        };

        let order_response = client.place_futures_order(order_request).await?;
        let exit_price: f64 = order_response.price.parse().unwrap_or(0.0);

        // Update trade record
        let mut trade_record = TradeRecord {
            id: None,
            symbol: position.symbol.clone(),
            side: position.side.clone(),
            quantity: position.size,
            entry_price: position.entry_price,
            exit_price: Some(exit_price),
            stop_loss: position.stop_loss,
            take_profit: position.take_profit,
            entry_time: position.timestamp,
            exit_time: Some(chrono::Utc::now().timestamp_millis()),
            pnl: Some(position.unrealized_pnl),
            status: "closed".to_string(),
            strategy_used: Some("multi_timeframe_analysis".to_string()),
        };

        storage.store_trade_record(&trade_record).await?;

        Ok(())
    }

    // Public API methods
    pub fn get_positions(&self) -> Vec<Position> {
        self.position_manager.get_all_positions()
    }

    pub async fn get_account_info(&self) -> Result<serde_json::Value> {
        self.client.get_futures_account().await
    }

    pub async fn force_close_position(&self, symbol: &str) -> Result<()> {
        if let Some(position) = self.position_manager.get_position(symbol) {
            Self::close_position(&self.client, &self.storage, &position).await?;
            self.position_manager.remove_position(&position.id);
            info!("Force closed position for {}", symbol);
        }
        Ok(())
    }

    pub async fn get_performance_stats(&self) -> Result<crate::storage::PerformanceStats> {
        self.storage.get_performance_stats().await
    }
} 