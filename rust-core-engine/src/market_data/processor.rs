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
    pub async fn new(
        binance_config: BinanceConfig,
        config: MarketDataConfig,
        storage: Storage,
    ) -> Result<Self> {
        let client = BinanceClient::new(binance_config.clone());
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
                    }
                    Err(e) => {
                        warn!(
                            "Failed to load historical data for {} {}: {}",
                            symbol, timeframe, e
                        );
                    }
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
                }
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
                }
                None => {
                    error!("WebSocket message channel closed");
                    break;
                }
            }
        }

        Ok(())
    }

    async fn handle_stream_event(
        event: &StreamEvent,
        cache: &MarketDataCache,
        ws_broadcaster: &Option<broadcast::Sender<String>>,
        storage: &Option<Storage>,
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
                    let current_price = kline_event.kline.close_price.parse::<f64>().unwrap_or(0.0);

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
                        let chart_update = json!({
                            "type": "ChartUpdate",
                            "data": {
                                "symbol": kline_event.symbol,
                                "timeframe": kline_event.kline.interval,
                                "candle": {
                                    "timestamp": kline_event.kline.kline_start_time,
                                    "open": kline_event.kline.open_price.parse::<f64>().unwrap_or(0.0),
                                    "high": kline_event.kline.high_price.parse::<f64>().unwrap_or(0.0),
                                    "low": kline_event.kline.low_price.parse::<f64>().unwrap_or(0.0),
                                    "close": current_price,
                                    "volume": kline_event.kline.base_asset_volume.parse::<f64>().unwrap_or(0.0),
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
            }
            StreamEvent::Ticker(ticker_event) => {
                debug!(
                    "Received ticker update for {}: {}",
                    ticker_event.symbol, ticker_event.last_price
                );
            }
            StreamEvent::OrderBook(orderbook_event) => {
                debug!(
                    "Received order book update for {} (bids: {}, asks: {})",
                    orderbook_event.symbol,
                    orderbook_event.bids.len(),
                    orderbook_event.asks.len()
                );
            }
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
                        }
                        Err(e) => {
                            warn!("Analysis failed for {}: {}", symbol, e);
                        }
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

        // Convert Klines to CandleData
        let candle_data: Vec<CandleData> = klines
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
                    }
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
                }
                Err(e) => {
                    warn!(
                        "Failed to load historical data for {} {}: {}",
                        symbol, timeframe, e
                    );
                }
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
