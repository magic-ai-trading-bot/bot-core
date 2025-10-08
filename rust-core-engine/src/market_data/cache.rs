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
        let symbol_upper = symbol.to_uppercase();
        format!("{symbol_upper}:{timeframe}")
    }

    pub fn update_kline(&self, symbol: &str, timeframe: &str, kline_data: &KlineData) {
        let key = Self::get_key(symbol, timeframe);
        let candle = CandleData::from(kline_data);

        // ALWAYS update price cache with latest close price for real-time updates
        self.price_cache.insert(symbol.to_uppercase(), candle.close);

        // For shorter timeframes (1m, 5m), update more frequently for real-time feel
        let should_log_update = matches!(timeframe, "1m" | "5m") || kline_data.is_this_kline_closed;

        if should_log_update {
            debug!(
                "Price update for {}: {} (closed: {})",
                symbol, candle.close, kline_data.is_this_kline_closed
            );
        }

        let timeframe_data = self.data.entry(key.clone()).or_insert_with(|| {
            Arc::new(RwLock::new(TimeframeData::new(
                symbol.to_uppercase(),
                timeframe.to_string(),
                self.max_candles_per_timeframe,
            )))
        });

        let mut data = timeframe_data.write();
        data.add_candle(candle);

        debug!(
            "Updated {} {} candle data, total candles: {}",
            symbol,
            timeframe,
            data.len()
        );
    }

    pub fn add_historical_klines(&self, symbol: &str, timeframe: &str, klines: Vec<Kline>) {
        let key = Self::get_key(symbol, timeframe);
        let candles: Vec<CandleData> = klines.iter().map(CandleData::from).collect();

        // Update price cache with latest candle
        if let Some(latest_candle) = candles.last() {
            self.price_cache
                .insert(symbol.to_uppercase(), latest_candle.close);
        }

        let timeframe_data = self.data.entry(key.clone()).or_insert_with(|| {
            Arc::new(RwLock::new(TimeframeData::new(
                symbol.to_uppercase(),
                timeframe.to_string(),
                self.max_candles_per_timeframe,
            )))
        });

        let mut data = timeframe_data.write();
        data.add_historical_candles(candles);

        info!(
            "Added {} historical candles for {} {}, total: {}",
            klines.len(),
            symbol,
            timeframe,
            data.len()
        );
    }

    pub fn get_latest_price(&self, symbol: &str) -> Option<f64> {
        self.price_cache
            .get(&symbol.to_uppercase())
            .map(|entry| *entry.value())
    }

    pub fn get_latest_candle(&self, symbol: &str, timeframe: &str) -> Option<CandleData> {
        let key = Self::get_key(symbol, timeframe);
        self.data.get(&key)?.read().get_latest_candle().cloned()
    }

    pub fn get_candles(
        &self,
        symbol: &str,
        timeframe: &str,
        limit: Option<usize>,
    ) -> Vec<CandleData> {
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
        let keys_to_remove: Vec<String> = self
            .data
            .iter()
            .filter_map(|entry| {
                let key = entry.key();
                if key.starts_with(&format!("{symbol_upper}:")) {
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::binance::types::{Kline, KlineData};

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

    #[test]
    fn test_candle_data_from_kline() {
        let kline = create_test_kline(1609459200000, 50000.0);
        let candle = CandleData::from(&kline);

        assert_eq!(candle.open_time, 1609459200000);
        assert_eq!(candle.close, 50000.0);
        assert_eq!(candle.volume, 1000.0);
        assert!(candle.is_closed);
    }

    #[test]
    fn test_candle_data_from_kline_data() {
        let kline_data = create_test_kline_data(1609459200000, 50000.0, true);
        let candle = CandleData::from(&kline_data);

        assert_eq!(candle.open_time, 1609459200000);
        assert_eq!(candle.close, 50000.0);
        assert_eq!(candle.volume, 1000.0);
        assert!(candle.is_closed);
    }

    #[test]
    fn test_candle_data_from_invalid_strings() {
        let mut kline = create_test_kline(1609459200000, 50000.0);
        kline.close = "invalid".to_string();
        kline.volume = "not_a_number".to_string();

        let candle = CandleData::from(&kline);

        // Should default to 0.0 for invalid values
        assert_eq!(candle.close, 0.0);
        assert_eq!(candle.volume, 0.0);
    }

    #[test]
    fn test_timeframe_data_new() {
        let tf_data = TimeframeData::new("BTCUSDT".to_string(), "1m".to_string(), 100);

        assert_eq!(tf_data.symbol, "BTCUSDT");
        assert_eq!(tf_data.timeframe, "1m");
        assert_eq!(tf_data.max_size, 100);
        assert!(tf_data.is_empty());
        assert_eq!(tf_data.len(), 0);
    }

    #[test]
    fn test_timeframe_data_add_candle() {
        let mut tf_data = TimeframeData::new("BTCUSDT".to_string(), "1m".to_string(), 3);
        let kline = create_test_kline(1609459200000, 50000.0);
        let candle = CandleData::from(&kline);

        tf_data.add_candle(candle);

        assert_eq!(tf_data.len(), 1);
        assert!(!tf_data.is_empty());
        assert_eq!(tf_data.get_latest_candle().unwrap().close, 50000.0);
    }

    #[test]
    fn test_timeframe_data_update_same_candle() {
        let mut tf_data = TimeframeData::new("BTCUSDT".to_string(), "1m".to_string(), 100);
        let open_time = 1609459200000;

        // Add initial candle
        let kline1 = create_test_kline(open_time, 50000.0);
        tf_data.add_candle(CandleData::from(&kline1));
        assert_eq!(tf_data.len(), 1);

        // Update same candle (same open_time)
        let kline2 = create_test_kline(open_time, 50100.0);
        tf_data.add_candle(CandleData::from(&kline2));

        // Should still have 1 candle, but updated
        assert_eq!(tf_data.len(), 1);
        assert_eq!(tf_data.get_latest_candle().unwrap().close, 50100.0);
    }

    #[test]
    fn test_timeframe_data_max_size_limit() {
        let mut tf_data = TimeframeData::new("BTCUSDT".to_string(), "1m".to_string(), 3);

        // Add 5 candles to a max_size of 3
        for i in 0..5 {
            let kline = create_test_kline(1609459200000 + i * 60000, 50000.0 + i as f64);
            tf_data.add_candle(CandleData::from(&kline));
        }

        // Should only keep last 3 candles
        assert_eq!(tf_data.len(), 3);
        assert_eq!(tf_data.get_latest_candle().unwrap().close, 50004.0);
    }

    #[test]
    fn test_timeframe_data_get_candles() {
        let mut tf_data = TimeframeData::new("BTCUSDT".to_string(), "1m".to_string(), 100);

        // Add 5 candles
        for i in 0..5 {
            let kline = create_test_kline(1609459200000 + i * 60000, 50000.0 + i as f64);
            tf_data.add_candle(CandleData::from(&kline));
        }

        // Get last 3 candles
        let candles = tf_data.get_candles(Some(3));
        assert_eq!(candles.len(), 3);
        // Should be in reverse order (latest first)
        assert_eq!(candles[0].close, 50004.0);
        assert_eq!(candles[1].close, 50003.0);
        assert_eq!(candles[2].close, 50002.0);
    }

    #[test]
    fn test_timeframe_data_get_all_candles() {
        let mut tf_data = TimeframeData::new("BTCUSDT".to_string(), "1m".to_string(), 100);

        // Add 3 candles
        for i in 0..3 {
            let kline = create_test_kline(1609459200000 + i * 60000, 50000.0 + i as f64);
            tf_data.add_candle(CandleData::from(&kline));
        }

        let all_candles = tf_data.get_all_candles();
        assert_eq!(all_candles.len(), 3);
        // Should be in order from oldest to newest
        assert_eq!(all_candles[0].close, 50000.0);
        assert_eq!(all_candles[2].close, 50002.0);
    }

    #[test]
    fn test_timeframe_data_add_historical_candles() {
        let mut tf_data = TimeframeData::new("BTCUSDT".to_string(), "1m".to_string(), 10);

        let candles: Vec<CandleData> = (0..5)
            .map(|i| {
                let kline = create_test_kline(1609459200000 + i * 60000, 50000.0 + i as f64);
                CandleData::from(&kline)
            })
            .collect();

        tf_data.add_historical_candles(candles);

        assert_eq!(tf_data.len(), 5);
        assert_eq!(tf_data.get_latest_candle().unwrap().close, 50004.0);
    }

    #[test]
    fn test_timeframe_data_add_historical_candles_exceeds_max() {
        let mut tf_data = TimeframeData::new("BTCUSDT".to_string(), "1m".to_string(), 3);

        let candles: Vec<CandleData> = (0..10)
            .map(|i| {
                let kline = create_test_kline(1609459200000 + i * 60000, 50000.0 + i as f64);
                CandleData::from(&kline)
            })
            .collect();

        tf_data.add_historical_candles(candles);

        // Should only keep last 3
        assert_eq!(tf_data.len(), 3);
        assert_eq!(tf_data.get_latest_candle().unwrap().close, 50009.0);
    }

    #[test]
    fn test_market_data_cache_new() {
        let cache = MarketDataCache::new(100);
        assert_eq!(cache.max_candles_per_timeframe, 100);
        assert!(cache.get_supported_symbols().is_empty());
    }

    #[test]
    fn test_market_data_cache_get_key() {
        let key1 = MarketDataCache::get_key("btcusdt", "1m");
        let key2 = MarketDataCache::get_key("BTCUSDT", "1m");

        // Should be case-insensitive for symbol
        assert_eq!(key1, key2);
        assert_eq!(key1, "BTCUSDT:1m");
    }

    #[test]
    fn test_market_data_cache_update_kline() {
        let cache = MarketDataCache::new(100);
        let kline_data = create_test_kline_data(1609459200000, 50000.0, false);

        cache.update_kline("BTCUSDT", "1m", &kline_data);

        // Check price cache
        let price = cache.get_latest_price("BTCUSDT").unwrap();
        assert_eq!(price, 50000.0);

        // Check candles
        let candles = cache.get_candles("BTCUSDT", "1m", None);
        assert_eq!(candles.len(), 1);
    }

    #[test]
    fn test_market_data_cache_update_kline_case_insensitive() {
        let cache = MarketDataCache::new(100);
        let kline_data = create_test_kline_data(1609459200000, 50000.0, true);

        cache.update_kline("btcusdt", "1m", &kline_data);

        // Should work with uppercase
        let price = cache.get_latest_price("BTCUSDT").unwrap();
        assert_eq!(price, 50000.0);

        // Should work with lowercase
        let price_lower = cache.get_latest_price("btcusdt").unwrap();
        assert_eq!(price_lower, 50000.0);
    }

    #[test]
    fn test_market_data_cache_add_historical_klines() {
        let cache = MarketDataCache::new(100);
        let klines: Vec<Kline> = (0..5)
            .map(|i| create_test_kline(1609459200000 + i * 60000, 50000.0 + i as f64))
            .collect();

        cache.add_historical_klines("BTCUSDT", "1m", klines);

        let candles = cache.get_all_candles("BTCUSDT", "1m");
        assert_eq!(candles.len(), 5);

        let latest_price = cache.get_latest_price("BTCUSDT").unwrap();
        assert_eq!(latest_price, 50004.0);
    }

    #[test]
    fn test_market_data_cache_get_latest_candle() {
        let cache = MarketDataCache::new(100);

        // No candles yet
        assert!(cache.get_latest_candle("BTCUSDT", "1m").is_none());

        // Add candles
        let klines: Vec<Kline> = (0..3)
            .map(|i| create_test_kline(1609459200000 + i * 60000, 50000.0 + i as f64))
            .collect();
        cache.add_historical_klines("BTCUSDT", "1m", klines);

        let latest = cache.get_latest_candle("BTCUSDT", "1m").unwrap();
        assert_eq!(latest.close, 50002.0);
    }

    #[test]
    fn test_market_data_cache_get_candles_with_limit() {
        let cache = MarketDataCache::new(100);
        let klines: Vec<Kline> = (0..10)
            .map(|i| create_test_kline(1609459200000 + i * 60000, 50000.0 + i as f64))
            .collect();

        cache.add_historical_klines("BTCUSDT", "1m", klines);

        let candles = cache.get_candles("BTCUSDT", "1m", Some(5));
        assert_eq!(candles.len(), 5);

        // Should return most recent 5 candles
        assert_eq!(candles[0].close, 50009.0);
    }

    #[test]
    fn test_market_data_cache_get_candles_empty() {
        let cache = MarketDataCache::new(100);

        let candles = cache.get_candles("NONEXISTENT", "1m", None);
        assert!(candles.is_empty());
    }

    #[test]
    fn test_market_data_cache_get_supported_symbols() {
        let cache = MarketDataCache::new(100);

        // Add data for multiple symbols
        let kline = create_test_kline(1609459200000, 50000.0);
        cache.add_historical_klines("BTCUSDT", "1m", vec![kline.clone()]);
        cache.add_historical_klines("ETHUSDT", "1m", vec![kline.clone()]);
        cache.add_historical_klines("BTCUSDT", "5m", vec![kline]);

        let symbols = cache.get_supported_symbols();
        assert_eq!(symbols.len(), 2);
        assert!(symbols.contains(&"BTCUSDT".to_string()));
        assert!(symbols.contains(&"ETHUSDT".to_string()));
    }

    #[test]
    fn test_market_data_cache_get_timeframes_for_symbol() {
        let cache = MarketDataCache::new(100);
        let kline = create_test_kline(1609459200000, 50000.0);

        cache.add_historical_klines("BTCUSDT", "1m", vec![kline.clone()]);
        cache.add_historical_klines("BTCUSDT", "5m", vec![kline.clone()]);
        cache.add_historical_klines("BTCUSDT", "1h", vec![kline]);

        let timeframes = cache.get_timeframes_for_symbol("BTCUSDT");
        assert_eq!(timeframes.len(), 3);
        assert!(timeframes.contains(&"1m".to_string()));
        assert!(timeframes.contains(&"5m".to_string()));
        assert!(timeframes.contains(&"1h".to_string()));
    }

    #[test]
    fn test_market_data_cache_get_cache_stats() {
        let cache = MarketDataCache::new(100);
        let kline = create_test_kline(1609459200000, 50000.0);

        cache.add_historical_klines("BTCUSDT", "1m", vec![kline.clone(), kline.clone()]);
        cache.add_historical_klines("BTCUSDT", "5m", vec![kline.clone()]);
        cache.add_historical_klines("ETHUSDT", "1m", vec![kline.clone(), kline.clone(), kline]);

        let stats = cache.get_cache_stats();

        assert_eq!(stats.cached_symbols, 2);
        assert_eq!(stats.total_timeframes, 3);
        assert_eq!(stats.total_candles, 6);
        assert_eq!(stats.timeframe_counts.get("1m"), Some(&5));
        assert_eq!(stats.timeframe_counts.get("5m"), Some(&1));
    }

    #[test]
    fn test_market_data_cache_remove_symbol() {
        let cache = MarketDataCache::new(100);
        let kline = create_test_kline(1609459200000, 50000.0);

        cache.add_historical_klines("BTCUSDT", "1m", vec![kline.clone()]);
        cache.add_historical_klines("BTCUSDT", "5m", vec![kline.clone()]);
        cache.add_historical_klines("ETHUSDT", "1m", vec![kline]);

        // Verify data exists
        assert!(cache.get_latest_price("BTCUSDT").is_some());
        assert_eq!(cache.get_supported_symbols().len(), 2);

        // Remove BTCUSDT
        cache.remove_symbol("BTCUSDT");

        // Verify removal
        assert!(cache.get_latest_price("BTCUSDT").is_none());
        assert!(cache.get_candles("BTCUSDT", "1m", None).is_empty());
        assert!(cache.get_candles("BTCUSDT", "5m", None).is_empty());

        // ETHUSDT should still exist
        assert!(cache.get_latest_price("ETHUSDT").is_some());
        let symbols = cache.get_supported_symbols();
        assert_eq!(symbols.len(), 1);
        assert!(symbols.contains(&"ETHUSDT".to_string()));
    }

    #[test]
    fn test_market_data_cache_remove_symbol_case_insensitive() {
        let cache = MarketDataCache::new(100);
        let kline = create_test_kline(1609459200000, 50000.0);

        cache.add_historical_klines("BTCUSDT", "1m", vec![kline]);

        // Remove with lowercase
        cache.remove_symbol("btcusdt");

        // Should be removed
        assert!(cache.get_latest_price("BTCUSDT").is_none());
        assert!(cache.get_latest_price("btcusdt").is_none());
    }

    #[test]
    fn test_market_data_cache_clone() {
        let cache1 = MarketDataCache::new(100);
        let kline = create_test_kline(1609459200000, 50000.0);
        cache1.add_historical_klines("BTCUSDT", "1m", vec![kline]);

        // Clone the cache
        let cache2 = cache1.clone();

        // Both should have access to the same data (Arc cloning)
        assert_eq!(
            cache1.get_latest_price("BTCUSDT"),
            cache2.get_latest_price("BTCUSDT")
        );
        assert_eq!(cache1.get_latest_price("BTCUSDT"), Some(50000.0));
    }

    #[test]
    fn test_edge_case_empty_candles() {
        let cache = MarketDataCache::new(100);

        // Add empty klines
        cache.add_historical_klines("BTCUSDT", "1m", vec![]);

        // Should handle gracefully
        assert!(cache.get_latest_candle("BTCUSDT", "1m").is_none());
        assert!(cache.get_latest_price("BTCUSDT").is_none());
    }

    #[test]
    fn test_edge_case_extreme_values() {
        let cache = MarketDataCache::new(100);

        // Test with very large and very small values
        let mut kline = create_test_kline(1609459200000, f64::MAX / 2.0);
        kline.volume = "0.0000001".to_string();

        cache.add_historical_klines("BTCUSDT", "1m", vec![kline]);

        let price = cache.get_latest_price("BTCUSDT").unwrap();
        assert!(price > 0.0);
        assert!(price.is_finite());
    }

    #[test]
    fn test_edge_case_zero_max_size() {
        let cache = MarketDataCache::new(0);
        let kline = create_test_kline(1609459200000, 50000.0);

        cache.add_historical_klines("BTCUSDT", "1m", vec![kline]);

        // Should not store any candles
        let candles = cache.get_candles("BTCUSDT", "1m", None);
        assert_eq!(candles.len(), 0);
    }
}
