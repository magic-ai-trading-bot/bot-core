mod common;

use binance_trading_bot::auth::models::User;
use binance_trading_bot::models::*;
use binance_trading_bot::storage::Storage;
use chrono::Utc;
use common::*;
use mongodb::bson::doc;

#[tokio::test]
async fn test_storage_connection() {
    let db = setup_test_db().await;
    let storage = Storage::new_with_db(Some(db.clone())).await;

    // Test connection is alive
    let result = storage.db.list_collection_names(None).await;
    assert!(result.is_ok());

    cleanup_test_db(db).await;
}

#[tokio::test]
async fn test_user_storage() {
    let db = setup_test_db().await;
    let storage = Storage::new_with_db(Some(db.clone())).await;

    // Create test user
    let user = User {
        id: None,
        email: "test@example.com".to_string(),
        password_hash: "hashed_password".to_string(),
        full_name: "Test User".to_string(),
        api_keys: vec![],
        created_at: Utc::now(),
        updated_at: Utc::now(),
        is_active: true,
        roles: vec!["user".to_string()],
    };

    // Save user
    let saved_user = storage.create_user(user).await.unwrap();
    assert!(saved_user.id.is_some());

    // Find user by email
    let found_user = storage
        .find_user_by_email("test@example.com")
        .await
        .unwrap();
    assert!(found_user.is_some());
    assert_eq!(found_user.unwrap().email, "test@example.com");

    // Find user by ID
    let user_id = saved_user.id.unwrap();
    let found_by_id = storage.find_user_by_id(&user_id.to_hex()).await.unwrap();
    assert!(found_by_id.is_some());

    cleanup_test_db(db).await;
}

#[tokio::test]
async fn test_trade_storage() {
    let db = setup_test_db().await;
    let storage = Storage::new_with_db(Some(db.clone())).await;

    // Create test trade
    let trade = doc! {
        "user_id": "user123",
        "symbol": "BTCUSDT",
        "side": "BUY",
        "quantity": 0.1,
        "price": 45000.0,
        "timestamp": Utc::now(),
        "order_id": "order123",
        "status": "executed"
    };

    // Save trade
    let result = storage.save_trade(trade.clone()).await;
    assert!(result.is_ok());

    // Find trades by user
    let user_trades = storage.get_user_trades("user123").await.unwrap();
    assert!(!user_trades.is_empty());
    assert_eq!(user_trades[0].get_str("symbol").unwrap(), "BTCUSDT");

    // Find trades by symbol
    let symbol_trades = storage.get_trades_by_symbol("BTCUSDT").await.unwrap();
    assert!(!symbol_trades.is_empty());

    cleanup_test_db(db).await;
}

#[tokio::test]
async fn test_candle_storage() {
    let db = setup_test_db().await;
    let storage = Storage::new_with_db(Some(db.clone())).await;

    // Create test candles
    let candles = vec![
        doc! {
            "symbol": "BTCUSDT",
            "interval": "1h",
            "open": 45000.0,
            "high": 45500.0,
            "low": 44800.0,
            "close": 45200.0,
            "volume": 1000.0,
            "open_time": Utc::now().timestamp_millis() - 3600000,
            "close_time": Utc::now().timestamp_millis() - 1,
        },
        doc! {
            "symbol": "BTCUSDT",
            "interval": "1h",
            "open": 45200.0,
            "high": 45600.0,
            "low": 45100.0,
            "close": 45400.0,
            "volume": 1200.0,
            "open_time": Utc::now().timestamp_millis(),
            "close_time": Utc::now().timestamp_millis() + 3599999,
        },
    ];

    // Save candles
    for candle in &candles {
        storage.save_candle(candle.clone()).await.unwrap();
    }

    // Query candles
    let stored_candles = storage
        .get_candles(
            "BTCUSDT",
            "1h",
            Utc::now().timestamp_millis() - 7200000,
            Utc::now().timestamp_millis() + 3600000,
        )
        .await
        .unwrap();

    assert_eq!(stored_candles.len(), 2);
    assert_eq!(stored_candles[0].get_f64("open").unwrap(), 45000.0);

    cleanup_test_db(db).await;
}

#[tokio::test]
async fn test_position_storage() {
    let db = setup_test_db().await;
    let storage = Storage::new_with_db(Some(db.clone())).await;

    // Create test position
    let position = doc! {
        "user_id": "user123",
        "symbol": "BTCUSDT",
        "side": "LONG",
        "quantity": 0.1,
        "entry_price": 45000.0,
        "current_price": 45500.0,
        "unrealized_pnl": 50.0,
        "created_at": Utc::now(),
        "status": "open"
    };

    // Save position
    storage.save_position(position).await.unwrap();

    // Get user positions
    let positions = storage.get_user_positions("user123").await.unwrap();
    assert!(!positions.is_empty());
    assert_eq!(positions[0].get_str("symbol").unwrap(), "BTCUSDT");

    // Update position
    let update = doc! {
        "$set": {
            "current_price": 46000.0,
            "unrealized_pnl": 100.0
        }
    };

    storage
        .update_position("user123", "BTCUSDT", update)
        .await
        .unwrap();

    // Verify update
    let updated_positions = storage.get_user_positions("user123").await.unwrap();
    assert_eq!(
        updated_positions[0].get_f64("current_price").unwrap(),
        46000.0
    );

    cleanup_test_db(db).await;
}

#[tokio::test]
async fn test_strategy_config_storage() {
    let db = setup_test_db().await;
    let storage = Storage::new_with_db(Some(db.clone())).await;

    // Create test strategy config
    let config = doc! {
        "strategy_id": "rsi_strategy_1",
        "name": "RSI Strategy",
        "type": "RSI",
        "parameters": {
            "period": 14,
            "overbought": 70,
            "oversold": 30
        },
        "enabled": true,
        "created_at": Utc::now()
    };

    // Save config
    storage.save_strategy_config_doc(config).await.unwrap();

    // Load config
    let loaded = storage
        .load_strategy_config_doc("rsi_strategy_1")
        .await
        .unwrap();
    assert!(loaded.is_some());
    assert_eq!(loaded.unwrap().get_str("type").unwrap(), "RSI");

    // List all configs
    let all_configs = storage.list_strategy_configs().await.unwrap();
    assert!(!all_configs.is_empty());

    cleanup_test_db(db).await;
}

#[tokio::test]
async fn test_api_key_storage() {
    let db = setup_test_db().await;
    let storage = Storage::new_with_db(Some(db.clone())).await;

    // Create user first
    let user = User {
        id: None,
        email: "apiuser@example.com".to_string(),
        password_hash: "hash".to_string(),
        full_name: "API User".to_string(),
        api_keys: vec![],
        created_at: Utc::now(),
        updated_at: Utc::now(),
        is_active: true,
        roles: vec!["user".to_string()],
    };

    let saved_user = storage.create_user(user).await.unwrap();
    let user_id = saved_user.id.unwrap().to_hex();

    // Add API key
    let api_key = ApiKey {
        key: "test_api_key_123".to_string(),
        secret: "test_secret_456".to_string(),
        exchange: "binance".to_string(),
        permissions: vec!["read".to_string(), "trade".to_string()],
        created_at: Utc::now(),
        is_active: true,
    };

    storage.add_api_key(&user_id, api_key).await.unwrap();

    // Verify API key was added
    let updated_user = storage.find_user_by_id(&user_id).await.unwrap().unwrap();
    assert_eq!(updated_user.api_keys.len(), 1);
    assert_eq!(updated_user.api_keys[0].key, "test_api_key_123");

    cleanup_test_db(db).await;
}

#[tokio::test]
async fn test_transaction_atomicity() {
    let db = setup_test_db().await;
    let storage = Storage::new_with_db(Some(db.clone())).await;

    // Start a session for transaction
    let mut session = storage.client.start_session(None).await.unwrap();
    session.start_transaction(None).await.unwrap();

    // Perform multiple operations in transaction
    let trade1 = doc! {
        "user_id": "user123",
        "symbol": "BTCUSDT",
        "side": "BUY",
        "quantity": 0.1,
        "price": 45000.0,
        "timestamp": Utc::now()
    };

    let trade2 = doc! {
        "user_id": "user123",
        "symbol": "ETHUSDT",
        "side": "BUY",
        "quantity": 1.0,
        "price": 3000.0,
        "timestamp": Utc::now()
    };

    // Insert trades
    storage
        .db
        .collection::<mongodb::bson::Document>("trades")
        .insert_one_with_session(&trade1, None, &mut session)
        .await
        .unwrap();

    storage
        .db
        .collection::<mongodb::bson::Document>("trades")
        .insert_one_with_session(&trade2, None, &mut session)
        .await
        .unwrap();

    // Commit transaction
    session.commit_transaction().await.unwrap();

    // Verify both trades exist
    let trades = storage.get_user_trades("user123").await.unwrap();
    assert_eq!(trades.len(), 2);

    cleanup_test_db(db).await;
}

#[tokio::test]
async fn test_index_creation() {
    let db = setup_test_db().await;
    let storage = Storage::new_with_db(Some(db.clone())).await;

    // Create indexes
    storage.create_indexes().await.unwrap();

    // Verify indexes exist
    let indexes = storage
        .db
        .collection::<mongodb::bson::Document>("users")
        .list_indexes(None)
        .await
        .unwrap();

    let index_names: Vec<String> = indexes
        .try_collect::<Vec<_>>()
        .await
        .unwrap()
        .into_iter()
        .filter_map(|idx| idx.get_str("name").ok().map(|s| s.to_string()))
        .collect();

    // Should have email index
    assert!(index_names.iter().any(|name| name.contains("email")));

    cleanup_test_db(db).await;
}

#[tokio::test]
async fn test_data_migration() {
    let db = setup_test_db().await;
    let storage = Storage::new_with_db(Some(db.clone())).await;

    // Insert old format data
    let old_trade = doc! {
        "user_id": "user123",
        "symbol": "BTCUSDT",
        "amount": 0.1, // Old field name
        "price": 45000.0,
        "timestamp": Utc::now()
    };

    storage
        .db
        .collection::<mongodb::bson::Document>("trades")
        .insert_one(&old_trade, None)
        .await
        .unwrap();

    // Run migration to rename 'amount' to 'quantity'
    storage
        .db
        .collection::<mongodb::bson::Document>("trades")
        .update_many(
            doc! { "amount": { "$exists": true } },
            doc! {
                "$rename": { "amount": "quantity" },
                "$set": { "migrated": true }
            },
            None,
        )
        .await
        .unwrap();

    // Verify migration
    let migrated = storage
        .db
        .collection::<mongodb::bson::Document>("trades")
        .find_one(doc! { "user_id": "user123" }, None)
        .await
        .unwrap()
        .unwrap();

    assert!(migrated.get("quantity").is_some());
    assert!(migrated.get("amount").is_none());
    assert_eq!(migrated.get_bool("migrated").unwrap(), true);

    cleanup_test_db(db).await;
}
