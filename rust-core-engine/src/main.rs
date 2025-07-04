use anyhow::Result;
use structopt::StructOpt;
use tracing::{info, Level};
use tracing_subscriber::FmtSubscriber;
use tokio::sync::broadcast;

mod config;
mod auth;
mod binance;
mod market_data;
mod trading;
mod storage;
mod monitoring;
mod api;

use config::Config;
use market_data::MarketDataProcessor;
use trading::TradingEngine;
use api::ApiServer;

#[derive(Debug, StructOpt)]
#[structopt(name = "binance-trading-bot", about = "A comprehensive Binance trading bot")]
struct Opt {
    #[structopt(short = "c", long = "config", default_value = "config.toml")]
    config_file: String,
    
    #[structopt(short = "v", long = "verbose", parse(from_occurrences))]
    verbose: u8,
}

#[tokio::main]
async fn main() -> Result<()> {
    let opt = Opt::from_args();
    
    // Initialize logging
    let level = match opt.verbose {
        0 => Level::INFO,
        1 => Level::DEBUG,
        _ => Level::TRACE,
    };
    
    let subscriber = FmtSubscriber::builder()
        .with_max_level(level)
        .finish();
    
    tracing::subscriber::set_global_default(subscriber)?;
    
    info!("Starting Binance Trading Bot");
    
    // Load configuration
    let config = Config::from_file(&opt.config_file)?;
    info!("Configuration loaded from {}", opt.config_file);
    
    // Initialize storage
    let storage = storage::Storage::new(&config.database).await?;
    
    // Initialize market data processor
    let mut market_data_processor = MarketDataProcessor::new(
        config.binance.clone(),
        config.market_data.clone(),
        storage.clone(),
    ).await?;
    
    // Initialize trading engine
    let trading_engine = TradingEngine::new(
        config.binance.clone(),
        config.trading.clone(),
        market_data_processor.clone(),
        storage.clone(),
    ).await?;
    
    // Create shared broadcast channel for WebSocket updates
    let (ws_sender, _) = broadcast::channel::<String>(1000);
    
    // Set WebSocket broadcaster for market data processor
    market_data_processor.set_ws_broadcaster(ws_sender.clone());
    
    // Initialize API server with WebSocket broadcaster
    let api_server = ApiServer::new(
        config.api.clone(),
        market_data_processor.clone(),
        trading_engine.clone(),
        ws_sender.clone(),
        storage.clone(),
    ).await?;
    
    // Start all components
    let market_data_handle = tokio::spawn(async move {
        market_data_processor.start().await
    });
    
    let trading_handle = tokio::spawn(async move {
        trading_engine.start().await
    });
    
    let api_handle = tokio::spawn(async move {
        api_server.start().await
    });
    
    info!("All systems started successfully");
    
    // Wait for all components
    tokio::try_join!(
        async { market_data_handle.await? },
        async { trading_handle.await? },
        async { api_handle.await? }
    )?;
    
    Ok(())
} 