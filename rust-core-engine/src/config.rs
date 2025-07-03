use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub binance: BinanceConfig,
    pub market_data: MarketDataConfig,
    pub trading: TradingConfig,
    pub database: DatabaseConfig,
    pub api: ApiConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BinanceConfig {
    pub api_key: String,
    pub secret_key: String,
    pub testnet: bool,
    pub base_url: String,
    pub ws_url: String,
    pub futures_base_url: String,
    pub futures_ws_url: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MarketDataConfig {
    pub symbols: Vec<String>,
    pub timeframes: Vec<String>,
    pub kline_limit: u16,
    pub update_interval_ms: u64,
    pub reconnect_interval_ms: u64,
    pub max_reconnect_attempts: u32,
    pub cache_size: usize,
    pub python_ai_service_url: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TradingConfig {
    pub enabled: bool,
    pub max_positions: u32,
    pub default_quantity: f64,
    pub risk_percentage: f64,
    pub stop_loss_percentage: f64,
    pub take_profit_percentage: f64,
    pub order_timeout_seconds: u64,
    pub position_check_interval_seconds: u64,
    pub leverage: u8,
    pub margin_type: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DatabaseConfig {
    pub url: String,
    pub max_connections: u32,
    pub enable_logging: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiConfig {
    pub host: String,
    pub port: u16,
    pub cors_origins: Vec<String>,
    pub enable_metrics: bool,
}

impl Default for Config {
    fn default() -> Self {
        Config {
            binance: BinanceConfig {
                api_key: String::new(),
                secret_key: String::new(),
                testnet: true,
                base_url: "https://testnet.binance.vision".to_string(),
                ws_url: "wss://testnet.binance.vision/ws".to_string(),
                futures_base_url: "https://testnet.binancefuture.com".to_string(),
                futures_ws_url: "wss://stream.binancefuture.com/ws".to_string(),
            },
            market_data: MarketDataConfig {
                symbols: vec!["BTCUSDT".to_string(), "ETHUSDT".to_string()],
                timeframes: vec!["1m".to_string(), "5m".to_string(), "15m".to_string(), "1h".to_string(), "4h".to_string(), "1d".to_string()],
                kline_limit: 500,
                update_interval_ms: 1000,
                reconnect_interval_ms: 5000,
                max_reconnect_attempts: 10,
                cache_size: 1000,
                python_ai_service_url: "http://localhost:8000".to_string(),
            },
            trading: TradingConfig {
                enabled: false,
                max_positions: 5,
                default_quantity: 0.01,
                risk_percentage: 2.0,
                stop_loss_percentage: 2.0,
                take_profit_percentage: 4.0,
                order_timeout_seconds: 30,
                position_check_interval_seconds: 5,
                leverage: 1,
                margin_type: "CROSSED".to_string(),
            },
            database: DatabaseConfig {
                url: "sqlite:./trading_data.db".to_string(),
                max_connections: 10,
                enable_logging: false,
            },
            api: ApiConfig {
                host: "0.0.0.0".to_string(),
                port: 8080,
                cors_origins: vec!["*".to_string()],
                enable_metrics: true,
            },
        }
    }
}

impl Config {
    pub fn from_file<P: AsRef<Path>>(path: P) -> Result<Self> {
        let path = path.as_ref();
        
        if !path.exists() {
            // Create default config file if it doesn't exist
            let default_config = Config::default();
            let config_str = toml::to_string_pretty(&default_config)?;
            fs::write(path, config_str)?;
            return Ok(default_config);
        }
        
        let content = fs::read_to_string(path)?;
        let config: Config = toml::from_str(&content)?;
        Ok(config)
    }
    
    pub fn save_to_file<P: AsRef<Path>>(&self, path: P) -> Result<()> {
        let config_str = toml::to_string_pretty(self)?;
        fs::write(path, config_str)?;
        Ok(())
    }
}

// Helper function to validate configuration
impl Config {
    pub fn validate(&self) -> Result<()> {
        if self.binance.api_key.is_empty() {
            return Err(anyhow::anyhow!("Binance API key is required"));
        }
        
        if self.binance.secret_key.is_empty() {
            return Err(anyhow::anyhow!("Binance secret key is required"));
        }
        
        if self.market_data.symbols.is_empty() {
            return Err(anyhow::anyhow!("At least one symbol must be configured"));
        }
        
        if self.market_data.timeframes.is_empty() {
            return Err(anyhow::anyhow!("At least one timeframe must be configured"));
        }
        
        if self.trading.risk_percentage <= 0.0 || self.trading.risk_percentage > 100.0 {
            return Err(anyhow::anyhow!("Risk percentage must be between 0 and 100"));
        }
        
        Ok(())
    }
} 