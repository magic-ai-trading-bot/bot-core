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
    let rs = if avg_loss == 0.0 {
        100.0
    } else {
        avg_gain / avg_loss
    };
    rsi_values.push(100.0 - (100.0 / (1.0 + rs)));

    // Calculate subsequent RSI values using smoothed averages
    for i in period..gains.len() {
        avg_gain = ((avg_gain * (period - 1) as f64) + gains[i]) / period as f64;
        avg_loss = ((avg_loss * (period - 1) as f64) + losses[i]) / period as f64;

        let rs = if avg_loss == 0.0 {
            100.0
        } else {
            avg_gain / avg_loss
        };
        rsi_values.push(100.0 - (100.0 / (1.0 + rs)));
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
        .max_by(|(_, a), (_, b)| a.partial_cmp(b).unwrap())
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
        let ema = (price * multiplier) + (ema_values.last().unwrap() * (1.0 - multiplier));
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
        let window = &candles[i - k_period + 1..=i];
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
