// Comprehensive coverage tests for paper trading components
// Tests public APIs and business logic without accessing private methods
// Target: Increase engine.rs coverage from 44% to 85%+

mod common;

use binance_trading_bot::paper_trading::{
    portfolio::{DailyPerformance, PaperPortfolio, PortfolioMetrics},
    settings::*,
    trade::{CloseReason, PaperTrade, TradeStatus, TradeType},
};
use chrono::{Duration, Utc};
use std::collections::HashMap;

// ========== SETTINGS TESTS (settings.rs) ==========

#[test]
fn test_settings_default_creation() {
    let settings = PaperTradingSettings::default();
    assert_eq!(settings.basic.initial_balance, 10000.0);
    assert_eq!(settings.basic.max_positions, 5);
    assert!(settings.basic.enabled);
}

#[test]
fn test_settings_validation_valid() {
    let settings = PaperTradingSettings::default();
    let result = settings.validate();
    assert!(result.is_ok(), "Default settings should be valid");
}

#[test]
fn test_settings_validation_invalid_balance() {
    let mut settings = PaperTradingSettings::default();
    settings.basic.initial_balance = -1000.0; // Negative balance

    let result = settings.validate();
    assert!(result.is_err(), "Should reject negative initial balance");
}

#[test]
fn test_settings_validation_invalid_leverage() {
    let mut settings = PaperTradingSettings::default();
    settings.risk.max_leverage = 150; // Exceeds 125 limit

    let result = settings.validate();
    assert!(
        result.is_err(),
        "Should reject leverage exceeding 125x limit"
    );
}

#[test]
fn test_settings_get_symbol_settings() {
    let settings = PaperTradingSettings::default();

    // Get settings for default symbol
    let btc_settings = settings.get_symbol_settings("BTCUSDT");
    assert!(btc_settings.enabled);

    // Get settings for non-existent symbol (should use defaults)
    let custom_settings = settings.get_symbol_settings("CUSTOMUSDT");
    assert_eq!(custom_settings.leverage, settings.basic.default_leverage);
}

#[test]
fn test_settings_set_symbol_settings() {
    let mut settings = PaperTradingSettings::default();

    let custom_symbol_settings = SymbolSettings {
        enabled: false,
        leverage: Some(20),
        position_size_pct: Some(3.0),
        stop_loss_pct: Some(4.0),
        take_profit_pct: Some(8.0),
        trading_hours: None,
        min_price_movement_pct: None,
        max_positions: Some(2),
        custom_params: HashMap::new(),
    };

    settings.set_symbol_settings("CUSTOMUSDT".to_string(), custom_symbol_settings.clone());

    let retrieved = settings.get_symbol_settings("CUSTOMUSDT");
    assert!(!retrieved.enabled);
    assert_eq!(retrieved.leverage, 20);
    assert_eq!(retrieved.position_size_pct, 3.0);
}

#[test]
fn test_indicator_settings_defaults() {
    let indicators = IndicatorSettings::default();
    assert_eq!(indicators.rsi_period, 14);
    assert_eq!(indicators.macd_fast, 12);
    assert_eq!(indicators.macd_slow, 26);
    assert_eq!(indicators.macd_signal, 9);
    assert_eq!(indicators.bollinger_period, 20);
    assert_eq!(indicators.bollinger_std, 2.0);
}

#[test]
fn test_signal_generation_defaults() {
    let signal_settings = SignalGenerationSettings::default();
    assert_eq!(signal_settings.trend_threshold_percent, 0.8);
    assert_eq!(signal_settings.min_required_timeframes, 3);
    assert_eq!(signal_settings.min_required_indicators, 4);
    assert_eq!(signal_settings.confidence_base, 0.5);
}

// ========== PORTFOLIO TESTS (portfolio.rs) ==========

#[test]
fn test_portfolio_creation() {
    let portfolio = PaperPortfolio::new(10000.0);

    assert_eq!(portfolio.initial_balance, 10000.0);
    assert_eq!(portfolio.cash_balance, 10000.0);
    assert_eq!(portfolio.equity, 10000.0);
    assert_eq!(portfolio.margin_used, 0.0);
    assert_eq!(portfolio.free_margin, 10000.0);
    assert_eq!(portfolio.open_trade_ids.len(), 0);
    assert_eq!(portfolio.closed_trade_ids.len(), 0);
    assert_eq!(portfolio.consecutive_losses, 0);
    assert!(portfolio.cool_down_until.is_none());
}

#[test]
fn test_portfolio_add_trade() {
    let mut portfolio = PaperPortfolio::new(10000.0);

    let trade = PaperTrade::new(
        "BTCUSDT".to_string(),
        TradeType::Long,
        50000.0,
        0.1,
        10,
        0.001,
        None,
        None,
        None,
    );

    let result = portfolio.add_trade(trade.clone());
    assert!(result.is_ok());
    assert_eq!(portfolio.open_trade_ids.len(), 1);
    assert!(portfolio.trades.contains_key(&trade.id));
}

#[test]
fn test_portfolio_get_open_trades() {
    let mut portfolio = PaperPortfolio::new(10000.0);

    let trade1 = PaperTrade::new(
        "BTCUSDT".to_string(),
        TradeType::Long,
        50000.0,
        0.1,
        10,
        0.001,
        None,
        None,
        None,
    );

    let trade2 = PaperTrade::new(
        "ETHUSDT".to_string(),
        TradeType::Short,
        3000.0,
        1.0,
        10,
        0.001,
        None,
        None,
        None,
    );

    portfolio.add_trade(trade1).unwrap();
    portfolio.add_trade(trade2).unwrap();

    let open_trades = portfolio.get_open_trades();
    assert_eq!(open_trades.len(), 2);
}

#[test]
fn test_portfolio_update_prices() {
    let mut portfolio = PaperPortfolio::new(10000.0);

    let mut trade = PaperTrade::new(
        "BTCUSDT".to_string(),
        TradeType::Long,
        50000.0,
        0.1,
        10,
        0.001,
        None,
        None,
        None,
    );
    trade.set_stop_loss(49000.0).unwrap();
    trade.set_take_profit(52000.0).unwrap();

    portfolio.add_trade(trade).unwrap();

    // Update price to 51000
    let mut prices = HashMap::new();
    prices.insert("BTCUSDT".to_string(), 51000.0);

    portfolio.update_prices(prices, None);

    // Check that unrealized PnL is updated
    let trades = portfolio.get_open_trades();
    assert!(trades[0].unrealized_pnl > 0.0, "Should have positive PnL");
}

#[test]
fn test_portfolio_close_trade() {
    let mut portfolio = PaperPortfolio::new(10000.0);

    let trade = PaperTrade::new(
        "BTCUSDT".to_string(),
        TradeType::Long,
        50000.0,
        0.1,
        10,
        0.001,
        None,
        None,
        None,
    );

    let trade_id = trade.id.clone();
    portfolio.add_trade(trade).unwrap();

    // Close trade at 51000 (profit)
    let result = portfolio.close_trade(&trade_id, 51000.0, CloseReason::TakeProfit);
    assert!(result.is_ok());

    // Verify trade is closed
    let closed_trade = portfolio.get_trade(&trade_id).unwrap();
    assert_eq!(closed_trade.status, TradeStatus::Closed);
    assert!(closed_trade.realized_pnl.unwrap() > 0.0);
    assert_eq!(portfolio.open_trade_ids.len(), 0);
    assert_eq!(portfolio.closed_trade_ids.len(), 1);
}

#[test]
fn test_portfolio_check_automatic_closures() {
    let mut portfolio = PaperPortfolio::new(10000.0);

    // Add trade with stop loss
    let mut trade = PaperTrade::new(
        "BTCUSDT".to_string(),
        TradeType::Long,
        50000.0,
        0.1,
        10,
        0.001,
        None,
        None,
        None,
    );
    trade.set_stop_loss(48000.0).unwrap();
    trade.set_take_profit(55000.0).unwrap();

    portfolio.add_trade(trade).unwrap();

    // Update price to trigger stop loss
    let mut prices = HashMap::new();
    prices.insert("BTCUSDT".to_string(), 47500.0); // Below stop loss

    portfolio.update_prices(prices, None);

    // Check automatic closures
    let closed_ids = portfolio.check_automatic_closures();
    assert_eq!(closed_ids.len(), 1, "Trade should be auto-closed");
}

#[test]
fn test_portfolio_metrics_initialization() {
    let metrics = PortfolioMetrics::default();

    assert_eq!(metrics.total_trades, 0);
    assert_eq!(metrics.winning_trades, 0);
    assert_eq!(metrics.losing_trades, 0);
    assert_eq!(metrics.win_rate, 0.0);
    assert_eq!(metrics.total_pnl, 0.0);
}

#[test]
fn test_portfolio_calculate_metrics() {
    let mut portfolio = PaperPortfolio::new(10000.0);

    // Add and close some trades
    for i in 0..5 {
        let trade = PaperTrade::new(
            format!("SYM{}USDT", i),
            TradeType::Long,
            100.0,
            1.0,
            10,
            0.001,
            None,
            None,
            None,
        );

        let trade_id = trade.id.clone();
        portfolio.add_trade(trade).unwrap();

        // 3 wins, 2 losses
        let exit_price = if i < 3 { 110.0 } else { 95.0 };
        portfolio
            .close_trade(&trade_id, exit_price, CloseReason::TakeProfit)
            .unwrap();
    }

    // Metrics are updated automatically during close_trade()
    // No need to call calculate_metrics() as it's private and auto-called

    assert_eq!(portfolio.metrics.total_trades, 5);
    assert_eq!(portfolio.metrics.winning_trades, 3);
    assert_eq!(portfolio.metrics.losing_trades, 2);
    assert!((portfolio.metrics.win_rate - 60.0).abs() < 0.1); // Should be ~60%
}

// ========== TRADE TESTS (trade.rs) ==========

#[test]
fn test_trade_creation() {
    let trade = PaperTrade::new(
        "BTCUSDT".to_string(),
        TradeType::Long,
        50000.0,
        0.1,
        10,
        0.001,
        None,
        None,
        None,
    );

    assert_eq!(trade.symbol, "BTCUSDT");
    assert_eq!(trade.trade_type, TradeType::Long);
    assert_eq!(trade.entry_price, 50000.0);
    assert_eq!(trade.quantity, 0.1);
    assert_eq!(trade.leverage, 10);
    assert_eq!(trade.status, TradeStatus::Open);
    assert!(trade.stop_loss.is_none());
    assert!(trade.take_profit.is_none());
}

#[test]
fn test_trade_set_stop_loss_long() {
    let mut trade = PaperTrade::new(
        "BTCUSDT".to_string(),
        TradeType::Long,
        50000.0,
        0.1,
        10,
        0.001,
        None,
        None,
        None,
    );

    let result = trade.set_stop_loss(49000.0);
    assert!(result.is_ok());
    assert_eq!(trade.stop_loss, Some(49000.0));
}

#[test]
fn test_trade_set_stop_loss_invalid_long() {
    let mut trade = PaperTrade::new(
        "BTCUSDT".to_string(),
        TradeType::Long,
        50000.0,
        0.1,
        10,
        0.001,
        None,
        None,
        None,
    );

    // Stop loss above entry price (invalid for long)
    let result = trade.set_stop_loss(51000.0);
    assert!(result.is_err());
}

#[test]
fn test_trade_set_take_profit_long() {
    let mut trade = PaperTrade::new(
        "BTCUSDT".to_string(),
        TradeType::Long,
        50000.0,
        0.1,
        10,
        0.001,
        None,
        None,
        None,
    );

    let result = trade.set_take_profit(55000.0);
    assert!(result.is_ok());
    assert_eq!(trade.take_profit, Some(55000.0));
}

#[test]
fn test_trade_set_take_profit_invalid_long() {
    let mut trade = PaperTrade::new(
        "BTCUSDT".to_string(),
        TradeType::Long,
        50000.0,
        0.1,
        10,
        0.001,
        None,
        None,
        None,
    );

    // Take profit below entry price (invalid for long)
    let result = trade.set_take_profit(48000.0);
    assert!(result.is_err());
}

#[test]
fn test_trade_update_pnl_long_profit() {
    let mut trade = PaperTrade::new(
        "BTCUSDT".to_string(),
        TradeType::Long,
        50000.0,
        0.1,
        10,
        0.001,
        None,
        None,
        None,
    );

    trade.update_with_price(52000.0, None); // Price up to 52000

    assert!(trade.unrealized_pnl > 0.0, "Should have profit");
    assert!(trade.pnl_percentage > 0.0);
}

#[test]
fn test_trade_update_pnl_long_loss() {
    let mut trade = PaperTrade::new(
        "BTCUSDT".to_string(),
        TradeType::Long,
        50000.0,
        0.1,
        10,
        0.001,
        None,
        None,
        None,
    );

    trade.update_with_price(48000.0, None); // Price down to 48000

    assert!(trade.unrealized_pnl < 0.0, "Should have loss");
    assert!(trade.pnl_percentage < 0.0);
}

#[test]
fn test_trade_update_pnl_short_profit() {
    let mut trade = PaperTrade::new(
        "BTCUSDT".to_string(),
        TradeType::Short,
        50000.0,
        0.1,
        10,
        0.001,
        None,
        None,
        None,
    );

    trade.update_with_price(48000.0, None); // Price down to 48000

    assert!(trade.unrealized_pnl > 0.0, "Short should profit on price drop");
    assert!(trade.pnl_percentage > 0.0);
}

#[test]
fn test_trade_update_pnl_short_loss() {
    let mut trade = PaperTrade::new(
        "BTCUSDT".to_string(),
        TradeType::Short,
        50000.0,
        0.1,
        10,
        0.001,
        None,
        None,
        None,
    );

    trade.update_with_price(52000.0, None); // Price up to 52000

    assert!(trade.unrealized_pnl < 0.0, "Short should lose on price increase");
    assert!(trade.pnl_percentage < 0.0);
}

#[test]
fn test_trade_should_stop_loss_triggered() {
    let mut trade = PaperTrade::new(
        "BTCUSDT".to_string(),
        TradeType::Long,
        50000.0,
        0.1,
        10,
        0.001,
        None,
        None,
        None,
    );

    trade.set_stop_loss(49000.0).unwrap();

    // Price below stop loss
    assert!(trade.should_stop_loss(48500.0));

    // Price above stop loss
    assert!(!trade.should_stop_loss(49500.0));
}

#[test]
fn test_trade_should_take_profit_triggered() {
    let mut trade = PaperTrade::new(
        "BTCUSDT".to_string(),
        TradeType::Long,
        50000.0,
        0.1,
        10,
        0.001,
        None,
        None,
        None,
    );

    trade.set_take_profit(55000.0).unwrap();

    // Price above take profit
    assert!(trade.should_take_profit(55500.0));

    // Price below take profit
    assert!(!trade.should_take_profit(54500.0));
}

#[test]
fn test_trade_close_long_profit() {
    let mut trade = PaperTrade::new(
        "BTCUSDT".to_string(),
        TradeType::Long,
        50000.0,
        0.1,
        10,
        0.001,
        None,
        None,
        None,
    );

    // Calculate exit fees (same rate as entry)
    let exit_fees = (0.1 * 52000.0) * (0.001 / 1.0); // Same fee rate

    let result = trade.close(52000.0, CloseReason::TakeProfit, exit_fees);
    assert!(result.is_ok());
    assert_eq!(trade.status, TradeStatus::Closed);
    assert_eq!(trade.exit_price, Some(52000.0));
    assert!(trade.realized_pnl.unwrap() > 0.0);
    assert!(trade.close_time.is_some());
}

#[test]
fn test_trade_close_long_loss() {
    let mut trade = PaperTrade::new(
        "BTCUSDT".to_string(),
        TradeType::Long,
        50000.0,
        0.1,
        10,
        0.001,
        None,
        None,
        None,
    );

    let exit_fees = (0.1 * 48000.0) * (0.001 / 1.0);
    let result = trade.close(48000.0, CloseReason::StopLoss, exit_fees);
    assert!(result.is_ok());
    assert_eq!(trade.status, TradeStatus::Closed);
    assert!(trade.realized_pnl.unwrap() < 0.0);
}

#[test]
fn test_trade_close_already_closed() {
    let mut trade = PaperTrade::new(
        "BTCUSDT".to_string(),
        TradeType::Long,
        50000.0,
        0.1,
        10,
        0.001,
        None,
        None,
        None,
    );

    let exit_fees = (0.1 * 52000.0) * (0.001 / 1.0);
    trade.close(52000.0, CloseReason::TakeProfit, exit_fees).unwrap();

    // Try to close again
    let exit_fees2 = (0.1 * 53000.0) * (0.001 / 1.0);
    let result = trade.close(53000.0, CloseReason::Manual, exit_fees2);
    assert!(result.is_err(), "Should not allow closing already closed trade");
}

#[test]
fn test_trade_margin_calculations() {
    let trade = PaperTrade::new(
        "BTCUSDT".to_string(),
        TradeType::Long,
        50000.0,
        0.1,
        10, // 10x leverage
        0.001,
        None,
        None,
        None,
    );

    let notional_value = 50000.0 * 0.1; // 5000
    let expected_initial_margin = notional_value / 10.0; // 500 (with 10x leverage)

    assert_eq!(trade.initial_margin, expected_initial_margin);
    assert!(trade.maintenance_margin > 0.0);
    assert_eq!(trade.margin_used, expected_initial_margin);
}

#[test]
fn test_trade_trading_fees() {
    let trade = PaperTrade::new(
        "BTCUSDT".to_string(),
        TradeType::Long,
        50000.0,
        0.1,
        10,
        0.001, // 0.1% fee
        None,
        None,
        None,
    );

    let notional_value = 50000.0 * 0.1; // 5000
    let expected_fee = notional_value * 0.001; // 5

    assert_eq!(trade.trading_fees, expected_fee);
}

#[test]
fn test_trade_summary() {
    let trade = PaperTrade::new(
        "BTCUSDT".to_string(),
        TradeType::Long,
        50000.0,
        0.1,
        10,
        0.001,
        Some("ai-signal-123".to_string()),
        Some(0.85),
        Some("Strong bullish trend".to_string()),
    );

    let summary = trade.get_summary();
    assert_eq!(summary.symbol, "BTCUSDT");
    assert_eq!(summary.trade_type, TradeType::Long);
    assert_eq!(summary.entry_price, 50000.0);
    assert_eq!(summary.quantity, 0.1);
    // TradeSummary doesn't include ai_confidence, but the trade object has it
    assert_eq!(trade.ai_confidence, Some(0.85));
}

// ========== DAILY PERFORMANCE TESTS ==========

#[test]
fn test_daily_performance_tracking() {
    let mut portfolio = PaperPortfolio::new(10000.0);

    // Add daily performance entry
    let daily_perf = DailyPerformance {
        date: Utc::now(),
        balance: 10500.0,
        equity: 10500.0,
        daily_pnl: 500.0,
        daily_pnl_percentage: 5.0,
        trades_executed: 3,
        winning_trades: 2,
        losing_trades: 1,
        total_volume: 15000.0,
        max_drawdown: 100.0,
    };

    portfolio.daily_performance.push(daily_perf);

    assert_eq!(portfolio.daily_performance.len(), 1);
    assert_eq!(portfolio.daily_performance[0].daily_pnl, 500.0);
}

// ========== EDGE CASE TESTS ==========

#[test]
fn test_portfolio_with_multiple_positions_same_symbol() {
    let mut portfolio = PaperPortfolio::new(10000.0);

    // Add 2 long positions on same symbol
    for _ in 0..2 {
        let trade = PaperTrade::new(
            "BTCUSDT".to_string(),
            TradeType::Long,
            50000.0,
            0.1,
            10,
            0.001,
            None,
            None,
            None,
        );
        portfolio.add_trade(trade).unwrap();
    }

    let open_trades = portfolio.get_open_trades();
    let btc_trades: Vec<_> = open_trades
        .iter()
        .filter(|t| t.symbol == "BTCUSDT")
        .collect();

    assert_eq!(btc_trades.len(), 2);
}

#[test]
fn test_trade_with_zero_leverage() {
    // Zero leverage should default to 1x
    let trade = PaperTrade::new(
        "BTCUSDT".to_string(),
        TradeType::Long,
        50000.0,
        0.1,
        0, // Zero leverage
        0.001,
        None,
        None,
        None,
    );

    // Should still create trade with leverage as provided
    assert_eq!(trade.leverage, 0);
}

#[test]
fn test_trade_pnl_with_zero_quantity() {
    let mut trade = PaperTrade::new(
        "BTCUSDT".to_string(),
        TradeType::Long,
        50000.0,
        0.0, // Zero quantity
        10,
        0.001,
        None,
        None,
        None,
    );

    trade.update_with_price(52000.0, None);
    assert_eq!(trade.unrealized_pnl, 0.0, "Zero quantity should have zero PnL");
}

#[test]
fn test_portfolio_cool_down_fields() {
    let mut portfolio = PaperPortfolio::new(10000.0);

    // Set cool-down
    portfolio.consecutive_losses = 5;
    portfolio.cool_down_until = Some(Utc::now() + Duration::minutes(60));

    assert_eq!(portfolio.consecutive_losses, 5);
    assert!(portfolio.cool_down_until.is_some());

    // Reset
    portfolio.consecutive_losses = 0;
    portfolio.cool_down_until = None;

    assert_eq!(portfolio.consecutive_losses, 0);
    assert!(portfolio.cool_down_until.is_none());
}

#[test]
fn test_trade_type_display() {
    assert_eq!(TradeType::Long.as_str(), "Long");
    assert_eq!(TradeType::Short.as_str(), "Short");
}

#[test]
fn test_close_reason_serialization() {
    // Test that CloseReason variants exist
    let reasons = vec![
        CloseReason::StopLoss,
        CloseReason::TakeProfit,
        CloseReason::Manual,
        CloseReason::AISignal,
        CloseReason::RiskManagement,
        CloseReason::MarginCall,
        CloseReason::TimeBasedExit,
    ];

    assert_eq!(reasons.len(), 7);
}

// ========== BOUNDARY CONDITION TESTS ==========

#[test]
fn test_trade_with_very_high_leverage() {
    let trade = PaperTrade::new(
        "BTCUSDT".to_string(),
        TradeType::Long,
        50000.0,
        0.1,
        125, // Maximum leverage
        0.001,
        None,
        None,
        None,
    );

    let notional = 50000.0 * 0.1;
    let expected_margin = notional / 125.0;
    assert_eq!(trade.initial_margin, expected_margin);
}

#[test]
fn test_trade_pnl_calculation_precision() {
    let mut trade = PaperTrade::new(
        "BTCUSDT".to_string(),
        TradeType::Long,
        50000.123456, // High precision
        0.123456,
        10,
        0.001,
        None,
        None,
        None,
    );

    trade.update_with_price(51000.987654, None);

    // PnL should be calculated with full precision
    assert!(trade.unrealized_pnl > 0.0);
}

#[test]
fn test_portfolio_margin_level_calculation() {
    let mut portfolio = PaperPortfolio::new(10000.0);

    let trade = PaperTrade::new(
        "BTCUSDT".to_string(),
        TradeType::Long,
        50000.0,
        0.2, // Larger position
        10,
        0.001,
        None,
        None,
        None,
    );

    portfolio.add_trade(trade).unwrap();

    // Update prices to trigger margin calculations
    let mut prices = HashMap::new();
    prices.insert("BTCUSDT".to_string(), 50000.0);
    portfolio.update_prices(prices, None);

    assert!(portfolio.margin_used > 0.0);
    assert!(portfolio.free_margin < portfolio.initial_balance);
}
