// @spec:FR-REAL-001 - Test Spot Order on Binance Testnet
// Run with: cargo run --example test_spot_order

use anyhow::Result;
use binance_trading_bot::binance::client::BinanceClient;
use binance_trading_bot::binance::types::{OrderSide, SpotOrderRequest};
use binance_trading_bot::config::Config;

#[tokio::main]
async fn main() -> Result<()> {
    // Load environment variables from .env file
    if let Ok(content) = std::fs::read_to_string("../.env") {
        for line in content.lines() {
            if let Some((key, value)) = line.split_once('=') {
                let key = key.trim();
                let value = value.trim();
                if !key.starts_with('#') && !key.is_empty() {
                    std::env::set_var(key, value);
                }
            }
        }
    }

    println!("🔧 Loading configuration...");

    // Load config
    let config = Config::from_file("config.toml")?;

    // Check testnet mode
    if !config.binance.testnet {
        println!("❌ ERROR: Not in testnet mode! Set BINANCE_TESTNET=true");
        return Ok(());
    }

    println!("✅ Testnet mode: {}", config.binance.testnet);
    println!("📡 Base URL: {}", config.binance.base_url);

    // Check API keys
    if config.binance.api_key.is_empty() || config.binance.secret_key.is_empty() {
        println!("❌ ERROR: API keys not set!");
        println!("   Set BINANCE_TESTNET_API_KEY and BINANCE_TESTNET_SECRET_KEY in .env");
        return Ok(());
    }

    println!("🔑 API Key: {}...", &config.binance.api_key[..20]);

    // Create Binance client
    let client = BinanceClient::new(config.binance.clone())?;

    // Test 1: Get account info
    println!("\n📊 Test 1: Get Account Info...");
    match client.get_account_info().await {
        Ok(account) => {
            println!("✅ Account loaded successfully!");
            println!("   Can Trade: {}", account.can_trade);
            println!("   Account Type: {}", account.account_type);

            // Show USDT balance
            for balance in &account.balances {
                if balance.asset == "USDT" || balance.asset == "BTC" {
                    let free: f64 = balance.free.parse().unwrap_or(0.0);
                    let locked: f64 = balance.locked.parse().unwrap_or(0.0);
                    if free > 0.0 || locked > 0.0 {
                        println!(
                            "   {} - Free: {:.8}, Locked: {:.8}",
                            balance.asset, free, locked
                        );
                    }
                }
            }
        },
        Err(e) => {
            println!("❌ Failed to get account info: {}", e);
            return Ok(());
        },
    }

    // Test 2: Get current BTC price
    println!("\n💰 Test 2: Get BTCUSDT Price...");
    match client.get_symbol_price("BTCUSDT").await {
        Ok(price) => {
            let price_f64: f64 = price.price.parse().unwrap_or(0.0);
            println!("✅ BTCUSDT Price: ${:.2}", price_f64);
        },
        Err(e) => {
            println!("❌ Failed to get price: {}", e);
        },
    }

    // Test 3: Place a small market buy order
    println!("\n📝 Test 3: Place Market Buy Order (0.0001 BTC)...");
    // Use the helper method for cleaner code
    let order_request =
        SpotOrderRequest::market("BTCUSDT", OrderSide::Buy, "0.0001").with_client_order_id(
            &format!("test_order_{}", chrono::Utc::now().timestamp_millis()),
        );

    match client.place_spot_order(order_request).await {
        Ok(response) => {
            println!("✅ Order placed successfully!");
            println!("   Order ID: {}", response.order_id);
            println!("   Client Order ID: {}", response.client_order_id);
            println!("   Status: {}", response.status);
            println!("   Symbol: {}", response.symbol);
            println!("   Executed Qty: {}", response.executed_qty);
            println!("   Cumulative Quote Qty: {}", response.cumulative_quote_qty);

            if !response.fills.is_empty() {
                println!("   Fills:");
                for fill in &response.fills {
                    println!(
                        "     - Price: {}, Qty: {}, Commission: {} {}",
                        fill.price, fill.qty, fill.commission, fill.commission_asset
                    );
                }
            }
        },
        Err(e) => {
            println!("❌ Failed to place order: {}", e);
        },
    }

    // Test 4: Get open orders
    println!("\n📋 Test 4: Get Open Orders...");
    match client.get_open_spot_orders(Some("BTCUSDT")).await {
        Ok(orders) => {
            if orders.is_empty() {
                println!("✅ No open orders (market order filled immediately)");
            } else {
                println!("✅ Open orders: {}", orders.len());
                for order in &orders {
                    println!(
                        "   - {} {} {} @ {} (Status: {})",
                        order.symbol, order.side, order.orig_qty, order.price, order.status
                    );
                }
            }
        },
        Err(e) => {
            println!("❌ Failed to get open orders: {}", e);
        },
    }

    println!("\n✅ All tests completed!");
    println!("📌 Note: This was a TESTNET order - no real money involved.");

    Ok(())
}
