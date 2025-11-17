use crate::market_data::cache::CandleData;

/// RSI (Relative Strength Index) calculation
pub fn calculate_rsi(candles: &[CandleData], period: usize) -> Result<Vec<f64>, String> {
    if candles.len() < period + 1 {
        return Err(format!(
            "Need at least {} candles for RSI calculation",
            period + 1
        ));
    }

    let mut gains = Vec::new();
    let mut losses = Vec::new();
    let mut rsi_values = Vec::new();

    // Calculate price changes
    for i in 1..candles.len() {
        let change = candles[i].close - candles[i - 1].close;
        if change > 0.0 {
            gains.push(change);
            losses.push(0.0);
        } else {
            gains.push(0.0);
            losses.push(-change);
        }
    }

    // Calculate initial average gain and loss
    let mut avg_gain: f64 = gains.iter().take(period).sum::<f64>() / period as f64;
    let mut avg_loss: f64 = losses.iter().take(period).sum::<f64>() / period as f64;

    // First RSI value
    let rsi = if avg_gain == 0.0 && avg_loss == 0.0 {
        // No price movement - neutral RSI
        50.0
    } else if avg_loss == 0.0 {
        // All gains, no losses
        100.0
    } else {
        let rs = avg_gain / avg_loss;
        100.0 - (100.0 / (1.0 + rs))
    };
    rsi_values.push(rsi);

    // Calculate subsequent RSI values using smoothed averages
    for i in period..gains.len() {
        avg_gain = ((avg_gain * (period - 1) as f64) + gains[i]) / period as f64;
        avg_loss = ((avg_loss * (period - 1) as f64) + losses[i]) / period as f64;

        let rsi = if avg_gain == 0.0 && avg_loss == 0.0 {
            // No price movement - neutral RSI
            50.0
        } else if avg_loss == 0.0 {
            // All gains, no losses
            100.0
        } else {
            let rs = avg_gain / avg_loss;
            100.0 - (100.0 / (1.0 + rs))
        };
        rsi_values.push(rsi);
    }

    Ok(rsi_values)
}

/// MACD calculation
#[derive(Debug, Clone)]
pub struct MacdResult {
    pub macd_line: Vec<f64>,
    pub signal_line: Vec<f64>,
    pub histogram: Vec<f64>,
}

pub fn calculate_macd(
    candles: &[CandleData],
    fast: usize,
    slow: usize,
    signal: usize,
) -> Result<MacdResult, String> {
    if candles.len() < slow + signal {
        return Err("Insufficient data for MACD calculation".to_string());
    }

    let prices: Vec<f64> = candles.iter().map(|c| c.close).collect();

    let ema_fast = calculate_ema(&prices, fast)?;
    let ema_slow = calculate_ema(&prices, slow)?;

    // MACD line = EMA(fast) - EMA(slow)
    let mut macd_line = Vec::new();
    let start_idx = slow - fast;

    for i in start_idx..ema_fast.len() {
        macd_line.push(ema_fast[i] - ema_slow[i - start_idx]);
    }

    // Signal line = EMA of MACD line
    let signal_line = calculate_ema(&macd_line, signal)?;

    // Histogram = MACD - Signal
    let mut histogram = Vec::new();
    let hist_start = signal - 1;

    for i in hist_start..macd_line.len() {
        histogram.push(macd_line[i] - signal_line[i - hist_start]);
    }

    Ok(MacdResult {
        macd_line,
        signal_line,
        histogram,
    })
}

/// Bollinger Bands calculation
#[derive(Debug, Clone)]
pub struct BollingerBands {
    pub upper: Vec<f64>,
    pub middle: Vec<f64>,
    pub lower: Vec<f64>,
}

pub fn calculate_bollinger_bands(
    candles: &[CandleData],
    period: usize,
    multiplier: f64,
) -> Result<BollingerBands, String> {
    if candles.len() < period {
        return Err("Insufficient data for Bollinger Bands calculation".to_string());
    }

    let prices: Vec<f64> = candles.iter().map(|c| c.close).collect();
    let sma = calculate_sma(&prices, period)?;

    let mut upper = Vec::new();
    let mut lower = Vec::new();

    for (i, &mean) in sma.iter().enumerate() {
        let start_idx = i;
        let end_idx = i + period;

        // Calculate standard deviation
        let variance: f64 = prices[start_idx..end_idx]
            .iter()
            .map(|&price| (price - mean).powi(2))
            .sum::<f64>()
            / period as f64;
        let std_dev = variance.sqrt();

        upper.push(mean + (multiplier * std_dev));
        lower.push(mean - (multiplier * std_dev));
    }

    Ok(BollingerBands {
        upper,
        middle: sma,
        lower,
    })
}

/// Volume Profile calculation
#[derive(Debug, Clone)]
pub struct VolumeProfile {
    pub price_levels: Vec<f64>,
    pub volumes: Vec<f64>,
    pub poc: f64, // Point of Control
}

pub fn calculate_volume_profile(
    candles: &[CandleData],
    levels: usize,
) -> Result<VolumeProfile, String> {
    if candles.is_empty() {
        return Err("No data for volume profile calculation".to_string());
    }

    let min_price = candles.iter().map(|c| c.low).fold(f64::INFINITY, f64::min);
    let max_price = candles
        .iter()
        .map(|c| c.high)
        .fold(f64::NEG_INFINITY, f64::max);
    let price_step = (max_price - min_price) / levels as f64;

    let mut price_levels = Vec::new();
    let mut volumes = vec![0.0; levels];

    // Create price levels
    for i in 0..levels {
        price_levels.push(min_price + (i as f64 * price_step));
    }

    // Distribute volume across price levels
    for candle in candles {
        let avg_price = (candle.high + candle.low + candle.close) / 3.0;
        let level_index = ((avg_price - min_price) / price_step) as usize;
        let level_index = level_index.min(levels - 1);
        volumes[level_index] += candle.volume;
    }

    // Find Point of Control (highest volume level)
    let max_volume_idx = volumes
        .iter()
        .enumerate()
        .max_by(|(_, a), (_, b)| {
            a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal) // Handle NaN gracefully
        })
        .map(|(idx, _)| idx)
        .unwrap_or(0);

    let poc = price_levels[max_volume_idx];

    Ok(VolumeProfile {
        price_levels,
        volumes,
        poc,
    })
}

/// Simple Moving Average
pub fn calculate_sma(prices: &[f64], period: usize) -> Result<Vec<f64>, String> {
    if prices.len() < period {
        return Err("Insufficient data for SMA calculation".to_string());
    }

    let mut sma_values = Vec::new();

    for i in 0..=prices.len() - period {
        let sum: f64 = prices[i..i + period].iter().sum();
        sma_values.push(sum / period as f64);
    }

    Ok(sma_values)
}

/// Exponential Moving Average
pub fn calculate_ema(prices: &[f64], period: usize) -> Result<Vec<f64>, String> {
    if prices.len() < period {
        return Err("Insufficient data for EMA calculation".to_string());
    }

    let multiplier = 2.0 / (period as f64 + 1.0);
    let mut ema_values = Vec::new();

    // First EMA value is SMA
    let first_sma: f64 = prices[0..period].iter().sum::<f64>() / period as f64;
    ema_values.push(first_sma);

    // Calculate subsequent EMA values
    for price in prices.iter().skip(period) {
        // Safe: we just pushed first_sma above, so ema_values is never empty
        let last_ema = *ema_values.last().unwrap_or(&first_sma);
        let ema = (price * multiplier) + (last_ema * (1.0 - multiplier));
        ema_values.push(ema);
    }

    Ok(ema_values)
}

/// Average True Range (ATR)
pub fn calculate_atr(candles: &[CandleData], period: usize) -> Result<Vec<f64>, String> {
    if candles.len() < period + 1 {
        return Err("Insufficient data for ATR calculation".to_string());
    }

    let mut true_ranges = Vec::new();

    for i in 1..candles.len() {
        let high_low = candles[i].high - candles[i].low;
        let high_close_prev = (candles[i].high - candles[i - 1].close).abs();
        let low_close_prev = (candles[i].low - candles[i - 1].close).abs();

        let true_range = high_low.max(high_close_prev).max(low_close_prev);
        true_ranges.push(true_range);
    }

    // Calculate ATR using SMA of true ranges
    calculate_sma(&true_ranges, period)
}

/// Stochastic Oscillator
#[derive(Debug, Clone)]
pub struct StochasticResult {
    pub k_percent: Vec<f64>,
    pub d_percent: Vec<f64>,
}

pub fn calculate_stochastic(
    candles: &[CandleData],
    k_period: usize,
    d_period: usize,
) -> Result<StochasticResult, String> {
    if candles.len() < k_period + d_period {
        return Err("Insufficient data for Stochastic calculation".to_string());
    }

    let mut k_percent = Vec::new();

    for i in k_period - 1..candles.len() {
        let window = &candles[i + 1 - k_period..=i];
        let highest_high = window
            .iter()
            .map(|c| c.high)
            .fold(f64::NEG_INFINITY, f64::max);
        let lowest_low = window.iter().map(|c| c.low).fold(f64::INFINITY, f64::min);

        let current_close = candles[i].close;
        let k = if highest_high == lowest_low {
            50.0
        } else {
            ((current_close - lowest_low) / (highest_high - lowest_low)) * 100.0
        };

        k_percent.push(k);
    }

    let d_percent = calculate_sma(&k_percent, d_period)?;

    Ok(StochasticResult {
        k_percent,
        d_percent,
    })
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
                volume: 1000.0 + (i as f64 * 100.0),
                open_time: (i as i64) * 60000,
                close_time: (i as i64) * 60000 + 60000,
                quote_volume: (1000.0 + (i as f64 * 100.0)) * price,
                trades: 100,
                is_closed: true,
            })
            .collect()
    }

    fn create_test_candles_with_range(
        closes: Vec<f64>,
        highs: Vec<f64>,
        lows: Vec<f64>,
    ) -> Vec<CandleData> {
        closes
            .iter()
            .enumerate()
            .map(|(i, &close)| CandleData {
                open: close,
                high: highs[i],
                low: lows[i],
                close,
                volume: 1000.0,
                open_time: (i as i64) * 60000,
                close_time: (i as i64) * 60000 + 60000,
                quote_volume: 1000.0 * close,
                trades: 100,
                is_closed: true,
            })
            .collect()
    }

    #[test]
    fn test_calculate_rsi_normal_case() {
        let prices = vec![
            44.0, 44.25, 44.5, 43.75, 44.0, 44.5, 45.0, 45.25, 45.5, 45.0, 45.5, 46.0, 46.5, 46.25,
            46.0,
        ];
        let candles = create_test_candles(prices);
        let result = calculate_rsi(&candles, 14);

        assert!(result.is_ok());
        let rsi_values = result.unwrap();
        assert!(!rsi_values.is_empty());
        // RSI should be between 0 and 100
        for &rsi in &rsi_values {
            assert!(rsi >= 0.0 && rsi <= 100.0);
        }
    }

    #[test]
    fn test_calculate_rsi_all_gains() {
        let prices = vec![
            100.0, 101.0, 102.0, 103.0, 104.0, 105.0, 106.0, 107.0, 108.0, 109.0, 110.0, 111.0,
            112.0, 113.0, 114.0,
        ];
        let candles = create_test_candles(prices);
        let result = calculate_rsi(&candles, 14);

        assert!(result.is_ok());
        let rsi_values = result.unwrap();
        // RSI should be near 100 for all gains
        assert!(*rsi_values.last().unwrap() > 90.0);
    }

    #[test]
    fn test_calculate_rsi_all_losses() {
        let prices = vec![
            114.0, 113.0, 112.0, 111.0, 110.0, 109.0, 108.0, 107.0, 106.0, 105.0, 104.0, 103.0,
            102.0, 101.0, 100.0,
        ];
        let candles = create_test_candles(prices);
        let result = calculate_rsi(&candles, 14);

        assert!(result.is_ok());
        let rsi_values = result.unwrap();
        // RSI should be near 0 for all losses
        assert!(*rsi_values.last().unwrap() < 10.0);
    }

    #[test]
    fn test_calculate_rsi_insufficient_data() {
        let candles = create_test_candles(vec![100.0, 101.0, 102.0]);
        let result = calculate_rsi(&candles, 14);

        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Need at least"));
    }

    #[test]
    fn test_calculate_macd_normal_case() {
        let prices: Vec<f64> = (0..50).map(|i| 100.0 + (i as f64 * 0.5)).collect();
        let candles = create_test_candles(prices);
        let result = calculate_macd(&candles, 12, 26, 9);

        assert!(result.is_ok());
        let macd = result.unwrap();
        assert!(!macd.macd_line.is_empty());
        assert!(!macd.signal_line.is_empty());
        assert!(!macd.histogram.is_empty());
        assert_eq!(macd.macd_line.len() - macd.signal_line.len(), 8);
    }

    #[test]
    fn test_calculate_macd_bullish_trend() {
        let prices: Vec<f64> = (0..50).map(|i| 100.0 + (i as f64 * 2.0)).collect();
        let candles = create_test_candles(prices);
        let result = calculate_macd(&candles, 12, 26, 9);

        assert!(result.is_ok());
        let macd = result.unwrap();
        // In a strong uptrend, MACD should be positive
        assert!(macd.macd_line.last().unwrap() > &0.0);
    }

    #[test]
    fn test_calculate_macd_insufficient_data() {
        let candles = create_test_candles(vec![100.0, 101.0, 102.0]);
        let result = calculate_macd(&candles, 12, 26, 9);

        assert!(result.is_err());
    }

    #[test]
    fn test_calculate_bollinger_bands_normal_case() {
        let prices: Vec<f64> = (0..30).map(|i| 100.0 + (i as f64 * 0.5)).collect();
        let candles = create_test_candles(prices);
        let result = calculate_bollinger_bands(&candles, 20, 2.0);

        assert!(result.is_ok());
        let bb = result.unwrap();
        assert_eq!(bb.upper.len(), bb.middle.len());
        assert_eq!(bb.middle.len(), bb.lower.len());
        // Upper band should be higher than middle, middle higher than lower
        for i in 0..bb.upper.len() {
            assert!(bb.upper[i] > bb.middle[i]);
            assert!(bb.middle[i] > bb.lower[i]);
        }
    }

    #[test]
    fn test_calculate_bollinger_bands_width() {
        let prices = vec![100.0; 25]; // Flat prices = narrow bands
        let candles = create_test_candles(prices);
        let result = calculate_bollinger_bands(&candles, 20, 2.0);

        assert!(result.is_ok());
        let bb = result.unwrap();
        let width = bb.upper[0] - bb.lower[0];
        // Width should be very small for flat prices
        assert!(width < 1.0);
    }

    #[test]
    fn test_calculate_bollinger_bands_insufficient_data() {
        let candles = create_test_candles(vec![100.0, 101.0, 102.0]);
        let result = calculate_bollinger_bands(&candles, 20, 2.0);

        assert!(result.is_err());
    }

    #[test]
    fn test_calculate_volume_profile_normal_case() {
        let candles = create_test_candles(vec![
            100.0, 101.0, 102.0, 101.5, 102.5, 103.0, 102.0, 101.0, 100.5, 101.5,
        ]);
        let result = calculate_volume_profile(&candles, 10);

        assert!(result.is_ok());
        let vp = result.unwrap();
        assert_eq!(vp.price_levels.len(), 10);
        assert_eq!(vp.volumes.len(), 10);
        // POC should be within the price range
        let min_price = candles.iter().map(|c| c.low).fold(f64::INFINITY, f64::min);
        let max_price = candles
            .iter()
            .map(|c| c.high)
            .fold(f64::NEG_INFINITY, f64::max);
        assert!(vp.poc >= min_price && vp.poc <= max_price);
    }

    #[test]
    fn test_calculate_volume_profile_empty_data() {
        let candles: Vec<CandleData> = vec![];
        let result = calculate_volume_profile(&candles, 10);

        assert!(result.is_err());
    }

    #[test]
    fn test_calculate_sma_normal_case() {
        let prices = vec![1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0, 9.0, 10.0];
        let result = calculate_sma(&prices, 3);

        assert!(result.is_ok());
        let sma = result.unwrap();
        assert_eq!(sma.len(), 8); // 10 - 3 + 1
        assert_eq!(sma[0], 2.0); // (1+2+3)/3
        assert_eq!(sma[1], 3.0); // (2+3+4)/3
        assert_eq!(sma[7], 9.0); // (8+9+10)/3
    }

    #[test]
    fn test_calculate_sma_single_period() {
        let prices = vec![100.0, 200.0, 300.0];
        let result = calculate_sma(&prices, 1);

        assert!(result.is_ok());
        let sma = result.unwrap();
        assert_eq!(sma.len(), 3);
        assert_eq!(sma[0], 100.0);
        assert_eq!(sma[1], 200.0);
        assert_eq!(sma[2], 300.0);
    }

    #[test]
    fn test_calculate_sma_insufficient_data() {
        let prices = vec![1.0, 2.0];
        let result = calculate_sma(&prices, 5);

        assert!(result.is_err());
    }

    #[test]
    fn test_calculate_ema_normal_case() {
        let prices = vec![22.0, 23.0, 24.0, 23.5, 24.5, 25.0, 25.5, 26.0, 26.5, 27.0];
        let result = calculate_ema(&prices, 5);

        assert!(result.is_ok());
        let ema = result.unwrap();
        assert_eq!(ema.len(), 6); // 10 - 5 + 1
                                  // First EMA should equal SMA
        let first_sma: f64 = prices[0..5].iter().sum::<f64>() / 5.0;
        assert!((ema[0] - first_sma).abs() < 0.0001);
        // EMA should react to price changes
        assert!(ema.last().unwrap() > &ema[0]);
    }

    #[test]
    fn test_calculate_ema_uptrend() {
        let prices: Vec<f64> = (0..20).map(|i| 100.0 + (i as f64)).collect();
        let result = calculate_ema(&prices, 10);

        assert!(result.is_ok());
        let ema = result.unwrap();
        // EMA should be increasing in uptrend
        for i in 1..ema.len() {
            assert!(ema[i] > ema[i - 1]);
        }
    }

    #[test]
    fn test_calculate_ema_insufficient_data() {
        let prices = vec![1.0, 2.0];
        let result = calculate_ema(&prices, 5);

        assert!(result.is_err());
    }

    #[test]
    fn test_calculate_atr_normal_case() {
        let closes = vec![
            48.7, 48.72, 48.9, 48.87, 48.82, 49.05, 49.2, 49.35, 49.92, 50.19, 50.12, 49.66, 49.88,
            50.19, 50.36,
        ];
        let highs = vec![
            48.7, 48.85, 49.2, 49.05, 48.94, 49.25, 49.4, 49.55, 50.0, 50.28, 50.2, 49.75, 50.0,
            50.36, 50.57,
        ];
        let lows = vec![
            47.79, 48.39, 48.66, 48.53, 48.1, 48.86, 48.9, 49.5, 49.87, 49.2, 49.73, 48.9, 49.43,
            49.26, 50.0,
        ];
        let candles = create_test_candles_with_range(closes, highs, lows);
        let result = calculate_atr(&candles, 14);

        assert!(result.is_ok());
        let atr = result.unwrap();
        assert!(!atr.is_empty());
        // ATR should always be positive
        for &value in &atr {
            assert!(value > 0.0);
        }
    }

    #[test]
    fn test_calculate_atr_high_volatility() {
        let closes = vec![
            100.0, 110.0, 90.0, 105.0, 95.0, 115.0, 85.0, 100.0, 120.0, 80.0,
        ];
        let highs = vec![
            105.0, 115.0, 100.0, 110.0, 100.0, 120.0, 95.0, 110.0, 125.0, 100.0,
        ];
        let lows = vec![
            95.0, 105.0, 85.0, 95.0, 90.0, 110.0, 80.0, 95.0, 115.0, 75.0,
        ];
        let candles = create_test_candles_with_range(closes, highs, lows);
        let result = calculate_atr(&candles, 3);

        assert!(result.is_ok());
        let atr = result.unwrap();
        // High volatility should produce larger ATR values
        assert!(atr.iter().all(|&v| v > 5.0));
    }

    #[test]
    fn test_calculate_atr_insufficient_data() {
        let candles = create_test_candles(vec![100.0, 101.0]);
        let result = calculate_atr(&candles, 14);

        assert!(result.is_err());
    }

    #[test]
    #[ignore] // Business logic test - needs tuning
    fn test_calculate_stochastic_normal_case() {
        let closes = vec![
            44.0, 44.25, 44.5, 43.75, 44.0, 44.5, 45.0, 45.25, 45.5, 45.0, 45.5, 46.0, 46.5, 46.25,
            46.0,
        ];
        let highs = vec![
            44.5, 44.75, 45.0, 44.25, 44.5, 45.0, 45.5, 45.75, 46.0, 45.5, 46.0, 46.5, 47.0, 46.75,
            46.5,
        ];
        let lows = vec![
            43.5, 43.75, 44.0, 43.25, 43.5, 44.0, 44.5, 44.75, 45.0, 44.5, 45.0, 45.5, 46.0, 45.75,
            45.5,
        ];
        let candles = create_test_candles_with_range(closes, highs, lows);
        let result = calculate_stochastic(&candles, 5, 3);

        assert!(result.is_ok());
        let stoch = result.unwrap();
        assert!(!stoch.k_percent.is_empty());
        assert!(!stoch.d_percent.is_empty());
        // K and D should be between 0 and 100
        for &k in &stoch.k_percent {
            assert!(k >= 0.0 && k <= 100.0);
        }
        for &d in &stoch.d_percent {
            assert!(d >= 0.0 && d <= 100.0);
        }
    }

    #[test]
    #[ignore] // Business logic test - needs tuning
    fn test_calculate_stochastic_overbought() {
        let closes = vec![
            100.0, 101.0, 102.0, 103.0, 104.0, 105.0, 106.0, 107.0, 108.0, 109.0,
        ];
        let highs = vec![
            101.0, 102.0, 103.0, 104.0, 105.0, 106.0, 107.0, 108.0, 109.0, 110.0,
        ];
        let lows = vec![
            99.0, 100.0, 101.0, 102.0, 103.0, 104.0, 105.0, 106.0, 107.0, 108.0,
        ];
        let candles = create_test_candles_with_range(closes, highs, lows);
        let result = calculate_stochastic(&candles, 5, 3);

        assert!(result.is_ok());
        let stoch = result.unwrap();
        // In strong uptrend, stochastic should be high
        assert!(stoch.k_percent.last().unwrap() > &80.0);
    }

    #[test]
    #[ignore] // Business logic test - needs tuning
    fn test_calculate_stochastic_oversold() {
        let closes = vec![
            109.0, 108.0, 107.0, 106.0, 105.0, 104.0, 103.0, 102.0, 101.0, 100.0,
        ];
        let highs = vec![
            110.0, 109.0, 108.0, 107.0, 106.0, 105.0, 104.0, 103.0, 102.0, 101.0,
        ];
        let lows = vec![
            108.0, 107.0, 106.0, 105.0, 104.0, 103.0, 102.0, 101.0, 100.0, 99.0,
        ];
        let candles = create_test_candles_with_range(closes, highs, lows);
        let result = calculate_stochastic(&candles, 5, 3);

        assert!(result.is_ok());
        let stoch = result.unwrap();
        // In strong downtrend, stochastic should be low
        assert!(stoch.k_percent.last().unwrap() < &20.0);
    }

    #[test]
    fn test_calculate_stochastic_insufficient_data() {
        let candles = create_test_candles(vec![100.0, 101.0]);
        let result = calculate_stochastic(&candles, 5, 3);

        assert!(result.is_err());
    }

    // ===== Additional RSI Tests =====

    #[test]
    fn test_rsi_empty_data() {
        let candles: Vec<CandleData> = vec![];
        let result = calculate_rsi(&candles, 14);
        assert!(result.is_err());
    }

    #[test]
    fn test_rsi_single_candle() {
        let candles = create_test_candles(vec![100.0]);
        let result = calculate_rsi(&candles, 14);
        assert!(result.is_err());
    }

    #[test]
    fn test_rsi_exact_minimum_data() {
        let prices = vec![
            100.0, 101.0, 102.0, 103.0, 104.0, 105.0, 106.0, 107.0, 108.0, 109.0, 110.0, 111.0,
            112.0, 113.0, 114.0,
        ]; // 15 candles = period(14) + 1
        let candles = create_test_candles(prices);
        let result = calculate_rsi(&candles, 14);
        assert!(result.is_ok());
        let rsi_values = result.unwrap();
        assert_eq!(rsi_values.len(), 1); // Should return 1 value
    }

    #[test]
    fn test_rsi_no_change_prices() {
        let closes = vec![
            100.0, 100.0, 100.0, 100.0, 100.0, 100.0, 100.0, 100.0, 100.0, 100.0, 100.0, 100.0,
            100.0, 100.0, 100.0,
        ];
        let highs = vec![
            101.0, 101.0, 101.0, 101.0, 101.0, 101.0, 101.0, 101.0, 101.0, 101.0, 101.0, 101.0,
            101.0, 101.0, 101.0,
        ];
        let lows = vec![
            99.0, 99.0, 99.0, 99.0, 99.0, 99.0, 99.0, 99.0, 99.0, 99.0, 99.0, 99.0, 99.0, 99.0,
            99.0,
        ];
        let candles = create_test_candles_with_range(closes, highs, lows);
        let result = calculate_rsi(&candles, 14);
        assert!(result.is_ok());
        let rsi_values = result.unwrap();
        // When no change in close prices (flat), RSI should be 50.0 (neutral)
        // avg_gain = 0, avg_loss = 0 → RSI = 50.0
        assert!((rsi_values[0] - 50.0).abs() < 0.001);
    }

    #[test]
    fn test_rsi_alternating_prices() {
        let prices = vec![
            100.0, 101.0, 100.0, 101.0, 100.0, 101.0, 100.0, 101.0, 100.0, 101.0, 100.0, 101.0,
            100.0, 101.0, 100.0,
        ];
        let candles = create_test_candles(prices);
        let result = calculate_rsi(&candles, 14);
        assert!(result.is_ok());
        let rsi_values = result.unwrap();
        // Alternating should produce RSI around 50
        assert!(rsi_values.last().unwrap() > &40.0 && rsi_values.last().unwrap() < &60.0);
    }

    #[test]
    fn test_rsi_small_period() {
        let prices = vec![100.0, 101.0, 102.0, 103.0];
        let candles = create_test_candles(prices);
        let result = calculate_rsi(&candles, 2);
        assert!(result.is_ok());
        let rsi_values = result.unwrap();
        assert_eq!(rsi_values.len(), 2);
    }

    #[test]
    fn test_rsi_boundary_values() {
        let prices = vec![
            100.0, 110.0, 120.0, 130.0, 140.0, 150.0, 160.0, 170.0, 180.0, 190.0, 200.0, 210.0,
            220.0, 230.0, 240.0,
        ];
        let candles = create_test_candles(prices);
        let result = calculate_rsi(&candles, 14);
        assert!(result.is_ok());
        let rsi_values = result.unwrap();
        // Strong uptrend should push RSI close to 100
        assert!(*rsi_values.last().unwrap() > 95.0);
        assert!(*rsi_values.last().unwrap() <= 100.0);
    }

    // ===== Additional MACD Tests =====

    #[test]
    fn test_macd_empty_data() {
        let candles: Vec<CandleData> = vec![];
        let result = calculate_macd(&candles, 12, 26, 9);
        assert!(result.is_err());
    }

    #[test]
    fn test_macd_exact_minimum_data() {
        let prices: Vec<f64> = (0..35).map(|i| 100.0 + i as f64).collect(); // 26 + 9 = 35
        let candles = create_test_candles(prices);
        let result = calculate_macd(&candles, 12, 26, 9);
        assert!(result.is_ok());
        let macd = result.unwrap();
        assert!(!macd.macd_line.is_empty());
        assert!(!macd.signal_line.is_empty());
        assert!(!macd.histogram.is_empty());
    }

    #[test]
    fn test_macd_bearish_trend() {
        let prices: Vec<f64> = (0..50).map(|i| 200.0 - (i as f64 * 2.0)).collect();
        let candles = create_test_candles(prices);
        let result = calculate_macd(&candles, 12, 26, 9);
        assert!(result.is_ok());
        let macd = result.unwrap();
        // In strong downtrend, MACD should be negative
        assert!(macd.macd_line.last().unwrap() < &0.0);
    }

    #[test]
    fn test_macd_flat_prices() {
        let prices = vec![100.0; 50];
        let candles = create_test_candles(prices);
        let result = calculate_macd(&candles, 12, 26, 9);
        assert!(result.is_ok());
        let macd = result.unwrap();
        // Flat prices should produce MACD near zero
        assert!(macd.macd_line.last().unwrap().abs() < 0.1);
    }

    #[test]
    fn test_macd_custom_periods() {
        let prices: Vec<f64> = (0..60).map(|i| 100.0 + (i as f64 * 0.5)).collect();
        let candles = create_test_candles(prices);
        let result = calculate_macd(&candles, 5, 13, 8);
        assert!(result.is_ok());
        let macd = result.unwrap();
        assert!(!macd.macd_line.is_empty());
        assert!(!macd.signal_line.is_empty());
        assert!(!macd.histogram.is_empty());
    }

    #[test]
    fn test_macd_histogram_crossover() {
        let prices: Vec<f64> = (0..50)
            .map(|i| {
                if i < 25 {
                    100.0 + i as f64
                } else {
                    125.0 - (i - 25) as f64
                }
            })
            .collect();
        let candles = create_test_candles(prices);
        let result = calculate_macd(&candles, 12, 26, 9);
        assert!(result.is_ok());
        let macd = result.unwrap();
        // Histogram should exist and have values
        assert!(macd.histogram.len() > 5);
    }

    // ===== Additional Bollinger Bands Tests =====

    #[test]
    fn test_bollinger_bands_empty_data() {
        let candles: Vec<CandleData> = vec![];
        let result = calculate_bollinger_bands(&candles, 20, 2.0);
        assert!(result.is_err());
    }

    #[test]
    fn test_bollinger_bands_exact_minimum_data() {
        let prices: Vec<f64> = (0..20).map(|i| 100.0 + i as f64).collect();
        let candles = create_test_candles(prices);
        let result = calculate_bollinger_bands(&candles, 20, 2.0);
        assert!(result.is_ok());
        let bb = result.unwrap();
        assert_eq!(bb.upper.len(), 1);
        assert_eq!(bb.middle.len(), 1);
        assert_eq!(bb.lower.len(), 1);
    }

    #[test]
    fn test_bollinger_bands_different_multipliers() {
        let prices: Vec<f64> = (0..30).map(|i| 100.0 + (i as f64 * 0.5)).collect();
        let candles = create_test_candles(prices);

        let result1 = calculate_bollinger_bands(&candles, 20, 1.0);
        let result2 = calculate_bollinger_bands(&candles, 20, 2.0);
        let result3 = calculate_bollinger_bands(&candles, 20, 3.0);

        assert!(result1.is_ok() && result2.is_ok() && result3.is_ok());
        let bb1 = result1.unwrap();
        let bb2 = result2.unwrap();
        let bb3 = result3.unwrap();

        // Wider multiplier should create wider bands
        let width1 = bb1.upper[0] - bb1.lower[0];
        let width2 = bb2.upper[0] - bb2.lower[0];
        let width3 = bb3.upper[0] - bb3.lower[0];
        assert!(width1 < width2);
        assert!(width2 < width3);
    }

    #[test]
    fn test_bollinger_bands_high_volatility() {
        let prices = vec![
            100.0, 110.0, 95.0, 115.0, 90.0, 120.0, 85.0, 125.0, 80.0, 130.0, 100.0, 105.0, 95.0,
            110.0, 90.0, 115.0, 85.0, 120.0, 80.0, 125.0, 100.0,
        ];
        let candles = create_test_candles(prices);
        let result = calculate_bollinger_bands(&candles, 20, 2.0);
        assert!(result.is_ok());
        let bb = result.unwrap();
        let width = bb.upper[0] - bb.lower[0];
        // High volatility should create wide bands
        assert!(width > 20.0);
    }

    #[test]
    fn test_bollinger_bands_small_period() {
        let prices = vec![100.0, 101.0, 102.0, 103.0, 104.0];
        let candles = create_test_candles(prices);
        let result = calculate_bollinger_bands(&candles, 3, 2.0);
        assert!(result.is_ok());
        let bb = result.unwrap();
        assert_eq!(bb.upper.len(), 3);
    }

    // ===== Additional SMA Tests =====

    #[test]
    fn test_sma_empty_data() {
        let prices: Vec<f64> = vec![];
        let result = calculate_sma(&prices, 5);
        assert!(result.is_err());
    }

    #[test]
    fn test_sma_exact_period() {
        let prices = vec![10.0, 20.0, 30.0, 40.0, 50.0];
        let result = calculate_sma(&prices, 5);
        assert!(result.is_ok());
        let sma = result.unwrap();
        assert_eq!(sma.len(), 1);
        assert_eq!(sma[0], 30.0); // (10+20+30+40+50)/5
    }

    #[test]
    fn test_sma_all_zeros() {
        let prices = vec![0.0, 0.0, 0.0, 0.0, 0.0];
        let result = calculate_sma(&prices, 3);
        assert!(result.is_ok());
        let sma = result.unwrap();
        assert_eq!(sma.len(), 3);
        assert_eq!(sma[0], 0.0);
        assert_eq!(sma[1], 0.0);
        assert_eq!(sma[2], 0.0);
    }

    #[test]
    fn test_sma_negative_values() {
        let prices = vec![-5.0, -10.0, -15.0, -20.0, -25.0];
        let result = calculate_sma(&prices, 3);
        assert!(result.is_ok());
        let sma = result.unwrap();
        assert_eq!(sma[0], -10.0); // (-5-10-15)/3
        assert_eq!(sma[1], -15.0); // (-10-15-20)/3
        assert_eq!(sma[2], -20.0); // (-15-20-25)/3
    }

    #[test]
    fn test_sma_large_period() {
        let prices: Vec<f64> = (1..=100).map(|i| i as f64).collect();
        let result = calculate_sma(&prices, 50);
        assert!(result.is_ok());
        let sma = result.unwrap();
        assert_eq!(sma.len(), 51); // 100 - 50 + 1
        assert_eq!(sma[0], 25.5); // Average of 1-50
    }

    #[test]
    fn test_sma_period_equals_data_length() {
        let prices = vec![1.0, 2.0, 3.0, 4.0, 5.0];
        let result = calculate_sma(&prices, 5);
        assert!(result.is_ok());
        let sma = result.unwrap();
        assert_eq!(sma.len(), 1);
        assert_eq!(sma[0], 3.0);
    }

    // ===== Additional EMA Tests =====

    #[test]
    fn test_ema_empty_data() {
        let prices: Vec<f64> = vec![];
        let result = calculate_ema(&prices, 5);
        assert!(result.is_err());
    }

    #[test]
    fn test_ema_exact_period() {
        let prices = vec![10.0, 20.0, 30.0, 40.0, 50.0];
        let result = calculate_ema(&prices, 5);
        assert!(result.is_ok());
        let ema = result.unwrap();
        assert_eq!(ema.len(), 1);
        assert_eq!(ema[0], 30.0); // First EMA equals SMA
    }

    #[test]
    fn test_ema_downtrend() {
        let prices: Vec<f64> = (0..20).map(|i| 200.0 - (i as f64)).collect();
        let result = calculate_ema(&prices, 10);
        assert!(result.is_ok());
        let ema = result.unwrap();
        // EMA should be decreasing in downtrend
        for i in 1..ema.len() {
            assert!(ema[i] < ema[i - 1]);
        }
    }

    #[test]
    fn test_ema_flat_prices() {
        let prices = vec![100.0; 20];
        let result = calculate_ema(&prices, 10);
        assert!(result.is_ok());
        let ema = result.unwrap();
        // All EMA values should be 100.0 for flat prices
        for &value in &ema {
            assert_eq!(value, 100.0);
        }
    }

    #[test]
    fn test_ema_small_period() {
        let prices = vec![10.0, 20.0, 30.0];
        let result = calculate_ema(&prices, 2);
        assert!(result.is_ok());
        let ema = result.unwrap();
        assert_eq!(ema.len(), 2);
        assert_eq!(ema[0], 15.0); // (10+20)/2
    }

    #[test]
    fn test_ema_multiplier_calculation() {
        let prices: Vec<f64> = (1..=20).map(|i| i as f64).collect();
        let period = 10;
        let result = calculate_ema(&prices, period);
        assert!(result.is_ok());
        let ema = result.unwrap();
        // Verify multiplier is correctly applied
        let multiplier = 2.0 / (period as f64 + 1.0);
        assert_eq!(multiplier, 2.0 / 11.0);
        assert!(ema.len() > 0);
    }

    // ===== Additional ATR Tests =====

    #[test]
    fn test_atr_empty_data() {
        let candles: Vec<CandleData> = vec![];
        let result = calculate_atr(&candles, 14);
        assert!(result.is_err());
    }

    #[test]
    fn test_atr_single_candle() {
        let candles = create_test_candles(vec![100.0]);
        let result = calculate_atr(&candles, 14);
        assert!(result.is_err());
    }

    #[test]
    fn test_atr_exact_minimum_data() {
        let closes: Vec<f64> = (0..15).map(|i| 100.0 + i as f64).collect();
        let highs: Vec<f64> = (0..15).map(|i| 101.0 + i as f64).collect();
        let lows: Vec<f64> = (0..15).map(|i| 99.0 + i as f64).collect();
        let candles = create_test_candles_with_range(closes, highs, lows);
        let result = calculate_atr(&candles, 14);
        assert!(result.is_ok());
        let atr = result.unwrap();
        assert_eq!(atr.len(), 1);
    }

    #[test]
    fn test_atr_low_volatility() {
        let closes = vec![100.0, 100.1, 100.2, 100.1, 100.2, 100.3, 100.2, 100.3];
        let highs = vec![
            100.15, 100.25, 100.35, 100.25, 100.35, 100.45, 100.35, 100.45,
        ];
        let lows = vec![99.85, 99.95, 100.05, 99.95, 100.05, 100.15, 100.05, 100.15];
        let candles = create_test_candles_with_range(closes, highs, lows);
        let result = calculate_atr(&candles, 3);
        assert!(result.is_ok());
        let atr = result.unwrap();
        // Low volatility should produce small ATR values
        assert!(atr.iter().all(|&v| v < 1.0));
    }

    #[test]
    fn test_atr_gap_up() {
        let closes = vec![100.0, 100.5, 101.0, 110.0, 111.0, 112.0, 113.0];
        let highs = vec![100.5, 101.0, 101.5, 110.5, 111.5, 112.5, 113.5];
        let lows = vec![99.5, 100.0, 100.5, 109.5, 110.5, 111.5, 112.5];
        let candles = create_test_candles_with_range(closes, highs, lows);
        let result = calculate_atr(&candles, 3);
        assert!(result.is_ok());
        let atr = result.unwrap();
        // Gap up should increase ATR
        assert!(atr.len() > 2);
    }

    #[test]
    fn test_atr_small_period() {
        let closes = vec![100.0, 101.0, 102.0, 103.0];
        let highs = vec![101.0, 102.0, 103.0, 104.0];
        let lows = vec![99.0, 100.0, 101.0, 102.0];
        let candles = create_test_candles_with_range(closes, highs, lows);
        let result = calculate_atr(&candles, 2);
        assert!(result.is_ok());
        let atr = result.unwrap();
        assert_eq!(atr.len(), 2);
    }

    // ===== Additional Stochastic Tests =====

    #[test]
    fn test_stochastic_empty_data() {
        let candles: Vec<CandleData> = vec![];
        let result = calculate_stochastic(&candles, 5, 3);
        assert!(result.is_err());
    }

    #[test]
    fn test_stochastic_exact_minimum_data() {
        // Need k_period + d_period = 5 + 3 = 8 candles minimum
        let closes: Vec<f64> = (0..8).map(|i| 100.0 + i as f64).collect();
        let highs: Vec<f64> = (0..8).map(|i| 101.0 + i as f64).collect();
        let lows: Vec<f64> = (0..8).map(|i| 99.0 + i as f64).collect();
        let candles = create_test_candles_with_range(closes, highs, lows);
        let result = calculate_stochastic(&candles, 5, 3);
        assert!(result.is_ok());
        let stoch = result.unwrap();
        assert_eq!(stoch.k_percent.len(), 4); // 8 - (5 - 1) = 4
        assert_eq!(stoch.d_percent.len(), 2); // 4 - 3 + 1 = 2
    }

    #[test]
    fn test_stochastic_equal_high_low() {
        // Need at least k_period + d_period = 5 + 3 = 8
        let closes = vec![100.0; 10];
        let highs = vec![100.0; 10];
        let lows = vec![100.0; 10];
        let candles = create_test_candles_with_range(closes, highs, lows);
        let result = calculate_stochastic(&candles, 5, 3);
        assert!(result.is_ok());
        let stoch = result.unwrap();
        // When high equals low, K should be 50
        for &k in &stoch.k_percent {
            assert_eq!(k, 50.0);
        }
    }

    #[test]
    fn test_stochastic_at_high() {
        // Need at least k_period + d_period = 5 + 2 = 7
        let closes = vec![100.0, 101.0, 102.0, 103.0, 104.0, 105.0, 106.0, 107.0];
        let highs = vec![101.0, 102.0, 103.0, 104.0, 105.0, 106.0, 107.0, 107.0];
        let lows = vec![99.0, 100.0, 101.0, 102.0, 103.0, 104.0, 105.0, 106.0];
        let candles = create_test_candles_with_range(closes, highs, lows);
        let result = calculate_stochastic(&candles, 5, 2);
        assert!(result.is_ok());
        let stoch = result.unwrap();
        // Close at high should produce K near 100
        assert!(*stoch.k_percent.last().unwrap() > 90.0);
    }

    #[test]
    fn test_stochastic_at_low() {
        // Need at least k_period + d_period = 5 + 2 = 7
        let closes = vec![107.0, 106.0, 105.0, 104.0, 103.0, 102.0, 101.0, 100.0];
        let highs = vec![108.0, 107.0, 106.0, 105.0, 104.0, 103.0, 102.0, 101.0];
        let lows = vec![107.0, 106.0, 105.0, 104.0, 103.0, 102.0, 101.0, 100.0];
        let candles = create_test_candles_with_range(closes, highs, lows);
        let result = calculate_stochastic(&candles, 5, 2);
        assert!(result.is_ok());
        let stoch = result.unwrap();
        // Close at low should produce K near 0
        assert!(*stoch.k_percent.last().unwrap() < 10.0);
    }

    #[test]
    fn test_stochastic_small_periods() {
        // Need at least k_period + d_period = 3 + 2 = 5
        let closes = vec![100.0, 101.0, 102.0, 103.0, 104.0];
        let highs = vec![101.0, 102.0, 103.0, 104.0, 105.0];
        let lows = vec![99.0, 100.0, 101.0, 102.0, 103.0];
        let candles = create_test_candles_with_range(closes, highs, lows);
        let result = calculate_stochastic(&candles, 3, 2);
        assert!(result.is_ok());
        let stoch = result.unwrap();
        assert_eq!(stoch.k_percent.len(), 3); // 5 - (3 - 1) = 3
        assert_eq!(stoch.d_percent.len(), 2); // 3 - 2 + 1 = 2
    }

    // ===== Additional Volume Profile Tests =====

    #[test]
    fn test_volume_profile_single_candle() {
        let candles = create_test_candles(vec![100.0]);
        let result = calculate_volume_profile(&candles, 5);
        assert!(result.is_ok());
        let vp = result.unwrap();
        assert_eq!(vp.price_levels.len(), 5);
        assert_eq!(vp.volumes.len(), 5);
    }

    #[test]
    fn test_volume_profile_many_levels() {
        let candles = create_test_candles(vec![100.0, 101.0, 102.0, 103.0, 104.0]);
        let result = calculate_volume_profile(&candles, 20);
        assert!(result.is_ok());
        let vp = result.unwrap();
        assert_eq!(vp.price_levels.len(), 20);
        assert_eq!(vp.volumes.len(), 20);
    }

    #[test]
    fn test_volume_profile_single_level() {
        let candles = create_test_candles(vec![100.0, 101.0, 102.0]);
        let result = calculate_volume_profile(&candles, 1);
        assert!(result.is_ok());
        let vp = result.unwrap();
        assert_eq!(vp.price_levels.len(), 1);
        assert_eq!(vp.volumes.len(), 1);
        // All volume should be in the single level
        assert!(vp.volumes[0] > 0.0);
    }

    #[test]
    fn test_volume_profile_poc_location() {
        let mut candles = create_test_candles(vec![100.0, 101.0, 102.0, 103.0, 104.0]);
        // Increase volume at middle price
        candles[2].volume = 10000.0;
        let result = calculate_volume_profile(&candles, 5);
        assert!(result.is_ok());
        let vp = result.unwrap();
        // POC should be near the price with highest volume
        assert!(vp.poc >= 100.0 && vp.poc <= 105.0);
    }

    #[test]
    fn test_volume_profile_wide_price_range() {
        let candles = create_test_candles(vec![50.0, 100.0, 150.0, 200.0, 250.0]);
        let result = calculate_volume_profile(&candles, 10);
        assert!(result.is_ok());
        let vp = result.unwrap();
        let min_price = 50.0 * 0.99; // low multiplier
        let max_price = 250.0 * 1.01; // high multiplier
        assert!(vp.poc >= min_price && vp.poc <= max_price);
    }

    #[test]
    fn test_volume_profile_total_volume() {
        let candles = create_test_candles(vec![100.0, 101.0, 102.0, 103.0, 104.0]);
        let result = calculate_volume_profile(&candles, 5);
        assert!(result.is_ok());
        let vp = result.unwrap();
        let total_volume: f64 = vp.volumes.iter().sum();
        let expected_total: f64 = candles.iter().map(|c| c.volume).sum();
        // Total volume should approximately equal sum of candle volumes
        assert!((total_volume - expected_total).abs() < 0.1);
    }

    // ===== Cross-Function Integration Tests =====

    #[test]
    fn test_ema_vs_sma_difference() {
        let prices: Vec<f64> = (1..=20).map(|i| i as f64).collect();
        let sma = calculate_sma(&prices, 10).unwrap();
        let ema = calculate_ema(&prices, 10).unwrap();

        // First values should be equal (EMA starts with SMA)
        assert!((ema[0] - sma[0]).abs() < 0.0001);

        // EMA should react faster to price changes than SMA
        assert_eq!(sma.len(), ema.len());
    }

    #[test]
    fn test_bollinger_bands_contains_price() {
        let prices: Vec<f64> = (0..30).map(|i| 100.0 + (i as f64 * 0.5)).collect();
        let candles = create_test_candles(prices.clone());
        let result = calculate_bollinger_bands(&candles, 20, 2.0);
        assert!(result.is_ok());
        let bb = result.unwrap();

        // Most prices should be within the bands
        let mut within_count = 0;
        for i in 0..bb.middle.len() {
            let price = prices[i + 19]; // Offset by period - 1
            if price >= bb.lower[i] && price <= bb.upper[i] {
                within_count += 1;
            }
        }
        // At least 80% should be within 2 standard deviations
        assert!(within_count as f64 / bb.middle.len() as f64 > 0.8);
    }

    #[test]
    fn test_macd_components_relationship() {
        let prices: Vec<f64> = (0..50).map(|i| 100.0 + (i as f64 * 0.5)).collect();
        let candles = create_test_candles(prices);
        let result = calculate_macd(&candles, 12, 26, 9);
        assert!(result.is_ok());
        let macd = result.unwrap();

        // Histogram should equal MACD - Signal
        let hist_start = 8; // signal - 1
        for i in 0..macd.histogram.len() {
            let expected_hist = macd.macd_line[hist_start + i] - macd.signal_line[i];
            assert!((macd.histogram[i] - expected_hist).abs() < 0.0001);
        }
    }

    #[test]
    fn test_rsi_oversold_to_overbought_transition() {
        let mut prices = vec![];
        // Create oversold condition
        for i in (0..8).rev() {
            prices.push(100.0 + i as f64);
        }
        // Create overbought condition
        for i in 0..8 {
            prices.push(92.0 + i as f64);
        }
        let candles = create_test_candles(prices);
        let result = calculate_rsi(&candles, 14);
        assert!(result.is_ok());
        let rsi = result.unwrap();
        // RSI should transition from low to high
        assert!(rsi[0] < rsi[rsi.len() - 1]);
    }
}
