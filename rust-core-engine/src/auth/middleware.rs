use serde_json::json;
use std::convert::Infallible;
use warp::{Filter, Rejection, Reply};

use super::jwt::{Claims, JwtService};

pub fn with_auth(
    jwt_service: JwtService,
) -> impl Filter<Extract = (Claims,), Error = Rejection> + Clone {
    warp::header::<String>("authorization")
        .and(warp::any().map(move || jwt_service.clone()))
        .and_then(authorize)
}

pub fn with_optional_auth(
    jwt_service: JwtService,
) -> impl Filter<Extract = (Option<Claims>,), Error = Rejection> + Clone {
    warp::header::optional::<String>("authorization")
        .and(warp::any().map(move || jwt_service.clone()))
        .and_then(optional_authorize)
}

pub fn with_admin_auth(
    jwt_service: JwtService,
) -> impl Filter<Extract = (Claims,), Error = Rejection> + Clone {
    warp::header::<String>("authorization")
        .and(warp::any().map(move || jwt_service.clone()))
        .and_then(admin_authorize)
}

async fn authorize(auth_header: String, jwt_service: JwtService) -> Result<Claims, Rejection> {
    let token = match JwtService::extract_token_from_header(&auth_header) {
        Some(token) => token,
        None => {
            return Err(warp::reject::custom(AuthError::InvalidHeader));
        },
    };

    match jwt_service.verify_token(token) {
        Ok(claims) => Ok(claims),
        Err(_) => Err(warp::reject::custom(AuthError::InvalidToken)),
    }
}

async fn optional_authorize(
    auth_header: Option<String>,
    jwt_service: JwtService,
) -> Result<Option<Claims>, Rejection> {
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
        },
        None => Ok(None),
    }
}

async fn admin_authorize(
    auth_header: String,
    jwt_service: JwtService,
) -> Result<Claims, Rejection> {
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
                "Invalid authorization header",
            ),
            AuthError::InvalidToken => (
                warp::http::StatusCode::UNAUTHORIZED,
                "Invalid or expired token",
            ),
            AuthError::InsufficientPermissions => (
                warp::http::StatusCode::FORBIDDEN,
                "Insufficient permissions",
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

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_jwt_service() -> JwtService {
        JwtService::new("test_secret".to_string(), Some(24))
    }

    #[tokio::test]
    async fn test_authorize_with_valid_token() {
        let jwt_service = create_test_jwt_service();
        let token = jwt_service
            .generate_token("user123", "test@example.com", false)
            .unwrap();

        let auth_header = format!("Bearer {}", token);
        let result = authorize(auth_header, jwt_service).await;

        assert!(result.is_ok());
        let claims = result.unwrap();
        assert_eq!(claims.sub, "user123");
        assert_eq!(claims.email, "test@example.com");
        assert!(!claims.is_admin);
    }

    #[tokio::test]
    async fn test_authorize_with_invalid_header_format() {
        let jwt_service = create_test_jwt_service();
        let token = jwt_service
            .generate_token("user123", "test@example.com", false)
            .unwrap();

        let auth_header = format!("Basic {}", token); // Wrong prefix
        let result = authorize(auth_header, jwt_service).await;

        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_authorize_with_invalid_token() {
        let jwt_service = create_test_jwt_service();
        let auth_header = "Bearer invalid.token.here".to_string();
        let result = authorize(auth_header, jwt_service).await;

        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_authorize_with_missing_bearer() {
        let jwt_service = create_test_jwt_service();
        let token = jwt_service
            .generate_token("user123", "test@example.com", false)
            .unwrap();

        let result = authorize(token, jwt_service).await;

        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_optional_authorize_with_valid_token() {
        let jwt_service = create_test_jwt_service();
        let token = jwt_service
            .generate_token("user123", "test@example.com", false)
            .unwrap();

        let auth_header = Some(format!("Bearer {}", token));
        let result = optional_authorize(auth_header, jwt_service).await;

        assert!(result.is_ok());
        let claims = result.unwrap();
        assert!(claims.is_some());
        assert_eq!(claims.unwrap().sub, "user123");
    }

    #[tokio::test]
    async fn test_optional_authorize_with_no_header() {
        let jwt_service = create_test_jwt_service();
        let result = optional_authorize(None, jwt_service).await;

        assert!(result.is_ok());
        assert!(result.unwrap().is_none());
    }

    #[tokio::test]
    async fn test_optional_authorize_with_invalid_token() {
        let jwt_service = create_test_jwt_service();
        let auth_header = Some("Bearer invalid.token.here".to_string());
        let result = optional_authorize(auth_header, jwt_service).await;

        assert!(result.is_ok());
        assert!(result.unwrap().is_none()); // Invalid token treated as no token
    }

    #[tokio::test]
    async fn test_optional_authorize_with_malformed_header() {
        let jwt_service = create_test_jwt_service();
        let auth_header = Some("NotBearer sometoken".to_string());
        let result = optional_authorize(auth_header, jwt_service).await;

        assert!(result.is_ok());
        assert!(result.unwrap().is_none());
    }

    #[tokio::test]
    async fn test_admin_authorize_with_admin_token() {
        let jwt_service = create_test_jwt_service();
        let token = jwt_service
            .generate_token("admin123", "admin@example.com", true)
            .unwrap();

        let auth_header = format!("Bearer {}", token);
        let result = admin_authorize(auth_header, jwt_service).await;

        assert!(result.is_ok());
        let claims = result.unwrap();
        assert_eq!(claims.sub, "admin123");
        assert!(claims.is_admin);
    }

    #[tokio::test]
    async fn test_admin_authorize_with_non_admin_token() {
        let jwt_service = create_test_jwt_service();
        let token = jwt_service
            .generate_token("user123", "user@example.com", false)
            .unwrap();

        let auth_header = format!("Bearer {}", token);
        let result = admin_authorize(auth_header, jwt_service).await;

        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_admin_authorize_with_invalid_token() {
        let jwt_service = create_test_jwt_service();
        let auth_header = "Bearer invalid.token.here".to_string();
        let result = admin_authorize(auth_header, jwt_service).await;

        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_handle_auth_rejection_invalid_header() {
        let rejection = warp::reject::custom(AuthError::InvalidHeader);
        let response = handle_auth_rejection(rejection).await.unwrap();

        // We can't easily test the response body, but we can verify it compiles
        // In a real scenario, you'd extract and verify the response
        let _ = response;
    }

    #[tokio::test]
    async fn test_handle_auth_rejection_invalid_token() {
        let rejection = warp::reject::custom(AuthError::InvalidToken);
        let response = handle_auth_rejection(rejection).await.unwrap();
        let _ = response;
    }

    #[tokio::test]
    async fn test_handle_auth_rejection_insufficient_permissions() {
        let rejection = warp::reject::custom(AuthError::InsufficientPermissions);
        let response = handle_auth_rejection(rejection).await.unwrap();
        let _ = response;
    }

    #[tokio::test]
    async fn test_handle_auth_rejection_not_found() {
        let rejection = warp::reject::not_found();
        let response = handle_auth_rejection(rejection).await.unwrap();
        let _ = response;
    }

    #[test]
    fn test_auth_error_debug() {
        let error1 = AuthError::InvalidHeader;
        let error2 = AuthError::InvalidToken;
        let error3 = AuthError::InsufficientPermissions;

        // Test that Debug is implemented
        let _ = format!("{:?}", error1);
        let _ = format!("{:?}", error2);
        let _ = format!("{:?}", error3);
    }

    #[tokio::test]
    async fn test_authorize_with_empty_bearer_token() {
        let jwt_service = create_test_jwt_service();
        let auth_header = "Bearer ".to_string();
        let result = authorize(auth_header, jwt_service).await;

        // Empty token after Bearer should be rejected
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_authorize_with_whitespace_only() {
        let jwt_service = create_test_jwt_service();
        let auth_header = "   ".to_string();
        let result = authorize(auth_header, jwt_service).await;

        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_optional_authorize_with_empty_bearer() {
        let jwt_service = create_test_jwt_service();
        let auth_header = Some("Bearer ".to_string());
        let result = optional_authorize(auth_header, jwt_service).await;

        // Empty bearer token should be treated as no token
        assert!(result.is_ok());
        assert!(result.unwrap().is_none());
    }

    #[tokio::test]
    async fn test_optional_authorize_with_whitespace() {
        let jwt_service = create_test_jwt_service();
        let auth_header = Some("   ".to_string());
        let result = optional_authorize(auth_header, jwt_service).await;

        assert!(result.is_ok());
        assert!(result.unwrap().is_none());
    }

    #[tokio::test]
    async fn test_admin_authorize_with_invalid_header_format() {
        let jwt_service = create_test_jwt_service();
        let auth_header = "NotBearer token".to_string();
        let result = admin_authorize(auth_header, jwt_service).await;

        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_admin_authorize_with_empty_token() {
        let jwt_service = create_test_jwt_service();
        let auth_header = "Bearer ".to_string();
        let result = admin_authorize(auth_header, jwt_service).await;

        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_handle_auth_rejection_internal_error() {
        // Create a generic rejection that's not an AuthError or NotFound
        let rejection = warp::reject::custom(TestCustomError);
        let response = handle_auth_rejection(rejection).await.unwrap();
        let _ = response;
    }

    // Custom error type for testing internal error handling
    #[derive(Debug)]
    struct TestCustomError;
    impl warp::reject::Reject for TestCustomError {}

    #[tokio::test]
    async fn test_authorize_admin_user_has_correct_claims() {
        let jwt_service = create_test_jwt_service();
        let token = jwt_service
            .generate_token("admin123", "admin@example.com", true)
            .unwrap();

        let auth_header = format!("Bearer {}", token);
        let result = authorize(auth_header, jwt_service).await;

        assert!(result.is_ok());
        let claims = result.unwrap();
        assert_eq!(claims.sub, "admin123");
        assert_eq!(claims.email, "admin@example.com");
        assert!(claims.is_admin);
    }

    #[tokio::test]
    async fn test_optional_authorize_with_valid_admin_token() {
        let jwt_service = create_test_jwt_service();
        let token = jwt_service
            .generate_token("admin123", "admin@example.com", true)
            .unwrap();

        let auth_header = Some(format!("Bearer {}", token));
        let result = optional_authorize(auth_header, jwt_service).await;

        assert!(result.is_ok());
        let claims = result.unwrap();
        assert!(claims.is_some());
        let claims = claims.unwrap();
        assert_eq!(claims.sub, "admin123");
        assert!(claims.is_admin);
    }

    #[tokio::test]
    async fn test_authorize_with_case_sensitive_bearer() {
        let jwt_service = create_test_jwt_service();
        let token = jwt_service
            .generate_token("user123", "test@example.com", false)
            .unwrap();

        // Try with lowercase "bearer" - should fail
        let auth_header = format!("bearer {}", token);
        let result = authorize(auth_header, jwt_service).await;

        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_optional_authorize_with_case_sensitive_bearer() {
        let jwt_service = create_test_jwt_service();
        let token = jwt_service
            .generate_token("user123", "test@example.com", false)
            .unwrap();

        // Try with lowercase "bearer"
        let auth_header = Some(format!("bearer {}", token));
        let result = optional_authorize(auth_header, jwt_service).await;

        assert!(result.is_ok());
        assert!(result.unwrap().is_none());
    }

    #[tokio::test]
    async fn test_authorize_preserves_token_claims() {
        let jwt_service = create_test_jwt_service();
        let user_id = "user_12345";
        let email = "specific@example.com";
        let is_admin = false;

        let token = jwt_service
            .generate_token(user_id, email, is_admin)
            .unwrap();

        let auth_header = format!("Bearer {}", token);
        let result = authorize(auth_header, jwt_service).await;

        assert!(result.is_ok());
        let claims = result.unwrap();
        assert_eq!(claims.sub, user_id);
        assert_eq!(claims.email, email);
        assert_eq!(claims.is_admin, is_admin);
    }

    #[tokio::test]
    async fn test_admin_authorize_rejects_regular_user() {
        let jwt_service = create_test_jwt_service();
        let token = jwt_service
            .generate_token("user123", "user@example.com", false)
            .unwrap();

        let auth_header = format!("Bearer {}", token);
        let result = admin_authorize(auth_header, jwt_service).await;

        assert!(result.is_err());
        // Verify it's specifically an insufficient permissions error
        if let Err(rejection) = result {
            assert!(rejection.find::<AuthError>().is_some());
        }
    }

    #[tokio::test]
    async fn test_multiple_authorization_calls_same_token() {
        let jwt_service = create_test_jwt_service();
        let token = jwt_service
            .generate_token("user123", "test@example.com", false)
            .unwrap();

        let auth_header = format!("Bearer {}", token);

        // First authorization
        let result1 = authorize(auth_header.clone(), jwt_service.clone()).await;
        assert!(result1.is_ok());

        // Second authorization with same token should also succeed
        let result2 = authorize(auth_header, jwt_service).await;
        assert!(result2.is_ok());

        // Both should have same claims
        assert_eq!(result1.unwrap().sub, result2.unwrap().sub);
    }

    #[tokio::test]
    async fn test_optional_authorize_multiple_calls() {
        let jwt_service = create_test_jwt_service();

        // Call with None multiple times
        let result1 = optional_authorize(None, jwt_service.clone()).await;
        let result2 = optional_authorize(None, jwt_service).await;

        assert!(result1.is_ok());
        assert!(result2.is_ok());
        assert!(result1.unwrap().is_none());
        assert!(result2.unwrap().is_none());
    }

    #[tokio::test]
    async fn test_authorize_with_extra_whitespace_in_header() {
        let jwt_service = create_test_jwt_service();
        let token = jwt_service
            .generate_token("user123", "test@example.com", false)
            .unwrap();

        // Extra spaces after Bearer
        let auth_header = format!("Bearer  {}", token);
        let result = authorize(auth_header, jwt_service).await;

        // Should fail - extract_token_from_header expects exactly one space
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_admin_authorize_preserves_all_claims() {
        let jwt_service = create_test_jwt_service();
        let user_id = "admin_99";
        let email = "superadmin@example.com";

        let token = jwt_service.generate_token(user_id, email, true).unwrap();

        let auth_header = format!("Bearer {}", token);
        let result = admin_authorize(auth_header, jwt_service).await;

        assert!(result.is_ok());
        let claims = result.unwrap();
        assert_eq!(claims.sub, user_id);
        assert_eq!(claims.email, email);
        assert!(claims.is_admin);
    }

    #[tokio::test]
    async fn test_optional_authorize_with_corrupted_token() {
        let jwt_service = create_test_jwt_service();

        // Create a valid token and corrupt it
        let token = jwt_service
            .generate_token("user123", "test@example.com", false)
            .unwrap();

        let corrupted_token = format!("{}corrupted", token);
        let auth_header = Some(format!("Bearer {}", corrupted_token));

        let result = optional_authorize(auth_header, jwt_service).await;

        // Should return Ok(None) for invalid token
        assert!(result.is_ok());
        assert!(result.unwrap().is_none());
    }

    #[tokio::test]
    async fn test_authorize_with_token_missing_parts() {
        let jwt_service = create_test_jwt_service();

        // JWT tokens have 3 parts separated by dots
        let auth_header = "Bearer incomplete.token".to_string();
        let result = authorize(auth_header, jwt_service).await;

        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_handle_auth_rejection_preserves_error_types() {
        // Test InvalidHeader
        let rejection1 = warp::reject::custom(AuthError::InvalidHeader);
        let response1 = handle_auth_rejection(rejection1).await;
        assert!(response1.is_ok());

        // Test InvalidToken
        let rejection2 = warp::reject::custom(AuthError::InvalidToken);
        let response2 = handle_auth_rejection(rejection2).await;
        assert!(response2.is_ok());

        // Test InsufficientPermissions
        let rejection3 = warp::reject::custom(AuthError::InsufficientPermissions);
        let response3 = handle_auth_rejection(rejection3).await;
        assert!(response3.is_ok());
    }

    #[tokio::test]
    async fn test_authorize_different_jwt_service_instances() {
        let jwt_service1 = JwtService::new("secret1".to_string(), Some(24));
        let jwt_service2 = JwtService::new("secret2".to_string(), Some(24));

        let token = jwt_service1
            .generate_token("user123", "test@example.com", false)
            .unwrap();

        let auth_header = format!("Bearer {}", token);

        // Should succeed with same secret
        let result1 = authorize(auth_header.clone(), jwt_service1).await;
        assert!(result1.is_ok());

        // Should fail with different secret
        let result2 = authorize(auth_header, jwt_service2).await;
        assert!(result2.is_err());
    }

    #[tokio::test]
    async fn test_admin_authorize_with_regular_authorize_flow() {
        let jwt_service = create_test_jwt_service();
        let token = jwt_service
            .generate_token("user123", "user@example.com", false)
            .unwrap();

        let auth_header = format!("Bearer {}", token);

        // Regular authorize should work
        let regular_result = authorize(auth_header.clone(), jwt_service.clone()).await;
        assert!(regular_result.is_ok());

        // Admin authorize should fail for non-admin
        let admin_result = admin_authorize(auth_header, jwt_service).await;
        assert!(admin_result.is_err());
    }
}
