mod common;

use binance_trading_bot::strategies::{
    indicators::{calculate_rsi, calculate_macd, calculate_bollinger_bands, calculate_sma, calculate_ema},
    rsi_strategy::RsiStrategy,
    macd_strategy::MacdStrategy,
    Strategy,
};

#[test]
fn test_calculate_rsi() {
    let prices = vec![44.0, 44.25, 44.5, 43.75, 44.65, 45.12, 45.84, 46.08, 45.89, 46.03, 45.61, 46.28, 46.28, 46.0, 46.03];
    let rsi = calculate_rsi(&prices, 14);
    
    // RSI should be between 0 and 100
    assert!(rsi >= 0.0 && rsi <= 100.0);
    
    // Test with all gains (should be near 100)
    let rising_prices: Vec<f64> = (1..20).map(|i| i as f64).collect();
    let high_rsi = calculate_rsi(&rising_prices, 14);
    assert!(high_rsi > 70.0);
    
    // Test with all losses (should be near 0)
    let falling_prices: Vec<f64> = (1..20).rev().map(|i| i as f64).collect();
    let low_rsi = calculate_rsi(&falling_prices, 14);
    assert!(low_rsi < 30.0);
}

#[test]
fn test_calculate_macd() {
    let prices = vec![100.0, 102.0, 101.5, 103.0, 104.0, 103.5, 105.0, 106.0, 105.5, 107.0, 108.0, 107.5, 109.0, 110.0, 109.5, 111.0, 112.0, 111.5, 113.0, 114.0, 113.5, 115.0, 116.0, 115.5, 117.0, 118.0];
    
    let (macd_line, signal_line, histogram) = calculate_macd(&prices, 12, 26, 9);
    
    // Verify calculations are numeric
    assert!(!macd_line.is_nan());
    assert!(!signal_line.is_nan());
    assert!(!histogram.is_nan());
    
    // Histogram should be MACD - Signal
    assert!((histogram - (macd_line - signal_line)).abs() < 0.0001);
}

#[test]
fn test_calculate_bollinger_bands() {
    let prices = vec![100.0, 101.0, 102.0, 101.5, 100.5, 99.5, 100.0, 101.0, 102.5, 103.0, 102.0, 101.0, 100.0, 99.0, 100.0, 101.0, 102.0, 103.0, 104.0, 103.5];
    
    let (upper, middle, lower) = calculate_bollinger_bands(&prices, 20, 2.0);
    
    // Middle band should be SMA
    let sma = calculate_sma(&prices, 20);
    assert!((middle - sma).abs() < 0.0001);
    
    // Upper band should be above middle
    assert!(upper > middle);
    
    // Lower band should be below middle
    assert!(lower < middle);
    
    // Bands should be symmetric around middle
    assert!((upper - middle - (middle - lower)).abs() < 0.0001);
}

#[test]
fn test_calculate_sma() {
    let prices = vec![10.0, 20.0, 30.0, 40.0, 50.0];
    
    // SMA of 5 values should be their average
    let sma = calculate_sma(&prices, 5);
    assert!((sma - 30.0).abs() < 0.0001);
    
    // Test with period smaller than data
    let sma_3 = calculate_sma(&prices, 3);
    // Should use last 3 values: 30, 40, 50
    assert!((sma_3 - 40.0).abs() < 0.0001);
}

#[test]
fn test_calculate_ema() {
    let prices = vec![10.0, 11.0, 12.0, 13.0, 14.0, 15.0, 14.0, 13.0, 12.0, 11.0];
    
    let ema = calculate_ema(&prices, 5);
    
    // EMA should be a valid number
    assert!(!ema.is_nan());
    
    // EMA should be within price range
    let min_price = prices.iter().fold(f64::INFINITY, |a, &b| a.min(b));
    let max_price = prices.iter().fold(f64::NEG_INFINITY, |a, &b| a.max(b));
    assert!(ema >= min_price && ema <= max_price);
}

#[test]
fn test_rsi_strategy_creation() {
    let strategy = RsiStrategy::new();
    // Just test that it can be created
    assert_eq!(strategy.name(), "RSI Strategy");
}

#[test]
fn test_macd_strategy_creation() {
    let strategy = MacdStrategy::new();
    assert_eq!(strategy.name(), "MACD Strategy");
}

#[test]
fn test_indicator_edge_cases() {
    // Test with empty data
    let empty: Vec<f64> = vec![];
    assert!(calculate_rsi(&empty, 14).is_nan() || calculate_rsi(&empty, 14) == 50.0);
    
    // Test with single value
    let single = vec![100.0];
    assert!(calculate_sma(&single, 1) == 100.0);
    
    // Test with same values (no volatility)
    let flat = vec![100.0; 20];
    let rsi_flat = calculate_rsi(&flat, 14);
    assert!(rsi_flat == 50.0 || rsi_flat.is_nan()); // RSI undefined for flat data
    
    let (upper_flat, middle_flat, lower_flat) = calculate_bollinger_bands(&flat, 20, 2.0);
    assert_eq!(upper_flat, middle_flat); // No volatility means bands collapse to SMA
    assert_eq!(lower_flat, middle_flat);
}

#[test]
fn test_extreme_values() {
    // Test with very large numbers
    let large = vec![1e10, 1e10 + 1.0, 1e10 + 2.0];
    let sma_large = calculate_sma(&large, 3);
    assert!(!sma_large.is_nan());
    
    // Test with very small numbers
    let small = vec![1e-10, 2e-10, 3e-10];
    let sma_small = calculate_sma(&small, 3);
    assert!(!sma_small.is_nan());
    
    // Test with negative numbers
    let negative = vec![-100.0, -50.0, -25.0, -10.0, -5.0];
    let rsi_neg = calculate_rsi(&negative, 4);
    assert!(rsi_neg >= 0.0 && rsi_neg <= 100.0);
}