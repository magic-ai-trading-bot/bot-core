// Comprehensive Position Manager and Risk Manager Tests

use binance_trading_bot::config::TradingConfig;
use binance_trading_bot::market_data::analyzer::{MultiTimeframeAnalysis, TradingSignal};
use binance_trading_bot::trading::position_manager::{Position, PositionManager};
use binance_trading_bot::trading::risk_manager::RiskManager;

// ============== POSITION MANAGER TESTS ==============

#[test]
fn test_position_manager_add_position() {
    let pm = PositionManager::new();

    let position = Position {
        id: "pos1".to_string(),
        symbol: "BTCUSDT".to_string(),
        side: "BUY".to_string(),
        size: 0.1,
        entry_price: 50000.0,
        current_price: 51000.0,
        unrealized_pnl: 100.0,
        stop_loss: Some(49000.0),
        take_profit: Some(52000.0),
        timestamp: 1234567890,
    };

    pm.add_position(position.clone());

    assert!(pm.has_position("BTCUSDT"));
    let retrieved = pm.get_position("BTCUSDT").unwrap();
    assert_eq!(retrieved.id, "pos1");
}

#[test]
fn test_position_manager_update_position() {
    let pm = PositionManager::new();

    let mut position = Position {
        id: "pos1".to_string(),
        symbol: "BTCUSDT".to_string(),
        side: "BUY".to_string(),
        size: 0.1,
        entry_price: 50000.0,
        current_price: 50000.0,
        unrealized_pnl: 0.0,
        stop_loss: None,
        take_profit: None,
        timestamp: 1234567890,
    };

    pm.add_position(position.clone());

    // Update with profit
    position.current_price = 51000.0;
    position.unrealized_pnl = 100.0;
    pm.update_position(position);

    let updated = pm.get_position("BTCUSDT").unwrap();
    assert_eq!(updated.current_price, 51000.0);
    assert_eq!(updated.unrealized_pnl, 100.0);
}

#[test]
fn test_position_manager_remove_position() {
    let pm = PositionManager::new();

    let position = Position {
        id: "pos1".to_string(),
        symbol: "BTCUSDT".to_string(),
        side: "BUY".to_string(),
        size: 0.1,
        entry_price: 50000.0,
        current_price: 50000.0,
        unrealized_pnl: 0.0,
        stop_loss: None,
        take_profit: None,
        timestamp: 1234567890,
    };

    pm.add_position(position);

    assert!(pm.has_position("BTCUSDT"));

    let removed = pm.remove_position("pos1");
    assert!(removed.is_some());
    assert!(!pm.has_position("BTCUSDT"));
}

#[test]
fn test_position_manager_get_all_positions() {
    let pm = PositionManager::new();

    // Add multiple positions
    for i in 0..3 {
        let position = Position {
            id: format!("pos{}", i),
            symbol: format!("SYMBOL{}", i),
            side: "BUY".to_string(),
            size: 0.1,
            entry_price: 50000.0,
            current_price: 50000.0,
            unrealized_pnl: 0.0,
            stop_loss: None,
            take_profit: None,
            timestamp: 1234567890,
        };
        pm.add_position(position);
    }

    let all_positions = pm.get_all_positions();
    assert_eq!(all_positions.len(), 3);
}

#[test]
fn test_position_manager_total_unrealized_pnl() {
    let pm = PositionManager::new();

    // Add positions with different PnL
    let pos1 = Position {
        id: "pos1".to_string(),
        symbol: "BTCUSDT".to_string(),
        side: "BUY".to_string(),
        size: 0.1,
        entry_price: 50000.0,
        current_price: 51000.0,
        unrealized_pnl: 100.0,
        stop_loss: None,
        take_profit: None,
        timestamp: 1234567890,
    };

    let pos2 = Position {
        id: "pos2".to_string(),
        symbol: "ETHUSDT".to_string(),
        side: "BUY".to_string(),
        size: 1.0,
        entry_price: 3000.0,
        current_price: 3050.0,
        unrealized_pnl: 50.0,
        stop_loss: None,
        take_profit: None,
        timestamp: 1234567890,
    };

    pm.add_position(pos1);
    pm.add_position(pos2);

    let total_pnl = pm.get_total_unrealized_pnl();
    assert_eq!(total_pnl, 150.0);
}

#[test]
fn test_position_manager_position_count() {
    let pm = PositionManager::new();

    assert_eq!(pm.get_position_count(), 0);

    for i in 0..5 {
        let position = Position {
            id: format!("pos{}", i),
            symbol: format!("SYMBOL{}", i),
            side: "BUY".to_string(),
            size: 0.1,
            entry_price: 50000.0,
            current_price: 50000.0,
            unrealized_pnl: 0.0,
            stop_loss: None,
            take_profit: None,
            timestamp: 1234567890,
        };
        pm.add_position(position);
    }

    assert_eq!(pm.get_position_count(), 5);
}

#[test]
fn test_position_manager_get_positions_by_side() {
    let pm = PositionManager::new();

    // Add long and short positions
    for i in 0..3 {
        let long_pos = Position {
            id: format!("long{}", i),
            symbol: format!("LONG{}", i),
            side: "BUY".to_string(),
            size: 0.1,
            entry_price: 50000.0,
            current_price: 50000.0,
            unrealized_pnl: 0.0,
            stop_loss: None,
            take_profit: None,
            timestamp: 1234567890,
        };
        pm.add_position(long_pos);
    }

    for i in 0..2 {
        let short_pos = Position {
            id: format!("short{}", i),
            symbol: format!("SHORT{}", i),
            side: "SELL".to_string(),
            size: 0.1,
            entry_price: 50000.0,
            current_price: 50000.0,
            unrealized_pnl: 0.0,
            stop_loss: None,
            take_profit: None,
            timestamp: 1234567890,
        };
        pm.add_position(short_pos);
    }

    let longs = pm.get_positions_by_side("BUY");
    let shorts = pm.get_positions_by_side("SELL");

    assert_eq!(longs.len(), 3);
    assert_eq!(shorts.len(), 2);
}

#[test]
fn test_position_manager_get_exposure_for_symbol() {
    let pm = PositionManager::new();

    let position = Position {
        id: "pos1".to_string(),
        symbol: "BTCUSDT".to_string(),
        side: "BUY".to_string(),
        size: 2.0,
        entry_price: 50000.0,
        current_price: 51000.0,
        unrealized_pnl: 2000.0,
        stop_loss: None,
        take_profit: None,
        timestamp: 1234567890,
    };

    pm.add_position(position);

    let exposure = pm.get_exposure_for_symbol("BTCUSDT");
    assert_eq!(exposure, 2.0 * 51000.0); // size * current_price = 102000

    let no_exposure = pm.get_exposure_for_symbol("ETHUSDT");
    assert_eq!(no_exposure, 0.0);
}

#[test]
fn test_position_manager_long_position_profit() {
    let pm = PositionManager::new();

    let position = Position {
        id: "pos1".to_string(),
        symbol: "BTCUSDT".to_string(),
        side: "BUY".to_string(),
        size: 1.0,
        entry_price: 50000.0,
        current_price: 55000.0,
        unrealized_pnl: 5000.0,
        stop_loss: Some(49000.0),
        take_profit: Some(60000.0),
        timestamp: 1234567890,
    };

    pm.add_position(position);

    let pos = pm.get_position("BTCUSDT").unwrap();
    assert!(pos.unrealized_pnl > 0.0);
    assert!(pos.current_price > pos.entry_price);
}

#[test]
fn test_position_manager_short_position_profit() {
    let pm = PositionManager::new();

    let position = Position {
        id: "pos1".to_string(),
        symbol: "BTCUSDT".to_string(),
        side: "SELL".to_string(),
        size: 1.0,
        entry_price: 50000.0,
        current_price: 45000.0,
        unrealized_pnl: 5000.0,
        stop_loss: Some(51000.0),
        take_profit: Some(40000.0),
        timestamp: 1234567890,
    };

    pm.add_position(position);

    let pos = pm.get_position("BTCUSDT").unwrap();
    assert!(pos.unrealized_pnl > 0.0);
    assert!(pos.current_price < pos.entry_price);
}

#[test]
fn test_position_manager_position_loss() {
    let pm = PositionManager::new();

    let position = Position {
        id: "pos1".to_string(),
        symbol: "BTCUSDT".to_string(),
        side: "BUY".to_string(),
        size: 1.0,
        entry_price: 50000.0,
        current_price: 48000.0,
        unrealized_pnl: -2000.0,
        stop_loss: Some(47000.0),
        take_profit: Some(55000.0),
        timestamp: 1234567890,
    };

    pm.add_position(position);

    let pos = pm.get_position("BTCUSDT").unwrap();
    assert!(pos.unrealized_pnl < 0.0);
}

#[test]
fn test_position_manager_concurrent_access() {
    use std::sync::Arc;
    use std::thread;

    let pm = Arc::new(PositionManager::new());
    let mut handles = vec![];

    // Spawn multiple threads adding positions
    for i in 0..10 {
        let pm_clone = Arc::clone(&pm);
        let handle = thread::spawn(move || {
            let position = Position {
                id: format!("pos{}", i),
                symbol: format!("SYMBOL{}", i),
                side: "BUY".to_string(),
                size: 0.1,
                entry_price: 50000.0,
                current_price: 50000.0,
                unrealized_pnl: 0.0,
                stop_loss: None,
                take_profit: None,
                timestamp: 1234567890,
            };
            pm_clone.add_position(position);
        });
        handles.push(handle);
    }

    for handle in handles {
        handle.join().unwrap();
    }

    assert_eq!(pm.get_position_count(), 10);
}

// ============== RISK MANAGER TESTS ==============

fn create_test_config() -> TradingConfig {
    TradingConfig {
        enabled: true,
        max_positions: 5,
        default_quantity: 0.01,
        leverage: 10,
        margin_type: "ISOLATED".to_string(),
        risk_percentage: 2.0,
        stop_loss_percentage: 2.0,
        take_profit_percentage: 4.0,
        order_timeout_seconds: 30,
        position_check_interval_seconds: 60,
    }
}

fn create_test_analysis(signal: TradingSignal, confidence: f64) -> MultiTimeframeAnalysis {
    MultiTimeframeAnalysis {
        symbol: "BTCUSDT".to_string(),
        overall_signal: signal,
        overall_confidence: confidence,
        timeframe_signals: std::collections::HashMap::new(),
        risk_reward_ratio: Some(2.0),
        entry_price: Some(50000.0),
        stop_loss: Some(49000.0),
        take_profit: Some(52000.0),
        timestamp: 1234567890,
    }
}

#[tokio::test]
async fn test_risk_manager_trading_enabled() {
    let config = create_test_config();
    let rm = RiskManager::new(config);

    let analysis = create_test_analysis(TradingSignal::StrongBuy, 0.85);

    let result = rm.can_open_position("BTCUSDT", &analysis).await;
    assert!(result.is_ok());
    assert!(result.unwrap());
}

#[tokio::test]
async fn test_risk_manager_trading_disabled() {
    let mut config = create_test_config();
    config.enabled = false;
    let rm = RiskManager::new(config);

    let analysis = create_test_analysis(TradingSignal::StrongBuy, 0.85);

    let result = rm.can_open_position("BTCUSDT", &analysis).await;
    assert!(result.is_ok());
    assert!(!result.unwrap());
}

#[tokio::test]
async fn test_risk_manager_strong_buy_confidence_check() {
    let config = create_test_config();
    let rm = RiskManager::new(config);

    // StrongBuy with high confidence should pass
    let analysis = create_test_analysis(TradingSignal::StrongBuy, 0.75);
    let result = rm.can_open_position("BTCUSDT", &analysis).await;
    assert!(result.unwrap());

    // StrongBuy with low confidence should fail
    let analysis_low = create_test_analysis(TradingSignal::StrongBuy, 0.65);
    let result_low = rm.can_open_position("BTCUSDT", &analysis_low).await;
    assert!(!result_low.unwrap());
}

#[tokio::test]
async fn test_risk_manager_buy_confidence_check() {
    let config = create_test_config();
    let rm = RiskManager::new(config);

    // Buy needs higher confidence (0.8)
    let analysis = create_test_analysis(TradingSignal::Buy, 0.85);
    let result = rm.can_open_position("BTCUSDT", &analysis).await;
    assert!(result.unwrap());

    // Buy with 0.75 confidence should fail
    let analysis_low = create_test_analysis(TradingSignal::Buy, 0.75);
    let result_low = rm.can_open_position("BTCUSDT", &analysis_low).await;
    assert!(!result_low.unwrap());
}

#[tokio::test]
async fn test_risk_manager_hold_signal() {
    let config = create_test_config();
    let rm = RiskManager::new(config);

    let analysis = create_test_analysis(TradingSignal::Hold, 0.99);

    let result = rm.can_open_position("BTCUSDT", &analysis).await;
    assert!(!result.unwrap());
}

#[tokio::test]
async fn test_risk_manager_risk_reward_ratio() {
    let config = create_test_config();
    let rm = RiskManager::new(config);

    // Good risk-reward ratio
    let mut analysis = create_test_analysis(TradingSignal::StrongBuy, 0.85);
    analysis.risk_reward_ratio = Some(2.0);
    let result = rm.can_open_position("BTCUSDT", &analysis).await;
    assert!(result.unwrap());

    // Poor risk-reward ratio
    analysis.risk_reward_ratio = Some(1.0);
    let result_poor = rm.can_open_position("BTCUSDT", &analysis).await;
    assert!(!result_poor.unwrap());
}

#[tokio::test]
async fn test_risk_manager_position_size_calculation() {
    let config = create_test_config();
    let rm = RiskManager::new(config);

    let size = rm.calculate_position_size("BTCUSDT", 50000.0, Some(49000.0), 10000.0);

    assert_eq!(size, 0.01); // Should return default_quantity
}

#[tokio::test]
async fn test_risk_manager_max_positions() {
    let config = create_test_config();
    let rm = RiskManager::new(config);

    let max_pos = rm.get_max_positions();
    assert_eq!(max_pos, 5);
}

#[tokio::test]
async fn test_risk_manager_risk_percentage() {
    let config = create_test_config();
    let rm = RiskManager::new(config);

    let risk_pct = rm.get_risk_percentage();
    assert_eq!(risk_pct, 2.0);
}

#[test]
fn test_trading_config_default_values() {
    let config = create_test_config();

    assert!(config.enabled);
    assert_eq!(config.max_positions, 5);
    assert_eq!(config.default_quantity, 0.01);
    assert_eq!(config.leverage, 10);
    assert_eq!(config.margin_type, "ISOLATED");
    assert_eq!(config.risk_percentage, 2.0);
}

#[test]
fn test_trading_config_custom_values() {
    let config = TradingConfig {
        enabled: false,
        max_positions: 10,
        default_quantity: 0.02,
        leverage: 20,
        margin_type: "CROSS".to_string(),
        risk_percentage: 1.5,
        stop_loss_percentage: 1.5,
        take_profit_percentage: 3.0,
        order_timeout_seconds: 30,
        position_check_interval_seconds: 60,
    };

    assert!(!config.enabled);
    assert_eq!(config.max_positions, 10);
    assert_eq!(config.leverage, 20);
    assert_eq!(config.margin_type, "CROSS");
}

// ============== INTEGRATION TESTS ==============

#[tokio::test]
async fn test_position_and_risk_integration() {
    let pm = PositionManager::new();
    let config = create_test_config();
    let rm = RiskManager::new(config);

    // Check if can open position
    let analysis = create_test_analysis(TradingSignal::StrongBuy, 0.85);
    let can_open = rm.can_open_position("BTCUSDT", &analysis).await.unwrap();

    assert!(can_open);

    // Open position
    if can_open {
        let position = Position {
            id: "pos1".to_string(),
            symbol: "BTCUSDT".to_string(),
            side: "BUY".to_string(),
            size: 0.01,
            entry_price: 50000.0,
            current_price: 50000.0,
            unrealized_pnl: 0.0,
            stop_loss: Some(49000.0),
            take_profit: Some(52000.0),
            timestamp: 1234567890,
        };
        pm.add_position(position);
    }

    assert_eq!(pm.get_position_count(), 1);
    assert!(pm.has_position("BTCUSDT"));
}

#[test]
fn test_position_pnl_calculations() {
    let pm = PositionManager::new();

    // Long position with profit
    let long_pos = Position {
        id: "long1".to_string(),
        symbol: "BTCUSDT".to_string(),
        side: "BUY".to_string(),
        size: 1.0,
        entry_price: 50000.0,
        current_price: 52000.0,
        unrealized_pnl: 2000.0,
        stop_loss: None,
        take_profit: None,
        timestamp: 1234567890,
    };

    // Short position with profit
    let short_pos = Position {
        id: "short1".to_string(),
        symbol: "ETHUSDT".to_string(),
        side: "SELL".to_string(),
        size: 10.0,
        entry_price: 3000.0,
        current_price: 2800.0,
        unrealized_pnl: 2000.0,
        stop_loss: None,
        take_profit: None,
        timestamp: 1234567890,
    };

    pm.add_position(long_pos);
    pm.add_position(short_pos);

    let total_pnl = pm.get_total_unrealized_pnl();
    assert_eq!(total_pnl, 4000.0);
}

#[test]
fn test_position_manager_default() {
    let pm1 = PositionManager::new();
    let pm2 = PositionManager::default();

    assert_eq!(pm1.get_position_count(), pm2.get_position_count());
}

#[test]
fn test_position_manager_clone() {
    let pm = PositionManager::new();

    let position = Position {
        id: "pos1".to_string(),
        symbol: "BTCUSDT".to_string(),
        side: "BUY".to_string(),
        size: 0.1,
        entry_price: 50000.0,
        current_price: 50000.0,
        unrealized_pnl: 0.0,
        stop_loss: None,
        take_profit: None,
        timestamp: 1234567890,
    };

    pm.add_position(position);

    let pm_clone = pm.clone();
    assert_eq!(pm.get_position_count(), pm_clone.get_position_count());
}

#[test]
fn test_position_serialization() {
    let position = Position {
        id: "pos1".to_string(),
        symbol: "BTCUSDT".to_string(),
        side: "BUY".to_string(),
        size: 0.1,
        entry_price: 50000.0,
        current_price: 51000.0,
        unrealized_pnl: 100.0,
        stop_loss: Some(49000.0),
        take_profit: Some(52000.0),
        timestamp: 1234567890,
    };

    let json = serde_json::to_string(&position).unwrap();
    let deserialized: Position = serde_json::from_str(&json).unwrap();

    assert_eq!(position.id, deserialized.id);
    assert_eq!(position.symbol, deserialized.symbol);
    assert_eq!(position.size, deserialized.size);
}
