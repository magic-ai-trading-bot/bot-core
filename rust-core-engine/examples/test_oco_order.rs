// @spec:FR-REAL-010 - Test OCO Order on Binance Testnet
// Run with: cargo run --example test_oco_order

use anyhow::Result;
use binance_trading_bot::binance::client::BinanceClient;
use binance_trading_bot::binance::types::{OcoOrderRequest, OrderSide, SpotOrderRequest};
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

    let config = Config::from_file("config.toml")?;

    if !config.binance.testnet {
        println!("❌ ERROR: Not in testnet mode! Set BINANCE_TESTNET=true");
        return Ok(());
    }

    println!("✅ Testnet mode: {}", config.binance.testnet);
    println!("📡 Base URL: {}", config.binance.base_url);

    let client = BinanceClient::new(config.binance.clone())?;

    // Get current BTC price
    println!("\n💰 Getting current BTCUSDT price...");
    let price_info = client.get_symbol_price("BTCUSDT").await?;
    let current_price: f64 = price_info.price.parse().unwrap_or(65000.0);
    println!("✅ Current price: ${:.2}", current_price);

    // First, buy some BTC to have a position
    println!("\n📝 Step 1: Buy 0.001 BTC (create position)...");
    let buy_order = SpotOrderRequest::market("BTCUSDT", OrderSide::Buy, "0.001")
        .with_client_order_id(&format!("buy_{}", chrono::Utc::now().timestamp_millis()));

    match client.place_spot_order(buy_order).await {
        Ok(response) => {
            println!("✅ Buy order filled!");
            println!("   Order ID: {}", response.order_id);
            println!("   Status: {}", response.status);

            // Calculate SL and TP prices
            // For a LONG position:
            // - Take Profit: Above current price (sell high)
            // - Stop Loss: Below current price (sell low to cut losses)
            let take_profit_price = current_price * 1.02; // +2%
            let stop_loss_trigger = current_price * 0.98; // -2%
            let stop_loss_limit = current_price * 0.975; // -2.5% (slightly lower for execution)

            println!("\n📝 Step 2: Place OCO order to protect position...");
            println!("   Take Profit: ${:.2} (+2%)", take_profit_price);
            println!("   Stop Loss Trigger: ${:.2} (-2%)", stop_loss_trigger);
            println!("   Stop Loss Limit: ${:.2} (-2.5%)", stop_loss_limit);

            // Create OCO: Sell 0.001 BTC with TP and SL
            let oco_request = OcoOrderRequest::new(
                "BTCUSDT",
                OrderSide::Sell, // Sell to close long position
                "0.001",
                &format!("{:.2}", take_profit_price),
                &format!("{:.2}", stop_loss_trigger),
                &format!("{:.2}", stop_loss_limit),
            )
            .with_client_order_ids(
                &format!("oco_list_{}", chrono::Utc::now().timestamp_millis()),
                &format!("oco_tp_{}", chrono::Utc::now().timestamp_millis()),
                &format!("oco_sl_{}", chrono::Utc::now().timestamp_millis()),
            );

            match client.place_oco_order(oco_request).await {
                Ok(oco_response) => {
                    println!("✅ OCO order placed successfully!");
                    println!("   Order List ID: {}", oco_response.order_list_id);
                    println!("   List Status: {}", oco_response.list_order_status);
                    println!("   Contingency Type: {}", oco_response.contingency_type);

                    println!("\n   Orders in OCO:");
                    for order in &oco_response.orders {
                        println!(
                            "     - Order ID: {}, Client ID: {}",
                            order.order_id, order.client_order_id
                        );
                    }

                    if !oco_response.order_reports.is_empty() {
                        println!("\n   Order Details:");
                        for report in &oco_response.order_reports {
                            println!(
                                "     - {} {} @ {} (Status: {}, Type: {})",
                                report.side,
                                report.orig_qty,
                                report.price,
                                report.status,
                                report.order_type
                            );
                        }
                    }

                    // Optional: Cancel the OCO to clean up
                    println!("\n📝 Step 3: Cancel OCO order (cleanup)...");
                    match client
                        .cancel_oco_order("BTCUSDT", Some(oco_response.order_list_id), None)
                        .await
                    {
                        Ok(cancel_response) => {
                            println!("✅ OCO cancelled successfully!");
                            println!("   List Status: {}", cancel_response.list_order_status);
                        },
                        Err(e) => {
                            println!("⚠️  Failed to cancel OCO: {}", e);
                            println!("   (May have already been filled or cancelled)");
                        },
                    }
                },
                Err(e) => {
                    println!("❌ Failed to place OCO order: {}", e);
                    println!("   Note: OCO may not be available on testnet for all pairs");
                },
            }
        },
        Err(e) => {
            println!("❌ Failed to place buy order: {}", e);
        },
    }

    // Test Stop-Loss Limit order directly
    println!("\n📝 Test: Place Stop-Loss Limit order...");
    let sl_price = current_price * 0.97; // -3%
    let sl_limit = current_price * 0.965; // -3.5%

    let sl_order = SpotOrderRequest::stop_loss_limit(
        "BTCUSDT",
        OrderSide::Sell,
        "0.0001",
        &format!("{:.2}", sl_limit),
        &format!("{:.2}", sl_price),
    )
    .with_client_order_id(&format!("sl_{}", chrono::Utc::now().timestamp_millis()));

    match client.place_spot_order(sl_order).await {
        Ok(response) => {
            println!("✅ Stop-Loss order placed!");
            println!("   Order ID: {}", response.order_id);
            println!("   Status: {}", response.status);

            // Cancel to clean up
            let _ = client
                .cancel_spot_order("BTCUSDT", Some(response.order_id), None)
                .await;
            println!("   (Cancelled for cleanup)");
        },
        Err(e) => {
            println!("❌ Failed to place SL order: {}", e);
        },
    }

    // Test Take-Profit Limit order directly
    println!("\n📝 Test: Place Take-Profit Limit order...");
    let tp_price = current_price * 1.03; // +3%
    let tp_limit = current_price * 1.035; // +3.5%

    let tp_order = SpotOrderRequest::take_profit_limit(
        "BTCUSDT",
        OrderSide::Sell,
        "0.0001",
        &format!("{:.2}", tp_limit),
        &format!("{:.2}", tp_price),
    )
    .with_client_order_id(&format!("tp_{}", chrono::Utc::now().timestamp_millis()));

    match client.place_spot_order(tp_order).await {
        Ok(response) => {
            println!("✅ Take-Profit order placed!");
            println!("   Order ID: {}", response.order_id);
            println!("   Status: {}", response.status);

            // Cancel to clean up
            let _ = client
                .cancel_spot_order("BTCUSDT", Some(response.order_id), None)
                .await;
            println!("   (Cancelled for cleanup)");
        },
        Err(e) => {
            println!("❌ Failed to place TP order: {}", e);
        },
    }

    println!("\n✅ All Phase 2 tests completed!");
    println!("📌 Note: This was on TESTNET - no real money involved.");

    Ok(())
}
