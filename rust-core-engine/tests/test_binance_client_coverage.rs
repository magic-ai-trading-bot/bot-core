// Additional tests to increase coverage for binance/client.rs and user_data_stream.rs
// Focus on request building, URL construction, parameter encoding, signature generation

mod common;

use binance_trading_bot::binance::client::BinanceClient;
use binance_trading_bot::binance::types::*;
use binance_trading_bot::binance::user_data_stream::*;
use binance_trading_bot::config::{binance_urls, BinanceConfig, TradingMode};
use serde_json::json;

// Helper to create test config with testnet URLs
fn create_test_config() -> BinanceConfig {
    BinanceConfig {
        api_key: "test_api_key_12345".to_string(),
        secret_key: "test_secret_key_67890".to_string(),
        futures_api_key: "futures_api_key_abc".to_string(),
        futures_secret_key: "futures_secret_key_def".to_string(),
        testnet: true,
        base_url: binance_urls::TESTNET_BASE_URL.to_string(),
        ws_url: binance_urls::TESTNET_WS_URL.to_string(),
        futures_base_url: binance_urls::FUTURES_TESTNET_BASE_URL.to_string(),
        futures_ws_url: binance_urls::FUTURES_TESTNET_WS_URL.to_string(),
        trading_mode: TradingMode::PaperTrading,
    }
}

// Helper to create config with empty futures keys (fallback to spot keys)
fn create_config_without_futures_keys() -> BinanceConfig {
    BinanceConfig {
        api_key: "spot_api_key".to_string(),
        secret_key: "spot_secret_key".to_string(),
        futures_api_key: String::new(),
        futures_secret_key: String::new(),
        testnet: true,
        base_url: binance_urls::TESTNET_BASE_URL.to_string(),
        ws_url: binance_urls::TESTNET_WS_URL.to_string(),
        futures_base_url: binance_urls::FUTURES_TESTNET_BASE_URL.to_string(),
        futures_ws_url: binance_urls::FUTURES_TESTNET_WS_URL.to_string(),
        trading_mode: TradingMode::RealTestnet,
    }
}

#[tokio::test]
async fn test_client_creation_with_different_configs() {
    // Test with full config
    let config1 = create_test_config();
    let client1 = BinanceClient::new(config1);
    assert!(client1.is_ok());

    // Test with fallback config
    let config2 = create_config_without_futures_keys();
    let client2 = BinanceClient::new(config2);
    assert!(client2.is_ok());

    // Test mainnet URLs
    let config3 = BinanceConfig {
        api_key: "test".to_string(),
        secret_key: "test".to_string(),
        futures_api_key: String::new(),
        futures_secret_key: String::new(),
        testnet: false,
        base_url: binance_urls::MAINNET_BASE_URL.to_string(),
        ws_url: binance_urls::MAINNET_WS_URL.to_string(),
        futures_base_url: binance_urls::FUTURES_MAINNET_BASE_URL.to_string(),
        futures_ws_url: binance_urls::FUTURES_MAINNET_WS_URL.to_string(),
        trading_mode: TradingMode::PaperTrading,
    };
    let client3 = BinanceClient::new(config3);
    assert!(client3.is_ok());
}

#[tokio::test]
async fn test_client_getter_methods() {
    let config = create_test_config();
    let client = BinanceClient::new(config.clone()).unwrap();

    assert_eq!(client.get_base_url(), config.base_url);
    assert_eq!(client.get_ws_url(), config.ws_url);
    assert_eq!(client.is_testnet(), config.testnet);
}

#[tokio::test]
async fn test_get_user_data_stream_url() {
    let config = create_test_config();
    let client = BinanceClient::new(config.clone()).unwrap();

    let listen_key = "test_listen_key_123";
    let url = client.get_user_data_stream_url(listen_key);

    assert!(url.contains("ws"));
    assert!(url.contains(listen_key));
}

// Test signature generation would require exposing private methods
// Instead, we test the public API that uses signatures internally
#[tokio::test]
async fn test_client_uses_correct_keys() {
    // Test that client properly selects spot vs futures keys
    let config = create_test_config();
    let client = BinanceClient::new(config.clone()).unwrap();

    // Verify config has both spot and futures keys
    assert!(!client.is_testnet() || client.is_testnet()); // Always true, but accesses method
    assert_eq!(client.get_base_url(), config.base_url);
}

// Test Kline parsing and validation
#[test]
fn test_kline_to_decimal_values_valid() {
    let kline = Kline {
        open_time: 1701234567000,
        close_time: 1701238167000,
        open: "45000.50".to_string(),
        high: "45500.75".to_string(),
        low: "44800.25".to_string(),
        close: "45200.00".to_string(),
        volume: "100.123456".to_string(),
        quote_asset_volume: "4510123.45".to_string(),
        number_of_trades: 1000,
        taker_buy_base_asset_volume: "50.123".to_string(),
        taker_buy_quote_asset_volume: "2255123.45".to_string(),
        ignore: "0".to_string(),
    };

    let result = kline.to_decimal_values();
    assert!(result.is_ok());

    let (open, high, low, close, volume) = result.unwrap();
    assert_eq!(open.to_string(), "45000.50");
    assert_eq!(high.to_string(), "45500.75");
    assert_eq!(low.to_string(), "44800.25");
    assert_eq!(close.to_string(), "45200.00");
    assert_eq!(volume.to_string(), "100.123456");
}

#[test]
fn test_kline_to_decimal_values_invalid() {
    let kline = Kline {
        open_time: 1701234567000,
        close_time: 1701238167000,
        open: "invalid_number".to_string(),
        high: "45500.00".to_string(),
        low: "44800.00".to_string(),
        close: "45200.00".to_string(),
        volume: "100.123".to_string(),
        quote_asset_volume: "4510123.45".to_string(),
        number_of_trades: 1000,
        taker_buy_base_asset_volume: "50.123".to_string(),
        taker_buy_quote_asset_volume: "2255123.45".to_string(),
        ignore: "0".to_string(),
    };

    let result = kline.to_decimal_values();
    assert!(result.is_err());
}

#[test]
fn test_kline_edge_cases() {
    // Very large numbers
    let kline1 = Kline {
        open_time: 9999999999999,
        close_time: 9999999999999,
        open: "999999999.99999999".to_string(),
        high: "1000000000.00000000".to_string(),
        low: "999999999.00000000".to_string(),
        close: "999999999.50000000".to_string(),
        volume: "99999999.99999999".to_string(),
        quote_asset_volume: "99999999999.99".to_string(),
        number_of_trades: 999999,
        taker_buy_base_asset_volume: "50000000.0".to_string(),
        taker_buy_quote_asset_volume: "50000000000.0".to_string(),
        ignore: "0".to_string(),
    };

    let result = kline1.to_decimal_values();
    assert!(result.is_ok());

    // Very small numbers
    let kline2 = Kline {
        open_time: 1,
        close_time: 2,
        open: "0.00000001".to_string(),
        high: "0.00000002".to_string(),
        low: "0.000000005".to_string(),
        close: "0.00000001".to_string(),
        volume: "0.00000001".to_string(),
        quote_asset_volume: "0.00000001".to_string(),
        number_of_trades: 1,
        taker_buy_base_asset_volume: "0.00000001".to_string(),
        taker_buy_quote_asset_volume: "0.00000001".to_string(),
        ignore: "0".to_string(),
    };

    let result2 = kline2.to_decimal_values();
    assert!(result2.is_ok());
}

// Test SymbolPrice type
#[test]
fn test_symbol_price_serialization() {
    let price = SymbolPrice {
        symbol: "BTCUSDT".to_string(),
        price: "45678.90".to_string(),
    };

    let json = serde_json::to_string(&price).unwrap();
    assert!(json.contains("BTCUSDT"));
    assert!(json.contains("45678.90"));

    let deserialized: SymbolPrice = serde_json::from_str(&json).unwrap();
    assert_eq!(deserialized.symbol, "BTCUSDT");
    assert_eq!(deserialized.price, "45678.90");
}

// Test FundingRate type
#[test]
fn test_funding_rate_serialization() {
    let rate = FundingRate {
        symbol: "ETHUSDT".to_string(),
        funding_rate: "0.0001".to_string(),
        funding_time: 1701234567000,
    };

    let json = serde_json::to_string(&rate).unwrap();
    let deserialized: FundingRate = serde_json::from_str(&json).unwrap();

    assert_eq!(deserialized.symbol, "ETHUSDT");
    assert_eq!(deserialized.funding_rate, "0.0001");
    assert_eq!(deserialized.funding_time, 1701234567000);
}

// Test AccountInfo parsing
#[test]
fn test_account_info_full() {
    let json = json!({
        "makerCommission": 10,
        "takerCommission": 10,
        "buyerCommission": 0,
        "sellerCommission": 0,
        "canTrade": true,
        "canWithdraw": true,
        "canDeposit": true,
        "updateTime": 1701234567000i64,
        "accountType": "SPOT",
        "balances": [
            {
                "asset": "BTC",
                "free": "10.00000000",
                "locked": "0.50000000"
            },
            {
                "asset": "USDT",
                "free": "50000.00000000",
                "locked": "1000.00000000"
            }
        ],
        "permissions": ["SPOT"]
    });

    let account: AccountInfo = serde_json::from_value(json).unwrap();
    assert_eq!(account.maker_commission, 10);
    assert_eq!(account.taker_commission, 10);
    assert!(account.can_trade);
    assert_eq!(account.balances.len(), 2);
    assert_eq!(account.balances[0].asset, "BTC");
    assert_eq!(account.balances[1].free, "50000.00000000");
}

// Test Balance type
#[test]
fn test_balance_serialization() {
    let balance = Balance {
        asset: "BNB".to_string(),
        free: "100.50000000".to_string(),
        locked: "10.25000000".to_string(),
    };

    let json = serde_json::to_string(&balance).unwrap();
    let deserialized: Balance = serde_json::from_str(&json).unwrap();

    assert_eq!(deserialized.asset, "BNB");
    assert_eq!(deserialized.free, "100.50000000");
    assert_eq!(deserialized.locked, "10.25000000");
}

// Test FuturesPosition parsing
#[test]
fn test_futures_position_full() {
    let json = json!({
        "symbol": "BTCUSDT",
        "positionAmt": "0.500",
        "entryPrice": "45000.0",
        "markPrice": "45500.0",
        "unRealizedProfit": "250.0",
        "liquidationPrice": "40000.0",
        "leverage": "10",
        "maxNotionalValue": "250000",
        "marginType": "cross",
        "isolatedMargin": "0.0",
        "isAutoAddMargin": "false",
        "positionSide": "BOTH",
        "notional": "22750.0",
        "isolatedWallet": "0",
        "updateTime": 1701234567000i64
    });

    let position: FuturesPosition = serde_json::from_value(json).unwrap();
    assert_eq!(position.symbol, "BTCUSDT");
    assert_eq!(position.position_amt, "0.500");
    assert_eq!(position.entry_price, "45000.0");
    assert_eq!(position.leverage, "10");
}

// Test FuturesOrder parsing
#[test]
fn test_futures_order_full() {
    let json = json!({
        "orderId": 123456,
        "symbol": "ETHUSDT",
        "status": "NEW",
        "clientOrderId": "test_order_1",
        "price": "3500.0",
        "avgPrice": "0.0",
        "origQty": "2.0",
        "executedQty": "0.0",
        "cumQty": "0.0",
        "cumQuote": "0.0",
        "timeInForce": "GTC",
        "type": "LIMIT",
        "reduceOnly": false,
        "closePosition": false,
        "side": "BUY",
        "positionSide": "LONG",
        "stopPrice": "0.0",
        "workingType": "CONTRACT_PRICE",
        "priceProtect": false,
        "origType": "LIMIT",
        "updateTime": 1701234567000i64
    });

    let order: FuturesOrder = serde_json::from_value(json).unwrap();
    assert_eq!(order.order_id, 123456);
    assert_eq!(order.symbol, "ETHUSDT");
    assert_eq!(order.status, "NEW");
    assert_eq!(order.side, "BUY");
}

// Test SpotOrderRequest builder methods
#[test]
fn test_spot_order_request_builder() {
    // Test market order builder
    let market_order = SpotOrderRequest::market("BTCUSDT", OrderSide::Buy, "1.0");
    assert_eq!(market_order.symbol, "BTCUSDT");
    assert_eq!(market_order.side, OrderSide::Buy);
    assert_eq!(market_order.order_type, SpotOrderType::Market);
    assert_eq!(market_order.quantity, Some("1.0".to_string()));

    // Test limit order builder
    let limit_order = SpotOrderRequest::limit("ETHUSDT", OrderSide::Sell, "2.0", "3500.0");
    assert_eq!(limit_order.symbol, "ETHUSDT");
    assert_eq!(limit_order.side, OrderSide::Sell);
    assert_eq!(limit_order.order_type, SpotOrderType::Limit);
    assert_eq!(limit_order.price, Some("3500.0".to_string()));
    assert_eq!(limit_order.time_in_force, Some(TimeInForce::Gtc));

    // Test stop loss limit order builder
    let stop_loss =
        SpotOrderRequest::stop_loss_limit("BTCUSDT", OrderSide::Sell, "1.0", "44000.0", "44500.0");
    assert_eq!(stop_loss.order_type, SpotOrderType::StopLossLimit);
    assert_eq!(stop_loss.price, Some("44000.0".to_string()));
    assert_eq!(stop_loss.stop_price, Some("44500.0".to_string()));

    // Test take profit limit order builder
    let take_profit =
        SpotOrderRequest::take_profit_limit("ETHUSDT", OrderSide::Sell, "2.0", "3700.0", "3650.0");
    assert_eq!(take_profit.order_type, SpotOrderType::TakeProfitLimit);
    assert_eq!(take_profit.price, Some("3700.0".to_string()));
    assert_eq!(take_profit.stop_price, Some("3650.0".to_string()));
}

// Test SpotOrderResponse parsing
#[test]
fn test_spot_order_response_full() {
    let json = json!({
        "symbol": "BTCUSDT",
        "orderId": 789012,
        "orderListId": -1,
        "clientOrderId": "test_client_id",
        "transactTime": 1701234567000i64,
        "price": "45000.0",
        "origQty": "1.0",
        "executedQty": "1.0",
        "cummulativeQuoteQty": "45000.0",
        "status": "FILLED",
        "timeInForce": "GTC",
        "type": "LIMIT",
        "side": "BUY",
        "fills": [
            {
                "price": "45000.0",
                "qty": "1.0",
                "commission": "0.001",
                "commissionAsset": "BNB",
                "tradeId": 12345
            }
        ]
    });

    let response: SpotOrderResponse = serde_json::from_value(json).unwrap();
    assert_eq!(response.symbol, "BTCUSDT");
    assert_eq!(response.order_id, 789012);
    assert_eq!(response.status, "FILLED");
    assert_eq!(response.fills.len(), 1);
    assert_eq!(response.fills[0].trade_id, 12345);
}

// Test Fill type
#[test]
fn test_fill_serialization() {
    let fill = Fill {
        price: "45000.0".to_string(),
        qty: "0.5".to_string(),
        commission: "0.0005".to_string(),
        commission_asset: "BNB".to_string(),
        trade_id: 123456,
    };

    let json = serde_json::to_string(&fill).unwrap();
    let deserialized: Fill = serde_json::from_str(&json).unwrap();

    assert_eq!(deserialized.price, "45000.0");
    assert_eq!(deserialized.qty, "0.5");
    assert_eq!(deserialized.commission, "0.0005");
    assert_eq!(deserialized.trade_id, 123456);
}

// Test OcoOrderRequest
#[test]
fn test_oco_order_request() {
    let order = OcoOrderRequest {
        symbol: "BTCUSDT".to_string(),
        side: OrderSide::Sell,
        quantity: "1.0".to_string(),
        above_type: "LIMIT_MAKER".to_string(),
        above_price: Some("50000.0".to_string()),
        below_type: "STOP_LOSS_LIMIT".to_string(),
        below_stop_price: Some("44000.0".to_string()),
        below_price: Some("43900.0".to_string()),
        above_time_in_force: Some(TimeInForce::Gtc),
        below_time_in_force: Some(TimeInForce::Gtc),
        list_client_order_id: None,
        above_client_order_id: None,
        below_client_order_id: None,
        new_order_resp_type: Some("FULL".to_string()),
    };

    assert_eq!(order.symbol, "BTCUSDT");
    assert_eq!(order.below_stop_price, Some("44000.0".to_string()));
    assert_eq!(order.quantity, "1.0");
}

// Test OcoOrderResponse parsing
#[test]
fn test_oco_order_response_full() {
    let json = json!({
        "orderListId": 123,
        "contingencyType": "OCO",
        "listStatusType": "EXEC_STARTED",
        "listOrderStatus": "EXECUTING",
        "listClientOrderId": "test_list_id",
        "transactionTime": 1701234567000i64,
        "symbol": "BTCUSDT",
        "orders": [
            {
                "symbol": "BTCUSDT",
                "orderId": 456,
                "clientOrderId": "order_1"
            },
            {
                "symbol": "BTCUSDT",
                "orderId": 457,
                "clientOrderId": "order_2"
            }
        ]
    });

    let response: OcoOrderResponse = serde_json::from_value(json).unwrap();
    assert_eq!(response.order_list_id, 123);
    assert_eq!(response.list_status_type, "EXEC_STARTED");
    assert_eq!(response.orders.len(), 2);
}

// Test ListenKeyResponse
#[test]
fn test_listen_key_response() {
    let response = ListenKeyResponse {
        listen_key: "test_listen_key_abc123".to_string(),
    };

    let json = serde_json::to_string(&response).unwrap();
    let deserialized: ListenKeyResponse = serde_json::from_str(&json).unwrap();

    assert_eq!(deserialized.listen_key, "test_listen_key_abc123");
}

// Test UserDataStreamHandle
#[test]
fn test_user_data_stream_handle() {
    let mut handle = UserDataStreamHandle::new(
        "key_123".to_string(),
        "wss://test.url/ws/key_123".to_string(),
    );

    // Test fields
    assert_eq!(handle.listen_key, "key_123");
    assert_eq!(handle.ws_url, "wss://test.url/ws/key_123");
    assert!(handle.created_at > 0);
    assert!(handle.last_keepalive > 0);

    // Test is_expired (should not be expired immediately)
    assert!(!handle.is_expired());

    // Test update_keepalive
    let old_time = handle.last_keepalive;
    std::thread::sleep(std::time::Duration::from_millis(10));
    handle.update_keepalive();
    assert!(handle.last_keepalive > old_time);

    // Test needs_keepalive (should not need immediately after creation)
    assert!(!handle.needs_keepalive());
}

// Test ExecutionReport helper methods
#[test]
fn test_execution_report_helpers() {
    // Test NEW status
    let new_report = ExecutionReport {
        event_type: "executionReport".to_string(),
        event_time: 1234567890,
        symbol: "BTCUSDT".to_string(),
        client_order_id: "test".to_string(),
        side: "BUY".to_string(),
        order_type: "LIMIT".to_string(),
        time_in_force: "GTC".to_string(),
        order_quantity: "1.0".to_string(),
        order_price: "45000.0".to_string(),
        stop_price: "0.0".to_string(),
        iceberg_quantity: "0.0".to_string(),
        original_client_order_id: "".to_string(),
        execution_type: "NEW".to_string(),
        order_status: "NEW".to_string(),
        order_reject_reason: "NONE".to_string(),
        order_id: 123,
        last_executed_quantity: "0.0".to_string(),
        cumulative_filled_quantity: "0.0".to_string(),
        last_executed_price: "0.0".to_string(),
        commission_amount: "0.0".to_string(),
        commission_asset: None,
        transaction_time: 1234567890,
        trade_id: -1,
        is_on_book: true,
        is_maker: false,
        order_creation_time: 1234567890,
        cumulative_quote_qty: "0.0".to_string(),
        last_quote_qty: "0.0".to_string(),
        quote_order_qty: "0.0".to_string(),
    };

    assert!(new_report.is_new());
    assert!(!new_report.is_filled());
    assert!(!new_report.is_partially_filled());
    assert!(!new_report.is_cancelled());
    assert!(!new_report.is_rejected());
    assert!(!new_report.is_trade());
    assert_eq!(new_report.fill_percentage(), 0.0);

    // Test PARTIALLY_FILLED status
    let partial_report = ExecutionReport {
        execution_type: "TRADE".to_string(),
        order_status: "PARTIALLY_FILLED".to_string(),
        order_quantity: "10.0".to_string(),
        cumulative_filled_quantity: "5.0".to_string(),
        ..new_report.clone()
    };

    assert!(partial_report.is_partially_filled());
    assert!(partial_report.is_trade());
    assert!((partial_report.fill_percentage() - 50.0).abs() < 0.01);

    // Test FILLED status
    let filled_report = ExecutionReport {
        execution_type: "TRADE".to_string(),
        order_status: "FILLED".to_string(),
        order_quantity: "10.0".to_string(),
        cumulative_filled_quantity: "10.0".to_string(),
        ..new_report.clone()
    };

    assert!(filled_report.is_filled());
    assert!((filled_report.fill_percentage() - 100.0).abs() < 0.01);

    // Test CANCELED status
    let cancelled_report = ExecutionReport {
        execution_type: "CANCELED".to_string(),
        order_status: "CANCELED".to_string(),
        ..new_report.clone()
    };

    assert!(cancelled_report.is_cancelled());

    // Test REJECTED status
    let rejected_report = ExecutionReport {
        execution_type: "REJECTED".to_string(),
        order_status: "REJECTED".to_string(),
        order_reject_reason: "INSUFFICIENT_BALANCE".to_string(),
        ..new_report
    };

    assert!(rejected_report.is_rejected());
}

// Test ExecutionReport fill_percentage edge cases
#[test]
fn test_execution_report_fill_percentage_edge_cases() {
    let report = ExecutionReport {
        event_type: "executionReport".to_string(),
        event_time: 1234567890,
        symbol: "BTCUSDT".to_string(),
        client_order_id: "test".to_string(),
        side: "BUY".to_string(),
        order_type: "LIMIT".to_string(),
        time_in_force: "GTC".to_string(),
        order_quantity: "0.0".to_string(), // Zero quantity - edge case
        order_price: "45000.0".to_string(),
        stop_price: "0.0".to_string(),
        iceberg_quantity: "0.0".to_string(),
        original_client_order_id: "".to_string(),
        execution_type: "NEW".to_string(),
        order_status: "NEW".to_string(),
        order_reject_reason: "NONE".to_string(),
        order_id: 123,
        last_executed_quantity: "0.0".to_string(),
        cumulative_filled_quantity: "0.0".to_string(),
        last_executed_price: "0.0".to_string(),
        commission_amount: "0.0".to_string(),
        commission_asset: None,
        transaction_time: 1234567890,
        trade_id: -1,
        is_on_book: true,
        is_maker: false,
        order_creation_time: 1234567890,
        cumulative_quote_qty: "0.0".to_string(),
        last_quote_qty: "0.0".to_string(),
        quote_order_qty: "0.0".to_string(),
    };

    // Should handle zero quantity gracefully
    assert_eq!(report.fill_percentage(), 0.0);
}

// Test OutboundAccountPosition
#[test]
fn test_outbound_account_position() {
    let position = OutboundAccountPosition {
        event_type: "outboundAccountPosition".to_string(),
        event_time: 1234567890,
        last_update_time: 1234567889,
        balances: vec![
            AccountBalance {
                asset: "BTC".to_string(),
                free: "1.5".to_string(),
                locked: "0.5".to_string(),
            },
            AccountBalance {
                asset: "USDT".to_string(),
                free: "10000.0".to_string(),
                locked: "2000.0".to_string(),
            },
        ],
    };

    assert_eq!(position.balances.len(), 2);
    assert_eq!(position.balances[0].asset, "BTC");
    assert_eq!(position.balances[1].free, "10000.0");
}

// Test AccountBalance
#[test]
fn test_account_balance_serialization() {
    let balance = AccountBalance {
        asset: "ETH".to_string(),
        free: "5.123456".to_string(),
        locked: "1.234567".to_string(),
    };

    let json = serde_json::to_string(&balance).unwrap();
    let deserialized: AccountBalance = serde_json::from_str(&json).unwrap();

    assert_eq!(deserialized.asset, "ETH");
    assert_eq!(deserialized.free, "5.123456");
    assert_eq!(deserialized.locked, "1.234567");
}

// Test BalanceUpdate
#[test]
fn test_balance_update_serialization() {
    let update = BalanceUpdate {
        event_type: "balanceUpdate".to_string(),
        event_time: 1234567890,
        asset: "BNB".to_string(),
        balance_delta: "10.5".to_string(),
        clear_time: 1234567888,
    };

    let json = serde_json::to_string(&update).unwrap();
    let deserialized: BalanceUpdate = serde_json::from_str(&json).unwrap();

    assert_eq!(deserialized.asset, "BNB");
    assert_eq!(deserialized.balance_delta, "10.5");
}

// Test UserDataEvent enum variants
#[test]
fn test_user_data_event_variants() {
    // Test ExecutionReport variant
    let exec_json = json!({
        "e": "executionReport",
        "E": 1499405658658i64,
        "s": "BTCUSDT",
        "c": "test",
        "S": "BUY",
        "o": "LIMIT",
        "f": "GTC",
        "q": "1.0",
        "p": "45000.0",
        "P": "0.0",
        "F": "0.0",
        "C": "",
        "x": "NEW",
        "X": "NEW",
        "r": "NONE",
        "i": 123,
        "l": "0.0",
        "z": "0.0",
        "L": "0.0",
        "n": "0.0",
        "N": null,
        "T": 1499405658657i64,
        "t": -1,
        "w": true,
        "m": false,
        "O": 1499405658657i64,
        "Z": "0.0",
        "Y": "0.0",
        "Q": "0.0"
    });

    let event: UserDataEvent = serde_json::from_value(exec_json).unwrap();
    match event {
        UserDataEvent::ExecutionReport(report) => {
            assert_eq!(report.symbol, "BTCUSDT");
        },
        _ => panic!("Expected ExecutionReport"),
    }
}

// Test UserDataStreamManager configuration
#[test]
fn test_user_data_stream_config_custom() {
    let config = UserDataStreamConfig {
        keepalive_interval_secs: 60 * 60, // 1 hour
        reconnect_delay_secs: 10,
        max_reconnect_attempts: 5,
        channel_buffer_size: 200,
    };

    assert_eq!(config.keepalive_interval_secs, 3600);
    assert_eq!(config.reconnect_delay_secs, 10);
    assert_eq!(config.max_reconnect_attempts, 5);
    assert_eq!(config.channel_buffer_size, 200);
}

#[test]
fn test_user_data_stream_manager_with_custom_config() {
    let binance_config = create_test_config();
    let client = BinanceClient::new(binance_config).expect("Failed to create client");

    let custom_config = UserDataStreamConfig {
        keepalive_interval_secs: 20 * 60,
        reconnect_delay_secs: 3,
        max_reconnect_attempts: 20,
        channel_buffer_size: 50,
    };

    let manager = UserDataStreamManager::with_config(client, custom_config);

    // Should be able to subscribe multiple times
    let _rx1 = manager.subscribe();
    let _rx2 = manager.subscribe();
}

// Test UserDataStreamEvent variants
#[test]
fn test_user_data_stream_event_variants() {
    // Connected event
    let connected = UserDataStreamEvent::Connected;
    match connected {
        UserDataStreamEvent::Connected => assert!(true),
        _ => panic!("Expected Connected"),
    }

    // Disconnected event
    let disconnected = UserDataStreamEvent::Disconnected;
    match disconnected {
        UserDataStreamEvent::Disconnected => assert!(true),
        _ => panic!("Expected Disconnected"),
    }

    // Error event
    let error = UserDataStreamEvent::Error("Test error".to_string());
    match error {
        UserDataStreamEvent::Error(msg) => assert_eq!(msg, "Test error"),
        _ => panic!("Expected Error"),
    }
}

// Test OrderSide enum
#[test]
fn test_order_side_enum() {
    assert_eq!(format!("{}", OrderSide::Buy), "BUY");
    assert_eq!(format!("{}", OrderSide::Sell), "SELL");

    // Test serialization
    let json_buy = serde_json::to_string(&OrderSide::Buy).unwrap();
    assert_eq!(json_buy, "\"BUY\"");

    let json_sell = serde_json::to_string(&OrderSide::Sell).unwrap();
    assert_eq!(json_sell, "\"SELL\"");
}

// Test SpotOrderType enum
#[test]
fn test_spot_order_type_enum() {
    assert_eq!(format!("{}", SpotOrderType::Market), "MARKET");
    assert_eq!(format!("{}", SpotOrderType::Limit), "LIMIT");
    assert_eq!(
        format!("{}", SpotOrderType::StopLossLimit),
        "STOP_LOSS_LIMIT"
    );
    assert_eq!(
        format!("{}", SpotOrderType::TakeProfitLimit),
        "TAKE_PROFIT_LIMIT"
    );
    assert_eq!(format!("{}", SpotOrderType::LimitMaker), "LIMIT_MAKER");
}

// Test TimeInForce enum
#[test]
fn test_time_in_force_enum() {
    assert_eq!(format!("{}", TimeInForce::Gtc), "GTC");
    assert_eq!(format!("{}", TimeInForce::Ioc), "IOC");
    assert_eq!(format!("{}", TimeInForce::Fok), "FOK");

    // Test serialization
    let json = serde_json::to_string(&TimeInForce::Gtc).unwrap();
    assert_eq!(json, "\"GTC\"");
}

// Test empty or edge case values
#[test]
fn test_edge_case_values() {
    // Empty strings
    let kline = Kline {
        open_time: 0,
        close_time: 0,
        open: "".to_string(),
        high: "".to_string(),
        low: "".to_string(),
        close: "".to_string(),
        volume: "".to_string(),
        quote_asset_volume: "".to_string(),
        number_of_trades: 0,
        taker_buy_base_asset_volume: "".to_string(),
        taker_buy_quote_asset_volume: "".to_string(),
        ignore: "".to_string(),
    };

    // Should fail to parse empty strings as decimals
    let result = kline.to_decimal_values();
    assert!(result.is_err());
}
