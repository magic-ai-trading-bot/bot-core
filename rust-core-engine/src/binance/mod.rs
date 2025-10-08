pub mod client;
pub mod types;
pub mod websocket;

pub use client::BinanceClient;
pub use types::*;
pub use websocket::BinanceWebSocket;

#[cfg(test)]
mod tests {
    use super::*;

    // Module structure and exports tests
    #[test]
    fn test_module_exports_client() {
        // Test that BinanceClient is properly exported
        let config = crate::config::BinanceConfig {
            api_key: "test".to_string(),
            secret_key: "test".to_string(),
            base_url: "https://api.binance.com".to_string(),
            ws_url: "wss://stream.binance.com:9443/ws".to_string(),
            futures_base_url: "https://fapi.binance.com".to_string(),
            futures_ws_url: "wss://fstream.binance.com".to_string(),
            testnet: false,
        };

        let _client = BinanceClient::new(config);
    }

    #[test]
    fn test_module_exports_websocket() {
        // Test that BinanceWebSocket is properly exported
        let config = crate::config::BinanceConfig {
            api_key: "test".to_string(),
            secret_key: "test".to_string(),
            base_url: "https://api.binance.com".to_string(),
            ws_url: "wss://stream.binance.com:9443/ws".to_string(),
            futures_base_url: "https://fapi.binance.com".to_string(),
            futures_ws_url: "wss://fstream.binance.com".to_string(),
            testnet: false,
        };

        let (_ws, _receiver) = BinanceWebSocket::new(config);
    }

    #[test]
    fn test_binance_client_with_testnet_config() {
        // Test BinanceClient with testnet configuration
        let config = crate::config::BinanceConfig {
            api_key: "testnet_key".to_string(),
            secret_key: "testnet_secret".to_string(),
            base_url: "https://testnet.binance.vision".to_string(),
            ws_url: "wss://testnet.binance.vision/ws".to_string(),
            futures_base_url: "https://testnet.binancefuture.com".to_string(),
            futures_ws_url: "wss://stream.binancefuture.com/ws".to_string(),
            testnet: true,
        };

        let _client = BinanceClient::new(config);
    }

    #[test]
    fn test_binance_websocket_with_empty_credentials() {
        // Test BinanceWebSocket with empty credentials (should still create)
        let config = crate::config::BinanceConfig {
            api_key: "".to_string(),
            secret_key: "".to_string(),
            base_url: "https://api.binance.com".to_string(),
            ws_url: "wss://stream.binance.com:9443/ws".to_string(),
            futures_base_url: "https://fapi.binance.com".to_string(),
            futures_ws_url: "wss://fstream.binance.com".to_string(),
            testnet: false,
        };

        let (_ws, _receiver) = BinanceWebSocket::new(config);
    }

    #[test]
    fn test_module_exports_types() {
        // Test that types are properly exported
        let _kline = Kline {
            open_time: 0,
            open: "0".to_string(),
            high: "0".to_string(),
            low: "0".to_string(),
            close: "0".to_string(),
            volume: "0".to_string(),
            close_time: 0,
            quote_asset_volume: "0".to_string(),
            number_of_trades: 0,
            taker_buy_base_asset_volume: "0".to_string(),
            taker_buy_quote_asset_volume: "0".to_string(),
            ignore: "0".to_string(),
        };

        let _symbol_price = SymbolPrice {
            symbol: "BTCUSDT".to_string(),
            price: "0".to_string(),
        };

        let _funding_rate = FundingRate {
            symbol: "BTCUSDT".to_string(),
            funding_rate: "0".to_string(),
            funding_time: 0,
        };
    }

    // Type instantiation tests
    #[test]
    fn test_kline_with_realistic_data() {
        // Test Kline with realistic trading data
        let kline = Kline {
            open_time: 1609459200000,
            open: "29000.50".to_string(),
            high: "29500.75".to_string(),
            low: "28800.25".to_string(),
            close: "29200.00".to_string(),
            volume: "1234.5678".to_string(),
            close_time: 1609459259999,
            quote_asset_volume: "36000000.12".to_string(),
            number_of_trades: 12345,
            taker_buy_base_asset_volume: "600.1234".to_string(),
            taker_buy_quote_asset_volume: "17500000.00".to_string(),
            ignore: "0".to_string(),
        };

        assert_eq!(kline.open_time, 1609459200000);
        assert_eq!(kline.number_of_trades, 12345);
    }

    #[test]
    fn test_symbol_price_multiple_symbols() {
        // Test SymbolPrice for different trading pairs
        let btc_price = SymbolPrice {
            symbol: "BTCUSDT".to_string(),
            price: "42000.50".to_string(),
        };
        let eth_price = SymbolPrice {
            symbol: "ETHUSDT".to_string(),
            price: "2200.75".to_string(),
        };

        assert_eq!(btc_price.symbol, "BTCUSDT");
        assert_eq!(eth_price.symbol, "ETHUSDT");
    }

    #[test]
    fn test_funding_rate_with_negative_rate() {
        // Test FundingRate with negative funding rate
        let funding_rate = FundingRate {
            symbol: "BTCUSDT".to_string(),
            funding_rate: "-0.0001".to_string(),
            funding_time: 1609459200000,
        };

        assert!(funding_rate.funding_rate.starts_with('-'));
    }

    #[test]
    fn test_funding_rate_with_positive_rate() {
        // Test FundingRate with positive funding rate
        let funding_rate = FundingRate {
            symbol: "ETHUSDT".to_string(),
            funding_rate: "0.0005".to_string(),
            funding_time: 1609459200000,
        };

        assert!(!funding_rate.funding_rate.starts_with('-'));
    }

    #[test]
    fn test_stream_event_enum() {
        // Test that StreamEvent enum is accessible
        let kline_data = KlineData {
            kline_start_time: 0,
            kline_close_time: 0,
            symbol: "BTCUSDT".to_string(),
            interval: "1m".to_string(),
            first_trade_id: 0,
            last_trade_id: 0,
            open_price: "0".to_string(),
            close_price: "0".to_string(),
            high_price: "0".to_string(),
            low_price: "0".to_string(),
            base_asset_volume: "0".to_string(),
            number_of_trades: 0,
            is_this_kline_closed: false,
            quote_asset_volume: "0".to_string(),
            taker_buy_base_asset_volume: "0".to_string(),
            taker_buy_quote_asset_volume: "0".to_string(),
        };

        let kline_event = KlineEvent {
            event_type: "kline".to_string(),
            event_time: 0,
            symbol: "BTCUSDT".to_string(),
            kline: kline_data,
        };

        let _stream_event = StreamEvent::Kline(kline_event);
    }

    #[test]
    fn test_kline_data_with_closed_candle() {
        // Test KlineData with closed candle
        let kline_data = KlineData {
            kline_start_time: 1609459200000,
            kline_close_time: 1609459259999,
            symbol: "BTCUSDT".to_string(),
            interval: "1m".to_string(),
            first_trade_id: 100000,
            last_trade_id: 100500,
            open_price: "29000.00".to_string(),
            close_price: "29100.00".to_string(),
            high_price: "29200.00".to_string(),
            low_price: "28950.00".to_string(),
            base_asset_volume: "100.5".to_string(),
            number_of_trades: 501,
            is_this_kline_closed: true,
            quote_asset_volume: "2920000.00".to_string(),
            taker_buy_base_asset_volume: "55.25".to_string(),
            taker_buy_quote_asset_volume: "1605000.00".to_string(),
        };

        assert!(kline_data.is_this_kline_closed);
        assert_eq!(kline_data.interval, "1m");
    }

    #[test]
    fn test_kline_data_with_open_candle() {
        // Test KlineData with open candle
        let kline_data = KlineData {
            kline_start_time: 1609459200000,
            kline_close_time: 1609459259999,
            symbol: "ETHUSDT".to_string(),
            interval: "5m".to_string(),
            first_trade_id: 200000,
            last_trade_id: 200100,
            open_price: "2200.00".to_string(),
            close_price: "2205.00".to_string(),
            high_price: "2210.00".to_string(),
            low_price: "2195.00".to_string(),
            base_asset_volume: "500.25".to_string(),
            number_of_trades: 101,
            is_this_kline_closed: false,
            quote_asset_volume: "1102500.00".to_string(),
            taker_buy_base_asset_volume: "250.10".to_string(),
            taker_buy_quote_asset_volume: "551000.00".to_string(),
        };

        assert!(!kline_data.is_this_kline_closed);
        assert_eq!(kline_data.interval, "5m");
    }

    #[test]
    fn test_kline_event_structure() {
        // Test KlineEvent structure
        let kline_data = KlineData {
            kline_start_time: 1609459200000,
            kline_close_time: 1609459259999,
            symbol: "BTCUSDT".to_string(),
            interval: "1h".to_string(),
            first_trade_id: 1000,
            last_trade_id: 2000,
            open_price: "30000.00".to_string(),
            close_price: "30500.00".to_string(),
            high_price: "30700.00".to_string(),
            low_price: "29800.00".to_string(),
            base_asset_volume: "1000.0".to_string(),
            number_of_trades: 1001,
            is_this_kline_closed: true,
            quote_asset_volume: "30250000.00".to_string(),
            taker_buy_base_asset_volume: "550.0".to_string(),
            taker_buy_quote_asset_volume: "16500000.00".to_string(),
        };

        let kline_event = KlineEvent {
            event_type: "kline".to_string(),
            event_time: 1609459260000,
            symbol: "BTCUSDT".to_string(),
            kline: kline_data,
        };

        assert_eq!(kline_event.event_type, "kline");
        assert_eq!(kline_event.symbol, "BTCUSDT");
    }

    #[test]
    fn test_websocket_event_enum() {
        // Test that WebSocketEvent enum is accessible
        let _error_event = WebSocketEvent::Error {
            message: "test error".to_string(),
        };

        let market_update = MarketDataUpdate {
            symbol: "BTCUSDT".to_string(),
            price: 0.0,
            price_change_24h: 0.0,
            price_change_percent_24h: 0.0,
            volume_24h: 0.0,
            timestamp: 0,
        };

        let _market_event = WebSocketEvent::MarketData(market_update);
    }

    #[test]
    fn test_websocket_event_error_variant() {
        // Test WebSocketEvent Error variant
        let error_event = WebSocketEvent::Error {
            message: "Connection timeout".to_string(),
        };

        match error_event {
            WebSocketEvent::Error { message } => {
                assert_eq!(message, "Connection timeout");
            }
            _ => panic!("Expected Error variant"),
        }
    }

    #[test]
    fn test_market_data_update_with_positive_change() {
        // Test MarketDataUpdate with positive price change
        let market_update = MarketDataUpdate {
            symbol: "BTCUSDT".to_string(),
            price: 42000.0,
            price_change_24h: 1000.0,
            price_change_percent_24h: 2.44,
            volume_24h: 50000.0,
            timestamp: 1609459200000,
        };

        assert!(market_update.price_change_24h > 0.0);
        assert!(market_update.price_change_percent_24h > 0.0);
    }

    #[test]
    fn test_market_data_update_with_negative_change() {
        // Test MarketDataUpdate with negative price change
        let market_update = MarketDataUpdate {
            symbol: "ETHUSDT".to_string(),
            price: 2150.0,
            price_change_24h: -50.0,
            price_change_percent_24h: -2.27,
            volume_24h: 30000.0,
            timestamp: 1609459200000,
        };

        assert!(market_update.price_change_24h < 0.0);
        assert!(market_update.price_change_percent_24h < 0.0);
    }

    #[test]
    fn test_order_types() {
        // Test order-related types
        let _order_request = NewOrderRequest {
            symbol: "BTCUSDT".to_string(),
            side: "BUY".to_string(),
            r#type: "LIMIT".to_string(),
            quantity: Some("1".to_string()),
            quote_order_qty: None,
            price: Some("30000".to_string()),
            new_client_order_id: None,
            stop_price: None,
            iceberg_qty: None,
            new_order_resp_type: None,
            time_in_force: Some("GTC".to_string()),
            reduce_only: None,
            close_position: None,
            position_side: None,
            working_type: None,
            price_protect: None,
        };

        let _fill = Fill {
            price: "30000".to_string(),
            qty: "1".to_string(),
            commission: "0.001".to_string(),
            commission_asset: "BNB".to_string(),
            trade_id: 123,
        };
    }

    #[test]
    fn test_new_order_request_buy_limit() {
        // Test NewOrderRequest for BUY LIMIT order
        let order = NewOrderRequest {
            symbol: "BTCUSDT".to_string(),
            side: "BUY".to_string(),
            r#type: "LIMIT".to_string(),
            quantity: Some("0.5".to_string()),
            quote_order_qty: None,
            price: Some("40000.00".to_string()),
            new_client_order_id: Some("buy_order_123".to_string()),
            stop_price: None,
            iceberg_qty: None,
            new_order_resp_type: Some("FULL".to_string()),
            time_in_force: Some("GTC".to_string()),
            reduce_only: None,
            close_position: None,
            position_side: Some("LONG".to_string()),
            working_type: None,
            price_protect: None,
        };

        assert_eq!(order.side, "BUY");
        assert_eq!(order.r#type, "LIMIT");
        assert_eq!(order.time_in_force, Some("GTC".to_string()));
    }

    #[test]
    fn test_new_order_request_sell_market() {
        // Test NewOrderRequest for SELL MARKET order
        let order = NewOrderRequest {
            symbol: "ETHUSDT".to_string(),
            side: "SELL".to_string(),
            r#type: "MARKET".to_string(),
            quantity: Some("2.0".to_string()),
            quote_order_qty: None,
            price: None,
            new_client_order_id: None,
            stop_price: None,
            iceberg_qty: None,
            new_order_resp_type: None,
            time_in_force: None,
            reduce_only: Some(true),
            close_position: None,
            position_side: Some("SHORT".to_string()),
            working_type: None,
            price_protect: None,
        };

        assert_eq!(order.side, "SELL");
        assert_eq!(order.r#type, "MARKET");
        assert_eq!(order.reduce_only, Some(true));
    }

    #[test]
    fn test_new_order_request_stop_loss() {
        // Test NewOrderRequest for STOP_LOSS order
        let order = NewOrderRequest {
            symbol: "BTCUSDT".to_string(),
            side: "SELL".to_string(),
            r#type: "STOP_LOSS".to_string(),
            quantity: Some("0.1".to_string()),
            quote_order_qty: None,
            price: None,
            new_client_order_id: None,
            stop_price: Some("38000.00".to_string()),
            iceberg_qty: None,
            new_order_resp_type: None,
            time_in_force: None,
            reduce_only: None,
            close_position: Some(true),
            position_side: None,
            working_type: Some("MARK_PRICE".to_string()),
            price_protect: Some(true),
        };

        assert_eq!(order.r#type, "STOP_LOSS");
        assert_eq!(order.stop_price, Some("38000.00".to_string()));
        assert_eq!(order.close_position, Some(true));
    }

    #[test]
    fn test_fill_structure() {
        // Test Fill structure
        let fill = Fill {
            price: "41000.50".to_string(),
            qty: "0.25".to_string(),
            commission: "0.00025".to_string(),
            commission_asset: "BNB".to_string(),
            trade_id: 987654321,
        };

        assert_eq!(fill.commission_asset, "BNB");
        assert_eq!(fill.trade_id, 987654321);
    }

    #[test]
    fn test_fill_with_usdt_commission() {
        // Test Fill with USDT commission
        let fill = Fill {
            price: "2200.00".to_string(),
            qty: "1.0".to_string(),
            commission: "2.2".to_string(),
            commission_asset: "USDT".to_string(),
            trade_id: 123456789,
        };

        assert_eq!(fill.commission_asset, "USDT");
    }

    #[test]
    fn test_account_types() {
        // Test account-related types
        let _balance = Balance {
            asset: "BTC".to_string(),
            free: "1.0".to_string(),
            locked: "0.0".to_string(),
        };

        let _position = FuturesPosition {
            symbol: "BTCUSDT".to_string(),
            position_amt: "0.1".to_string(),
            entry_price: "30000".to_string(),
            mark_price: "30100".to_string(),
            unrealized_pnl: "10".to_string(),
            liquidation_price: "25000".to_string(),
            leverage: "10".to_string(),
            max_notional_value: "100000".to_string(),
            margin_type: "isolated".to_string(),
            isolated_margin: "3000".to_string(),
            is_auto_add_margin: false,
            position_side: "LONG".to_string(),
            notional: "3010".to_string(),
            isolated_wallet: "3000".to_string(),
            update_time: 0,
        };
    }

    #[test]
    fn test_balance_with_locked_amount() {
        // Test Balance with locked amount
        let balance = Balance {
            asset: "ETH".to_string(),
            free: "5.5".to_string(),
            locked: "1.5".to_string(),
        };

        assert_eq!(balance.asset, "ETH");
        assert_eq!(balance.free, "5.5");
        assert_eq!(balance.locked, "1.5");
    }

    #[test]
    fn test_balance_all_free() {
        // Test Balance with all amount free
        let balance = Balance {
            asset: "USDT".to_string(),
            free: "10000.00".to_string(),
            locked: "0.00".to_string(),
        };

        assert_eq!(balance.locked, "0.00");
    }

    #[test]
    fn test_futures_position_long() {
        // Test FuturesPosition for LONG position
        let position = FuturesPosition {
            symbol: "BTCUSDT".to_string(),
            position_amt: "0.5".to_string(),
            entry_price: "40000.00".to_string(),
            mark_price: "41000.00".to_string(),
            unrealized_pnl: "500.00".to_string(),
            liquidation_price: "35000.00".to_string(),
            leverage: "5".to_string(),
            max_notional_value: "200000.00".to_string(),
            margin_type: "cross".to_string(),
            isolated_margin: "0.00".to_string(),
            is_auto_add_margin: false,
            position_side: "LONG".to_string(),
            notional: "20500.00".to_string(),
            isolated_wallet: "0.00".to_string(),
            update_time: 1609459200000,
        };

        assert_eq!(position.position_side, "LONG");
        assert_eq!(position.margin_type, "cross");
        assert!(!position.is_auto_add_margin);
    }

    #[test]
    fn test_futures_position_short() {
        // Test FuturesPosition for SHORT position
        let position = FuturesPosition {
            symbol: "ETHUSDT".to_string(),
            position_amt: "-2.0".to_string(),
            entry_price: "2200.00".to_string(),
            mark_price: "2150.00".to_string(),
            unrealized_pnl: "100.00".to_string(),
            liquidation_price: "2500.00".to_string(),
            leverage: "3".to_string(),
            max_notional_value: "50000.00".to_string(),
            margin_type: "isolated".to_string(),
            isolated_margin: "1500.00".to_string(),
            is_auto_add_margin: true,
            position_side: "SHORT".to_string(),
            notional: "4300.00".to_string(),
            isolated_wallet: "1600.00".to_string(),
            update_time: 1609459200000,
        };

        assert_eq!(position.position_side, "SHORT");
        assert_eq!(position.margin_type, "isolated");
        assert!(position.is_auto_add_margin);
        assert!(position.position_amt.starts_with('-'));
    }

    #[test]
    fn test_futures_position_with_negative_pnl() {
        // Test FuturesPosition with negative PnL
        let position = FuturesPosition {
            symbol: "BTCUSDT".to_string(),
            position_amt: "0.2".to_string(),
            entry_price: "45000.00".to_string(),
            mark_price: "44000.00".to_string(),
            unrealized_pnl: "-200.00".to_string(),
            liquidation_price: "40000.00".to_string(),
            leverage: "10".to_string(),
            max_notional_value: "100000.00".to_string(),
            margin_type: "isolated".to_string(),
            isolated_margin: "900.00".to_string(),
            is_auto_add_margin: false,
            position_side: "LONG".to_string(),
            notional: "8800.00".to_string(),
            isolated_wallet: "1000.00".to_string(),
            update_time: 1609459200000,
        };

        assert!(position.unrealized_pnl.starts_with('-'));
    }

    #[test]
    fn test_module_submodule_access() {
        // Test that submodules are accessible
        use crate::binance::client;
        use crate::binance::types;
        use crate::binance::websocket;

        // Verify modules exist by accessing them
        let _client_module = std::any::type_name::<client::BinanceClient>();
        let _types_module = std::any::type_name::<types::Kline>();
        let _websocket_module = std::any::type_name::<websocket::BinanceWebSocket>();
    }

    #[test]
    fn test_pub_use_reexports() {
        // Test that pub use statements properly re-export types
        // These should compile if re-exports are working correctly
        let _client_type: Option<BinanceClient> = None;
        let _websocket_type: Option<BinanceWebSocket> = None;
        let _kline_type: Option<Kline> = None;
        let _symbol_price_type: Option<SymbolPrice> = None;
        let _funding_rate_type: Option<FundingRate> = None;
    }
}
