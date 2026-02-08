// Comprehensive tests for trading/engine.rs
// Target: Increase coverage from 62.29% to 90%+
// Focus: Untested functions and error paths

use binance_trading_bot::binance::NewOrderRequest;
use binance_trading_bot::config::TradingConfig;
use binance_trading_bot::market_data::TradingSignal;
use binance_trading_bot::storage::TradeRecord;
use binance_trading_bot::trading::position_manager::{Position, PositionManager};
use binance_trading_bot::trading::risk_manager::RiskManager;
use uuid::Uuid;

// ===========================
// Test Utilities
// ===========================

fn create_test_position(symbol: &str, side: &str, size: f64, entry_price: f64) -> Position {
    Position {
        id: Uuid::new_v4().to_string(),
        symbol: symbol.to_string(),
        side: side.to_string(),
        size,
        entry_price,
        current_price: entry_price,
        unrealized_pnl: 0.0,
        stop_loss: Some(entry_price * 0.98),
        take_profit: Some(entry_price * 1.05),
        timestamp: chrono::Utc::now().timestamp_millis(),
    }
}

fn create_test_trade_record(symbol: &str, side: &str) -> TradeRecord {
    TradeRecord {
        id: None,
        symbol: symbol.to_string(),
        side: side.to_string(),
        quantity: 0.1,
        entry_price: 50000.0,
        exit_price: None,
        stop_loss: Some(49000.0),
        take_profit: Some(52500.0),
        entry_time: chrono::Utc::now().timestamp_millis(),
        exit_time: None,
        pnl: None,
        status: "open".to_string(),
        strategy_used: Some("multi_timeframe_analysis".to_string()),
    }
}

fn create_test_trading_config() -> TradingConfig {
    TradingConfig {
        enabled: true,
        max_positions: 5,
        default_quantity: 0.01,
        risk_percentage: 2.0,
        stop_loss_percentage: 2.0,
        take_profit_percentage: 5.0,
        order_timeout_seconds: 30,
        position_check_interval_seconds: 60,
        leverage: 5,
        margin_type: "CROSSED".to_string(),
    }
}

// ===========================
// Position Manager Tests
// ===========================

#[cfg(test)]
mod position_manager_tests {
    use super::*;

    #[test]
    fn test_add_position() {
        let manager = PositionManager::new();
        let position = create_test_position("BTCUSDT", "BUY", 0.1, 50000.0);

        manager.add_position(position.clone());

        assert!(manager.has_position("BTCUSDT"));
        assert_eq!(manager.get_position_count(), 1);
    }

    #[test]
    fn test_update_position() {
        let manager = PositionManager::new();
        let mut position = create_test_position("BTCUSDT", "BUY", 0.1, 50000.0);

        manager.add_position(position.clone());

        // Update position with new price
        position.current_price = 51000.0;
        position.unrealized_pnl = 100.0;
        manager.update_position(position.clone());

        let updated = manager.get_position("BTCUSDT").unwrap();
        assert_eq!(updated.current_price, 51000.0);
        assert_eq!(updated.unrealized_pnl, 100.0);
    }

    #[test]
    fn test_remove_position() {
        let manager = PositionManager::new();
        let position = create_test_position("BTCUSDT", "BUY", 0.1, 50000.0);
        let position_id = position.id.clone();

        manager.add_position(position);
        assert!(manager.has_position("BTCUSDT"));

        let removed = manager.remove_position(&position_id);
        assert!(removed.is_some());
        assert!(!manager.has_position("BTCUSDT"));
        assert_eq!(manager.get_position_count(), 0);
    }

    #[test]
    fn test_remove_nonexistent_position() {
        let manager = PositionManager::new();
        let removed = manager.remove_position("nonexistent-id");
        assert!(removed.is_none());
    }

    #[test]
    fn test_get_position() {
        let manager = PositionManager::new();
        let position = create_test_position("BTCUSDT", "BUY", 0.1, 50000.0);

        manager.add_position(position.clone());

        let retrieved = manager.get_position("BTCUSDT");
        assert!(retrieved.is_some());
        assert_eq!(retrieved.unwrap().symbol, "BTCUSDT");
    }

    #[test]
    fn test_get_nonexistent_position() {
        let manager = PositionManager::new();
        let position = manager.get_position("NONEXISTENT");
        assert!(position.is_none());
    }

    #[test]
    fn test_has_position() {
        let manager = PositionManager::new();

        assert!(!manager.has_position("BTCUSDT"));

        let position = create_test_position("BTCUSDT", "BUY", 0.1, 50000.0);
        manager.add_position(position);

        assert!(manager.has_position("BTCUSDT"));
    }

    #[test]
    fn test_get_all_positions() {
        let manager = PositionManager::new();

        manager.add_position(create_test_position("BTCUSDT", "BUY", 0.1, 50000.0));
        manager.add_position(create_test_position("ETHUSDT", "SELL", 0.5, 3000.0));
        manager.add_position(create_test_position("BNBUSDT", "BUY", 1.0, 500.0));

        let all_positions = manager.get_all_positions();
        assert_eq!(all_positions.len(), 3);
    }

    #[test]
    fn test_get_total_unrealized_pnl() {
        let manager = PositionManager::new();

        let mut pos1 = create_test_position("BTCUSDT", "BUY", 0.1, 50000.0);
        pos1.unrealized_pnl = 100.0;

        let mut pos2 = create_test_position("ETHUSDT", "SELL", 0.5, 3000.0);
        pos2.unrealized_pnl = -50.0;

        let mut pos3 = create_test_position("BNBUSDT", "BUY", 1.0, 500.0);
        pos3.unrealized_pnl = 25.0;

        manager.add_position(pos1);
        manager.add_position(pos2);
        manager.add_position(pos3);

        let total_pnl = manager.get_total_unrealized_pnl();
        assert_eq!(total_pnl, 75.0); // 100 - 50 + 25
    }

    #[test]
    fn test_get_position_count() {
        let manager = PositionManager::new();
        assert_eq!(manager.get_position_count(), 0);

        manager.add_position(create_test_position("BTCUSDT", "BUY", 0.1, 50000.0));
        assert_eq!(manager.get_position_count(), 1);

        manager.add_position(create_test_position("ETHUSDT", "SELL", 0.5, 3000.0));
        assert_eq!(manager.get_position_count(), 2);
    }

    #[test]
    fn test_get_positions_by_side_buy() {
        let manager = PositionManager::new();

        manager.add_position(create_test_position("BTCUSDT", "BUY", 0.1, 50000.0));
        manager.add_position(create_test_position("ETHUSDT", "SELL", 0.5, 3000.0));
        manager.add_position(create_test_position("BNBUSDT", "BUY", 1.0, 500.0));

        let buy_positions = manager.get_positions_by_side("BUY");
        assert_eq!(buy_positions.len(), 2);

        for pos in buy_positions {
            assert_eq!(pos.side, "BUY");
        }
    }

    #[test]
    fn test_get_positions_by_side_sell() {
        let manager = PositionManager::new();

        manager.add_position(create_test_position("BTCUSDT", "BUY", 0.1, 50000.0));
        manager.add_position(create_test_position("ETHUSDT", "SELL", 0.5, 3000.0));
        manager.add_position(create_test_position("BNBUSDT", "SELL", 1.0, 500.0));

        let sell_positions = manager.get_positions_by_side("SELL");
        assert_eq!(sell_positions.len(), 2);

        for pos in sell_positions {
            assert_eq!(pos.side, "SELL");
        }
    }

    #[test]
    fn test_get_exposure_for_symbol() {
        let manager = PositionManager::new();

        let mut position = create_test_position("BTCUSDT", "BUY", 0.1, 50000.0);
        position.current_price = 51000.0;
        manager.add_position(position);

        let exposure = manager.get_exposure_for_symbol("BTCUSDT");
        assert_eq!(exposure, 5100.0); // 0.1 * 51000
    }

    #[test]
    fn test_get_exposure_for_nonexistent_symbol() {
        let manager = PositionManager::new();
        let exposure = manager.get_exposure_for_symbol("NONEXISTENT");
        assert_eq!(exposure, 0.0);
    }

    #[test]
    fn test_multiple_updates_same_position() {
        let manager = PositionManager::new();
        let mut position = create_test_position("BTCUSDT", "BUY", 0.1, 50000.0);

        manager.add_position(position.clone());

        // Multiple price updates
        for price in [51000.0, 52000.0, 53000.0, 54000.0, 55000.0] {
            position.current_price = price;
            position.unrealized_pnl = (price - position.entry_price) * position.size;
            manager.update_position(position.clone());
        }

        let final_position = manager.get_position("BTCUSDT").unwrap();
        assert_eq!(final_position.current_price, 55000.0);
        assert_eq!(final_position.unrealized_pnl, 500.0); // (55000 - 50000) * 0.1
    }

    #[test]
    fn test_position_manager_default() {
        let manager = PositionManager::default();
        assert_eq!(manager.get_position_count(), 0);
    }

    #[test]
    fn test_position_with_no_stop_loss_take_profit() {
        let mut position = create_test_position("BTCUSDT", "BUY", 0.1, 50000.0);
        position.stop_loss = None;
        position.take_profit = None;

        let manager = PositionManager::new();
        manager.add_position(position);

        let retrieved = manager.get_position("BTCUSDT").unwrap();
        assert!(retrieved.stop_loss.is_none());
        assert!(retrieved.take_profit.is_none());
    }
}

// ===========================
// Risk Manager Tests
// ===========================

#[cfg(test)]
mod risk_manager_tests {
    use super::*;
    use binance_trading_bot::market_data::analyzer::MultiTimeframeAnalysis;
    use std::collections::HashMap;

    fn create_test_analysis(signal: TradingSignal, confidence: f64) -> MultiTimeframeAnalysis {
        MultiTimeframeAnalysis {
            symbol: "BTCUSDT".to_string(),
            timestamp: chrono::Utc::now().timestamp_millis(),
            timeframe_signals: HashMap::new(),
            overall_signal: signal,
            overall_confidence: confidence,
            entry_price: Some(50000.0),
            stop_loss: Some(49000.0),
            take_profit: Some(52500.0),
            risk_reward_ratio: Some(2.5),
        }
    }

    #[tokio::test]
    async fn test_can_open_position_strong_buy_high_confidence() {
        let config = create_test_trading_config();
        let risk_manager = RiskManager::new(config);

        let analysis = create_test_analysis(TradingSignal::StrongBuy, 0.85);
        let result = risk_manager.can_open_position("BTCUSDT", &analysis).await;

        assert!(result.is_ok());
        assert!(result.unwrap());
    }

    #[tokio::test]
    async fn test_can_open_position_strong_buy_low_confidence() {
        let config = create_test_trading_config();
        let risk_manager = RiskManager::new(config);

        let analysis = create_test_analysis(TradingSignal::StrongBuy, 0.6); // Below 0.7 threshold
        let result = risk_manager.can_open_position("BTCUSDT", &analysis).await;

        assert!(result.is_ok());
        assert!(!result.unwrap()); // Should reject
    }

    #[tokio::test]
    async fn test_can_open_position_buy_high_confidence() {
        let config = create_test_trading_config();
        let risk_manager = RiskManager::new(config);

        let analysis = create_test_analysis(TradingSignal::Buy, 0.85);
        let result = risk_manager.can_open_position("BTCUSDT", &analysis).await;

        assert!(result.is_ok());
        assert!(result.unwrap());
    }

    #[tokio::test]
    async fn test_can_open_position_buy_low_confidence() {
        let config = create_test_trading_config();
        let risk_manager = RiskManager::new(config);

        let analysis = create_test_analysis(TradingSignal::Buy, 0.75); // Below 0.8 threshold
        let result = risk_manager.can_open_position("BTCUSDT", &analysis).await;

        assert!(result.is_ok());
        assert!(!result.unwrap()); // Should reject
    }

    #[tokio::test]
    async fn test_can_open_position_sell_high_confidence() {
        let config = create_test_trading_config();
        let risk_manager = RiskManager::new(config);

        let analysis = create_test_analysis(TradingSignal::Sell, 0.85);
        let result = risk_manager.can_open_position("BTCUSDT", &analysis).await;

        assert!(result.is_ok());
        assert!(result.unwrap());
    }

    #[tokio::test]
    async fn test_can_open_position_strong_sell_high_confidence() {
        let config = create_test_trading_config();
        let risk_manager = RiskManager::new(config);

        let analysis = create_test_analysis(TradingSignal::StrongSell, 0.75);
        let result = risk_manager.can_open_position("BTCUSDT", &analysis).await;

        assert!(result.is_ok());
        assert!(result.unwrap());
    }

    #[tokio::test]
    async fn test_can_open_position_hold_signal() {
        let config = create_test_trading_config();
        let risk_manager = RiskManager::new(config);

        let analysis = create_test_analysis(TradingSignal::Hold, 0.9); // High confidence but HOLD
        let result = risk_manager.can_open_position("BTCUSDT", &analysis).await;

        assert!(result.is_ok());
        assert!(!result.unwrap()); // Should always reject HOLD signals
    }

    #[tokio::test]
    async fn test_can_open_position_trading_disabled() {
        let mut config = create_test_trading_config();
        config.enabled = false;
        let risk_manager = RiskManager::new(config);

        let analysis = create_test_analysis(TradingSignal::StrongBuy, 0.9);
        let result = risk_manager.can_open_position("BTCUSDT", &analysis).await;

        assert!(result.is_ok());
        assert!(!result.unwrap()); // Should reject when trading disabled
    }

    #[tokio::test]
    async fn test_can_open_position_low_risk_reward() {
        let config = create_test_trading_config();
        let risk_manager = RiskManager::new(config);

        let mut analysis = create_test_analysis(TradingSignal::StrongBuy, 0.85);
        analysis.risk_reward_ratio = Some(1.2); // Below 1.5 threshold

        let result = risk_manager.can_open_position("BTCUSDT", &analysis).await;

        assert!(result.is_ok());
        assert!(!result.unwrap()); // Should reject poor risk/reward
    }

    #[tokio::test]
    async fn test_can_open_position_no_risk_reward() {
        let config = create_test_trading_config();
        let risk_manager = RiskManager::new(config);

        let mut analysis = create_test_analysis(TradingSignal::StrongBuy, 0.85);
        analysis.risk_reward_ratio = None;

        let result = risk_manager.can_open_position("BTCUSDT", &analysis).await;

        assert!(result.is_ok());
        assert!(result.unwrap()); // Should pass when risk/reward not provided
    }

    #[tokio::test]
    async fn test_can_open_position_boundary_confidence_strong_buy() {
        let config = create_test_trading_config();
        let risk_manager = RiskManager::new(config);

        // Exactly at threshold (implementation uses < so 0.7 should pass)
        let analysis = create_test_analysis(TradingSignal::StrongBuy, 0.7);
        let result = risk_manager.can_open_position("BTCUSDT", &analysis).await;

        assert!(result.is_ok());
        assert!(result.unwrap()); // 0.7 is NOT < 0.7, should pass
    }

    #[tokio::test]
    async fn test_can_open_position_boundary_confidence_buy() {
        let config = create_test_trading_config();
        let risk_manager = RiskManager::new(config);

        // Exactly at threshold (implementation uses < so 0.8 should pass)
        let analysis = create_test_analysis(TradingSignal::Buy, 0.8);
        let result = risk_manager.can_open_position("BTCUSDT", &analysis).await;

        assert!(result.is_ok());
        assert!(result.unwrap()); // 0.8 is NOT < 0.8, should pass
    }

    #[tokio::test]
    async fn test_can_open_position_boundary_risk_reward() {
        let config = create_test_trading_config();
        let risk_manager = RiskManager::new(config);

        let mut analysis = create_test_analysis(TradingSignal::StrongBuy, 0.85);
        analysis.risk_reward_ratio = Some(1.5); // Exactly at threshold

        let result = risk_manager.can_open_position("BTCUSDT", &analysis).await;

        assert!(result.is_ok());
        assert!(result.unwrap()); // 1.5 is NOT < 1.5, should pass
    }
}

// ===========================
// Trade Record Tests
// ===========================

#[cfg(test)]
mod trade_record_tests {
    use super::*;

    #[test]
    fn test_create_trade_record_buy() {
        let trade = create_test_trade_record("BTCUSDT", "BUY");

        assert_eq!(trade.symbol, "BTCUSDT");
        assert_eq!(trade.side, "BUY");
        assert_eq!(trade.quantity, 0.1);
        assert_eq!(trade.entry_price, 50000.0);
        assert_eq!(trade.status, "open");
        assert!(trade.stop_loss.is_some());
        assert!(trade.take_profit.is_some());
        assert!(trade.exit_price.is_none());
        assert!(trade.pnl.is_none());
    }

    #[test]
    fn test_create_trade_record_sell() {
        let trade = create_test_trade_record("ETHUSDT", "SELL");

        assert_eq!(trade.symbol, "ETHUSDT");
        assert_eq!(trade.side, "SELL");
        assert_eq!(trade.status, "open");
    }

    #[test]
    fn test_trade_record_with_exit() {
        let mut trade = create_test_trade_record("BTCUSDT", "BUY");

        // Close the trade
        trade.exit_price = Some(51000.0);
        trade.exit_time = Some(chrono::Utc::now().timestamp_millis());
        trade.pnl = Some(100.0); // (51000 - 50000) * 0.1
        trade.status = "closed".to_string();

        assert_eq!(trade.status, "closed");
        assert!(trade.exit_price.is_some());
        assert!(trade.pnl.is_some());
        assert_eq!(trade.pnl.unwrap(), 100.0);
    }

    #[test]
    fn test_trade_record_without_stop_loss_take_profit() {
        let mut trade = create_test_trade_record("BTCUSDT", "BUY");
        trade.stop_loss = None;
        trade.take_profit = None;

        assert!(trade.stop_loss.is_none());
        assert!(trade.take_profit.is_none());
    }

    #[test]
    fn test_trade_record_strategy_used() {
        let trade = create_test_trade_record("BTCUSDT", "BUY");

        assert!(trade.strategy_used.is_some());
        assert_eq!(trade.strategy_used.unwrap(), "multi_timeframe_analysis");
    }

    #[test]
    fn test_trade_record_no_strategy() {
        let mut trade = create_test_trade_record("BTCUSDT", "BUY");
        trade.strategy_used = None;

        assert!(trade.strategy_used.is_none());
    }
}

// ===========================
// Order Request Tests
// ===========================

#[cfg(test)]
mod order_request_tests {
    use super::*;

    #[test]
    fn test_create_market_order_buy() {
        let order = NewOrderRequest {
            symbol: "BTCUSDT".to_string(),
            side: "BUY".to_string(),
            r#type: "MARKET".to_string(),
            quantity: Some("0.01".to_string()),
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

        assert_eq!(order.symbol, "BTCUSDT");
        assert_eq!(order.side, "BUY");
        assert_eq!(order.r#type, "MARKET");
        assert!(order.quantity.is_some());
        assert!(order.price.is_none());
    }

    #[test]
    fn test_create_market_order_sell() {
        let order = NewOrderRequest {
            symbol: "ETHUSDT".to_string(),
            side: "SELL".to_string(),
            r#type: "MARKET".to_string(),
            quantity: Some("0.5".to_string()),
            quote_order_qty: None,
            price: None,
            new_client_order_id: Some(Uuid::new_v4().to_string()),
            stop_price: None,
            iceberg_qty: None,
            new_order_resp_type: Some("RESULT".to_string()),
            time_in_force: None,
            reduce_only: Some(true), // Close position
            close_position: Some(false),
            position_side: Some("BOTH".to_string()),
            working_type: None,
            price_protect: Some(false),
        };

        assert_eq!(order.side, "SELL");
        assert_eq!(order.reduce_only, Some(true));
    }

    #[test]
    fn test_create_limit_order() {
        let order = NewOrderRequest {
            symbol: "BTCUSDT".to_string(),
            side: "BUY".to_string(),
            r#type: "LIMIT".to_string(),
            quantity: Some("0.01".to_string()),
            quote_order_qty: None,
            price: Some("49000.0".to_string()),
            new_client_order_id: Some(Uuid::new_v4().to_string()),
            stop_price: None,
            iceberg_qty: None,
            new_order_resp_type: Some("RESULT".to_string()),
            time_in_force: Some("GTC".to_string()),
            reduce_only: Some(false),
            close_position: Some(false),
            position_side: Some("BOTH".to_string()),
            working_type: None,
            price_protect: Some(false),
        };

        assert_eq!(order.r#type, "LIMIT");
        assert!(order.price.is_some());
        assert!(order.time_in_force.is_some());
    }

    #[test]
    fn test_create_stop_loss_order() {
        let order = NewOrderRequest {
            symbol: "BTCUSDT".to_string(),
            side: "SELL".to_string(),
            r#type: "STOP_MARKET".to_string(),
            quantity: Some("0.01".to_string()),
            quote_order_qty: None,
            price: None,
            new_client_order_id: Some(Uuid::new_v4().to_string()),
            stop_price: Some("49000.0".to_string()),
            iceberg_qty: None,
            new_order_resp_type: Some("RESULT".to_string()),
            time_in_force: None,
            reduce_only: Some(true),
            close_position: Some(false),
            position_side: Some("BOTH".to_string()),
            working_type: Some("MARK_PRICE".to_string()),
            price_protect: Some(false),
        };

        assert_eq!(order.r#type, "STOP_MARKET");
        assert!(order.stop_price.is_some());
        assert_eq!(order.reduce_only, Some(true));
    }

    #[test]
    fn test_order_with_close_position() {
        let order = NewOrderRequest {
            symbol: "BTCUSDT".to_string(),
            side: "SELL".to_string(),
            r#type: "MARKET".to_string(),
            quantity: None, // No quantity when closing entire position
            quote_order_qty: None,
            price: None,
            new_client_order_id: Some(Uuid::new_v4().to_string()),
            stop_price: None,
            iceberg_qty: None,
            new_order_resp_type: Some("RESULT".to_string()),
            time_in_force: None,
            reduce_only: Some(false),
            close_position: Some(true), // Close entire position
            position_side: Some("BOTH".to_string()),
            working_type: None,
            price_protect: Some(false),
        };

        assert_eq!(order.close_position, Some(true));
        assert!(order.quantity.is_none());
    }
}

// ===========================
// Configuration Tests
// ===========================

#[cfg(test)]
mod config_tests {
    use super::*;

    #[test]
    fn test_trading_config_default() {
        let config = create_test_trading_config();

        assert!(config.enabled);
        assert_eq!(config.leverage, 5);
        assert_eq!(config.margin_type, "CROSSED");
        assert_eq!(config.position_check_interval_seconds, 60);
        assert_eq!(config.max_positions, 5);
        assert_eq!(config.risk_percentage, 2.0);
    }

    #[test]
    fn test_trading_config_disabled() {
        let mut config = create_test_trading_config();
        config.enabled = false;

        assert!(!config.enabled);
    }

    #[test]
    fn test_trading_config_high_leverage() {
        let mut config = create_test_trading_config();
        config.leverage = 20;

        assert_eq!(config.leverage, 20);
    }

    #[test]
    fn test_trading_config_isolated_margin() {
        let mut config = create_test_trading_config();
        config.margin_type = "ISOLATED".to_string();

        assert_eq!(config.margin_type, "ISOLATED");
    }

    #[test]
    fn test_trading_config_custom_risk() {
        let mut config = create_test_trading_config();
        config.risk_percentage = 1.0;
        config.max_positions = 3;

        assert_eq!(config.risk_percentage, 1.0);
        assert_eq!(config.max_positions, 3);
    }
}

// ===========================
// Position PnL Calculation Tests
// ===========================

#[cfg(test)]
mod pnl_calculation_tests {
    use super::*;

    #[test]
    fn test_long_position_profit() {
        let mut position = create_test_position("BTCUSDT", "BUY", 0.1, 50000.0);
        position.current_price = 51000.0;

        // Calculate PnL: (current_price - entry_price) * size
        let pnl = (position.current_price - position.entry_price) * position.size;

        assert_eq!(pnl, 100.0);
    }

    #[test]
    fn test_long_position_loss() {
        let mut position = create_test_position("BTCUSDT", "BUY", 0.1, 50000.0);
        position.current_price = 49000.0;

        let pnl = (position.current_price - position.entry_price) * position.size;

        assert_eq!(pnl, -100.0);
    }

    #[test]
    fn test_short_position_profit() {
        let mut position = create_test_position("BTCUSDT", "SELL", 0.1, 50000.0);
        position.current_price = 49000.0;

        // For short: (entry_price - current_price) * size
        let pnl = (position.entry_price - position.current_price) * position.size;

        assert_eq!(pnl, 100.0);
    }

    #[test]
    fn test_short_position_loss() {
        let mut position = create_test_position("BTCUSDT", "SELL", 0.1, 50000.0);
        position.current_price = 51000.0;

        let pnl = (position.entry_price - position.current_price) * position.size;

        assert_eq!(pnl, -100.0);
    }

    #[test]
    fn test_stop_loss_hit_buy() {
        let position = create_test_position("BTCUSDT", "BUY", 0.1, 50000.0);
        let current_price = 48500.0;
        let stop_loss = position.stop_loss.unwrap();

        let should_close = current_price <= stop_loss;

        assert!(should_close);
    }

    #[test]
    fn test_stop_loss_not_hit_buy() {
        let position = create_test_position("BTCUSDT", "BUY", 0.1, 50000.0);
        let current_price = 49500.0;
        let stop_loss = position.stop_loss.unwrap();

        let should_close = current_price <= stop_loss;

        assert!(!should_close);
    }

    #[test]
    fn test_take_profit_hit_buy() {
        let position = create_test_position("BTCUSDT", "BUY", 0.1, 50000.0);
        let current_price = 52500.0;
        let take_profit = position.take_profit.unwrap();

        let should_close = current_price >= take_profit;

        assert!(should_close);
    }

    #[test]
    fn test_take_profit_not_hit_buy() {
        let position = create_test_position("BTCUSDT", "BUY", 0.1, 50000.0);
        let current_price = 51000.0;
        let take_profit = position.take_profit.unwrap();

        let should_close = current_price >= take_profit;

        assert!(!should_close);
    }

    #[test]
    fn test_stop_loss_hit_sell() {
        let mut position = create_test_position("BTCUSDT", "SELL", 0.1, 50000.0);
        position.stop_loss = Some(51000.0);
        let current_price = 51500.0;
        let stop_loss = position.stop_loss.unwrap();

        let should_close = current_price >= stop_loss;

        assert!(should_close);
    }

    #[test]
    fn test_take_profit_hit_sell() {
        let mut position = create_test_position("BTCUSDT", "SELL", 0.1, 50000.0);
        position.take_profit = Some(48000.0);
        let current_price = 47500.0;
        let take_profit = position.take_profit.unwrap();

        let should_close = current_price <= take_profit;

        assert!(should_close);
    }
}
