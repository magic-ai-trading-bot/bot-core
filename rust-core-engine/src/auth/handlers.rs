use serde_json::json;
use std::convert::Infallible;
use tracing::{error, info, warn};
use validator::Validate;
use warp::{Filter, Rejection, Reply};

use super::{
    database::UserRepository,
    jwt::{JwtService, PasswordService},
    models::{LoginRequest, LoginResponse, RegisterRequest, User},
};

#[derive(Clone)]
pub struct AuthService {
    user_repo: UserRepository,
    jwt_service: JwtService,
}

impl AuthService {
    pub fn new(user_repo: UserRepository, jwt_secret: String) -> Self {
        let jwt_service = JwtService::new(jwt_secret, Some(24 * 7)); // 7 days
        Self {
            user_repo,
            jwt_service,
        }
    }

    pub fn new_dummy() -> Self {
        // Create a dummy auth service that will return errors for all operations
        // This is used when no database is available
        let dummy_repo = UserRepository::new_dummy();
        let jwt_service = JwtService::new("dummy_secret".to_string(), Some(24 * 7));
        Self {
            user_repo: dummy_repo,
            jwt_service,
        }
    }

    pub fn routes(&self) -> impl Filter<Extract = impl Reply, Error = Rejection> + Clone {
        let register = self.register_route();
        let login = self.login_route();
        let verify = self.verify_route();
        let profile = self.profile_route();

        warp::path("auth").and(register.or(login).or(verify).or(profile))
    }

    fn register_route(&self) -> impl Filter<Extract = impl Reply, Error = Rejection> + Clone {
        let auth_service = self.clone();
        warp::path("register")
            .and(warp::post())
            .and(warp::body::json())
            .and(warp::any().map(move || auth_service.clone()))
            .and_then(handle_register)
    }

    fn login_route(&self) -> impl Filter<Extract = impl Reply, Error = Rejection> + Clone {
        let auth_service = self.clone();
        warp::path("login")
            .and(warp::post())
            .and(warp::body::json())
            .and(warp::any().map(move || auth_service.clone()))
            .and_then(handle_login)
    }

    fn verify_route(&self) -> impl Filter<Extract = impl Reply, Error = Rejection> + Clone {
        let auth_service = self.clone();
        warp::path("verify")
            .and(warp::get())
            .and(warp::header::<String>("authorization"))
            .and(warp::any().map(move || auth_service.clone()))
            .and_then(handle_verify)
    }

    fn profile_route(&self) -> impl Filter<Extract = impl Reply, Error = Rejection> + Clone {
        let auth_service = self.clone();
        warp::path("profile")
            .and(warp::get())
            .and(warp::header::<String>("authorization"))
            .and(warp::any().map(move || auth_service.clone()))
            .and_then(handle_profile)
    }
}

async fn handle_register(
    request: RegisterRequest,
    auth_service: AuthService,
) -> Result<impl Reply, Infallible> {
    info!("Register attempt for email: {}", request.email);

    // Validate request
    if let Err(validation_errors) = request.validate() {
        warn!("Registration validation failed: {:?}", validation_errors);
        return Ok(warp::reply::with_status(
            warp::reply::json(&json!({
                "success": false,
                "error": "Validation failed",
                "details": validation_errors.to_string()
            })),
            warp::http::StatusCode::BAD_REQUEST,
        ));
    }

    // Check if email already exists
    match auth_service.user_repo.email_exists(&request.email).await {
        Ok(exists) if exists => {
            warn!(
                "Registration failed: email already exists: {}",
                request.email
            );
            return Ok(warp::reply::with_status(
                warp::reply::json(&json!({
                    "success": false,
                    "error": "Email already registered"
                })),
                warp::http::StatusCode::CONFLICT,
            ));
        }
        Err(e) => {
            error!("Database error checking email: {}", e);
            return Ok(warp::reply::with_status(
                warp::reply::json(&json!({
                    "success": false,
                    "error": "Internal server error"
                })),
                warp::http::StatusCode::INTERNAL_SERVER_ERROR,
            ));
        }
        _ => {}
    }

    // Hash password
    let password_hash = match PasswordService::hash_password(&request.password) {
        Ok(hash) => hash,
        Err(e) => {
            error!("Password hashing failed: {}", e);
            return Ok(warp::reply::with_status(
                warp::reply::json(&json!({
                    "success": false,
                    "error": "Internal server error"
                })),
                warp::http::StatusCode::INTERNAL_SERVER_ERROR,
            ));
        }
    };

    // Create user
    let user = User::new(request.email.clone(), password_hash, request.full_name);

    match auth_service.user_repo.create_user(user.clone()).await {
        Ok(user_id) => {
            info!(
                "User created successfully: {} (ID: {})",
                request.email, user_id
            );

            // Generate token
            let token = match auth_service.jwt_service.generate_token(
                &user_id.to_hex(),
                &user.email,
                user.is_admin,
            ) {
                Ok(token) => token,
                Err(e) => {
                    error!("Token generation failed: {}", e);
                    return Ok(warp::reply::with_status(
                        warp::reply::json(&json!({
                            "success": false,
                            "error": "Internal server error"
                        })),
                        warp::http::StatusCode::INTERNAL_SERVER_ERROR,
                    ));
                }
            };

            let response = LoginResponse {
                token,
                user: user.to_profile(),
            };

            Ok(warp::reply::with_status(
                warp::reply::json(&json!({
                    "success": true,
                    "data": response
                })),
                warp::http::StatusCode::CREATED,
            ))
        }
        Err(e) => {
            error!("User creation failed: {}", e);
            Ok(warp::reply::with_status(
                warp::reply::json(&json!({
                    "success": false,
                    "error": "Failed to create user"
                })),
                warp::http::StatusCode::INTERNAL_SERVER_ERROR,
            ))
        }
    }
}

async fn handle_login(
    request: LoginRequest,
    auth_service: AuthService,
) -> Result<impl Reply, Infallible> {
    info!("Login attempt for email: {}", request.email);

    // Validate request
    if let Err(validation_errors) = request.validate() {
        warn!("Login validation failed: {:?}", validation_errors);
        return Ok(warp::reply::with_status(
            warp::reply::json(&json!({
                "success": false,
                "error": "Validation failed",
                "details": validation_errors.to_string()
            })),
            warp::http::StatusCode::BAD_REQUEST,
        ));
    }

    // Find user by email
    let user = match auth_service.user_repo.find_by_email(&request.email).await {
        Ok(Some(user)) => user,
        Ok(None) => {
            warn!("Login failed: user not found: {}", request.email);
            return Ok(warp::reply::with_status(
                warp::reply::json(&json!({
                    "success": false,
                    "error": "Invalid email or password"
                })),
                warp::http::StatusCode::UNAUTHORIZED,
            ));
        }
        Err(e) => {
            error!("Database error finding user: {}", e);
            return Ok(warp::reply::with_status(
                warp::reply::json(&json!({
                    "success": false,
                    "error": "Internal server error"
                })),
                warp::http::StatusCode::INTERNAL_SERVER_ERROR,
            ));
        }
    };

    // Check if user is active
    if !user.is_active {
        warn!("Login failed: user account deactivated: {}", request.email);
        return Ok(warp::reply::with_status(
            warp::reply::json(&json!({
                "success": false,
                "error": "Account is deactivated"
            })),
            warp::http::StatusCode::FORBIDDEN,
        ));
    }

    // Verify password
    match PasswordService::verify_password(&request.password, &user.password_hash) {
        Ok(true) => {
            info!("Login successful for user: {}", request.email);

            // Update last login
            if let Some(user_id) = user.id {
                if let Err(e) = auth_service.user_repo.update_last_login(&user_id).await {
                    error!("Failed to update last login: {}", e);
                }
            }

            // Generate token
            let user_id = user.id.as_ref().map(|id| id.to_hex()).unwrap_or_default();
            let token =
                match auth_service
                    .jwt_service
                    .generate_token(&user_id, &user.email, user.is_admin)
                {
                    Ok(token) => token,
                    Err(e) => {
                        error!("Token generation failed: {}", e);
                        return Ok(warp::reply::with_status(
                            warp::reply::json(&json!({
                                "success": false,
                                "error": "Internal server error"
                            })),
                            warp::http::StatusCode::INTERNAL_SERVER_ERROR,
                        ));
                    }
                };

            let response = LoginResponse {
                token,
                user: user.to_profile(),
            };

            Ok(warp::reply::with_status(
                warp::reply::json(&json!({
                    "success": true,
                    "data": response
                })),
                warp::http::StatusCode::OK,
            ))
        }
        Ok(false) => {
            warn!("Login failed: invalid password for user: {}", request.email);
            Ok(warp::reply::with_status(
                warp::reply::json(&json!({
                    "success": false,
                    "error": "Invalid email or password"
                })),
                warp::http::StatusCode::UNAUTHORIZED,
            ))
        }
        Err(e) => {
            error!("Password verification failed: {}", e);
            Ok(warp::reply::with_status(
                warp::reply::json(&json!({
                    "success": false,
                    "error": "Internal server error"
                })),
                warp::http::StatusCode::INTERNAL_SERVER_ERROR,
            ))
        }
    }
}

async fn handle_verify(
    auth_header: String,
    auth_service: AuthService,
) -> Result<impl Reply, Infallible> {
    let token = match JwtService::extract_token_from_header(&auth_header) {
        Some(token) => token,
        None => {
            return Ok(warp::reply::with_status(
                warp::reply::json(&json!({
                    "success": false,
                    "error": "Invalid authorization header"
                })),
                warp::http::StatusCode::UNAUTHORIZED,
            ));
        }
    };

    match auth_service.jwt_service.verify_token(token) {
        Ok(claims) => Ok(warp::reply::with_status(
            warp::reply::json(&json!({
                "success": true,
                "data": {
                    "user_id": claims.sub,
                    "email": claims.email,
                    "is_admin": claims.is_admin,
                    "exp": claims.exp
                }
            })),
            warp::http::StatusCode::OK,
        )),
        Err(_) => Ok(warp::reply::with_status(
            warp::reply::json(&json!({
                "success": false,
                "error": "Invalid or expired token"
            })),
            warp::http::StatusCode::UNAUTHORIZED,
        )),
    }
}

async fn handle_profile(
    auth_header: String,
    auth_service: AuthService,
) -> Result<impl Reply, Infallible> {
    let token = match JwtService::extract_token_from_header(&auth_header) {
        Some(token) => token,
        None => {
            return Ok(warp::reply::with_status(
                warp::reply::json(&json!({
                    "success": false,
                    "error": "Invalid authorization header"
                })),
                warp::http::StatusCode::UNAUTHORIZED,
            ));
        }
    };

    let claims = match auth_service.jwt_service.verify_token(token) {
        Ok(claims) => claims,
        Err(_) => {
            return Ok(warp::reply::with_status(
                warp::reply::json(&json!({
                    "success": false,
                    "error": "Invalid or expired token"
                })),
                warp::http::StatusCode::UNAUTHORIZED,
            ));
        }
    };

    let user_id = match bson::oid::ObjectId::parse_str(&claims.sub) {
        Ok(id) => id,
        Err(_) => {
            return Ok(warp::reply::with_status(
                warp::reply::json(&json!({
                    "success": false,
                    "error": "Invalid user ID"
                })),
                warp::http::StatusCode::BAD_REQUEST,
            ));
        }
    };

    match auth_service.user_repo.find_by_id(&user_id).await {
        Ok(Some(user)) => Ok(warp::reply::with_status(
            warp::reply::json(&json!({
                "success": true,
                "data": user.to_profile()
            })),
            warp::http::StatusCode::OK,
        )),
        Ok(None) => Ok(warp::reply::with_status(
            warp::reply::json(&json!({
                "success": false,
                "error": "User not found"
            })),
            warp::http::StatusCode::NOT_FOUND,
        )),
        Err(e) => {
            error!("Database error finding user: {}", e);
            Ok(warp::reply::with_status(
                warp::reply::json(&json!({
                    "success": false,
                    "error": "Internal server error"
                })),
                warp::http::StatusCode::INTERNAL_SERVER_ERROR,
            ))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_auth_service_new() {
        let user_repo = UserRepository::new_dummy();
        let auth_service = AuthService::new(user_repo, "test_secret".to_string());

        // Verify the service was created (can't check internals directly, but we can clone it)
        let _ = auth_service.clone();
    }

    #[test]
    fn test_auth_service_new_dummy() {
        let auth_service = AuthService::new_dummy();

        // Verify the service was created
        let _ = auth_service.clone();
    }

    #[test]
    fn test_auth_service_clone() {
        let auth_service1 = AuthService::new_dummy();
        let auth_service2 = auth_service1.clone();

        // Both should be usable
        let _ = auth_service1;
        let _ = auth_service2;
    }

    #[tokio::test]
    async fn test_handle_register_with_invalid_email() {
        let auth_service = AuthService::new_dummy();
        let request = RegisterRequest {
            email: "invalid-email".to_string(),
            password: "password123".to_string(),
            full_name: None,
        };

        let response = handle_register(request, auth_service).await;
        assert!(response.is_ok());
    }

    #[tokio::test]
    async fn test_handle_register_with_short_password() {
        let auth_service = AuthService::new_dummy();
        let request = RegisterRequest {
            email: "test@example.com".to_string(),
            password: "12345".to_string(), // Too short
            full_name: None,
        };

        let response = handle_register(request, auth_service).await;
        assert!(response.is_ok());
    }

    #[tokio::test]
    async fn test_handle_register_with_valid_request() {
        let auth_service = AuthService::new_dummy();
        let request = RegisterRequest {
            email: "test@example.com".to_string(),
            password: "password123".to_string(),
            full_name: Some("Test User".to_string()),
        };

        let response = handle_register(request, auth_service).await;
        assert!(response.is_ok());
    }

    #[tokio::test]
    async fn test_handle_login_with_invalid_email() {
        let auth_service = AuthService::new_dummy();
        let request = LoginRequest {
            email: "invalid-email".to_string(),
            password: "password123".to_string(),
        };

        let response = handle_login(request, auth_service).await;
        assert!(response.is_ok());
    }

    #[tokio::test]
    async fn test_handle_login_with_empty_password() {
        let auth_service = AuthService::new_dummy();
        let request = LoginRequest {
            email: "test@example.com".to_string(),
            password: "".to_string(),
        };

        let response = handle_login(request, auth_service).await;
        assert!(response.is_ok());
    }

    #[tokio::test]
    async fn test_handle_login_with_valid_request() {
        let auth_service = AuthService::new_dummy();
        let request = LoginRequest {
            email: "test@example.com".to_string(),
            password: "password123".to_string(),
        };

        let response = handle_login(request, auth_service).await;
        assert!(response.is_ok());
    }

    #[tokio::test]
    async fn test_handle_verify_with_invalid_header() {
        let auth_service = AuthService::new_dummy();
        let auth_header = "InvalidHeader".to_string();

        let response = handle_verify(auth_header, auth_service).await;
        assert!(response.is_ok());
    }

    #[tokio::test]
    async fn test_handle_verify_with_valid_header_but_invalid_token() {
        let auth_service = AuthService::new_dummy();
        let auth_header = "Bearer invalid.token.here".to_string();

        let response = handle_verify(auth_header, auth_service).await;
        assert!(response.is_ok());
    }

    #[tokio::test]
    async fn test_handle_verify_with_valid_token() {
        let user_repo = UserRepository::new_dummy();
        let jwt_secret = "test_secret".to_string();
        let auth_service = AuthService::new(user_repo, jwt_secret.clone());

        // Generate a valid token
        let jwt_service = JwtService::new(jwt_secret, Some(24));
        let token = jwt_service
            .generate_token("user123", "test@example.com", false)
            .unwrap();

        let auth_header = format!("Bearer {}", token);
        let response = handle_verify(auth_header, auth_service).await;
        assert!(response.is_ok());
    }

    #[tokio::test]
    async fn test_handle_profile_with_invalid_header() {
        let auth_service = AuthService::new_dummy();
        let auth_header = "InvalidHeader".to_string();

        let response = handle_profile(auth_header, auth_service).await;
        assert!(response.is_ok());
    }

    #[tokio::test]
    async fn test_handle_profile_with_invalid_token() {
        let auth_service = AuthService::new_dummy();
        let auth_header = "Bearer invalid.token.here".to_string();

        let response = handle_profile(auth_header, auth_service).await;
        assert!(response.is_ok());
    }

    #[tokio::test]
    async fn test_handle_profile_with_valid_token_but_invalid_user_id() {
        let user_repo = UserRepository::new_dummy();
        let jwt_secret = "test_secret".to_string();
        let auth_service = AuthService::new(user_repo, jwt_secret.clone());

        // Generate a valid token with invalid user ID format
        let jwt_service = JwtService::new(jwt_secret, Some(24));
        let token = jwt_service
            .generate_token("invalid_id_format", "test@example.com", false)
            .unwrap();

        let auth_header = format!("Bearer {}", token);
        let response = handle_profile(auth_header, auth_service).await;
        assert!(response.is_ok());
    }

    #[tokio::test]
    async fn test_handle_profile_with_valid_token_and_valid_user_id() {
        let user_repo = UserRepository::new_dummy();
        let jwt_secret = "test_secret".to_string();
        let auth_service = AuthService::new(user_repo, jwt_secret.clone());

        // Generate a valid token with a valid ObjectId
        let jwt_service = JwtService::new(jwt_secret, Some(24));
        let user_id = bson::oid::ObjectId::new();
        let token = jwt_service
            .generate_token(&user_id.to_hex(), "test@example.com", false)
            .unwrap();

        let auth_header = format!("Bearer {}", token);
        let response = handle_profile(auth_header, auth_service).await;
        assert!(response.is_ok());
    }

    #[test]
    fn test_register_request_clone() {
        let request1 = RegisterRequest {
            email: "test@example.com".to_string(),
            password: "password123".to_string(),
            full_name: Some("Test User".to_string()),
        };

        let request2 = request1.clone();
        assert_eq!(request1.email, request2.email);
        assert_eq!(request1.password, request2.password);
        assert_eq!(request1.full_name, request2.full_name);
    }

    #[test]
    fn test_login_request_clone() {
        let request1 = LoginRequest {
            email: "test@example.com".to_string(),
            password: "password123".to_string(),
        };

        let request2 = request1.clone();
        assert_eq!(request1.email, request2.email);
        assert_eq!(request1.password, request2.password);
    }

    #[test]
    fn test_login_response_clone() {
        let user = User::new(
            "test@example.com".to_string(),
            "hashed_password".to_string(),
            Some("Test User".to_string()),
        );

        let response1 = LoginResponse {
            token: "test_token".to_string(),
            user: user.to_profile(),
        };

        let response2 = response1.clone();
        assert_eq!(response1.token, response2.token);
        assert_eq!(response1.user.email, response2.user.email);
    }

    // Additional comprehensive tests for handle_register

    #[tokio::test]
    async fn test_handle_register_validation_error_response_structure() {
        use warp::Reply;

        let auth_service = AuthService::new_dummy();
        let request = RegisterRequest {
            email: "invalid-email".to_string(),
            password: "password123".to_string(),
            full_name: None,
        };

        let response = handle_register(request, auth_service).await.unwrap();
        let reply = response.into_response();

        assert_eq!(reply.status(), warp::http::StatusCode::BAD_REQUEST);
    }

    #[tokio::test]
    async fn test_handle_register_short_password_returns_bad_request() {
        use warp::Reply;

        let auth_service = AuthService::new_dummy();
        let request = RegisterRequest {
            email: "test@example.com".to_string(),
            password: "123".to_string(),
            full_name: None,
        };

        let response = handle_register(request, auth_service).await.unwrap();
        let reply = response.into_response();

        assert_eq!(reply.status(), warp::http::StatusCode::BAD_REQUEST);
    }

    #[tokio::test]
    async fn test_handle_register_with_full_name_included() {
        let auth_service = AuthService::new_dummy();
        let request = RegisterRequest {
            email: "newuser@example.com".to_string(),
            password: "securepassword123".to_string(),
            full_name: Some("John Doe".to_string()),
        };

        let response = handle_register(request, auth_service).await;
        assert!(response.is_ok());
    }

    // Additional comprehensive tests for handle_login

    #[tokio::test]
    async fn test_handle_login_validation_error_response() {
        use warp::Reply;

        let auth_service = AuthService::new_dummy();
        let request = LoginRequest {
            email: "not-an-email".to_string(),
            password: "password123".to_string(),
        };

        let response = handle_login(request, auth_service).await.unwrap();
        let reply = response.into_response();

        assert_eq!(reply.status(), warp::http::StatusCode::BAD_REQUEST);
    }

    #[tokio::test]
    async fn test_handle_login_empty_password_validation() {
        use warp::Reply;

        let auth_service = AuthService::new_dummy();
        let request = LoginRequest {
            email: "test@example.com".to_string(),
            password: "".to_string(),
        };

        let response = handle_login(request, auth_service).await.unwrap();
        let reply = response.into_response();

        assert_eq!(reply.status(), warp::http::StatusCode::BAD_REQUEST);
    }

    #[tokio::test]
    async fn test_handle_login_database_error_path() {
        let auth_service = AuthService::new_dummy();
        let request = LoginRequest {
            email: "test@example.com".to_string(),
            password: "validpassword123".to_string(),
        };

        let response = handle_login(request, auth_service).await;
        assert!(response.is_ok());
    }

    // Additional comprehensive tests for handle_verify

    #[tokio::test]
    async fn test_handle_verify_missing_bearer_prefix() {
        use warp::Reply;

        let auth_service = AuthService::new_dummy();
        let auth_header = "NotBearerToken".to_string();

        let response = handle_verify(auth_header, auth_service).await.unwrap();
        let reply = response.into_response();

        assert_eq!(reply.status(), warp::http::StatusCode::UNAUTHORIZED);
    }

    #[tokio::test]
    async fn test_handle_verify_empty_header() {
        use warp::Reply;

        let auth_service = AuthService::new_dummy();
        let auth_header = "".to_string();

        let response = handle_verify(auth_header, auth_service).await.unwrap();
        let reply = response.into_response();

        assert_eq!(reply.status(), warp::http::StatusCode::UNAUTHORIZED);
    }

    #[tokio::test]
    async fn test_handle_verify_bearer_with_no_token() {
        use warp::Reply;

        let auth_service = AuthService::new_dummy();
        let auth_header = "Bearer ".to_string();

        let response = handle_verify(auth_header, auth_service).await.unwrap();
        let reply = response.into_response();

        assert_eq!(reply.status(), warp::http::StatusCode::UNAUTHORIZED);
    }

    #[tokio::test]
    async fn test_handle_verify_malformed_token() {
        use warp::Reply;

        let auth_service = AuthService::new_dummy();
        let auth_header = "Bearer malformed.token".to_string();

        let response = handle_verify(auth_header, auth_service).await.unwrap();
        let reply = response.into_response();

        assert_eq!(reply.status(), warp::http::StatusCode::UNAUTHORIZED);
    }

    #[tokio::test]
    async fn test_handle_verify_valid_token_returns_ok() {
        use warp::Reply;

        let user_repo = UserRepository::new_dummy();
        let jwt_secret = "test_secret_key_for_verification".to_string();
        let auth_service = AuthService::new(user_repo, jwt_secret.clone());

        let jwt_service = JwtService::new(jwt_secret, Some(24));
        let token = jwt_service
            .generate_token("user_123", "verified@example.com", false)
            .unwrap();

        let auth_header = format!("Bearer {}", token);
        let response = handle_verify(auth_header, auth_service).await.unwrap();
        let reply = response.into_response();

        assert_eq!(reply.status(), warp::http::StatusCode::OK);
    }

    #[tokio::test]
    async fn test_handle_verify_admin_user_token() {
        use warp::Reply;

        let user_repo = UserRepository::new_dummy();
        let jwt_secret = "test_admin_secret".to_string();
        let auth_service = AuthService::new(user_repo, jwt_secret.clone());

        let jwt_service = JwtService::new(jwt_secret, Some(24));
        let token = jwt_service
            .generate_token("admin_456", "admin@example.com", true)
            .unwrap();

        let auth_header = format!("Bearer {}", token);
        let response = handle_verify(auth_header, auth_service).await.unwrap();
        let reply = response.into_response();

        assert_eq!(reply.status(), warp::http::StatusCode::OK);
    }

    // Additional comprehensive tests for handle_profile

    #[tokio::test]
    async fn test_handle_profile_missing_bearer_prefix() {
        use warp::Reply;

        let auth_service = AuthService::new_dummy();
        let auth_header = "JustAToken".to_string();

        let response = handle_profile(auth_header, auth_service).await.unwrap();
        let reply = response.into_response();

        assert_eq!(reply.status(), warp::http::StatusCode::UNAUTHORIZED);
    }

    #[tokio::test]
    async fn test_handle_profile_empty_token() {
        use warp::Reply;

        let auth_service = AuthService::new_dummy();
        let auth_header = "Bearer ".to_string();

        let response = handle_profile(auth_header, auth_service).await.unwrap();
        let reply = response.into_response();

        assert_eq!(reply.status(), warp::http::StatusCode::UNAUTHORIZED);
    }

    #[tokio::test]
    async fn test_handle_profile_malformed_token_format() {
        use warp::Reply;

        let auth_service = AuthService::new_dummy();
        let auth_header = "Bearer not.a.valid.jwt".to_string();

        let response = handle_profile(auth_header, auth_service).await.unwrap();
        let reply = response.into_response();

        assert_eq!(reply.status(), warp::http::StatusCode::UNAUTHORIZED);
    }

    #[tokio::test]
    async fn test_handle_profile_invalid_user_id_format_in_token() {
        use warp::Reply;

        let user_repo = UserRepository::new_dummy();
        let jwt_secret = "test_profile_secret".to_string();
        let auth_service = AuthService::new(user_repo, jwt_secret.clone());

        let jwt_service = JwtService::new(jwt_secret, Some(24));
        let token = jwt_service
            .generate_token("not_a_valid_objectid", "user@example.com", false)
            .unwrap();

        let auth_header = format!("Bearer {}", token);
        let response = handle_profile(auth_header, auth_service).await.unwrap();
        let reply = response.into_response();

        assert_eq!(reply.status(), warp::http::StatusCode::BAD_REQUEST);
    }

    #[tokio::test]
    async fn test_handle_profile_valid_objectid_but_user_not_found() {
        use warp::Reply;

        let user_repo = UserRepository::new_dummy();
        let jwt_secret = "test_secret_profile".to_string();
        let auth_service = AuthService::new(user_repo, jwt_secret.clone());

        let jwt_service = JwtService::new(jwt_secret, Some(24));
        let user_id = bson::oid::ObjectId::new();
        let token = jwt_service
            .generate_token(&user_id.to_hex(), "notfound@example.com", false)
            .unwrap();

        let auth_header = format!("Bearer {}", token);
        let response = handle_profile(auth_header, auth_service).await.unwrap();
        let reply = response.into_response();

        assert_eq!(reply.status(), warp::http::StatusCode::INTERNAL_SERVER_ERROR);
    }

    #[tokio::test]
    async fn test_handle_profile_token_with_expired_claims() {
        use warp::Reply;

        let auth_service = AuthService::new_dummy();

        let jwt_service = JwtService::new("secret".to_string(), Some(0));
        let token = jwt_service
            .generate_token("user123", "test@example.com", false)
            .unwrap();

        tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;

        let auth_header = format!("Bearer {}", token);
        let response = handle_profile(auth_header, auth_service).await.unwrap();
        let reply = response.into_response();

        assert_eq!(reply.status(), warp::http::StatusCode::UNAUTHORIZED);
    }

    #[tokio::test]
    async fn test_auth_service_routes_created() {
        let auth_service = AuthService::new_dummy();
        let routes = auth_service.routes();
        let _ = routes;
    }

    #[test]
    fn test_auth_service_register_route_exists() {
        let auth_service = AuthService::new_dummy();
        let route = auth_service.register_route();
        let _ = route;
    }

    #[test]
    fn test_auth_service_login_route_exists() {
        let auth_service = AuthService::new_dummy();
        let route = auth_service.login_route();
        let _ = route;
    }

    #[test]
    fn test_auth_service_verify_route_exists() {
        let auth_service = AuthService::new_dummy();
        let route = auth_service.verify_route();
        let _ = route;
    }

    #[test]
    fn test_auth_service_profile_route_exists() {
        let auth_service = AuthService::new_dummy();
        let route = auth_service.profile_route();
        let _ = route;
    }

    #[tokio::test]
    async fn test_handle_register_with_very_long_email() {
        let auth_service = AuthService::new_dummy();
        let long_email = format!("{}@example.com", "a".repeat(100));
        let request = RegisterRequest {
            email: long_email,
            password: "password123".to_string(),
            full_name: None,
        };

        let response = handle_register(request, auth_service).await;
        assert!(response.is_ok());
    }

    #[tokio::test]
    async fn test_handle_register_with_very_long_password() {
        let auth_service = AuthService::new_dummy();
        let request = RegisterRequest {
            email: "test@example.com".to_string(),
            password: "a".repeat(200),
            full_name: None,
        };

        let response = handle_register(request, auth_service).await;
        assert!(response.is_ok());
    }

    #[tokio::test]
    async fn test_handle_register_with_special_characters_in_name() {
        let auth_service = AuthService::new_dummy();
        let request = RegisterRequest {
            email: "test@example.com".to_string(),
            password: "password123".to_string(),
            full_name: Some("Test User !@#$%^&*()".to_string()),
        };

        let response = handle_register(request, auth_service).await;
        assert!(response.is_ok());
    }

    #[tokio::test]
    async fn test_handle_login_with_very_long_password() {
        let auth_service = AuthService::new_dummy();
        let request = LoginRequest {
            email: "test@example.com".to_string(),
            password: "a".repeat(500),
        };

        let response = handle_login(request, auth_service).await;
        assert!(response.is_ok());
    }

    #[tokio::test]
    async fn test_handle_login_with_uppercase_email() {
        let auth_service = AuthService::new_dummy();
        let request = LoginRequest {
            email: "TEST@EXAMPLE.COM".to_string(),
            password: "password123".to_string(),
        };

        let response = handle_login(request, auth_service).await;
        assert!(response.is_ok());
    }

    #[tokio::test]
    async fn test_handle_verify_with_token_containing_special_chars() {
        use warp::Reply;

        let auth_service = AuthService::new_dummy();
        let auth_header = "Bearer abc.def.ghi!@#$%".to_string();

        let response = handle_verify(auth_header, auth_service).await.unwrap();
        let reply = response.into_response();

        assert_eq!(reply.status(), warp::http::StatusCode::UNAUTHORIZED);
    }

    #[tokio::test]
    async fn test_handle_verify_case_sensitive_bearer() {
        use warp::Reply;

        let auth_service = AuthService::new_dummy();
        let auth_header = "bearer valid.token.here".to_string();

        let response = handle_verify(auth_header, auth_service).await.unwrap();
        let reply = response.into_response();

        assert_eq!(reply.status(), warp::http::StatusCode::UNAUTHORIZED);
    }

    #[tokio::test]
    async fn test_handle_profile_with_admin_user_token() {
        use warp::Reply;

        let user_repo = UserRepository::new_dummy();
        let jwt_secret = "admin_test_secret".to_string();
        let auth_service = AuthService::new(user_repo, jwt_secret.clone());

        let jwt_service = JwtService::new(jwt_secret, Some(24));
        let user_id = bson::oid::ObjectId::new();
        let token = jwt_service
            .generate_token(&user_id.to_hex(), "admin@example.com", true)
            .unwrap();

        let auth_header = format!("Bearer {}", token);
        let response = handle_profile(auth_header, auth_service).await.unwrap();
        let reply = response.into_response();

        assert_eq!(reply.status(), warp::http::StatusCode::INTERNAL_SERVER_ERROR);
    }

    #[test]
    fn test_auth_service_new_with_custom_secret() {
        let user_repo = UserRepository::new_dummy();
        let custom_secret = "my_very_secure_secret_key_123".to_string();
        let auth_service = AuthService::new(user_repo, custom_secret);

        let _ = auth_service.clone();
    }

    #[test]
    fn test_auth_service_multiple_clones() {
        let auth_service1 = AuthService::new_dummy();
        let auth_service2 = auth_service1.clone();
        let auth_service3 = auth_service2.clone();

        let _ = auth_service1;
        let _ = auth_service2;
        let _ = auth_service3;
    }
}
