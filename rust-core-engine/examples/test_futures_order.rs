// @spec:FR-REAL-020 - Test Futures Order on Binance Testnet
// Run with: cargo run --example test_futures_order
//
// IMPORTANT: Futures testnet uses DIFFERENT API keys than Spot testnet!
// Set these environment variables:
// - BINANCE_FUTURES_TESTNET_API_KEY
// - BINANCE_FUTURES_TESTNET_SECRET_KEY

use anyhow::Result;
use binance_trading_bot::binance::client::BinanceClient;
use binance_trading_bot::binance::types::NewOrderRequest;
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

    // Set Futures testnet keys (from user input)
    std::env::set_var("BINANCE_FUTURES_TESTNET_API_KEY", "xTAotoNWsNbtk8GmJ3xDSjKt43J192ZmRDVTsnhodjtNYhcdlwuaaGEJJjlx2eHi");
    std::env::set_var("BINANCE_FUTURES_TESTNET_SECRET_KEY", "yQj7Y7eDXJCDKEFDwT5JhmCItz4tAdd1SBeUCW7tyjP6Ugj9UUqentNmU33wUh5D");

    println!("🔧 Loading configuration...");

    let config = Config::from_file("config.toml")?;

    if !config.binance.testnet {
        println!("❌ ERROR: Not in testnet mode! Set BINANCE_TESTNET=true");
        return Ok(());
    }

    println!("✅ Testnet mode: {}", config.binance.testnet);
    println!("📡 Futures Base URL: {}", config.binance.futures_base_url);

    let client = BinanceClient::new(config.binance.clone())?;

    // Test 1: Get Futures Account Info
    println!("\n📊 Test 1: Get Futures Account Info...");
    match client.get_futures_account().await {
        Ok(account) => {
            println!("✅ Futures account loaded!");
            // Parse relevant fields
            if let Some(total_wallet) = account.get("totalWalletBalance") {
                println!("   Total Wallet Balance: {}", total_wallet);
            }
            if let Some(available) = account.get("availableBalance") {
                println!("   Available Balance: {}", available);
            }
            if let Some(can_trade) = account.get("canTrade") {
                println!("   Can Trade: {}", can_trade);
            }
        }
        Err(e) => {
            println!("❌ Failed to get futures account: {}", e);
            return Ok(());
        }
    }

    // Test 2: Get current BTCUSDT price (futures)
    println!("\n💰 Test 2: Get BTCUSDT Futures Price...");
    let price_info = client.get_symbol_price("BTCUSDT").await?;
    let current_price: f64 = price_info.price.parse().unwrap_or(65000.0);
    println!("✅ Current price: ${:.2}", current_price);

    // Test 3: Set Leverage to 5x
    println!("\n⚙️ Test 3: Set Leverage to 5x...");
    match client.change_leverage("BTCUSDT", 5).await {
        Ok(response) => {
            println!("✅ Leverage set successfully!");
            if let Some(leverage) = response.get("leverage") {
                println!("   Leverage: {}x", leverage);
            }
            if let Some(max_notional) = response.get("maxNotionalValue") {
                println!("   Max Notional: {}", max_notional);
            }
        }
        Err(e) => {
            println!("⚠️ Leverage change: {} (may already be set)", e);
        }
    }

    // Test 4: Set Margin Type to ISOLATED
    println!("\n⚙️ Test 4: Set Margin Type to ISOLATED...");
    match client.change_margin_type("BTCUSDT", "ISOLATED").await {
        Ok(_) => {
            println!("✅ Margin type set to ISOLATED!");
        }
        Err(e) => {
            // Error -4046 means "No need to change margin type" - this is OK
            let err_str = e.to_string();
            if err_str.contains("-4046") {
                println!("✅ Margin type already ISOLATED");
            } else {
                println!("⚠️ Margin type change: {}", e);
            }
        }
    }

    // Test 5: Get Position Risk
    println!("\n📈 Test 5: Get Position Risk...");
    match client.get_futures_positions().await {
        Ok(positions) => {
            println!("✅ Got {} position(s)", positions.len());
            for pos in positions.iter().take(5) {
                let position_amt: f64 = pos.position_amt.parse().unwrap_or(0.0);
                if position_amt.abs() > 0.0 {
                    println!(
                        "   {} - Amt: {}, Entry: {}, Unrealized PnL: {}, Leverage: {}x",
                        pos.symbol, pos.position_amt, pos.entry_price, pos.unrealized_pnl, pos.leverage
                    );
                }
            }
        }
        Err(e) => {
            println!("❌ Failed to get positions: {}", e);
        }
    }

    // Test 6: Place a LONG Market Order
    // Min notional = $100, so at $64k BTC, we need at least 0.002 BTC (~$128)
    println!("\n📝 Test 6: Place LONG Market Order (0.002 BTC)...");
    let long_order = NewOrderRequest {
        symbol: "BTCUSDT".to_string(),
        side: "BUY".to_string(),
        r#type: "MARKET".to_string(),
        quantity: Some("0.002".to_string()),
        quote_order_qty: None,
        price: None,
        new_client_order_id: Some(format!("long_{}", chrono::Utc::now().timestamp_millis())),
        stop_price: None,
        iceberg_qty: None,
        new_order_resp_type: None,
        time_in_force: None,
        reduce_only: None,
        close_position: None,
        position_side: Some("BOTH".to_string()), // One-way mode
        working_type: None,
        price_protect: None,
    };

    match client.place_futures_order(long_order).await {
        Ok(response) => {
            println!("✅ LONG order placed!");
            println!("   Order ID: {}", response.order_id);
            println!("   Status: {}", response.status);
            println!("   Executed Qty: {}", response.executed_qty);
            println!("   Avg Price: {}", response.price);
        }
        Err(e) => {
            println!("❌ Failed to place LONG order: {}", e);
        }
    }

    // Wait a bit
    tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;

    // Test 7: Check position after opening
    println!("\n📊 Test 7: Check Position After Opening...");
    match client.get_futures_positions().await {
        Ok(positions) => {
            for pos in positions.iter() {
                if pos.symbol == "BTCUSDT" {
                    let position_amt: f64 = pos.position_amt.parse().unwrap_or(0.0);
                    if position_amt.abs() > 0.0 {
                        println!("✅ Active BTCUSDT position:");
                        println!("   Position Amount: {}", pos.position_amt);
                        println!("   Entry Price: {}", pos.entry_price);
                        println!("   Mark Price: {}", pos.mark_price);
                        println!("   Unrealized PnL: {}", pos.unrealized_pnl);
                        println!("   Liquidation Price: {}", pos.liquidation_price);
                        println!("   Leverage: {}x", pos.leverage);
                        println!("   Margin Type: {}", pos.margin_type);
                    }
                }
            }
        }
        Err(e) => {
            println!("❌ Failed to get positions: {}", e);
        }
    }

    // Test 8: Close position with reduce-only order
    println!("\n📝 Test 8: Close Position (Reduce Only)...");
    let close_order = NewOrderRequest {
        symbol: "BTCUSDT".to_string(),
        side: "SELL".to_string(), // Opposite side to close LONG
        r#type: "MARKET".to_string(),
        quantity: Some("0.002".to_string()),
        quote_order_qty: None,
        price: None,
        new_client_order_id: Some(format!("close_{}", chrono::Utc::now().timestamp_millis())),
        stop_price: None,
        iceberg_qty: None,
        new_order_resp_type: None,
        time_in_force: None,
        reduce_only: Some(true), // Important: reduce-only to close position
        close_position: None,
        position_side: Some("BOTH".to_string()),
        working_type: None,
        price_protect: None,
    };

    match client.place_futures_order(close_order).await {
        Ok(response) => {
            println!("✅ Position closed!");
            println!("   Order ID: {}", response.order_id);
            println!("   Status: {}", response.status);
            println!("   Executed Qty: {}", response.executed_qty);
        }
        Err(e) => {
            println!("❌ Failed to close position: {}", e);
        }
    }

    // Test 9: Verify position closed
    println!("\n📊 Test 9: Verify Position Closed...");
    match client.get_futures_positions().await {
        Ok(positions) => {
            for pos in positions.iter() {
                if pos.symbol == "BTCUSDT" {
                    let position_amt: f64 = pos.position_amt.parse().unwrap_or(0.0);
                    if position_amt.abs() < 0.0001 {
                        println!("✅ BTCUSDT position closed (amt: {})", pos.position_amt);
                    } else {
                        println!("⚠️ Position still open: {}", pos.position_amt);
                    }
                }
            }
        }
        Err(e) => {
            println!("❌ Failed to get positions: {}", e);
        }
    }

    println!("\n✅ All Phase 3 Futures tests completed!");
    println!("📌 Note: This was on TESTNET - no real money involved.");

    Ok(())
}
