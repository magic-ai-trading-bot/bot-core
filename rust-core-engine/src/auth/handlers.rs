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
