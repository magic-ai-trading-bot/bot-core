use crate::market_data::cache::CandleData;
use crate::strategies::indicators::calculate_ema;
use serde::{Deserialize, Serialize};
use std::fmt;

/// Trend direction on a specific timeframe
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TrendDirection {
    Uptrend,
    Downtrend,
    Neutral,
}

impl fmt::Display for TrendDirection {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            TrendDirection::Uptrend => write!(f, "Uptrend"),
            TrendDirection::Downtrend => write!(f, "Downtrend"),
            TrendDirection::Neutral => write!(f, "Neutral"),
        }
    }
}

/// Multi-timeframe trend alignment analysis
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrendAlignment {
    pub daily: TrendDirection,
    pub four_hour: TrendDirection,
    pub one_hour: TrendDirection,
    pub alignment_score: f64, // 0.0 - 1.0
    pub is_aligned: bool,
}

impl TrendAlignment {
    /// Check if trends are aligned for LONG positions
    pub fn is_long_aligned(&self) -> bool {
        matches!(
            (self.daily, self.four_hour),
            (TrendDirection::Uptrend, TrendDirection::Uptrend)
                | (TrendDirection::Uptrend, TrendDirection::Neutral)
                | (TrendDirection::Neutral, TrendDirection::Uptrend)
        )
    }

    /// Check if trends are aligned for SHORT positions
    pub fn is_short_aligned(&self) -> bool {
        matches!(
            (self.daily, self.four_hour),
            (TrendDirection::Downtrend, TrendDirection::Downtrend)
                | (TrendDirection::Downtrend, TrendDirection::Neutral)
                | (TrendDirection::Neutral, TrendDirection::Downtrend)
        )
    }
}

/// Configuration for trend filtering
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrendFilterConfig {
    pub ema_period: usize,
    pub trend_threshold: f64, // % distance from EMA to confirm trend
    pub min_alignment_score: f64,
    pub require_daily_alignment: bool,
    pub require_4h_alignment: bool,
}

impl Default for TrendFilterConfig {
    fn default() -> Self {
        Self {
            ema_period: 200,
            trend_threshold: 0.01, // 1%
            min_alignment_score: 0.6,
            require_daily_alignment: true,
            require_4h_alignment: true,
        }
    }
}

/// Multi-timeframe trend filter
pub struct TrendFilter {
    config: TrendFilterConfig,
}

impl TrendFilter {
    pub fn new(config: TrendFilterConfig) -> Self {
        Self { config }
    }

    /// Calculate trend direction using EMA
    pub fn calculate_ema_trend(
        &self,
        candles: &[CandleData],
    ) -> Result<TrendDirection, String> {
        if candles.len() < self.config.ema_period {
            return Err(format!(
                "Insufficient candles: need {} got {}",
                self.config.ema_period,
                candles.len()
            ));
        }

        let closes: Vec<f64> = candles.iter().map(|c| c.close).collect();
        let ema = calculate_ema(&closes, self.config.ema_period)
            .map_err(|e| format!("EMA calculation error: {}", e))?;

        let current_price = candles.last().unwrap().close;
        let ema_value = *ema.last().unwrap();

        // Calculate % distance from EMA
        let distance = (current_price - ema_value) / ema_value;

        if distance > self.config.trend_threshold {
            Ok(TrendDirection::Uptrend)
        } else if distance < -self.config.trend_threshold {
            Ok(TrendDirection::Downtrend)
        } else {
            Ok(TrendDirection::Neutral)
        }
    }

    /// Check multi-timeframe trend alignment
    pub fn check_alignment(
        &self,
        candles_1d: Option<&[CandleData]>,
        candles_4h: &[CandleData],
        candles_1h: &[CandleData],
    ) -> Result<TrendAlignment, String> {
        // Calculate trend for each timeframe
        let daily = if let Some(candles) = candles_1d {
            self.calculate_ema_trend(candles)?
        } else {
            TrendDirection::Neutral
        };

        let four_hour = self.calculate_ema_trend(candles_4h)?;
        let one_hour = self.calculate_ema_trend(candles_1h)?;

        // Calculate alignment score
        let alignment_score = self.calculate_alignment_score(daily, four_hour, one_hour);
        let is_aligned = alignment_score >= self.config.min_alignment_score;

        Ok(TrendAlignment {
            daily,
            four_hour,
            one_hour,
            alignment_score,
            is_aligned,
        })
    }

    /// Calculate alignment score (0.0 - 1.0)
    fn calculate_alignment_score(
        &self,
        daily: TrendDirection,
        four_hour: TrendDirection,
        one_hour: TrendDirection,
    ) -> f64 {
        match (daily, four_hour, one_hour) {
            // Perfect alignment
            (TrendDirection::Uptrend, TrendDirection::Uptrend, TrendDirection::Uptrend)
            | (TrendDirection::Downtrend, TrendDirection::Downtrend, TrendDirection::Downtrend) => {
                1.0
            }

            // Strong alignment (Daily + 4H)
            (TrendDirection::Uptrend, TrendDirection::Uptrend, _)
            | (TrendDirection::Downtrend, TrendDirection::Downtrend, _) => 0.85,

            // Moderate alignment (Daily + 1H or 4H + 1H)
            (TrendDirection::Uptrend, _, TrendDirection::Uptrend)
            | (TrendDirection::Downtrend, _, TrendDirection::Downtrend)
            | (_, TrendDirection::Uptrend, TrendDirection::Uptrend)
            | (_, TrendDirection::Downtrend, TrendDirection::Downtrend) => 0.65,

            // Weak alignment (only 1 timeframe aligned)
            (TrendDirection::Uptrend, TrendDirection::Neutral, TrendDirection::Neutral)
            | (TrendDirection::Downtrend, TrendDirection::Neutral, TrendDirection::Neutral)
            | (TrendDirection::Neutral, TrendDirection::Uptrend, TrendDirection::Neutral)
            | (TrendDirection::Neutral, TrendDirection::Downtrend, TrendDirection::Neutral)
            | (TrendDirection::Neutral, TrendDirection::Neutral, TrendDirection::Uptrend)
            | (TrendDirection::Neutral, TrendDirection::Neutral, TrendDirection::Downtrend) => 0.4,

            // Mixed/conflicting signals
            _ => 0.2,
        }
    }

    /// Get configuration
    pub fn config(&self) -> &TrendFilterConfig {
        &self.config
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_candles(prices: Vec<f64>) -> Vec<CandleData> {
        prices
            .iter()
            .enumerate()
            .map(|(i, &price)| CandleData {
                open: price,
                high: price * 1.01,
                low: price * 0.99,
                close: price,
                volume: 1000.0,
                open_time: (i as i64) * 3600000,
                close_time: (i as i64) * 3600000 + 3600000,
                quote_volume: 1000.0 * price,
                trades: 100,
                is_closed: true,
            })
            .collect()
    }

    #[test]
    fn test_calculate_ema_trend_uptrend() {
        let filter = TrendFilter::new(TrendFilterConfig {
            ema_period: 20,
            ..Default::default()
        });

        // Create uptrend: prices increasing above EMA
        let prices: Vec<f64> = (0..25).map(|i| 100.0 + (i as f64 * 2.0)).collect();
        let candles = create_test_candles(prices);

        let trend = filter.calculate_ema_trend(&candles).unwrap();
        assert_eq!(trend, TrendDirection::Uptrend);
    }

    #[test]
    fn test_calculate_ema_trend_downtrend() {
        let filter = TrendFilter::new(TrendFilterConfig {
            ema_period: 20,
            ..Default::default()
        });

        // Create downtrend: prices decreasing below EMA
        let prices: Vec<f64> = (0..25).map(|i| 200.0 - (i as f64 * 2.0)).collect();
        let candles = create_test_candles(prices);

        let trend = filter.calculate_ema_trend(&candles).unwrap();
        assert_eq!(trend, TrendDirection::Downtrend);
    }

    #[test]
    fn test_calculate_ema_trend_neutral() {
        let filter = TrendFilter::new(TrendFilterConfig {
            ema_period: 20,
            ..Default::default()
        });

        // Create sideways: prices around EMA
        let prices: Vec<f64> = (0..25).map(|i| 100.0 + ((i % 3) as f64 - 1.0)).collect();
        let candles = create_test_candles(prices);

        let trend = filter.calculate_ema_trend(&candles).unwrap();
        assert_eq!(trend, TrendDirection::Neutral);
    }

    #[test]
    fn test_check_alignment_perfect() {
        let filter = TrendFilter::new(TrendFilterConfig {
            ema_period: 20,
            ..Default::default()
        });

        // All timeframes in uptrend
        let prices_1d: Vec<f64> = (0..25).map(|i| 100.0 + (i as f64 * 2.0)).collect();
        let prices_4h: Vec<f64> = (0..25).map(|i| 100.0 + (i as f64 * 1.5)).collect();
        let prices_1h: Vec<f64> = (0..25).map(|i| 100.0 + (i as f64 * 1.0)).collect();

        let candles_1d = create_test_candles(prices_1d);
        let candles_4h = create_test_candles(prices_4h);
        let candles_1h = create_test_candles(prices_1h);

        let alignment = filter
            .check_alignment(Some(&candles_1d), &candles_4h, &candles_1h)
            .unwrap();

        assert_eq!(alignment.daily, TrendDirection::Uptrend);
        assert_eq!(alignment.four_hour, TrendDirection::Uptrend);
        assert_eq!(alignment.one_hour, TrendDirection::Uptrend);
        assert_eq!(alignment.alignment_score, 1.0);
        assert!(alignment.is_aligned);
        assert!(alignment.is_long_aligned());
    }

    #[test]
    fn test_check_alignment_conflicting() {
        let filter = TrendFilter::new(TrendFilterConfig {
            ema_period: 20,
            ..Default::default()
        });

        // Daily uptrend, 4H downtrend, 1H neutral
        let prices_1d: Vec<f64> = (0..25).map(|i| 100.0 + (i as f64 * 2.0)).collect();
        let prices_4h: Vec<f64> = (0..25).map(|i| 200.0 - (i as f64 * 2.0)).collect();
        let prices_1h: Vec<f64> = (0..25).map(|i| 100.0 + ((i % 3) as f64 - 1.0)).collect();

        let candles_1d = create_test_candles(prices_1d);
        let candles_4h = create_test_candles(prices_4h);
        let candles_1h = create_test_candles(prices_1h);

        let alignment = filter
            .check_alignment(Some(&candles_1d), &candles_4h, &candles_1h)
            .unwrap();

        assert_eq!(alignment.alignment_score, 0.2);
        assert!(!alignment.is_aligned);
    }

    #[test]
    fn test_is_long_aligned() {
        let alignment = TrendAlignment {
            daily: TrendDirection::Uptrend,
            four_hour: TrendDirection::Uptrend,
            one_hour: TrendDirection::Neutral,
            alignment_score: 0.85,
            is_aligned: true,
        };

        assert!(alignment.is_long_aligned());
        assert!(!alignment.is_short_aligned());
    }

    #[test]
    fn test_is_short_aligned() {
        let alignment = TrendAlignment {
            daily: TrendDirection::Downtrend,
            four_hour: TrendDirection::Downtrend,
            one_hour: TrendDirection::Neutral,
            alignment_score: 0.85,
            is_aligned: true,
        };

        assert!(!alignment.is_long_aligned());
        assert!(alignment.is_short_aligned());
    }

    #[test]
    fn test_calculate_ema_trend_insufficient_data() {
        let filter = TrendFilter::new(TrendFilterConfig {
            ema_period: 200,
            ..Default::default()
        });

        let prices: Vec<f64> = (0..50).map(|i| 100.0 + (i as f64)).collect();
        let candles = create_test_candles(prices);

        let result = filter.calculate_ema_trend(&candles);
        assert!(result.is_err());
    }

    #[test]
    fn test_alignment_score_calculation() {
        let filter = TrendFilter::new(TrendFilterConfig::default());

        // Test perfect alignment
        let score = filter.calculate_alignment_score(
            TrendDirection::Uptrend,
            TrendDirection::Uptrend,
            TrendDirection::Uptrend,
        );
        assert_eq!(score, 1.0);

        // Test strong alignment (Daily + 4H)
        let score = filter.calculate_alignment_score(
            TrendDirection::Uptrend,
            TrendDirection::Uptrend,
            TrendDirection::Neutral,
        );
        assert_eq!(score, 0.85);

        // Test moderate alignment
        let score = filter.calculate_alignment_score(
            TrendDirection::Uptrend,
            TrendDirection::Neutral,
            TrendDirection::Uptrend,
        );
        assert_eq!(score, 0.65);

        // Test weak alignment
        let score = filter.calculate_alignment_score(
            TrendDirection::Uptrend,
            TrendDirection::Neutral,
            TrendDirection::Neutral,
        );
        assert_eq!(score, 0.4);

        // Test conflicting
        let score = filter.calculate_alignment_score(
            TrendDirection::Uptrend,
            TrendDirection::Downtrend,
            TrendDirection::Neutral,
        );
        assert_eq!(score, 0.2);
    }
}
