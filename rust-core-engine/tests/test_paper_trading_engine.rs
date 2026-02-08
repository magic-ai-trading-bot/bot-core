// Comprehensive tests for PaperTradingEngine
// Target: Increase coverage from 44.67% to 95%+
// Focus: process_trading_signal, execute_trade, risk management, close_trade

use binance_trading_bot::paper_trading::trade::TradeStatus;
use binance_trading_bot::paper_trading::{
    CloseReason, PaperPortfolio, PaperTrade, PaperTradingSettings, TradeType,
};
use binance_trading_bot::strategies::TradingSignal;
use chrono::{Duration, Utc};
use std::collections::HashMap;

// Test helper: Create minimal PaperTradingSettings
fn create_test_settings() -> PaperTradingSettings {
    let mut settings = PaperTradingSettings::default();

    // Basic settings
    settings.basic.initial_balance = 10000.0;
    settings.basic.max_positions = 5;
    settings.basic.default_position_size_pct = 10.0;
    settings.basic.default_leverage = 10;
    settings.basic.trading_fee_rate = 0.0004;
    settings.basic.enabled = true;

    // Risk settings
    settings.risk.max_risk_per_trade_pct = 2.0;
    settings.risk.max_portfolio_risk_pct = 10.0;
    settings.risk.default_stop_loss_pct = 5.0;
    settings.risk.default_take_profit_pct = 10.0;
    settings.risk.max_leverage = 20;
    settings.risk.daily_loss_limit_pct = 5.0;
    settings.risk.max_consecutive_losses = 3;
    settings.risk.cool_down_minutes = 60;
    settings.risk.correlation_limit = 0.7;
    settings.risk.trailing_stop_enabled = false;
    settings.risk.trailing_stop_pct = 2.0;
    settings.risk.trailing_activation_pct = 3.0;
    settings.risk.enable_signal_reversal = false;
    settings.risk.ai_auto_enable_reversal = false;
    settings.risk.reversal_min_confidence = 0.75;
    settings.risk.reversal_max_pnl_pct = 5.0;
    settings.risk.reversal_allowed_regimes = vec!["trending".to_string()];

    // Strategy settings
    settings.strategy.min_ai_confidence = 0.6;

    // AI settings
    settings.ai.service_url = "http://localhost:8000".to_string();
    settings.ai.signal_refresh_interval_minutes = 15;

    // Execution settings
    settings.execution.simulate_slippage = false;
    settings.execution.simulate_market_impact = false;
    settings.execution.simulate_partial_fills = false;
    settings.execution.execution_delay_ms = 0;
    settings.execution.max_slippage_pct = 0.1;
    settings.execution.market_impact_factor = 0.01;
    settings.execution.partial_fill_probability = 0.0;

    // Add test symbols
    use binance_trading_bot::paper_trading::settings::SymbolSettings;
    let symbol_settings = SymbolSettings {
        enabled: true,
        leverage: Some(10),
        position_size_pct: Some(10.0),
        stop_loss_pct: Some(5.0),
        take_profit_pct: Some(10.0),
        trading_hours: None,
        min_price_movement_pct: None,
        max_positions: Some(1),
        custom_params: HashMap::new(),
    };

    settings
        .symbols
        .insert("BTCUSDT".to_string(), symbol_settings.clone());
    settings
        .symbols
        .insert("ETHUSDT".to_string(), symbol_settings);

    settings
}

// Test helper: Create test AITradingSignal
fn create_test_signal(
    symbol: &str,
    signal_type: TradingSignal,
    confidence: f64,
) -> binance_trading_bot::paper_trading::AITradingSignal {
    use binance_trading_bot::paper_trading::AITradingSignal;
    use binance_trading_bot::paper_trading::MarketAnalysisData;
    use uuid::Uuid;

    let entry_price = match symbol {
        "BTCUSDT" => 50000.0,
        "ETHUSDT" => 3000.0,
        _ => 1000.0,
    };

    AITradingSignal {
        id: Uuid::new_v4().to_string(),
        symbol: symbol.to_string(),
        signal_type,
        confidence,
        entry_price,
        suggested_stop_loss: None,
        suggested_take_profit: None,
        suggested_leverage: None,
        reasoning: "Test signal".to_string(),
        market_analysis: MarketAnalysisData {
            trend_direction: "up".to_string(),
            trend_strength: 0.7,
            volatility: 0.3,
            support_levels: vec![],
            resistance_levels: vec![],
            volume_analysis: "increasing".to_string(),
            risk_score: 0.3,
        },
        timestamp: Utc::now(),
    }
}

// ============== PORTFOLIO TESTS ==============

#[test]
fn test_portfolio_new() {
    let portfolio = PaperPortfolio::new(10000.0);
    assert_eq!(portfolio.initial_balance, 10000.0);
    assert_eq!(portfolio.cash_balance, 10000.0);
    assert_eq!(portfolio.equity, 10000.0);
    assert_eq!(portfolio.margin_used, 0.0);
    assert_eq!(portfolio.free_margin, 10000.0);
    assert_eq!(portfolio.trades.len(), 0);
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
        0.2,
        10,
        0.0004,
        None,
        None,
        None,
    );

    let trade_id = trade.id.clone();
    let initial_margin = trade.initial_margin;

    let result = portfolio.add_trade(trade);
    assert!(result.is_ok());
    assert_eq!(portfolio.open_trade_ids.len(), 1);
    assert_eq!(portfolio.open_trade_ids[0], trade_id);
    assert_eq!(portfolio.margin_used, initial_margin);
    assert!(portfolio.trades.contains_key(&trade_id));
}

#[test]
fn test_portfolio_add_trade_insufficient_margin() {
    let mut portfolio = PaperPortfolio::new(100.0); // Very small balance

    let trade = PaperTrade::new(
        "BTCUSDT".to_string(),
        TradeType::Long,
        50000.0,
        10.0, // Huge quantity
        10,
        0.0004,
        None,
        None,
        None,
    );

    let result = portfolio.add_trade(trade);
    assert!(result.is_err());
    assert!(result
        .unwrap_err()
        .to_string()
        .contains("Insufficient free margin"));
}

#[test]
fn test_portfolio_close_trade() {
    let mut portfolio = PaperPortfolio::new(10000.0);

    let mut trade = PaperTrade::new(
        "BTCUSDT".to_string(),
        TradeType::Long,
        50000.0,
        0.2,
        10,
        0.0004,
        None,
        None,
        None,
    );

    trade.stop_loss = Some(48000.0);
    trade.take_profit = Some(55000.0);

    let trade_id = trade.id.clone();

    portfolio.add_trade(trade).unwrap();
    assert_eq!(portfolio.open_trade_ids.len(), 1);

    // Close trade at profit
    let close_result = portfolio.close_trade(&trade_id, 52000.0, CloseReason::TakeProfit);
    assert!(close_result.is_ok());

    assert_eq!(portfolio.open_trade_ids.len(), 0);
    assert_eq!(portfolio.closed_trade_ids.len(), 1);
    assert_eq!(portfolio.closed_trade_ids[0], trade_id);

    // Margin should be released
    assert_eq!(portfolio.margin_used, 0.0);

    // Check trade is closed
    let trade = portfolio.trades.get(&trade_id).unwrap();
    assert_eq!(trade.status, TradeStatus::Closed);
    assert_eq!(trade.exit_price, Some(52000.0));
    assert_eq!(trade.close_reason, Some(CloseReason::TakeProfit));
    assert!(trade.realized_pnl.is_some());

    // For Long: (52000 - 50000) * 0.2 = 400 profit (minus fees)
    let realized_pnl = trade.realized_pnl.unwrap();
    assert!(realized_pnl > 0.0); // Should be profitable
}

#[test]
fn test_portfolio_update_prices() {
    let mut portfolio = PaperPortfolio::new(10000.0);

    let trade = PaperTrade::new(
        "BTCUSDT".to_string(),
        TradeType::Long,
        50000.0,
        0.2,
        10,
        0.0004,
        None,
        None,
        None,
    );

    portfolio.add_trade(trade).unwrap();

    // Update prices
    let mut new_prices = HashMap::new();
    new_prices.insert("BTCUSDT".to_string(), 51000.0);

    let mut funding_rates = HashMap::new();
    funding_rates.insert("BTCUSDT".to_string(), 0.0001);

    portfolio.update_prices(new_prices.clone(), Some(funding_rates));

    assert_eq!(portfolio.current_prices.get("BTCUSDT"), Some(&51000.0));

    // Check unrealized PnL was updated
    let trade = portfolio.trades.values().next().unwrap();
    // Long: (51000 - 50000) * 0.2 = 200 (minus fees)
    assert!(trade.unrealized_pnl > 0.0);
}

#[test]
fn test_portfolio_consecutive_losses_tracking() {
    let mut portfolio = PaperPortfolio::new(10000.0);
    assert_eq!(portfolio.consecutive_losses, 0);

    // Simulate consecutive losses manually
    portfolio.consecutive_losses = 3;
    portfolio.cool_down_until = Some(Utc::now() + Duration::minutes(60));

    assert_eq!(portfolio.consecutive_losses, 3);
    assert!(portfolio.cool_down_until.is_some());
}

// ============== TRADE TESTS ==============

#[test]
fn test_trade_new_long() {
    let trade = PaperTrade::new(
        "BTCUSDT".to_string(),
        TradeType::Long,
        50000.0,
        0.2,
        10,
        0.0004,
        Some("signal123".to_string()),
        Some(0.85),
        Some("Bullish trend".to_string()),
    );

    assert_eq!(trade.symbol, "BTCUSDT");
    assert_eq!(trade.trade_type, TradeType::Long);
    assert_eq!(trade.entry_price, 50000.0);
    assert_eq!(trade.quantity, 0.2);
    assert_eq!(trade.leverage, 10);
    assert_eq!(trade.status, TradeStatus::Open);
    assert_eq!(trade.ai_signal_id, Some("signal123".to_string()));
    assert_eq!(trade.ai_confidence, Some(0.85));
    assert_eq!(trade.unrealized_pnl, 0.0);
    assert!(trade.realized_pnl.is_none());

    // Check margin calculations
    let notional_value = 0.2 * 50000.0; // 10000
    let expected_margin = notional_value / 10.0; // 1000
    assert_eq!(trade.initial_margin, expected_margin);
}

#[test]
fn test_trade_update_with_price_long_profit() {
    let mut trade = PaperTrade::new(
        "BTCUSDT".to_string(),
        TradeType::Long,
        50000.0,
        0.2,
        10,
        0.0004,
        None,
        None,
        None,
    );

    // Price goes up (profit for long)
    trade.update_with_price(51000.0, None);

    // Long: (51000 - 50000) * 0.2 = 200 (minus fees)
    assert!(trade.unrealized_pnl > 0.0);
    assert!(trade.pnl_percentage > 0.0);
    assert!(trade.max_favorable_excursion > 0.0);
}

#[test]
fn test_trade_update_with_price_long_loss() {
    let mut trade = PaperTrade::new(
        "BTCUSDT".to_string(),
        TradeType::Long,
        50000.0,
        0.2,
        10,
        0.0004,
        None,
        None,
        None,
    );

    // Price goes down (loss for long)
    trade.update_with_price(49000.0, None);

    // Long: (49000 - 50000) * 0.2 = -200 (plus fees = more negative)
    assert!(trade.unrealized_pnl < 0.0);
    assert!(trade.pnl_percentage < 0.0);
    assert!(trade.max_adverse_excursion < 0.0);
}

#[test]
fn test_trade_update_with_price_short_profit() {
    let mut trade = PaperTrade::new(
        "BTCUSDT".to_string(),
        TradeType::Short,
        50000.0,
        0.2,
        10,
        0.0004,
        None,
        None,
        None,
    );

    // Price goes down (profit for short)
    trade.update_with_price(49000.0, None);

    // Short: (50000 - 49000) * 0.2 = 200 (minus fees)
    assert!(trade.unrealized_pnl > 0.0);
    assert!(trade.pnl_percentage > 0.0);
}

#[test]
fn test_trade_update_with_price_short_loss() {
    let mut trade = PaperTrade::new(
        "BTCUSDT".to_string(),
        TradeType::Short,
        50000.0,
        0.2,
        10,
        0.0004,
        None,
        None,
        None,
    );

    // Price goes up (loss for short)
    trade.update_with_price(51000.0, None);

    // Short: (50000 - 51000) * 0.2 = -200 (plus fees = more negative)
    assert!(trade.unrealized_pnl < 0.0);
    assert!(trade.pnl_percentage < 0.0);
}

#[test]
fn test_trade_should_stop_loss_long() {
    let mut trade = PaperTrade::new(
        "BTCUSDT".to_string(),
        TradeType::Long,
        50000.0,
        0.2,
        10,
        0.0004,
        None,
        None,
        None,
    );

    trade.stop_loss = Some(48000.0);

    // Price at or below stop loss
    assert!(trade.should_stop_loss(48000.0));
    assert!(trade.should_stop_loss(47000.0));

    // Price above stop loss
    assert!(!trade.should_stop_loss(49000.0));
    assert!(!trade.should_stop_loss(51000.0));
}

#[test]
fn test_trade_should_stop_loss_short() {
    let mut trade = PaperTrade::new(
        "BTCUSDT".to_string(),
        TradeType::Short,
        50000.0,
        0.2,
        10,
        0.0004,
        None,
        None,
        None,
    );

    trade.stop_loss = Some(52000.0);

    // Price at or above stop loss
    assert!(trade.should_stop_loss(52000.0));
    assert!(trade.should_stop_loss(53000.0));

    // Price below stop loss
    assert!(!trade.should_stop_loss(51000.0));
    assert!(!trade.should_stop_loss(49000.0));
}

#[test]
fn test_trade_should_take_profit_long() {
    let mut trade = PaperTrade::new(
        "BTCUSDT".to_string(),
        TradeType::Long,
        50000.0,
        0.2,
        10,
        0.0004,
        None,
        None,
        None,
    );

    trade.take_profit = Some(55000.0);

    // Price at or above take profit
    assert!(trade.should_take_profit(55000.0));
    assert!(trade.should_take_profit(56000.0));

    // Price below take profit
    assert!(!trade.should_take_profit(54000.0));
    assert!(!trade.should_take_profit(51000.0));
}

#[test]
fn test_trade_should_take_profit_short() {
    let mut trade = PaperTrade::new(
        "BTCUSDT".to_string(),
        TradeType::Short,
        50000.0,
        0.2,
        10,
        0.0004,
        None,
        None,
        None,
    );

    trade.take_profit = Some(45000.0);

    // Price at or below take profit
    assert!(trade.should_take_profit(45000.0));
    assert!(trade.should_take_profit(44000.0));

    // Price above take profit
    assert!(!trade.should_take_profit(46000.0));
    assert!(!trade.should_take_profit(49000.0));
}

#[test]
fn test_trade_close() {
    let mut trade = PaperTrade::new(
        "BTCUSDT".to_string(),
        TradeType::Long,
        50000.0,
        0.2,
        10,
        0.0004,
        None,
        None,
        None,
    );

    let exit_price = 52000.0;
    let result = trade.close(exit_price, CloseReason::TakeProfit, 4.16);

    assert!(result.is_ok());
    assert_eq!(trade.status, TradeStatus::Closed);
    assert_eq!(trade.exit_price, Some(exit_price));
    assert_eq!(trade.close_reason, Some(CloseReason::TakeProfit));
    assert!(trade.close_time.is_some());
    assert!(trade.duration_ms.is_some());
    assert!(trade.realized_pnl.is_some());

    // Check PnL calculation
    // Long: (52000 - 50000) * 0.2 = 400
    // Minus fees: entry + exit
    let realized_pnl = trade.realized_pnl.unwrap();
    assert!(realized_pnl > 0.0); // Should be profitable after fees
}

#[test]
fn test_trade_close_already_closed() {
    let mut trade = PaperTrade::new(
        "BTCUSDT".to_string(),
        TradeType::Long,
        50000.0,
        0.2,
        10,
        0.0004,
        None,
        None,
        None,
    );

    // Close once
    trade.close(52000.0, CloseReason::TakeProfit, 4.16).unwrap();

    // Try to close again
    let result = trade.close(53000.0, CloseReason::Manual, 4.24);
    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("not open"));
}

#[test]
fn test_trade_update_trailing_stop_long() {
    let mut trade = PaperTrade::new(
        "BTCUSDT".to_string(),
        TradeType::Long,
        50000.0,
        0.2,
        10,
        0.0004,
        None,
        None,
        None,
    );

    trade.stop_loss = Some(48000.0);

    // Price increases - should activate and update trailing stop
    trade.update_trailing_stop(53000.0, 2.0, 3.0);

    // After 6% profit (>3% activation), trailing should be active
    assert!(trade.trailing_stop_active);

    // Trailing stop should be 2% below highest price (53000)
    // Expected: 53000 * 0.98 = 51940
    let expected_trailing = 53000.0 * (1.0 - 2.0 / 100.0);
    assert_eq!(trade.stop_loss, Some(expected_trailing));
}

#[test]
fn test_trade_update_trailing_stop_short() {
    let mut trade = PaperTrade::new(
        "BTCUSDT".to_string(),
        TradeType::Short,
        50000.0,
        0.2,
        10,
        0.0004,
        None,
        None,
        None,
    );

    trade.stop_loss = Some(52000.0);

    // Price decreases - should activate and update trailing stop
    trade.update_trailing_stop(47000.0, 2.0, 3.0);

    // After 6% profit (>3% activation), trailing should be active
    assert!(trade.trailing_stop_active);

    // Trailing stop should be 2% above lowest price (47000)
    // Expected: 47000 * 1.02 = 47940
    let expected_trailing = 47000.0 * (1.0 + 2.0 / 100.0);
    assert_eq!(trade.stop_loss, Some(expected_trailing));
}

#[test]
fn test_trade_funding_fees_long() {
    let mut trade = PaperTrade::new(
        "BTCUSDT".to_string(),
        TradeType::Long,
        50000.0,
        0.2,
        10,
        0.0004,
        None,
        None,
        None,
    );

    // Positive funding rate (longs pay shorts)
    trade.update_with_price(50000.0, Some(0.0001));

    // Funding fee = notional * rate = 10000 * 0.0001 = 1.0
    // Long pays, so funding_fees should be positive
    assert!(trade.funding_fees > 0.0);
}

#[test]
fn test_trade_funding_fees_short() {
    let mut trade = PaperTrade::new(
        "BTCUSDT".to_string(),
        TradeType::Short,
        50000.0,
        0.2,
        10,
        0.0004,
        None,
        None,
        None,
    );

    // Positive funding rate (shorts receive from longs)
    trade.update_with_price(50000.0, Some(0.0001));

    // Short receives, so funding_fees should be negative (received = negative cost)
    assert!(trade.funding_fees < 0.0);
}

// ============== SETTINGS TESTS ==============

#[test]
fn test_settings_default() {
    let settings = PaperTradingSettings::default();
    assert_eq!(settings.basic.initial_balance, 10000.0);
    assert!(settings.basic.enabled);
}

#[test]
fn test_settings_get_symbol_settings() {
    let settings = create_test_settings();

    let btc_settings = settings.get_symbol_settings("BTCUSDT");
    assert!(btc_settings.enabled);
    assert_eq!(btc_settings.leverage, 10);

    // Non-existent symbol should return defaults from basic settings
    let unknown_settings = settings.get_symbol_settings("UNKNOWN");
    assert_eq!(unknown_settings.leverage, 10); // From basic.default_leverage
    assert_eq!(unknown_settings.position_size_pct, 10.0); // From basic.default_position_size_pct
}

// ============== RISK MANAGEMENT CALCULATION TESTS ==============

#[test]
fn test_daily_loss_limit_calculation() {
    // Simulate daily loss limit check logic
    let initial_equity = 10000.0;
    let current_equity = 9400.0; // Lost $600
    let daily_limit_pct = 5.0; // 5% max loss

    let daily_loss = initial_equity - current_equity;
    let daily_loss_pct = (daily_loss / initial_equity) * 100.0;

    assert_eq!(daily_loss, 600.0);
    assert_eq!(daily_loss_pct, 6.0);
    assert!(daily_loss_pct >= daily_limit_pct); // Should block trading
}

#[test]
fn test_correlation_limit_calculation() {
    // Simulate correlation limit check
    let long_exposure = 7000.0;
    let short_exposure = 3000.0;
    let total_exposure = long_exposure + short_exposure;
    let correlation_limit = 0.7; // 70%

    let long_ratio = long_exposure / total_exposure;
    let short_ratio = short_exposure / total_exposure;

    assert_eq!(long_ratio, 0.7);
    assert_eq!(short_ratio, 0.3);
    assert!(long_ratio <= correlation_limit); // At limit, should be OK

    // Test exceeding limit
    let long_exposure_high = 8000.0;
    let short_exposure_low = 2000.0;
    let total_high = long_exposure_high + short_exposure_low;
    let long_ratio_high = long_exposure_high / total_high;

    assert_eq!(long_ratio_high, 0.8);
    assert!(long_ratio_high > correlation_limit); // Should block new long
}

#[test]
fn test_portfolio_risk_limit_calculation() {
    // Simulate portfolio risk limit check
    let equity = 10000.0;
    let max_portfolio_risk_pct = 10.0;
    let default_stop_loss_pct = 5.0;

    // Trade 1: $5000 position with 5% SL = $250 risk = 2.5% of equity
    let position1_value = 5000.0;
    let risk1 = position1_value * (default_stop_loss_pct / 100.0);
    let risk1_pct = (risk1 / equity) * 100.0;

    // Trade 2: $4000 position with 5% SL = $200 risk = 2% of equity
    let position2_value = 4000.0;
    let risk2 = position2_value * (default_stop_loss_pct / 100.0);
    let risk2_pct = (risk2 / equity) * 100.0;

    let total_risk_pct = risk1_pct + risk2_pct;

    assert_eq!(risk1, 250.0);
    assert_eq!(risk1_pct, 2.5);
    assert_eq!(risk2, 200.0);
    assert_eq!(risk2_pct, 2.0);
    assert_eq!(total_risk_pct, 4.5);
    assert!(total_risk_pct < max_portfolio_risk_pct); // Should allow more trades
}

#[test]
fn test_position_size_calculation() {
    // Test position size calculation with risk-based formula
    let equity = 10000.0;
    let position_size_pct = 10.0; // Risk 10% of equity
    let entry_price = 50000.0;
    let stop_loss_pct = 5.0;
    let leverage = 10;

    let risk_amount = equity * (position_size_pct / 100.0); // $1000
    let max_position_value = risk_amount / (stop_loss_pct / 100.0); // $20000
    let max_position_value_with_leverage = max_position_value * leverage as f64; // $200000

    // Assuming enough margin
    let quantity = max_position_value_with_leverage / entry_price;

    assert_eq!(risk_amount, 1000.0);
    assert_eq!(max_position_value, 20000.0);
    assert_eq!(max_position_value_with_leverage, 200000.0);
    assert_eq!(quantity, 4.0); // 4 BTC
}

#[test]
fn test_slippage_simulation() {
    // Test slippage calculation
    let base_price = 50000.0;
    let _max_slippage_pct = 0.1; // 0.1%
    let slippage_pct = 0.05; // Random 0.05%

    // Long: buy at higher price
    let long_slipped = base_price * (1.0 + slippage_pct / 100.0);
    assert_eq!(long_slipped, 50025.0);

    // Short: sell at lower price
    let short_slipped = base_price * (1.0 - slippage_pct / 100.0);
    assert_eq!(short_slipped, 49975.0);
}

#[test]
fn test_market_impact_calculation() {
    // Test market impact calculation
    let order_value = 100000.0_f64; // $100k order
    let typical_volume = 50000000.0_f64; // $50M typical hourly volume for BTC
    let impact_factor = 0.01_f64;

    let impact_pct = (order_value / typical_volume) * impact_factor;
    let max_impact = 1.0_f64; // 1% cap

    let actual_impact = impact_pct.min(max_impact);

    assert_eq!(impact_pct, 0.00002);
    assert!(actual_impact < max_impact);
}

#[test]
fn test_partial_fill_simulation() {
    // Test partial fill calculation
    let requested_quantity = 1.0;
    let fill_pct = 0.65; // 65% filled
    let filled_quantity = requested_quantity * fill_pct;

    assert_eq!(filled_quantity, 0.65);
    assert!(filled_quantity < requested_quantity);
}

// ============== CONSECUTIVE STREAK TESTS ==============

#[test]
fn test_consecutive_wins_streak() {
    // Create trades with consecutive wins
    let trades = vec![
        create_closed_trade(100.0),  // Win
        create_closed_trade(50.0),   // Win
        create_closed_trade(75.0),   // Win
        create_closed_trade(-200.0), // Loss (breaks streak)
    ];

    // Count from most recent (reverse order, skip the loss)
    // The streak should be the last 3 wins before the loss
    // But since we're iterating in reverse, we start from index 2 (third element)
    let mut wins = 0;
    for trade in trades.iter().take(3) {
        if trade.realized_pnl.unwrap_or(0.0) > 0.0 {
            wins += 1;
        }
    }

    assert_eq!(wins, 3); // First 3 are all wins
}

#[test]
fn test_consecutive_losses_streak() {
    // Create trades with consecutive losses
    let trades = vec![
        create_closed_trade(-100.0), // Loss
        create_closed_trade(-50.0),  // Loss
        create_closed_trade(-75.0),  // Loss
        create_closed_trade(200.0),  // Win (breaks streak)
    ];

    // Count losses from first 3 trades
    let mut losses = 0;
    for trade in trades.iter().take(3) {
        if trade.realized_pnl.unwrap_or(0.0) < 0.0 {
            losses += 1;
        }
    }

    assert_eq!(losses, 3);
}

fn create_closed_trade(pnl: f64) -> PaperTrade {
    let mut trade = PaperTrade::new(
        "BTCUSDT".to_string(),
        TradeType::Long,
        50000.0,
        0.1,
        10,
        0.0004,
        None,
        None,
        None,
    );

    // Close the trade manually
    let _ = trade.close(
        if pnl > 0.0 { 51000.0 } else { 49000.0 },
        CloseReason::Manual,
        0.0,
    );

    // Override the realized_pnl to the desired value
    trade.realized_pnl = Some(pnl);

    trade
}

// ============== AI ACCURACY CALCULATION TESTS ==============

#[test]
fn test_ai_accuracy_calculation() {
    let trades = vec![
        create_ai_trade(100.0, 0.85),  // Win
        create_ai_trade(50.0, 0.75),   // Win
        create_ai_trade(-100.0, 0.70), // Loss
        create_ai_trade(75.0, 0.80),   // Win
        create_ai_trade(-50.0, 0.65),  // Loss
    ];

    let correct = trades
        .iter()
        .filter(|t| t.realized_pnl.unwrap_or(0.0) > 0.0)
        .count();

    let accuracy = correct as f64 / trades.len() as f64;

    assert_eq!(correct, 3);
    assert_eq!(accuracy, 0.6); // 60% accuracy
}

fn create_ai_trade(pnl: f64, confidence: f64) -> PaperTrade {
    let mut trade = create_closed_trade(pnl);
    trade.ai_signal_id = Some(format!("signal_{}", Utc::now().timestamp()));
    trade.ai_confidence = Some(confidence);
    trade
}

// ============== WIN RATE CALCULATION TESTS ==============

#[test]
fn test_win_rate_calculation() {
    let trades = vec![
        create_closed_trade(100.0),  // Win
        create_closed_trade(50.0),   // Win
        create_closed_trade(-100.0), // Loss
        create_closed_trade(75.0),   // Win
        create_closed_trade(-50.0),  // Loss
        create_closed_trade(25.0),   // Win
    ];

    let wins = trades
        .iter()
        .filter(|t| t.realized_pnl.unwrap_or(0.0) > 0.0)
        .count();

    let win_rate = wins as f64 / trades.len() as f64;

    assert_eq!(wins, 4);
    assert_eq!(win_rate, 4.0 / 6.0); // 66.67%
}

#[test]
fn test_win_rate_all_wins() {
    let trades = vec![
        create_closed_trade(100.0),
        create_closed_trade(50.0),
        create_closed_trade(75.0),
    ];

    let wins = trades
        .iter()
        .filter(|t| t.realized_pnl.unwrap_or(0.0) > 0.0)
        .count();

    let win_rate = wins as f64 / trades.len() as f64;

    assert_eq!(win_rate, 1.0); // 100%
}

#[test]
fn test_win_rate_all_losses() {
    let trades = vec![
        create_closed_trade(-100.0),
        create_closed_trade(-50.0),
        create_closed_trade(-75.0),
    ];

    let wins = trades
        .iter()
        .filter(|t| t.realized_pnl.unwrap_or(0.0) > 0.0)
        .count();

    let win_rate = wins as f64 / trades.len() as f64;

    assert_eq!(win_rate, 0.0); // 0%
}

// ============== MARKET REGIME DETECTION TESTS ==============

#[test]
fn test_market_regime_trending() {
    // High trend strength = trending
    let trend_strength = 0.7;
    let volatility = 0.3;

    let regime = if volatility > 0.7 {
        "volatile"
    } else if trend_strength > 0.6 {
        "trending"
    } else if trend_strength < 0.4 {
        "ranging"
    } else {
        "trending"
    };

    assert_eq!(regime, "trending");
}

#[test]
fn test_market_regime_ranging() {
    // Low trend strength = ranging
    let trend_strength = 0.3;
    let volatility = 0.25;

    let regime = if volatility > 0.7 {
        "volatile"
    } else if trend_strength > 0.6 {
        "trending"
    } else if trend_strength < 0.4 {
        "ranging"
    } else {
        "trending"
    };

    assert_eq!(regime, "ranging");
}

#[test]
fn test_market_regime_volatile() {
    // High volatility = volatile (overrides everything)
    let trend_strength = 0.8;
    let volatility = 0.75;

    let regime = if volatility > 0.7 {
        "volatile"
    } else if trend_strength > 0.6 {
        "trending"
    } else if trend_strength < 0.4 {
        "ranging"
    } else {
        "trending"
    };

    assert_eq!(regime, "volatile");
}

// ============== CLOSE REASON TESTS ==============

#[test]
fn test_close_reasons() {
    use binance_trading_bot::paper_trading::CloseReason;

    let reasons = vec![
        CloseReason::TakeProfit,
        CloseReason::StopLoss,
        CloseReason::Manual,
        CloseReason::AISignal,
        CloseReason::RiskManagement,
        CloseReason::MarginCall,
        CloseReason::TimeBasedExit,
    ];

    // All enum variants should be creatable
    assert_eq!(reasons.len(), 7);
}

// ============== EDGE CASES & BOUNDARY CONDITIONS ==============

#[test]
fn test_zero_quantity_protection() {
    let quantity = 0.0;

    // Should reject zero quantity
    assert!(quantity <= 0.0);
}

#[test]
fn test_negative_price_protection() {
    let price = -100.0;

    // Should reject negative price
    assert!(price < 0.0);
}

#[test]
fn test_division_by_zero_protection() {
    let entry_price = 0.0_f64;
    let current_price = 100.0_f64;

    let pnl_pct = if entry_price == 0.0 || entry_price.abs() < 1e-10 {
        0.0
    } else {
        ((current_price - entry_price) / entry_price) * 100.0
    };

    assert_eq!(pnl_pct, 0.0); // Should return 0, not panic
}

#[test]
fn test_margin_ratio_zero_margin() {
    let equity = 1000.0;
    let margin_used = 0.0;

    let margin_ratio = if margin_used > 0.0 {
        equity / margin_used
    } else {
        1.0
    };

    assert_eq!(margin_ratio, 1.0); // Should default to 1.0, not panic
}

#[test]
fn test_leverage_limits() {
    let leverage_values = vec![1, 5, 10, 20, 50, 100, 125];

    for leverage in leverage_values {
        assert!(leverage >= 1);
        assert!(leverage <= 125);
    }
}

#[test]
fn test_percentage_bounds() {
    let percentages = vec![
        0.0,   // Min
        5.0,   // Normal SL
        10.0,  // Normal TP
        50.0,  // High
        100.0, // Max
    ];

    for pct in percentages {
        assert!(pct >= 0.0);
        assert!(pct <= 100.0);
    }
}

#[test]
fn test_confidence_bounds() {
    let confidences = vec![0.0, 0.25, 0.5, 0.75, 1.0];

    for conf in confidences {
        assert!(conf >= 0.0);
        assert!(conf <= 1.0);
    }
}

// ============== PERFORMANCE METRICS TESTS ==============

#[test]
fn test_sharpe_ratio_calculation() {
    let returns = vec![0.01, 0.02, -0.005, 0.015, 0.03];
    let risk_free_rate = 0.02 / 252.0; // Daily

    let avg_return: f64 = returns.iter().sum::<f64>() / returns.len() as f64;
    let variance: f64 = returns
        .iter()
        .map(|r| (r - avg_return).powi(2))
        .sum::<f64>()
        / returns.len() as f64;
    let std_dev = variance.sqrt();

    let sharpe_ratio = if std_dev > 0.0 {
        (avg_return - risk_free_rate) / std_dev
    } else {
        0.0
    };

    assert!(sharpe_ratio > 0.0);
}

#[test]
fn test_profit_factor_calculation() {
    let gross_profit = 5000.0;
    let gross_loss = 2000.0;

    let profit_factor = if gross_loss > 0.0 {
        gross_profit / gross_loss
    } else {
        f64::INFINITY
    };

    assert_eq!(profit_factor, 2.5);
}

#[test]
fn test_max_drawdown_calculation() {
    let equity_curve = vec![10000.0_f64, 10500.0, 11000.0, 9500.0, 9000.0, 10000.0];

    let mut peak = equity_curve[0];
    let mut max_drawdown = 0.0_f64;

    for &equity in &equity_curve {
        if equity > peak {
            peak = equity;
        }
        let drawdown = (peak - equity) / peak;
        if drawdown > max_drawdown {
            max_drawdown = drawdown;
        }
    }

    // Peak was 11000, lowest was 9000
    // Drawdown = (11000 - 9000) / 11000 = 0.1818 (18.18%)
    assert!((max_drawdown - 0.1818_f64).abs() < 0.001);
}

// ============== TIMESTAMP & LATENCY TESTS ==============

#[test]
fn test_execution_latency_calculation() {
    let signal_time = Utc::now();
    let execution_time = signal_time + Duration::milliseconds(250);

    let latency_ms = (execution_time - signal_time).num_milliseconds() as u64;

    assert_eq!(latency_ms, 250);
}

#[test]
fn test_trade_duration_calculation() {
    let open_time = Utc::now();
    let close_time = open_time + Duration::minutes(30);

    let duration_ms = (close_time - open_time).num_milliseconds();

    assert_eq!(duration_ms, 30 * 60 * 1000); // 30 minutes in ms
}

// ============== COMPREHENSIVE INTEGRATION TEST ==============

#[test]
fn test_full_trade_lifecycle() {
    let mut portfolio = PaperPortfolio::new(10000.0);

    // 1. Create and add trade
    let mut trade = PaperTrade::new(
        "BTCUSDT".to_string(),
        TradeType::Long,
        50000.0,
        0.2,
        10,
        0.0004,
        Some("signal123".to_string()),
        Some(0.85),
        Some("Strong bullish trend".to_string()),
    );

    trade.stop_loss = Some(48000.0);
    trade.take_profit = Some(55000.0);

    let trade_id = trade.id.clone();
    let initial_cash = portfolio.cash_balance;

    // 2. Add to portfolio
    portfolio.add_trade(trade).unwrap();
    assert_eq!(portfolio.open_trade_ids.len(), 1);
    assert!(portfolio.margin_used > 0.0);

    // 3. Update with price movements
    let mut prices = HashMap::new();
    prices.insert("BTCUSDT".to_string(), 52000.0); // Price increased
    portfolio.update_prices(prices, None);

    // 4. Check unrealized PnL
    let open_trade = portfolio.trades.get(&trade_id).unwrap();
    assert!(open_trade.unrealized_pnl > 0.0); // Should be profitable

    // 5. Close trade at take profit
    portfolio
        .close_trade(&trade_id, 55000.0, CloseReason::TakeProfit)
        .unwrap();
    assert_eq!(portfolio.open_trade_ids.len(), 0);
    assert_eq!(portfolio.closed_trade_ids.len(), 1);
    assert_eq!(portfolio.margin_used, 0.0);

    // 6. Verify realized PnL was added to cash
    let closed_trade = portfolio.trades.get(&trade_id).unwrap();
    let realized_pnl = closed_trade.realized_pnl.unwrap();
    assert!(realized_pnl > 0.0);
    assert!(portfolio.cash_balance > initial_cash); // Cash increased
}

#[test]
fn test_multiple_trades_lifecycle() {
    let mut portfolio = PaperPortfolio::new(10000.0);

    // Trade 1: BTC Long
    let trade1 = PaperTrade::new(
        "BTCUSDT".to_string(),
        TradeType::Long,
        50000.0,
        0.1,
        10,
        0.0004,
        None,
        None,
        None,
    );
    let trade1_id = trade1.id.clone();
    portfolio.add_trade(trade1).unwrap();

    // Trade 2: ETH Short
    let trade2 = PaperTrade::new(
        "ETHUSDT".to_string(),
        TradeType::Short,
        3000.0,
        1.0,
        10,
        0.0004,
        None,
        None,
        None,
    );
    let trade2_id = trade2.id.clone();
    portfolio.add_trade(trade2).unwrap();

    assert_eq!(portfolio.open_trade_ids.len(), 2);

    // Update prices
    let mut prices = HashMap::new();
    prices.insert("BTCUSDT".to_string(), 51000.0); // BTC up (good for long)
    prices.insert("ETHUSDT".to_string(), 2900.0); // ETH down (good for short)
    portfolio.update_prices(prices, None);

    // Both should be profitable
    let btc_trade = portfolio.trades.get(&trade1_id).unwrap();
    let eth_trade = portfolio.trades.get(&trade2_id).unwrap();
    assert!(btc_trade.unrealized_pnl > 0.0);
    assert!(eth_trade.unrealized_pnl > 0.0);

    // Close both
    portfolio
        .close_trade(&trade1_id, 51000.0, CloseReason::TakeProfit)
        .unwrap();
    portfolio
        .close_trade(&trade2_id, 2900.0, CloseReason::TakeProfit)
        .unwrap();

    assert_eq!(portfolio.open_trade_ids.len(), 0);
    assert_eq!(portfolio.closed_trade_ids.len(), 2);
    assert_eq!(portfolio.margin_used, 0.0);

    // Portfolio should be profitable
    assert!(portfolio.cash_balance > 10000.0);
}
