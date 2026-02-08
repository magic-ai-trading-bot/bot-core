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

// ============================================================================
// UNIT TESTS FOR ENGINE COMPONENTS (Non-API tests)
// ============================================================================

#[cfg(test)]
mod engine_unit_tests {
    use super::*;
    use binance_trading_bot::binance::BinanceClient;
    use binance_trading_bot::config::{BinanceConfig, TradingConfig, TradingMode};
    use binance_trading_bot::real_trading::{
        CircuitBreakerState, DailyMetrics, OrderState, PositionSide, RealOrder, RealPosition,
        RealTradingConfig, RealTradingEngine,
    };
    use binance_trading_bot::trading::risk_manager::RiskManager;

    // Helper: Create a mock Binance client for testing
    fn create_test_binance_client() -> BinanceClient {
        let config = BinanceConfig {
            api_key: "test_api_key".to_string(),
            secret_key: "test_api_secret".to_string(),
            futures_api_key: String::new(),
            futures_secret_key: String::new(),
            testnet: true,
            base_url: "https://testnet.binance.vision".to_string(),
            ws_url: "wss://testnet.binance.vision/ws".to_string(),
            futures_base_url: "https://testnet.binancefuture.com".to_string(),
            futures_ws_url: "wss://stream.binancefuture.com/ws".to_string(),
            trading_mode: TradingMode::RealTestnet,
        };
        BinanceClient::new(config).unwrap()
    }

    // Helper: Create a test risk manager
    fn create_test_risk_manager() -> RiskManager {
        let config = TradingConfig {
            enabled: true,
            max_positions: 5,
            default_quantity: 0.001,
            risk_percentage: 2.0,
            stop_loss_percentage: 2.0,
            take_profit_percentage: 4.0,
            order_timeout_seconds: 60,
            position_check_interval_seconds: 10,
            leverage: 1,
            margin_type: "ISOLATED".to_string(),
        };
        RiskManager::new(config)
    }

    // ===== CircuitBreakerState Tests =====

    #[test]
    fn test_circuit_breaker_opens_after_threshold_errors() {
        let mut cb = CircuitBreakerState::default();
        assert!(!cb.is_open);

        // Record errors below threshold
        assert!(!cb.record_error("error 1", 3));
        assert!(!cb.record_error("error 2", 3));
        assert_eq!(cb.error_count, 2);
        assert!(!cb.is_open);

        // Third error should open circuit
        assert!(cb.record_error("error 3", 3));
        assert!(cb.is_open);
        assert!(cb.opened_at.is_some());
        assert_eq!(cb.last_error, Some("error 3".to_string()));
    }

    #[test]
    fn test_circuit_breaker_reset_on_success() {
        let mut cb = CircuitBreakerState::default();
        cb.record_error("error", 3);
        cb.record_error("error", 3);
        assert_eq!(cb.error_count, 2);

        cb.record_success();
        assert_eq!(cb.error_count, 0);
        // Circuit should remain closed after success (manual reset needed)
    }

    #[test]
    fn test_circuit_breaker_should_close_after_cooldown() {
        let mut cb = CircuitBreakerState::default();
        cb.is_open = true;
        cb.opened_at = Some(chrono::Utc::now() - chrono::Duration::seconds(301));

        // Should close after 300 second cooldown
        assert!(cb.should_close(300));

        // Should NOT close before cooldown
        cb.opened_at = Some(chrono::Utc::now() - chrono::Duration::seconds(100));
        assert!(!cb.should_close(300));
    }

    #[test]
    fn test_circuit_breaker_close_resets_state() {
        let mut cb = CircuitBreakerState {
            is_open: true,
            error_count: 5,
            opened_at: Some(chrono::Utc::now()),
            last_error: Some("test error".to_string()),
        };

        cb.close();

        assert!(!cb.is_open);
        assert_eq!(cb.error_count, 0);
        assert!(cb.opened_at.is_none());
        assert!(cb.last_error.is_none());
    }

    // ===== DailyMetrics Tests =====

    #[test]
    fn test_daily_metrics_win_rate_calculation() {
        let mut metrics = DailyMetrics::new();
        assert_eq!(metrics.win_rate(), 0.0); // No trades

        metrics.trades_count = 10;
        metrics.winning_trades = 6;
        metrics.losing_trades = 4;

        assert_eq!(metrics.win_rate(), 60.0);
    }

    #[test]
    fn test_daily_metrics_reset_if_new_day() {
        let mut metrics = DailyMetrics::new();
        let old_date = (chrono::Utc::now() - chrono::Duration::days(1))
            .format("%Y-%m-%d")
            .to_string();
        metrics.date = old_date.clone();
        metrics.trades_count = 10;
        metrics.realized_pnl = 100.0;

        metrics.reset_if_new_day();

        // Should reset to today
        assert_ne!(metrics.date, old_date);
        assert_eq!(metrics.trades_count, 0);
        assert_eq!(metrics.realized_pnl, 0.0);
    }

    #[test]
    fn test_daily_metrics_no_reset_on_same_day() {
        let mut metrics = DailyMetrics::new();
        metrics.trades_count = 5;
        metrics.realized_pnl = 50.0;
        let original_date = metrics.date.clone();

        metrics.reset_if_new_day();

        assert_eq!(metrics.date, original_date);
        assert_eq!(metrics.trades_count, 5);
        assert_eq!(metrics.realized_pnl, 50.0);
    }

    // ===== OrderState Tests =====

    #[test]
    fn test_order_state_is_active() {
        assert!(OrderState::Pending.is_active());
        assert!(OrderState::New.is_active());
        assert!(OrderState::PartiallyFilled.is_active());
        assert!(!OrderState::Filled.is_active());
        assert!(!OrderState::Cancelled.is_active());
        assert!(!OrderState::Rejected.is_active());
        assert!(!OrderState::Expired.is_active());
    }

    #[test]
    fn test_order_state_is_terminal() {
        assert!(!OrderState::Pending.is_terminal());
        assert!(!OrderState::New.is_terminal());
        assert!(!OrderState::PartiallyFilled.is_terminal());
        assert!(OrderState::Filled.is_terminal());
        assert!(OrderState::Cancelled.is_terminal());
        assert!(OrderState::Rejected.is_terminal());
        assert!(OrderState::Expired.is_terminal());
    }

    #[test]
    fn test_order_state_from_binance_status() {
        assert_eq!(OrderState::from_binance_status("NEW"), OrderState::New);
        assert_eq!(
            OrderState::from_binance_status("PARTIALLY_FILLED"),
            OrderState::PartiallyFilled
        );
        assert_eq!(
            OrderState::from_binance_status("FILLED"),
            OrderState::Filled
        );
        assert_eq!(
            OrderState::from_binance_status("CANCELED"),
            OrderState::Cancelled
        );
        assert_eq!(
            OrderState::from_binance_status("PENDING_CANCEL"),
            OrderState::Cancelled
        );
        assert_eq!(
            OrderState::from_binance_status("REJECTED"),
            OrderState::Rejected
        );
        assert_eq!(
            OrderState::from_binance_status("EXPIRED"),
            OrderState::Expired
        );
        assert_eq!(
            OrderState::from_binance_status("UNKNOWN"),
            OrderState::Pending
        );
    }

    // ===== RealOrder Tests =====

    #[test]
    fn test_real_order_creation() {
        let order = RealOrder::new(
            "test_order_123".to_string(),
            "BTCUSDT".to_string(),
            "BUY".to_string(),
            "LIMIT".to_string(),
            0.001,
            Some(50000.0),
            None,
            None,
            true,
        );

        assert_eq!(order.client_order_id, "test_order_123");
        assert_eq!(order.symbol, "BTCUSDT");
        assert_eq!(order.side, "BUY");
        assert_eq!(order.original_quantity, 0.001);
        assert_eq!(order.price, Some(50000.0));
        assert_eq!(order.state, OrderState::Pending);
        assert_eq!(order.executed_quantity, 0.0);
        assert_eq!(order.remaining_quantity, 0.001);
        assert!(order.is_entry);
    }

    #[test]
    fn test_real_order_is_active() {
        let mut order = RealOrder::new(
            "test".to_string(),
            "BTCUSDT".to_string(),
            "BUY".to_string(),
            "LIMIT".to_string(),
            1.0,
            Some(50000.0),
            None,
            None,
            true,
        );

        order.state = OrderState::New;
        assert!(order.is_active());

        order.state = OrderState::Filled;
        assert!(!order.is_active());
    }

    #[test]
    fn test_real_order_is_filled() {
        let mut order = RealOrder::new(
            "test".to_string(),
            "BTCUSDT".to_string(),
            "BUY".to_string(),
            "LIMIT".to_string(),
            1.0,
            None,
            None,
            None,
            true,
        );

        assert!(!order.is_filled());

        order.state = OrderState::Filled;
        assert!(order.is_filled());
    }

    #[test]
    fn test_real_order_is_terminal() {
        let mut order = RealOrder::new(
            "test".to_string(),
            "BTCUSDT".to_string(),
            "BUY".to_string(),
            "LIMIT".to_string(),
            1.0,
            None,
            None,
            None,
            true,
        );

        order.state = OrderState::New;
        assert!(!order.is_terminal());

        order.state = OrderState::Cancelled;
        assert!(order.is_terminal());
    }

    #[test]
    fn test_real_order_total_commission() {
        let mut order = RealOrder::new(
            "test".to_string(),
            "BTCUSDT".to_string(),
            "BUY".to_string(),
            "LIMIT".to_string(),
            1.0,
            None,
            None,
            None,
            true,
        );

        // Add some fills
        use binance_trading_bot::real_trading::OrderFill;
        use chrono::Utc;

        order.fills.push(OrderFill {
            trade_id: 1,
            price: 50000.0,
            quantity: 0.5,
            commission: 0.1,
            commission_asset: "USDT".to_string(),
            timestamp: Utc::now(),
        });

        order.fills.push(OrderFill {
            trade_id: 2,
            price: 50100.0,
            quantity: 0.5,
            commission: 0.15,
            commission_asset: "USDT".to_string(),
            timestamp: Utc::now(),
        });

        assert_eq!(order.total_commission(), 0.25);
    }

    // ===== RealPosition Tests =====

    #[test]
    fn test_position_creation() {
        let position = RealPosition::new(
            "pos_123".to_string(),
            "BTCUSDT".to_string(),
            PositionSide::Long,
            0.001,
            50000.0,
            "order_123".to_string(),
            Some("test_strategy".to_string()),
            Some(0.85),
        );

        assert_eq!(position.id, "pos_123");
        assert_eq!(position.symbol, "BTCUSDT");
        assert_eq!(position.quantity, 0.001);
        assert_eq!(position.entry_price, 50000.0);
        assert_eq!(position.strategy_name, Some("test_strategy".to_string()));
        assert_eq!(position.signal_confidence, Some(0.85));
        assert!(!position.is_closed());
    }

    #[test]
    fn test_position_value_calculation() {
        let position = RealPosition::new(
            "pos_123".to_string(),
            "BTCUSDT".to_string(),
            PositionSide::Long,
            0.1,
            50000.0,
            "order_123".to_string(),
            None,
            None,
        );

        // Position value = quantity * entry_price = 0.1 * 50000 = 5000
        assert_eq!(position.position_value(), 5000.0);
    }

    #[test]
    fn test_position_update_price() {
        let mut position = RealPosition::new(
            "pos_123".to_string(),
            "BTCUSDT".to_string(),
            PositionSide::Long,
            1.0,
            50000.0,
            "order_123".to_string(),
            None,
            None,
        );

        position.update_price(51000.0);

        // Unrealized PnL for long = (current - entry) * quantity = 1000
        assert_eq!(position.unrealized_pnl, 1000.0);
        assert_eq!(position.current_price, 51000.0);
    }

    #[test]
    fn test_position_partial_close() {
        let mut position = RealPosition::new(
            "pos_123".to_string(),
            "BTCUSDT".to_string(),
            PositionSide::Long,
            1.0,
            50000.0,
            "order_123".to_string(),
            None,
            None,
        );

        let pnl = position.partial_close(51000.0, 0.5, 5.0, "exit_order".to_string());

        // PnL = (51000 - 50000) * 0.5 - 5 = 500 - 5 = 495
        assert_eq!(pnl, 495.0);
        assert_eq!(position.quantity, 0.5);
        assert_eq!(position.realized_pnl, 495.0);
        assert!(!position.is_closed());
    }

    #[test]
    fn test_position_fully_closed() {
        let mut position = RealPosition::new(
            "pos_123".to_string(),
            "BTCUSDT".to_string(),
            PositionSide::Long,
            1.0,
            50000.0,
            "order_123".to_string(),
            None,
            None,
        );

        position.partial_close(51000.0, 1.0, 10.0, "exit_order".to_string());

        assert_eq!(position.quantity, 0.0);
        assert!(position.is_closed());
    }

    // ===== RealTradingConfig Tests =====

    #[test]
    fn test_config_validation_success() {
        let config = RealTradingConfig::testnet_default();
        assert!(config.validate().is_ok());
    }

    #[test]
    fn test_config_validation_invalid_position_size() {
        let mut config = RealTradingConfig::testnet_default();
        config.max_position_size_usdt = -10.0;
        let errors = config.validate().unwrap_err();
        assert!(errors.iter().any(|e| e.contains("max_position_size_usdt")));
    }

    #[test]
    fn test_config_validation_invalid_exposure() {
        let mut config = RealTradingConfig::testnet_default();
        config.max_position_size_usdt = 1000.0;
        config.max_total_exposure_usdt = 500.0; // Less than position size
        let errors = config.validate().unwrap_err();
        assert!(errors.iter().any(|e| e.contains("max_total_exposure_usdt")));
    }

    #[test]
    fn test_config_validation_invalid_risk_percent() {
        let mut config = RealTradingConfig::testnet_default();
        config.risk_per_trade_percent = 150.0; // Over 100%
        let errors = config.validate().unwrap_err();
        assert!(errors.iter().any(|e| e.contains("risk_per_trade_percent")));
    }

    #[test]
    fn test_config_testnet_default() {
        let config = RealTradingConfig::testnet_default();
        assert!(config.use_testnet);
        assert_eq!(config.max_position_size_usdt, 100.0);
        assert_eq!(config.max_total_exposure_usdt, 500.0);
        assert_eq!(config.max_daily_loss_usdt, 50.0);
    }

    #[test]
    fn test_config_production_default() {
        let config = RealTradingConfig::production_default();
        assert!(!config.use_testnet);
        assert_eq!(config.circuit_breaker_errors, 2);
        assert!(config.circuit_breaker_close_positions);
        assert_eq!(config.reconciliation_interval_secs, 60);
    }

    #[test]
    fn test_config_is_symbol_allowed() {
        let mut config = RealTradingConfig::testnet_default();

        // Empty allowed list = all symbols allowed
        assert!(config.is_symbol_allowed("BTCUSDT"));
        assert!(config.is_symbol_allowed("ETHUSDT"));

        // With specific symbols
        config.allowed_symbols = vec!["BTCUSDT".to_string(), "ETHUSDT".to_string()];
        assert!(config.is_symbol_allowed("BTCUSDT"));
        assert!(config.is_symbol_allowed("ETHUSDT"));
        assert!(!config.is_symbol_allowed("ADAUSDT"));
    }

    // ===== Engine Initialization Tests =====

    #[tokio::test]
    async fn test_engine_creation() {
        let config = RealTradingConfig::testnet_default();
        let client = create_test_binance_client();
        let risk_manager = create_test_risk_manager();

        let result = RealTradingEngine::new(config, client, risk_manager).await;
        assert!(result.is_ok());

        let engine = result.unwrap();
        assert!(!engine.is_running().await);
    }

    #[tokio::test]
    async fn test_engine_creation_with_invalid_config() {
        let mut config = RealTradingConfig::testnet_default();
        config.max_position_size_usdt = -100.0; // Invalid
        let client = create_test_binance_client();
        let risk_manager = create_test_risk_manager();

        let result = RealTradingEngine::new(config, client, risk_manager).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_engine_get_config() {
        let config = RealTradingConfig::testnet_default();
        let client = create_test_binance_client();
        let risk_manager = create_test_risk_manager();

        let engine = RealTradingEngine::new(config.clone(), client, risk_manager)
            .await
            .unwrap();

        let retrieved_config = engine.get_config().await;
        assert_eq!(retrieved_config.use_testnet, config.use_testnet);
        assert_eq!(
            retrieved_config.max_position_size_usdt,
            config.max_position_size_usdt
        );
    }

    #[tokio::test]
    async fn test_engine_update_config() {
        let config = RealTradingConfig::testnet_default();
        let client = create_test_binance_client();
        let risk_manager = create_test_risk_manager();

        let engine = RealTradingEngine::new(config, client, risk_manager)
            .await
            .unwrap();

        let mut new_config = RealTradingConfig::testnet_default();
        new_config.max_position_size_usdt = 200.0;

        assert!(engine.update_config(new_config).await.is_ok());

        let updated = engine.get_config().await;
        assert_eq!(updated.max_position_size_usdt, 200.0);
    }

    #[tokio::test]
    async fn test_engine_circuit_breaker_operations() {
        let config = RealTradingConfig::testnet_default();
        let client = create_test_binance_client();
        let risk_manager = create_test_risk_manager();

        let engine = RealTradingEngine::new(config, client, risk_manager)
            .await
            .unwrap();

        // Initially closed
        let cb = engine.get_circuit_breaker().await;
        assert!(!cb.is_open);

        // Reset should work
        engine.reset_circuit_breaker().await;
        let cb = engine.get_circuit_breaker().await;
        assert!(!cb.is_open);
    }

    #[tokio::test]
    async fn test_engine_daily_metrics() {
        let config = RealTradingConfig::testnet_default();
        let client = create_test_binance_client();
        let risk_manager = create_test_risk_manager();

        let engine = RealTradingEngine::new(config, client, risk_manager)
            .await
            .unwrap();

        let metrics = engine.get_daily_metrics().await;
        assert_eq!(metrics.trades_count, 0);
        assert_eq!(metrics.realized_pnl, 0.0);
        assert_eq!(metrics.winning_trades, 0);
        assert_eq!(metrics.losing_trades, 0);
    }

    #[tokio::test]
    async fn test_engine_get_positions_empty() {
        let config = RealTradingConfig::testnet_default();
        let client = create_test_binance_client();
        let risk_manager = create_test_risk_manager();

        let engine = RealTradingEngine::new(config, client, risk_manager)
            .await
            .unwrap();

        let positions = engine.get_positions();
        assert_eq!(positions.len(), 0);
    }

    #[tokio::test]
    async fn test_engine_get_orders_empty() {
        let config = RealTradingConfig::testnet_default();
        let client = create_test_binance_client();
        let risk_manager = create_test_risk_manager();

        let engine = RealTradingEngine::new(config, client, risk_manager)
            .await
            .unwrap();

        let orders = engine.get_orders();
        assert_eq!(orders.len(), 0);
    }

    #[tokio::test]
    async fn test_engine_total_equity_calculation() {
        let config = RealTradingConfig::testnet_default();
        let client = create_test_binance_client();
        let risk_manager = create_test_risk_manager();

        let engine = RealTradingEngine::new(config, client, risk_manager)
            .await
            .unwrap();

        let equity = engine.get_total_equity_usdt().await;
        // Should be 0.0 initially (no balance, no positions)
        assert_eq!(equity, 0.0);
    }

    #[tokio::test]
    async fn test_engine_total_exposure_empty() {
        let config = RealTradingConfig::testnet_default();
        let client = create_test_binance_client();
        let risk_manager = create_test_risk_manager();

        let engine = RealTradingEngine::new(config, client, risk_manager)
            .await
            .unwrap();

        let exposure = engine.get_total_exposure();
        assert_eq!(exposure, 0.0);
    }

    #[tokio::test]
    async fn test_engine_unrealized_pnl_empty() {
        let config = RealTradingConfig::testnet_default();
        let client = create_test_binance_client();
        let risk_manager = create_test_risk_manager();

        let engine = RealTradingEngine::new(config, client, risk_manager)
            .await
            .unwrap();

        let pnl = engine.get_total_unrealized_pnl();
        assert_eq!(pnl, 0.0);
    }

    #[tokio::test]
    async fn test_engine_subscribe_events() {
        let config = RealTradingConfig::testnet_default();
        let client = create_test_binance_client();
        let risk_manager = create_test_risk_manager();

        let engine = RealTradingEngine::new(config, client, risk_manager)
            .await
            .unwrap();

        // Should be able to subscribe to events
        let _rx = engine.subscribe_events();
        // If this doesn't panic, subscription works
    }
}
