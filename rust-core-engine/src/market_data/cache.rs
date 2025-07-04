use chrono::{DateTime, Utc};
use dashmap::DashMap;
use parking_lot::RwLock;
use std::collections::{BTreeMap, VecDeque};
use std::sync::Arc;
use tracing::{debug, info};

use crate::binance::types::{Kline, KlineData};

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct CandleData {
    pub open_time: i64,
    pub close_time: i64,
    pub open: f64,
    pub high: f64,
    pub low: f64,
    pub close: f64,
    pub volume: f64,
    pub quote_volume: f64,
    pub trades: i64,
    pub is_closed: bool,
}

impl From<&Kline> for CandleData {
    fn from(kline: &Kline) -> Self {
        CandleData {
            open_time: kline.open_time,
            close_time: kline.close_time,
            open: kline.open.parse().unwrap_or(0.0),
            high: kline.high.parse().unwrap_or(0.0),
            low: kline.low.parse().unwrap_or(0.0),
            close: kline.close.parse().unwrap_or(0.0),
            volume: kline.volume.parse().unwrap_or(0.0),
            quote_volume: kline.quote_asset_volume.parse().unwrap_or(0.0),
            trades: kline.number_of_trades,
            is_closed: true,
        }
    }
}

impl From<&KlineData> for CandleData {
    fn from(kline: &KlineData) -> Self {
        CandleData {
            open_time: kline.kline_start_time,
            close_time: kline.kline_close_time,
            open: kline.open_price.parse().unwrap_or(0.0),
            high: kline.high_price.parse().unwrap_or(0.0),
            low: kline.low_price.parse().unwrap_or(0.0),
            close: kline.close_price.parse().unwrap_or(0.0),
            volume: kline.base_asset_volume.parse().unwrap_or(0.0),
            quote_volume: kline.quote_asset_volume.parse().unwrap_or(0.0),
            trades: kline.number_of_trades,
            is_closed: kline.is_this_kline_closed,
        }
    }
}

#[derive(Debug, Clone)]
pub struct TimeframeData {
    pub symbol: String,
    pub timeframe: String,
    pub candles: VecDeque<CandleData>,
    pub max_size: usize,
    pub last_update: DateTime<Utc>,
}

impl TimeframeData {
    pub fn new(symbol: String, timeframe: String, max_size: usize) -> Self {
        Self {
            symbol,
            timeframe,
            candles: VecDeque::with_capacity(max_size),
            max_size,
            last_update: Utc::now(),
        }
    }

    pub fn add_candle(&mut self, candle: CandleData) {
        // Check if this candle updates an existing one (same open time)
        if let Some(last_candle) = self.candles.back_mut() {
            if last_candle.open_time == candle.open_time {
                // Update existing candle
                *last_candle = candle;
                self.last_update = Utc::now();
                return;
            }
        }

        // Add new candle
        self.candles.push_back(candle);
        
        // Remove old candles if we exceed max size
        while self.candles.len() > self.max_size {
            self.candles.pop_front();
        }
        
        self.last_update = Utc::now();
    }

    pub fn add_historical_candles(&mut self, candles: Vec<CandleData>) {
        for candle in candles {
            self.candles.push_back(candle);
        }
        
        // Remove excess candles from the front
        while self.candles.len() > self.max_size {
            self.candles.pop_front();
        }
        
        self.last_update = Utc::now();
    }

    pub fn get_latest_candle(&self) -> Option<&CandleData> {
        self.candles.back()
    }

    pub fn get_candles(&self, limit: Option<usize>) -> Vec<&CandleData> {
        let limit = limit.unwrap_or(self.candles.len());
        self.candles.iter().rev().take(limit).collect()
    }

    pub fn get_all_candles(&self) -> Vec<&CandleData> {
        self.candles.iter().collect()
    }

    pub fn len(&self) -> usize {
        self.candles.len()
    }

    pub fn is_empty(&self) -> bool {
        self.candles.is_empty()
    }
}

pub struct MarketDataCache {
    // Key: "symbol:timeframe" -> TimeframeData
    data: Arc<DashMap<String, Arc<RwLock<TimeframeData>>>>,
    // Price cache for quick access
    price_cache: Arc<DashMap<String, f64>>,
    max_candles_per_timeframe: usize,
}

impl Clone for MarketDataCache {
    fn clone(&self) -> Self {
        Self {
            data: self.data.clone(),
            price_cache: self.price_cache.clone(),
            max_candles_per_timeframe: self.max_candles_per_timeframe,
        }
    }
}

impl MarketDataCache {
    pub fn new(max_candles_per_timeframe: usize) -> Self {
        Self {
            data: Arc::new(DashMap::new()),
            price_cache: Arc::new(DashMap::new()),
            max_candles_per_timeframe,
        }
    }

    fn get_key(symbol: &str, timeframe: &str) -> String {
        format!("{}:{}", symbol.to_uppercase(), timeframe)
    }

    pub fn update_kline(&self, symbol: &str, timeframe: &str, kline_data: &KlineData) {
        let key = Self::get_key(symbol, timeframe);
        let candle = CandleData::from(kline_data);
        
        // ALWAYS update price cache with latest close price for real-time updates
        self.price_cache.insert(symbol.to_uppercase(), candle.close);
        
        // For shorter timeframes (1m, 5m), update more frequently for real-time feel
        let should_log_update = matches!(timeframe, "1m" | "5m") || kline_data.is_this_kline_closed;
        
        if should_log_update {
            debug!("Price update for {}: {} (closed: {})", 
                   symbol, candle.close, kline_data.is_this_kline_closed);
        }

        let timeframe_data = self.data
            .entry(key.clone())
            .or_insert_with(|| {
                Arc::new(RwLock::new(TimeframeData::new(
                    symbol.to_uppercase(),
                    timeframe.to_string(),
                    self.max_candles_per_timeframe,
                )))
            });

        let mut data = timeframe_data.write();
        data.add_candle(candle);
        
        debug!("Updated {} {} candle data, total candles: {}", symbol, timeframe, data.len());
    }

    pub fn add_historical_klines(&self, symbol: &str, timeframe: &str, klines: Vec<Kline>) {
        let key = Self::get_key(symbol, timeframe);
        let candles: Vec<CandleData> = klines.iter().map(CandleData::from).collect();
        
        // Update price cache with latest candle
        if let Some(latest_candle) = candles.last() {
            self.price_cache.insert(symbol.to_uppercase(), latest_candle.close);
        }

        let timeframe_data = self.data
            .entry(key.clone())
            .or_insert_with(|| {
                Arc::new(RwLock::new(TimeframeData::new(
                    symbol.to_uppercase(),
                    timeframe.to_string(),
                    self.max_candles_per_timeframe,
                )))
            });

        let mut data = timeframe_data.write();
        data.add_historical_candles(candles);
        
        info!("Added {} historical candles for {} {}, total: {}", 
              klines.len(), symbol, timeframe, data.len());
    }

    pub fn get_latest_price(&self, symbol: &str) -> Option<f64> {
        self.price_cache.get(&symbol.to_uppercase()).map(|entry| *entry.value())
    }

    pub fn get_latest_candle(&self, symbol: &str, timeframe: &str) -> Option<CandleData> {
        let key = Self::get_key(symbol, timeframe);
        self.data.get(&key)?
            .read()
            .get_latest_candle()
            .cloned()
    }

    pub fn get_candles(&self, symbol: &str, timeframe: &str, limit: Option<usize>) -> Vec<CandleData> {
        let key = Self::get_key(symbol, timeframe);
        if let Some(timeframe_data) = self.data.get(&key) {
            let data = timeframe_data.read();
            data.get_candles(limit).into_iter().cloned().collect()
        } else {
            Vec::new()
        }
    }

    pub fn get_all_candles(&self, symbol: &str, timeframe: &str) -> Vec<CandleData> {
        let key = Self::get_key(symbol, timeframe);
        if let Some(timeframe_data) = self.data.get(&key) {
            let data = timeframe_data.read();
            data.get_all_candles().into_iter().cloned().collect()
        } else {
            Vec::new()
        }
    }

    pub fn get_supported_symbols(&self) -> Vec<String> {
        let mut symbols = std::collections::HashSet::new();
        
        for entry in self.data.iter() {
            let key = entry.key();
            if let Some(symbol) = key.split(':').next() {
                symbols.insert(symbol.to_string());
            }
        }
        
        symbols.into_iter().collect()
    }

    pub fn get_timeframes_for_symbol(&self, symbol: &str) -> Vec<String> {
        let symbol_upper = symbol.to_uppercase();
        let mut timeframes = Vec::new();
        
        for entry in self.data.iter() {
            let key = entry.key();
            let parts: Vec<&str> = key.split(':').collect();
            if parts.len() == 2 && parts[0] == symbol_upper {
                timeframes.push(parts[1].to_string());
            }
        }
        
        timeframes
    }

    pub fn get_cache_stats(&self) -> CacheStats {
        let mut timeframe_counts = BTreeMap::new();
        let mut total_candles = 0;
        let mut symbols = std::collections::HashSet::new();
        
        for entry in self.data.iter() {
            let key = entry.key();
            let parts: Vec<&str> = key.split(':').collect();
            if parts.len() == 2 {
                let symbol = parts[0];
                let timeframe = parts[1];
                
                symbols.insert(symbol.to_string());
                
                let data = entry.value().read();
                let candle_count = data.len();
                total_candles += candle_count;
                
                *timeframe_counts.entry(timeframe.to_string()).or_insert(0) += candle_count;
            }
        }
        
        CacheStats {
            total_timeframes: self.data.len(),
            total_candles,
            timeframe_counts,
            cached_symbols: symbols.len(),
        }
    }

    // NEW: Remove symbol from cache
    pub fn remove_symbol(&self, symbol: &str) {
        let symbol_upper = symbol.to_uppercase();
        
        // Remove from price cache
        self.price_cache.remove(&symbol_upper);
        
        // Remove all timeframe data for this symbol
        let keys_to_remove: Vec<String> = self.data
            .iter()
            .filter_map(|entry| {
                let key = entry.key();
                if key.starts_with(&format!("{}:", symbol_upper)) {
                    Some(key.clone())
                } else {
                    None
                }
            })
            .collect();
        
        for key in keys_to_remove {
            self.data.remove(&key);
        }
        
        info!("Removed symbol {} from cache", symbol);
    }
}

#[derive(Debug, Clone)]
pub struct CacheStats {
    pub total_timeframes: usize,
    pub total_candles: usize,
    pub timeframe_counts: BTreeMap<String, usize>,
    pub cached_symbols: usize,
} 