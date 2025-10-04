mod common;

#[test]
fn test_paper_trading_calculations() {
    // Test PnL calculation
    let entry_price = 50000.0;
    let current_price = 51000.0;
    let quantity = 0.1;

    let unrealized_pnl = (current_price - entry_price) * quantity;
    assert_eq!(unrealized_pnl, 100.0);
}

#[test]
fn test_portfolio_statistics() {
    let total_trades = 10;
    let winning_trades = 6;
    let _losing_trades = 4;

    // Calculate win rate
    let win_rate = (winning_trades as f64 / total_trades as f64) * 100.0;
    assert_eq!(win_rate, 60.0);
}

#[test]
fn test_risk_reward_calculation() {
    let entry_price = 50000.0;
    let stop_loss = 48000.0;
    let take_profit = 54000.0;

    let risk = entry_price - stop_loss;
    let reward = take_profit - entry_price;
    let risk_reward_ratio = reward / risk;

    assert_eq!(risk, 2000.0);
    assert_eq!(reward, 4000.0);
    assert_eq!(risk_reward_ratio, 2.0);
}

#[test]
fn test_portfolio_value_calculation() {
    let cash_balance = 5000.0;
    let btc_quantity = 0.5;
    let btc_price = 50000.0;
    let eth_quantity = 2.0;
    let eth_price = 3000.0;

    let btc_value = btc_quantity * btc_price;
    let eth_value = eth_quantity * eth_price;
    let total_value = cash_balance + btc_value + eth_value;

    assert_eq!(btc_value, 25000.0);
    assert_eq!(eth_value, 6000.0);
    assert_eq!(total_value, 36000.0);
}

#[test]
fn test_sharpe_ratio_calculation() {
    let returns = [0.01, 0.02, -0.005, 0.015, 0.03, -0.01, 0.025];
    let risk_free_rate = 0.02 / 252.0; // Daily risk-free rate

    // Calculate average return
    let avg_return: f64 = returns.iter().sum::<f64>() / returns.len() as f64;

    // Calculate standard deviation
    let variance: f64 = returns
        .iter()
        .map(|r| (r - avg_return).powi(2))
        .sum::<f64>()
        / returns.len() as f64;
    let std_dev = variance.sqrt();

    // Calculate Sharpe ratio
    let sharpe_ratio = (avg_return - risk_free_rate) / std_dev;

    assert!(sharpe_ratio > 0.0);
}

#[test]
fn test_position_pnl() {
    // Test long position PnL
    let entry_price = 3000.0;
    let current_price = 3150.0;
    let quantity = 1.0;

    let unrealized_pnl = (current_price - entry_price) * quantity;
    assert_eq!(unrealized_pnl, 150.0);

    // Test short position PnL
    let short_pnl = (entry_price - current_price) * quantity;
    assert_eq!(short_pnl, -150.0);
}

#[test]
fn test_trade_fees() {
    let trade_value = 10000.0;
    let fee_rate = 0.001; // 0.1%

    let fee = trade_value * fee_rate;
    let net_value = trade_value - fee;

    assert_eq!(fee, 10.0);
    assert_eq!(net_value, 9990.0);
}

#[test]
fn test_max_drawdown() {
    let peak_value = 12000.0;
    let current_value = 9600.0;

    let drawdown = (peak_value - current_value) / peak_value * 100.0;
    assert_eq!(drawdown, 20.0); // 20% drawdown
}
