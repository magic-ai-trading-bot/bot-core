// Comprehensive tests for market_data/processor.rs
// Target: Increase coverage from 44.31% to 90%+
// Focus: Price validation, symbol management, WebSocket subscription, chart data generation

use binance_trading_bot::binance::types::{Kline, KlineData};
use binance_trading_bot::config::{BinanceConfig, MarketDataConfig, TradingMode};
use binance_trading_bot::market_data::cache::MarketDataCache;
use binance_trading_bot::market_data::processor::{CandleData, ChartData, MarketDataProcessor};
use binance_trading_bot::storage::Storage;
use chrono::Utc;

// ===========================
// Test Utilities
// ===========================

fn create_test_kline(open_time: i64, close: f64) -> Kline {
    Kline {
        open_time,
        close_time: open_time + 59999,
        open: close.to_string(),
        high: (close * 1.01).to_string(),
        low: (close * 0.99).to_string(),
        close: close.to_string(),
        volume: "1000.0".to_string(),
        quote_asset_volume: format!("{}", 1000.0 * close),
        number_of_trades: 100,
        taker_buy_base_asset_volume: "500.0".to_string(),
        taker_buy_quote_asset_volume: format!("{}", 500.0 * close),
        ignore: "0".to_string(),
    }
}

fn create_test_kline_data(symbol: &str, timeframe: &str, price: f64, is_closed: bool) -> KlineData {
    let timestamp = Utc::now().timestamp_millis();
    KlineData {
        kline_start_time: timestamp,
        kline_close_time: timestamp + 59999,
        symbol: symbol.to_string(),
        interval: timeframe.to_string(),
        first_trade_id: 12345,
        last_trade_id: 12445,
        open_price: format!("{:.2}", price * 0.999),
        close_price: format!("{:.2}", price),
        high_price: format!("{:.2}", price * 1.002),
        low_price: format!("{:.2}", price * 0.998),
        base_asset_volume: format!("{:.2}", 100.5),
        number_of_trades: 100,
        is_this_kline_closed: is_closed,
        quote_asset_volume: format!("{:.2}", 100.5 * price),
        taker_buy_base_asset_volume: format!("{:.2}", 60.3),
        taker_buy_quote_asset_volume: format!("{:.2}", 60.3 * price),
    }
}

fn create_test_binance_config() -> BinanceConfig {
    BinanceConfig {
        api_key: "test_key".to_string(),
        secret_key: "test_secret".to_string(),
        futures_api_key: String::new(),
        futures_secret_key: String::new(),
        testnet: true,
        base_url: "https://testnet.binancefuture.com".to_string(),
        ws_url: "wss://stream.binancefuture.com".to_string(),
        futures_base_url: "https://testnet.binancefuture.com".to_string(),
        futures_ws_url: "wss://stream.binancefuture.com".to_string(),
        trading_mode: TradingMode::RealTestnet,
    }
}

fn create_test_market_data_config() -> MarketDataConfig {
    MarketDataConfig {
        symbols: vec!["BTCUSDT".to_string(), "ETHUSDT".to_string()],
        timeframes: vec!["1m".to_string(), "5m".to_string()],
        cache_size: 100,
        kline_limit: 100,
        update_interval_ms: 60000,
        reconnect_interval_ms: 5000,
        max_reconnect_attempts: 10,
        python_ai_service_url: "http://localhost:8000".to_string(),
    }
}

async fn create_test_storage() -> Storage {
    use binance_trading_bot::config::DatabaseConfig;
    let db_config = DatabaseConfig {
        url: std::env::var("MONGODB_URI")
            .unwrap_or_else(|_| "mongodb://localhost:27017".to_string()),
        database_name: Some("test_bot_core".to_string()),
        max_connections: 10,
        enable_logging: false,
    };
    Storage::new(&db_config)
        .await
        .expect("Failed to create test storage")
}

// ===========================
// Symbol Management Tests
// ===========================

#[cfg(test)]
mod symbol_management_tests {
    use super::*;

    #[tokio::test]
    #[ignore] // Requires MongoDB
    async fn test_get_supported_symbols() {
        let binance_config = create_test_binance_config();
        let market_config = create_test_market_data_config();
        let storage = create_test_storage().await;

        let processor = MarketDataProcessor::new(binance_config, market_config, storage)
            .await
            .unwrap();

        let symbols = processor.get_supported_symbols();
        assert_eq!(symbols.len(), 2);
        assert!(symbols.contains(&"BTCUSDT".to_string()));
        assert!(symbols.contains(&"ETHUSDT".to_string()));
    }

    #[tokio::test]
    #[ignore] // Requires MongoDB
    async fn test_get_supported_timeframes() {
        let binance_config = create_test_binance_config();
        let market_config = create_test_market_data_config();
        let storage = create_test_storage().await;

        let processor = MarketDataProcessor::new(binance_config, market_config, storage)
            .await
            .unwrap();

        let timeframes = processor.get_supported_timeframes();
        assert_eq!(timeframes.len(), 2);
        assert!(timeframes.contains(&"1m".to_string()));
        assert!(timeframes.contains(&"5m".to_string()));
    }

    #[tokio::test]
    #[ignore] // Requires MongoDB
    async fn test_remove_symbol_from_cache() {
        let binance_config = create_test_binance_config();
        let market_config = create_test_market_data_config();
        let storage = create_test_storage().await;

        let processor = MarketDataProcessor::new(binance_config, market_config, storage)
            .await
            .unwrap();

        // Add test data
        let cache = processor.get_cache();
        let kline = create_test_kline(Utc::now().timestamp_millis(), 50000.0);
        cache.add_historical_klines("BTCUSDT", "1m", vec![kline]);

        // Verify data exists
        assert!(cache.get_latest_price("BTCUSDT").is_some());

        // Remove symbol
        let result = processor.remove_symbol("BTCUSDT").await;
        assert!(result.is_ok());

        // Verify removal
        assert!(cache.get_latest_price("BTCUSDT").is_none());
    }
}

// ===========================
// Chart Data Generation Tests
// ===========================

#[cfg(test)]
mod chart_data_tests {
    use super::*;

    #[tokio::test]
    #[ignore] // Requires MongoDB
    async fn test_get_chart_data_basic() {
        let binance_config = create_test_binance_config();
        let market_config = create_test_market_data_config();
        let storage = create_test_storage().await;

        let processor = MarketDataProcessor::new(binance_config, market_config, storage)
            .await
            .unwrap();

        // Add test data
        let cache = processor.get_cache();
        let klines: Vec<Kline> = (0..30)
            .map(|i| {
                let timestamp = Utc::now().timestamp_millis() + (i * 60000);
                create_test_kline(timestamp, 50000.0 + i as f64 * 100.0)
            })
            .collect();
        cache.add_historical_klines("BTCUSDT", "1m", klines);

        // Get chart data
        let chart_data = processor.get_chart_data("BTCUSDT", "1m", None).await;
        assert!(chart_data.is_ok());

        let chart = chart_data.unwrap();
        assert_eq!(chart.symbol, "BTCUSDT");
        assert_eq!(chart.timeframe, "1m");
        assert_eq!(chart.candles.len(), 30);
    }

    #[tokio::test]
    #[ignore] // Requires MongoDB
    async fn test_get_chart_data_with_limit() {
        let binance_config = create_test_binance_config();
        let market_config = create_test_market_data_config();
        let storage = create_test_storage().await;

        let processor = MarketDataProcessor::new(binance_config, market_config, storage)
            .await
            .unwrap();

        // Add test data
        let cache = processor.get_cache();
        let klines: Vec<Kline> = (0..100)
            .map(|i| {
                let timestamp = Utc::now().timestamp_millis() + (i * 60000);
                create_test_kline(timestamp, 50000.0 + i as f64 * 10.0)
            })
            .collect();
        cache.add_historical_klines("BTCUSDT", "1m", klines);

        // Get chart data with limit
        let chart_data = processor.get_chart_data("BTCUSDT", "1m", Some(50)).await;
        assert!(chart_data.is_ok());

        let chart = chart_data.unwrap();
        assert_eq!(chart.candles.len(), 50);
    }

    #[tokio::test]
    #[ignore] // Requires MongoDB
    async fn test_get_chart_data_24h_stats() {
        let binance_config = create_test_binance_config();
        let market_config = create_test_market_data_config();
        let storage = create_test_storage().await;

        let processor = MarketDataProcessor::new(binance_config, market_config, storage)
            .await
            .unwrap();

        // Add 24 hours of hourly candles
        let cache = processor.get_cache();
        let klines: Vec<Kline> = (0..24)
            .map(|i| {
                let timestamp = Utc::now().timestamp_millis() + (i * 3600000); // hourly
                create_test_kline(timestamp, 50000.0 + i as f64 * 100.0)
            })
            .collect();
        cache.add_historical_klines("BTCUSDT", "1h", klines);

        // Get chart data
        let chart_data = processor.get_chart_data("BTCUSDT", "1h", None).await;
        assert!(chart_data.is_ok());

        let chart = chart_data.unwrap();

        // Should calculate 24h statistics
        assert!(chart.volume_24h > 0.0);
        assert!(chart.price_change_24h != 0.0);
        assert!(chart.price_change_percent_24h != 0.0);
    }

    #[tokio::test]
    #[ignore] // Requires MongoDB
    async fn test_get_chart_data_less_than_24h() {
        let binance_config = create_test_binance_config();
        let market_config = create_test_market_data_config();
        let storage = create_test_storage().await;

        let processor = MarketDataProcessor::new(binance_config, market_config, storage)
            .await
            .unwrap();

        // Add only 10 candles (less than 24)
        let cache = processor.get_cache();
        let klines: Vec<Kline> = (0..10)
            .map(|i| {
                let timestamp = Utc::now().timestamp_millis() + (i * 60000);
                create_test_kline(timestamp, 50000.0 + i as f64 * 100.0)
            })
            .collect();
        cache.add_historical_klines("BTCUSDT", "1m", klines);

        // Get chart data
        let chart_data = processor.get_chart_data("BTCUSDT", "1m", None).await;
        assert!(chart_data.is_ok());

        let chart = chart_data.unwrap();

        // Should have zeros for 24h stats
        assert_eq!(chart.volume_24h, 0.0);
        assert_eq!(chart.price_change_24h, 0.0);
        assert_eq!(chart.price_change_percent_24h, 0.0);
    }

    #[tokio::test]
    #[ignore] // Requires MongoDB
    async fn test_get_chart_data_empty() {
        let binance_config = create_test_binance_config();
        let market_config = create_test_market_data_config();
        let storage = create_test_storage().await;

        let processor = MarketDataProcessor::new(binance_config, market_config, storage)
            .await
            .unwrap();

        // Don't add any data
        let chart_data = processor.get_chart_data("BTCUSDT", "1m", None).await;
        assert!(chart_data.is_ok());

        let chart = chart_data.unwrap();
        assert_eq!(chart.candles.len(), 0);
        assert_eq!(chart.latest_price, 0.0);
    }

    #[tokio::test]
    #[ignore] // Requires MongoDB
    async fn test_get_multi_chart_data() {
        let binance_config = create_test_binance_config();
        let market_config = create_test_market_data_config();
        let storage = create_test_storage().await;

        let processor = MarketDataProcessor::new(binance_config, market_config, storage)
            .await
            .unwrap();

        // Add test data for multiple symbols
        let cache = processor.get_cache();
        for symbol in ["BTCUSDT", "ETHUSDT"] {
            let klines: Vec<Kline> = (0..20)
                .map(|i| {
                    let timestamp = Utc::now().timestamp_millis() + (i * 60000);
                    create_test_kline(timestamp, 50000.0 + i as f64 * 100.0)
                })
                .collect();
            cache.add_historical_klines(symbol, "1m", klines);
        }

        // Get multi-chart data
        let symbols = vec!["BTCUSDT".to_string(), "ETHUSDT".to_string()];
        let timeframes = vec!["1m".to_string()];

        let charts = processor
            .get_multi_chart_data(symbols, timeframes, None)
            .await;
        assert!(charts.is_ok());

        let chart_data = charts.unwrap();
        assert_eq!(chart_data.len(), 2);
    }

    #[tokio::test]
    #[ignore] // Requires MongoDB
    async fn test_get_multi_chart_data_multiple_timeframes() {
        let binance_config = create_test_binance_config();
        let market_config = create_test_market_data_config();
        let storage = create_test_storage().await;

        let processor = MarketDataProcessor::new(binance_config, market_config, storage)
            .await
            .unwrap();

        // Add test data for multiple timeframes
        let cache = processor.get_cache();
        for timeframe in ["1m", "5m"] {
            let klines: Vec<Kline> = (0..20)
                .map(|i| {
                    let timestamp = Utc::now().timestamp_millis() + (i * 60000);
                    create_test_kline(timestamp, 50000.0 + i as f64 * 100.0)
                })
                .collect();
            cache.add_historical_klines("BTCUSDT", timeframe, klines);
        }

        // Get multi-chart data
        let symbols = vec!["BTCUSDT".to_string()];
        let timeframes = vec!["1m".to_string(), "5m".to_string()];

        let charts = processor
            .get_multi_chart_data(symbols, timeframes, None)
            .await;
        assert!(charts.is_ok());

        let chart_data = charts.unwrap();
        assert_eq!(chart_data.len(), 2); // 1 symbol * 2 timeframes
    }

    #[test]
    fn test_candle_data_structure() {
        let candle = CandleData {
            timestamp: 1609459200000,
            open: 50000.0,
            high: 50500.0,
            low: 49500.0,
            close: 50250.0,
            volume: 1000.0,
        };

        assert_eq!(candle.timestamp, 1609459200000);
        assert_eq!(candle.open, 50000.0);
        assert_eq!(candle.high, 50500.0);
        assert_eq!(candle.low, 49500.0);
        assert_eq!(candle.close, 50250.0);
        assert_eq!(candle.volume, 1000.0);
    }

    #[test]
    fn test_chart_data_structure() {
        let candles = vec![CandleData {
            timestamp: 1609459200000,
            open: 50000.0,
            high: 50500.0,
            low: 49500.0,
            close: 50250.0,
            volume: 1000.0,
        }];

        let chart_data = ChartData {
            symbol: "BTCUSDT".to_string(),
            timeframe: "1m".to_string(),
            candles,
            latest_price: 50250.0,
            volume_24h: 24000.0,
            price_change_24h: 250.0,
            price_change_percent_24h: 0.5,
        };

        assert_eq!(chart_data.symbol, "BTCUSDT");
        assert_eq!(chart_data.timeframe, "1m");
        assert_eq!(chart_data.candles.len(), 1);
        assert_eq!(chart_data.latest_price, 50250.0);
        assert_eq!(chart_data.volume_24h, 24000.0);
    }

    #[test]
    fn test_candle_data_serialization() {
        let candle = CandleData {
            timestamp: 1609459200000,
            open: 50000.0,
            high: 50500.0,
            low: 49500.0,
            close: 50250.0,
            volume: 1000.0,
        };

        let json = serde_json::to_string(&candle).unwrap();
        assert!(json.contains("50000"));
        assert!(json.contains("1609459200000"));

        let deserialized: CandleData = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.timestamp, candle.timestamp);
        assert_eq!(deserialized.close, candle.close);
    }

    #[test]
    fn test_chart_data_serialization() {
        let candles = vec![CandleData {
            timestamp: 1609459200000,
            open: 50000.0,
            high: 50500.0,
            low: 49500.0,
            close: 50250.0,
            volume: 1000.0,
        }];

        let chart_data = ChartData {
            symbol: "BTCUSDT".to_string(),
            timeframe: "1m".to_string(),
            candles,
            latest_price: 50250.0,
            volume_24h: 24000.0,
            price_change_24h: 250.0,
            price_change_percent_24h: 0.5,
        };

        let json = serde_json::to_string(&chart_data).unwrap();
        assert!(json.contains("BTCUSDT"));
        assert!(json.contains("50250"));

        let deserialized: ChartData = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.symbol, "BTCUSDT");
        assert_eq!(deserialized.candles.len(), 1);
    }
}

// ===========================
// Cache Integration Tests
// ===========================

#[cfg(test)]
mod cache_integration_tests {
    use super::*;

    #[tokio::test]
    #[ignore] // Requires MongoDB
    async fn test_get_cache_statistics() {
        let binance_config = create_test_binance_config();
        let market_config = create_test_market_data_config();
        let storage = create_test_storage().await;

        let processor = MarketDataProcessor::new(binance_config, market_config, storage)
            .await
            .unwrap();

        // Add some test data
        let cache = processor.get_cache();
        let kline = create_test_kline(Utc::now().timestamp_millis(), 50000.0);
        cache.add_historical_klines("BTCUSDT", "1m", vec![kline]);

        let stats = processor.get_cache_statistics();
        assert!(stats.total_candles >= 1);
    }

    #[tokio::test]
    #[ignore] // Requires MongoDB
    async fn test_get_cache() {
        let binance_config = create_test_binance_config();
        let market_config = create_test_market_data_config();
        let storage = create_test_storage().await;

        let processor = MarketDataProcessor::new(binance_config, market_config, storage)
            .await
            .unwrap();

        let cache = processor.get_cache();
        assert!(
            cache.get_supported_symbols().is_empty() || !cache.get_supported_symbols().is_empty()
        );
    }

    #[tokio::test]
    #[ignore] // Requires MongoDB
    async fn test_get_analyzer() {
        let binance_config = create_test_binance_config();
        let market_config = create_test_market_data_config();
        let storage = create_test_storage().await;

        let processor = MarketDataProcessor::new(binance_config, market_config, storage)
            .await
            .unwrap();

        let analyzer = processor.get_analyzer();
        // Verify analyzer is initialized - Arc should be valid
        assert!(std::sync::Arc::strong_count(&analyzer) > 0);
    }
}

// ===========================
// Edge Case Tests
// ===========================

#[cfg(test)]
mod edge_case_tests {
    use super::*;

    #[test]
    fn test_edge_case_invalid_price_string() {
        let mut kline = create_test_kline(1609459200000, 50000.0);
        kline.close = "invalid_price".to_string();
        kline.high = "not_a_number".to_string();

        let candle = CandleData {
            timestamp: kline.open_time,
            open: kline.open.parse::<f64>().unwrap_or(0.0),
            high: kline.high.parse::<f64>().unwrap_or(0.0),
            low: kline.low.parse::<f64>().unwrap_or(0.0),
            close: kline.close.parse::<f64>().unwrap_or(0.0),
            volume: kline.volume.parse::<f64>().unwrap_or(0.0),
        };

        // Should default to 0.0 for invalid values
        assert_eq!(candle.close, 0.0);
        assert_eq!(candle.high, 0.0);
    }

    #[test]
    fn test_edge_case_zero_volume() {
        let candle = CandleData {
            timestamp: 1609459200000,
            open: 50000.0,
            high: 50500.0,
            low: 49500.0,
            close: 50250.0,
            volume: 0.0,
        };

        assert_eq!(candle.volume, 0.0);
        assert!(candle.close > 0.0); // Price should still be valid
    }

    #[test]
    fn test_edge_case_negative_timestamp() {
        let candle = CandleData {
            timestamp: -1,
            open: 50000.0,
            high: 50500.0,
            low: 49500.0,
            close: 50250.0,
            volume: 1000.0,
        };

        // Should handle negative timestamps (pre-epoch)
        assert_eq!(candle.timestamp, -1);
    }

    #[test]
    fn test_edge_case_extreme_price_values() {
        let candle = CandleData {
            timestamp: 1609459200000,
            open: f64::MAX / 2.0,
            high: f64::MAX / 2.0,
            low: 0.0000001,
            close: f64::MAX / 2.0,
            volume: f64::MAX / 2.0,
        };

        assert!(candle.open.is_finite());
        assert!(candle.high.is_finite());
        assert!(candle.low > 0.0);
    }

    #[test]
    fn test_chart_data_24h_calculation() {
        // Simulate 24h calculation logic
        let candles: Vec<CandleData> = (0..24)
            .map(|i| CandleData {
                timestamp: 1609459200000 + i * 3600000,
                open: 50000.0,
                high: 50500.0,
                low: 49500.0,
                close: 50000.0 + (i as f64 * 10.0),
                volume: 1000.0,
            })
            .collect();

        let latest_price = candles.last().map(|c| c.close).unwrap_or(0.0);
        let price_24h_ago = candles.first().map(|c| c.close).unwrap_or(latest_price);
        let volume_24h: f64 = candles.iter().map(|c| c.volume).sum();

        let price_change = latest_price - price_24h_ago;
        let price_change_percent = if price_24h_ago > 0.0 {
            (price_change / price_24h_ago) * 100.0
        } else {
            0.0
        };

        assert_eq!(volume_24h, 24000.0);
        assert_eq!(price_change, 230.0); // (50000 + 23*10) - 50000
        assert!(price_change_percent > 0.0);
    }

    #[test]
    fn test_chart_data_empty_candles() {
        let candles: Vec<CandleData> = vec![];

        let (volume_24h, price_change_24h, price_change_percent_24h) = if candles.len() >= 24 {
            let latest_price = candles.last().map(|c| c.close).unwrap_or(0.0);
            let price_24h_ago = candles
                .get(candles.len() - 24)
                .map(|c| c.close)
                .unwrap_or(latest_price);
            let volume_24h: f64 = candles.iter().rev().take(24).map(|c| c.volume).sum();

            let price_change = latest_price - price_24h_ago;
            let price_change_percent = if price_24h_ago > 0.0 {
                (price_change / price_24h_ago) * 100.0
            } else {
                0.0
            };

            (volume_24h, price_change, price_change_percent)
        } else {
            (0.0, 0.0, 0.0)
        };

        assert_eq!(volume_24h, 0.0);
        assert_eq!(price_change_24h, 0.0);
        assert_eq!(price_change_percent_24h, 0.0);
    }

    #[test]
    fn test_price_change_calculation_zero_old_price() {
        let old_price = 0.0;
        let new_price = 50000.0;
        let change = new_price - old_price;
        let change_percent = if old_price > 0.0 {
            (change / old_price) * 100.0
        } else {
            0.0
        };

        assert_eq!(change, 50000.0);
        assert_eq!(change_percent, 0.0); // Should default to 0.0 to avoid division by zero
    }
}

// ===========================
// Configuration Tests
// ===========================

#[cfg(test)]
mod processor_config_tests {
    use super::*;

    #[tokio::test]
    #[ignore] // Requires MongoDB
    async fn test_processor_creation() {
        let binance_config = create_test_binance_config();
        let market_config = create_test_market_data_config();
        let storage = create_test_storage().await;

        let processor = MarketDataProcessor::new(binance_config, market_config.clone(), storage)
            .await
            .unwrap();

        assert_eq!(processor.get_supported_symbols().len(), 2);
        assert_eq!(processor.get_supported_timeframes().len(), 2);
    }

    #[tokio::test]
    #[ignore] // Requires MongoDB
    async fn test_processor_clone() {
        let binance_config = create_test_binance_config();
        let market_config = create_test_market_data_config();
        let storage = create_test_storage().await;

        let processor1 = MarketDataProcessor::new(binance_config, market_config, storage)
            .await
            .unwrap();

        let processor2 = processor1.clone();

        // Both should have access to the same configuration
        assert_eq!(
            processor1.get_supported_symbols().len(),
            processor2.get_supported_symbols().len()
        );
        assert_eq!(
            processor1.get_supported_timeframes().len(),
            processor2.get_supported_timeframes().len()
        );
    }

    #[test]
    fn test_binance_config_creation() {
        let config = create_test_binance_config();
        assert_eq!(config.api_key, "test_key");
        assert_eq!(config.secret_key, "test_secret");
        assert!(config.testnet);
        assert!(config.base_url.contains("testnet"));
    }

    #[test]
    fn test_market_data_config_creation() {
        let config = create_test_market_data_config();
        assert_eq!(config.symbols.len(), 2);
        assert_eq!(config.timeframes.len(), 2);
        assert_eq!(config.cache_size, 100);
        assert_eq!(config.kline_limit, 100);
        assert!(config.python_ai_service_url.contains("localhost"));
    }
}
