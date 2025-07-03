use anyhow::Result;
use crossbeam_channel::Receiver;
use std::sync::Arc;
use std::time::Duration;
use tokio::time::{interval, sleep};
use tracing::{debug, error, info, warn};

use crate::binance::{BinanceClient, BinanceWebSocket, StreamEvent};
use crate::config::{BinanceConfig, MarketDataConfig};
use crate::storage::Storage;

use super::cache::MarketDataCache;
use super::analyzer::MarketDataAnalyzer;

#[derive(Clone)]
pub struct MarketDataProcessor {
    binance_config: BinanceConfig,
    config: MarketDataConfig,
    client: BinanceClient,
    cache: MarketDataCache,
    analyzer: Arc<MarketDataAnalyzer>,
    storage: Storage,
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
        })
    }

    pub async fn start(&self) -> Result<()> {
        info!("Starting Market Data Processor");
        
        // Load historical data first
        self.load_historical_data().await?;
        
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
        
        Ok(())
    }

    async fn load_historical_data(&self) -> Result<()> {
        info!("Loading historical market data");
        
        for symbol in &self.config.symbols {
            for timeframe in &self.config.timeframes {
                match self.load_historical_klines(symbol, timeframe).await {
                    Ok(count) => {
                        info!("Loaded {} historical candles for {} {}", count, symbol, timeframe);
                    }
                    Err(e) => {
                        warn!("Failed to load historical data for {} {}: {}", symbol, timeframe, e);
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
        let klines = self.client
            .get_futures_klines(symbol, timeframe, Some(self.config.kline_limit))
            .await?;
        
        let count = klines.len();
        self.cache.add_historical_klines(symbol, timeframe, klines);
        
        Ok(count)
    }

    async fn start_websocket_streams(&self) -> Result<tokio::task::JoinHandle<Result<()>>> {
        let (websocket, receiver) = BinanceWebSocket::new(self.binance_config.clone());
        let symbols = self.config.symbols.clone();
        let timeframes = self.config.timeframes.clone();
        let cache = self.cache.clone();
        
        // Start WebSocket connection
        let ws_handle = tokio::spawn(async move {
            websocket.start(symbols, timeframes).await
        });
        
        // Start message processing
        let processor_handle = tokio::spawn(async move {
            Self::process_websocket_messages(receiver, cache).await
        });
        
        // Return a combined handle
        Ok(tokio::spawn(async move {
            tokio::try_join!(
                async { ws_handle.await? },
                async { processor_handle.await? }
            )?;
            Ok(())
        }))
    }

    async fn process_websocket_messages(
        receiver: Receiver<StreamEvent>,
        cache: MarketDataCache,
    ) -> Result<()> {
        info!("Starting WebSocket message processing");
        
        loop {
            match receiver.recv() {
                Ok(event) => {
                    if let Err(e) = Self::handle_stream_event(&event, &cache).await {
                        error!("Error handling stream event: {}", e);
                    }
                }
                Err(e) => {
                    error!("Error receiving WebSocket message: {}", e);
                    break;
                }
            }
        }
        
        Ok(())
    }

    async fn handle_stream_event(event: &StreamEvent, cache: &MarketDataCache) -> Result<()> {
        match event {
            StreamEvent::Kline(kline_event) => {
                cache.update_kline(
                    &kline_event.symbol,
                    &kline_event.kline.interval,
                    &kline_event.kline,
                );
                
                debug!("Updated kline data for {} {} - Close: {} (closed: {})",
                       kline_event.symbol, 
                       kline_event.kline.interval,
                       kline_event.kline.close_price,
                       kline_event.kline.is_this_kline_closed);
            }
            StreamEvent::Ticker(ticker_event) => {
                debug!("Received ticker update for {}: {}", 
                       ticker_event.symbol, ticker_event.last_price);
            }
            StreamEvent::OrderBook(orderbook_event) => {
                debug!("Received order book update for {} (bids: {}, asks: {})",
                       orderbook_event.symbol,
                       orderbook_event.bids.len(),
                       orderbook_event.asks.len());
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
                            if let Err(e) = Self::refresh_timeframe_data(
                                &client, 
                                &cache, 
                                symbol, 
                                timeframe
                            ).await {
                                warn!("Failed to refresh {} {}: {}", symbol, timeframe, e);
                            }
                        }
                    }
                }
                
                // Log cache statistics
                let stats = cache.get_cache_stats();
                debug!("Cache stats: {} symbols, {} timeframes, {} total candles", 
                       stats.cached_symbols, stats.total_timeframes, stats.total_candles);
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
                            info!("Analysis completed for {}: {:?} (confidence: {:.2})",
                                  symbol, analysis.overall_signal, analysis.overall_confidence);
                            
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

    pub async fn get_latest_analysis(&self, symbol: &str) -> Result<super::analyzer::MultiTimeframeAnalysis> {
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
        self.analyzer.get_market_overview(&self.config.symbols).await
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
} 