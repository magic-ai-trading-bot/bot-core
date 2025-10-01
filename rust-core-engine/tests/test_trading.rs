mod common;

#[test]
fn test_trading_calculations() {
    // Test position sizing
    let account_balance = 10000.0;
    let risk_percentage = 2.0; // Risk 2% per trade
    let stop_loss_percentage = 5.0; // 5% stop loss

    let risk_amount = account_balance * (risk_percentage / 100.0);
    let position_size = risk_amount / (stop_loss_percentage / 100.0);

    assert_eq!(risk_amount, 200.0);
    assert_eq!(position_size, 4000.0);
}

#[test]
fn test_pnl_calculation() {
    let entry_price = 50000.0;
    let current_price = 51000.0;
    let quantity = 0.1;

    // Long position PnL
    let long_pnl = (current_price - entry_price) * quantity;
    assert_eq!(long_pnl, 100.0);

    // Short position PnL
    let short_pnl = (entry_price - current_price) * quantity;
    assert_eq!(short_pnl, -100.0);
}

#[test]
fn test_leverage_calculation() {
    let position_value = 50000.0;
    let margin_required = 10000.0;

    let leverage = position_value / margin_required;
    assert_eq!(leverage, 5.0);

    // Test margin calculation from leverage
    let desired_leverage = 10.0;
    let required_margin = position_value / desired_leverage;
    assert_eq!(required_margin, 5000.0);
}

#[test]
fn test_fee_calculation() {
    let trade_value = 10000.0;
    let maker_fee = 0.001; // 0.1%
    let taker_fee = 0.001; // 0.1%

    let maker_fee_amount = trade_value * maker_fee;
    let taker_fee_amount = trade_value * taker_fee;

    assert_eq!(maker_fee_amount, 10.0);
    assert_eq!(taker_fee_amount, 10.0);
}

#[test]
fn test_breakeven_calculation() {
    let entry_price = 50000.0;
    let fee_rate = 0.001; // 0.1%

    // For long position
    let long_breakeven = entry_price * (1.0 + 2.0 * fee_rate);
    assert!((long_breakeven - 50100.0_f64).abs() < 0.01);

    // For short position
    let short_breakeven = entry_price * (1.0 - 2.0 * fee_rate);
    assert!((short_breakeven - 49900.0_f64).abs() < 0.01);
}

#[test]
fn test_risk_reward_ratio() {
    let entry_price = 100.0;
    let stop_loss = 95.0;
    let take_profit = 110.0;

    let risk = entry_price - stop_loss;
    let reward = take_profit - entry_price;
    let risk_reward_ratio = reward / risk;

    assert_eq!(risk, 5.0);
    assert_eq!(reward, 10.0);
    assert_eq!(risk_reward_ratio, 2.0);
}

#[test]
fn test_portfolio_allocation() {
    use std::collections::HashMap;

    let total_capital = 100000.0;
    let mut allocations = HashMap::new();

    // Define allocation percentages
    allocations.insert("BTC", 0.4);  // 40%
    allocations.insert("ETH", 0.3);  // 30%
    allocations.insert("SOL", 0.2);  // 20%
    allocations.insert("CASH", 0.1); // 10%

    // Calculate actual amounts
    let btc_allocation = total_capital * allocations["BTC"];
    let eth_allocation = total_capital * allocations["ETH"];
    let sol_allocation = total_capital * allocations["SOL"];
    let cash_allocation = total_capital * allocations["CASH"];

    assert_eq!(btc_allocation, 40000.0);
    assert_eq!(eth_allocation, 30000.0);
    assert_eq!(sol_allocation, 20000.0);
    assert_eq!(cash_allocation, 10000.0);

    // Verify total
    let total = btc_allocation + eth_allocation + sol_allocation + cash_allocation;
    assert_eq!(total, total_capital);
}

#[test]
fn test_stop_loss_calculation() {
    // Test stop loss price calculation
    let entry_price = 50000.0;
    let stop_loss_percent = 2.0; // 2% stop loss

    // For long position
    let long_stop = entry_price * (1.0 - stop_loss_percent / 100.0);
    assert!((long_stop - 49000.0_f64).abs() < 0.01);

    // For short position
    let short_stop = entry_price * (1.0 + stop_loss_percent / 100.0);
    assert!((short_stop - 51000.0_f64).abs() < 0.01);
}

#[test]
fn test_take_profit_calculation() {
    // Test take profit price calculation
    let entry_price = 50000.0;
    let take_profit_percent = 5.0; // 5% take profit

    // For long position
    let long_tp = entry_price * (1.0 + take_profit_percent / 100.0);
    assert!((long_tp - 52500.0_f64).abs() < 0.01);

    // For short position
    let short_tp = entry_price * (1.0 - take_profit_percent / 100.0);
    assert!((short_tp - 47500.0_f64).abs() < 0.01);
}

#[test]
fn test_max_position_size() {
    let account_balance = 10000.0;
    let max_risk_per_trade = 0.02; // 2% max risk
    let max_position_percent = 0.3; // 30% max position size

    // Calculate max position based on risk
    let max_risk_amount = account_balance * max_risk_per_trade;
    assert_eq!(max_risk_amount, 200.0);

    // Calculate max position based on capital
    let max_position_value = account_balance * max_position_percent;
    assert_eq!(max_position_value, 3000.0);
}