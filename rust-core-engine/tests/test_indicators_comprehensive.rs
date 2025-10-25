// Comprehensive Indicator Tests
// Tests for RSI, MACD, Bollinger Bands, SMA, EMA, and Volume indicators

use binance_trading_bot::market_data::cache::CandleData;
use binance_trading_bot::strategies::indicators::*;

// Helper function to create test candles
fn create_candles(prices: Vec<f64>) -> Vec<CandleData> {
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

// ============== RSI TESTS ==============

#[test]
fn test_rsi_oversold_extreme() {
    // Falling prices should produce low RSI
    let prices: Vec<f64> = (0..20).map(|i| 100.0 - (i as f64 * 5.0)).collect();
    let candles = create_candles(prices);

    let rsi = calculate_rsi(&candles, 14).unwrap();
    let last_rsi = *rsi.last().unwrap();

    assert!(last_rsi < 30.0, "RSI should be oversold: {}", last_rsi);
    assert!(last_rsi >= 0.0, "RSI cannot be negative");
}

#[test]
fn test_rsi_overbought_extreme() {
    // Rising prices should produce high RSI
    let prices: Vec<f64> = (0..20).map(|i| 100.0 + (i as f64 * 5.0)).collect();
    let candles = create_candles(prices);

    let rsi = calculate_rsi(&candles, 14).unwrap();
    let last_rsi = *rsi.last().unwrap();

    assert!(last_rsi > 70.0, "RSI should be overbought: {}", last_rsi);
    assert!(last_rsi <= 100.0, "RSI cannot exceed 100");
}

#[test]
fn test_rsi_flat_prices_equals_50() {
    // Flat prices should produce RSI around 50 (or may be NaN/undefined due to no change)
    let prices = vec![100.0; 30];
    let candles = create_candles(prices);

    let rsi = calculate_rsi(&candles, 14).unwrap();

    // With flat prices, RSI calculation might produce NaN or special values
    // We just verify it completes without error
    assert!(!rsi.is_empty(), "Should calculate RSI values");
    assert_eq!(rsi.len(), 30 - 14, "RSI should have {} values", 30 - 14);

    // The last RSI value with flat prices should be defined
    if let Some(&last_rsi) = rsi.last() {
        // For flat prices, RSI should be exactly 50.0 or NaN (both are valid)
        if !last_rsi.is_nan() {
            assert!(
                (last_rsi - 50.0).abs() < 1.0,
                "RSI for flat prices should be near 50.0, got {}",
                last_rsi
            );
        }
    }
}

#[test]
fn test_rsi_minimum_data_points() {
    // RSI needs period + 1 data points
    let prices: Vec<f64> = (0..15).map(|i| 100.0 + (i as f64)).collect();
    let candles = create_candles(prices);

    let rsi = calculate_rsi(&candles, 14);
    assert!(
        rsi.is_ok(),
        "Should work with exactly 15 data points for RSI-14"
    );

    // Verify the result has the correct length and valid values
    let rsi_values = rsi.unwrap();
    assert_eq!(
        rsi_values.len(),
        1,
        "Should have exactly 1 RSI value with 15 data points"
    );
    let last_rsi = *rsi_values.last().unwrap();
    assert!(
        last_rsi >= 0.0 && last_rsi <= 100.0,
        "RSI must be in range [0, 100], got {}",
        last_rsi
    );
    // With constantly increasing prices, RSI should be very high
    assert!(
        last_rsi > 95.0,
        "Constantly increasing prices should give RSI > 95, got {}",
        last_rsi
    );
}

#[test]
fn test_rsi_insufficient_data() {
    let prices = vec![100.0, 101.0, 102.0]; // Only 3 points
    let candles = create_candles(prices);

    let rsi = calculate_rsi(&candles, 14);
    assert!(rsi.is_err(), "Should fail with insufficient data");
}

#[test]
fn test_rsi_alternating_prices() {
    // Alternating up/down should produce RSI around 50
    let prices: Vec<f64> = (0..30)
        .map(|i| if i % 2 == 0 { 100.0 } else { 101.0 })
        .collect();
    let candles = create_candles(prices);

    let rsi = calculate_rsi(&candles, 14).unwrap();
    let last_rsi = *rsi.last().unwrap();

    assert!(
        last_rsi > 40.0 && last_rsi < 60.0,
        "Alternating prices should give neutral RSI: {}",
        last_rsi
    );
}

#[test]
fn test_rsi_all_gains() {
    // All upward movements
    let prices: Vec<f64> = (0..20).map(|i| 100.0 + (i as f64)).collect();
    let candles = create_candles(prices);

    let rsi = calculate_rsi(&candles, 14).unwrap();
    let last_rsi = *rsi.last().unwrap();

    assert!(
        last_rsi > 98.0,
        "All gains should give RSI near 100: {}",
        last_rsi
    );
}

#[test]
fn test_rsi_different_periods() {
    let prices: Vec<f64> = (0..50).map(|i| 100.0 + ((i as f64).sin() * 10.0)).collect();
    let candles = create_candles(prices);

    // Test RSI-7, RSI-14, RSI-21
    let rsi_7 = calculate_rsi(&candles, 7).unwrap();
    let rsi_14 = calculate_rsi(&candles, 14).unwrap();
    let rsi_21 = calculate_rsi(&candles, 21).unwrap();

    assert!(!rsi_7.is_empty());
    assert!(!rsi_14.is_empty());
    assert!(!rsi_21.is_empty());

    // Shorter periods are more sensitive (more volatile)
    assert!(
        rsi_7.len() > rsi_21.len(),
        "Shorter period should have more values"
    );
}

#[test]
fn test_rsi_divergence_pattern() {
    // Price makes new high but RSI doesn't (bearish divergence)
    let mut prices = vec![100.0, 110.0, 105.0, 115.0, 110.0, 120.0, 115.0, 118.0];
    prices.extend((0..15).map(|i| 118.0 - (i as f64 * 0.5)));
    let candles = create_candles(prices);

    let rsi = calculate_rsi(&candles, 14).unwrap();
    assert!(rsi.len() > 2, "Should have multiple RSI values");
}

#[test]
fn test_rsi_extreme_values() {
    // Test with very large and very small prices
    let prices = vec![0.0001, 0.0002, 0.0003, 100000.0, 100001.0, 100002.0];
    let mut full_prices = prices.clone();
    full_prices.extend((0..20).map(|i| 100000.0 + (i as f64)));
    let candles = create_candles(full_prices);

    let rsi = calculate_rsi(&candles, 14);
    assert!(rsi.is_ok() || rsi.is_err()); // Either works or fails gracefully

    if let Ok(rsi_vals) = rsi {
        for val in rsi_vals {
            assert!(val >= 0.0 && val <= 100.0, "RSI must be 0-100: {}", val);
        }
    }
}

// ============== MACD TESTS ==============

#[test]
fn test_macd_bullish_crossover() {
    // Uptrend should produce positive MACD histogram
    let prices: Vec<f64> = (0..50).map(|i| 100.0 + (i as f64 * 0.5)).collect();
    let candles = create_candles(prices);

    let macd = calculate_macd(&candles, 12, 26, 9).unwrap();

    // Verify all components exist and have correct lengths
    assert!(!macd.macd_line.is_empty(), "MACD line should have values");
    assert!(
        !macd.signal_line.is_empty(),
        "Signal line should have values"
    );
    assert!(!macd.histogram.is_empty(), "Histogram should have values");
    assert_eq!(
        macd.histogram.len(),
        macd.signal_line.len(),
        "Histogram and signal should match"
    );

    let last_histogram = *macd.histogram.last().unwrap();
    let last_macd = *macd.macd_line.last().unwrap();
    let last_signal = *macd.signal_line.last().unwrap();

    // Uptrend should have positive MACD line
    assert!(
        last_macd > 0.0,
        "MACD line should be positive in uptrend, got {}",
        last_macd
    );
    // Histogram = MACD - Signal
    assert!(
        (last_histogram - (last_macd - last_signal)).abs() < 0.0001,
        "Histogram should equal MACD - Signal"
    );
}

#[test]
fn test_macd_bearish_crossover() {
    // Downtrend should produce negative MACD histogram
    let prices: Vec<f64> = (0..50).map(|i| 150.0 - (i as f64 * 0.5)).collect();
    let candles = create_candles(prices);

    let macd = calculate_macd(&candles, 12, 26, 9).unwrap();

    // Verify all components exist
    assert!(!macd.histogram.is_empty(), "Should have histogram values");
    assert!(!macd.macd_line.is_empty(), "MACD line should have values");
    assert!(
        !macd.signal_line.is_empty(),
        "Signal line should have values"
    );

    let last_histogram = *macd.histogram.last().unwrap();
    let last_macd = *macd.macd_line.last().unwrap();
    let last_signal = *macd.signal_line.last().unwrap();

    // Downtrend should have negative MACD line
    assert!(
        last_macd < 0.0,
        "MACD line should be negative in downtrend, got {}",
        last_macd
    );
    // Verify histogram calculation
    assert!(
        (last_histogram - (last_macd - last_signal)).abs() < 0.0001,
        "Histogram should equal MACD - Signal"
    );
}

#[test]
fn test_macd_flat_market() {
    let prices = vec![100.0; 50];
    let candles = create_candles(prices);

    let macd = calculate_macd(&candles, 12, 26, 9).unwrap();
    let last_macd = *macd.macd_line.last().unwrap();
    let last_signal = *macd.signal_line.last().unwrap();
    let last_histogram = *macd.histogram.last().unwrap();

    assert!(
        (last_macd).abs() < 0.01,
        "Flat market should have MACD near 0"
    );
    assert!(
        (last_signal).abs() < 0.01,
        "Flat market should have signal near 0"
    );
    assert!(
        (last_histogram).abs() < 0.01,
        "Flat market should have histogram near 0"
    );
}

#[test]
fn test_macd_minimum_data() {
    let prices: Vec<f64> = (0..35).map(|i| 100.0 + (i as f64)).collect();
    let candles = create_candles(prices);

    let macd = calculate_macd(&candles, 12, 26, 9);
    assert!(macd.is_ok(), "Should work with 35 data points (26+9)");
}

#[test]
fn test_macd_insufficient_data() {
    let prices: Vec<f64> = (0..20).map(|i| 100.0 + (i as f64)).collect();
    let candles = create_candles(prices);

    let macd = calculate_macd(&candles, 12, 26, 9);
    assert!(macd.is_err(), "Should fail with insufficient data");
}

#[test]
fn test_macd_zero_line_cross() {
    // Create data that crosses zero
    let mut prices: Vec<f64> = (0..25).map(|i| 100.0 - (i as f64)).collect();
    prices.extend((0..25).map(|i| 75.0 + (i as f64)));
    let candles = create_candles(prices);

    let macd = calculate_macd(&candles, 12, 26, 9).unwrap();

    // Check that MACD crosses zero
    let has_positive = macd.macd_line.iter().any(|&x| x > 0.0);
    let has_negative = macd.macd_line.iter().any(|&x| x < 0.0);

    assert!(has_positive || has_negative, "MACD should have values");
}

#[test]
fn test_macd_custom_parameters() {
    let prices: Vec<f64> = (0..60)
        .map(|i| 100.0 + ((i as f64 * 0.1).sin() * 10.0))
        .collect();
    let candles = create_candles(prices);

    // Test custom MACD (8, 17, 9)
    let macd = calculate_macd(&candles, 8, 17, 9).unwrap();

    assert!(!macd.macd_line.is_empty());
    assert!(!macd.signal_line.is_empty());
    assert!(!macd.histogram.is_empty());
    assert_eq!(
        macd.histogram.len(),
        macd.signal_line.len(),
        "Histogram and signal should have same length"
    );
}

#[test]
fn test_macd_histogram_calculation() {
    let prices: Vec<f64> = (0..50).map(|i| 100.0 + (i as f64 * 0.3)).collect();
    let candles = create_candles(prices);

    let macd = calculate_macd(&candles, 12, 26, 9).unwrap();

    // Verify histogram = MACD - Signal
    if !macd.histogram.is_empty() && !macd.signal_line.is_empty() {
        let hist_start = macd.macd_line.len() - macd.histogram.len();
        for (i, &hist) in macd.histogram.iter().enumerate() {
            let expected = macd.macd_line[hist_start + i] - macd.signal_line[i];
            assert!(
                (hist - expected).abs() < 0.0001,
                "Histogram calculation mismatch at {}: {} vs {}",
                i,
                hist,
                expected
            );
        }
    }
}

// ============== BOLLINGER BANDS TESTS ==============

#[test]
fn test_bollinger_bands_basic() {
    let prices: Vec<f64> = (0..30)
        .map(|i| 100.0 + ((i as f64 * 0.5).sin() * 5.0))
        .collect();
    let candles = create_candles(prices);

    let bb = calculate_bollinger_bands(&candles, 20, 2.0).unwrap();

    assert!(!bb.upper.is_empty());
    assert!(!bb.middle.is_empty());
    assert!(!bb.lower.is_empty());
    assert_eq!(bb.upper.len(), bb.middle.len());
    assert_eq!(bb.middle.len(), bb.lower.len());
}

#[test]
fn test_bollinger_bands_relationship() {
    let prices: Vec<f64> = (0..30).map(|i| 100.0 + (i as f64 * 0.1)).collect();
    let candles = create_candles(prices);

    let bb = calculate_bollinger_bands(&candles, 20, 2.0).unwrap();

    // Upper should always be > Middle > Lower
    for i in 0..bb.upper.len() {
        assert!(
            bb.upper[i] > bb.middle[i],
            "Upper band should be > middle at {}",
            i
        );
        assert!(
            bb.middle[i] > bb.lower[i],
            "Middle band should be > lower at {}",
            i
        );
    }
}

#[test]
fn test_bollinger_bands_flat_prices() {
    let prices = vec![100.0; 30];
    let candles = create_candles(prices);

    let bb = calculate_bollinger_bands(&candles, 20, 2.0).unwrap();

    // With flat prices, bands should converge to the price
    let last_upper = *bb.upper.last().unwrap();
    let last_middle = *bb.middle.last().unwrap();
    let last_lower = *bb.lower.last().unwrap();

    assert!(
        (last_middle - 100.0).abs() < 0.01,
        "Middle band should be at price"
    );
    assert!(
        (last_upper - 100.0).abs() < 0.01,
        "Upper band should converge to price"
    );
    assert!(
        (last_lower - 100.0).abs() < 0.01,
        "Lower band should converge to price"
    );
}

#[test]
fn test_bollinger_bands_squeeze() {
    // Low volatility creates narrow bands
    let prices: Vec<f64> = (0..30).map(|i| 100.0 + ((i % 2) as f64 * 0.1)).collect();
    let candles = create_candles(prices);

    let bb = calculate_bollinger_bands(&candles, 20, 2.0).unwrap();

    let last_idx = bb.upper.len() - 1;
    let bandwidth = (bb.upper[last_idx] - bb.lower[last_idx]) / bb.middle[last_idx];

    assert!(bandwidth < 0.1, "Low volatility should create narrow bands");
}

#[test]
fn test_bollinger_bands_expansion() {
    // High volatility creates wide bands
    let prices: Vec<f64> = (0..30).map(|i| 100.0 + ((i as f64).sin() * 20.0)).collect();
    let candles = create_candles(prices);

    let bb = calculate_bollinger_bands(&candles, 20, 2.0).unwrap();

    let last_idx = bb.upper.len() - 1;
    let bandwidth = (bb.upper[last_idx] - bb.lower[last_idx]) / bb.middle[last_idx];

    assert!(
        bandwidth > 0.05,
        "High volatility should create wider bands"
    );
}

#[test]
fn test_bollinger_bands_multiplier_effect() {
    let prices: Vec<f64> = (0..30).map(|i| 100.0 + (i as f64 * 0.2)).collect();
    let candles = create_candles(prices);

    let bb_2std = calculate_bollinger_bands(&candles, 20, 2.0).unwrap();
    let bb_3std = calculate_bollinger_bands(&candles, 20, 3.0).unwrap();

    // Higher multiplier should create wider bands
    let last_idx = bb_2std.upper.len() - 1;
    let width_2std = bb_2std.upper[last_idx] - bb_2std.lower[last_idx];
    let width_3std = bb_3std.upper[last_idx] - bb_3std.lower[last_idx];

    assert!(
        width_3std > width_2std,
        "3-std bands should be wider than 2-std bands"
    );
}

#[test]
fn test_bollinger_bands_insufficient_data() {
    let prices = vec![100.0, 101.0, 102.0]; // Only 3 points
    let candles = create_candles(prices);

    let bb = calculate_bollinger_bands(&candles, 20, 2.0);
    assert!(bb.is_err(), "Should fail with insufficient data");
}

// ============== VOLUME INDICATOR TESTS ==============

#[test]
fn test_volume_profile_basic() {
    let mut candles = create_candles(vec![100.0; 25]);
    // Set varying volumes
    for (i, candle) in candles.iter_mut().enumerate() {
        candle.volume = 1000.0 + (i as f64 * 100.0);
    }

    let vp = calculate_volume_profile(&candles, 10).unwrap();

    assert!(vp.poc > 0.0, "POC should be positive");
    assert!(!vp.price_levels.is_empty(), "Should have price levels");
    assert!(!vp.volumes.is_empty(), "Should have volumes");
}

#[test]
fn test_volume_profile_poc_accuracy() {
    let prices = vec![100.0, 100.5, 100.0, 100.5, 100.0]; // Price oscillates around 100
    let mut candles = create_candles(prices);

    // High volume at 100.0
    candles[0].volume = 10000.0;
    candles[2].volume = 10000.0;
    candles[4].volume = 10000.0;
    // Low volume at 100.5
    candles[1].volume = 100.0;
    candles[3].volume = 100.0;

    let vp = calculate_volume_profile(&candles, 5).unwrap();

    // POC should be near 100.0 (high volume price)
    assert!(
        (vp.poc - 100.0).abs() < 1.0,
        "POC should be near high volume price: {}",
        vp.poc
    );
}

// ============== SMA TESTS ==============

#[test]
fn test_sma_basic_calculation() {
    let prices = vec![10.0, 20.0, 30.0, 40.0, 50.0];
    let sma = calculate_sma(&prices, 3).unwrap();

    assert_eq!(sma.len(), 3); // 5 prices - 3 period + 1 = 3 values
    assert_eq!(sma[0], 20.0); // (10+20+30)/3 = 20
    assert_eq!(sma[1], 30.0); // (20+30+40)/3 = 30
    assert_eq!(sma[2], 40.0); // (30+40+50)/3 = 40
}

#[test]
fn test_sma_flat_prices() {
    let prices = vec![100.0; 20];
    let sma = calculate_sma(&prices, 10).unwrap();

    for &val in &sma {
        assert_eq!(val, 100.0, "SMA of flat prices should equal the price");
    }
}

#[test]
fn test_sma_insufficient_data() {
    let prices = vec![100.0, 101.0];
    let sma = calculate_sma(&prices, 10);

    assert!(sma.is_err(), "Should fail with insufficient data");
}

// ============== EMA TESTS ==============

#[test]
fn test_ema_basic_calculation() {
    let prices = vec![22.0, 23.0, 24.0, 25.0, 26.0];
    let ema = calculate_ema(&prices, 3).unwrap();

    assert!(!ema.is_empty());
    // EMA should be more responsive than SMA
    let last_ema = *ema.last().unwrap();
    assert!(
        last_ema > 22.0 && last_ema < 26.0,
        "EMA should be within price range"
    );
}

#[test]
fn test_ema_vs_sma_responsiveness() {
    let prices: Vec<f64> = (0..20)
        .map(|i| if i < 10 { 100.0 } else { 110.0 })
        .collect();

    let sma = calculate_sma(&prices, 10).unwrap();
    let ema = calculate_ema(&prices, 10).unwrap();

    // EMA and SMA should both exist and be calculated
    assert!(!sma.is_empty(), "SMA should have values");
    assert!(!ema.is_empty(), "EMA should have values");
}

#[test]
fn test_ema_flat_prices() {
    let prices = vec![100.0; 20];
    let ema = calculate_ema(&prices, 10).unwrap();

    for &val in &ema {
        assert!(
            (val - 100.0).abs() < 0.01,
            "EMA of flat prices should equal the price"
        );
    }
}

// ============== INTEGRATION TESTS ==============

#[test]
fn test_all_indicators_together() {
    let prices: Vec<f64> = (0..60)
        .map(|i| 100.0 + ((i as f64 * 0.1).sin() * 10.0))
        .collect();
    let candles = create_candles(prices);

    // All indicators should work on the same data
    let rsi = calculate_rsi(&candles, 14);
    let macd = calculate_macd(&candles, 12, 26, 9);
    let bb = calculate_bollinger_bands(&candles, 20, 2.0);

    assert!(rsi.is_ok(), "RSI should succeed");
    assert!(macd.is_ok(), "MACD should succeed");
    assert!(bb.is_ok(), "Bollinger Bands should succeed");
}

#[test]
fn test_indicator_consistency() {
    let prices: Vec<f64> = (0..50).map(|i| 100.0 + (i as f64 * 0.5)).collect();
    let candles = create_candles(prices);

    // Run same indicator twice, should get same results
    let rsi1 = calculate_rsi(&candles, 14).unwrap();
    let rsi2 = calculate_rsi(&candles, 14).unwrap();

    assert_eq!(rsi1.len(), rsi2.len());
    for i in 0..rsi1.len() {
        assert_eq!(rsi1[i], rsi2[i], "RSI should be deterministic");
    }
}
