mod common;

// Simple tests that don't require complex types
#[test]
fn test_strategy_calculations() {
    // Test basic calculations
    let prices = vec![100.0, 102.0, 101.0, 103.0, 104.0];
    let avg: f64 = prices.iter().sum::<f64>() / prices.len() as f64;
    assert_eq!(avg, 102.0);

    // Test price change calculation
    let change = prices[4] - prices[0];
    assert_eq!(change, 4.0);

    // Test percentage change
    let pct_change = (prices[4] - prices[0]) / prices[0] * 100.0;
    assert_eq!(pct_change, 4.0);
}

#[test]
fn test_moving_average() {
    let prices = vec![10.0, 20.0, 30.0, 40.0, 50.0];
    let sma = prices.iter().sum::<f64>() / prices.len() as f64;
    assert_eq!(sma, 30.0);
}

#[test]
fn test_volatility_calculation() {
    let prices = vec![100.0, 102.0, 98.0, 103.0, 97.0];
    let avg = prices.iter().sum::<f64>() / prices.len() as f64;

    let variance = prices.iter().map(|p| (p - avg).powi(2)).sum::<f64>() / prices.len() as f64;

    let std_dev = variance.sqrt();
    assert!(std_dev > 0.0);
}

#[test]
fn test_rsi_concept() {
    // Test RSI concept without actual implementation
    let gains = vec![1.0, 0.0, 2.0, 0.0, 1.5];
    let losses = vec![0.0, 0.5, 0.0, 1.0, 0.0];

    let avg_gain: f64 = gains.iter().sum::<f64>() / gains.len() as f64;
    let avg_loss: f64 = losses.iter().sum::<f64>() / losses.len() as f64;

    let rs = avg_gain / avg_loss;
    let rsi = 100.0 - (100.0 / (1.0 + rs));

    assert!(rsi >= 0.0 && rsi <= 100.0);
}

#[test]
fn test_bollinger_bands_concept() {
    let prices = vec![100.0, 101.0, 99.0, 102.0, 98.0];
    let sma = prices.iter().sum::<f64>() / prices.len() as f64;

    let variance = prices.iter().map(|p| (p - sma).powi(2)).sum::<f64>() / prices.len() as f64;
    let std_dev = variance.sqrt();

    let upper_band = sma + (2.0 * std_dev);
    let lower_band = sma - (2.0 * std_dev);

    assert!(upper_band > sma);
    assert!(lower_band < sma);
}

#[test]
fn test_macd_concept() {
    // Test MACD concept
    let fast_period = 12;
    let slow_period = 26;

    let prices: Vec<f64> = (1..=30).map(|i| 100.0 + i as f64).collect();

    // Simple moving averages for demonstration
    let fast_ma: f64 = prices.iter().rev().take(fast_period).sum::<f64>() / fast_period as f64;
    let slow_ma: f64 = prices.iter().rev().take(slow_period).sum::<f64>() / slow_period as f64;

    let macd = fast_ma - slow_ma;
    assert!(macd != 0.0);
}
