use criterion::{black_box, criterion_group, criterion_main, Criterion};

/// Benchmark position PnL calculation
fn benchmark_position_pnl_calculation(c: &mut Criterion) {
    c.bench_function("calculate_position_pnl_long", |b| {
        b.iter(|| {
            let entry_price = black_box(50000.0);
            let current_price = black_box(51000.0);
            let quantity = black_box(0.1);
            let leverage = black_box(10_u8);

            // Long position PnL = (current - entry) * quantity * leverage
            let pnl = (current_price - entry_price) * quantity * leverage as f64;
            black_box(pnl);
        });
    });

    c.bench_function("calculate_position_pnl_short", |b| {
        b.iter(|| {
            let entry_price = black_box(50000.0);
            let current_price = black_box(49000.0);
            let quantity = black_box(0.1);
            let leverage = black_box(10_u8);

            // Short position PnL = (entry - current) * quantity * leverage
            let pnl = (entry_price - current_price) * quantity * leverage as f64;
            black_box(pnl);
        });
    });
}

/// Benchmark position size calculation with risk management
fn benchmark_position_size_calculation(c: &mut Criterion) {
    c.bench_function("calculate_position_size_with_risk", |b| {
        b.iter(|| {
            let account_balance = black_box(10000.0);
            let risk_percentage = black_box(2.0);
            let entry_price = black_box(50000.0);
            let stop_loss_price = black_box(48000.0);

            let risk_amount = account_balance * (risk_percentage / 100.0);
            let stop_loss_distance = (entry_price - stop_loss_price).abs();
            let position_size = risk_amount / stop_loss_distance;

            black_box(position_size);
        });
    });
}

/// Benchmark liquidation price calculation
fn benchmark_liquidation_price(c: &mut Criterion) {
    c.bench_function("calculate_liquidation_price_long", |b| {
        b.iter(|| {
            let entry_price = black_box(50000.0);
            let leverage = black_box(10_u8);
            let maintenance_margin_rate = black_box(0.005); // 0.5%

            // Liquidation price for long = entry_price * (1 - 1/leverage + maintenance_margin_rate)
            let liquidation_price = entry_price
                * (1.0 - 1.0 / leverage as f64 + maintenance_margin_rate);

            black_box(liquidation_price);
        });
    });

    c.bench_function("calculate_liquidation_price_short", |b| {
        b.iter(|| {
            let entry_price = black_box(50000.0);
            let leverage = black_box(10_u8);
            let maintenance_margin_rate = black_box(0.005);

            // Liquidation price for short = entry_price * (1 + 1/leverage - maintenance_margin_rate)
            let liquidation_price = entry_price
                * (1.0 + 1.0 / leverage as f64 - maintenance_margin_rate);

            black_box(liquidation_price);
        });
    });
}

/// Benchmark portfolio risk calculation
fn benchmark_portfolio_risk(c: &mut Criterion) {
    c.bench_function("calculate_portfolio_var_95", |b| {
        b.iter(|| {
            let portfolio_value = black_box(10000.0);
            let daily_volatility = black_box(0.02); // 2% daily volatility
            let z_score_95 = black_box(1.645); // 95% confidence interval

            // Value at Risk (VaR) = portfolio_value * volatility * z_score
            let var_95 = portfolio_value * daily_volatility * z_score_95;

            black_box(var_95);
        });
    });
}

/// Benchmark multiple position management operations
fn benchmark_position_operations(c: &mut Criterion) {
    let mut group = c.benchmark_group("position_operations");

    // Benchmark opening a position
    group.bench_function("open_position", |b| {
        b.iter(|| {
            let symbol = black_box("BTCUSDT");
            let entry_price = black_box(50000.0);
            let quantity = black_box(0.1);
            let leverage = black_box(10_u8);
            let position_type = black_box("LONG");

            // Simulate position opening logic
            let required_margin = (entry_price * quantity) / leverage as f64;
            let position_value = entry_price * quantity * leverage as f64;

            black_box((required_margin, position_value));
        });
    });

    // Benchmark updating position
    group.bench_function("update_position_price", |b| {
        b.iter(|| {
            let entry_price = black_box(50000.0);
            let current_price = black_box(51500.0);
            let quantity = black_box(0.1);
            let leverage = black_box(10_u8);

            // Calculate unrealized PnL
            let unrealized_pnl = (current_price - entry_price) * quantity * leverage as f64;

            // Calculate PnL percentage
            let pnl_percentage = (unrealized_pnl / (entry_price * quantity)) * 100.0;

            black_box((unrealized_pnl, pnl_percentage));
        });
    });

    // Benchmark closing a position
    group.bench_function("close_position", |b| {
        b.iter(|| {
            let entry_price = black_box(50000.0);
            let exit_price = black_box(52000.0);
            let quantity = black_box(0.1);
            let leverage = black_box(10_u8);
            let trading_fee_rate = black_box(0.0004); // 0.04%

            // Calculate realized PnL
            let gross_pnl = (exit_price - entry_price) * quantity * leverage as f64;

            // Calculate trading fees
            let entry_fee = entry_price * quantity * trading_fee_rate;
            let exit_fee = exit_price * quantity * trading_fee_rate;
            let total_fees = entry_fee + exit_fee;

            // Net PnL
            let net_pnl = gross_pnl - total_fees;

            black_box((net_pnl, total_fees));
        });
    });

    group.finish();
}

criterion_group!(
    benches,
    benchmark_position_pnl_calculation,
    benchmark_position_size_calculation,
    benchmark_liquidation_price,
    benchmark_portfolio_risk,
    benchmark_position_operations
);
criterion_main!(benches);
