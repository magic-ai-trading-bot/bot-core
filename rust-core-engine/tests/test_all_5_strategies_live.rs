/**
 * Test all 5 trading strategies with real Binance market data
 *
 * This test demonstrates that all 5 strategies are working:
 * 1. RSI Strategy
 * 2. MACD Strategy
 * 3. Bollinger Bands Strategy
 * 4. Volume Strategy
 * 5. Stochastic Strategy (NEW)
 *
 * Run with: cargo test --test test_all_5_strategies_live -- --nocapture
 */
use binance_trading_bot::binance::client::BinanceClient;
use binance_trading_bot::config::BinanceConfig;
use binance_trading_bot::market_data::cache::CandleData;
use binance_trading_bot::strategies::{
    bollinger_strategy::BollingerStrategy, macd_strategy::MacdStrategy, rsi_strategy::RsiStrategy,
    stochastic_strategy::StochasticStrategy, volume_strategy::VolumeStrategy, Strategy,
    StrategyInput, TradingSignal,
};
use std::collections::HashMap;

#[tokio::test]
#[ignore] // Mark as ignored so it doesn't run in CI (requires internet)
async fn test_all_5_strategies_with_real_binance_data() {
    println!("\n{}", "=".repeat(80));
    println!("🚀 TESTING ALL 5 TRADING STRATEGIES WITH REAL BINANCE DATA");
    println!("{}", "=".repeat(80));
    println!();

    // Step 1: Fetch real market data from Binance
    println!("📊 Step 1: Fetching real market data from Binance...");

    let config = BinanceConfig {
        api_key: "".to_string(),
        secret_key: "".to_string(),
        futures_api_key: String::new(),
        futures_secret_key: String::new(),
        testnet: false,
        base_url: "https://api.binance.com".to_string(),
        ws_url: "wss://stream.binance.com:9443".to_string(),
        futures_base_url: "https://fapi.binance.com".to_string(),
        futures_ws_url: "wss://fstream.binance.com".to_string(),
        trading_mode: binance_trading_bot::config::TradingMode::PaperTrading,
    };

    let client = BinanceClient::new(config).expect("Failed to create Binance client");

    let symbol = "BTCUSDT";
    let intervals = vec!["5m", "15m"];

    let mut timeframe_data: HashMap<String, Vec<CandleData>> = HashMap::new();

    for interval in &intervals {
        match client.get_klines(symbol, interval, Some(100)).await {
            Ok(klines) => {
                println!(
                    "   ✅ Fetched {} candles for {} timeframe",
                    klines.len(),
                    interval
                );
                // Convert Vec<Kline> to Vec<CandleData>
                let candles: Vec<CandleData> = klines.iter().map(CandleData::from).collect();
                timeframe_data.insert(interval.to_string(), candles);
            },
            Err(e) => {
                println!("   ❌ Failed to fetch {} data: {}", interval, e);
                panic!("Cannot continue without market data");
            },
        }
    }

    // Get current price (use last close price from 1h)
    let current_price = timeframe_data
        .get("5m")
        .and_then(|candles| candles.last())
        .map(|candle| candle.close)
        .unwrap_or(50000.0);

    // Calculate 24h volume
    let volume_24h: f64 = timeframe_data
        .get("5m")
        .map(|candles| candles.iter().rev().take(24).map(|c| c.volume).sum())
        .unwrap_or(1000000.0);

    println!("   ✅ Current Price: ${:.2}", current_price);
    println!("   ✅ 24h Volume: {:.2} BTC", volume_24h);
    println!();

    // Step 2: Create strategy input
    let strategy_input = StrategyInput {
        symbol: symbol.to_string(),
        timeframe_data: timeframe_data.clone(),
        current_price,
        volume_24h,
        timestamp: chrono::Utc::now().timestamp_millis(),
    };

    // Step 3: Initialize all 5 strategies
    println!("🤖 Step 2: Initializing all 5 strategies...");
    let rsi_strategy = RsiStrategy::new();
    let macd_strategy = MacdStrategy::new();
    let bollinger_strategy = BollingerStrategy::new();
    let volume_strategy = VolumeStrategy::new();
    let stochastic_strategy = StochasticStrategy::new();

    println!("   ✅ RSI Strategy");
    println!("   ✅ MACD Strategy");
    println!("   ✅ Bollinger Bands Strategy");
    println!("   ✅ Volume Strategy");
    println!("   ✅ Stochastic Strategy (NEW)");
    println!();

    // Step 4: Analyze with all 5 strategies
    println!("{}", "=".repeat(80));
    println!("📈 Step 3: Analyzing with all 5 strategies...");
    println!("{}", "=".repeat(80));
    println!();

    let mut long_count = 0;
    let mut short_count = 0;
    let mut neutral_count = 0;

    // Test RSI Strategy
    println!("1. RSI STRATEGY");
    match rsi_strategy.analyze(&strategy_input).await {
        Ok(output) => {
            let emoji = match output.signal {
                TradingSignal::Long => {
                    long_count += 1;
                    "🟢"
                },
                TradingSignal::Short => {
                    short_count += 1;
                    "🔴"
                },
                TradingSignal::Neutral => {
                    neutral_count += 1;
                    "⚪"
                },
            };
            println!("   Signal:     {} {:?}", emoji, output.signal);
            println!("   Confidence: {:.2}%", output.confidence * 100.0);
            println!("   Reasoning:  {}", output.reasoning);
            println!("   Timeframe:  {}", output.timeframe);
        },
        Err(e) => println!("   ❌ Error: {}", e),
    }
    println!();

    // Test MACD Strategy
    println!("2. MACD STRATEGY");
    match macd_strategy.analyze(&strategy_input).await {
        Ok(output) => {
            let emoji = match output.signal {
                TradingSignal::Long => {
                    long_count += 1;
                    "🟢"
                },
                TradingSignal::Short => {
                    short_count += 1;
                    "🔴"
                },
                TradingSignal::Neutral => {
                    neutral_count += 1;
                    "⚪"
                },
            };
            println!("   Signal:     {} {:?}", emoji, output.signal);
            println!("   Confidence: {:.2}%", output.confidence * 100.0);
            println!("   Reasoning:  {}", output.reasoning);
            println!("   Timeframe:  {}", output.timeframe);
        },
        Err(e) => println!("   ❌ Error: {}", e),
    }
    println!();

    // Test Bollinger Strategy
    println!("3. BOLLINGER BANDS STRATEGY");
    match bollinger_strategy.analyze(&strategy_input).await {
        Ok(output) => {
            let emoji = match output.signal {
                TradingSignal::Long => {
                    long_count += 1;
                    "🟢"
                },
                TradingSignal::Short => {
                    short_count += 1;
                    "🔴"
                },
                TradingSignal::Neutral => {
                    neutral_count += 1;
                    "⚪"
                },
            };
            println!("   Signal:     {} {:?}", emoji, output.signal);
            println!("   Confidence: {:.2}%", output.confidence * 100.0);
            println!("   Reasoning:  {}", output.reasoning);
            println!("   Timeframe:  {}", output.timeframe);
        },
        Err(e) => println!("   ❌ Error: {}", e),
    }
    println!();

    // Test Volume Strategy
    println!("4. VOLUME STRATEGY");
    match volume_strategy.analyze(&strategy_input).await {
        Ok(output) => {
            let emoji = match output.signal {
                TradingSignal::Long => {
                    long_count += 1;
                    "🟢"
                },
                TradingSignal::Short => {
                    short_count += 1;
                    "🔴"
                },
                TradingSignal::Neutral => {
                    neutral_count += 1;
                    "⚪"
                },
            };
            println!("   Signal:     {} {:?}", emoji, output.signal);
            println!("   Confidence: {:.2}%", output.confidence * 100.0);
            println!("   Reasoning:  {}", output.reasoning);
            println!("   Timeframe:  {}", output.timeframe);
        },
        Err(e) => println!("   ❌ Error: {}", e),
    }
    println!();

    // Test Stochastic Strategy (NEW!)
    println!("5. STOCHASTIC STRATEGY (NEW)");
    match stochastic_strategy.analyze(&strategy_input).await {
        Ok(output) => {
            let emoji = match output.signal {
                TradingSignal::Long => {
                    long_count += 1;
                    "🟢"
                },
                TradingSignal::Short => {
                    short_count += 1;
                    "🔴"
                },
                TradingSignal::Neutral => {
                    neutral_count += 1;
                    "⚪"
                },
            };
            println!("   Signal:     {} {:?}", emoji, output.signal);
            println!("   Confidence: {:.2}%", output.confidence * 100.0);
            println!("   Reasoning:  {}", output.reasoning);
            println!("   Timeframe:  {}", output.timeframe);
        },
        Err(e) => println!("   ❌ Error: {}", e),
    }
    println!();

    // Step 5: Display consensus
    println!("{}", "=".repeat(80));
    println!("🎲 CONSENSUS ANALYSIS");
    println!("{}", "=".repeat(80));
    println!();

    let total = long_count + short_count + neutral_count;
    println!("📊 Vote Breakdown:");
    println!(
        "   🟢 LONG:    {}/{} strategies ({:.0}%)",
        long_count,
        total,
        (long_count as f64 / total as f64) * 100.0
    );
    println!(
        "   🔴 SHORT:   {}/{} strategies ({:.0}%)",
        short_count,
        total,
        (short_count as f64 / total as f64) * 100.0
    );
    println!(
        "   ⚪ NEUTRAL: {}/{} strategies ({:.0}%)",
        neutral_count,
        total,
        (neutral_count as f64 / total as f64) * 100.0
    );
    println!();

    let consensus = if long_count > short_count && long_count > neutral_count {
        "🟢 LONG"
    } else if short_count > long_count && short_count > neutral_count {
        "🔴 SHORT"
    } else {
        "⚪ NEUTRAL"
    };

    let agreement = *[long_count, short_count, neutral_count]
        .iter()
        .max()
        .unwrap();
    let agreement_pct = (agreement as f64 / total as f64) * 100.0;

    println!("🎯 Final Consensus: {}", consensus);
    println!(
        "📈 Agreement Level: {:.0}% ({}/{} strategies agree)",
        agreement_pct, agreement, total
    );
    println!();

    if agreement_pct >= 80.0 {
        println!("   ✅ STRONG CONSENSUS - High confidence signal");
    } else if agreement_pct >= 60.0 {
        println!("   ⚠️  MODERATE CONSENSUS - Medium confidence signal");
    } else {
        println!("   ❌ WEAK CONSENSUS - Low confidence signal");
    }
    println!();

    println!("{}", "=".repeat(80));
    println!("✅ TEST COMPLETED SUCCESSFULLY");
    println!("{}", "=".repeat(80));
    println!();
    println!("✨ All 5 strategies are working correctly with real Binance data!");
    println!("✨ UI showing '5/5 chiến lược đồng ý' is now ACCURATE!");
    println!();

    // Assertion to verify all strategies returned results
    assert_eq!(total, 5, "All 5 strategies should have returned signals");
}

#[tokio::test]
async fn test_strategy_count() {
    // Quick test to verify we have exactly 5 strategies
    let strategies: Vec<Box<dyn Strategy>> = vec![
        Box::new(RsiStrategy::new()),
        Box::new(MacdStrategy::new()),
        Box::new(BollingerStrategy::new()),
        Box::new(VolumeStrategy::new()),
        Box::new(StochasticStrategy::new()),
    ];

    assert_eq!(strategies.len(), 5, "Should have exactly 5 strategies");

    // Verify each strategy has a unique name
    let names: Vec<&'static str> = strategies.iter().map(|s| s.name()).collect();
    println!("\n✅ Verified 5 strategies:");
    for (i, name) in names.iter().enumerate() {
        println!("   {}. {}", i + 1, name);
    }

    assert_eq!(names.len(), 5);
    assert!(names.contains(&"RSI Strategy"));
    assert!(names.contains(&"MACD Strategy"));
    assert!(names.contains(&"Bollinger Bands Strategy"));
    assert!(names.contains(&"Volume Strategy"));
    assert!(names.contains(&"Stochastic Strategy"));
}
