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
            }
            AppError::Auth(ref msg) => (StatusCode::UNAUTHORIZED, msg.as_str(), "auth_error"),
            AppError::Validation(ref msg) => {
                (StatusCode::BAD_REQUEST, msg.as_str(), "validation_error")
            }
            AppError::ExternalApi(ref msg) => {
                tracing::error!("External API error: {msg}");
                (
                    StatusCode::BAD_GATEWAY,
                    "External service error",
                    "external_api_error",
                )
            }
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
            }
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
            }
            AppError::Config(ref msg) => {
                tracing::error!("Configuration error: {msg}");
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "Configuration error",
                    "config_error",
                )
            }
            AppError::Internal => {
                tracing::error!("Internal server error");
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "Internal server error",
                    "internal_error",
                )
            }
            AppError::ServiceUnavailable(ref service) => (
                StatusCode::SERVICE_UNAVAILABLE,
                service.as_str(),
                "service_unavailable",
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
