use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;

// @spec:FR-TRADING-001 - Binance API Configuration
// @ref:specs/01-requirements/1.1-functional-requirements/FR-TRADING.md

/// Binance API URL constants for mainnet (production)
pub mod binance_urls {
    // Spot mainnet
    pub const MAINNET_BASE_URL: &str = "https://api.binance.com";
    pub const MAINNET_WS_URL: &str = "wss://stream.binance.com:9443/ws";
    pub const MAINNET_USER_DATA_WS_URL: &str = "wss://stream.binance.com:9443/ws";

    // Spot testnet
    pub const TESTNET_BASE_URL: &str = "https://testnet.binance.vision";
    pub const TESTNET_WS_URL: &str = "wss://testnet.binance.vision/ws";
    pub const TESTNET_USER_DATA_WS_URL: &str = "wss://testnet.binance.vision/ws";

    // Futures mainnet
    pub const FUTURES_MAINNET_BASE_URL: &str = "https://fapi.binance.com";
    pub const FUTURES_MAINNET_WS_URL: &str = "wss://fstream.binance.com";

    // Futures testnet
    pub const FUTURES_TESTNET_BASE_URL: &str = "https://testnet.binancefuture.com";
    pub const FUTURES_TESTNET_WS_URL: &str = "wss://stream.binancefuture.com/ws";
}

/// Trading mode for the system
/// @spec:FR-TRADING-002 - Trading Mode Selection
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum TradingMode {
    /// Paper trading with simulated orders (uses real prices)
    #[default]
    PaperTrading,
    /// Real trading on Binance testnet
    RealTestnet,
    /// Real trading on Binance mainnet (PRODUCTION - USE WITH CAUTION)
    RealMainnet,
}

impl TradingMode {
    /// Check if this mode executes real orders
    pub fn is_real_trading(&self) -> bool {
        matches!(self, TradingMode::RealTestnet | TradingMode::RealMainnet)
    }

    /// Check if this mode uses testnet
    pub fn is_testnet(&self) -> bool {
        matches!(self, TradingMode::RealTestnet)
    }

    /// Check if this mode uses mainnet (production)
    pub fn is_mainnet(&self) -> bool {
        matches!(self, TradingMode::RealMainnet)
    }

    /// Check if this is paper trading
    pub fn is_paper(&self) -> bool {
        matches!(self, TradingMode::PaperTrading)
    }

    /// Get the appropriate base URL for this trading mode
    pub fn get_base_url(&self) -> &'static str {
        match self {
            TradingMode::PaperTrading => binance_urls::MAINNET_BASE_URL, // Real prices
            TradingMode::RealTestnet => binance_urls::TESTNET_BASE_URL,
            TradingMode::RealMainnet => binance_urls::MAINNET_BASE_URL,
        }
    }

    /// Get the appropriate WebSocket URL for this trading mode
    pub fn get_ws_url(&self) -> &'static str {
        match self {
            TradingMode::PaperTrading => binance_urls::MAINNET_WS_URL, // Real prices
            TradingMode::RealTestnet => binance_urls::TESTNET_WS_URL,
            TradingMode::RealMainnet => binance_urls::MAINNET_WS_URL,
        }
    }

    /// Get the appropriate User Data Stream WebSocket URL
    pub fn get_user_data_ws_url(&self) -> &'static str {
        match self {
            TradingMode::PaperTrading => binance_urls::MAINNET_USER_DATA_WS_URL,
            TradingMode::RealTestnet => binance_urls::TESTNET_USER_DATA_WS_URL,
            TradingMode::RealMainnet => binance_urls::MAINNET_USER_DATA_WS_URL,
        }
    }
}

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
    /// Trading mode: paper_trading, real_testnet, or real_mainnet
    #[serde(default)]
    pub trading_mode: TradingMode,
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
    pub database_name: Option<String>,
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
                base_url: binance_urls::TESTNET_BASE_URL.to_string(),
                ws_url: binance_urls::TESTNET_WS_URL.to_string(),
                futures_base_url: binance_urls::FUTURES_TESTNET_BASE_URL.to_string(),
                futures_ws_url: binance_urls::FUTURES_TESTNET_WS_URL.to_string(),
                trading_mode: TradingMode::default(), // PaperTrading by default
            },
            market_data: MarketDataConfig {
                symbols: vec!["BTCUSDT".to_string(), "ETHUSDT".to_string()],
                timeframes: vec![
                    "1m".to_string(),
                    "5m".to_string(),
                    "15m".to_string(),
                    "1h".to_string(),
                    "4h".to_string(),
                    "1d".to_string(),
                ],
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
                url: "mongodb://botuser:defaultpassword@mongodb:27017/bot_core?authSource=admin"
                    .to_string(),
                database_name: Some("bot_core".to_string()),
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
        let mut config: Config = toml::from_str(&content)?;

        // Override with environment variables if they exist
        if let Ok(database_url) = std::env::var("DATABASE_URL") {
            config.database.url = database_url;
        }

        // Check testnet flag first to determine which keys to use
        let use_testnet = std::env::var("BINANCE_TESTNET")
            .map(|v| v == "true")
            .unwrap_or(config.binance.testnet);

        // Use testnet-specific keys if available and testnet mode is enabled
        if use_testnet {
            if let Ok(testnet_api_key) = std::env::var("BINANCE_TESTNET_API_KEY") {
                config.binance.api_key = testnet_api_key;
            } else if let Ok(api_key) = std::env::var("BINANCE_API_KEY") {
                config.binance.api_key = api_key;
            }

            if let Ok(testnet_secret_key) = std::env::var("BINANCE_TESTNET_SECRET_KEY") {
                config.binance.secret_key = testnet_secret_key;
            } else if let Ok(secret_key) = std::env::var("BINANCE_SECRET_KEY") {
                config.binance.secret_key = secret_key;
            }
        } else {
            // Mainnet: use regular keys
            if let Ok(api_key) = std::env::var("BINANCE_API_KEY") {
                config.binance.api_key = api_key;
            }

            if let Ok(secret_key) = std::env::var("BINANCE_SECRET_KEY") {
                config.binance.secret_key = secret_key;
            }
        }

        if let Ok(testnet) = std::env::var("BINANCE_TESTNET") {
            let use_testnet = testnet == "true";
            config.binance.testnet = use_testnet;

            // CRITICAL: Also update URLs based on testnet flag
            // This ensures paper trading uses real prices from production API
            if use_testnet {
                config.binance.base_url = binance_urls::TESTNET_BASE_URL.to_string();
                config.binance.ws_url = binance_urls::TESTNET_WS_URL.to_string();
                config.binance.futures_base_url =
                    binance_urls::FUTURES_TESTNET_BASE_URL.to_string();
                config.binance.futures_ws_url = binance_urls::FUTURES_TESTNET_WS_URL.to_string();
            } else {
                // Use PRODUCTION API for real market prices
                // This is safe because paper trading doesn't execute real trades
                config.binance.base_url = binance_urls::MAINNET_BASE_URL.to_string();
                config.binance.ws_url = binance_urls::MAINNET_WS_URL.to_string();
                config.binance.futures_base_url =
                    binance_urls::FUTURES_MAINNET_BASE_URL.to_string();
                config.binance.futures_ws_url = binance_urls::FUTURES_MAINNET_WS_URL.to_string();
            }
        }

        // Trading mode override (paper_trading, real_testnet, real_mainnet)
        if let Ok(mode) = std::env::var("TRADING_MODE") {
            config.binance.trading_mode = match mode.to_lowercase().as_str() {
                "paper_trading" | "paper" => TradingMode::PaperTrading,
                "real_testnet" | "testnet" => TradingMode::RealTestnet,
                "real_mainnet" | "mainnet" | "production" => TradingMode::RealMainnet,
                _ => {
                    eprintln!(
                        "⚠️  Invalid TRADING_MODE '{}', defaulting to PaperTrading",
                        mode
                    );
                    TradingMode::PaperTrading
                },
            };
        }

        if let Ok(python_url) = std::env::var("PYTHON_AI_SERVICE_URL") {
            config.market_data.python_ai_service_url = python_url;
        }

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
        // For paper trading, we can skip API key validation
        if self.trading.enabled {
            if self.binance.api_key.is_empty() {
                return Err(anyhow::anyhow!(
                    "Binance API key is required for live trading"
                ));
            }

            if self.binance.secret_key.is_empty() {
                return Err(anyhow::anyhow!(
                    "Binance secret key is required for live trading"
                ));
            }
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

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Mutex;

    // Global mutex to serialize all environment variable tests
    // This prevents race conditions when tests run in parallel
    static ENV_TEST_MUTEX: Mutex<()> = Mutex::new(());

    #[test]
    fn test_config_default() {
        let config = Config::default();
        assert!(config.binance.testnet);
        assert!(!config.trading.enabled);
        assert!(!config.market_data.symbols.is_empty());
    }

    #[test]
    fn test_config_validate_trading_enabled_with_keys() {
        let mut config = Config::default();
        config.trading.enabled = true;
        config.binance.api_key = "test_key".to_string();
        config.binance.secret_key = "test_secret".to_string();
        assert!(config.validate().is_ok());
    }

    #[test]
    fn test_config_validate_trading_enabled_missing_api_key() {
        let mut config = Config::default();
        config.trading.enabled = true;
        config.binance.api_key = "".to_string();
        config.binance.secret_key = "test_secret".to_string();
        assert!(config.validate().is_err());
    }

    #[test]
    fn test_config_validate_trading_disabled_no_keys() {
        let mut config = Config::default();
        config.trading.enabled = false;
        config.binance.api_key = "".to_string();
        config.binance.secret_key = "".to_string();
        assert!(config.validate().is_ok());
    }

    #[test]
    fn test_config_validate_empty_symbols() {
        let mut config = Config::default();
        config.market_data.symbols = vec![];
        assert!(config.validate().is_err());
    }

    #[test]
    fn test_config_validate_empty_timeframes() {
        let mut config = Config::default();
        config.market_data.timeframes = vec![];
        assert!(config.validate().is_err());
    }

    #[test]
    fn test_config_validate_risk_percentage_negative() {
        let mut config = Config::default();
        config.trading.risk_percentage = -1.0;
        assert!(config.validate().is_err());
    }

    #[test]
    fn test_config_validate_risk_percentage_over_100() {
        let mut config = Config::default();
        config.trading.risk_percentage = 101.0;
        assert!(config.validate().is_err());
    }

    #[test]
    fn test_config_validate_risk_percentage_valid_boundary() {
        let mut config = Config::default();
        config.trading.risk_percentage = 100.0;
        assert!(config.validate().is_ok());

        config.trading.risk_percentage = 0.1;
        assert!(config.validate().is_ok());
    }

    #[test]
    fn test_config_validate_trading_enabled_missing_secret_key() {
        let mut config = Config::default();
        config.trading.enabled = true;
        config.binance.api_key = "test_key".to_string();
        config.binance.secret_key = "".to_string();
        assert!(config.validate().is_err());
        assert!(config
            .validate()
            .unwrap_err()
            .to_string()
            .contains("secret"));
    }

    #[test]
    fn test_config_from_file_creates_default_if_missing() {
        use std::env;

        // Use global mutex to serialize all env var tests
        let _guard = ENV_TEST_MUTEX.lock().unwrap();

        // Clear env vars to prevent interference
        env::remove_var("BINANCE_API_KEY");
        env::remove_var("BINANCE_SECRET_KEY");
        env::remove_var("DATABASE_URL");
        env::remove_var("BINANCE_TESTNET");
        env::remove_var("PYTHON_AI_SERVICE_URL");

        let temp_path = env::temp_dir().join("test_config_missing.toml");

        // Ensure file doesn't exist
        let _ = std::fs::remove_file(&temp_path);

        let config = Config::from_file(&temp_path).unwrap();
        assert!(config.binance.testnet);
        assert!(!config.trading.enabled);

        // Cleanup
        let _ = std::fs::remove_file(&temp_path);
    }

    #[test]
    fn test_config_from_file_loads_existing() {
        use std::env;

        let temp_path = env::temp_dir().join("test_config_existing_unique.toml");

        // Create a test config file
        let test_config = Config::default();
        test_config.save_to_file(&temp_path).unwrap();

        // Load it back using toml parsing directly to avoid env var interference
        let content = std::fs::read_to_string(&temp_path).unwrap();
        let loaded_config: Config = toml::from_str(&content).unwrap();
        assert_eq!(loaded_config.binance.testnet, test_config.binance.testnet);
        assert_eq!(loaded_config.trading.enabled, test_config.trading.enabled);

        // Cleanup
        let _ = std::fs::remove_file(&temp_path);
    }

    #[test]
    fn test_config_save_to_file() {
        use std::env;

        let temp_path = env::temp_dir().join("test_config_save_unique.toml");

        let mut config = Config::default();
        config.trading.max_positions = 10;
        config.binance.api_key = "custom_key".to_string();

        config.save_to_file(&temp_path).unwrap();

        // Verify file was created
        assert!(temp_path.exists());

        // Read and parse the TOML directly to avoid env var interference
        let content = std::fs::read_to_string(&temp_path).unwrap();
        let loaded: Config = toml::from_str(&content).unwrap();
        assert_eq!(loaded.trading.max_positions, 10);
        assert_eq!(loaded.binance.api_key, "custom_key");

        // Cleanup
        let _ = std::fs::remove_file(&temp_path);
    }

    #[test]
    fn test_config_env_var_override_database_url() {
        use std::env;

        // Use global mutex to serialize all env var tests
        let _guard = ENV_TEST_MUTEX.lock().unwrap();

        let temp_path = env::temp_dir().join("test_config_env_db.toml");

        // Create test config
        Config::default().save_to_file(&temp_path).unwrap();

        // Set env var
        env::set_var("DATABASE_URL", "mongodb://custom:url@localhost/test");

        let config = Config::from_file(&temp_path).unwrap();
        assert_eq!(config.database.url, "mongodb://custom:url@localhost/test");

        // Cleanup
        env::remove_var("DATABASE_URL");
        let _ = std::fs::remove_file(&temp_path);
    }

    #[test]
    fn test_config_env_var_override_binance_keys() {
        use std::env;

        // Use global mutex to serialize all env var tests
        let _guard = ENV_TEST_MUTEX.lock().unwrap();

        let temp_path = env::temp_dir().join("test_config_env_binance.toml");

        Config::default().save_to_file(&temp_path).unwrap();

        env::set_var("BINANCE_API_KEY", "env_api_key");
        env::set_var("BINANCE_SECRET_KEY", "env_secret_key");

        let config = Config::from_file(&temp_path).unwrap();
        assert_eq!(config.binance.api_key, "env_api_key");
        assert_eq!(config.binance.secret_key, "env_secret_key");

        // Cleanup
        env::remove_var("BINANCE_API_KEY");
        env::remove_var("BINANCE_SECRET_KEY");
        let _ = std::fs::remove_file(&temp_path);
    }

    #[test]
    fn test_config_env_var_override_testnet() {
        use std::env;

        // Use global mutex to serialize all env var tests
        let _guard = ENV_TEST_MUTEX.lock().unwrap();

        // Save and clear env var to ensure clean state
        let original_testnet = env::var("BINANCE_TESTNET").ok();

        let temp_path = env::temp_dir().join("test_config_env_testnet_unique.toml");

        Config::default().save_to_file(&temp_path).unwrap();

        // Test setting to false - should switch to production URLs
        env::set_var("BINANCE_TESTNET", "false");
        // Small delay to ensure env var is set
        std::thread::sleep(std::time::Duration::from_millis(10));
        let config = Config::from_file(&temp_path).unwrap();
        assert!(
            !config.binance.testnet,
            "Expected testnet=false but got testnet=true (env var should override file default)"
        );
        // Verify URLs are updated to production
        assert_eq!(
            config.binance.base_url,
            binance_urls::MAINNET_BASE_URL,
            "Expected production base_url when testnet=false"
        );
        assert_eq!(
            config.binance.ws_url,
            binance_urls::MAINNET_WS_URL,
            "Expected production ws_url when testnet=false"
        );

        // Test setting to true - should use testnet URLs
        env::set_var("BINANCE_TESTNET", "true");
        std::thread::sleep(std::time::Duration::from_millis(10));
        let config = Config::from_file(&temp_path).unwrap();
        assert!(
            config.binance.testnet,
            "Expected testnet=true but got testnet=false"
        );
        // Verify URLs are updated to testnet
        assert_eq!(
            config.binance.base_url,
            binance_urls::TESTNET_BASE_URL,
            "Expected testnet base_url when testnet=true"
        );

        // Restore original env var or remove it
        match original_testnet {
            Some(val) => env::set_var("BINANCE_TESTNET", val),
            None => env::remove_var("BINANCE_TESTNET"),
        }
        let _ = std::fs::remove_file(&temp_path);
    }

    #[test]
    fn test_config_env_var_override_python_url() {
        use std::env;

        // Use global mutex to serialize all env var tests
        let _guard = ENV_TEST_MUTEX.lock().unwrap();

        let temp_path = env::temp_dir().join("test_config_env_python.toml");

        Config::default().save_to_file(&temp_path).unwrap();

        env::set_var("PYTHON_AI_SERVICE_URL", "http://custom:9000");
        let config = Config::from_file(&temp_path).unwrap();
        assert_eq!(
            config.market_data.python_ai_service_url,
            "http://custom:9000"
        );

        // Cleanup
        env::remove_var("PYTHON_AI_SERVICE_URL");
        let _ = std::fs::remove_file(&temp_path);
    }

    #[test]
    fn test_config_serialization_roundtrip() {
        let config = Config::default();
        let serialized = toml::to_string_pretty(&config).unwrap();
        let deserialized: Config = toml::from_str(&serialized).unwrap();

        assert_eq!(config.binance.testnet, deserialized.binance.testnet);
        assert_eq!(config.trading.enabled, deserialized.trading.enabled);
        assert_eq!(config.market_data.symbols, deserialized.market_data.symbols);
    }
}
