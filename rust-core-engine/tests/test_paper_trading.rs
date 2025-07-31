mod common;

use actix_web::{test, web, App};
use binance_trading_bot::models::*;
use binance_trading_bot::paper_trading::*;
use binance_trading_bot::storage::Storage;
use chrono::Utc;
use common::*;
use rust_decimal::Decimal;
use serde_json::json;
use std::str::FromStr;

#[actix_web::test]
async fn test_paper_trading_account_creation() {
    let db = setup_test_db().await;
    let storage = Storage::new_with_db(Some(db.clone())).await;

    // Create paper trading account
    let account = PaperTradingAccount::new("user123".to_string(), 10000.0);

    assert_eq!(account.user_id, "user123");
    assert_eq!(account.balance, 10000.0);
    assert!(account.positions.is_empty());
    assert!(account.trade_history.is_empty());

    // Save to storage
    storage.save_paper_account(&account).await.unwrap();

    // Load back
    let loaded = storage.load_paper_account("user123").await.unwrap();
    assert!(loaded.is_some());
    assert_eq!(loaded.unwrap().balance, 10000.0);

    cleanup_test_db(db).await;
}

#[actix_web::test]
async fn test_execute_paper_trade_buy() {
    let db = setup_test_db().await;
    let storage = Storage::new_with_db(Some(db.clone())).await;

    // Create account with balance
    let mut account = PaperTradingAccount::new("user123".to_string(), 10000.0);

    // Execute buy trade
    let trade = Trade {
        id: "trade1".to_string(),
        user_id: "user123".to_string(),
        symbol: "BTCUSDT".to_string(),
        side: TradeSide::Buy,
        quantity: Decimal::from_str("0.1").unwrap(),
        price: Decimal::from_str("45000").unwrap(),
        trade_type: TradeType::Long,
        timestamp: Utc::now(),
        order_id: "order1".to_string(),
        status: TradeStatus::Executed,
        pnl: None,
        commission: Some(Decimal::from_str("4.5").unwrap()), // 0.1% commission
    };

    // Apply trade
    account.apply_trade(&trade).unwrap();

    // Check balance decreased
    assert!(account.balance < 10000.0);
    assert_eq!(account.balance, 10000.0 - 4500.0 - 4.5); // cost + commission

    // Check position created
    assert_eq!(account.positions.len(), 1);
    let position = &account.positions["BTCUSDT"];
    assert_eq!(position.quantity, Decimal::from_str("0.1").unwrap());
    assert_eq!(position.entry_price, Decimal::from_str("45000").unwrap());

    cleanup_test_db(db).await;
}

#[actix_web::test]
async fn test_execute_paper_trade_sell() {
    let db = setup_test_db().await;
    let storage = Storage::new_with_db(Some(db.clone())).await;

    // Create account with existing position
    let mut account = PaperTradingAccount::new("user123".to_string(), 10000.0);

    // First buy
    let buy_trade = Trade {
        id: "trade1".to_string(),
        user_id: "user123".to_string(),
        symbol: "BTCUSDT".to_string(),
        side: TradeSide::Buy,
        quantity: Decimal::from_str("0.1").unwrap(),
        price: Decimal::from_str("45000").unwrap(),
        trade_type: TradeType::Long,
        timestamp: Utc::now(),
        order_id: "order1".to_string(),
        status: TradeStatus::Executed,
        pnl: None,
        commission: Some(Decimal::from_str("4.5").unwrap()),
    };
    account.apply_trade(&buy_trade).unwrap();

    // Then sell at profit
    let sell_trade = Trade {
        id: "trade2".to_string(),
        user_id: "user123".to_string(),
        symbol: "BTCUSDT".to_string(),
        side: TradeSide::Sell,
        quantity: Decimal::from_str("0.1").unwrap(),
        price: Decimal::from_str("46000").unwrap(),
        trade_type: TradeType::Long,
        timestamp: Utc::now(),
        order_id: "order2".to_string(),
        status: TradeStatus::Executed,
        pnl: None,
        commission: Some(Decimal::from_str("4.6").unwrap()),
    };
    account.apply_trade(&sell_trade).unwrap();

    // Check position closed
    assert!(account.positions.is_empty());

    // Check profit
    let final_balance = account.balance;
    let profit = (46000.0 - 45000.0) * 0.1 - 4.5 - 4.6; // price diff - commissions
    assert!((final_balance - (10000.0 + profit)).abs() < 0.01);

    // Check trade history
    assert_eq!(account.trade_history.len(), 2);

    cleanup_test_db(db).await;
}

#[actix_web::test]
async fn test_paper_trading_insufficient_balance() {
    let db = setup_test_db().await;
    let storage = Storage::new_with_db(Some(db.clone())).await;

    // Create account with small balance
    let mut account = PaperTradingAccount::new("user123".to_string(), 100.0);

    // Try to buy more than balance
    let trade = Trade {
        id: "trade1".to_string(),
        user_id: "user123".to_string(),
        symbol: "BTCUSDT".to_string(),
        side: TradeSide::Buy,
        quantity: Decimal::from_str("1.0").unwrap(),
        price: Decimal::from_str("45000").unwrap(),
        trade_type: TradeType::Long,
        timestamp: Utc::now(),
        order_id: "order1".to_string(),
        status: TradeStatus::Executed,
        pnl: None,
        commission: Some(Decimal::from_str("45").unwrap()),
    };

    // Should fail
    let result = account.apply_trade(&trade);
    assert!(result.is_err());
    assert!(result
        .unwrap_err()
        .to_string()
        .contains("Insufficient balance"));

    cleanup_test_db(db).await;
}

#[actix_web::test]
async fn test_paper_trading_short_position() {
    let db = setup_test_db().await;
    let storage = Storage::new_with_db(Some(db.clone())).await;

    // Create account
    let mut account = PaperTradingAccount::new("user123".to_string(), 10000.0);

    // Open short position
    let short_trade = Trade {
        id: "trade1".to_string(),
        user_id: "user123".to_string(),
        symbol: "BTCUSDT".to_string(),
        side: TradeSide::Sell,
        quantity: Decimal::from_str("0.1").unwrap(),
        price: Decimal::from_str("45000").unwrap(),
        trade_type: TradeType::Short,
        timestamp: Utc::now(),
        order_id: "order1".to_string(),
        status: TradeStatus::Executed,
        pnl: None,
        commission: Some(Decimal::from_str("4.5").unwrap()),
    };
    account.apply_trade(&short_trade).unwrap();

    // Check short position
    let position = &account.positions["BTCUSDT"];
    assert_eq!(position.side, PositionSide::Short);
    assert_eq!(position.quantity, Decimal::from_str("0.1").unwrap());

    // Close short at profit (price went down)
    let cover_trade = Trade {
        id: "trade2".to_string(),
        user_id: "user123".to_string(),
        symbol: "BTCUSDT".to_string(),
        side: TradeSide::Buy,
        quantity: Decimal::from_str("0.1").unwrap(),
        price: Decimal::from_str("44000").unwrap(),
        trade_type: TradeType::Short,
        timestamp: Utc::now(),
        order_id: "order2".to_string(),
        status: TradeStatus::Executed,
        pnl: None,
        commission: Some(Decimal::from_str("4.4").unwrap()),
    };
    account.apply_trade(&cover_trade).unwrap();

    // Check profit from short
    let profit = (45000.0 - 44000.0) * 0.1 - 4.5 - 4.4;
    assert!(account.balance > 10000.0 + profit - 1.0); // Allow small rounding

    cleanup_test_db(db).await;
}

#[actix_web::test]
async fn test_paper_trading_statistics() {
    let db = setup_test_db().await;
    let storage = Storage::new_with_db(Some(db.clone())).await;

    // Create account and execute multiple trades
    let mut account = PaperTradingAccount::new("user123".to_string(), 10000.0);

    // Winning trade
    let buy1 = Trade {
        id: "trade1".to_string(),
        user_id: "user123".to_string(),
        symbol: "BTCUSDT".to_string(),
        side: TradeSide::Buy,
        quantity: Decimal::from_str("0.1").unwrap(),
        price: Decimal::from_str("45000").unwrap(),
        trade_type: TradeType::Long,
        timestamp: Utc::now(),
        order_id: "order1".to_string(),
        status: TradeStatus::Executed,
        pnl: None,
        commission: Some(Decimal::from_str("4.5").unwrap()),
    };
    account.apply_trade(&buy1).unwrap();

    let sell1 = Trade {
        id: "trade2".to_string(),
        user_id: "user123".to_string(),
        symbol: "BTCUSDT".to_string(),
        side: TradeSide::Sell,
        quantity: Decimal::from_str("0.1").unwrap(),
        price: Decimal::from_str("46000").unwrap(),
        trade_type: TradeType::Long,
        timestamp: Utc::now(),
        order_id: "order2".to_string(),
        status: TradeStatus::Executed,
        pnl: Some(Decimal::from_str("100").unwrap()),
        commission: Some(Decimal::from_str("4.6").unwrap()),
    };
    account.apply_trade(&sell1).unwrap();

    // Losing trade
    let buy2 = Trade {
        id: "trade3".to_string(),
        user_id: "user123".to_string(),
        symbol: "ETHUSDT".to_string(),
        side: TradeSide::Buy,
        quantity: Decimal::from_str("1.0").unwrap(),
        price: Decimal::from_str("3000").unwrap(),
        trade_type: TradeType::Long,
        timestamp: Utc::now(),
        order_id: "order3".to_string(),
        status: TradeStatus::Executed,
        pnl: None,
        commission: Some(Decimal::from_str("3.0").unwrap()),
    };
    account.apply_trade(&buy2).unwrap();

    let sell2 = Trade {
        id: "trade4".to_string(),
        user_id: "user123".to_string(),
        symbol: "ETHUSDT".to_string(),
        side: TradeSide::Sell,
        quantity: Decimal::from_str("1.0").unwrap(),
        price: Decimal::from_str("2950").unwrap(),
        trade_type: TradeType::Long,
        timestamp: Utc::now(),
        order_id: "order4".to_string(),
        status: TradeStatus::Executed,
        pnl: Some(Decimal::from_str("-50").unwrap()),
        commission: Some(Decimal::from_str("2.95").unwrap()),
    };
    account.apply_trade(&sell2).unwrap();

    // Calculate statistics
    let stats = account.get_statistics();

    assert_eq!(stats.total_trades, 4);
    assert_eq!(stats.winning_trades, 1);
    assert_eq!(stats.losing_trades, 1);
    assert_eq!(stats.win_rate, 0.5); // 1 win out of 2 completed trades
    assert!(stats.total_pnl > Decimal::zero()); // Net positive

    cleanup_test_db(db).await;
}

#[actix_web::test]
async fn test_paper_trading_risk_limits() {
    let db = setup_test_db().await;
    let storage = Storage::new_with_db(Some(db.clone())).await;

    // Create account with risk limits
    let mut account = PaperTradingAccount::new("user123".to_string(), 10000.0);
    account.max_position_size = 0.2; // Max 20% per position
    account.max_daily_loss = 500.0; // Max $500 daily loss

    // Try to open position larger than limit
    let large_trade = Trade {
        id: "trade1".to_string(),
        user_id: "user123".to_string(),
        symbol: "BTCUSDT".to_string(),
        side: TradeSide::Buy,
        quantity: Decimal::from_str("0.5").unwrap(), // Would be 50% of balance
        price: Decimal::from_str("45000").unwrap(),
        trade_type: TradeType::Long,
        timestamp: Utc::now(),
        order_id: "order1".to_string(),
        status: TradeStatus::Executed,
        pnl: None,
        commission: Some(Decimal::from_str("22.5").unwrap()),
    };

    let result = account.validate_risk_limits(&large_trade);
    assert!(result.is_err());
    assert!(result
        .unwrap_err()
        .to_string()
        .contains("exceeds maximum position size"));

    cleanup_test_db(db).await;
}

#[actix_web::test]
async fn test_paper_trading_multiple_positions() {
    let db = setup_test_db().await;
    let storage = Storage::new_with_db(Some(db.clone())).await;

    // Create account
    let mut account = PaperTradingAccount::new("user123".to_string(), 10000.0);

    // Open multiple positions
    let symbols = vec!["BTCUSDT", "ETHUSDT", "BNBUSDT"];
    let prices = vec![45000.0, 3000.0, 300.0];
    let quantities = vec![0.01, 0.1, 1.0];

    for i in 0..3 {
        let trade = Trade {
            id: format!("trade{}", i),
            user_id: "user123".to_string(),
            symbol: symbols[i].to_string(),
            side: TradeSide::Buy,
            quantity: Decimal::from_f64(quantities[i]).unwrap(),
            price: Decimal::from_f64(prices[i]).unwrap(),
            trade_type: TradeType::Long,
            timestamp: Utc::now(),
            order_id: format!("order{}", i),
            status: TradeStatus::Executed,
            pnl: None,
            commission: Some(Decimal::from_f64(prices[i] * quantities[i] * 0.001).unwrap()),
        };
        account.apply_trade(&trade).unwrap();
    }

    // Check all positions exist
    assert_eq!(account.positions.len(), 3);

    // Calculate total position value
    let total_value: f64 = account
        .positions
        .values()
        .map(|p| p.quantity.to_f64().unwrap() * p.current_price.to_f64().unwrap())
        .sum();

    assert!(total_value > 0.0);

    cleanup_test_db(db).await;
}
