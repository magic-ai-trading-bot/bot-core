// Common test utilities and fixtures

// Removed unused imports
use chrono::Utc;
use mongodb::{Client, Database};
use serde_json::json;

pub async fn setup_test_db() -> Database {
    let client = Client::with_uri_str("mongodb://localhost:27017")
        .await
        .expect("Failed to connect to test MongoDB");

    let db_name = format!("test_trading_bot_{}", Utc::now().timestamp());
    client.database(&db_name)
}

pub async fn cleanup_test_db(db: Database) {
    db.drop(None).await.ok();
}

pub fn create_test_jwt(user_id: &str) -> String {
    // In real implementation, use proper JWT generation
    format!("test_jwt_token_{}", user_id)
}

pub fn sample_candle_data() -> serde_json::Value {
    json!({
        "open": 45000.0,
        "high": 45500.0,
        "low": 44800.0,
        "close": 45200.0,
        "volume": 1000.0,
        "open_time": 1701234567000i64,
        "close_time": 1701238167000i64
    })
}

pub fn sample_trade_request() -> serde_json::Value {
    json!({
        "symbol": "BTCUSDT",
        "side": "BUY",
        "type": "LIMIT",
        "quantity": 0.001,
        "price": 45000.0,
        "time_in_force": "GTC"
    })
}

#[macro_export]
macro_rules! assert_success_response {
    ($resp:expr) => {
        assert!(
            $resp.status().is_success(),
            "Expected success response, got: {:?}",
            $resp.status()
        );
    };
}

#[macro_export]
macro_rules! assert_error_response {
    ($resp:expr, $expected_status:expr) => {
        assert_eq!(
            $resp.status(),
            $expected_status,
            "Expected status {}, got: {:?}",
            $expected_status,
            $resp.status()
        );
    };
}
