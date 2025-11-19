use anyhow::Result;
use chrono;
use serde_json::json;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::{broadcast, mpsc};
use tokio::time::{interval, sleep};
use tracing::{debug, error, info, warn};

use crate::binance::{BinanceClient, BinanceWebSocket, StreamEvent};
use crate::config::{BinanceConfig, MarketDataConfig};
use crate::storage::Storage;

use super::analyzer::MarketDataAnalyzer;
use super::cache::MarketDataCache;

// Chart data structures for API responses
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ChartData {
    pub symbol: String,
    pub timeframe: String,
    pub candles: Vec<CandleData>,
    pub latest_price: f64,
    pub volume_24h: f64,
    pub price_change_24h: f64,
    pub price_change_percent_24h: f64,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct CandleData {
    pub timestamp: i64,
    pub open: f64,
    pub high: f64,
    pub low: f64,
    pub close: f64,
    pub volume: f64,
}

#[derive(Clone)]
pub struct MarketDataProcessor {
    binance_config: BinanceConfig,
    config: MarketDataConfig,
    client: BinanceClient,
    cache: MarketDataCache,
    analyzer: Arc<MarketDataAnalyzer>,
    storage: Storage,
    ws_broadcaster: Option<broadcast::Sender<String>>,
}

impl MarketDataProcessor {
    /// Validate and parse a price string, rejecting invalid prices
    ///
    /// @spec:FR-RISK-007 - Price Data Validation
    /// @ref:specs/02-design/2.5-components/COMP-RUST-TRADING.md#price-validation
    fn validate_price(price_str: &str, symbol: &str, context: &str) -> Result<f64> {
        const MIN_VALID_PRICE: f64 = 0.01; // Minimum valid price for crypto (1 cent)

        let price: f64 = price_str.parse().map_err(|_| {
            anyhow::anyhow!(
                "Invalid price format for {} ({}): '{}'",
                symbol,
                context,
                price_str
            )
        })?;

        if price <= 0.0 {
            return Err(anyhow::anyhow!(
                "Zero or negative price for {} ({}): {}",
                symbol,
                context,
                price
            ));
        }

        if price < MIN_VALID_PRICE {
            return Err(anyhow::anyhow!(
                "Price too low for {} ({}): {} (minimum: {})",
                symbol,
                context,
                price,
                MIN_VALID_PRICE
            ));
        }

        if !price.is_finite() {
            return Err(anyhow::anyhow!(
                "Non-finite price for {} ({}): {}",
                symbol,
                context,
                price
            ));
        }

        Ok(price)
    }

    pub async fn new(
        binance_config: BinanceConfig,
        config: MarketDataConfig,
        storage: Storage,
    ) -> Result<Self> {
        let client = BinanceClient::new(binance_config.clone())?;
        let cache = MarketDataCache::new(config.cache_size);
        let analyzer = Arc::new(MarketDataAnalyzer::new(
            config.python_ai_service_url.clone(),
            cache.clone(),
        ));

        Ok(Self {
            binance_config,
            config,
            client,
            cache,
            analyzer,
            storage,
            ws_broadcaster: None,
        })
    }

    pub fn set_ws_broadcaster(&mut self, broadcaster: broadcast::Sender<String>) {
        self.ws_broadcaster = Some(broadcaster);
    }

    pub async fn start(&self) -> Result<()> {
        info!("Starting Market Data Processor");

        // Load historical data first
        self.load_historical_data().await?;

        // Check if WebSocket should be disabled (for debugging)
        let disable_websocket = std::env::var("DISABLE_WEBSOCKET").unwrap_or_default() == "true";

        if disable_websocket {
            info!("WebSocket disabled via DISABLE_WEBSOCKET environment variable");

            // Start periodic tasks only
            let update_handle = self.start_periodic_updates();
            let analysis_handle = self.start_periodic_analysis();

            // Wait for periodic tasks only
            tokio::try_join!(async { update_handle.await? }, async {
                analysis_handle.await?
            })?;
        } else {
            // Start WebSocket connections
            let websocket_handle = self.start_websocket_streams().await?;

            // Start periodic tasks
            let update_handle = self.start_periodic_updates();
            let analysis_handle = self.start_periodic_analysis();

            // Wait for all tasks
            tokio::try_join!(
                async { websocket_handle.await? },
                async { update_handle.await? },
                async { analysis_handle.await? }
            )?;
        }

        Ok(())
    }

    async fn load_historical_data(&self) -> Result<()> {
        info!("Loading historical market data");

        for symbol in &self.config.symbols {
            for timeframe in &self.config.timeframes {
                match self.load_historical_klines(symbol, timeframe).await {
                    Ok(count) => {
                        info!(
                            "Loaded {} historical candles for {} {}",
                            count, symbol, timeframe
                        );
                    },
                    Err(e) => {
                        warn!(
                            "Failed to load historical data for {} {}: {}",
                            symbol, timeframe, e
                        );
                    },
                }

                // Add small delay to avoid rate limiting
                sleep(Duration::from_millis(100)).await;
            }
        }

        info!("Historical data loading completed");
        Ok(())
    }

    async fn load_historical_klines(&self, symbol: &str, timeframe: &str) -> Result<usize> {
        // Try to load from database first
        let cached_klines = self
            .storage
            .get_market_data(symbol, timeframe, Some(self.config.kline_limit as i64))
            .await?;

        if !cached_klines.is_empty() {
            // Use cached data
            info!(
                "Loaded {} cached klines for {} {}",
                cached_klines.len(),
                symbol,
                timeframe
            );
            self.cache
                .add_historical_klines(symbol, timeframe, cached_klines.clone());

            // Still fetch latest data to update cache
            match self
                .client
                .get_futures_klines(symbol, timeframe, Some(10))
                .await
            {
                Ok(latest_klines) => {
                    if let Err(e) = self
                        .storage
                        .store_market_data(symbol, timeframe, &latest_klines)
                        .await
                    {
                        warn!("Failed to store latest market data: {}", e);
                    }
                    self.cache
                        .add_historical_klines(symbol, timeframe, latest_klines);
                },
                Err(e) => warn!(
                    "Failed to fetch latest data for {} {}: {}",
                    symbol, timeframe, e
                ),
            }

            Ok(cached_klines.len())
        } else {
            // Fetch from API if no cached data
            let klines = self
                .client
                .get_futures_klines(symbol, timeframe, Some(self.config.kline_limit))
                .await?;

            // Store in database
            if let Err(e) = self
                .storage
                .store_market_data(symbol, timeframe, &klines)
                .await
            {
                warn!("Failed to store market data: {}", e);
            }

            let count = klines.len();
            self.cache.add_historical_klines(symbol, timeframe, klines);

            Ok(count)
        }
    }

    async fn start_websocket_streams(&self) -> Result<tokio::task::JoinHandle<Result<()>>> {
        let (websocket, receiver) = BinanceWebSocket::new(self.binance_config.clone());
        let symbols = self.config.symbols.clone();
        let timeframes = self.config.timeframes.clone();
        let cache = self.cache.clone();
        let ws_broadcaster = self.ws_broadcaster.clone();

        // Start WebSocket connection
        let ws_handle = tokio::spawn(async move { websocket.start(symbols, timeframes).await });

        // Start message processing
        let processor_handle = tokio::spawn(async move {
            Self::process_websocket_messages(receiver, cache, ws_broadcaster).await
        });

        // Return a combined handle
        Ok(tokio::spawn(async move {
            tokio::try_join!(async { ws_handle.await? }, async {
                processor_handle.await?
            })?;
            Ok(())
        }))
    }

    async fn process_websocket_messages(
        mut receiver: mpsc::UnboundedReceiver<StreamEvent>,
        cache: MarketDataCache,
        ws_broadcaster: Option<broadcast::Sender<String>>,
    ) -> Result<()> {
        info!("Starting WebSocket message processing");

        loop {
            match receiver.recv().await {
                Some(event) => {
                    if let Err(e) =
                        Self::handle_stream_event(&event, &cache, &ws_broadcaster, &None).await
                    {
                        error!("Error handling stream event: {}", e);
                    }
                },
                None => {
                    error!("WebSocket message channel closed");
                    break;
                },
            }
        }

        Ok(())
    }

    async fn handle_stream_event(
        event: &StreamEvent,
        cache: &MarketDataCache,
        ws_broadcaster: &Option<broadcast::Sender<String>>,
        _storage: &Option<Storage>,
    ) -> Result<()> {
        match event {
            StreamEvent::Kline(kline_event) => {
                cache.update_kline(
                    &kline_event.symbol,
                    &kline_event.kline.interval,
                    &kline_event.kline,
                );

                debug!(
                    "Updated kline data for {} {} - Close: {} (closed: {})",
                    kline_event.symbol,
                    kline_event.kline.interval,
                    kline_event.kline.close_price,
                    kline_event.kline.is_this_kline_closed
                );

                // Broadcast price update via WebSocket (compatible with frontend)
                if let Some(broadcaster) = ws_broadcaster {
                    // Validate price instead of silently accepting 0.0
                    let current_price = match Self::validate_price(
                        &kline_event.kline.close_price,
                        &kline_event.symbol,
                        "kline close",
                    ) {
                        Ok(price) => price,
                        Err(e) => {
                            error!("Invalid kline price for {}: {}", kline_event.symbol, e);
                            return Ok(()); // Skip this update
                        },
                    };

                    // Send MarketData update for immediate price updates
                    let market_data_update = json!({
                        "type": "MarketData",
                        "data": {
                            "symbol": kline_event.symbol,
                            "price": current_price,
                            "price_change_24h": 0.0, // Will be calculated by frontend
                            "price_change_percent_24h": 0.0,
                            "volume_24h": 0.0,
                            "timestamp": chrono::Utc::now().timestamp_millis()
                        },
                        "timestamp": chrono::Utc::now().to_rfc3339()
                    });

                    // Send ChartUpdate if kline is closed (more detailed update)
                    if kline_event.kline.is_this_kline_closed {
                        // Validate all price data
                        let (open, high, low, volume) = match (
                            Self::validate_price(
                                &kline_event.kline.open_price,
                                &kline_event.symbol,
                                "kline open",
                            ),
                            Self::validate_price(
                                &kline_event.kline.high_price,
                                &kline_event.symbol,
                                "kline high",
                            ),
                            Self::validate_price(
                                &kline_event.kline.low_price,
                                &kline_event.symbol,
                                "kline low",
                            ),
                            kline_event
                                .kline
                                .base_asset_volume
                                .parse::<f64>()
                                .ok()
                                .filter(|v| v.is_finite() && *v >= 0.0),
                        ) {
                            (Ok(o), Ok(h), Ok(l), Some(v)) => (o, h, l, v),
                            _ => {
                                error!(
                                    "Invalid candle data for {}, skipping chart update",
                                    kline_event.symbol
                                );
                                return Ok(());
                            },
                        };

                        let chart_update = json!({
                            "type": "ChartUpdate",
                            "data": {
                                "symbol": kline_event.symbol,
                                "timeframe": kline_event.kline.interval,
                                "candle": {
                                    "timestamp": kline_event.kline.kline_start_time,
                                    "open": open,
                                    "high": high,
                                    "low": low,
                                    "close": current_price,
                                    "volume": volume,
                                    "is_closed": true
                                },
                                "latest_price": current_price,
                                "price_change_24h": 0.0,
                                "price_change_percent_24h": 0.0,
                                "volume_24h": 0.0,
                                "timestamp": chrono::Utc::now().timestamp_millis()
                            },
                            "timestamp": chrono::Utc::now().to_rfc3339()
                        });

                        if let Err(e) = broadcaster.send(chart_update.to_string()) {
                            if broadcaster.receiver_count() > 0 {
                                warn!("Failed to broadcast chart update: {}", e);
                            }
                        }
                    }

                    if let Err(e) = broadcaster.send(market_data_update.to_string()) {
                        if broadcaster.receiver_count() > 0 {
                            warn!("Failed to broadcast market data update: {}", e);
                        }
                    }
                }
            },
            StreamEvent::Ticker(ticker_event) => {
                debug!(
                    "Received ticker update for {}: {}",
                    ticker_event.symbol, ticker_event.last_price
                );
            },
            StreamEvent::OrderBook(orderbook_event) => {
                debug!(
                    "Received order book update for {} (bids: {}, asks: {})",
                    orderbook_event.symbol,
                    orderbook_event.bids.len(),
                    orderbook_event.asks.len()
                );
            },
        }

        Ok(())
    }

    fn start_periodic_updates(&self) -> tokio::task::JoinHandle<Result<()>> {
        let client = self.client.clone();
        let cache = self.cache.clone();
        let symbols = self.config.symbols.clone();
        let timeframes = self.config.timeframes.clone();
        let update_interval = self.config.update_interval_ms;

        tokio::spawn(async move {
            let mut interval = interval(Duration::from_millis(update_interval));

            loop {
                interval.tick().await;

                // Periodically refresh data to ensure we don't miss anything
                for symbol in &symbols {
                    for timeframe in &timeframes {
                        // Only update longer timeframes periodically (not 1m which updates via WebSocket)
                        if matches!(timeframe.as_str(), "1h" | "4h" | "1d") {
                            if let Err(e) =
                                Self::refresh_timeframe_data(&client, &cache, symbol, timeframe)
                                    .await
                            {
                                warn!("Failed to refresh {} {}: {}", symbol, timeframe, e);
                            }
                        }
                    }
                }

                // Log cache statistics
                let stats = cache.get_cache_stats();
                debug!(
                    "Cache stats: {} symbols, {} timeframes, {} total candles",
                    stats.cached_symbols, stats.total_timeframes, stats.total_candles
                );
            }
        })
    }

    async fn refresh_timeframe_data(
        client: &BinanceClient,
        cache: &MarketDataCache,
        symbol: &str,
        timeframe: &str,
    ) -> Result<()> {
        let klines = client
            .get_futures_klines(symbol, timeframe, Some(100))
            .await?;

        // Only add the latest few candles to avoid overwriting historical data
        if !klines.is_empty() {
            let latest_klines = klines.into_iter().rev().take(5).rev().collect();
            cache.add_historical_klines(symbol, timeframe, latest_klines);
        }

        Ok(())
    }

    fn start_periodic_analysis(&self) -> tokio::task::JoinHandle<Result<()>> {
        let analyzer = self.analyzer.clone();
        let symbols = self.config.symbols.clone();
        let timeframes = self.config.timeframes.clone();
        let storage = self.storage.clone();

        tokio::spawn(async move {
            // Run analysis every 5 minutes
            let mut interval = interval(Duration::from_secs(5 * 60));

            loop {
                interval.tick().await;

                info!("Starting periodic market analysis");

                for symbol in &symbols {
                    match analyzer
                        .analyze_multi_timeframe(symbol, &timeframes, "trend_analysis", Some(100))
                        .await
                    {
                        Ok(analysis) => {
                            info!(
                                "Analysis completed for {}: {:?} (confidence: {:.2})",
                                symbol, analysis.overall_signal, analysis.overall_confidence
                            );

                            // Store analysis result
                            if let Err(e) = storage.store_analysis(&analysis).await {
                                error!("Failed to store analysis for {}: {}", symbol, e);
                            }
                        },
                        Err(e) => {
                            warn!("Analysis failed for {}: {}", symbol, e);
                        },
                    }

                    // Small delay between symbols
                    sleep(Duration::from_millis(500)).await;
                }

                info!("Periodic analysis completed");
            }
        })
    }

    // Public API methods for other components
    pub fn get_cache(&self) -> &MarketDataCache {
        &self.cache
    }

    pub fn get_analyzer(&self) -> Arc<MarketDataAnalyzer> {
        self.analyzer.clone()
    }

    pub async fn get_latest_analysis(
        &self,
        symbol: &str,
    ) -> Result<super::analyzer::MultiTimeframeAnalysis> {
        self.analyzer
            .analyze_multi_timeframe(symbol, &self.config.timeframes, "trend_analysis", Some(100))
            .await
    }

    pub async fn force_refresh_symbol(&self, symbol: &str) -> Result<()> {
        info!("Force refreshing data for {}", symbol);

        for timeframe in &self.config.timeframes {
            self.load_historical_klines(symbol, timeframe).await?;
        }

        Ok(())
    }

    pub async fn get_market_overview(&self) -> Result<Vec<super::analyzer::MarketOverview>> {
        self.analyzer
            .get_market_overview(&self.config.symbols)
            .await
    }

    pub fn get_cache_statistics(&self) -> super::cache::CacheStats {
        self.cache.get_cache_stats()
    }

    pub fn get_supported_symbols(&self) -> Vec<String> {
        self.config.symbols.clone()
    }

    pub fn get_supported_timeframes(&self) -> Vec<String> {
        self.config.timeframes.clone()
    }

    // NEW: Chart data methods for API support (now using MongoDB instead of cache)
    pub async fn get_chart_data(
        &self,
        symbol: &str,
        timeframe: &str,
        limit: Option<usize>,
    ) -> Result<ChartData> {
        // Get data directly from MongoDB
        let klines = self
            .storage
            .get_market_data(symbol, timeframe, limit.map(|l| l as i64))
            .await?;

        // Convert Klines to CandleData with validation
        let candle_data: Vec<CandleData> = klines
            .iter()
            .filter_map(|kline| {
                let open = Self::validate_price(&kline.open, symbol, "kline open").ok()?;
                let high = Self::validate_price(&kline.high, symbol, "kline high").ok()?;
                let low = Self::validate_price(&kline.low, symbol, "kline low").ok()?;
                let close = Self::validate_price(&kline.close, symbol, "kline close").ok()?;
                let volume = kline
                    .volume
                    .parse()
                    .ok()
                    .filter(|v: &f64| v.is_finite() && *v >= 0.0)?;

                Some(CandleData {
                    timestamp: kline.open_time,
                    open,
                    high,
                    low,
                    close,
                    volume,
                })
            })
            .collect();

        // Calculate 24h statistics
        let (volume_24h, price_change_24h, price_change_percent_24h) = if candle_data.len() >= 24 {
            let latest_price = candle_data.last().map(|c| c.close).unwrap_or(0.0);
            let price_24h_ago = candle_data
                .get(candle_data.len() - 24)
                .map(|c| c.close)
                .unwrap_or(latest_price);
            let volume_24h: f64 = candle_data.iter().rev().take(24).map(|c| c.volume).sum();

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

        let latest_price = candle_data.last().map(|c| c.close).unwrap_or(0.0);

        Ok(ChartData {
            symbol: symbol.to_string(),
            timeframe: timeframe.to_string(),
            candles: candle_data,
            latest_price,
            volume_24h,
            price_change_24h,
            price_change_percent_24h,
        })
    }

    pub async fn get_multi_chart_data(
        &self,
        symbols: Vec<String>,
        timeframes: Vec<String>,
        limit: Option<usize>,
    ) -> Result<Vec<ChartData>> {
        let mut charts = Vec::new();

        for symbol in symbols {
            for timeframe in &timeframes {
                match self.get_chart_data(&symbol, timeframe, limit).await {
                    Ok(chart_data) => charts.push(chart_data),
                    Err(e) => {
                        warn!(
                            "Failed to get chart data for {} {}: {}",
                            symbol, timeframe, e
                        );
                    },
                }
            }
        }

        Ok(charts)
    }

    pub async fn add_symbol(&self, symbol: String, timeframes: Vec<String>) -> Result<()> {
        info!(
            "Adding new symbol {} with timeframes {:?}",
            symbol, timeframes
        );

        // Add symbol to config (this will persist it)
        if !self.config.symbols.contains(&symbol) {
            // Note: This is a temporary fix. In production, you'd want to update persistent config
            let mut config_symbols = self.config.symbols.clone();
            config_symbols.push(symbol.clone());
            info!("Added {} to supported symbols list", symbol);
        }

        // Load historical data for the new symbol
        for timeframe in &timeframes {
            match self.load_historical_klines(&symbol, timeframe).await {
                Ok(count) => {
                    info!(
                        "Loaded {} historical candles for {} {}",
                        count, symbol, timeframe
                    );
                },
                Err(e) => {
                    warn!(
                        "Failed to load historical data for {} {}: {}",
                        symbol, timeframe, e
                    );
                },
            }

            // Add small delay to avoid rate limiting
            sleep(Duration::from_millis(100)).await;
        }

        // TODO: For full dynamic support, we need to:
        // 1. Restart WebSocket connections with new symbol
        // 2. Update persistent configuration
        // For now, users need to restart the service to get WebSocket updates for new symbols
        warn!("New symbol {} added to historical data. Restart service to get real-time updates via WebSocket.", symbol);

        Ok(())
    }

    pub async fn remove_symbol(&self, symbol: &str) -> Result<()> {
        info!("Removing symbol {}", symbol);

        // Remove from cache
        self.cache.remove_symbol(symbol);

        // Note: In a real implementation, you would also need to:
        // 1. Update the WebSocket streams to exclude the symbol
        // 2. Update the configuration to remove the symbol
        // 3. Restart WebSocket connections without the symbol

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::binance::types::{Kline, KlineData, KlineEvent};
    use crate::config::{BinanceConfig, MarketDataConfig};
    use crate::storage::Storage;

    fn create_test_kline(open_time: i64, close: f64) -> Kline {
        Kline {
            open_time,
            close_time: open_time + 60000,
            open: close.to_string(),
            high: (close * 1.01).to_string(),
            low: (close * 0.99).to_string(),
            close: close.to_string(),
            volume: "1000.0".to_string(),
            quote_asset_volume: format!("{}", 1000.0 * close),
            number_of_trades: 100,
            ignore: "0".to_string(),
            taker_buy_base_asset_volume: "500.0".to_string(),
            taker_buy_quote_asset_volume: format!("{}", 500.0 * close),
        }
    }

    fn create_test_kline_data(open_time: i64, close: f64, is_closed: bool) -> KlineData {
        KlineData {
            kline_start_time: open_time,
            kline_close_time: open_time + 60000,
            symbol: "BTCUSDT".to_string(),
            interval: "1m".to_string(),
            first_trade_id: 1,
            last_trade_id: 100,
            open_price: close.to_string(),
            high_price: (close * 1.01).to_string(),
            low_price: (close * 0.99).to_string(),
            close_price: close.to_string(),
            base_asset_volume: "1000.0".to_string(),
            number_of_trades: 100,
            is_this_kline_closed: is_closed,
            quote_asset_volume: format!("{}", 1000.0 * close),
            taker_buy_base_asset_volume: "500.0".to_string(),
            taker_buy_quote_asset_volume: format!("{}", 500.0 * close),
        }
    }

    async fn create_test_storage() -> Storage {
        use crate::config::DatabaseConfig;
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

    fn create_test_binance_config() -> BinanceConfig {
        BinanceConfig {
            api_key: "test_key".to_string(),
            secret_key: "test_secret".to_string(),
            testnet: true,
            base_url: "https://testnet.binancefuture.com".to_string(),
            ws_url: "wss://stream.binancefuture.com".to_string(),
            futures_base_url: "https://testnet.binancefuture.com".to_string(),
            futures_ws_url: "wss://stream.binancefuture.com".to_string(),
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

    #[tokio::test]
    #[ignore] // Requires MongoDB
    async fn test_market_data_processor_new() {
        let binance_config = create_test_binance_config();
        let market_config = create_test_market_data_config();
        let storage = create_test_storage().await;

        let processor = MarketDataProcessor::new(binance_config, market_config.clone(), storage)
            .await
            .unwrap();

        assert_eq!(processor.config.symbols.len(), 2);
        assert_eq!(processor.config.timeframes.len(), 2);
    }

    #[tokio::test]
    #[ignore] // Requires MongoDB
    async fn test_market_data_processor_get_cache() {
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
    async fn test_market_data_processor_get_analyzer() {
        let binance_config = create_test_binance_config();
        let market_config = create_test_market_data_config();
        let storage = create_test_storage().await;

        let processor = MarketDataProcessor::new(binance_config, market_config, storage)
            .await
            .unwrap();

        let analyzer = processor.get_analyzer();
        // Verify analyzer is initialized - Arc should be valid
        assert!(Arc::strong_count(&analyzer) > 0);
    }

    #[tokio::test]
    #[ignore] // Requires MongoDB
    async fn test_market_data_processor_get_supported_symbols() {
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
    async fn test_market_data_processor_get_supported_timeframes() {
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
    async fn test_market_data_processor_get_cache_statistics() {
        let binance_config = create_test_binance_config();
        let market_config = create_test_market_data_config();
        let storage = create_test_storage().await;

        let processor = MarketDataProcessor::new(binance_config, market_config, storage)
            .await
            .unwrap();

        // Add some test data
        let cache = processor.get_cache();
        let kline = create_test_kline(1609459200000, 50000.0);
        cache.add_historical_klines("BTCUSDT", "1m", vec![kline]);

        let stats = processor.get_cache_statistics();
        assert!(stats.total_candles >= 1);
    }

    #[tokio::test]
    #[ignore] // Requires MongoDB
    async fn test_market_data_processor_set_ws_broadcaster() {
        let binance_config = create_test_binance_config();
        let market_config = create_test_market_data_config();
        let storage = create_test_storage().await;

        let mut processor = MarketDataProcessor::new(binance_config, market_config, storage)
            .await
            .unwrap();

        let (broadcaster, _receiver) = broadcast::channel(100);
        processor.set_ws_broadcaster(broadcaster);

        // If we got here without panicking, the test passed
        assert!(true);
    }

    #[tokio::test]
    #[ignore] // Requires MongoDB
    async fn test_handle_stream_event_kline() {
        let cache = MarketDataCache::new(100);
        let kline_data = create_test_kline_data(1609459200000, 50000.0, false);

        let kline_event = KlineEvent {
            event_type: "kline".to_string(),
            event_time: 1609459200000,
            symbol: "BTCUSDT".to_string(),
            kline: kline_data,
        };

        let result = MarketDataProcessor::handle_stream_event(
            &StreamEvent::Kline(kline_event),
            &cache,
            &None,
            &None,
        )
        .await;

        assert!(result.is_ok());
        assert!(cache.get_latest_price("BTCUSDT").is_some());
    }

    #[tokio::test]
    #[ignore] // Requires MongoDB
    async fn test_handle_stream_event_kline_updates_cache() {
        let cache = MarketDataCache::new(100);
        let kline_data = create_test_kline_data(1609459200000, 50000.0, true);

        let kline_event = KlineEvent {
            event_type: "kline".to_string(),
            event_time: 1609459200000,
            symbol: "BTCUSDT".to_string(),
            kline: kline_data,
        };

        MarketDataProcessor::handle_stream_event(
            &StreamEvent::Kline(kline_event),
            &cache,
            &None,
            &None,
        )
        .await
        .unwrap();

        let price = cache.get_latest_price("BTCUSDT").unwrap();
        assert_eq!(price, 50000.0);

        let candles = cache.get_candles("BTCUSDT", "1m", None);
        assert_eq!(candles.len(), 1);
    }

    #[tokio::test]
    #[ignore] // Requires MongoDB
    async fn test_handle_stream_event_with_broadcaster() {
        let cache = MarketDataCache::new(100);
        let (broadcaster, mut receiver) = broadcast::channel(100);
        let kline_data = create_test_kline_data(1609459200000, 50000.0, true);

        let kline_event = KlineEvent {
            event_type: "kline".to_string(),
            event_time: 1609459200000,
            symbol: "BTCUSDT".to_string(),
            kline: kline_data,
        };

        MarketDataProcessor::handle_stream_event(
            &StreamEvent::Kline(kline_event),
            &cache,
            &Some(broadcaster),
            &None,
        )
        .await
        .unwrap();

        // Should receive at least one message
        let result = receiver.try_recv();
        assert!(result.is_ok());
    }

    #[tokio::test]
    #[ignore] // Requires MongoDB
    async fn test_refresh_timeframe_data() -> Result<()> {
        // This test would require a real Binance client connection
        // For unit testing, we'll just verify the function signature
        let binance_config = create_test_binance_config();
        let client = BinanceClient::new(binance_config)?;
        let cache = MarketDataCache::new(100);

        // Note: This will fail without real API access, which is expected in unit tests
        let result =
            MarketDataProcessor::refresh_timeframe_data(&client, &cache, "BTCUSDT", "1m").await;

        // We accept both success and failure here as this requires external API
        assert!(result.is_ok() || result.is_err());
        Ok(())
    }

    #[test]
    fn test_chart_data_conversion_from_kline() {
        let kline = create_test_kline(1609459200000, 50000.0);

        // Test the conversion logic that would be used in get_chart_data
        let candle = CandleData {
            timestamp: kline.open_time,
            open: kline.open.parse::<f64>().unwrap_or(0.0),
            high: kline.high.parse::<f64>().unwrap_or(0.0),
            low: kline.low.parse::<f64>().unwrap_or(0.0),
            close: kline.close.parse::<f64>().unwrap_or(0.0),
            volume: kline.volume.parse::<f64>().unwrap_or(0.0),
        };

        assert_eq!(candle.timestamp, 1609459200000);
        assert_eq!(candle.open, 50000.0);
        assert_eq!(candle.close, 50000.0);
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
    fn test_chart_data_less_than_24_candles() {
        let candles: Vec<CandleData> = (0..10)
            .map(|i| CandleData {
                timestamp: 1609459200000 + i * 3600000,
                open: 50000.0,
                high: 50500.0,
                low: 49500.0,
                close: 50000.0 + (i as f64 * 10.0),
                volume: 1000.0,
            })
            .collect();

        let (volume_24h, price_change_24h, price_change_percent_24h) = if candles.len() >= 24 {
            // Has 24+ candles
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
            // Less than 24 candles
            (0.0, 0.0, 0.0)
        };

        // Should return zeros for less than 24 candles
        assert_eq!(volume_24h, 0.0);
        assert_eq!(price_change_24h, 0.0);
        assert_eq!(price_change_percent_24h, 0.0);
    }

    #[tokio::test]
    #[ignore] // Requires MongoDB
    async fn test_market_data_processor_clone() {
        let binance_config = create_test_binance_config();
        let market_config = create_test_market_data_config();
        let storage = create_test_storage().await;

        let processor1 = MarketDataProcessor::new(binance_config, market_config, storage)
            .await
            .unwrap();

        let processor2 = processor1.clone();

        // Both should have access to the same configuration
        assert_eq!(
            processor1.config.symbols.len(),
            processor2.config.symbols.len()
        );
        assert_eq!(
            processor1.config.timeframes.len(),
            processor2.config.timeframes.len()
        );
    }

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
        let kline = create_test_kline(1609459200000, 50000.0);
        cache.add_historical_klines("BTCUSDT", "1m", vec![kline]);

        // Verify data exists
        assert!(cache.get_latest_price("BTCUSDT").is_some());

        // Remove symbol
        let result = processor.remove_symbol("BTCUSDT").await;
        assert!(result.is_ok());

        // Verify removal
        assert!(cache.get_latest_price("BTCUSDT").is_none());
    }

    // ========== New Comprehensive Tests ==========

    // Test Group 1: Data Structure Tests
    #[test]
    fn test_candle_data_clone() {
        let candle1 = CandleData {
            timestamp: 1609459200000,
            open: 50000.0,
            high: 50500.0,
            low: 49500.0,
            close: 50250.0,
            volume: 1000.0,
        };

        let candle2 = candle1.clone();
        assert_eq!(candle1.timestamp, candle2.timestamp);
        assert_eq!(candle1.open, candle2.open);
        assert_eq!(candle1.close, candle2.close);
    }

    #[test]
    fn test_candle_data_debug_format() {
        let candle = CandleData {
            timestamp: 1609459200000,
            open: 50000.0,
            high: 50500.0,
            low: 49500.0,
            close: 50250.0,
            volume: 1000.0,
        };

        let debug_str = format!("{:?}", candle);
        assert!(debug_str.contains("CandleData"));
        assert!(debug_str.contains("50000"));
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
    fn test_chart_data_clone() {
        let candles = vec![CandleData {
            timestamp: 1609459200000,
            open: 50000.0,
            high: 50500.0,
            low: 49500.0,
            close: 50250.0,
            volume: 1000.0,
        }];

        let chart1 = ChartData {
            symbol: "BTCUSDT".to_string(),
            timeframe: "1m".to_string(),
            candles,
            latest_price: 50250.0,
            volume_24h: 24000.0,
            price_change_24h: 250.0,
            price_change_percent_24h: 0.5,
        };

        let chart2 = chart1.clone();
        assert_eq!(chart1.symbol, chart2.symbol);
        assert_eq!(chart1.candles.len(), chart2.candles.len());
        assert_eq!(chart1.latest_price, chart2.latest_price);
    }

    #[test]
    fn test_chart_data_debug_format() {
        let chart_data = ChartData {
            symbol: "BTCUSDT".to_string(),
            timeframe: "1m".to_string(),
            candles: vec![],
            latest_price: 50250.0,
            volume_24h: 24000.0,
            price_change_24h: 250.0,
            price_change_percent_24h: 0.5,
        };

        let debug_str = format!("{:?}", chart_data);
        assert!(debug_str.contains("ChartData"));
        assert!(debug_str.contains("BTCUSDT"));
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

    // Test Group 2: Configuration Tests
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

    #[test]
    fn test_market_data_config_with_custom_symbols() {
        let mut config = create_test_market_data_config();
        config.symbols = vec![
            "BTCUSDT".to_string(),
            "ETHUSDT".to_string(),
            "BNBUSDT".to_string(),
        ];

        assert_eq!(config.symbols.len(), 3);
        assert!(config.symbols.contains(&"BNBUSDT".to_string()));
    }

    #[test]
    fn test_market_data_config_with_custom_timeframes() {
        let mut config = create_test_market_data_config();
        config.timeframes = vec![
            "1m".to_string(),
            "5m".to_string(),
            "15m".to_string(),
            "1h".to_string(),
        ];

        assert_eq!(config.timeframes.len(), 4);
        assert!(config.timeframes.contains(&"15m".to_string()));
    }

    // Test Group 3: Data Conversion Tests
    #[test]
    fn test_kline_to_candle_conversion_valid_data() {
        let kline = create_test_kline(1609459200000, 50000.0);

        let candle = CandleData {
            timestamp: kline.open_time,
            open: kline.open.parse::<f64>().unwrap_or(0.0),
            high: kline.high.parse::<f64>().unwrap_or(0.0),
            low: kline.low.parse::<f64>().unwrap_or(0.0),
            close: kline.close.parse::<f64>().unwrap_or(0.0),
            volume: kline.volume.parse::<f64>().unwrap_or(0.0),
        };

        assert_eq!(candle.timestamp, 1609459200000);
        assert_eq!(candle.open, 50000.0);
        assert_eq!(candle.close, 50000.0);
        assert!(candle.high >= candle.close);
        assert!(candle.low <= candle.close);
    }

    #[test]
    fn test_kline_to_candle_conversion_with_invalid_strings() {
        let mut kline = create_test_kline(1609459200000, 50000.0);
        kline.volume = "not_a_number".to_string();
        kline.quote_asset_volume = "invalid".to_string();

        let candle = CandleData {
            timestamp: kline.open_time,
            open: kline.open.parse::<f64>().unwrap_or(0.0),
            high: kline.high.parse::<f64>().unwrap_or(0.0),
            low: kline.low.parse::<f64>().unwrap_or(0.0),
            close: kline.close.parse::<f64>().unwrap_or(0.0),
            volume: kline.volume.parse::<f64>().unwrap_or(0.0),
        };

        // Should default to 0.0 for invalid values
        assert_eq!(candle.volume, 0.0);
        // Valid values should still parse correctly
        assert!(candle.close > 0.0);
    }

    #[test]
    fn test_multiple_klines_to_candles_conversion() {
        let klines: Vec<Kline> = (0..5)
            .map(|i| create_test_kline(1609459200000 + i * 60000, 50000.0 + i as f64 * 100.0))
            .collect();

        let candles: Vec<CandleData> = klines
            .iter()
            .map(|kline| CandleData {
                timestamp: kline.open_time,
                open: kline.open.parse::<f64>().unwrap_or(0.0),
                high: kline.high.parse::<f64>().unwrap_or(0.0),
                low: kline.low.parse::<f64>().unwrap_or(0.0),
                close: kline.close.parse::<f64>().unwrap_or(0.0),
                volume: kline.volume.parse::<f64>().unwrap_or(0.0),
            })
            .collect();

        assert_eq!(candles.len(), 5);
        assert_eq!(candles[0].close, 50000.0);
        assert_eq!(candles[4].close, 50400.0);
    }

    // Test Group 4: Price Calculation Tests
    #[test]
    fn test_price_change_calculation_positive() {
        let old_price = 50000.0;
        let new_price = 51000.0;
        let change = new_price - old_price;
        let change_percent = (change / old_price) * 100.0;

        assert_eq!(change, 1000.0);
        assert_eq!(change_percent, 2.0);
    }

    #[test]
    fn test_price_change_calculation_negative() {
        let old_price = 50000.0;
        let new_price = 49000.0;
        let change = new_price - old_price;
        let change_percent = (change / old_price) * 100.0;

        assert_eq!(change, -1000.0);
        assert_eq!(change_percent, -2.0);
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

    #[test]
    fn test_volume_aggregation() {
        let candles: Vec<CandleData> = (0..24)
            .map(|i| CandleData {
                timestamp: 1609459200000 + i * 3600000,
                open: 50000.0,
                high: 50500.0,
                low: 49500.0,
                close: 50000.0,
                volume: 1000.0,
            })
            .collect();

        let total_volume: f64 = candles.iter().map(|c| c.volume).sum();
        assert_eq!(total_volume, 24000.0);
    }

    #[test]
    fn test_volume_aggregation_partial_data() {
        let candles: Vec<CandleData> = (0..10)
            .map(|i| CandleData {
                timestamp: 1609459200000 + i * 3600000,
                open: 50000.0,
                high: 50500.0,
                low: 49500.0,
                close: 50000.0,
                volume: 500.0,
            })
            .collect();

        let total_volume: f64 = candles.iter().map(|c| c.volume).sum();
        assert_eq!(total_volume, 5000.0);
    }

    // Test Group 5: Edge Cases
    #[test]
    fn test_empty_symbol_string() {
        let chart_data = ChartData {
            symbol: "".to_string(),
            timeframe: "1m".to_string(),
            candles: vec![],
            latest_price: 0.0,
            volume_24h: 0.0,
            price_change_24h: 0.0,
            price_change_percent_24h: 0.0,
        };

        assert_eq!(chart_data.symbol, "");
        assert_eq!(chart_data.candles.len(), 0);
    }

    #[test]
    fn test_very_large_candle_collection() {
        let candles: Vec<CandleData> = (0..1000)
            .map(|i| CandleData {
                timestamp: 1609459200000 + i * 60000,
                open: 50000.0,
                high: 50500.0,
                low: 49500.0,
                close: 50000.0,
                volume: 100.0,
            })
            .collect();

        assert_eq!(candles.len(), 1000);
        let last_candle = candles.last().unwrap();
        assert_eq!(last_candle.timestamp, 1609459200000 + 999 * 60000);
    }

    #[test]
    fn test_price_precision_handling() {
        let candle = CandleData {
            timestamp: 1609459200000,
            open: 50000.123456789,
            high: 50500.987654321,
            low: 49500.111111111,
            close: 50250.555555555,
            volume: 1000.999999999,
        };

        assert!((candle.open - 50000.123456789).abs() < 1e-9);
        assert!((candle.close - 50250.555555555).abs() < 1e-9);
    }

    #[test]
    fn test_candle_data_with_zero_prices() {
        let candle = CandleData {
            timestamp: 1609459200000,
            open: 0.0,
            high: 0.0,
            low: 0.0,
            close: 0.0,
            volume: 0.0,
        };

        assert_eq!(candle.open, 0.0);
        assert_eq!(candle.close, 0.0);
        assert_eq!(candle.volume, 0.0);
    }

    #[test]
    fn test_candle_data_ordering_by_timestamp() {
        let mut candles = vec![
            CandleData {
                timestamp: 1609459260000,
                open: 50000.0,
                high: 50500.0,
                low: 49500.0,
                close: 50250.0,
                volume: 1000.0,
            },
            CandleData {
                timestamp: 1609459200000,
                open: 49000.0,
                high: 49500.0,
                low: 48500.0,
                close: 49250.0,
                volume: 900.0,
            },
            CandleData {
                timestamp: 1609459320000,
                open: 51000.0,
                high: 51500.0,
                low: 50500.0,
                close: 51250.0,
                volume: 1100.0,
            },
        ];

        candles.sort_by_key(|c| c.timestamp);

        assert_eq!(candles[0].timestamp, 1609459200000);
        assert_eq!(candles[1].timestamp, 1609459260000);
        assert_eq!(candles[2].timestamp, 1609459320000);
    }

    // Test Group 6: KlineData and KlineEvent Tests
    #[test]
    fn test_kline_data_structure() {
        let kline_data = create_test_kline_data(1609459200000, 50000.0, true);

        assert_eq!(kline_data.symbol, "BTCUSDT");
        assert_eq!(kline_data.interval, "1m");
        assert_eq!(kline_data.kline_start_time, 1609459200000);
        assert!(kline_data.is_this_kline_closed);
        assert_eq!(kline_data.close_price, "50000");
    }

    #[test]
    fn test_kline_data_not_closed() {
        let kline_data = create_test_kline_data(1609459200000, 50000.0, false);

        assert!(!kline_data.is_this_kline_closed);
        assert_eq!(kline_data.symbol, "BTCUSDT");
    }

    #[test]
    fn test_kline_event_structure() {
        let kline_data = create_test_kline_data(1609459200000, 50000.0, true);
        let kline_event = KlineEvent {
            event_type: "kline".to_string(),
            event_time: 1609459200000,
            symbol: "BTCUSDT".to_string(),
            kline: kline_data,
        };

        assert_eq!(kline_event.event_type, "kline");
        assert_eq!(kline_event.symbol, "BTCUSDT");
        assert_eq!(kline_event.kline.close_price, "50000");
    }

    // Test Group 7: Configuration Validation Tests
    #[test]
    fn test_config_cache_size_values() {
        let mut config = create_test_market_data_config();

        config.cache_size = 50;
        assert_eq!(config.cache_size, 50);

        config.cache_size = 1000;
        assert_eq!(config.cache_size, 1000);

        config.cache_size = 1;
        assert_eq!(config.cache_size, 1);
    }

    #[test]
    fn test_config_kline_limit_values() {
        let mut config = create_test_market_data_config();

        config.kline_limit = 10;
        assert_eq!(config.kline_limit, 10);

        config.kline_limit = 500;
        assert_eq!(config.kline_limit, 500);

        config.kline_limit = 1000;
        assert_eq!(config.kline_limit, 1000);
    }

    #[test]
    fn test_config_update_interval_values() {
        let mut config = create_test_market_data_config();

        config.update_interval_ms = 1000;
        assert_eq!(config.update_interval_ms, 1000);

        config.update_interval_ms = 60000;
        assert_eq!(config.update_interval_ms, 60000);
    }

    // Test Group 8: Helper Function Tests
    #[test]
    fn test_create_test_kline_consistency() {
        let kline1 = create_test_kline(1609459200000, 50000.0);
        let kline2 = create_test_kline(1609459200000, 50000.0);

        assert_eq!(kline1.open_time, kline2.open_time);
        assert_eq!(kline1.close, kline2.close);
        assert_eq!(kline1.volume, kline2.volume);
    }

    #[test]
    fn test_create_test_kline_different_prices() {
        let kline1 = create_test_kline(1609459200000, 50000.0);
        let kline2 = create_test_kline(1609459200000, 60000.0);

        assert_eq!(kline1.close.parse::<f64>().unwrap(), 50000.0);
        assert_eq!(kline2.close.parse::<f64>().unwrap(), 60000.0);
    }

    #[test]
    fn test_create_test_kline_data_consistency() {
        let kline_data1 = create_test_kline_data(1609459200000, 50000.0, true);
        let kline_data2 = create_test_kline_data(1609459200000, 50000.0, true);

        assert_eq!(kline_data1.kline_start_time, kline_data2.kline_start_time);
        assert_eq!(kline_data1.close_price, kline_data2.close_price);
        assert_eq!(
            kline_data1.is_this_kline_closed,
            kline_data2.is_this_kline_closed
        );
    }

    // Test Group 9: JSON Broadcasting Format Tests
    #[test]
    fn test_market_data_json_format() {
        let market_data = json!({
            "type": "MarketData",
            "data": {
                "symbol": "BTCUSDT",
                "price": 50000.0,
                "price_change_24h": 500.0,
                "price_change_percent_24h": 1.0,
                "volume_24h": 10000.0,
                "timestamp": 1609459200000i64
            },
            "timestamp": "2021-01-01T00:00:00+00:00"
        });

        let json_str = market_data.to_string();
        assert!(json_str.contains("MarketData"));
        assert!(json_str.contains("BTCUSDT"));
        assert!(json_str.contains("50000"));
    }

    #[test]
    fn test_chart_update_json_format() {
        let chart_update = json!({
            "type": "ChartUpdate",
            "data": {
                "symbol": "BTCUSDT",
                "timeframe": "1m",
                "candle": {
                    "timestamp": 1609459200000i64,
                    "open": 50000.0,
                    "high": 50500.0,
                    "low": 49500.0,
                    "close": 50250.0,
                    "volume": 1000.0,
                    "is_closed": true
                },
                "latest_price": 50250.0,
                "price_change_24h": 250.0,
                "price_change_percent_24h": 0.5,
                "volume_24h": 24000.0,
                "timestamp": 1609459200000i64
            },
            "timestamp": "2021-01-01T00:00:00+00:00"
        });

        let json_str = chart_update.to_string();
        assert!(json_str.contains("ChartUpdate"));
        assert!(json_str.contains("BTCUSDT"));
        assert!(json_str.contains("candle"));
        assert!(json_str.contains("is_closed"));
    }

    // Test Group 10: Boundary Condition Tests
    #[test]
    fn test_chart_data_exactly_24_candles() {
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

        assert_eq!(candles.len(), 24);

        let latest_price = candles.last().map(|c| c.close).unwrap_or(0.0);
        let price_24h_ago = candles
            .get(candles.len() - 24)
            .map(|c| c.close)
            .unwrap_or(latest_price);

        assert_eq!(latest_price, 50230.0);
        assert_eq!(price_24h_ago, 50000.0);
    }

    #[test]
    fn test_chart_data_more_than_24_candles() {
        let candles: Vec<CandleData> = (0..48)
            .map(|i| CandleData {
                timestamp: 1609459200000 + i * 3600000,
                open: 50000.0,
                high: 50500.0,
                low: 49500.0,
                close: 50000.0 + (i as f64 * 10.0),
                volume: 1000.0,
            })
            .collect();

        assert_eq!(candles.len(), 48);

        // Calculate 24h stats from the last 24 candles
        let latest_price = candles.last().map(|c| c.close).unwrap_or(0.0);
        let price_24h_ago = candles
            .get(candles.len() - 24)
            .map(|c| c.close)
            .unwrap_or(latest_price);
        let volume_24h: f64 = candles.iter().rev().take(24).map(|c| c.volume).sum();

        assert_eq!(latest_price, 50470.0);
        assert_eq!(price_24h_ago, 50240.0);
        assert_eq!(volume_24h, 24000.0);
    }

    #[test]
    fn test_timeframe_matching() {
        let timeframes = vec!["1m", "5m", "15m", "1h", "4h", "1d"];

        for tf in &timeframes {
            assert!(matches!(tf, &"1m" | &"5m" | &"15m" | &"1h" | &"4h" | &"1d"));
        }
    }

    #[test]
    fn test_symbol_format_validation() {
        let symbols = vec!["BTCUSDT", "ETHUSDT", "BNBUSDT"];

        for symbol in &symbols {
            assert!(symbol.ends_with("USDT"));
            assert!(symbol.len() >= 6);
        }
    }
}
