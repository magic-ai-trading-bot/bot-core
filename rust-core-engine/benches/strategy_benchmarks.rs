use binance_trading_bot::strategies::indicators::{
    calculate_bollinger_bands, calculate_ema, calculate_macd, calculate_rsi, calculate_sma,
};
use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion, Throughput};

/// Benchmark RSI calculation with various data sizes
fn benchmark_rsi(c: &mut Criterion) {
    let mut group = c.benchmark_group("rsi_calculation");

    for size in [100, 500, 1000, 5000].iter() {
        let prices: Vec<f64> = (0..*size).map(|i| 50000.0 + (i as f64 % 100.0)).collect();

        group.throughput(Throughput::Elements(*size as u64));
        group.bench_with_input(BenchmarkId::from_parameter(size), &prices, |b, prices| {
            b.iter(|| {
                calculate_rsi(black_box(prices), black_box(14));
            });
        });
    }

    group.finish();
}

/// Benchmark MACD calculation
fn benchmark_macd(c: &mut Criterion) {
    let mut group = c.benchmark_group("macd_calculation");

    for size in [100, 500, 1000, 5000].iter() {
        let prices: Vec<f64> = (0..*size).map(|i| 50000.0 + (i as f64 % 100.0)).collect();

        group.throughput(Throughput::Elements(*size as u64));
        group.bench_with_input(BenchmarkId::from_parameter(size), &prices, |b, prices| {
            b.iter(|| {
                calculate_macd(
                    black_box(prices),
                    black_box(12),
                    black_box(26),
                    black_box(9),
                );
            });
        });
    }

    group.finish();
}

/// Benchmark Bollinger Bands calculation
fn benchmark_bollinger_bands(c: &mut Criterion) {
    let mut group = c.benchmark_group("bollinger_bands_calculation");

    for size in [100, 500, 1000, 5000].iter() {
        let prices: Vec<f64> = (0..*size).map(|i| 50000.0 + (i as f64 % 100.0)).collect();

        group.throughput(Throughput::Elements(*size as u64));
        group.bench_with_input(BenchmarkId::from_parameter(size), &prices, |b, prices| {
            b.iter(|| {
                calculate_bollinger_bands(black_box(prices), black_box(20), black_box(2.0));
            });
        });
    }

    group.finish();
}

/// Benchmark SMA calculation
fn benchmark_sma(c: &mut Criterion) {
    let mut group = c.benchmark_group("sma_calculation");

    for size in [100, 500, 1000, 5000].iter() {
        let prices: Vec<f64> = (0..*size).map(|i| 50000.0 + (i as f64 % 100.0)).collect();

        group.throughput(Throughput::Elements(*size as u64));
        group.bench_with_input(BenchmarkId::from_parameter(size), &prices, |b, prices| {
            b.iter(|| {
                calculate_sma(black_box(prices), black_box(20));
            });
        });
    }

    group.finish();
}

/// Benchmark EMA calculation
fn benchmark_ema(c: &mut Criterion) {
    let mut group = c.benchmark_group("ema_calculation");

    for size in [100, 500, 1000, 5000].iter() {
        let prices: Vec<f64> = (0..*size).map(|i| 50000.0 + (i as f64 % 100.0)).collect();

        group.throughput(Throughput::Elements(*size as u64));
        group.bench_with_input(BenchmarkId::from_parameter(size), &prices, |b, prices| {
            b.iter(|| {
                calculate_ema(black_box(prices), black_box(20));
            });
        });
    }

    group.finish();
}

/// Benchmark comparison of all indicators
fn benchmark_all_indicators(c: &mut Criterion) {
    let mut group = c.benchmark_group("all_indicators_comparison");
    let prices: Vec<f64> = (0..1000).map(|i| 50000.0 + (i as f64 % 100.0)).collect();

    group.bench_function("rsi_14", |b| {
        b.iter(|| calculate_rsi(black_box(&prices), black_box(14)))
    });

    group.bench_function("sma_20", |b| {
        b.iter(|| calculate_sma(black_box(&prices), black_box(20)))
    });

    group.bench_function("ema_20", |b| {
        b.iter(|| calculate_ema(black_box(&prices), black_box(20)))
    });

    group.bench_function("macd_12_26_9", |b| {
        b.iter(|| {
            calculate_macd(
                black_box(&prices),
                black_box(12),
                black_box(26),
                black_box(9),
            )
        })
    });

    group.bench_function("bollinger_20_2", |b| {
        b.iter(|| calculate_bollinger_bands(black_box(&prices), black_box(20), black_box(2.0)))
    });

    group.finish();
}

criterion_group!(
    benches,
    benchmark_rsi,
    benchmark_macd,
    benchmark_bollinger_bands,
    benchmark_sma,
    benchmark_ema,
    benchmark_all_indicators
);
criterion_main!(benches);
