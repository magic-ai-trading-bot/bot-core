use binance_trading_bot::paper_trading::trade::{PaperTrade, TradeStatus, TradeType};

/// Test that trailing stop activates after reaching profit threshold
#[test]
fn test_trailing_activation_on_profit() {
    let mut trade = PaperTrade::new(
        "BTCUSDT".to_string(),
        TradeType::Long,
        100.0, // entry
        1.0,   // quantity
        3,     // leverage
        0.001, // fee
        None,
        None,
        None,
    );

    // Initially no trailing stop
    assert!(!trade.trailing_stop_active);
    assert!(trade.highest_price_achieved.is_none());

    // Move to +5% (activation threshold)
    trade.update_trailing_stop(105.0, 3.0, 5.0);

    // Should activate
    assert!(trade.trailing_stop_active);
    assert_eq!(trade.highest_price_achieved, Some(105.0));

    // Stop should be set to 3% below $105 = $101.85
    let expected_stop = 105.0 * (1.0 - 3.0 / 100.0);
    assert!((trade.stop_loss.unwrap() - expected_stop).abs() < 0.01);
}

/// Test that trailing stop does NOT activate below threshold
#[test]
fn test_no_activation_below_threshold() {
    let mut trade = PaperTrade::new(
        "BTCUSDT".to_string(),
        TradeType::Long,
        100.0,
        1.0,
        3,
        0.001,
        None,
        None,
        None,
    );

    // Move to +3% (below 5% threshold)
    trade.update_trailing_stop(103.0, 3.0, 5.0);

    // Should NOT activate
    assert!(!trade.trailing_stop_active);
    assert!(trade.highest_price_achieved.is_none());
    assert!(trade.stop_loss.is_none());
}

/// Test LONG position trailing: price moves up, stop follows
#[test]
fn test_long_trailing_moves_up() {
    let mut trade = PaperTrade::new(
        "BTCUSDT".to_string(),
        TradeType::Long,
        100.0,
        1.0,
        3,
        0.001,
        None,
        None,
        None,
    );

    // Activate at +5%
    trade.update_trailing_stop(105.0, 3.0, 5.0);
    let first_stop = trade.stop_loss.unwrap();
    assert!((first_stop - 101.85).abs() < 0.01); // 105 * 0.97

    // Price moves to +10%
    trade.update_trailing_stop(110.0, 3.0, 5.0);
    let second_stop = trade.stop_loss.unwrap();
    assert!(second_stop > first_stop); // Stop moved up
    assert!((second_stop - 106.70).abs() < 0.01); // 110 * 0.97
}

/// Test LONG position trailing: price drops, stop stays (doesn't move down)
#[test]
fn test_long_trailing_stops_dont_move_down() {
    let mut trade = PaperTrade::new(
        "BTCUSDT".to_string(),
        TradeType::Long,
        100.0,
        1.0,
        3,
        0.001,
        None,
        None,
        None,
    );

    // Activate and move stop up
    trade.update_trailing_stop(110.0, 3.0, 5.0);
    let high_stop = trade.stop_loss.unwrap();
    assert!((high_stop - 106.70).abs() < 0.01);

    // Price drops to $108
    trade.update_trailing_stop(108.0, 3.0, 5.0);

    // Stop should NOT move down
    assert_eq!(trade.stop_loss.unwrap(), high_stop);
}

/// Test LONG position: trailing stop gets hit
#[test]
fn test_long_trailing_stop_hit() {
    let mut trade = PaperTrade::new(
        "BTCUSDT".to_string(),
        TradeType::Long,
        100.0,
        1.0,
        3,
        0.001,
        None,
        None,
        None,
    );

    // Activate trailing
    trade.update_trailing_stop(110.0, 3.0, 5.0);
    let _stop_price = trade.stop_loss.unwrap(); // ~106.70

    // Check if trailing stop is hit
    assert!(trade.should_stop_loss(106.0)); // Below stop
    assert!(!trade.should_stop_loss(107.0)); // Above stop
}

/// Test SHORT position trailing: price moves down, stop follows
#[test]
fn test_short_trailing_moves_down() {
    let mut trade = PaperTrade::new(
        "BTCUSDT".to_string(),
        TradeType::Short,
        100.0,
        1.0,
        3,
        0.001,
        None,
        None,
        None,
    );

    // Activate at -5% (price drops to $95)
    trade.update_trailing_stop(95.0, 3.0, 5.0);
    let first_stop = trade.stop_loss.unwrap();
    assert!((first_stop - 97.85).abs() < 0.01); // 95 * 1.03

    // Price drops to $90 (-10%)
    trade.update_trailing_stop(90.0, 3.0, 5.0);
    let second_stop = trade.stop_loss.unwrap();
    assert!(second_stop < first_stop); // Stop moved down
    assert!((second_stop - 92.70).abs() < 0.01); // 90 * 1.03
}

/// Test SHORT position trailing: price rises, stop stays (doesn't move up)
#[test]
fn test_short_trailing_stops_dont_move_up() {
    let mut trade = PaperTrade::new(
        "BTCUSDT".to_string(),
        TradeType::Short,
        100.0,
        1.0,
        3,
        0.001,
        None,
        None,
        None,
    );

    // Activate and move stop down
    trade.update_trailing_stop(90.0, 3.0, 5.0);
    let low_stop = trade.stop_loss.unwrap();
    assert!((low_stop - 92.70).abs() < 0.01);

    // Price rises to $92
    trade.update_trailing_stop(92.0, 3.0, 5.0);

    // Stop should NOT move up
    assert_eq!(trade.stop_loss.unwrap(), low_stop);
}

/// Test SHORT position: trailing stop gets hit
#[test]
fn test_short_trailing_stop_hit() {
    let mut trade = PaperTrade::new(
        "BTCUSDT".to_string(),
        TradeType::Short,
        100.0,
        1.0,
        3,
        0.001,
        None,
        None,
        None,
    );

    // Activate trailing at $90
    trade.update_trailing_stop(90.0, 3.0, 5.0);
    let _stop_price = trade.stop_loss.unwrap(); // ~92.70

    // Check if trailing stop is hit
    assert!(trade.should_stop_loss(93.0)); // Above stop (bad for short)
    assert!(!trade.should_stop_loss(92.0)); // Below stop (still good)
}

/// Test that closed trades don't update trailing stops
#[test]
fn test_closed_trade_no_trailing_update() {
    let mut trade = PaperTrade::new(
        "BTCUSDT".to_string(),
        TradeType::Long,
        100.0,
        1.0,
        3,
        0.001,
        None,
        None,
        None,
    );

    // Activate trailing
    trade.update_trailing_stop(110.0, 3.0, 5.0);
    assert!(trade.trailing_stop_active);

    // Close the trade
    trade.status = TradeStatus::Closed;

    // Try to update trailing (should do nothing)
    let old_stop = trade.stop_loss;
    trade.update_trailing_stop(120.0, 3.0, 5.0);

    // Stop should not change
    assert_eq!(trade.stop_loss, old_stop);
}

/// Test trailing stop with no initial stop loss
#[test]
fn test_trailing_creates_stop_when_none_exists() {
    let mut trade = PaperTrade::new(
        "BTCUSDT".to_string(),
        TradeType::Long,
        100.0,
        1.0,
        3,
        0.001,
        None,
        None,
        None,
    );

    // No initial stop loss
    assert!(trade.stop_loss.is_none());

    // Activate trailing
    trade.update_trailing_stop(110.0, 3.0, 5.0);

    // Should create a stop loss
    assert!(trade.stop_loss.is_some());
    assert!((trade.stop_loss.unwrap() - 106.70).abs() < 0.01);
}

/// Test price exactly at activation threshold
#[test]
fn test_activation_at_exact_threshold() {
    let mut trade = PaperTrade::new(
        "BTCUSDT".to_string(),
        TradeType::Long,
        100.0,
        1.0,
        3,
        0.001,
        None,
        None,
        None,
    );

    // Price exactly at +5.0%
    trade.update_trailing_stop(105.0, 3.0, 5.0);

    // Should activate (>= not just >)
    assert!(trade.trailing_stop_active);
}

/// Test multiple price updates tracking highest correctly
#[test]
fn test_multiple_updates_track_highest() {
    let mut trade = PaperTrade::new(
        "BTCUSDT".to_string(),
        TradeType::Long,
        100.0,
        1.0,
        3,
        0.001,
        None,
        None,
        None,
    );

    // Price sequence: 100 → 105 → 110 → 108 → 112 → 109
    trade.update_trailing_stop(105.0, 3.0, 5.0); // Activate
    assert_eq!(trade.highest_price_achieved, Some(105.0));

    trade.update_trailing_stop(110.0, 3.0, 5.0); // New high
    assert_eq!(trade.highest_price_achieved, Some(110.0));

    trade.update_trailing_stop(108.0, 3.0, 5.0); // Drop (highest stays)
    assert_eq!(trade.highest_price_achieved, Some(110.0));

    trade.update_trailing_stop(112.0, 3.0, 5.0); // New high
    assert_eq!(trade.highest_price_achieved, Some(112.0));

    trade.update_trailing_stop(109.0, 3.0, 5.0); // Drop (highest stays)
    assert_eq!(trade.highest_price_achieved, Some(112.0));

    // Stop should be at 3% below $112 = $108.64
    let expected_stop = 112.0 * 0.97;
    assert!((trade.stop_loss.unwrap() - expected_stop).abs() < 0.01);
}

/// Test activation persists even if profit drops below threshold
#[test]
fn test_activation_persists_after_profit_drop() {
    let mut trade = PaperTrade::new(
        "BTCUSDT".to_string(),
        TradeType::Long,
        100.0,
        1.0,
        3,
        0.001,
        None,
        None,
        None,
    );

    // Activate at +10%
    trade.update_trailing_stop(110.0, 3.0, 5.0);
    assert!(trade.trailing_stop_active);

    // Price drops to +3% (below activation threshold)
    trade.update_trailing_stop(103.0, 3.0, 5.0);

    // Should still be active (once activated, stays active)
    assert!(trade.trailing_stop_active);
}

/// Test trailing stop replaces fixed stop when better (LONG)
#[test]
fn test_trailing_replaces_fixed_stop_long() {
    let mut trade = PaperTrade::new(
        "BTCUSDT".to_string(),
        TradeType::Long,
        100.0,
        1.0,
        3,
        0.001,
        None,
        None,
        None,
    );

    // Set fixed stop loss at $95 (-5%)
    trade.stop_loss = Some(95.0);

    // Price moves to $110, activate trailing
    trade.update_trailing_stop(110.0, 3.0, 5.0);

    // Trailing stop should be $106.70, which is better than $95
    let trailing_stop = trade.stop_loss.unwrap();
    assert!(trailing_stop > 95.0);
    assert!((trailing_stop - 106.70).abs() < 0.01);
}

/// Test trailing stop replaces fixed stop when better (SHORT)
#[test]
fn test_trailing_replaces_fixed_stop_short() {
    let mut trade = PaperTrade::new(
        "BTCUSDT".to_string(),
        TradeType::Short,
        100.0,
        1.0,
        3,
        0.001,
        None,
        None,
        None,
    );

    // Set fixed stop loss at $105 (+5%)
    trade.stop_loss = Some(105.0);

    // Price drops to $90, activate trailing
    trade.update_trailing_stop(90.0, 3.0, 5.0);

    // Trailing stop should be $92.70, which is better than $105
    let trailing_stop = trade.stop_loss.unwrap();
    assert!(trailing_stop < 105.0);
    assert!((trailing_stop - 92.70).abs() < 0.01);
}

/// Test different trailing percentages
#[test]
fn test_different_trailing_percentages() {
    let mut trade = PaperTrade::new(
        "BTCUSDT".to_string(),
        TradeType::Long,
        100.0,
        1.0,
        3,
        0.001,
        None,
        None,
        None,
    );

    // Test with 5% trailing (wider)
    trade.update_trailing_stop(110.0, 5.0, 5.0);
    let wide_stop = trade.stop_loss.unwrap();
    assert!((wide_stop - 104.50).abs() < 0.01); // 110 * 0.95

    // Reset and test with 2% trailing (tighter)
    let mut trade2 = PaperTrade::new(
        "BTCUSDT".to_string(),
        TradeType::Long,
        100.0,
        1.0,
        3,
        0.001,
        None,
        None,
        None,
    );

    trade2.update_trailing_stop(110.0, 2.0, 5.0);
    let tight_stop = trade2.stop_loss.unwrap();
    assert!((tight_stop - 107.80).abs() < 0.01); // 110 * 0.98

    // Tighter stop should be higher than wider stop
    assert!(tight_stop > wide_stop);
}

/// Test different activation thresholds
#[test]
fn test_different_activation_thresholds() {
    // Test 3% activation
    let mut trade1 = PaperTrade::new(
        "BTCUSDT".to_string(),
        TradeType::Long,
        100.0,
        1.0,
        3,
        0.001,
        None,
        None,
        None,
    );

    trade1.update_trailing_stop(103.0, 3.0, 3.0); // 3% threshold
    assert!(trade1.trailing_stop_active); // Should activate

    // Test 10% activation
    let mut trade2 = PaperTrade::new(
        "BTCUSDT".to_string(),
        TradeType::Long,
        100.0,
        1.0,
        3,
        0.001,
        None,
        None,
        None,
    );

    trade2.update_trailing_stop(108.0, 3.0, 10.0); // 10% threshold
    assert!(!trade2.trailing_stop_active); // Should NOT activate (+8% < 10%)

    trade2.update_trailing_stop(110.0, 3.0, 10.0); // 10% threshold
    assert!(trade2.trailing_stop_active); // Should activate (+10% >= 10%)
}
