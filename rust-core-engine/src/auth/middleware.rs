use std::convert::Infallible;
use warp::{Filter, Rejection, Reply};
use serde_json::json;

use super::jwt::{Claims, JwtService};

pub fn with_auth(jwt_service: JwtService) -> impl Filter<Extract = (Claims,), Error = Rejection> + Clone {
    warp::header::<String>("authorization")
        .and(warp::any().map(move || jwt_service.clone()))
        .and_then(authorize)
}

pub fn with_optional_auth(jwt_service: JwtService) -> impl Filter<Extract = (Option<Claims>,), Error = Rejection> + Clone {
    warp::header::optional::<String>("authorization")
        .and(warp::any().map(move || jwt_service.clone()))
        .and_then(optional_authorize)
}

pub fn with_admin_auth(jwt_service: JwtService) -> impl Filter<Extract = (Claims,), Error = Rejection> + Clone {
    warp::header::<String>("authorization")
        .and(warp::any().map(move || jwt_service.clone()))
        .and_then(admin_authorize)
}

async fn authorize(auth_header: String, jwt_service: JwtService) -> Result<Claims, Rejection> {
    let token = match JwtService::extract_token_from_header(&auth_header) {
        Some(token) => token,
        None => {
            return Err(warp::reject::custom(AuthError::InvalidHeader));
        }
    };

    match jwt_service.verify_token(token) {
        Ok(claims) => Ok(claims),
        Err(_) => Err(warp::reject::custom(AuthError::InvalidToken)),
    }
}

async fn optional_authorize(auth_header: Option<String>, jwt_service: JwtService) -> Result<Option<Claims>, Rejection> {
    match auth_header {
        Some(header) => {
            let token = match JwtService::extract_token_from_header(&header) {
                Some(token) => token,
                None => return Ok(None),
            };

            match jwt_service.verify_token(token) {
                Ok(claims) => Ok(Some(claims)),
                Err(_) => Ok(None), // Invalid token is treated as no token
            }
        }
        None => Ok(None),
    }
}

async fn admin_authorize(auth_header: String, jwt_service: JwtService) -> Result<Claims, Rejection> {
    let claims = authorize(auth_header, jwt_service).await?;
    
    if claims.is_admin {
        Ok(claims)
    } else {
        Err(warp::reject::custom(AuthError::InsufficientPermissions))
    }
}

// Custom rejection types
#[derive(Debug)]
pub enum AuthError {
    InvalidHeader,
    InvalidToken,
    InsufficientPermissions,
}

impl warp::reject::Reject for AuthError {}

// Handle auth rejections
pub async fn handle_auth_rejection(err: Rejection) -> Result<impl Reply, Infallible> {
    if let Some(auth_error) = err.find::<AuthError>() {
        let (code, message) = match auth_error {
            AuthError::InvalidHeader => (
                warp::http::StatusCode::UNAUTHORIZED,
                "Invalid authorization header"
            ),
            AuthError::InvalidToken => (
                warp::http::StatusCode::UNAUTHORIZED,
                "Invalid or expired token"
            ),
            AuthError::InsufficientPermissions => (
                warp::http::StatusCode::FORBIDDEN,
                "Insufficient permissions"
            ),
        };

        Ok(warp::reply::with_status(
            warp::reply::json(&json!({
                "success": false,
                "error": message
            })),
            code,
        ))
    } else if err.is_not_found() {
        Ok(warp::reply::with_status(
            warp::reply::json(&json!({
                "success": false,
                "error": "Route not found"
            })),
            warp::http::StatusCode::NOT_FOUND,
        ))
    } else {
        Ok(warp::reply::with_status(
            warp::reply::json(&json!({
                "success": false,
                "error": "Internal server error"
            })),
            warp::http::StatusCode::INTERNAL_SERVER_ERROR,
        ))
    }
} 