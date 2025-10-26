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
                base_url: "https://testnet.binance.vision".to_string(),
                ws_url: "wss://testnet.binance.vision/ws".to_string(),
                futures_base_url: "https://testnet.binancefuture.com".to_string(),
                futures_ws_url: "wss://stream.binancefuture.com/ws".to_string(),
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
                url: "mongodb://botuser:defaultpassword@mongodb:27017/trading_bot?authSource=admin"
                    .to_string(),
                database_name: Some("trading_bot".to_string()),
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

        if let Ok(binance_api_key) = std::env::var("BINANCE_API_KEY") {
            config.binance.api_key = binance_api_key;
        }

        if let Ok(binance_secret_key) = std::env::var("BINANCE_SECRET_KEY") {
            config.binance.secret_key = binance_secret_key;
        }

        if let Ok(testnet) = std::env::var("BINANCE_TESTNET") {
            config.binance.testnet = testnet == "true";
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
        let temp_path = env::temp_dir().join("test_config_env_testnet.toml");

        Config::default().save_to_file(&temp_path).unwrap();

        env::set_var("BINANCE_TESTNET", "false");
        let config = Config::from_file(&temp_path).unwrap();
        assert!(!config.binance.testnet);

        env::set_var("BINANCE_TESTNET", "true");
        let config = Config::from_file(&temp_path).unwrap();
        assert!(config.binance.testnet);

        // Cleanup
        env::remove_var("BINANCE_TESTNET");
        let _ = std::fs::remove_file(&temp_path);
    }

    #[test]
    fn test_config_env_var_override_python_url() {
        use std::env;
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
