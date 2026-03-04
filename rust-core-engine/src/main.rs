// Increase recursion limit for Warp's deeply nested filter types
// Required when chaining many .or() filters in API routes
#![recursion_limit = "512"]
#![allow(dead_code)]
#![allow(unused_variables)]
#![allow(unused_imports)]

use anyhow::Result;
use structopt::StructOpt;
use tokio::sync::broadcast;
use tracing::{info, Level};
use tracing_subscriber::FmtSubscriber;

mod ai;
mod api;
mod auth;
mod binance;
mod config;
mod error;
mod market_data;
mod monitoring;
mod paper_trading;
mod real_trading;
mod storage;
mod strategies;
mod trading;

use api::ApiServer;
use config::Config;
use market_data::MarketDataProcessor;
use paper_trading::{PaperTradingEngine, PaperTradingSettings};
use real_trading::RealTradingEngine;
use trading::risk_manager::RiskManager;
use trading::TradingEngine;

#[derive(Debug, StructOpt)]
#[structopt(
    name = "binance-trading-bot",
    about = "A comprehensive Binance trading bot"
)]
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

    let subscriber = FmtSubscriber::builder().with_max_level(level).finish();

    tracing::subscriber::set_global_default(subscriber)?;

    info!("Starting Binance Trading Bot");

    // Load configuration
    let config = Config::from_file(&opt.config_file)?;
    info!("Configuration loaded from {}", opt.config_file);

    // Log important configuration for debugging
    info!(
        "🌐 Binance API: testnet={}, base_url={}",
        config.binance.testnet, config.binance.base_url
    );
    info!(
        "📊 Price updates will use: {}",
        if config.binance.base_url.contains("testnet") {
            "TESTNET (⚠️ May be unstable!)"
        } else {
            "PRODUCTION (✅ Real market prices)"
        }
    );

    // Initialize storage
    let storage = storage::Storage::new(&config.database).await?;

    // Initialize market data processor
    let mut market_data_processor = MarketDataProcessor::new(
        config.binance.clone(),
        config.market_data.clone(),
        storage.clone(),
    )
    .await?;

    // Initialize trading engine
    let trading_engine = TradingEngine::new(
        config.binance.clone(),
        config.trading.clone(),
        market_data_processor.clone(),
        storage.clone(),
    )
    .await?;

    // Create shared broadcast channel for WebSocket updates
    let (ws_sender, _) = broadcast::channel::<String>(1000);
    let (paper_trading_event_sender, _) =
        broadcast::channel::<paper_trading::PaperTradingEvent>(1000);

    // Set WebSocket broadcaster for market data processor
    market_data_processor.set_ws_broadcaster(ws_sender.clone());

    // Load paper trading settings from YAML baseline (git-tracked source of truth).
    // On startup: YAML always wins → written to DB (overwrites stale runtime values).
    // Runtime tuning via API/self-tuning goes to DB only (reset on restart).
    let yaml_path = std::env::var("PAPER_TRADING_YAML")
        .unwrap_or_else(|_| "config/paper-trading-defaults.yml".to_string());
    let mut paper_trading_settings = PaperTradingSettings::from_yaml(&yaml_path)
        .expect("FATAL: Cannot load paper trading YAML baseline. Fix the file and restart.");
    info!("📋 Paper trading settings loaded from YAML: {}", yaml_path);

    // Append user-added symbols from DB (dynamic additions not in YAML baseline)
    match storage.load_user_symbols().await {
        Ok(user_symbols) => {
            for symbol in user_symbols {
                if !paper_trading_settings.symbols.contains_key(&symbol) {
                    info!("📊 Loading user-added symbol: {}", symbol);
                    paper_trading_settings.set_symbol_settings(
                        symbol,
                        paper_trading::settings::SymbolSettings {
                            enabled: true,
                            leverage: None,
                            position_size_pct: None,
                            stop_loss_pct: None,
                            take_profit_pct: None,
                            trading_hours: None,
                            min_price_movement_pct: None,
                            max_positions: Some(1),
                            custom_params: std::collections::HashMap::new(),
                        },
                    );
                }
            }
        },
        Err(e) => {
            info!(
                "No user symbols found in database (this is normal for first run): {}",
                e
            );
        },
    }

    info!(
        "🎯 Total symbols for AI analysis: {:?}",
        paper_trading_settings.symbols.keys().collect::<Vec<_>>()
    );

    let binance_client = binance::BinanceClient::new(config.binance.clone())?;
    let ai_service = ai::AIService::new(ai::AIServiceConfig {
        python_service_url: config.market_data.python_ai_service_url.clone(),
        request_timeout_seconds: 30,
        max_retries: 3,
        enable_caching: true,
        cache_ttl_seconds: 300,
    });

    let mut paper_trading_engine_inner = PaperTradingEngine::new(
        paper_trading_settings,
        binance_client,
        ai_service,
        storage.clone(),
        paper_trading_event_sender,
    )
    .await?;

    // Connect WebSocket price cache to PaperTradingEngine
    // This replaces REST polling (~480 calls/min → 0) with O(1) cache reads
    paper_trading_engine_inner.set_market_data_cache(market_data_processor.get_cache().clone());

    let paper_trading_engine = std::sync::Arc::new(paper_trading_engine_inner);

    // Initialize Real Trading Engine if configured
    let real_trading_engine = if let Some(ref real_trading_config) = config.real_trading {
        info!("🔥 Real trading configuration found, initializing engine...");

        // Create a separate Binance client for real trading with testnet settings
        let real_binance_config = config::BinanceConfig {
            api_key: std::env::var("BINANCE_TESTNET_API_KEY")
                .unwrap_or_else(|_| config.binance.api_key.clone()),
            secret_key: std::env::var("BINANCE_TESTNET_SECRET_KEY")
                .unwrap_or_else(|_| config.binance.secret_key.clone()),
            futures_api_key: std::env::var("BINANCE_FUTURES_TESTNET_API_KEY")
                .unwrap_or_else(|_| config.binance.futures_api_key.clone()),
            futures_secret_key: std::env::var("BINANCE_FUTURES_TESTNET_SECRET_KEY")
                .unwrap_or_else(|_| config.binance.futures_secret_key.clone()),
            testnet: real_trading_config.use_testnet,
            base_url: if real_trading_config.use_testnet {
                config::binance_urls::TESTNET_BASE_URL.to_string()
            } else {
                config::binance_urls::MAINNET_BASE_URL.to_string()
            },
            ws_url: if real_trading_config.use_testnet {
                config::binance_urls::TESTNET_WS_URL.to_string()
            } else {
                config::binance_urls::MAINNET_WS_URL.to_string()
            },
            futures_base_url: if real_trading_config.use_testnet {
                config::binance_urls::FUTURES_TESTNET_BASE_URL.to_string()
            } else {
                config::binance_urls::FUTURES_MAINNET_BASE_URL.to_string()
            },
            futures_ws_url: if real_trading_config.use_testnet {
                config::binance_urls::FUTURES_TESTNET_WS_URL.to_string()
            } else {
                config::binance_urls::FUTURES_MAINNET_WS_URL.to_string()
            },
            trading_mode: if real_trading_config.use_testnet {
                config::TradingMode::RealTestnet
            } else {
                config::TradingMode::RealMainnet
            },
        };

        let real_binance_client = binance::BinanceClient::new(real_binance_config)?;
        let risk_manager = RiskManager::new(config.trading.clone());

        match RealTradingEngine::new(
            real_trading_config.clone(),
            real_binance_client,
            risk_manager,
        )
        .await
        {
            Ok(mut engine) => {
                info!(
                    "✅ Real trading engine initialized successfully (testnet={})",
                    real_trading_config.use_testnet
                );
                // Connect WebSocket price cache for O(1) lookups (same cache as paper trading)
                engine.set_market_data_cache(market_data_processor.get_cache().clone());
                let engine = std::sync::Arc::new(engine);
                // Auto-start the real trading engine so balance/orders are available immediately
                match engine.start().await {
                    Ok(_) => info!("🚀 Real trading engine auto-started successfully"),
                    Err(e) => tracing::warn!("⚠️ Failed to auto-start real trading engine: {}", e),
                }
                Some(engine)
            },
            Err(e) => {
                tracing::warn!(
                    "⚠️ Failed to initialize real trading engine: {}. Continuing without it.",
                    e
                );
                None
            },
        }
    } else {
        info!("📝 No real trading configuration found, running in paper trading mode only");
        None
    };

    // Initialize API server with WebSocket broadcaster
    let api_server = ApiServer::new(
        config.api.clone(),
        config.binance.clone(),
        market_data_processor.clone(),
        trading_engine.clone(),
        paper_trading_engine.clone(),
        real_trading_engine,
        ws_sender.clone(),
        storage.clone(),
    )
    .await?;

    // Start all components
    let market_data_handle = tokio::spawn(async move { market_data_processor.start().await });

    let trading_handle = tokio::spawn(async move { trading_engine.start().await });

    let paper_trading_handle = tokio::spawn(async move {
        let engine = paper_trading_engine.clone();
        engine.start().await
    });

    let api_handle = tokio::spawn(async move { api_server.start().await });

    info!("All systems started successfully");

    // Wait for all components
    tokio::try_join!(
        async { market_data_handle.await? },
        async { trading_handle.await? },
        async { paper_trading_handle.await? },
        async { api_handle.await? }
    )?;

    Ok(())
}
