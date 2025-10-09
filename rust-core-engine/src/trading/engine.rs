use anyhow::Result;
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
    #[allow(dead_code)]
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
        tokio::try_join!(async { trading_handle.await? }, async {
            monitoring_handle.await?
        })?;

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
                    side: if position_amt > 0.0 {
                        "BUY".to_string()
                    } else {
                        "SELL".to_string()
                    },
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

                info!(
                    "Synced position: {} {} {}",
                    binance_position.symbol,
                    if position_amt > 0.0 { "LONG" } else { "SHORT" },
                    position_amt.abs()
                );
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
                    )
                    .await
                    {
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
            debug!("Already have position for {symbol}, skipping");
            return Ok(());
        }

        // Get latest analysis
        let analysis = match market_data.get_latest_analysis(symbol).await {
            Ok(analysis) => analysis,
            Err(e) => {
                debug!("No analysis available for {}: {}", symbol, e);
                return Ok(());
            },
        };

        // Check if signal is strong enough and has good confidence
        let should_trade = match analysis.overall_signal {
            TradingSignal::StrongBuy | TradingSignal::StrongSell => {
                analysis.overall_confidence >= 0.7
            },
            TradingSignal::Buy | TradingSignal::Sell => analysis.overall_confidence >= 0.8,
            TradingSignal::Hold => false,
        };

        if !should_trade {
            debug!(
                "Signal not strong enough for {}: {:?} (confidence: {:.2})",
                symbol, analysis.overall_signal, analysis.overall_confidence
            );
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
                info!(
                    "Executed trade: {} {} {} @ {}",
                    trade_record.symbol,
                    trade_record.side,
                    trade_record.quantity,
                    trade_record.entry_price
                );

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
            },
            Err(e) => {
                error!("Failed to execute trade for {}: {}", symbol, e);
            },
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
                    )
                    .await
                    {
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
            (position.side == "BUY" && current_price <= stop_loss)
                || (position.side == "SELL" && current_price >= stop_loss)
        } else {
            false
        } || if let Some(take_profit) = position.take_profit {
            (position.side == "BUY" && current_price >= take_profit)
                || (position.side == "SELL" && current_price <= take_profit)
        } else {
            false
        };

        if should_close {
            info!(
                "Closing position {} due to stop loss/take profit",
                position.symbol
            );

            match Self::close_position(client, storage, position).await {
                Ok(_) => {
                    position_manager.remove_position(&position.id);
                    info!("Successfully closed position {}", position.symbol);
                },
                Err(e) => {
                    error!("Failed to close position {}: {}", position.symbol, e);
                },
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
        let close_side = if position.side == "BUY" {
            "SELL"
        } else {
            "BUY"
        };

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
        let trade_record = TradeRecord {
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

#[cfg(test)]
mod tests {

    // Helper function to calculate unrealized PnL for BUY positions
    fn calculate_buy_pnl(size: f64, entry_price: f64, current_price: f64) -> f64 {
        (current_price - entry_price) * size
    }

    // Helper function to calculate unrealized PnL for SELL positions
    fn calculate_sell_pnl(size: f64, entry_price: f64, current_price: f64) -> f64 {
        (entry_price - current_price) * size
    }

    // Helper function to determine if stop loss is hit for BUY position
    fn is_buy_stop_loss_hit(current_price: f64, stop_loss: Option<f64>) -> bool {
        if let Some(sl) = stop_loss {
            current_price <= sl
        } else {
            false
        }
    }

    // Helper function to determine if stop loss is hit for SELL position
    fn is_sell_stop_loss_hit(current_price: f64, stop_loss: Option<f64>) -> bool {
        if let Some(sl) = stop_loss {
            current_price >= sl
        } else {
            false
        }
    }

    // Helper function to determine if take profit is hit for BUY position
    fn is_buy_take_profit_hit(current_price: f64, take_profit: Option<f64>) -> bool {
        if let Some(tp) = take_profit {
            current_price >= tp
        } else {
            false
        }
    }

    // Helper function to determine if take profit is hit for SELL position
    fn is_sell_take_profit_hit(current_price: f64, take_profit: Option<f64>) -> bool {
        if let Some(tp) = take_profit {
            current_price <= tp
        } else {
            false
        }
    }

    #[test]
    fn test_calculate_buy_pnl_positive() {
        // Buy at 50000, current price 51000, size 0.1
        let pnl = calculate_buy_pnl(0.1, 50000.0, 51000.0);
        assert_eq!(pnl, 100.0); // (51000 - 50000) * 0.1 = 100
    }

    #[test]
    fn test_calculate_buy_pnl_negative() {
        // Buy at 50000, current price 49000, size 0.1
        let pnl = calculate_buy_pnl(0.1, 50000.0, 49000.0);
        assert_eq!(pnl, -100.0); // (49000 - 50000) * 0.1 = -100
    }

    #[test]
    fn test_calculate_buy_pnl_zero() {
        // Buy at 50000, current price 50000, size 0.1
        let pnl = calculate_buy_pnl(0.1, 50000.0, 50000.0);
        assert_eq!(pnl, 0.0);
    }

    #[test]
    fn test_calculate_buy_pnl_zero_size() {
        let pnl = calculate_buy_pnl(0.0, 50000.0, 51000.0);
        assert_eq!(pnl, 0.0);
    }

    #[test]
    fn test_calculate_buy_pnl_large_size() {
        // Buy at 50000, current price 51000, size 100
        let pnl = calculate_buy_pnl(100.0, 50000.0, 51000.0);
        assert_eq!(pnl, 100000.0);
    }

    #[test]
    fn test_calculate_sell_pnl_positive() {
        // Sell at 50000, current price 49000, size 0.1
        let pnl = calculate_sell_pnl(0.1, 50000.0, 49000.0);
        assert_eq!(pnl, 100.0); // (50000 - 49000) * 0.1 = 100
    }

    #[test]
    fn test_calculate_sell_pnl_negative() {
        // Sell at 50000, current price 51000, size 0.1
        let pnl = calculate_sell_pnl(0.1, 50000.0, 51000.0);
        assert_eq!(pnl, -100.0); // (50000 - 51000) * 0.1 = -100
    }

    #[test]
    fn test_calculate_sell_pnl_zero() {
        // Sell at 50000, current price 50000, size 0.1
        let pnl = calculate_sell_pnl(0.1, 50000.0, 50000.0);
        assert_eq!(pnl, 0.0);
    }

    #[test]
    fn test_calculate_sell_pnl_zero_size() {
        let pnl = calculate_sell_pnl(0.0, 50000.0, 49000.0);
        assert_eq!(pnl, 0.0);
    }

    #[test]
    fn test_calculate_sell_pnl_large_size() {
        // Sell at 50000, current price 49000, size 100
        let pnl = calculate_sell_pnl(100.0, 50000.0, 49000.0);
        assert_eq!(pnl, 100000.0);
    }

    #[test]
    fn test_is_buy_stop_loss_hit_true() {
        // Current price 49000, stop loss 49500
        assert!(is_buy_stop_loss_hit(49000.0, Some(49500.0)));
    }

    #[test]
    fn test_is_buy_stop_loss_hit_exactly() {
        // Current price equals stop loss
        assert!(is_buy_stop_loss_hit(49000.0, Some(49000.0)));
    }

    #[test]
    fn test_is_buy_stop_loss_hit_false() {
        // Current price 51000, stop loss 49500
        assert!(!is_buy_stop_loss_hit(51000.0, Some(49500.0)));
    }

    #[test]
    fn test_is_buy_stop_loss_hit_no_stop_loss() {
        // No stop loss set
        assert!(!is_buy_stop_loss_hit(49000.0, None));
    }

    #[test]
    fn test_is_sell_stop_loss_hit_true() {
        // Current price 51000, stop loss 50500
        assert!(is_sell_stop_loss_hit(51000.0, Some(50500.0)));
    }

    #[test]
    fn test_is_sell_stop_loss_hit_exactly() {
        // Current price equals stop loss
        assert!(is_sell_stop_loss_hit(51000.0, Some(51000.0)));
    }

    #[test]
    fn test_is_sell_stop_loss_hit_false() {
        // Current price 49000, stop loss 50500
        assert!(!is_sell_stop_loss_hit(49000.0, Some(50500.0)));
    }

    #[test]
    fn test_is_sell_stop_loss_hit_no_stop_loss() {
        // No stop loss set
        assert!(!is_sell_stop_loss_hit(51000.0, None));
    }

    #[test]
    fn test_is_buy_take_profit_hit_true() {
        // Current price 55000, take profit 54000
        assert!(is_buy_take_profit_hit(55000.0, Some(54000.0)));
    }

    #[test]
    fn test_is_buy_take_profit_hit_exactly() {
        // Current price equals take profit
        assert!(is_buy_take_profit_hit(55000.0, Some(55000.0)));
    }

    #[test]
    fn test_is_buy_take_profit_hit_false() {
        // Current price 52000, take profit 54000
        assert!(!is_buy_take_profit_hit(52000.0, Some(54000.0)));
    }

    #[test]
    fn test_is_buy_take_profit_hit_no_take_profit() {
        // No take profit set
        assert!(!is_buy_take_profit_hit(55000.0, None));
    }

    #[test]
    fn test_is_sell_take_profit_hit_true() {
        // Current price 45000, take profit 46000
        assert!(is_sell_take_profit_hit(45000.0, Some(46000.0)));
    }

    #[test]
    fn test_is_sell_take_profit_hit_exactly() {
        // Current price equals take profit
        assert!(is_sell_take_profit_hit(46000.0, Some(46000.0)));
    }

    #[test]
    fn test_is_sell_take_profit_hit_false() {
        // Current price 48000, take profit 46000
        assert!(!is_sell_take_profit_hit(48000.0, Some(46000.0)));
    }

    #[test]
    fn test_is_sell_take_profit_hit_no_take_profit() {
        // No take profit set
        assert!(!is_sell_take_profit_hit(45000.0, None));
    }

    #[test]
    fn test_pnl_calculation_extreme_price_movement() {
        // Extreme upward movement
        let pnl = calculate_buy_pnl(1.0, 10000.0, 100000.0);
        assert_eq!(pnl, 90000.0);

        // Extreme downward movement
        let pnl = calculate_buy_pnl(1.0, 100000.0, 10000.0);
        assert_eq!(pnl, -90000.0);
    }

    #[test]
    fn test_pnl_calculation_very_small_price_movement() {
        // Very small price movement
        let pnl = calculate_buy_pnl(1.0, 50000.0, 50000.01);
        assert!((pnl - 0.01).abs() < 0.0001);
    }

    #[test]
    fn test_pnl_calculation_very_small_size() {
        // Very small position size
        let pnl = calculate_buy_pnl(0.0001, 50000.0, 51000.0);
        assert!((pnl - 0.1).abs() < 0.0001);
    }

    #[test]
    fn test_combined_buy_scenario_profit() {
        // Scenario: BUY position in profit
        let entry_price = 50000.0;
        let current_price = 52000.0;
        let size = 0.5;
        let stop_loss = Some(49000.0);
        let take_profit = Some(55000.0);

        let pnl = calculate_buy_pnl(size, entry_price, current_price);
        assert_eq!(pnl, 1000.0);

        assert!(!is_buy_stop_loss_hit(current_price, stop_loss));
        assert!(!is_buy_take_profit_hit(current_price, take_profit));
    }

    #[test]
    fn test_combined_buy_scenario_stop_loss() {
        // Scenario: BUY position hits stop loss
        let entry_price = 50000.0;
        let current_price = 48500.0;
        let size = 0.5;
        let stop_loss = Some(49000.0);
        let take_profit = Some(55000.0);

        let pnl = calculate_buy_pnl(size, entry_price, current_price);
        assert_eq!(pnl, -750.0);

        assert!(is_buy_stop_loss_hit(current_price, stop_loss));
        assert!(!is_buy_take_profit_hit(current_price, take_profit));
    }

    #[test]
    fn test_combined_buy_scenario_take_profit() {
        // Scenario: BUY position hits take profit
        let entry_price = 50000.0;
        let current_price = 55500.0;
        let size = 0.5;
        let stop_loss = Some(49000.0);
        let take_profit = Some(55000.0);

        let pnl = calculate_buy_pnl(size, entry_price, current_price);
        assert_eq!(pnl, 2750.0);

        assert!(!is_buy_stop_loss_hit(current_price, stop_loss));
        assert!(is_buy_take_profit_hit(current_price, take_profit));
    }

    #[test]
    fn test_combined_sell_scenario_profit() {
        // Scenario: SELL position in profit
        let entry_price = 50000.0;
        let current_price = 48000.0;
        let size = 0.5;
        let stop_loss = Some(51000.0);
        let take_profit = Some(45000.0);

        let pnl = calculate_sell_pnl(size, entry_price, current_price);
        assert_eq!(pnl, 1000.0);

        assert!(!is_sell_stop_loss_hit(current_price, stop_loss));
        assert!(!is_sell_take_profit_hit(current_price, take_profit));
    }

    #[test]
    fn test_combined_sell_scenario_stop_loss() {
        // Scenario: SELL position hits stop loss
        let entry_price = 50000.0;
        let current_price = 51500.0;
        let size = 0.5;
        let stop_loss = Some(51000.0);
        let take_profit = Some(45000.0);

        let pnl = calculate_sell_pnl(size, entry_price, current_price);
        assert_eq!(pnl, -750.0);

        assert!(is_sell_stop_loss_hit(current_price, stop_loss));
        assert!(!is_sell_take_profit_hit(current_price, take_profit));
    }

    #[test]
    fn test_combined_sell_scenario_take_profit() {
        // Scenario: SELL position hits take profit
        let entry_price = 50000.0;
        let current_price = 44500.0;
        let size = 0.5;
        let stop_loss = Some(51000.0);
        let take_profit = Some(45000.0);

        let pnl = calculate_sell_pnl(size, entry_price, current_price);
        assert_eq!(pnl, 2750.0);

        assert!(!is_sell_stop_loss_hit(current_price, stop_loss));
        assert!(is_sell_take_profit_hit(current_price, take_profit));
    }

    #[test]
    fn test_zero_price_edge_case() {
        // Test with zero current price
        let pnl = calculate_buy_pnl(1.0, 50000.0, 0.0);
        assert_eq!(pnl, -50000.0);
    }

    #[test]
    fn test_zero_entry_price_edge_case() {
        // Test with zero entry price (unrealistic but testing edge case)
        let pnl = calculate_buy_pnl(1.0, 0.0, 50000.0);
        assert_eq!(pnl, 50000.0);
    }

    #[test]
    fn test_negative_pnl_calculations() {
        // Ensure negative PnL is calculated correctly for various scenarios
        let scenarios = vec![
            (0.1, 50000.0, 45000.0, -500.0), // BUY loss
            (1.0, 3000.0, 2500.0, -500.0),   // BUY loss
            (5.0, 300.0, 250.0, -250.0),     // BUY loss
        ];

        for (size, entry, current, expected) in scenarios {
            let pnl = calculate_buy_pnl(size, entry, current);
            assert_eq!(pnl, expected);
        }
    }

    #[test]
    fn test_multiple_precision_levels() {
        // Test with different precision levels
        let pnl1 = calculate_buy_pnl(0.123456, 50000.0, 51000.0);
        assert!((pnl1 - 123.456).abs() < 0.001);

        let pnl2 = calculate_sell_pnl(0.987654, 3000.0, 2900.0);
        assert!((pnl2 - 98.7654).abs() < 0.0001);
    }

    #[test]
    fn test_stop_loss_and_take_profit_boundaries() {
        // Test exact boundary conditions
        let current = 50000.0;

        // Stop loss boundaries for BUY
        assert!(is_buy_stop_loss_hit(current, Some(50000.0)));
        assert!(is_buy_stop_loss_hit(current, Some(50001.0)));
        assert!(!is_buy_stop_loss_hit(current, Some(49999.0)));

        // Take profit boundaries for BUY
        assert!(is_buy_take_profit_hit(current, Some(50000.0)));
        assert!(is_buy_take_profit_hit(current, Some(49999.0)));
        assert!(!is_buy_take_profit_hit(current, Some(50001.0)));

        // Stop loss boundaries for SELL
        assert!(is_sell_stop_loss_hit(current, Some(50000.0)));
        assert!(is_sell_stop_loss_hit(current, Some(49999.0)));
        assert!(!is_sell_stop_loss_hit(current, Some(50001.0)));

        // Take profit boundaries for SELL
        assert!(is_sell_take_profit_hit(current, Some(50000.0)));
        assert!(is_sell_take_profit_hit(current, Some(50001.0)));
        assert!(!is_sell_take_profit_hit(current, Some(49999.0)));
    }

    #[test]
    fn test_fractional_pnl_buy() {
        // Test fractional position sizes and prices
        // (45678.90 - 45123.45) * 0.333 = 555.45 * 0.333 = 184.96485
        let pnl = calculate_buy_pnl(0.333, 45123.45, 45678.90);
        assert!((pnl - 184.96485).abs() < 0.001);
    }

    #[test]
    fn test_fractional_pnl_sell() {
        // Test fractional position sizes and prices
        // (12345.67 - 12000.00) * 0.777 = 345.67 * 0.777 = 268.585590
        let pnl = calculate_sell_pnl(0.777, 12345.67, 12000.00);
        assert!((pnl - 268.58559).abs() < 0.001);
    }

    #[test]
    fn test_very_large_position_size() {
        // Test with very large position size
        let pnl = calculate_buy_pnl(1000.0, 50000.0, 51000.0);
        assert_eq!(pnl, 1000000.0);
    }

    #[test]
    fn test_very_large_price() {
        // Test with very large prices (e.g., BTC in distant future)
        let pnl = calculate_buy_pnl(0.01, 500000.0, 510000.0);
        assert_eq!(pnl, 100.0);
    }

    #[test]
    fn test_very_small_prices() {
        // Test with very small prices (altcoins)
        let pnl = calculate_buy_pnl(10000.0, 0.001, 0.0015);
        assert!((pnl - 5.0).abs() < 0.0001);
    }

    #[test]
    fn test_micro_price_movement_buy() {
        // Test with extremely small price movements
        let pnl = calculate_buy_pnl(100.0, 50000.0, 50000.001);
        assert!((pnl - 0.1).abs() < 0.0001);
    }

    #[test]
    fn test_micro_price_movement_sell() {
        // Test with extremely small price movements
        let pnl = calculate_sell_pnl(100.0, 50000.0, 49999.999);
        assert!((pnl - 0.1).abs() < 0.0001);
    }

    #[test]
    fn test_high_leverage_simulation_buy() {
        // Simulate high leverage (10x) with 1% price movement
        let size = 10.0; // 10x leverage effect
        let entry = 50000.0;
        let current = 50500.0; // 1% increase
        let pnl = calculate_buy_pnl(size, entry, current);
        assert_eq!(pnl, 5000.0); // 10% gain with 10x leverage
    }

    #[test]
    fn test_high_leverage_simulation_sell() {
        // Simulate high leverage (10x) with 1% price movement
        let size = 10.0; // 10x leverage effect
        let entry = 50000.0;
        let current = 49500.0; // 1% decrease
        let pnl = calculate_sell_pnl(size, entry, current);
        assert_eq!(pnl, 5000.0); // 10% gain with 10x leverage
    }

    #[test]
    fn test_multiple_buy_scenarios_batch() {
        // Test multiple scenarios in one test
        let scenarios = vec![
            (1.0, 30000.0, 31000.0, 1000.0),
            (0.5, 40000.0, 42000.0, 1000.0),
            (2.0, 25000.0, 26000.0, 2000.0),
            (0.1, 60000.0, 65000.0, 500.0),
        ];

        for (size, entry, current, expected) in scenarios {
            let pnl = calculate_buy_pnl(size, entry, current);
            assert_eq!(pnl, expected);
        }
    }

    #[test]
    fn test_multiple_sell_scenarios_batch() {
        // Test multiple scenarios in one test
        let scenarios = vec![
            (1.0, 31000.0, 30000.0, 1000.0),
            (0.5, 42000.0, 40000.0, 1000.0),
            (2.0, 26000.0, 25000.0, 2000.0),
            (0.1, 65000.0, 60000.0, 500.0),
        ];

        for (size, entry, current, expected) in scenarios {
            let pnl = calculate_sell_pnl(size, entry, current);
            assert_eq!(pnl, expected);
        }
    }

    #[test]
    fn test_symmetry_buy_sell_pnl() {
        // Verify that buy profit equals sell profit for opposite movements
        let size = 1.0;
        let entry = 50000.0;
        let price_diff = 1000.0;

        let buy_profit = calculate_buy_pnl(size, entry, entry + price_diff);
        let sell_profit = calculate_sell_pnl(size, entry, entry - price_diff);

        assert_eq!(buy_profit, sell_profit);
    }

    #[test]
    fn test_stop_loss_near_entry_buy() {
        // Stop loss very close to entry price
        let current = 50000.0;
        let stop_loss = Some(49999.0);
        assert!(!is_buy_stop_loss_hit(current, stop_loss));

        let current = 49998.9;
        assert!(is_buy_stop_loss_hit(current, stop_loss));
    }

    #[test]
    fn test_stop_loss_near_entry_sell() {
        // Stop loss very close to entry price
        let current = 50000.0;
        let stop_loss = Some(50001.0);
        assert!(!is_sell_stop_loss_hit(current, stop_loss));

        let current = 50001.1;
        assert!(is_sell_stop_loss_hit(current, stop_loss));
    }

    #[test]
    fn test_take_profit_far_from_entry_buy() {
        // Take profit very far from entry
        let current = 51000.0;
        let take_profit = Some(100000.0);
        assert!(!is_buy_take_profit_hit(current, take_profit));

        let current = 100000.1;
        assert!(is_buy_take_profit_hit(current, take_profit));
    }

    #[test]
    fn test_take_profit_far_from_entry_sell() {
        // Take profit very far from entry
        let current = 49000.0;
        let take_profit = Some(10000.0);
        assert!(!is_sell_take_profit_hit(current, take_profit));

        let current = 9999.9;
        assert!(is_sell_take_profit_hit(current, take_profit));
    }

    #[test]
    fn test_multiple_stop_loss_checks_buy() {
        // Multiple price points around stop loss
        let stop_loss = Some(49000.0);

        assert!(is_buy_stop_loss_hit(48000.0, stop_loss));
        assert!(is_buy_stop_loss_hit(48999.0, stop_loss));
        assert!(is_buy_stop_loss_hit(49000.0, stop_loss));
        assert!(!is_buy_stop_loss_hit(49001.0, stop_loss));
        assert!(!is_buy_stop_loss_hit(50000.0, stop_loss));
    }

    #[test]
    fn test_multiple_stop_loss_checks_sell() {
        // Multiple price points around stop loss
        let stop_loss = Some(51000.0);

        assert!(!is_sell_stop_loss_hit(50000.0, stop_loss));
        assert!(!is_sell_stop_loss_hit(50999.0, stop_loss));
        assert!(is_sell_stop_loss_hit(51000.0, stop_loss));
        assert!(is_sell_stop_loss_hit(51001.0, stop_loss));
        assert!(is_sell_stop_loss_hit(52000.0, stop_loss));
    }

    #[test]
    fn test_multiple_take_profit_checks_buy() {
        // Multiple price points around take profit
        let take_profit = Some(55000.0);

        assert!(!is_buy_take_profit_hit(54000.0, take_profit));
        assert!(!is_buy_take_profit_hit(54999.0, take_profit));
        assert!(is_buy_take_profit_hit(55000.0, take_profit));
        assert!(is_buy_take_profit_hit(55001.0, take_profit));
        assert!(is_buy_take_profit_hit(56000.0, take_profit));
    }

    #[test]
    fn test_multiple_take_profit_checks_sell() {
        // Multiple price points around take profit
        let take_profit = Some(45000.0);

        assert!(is_sell_take_profit_hit(44000.0, take_profit));
        assert!(is_sell_take_profit_hit(44999.0, take_profit));
        assert!(is_sell_take_profit_hit(45000.0, take_profit));
        assert!(!is_sell_take_profit_hit(45001.0, take_profit));
        assert!(!is_sell_take_profit_hit(46000.0, take_profit));
    }

    #[test]
    fn test_realistic_btc_long_scenario() {
        // Realistic BTC long trade scenario
        let size = 0.1; // 0.1 BTC
        let entry = 43250.0;
        let current = 44100.0;
        let stop_loss = Some(42800.0);
        let take_profit = Some(45000.0);

        let pnl = calculate_buy_pnl(size, entry, current);
        assert_eq!(pnl, 85.0);
        assert!(!is_buy_stop_loss_hit(current, stop_loss));
        assert!(!is_buy_take_profit_hit(current, take_profit));
    }

    #[test]
    fn test_realistic_btc_short_scenario() {
        // Realistic BTC short trade scenario
        let size = 0.05; // 0.05 BTC
        let entry = 44500.0;
        let current = 43800.0;
        let stop_loss = Some(45200.0);
        let take_profit = Some(42000.0);

        let pnl = calculate_sell_pnl(size, entry, current);
        assert_eq!(pnl, 35.0);
        assert!(!is_sell_stop_loss_hit(current, stop_loss));
        assert!(!is_sell_take_profit_hit(current, take_profit));
    }

    #[test]
    fn test_realistic_eth_long_scenario() {
        // Realistic ETH long trade scenario
        let size = 2.5; // 2.5 ETH
        let entry = 2300.0;
        let current = 2450.0;
        let stop_loss = Some(2200.0);
        let take_profit = Some(2600.0);

        let pnl = calculate_buy_pnl(size, entry, current);
        assert_eq!(pnl, 375.0);
        assert!(!is_buy_stop_loss_hit(current, stop_loss));
        assert!(!is_buy_take_profit_hit(current, take_profit));
    }

    #[test]
    fn test_realistic_eth_short_scenario() {
        // Realistic ETH short trade scenario
        let size = 1.0; // 1 ETH
        let entry = 2400.0;
        let current = 2250.0;
        let stop_loss = Some(2500.0);
        let take_profit = Some(2100.0);

        let pnl = calculate_sell_pnl(size, entry, current);
        assert_eq!(pnl, 150.0);
        assert!(!is_sell_stop_loss_hit(current, stop_loss));
        assert!(!is_sell_take_profit_hit(current, take_profit));
    }

    #[test]
    fn test_altcoin_high_volatility_buy() {
        // Altcoin with high volatility (10% move)
        let size = 1000.0;
        let entry = 1.50;
        let current = 1.65; // 10% increase
        let pnl = calculate_buy_pnl(size, entry, current);
        assert!((pnl - 150.0).abs() < 0.1);
    }

    #[test]
    fn test_altcoin_high_volatility_sell() {
        // Altcoin with high volatility (15% move)
        let size = 500.0;
        let entry = 0.80;
        let current = 0.68; // 15% decrease
        let pnl = calculate_sell_pnl(size, entry, current);
        assert!((pnl - 60.0).abs() < 0.1);
    }

    #[test]
    fn test_pnl_with_max_float_precision() {
        // Test maximum float precision
        let size = 0.123456789;
        let entry = 12345.6789;
        let current = 12346.6789;
        let pnl = calculate_buy_pnl(size, entry, current);
        assert!((pnl - 0.123456789).abs() < 0.000001);
    }

    #[test]
    fn test_zero_all_values() {
        // All zeros edge case
        let pnl = calculate_buy_pnl(0.0, 0.0, 0.0);
        assert_eq!(pnl, 0.0);
    }

    #[test]
    fn test_combined_scenario_no_sl_tp_buy() {
        // BUY position with no stop loss or take profit
        let entry = 50000.0;
        let current = 52000.0;
        let size = 1.0;

        let pnl = calculate_buy_pnl(size, entry, current);
        assert_eq!(pnl, 2000.0);

        assert!(!is_buy_stop_loss_hit(current, None));
        assert!(!is_buy_take_profit_hit(current, None));
    }

    #[test]
    fn test_combined_scenario_no_sl_tp_sell() {
        // SELL position with no stop loss or take profit
        let entry = 50000.0;
        let current = 48000.0;
        let size = 1.0;

        let pnl = calculate_sell_pnl(size, entry, current);
        assert_eq!(pnl, 2000.0);

        assert!(!is_sell_stop_loss_hit(current, None));
        assert!(!is_sell_take_profit_hit(current, None));
    }

    #[test]
    fn test_break_even_buy_position() {
        // Position at break even
        let size = 1.0;
        let entry = 50000.0;
        let current = 50000.0;

        let pnl = calculate_buy_pnl(size, entry, current);
        assert_eq!(pnl, 0.0);
    }

    #[test]
    fn test_break_even_sell_position() {
        // Position at break even
        let size = 1.0;
        let entry = 50000.0;
        let current = 50000.0;

        let pnl = calculate_sell_pnl(size, entry, current);
        assert_eq!(pnl, 0.0);
    }

    #[test]
    fn test_extreme_loss_buy() {
        // 90% loss scenario
        let size = 1.0;
        let entry = 50000.0;
        let current = 5000.0;

        let pnl = calculate_buy_pnl(size, entry, current);
        assert_eq!(pnl, -45000.0);
    }

    #[test]
    fn test_extreme_loss_sell() {
        // 90% loss scenario (price doubles from entry)
        let size = 1.0;
        let entry = 50000.0;
        let current = 95000.0;

        let pnl = calculate_sell_pnl(size, entry, current);
        assert_eq!(pnl, -45000.0);
    }

    #[test]
    fn test_extreme_gain_buy() {
        // 10x gain scenario
        let size = 0.1;
        let entry = 5000.0;
        let current = 50000.0;

        let pnl = calculate_buy_pnl(size, entry, current);
        assert_eq!(pnl, 4500.0);
    }

    #[test]
    fn test_extreme_gain_sell() {
        // 90% gain scenario (price drops to 10%)
        let size = 0.1;
        let entry = 50000.0;
        let current = 5000.0;

        let pnl = calculate_sell_pnl(size, entry, current);
        assert_eq!(pnl, 4500.0);
    }

    #[test]
    fn test_sl_and_tp_both_none() {
        // Both stop loss and take profit are None
        assert!(!is_buy_stop_loss_hit(50000.0, None));
        assert!(!is_buy_take_profit_hit(50000.0, None));
        assert!(!is_sell_stop_loss_hit(50000.0, None));
        assert!(!is_sell_take_profit_hit(50000.0, None));
    }

    #[test]
    fn test_negative_size_edge_case() {
        // Negative size (should not happen in real scenario but testing math)
        let pnl = calculate_buy_pnl(-1.0, 50000.0, 51000.0);
        assert_eq!(pnl, -1000.0); // Negative size inverts the PnL
    }

    #[test]
    fn test_identical_entry_and_current_multiple_sizes() {
        // Various sizes with identical entry and current prices
        let prices = (50000.0, 50000.0);

        for size in [0.1, 0.5, 1.0, 10.0, 100.0] {
            assert_eq!(calculate_buy_pnl(size, prices.0, prices.1), 0.0);
            assert_eq!(calculate_sell_pnl(size, prices.0, prices.1), 0.0);
        }
    }

    #[test]
    fn test_price_increment_precision() {
        // Test with very precise price increments
        let size = 1.0;
        let entry = 50000.0;

        let pnl1 = calculate_buy_pnl(size, entry, 50000.1);
        assert!((pnl1 - 0.1).abs() < 0.0001);

        let pnl2 = calculate_buy_pnl(size, entry, 50000.01);
        assert!((pnl2 - 0.01).abs() < 0.0001);

        let pnl3 = calculate_buy_pnl(size, entry, 50000.001);
        assert!((pnl3 - 0.001).abs() < 0.0001);
    }

    #[test]
    fn test_round_trip_pnl() {
        // Test that opposite trades cancel out
        let size = 1.0;
        let entry = 50000.0;
        let current = 51000.0;

        let buy_pnl = calculate_buy_pnl(size, entry, current);
        let sell_pnl = calculate_sell_pnl(size, current, entry);

        // Combined PnL should be zero (ignoring fees)
        assert_eq!(buy_pnl + sell_pnl, 2000.0);
    }

    #[test]
    fn test_stablecoin_pair_small_movement() {
        // Simulating stablecoin pair with small price movement
        let size = 10000.0; // Large position
        let entry = 1.0001;
        let current = 1.0002;

        let pnl = calculate_buy_pnl(size, entry, current);
        assert!((pnl - 1.0).abs() < 0.01);
    }

    #[test]
    fn test_consecutive_price_updates_buy() {
        // Simulate consecutive price updates for BUY position
        let size = 1.0;
        let entry = 50000.0;

        let prices = [50100.0, 50200.0, 50300.0, 50400.0, 50500.0];
        let expected_pnls = [100.0, 200.0, 300.0, 400.0, 500.0];

        for (price, expected) in prices.iter().zip(expected_pnls.iter()) {
            let pnl = calculate_buy_pnl(size, entry, *price);
            assert_eq!(pnl, *expected);
        }
    }

    #[test]
    fn test_consecutive_price_updates_sell() {
        // Simulate consecutive price updates for SELL position
        let size = 1.0;
        let entry = 50000.0;

        let prices = [49900.0, 49800.0, 49700.0, 49600.0, 49500.0];
        let expected_pnls = [100.0, 200.0, 300.0, 400.0, 500.0];

        for (price, expected) in prices.iter().zip(expected_pnls.iter()) {
            let pnl = calculate_sell_pnl(size, entry, *price);
            assert_eq!(pnl, *expected);
        }
    }
}
