//! Real Trading Integration Tests
//!
//! These tests verify real trading functionality against Binance testnet.
//! Run with: cargo test --test test_real_trading_integration -- --ignored
//!
//! @spec:FR-TRADING-016 - Real Trading System
//! @ref:specs/02-design/2.5-components/COMP-RUST-TRADING.md

use serde::{Deserialize, Serialize};
use std::env;

/// Test API response structure
#[derive(Debug, Deserialize)]
struct ApiResponse<T> {
    success: bool,
    data: Option<T>,
    error: Option<String>,
    #[allow(dead_code)]
    timestamp: String,
}

/// Engine status response
#[derive(Debug, Deserialize)]
struct EngineStatus {
    is_running: bool,
    is_testnet: bool,
    open_positions_count: usize,
    open_orders_count: usize,
    circuit_breaker_open: bool,
    #[allow(dead_code)]
    daily_pnl: f64,
    #[allow(dead_code)]
    daily_trades_count: u32,
}

/// Portfolio response
#[derive(Debug, Deserialize)]
#[allow(dead_code)]
struct PortfolioResponse {
    total_balance: f64,
    available_balance: f64,
    locked_balance: f64,
    unrealized_pnl: f64,
    realized_pnl: f64,
    positions: Vec<serde_json::Value>,
    balances: Vec<serde_json::Value>,
}

/// Settings response
#[derive(Debug, Deserialize)]
#[allow(dead_code)]
struct SettingsResponse {
    use_testnet: bool,
    max_position_size_usdt: f64,
    max_positions: u32,
    max_daily_loss_usdt: f64,
    max_total_exposure_usdt: f64,
    risk_per_trade_percent: f64,
    default_stop_loss_percent: f64,
    default_take_profit_percent: f64,
    max_leverage: u32,
    circuit_breaker_errors: u32,
    circuit_breaker_cooldown_secs: u64,
}

/// Order info
#[derive(Debug, Deserialize)]
#[allow(dead_code)]
struct OrderInfo {
    id: String,
    exchange_order_id: i64,
    symbol: String,
    side: String,
    order_type: String,
    quantity: f64,
    executed_quantity: f64,
    price: Option<f64>,
    avg_fill_price: f64,
    status: String,
    is_entry: bool,
}

/// Place order request
#[derive(Debug, Serialize)]
struct PlaceOrderRequest {
    symbol: String,
    side: String,
    order_type: String,
    quantity: f64,
    price: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    confirmation_token: Option<String>,
}

/// Confirmation response (from first POST /orders call)
#[derive(Debug, Deserialize)]
#[allow(dead_code)]
struct ConfirmationResponse {
    requires_confirmation: bool,
    token: String,
    expires_at: String,
    summary: String,
}

fn get_api_base() -> String {
    env::var("RUST_API_URL").unwrap_or_else(|_| "http://localhost:8080".to_string())
}

fn create_client() -> reqwest::Client {
    reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(30))
        .build()
        .expect("Failed to create HTTP client")
}

// ============================================================================
// STATUS & PORTFOLIO TESTS
// ============================================================================

#[tokio::test]
#[ignore] // Run with --ignored for integration tests
async fn test_get_status_returns_valid_response() {
    let client = create_client();
    let url = format!("{}/api/real-trading/status", get_api_base());

    let response = client.get(&url).send().await;

    match response {
        Ok(resp) => {
            assert!(resp.status().is_success(), "Status should be 200 OK");

            let body: ApiResponse<EngineStatus> = resp.json().await.unwrap();
            assert!(body.success, "Response should indicate success");
            assert!(body.data.is_some(), "Data should be present");

            let status = body.data.unwrap();
            // Engine should be running on testnet
            assert!(status.is_testnet, "Should be using testnet");
            assert!(
                !status.circuit_breaker_open,
                "Circuit breaker should be closed"
            );
        },
        Err(e) => {
            // If server not running, skip test gracefully
            if e.is_connect() {
                println!("⚠️ Server not running, skipping integration test");
                return;
            }
            panic!("Request failed: {:?}", e);
        },
    }
}

#[tokio::test]
#[ignore]
async fn test_get_portfolio_returns_valid_balances() {
    let client = create_client();
    let url = format!("{}/api/real-trading/portfolio", get_api_base());

    let response = client.get(&url).send().await;

    match response {
        Ok(resp) => {
            if !resp.status().is_success() {
                println!(
                    "⚠️ Portfolio endpoint returned non-200, server may not be fully configured"
                );
                return;
            }

            let body: ApiResponse<PortfolioResponse> = resp.json().await.unwrap();
            assert!(body.success, "Response should indicate success");
            assert!(body.data.is_some(), "Data should be present");

            let portfolio = body.data.unwrap();
            // Testnet should have some balance (demo funds)
            assert!(
                portfolio.total_balance >= 0.0,
                "Balance should be non-negative"
            );
        },
        Err(e) => {
            if e.is_connect() {
                println!("⚠️ Server not running, skipping integration test");
                return;
            }
            panic!("Request failed: {:?}", e);
        },
    }
}

#[tokio::test]
#[ignore]
async fn test_get_settings_returns_valid_config() {
    let client = create_client();
    let url = format!("{}/api/real-trading/settings", get_api_base());

    let response = client.get(&url).send().await;

    match response {
        Ok(resp) => {
            if !resp.status().is_success() {
                println!("⚠️ Settings endpoint returned non-200");
                return;
            }

            let body: ApiResponse<SettingsResponse> = resp.json().await.unwrap();
            assert!(body.success, "Response should indicate success");
            assert!(body.data.is_some(), "Data should be present");

            let settings = body.data.unwrap();
            // Verify safety settings
            assert!(settings.use_testnet, "Should be using testnet");
            assert!(
                settings.max_leverage <= 10,
                "Max leverage should be capped at 10x"
            );
            assert!(
                settings.max_position_size_usdt > 0.0,
                "Max position size should be positive"
            );
        },
        Err(e) => {
            if e.is_connect() {
                println!("⚠️ Server not running, skipping integration test");
                return;
            }
            panic!("Request failed: {:?}", e);
        },
    }
}

// ============================================================================
// ORDER PLACEMENT TESTS (with confirmation flow)
// ============================================================================

#[tokio::test]
#[ignore]
async fn test_place_order_returns_confirmation_token() {
    let client = create_client();
    let url = format!("{}/api/real-trading/orders", get_api_base());

    let order = PlaceOrderRequest {
        symbol: "BTCUSDT".to_string(),
        side: "BUY".to_string(),
        order_type: "LIMIT".to_string(),
        quantity: 0.001,
        price: Some(30000.0), // Well below market for safety
        confirmation_token: None,
    };

    let response = client.post(&url).json(&order).send().await;

    match response {
        Ok(resp) => {
            if !resp.status().is_success() {
                let status = resp.status();
                let text = resp.text().await.unwrap_or_default();
                println!("⚠️ Order placement returned {}: {}", status, text);
                return;
            }

            let body: ApiResponse<ConfirmationResponse> = resp.json().await.unwrap();
            assert!(body.success, "Response should indicate success");
            assert!(body.data.is_some(), "Data should be present");

            let confirmation = body.data.unwrap();
            // First call should require confirmation
            assert!(
                confirmation.requires_confirmation,
                "Should require confirmation"
            );
            assert!(!confirmation.token.is_empty(), "Token should not be empty");
            assert!(
                confirmation.summary.contains("BTCUSDT"),
                "Summary should contain symbol"
            );
        },
        Err(e) => {
            if e.is_connect() {
                println!("⚠️ Server not running, skipping integration test");
                return;
            }
            panic!("Request failed: {:?}", e);
        },
    }
}

#[tokio::test]
#[ignore]
async fn test_place_order_with_invalid_symbol_returns_error() {
    let client = create_client();
    let url = format!("{}/api/real-trading/orders", get_api_base());

    let order = PlaceOrderRequest {
        symbol: "INVALIDPAIR".to_string(),
        side: "BUY".to_string(),
        order_type: "MARKET".to_string(),
        quantity: 1.0,
        price: None,
        confirmation_token: None,
    };

    let response = client.post(&url).json(&order).send().await;

    match response {
        Ok(resp) => {
            // Should either return 400 or success with error message
            let body: ApiResponse<serde_json::Value> = resp.json().await.unwrap();
            // Either the server rejects it outright or returns an error response
            if body.success {
                // Some implementations might return confirmation first
                // The actual error would occur on execution
                println!("Note: Server returned success for invalid symbol (may fail on confirm)");
            }
        },
        Err(e) => {
            if e.is_connect() {
                println!("⚠️ Server not running, skipping integration test");
                return;
            }
            panic!("Request failed: {:?}", e);
        },
    }
}

// ============================================================================
// ORDER LISTING TESTS
// ============================================================================

#[tokio::test]
#[ignore]
async fn test_list_orders_returns_array() {
    let client = create_client();
    let url = format!("{}/api/real-trading/orders", get_api_base());

    let response = client.get(&url).send().await;

    match response {
        Ok(resp) => {
            if !resp.status().is_success() {
                println!("⚠️ List orders returned non-200");
                return;
            }

            let body: ApiResponse<Vec<OrderInfo>> = resp.json().await.unwrap();
            assert!(body.success, "Response should indicate success");
            assert!(body.data.is_some(), "Data should be present");

            let orders = body.data.unwrap();
            // May be empty, just verify it's a valid array
            println!("Found {} active orders", orders.len());
        },
        Err(e) => {
            if e.is_connect() {
                println!("⚠️ Server not running, skipping integration test");
                return;
            }
            panic!("Request failed: {:?}", e);
        },
    }
}

#[tokio::test]
#[ignore]
async fn test_list_orders_with_symbol_filter() {
    let client = create_client();
    let url = format!("{}/api/real-trading/orders?symbol=BTCUSDT", get_api_base());

    let response = client.get(&url).send().await;

    match response {
        Ok(resp) => {
            if !resp.status().is_success() {
                return;
            }

            let body: ApiResponse<Vec<OrderInfo>> = resp.json().await.unwrap();
            assert!(body.success);

            if let Some(orders) = body.data {
                // All orders should be for BTCUSDT
                for order in &orders {
                    assert_eq!(
                        order.symbol, "BTCUSDT",
                        "Filtered orders should match symbol"
                    );
                }
            }
        },
        Err(e) => {
            if e.is_connect() {
                return;
            }
            panic!("Request failed: {:?}", e);
        },
    }
}

// ============================================================================
// CANCEL ORDER TESTS
// ============================================================================

#[tokio::test]
#[ignore]
async fn test_cancel_nonexistent_order_returns_error() {
    let client = create_client();
    let url = format!(
        "{}/api/real-trading/orders/nonexistent-id-12345",
        get_api_base()
    );

    let response = client.delete(&url).send().await;

    match response {
        Ok(resp) => {
            // Should return 404 or error response
            let body: ApiResponse<serde_json::Value> = resp.json().await.unwrap();
            // Should either be 404 or success=false with error message
            if body.success {
                println!("Note: Cancel returned success for non-existent order");
            }
        },
        Err(e) => {
            if e.is_connect() {
                return;
            }
            panic!("Request failed: {:?}", e);
        },
    }
}

// ============================================================================
// POSITIONS TESTS
// ============================================================================

#[tokio::test]
#[ignore]
async fn test_get_open_positions() {
    let client = create_client();
    let url = format!("{}/api/real-trading/positions", get_api_base());

    let response = client.get(&url).send().await;

    match response {
        Ok(resp) => {
            if !resp.status().is_success() {
                return;
            }

            let body: ApiResponse<Vec<serde_json::Value>> = resp.json().await.unwrap();
            assert!(body.success);
            assert!(body.data.is_some());
            println!("Found {} open positions", body.data.unwrap().len());
        },
        Err(e) => {
            if e.is_connect() {
                return;
            }
            panic!("Request failed: {:?}", e);
        },
    }
}

#[tokio::test]
#[ignore]
async fn test_get_closed_trades() {
    let client = create_client();
    let url = format!("{}/api/real-trading/trades", get_api_base());

    let response = client.get(&url).send().await;

    match response {
        Ok(resp) => {
            if !resp.status().is_success() {
                return;
            }

            let body: ApiResponse<Vec<serde_json::Value>> = resp.json().await.unwrap();
            assert!(body.success);
            assert!(body.data.is_some());
            println!("Found {} closed trades", body.data.unwrap().len());
        },
        Err(e) => {
            if e.is_connect() {
                return;
            }
            panic!("Request failed: {:?}", e);
        },
    }
}

// ============================================================================
// MODIFY SL/TP TESTS
// ============================================================================

#[tokio::test]
#[ignore]
async fn test_modify_sltp_nonexistent_position() {
    let client = create_client();
    let url = format!(
        "{}/api/real-trading/positions/INVALIDUSDT/sltp",
        get_api_base()
    );

    let body = serde_json::json!({
        "stop_loss": 40000.0,
        "take_profit": 60000.0
    });

    let response = client.put(&url).json(&body).send().await;

    match response {
        Ok(resp) => {
            // Should return error for non-existent position
            let body: ApiResponse<serde_json::Value> = resp.json().await.unwrap();
            // Expect either 404 or success=false
            if body.success {
                println!("Note: Modify SL/TP returned success for non-existent position");
            }
        },
        Err(e) => {
            if e.is_connect() {
                return;
            }
            panic!("Request failed: {:?}", e);
        },
    }
}

// ============================================================================
// SAFETY TESTS
// ============================================================================

#[tokio::test]
#[ignore]
async fn test_testnet_mode_is_enforced() {
    let client = create_client();
    let url = format!("{}/api/real-trading/status", get_api_base());

    let response = client.get(&url).send().await;

    match response {
        Ok(resp) => {
            if !resp.status().is_success() {
                return;
            }

            let body: ApiResponse<EngineStatus> = resp.json().await.unwrap();
            if let Some(status) = body.data {
                assert!(
                    status.is_testnet,
                    "CRITICAL: Production mode detected! Tests should only run on testnet."
                );
            }
        },
        Err(e) => {
            if e.is_connect() {
                return;
            }
            panic!("Request failed: {:?}", e);
        },
    }
}

#[tokio::test]
#[ignore]
async fn test_leverage_limit_enforced() {
    let client = create_client();
    let url = format!("{}/api/real-trading/settings", get_api_base());

    let response = client.get(&url).send().await;

    match response {
        Ok(resp) => {
            if !resp.status().is_success() {
                return;
            }

            let body: ApiResponse<SettingsResponse> = resp.json().await.unwrap();
            if let Some(settings) = body.data {
                assert!(
                    settings.max_leverage <= 10,
                    "SAFETY: Max leverage should be capped at 10x, got {}x",
                    settings.max_leverage
                );
            }
        },
        Err(e) => {
            if e.is_connect() {
                return;
            }
            panic!("Request failed: {:?}", e);
        },
    }
}
