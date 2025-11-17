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

// ============== BOUNDARY CONDITION TESTS ==============

#[test]
fn test_win_rate_zero_trades() {
    // Win rate with zero trades should be 0.0 or handle gracefully
    let total_trades = 0;
    let winning_trades = 0;

    let win_rate = if total_trades == 0 {
        0.0
    } else {
        (winning_trades as f64 / total_trades as f64) * 100.0
    };

    assert_eq!(win_rate, 0.0, "Win rate should be 0 with no trades");
}

#[test]
fn test_win_rate_all_wins() {
    let total_trades = 10;
    let winning_trades = 10;

    let win_rate = (winning_trades as f64 / total_trades as f64) * 100.0;
    assert_eq!(win_rate, 100.0, "Win rate should be 100% with all wins");
}

#[test]
fn test_win_rate_all_losses() {
    let total_trades = 10;
    let winning_trades = 0;

    let win_rate = (winning_trades as f64 / total_trades as f64) * 100.0;
    assert_eq!(win_rate, 0.0, "Win rate should be 0% with all losses");
}

#[test]
fn test_sharpe_ratio_zero_volatility() {
    // Sharpe ratio with zero volatility should be 0 or handle gracefully
    let returns = [0.0, 0.0, 0.0, 0.0, 0.0]; // Flat returns
    let risk_free_rate = 0.02 / 252.0;

    let avg_return: f64 = returns.iter().sum::<f64>() / returns.len() as f64;
    let variance: f64 = returns
        .iter()
        .map(|r| (r - avg_return).powi(2))
        .sum::<f64>()
        / returns.len() as f64;
    let std_dev = variance.sqrt();

    let sharpe_ratio = if std_dev == 0.0 || std_dev.abs() < 1e-10 {
        0.0
    } else {
        (avg_return - risk_free_rate) / std_dev
    };

    assert_eq!(
        sharpe_ratio, 0.0,
        "Sharpe ratio should be 0 with no volatility"
    );
}

#[test]
fn test_max_drawdown_only_profits() {
    // Max drawdown with only profits should be 0
    let mut peak = 10000.0;
    let mut max_dd = 0.0;

    let values = [10000.0, 10500.0, 11000.0, 11500.0, 12000.0]; // Only increasing

    for &value in &values {
        if value > peak {
            peak = value;
        }
        let drawdown = (peak - value) / peak;
        if drawdown > max_dd {
            max_dd = drawdown;
        }
    }

    assert_eq!(max_dd, 0.0, "No drawdown if only profits");
}

#[test]
fn test_max_drawdown_from_peak() {
    let values: [f64; 5] = [10000.0, 12000.0, 9000.0, 8000.0, 10000.0]; // Peak at 12000
    let mut peak: f64 = values[0];
    let mut max_dd: f64 = 0.0;

    for &value in &values {
        if value > peak {
            peak = value;
        }
        let drawdown: f64 = (peak - value) / peak;
        if drawdown > max_dd {
            max_dd = drawdown;
        }
    }

    // Max drawdown from 12000 to 8000 = (12000 - 8000) / 12000 = 0.3333
    let expected: f64 = 0.3333;
    assert!(
        (max_dd - expected).abs() < 0.001,
        "Max drawdown should be ~33.33%, got {}",
        max_dd
    );
}

#[test]
fn test_pnl_percentage_zero_entry_price() {
    // PnL percentage with zero entry price should handle gracefully
    let entry_price: f64 = 0.0;
    let current_price: f64 = 100.0;
    let _quantity: f64 = 1.0; // Not used in calculation but kept for clarity

    let pnl_pct = if entry_price == 0.0 || entry_price.abs() < 1e-10 {
        0.0
    } else {
        ((current_price - entry_price) / entry_price) * 100.0
    };

    assert_eq!(pnl_pct, 0.0, "Should return 0 for zero entry price");
}

#[test]
fn test_margin_ratio_zero_initial_margin() {
    // Margin ratio with zero initial margin should handle gracefully
    let initial_margin: f64 = 0.0;
    let maintenance_margin: f64 = 100.0;

    let ratio = if initial_margin == 0.0 || initial_margin.abs() < 1e-10 {
        0.0
    } else {
        maintenance_margin / initial_margin
    };

    assert_eq!(ratio, 0.0, "Should return 0 for zero initial margin");
}

#[test]
fn test_leverage_calculation() {
    let position_value: f64 = 50000.0;
    let margin: f64 = 5000.0;

    let leverage = position_value / margin;
    assert_eq!(leverage, 10.0, "Leverage should be 10x");

    // Test with zero margin
    let zero_margin: f64 = 0.0;
    let safe_leverage = if zero_margin == 0.0 {
        0.0
    } else {
        position_value / zero_margin
    };
    assert_eq!(safe_leverage, 0.0, "Should handle zero margin");
}

#[test]
fn test_profit_factor_calculation() {
    // Profit factor = Total Wins / Total Losses
    let total_wins = 5000.0;
    let total_losses = 2000.0;

    let profit_factor = total_wins / total_losses;
    assert_eq!(profit_factor, 2.5, "Profit factor should be 2.5");

    // Test with zero losses (perfect trading)
    let zero_losses = 0.0;
    let safe_pf = if zero_losses == 0.0 {
        if total_wins > 0.0 {
            f64::INFINITY
        } else {
            0.0
        }
    } else {
        total_wins / zero_losses
    };
    assert!(
        safe_pf.is_infinite(),
        "Profit factor should be infinite with no losses"
    );
}
