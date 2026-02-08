// Additional tests to increase coverage for storage/mod.rs
// Focus on data transformation, serialization, error handling

use binance_trading_bot::binance::types::Kline;
use binance_trading_bot::config::DatabaseConfig;
use binance_trading_bot::storage::*;
use chrono::Utc;
use serde_json::json;

// Helper to create test storage with invalid MongoDB URL (uses in-memory fallback)
async fn create_test_storage() -> Storage {
    let config = DatabaseConfig {
        url: "invalid://localhost".to_string(),
        database_name: Some("test_db".to_string()),
        max_connections: 10,
        enable_logging: false,
    };
    Storage::new(&config)
        .await
        .expect("Failed to create storage")
}

#[tokio::test]
async fn test_storage_creation_without_feature() {
    let config = DatabaseConfig {
        url: "test://localhost".to_string(),
        database_name: None,
        max_connections: 5,
        enable_logging: true,
    };

    let storage = Storage::new(&config).await;
    assert!(storage.is_ok());
}

#[tokio::test]
async fn test_get_database_without_db() {
    let storage = create_test_storage().await;

    #[cfg(feature = "database")]
    {
        let db = storage.get_database();
        assert!(db.is_none());
    }

    #[cfg(not(feature = "database"))]
    {
        let db = storage.get_database();
        assert!(db.is_none());
    }
}

#[tokio::test]
async fn test_collection_getters_without_db() {
    let storage = create_test_storage().await;

    // All collection getters should return error when DB is not initialized
    assert!(storage.paper_trades().is_err());
    assert!(storage.portfolio_history().is_err());
    assert!(storage.ai_signals().is_err());
    assert!(storage.performance_metrics().is_err());
    assert!(storage.paper_trading_settings().is_err());
    assert!(storage.user_symbols().is_err());
    assert!(storage.trade_analyses().is_err());
    assert!(storage.config_suggestions().is_err());
}

// Test TradeRecord serialization/deserialization
#[test]
fn test_trade_record_serialization_full() {
    let trade = TradeRecord {
        id: None,
        symbol: "ETHUSDT".to_string(),
        side: "SELL".to_string(),
        quantity: 2.5,
        entry_price: 3500.0,
        exit_price: Some(3450.0),
        stop_loss: Some(3600.0),
        take_profit: Some(3400.0),
        entry_time: 1701234567000,
        exit_time: Some(1701238167000),
        pnl: Some(-125.0),
        status: "closed".to_string(),
        strategy_used: Some("RSI_MACD".to_string()),
    };

    let json = serde_json::to_string(&trade).unwrap();
    let deserialized: TradeRecord = serde_json::from_str(&json).unwrap();

    assert_eq!(deserialized.symbol, "ETHUSDT");
    assert_eq!(deserialized.side, "SELL");
    assert_eq!(deserialized.quantity, 2.5);
    assert_eq!(deserialized.pnl, Some(-125.0));
}

#[test]
fn test_trade_record_with_none_values() {
    let trade = TradeRecord {
        id: None,
        symbol: "BTCUSDT".to_string(),
        side: "BUY".to_string(),
        quantity: 1.0,
        entry_price: 50000.0,
        exit_price: None,
        stop_loss: None,
        take_profit: None,
        entry_time: 1701234567000,
        exit_time: None,
        pnl: None,
        status: "open".to_string(),
        strategy_used: None,
    };

    let json = serde_json::to_string(&trade).unwrap();
    assert!(json.contains("\"exit_price\":null"));
    assert!(json.contains("\"pnl\":null"));
}

#[test]
fn test_performance_stats_default() {
    let stats = PerformanceStats::default();

    assert_eq!(stats.total_trades, 0);
    assert_eq!(stats.winning_trades, 0);
    assert_eq!(stats.losing_trades, 0);
    assert_eq!(stats.win_rate, 0.0);
    assert_eq!(stats.total_pnl, 0.0);
    assert_eq!(stats.avg_pnl, 0.0);
    assert_eq!(stats.max_win, 0.0);
    assert_eq!(stats.max_loss, 0.0);
}

#[test]
fn test_performance_stats_serialization() {
    let stats = PerformanceStats {
        total_trades: 100,
        winning_trades: 65,
        losing_trades: 35,
        win_rate: 65.0,
        total_pnl: 1250.50,
        avg_pnl: 12.505,
        max_win: 350.0,
        max_loss: -180.0,
    };

    let json = serde_json::to_string(&stats).unwrap();
    let deserialized: PerformanceStats = serde_json::from_str(&json).unwrap();

    assert_eq!(deserialized.total_trades, 100);
    assert_eq!(deserialized.win_rate, 65.0);
    assert_eq!(deserialized.max_loss, -180.0);
}

#[test]
fn test_paper_trading_record_serialization() {
    let now = Utc::now();
    let record = PaperTradingRecord {
        id: None,
        trade_id: "trade_123".to_string(),
        symbol: "BTCUSDT".to_string(),
        trade_type: "Long".to_string(),
        status: "Open".to_string(),
        entry_price: 45000.0,
        exit_price: None,
        quantity: 0.5,
        leverage: 10,
        pnl: None,
        pnl_percentage: 0.0,
        trading_fees: 22.5,
        funding_fees: 0.0,
        open_time: now,
        close_time: None,
        ai_signal_id: Some("signal_456".to_string()),
        ai_confidence: Some(0.85),
        close_reason: None,
        created_at: now,
    };

    let json = serde_json::to_string(&record).unwrap();
    let deserialized: PaperTradingRecord = serde_json::from_str(&json).unwrap();

    assert_eq!(deserialized.trade_id, "trade_123");
    assert_eq!(deserialized.symbol, "BTCUSDT");
    assert_eq!(deserialized.leverage, 10);
    assert_eq!(deserialized.ai_confidence, Some(0.85));
}

#[test]
fn test_portfolio_history_record_serialization() {
    let now = Utc::now();
    let record = PortfolioHistoryRecord {
        id: None,
        timestamp: now,
        current_balance: 10000.0,
        equity: 10500.0,
        margin_used: 2000.0,
        free_margin: 8000.0,
        total_pnl: 500.0,
        total_pnl_percentage: 5.0,
        total_trades: 25,
        win_rate: 68.0,
        profit_factor: 1.8,
        max_drawdown: -250.0,
        max_drawdown_percentage: -2.5,
        open_positions: 3,
        created_at: now,
    };

    let json = serde_json::to_string(&record).unwrap();
    let deserialized: PortfolioHistoryRecord = serde_json::from_str(&json).unwrap();

    assert_eq!(deserialized.current_balance, 10000.0);
    assert_eq!(deserialized.total_pnl, 500.0);
    assert_eq!(deserialized.win_rate, 68.0);
    assert_eq!(deserialized.open_positions, 3);
}

#[test]
fn test_ai_signal_record_full() {
    let now = Utc::now();
    let record = AISignalRecord {
        id: None,
        signal_id: "sig_123".to_string(),
        symbol: "ETHUSDT".to_string(),
        signal_type: "Long".to_string(),
        confidence: 0.92,
        reasoning: "Strong bullish momentum".to_string(),
        entry_price: 3500.0,
        trend_direction: "Bullish".to_string(),
        trend_strength: 0.85,
        volatility: 0.15,
        risk_score: 0.25,
        executed: true,
        trade_id: Some("trade_789".to_string()),
        created_at: now,
        timestamp: now,
        outcome: Some("win".to_string()),
        actual_pnl: Some(125.50),
        pnl_percentage: Some(3.58),
        exit_price: Some(3625.0),
        close_reason: Some("TakeProfit".to_string()),
        closed_at: Some(now),
    };

    let json = serde_json::to_string(&record).unwrap();
    let deserialized: AISignalRecord = serde_json::from_str(&json).unwrap();

    assert_eq!(deserialized.signal_id, "sig_123");
    assert_eq!(deserialized.confidence, 0.92);
    assert_eq!(deserialized.outcome, Some("win".to_string()));
    assert_eq!(deserialized.actual_pnl, Some(125.50));
}

#[test]
fn test_ai_signal_record_with_defaults() {
    let now = Utc::now();

    // Test record with default optional fields
    let json = json!({
        "signal_id": "sig_pending",
        "symbol": "BTCUSDT",
        "signal_type": "Short",
        "confidence": 0.75,
        "reasoning": "Test",
        "entry_price": 45000.0,
        "trend_direction": "Bearish",
        "trend_strength": 0.7,
        "volatility": 0.2,
        "risk_score": 0.3,
        "executed": false,
        "created_at": now,
        "timestamp": now
    });

    let record: AISignalRecord = serde_json::from_value(json).unwrap();
    assert_eq!(record.signal_id, "sig_pending");
    assert_eq!(record.outcome, None);
    assert_eq!(record.actual_pnl, None);
    assert_eq!(record.closed_at, None);
}

#[test]
fn test_performance_metrics_record_serialization() {
    let now = Utc::now();
    let record = PerformanceMetricsRecord {
        id: None,
        date: now,
        total_trades: 50,
        winning_trades: 35,
        losing_trades: 15,
        win_rate: 70.0,
        average_win: 125.0,
        average_loss: -65.0,
        largest_win: 450.0,
        largest_loss: -220.0,
        profit_factor: 2.1,
        sharpe_ratio: 1.6,
        max_drawdown: -380.0,
        max_drawdown_percentage: -3.8,
        total_pnl: 1850.0,
        daily_pnl: 185.0,
        created_at: now,
    };

    let json = serde_json::to_string(&record).unwrap();
    let deserialized: PerformanceMetricsRecord = serde_json::from_str(&json).unwrap();

    assert_eq!(deserialized.total_trades, 50);
    assert_eq!(deserialized.winning_trades, 35);
    assert_eq!(deserialized.sharpe_ratio, 1.6);
    assert_eq!(deserialized.daily_pnl, 185.0);
}

#[tokio::test]
async fn test_store_analysis_without_db() {
    let storage = create_test_storage().await;

    use std::collections::HashMap;
    let analysis = binance_trading_bot::market_data::analyzer::MultiTimeframeAnalysis {
        symbol: "BTCUSDT".to_string(),
        timestamp: 1704067200000, // 2024-01-01 00:00:00 UTC in milliseconds
        overall_signal: binance_trading_bot::market_data::analyzer::TradingSignal::Hold,
        overall_confidence: 0.65,
        timeframe_signals: HashMap::new(),
        entry_price: Some(45000.0),
        stop_loss: Some(44000.0),
        take_profit: Some(47000.0),
        risk_reward_ratio: Some(1.5),
    };

    // Should not fail, just logs without database
    let result = storage.store_analysis(&analysis).await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_get_latest_analysis_without_db() {
    let storage = create_test_storage().await;
    let result = storage.get_latest_analysis("BTCUSDT").await;

    assert!(result.is_ok());
    assert!(result.unwrap().is_none());
}

#[tokio::test]
async fn test_get_analysis_history_without_db() {
    let storage = create_test_storage().await;
    let result = storage.get_analysis_history("ETHUSDT", Some(50)).await;

    assert!(result.is_ok());
    assert_eq!(result.unwrap().len(), 0);
}

#[tokio::test]
async fn test_store_trade_record_without_db() {
    let storage = create_test_storage().await;

    let trade = TradeRecord {
        id: None,
        symbol: "BTCUSDT".to_string(),
        side: "BUY".to_string(),
        quantity: 1.0,
        entry_price: 45000.0,
        exit_price: Some(46000.0),
        stop_loss: Some(44000.0),
        take_profit: Some(47000.0),
        entry_time: 1701234567000,
        exit_time: Some(1701238167000),
        pnl: Some(1000.0),
        status: "closed".to_string(),
        strategy_used: Some("MACD".to_string()),
    };

    // Should not fail, just logs
    let result = storage.store_trade_record(&trade).await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_get_trade_history_without_db() {
    let storage = create_test_storage().await;

    // Test with symbol filter
    let result1 = storage.get_trade_history(Some("BTCUSDT"), Some(100)).await;
    assert!(result1.is_ok());
    assert_eq!(result1.unwrap().len(), 0);

    // Test without symbol filter
    let result2 = storage.get_trade_history(None, None).await;
    assert!(result2.is_ok());
    assert_eq!(result2.unwrap().len(), 0);
}

#[tokio::test]
async fn test_get_performance_stats_without_db() {
    let storage = create_test_storage().await;
    let result = storage.get_performance_stats().await;

    assert!(result.is_ok());
    let stats = result.unwrap();

    // Should return default stats
    assert_eq!(stats.total_trades, 0);
    assert_eq!(stats.total_pnl, 0.0);
}

#[tokio::test]
async fn test_store_market_data_with_valid_klines() {
    let storage = create_test_storage().await;

    let klines = vec![Kline {
        open_time: 1701234567000,
        close_time: 1701238167000,
        open: "45000.00".to_string(),
        high: "45500.00".to_string(),
        low: "44800.00".to_string(),
        close: "45200.00".to_string(),
        volume: "100.123".to_string(),
        quote_asset_volume: "4510123.45".to_string(),
        number_of_trades: 1000,
        taker_buy_base_asset_volume: "50.123".to_string(),
        taker_buy_quote_asset_volume: "2255123.45".to_string(),
        ignore: "0".to_string(),
    }];

    let result = storage.store_market_data("BTCUSDT", "1h", &klines).await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_store_market_data_with_invalid_prices() {
    let storage = create_test_storage().await;

    // Test with invalid price formats
    let klines = vec![Kline {
        open_time: 1701234567000,
        close_time: 1701238167000,
        open: "invalid".to_string(),
        high: "45500.00".to_string(),
        low: "44800.00".to_string(),
        close: "45200.00".to_string(),
        volume: "100.123".to_string(),
        quote_asset_volume: "4510123.45".to_string(),
        number_of_trades: 1000,
        taker_buy_base_asset_volume: "50.123".to_string(),
        taker_buy_quote_asset_volume: "2255123.45".to_string(),
        ignore: "0".to_string(),
    }];

    // Should handle gracefully without crashing
    let result = storage.store_market_data("BTCUSDT", "1h", &klines).await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_store_market_data_with_zero_prices() {
    let storage = create_test_storage().await;

    let klines = vec![Kline {
        open_time: 1701234567000,
        close_time: 1701238167000,
        open: "0.0".to_string(),
        high: "0.0".to_string(),
        low: "0.0".to_string(),
        close: "0.0".to_string(),
        volume: "100.123".to_string(),
        quote_asset_volume: "4510123.45".to_string(),
        number_of_trades: 1000,
        taker_buy_base_asset_volume: "50.123".to_string(),
        taker_buy_quote_asset_volume: "2255123.45".to_string(),
        ignore: "0".to_string(),
    }];

    // Should skip invalid klines with zero prices
    let result = storage.store_market_data("BTCUSDT", "1h", &klines).await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_get_market_data_without_db() {
    let storage = create_test_storage().await;
    let result = storage.get_market_data("BTCUSDT", "1h", Some(500)).await;

    assert!(result.is_ok());
    assert_eq!(result.unwrap().len(), 0);
}

#[tokio::test]
async fn test_store_price_history_without_db() {
    let storage = create_test_storage().await;

    let result = storage
        .store_price_history("BTCUSDT", 45000.0, 1000000.0, 500.0, 1.12)
        .await;

    assert!(result.is_ok());
}

#[tokio::test]
async fn test_load_user_symbols_without_db() {
    let storage = create_test_storage().await;
    let result = storage.load_user_symbols().await;

    assert!(result.is_ok());
    assert_eq!(result.unwrap().len(), 0);
}

#[tokio::test]
async fn test_save_user_symbols_without_db() {
    let storage = create_test_storage().await;
    let symbols = vec!["BTCUSDT".to_string(), "ETHUSDT".to_string()];

    let result = storage.save_user_symbols(&symbols).await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_add_remove_user_symbol_without_db() {
    let storage = create_test_storage().await;

    // Add symbol
    let result1 = storage.add_user_symbol("BNBUSDT").await;
    assert!(result1.is_ok());

    // Remove symbol
    let result2 = storage.remove_user_symbol("BNBUSDT").await;
    assert!(result2.is_ok());
}

#[tokio::test]
async fn test_get_paper_trades_history_without_db() {
    let storage = create_test_storage().await;
    let result = storage.get_paper_trades_history(Some(100)).await;

    // Should return error because DB is not initialized
    assert!(result.is_err());
}

#[tokio::test]
async fn test_get_portfolio_history_without_db() {
    let storage = create_test_storage().await;

    // Test with days filter
    let result1 = storage.get_portfolio_history(Some(7)).await;
    assert!(result1.is_err());

    // Test without filter
    let result2 = storage.get_portfolio_history(None).await;
    assert!(result2.is_err());
}

#[tokio::test]
async fn test_get_ai_signals_history_without_db() {
    let storage = create_test_storage().await;

    // Test with symbol filter
    let result1 = storage
        .get_ai_signals_history(Some("BTCUSDT"), Some(50))
        .await;
    assert!(result1.is_err());

    // Test without filter
    let result2 = storage.get_ai_signals_history(None, None).await;
    assert!(result2.is_err());
}

#[tokio::test]
async fn test_get_latest_signals_per_symbol_without_db() {
    let storage = create_test_storage().await;
    let result = storage.get_latest_signals_per_symbol().await;

    assert!(result.is_err());
}

#[tokio::test]
async fn test_get_trade_analyses_without_db() {
    let storage = create_test_storage().await;

    // Test only losing trades
    let result1 = storage.get_trade_analyses(true, Some(50)).await;
    assert!(result1.is_ok());
    assert_eq!(result1.unwrap().len(), 0);

    // Test all trades
    let result2 = storage.get_trade_analyses(false, None).await;
    assert!(result2.is_ok());
    assert_eq!(result2.unwrap().len(), 0);
}

#[tokio::test]
async fn test_get_trade_analysis_by_id_without_db() {
    let storage = create_test_storage().await;
    let result = storage.get_trade_analysis_by_id("trade_123").await;

    assert!(result.is_ok());
    assert!(result.unwrap().is_none());
}

#[tokio::test]
async fn test_get_config_suggestions_without_db() {
    let storage = create_test_storage().await;
    let result = storage.get_config_suggestions(Some(50)).await;

    assert!(result.is_ok());
    assert_eq!(result.unwrap().len(), 0);
}

#[tokio::test]
async fn test_get_latest_config_suggestion_without_db() {
    let storage = create_test_storage().await;
    let result = storage.get_latest_config_suggestion().await;

    assert!(result.is_ok());
    assert!(result.unwrap().is_none());
}

// Test edge cases for helper functions in mod.rs tests
#[test]
fn test_multiple_kline_formats() {
    let kline1 = Kline {
        open_time: 1701234567000,
        close_time: 1701238167000,
        open: "100.5".to_string(),
        high: "101.2".to_string(),
        low: "99.8".to_string(),
        close: "100.9".to_string(),
        volume: "1000.0".to_string(),
        quote_asset_volume: "100500.0".to_string(),
        number_of_trades: 500,
        taker_buy_base_asset_volume: "500.0".to_string(),
        taker_buy_quote_asset_volume: "50250.0".to_string(),
        ignore: "0".to_string(),
    };

    // Test serialization
    let json = serde_json::to_string(&kline1).unwrap();
    assert!(json.contains("100.5"));

    // Test deserialization
    let kline2: Kline = serde_json::from_str(&json).unwrap();
    assert_eq!(kline2.open, "100.5");
    assert_eq!(kline2.number_of_trades, 500);
}
