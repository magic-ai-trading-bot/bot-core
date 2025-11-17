// Comprehensive unit tests for utility modules
// Target: 90%+ coverage for config.rs, error.rs, and monitoring/mod.rs

use binance_trading_bot::config::*;
use binance_trading_bot::error::*;
use binance_trading_bot::monitoring::*;
use binance_trading_bot::storage::PerformanceStats;
use std::env;
use std::fs;
use std::path::PathBuf;
use tempfile::TempDir;

// =============================================================================
// CONFIG MODULE TESTS
// =============================================================================

mod config_tests {
    use super::*;
    use std::sync::Mutex;

    // Global mutex to serialize all environment variable tests
    // This prevents race conditions when tests run in parallel
    static ENV_TEST_MUTEX: Mutex<()> = Mutex::new(());

    // Helper function to create a temporary directory and config file
    fn setup_test_config_file(content: &str) -> (TempDir, PathBuf) {
        let temp_dir = TempDir::new().expect("Failed to create temp directory");
        let config_path = temp_dir.path().join("test_config.toml");
        fs::write(&config_path, content).expect("Failed to write test config file");
        (temp_dir, config_path)
    }

    #[test]
    fn test_config_default() {
        let config = Config::default();

        // Test binance config defaults
        assert!(config.binance.testnet);
        assert_eq!(config.binance.base_url, "https://testnet.binance.vision");
        assert_eq!(config.binance.ws_url, "wss://testnet.binance.vision/ws");
        assert_eq!(
            config.binance.futures_base_url,
            "https://testnet.binancefuture.com"
        );
        assert_eq!(
            config.binance.futures_ws_url,
            "wss://stream.binancefuture.com/ws"
        );

        // Test market data config defaults
        assert!(config.market_data.symbols.contains(&"BTCUSDT".to_string()));
        assert!(config.market_data.symbols.contains(&"ETHUSDT".to_string()));
        assert_eq!(config.market_data.kline_limit, 500);
        assert_eq!(config.market_data.update_interval_ms, 1000);
        assert_eq!(config.market_data.reconnect_interval_ms, 5000);
        assert_eq!(config.market_data.max_reconnect_attempts, 10);
        assert_eq!(config.market_data.cache_size, 1000);

        // Test trading config defaults
        assert!(!config.trading.enabled);
        assert_eq!(config.trading.max_positions, 5);
        assert_eq!(config.trading.default_quantity, 0.01);
        assert_eq!(config.trading.risk_percentage, 2.0);
        assert_eq!(config.trading.leverage, 1);

        // Test database config defaults
        assert_eq!(config.database.max_connections, 10);
        assert!(!config.database.enable_logging);

        // Test API config defaults
        assert_eq!(config.api.host, "0.0.0.0");
        assert_eq!(config.api.port, 8080);
        assert!(config.api.enable_metrics);
    }

    #[test]
    fn test_config_from_file_creates_default_if_missing() {
        let temp_dir = TempDir::new().expect("Failed to create temp directory");
        let config_path = temp_dir.path().join("missing_config.toml");

        // File doesn't exist yet
        assert!(!config_path.exists());

        // Load config (should create default)
        let config = Config::from_file(&config_path).expect("Failed to load config");

        // File should now exist
        assert!(config_path.exists());

        // Should return default config
        assert!(config.binance.testnet);
        assert!(!config.trading.enabled);
    }

    #[test]
    fn test_config_from_file_valid_toml() {
        // Use global mutex to serialize all env var tests
        let _guard = ENV_TEST_MUTEX.lock().unwrap();

        // Clean up any env vars that might interfere
        env::remove_var("BINANCE_API_KEY");
        env::remove_var("BINANCE_SECRET_KEY");
        env::remove_var("DATABASE_URL");
        env::remove_var("BINANCE_TESTNET");
        env::remove_var("PYTHON_AI_SERVICE_URL");

        let toml_content = r#"
[binance]
api_key = "test_api_key"
secret_key = "test_secret_key"
testnet = false
base_url = "https://api.binance.com"
ws_url = "wss://stream.binance.com/ws"
futures_base_url = "https://fapi.binance.com"
futures_ws_url = "wss://fstream.binance.com/ws"

[market_data]
symbols = ["BTCUSDT", "ETHUSDT", "BNBUSDT"]
timeframes = ["1m", "5m", "1h"]
kline_limit = 1000
update_interval_ms = 2000
reconnect_interval_ms = 10000
max_reconnect_attempts = 5
cache_size = 2000
python_ai_service_url = "http://python-ai:8000"

[trading]
enabled = true
max_positions = 10
default_quantity = 0.1
risk_percentage = 5.0
stop_loss_percentage = 3.0
take_profit_percentage = 6.0
order_timeout_seconds = 60
position_check_interval_seconds = 10
leverage = 5
margin_type = "ISOLATED"

[database]
url = "mongodb://test:test@localhost:27017/test"
database_name = "test_db"
max_connections = 20
enable_logging = true

[api]
host = "127.0.0.1"
port = 9090
cors_origins = ["http://localhost:3000"]
enable_metrics = false
"#;

        let (_temp_dir, config_path) = setup_test_config_file(toml_content);
        let config = Config::from_file(&config_path).expect("Failed to load config");

        // Verify all fields are loaded correctly
        assert_eq!(config.binance.api_key, "test_api_key");
        assert_eq!(config.binance.secret_key, "test_secret_key");
        assert!(!config.binance.testnet);
        assert_eq!(config.binance.base_url, "https://api.binance.com");

        assert_eq!(
            config.market_data.symbols,
            vec!["BTCUSDT", "ETHUSDT", "BNBUSDT"]
        );
        assert_eq!(config.market_data.kline_limit, 1000);
        assert_eq!(config.market_data.update_interval_ms, 2000);

        assert!(config.trading.enabled);
        assert_eq!(config.trading.max_positions, 10);
        assert_eq!(config.trading.leverage, 5);

        assert_eq!(config.database.max_connections, 20);
        assert!(config.database.enable_logging);

        assert_eq!(config.api.port, 9090);
        assert!(!config.api.enable_metrics);
    }

    #[test]
    fn test_config_from_file_invalid_toml() {
        let invalid_toml = r#"
[binance
api_key = "missing_closing_bracket"
"#;

        let (_temp_dir, config_path) = setup_test_config_file(invalid_toml);
        let result = Config::from_file(&config_path);

        // Should fail to parse invalid TOML
        assert!(result.is_err());
    }

    #[test]
    fn test_config_from_file_missing_required_fields() {
        let incomplete_toml = r#"
[binance]
api_key = "test_key"
"#;

        let (_temp_dir, config_path) = setup_test_config_file(incomplete_toml);
        let result = Config::from_file(&config_path);

        // Should fail due to missing required fields
        assert!(result.is_err());
    }

    #[test]
    fn test_config_environment_variable_override_database_url() {
        // Use global mutex to serialize all env var tests
        let _guard = ENV_TEST_MUTEX.lock().unwrap();

        let toml_content = r#"
[binance]
api_key = ""
secret_key = ""
testnet = true
base_url = "https://testnet.binance.vision"
ws_url = "wss://testnet.binance.vision/ws"
futures_base_url = "https://testnet.binancefuture.com"
futures_ws_url = "wss://stream.binancefuture.com/ws"

[market_data]
symbols = ["BTCUSDT"]
timeframes = ["1m"]
kline_limit = 500
update_interval_ms = 1000
reconnect_interval_ms = 5000
max_reconnect_attempts = 10
cache_size = 1000
python_ai_service_url = "http://localhost:8000"

[trading]
enabled = false
max_positions = 5
default_quantity = 0.01
risk_percentage = 2.0
stop_loss_percentage = 2.0
take_profit_percentage = 4.0
order_timeout_seconds = 30
position_check_interval_seconds = 5
leverage = 1
margin_type = "CROSSED"

[database]
url = "mongodb://original:password@localhost:27017/original"
database_name = "original_db"
max_connections = 10
enable_logging = false

[api]
host = "0.0.0.0"
port = 8080
cors_origins = ["*"]
enable_metrics = true
"#;

        let (_temp_dir, config_path) = setup_test_config_file(toml_content);

        // Set environment variable
        env::set_var(
            "DATABASE_URL",
            "mongodb://overridden:password@localhost:27017/overridden",
        );

        let config = Config::from_file(&config_path).expect("Failed to load config");

        // Verify environment variable override
        assert_eq!(
            config.database.url,
            "mongodb://overridden:password@localhost:27017/overridden"
        );

        // Clean up
        env::remove_var("DATABASE_URL");
    }

    #[test]
    fn test_config_environment_variable_override_binance_keys() {
        // Use global mutex to serialize all env var tests
        let _guard = ENV_TEST_MUTEX.lock().unwrap();

        let toml_content = r#"
[binance]
api_key = "original_api_key"
secret_key = "original_secret_key"
testnet = true
base_url = "https://testnet.binance.vision"
ws_url = "wss://testnet.binance.vision/ws"
futures_base_url = "https://testnet.binancefuture.com"
futures_ws_url = "wss://stream.binancefuture.com/ws"

[market_data]
symbols = ["BTCUSDT"]
timeframes = ["1m"]
kline_limit = 500
update_interval_ms = 1000
reconnect_interval_ms = 5000
max_reconnect_attempts = 10
cache_size = 1000
python_ai_service_url = "http://localhost:8000"

[trading]
enabled = false
max_positions = 5
default_quantity = 0.01
risk_percentage = 2.0
stop_loss_percentage = 2.0
take_profit_percentage = 4.0
order_timeout_seconds = 30
position_check_interval_seconds = 5
leverage = 1
margin_type = "CROSSED"

[database]
url = "mongodb://localhost:27017/test"
database_name = "test_db"
max_connections = 10
enable_logging = false

[api]
host = "0.0.0.0"
port = 8080
cors_origins = ["*"]
enable_metrics = true
"#;

        let (_temp_dir, config_path) = setup_test_config_file(toml_content);

        // Set environment variables
        env::set_var("BINANCE_API_KEY", "env_api_key");
        env::set_var("BINANCE_SECRET_KEY", "env_secret_key");

        let config = Config::from_file(&config_path).expect("Failed to load config");

        // Verify environment variable overrides
        assert_eq!(config.binance.api_key, "env_api_key");
        assert_eq!(config.binance.secret_key, "env_secret_key");

        // Clean up
        env::remove_var("BINANCE_API_KEY");
        env::remove_var("BINANCE_SECRET_KEY");
    }

    #[test]
    fn test_config_environment_variable_override_testnet() {
        // Use global mutex to serialize all env var tests
        let _guard = ENV_TEST_MUTEX.lock().unwrap();

        let toml_content = r#"
[binance]
api_key = ""
secret_key = ""
testnet = true
base_url = "https://testnet.binance.vision"
ws_url = "wss://testnet.binance.vision/ws"
futures_base_url = "https://testnet.binancefuture.com"
futures_ws_url = "wss://stream.binancefuture.com/ws"

[market_data]
symbols = ["BTCUSDT"]
timeframes = ["1m"]
kline_limit = 500
update_interval_ms = 1000
reconnect_interval_ms = 5000
max_reconnect_attempts = 10
cache_size = 1000
python_ai_service_url = "http://localhost:8000"

[trading]
enabled = false
max_positions = 5
default_quantity = 0.01
risk_percentage = 2.0
stop_loss_percentage = 2.0
take_profit_percentage = 4.0
order_timeout_seconds = 30
position_check_interval_seconds = 5
leverage = 1
margin_type = "CROSSED"

[database]
url = "mongodb://localhost:27017/test"
database_name = "test_db"
max_connections = 10
enable_logging = false

[api]
host = "0.0.0.0"
port = 8080
cors_origins = ["*"]
enable_metrics = true
"#;

        let (_temp_dir, config_path) = setup_test_config_file(toml_content);

        // Test setting to false
        env::set_var("BINANCE_TESTNET", "false");
        let config = Config::from_file(&config_path).expect("Failed to load config");
        assert!(!config.binance.testnet);
        env::remove_var("BINANCE_TESTNET");

        // Test setting to true
        env::set_var("BINANCE_TESTNET", "true");
        let config = Config::from_file(&config_path).expect("Failed to load config");
        assert!(config.binance.testnet);
        env::remove_var("BINANCE_TESTNET");

        // Test non-true value (should be false)
        env::set_var("BINANCE_TESTNET", "anything_else");
        let config = Config::from_file(&config_path).expect("Failed to load config");
        assert!(!config.binance.testnet);
        env::remove_var("BINANCE_TESTNET");
    }

    #[test]
    fn test_config_environment_variable_override_python_service_url() {
        // Use global mutex to serialize all env var tests
        let _guard = ENV_TEST_MUTEX.lock().unwrap();

        let toml_content = r#"
[binance]
api_key = ""
secret_key = ""
testnet = true
base_url = "https://testnet.binance.vision"
ws_url = "wss://testnet.binance.vision/ws"
futures_base_url = "https://testnet.binancefuture.com"
futures_ws_url = "wss://stream.binancefuture.com/ws"

[market_data]
symbols = ["BTCUSDT"]
timeframes = ["1m"]
kline_limit = 500
update_interval_ms = 1000
reconnect_interval_ms = 5000
max_reconnect_attempts = 10
cache_size = 1000
python_ai_service_url = "http://localhost:8000"

[trading]
enabled = false
max_positions = 5
default_quantity = 0.01
risk_percentage = 2.0
stop_loss_percentage = 2.0
take_profit_percentage = 4.0
order_timeout_seconds = 30
position_check_interval_seconds = 5
leverage = 1
margin_type = "CROSSED"

[database]
url = "mongodb://localhost:27017/test"
database_name = "test_db"
max_connections = 10
enable_logging = false

[api]
host = "0.0.0.0"
port = 8080
cors_origins = ["*"]
enable_metrics = true
"#;

        let (_temp_dir, config_path) = setup_test_config_file(toml_content);

        env::set_var("PYTHON_AI_SERVICE_URL", "http://python-ai-service:8000");
        let config = Config::from_file(&config_path).expect("Failed to load config");

        assert_eq!(
            config.market_data.python_ai_service_url,
            "http://python-ai-service:8000"
        );

        env::remove_var("PYTHON_AI_SERVICE_URL");
    }

    #[test]
    fn test_config_save_to_file() {
        // Use global mutex to serialize all env var tests
        let _guard = ENV_TEST_MUTEX.lock().unwrap();

        // Clean up env vars to prevent interference
        env::remove_var("BINANCE_API_KEY");
        env::remove_var("BINANCE_SECRET_KEY");
        env::remove_var("DATABASE_URL");
        env::remove_var("BINANCE_TESTNET");
        env::remove_var("PYTHON_AI_SERVICE_URL");

        let temp_dir = TempDir::new().expect("Failed to create temp directory");
        let config_path = temp_dir.path().join("save_test_config.toml");

        let mut config = Config::default();
        config.binance.api_key = "saved_api_key".to_string();
        config.trading.enabled = true;
        config.api.port = 9999;

        // Save config to file
        config
            .save_to_file(&config_path)
            .expect("Failed to save config");

        // Verify file exists
        assert!(config_path.exists());

        // Load it back and verify
        let loaded_config = Config::from_file(&config_path).expect("Failed to load saved config");
        assert_eq!(loaded_config.binance.api_key, "saved_api_key");
        assert!(loaded_config.trading.enabled);
        assert_eq!(loaded_config.api.port, 9999);
    }

    #[test]
    fn test_config_validate_trading_disabled_no_keys() {
        let mut config = Config::default();
        config.trading.enabled = false;
        config.binance.api_key = "".to_string();
        config.binance.secret_key = "".to_string();

        // Should pass validation when trading is disabled
        assert!(config.validate().is_ok());
    }

    #[test]
    fn test_config_validate_trading_enabled_missing_api_key() {
        let mut config = Config::default();
        config.trading.enabled = true;
        config.binance.api_key = "".to_string();
        config.binance.secret_key = "test_secret".to_string();

        // Should fail validation
        let result = config.validate();
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("Binance API key is required"));
    }

    #[test]
    fn test_config_validate_trading_enabled_missing_secret_key() {
        let mut config = Config::default();
        config.trading.enabled = true;
        config.binance.api_key = "test_api".to_string();
        config.binance.secret_key = "".to_string();

        // Should fail validation
        let result = config.validate();
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("Binance secret key is required"));
    }

    #[test]
    fn test_config_validate_trading_enabled_with_keys() {
        let mut config = Config::default();
        config.trading.enabled = true;
        config.binance.api_key = "test_api".to_string();
        config.binance.secret_key = "test_secret".to_string();

        // Should pass validation
        assert!(config.validate().is_ok());
    }

    #[test]
    fn test_config_validate_empty_symbols() {
        let mut config = Config::default();
        config.market_data.symbols.clear();

        // Should fail validation
        let result = config.validate();
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("At least one symbol must be configured"));
    }

    #[test]
    fn test_config_validate_empty_timeframes() {
        let mut config = Config::default();
        config.market_data.timeframes.clear();

        // Should fail validation
        let result = config.validate();
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("At least one timeframe must be configured"));
    }

    #[test]
    fn test_config_validate_risk_percentage_zero() {
        let mut config = Config::default();
        config.trading.risk_percentage = 0.0;

        // Should fail validation
        let result = config.validate();
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("Risk percentage must be between 0 and 100"));
    }

    #[test]
    fn test_config_validate_risk_percentage_negative() {
        let mut config = Config::default();
        config.trading.risk_percentage = -5.0;

        // Should fail validation
        let result = config.validate();
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("Risk percentage must be between 0 and 100"));
    }

    #[test]
    fn test_config_validate_risk_percentage_over_100() {
        let mut config = Config::default();
        config.trading.risk_percentage = 150.0;

        // Should fail validation
        let result = config.validate();
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("Risk percentage must be between 0 and 100"));
    }

    #[test]
    fn test_config_validate_risk_percentage_valid_boundary() {
        let mut config = Config::default();

        // Test minimum valid value (just above 0)
        config.trading.risk_percentage = 0.01;
        assert!(config.validate().is_ok());

        // Test maximum valid value
        config.trading.risk_percentage = 100.0;
        assert!(config.validate().is_ok());

        // Test mid-range value
        config.trading.risk_percentage = 50.0;
        assert!(config.validate().is_ok());
    }

    #[test]
    fn test_config_clone() {
        let config1 = Config::default();
        let config2 = config1.clone();

        assert_eq!(config1.binance.testnet, config2.binance.testnet);
        assert_eq!(config1.api.port, config2.api.port);
        assert_eq!(config1.trading.enabled, config2.trading.enabled);
    }

    #[test]
    fn test_config_serialize_deserialize() {
        let config = Config::default();

        // Serialize to TOML string
        let toml_str = toml::to_string(&config).expect("Failed to serialize config");

        // Deserialize back
        let deserialized: Config = toml::from_str(&toml_str).expect("Failed to deserialize config");

        // Verify key fields match
        assert_eq!(config.binance.testnet, deserialized.binance.testnet);
        assert_eq!(config.api.port, deserialized.api.port);
        assert_eq!(config.trading.enabled, deserialized.trading.enabled);
    }
}

// =============================================================================
// ERROR MODULE TESTS
// =============================================================================

mod error_tests {
    use super::*;
    use warp::reject::custom;
    use warp::Rejection;

    #[test]
    fn test_app_error_database_display() {
        let db_error = mongodb::error::Error::custom("Test database error");
        let app_error = AppError::Database(db_error);

        let error_string = format!("{}", app_error);
        assert!(error_string.contains("Database error"));
        // MongoDB error may have additional formatting, just check the prefix
    }

    #[test]
    fn test_app_error_auth_display() {
        let app_error = AppError::Auth("Invalid credentials".to_string());
        let error_string = format!("{}", app_error);
        assert_eq!(error_string, "Authentication error: Invalid credentials");
    }

    #[test]
    fn test_app_error_validation_display() {
        let app_error = AppError::Validation("Invalid input format".to_string());
        let error_string = format!("{}", app_error);
        assert_eq!(error_string, "Validation error: Invalid input format");
    }

    #[test]
    fn test_app_error_external_api_display() {
        let app_error = AppError::ExternalApi("Binance API timeout".to_string());
        let error_string = format!("{}", app_error);
        assert_eq!(error_string, "External API error: Binance API timeout");
    }

    #[test]
    fn test_app_error_trading_display() {
        let app_error = AppError::Trading("Order rejected".to_string());
        let error_string = format!("{}", app_error);
        assert_eq!(error_string, "Trading error: Order rejected");
    }

    #[test]
    fn test_app_error_rate_limit_display() {
        let app_error = AppError::RateLimit;
        let error_string = format!("{}", app_error);
        assert_eq!(error_string, "Rate limit exceeded");
    }

    #[test]
    fn test_app_error_not_found_display() {
        let app_error = AppError::NotFound("Order ID 12345".to_string());
        let error_string = format!("{}", app_error);
        assert_eq!(error_string, "Resource not found: Order ID 12345");
    }

    #[test]
    fn test_app_error_insufficient_funds_display() {
        let app_error = AppError::InsufficientFunds;
        let error_string = format!("{}", app_error);
        assert_eq!(error_string, "Insufficient funds");
    }

    #[test]
    fn test_app_error_invalid_market_conditions_display() {
        let app_error = AppError::InvalidMarketConditions("High volatility".to_string());
        let error_string = format!("{}", app_error);
        assert_eq!(error_string, "Invalid market conditions: High volatility");
    }

    #[test]
    fn test_app_error_websocket_display() {
        let app_error = AppError::WebSocket("Connection lost".to_string());
        let error_string = format!("{}", app_error);
        assert_eq!(error_string, "WebSocket error: Connection lost");
    }

    #[test]
    fn test_app_error_config_display() {
        let app_error = AppError::Config("Missing API key".to_string());
        let error_string = format!("{}", app_error);
        assert_eq!(error_string, "Configuration error: Missing API key");
    }

    #[test]
    fn test_app_error_internal_display() {
        let app_error = AppError::Internal;
        let error_string = format!("{}", app_error);
        assert_eq!(error_string, "Internal server error");
    }

    #[test]
    fn test_app_error_service_unavailable_display() {
        let app_error = AppError::ServiceUnavailable("Python AI Service".to_string());
        let error_string = format!("{}", app_error);
        assert_eq!(error_string, "Service unavailable: Python AI Service");
    }

    #[test]
    fn test_app_error_from_mongodb_error() {
        let db_error = mongodb::error::Error::custom("Connection failed");
        let app_error: AppError = db_error.into();

        match app_error {
            AppError::Database(_) => {
                // Successfully converted to Database error
            },
            _ => panic!("Expected Database error"),
        }
    }

    #[tokio::test]
    async fn test_handle_rejection_auth_error() {
        let rejection: Rejection = custom(AppError::Auth("Token expired".to_string()));
        let result = handle_rejection(rejection).await;

        // Verify the handler returns Ok
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_handle_rejection_validation_error() {
        let rejection: Rejection = custom(AppError::Validation("Invalid price".to_string()));
        let result = handle_rejection(rejection).await;

        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_handle_rejection_trading_error() {
        let rejection: Rejection = custom(AppError::Trading("Insufficient margin".to_string()));
        let result = handle_rejection(rejection).await;

        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_handle_rejection_rate_limit() {
        let rejection: Rejection = custom(AppError::RateLimit);
        let result = handle_rejection(rejection).await;

        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_handle_rejection_not_found() {
        let rejection: Rejection = custom(AppError::NotFound("Position 123".to_string()));
        let result = handle_rejection(rejection).await;

        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_handle_rejection_insufficient_funds() {
        let rejection: Rejection = custom(AppError::InsufficientFunds);
        let result = handle_rejection(rejection).await;

        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_handle_rejection_websocket_error() {
        let rejection: Rejection = custom(AppError::WebSocket("Protocol error".to_string()));
        let result = handle_rejection(rejection).await;

        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_handle_rejection_internal_error() {
        let rejection: Rejection = custom(AppError::Internal);
        let result = handle_rejection(rejection).await;

        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_handle_rejection_external_api_error() {
        let rejection: Rejection = custom(AppError::ExternalApi("API timeout".to_string()));
        let result = handle_rejection(rejection).await;

        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_handle_rejection_config_error() {
        let rejection: Rejection = custom(AppError::Config("Missing config".to_string()));
        let result = handle_rejection(rejection).await;

        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_handle_rejection_service_unavailable() {
        let rejection: Rejection = custom(AppError::ServiceUnavailable("Database".to_string()));
        let result = handle_rejection(rejection).await;

        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_handle_rejection_invalid_market_conditions() {
        let rejection: Rejection = custom(AppError::InvalidMarketConditions("Halted".to_string()));
        let result = handle_rejection(rejection).await;

        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_handle_rejection_database_error() {
        let db_error = mongodb::error::Error::custom("Connection failed");
        let rejection: Rejection = custom(AppError::Database(db_error));
        let result = handle_rejection(rejection).await;

        assert!(result.is_ok());
    }

    #[test]
    fn test_app_error_debug_format() {
        let error = AppError::Auth("Test".to_string());
        let debug_str = format!("{:?}", error);
        assert!(debug_str.contains("Auth"));
        assert!(debug_str.contains("Test"));
    }
}

// =============================================================================
// MONITORING MODULE TESTS
// =============================================================================

mod monitoring_tests {
    use super::*;

    #[test]
    fn test_monitoring_service_new() {
        let service = MonitoringService::new();

        // Verify initial state
        assert_eq!(service.get_system_metrics().active_positions, 0);
        assert_eq!(service.get_system_metrics().total_trades, 0);
        assert_eq!(service.get_system_metrics().cache_size, 0);

        assert_eq!(service.get_trading_metrics().total_pnl, 0.0);
        assert_eq!(service.get_trading_metrics().win_rate, 0.0);
        assert_eq!(service.get_trading_metrics().total_volume, 0.0);

        assert!(!service.get_connection_status().websocket_connected);
        assert!(!service.get_connection_status().api_responsive);
        assert_eq!(service.get_connection_status().reconnect_count, 0);
    }

    #[test]
    fn test_monitoring_service_default() {
        let service = MonitoringService::default();

        // Default should be same as new
        assert_eq!(service.get_system_metrics().active_positions, 0);
        assert_eq!(service.get_system_metrics().total_trades, 0);
    }

    #[test]
    fn test_update_system_metrics() {
        let mut service = MonitoringService::new();

        // Wait a bit so uptime is non-zero
        std::thread::sleep(std::time::Duration::from_millis(100));

        service.update_system_metrics(5, 1000);

        let metrics = service.get_system_metrics();
        assert_eq!(metrics.active_positions, 5);
        assert_eq!(metrics.cache_size, 1000);
        // uptime_seconds is u64, always >= 0
        assert!(metrics.last_update > 0);

        // Placeholder values
        assert_eq!(metrics.memory_usage_mb, 50.0);
        assert_eq!(metrics.cpu_usage_percent, 10.0);
    }

    #[test]
    fn test_update_system_metrics_uptime_increases() {
        let mut service = MonitoringService::new();

        std::thread::sleep(std::time::Duration::from_millis(100));
        service.update_system_metrics(1, 100);
        let uptime1 = service.get_system_metrics().uptime_seconds;

        std::thread::sleep(std::time::Duration::from_millis(100));
        service.update_system_metrics(2, 200);
        let uptime2 = service.get_system_metrics().uptime_seconds;

        // Uptime should increase
        assert!(uptime2 >= uptime1);
    }

    #[test]
    fn test_update_trading_metrics() {
        let mut service = MonitoringService::new();

        let stats = PerformanceStats {
            total_trades: 100,
            winning_trades: 60,
            losing_trades: 40,
            win_rate: 60.0,
            total_pnl: 1500.50,
            avg_pnl: 15.005,
            max_win: 250.0,
            max_loss: -100.0,
        };

        service.update_trading_metrics(&stats);

        let metrics = service.get_trading_metrics();
        assert_eq!(metrics.total_pnl, 1500.50);
        assert_eq!(metrics.win_rate, 60.0);
    }

    #[test]
    fn test_update_connection_status() {
        let mut service = MonitoringService::new();

        // Initially disconnected
        assert!(!service.get_connection_status().websocket_connected);
        assert!(!service.get_connection_status().api_responsive);

        // Update to connected
        service.update_connection_status(true, true);

        let status = service.get_connection_status();
        assert!(status.websocket_connected);
        assert!(status.api_responsive);
        assert!(status.last_data_update > 0);

        // Update to partially connected
        service.update_connection_status(true, false);

        let status = service.get_connection_status();
        assert!(status.websocket_connected);
        assert!(!status.api_responsive);
    }

    #[test]
    fn test_record_reconnect() {
        let mut service = MonitoringService::new();

        assert_eq!(service.get_connection_status().reconnect_count, 0);

        service.record_reconnect();
        assert_eq!(service.get_connection_status().reconnect_count, 1);

        service.record_reconnect();
        assert_eq!(service.get_connection_status().reconnect_count, 2);

        service.record_reconnect();
        assert_eq!(service.get_connection_status().reconnect_count, 3);
    }

    #[test]
    fn test_get_system_metrics() {
        let mut service = MonitoringService::new();
        service.update_system_metrics(10, 500);

        let metrics = service.get_system_metrics();

        // Verify we can access all fields
        assert_eq!(metrics.active_positions, 10);
        assert_eq!(metrics.cache_size, 500);
        // uptime_seconds is u64, always >= 0
        assert_eq!(metrics.total_trades, 0);
        assert!(metrics.memory_usage_mb >= 0.0);
        assert!(metrics.cpu_usage_percent >= 0.0);
        assert!(metrics.last_update > 0);
    }

    #[test]
    fn test_get_trading_metrics() {
        let service = MonitoringService::new();
        let metrics = service.get_trading_metrics();

        // Verify we can access all fields
        assert_eq!(metrics.total_pnl, 0.0);
        assert_eq!(metrics.win_rate, 0.0);
        assert_eq!(metrics.avg_trade_duration_minutes, 0.0);
        assert_eq!(metrics.max_drawdown, 0.0);
        assert_eq!(metrics.sharpe_ratio, None);
        assert_eq!(metrics.total_volume, 0.0);
    }

    #[test]
    fn test_get_connection_status() {
        let service = MonitoringService::new();
        let status = service.get_connection_status();

        // Verify we can access all fields
        assert!(!status.websocket_connected);
        assert!(!status.api_responsive);
        assert_eq!(status.last_data_update, 0);
        assert_eq!(status.reconnect_count, 0);
    }

    #[test]
    fn test_log_health_check_no_panic() {
        let mut service = MonitoringService::new();
        service.update_system_metrics(3, 250);

        let stats = PerformanceStats {
            total_trades: 50,
            winning_trades: 30,
            losing_trades: 20,
            win_rate: 60.0,
            total_pnl: 500.0,
            avg_pnl: 10.0,
            max_win: 100.0,
            max_loss: -50.0,
        };
        service.update_trading_metrics(&stats);
        service.update_connection_status(true, true);

        // Should not panic
        service.log_health_check();
    }

    #[test]
    fn test_system_metrics_clone() {
        let metrics = SystemMetrics {
            uptime_seconds: 100,
            active_positions: 5,
            total_trades: 50,
            cache_size: 1000,
            memory_usage_mb: 100.0,
            cpu_usage_percent: 25.0,
            last_update: 1234567890,
        };

        let cloned = metrics.clone();

        assert_eq!(metrics.uptime_seconds, cloned.uptime_seconds);
        assert_eq!(metrics.active_positions, cloned.active_positions);
        assert_eq!(metrics.total_trades, cloned.total_trades);
        assert_eq!(metrics.cache_size, cloned.cache_size);
        assert_eq!(metrics.memory_usage_mb, cloned.memory_usage_mb);
        assert_eq!(metrics.cpu_usage_percent, cloned.cpu_usage_percent);
        assert_eq!(metrics.last_update, cloned.last_update);
    }

    #[test]
    fn test_trading_metrics_clone() {
        let metrics = TradingMetrics {
            total_pnl: 1000.0,
            win_rate: 65.5,
            avg_trade_duration_minutes: 45.5,
            max_drawdown: -200.0,
            sharpe_ratio: Some(1.5),
            total_volume: 50000.0,
        };

        let cloned = metrics.clone();

        assert_eq!(metrics.total_pnl, cloned.total_pnl);
        assert_eq!(metrics.win_rate, cloned.win_rate);
        assert_eq!(
            metrics.avg_trade_duration_minutes,
            cloned.avg_trade_duration_minutes
        );
        assert_eq!(metrics.max_drawdown, cloned.max_drawdown);
        assert_eq!(metrics.sharpe_ratio, cloned.sharpe_ratio);
        assert_eq!(metrics.total_volume, cloned.total_volume);
    }

    #[test]
    fn test_connection_status_clone() {
        let status = ConnectionStatus {
            websocket_connected: true,
            api_responsive: true,
            last_data_update: 1234567890,
            reconnect_count: 5,
        };

        let cloned = status.clone();

        assert_eq!(status.websocket_connected, cloned.websocket_connected);
        assert_eq!(status.api_responsive, cloned.api_responsive);
        assert_eq!(status.last_data_update, cloned.last_data_update);
        assert_eq!(status.reconnect_count, cloned.reconnect_count);
    }

    #[test]
    fn test_system_metrics_debug_format() {
        let metrics = SystemMetrics {
            uptime_seconds: 100,
            active_positions: 5,
            total_trades: 50,
            cache_size: 1000,
            memory_usage_mb: 100.0,
            cpu_usage_percent: 25.0,
            last_update: 1234567890,
        };

        let debug_str = format!("{:?}", metrics);
        assert!(debug_str.contains("SystemMetrics"));
        assert!(debug_str.contains("100"));
    }

    #[test]
    fn test_trading_metrics_serialize_deserialize() {
        let metrics = TradingMetrics {
            total_pnl: 1500.0,
            win_rate: 70.0,
            avg_trade_duration_minutes: 30.0,
            max_drawdown: -300.0,
            sharpe_ratio: Some(2.0),
            total_volume: 100000.0,
        };

        let json_str = serde_json::to_string(&metrics).expect("Failed to serialize");
        let deserialized: TradingMetrics =
            serde_json::from_str(&json_str).expect("Failed to deserialize");

        assert_eq!(metrics.total_pnl, deserialized.total_pnl);
        assert_eq!(metrics.win_rate, deserialized.win_rate);
        assert_eq!(metrics.sharpe_ratio, deserialized.sharpe_ratio);
    }

    #[test]
    fn test_connection_status_serialize_deserialize() {
        let status = ConnectionStatus {
            websocket_connected: true,
            api_responsive: false,
            last_data_update: 1234567890,
            reconnect_count: 10,
        };

        let json_str = serde_json::to_string(&status).expect("Failed to serialize");
        let deserialized: ConnectionStatus =
            serde_json::from_str(&json_str).expect("Failed to deserialize");

        assert_eq!(status.websocket_connected, deserialized.websocket_connected);
        assert_eq!(status.api_responsive, deserialized.api_responsive);
        assert_eq!(status.last_data_update, deserialized.last_data_update);
        assert_eq!(status.reconnect_count, deserialized.reconnect_count);
    }
}
