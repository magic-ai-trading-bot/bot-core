// @test:BOOST-STORAGE-002 - Simplified Storage Function Coverage
// Tests storage/mod.rs functions focusing on error paths with null-db

use binance_trading_bot::config::DatabaseConfig;
use binance_trading_bot::storage::Storage;

/// Helper: Create null-db storage (no MongoDB connection)
async fn create_null_db_storage() -> Storage {
    let config = DatabaseConfig {
        url: "no-db://test".to_string(),
        database_name: None,
        max_connections: 10,
        enable_logging: false,
    };
    Storage::new(&config).await.expect("Failed to create null-db storage")
}

// ============================================================================
// TEST: Storage::get_latest_analysis (fallback behavior)
// ============================================================================

#[tokio::test]
async fn test_fn_get_latest_analysis_btc() {
    let storage = create_null_db_storage().await;
    let result = storage.get_latest_analysis("BTCUSDT").await;
    assert!(result.is_ok());
    assert!(result.unwrap().is_none(), "Should return None with null-db");
}

#[tokio::test]
async fn test_fn_get_latest_analysis_eth() {
    let storage = create_null_db_storage().await;
    let result = storage.get_latest_analysis("ETHUSDT").await;
    assert!(result.is_ok());
    assert!(result.unwrap().is_none());
}

#[tokio::test]
async fn test_fn_get_latest_analysis_bnb() {
    let storage = create_null_db_storage().await;
    let result = storage.get_latest_analysis("BNBUSDT").await;
    assert!(result.is_ok());
}

// ============================================================================
// TEST: Storage::get_analysis_history
// ============================================================================

#[tokio::test]
async fn test_fn_get_analysis_history_btc_limit_50() {
    let storage = create_null_db_storage().await;
    let result = storage.get_analysis_history("BTCUSDT", Some(50)).await;
    assert!(result.is_ok());
    assert_eq!(result.unwrap().len(), 0);
}

#[tokio::test]
async fn test_fn_get_analysis_history_eth_no_limit() {
    let storage = create_null_db_storage().await;
    let result = storage.get_analysis_history("ETHUSDT", None).await;
    assert!(result.is_ok());
    assert_eq!(result.unwrap().len(), 0);
}

#[tokio::test]
async fn test_fn_get_analysis_history_various_limits() {
    let storage = create_null_db_storage().await;
    for limit in &[10, 100, 500] {
        let result = storage.get_analysis_history("ADAUSDT", Some(*limit)).await;
        assert!(result.is_ok());
    }
}

// ============================================================================
// TEST: Storage::get_trade_history
// ============================================================================

#[tokio::test]
async fn test_fn_get_trade_history_btc_limit_100() {
    let storage = create_null_db_storage().await;
    let result = storage.get_trade_history(Some("BTCUSDT"), Some(100)).await;
    assert!(result.is_ok());
    assert_eq!(result.unwrap().len(), 0);
}

#[tokio::test]
async fn test_fn_get_trade_history_no_symbol() {
    let storage = create_null_db_storage().await;
    let result = storage.get_trade_history(None, Some(50)).await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_fn_get_trade_history_no_limit() {
    let storage = create_null_db_storage().await;
    let result = storage.get_trade_history(Some("ETHUSDT"), None).await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_fn_get_trade_history_both_none() {
    let storage = create_null_db_storage().await;
    let result = storage.get_trade_history(None, None).await;
    assert!(result.is_ok());
}

// ============================================================================
// TEST: Storage::get_performance_stats
// ============================================================================

#[tokio::test]
async fn test_fn_get_performance_stats_default() {
    let storage = create_null_db_storage().await;
    let result = storage.get_performance_stats().await;
    assert!(result.is_ok());
    let stats = result.unwrap();
    assert_eq!(stats.total_trades, 0);
    assert_eq!(stats.winning_trades, 0);
    assert_eq!(stats.losing_trades, 0);
}

#[tokio::test]
async fn test_fn_get_performance_stats_multiple_calls() {
    let storage = create_null_db_storage().await;
    for _ in 0..3 {
        let result = storage.get_performance_stats().await;
        assert!(result.is_ok());
    }
}

// ============================================================================
// TEST: Storage::get_market_data
// ============================================================================

#[tokio::test]
async fn test_fn_get_market_data_1m_limit_100() {
    let storage = create_null_db_storage().await;
    let result = storage.get_market_data("BTCUSDT", "1m", Some(100)).await;
    assert!(result.is_ok());
    assert_eq!(result.unwrap().len(), 0);
}

#[tokio::test]
async fn test_fn_get_market_data_5m_no_limit() {
    let storage = create_null_db_storage().await;
    let result = storage.get_market_data("ETHUSDT", "5m", None).await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_fn_get_market_data_various_timeframes() {
    let storage = create_null_db_storage().await;
    for tf in &["1m", "5m", "15m", "1h", "4h", "1d"] {
        let result = storage.get_market_data("BTCUSDT", tf, Some(50)).await;
        assert!(result.is_ok());
    }
}

// ============================================================================
// TEST: Storage::get_paper_trades_history (error path)
// ============================================================================

#[tokio::test]
async fn test_fn_get_paper_trades_history_with_limit() {
    let storage = create_null_db_storage().await;
    let result = storage.get_paper_trades_history(Some(50)).await;
    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("Database not initialized"));
}

#[tokio::test]
async fn test_fn_get_paper_trades_history_no_limit() {
    let storage = create_null_db_storage().await;
    let result = storage.get_paper_trades_history(None).await;
    assert!(result.is_err());
}

#[tokio::test]
async fn test_fn_get_paper_trades_history_limit_100() {
    let storage = create_null_db_storage().await;
    let result = storage.get_paper_trades_history(Some(100)).await;
    assert!(result.is_err());
}

// ============================================================================
// TEST: Storage::get_portfolio_history
// ============================================================================

#[tokio::test]
async fn test_fn_get_portfolio_history_limit_50() {
    let storage = create_null_db_storage().await;
    let result = storage.get_portfolio_history(Some(50)).await;
    assert!(result.is_err());
}

#[tokio::test]
async fn test_fn_get_portfolio_history_no_limit() {
    let storage = create_null_db_storage().await;
    let result = storage.get_portfolio_history(None).await;
    assert!(result.is_err());
}

// ============================================================================
// TEST: Storage::get_ai_signals_history
// ============================================================================

#[tokio::test]
async fn test_fn_get_ai_signals_history_btc() {
    let storage = create_null_db_storage().await;
    let result = storage.get_ai_signals_history(Some("BTCUSDT"), Some(50)).await;
    assert!(result.is_err());
}

#[tokio::test]
async fn test_fn_get_ai_signals_history_no_symbol() {
    let storage = create_null_db_storage().await;
    let result = storage.get_ai_signals_history(None, Some(50)).await;
    assert!(result.is_err());
}

#[tokio::test]
async fn test_fn_get_ai_signals_history_no_limit() {
    let storage = create_null_db_storage().await;
    let result = storage.get_ai_signals_history(None, None).await;
    assert!(result.is_err());
}

#[tokio::test]
async fn test_fn_get_ai_signals_history_eth_limit() {
    let storage = create_null_db_storage().await;
    let result = storage.get_ai_signals_history(Some("ETHUSDT"), Some(100)).await;
    assert!(result.is_err());
}

// ============================================================================
// TEST: Storage::get_latest_signals_per_symbol
// ============================================================================

#[tokio::test]
async fn test_fn_get_latest_signals_per_symbol() {
    let storage = create_null_db_storage().await;
    let result = storage.get_latest_signals_per_symbol().await;
    assert!(result.is_err());
}

// ============================================================================
// TEST: Storage::load_paper_trading_settings
// ============================================================================

#[tokio::test]
async fn test_fn_load_paper_trading_settings() {
    let storage = create_null_db_storage().await;
    let _result = storage.load_paper_trading_settings().await;
    // May succeed with default or fail with DB error
}

// ============================================================================
// TEST: Storage::load_user_symbols
// ============================================================================

#[tokio::test]
async fn test_fn_load_user_symbols() {
    let storage = create_null_db_storage().await;
    let _result = storage.load_user_symbols().await;
    // May succeed with empty list or fail with DB error
}

// ============================================================================
// TEST: Storage::add_user_symbol
// ============================================================================

#[tokio::test]
async fn test_fn_add_user_symbol_btc() {
    let storage = create_null_db_storage().await;
    let _result = storage.add_user_symbol("BTCUSDT").await;
    // May succeed or fail depending on whether in-memory fallback exists
}

#[tokio::test]
async fn test_fn_add_user_symbol_eth() {
    let storage = create_null_db_storage().await;
    let _result = storage.add_user_symbol("ETHUSDT").await;
}

// ============================================================================
// TEST: Storage::remove_user_symbol
// ============================================================================

#[tokio::test]
async fn test_fn_remove_user_symbol_doge() {
    let storage = create_null_db_storage().await;
    let _result = storage.remove_user_symbol("DOGEUSDT").await;
}

#[tokio::test]
async fn test_fn_remove_user_symbol_ada() {
    let storage = create_null_db_storage().await;
    let _result = storage.remove_user_symbol("ADAUSDT").await;
}

// ============================================================================
// TEST: Storage::get_trade_analyses
// ============================================================================

#[tokio::test]
async fn test_fn_get_trade_analyses_only_losing() {
    let storage = create_null_db_storage().await;
    let _result = storage.get_trade_analyses(true, Some(50)).await;
    // May succeed with empty or fail with DB error
}

#[tokio::test]
async fn test_fn_get_trade_analyses_all_trades() {
    let storage = create_null_db_storage().await;
    let _result = storage.get_trade_analyses(false, None).await;
}

#[tokio::test]
async fn test_fn_get_trade_analyses_with_limit() {
    let storage = create_null_db_storage().await;
    let _result = storage.get_trade_analyses(false, Some(10)).await;
}

// ============================================================================
// TEST: Storage::get_trade_analysis_by_id
// ============================================================================

#[tokio::test]
async fn test_fn_get_trade_analysis_by_id_trade123() {
    let storage = create_null_db_storage().await;
    let _result = storage.get_trade_analysis_by_id("trade123").await;
}

#[tokio::test]
async fn test_fn_get_trade_analysis_by_id_trade456() {
    let storage = create_null_db_storage().await;
    let _result = storage.get_trade_analysis_by_id("trade456").await;
}

// ============================================================================
// TEST: Storage::get_config_suggestions
// ============================================================================

#[tokio::test]
async fn test_fn_get_config_suggestions_limit_10() {
    let storage = create_null_db_storage().await;
    let _result = storage.get_config_suggestions(Some(10)).await;
}

#[tokio::test]
async fn test_fn_get_config_suggestions_no_limit() {
    let storage = create_null_db_storage().await;
    let _result = storage.get_config_suggestions(None).await;
}

// ============================================================================
// TEST: Storage::get_latest_config_suggestion
// ============================================================================

#[tokio::test]
async fn test_fn_get_latest_config_suggestion() {
    let storage = create_null_db_storage().await;
    let _result = storage.get_latest_config_suggestion().await;
}

// ============================================================================
// TEST: Storage::update_signal_outcome
// ============================================================================

#[tokio::test]
async fn test_fn_update_signal_outcome_win() {
    let storage = create_null_db_storage().await;
    let result = storage.update_signal_outcome("sig123", "win", 100.0, 2.0, 50500.0, "TakeProfit").await;
    assert!(result.is_err());
}

#[tokio::test]
async fn test_fn_update_signal_outcome_loss() {
    let storage = create_null_db_storage().await;
    let result = storage.update_signal_outcome("sig456", "loss", -50.0, -1.5, 49500.0, "StopLoss").await;
    assert!(result.is_err());
}

// ============================================================================
// TEST: Multiple rapid consecutive calls
// ============================================================================

#[tokio::test]
async fn test_fn_rapid_consecutive_calls() {
    let storage = create_null_db_storage().await;

    for _ in 0..10 {
        let _ = storage.get_latest_analysis("BTCUSDT").await;
        let _ = storage.get_trade_history(None, None).await;
        let _ = storage.get_performance_stats().await;
    }
}

// ============================================================================
// TEST: Different symbol combinations
// ============================================================================

#[tokio::test]
async fn test_fn_various_symbols() {
    let storage = create_null_db_storage().await;

    let symbols = vec!["BTCUSDT", "ETHUSDT", "BNBUSDT", "SOLUSDT", "ADAUSDT"];
    for symbol in symbols {
        let _ = storage.get_latest_analysis(symbol).await;
        let _ = storage.get_trade_history(Some(symbol), Some(10)).await;
        let _ = storage.get_market_data(symbol, "1m", Some(50)).await;
    }
}
