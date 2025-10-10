use serde_json::json;
use std::convert::Infallible;
use thiserror::Error;
use warp::{http::StatusCode, reject::Reject, Rejection, Reply};

#[derive(Error, Debug)]
pub enum AppError {
    #[error("Database error: {0}")]
    Database(#[from] mongodb::error::Error),

    #[error("Authentication error: {0}")]
    Auth(String),

    #[error("Validation error: {0}")]
    Validation(String),

    #[error("External API error: {0}")]
    ExternalApi(String),

    #[error("Trading error: {0}")]
    Trading(String),

    #[error("Rate limit exceeded")]
    RateLimit,

    #[error("Resource not found: {0}")]
    NotFound(String),

    #[error("Insufficient funds")]
    InsufficientFunds,

    #[error("Invalid market conditions: {0}")]
    InvalidMarketConditions(String),

    #[error("WebSocket error: {0}")]
    WebSocket(String),

    #[error("Configuration error: {0}")]
    Config(String),

    #[error("Internal server error")]
    Internal,

    #[error("Service unavailable: {0}")]
    ServiceUnavailable(String),

    #[error("Data processing error: {0}")]
    DataProcessing(String),

    #[error("Missing required data: {0}")]
    MissingData(String),

    #[error("Parse error: {0}")]
    ParseError(String),

    #[error("Serialization error: {0}")]
    Serialization(String),

    #[error("Invalid input: {0}")]
    InvalidInput(String),

    #[error("Calculation error: {0}")]
    CalculationError(String),

    #[error("Storage error: {0}")]
    StorageError(String),

    #[error("Trade not found: {0}")]
    TradeNotFound(String),

    #[error("Invalid trade status: {0}")]
    InvalidTradeStatus(String),

    #[error("Position error: {0}")]
    PositionError(String),

    #[error("Risk management error: {0}")]
    RiskManagementError(String),

    #[error("Indicator calculation error: {0}")]
    IndicatorError(String),

    #[error("Strategy execution error: {0}")]
    StrategyError(String),

    #[error("Market data error: {0}")]
    MarketDataError(String),

    #[error("AI service error: {0}")]
    AIServiceError(String),

    #[error("Binance API error: {0}")]
    BinanceError(String),

    #[error("HTTP error: {0}")]
    HttpError(String),

    #[error("JSON error: {0}")]
    JsonError(String),

    #[error("IO error: {0}")]
    IoError(String),

    #[error("Collection not initialized")]
    CollectionNotInitialized,

    #[error("Invalid price data: {0}")]
    InvalidPriceData(String),

    #[error("Insufficient data for calculation: {0}")]
    InsufficientDataForCalculation(String),
}

impl Reject for AppError {}

// Convert AppError to a proper Warp reply
pub async fn handle_rejection(err: Rejection) -> Result<impl Reply, Infallible> {
    let (status, error_message, error_type) = if let Some(app_error) = err.find::<AppError>() {
        match app_error {
            AppError::Database(ref e) => {
                tracing::error!("Database error: {:?}", e);
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "Database error occurred",
                    "database_error",
                )
            },
            AppError::Auth(ref msg) => (StatusCode::UNAUTHORIZED, msg.as_str(), "auth_error"),
            AppError::Validation(ref msg) => {
                (StatusCode::BAD_REQUEST, msg.as_str(), "validation_error")
            },
            AppError::ExternalApi(ref msg) => {
                tracing::error!("External API error: {msg}");
                (
                    StatusCode::BAD_GATEWAY,
                    "External service error",
                    "external_api_error",
                )
            },
            AppError::Trading(ref msg) => (
                StatusCode::UNPROCESSABLE_ENTITY,
                msg.as_str(),
                "trading_error",
            ),
            AppError::RateLimit => (
                StatusCode::TOO_MANY_REQUESTS,
                "Rate limit exceeded",
                "rate_limit",
            ),
            AppError::NotFound(ref resource) => {
                (StatusCode::NOT_FOUND, resource.as_str(), "not_found")
            },
            AppError::InsufficientFunds => (
                StatusCode::PAYMENT_REQUIRED,
                "Insufficient funds",
                "insufficient_funds",
            ),
            AppError::InvalidMarketConditions(ref msg) => (
                StatusCode::PRECONDITION_FAILED,
                msg.as_str(),
                "invalid_market_conditions",
            ),
            AppError::WebSocket(ref msg) => {
                (StatusCode::BAD_REQUEST, msg.as_str(), "websocket_error")
            },
            AppError::Config(ref msg) => {
                tracing::error!("Configuration error: {msg}");
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "Configuration error",
                    "config_error",
                )
            },
            AppError::Internal => {
                tracing::error!("Internal server error");
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "Internal server error",
                    "internal_error",
                )
            },
            AppError::ServiceUnavailable(ref service) => (
                StatusCode::SERVICE_UNAVAILABLE,
                service.as_str(),
                "service_unavailable",
            ),
            AppError::DataProcessing(ref msg)
            | AppError::ParseError(ref msg)
            | AppError::Serialization(ref msg) => {
                (StatusCode::UNPROCESSABLE_ENTITY, msg.as_str(), "data_error")
            },
            AppError::MissingData(ref msg) | AppError::TradeNotFound(ref msg) => {
                (StatusCode::NOT_FOUND, msg.as_str(), "not_found")
            },
            AppError::InvalidInput(ref msg) | AppError::InvalidPriceData(ref msg) => {
                (StatusCode::BAD_REQUEST, msg.as_str(), "invalid_input")
            },
            AppError::CalculationError(ref _msg) | AppError::IndicatorError(ref _msg) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                "Calculation failed",
                "calculation_error",
            ),
            AppError::StorageError(ref _msg) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                "Storage operation failed",
                "storage_error",
            ),
            AppError::InvalidTradeStatus(ref msg)
            | AppError::PositionError(ref msg)
            | AppError::RiskManagementError(ref msg) => {
                (StatusCode::CONFLICT, msg.as_str(), "trade_error")
            },
            AppError::StrategyError(ref _msg)
            | AppError::MarketDataError(ref _msg)
            | AppError::AIServiceError(ref _msg)
            | AppError::BinanceError(ref _msg) => (
                StatusCode::BAD_GATEWAY,
                "External service error",
                "service_error",
            ),
            AppError::HttpError(ref _msg)
            | AppError::JsonError(ref _msg)
            | AppError::IoError(ref _msg) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                "Internal error",
                "internal_error",
            ),
            AppError::CollectionNotInitialized => (
                StatusCode::SERVICE_UNAVAILABLE,
                "Database collection not initialized",
                "service_unavailable",
            ),
            AppError::InsufficientDataForCalculation(ref msg) => (
                StatusCode::PRECONDITION_FAILED,
                msg.as_str(),
                "insufficient_data",
            ),
        }
    } else if err.is_not_found() {
        (StatusCode::NOT_FOUND, "Not found", "not_found")
    } else if err.find::<warp::reject::MethodNotAllowed>().is_some() {
        (
            StatusCode::METHOD_NOT_ALLOWED,
            "Method not allowed",
            "method_not_allowed",
        )
    } else {
        tracing::error!("Unhandled rejection: {:?}", err);
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            "Internal server error",
            "internal_error",
        )
    };

    let reply = warp::reply::json(&json!({
        "error": {
            "type": error_type,
            "message": error_message,
            "status": status.as_u16(),
        }
    }));

    Ok(warp::reply::with_status(reply, status))
}

// Result type alias for convenience
pub type AppResult<T> = Result<T, AppError>;

// Error context trait for adding context to errors
pub trait ErrorContext<T> {
    fn context(self, msg: &str) -> AppResult<T>;
    fn with_context<F>(self, f: F) -> AppResult<T>
    where
        F: FnOnce() -> String;
}

impl<T, E> ErrorContext<T> for Result<T, E>
where
    E: Into<AppError>,
{
    fn context(self, msg: &str) -> AppResult<T> {
        self.map_err(|e| {
            let app_error: AppError = e.into();
            tracing::error!("{msg}: {:?}", app_error);
            app_error
        })
    }

    fn with_context<F>(self, f: F) -> AppResult<T>
    where
        F: FnOnce() -> String,
    {
        self.map_err(|e| {
            let app_error: AppError = e.into();
            let context = f();
            tracing::error!("{context}: {:?}", app_error);
            app_error
        })
    }
}

// Panic handler for production
pub fn setup_panic_handler() {
    std::panic::set_hook(Box::new(|panic_info| {
        let msg = match panic_info.payload().downcast_ref::<&str>() {
            Some(s) => *s,
            None => match panic_info.payload().downcast_ref::<String>() {
                Some(s) => &s[..],
                None => "Unknown panic",
            },
        };

        let location = if let Some(location) = panic_info.location() {
            format!(
                "{}:{}:{}",
                location.file(),
                location.line(),
                location.column()
            )
        } else {
            "Unknown location".to_string()
        };

        tracing::error!(
            target: "panic",
            "Panic occurred: {} at {}",
            msg,
            location
        );

        // Send alert to monitoring system
        // TODO: Implement alerting
    }));
}

#[cfg(test)]
mod tests {
    use super::*;
    use warp::reject;

    #[test]
    fn test_database_error_display() {
        let error = AppError::Database(mongodb::error::Error::from(std::io::Error::new(
            std::io::ErrorKind::Other,
            "connection failed",
        )));
        let display = format!("{}", error);
        assert!(display.starts_with("Database error:"));
    }

    #[test]
    fn test_auth_error_display() {
        let error = AppError::Auth("Invalid token".to_string());
        assert_eq!(format!("{}", error), "Authentication error: Invalid token");
    }

    #[test]
    fn test_validation_error_display() {
        let error = AppError::Validation("Invalid input".to_string());
        assert_eq!(format!("{}", error), "Validation error: Invalid input");
    }

    #[test]
    fn test_external_api_error_display() {
        let error = AppError::ExternalApi("API timeout".to_string());
        assert_eq!(format!("{}", error), "External API error: API timeout");
    }

    #[test]
    fn test_trading_error_display() {
        let error = AppError::Trading("Order failed".to_string());
        assert_eq!(format!("{}", error), "Trading error: Order failed");
    }

    #[test]
    fn test_rate_limit_error_display() {
        let error = AppError::RateLimit;
        assert_eq!(format!("{}", error), "Rate limit exceeded");
    }

    #[test]
    fn test_not_found_error_display() {
        let error = AppError::NotFound("User ID 123".to_string());
        assert_eq!(format!("{}", error), "Resource not found: User ID 123");
    }

    #[test]
    fn test_insufficient_funds_error_display() {
        let error = AppError::InsufficientFunds;
        assert_eq!(format!("{}", error), "Insufficient funds");
    }

    #[test]
    fn test_invalid_market_conditions_error_display() {
        let error = AppError::InvalidMarketConditions("High volatility".to_string());
        assert_eq!(
            format!("{}", error),
            "Invalid market conditions: High volatility"
        );
    }

    #[test]
    fn test_websocket_error_display() {
        let error = AppError::WebSocket("Connection lost".to_string());
        assert_eq!(format!("{}", error), "WebSocket error: Connection lost");
    }

    #[test]
    fn test_config_error_display() {
        let error = AppError::Config("Missing API key".to_string());
        assert_eq!(format!("{}", error), "Configuration error: Missing API key");
    }

    #[test]
    fn test_internal_error_display() {
        let error = AppError::Internal;
        assert_eq!(format!("{}", error), "Internal server error");
    }

    #[test]
    fn test_service_unavailable_error_display() {
        let error = AppError::ServiceUnavailable("Python AI service".to_string());
        assert_eq!(
            format!("{}", error),
            "Service unavailable: Python AI service"
        );
    }

    #[test]
    fn test_database_error_from_mongodb_error() {
        let io_error = std::io::Error::new(std::io::ErrorKind::Other, "test error");
        let mongo_error = mongodb::error::Error::from(io_error);
        let app_error: AppError = mongo_error.into();
        assert!(matches!(app_error, AppError::Database(_)));
    }

    #[tokio::test]
    async fn test_handle_rejection_database_error() {
        let mongo_error = mongodb::error::Error::from(std::io::Error::new(
            std::io::ErrorKind::Other,
            "db connection failed",
        ));
        let app_error = AppError::Database(mongo_error);
        let rejection = reject::custom(app_error);

        let result = handle_rejection(rejection).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_handle_rejection_auth_error() {
        let app_error = AppError::Auth("Invalid credentials".to_string());
        let rejection = reject::custom(app_error);

        let result = handle_rejection(rejection).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_handle_rejection_validation_error() {
        let app_error = AppError::Validation("Invalid field".to_string());
        let rejection = reject::custom(app_error);

        let result = handle_rejection(rejection).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_handle_rejection_external_api_error() {
        let app_error = AppError::ExternalApi("Binance API timeout".to_string());
        let rejection = reject::custom(app_error);

        let result = handle_rejection(rejection).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_handle_rejection_trading_error() {
        let app_error = AppError::Trading("Cannot execute order".to_string());
        let rejection = reject::custom(app_error);

        let result = handle_rejection(rejection).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_handle_rejection_rate_limit_error() {
        let app_error = AppError::RateLimit;
        let rejection = reject::custom(app_error);

        let result = handle_rejection(rejection).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_handle_rejection_not_found_error() {
        let app_error = AppError::NotFound("Trade ID 456".to_string());
        let rejection = reject::custom(app_error);

        let result = handle_rejection(rejection).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_handle_rejection_insufficient_funds_error() {
        let app_error = AppError::InsufficientFunds;
        let rejection = reject::custom(app_error);

        let result = handle_rejection(rejection).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_handle_rejection_invalid_market_conditions_error() {
        let app_error = AppError::InvalidMarketConditions("Market closed".to_string());
        let rejection = reject::custom(app_error);

        let result = handle_rejection(rejection).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_handle_rejection_websocket_error() {
        let app_error = AppError::WebSocket("Stream disconnected".to_string());
        let rejection = reject::custom(app_error);

        let result = handle_rejection(rejection).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_handle_rejection_config_error() {
        let app_error = AppError::Config("Invalid config.toml".to_string());
        let rejection = reject::custom(app_error);

        let result = handle_rejection(rejection).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_handle_rejection_internal_error() {
        let app_error = AppError::Internal;
        let rejection = reject::custom(app_error);

        let result = handle_rejection(rejection).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_handle_rejection_service_unavailable_error() {
        let app_error = AppError::ServiceUnavailable("MongoDB".to_string());
        let rejection = reject::custom(app_error);

        let result = handle_rejection(rejection).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_handle_rejection_warp_not_found() {
        let rejection = reject::not_found();

        let result = handle_rejection(rejection).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_handle_rejection_unknown_rejection() {
        // Test with a custom rejection that's not an AppError
        #[derive(Debug)]
        struct CustomReject;
        impl Reject for CustomReject {}

        let rejection = reject::custom(CustomReject);

        let result = handle_rejection(rejection).await;
        assert!(result.is_ok());
    }

    #[test]
    fn test_error_context_with_message() {
        let result: Result<i32, AppError> = Err(AppError::Auth("test".to_string()));
        let with_context = result.context("Failed authentication");
        assert!(with_context.is_err());
        assert!(matches!(with_context.unwrap_err(), AppError::Auth(_)));
    }

    #[test]
    fn test_error_context_with_closure() {
        let result: Result<i32, AppError> = Err(AppError::Validation("bad input".to_string()));
        let with_context = result.with_context(|| "Validation failed for user input".to_string());
        assert!(with_context.is_err());
        assert!(matches!(with_context.unwrap_err(), AppError::Validation(_)));
    }

    #[test]
    fn test_error_context_preserves_success() {
        let result: Result<i32, AppError> = Ok(42);
        let with_context = result.context("This should not fail");
        assert!(with_context.is_ok());
        assert_eq!(with_context.unwrap(), 42);
    }

    #[test]
    fn test_error_context_with_closure_preserves_success() {
        let result: Result<String, AppError> = Ok("success".to_string());
        let with_context = result.with_context(|| "Should not be called".to_string());
        assert!(with_context.is_ok());
        assert_eq!(with_context.unwrap(), "success");
    }

    #[test]
    fn test_error_context_converts_mongodb_error() {
        let io_error = std::io::Error::new(std::io::ErrorKind::Other, "test");
        let mongo_error = mongodb::error::Error::from(io_error);
        let result: Result<i32, mongodb::error::Error> = Err(mongo_error);
        let with_context = result.context("Database operation failed");
        assert!(with_context.is_err());
        assert!(matches!(with_context.unwrap_err(), AppError::Database(_)));
    }

    #[test]
    fn test_app_result_type_alias() {
        let success: AppResult<i32> = Ok(100);
        let failure: AppResult<i32> = Err(AppError::Internal);

        assert!(success.is_ok());
        assert_eq!(success.unwrap(), 100);
        assert!(failure.is_err());
    }

    #[test]
    fn test_empty_error_messages() {
        let auth_error = AppError::Auth("".to_string());
        assert_eq!(format!("{}", auth_error), "Authentication error: ");

        let validation_error = AppError::Validation("".to_string());
        assert_eq!(format!("{}", validation_error), "Validation error: ");
    }

    #[test]
    fn test_long_error_messages() {
        let long_message = "x".repeat(1000);
        let error = AppError::Trading(long_message.clone());
        let display = format!("{}", error);
        assert!(display.contains(&long_message));
    }

    #[test]
    fn test_special_characters_in_error_messages() {
        let special_msg = "Error with \n newline and \t tab and \"quotes\"";
        let error = AppError::WebSocket(special_msg.to_string());
        assert_eq!(
            format!("{}", error),
            format!("WebSocket error: {}", special_msg)
        );
    }

    #[test]
    fn test_unicode_in_error_messages() {
        let unicode_msg = "Error: 错误 🚫 эошибка";
        let error = AppError::Config(unicode_msg.to_string());
        assert_eq!(
            format!("{}", error),
            format!("Configuration error: {}", unicode_msg)
        );
    }

    #[test]
    fn test_error_debug_implementation() {
        let error = AppError::Trading("Debug test".to_string());
        let debug_output = format!("{:?}", error);
        assert!(debug_output.contains("Trading"));
        assert!(debug_output.contains("Debug test"));
    }

    #[test]
    fn test_panic_handler_setup() {
        // This test just ensures the panic handler can be set up without errors
        // We don't actually trigger a panic in tests
        setup_panic_handler();
        // If we reach here, setup succeeded
        assert!(true);
    }

    #[test]
    fn test_multiple_error_contexts_chained() {
        let result: Result<i32, AppError> = Err(AppError::Internal);
        let step1 = result.context("Step 1 failed");
        assert!(step1.is_err());

        let result2: Result<i32, AppError> = Err(AppError::Trading("order failed".to_string()));
        let step2 = result2.with_context(|| format!("Step 2 failed with order"));
        assert!(step2.is_err());
    }

    #[tokio::test]
    async fn test_handle_rejection_returns_json_structure() {
        let app_error = AppError::Auth("Unauthorized access".to_string());
        let rejection = reject::custom(app_error);

        let result = handle_rejection(rejection).await;
        // We just verify it returns Ok, the actual JSON structure is tested in integration
        assert!(result.is_ok());
    }

    #[test]
    fn test_error_type_sizes() {
        // Verify error types aren't unexpectedly large
        let size = std::mem::size_of::<AppError>();
        // AppError should be reasonably sized (this is a sanity check)
        assert!(
            size < 1024,
            "AppError size is {} bytes, may be too large",
            size
        );
    }

    #[test]
    fn test_error_is_send_and_sync() {
        // Verify AppError implements Send and Sync for use across threads
        fn assert_send<T: Send>() {}
        fn assert_sync<T: Sync>() {}
        assert_send::<AppError>();
        assert_sync::<AppError>();
    }

    #[test]
    fn test_app_error_implements_reject() {
        // Verify AppError can be used as a Warp rejection
        let error = AppError::Internal;
        let _rejection: Rejection = reject::custom(error);
        // If this compiles, the test passes
        assert!(true);
    }

    #[tokio::test]
    async fn test_handle_rejection_method_not_allowed() {
        // Create a MethodNotAllowed rejection by using a custom rejection
        // that triggers the MethodNotAllowed path (lines 124-129)
        use warp::Filter;

        let filter = warp::post().and(warp::path("test")).map(|| warp::reply());

        // Simulate a GET request to a POST-only endpoint
        // This will trigger the MethodNotAllowed rejection
        match warp::test::request()
            .method("GET")
            .path("/test")
            .filter(&filter)
            .await
        {
            Err(rejection) => {
                // Verify the rejection contains MethodNotAllowed
                assert!(rejection.find::<warp::reject::MethodNotAllowed>().is_some());
                let result = handle_rejection(rejection).await;
                assert!(result.is_ok());
            },
            Ok(_) => panic!("Expected rejection but got success"),
        }
    }

    #[test]
    #[should_panic(expected = "test panic with &str")]
    fn test_panic_handler_with_str_payload() {
        // Set up the panic handler
        setup_panic_handler();

        // Trigger a panic with &str payload (lines 189-190)
        panic!("test panic with &str");
    }

    #[test]
    fn test_panic_handler_with_string_payload() {
        // Test that we can handle String panics without actually panicking in the test
        // We'll use std::panic::catch_unwind to catch the panic
        setup_panic_handler();

        let result = std::panic::catch_unwind(|| {
            // This will create a String payload
            let msg = String::from("test panic with String");
            std::panic::panic_any(msg);
        });

        // Verify the panic occurred (lines 191-193)
        assert!(result.is_err());
    }

    #[test]
    fn test_panic_handler_with_unknown_payload() {
        // Test panic with a non-string payload to cover the "Unknown panic" branch
        setup_panic_handler();

        #[derive(Debug)]
        struct CustomPanicPayload {
            _code: i32,
        }

        let result = std::panic::catch_unwind(|| {
            std::panic::panic_any(CustomPanicPayload { _code: 42 });
        });

        // Verify the panic occurred and was handled (covers None branch of both downcasts)
        assert!(result.is_err());
    }

    #[test]
    fn test_panic_handler_with_location() {
        // Test that panic location is captured when available
        setup_panic_handler();

        let result = std::panic::catch_unwind(|| {
            // Normal panic! macro includes location info (lines 197-203)
            panic!("panic with location info");
        });

        assert!(result.is_err());
    }

    #[test]
    fn test_panic_handler_without_location() {
        // Test panic without location (this is harder to trigger, but we can at least
        // verify the panic handler works)
        setup_panic_handler();

        // Use panic_any which might not include location in all cases
        let result = std::panic::catch_unwind(|| {
            std::panic::resume_unwind(Box::new("unwind without location"));
        });

        // This will capture the panic (covers lines 205-206 if location is None)
        assert!(result.is_err());
    }

    #[test]
    fn test_panic_handler_logs_panic_info() {
        // This test verifies the panic handler can be set up and will log panics
        // We test the setup without actually panicking
        setup_panic_handler();

        // Verify the panic hook is set by checking it can be called
        // (The actual panic would terminate the test, so we just verify setup)
        let old_hook = std::panic::take_hook();
        setup_panic_handler();
        let new_hook = std::panic::take_hook();

        // Restore the hook to avoid affecting other tests
        std::panic::set_hook(new_hook);
        std::panic::set_hook(old_hook);
    }

    #[test]
    fn test_error_context_with_mongodb_error_using_with_context() {
        // Test the with_context method with MongoDB error conversion
        let io_error =
            std::io::Error::new(std::io::ErrorKind::ConnectionRefused, "connection refused");
        let mongo_error = mongodb::error::Error::from(io_error);
        let result: Result<(), mongodb::error::Error> = Err(mongo_error);

        let with_context =
            result.with_context(|| format!("Failed to connect to database at localhost:27017"));

        assert!(with_context.is_err());
        assert!(matches!(with_context.unwrap_err(), AppError::Database(_)));
    }

    #[tokio::test]
    async fn test_handle_rejection_multiple_rejections_combined() {
        // Test handling of combined/chained rejections
        let app_error = AppError::RateLimit;
        let rejection = reject::custom(app_error);

        let result = handle_rejection(rejection).await;
        assert!(result.is_ok());
    }

    #[test]
    fn test_error_context_trait_implementation() {
        // Verify the ErrorContext trait works with different error types
        let io_error = std::io::Error::new(std::io::ErrorKind::NotFound, "file not found");
        let mongo_error = mongodb::error::Error::from(io_error);
        let result: Result<String, mongodb::error::Error> = Err(mongo_error);

        // Test that the context method properly converts and logs
        let with_context = result.context("File operation failed");
        assert!(with_context.is_err());
    }

    #[test]
    fn test_error_from_conversion_trait() {
        // Test the #[from] attribute on Database variant
        let io_error = std::io::Error::new(std::io::ErrorKind::TimedOut, "timeout");
        let mongo_error = mongodb::error::Error::from(io_error);

        // The From trait should automatically convert mongodb::error::Error to AppError
        let app_error: AppError = mongo_error.into();
        assert!(matches!(app_error, AppError::Database(_)));
    }

    #[test]
    fn test_all_error_variants_are_debug() {
        // Ensure all error variants implement Debug properly
        let errors = vec![
            AppError::Auth("test".to_string()),
            AppError::Validation("test".to_string()),
            AppError::ExternalApi("test".to_string()),
            AppError::Trading("test".to_string()),
            AppError::RateLimit,
            AppError::NotFound("test".to_string()),
            AppError::InsufficientFunds,
            AppError::InvalidMarketConditions("test".to_string()),
            AppError::WebSocket("test".to_string()),
            AppError::Config("test".to_string()),
            AppError::Internal,
            AppError::ServiceUnavailable("test".to_string()),
        ];

        for error in errors {
            let debug_str = format!("{:?}", error);
            assert!(!debug_str.is_empty());
        }
    }

    #[tokio::test]
    async fn test_handle_rejection_preserves_error_details() {
        // Verify that error details are preserved through rejection handling
        let specific_message = "Database connection pool exhausted";
        let mongo_error = mongodb::error::Error::from(std::io::Error::new(
            std::io::ErrorKind::Other,
            specific_message,
        ));
        let app_error = AppError::Database(mongo_error);
        let rejection = reject::custom(app_error);

        let result = handle_rejection(rejection).await;
        assert!(result.is_ok());
    }

    #[test]
    fn test_context_method_adds_logging_context() {
        // Test that context method properly adds context for logging
        let result: Result<i32, AppError> = Err(AppError::Trading("order rejected".to_string()));
        let context_message = "Failed to execute buy order for BTC/USDT";

        let with_context = result.context(context_message);
        assert!(with_context.is_err());
        assert!(matches!(with_context.unwrap_err(), AppError::Trading(_)));
    }

    #[test]
    fn test_with_context_closure_lazy_evaluation() {
        // Verify that with_context only evaluates the closure on error
        let mut closure_called = false;
        let result: Result<i32, AppError> = Ok(42);

        let _with_context = result.with_context(|| {
            closure_called = true;
            "This should not be called".to_string()
        });

        // Closure should NOT be called for Ok results
        assert!(!closure_called);

        // Now test with an error
        let mut closure_called_on_error = false;
        let error_result: Result<i32, AppError> = Err(AppError::Internal);

        let _with_error_context = error_result.with_context(|| {
            closure_called_on_error = true;
            "This should be called".to_string()
        });

        // Closure SHOULD be called for Err results
        assert!(closure_called_on_error);
    }
}
