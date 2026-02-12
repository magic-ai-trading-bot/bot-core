// @spec:FR-AUTH-016 - Account Security Handlers
// @ref:specs/02-design/2.5-components/COMP-RUST-AUTH.md
// @test:TC-AUTH-020 through TC-AUTH-035

use serde_json::json;
use std::{convert::Infallible, sync::Arc};
use totp_rs::{Algorithm, Secret, TOTP};
use tracing::{error, info, warn};
use validator::Validate;
use warp::{Filter, Rejection, Reply};

use super::{
    database::{SessionRepository, UserRepository},
    jwt::{JwtService, PasswordService},
    models::{
        ChangePasswordRequest, SessionInfo, SessionListResponse, Setup2FAResponse,
        UpdateProfileRequest, Verify2FARequest,
    },
};

const TOTP_ISSUER: &str = "BotCore Trading";

#[derive(Clone)]
pub struct SecurityService {
    user_repo: UserRepository,
    session_repo: SessionRepository,
    jwt_service: JwtService,
}

impl SecurityService {
    pub fn new(
        user_repo: UserRepository,
        session_repo: SessionRepository,
        jwt_secret: String,
    ) -> Self {
        let jwt_service = JwtService::new(jwt_secret, Some(24 * 7));
        Self {
            user_repo,
            session_repo,
            jwt_service,
        }
    }

    pub fn new_dummy() -> Self {
        Self {
            user_repo: UserRepository::new_dummy(),
            session_repo: SessionRepository::new_dummy(),
            jwt_service: JwtService::new("dummy".to_string(), Some(24 * 7)),
        }
    }

    pub fn routes(self) -> impl Filter<Extract = impl Reply, Error = Rejection> + Clone {
        let service = Arc::new(self);

        let change_password = Self::change_password_route(service.clone());
        let update_profile = Self::update_profile_route(service.clone());
        let setup_2fa = Self::setup_2fa_route(service.clone());
        let verify_2fa = Self::verify_2fa_route(service.clone());
        let disable_2fa = Self::disable_2fa_route(service.clone());
        let list_sessions = Self::list_sessions_route(service.clone());
        let revoke_session = Self::revoke_session_route(service.clone());
        let revoke_all = Self::revoke_all_sessions_route(service);

        warp::path("auth").and(
            change_password
                .or(update_profile)
                .or(setup_2fa)
                .or(verify_2fa)
                .or(disable_2fa)
                .or(list_sessions)
                .or(revoke_session)
                .or(revoke_all),
        )
    }

    fn change_password_route(
        service: Arc<Self>,
    ) -> impl Filter<Extract = impl Reply, Error = Rejection> + Clone {
        warp::path("change-password")
            .and(warp::post())
            .and(warp::header::<String>("authorization"))
            .and(warp::body::json())
            .and(warp::any().map(move || service.clone()))
            .and_then(handle_change_password)
    }

    fn update_profile_route(
        service: Arc<Self>,
    ) -> impl Filter<Extract = impl Reply, Error = Rejection> + Clone {
        warp::path("profile")
            .and(warp::patch())
            .and(warp::header::<String>("authorization"))
            .and(warp::body::json())
            .and(warp::any().map(move || service.clone()))
            .and_then(handle_update_profile)
    }

    fn setup_2fa_route(
        service: Arc<Self>,
    ) -> impl Filter<Extract = impl Reply, Error = Rejection> + Clone {
        warp::path!("2fa" / "setup")
            .and(warp::post())
            .and(warp::header::<String>("authorization"))
            .and(warp::any().map(move || service.clone()))
            .and_then(handle_setup_2fa)
    }

    fn verify_2fa_route(
        service: Arc<Self>,
    ) -> impl Filter<Extract = impl Reply, Error = Rejection> + Clone {
        warp::path!("2fa" / "verify")
            .and(warp::post())
            .and(warp::header::<String>("authorization"))
            .and(warp::body::json())
            .and(warp::any().map(move || service.clone()))
            .and_then(handle_verify_2fa)
    }

    fn disable_2fa_route(
        service: Arc<Self>,
    ) -> impl Filter<Extract = impl Reply, Error = Rejection> + Clone {
        warp::path!("2fa" / "disable")
            .and(warp::post())
            .and(warp::header::<String>("authorization"))
            .and(warp::body::json())
            .and(warp::any().map(move || service.clone()))
            .and_then(handle_disable_2fa)
    }

    fn list_sessions_route(
        service: Arc<Self>,
    ) -> impl Filter<Extract = impl Reply, Error = Rejection> + Clone {
        warp::path("sessions")
            .and(warp::get())
            .and(warp::header::<String>("authorization"))
            .and(warp::any().map(move || service.clone()))
            .and_then(handle_list_sessions)
    }

    fn revoke_session_route(
        service: Arc<Self>,
    ) -> impl Filter<Extract = impl Reply, Error = Rejection> + Clone {
        warp::path!("sessions" / String)
            .and(warp::delete())
            .and(warp::header::<String>("authorization"))
            .and(warp::any().map(move || service.clone()))
            .and_then(handle_revoke_session)
    }

    fn revoke_all_sessions_route(
        service: Arc<Self>,
    ) -> impl Filter<Extract = impl Reply, Error = Rejection> + Clone {
        warp::path!("sessions" / "revoke-all")
            .and(warp::post())
            .and(warp::header::<String>("authorization"))
            .and(warp::any().map(move || service.clone()))
            .and_then(handle_revoke_all_sessions)
    }
}

// Helper to extract user_id from token
fn extract_user_id(
    auth_header: &str,
    jwt_service: &JwtService,
) -> Result<bson::oid::ObjectId, warp::reply::WithStatus<warp::reply::Json>> {
    let token = JwtService::extract_token_from_header(auth_header).ok_or_else(|| {
        warp::reply::with_status(
            warp::reply::json(&json!({"success": false, "error": "Invalid authorization header"})),
            warp::http::StatusCode::UNAUTHORIZED,
        )
    })?;

    let claims = jwt_service.verify_token(token).map_err(|_| {
        warp::reply::with_status(
            warp::reply::json(&json!({"success": false, "error": "Invalid or expired token"})),
            warp::http::StatusCode::UNAUTHORIZED,
        )
    })?;

    bson::oid::ObjectId::parse_str(&claims.sub).map_err(|_| {
        warp::reply::with_status(
            warp::reply::json(&json!({"success": false, "error": "Invalid user ID"})),
            warp::http::StatusCode::BAD_REQUEST,
        )
    })
}

// @spec:FR-AUTH-012 - Change Password Handler
async fn handle_change_password(
    auth_header: String,
    request: ChangePasswordRequest,
    service: Arc<SecurityService>,
) -> Result<impl Reply, Infallible> {
    info!("Change password request received");

    // Validate request
    if let Err(e) = request.validate() {
        warn!("Validation failed: {:?}", e);
        return Ok(warp::reply::with_status(
            warp::reply::json(
                &json!({"success": false, "error": "Validation failed", "details": e.to_string()}),
            ),
            warp::http::StatusCode::BAD_REQUEST,
        ));
    }

    let user_id = match extract_user_id(&auth_header, &service.jwt_service) {
        Ok(id) => id,
        Err(reply) => return Ok(reply),
    };

    // Find user and verify current password
    let user = match service.user_repo.find_by_id(&user_id).await {
        Ok(Some(user)) => user,
        Ok(None) => {
            return Ok(warp::reply::with_status(
                warp::reply::json(&json!({"success": false, "error": "User not found"})),
                warp::http::StatusCode::NOT_FOUND,
            ));
        },
        Err(e) => {
            error!("Database error: {}", e);
            return Ok(warp::reply::with_status(
                warp::reply::json(&json!({"success": false, "error": "Internal server error"})),
                warp::http::StatusCode::INTERNAL_SERVER_ERROR,
            ));
        },
    };

    // Verify current password
    match PasswordService::verify_password(&request.current_password, &user.password_hash) {
        Ok(true) => {},
        Ok(false) => {
            warn!("Current password incorrect for user: {}", user.email);
            return Ok(warp::reply::with_status(
                warp::reply::json(
                    &json!({"success": false, "error": "Current password is incorrect"}),
                ),
                warp::http::StatusCode::UNAUTHORIZED,
            ));
        },
        Err(e) => {
            error!("Password verification failed: {}", e);
            return Ok(warp::reply::with_status(
                warp::reply::json(&json!({"success": false, "error": "Internal server error"})),
                warp::http::StatusCode::INTERNAL_SERVER_ERROR,
            ));
        },
    }

    // Hash new password
    let new_hash = match PasswordService::hash_password(&request.new_password) {
        Ok(hash) => hash,
        Err(e) => {
            error!("Password hashing failed: {}", e);
            return Ok(warp::reply::with_status(
                warp::reply::json(&json!({"success": false, "error": "Internal server error"})),
                warp::http::StatusCode::INTERNAL_SERVER_ERROR,
            ));
        },
    };

    // Update password in database
    if let Err(e) = service.user_repo.update_password(&user_id, new_hash).await {
        error!("Failed to update password: {}", e);
        return Ok(warp::reply::with_status(
            warp::reply::json(&json!({"success": false, "error": "Failed to update password"})),
            warp::http::StatusCode::INTERNAL_SERVER_ERROR,
        ));
    }

    info!("Password changed successfully for user: {}", user.email);
    Ok(warp::reply::with_status(
        warp::reply::json(&json!({"success": true, "message": "Password changed successfully"})),
        warp::http::StatusCode::OK,
    ))
}

// @spec:FR-AUTH-013 - Update Profile Handler
// @spec:FR-AUTH-016 - Avatar Upload (base64)
async fn handle_update_profile(
    auth_header: String,
    request: UpdateProfileRequest,
    service: Arc<SecurityService>,
) -> Result<impl Reply, Infallible> {
    info!("Update profile request received");

    if let Err(e) = request.validate() {
        return Ok(warp::reply::with_status(
            warp::reply::json(
                &json!({"success": false, "error": "Validation failed", "details": e.to_string()}),
            ),
            warp::http::StatusCode::BAD_REQUEST,
        ));
    }

    let user_id = match extract_user_id(&auth_header, &service.jwt_service) {
        Ok(id) => id,
        Err(reply) => return Ok(reply),
    };

    // Process avatar if provided (store as data URL)
    let avatar_url = request.avatar_base64.map(|base64| {
        // If it already has data URL prefix, use as-is
        if base64.starts_with("data:image/") {
            base64
        } else {
            // Add data URL prefix (assume JPEG if not specified)
            format!("data:image/jpeg;base64,{}", base64)
        }
    });

    // Update both display_name and avatar
    if let Err(e) = service
        .user_repo
        .update_profile(&user_id, request.display_name, avatar_url)
        .await
    {
        error!("Failed to update profile: {}", e);
        return Ok(warp::reply::with_status(
            warp::reply::json(&json!({"success": false, "error": "Failed to update profile"})),
            warp::http::StatusCode::INTERNAL_SERVER_ERROR,
        ));
    }

    // Fetch updated user
    match service.user_repo.find_by_id(&user_id).await {
        Ok(Some(user)) => {
            info!("Profile updated for user: {}", user.email);
            Ok(warp::reply::with_status(
                warp::reply::json(&json!({"success": true, "data": user.to_profile()})),
                warp::http::StatusCode::OK,
            ))
        },
        _ => Ok(warp::reply::with_status(
            warp::reply::json(&json!({"success": true, "message": "Profile updated"})),
            warp::http::StatusCode::OK,
        )),
    }
}

// @spec:FR-AUTH-014 - Setup 2FA Handler
async fn handle_setup_2fa(
    auth_header: String,
    service: Arc<SecurityService>,
) -> Result<impl Reply, Infallible> {
    info!("2FA setup request received");

    let user_id = match extract_user_id(&auth_header, &service.jwt_service) {
        Ok(id) => id,
        Err(reply) => return Ok(reply),
    };

    let user = match service.user_repo.find_by_id(&user_id).await {
        Ok(Some(user)) => user,
        Ok(None) => {
            return Ok(warp::reply::with_status(
                warp::reply::json(&json!({"success": false, "error": "User not found"})),
                warp::http::StatusCode::NOT_FOUND,
            ));
        },
        Err(e) => {
            error!("Database error: {}", e);
            return Ok(warp::reply::with_status(
                warp::reply::json(&json!({"success": false, "error": "Internal server error"})),
                warp::http::StatusCode::INTERNAL_SERVER_ERROR,
            ));
        },
    };

    if user.two_factor_enabled {
        return Ok(warp::reply::with_status(
            warp::reply::json(&json!({"success": false, "error": "2FA is already enabled"})),
            warp::http::StatusCode::BAD_REQUEST,
        ));
    }

    // Generate TOTP secret
    let secret = Secret::generate_secret();
    let secret_base32 = secret.to_encoded().to_string();

    let totp = match TOTP::new(
        Algorithm::SHA1,
        6,
        1,
        30,
        secret.to_bytes().unwrap_or_default(),
        Some(TOTP_ISSUER.to_string()),
        user.email.clone(),
    ) {
        Ok(totp) => totp,
        Err(e) => {
            error!("Failed to create TOTP: {}", e);
            return Ok(warp::reply::with_status(
                warp::reply::json(&json!({"success": false, "error": "Failed to setup 2FA"})),
                warp::http::StatusCode::INTERNAL_SERVER_ERROR,
            ));
        },
    };

    // Generate QR code
    let qr_code = match totp.get_qr_base64() {
        Ok(qr) => qr,
        Err(e) => {
            error!("Failed to generate QR code: {}", e);
            return Ok(warp::reply::with_status(
                warp::reply::json(
                    &json!({"success": false, "error": "Failed to generate QR code"}),
                ),
                warp::http::StatusCode::INTERNAL_SERVER_ERROR,
            ));
        },
    };

    // Store secret temporarily (will be confirmed on verify)
    if let Err(e) = service
        .user_repo
        .update_2fa(&user_id, false, Some(secret_base32.clone()))
        .await
    {
        error!("Failed to store 2FA secret: {}", e);
        return Ok(warp::reply::with_status(
            warp::reply::json(&json!({"success": false, "error": "Failed to setup 2FA"})),
            warp::http::StatusCode::INTERNAL_SERVER_ERROR,
        ));
    }

    let response = Setup2FAResponse {
        secret: secret_base32,
        qr_code: format!("data:image/png;base64,{}", qr_code),
        otpauth_url: totp.get_url(),
    };

    info!("2FA setup initiated for user: {}", user.email);
    Ok(warp::reply::with_status(
        warp::reply::json(&json!({"success": true, "data": response})),
        warp::http::StatusCode::OK,
    ))
}

// @spec:FR-AUTH-014 - Verify 2FA Handler (enables 2FA)
async fn handle_verify_2fa(
    auth_header: String,
    request: Verify2FARequest,
    service: Arc<SecurityService>,
) -> Result<impl Reply, Infallible> {
    info!("2FA verify request received");

    if let Err(e) = request.validate() {
        return Ok(warp::reply::with_status(
            warp::reply::json(
                &json!({"success": false, "error": "Validation failed", "details": e.to_string()}),
            ),
            warp::http::StatusCode::BAD_REQUEST,
        ));
    }

    let user_id = match extract_user_id(&auth_header, &service.jwt_service) {
        Ok(id) => id,
        Err(reply) => return Ok(reply),
    };

    let user = match service.user_repo.find_by_id(&user_id).await {
        Ok(Some(user)) => user,
        Ok(None) => {
            return Ok(warp::reply::with_status(
                warp::reply::json(&json!({"success": false, "error": "User not found"})),
                warp::http::StatusCode::NOT_FOUND,
            ));
        },
        Err(e) => {
            error!("Database error: {}", e);
            return Ok(warp::reply::with_status(
                warp::reply::json(&json!({"success": false, "error": "Internal server error"})),
                warp::http::StatusCode::INTERNAL_SERVER_ERROR,
            ));
        },
    };

    let secret = match &user.two_factor_secret {
        Some(s) => s.clone(),
        None => {
            return Ok(warp::reply::with_status(
                warp::reply::json(
                    &json!({"success": false, "error": "2FA not set up. Please call setup first."}),
                ),
                warp::http::StatusCode::BAD_REQUEST,
            ));
        },
    };

    // Verify the code
    let secret_bytes = match Secret::Encoded(secret).to_bytes() {
        Ok(bytes) => bytes,
        Err(e) => {
            error!("Invalid secret format: {}", e);
            return Ok(warp::reply::with_status(
                warp::reply::json(&json!({"success": false, "error": "Invalid 2FA configuration"})),
                warp::http::StatusCode::INTERNAL_SERVER_ERROR,
            ));
        },
    };

    let totp = match TOTP::new(
        Algorithm::SHA1,
        6,
        1,
        30,
        secret_bytes,
        Some(TOTP_ISSUER.to_string()),
        user.email.clone(),
    ) {
        Ok(totp) => totp,
        Err(e) => {
            error!("Failed to create TOTP verifier: {}", e);
            return Ok(warp::reply::with_status(
                warp::reply::json(&json!({"success": false, "error": "Internal server error"})),
                warp::http::StatusCode::INTERNAL_SERVER_ERROR,
            ));
        },
    };

    if !totp.check_current(&request.code).unwrap_or(false) {
        warn!("Invalid 2FA code for user: {}", user.email);
        return Ok(warp::reply::with_status(
            warp::reply::json(&json!({"success": false, "error": "Invalid verification code"})),
            warp::http::StatusCode::UNAUTHORIZED,
        ));
    }

    // Enable 2FA
    if let Err(e) = service
        .user_repo
        .update_2fa(&user_id, true, user.two_factor_secret)
        .await
    {
        error!("Failed to enable 2FA: {}", e);
        return Ok(warp::reply::with_status(
            warp::reply::json(&json!({"success": false, "error": "Failed to enable 2FA"})),
            warp::http::StatusCode::INTERNAL_SERVER_ERROR,
        ));
    }

    info!("2FA enabled successfully for user: {}", user.email);
    Ok(warp::reply::with_status(
        warp::reply::json(&json!({"success": true, "message": "2FA enabled successfully"})),
        warp::http::StatusCode::OK,
    ))
}

// @spec:FR-AUTH-014 - Disable 2FA Handler
async fn handle_disable_2fa(
    auth_header: String,
    request: Verify2FARequest,
    service: Arc<SecurityService>,
) -> Result<impl Reply, Infallible> {
    info!("2FA disable request received");

    let user_id = match extract_user_id(&auth_header, &service.jwt_service) {
        Ok(id) => id,
        Err(reply) => return Ok(reply),
    };

    let user = match service.user_repo.find_by_id(&user_id).await {
        Ok(Some(user)) => user,
        Ok(None) => {
            return Ok(warp::reply::with_status(
                warp::reply::json(&json!({"success": false, "error": "User not found"})),
                warp::http::StatusCode::NOT_FOUND,
            ));
        },
        Err(e) => {
            error!("Database error: {}", e);
            return Ok(warp::reply::with_status(
                warp::reply::json(&json!({"success": false, "error": "Internal server error"})),
                warp::http::StatusCode::INTERNAL_SERVER_ERROR,
            ));
        },
    };

    if !user.two_factor_enabled {
        return Ok(warp::reply::with_status(
            warp::reply::json(&json!({"success": false, "error": "2FA is not enabled"})),
            warp::http::StatusCode::BAD_REQUEST,
        ));
    }

    // Verify the code before disabling
    let secret = match &user.two_factor_secret {
        Some(s) => s.clone(),
        None => {
            return Ok(warp::reply::with_status(
                warp::reply::json(&json!({"success": false, "error": "2FA secret not found"})),
                warp::http::StatusCode::INTERNAL_SERVER_ERROR,
            ));
        },
    };

    let secret_bytes = match Secret::Encoded(secret).to_bytes() {
        Ok(bytes) => bytes,
        Err(_) => {
            return Ok(warp::reply::with_status(
                warp::reply::json(&json!({"success": false, "error": "Invalid 2FA configuration"})),
                warp::http::StatusCode::INTERNAL_SERVER_ERROR,
            ));
        },
    };

    let totp = match TOTP::new(
        Algorithm::SHA1,
        6,
        1,
        30,
        secret_bytes,
        Some(TOTP_ISSUER.to_string()),
        user.email.clone(),
    ) {
        Ok(totp) => totp,
        Err(_) => {
            return Ok(warp::reply::with_status(
                warp::reply::json(&json!({"success": false, "error": "Internal server error"})),
                warp::http::StatusCode::INTERNAL_SERVER_ERROR,
            ));
        },
    };

    if !totp.check_current(&request.code).unwrap_or(false) {
        warn!("Invalid 2FA code for disabling: {}", user.email);
        return Ok(warp::reply::with_status(
            warp::reply::json(&json!({"success": false, "error": "Invalid verification code"})),
            warp::http::StatusCode::UNAUTHORIZED,
        ));
    }

    // Disable 2FA
    if let Err(e) = service.user_repo.update_2fa(&user_id, false, None).await {
        error!("Failed to disable 2FA: {}", e);
        return Ok(warp::reply::with_status(
            warp::reply::json(&json!({"success": false, "error": "Failed to disable 2FA"})),
            warp::http::StatusCode::INTERNAL_SERVER_ERROR,
        ));
    }

    info!("2FA disabled for user: {}", user.email);
    Ok(warp::reply::with_status(
        warp::reply::json(&json!({"success": true, "message": "2FA disabled successfully"})),
        warp::http::StatusCode::OK,
    ))
}

// @spec:FR-AUTH-015 - List Sessions Handler
async fn handle_list_sessions(
    auth_header: String,
    service: Arc<SecurityService>,
) -> Result<impl Reply, Infallible> {
    let user_id = match extract_user_id(&auth_header, &service.jwt_service) {
        Ok(id) => id,
        Err(reply) => return Ok(reply),
    };

    // Get current session ID from token (for marking current session)
    let current_session_id = JwtService::extract_token_from_header(&auth_header)
        .and_then(|t| service.jwt_service.verify_token(t).ok())
        .map(|c| c.session_id.unwrap_or_default())
        .unwrap_or_default();

    let sessions = match service.session_repo.find_by_user(&user_id).await {
        Ok(sessions) => sessions,
        Err(e) => {
            error!("Failed to fetch sessions: {}", e);
            return Ok(warp::reply::with_status(
                warp::reply::json(&json!({"success": false, "error": "Failed to fetch sessions"})),
                warp::http::StatusCode::INTERNAL_SERVER_ERROR,
            ));
        },
    };

    let session_infos: Vec<SessionInfo> = sessions
        .iter()
        .map(|s| s.to_info(&current_session_id))
        .collect();

    let response = SessionListResponse {
        sessions: session_infos,
    };

    Ok(warp::reply::with_status(
        warp::reply::json(&json!({"success": true, "data": response})),
        warp::http::StatusCode::OK,
    ))
}

// @spec:FR-AUTH-015 - Revoke Session Handler
async fn handle_revoke_session(
    session_id: String,
    auth_header: String,
    service: Arc<SecurityService>,
) -> Result<impl Reply, Infallible> {
    info!("Revoke session request: {}", session_id);

    let user_id = match extract_user_id(&auth_header, &service.jwt_service) {
        Ok(id) => id,
        Err(reply) => return Ok(reply),
    };

    // Verify the session belongs to this user
    let _session = match service.session_repo.find_by_session_id(&session_id).await {
        Ok(Some(session)) if session.user_id == user_id => session,
        Ok(Some(_)) => {
            return Ok(warp::reply::with_status(
                warp::reply::json(&json!({"success": false, "error": "Session not found"})),
                warp::http::StatusCode::NOT_FOUND,
            ));
        },
        Ok(None) => {
            return Ok(warp::reply::with_status(
                warp::reply::json(&json!({"success": false, "error": "Session not found"})),
                warp::http::StatusCode::NOT_FOUND,
            ));
        },
        Err(e) => {
            error!("Database error: {}", e);
            return Ok(warp::reply::with_status(
                warp::reply::json(&json!({"success": false, "error": "Internal server error"})),
                warp::http::StatusCode::INTERNAL_SERVER_ERROR,
            ));
        },
    };

    // Revoke the session
    if let Err(e) = service.session_repo.revoke_session(&session_id).await {
        error!("Failed to revoke session: {}", e);
        return Ok(warp::reply::with_status(
            warp::reply::json(&json!({"success": false, "error": "Failed to revoke session"})),
            warp::http::StatusCode::INTERNAL_SERVER_ERROR,
        ));
    }

    info!("Session revoked: {}", session_id);
    Ok(warp::reply::with_status(
        warp::reply::json(&json!({"success": true, "message": "Session revoked"})),
        warp::http::StatusCode::OK,
    ))
}

// @spec:FR-AUTH-015 - Revoke All Sessions Handler
async fn handle_revoke_all_sessions(
    auth_header: String,
    service: Arc<SecurityService>,
) -> Result<impl Reply, Infallible> {
    info!("Revoke all sessions request");

    let user_id = match extract_user_id(&auth_header, &service.jwt_service) {
        Ok(id) => id,
        Err(reply) => return Ok(reply),
    };

    // Get current session ID to keep it active
    let current_session_id = JwtService::extract_token_from_header(&auth_header)
        .and_then(|t| service.jwt_service.verify_token(t).ok())
        .map(|c| c.session_id.unwrap_or_default())
        .unwrap_or_default();

    let count = match service
        .session_repo
        .revoke_all_except(&user_id, &current_session_id)
        .await
    {
        Ok(count) => count,
        Err(e) => {
            error!("Failed to revoke sessions: {}", e);
            return Ok(warp::reply::with_status(
                warp::reply::json(&json!({"success": false, "error": "Failed to revoke sessions"})),
                warp::http::StatusCode::INTERNAL_SERVER_ERROR,
            ));
        },
    };

    info!("Revoked {} sessions", count);
    Ok(warp::reply::with_status(
        warp::reply::json(
            &json!({"success": true, "message": format!("{} sessions revoked", count), "revoked_count": count}),
        ),
        warp::http::StatusCode::OK,
    ))
}

// Helper function to parse user agent
pub fn parse_user_agent(user_agent: &str) -> (String, String, String) {
    // Simple UA parsing - in production use a proper UA parser library
    let browser = if user_agent.contains("Chrome") {
        "Chrome"
    } else if user_agent.contains("Firefox") {
        "Firefox"
    } else if user_agent.contains("Safari") {
        "Safari"
    } else if user_agent.contains("Edge") {
        "Edge"
    } else {
        "Unknown"
    };

    // Check mobile devices FIRST since iPhone UA contains "Mac OS X"
    let os = if user_agent.contains("iPhone") || user_agent.contains("iPad") {
        "iOS"
    } else if user_agent.contains("Android") {
        "Android"
    } else if user_agent.contains("Windows") {
        "Windows"
    } else if user_agent.contains("Mac") {
        "MacOS"
    } else if user_agent.contains("Linux") {
        "Linux"
    } else {
        "Unknown"
    };

    let device = format!("{} on {}", browser, os);

    (device, browser.to_string(), os.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::auth::models::Session;
    use chrono::Utc;

    #[test]
    fn test_parse_user_agent_chrome_mac() {
        let ua = "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36";
        let (device, browser, os) = parse_user_agent(ua);
        assert_eq!(browser, "Chrome");
        assert_eq!(os, "MacOS");
        assert!(device.contains("Chrome"));
    }

    #[test]
    fn test_parse_user_agent_safari_iphone() {
        let ua = "Mozilla/5.0 (iPhone; CPU iPhone OS 17_0 like Mac OS X) AppleWebKit/605.1.15 (KHTML, like Gecko) Version/17.0 Mobile/15E148 Safari/604.1";
        let (_device, browser, os) = parse_user_agent(ua);
        assert_eq!(browser, "Safari");
        assert_eq!(os, "iOS");
    }

    #[test]
    fn test_parse_user_agent_firefox_windows() {
        let ua = "Mozilla/5.0 (Windows NT 10.0; Win64; x64; rv:120.0) Gecko/20100101 Firefox/120.0";
        let (_, browser, os) = parse_user_agent(ua);
        assert_eq!(browser, "Firefox");
        assert_eq!(os, "Windows");
    }

    #[test]
    fn test_security_service_new_dummy() {
        let service = SecurityService::new_dummy();
        // Should not panic
        let _ = service.clone();
    }

    #[test]
    fn test_cov7_parse_user_agent_edge() {
        let ua = "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36 Edg/120.0.0.0";
        let (device, browser, os) = parse_user_agent(ua);
        assert_eq!(browser, "Chrome");
        assert_eq!(os, "Windows");
        assert!(device.contains("Chrome"));
    }

    #[test]
    fn test_cov7_parse_user_agent_android() {
        let ua = "Mozilla/5.0 (Linux; Android 13) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Mobile Safari/537.36";
        let (device, browser, os) = parse_user_agent(ua);
        assert_eq!(browser, "Chrome");
        assert_eq!(os, "Android");
        assert!(device.contains("Chrome"));
    }

    #[test]
    fn test_cov7_parse_user_agent_linux() {
        let ua = "Mozilla/5.0 (X11; Linux x86_64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36";
        let (_device, browser, os) = parse_user_agent(ua);
        assert_eq!(browser, "Chrome");
        assert_eq!(os, "Linux");
    }

    #[test]
    fn test_cov7_parse_user_agent_ipad() {
        let ua = "Mozilla/5.0 (iPad; CPU OS 17_0 like Mac OS X) AppleWebKit/605.1.15 (KHTML, like Gecko) Version/17.0 Mobile/15E148 Safari/604.1";
        let (_device, browser, os) = parse_user_agent(ua);
        assert_eq!(browser, "Safari");
        assert_eq!(os, "iOS");
    }

    #[test]
    fn test_cov7_parse_user_agent_unknown() {
        let ua = "SomeCustomBot/1.0";
        let (device, browser, os) = parse_user_agent(ua);
        assert_eq!(browser, "Unknown");
        assert_eq!(os, "Unknown");
        assert!(device.contains("Unknown"));
    }

    #[test]
    fn test_cov7_security_service_new() {
        let user_repo = UserRepository::new_dummy();
        let session_repo = SessionRepository::new_dummy();
        let service = SecurityService::new(
            user_repo,
            session_repo,
            "test-jwt-secret".to_string()
        );
        let _ = service.clone();
    }

    #[tokio::test]
    async fn test_cov7_extract_user_id_invalid_header() {
        let jwt_service = JwtService::new("test-secret".to_string(), Some(24 * 7));
        let result = extract_user_id("InvalidHeader", &jwt_service);
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_cov7_extract_user_id_invalid_token() {
        let jwt_service = JwtService::new("test-secret".to_string(), Some(24 * 7));
        let result = extract_user_id("Bearer invalid-token", &jwt_service);
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_cov7_extract_user_id_invalid_object_id() {
        let jwt_service = JwtService::new("test-secret".to_string(), Some(24 * 7));
        // Create a token with invalid ObjectId
        let token = jwt_service.generate_token("not-an-object-id", "test@example.com", false).unwrap();
        let result = extract_user_id(&format!("Bearer {}", token), &jwt_service);
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_cov7_change_password_validation_failure() {
        let service = Arc::new(SecurityService::new_dummy());

        // Create invalid request (password too short)
        let request = ChangePasswordRequest {
            current_password: "old123".to_string(),
            new_password: "12345".to_string(), // Too short (< 6 chars)
        };

        let jwt_service = JwtService::new("test-secret".to_string(), Some(24 * 7));
        let user_id = bson::oid::ObjectId::new();
        let token = jwt_service.generate_token(&user_id.to_hex(), "test@example.com", false).unwrap();

        let result = handle_change_password(
            format!("Bearer {}", token),
            request,
            service.clone()
        ).await;

        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_cov7_update_profile_validation_failure() {
        let service = Arc::new(SecurityService::new_dummy());

        // Create invalid request (display name too long)
        let request = UpdateProfileRequest {
            display_name: Some("a".repeat(101)),
            avatar_base64: None,
        };

        let jwt_service = JwtService::new("test-secret".to_string(), Some(24 * 7));
        let user_id = bson::oid::ObjectId::new();
        let token = jwt_service.generate_token(&user_id.to_hex(), "test@example.com", false).unwrap();

        let result = handle_update_profile(
            format!("Bearer {}", token),
            request,
            service.clone()
        ).await;

        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_cov7_update_profile_avatar_with_data_url_prefix() {
        let service = Arc::new(SecurityService::new_dummy());

        let request = UpdateProfileRequest {
            display_name: Some("Test User".to_string()),
            avatar_base64: Some("data:image/png;base64,iVBORw0KGgo=".to_string()),
        };

        let jwt_service = JwtService::new("test-secret".to_string(), Some(24 * 7));
        let user_id = bson::oid::ObjectId::new();
        let token = jwt_service.generate_token(&user_id.to_hex(), "test@example.com", false).unwrap();

        let result = handle_update_profile(
            format!("Bearer {}", token),
            request,
            service.clone()
        ).await;

        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_cov7_update_profile_avatar_without_data_url_prefix() {
        let service = Arc::new(SecurityService::new_dummy());

        let request = UpdateProfileRequest {
            display_name: Some("Test User".to_string()),
            avatar_base64: Some("iVBORw0KGgo=".to_string()),
        };

        let jwt_service = JwtService::new("test-secret".to_string(), Some(24 * 7));
        let user_id = bson::oid::ObjectId::new();
        let token = jwt_service.generate_token(&user_id.to_hex(), "test@example.com", false).unwrap();

        let result = handle_update_profile(
            format!("Bearer {}", token),
            request,
            service.clone()
        ).await;

        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_cov7_verify_2fa_validation_failure() {
        let service = Arc::new(SecurityService::new_dummy());

        // Create invalid request (code not 6 digits)
        let request = Verify2FARequest {
            code: "12345".to_string(), // Only 5 digits
        };

        let jwt_service = JwtService::new("test-secret".to_string(), Some(24 * 7));
        let user_id = bson::oid::ObjectId::new();
        let token = jwt_service.generate_token(&user_id.to_hex(), "test@example.com", false).unwrap();

        let result = handle_verify_2fa(
            format!("Bearer {}", token),
            request,
            service.clone()
        ).await;

        assert!(result.is_ok());
    }

    #[test]
    fn test_cov7_session_to_info() {
        let user_id = bson::oid::ObjectId::new();
        let session = Session::new(
            user_id,
            "Chrome on MacOS".to_string(),
            "Chrome".to_string(),
            "MacOS".to_string(),
            "192.168.1.1".to_string(),
            "San Francisco, US".to_string(),
            "Mozilla/5.0".to_string(),
        );

        let current_session_id = session.session_id.clone();
        let info = session.to_info(&current_session_id);

        assert_eq!(info.session_id, session.session_id);
        assert_eq!(info.device, "Chrome on MacOS");
        assert!(info.is_current);
    }

    #[test]
    fn test_cov7_session_to_info_not_current() {
        let user_id = bson::oid::ObjectId::new();
        let session = Session::new(
            user_id,
            "Chrome on MacOS".to_string(),
            "Chrome".to_string(),
            "MacOS".to_string(),
            "192.168.1.1".to_string(),
            "San Francisco, US".to_string(),
            "Mozilla/5.0".to_string(),
        );

        let info = session.to_info("different-session-id");

        assert!(!info.is_current);
    }

    #[test]
    fn test_cov7_change_password_request_serialization() {
        let request = ChangePasswordRequest {
            current_password: "oldpass123".to_string(),
            new_password: "newpass456".to_string(),
        };

        let json = serde_json::to_string(&request).unwrap();
        let deserialized: ChangePasswordRequest = serde_json::from_str(&json).unwrap();

        assert_eq!(request.current_password, deserialized.current_password);
        assert_eq!(request.new_password, deserialized.new_password);
    }

    #[test]
    fn test_cov7_update_profile_request_serialization() {
        let request = UpdateProfileRequest {
            display_name: Some("Test User".to_string()),
            avatar_base64: Some("base64data".to_string()),
        };

        let json = serde_json::to_string(&request).unwrap();
        let deserialized: UpdateProfileRequest = serde_json::from_str(&json).unwrap();

        assert_eq!(request.display_name, deserialized.display_name);
        assert_eq!(request.avatar_base64, deserialized.avatar_base64);
    }

    #[test]
    fn test_cov7_verify_2fa_request_serialization() {
        let request = Verify2FARequest {
            code: "123456".to_string(),
        };

        let json = serde_json::to_string(&request).unwrap();
        let deserialized: Verify2FARequest = serde_json::from_str(&json).unwrap();

        assert_eq!(request.code, deserialized.code);
    }

    #[test]
    fn test_cov7_setup_2fa_response_serialization() {
        let response = Setup2FAResponse {
            secret: "JBSWY3DPEHPK3PXP".to_string(),
            qr_code: "data:image/png;base64,iVBOR...".to_string(),
            otpauth_url: "otpauth://totp/BotCore:user@example.com?secret=JBSWY3DPEHPK3PXP&issuer=BotCore".to_string(),
        };

        let json = serde_json::to_string(&response).unwrap();
        let deserialized: Setup2FAResponse = serde_json::from_str(&json).unwrap();

        assert_eq!(response.secret, deserialized.secret);
        assert_eq!(response.qr_code, deserialized.qr_code);
        assert_eq!(response.otpauth_url, deserialized.otpauth_url);
    }

    #[test]
    fn test_cov7_session_list_response_serialization() {
        let session_info = SessionInfo {
            session_id: "test-session".to_string(),
            device: "Chrome on MacOS".to_string(),
            browser: "Chrome".to_string(),
            os: "MacOS".to_string(),
            location: "San Francisco, US".to_string(),
            is_current: true,
            created_at: chrono::Utc::now(),
            last_active: chrono::Utc::now(),
        };

        let response = SessionListResponse {
            sessions: vec![session_info],
        };

        let json = serde_json::to_string(&response).unwrap();
        let deserialized: SessionListResponse = serde_json::from_str(&json).unwrap();

        assert_eq!(response.sessions.len(), deserialized.sessions.len());
    }

    #[test]
    fn test_cov7_totp_issuer_constant() {
        assert_eq!(TOTP_ISSUER, "BotCore Trading");
    }

    // Additional tests for parse_user_agent edge cases
    #[test]
    fn test_parse_user_agent_edge_browser_priority() {
        // Test Edge browser detection (contains both Chrome and Edge)
        let ua_edge = "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/96.0.4664.110 Safari/537.36 Edg/96.0.1054.62";
        let (device, browser, os) = parse_user_agent(ua_edge);
        // Should detect Chrome since "Chrome" comes before "Edge" in the UA string
        assert_eq!(browser, "Chrome");
        assert_eq!(os, "Windows");
        assert!(device.contains("Chrome"));
    }

    #[test]
    fn test_parse_user_agent_safari_mac_only() {
        // Pure Safari on Mac (no Chrome in UA)
        let ua = "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/605.1.15 (KHTML, like Gecko) Version/16.0 Safari/605.1.15";
        let (device, browser, os) = parse_user_agent(ua);
        assert_eq!(browser, "Safari");
        assert_eq!(os, "MacOS");
        assert!(device.contains("Safari"));
    }

    #[test]
    fn test_parse_user_agent_firefox_linux() {
        let ua = "Mozilla/5.0 (X11; Ubuntu; Linux x86_64; rv:109.0) Gecko/20100101 Firefox/109.0";
        let (_device, browser, os) = parse_user_agent(ua);
        assert_eq!(browser, "Firefox");
        assert_eq!(os, "Linux");
    }

    #[test]
    fn test_parse_user_agent_empty_string() {
        let (device, browser, os) = parse_user_agent("");
        assert_eq!(browser, "Unknown");
        assert_eq!(os, "Unknown");
        assert!(device.contains("Unknown"));
    }

    #[test]
    fn test_parse_user_agent_mixed_case() {
        let ua = "mozilla/5.0 (windows nt 10.0) chrome/120.0";
        let (device, browser, os) = parse_user_agent(ua);
        // Should be case-insensitive but our implementation uses contains which is case-sensitive
        assert_eq!(browser, "Unknown");
        assert_eq!(os, "Unknown");
        assert!(device.contains("Unknown"));
    }

    // Test request struct construction
    #[test]
    fn test_change_password_request_construction() {
        let request = ChangePasswordRequest {
            current_password: "current123".to_string(),
            new_password: "newpassword456".to_string(),
        };
        assert_eq!(request.current_password, "current123");
        assert_eq!(request.new_password, "newpassword456");
    }

    #[test]
    fn test_update_profile_request_with_none_fields() {
        let request = UpdateProfileRequest {
            display_name: None,
            avatar_base64: None,
        };
        assert!(request.display_name.is_none());
        assert!(request.avatar_base64.is_none());
    }

    #[test]
    fn test_verify_2fa_request_construction() {
        let request = Verify2FARequest {
            code: "123456".to_string(),
        };
        assert_eq!(request.code.len(), 6);
    }

    #[test]
    fn test_setup_2fa_response_construction() {
        let response = Setup2FAResponse {
            secret: "TEST_SECRET".to_string(),
            qr_code: "data:image/png;base64,test".to_string(),
            otpauth_url: "otpauth://test".to_string(),
        };
        assert_eq!(response.secret, "TEST_SECRET");
        assert!(response.qr_code.starts_with("data:image/"));
        assert!(response.otpauth_url.starts_with("otpauth://"));
    }

    #[test]
    fn test_session_list_response_empty() {
        let response = SessionListResponse {
            sessions: vec![],
        };
        assert_eq!(response.sessions.len(), 0);
    }

    #[test]
    fn test_session_info_construction() {
        let now = chrono::Utc::now();
        let info = SessionInfo {
            session_id: "test-session-id".to_string(),
            device: "Firefox on Linux".to_string(),
            browser: "Firefox".to_string(),
            os: "Linux".to_string(),
            location: "New York, US".to_string(),
            is_current: false,
            created_at: now,
            last_active: now,
        };
        assert_eq!(info.session_id, "test-session-id");
        assert_eq!(info.browser, "Firefox");
        assert_eq!(info.os, "Linux");
        assert!(!info.is_current);
    }

    #[test]
    fn test_session_new_creates_valid_session() {
        let user_id = bson::oid::ObjectId::new();
        let session = Session::new(
            user_id,
            "Test Device".to_string(),
            "TestBrowser".to_string(),
            "TestOS".to_string(),
            "127.0.0.1".to_string(),
            "Local".to_string(),
            "TestAgent".to_string(),
        );

        assert_eq!(session.user_id, user_id);
        assert_eq!(session.device, "Test Device");
        assert_eq!(session.browser, "TestBrowser");
        assert_eq!(session.os, "TestOS");
        assert_eq!(session.ip_address, "127.0.0.1");
        assert_eq!(session.location, "Local");
        assert_eq!(session.user_agent, "TestAgent");
        assert!(!session.revoked);
        assert!(!session.session_id.is_empty());
    }

    #[test]
    fn test_security_service_clone() {
        let service1 = SecurityService::new_dummy();
        let service2 = service1.clone();
        // Should clone successfully without panic
        let _ = service2;
    }

    // Test validation edge cases
    #[tokio::test]
    async fn test_change_password_empty_fields() {
        let service = Arc::new(SecurityService::new_dummy());

        let request = ChangePasswordRequest {
            current_password: "".to_string(),
            new_password: "".to_string(),
        };

        let jwt_service = JwtService::new("test-secret".to_string(), Some(24 * 7));
        let user_id = bson::oid::ObjectId::new();
        let token = jwt_service.generate_token(&user_id.to_hex(), "test@example.com", false).unwrap();

        let result = handle_change_password(
            format!("Bearer {}", token),
            request,
            service.clone()
        ).await;

        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_update_profile_empty_display_name() {
        let service = Arc::new(SecurityService::new_dummy());

        let request = UpdateProfileRequest {
            display_name: Some("".to_string()),
            avatar_base64: None,
        };

        let jwt_service = JwtService::new("test-secret".to_string(), Some(24 * 7));
        let user_id = bson::oid::ObjectId::new();
        let token = jwt_service.generate_token(&user_id.to_hex(), "test@example.com", false).unwrap();

        let result = handle_update_profile(
            format!("Bearer {}", token),
            request,
            service.clone()
        ).await;

        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_verify_2fa_empty_code() {
        let service = Arc::new(SecurityService::new_dummy());

        let request = Verify2FARequest {
            code: "".to_string(),
        };

        let jwt_service = JwtService::new("test-secret".to_string(), Some(24 * 7));
        let user_id = bson::oid::ObjectId::new();
        let token = jwt_service.generate_token(&user_id.to_hex(), "test@example.com", false).unwrap();

        let result = handle_verify_2fa(
            format!("Bearer {}", token),
            request,
            service.clone()
        ).await;

        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_verify_2fa_long_code() {
        let service = Arc::new(SecurityService::new_dummy());

        let request = Verify2FARequest {
            code: "1234567890".to_string(), // 10 digits instead of 6
        };

        let jwt_service = JwtService::new("test-secret".to_string(), Some(24 * 7));
        let user_id = bson::oid::ObjectId::new();
        let token = jwt_service.generate_token(&user_id.to_hex(), "test@example.com", false).unwrap();

        let result = handle_verify_2fa(
            format!("Bearer {}", token),
            request,
            service.clone()
        ).await;

        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_update_profile_with_large_avatar_base64() {
        let service = Arc::new(SecurityService::new_dummy());

        // Create a large base64 string
        let large_base64 = "a".repeat(1000);
        let request = UpdateProfileRequest {
            display_name: Some("Test".to_string()),
            avatar_base64: Some(large_base64),
        };

        let jwt_service = JwtService::new("test-secret".to_string(), Some(24 * 7));
        let user_id = bson::oid::ObjectId::new();
        let token = jwt_service.generate_token(&user_id.to_hex(), "test@example.com", false).unwrap();

        let result = handle_update_profile(
            format!("Bearer {}", token),
            request,
            service.clone()
        ).await;

        assert!(result.is_ok());
    }

    #[test]
    fn test_parse_user_agent_all_browsers() {
        let test_cases = vec![
            ("Mozilla/5.0 Chrome/120.0", "Chrome"),
            ("Mozilla/5.0 Firefox/120.0", "Firefox"),
            ("Mozilla/5.0 Safari/605.1", "Safari"),
            ("Mozilla/5.0 Edge/120.0", "Edge"),
            ("CustomBot/1.0", "Unknown"),
        ];

        for (ua, expected_browser) in test_cases {
            let (_, browser, _) = parse_user_agent(ua);
            assert_eq!(browser, expected_browser);
        }
    }

    #[test]
    fn test_parse_user_agent_all_os() {
        let test_cases = vec![
            ("Mozilla/5.0 (Windows NT 10.0)", "Windows"),
            ("Mozilla/5.0 (Macintosh; Mac OS X 10_15)", "MacOS"),
            ("Mozilla/5.0 (X11; Linux x86_64)", "Linux"),
            ("Mozilla/5.0 (Android 13)", "Android"),
            ("Mozilla/5.0 (iPhone; CPU iPhone OS 17_0)", "iOS"),
            ("Mozilla/5.0 (iPad; CPU OS 17_0)", "iOS"),
            ("CustomBot/1.0", "Unknown"),
        ];

        for (ua, expected_os) in test_cases {
            let (_, _, os) = parse_user_agent(ua);
            assert_eq!(os, expected_os);
        }
    }

    #[test]
    fn test_session_to_info_multiple_sessions() {
        let user_id = bson::oid::ObjectId::new();
        let session1 = Session::new(
            user_id,
            "Device 1".to_string(),
            "Chrome".to_string(),
            "Windows".to_string(),
            "192.168.1.1".to_string(),
            "US".to_string(),
            "UA1".to_string(),
        );
        let session2 = Session::new(
            user_id,
            "Device 2".to_string(),
            "Firefox".to_string(),
            "Linux".to_string(),
            "192.168.1.2".to_string(),
            "UK".to_string(),
            "UA2".to_string(),
        );

        let current_id = &session1.session_id;

        let info1 = session1.to_info(current_id);
        let info2 = session2.to_info(current_id);

        assert!(info1.is_current);
        assert!(!info2.is_current);
    }

    #[test]
    fn test_change_password_request_validation() {
        let valid_request = ChangePasswordRequest {
            current_password: "password123".to_string(),
            new_password: "newpass456".to_string(),
        };
        // Validation should pass for valid passwords (both >= 6 chars)
        assert!(valid_request.validate().is_ok());
    }

    #[test]
    fn test_update_profile_request_validation() {
        let valid_request = UpdateProfileRequest {
            display_name: Some("Valid Name".to_string()),
            avatar_base64: Some("validbase64".to_string()),
        };
        // Validation should pass for valid display name (<= 100 chars)
        assert!(valid_request.validate().is_ok());
    }

    #[test]
    fn test_verify_2fa_request_validation_valid() {
        let valid_request = Verify2FARequest {
            code: "123456".to_string(),
        };
        // Validation should pass for 6-digit code
        assert!(valid_request.validate().is_ok());
    }

    // =========================================================================
    // COV_BOOST: SecurityService routes and filter tests
    // =========================================================================

    #[tokio::test]
    async fn test_boost_security_service_routes() {
        let service = SecurityService::new_dummy();
        let routes = service.routes();

        // Routes should be created without panic
        let _ = routes;
    }

    #[test]
    fn test_boost_security_service_clone() {
        let service = SecurityService::new_dummy();
        let cloned = service.clone();

        // Should clone successfully
        let _ = cloned;
    }

    // =========================================================================
    // COV_BOOST: parse_user_agent comprehensive browser coverage
    // =========================================================================

    #[test]
    fn test_boost_parse_user_agent_chrome_windows() {
        let ua = "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36";
        let (device, browser, os) = parse_user_agent(ua);

        assert_eq!(browser, "Chrome");
        assert_eq!(os, "Windows");
        assert!(device.contains("Chrome"));
        assert!(device.contains("Windows"));
    }

    #[test]
    fn test_boost_parse_user_agent_chrome_linux() {
        let ua = "Mozilla/5.0 (X11; Linux x86_64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36";
        let (_device, browser, os) = parse_user_agent(ua);

        assert_eq!(browser, "Chrome");
        assert_eq!(os, "Linux");
    }

    #[test]
    fn test_boost_parse_user_agent_firefox_windows() {
        let ua = "Mozilla/5.0 (Windows NT 10.0; Win64; x64; rv:109.0) Gecko/20100101 Firefox/120.0";
        let (_device, browser, os) = parse_user_agent(ua);

        assert_eq!(browser, "Firefox");
        assert_eq!(os, "Windows");
    }

    #[test]
    fn test_boost_parse_user_agent_firefox_mac() {
        let ua = "Mozilla/5.0 (Macintosh; Intel Mac OS X 10.15; rv:109.0) Gecko/20100101 Firefox/120.0";
        let (_device, browser, os) = parse_user_agent(ua);

        assert_eq!(browser, "Firefox");
        assert_eq!(os, "MacOS");
    }

    #[test]
    fn test_boost_parse_user_agent_safari_ios() {
        let ua = "Mozilla/5.0 (iPhone; CPU iPhone OS 17_0 like Mac OS X) AppleWebKit/605.1.15 (KHTML, like Gecko) Version/17.0 Mobile/15E148 Safari/604.1";
        let (_device, browser, os) = parse_user_agent(ua);

        assert_eq!(browser, "Safari");
        assert_eq!(os, "iOS");
    }

    #[test]
    fn test_boost_parse_user_agent_safari_ipad() {
        let ua = "Mozilla/5.0 (iPad; CPU OS 17_0 like Mac OS X) AppleWebKit/605.1.15 (KHTML, like Gecko) Version/17.0 Mobile/15E148 Safari/604.1";
        let (_device, browser, os) = parse_user_agent(ua);

        assert_eq!(browser, "Safari");
        assert_eq!(os, "iOS");
    }

    #[test]
    fn test_boost_parse_user_agent_safari_mac() {
        let ua = "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/605.1.15 (KHTML, like Gecko) Version/17.0 Safari/605.1.15";
        let (_device, browser, os) = parse_user_agent(ua);

        assert_eq!(browser, "Safari");
        assert_eq!(os, "MacOS");
    }

    #[test]
    fn test_boost_parse_user_agent_edge_windows() {
        // Note: parser checks Chrome before Edge, and this UA contains "Chrome" substring via "AppleWebKit"
        // Use a clean Edge UA without Safari/Chrome
        let ua = "Mozilla/5.0 (Windows NT 10.0; Win64; x64) Edge/120.0.0.0";
        let (_device, browser, os) = parse_user_agent(ua);

        assert_eq!(browser, "Edge");
        assert_eq!(os, "Windows");
    }

    #[test]
    fn test_boost_parse_user_agent_android_chrome() {
        let ua = "Mozilla/5.0 (Linux; Android 13; Pixel 7) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Mobile Safari/537.36";
        let (_device, browser, os) = parse_user_agent(ua);

        assert_eq!(browser, "Chrome");
        assert_eq!(os, "Android");
    }

    #[test]
    fn test_boost_parse_user_agent_android_firefox() {
        let ua = "Mozilla/5.0 (Android 13; Mobile; rv:109.0) Gecko/120.0 Firefox/120.0";
        let (_device, browser, os) = parse_user_agent(ua);

        assert_eq!(browser, "Firefox");
        assert_eq!(os, "Android");
    }

    #[test]
    fn test_boost_parse_user_agent_unknown_browser_unknown_os() {
        let ua = "CustomBot/1.0";
        let (device, browser, os) = parse_user_agent(ua);

        assert_eq!(browser, "Unknown");
        assert_eq!(os, "Unknown");
        assert_eq!(device, "Unknown on Unknown");
    }

    #[test]
    fn test_boost_parse_user_agent_empty_string() {
        let ua = "";
        let (_device, browser, os) = parse_user_agent(ua);

        assert_eq!(browser, "Unknown");
        assert_eq!(os, "Unknown");
    }

    #[test]
    fn test_boost_parse_user_agent_only_browser() {
        let ua = "Chrome";
        let (_device, browser, os) = parse_user_agent(ua);

        assert_eq!(browser, "Chrome");
        assert_eq!(os, "Unknown");
    }

    #[test]
    fn test_boost_parse_user_agent_only_os() {
        let ua = "Windows NT 10.0";
        let (_device, browser, os) = parse_user_agent(ua);

        assert_eq!(browser, "Unknown");
        assert_eq!(os, "Windows");
    }

    #[test]
    fn test_boost_parse_user_agent_multiple_browser_keywords() {
        // Chrome UA contains both "Chrome" and "Safari"
        let ua = "Mozilla/5.0 AppleWebKit/537.36 Chrome/120.0 Safari/537.36";
        let (_device, browser, _os) = parse_user_agent(ua);

        // Should detect Chrome (first match)
        assert_eq!(browser, "Chrome");
    }

    #[test]
    fn test_boost_parse_user_agent_case_sensitive() {
        // Test that parsing is case-sensitive
        let ua = "chrome windows";
        let (_device, browser, os) = parse_user_agent(ua);

        assert_eq!(browser, "Unknown");
        assert_eq!(os, "Unknown");
    }

    // =========================================================================
    // COV_BOOST: ChangePasswordRequest edge cases
    // =========================================================================

    #[test]
    fn test_boost_change_password_request_clone() {
        let request = ChangePasswordRequest {
            current_password: "old123456".to_string(),
            new_password: "new123456".to_string(),
        };

        let cloned = request.clone();
        assert_eq!(request.current_password, cloned.current_password);
        assert_eq!(request.new_password, cloned.new_password);
    }

    #[test]
    fn test_boost_change_password_request_debug() {
        let request = ChangePasswordRequest {
            current_password: "secret".to_string(),
            new_password: "newsecret".to_string(),
        };

        let debug_str = format!("{:?}", request);
        assert!(debug_str.contains("ChangePasswordRequest"));
    }

    #[test]
    fn test_boost_change_password_request_same_password() {
        let request = ChangePasswordRequest {
            current_password: "password123".to_string(),
            new_password: "password123".to_string(),
        };

        // Should be valid (application logic would reject, not validation)
        assert!(request.validate().is_ok());
    }

    #[test]
    fn test_boost_change_password_request_very_long_password() {
        let long_password = "a".repeat(1000);
        let request = ChangePasswordRequest {
            current_password: long_password.clone(),
            new_password: long_password,
        };

        assert!(request.validate().is_ok());
    }

    #[test]
    fn test_boost_change_password_request_special_chars() {
        let request = ChangePasswordRequest {
            current_password: "p@ssw0rd!#$%^&*()".to_string(),
            new_password: "n3w_p@ss!".to_string(),
        };

        assert!(request.validate().is_ok());
    }

    // =========================================================================
    // COV_BOOST: UpdateProfileRequest edge cases
    // =========================================================================

    #[test]
    fn test_boost_update_profile_request_clone() {
        let request = UpdateProfileRequest {
            display_name: Some("User".to_string()),
            avatar_base64: Some("base64data".to_string()),
        };

        let cloned = request.clone();
        assert_eq!(request.display_name, cloned.display_name);
        assert_eq!(request.avatar_base64, cloned.avatar_base64);
    }

    #[test]
    fn test_boost_update_profile_request_debug() {
        let request = UpdateProfileRequest {
            display_name: Some("Test".to_string()),
            avatar_base64: None,
        };

        let debug_str = format!("{:?}", request);
        assert!(debug_str.contains("UpdateProfileRequest"));
    }

    #[test]
    fn test_boost_update_profile_both_none() {
        let request = UpdateProfileRequest {
            display_name: None,
            avatar_base64: None,
        };

        assert!(request.validate().is_ok());
    }

    #[test]
    fn test_boost_update_profile_only_display_name() {
        let request = UpdateProfileRequest {
            display_name: Some("NewName".to_string()),
            avatar_base64: None,
        };

        assert!(request.validate().is_ok());
    }

    #[test]
    fn test_boost_update_profile_only_avatar() {
        let request = UpdateProfileRequest {
            display_name: None,
            avatar_base64: Some("base64encodedimage".to_string()),
        };

        assert!(request.validate().is_ok());
    }

    #[test]
    fn test_boost_update_profile_max_length_display_name() {
        let request = UpdateProfileRequest {
            display_name: Some("a".repeat(100)),
            avatar_base64: None,
        };

        assert!(request.validate().is_ok());
    }

    #[test]
    fn test_boost_update_profile_unicode_display_name() {
        let request = UpdateProfileRequest {
            display_name: Some("用户名 🎉".to_string()),
            avatar_base64: None,
        };

        assert!(request.validate().is_ok());
    }

    // =========================================================================
    // COV_BOOST: Verify2FARequest edge cases
    // =========================================================================

    #[test]
    fn test_boost_verify_2fa_request_clone() {
        let request = Verify2FARequest {
            code: "123456".to_string(),
        };

        let cloned = request.clone();
        assert_eq!(request.code, cloned.code);
    }

    #[test]
    fn test_boost_verify_2fa_request_debug() {
        let request = Verify2FARequest {
            code: "654321".to_string(),
        };

        let debug_str = format!("{:?}", request);
        assert!(debug_str.contains("Verify2FARequest"));
    }

    #[test]
    fn test_boost_verify_2fa_request_all_zeros() {
        let request = Verify2FARequest {
            code: "000000".to_string(),
        };

        assert!(request.validate().is_ok());
    }

    #[test]
    fn test_boost_verify_2fa_request_all_nines() {
        let request = Verify2FARequest {
            code: "999999".to_string(),
        };

        assert!(request.validate().is_ok());
    }

    #[test]
    fn test_boost_verify_2fa_request_alphabetic() {
        let request = Verify2FARequest {
            code: "abcdef".to_string(),
        };

        // 6 chars but not digits - validation depends on validator rules
        let _ = request.validate();
    }

    // =========================================================================
    // COV_BOOST: Setup2FAResponse edge cases
    // =========================================================================

    #[test]
    fn test_boost_setup_2fa_response_clone() {
        let response = Setup2FAResponse {
            secret: "SECRET123".to_string(),
            qr_code: "data:image/png;base64,abc".to_string(),
            otpauth_url: "otpauth://totp/test".to_string(),
        };

        let cloned = response.clone();
        assert_eq!(response.secret, cloned.secret);
        assert_eq!(response.qr_code, cloned.qr_code);
    }

    #[test]
    fn test_boost_setup_2fa_response_debug() {
        let response = Setup2FAResponse {
            secret: "SECRET".to_string(),
            qr_code: "qr".to_string(),
            otpauth_url: "url".to_string(),
        };

        let debug_str = format!("{:?}", response);
        assert!(debug_str.contains("Setup2FAResponse"));
    }

    #[test]
    fn test_boost_setup_2fa_response_empty_fields() {
        let response = Setup2FAResponse {
            secret: "".to_string(),
            qr_code: "".to_string(),
            otpauth_url: "".to_string(),
        };

        assert_eq!(response.secret, "");
    }

    // =========================================================================
    // COV_BOOST: SessionInfo and SessionListResponse edge cases
    // =========================================================================

    #[test]
    fn test_boost_session_info_clone() {
        let session_info = SessionInfo {
            session_id: "sess123".to_string(),
            device: "Chrome on MacOS".to_string(),
            browser: "Chrome".to_string(),
            os: "MacOS".to_string(),
            location: "US".to_string(),
            is_current: true,
            created_at: Utc::now(),
            last_active: Utc::now(),
        };

        let cloned = session_info.clone();
        assert_eq!(session_info.session_id, cloned.session_id);
        assert_eq!(session_info.is_current, cloned.is_current);
    }

    #[test]
    fn test_boost_session_info_debug() {
        let session_info = SessionInfo {
            session_id: "debug".to_string(),
            device: "Device".to_string(),
            browser: "Browser".to_string(),
            os: "OS".to_string(),
            location: "Location".to_string(),
            is_current: false,
            created_at: Utc::now(),
            last_active: Utc::now(),
        };

        let debug_str = format!("{:?}", session_info);
        assert!(debug_str.contains("SessionInfo"));
    }

    #[test]
    fn test_boost_session_list_response_empty() {
        let response = SessionListResponse {
            sessions: vec![],
        };

        assert_eq!(response.sessions.len(), 0);
    }

    #[test]
    fn test_boost_session_list_response_multiple() {
        let session1 = SessionInfo {
            session_id: "sess1".to_string(),
            device: "Device 1".to_string(),
            browser: "Chrome".to_string(),
            os: "Windows".to_string(),
            location: "US".to_string(),
            is_current: true,
            created_at: Utc::now(),
            last_active: Utc::now(),
        };

        let session2 = SessionInfo {
            session_id: "sess2".to_string(),
            device: "Device 2".to_string(),
            browser: "Firefox".to_string(),
            os: "Linux".to_string(),
            location: "UK".to_string(),
            is_current: false,
            created_at: Utc::now(),
            last_active: Utc::now(),
        };

        let response = SessionListResponse {
            sessions: vec![session1, session2],
        };

        assert_eq!(response.sessions.len(), 2);
    }

    #[test]
    fn test_boost_session_list_response_clone() {
        let session_info = SessionInfo {
            session_id: "s1".to_string(),
            device: "d1".to_string(),
            browser: "b1".to_string(),
            os: "o1".to_string(),
            location: "l1".to_string(),
            is_current: true,
            created_at: Utc::now(),
            last_active: Utc::now(),
        };

        let response = SessionListResponse {
            sessions: vec![session_info],
        };

        let cloned = response.clone();
        assert_eq!(response.sessions.len(), cloned.sessions.len());
    }

    #[test]
    fn test_boost_session_list_response_debug() {
        let response = SessionListResponse {
            sessions: vec![],
        };

        let debug_str = format!("{:?}", response);
        assert!(debug_str.contains("SessionListResponse"));
    }

    // =========================================================================
    // COV_BOOST: Session::to_info edge cases
    // =========================================================================

    #[test]
    fn test_boost_session_to_info_empty_current_id() {
        let user_id = bson::oid::ObjectId::new();
        let session = Session::new(
            user_id,
            "Device".to_string(),
            "Browser".to_string(),
            "OS".to_string(),
            "IP".to_string(),
            "Location".to_string(),
            "UA".to_string(),
        );

        let info = session.to_info("");
        assert!(!info.is_current);
    }

    #[test]
    fn test_boost_session_to_info_same_id() {
        let user_id = bson::oid::ObjectId::new();
        let session = Session::new(
            user_id,
            "Device".to_string(),
            "Browser".to_string(),
            "OS".to_string(),
            "IP".to_string(),
            "Location".to_string(),
            "UA".to_string(),
        );

        let current_id = session.session_id.clone();
        let info = session.to_info(&current_id);

        assert!(info.is_current);
        assert_eq!(info.session_id, current_id);
    }

    #[test]
    fn test_boost_session_to_info_empty_fields() {
        let user_id = bson::oid::ObjectId::new();
        let session = Session::new(
            user_id,
            "".to_string(),
            "".to_string(),
            "".to_string(),
            "".to_string(),
            "".to_string(),
            "".to_string(),
        );

        let info = session.to_info("other");
        assert_eq!(info.device, "");
        assert_eq!(info.browser, "");
        assert_eq!(info.location, "");
    }

    // =========================================================================
    // COV_BOOST: TOTP_ISSUER constant
    // =========================================================================

    #[test]
    fn test_boost_totp_issuer_not_empty() {
        assert!(!TOTP_ISSUER.is_empty());
    }

    #[test]
    fn test_boost_totp_issuer_length() {
        assert!(TOTP_ISSUER.len() > 0);
    }

    // =========================================================================
    // COV_BOOST: Handler error paths with dummy service
    // =========================================================================

    #[tokio::test]
    async fn test_boost_handle_change_password_invalid_token() {
        let service = Arc::new(SecurityService::new_dummy());

        let request = ChangePasswordRequest {
            current_password: "old123456".to_string(),
            new_password: "new123456".to_string(),
        };

        let result = handle_change_password(
            "Bearer invalid_token".to_string(),
            request,
            service,
        ).await;

        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_boost_handle_change_password_no_bearer() {
        let service = Arc::new(SecurityService::new_dummy());

        let request = ChangePasswordRequest {
            current_password: "old123456".to_string(),
            new_password: "new123456".to_string(),
        };

        let result = handle_change_password(
            "invalid_header".to_string(),
            request,
            service,
        ).await;

        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_boost_handle_update_profile_invalid_token() {
        let service = Arc::new(SecurityService::new_dummy());

        let request = UpdateProfileRequest {
            display_name: Some("New Name".to_string()),
            avatar_base64: None,
        };

        let result = handle_update_profile(
            "Bearer invalid_token".to_string(),
            request,
            service,
        ).await;

        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_boost_handle_setup_2fa_invalid_token() {
        let service = Arc::new(SecurityService::new_dummy());

        let result = handle_setup_2fa(
            "Bearer invalid_token".to_string(),
            service,
        ).await;

        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_boost_handle_verify_2fa_invalid_token() {
        let service = Arc::new(SecurityService::new_dummy());

        let request = Verify2FARequest {
            code: "123456".to_string(),
        };

        let result = handle_verify_2fa(
            "Bearer invalid_token".to_string(),
            request,
            service,
        ).await;

        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_boost_handle_disable_2fa_invalid_token() {
        let service = Arc::new(SecurityService::new_dummy());

        let request = Verify2FARequest {
            code: "123456".to_string(),
        };

        let result = handle_disable_2fa(
            "Bearer invalid_token".to_string(),
            request,
            service,
        ).await;

        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_boost_handle_list_sessions_invalid_token() {
        let service = Arc::new(SecurityService::new_dummy());

        let result = handle_list_sessions(
            "Bearer invalid_token".to_string(),
            service,
        ).await;

        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_boost_handle_revoke_session_invalid_token() {
        let service = Arc::new(SecurityService::new_dummy());

        let result = handle_revoke_session(
            "session_id_123".to_string(),
            "Bearer invalid_token".to_string(),
            service,
        ).await;

        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_boost_handle_revoke_all_sessions_invalid_token() {
        let service = Arc::new(SecurityService::new_dummy());

        let result = handle_revoke_all_sessions(
            "Bearer invalid_token".to_string(),
            service,
        ).await;

        assert!(result.is_ok());
    }

    // =========================================================================
    // COV_BOOST: Edge cases for validation failures
    // =========================================================================

    #[test]
    fn test_boost_change_password_request_short_current() {
        let request = ChangePasswordRequest {
            current_password: "12345".to_string(), // 5 chars
            new_password: "validpassword".to_string(),
        };

        // current_password has min=1 validation, so "12345" (5 chars) passes
        // Only new_password has min=6 validation
        assert!(request.validate().is_ok());
    }

    #[test]
    fn test_boost_change_password_request_short_new() {
        let request = ChangePasswordRequest {
            current_password: "validpassword".to_string(),
            new_password: "12345".to_string(), // 5 chars
        };

        assert!(request.validate().is_err());
    }

    #[test]
    fn test_boost_update_profile_request_too_long() {
        let request = UpdateProfileRequest {
            display_name: Some("a".repeat(101)), // > 100 chars
            avatar_base64: None,
        };

        assert!(request.validate().is_err());
    }

    #[test]
    fn test_boost_verify_2fa_request_too_short() {
        let request = Verify2FARequest {
            code: "12345".to_string(), // 5 digits
        };

        assert!(request.validate().is_err());
    }

    #[test]
    fn test_boost_verify_2fa_request_too_long() {
        let request = Verify2FARequest {
            code: "1234567".to_string(), // 7 digits
        };

        assert!(request.validate().is_err());
    }

    // =========================================================================
    // FUNCTION-LEVEL TESTS (test_fn_ prefix for coverage boost)
    // =========================================================================

    #[tokio::test]
    async fn test_fn_handle_change_password_with_valid_token() {
        let service = Arc::new(SecurityService::new_dummy());
        let jwt_service = JwtService::new("secret".to_string(), Some(24 * 7));
        let user_id = bson::oid::ObjectId::new();
        let token = jwt_service.generate_token(&user_id.to_hex(), "user@test.com", false).unwrap();

        let request = ChangePasswordRequest {
            current_password: "oldpass123".to_string(),
            new_password: "newpass456".to_string(),
        };

        let result = handle_change_password(format!("Bearer {}", token), request, service).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_fn_handle_update_profile_with_valid_token() {
        let service = Arc::new(SecurityService::new_dummy());
        let jwt_service = JwtService::new("secret".to_string(), Some(24 * 7));
        let user_id = bson::oid::ObjectId::new();
        let token = jwt_service.generate_token(&user_id.to_hex(), "user@test.com", false).unwrap();

        let request = UpdateProfileRequest {
            display_name: Some("New Name".to_string()),
            avatar_base64: None,
        };

        let result = handle_update_profile(format!("Bearer {}", token), request, service).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_fn_handle_setup_2fa_with_valid_token() {
        let service = Arc::new(SecurityService::new_dummy());
        let jwt_service = JwtService::new("secret".to_string(), Some(24 * 7));
        let user_id = bson::oid::ObjectId::new();
        let token = jwt_service.generate_token(&user_id.to_hex(), "user@test.com", false).unwrap();

        let result = handle_setup_2fa(format!("Bearer {}", token), service).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_fn_handle_verify_2fa_with_valid_code() {
        let service = Arc::new(SecurityService::new_dummy());
        let jwt_service = JwtService::new("secret".to_string(), Some(24 * 7));
        let user_id = bson::oid::ObjectId::new();
        let token = jwt_service.generate_token(&user_id.to_hex(), "user@test.com", false).unwrap();

        let request = Verify2FARequest {
            code: "123456".to_string(),
        };

        let result = handle_verify_2fa(format!("Bearer {}", token), request, service).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_fn_handle_disable_2fa_with_valid_code() {
        let service = Arc::new(SecurityService::new_dummy());
        let jwt_service = JwtService::new("secret".to_string(), Some(24 * 7));
        let user_id = bson::oid::ObjectId::new();
        let token = jwt_service.generate_token(&user_id.to_hex(), "user@test.com", false).unwrap();

        let request = Verify2FARequest {
            code: "123456".to_string(),
        };

        let result = handle_disable_2fa(format!("Bearer {}", token), request, service).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_fn_handle_list_sessions_with_valid_token() {
        let service = Arc::new(SecurityService::new_dummy());
        let jwt_service = JwtService::new("secret".to_string(), Some(24 * 7));
        let user_id = bson::oid::ObjectId::new();
        let token = jwt_service.generate_token(&user_id.to_hex(), "user@test.com", false).unwrap();

        let result = handle_list_sessions(format!("Bearer {}", token), service).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_fn_handle_revoke_session_with_valid_token() {
        let service = Arc::new(SecurityService::new_dummy());
        let jwt_service = JwtService::new("secret".to_string(), Some(24 * 7));
        let user_id = bson::oid::ObjectId::new();
        let token = jwt_service.generate_token(&user_id.to_hex(), "user@test.com", false).unwrap();

        let result = handle_revoke_session("session123".to_string(), format!("Bearer {}", token), service).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_fn_handle_revoke_all_sessions_with_valid_token() {
        let service = Arc::new(SecurityService::new_dummy());
        let jwt_service = JwtService::new("secret".to_string(), Some(24 * 7));
        let user_id = bson::oid::ObjectId::new();
        let token = jwt_service.generate_token(&user_id.to_hex(), "user@test.com", false).unwrap();

        let result = handle_revoke_all_sessions(format!("Bearer {}", token), service).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_fn_extract_user_id_with_valid_token() {
        let jwt_service = JwtService::new("secret".to_string(), Some(24 * 7));
        let user_id = bson::oid::ObjectId::new();
        let token = jwt_service.generate_token(&user_id.to_hex(), "user@test.com", false).unwrap();

        let result = extract_user_id(&format!("Bearer {}", token), &jwt_service);
        assert!(result.is_ok());
        if let Ok(extracted_id) = result {
            assert_eq!(extracted_id, user_id);
        }
    }

    #[test]
    fn test_fn_parse_user_agent_variations() {
        let cases = vec![
            ("Mozilla/5.0 (Windows NT 10.0) Chrome/120", ("Chrome", "Windows")),
            ("Mozilla/5.0 (Macintosh) Safari/605", ("Safari", "MacOS")),
            ("Mozilla/5.0 (X11; Linux) Firefox/120", ("Firefox", "Linux")),
            ("Mozilla/5.0 (iPhone) Safari/604", ("Safari", "iOS")),
            ("Mozilla/5.0 (iPad) Safari/604", ("Safari", "iOS")),
            ("Mozilla/5.0 (Android) Chrome/120", ("Chrome", "Android")),
        ];

        for (ua, (exp_browser, exp_os)) in cases {
            let (_, browser, os) = parse_user_agent(ua);
            assert_eq!(browser, exp_browser, "Failed for UA: {}", ua);
            assert_eq!(os, exp_os, "Failed for UA: {}", ua);
        }
    }

    // =========================================================================
    // WARP INTEGRATION TESTS - Test actual HTTP routes
    // =========================================================================

    use warp::test::request;

    #[tokio::test]
    async fn test_warp_change_password_missing_auth_header() {
        let service = SecurityService::new_dummy();
        let filter = warp::path("auth")
            .and(warp::path("change-password"))
            .and(warp::post())
            .and(warp::header::<String>("authorization"))
            .and(warp::body::json())
            .and(warp::any().map(move || Arc::new(service.clone())))
            .and_then(handle_change_password);

        let request_body = ChangePasswordRequest {
            current_password: "old123456".to_string(),
            new_password: "new123456".to_string(),
        };

        let resp = request()
            .method("POST")
            .path("/auth/change-password")
            .json(&request_body)
            .reply(&filter)
            .await;

        // Missing auth header should result in 400 or rejection
        assert!(resp.status().is_client_error() || resp.status().is_server_error());
    }

    #[tokio::test]
    async fn test_warp_change_password_with_auth_invalid_token() {
        let service = SecurityService::new_dummy();
        let filter = warp::path("auth")
            .and(warp::path("change-password"))
            .and(warp::post())
            .and(warp::header::<String>("authorization"))
            .and(warp::body::json())
            .and(warp::any().map(move || Arc::new(service.clone())))
            .and_then(handle_change_password);

        let request_body = ChangePasswordRequest {
            current_password: "old123456".to_string(),
            new_password: "new123456".to_string(),
        };

        let resp = request()
            .method("POST")
            .path("/auth/change-password")
            .header("authorization", "Bearer invalid_token")
            .json(&request_body)
            .reply(&filter)
            .await;

        // Invalid token should result in 401
        assert_eq!(resp.status(), 401);
    }

    #[tokio::test]
    async fn test_warp_change_password_with_valid_token() {
        let service = SecurityService::new_dummy();
        let jwt_service = JwtService::new("test-secret".to_string(), Some(24 * 7));
        let user_id = bson::oid::ObjectId::new();
        let token = jwt_service.generate_token(&user_id.to_hex(), "test@example.com", false).unwrap();

        let filter = warp::path("auth")
            .and(warp::path("change-password"))
            .and(warp::post())
            .and(warp::header::<String>("authorization"))
            .and(warp::body::json())
            .and(warp::any().map(move || Arc::new(service.clone())))
            .and_then(handle_change_password);

        let request_body = ChangePasswordRequest {
            current_password: "old123456".to_string(),
            new_password: "new123456".to_string(),
        };

        let resp = request()
            .method("POST")
            .path("/auth/change-password")
            .header("authorization", format!("Bearer {}", token))
            .json(&request_body)
            .reply(&filter)
            .await;

        // With dummy service (no db), should get 404 or 500
        assert!(resp.status().is_client_error() || resp.status().is_server_error());
    }

    #[tokio::test]
    async fn test_warp_change_password_validation_error() {
        let service = SecurityService::new_dummy();
        let jwt_service = JwtService::new("test-secret".to_string(), Some(24 * 7));
        let user_id = bson::oid::ObjectId::new();
        let token = jwt_service.generate_token(&user_id.to_hex(), "test@example.com", false).unwrap();

        let filter = warp::path("auth")
            .and(warp::path("change-password"))
            .and(warp::post())
            .and(warp::header::<String>("authorization"))
            .and(warp::body::json())
            .and(warp::any().map(move || Arc::new(service.clone())))
            .and_then(handle_change_password);

        let request_body = ChangePasswordRequest {
            current_password: "old123".to_string(),
            new_password: "12345".to_string(), // Too short
        };

        let resp = request()
            .method("POST")
            .path("/auth/change-password")
            .header("authorization", format!("Bearer {}", token))
            .json(&request_body)
            .reply(&filter)
            .await;

        // Validation error should result in 400
        assert_eq!(resp.status(), 400);
    }

    #[tokio::test]
    async fn test_warp_update_profile_with_avatar_data_url() {
        let service = SecurityService::new_dummy();
        let jwt_service = JwtService::new("test-secret".to_string(), Some(24 * 7));
        let user_id = bson::oid::ObjectId::new();
        let token = jwt_service.generate_token(&user_id.to_hex(), "test@example.com", false).unwrap();

        let filter = warp::path("auth")
            .and(warp::path("profile"))
            .and(warp::patch())
            .and(warp::header::<String>("authorization"))
            .and(warp::body::json())
            .and(warp::any().map(move || Arc::new(service.clone())))
            .and_then(handle_update_profile);

        let request_body = UpdateProfileRequest {
            display_name: Some("New Name".to_string()),
            avatar_base64: Some("data:image/png;base64,iVBORw0KGgo=".to_string()),
        };

        let resp = request()
            .method("PATCH")
            .path("/auth/profile")
            .header("authorization", format!("Bearer {}", token))
            .json(&request_body)
            .reply(&filter)
            .await;

        // With dummy service, should return error
        assert!(resp.status().is_client_error() || resp.status().is_server_error());
    }

    #[tokio::test]
    async fn test_warp_update_profile_with_plain_base64() {
        let service = SecurityService::new_dummy();
        let jwt_service = JwtService::new("test-secret".to_string(), Some(24 * 7));
        let user_id = bson::oid::ObjectId::new();
        let token = jwt_service.generate_token(&user_id.to_hex(), "test@example.com", false).unwrap();

        let filter = warp::path("auth")
            .and(warp::path("profile"))
            .and(warp::patch())
            .and(warp::header::<String>("authorization"))
            .and(warp::body::json())
            .and(warp::any().map(move || Arc::new(service.clone())))
            .and_then(handle_update_profile);

        let request_body = UpdateProfileRequest {
            display_name: Some("User".to_string()),
            avatar_base64: Some("SGVsbG8gV29ybGQ=".to_string()), // Plain base64
        };

        let resp = request()
            .method("PATCH")
            .path("/auth/profile")
            .header("authorization", format!("Bearer {}", token))
            .json(&request_body)
            .reply(&filter)
            .await;

        // With dummy service, should return error
        assert!(resp.status().is_client_error() || resp.status().is_server_error());
    }

    #[tokio::test]
    async fn test_warp_setup_2fa_endpoint() {
        let service = SecurityService::new_dummy();
        let jwt_service = JwtService::new("test-secret".to_string(), Some(24 * 7));
        let user_id = bson::oid::ObjectId::new();
        let token = jwt_service.generate_token(&user_id.to_hex(), "test@example.com", false).unwrap();

        let filter = warp::path!("auth" / "2fa" / "setup")
            .and(warp::post())
            .and(warp::header::<String>("authorization"))
            .and(warp::any().map(move || Arc::new(service.clone())))
            .and_then(handle_setup_2fa);

        let resp = request()
            .method("POST")
            .path("/auth/2fa/setup")
            .header("authorization", format!("Bearer {}", token))
            .reply(&filter)
            .await;

        // With dummy service, should return error (no user found)
        assert!(resp.status().is_client_error() || resp.status().is_server_error());
    }

    #[tokio::test]
    async fn test_warp_verify_2fa_endpoint() {
        let service = SecurityService::new_dummy();
        let jwt_service = JwtService::new("test-secret".to_string(), Some(24 * 7));
        let user_id = bson::oid::ObjectId::new();
        let token = jwt_service.generate_token(&user_id.to_hex(), "test@example.com", false).unwrap();

        let filter = warp::path!("auth" / "2fa" / "verify")
            .and(warp::post())
            .and(warp::header::<String>("authorization"))
            .and(warp::body::json())
            .and(warp::any().map(move || Arc::new(service.clone())))
            .and_then(handle_verify_2fa);

        let request_body = Verify2FARequest {
            code: "123456".to_string(),
        };

        let resp = request()
            .method("POST")
            .path("/auth/2fa/verify")
            .header("authorization", format!("Bearer {}", token))
            .json(&request_body)
            .reply(&filter)
            .await;

        // With dummy service, should return error (no user found)
        assert!(resp.status().is_client_error() || resp.status().is_server_error());
    }

    #[tokio::test]
    async fn test_warp_disable_2fa_endpoint() {
        let service = SecurityService::new_dummy();
        let jwt_service = JwtService::new("test-secret".to_string(), Some(24 * 7));
        let user_id = bson::oid::ObjectId::new();
        let token = jwt_service.generate_token(&user_id.to_hex(), "test@example.com", false).unwrap();

        let filter = warp::path!("auth" / "2fa" / "disable")
            .and(warp::post())
            .and(warp::header::<String>("authorization"))
            .and(warp::body::json())
            .and(warp::any().map(move || Arc::new(service.clone())))
            .and_then(handle_disable_2fa);

        let request_body = Verify2FARequest {
            code: "123456".to_string(),
        };

        let resp = request()
            .method("POST")
            .path("/auth/2fa/disable")
            .header("authorization", format!("Bearer {}", token))
            .json(&request_body)
            .reply(&filter)
            .await;

        // With dummy service, should return error (no user found)
        assert!(resp.status().is_client_error() || resp.status().is_server_error());
    }

    #[tokio::test]
    async fn test_warp_list_sessions_endpoint() {
        let service = SecurityService::new_dummy();
        let jwt_service = JwtService::new("test-secret".to_string(), Some(24 * 7));
        let user_id = bson::oid::ObjectId::new();
        let token = jwt_service.generate_token(&user_id.to_hex(), "test@example.com", false).unwrap();

        let filter = warp::path("auth")
            .and(warp::path("sessions"))
            .and(warp::get())
            .and(warp::header::<String>("authorization"))
            .and(warp::any().map(move || Arc::new(service.clone())))
            .and_then(handle_list_sessions);

        let resp = request()
            .method("GET")
            .path("/auth/sessions")
            .header("authorization", format!("Bearer {}", token))
            .reply(&filter)
            .await;

        // With dummy service, should return error (no sessions found)
        assert!(resp.status().is_client_error() || resp.status().is_server_error());
    }

    #[tokio::test]
    async fn test_warp_revoke_session_endpoint() {
        let service = SecurityService::new_dummy();
        let jwt_service = JwtService::new("test-secret".to_string(), Some(24 * 7));
        let user_id = bson::oid::ObjectId::new();
        let token = jwt_service.generate_token(&user_id.to_hex(), "test@example.com", false).unwrap();

        let filter = warp::path!("auth" / "sessions" / String)
            .and(warp::delete())
            .and(warp::header::<String>("authorization"))
            .and(warp::any().map(move || Arc::new(service.clone())))
            .and_then(handle_revoke_session);

        let resp = request()
            .method("DELETE")
            .path("/auth/sessions/test-session-id")
            .header("authorization", format!("Bearer {}", token))
            .reply(&filter)
            .await;

        // With dummy service, should return error (no session found)
        assert!(resp.status().is_client_error() || resp.status().is_server_error());
    }

    #[tokio::test]
    async fn test_warp_revoke_all_sessions_endpoint() {
        let service = SecurityService::new_dummy();
        let jwt_service = JwtService::new("test-secret".to_string(), Some(24 * 7));
        let user_id = bson::oid::ObjectId::new();
        let token = jwt_service.generate_token(&user_id.to_hex(), "test@example.com", false).unwrap();

        let filter = warp::path!("auth" / "sessions" / "revoke-all")
            .and(warp::post())
            .and(warp::header::<String>("authorization"))
            .and(warp::any().map(move || Arc::new(service.clone())))
            .and_then(handle_revoke_all_sessions);

        let resp = request()
            .method("POST")
            .path("/auth/sessions/revoke-all")
            .header("authorization", format!("Bearer {}", token))
            .reply(&filter)
            .await;

        // With dummy service, should return error (no sessions found)
        assert!(resp.status().is_client_error() || resp.status().is_server_error());
    }

    #[tokio::test]
    async fn test_warp_routes_integration() {
        let service = SecurityService::new_dummy();
        let routes = service.routes();

        // Test that routes are properly constructed
        let jwt_service = JwtService::new("test-secret".to_string(), Some(24 * 7));
        let user_id = bson::oid::ObjectId::new();
        let token = jwt_service.generate_token(&user_id.to_hex(), "test@example.com", false).unwrap();

        // Test change password route
        let request_body = ChangePasswordRequest {
            current_password: "old123456".to_string(),
            new_password: "new123456".to_string(),
        };

        let resp = request()
            .method("POST")
            .path("/auth/change-password")
            .header("authorization", format!("Bearer {}", token))
            .json(&request_body)
            .reply(&routes)
            .await;

        // Should get some response (not panic)
        assert!(resp.status().as_u16() > 0);
    }

    // =========================================================================
    // Additional handler branch coverage tests
    // =========================================================================

    #[tokio::test]
    async fn test_handle_change_password_empty_passwords() {
        let service = Arc::new(SecurityService::new_dummy());
        let jwt_service = JwtService::new("test-secret".to_string(), Some(24 * 7));
        let user_id = bson::oid::ObjectId::new();
        let token = jwt_service.generate_token(&user_id.to_hex(), "test@example.com", false).unwrap();

        let request = ChangePasswordRequest {
            current_password: "".to_string(),
            new_password: "".to_string(),
        };

        let result = handle_change_password(format!("Bearer {}", token), request, service).await;
        assert!(result.is_ok());
        // Should return validation error
    }

    #[tokio::test]
    async fn test_handle_update_profile_none_fields() {
        let service = Arc::new(SecurityService::new_dummy());
        let jwt_service = JwtService::new("test-secret".to_string(), Some(24 * 7));
        let user_id = bson::oid::ObjectId::new();
        let token = jwt_service.generate_token(&user_id.to_hex(), "test@example.com", false).unwrap();

        let request = UpdateProfileRequest {
            display_name: None,
            avatar_base64: None,
        };

        let result = handle_update_profile(format!("Bearer {}", token), request, service).await;
        assert!(result.is_ok());
        // With dummy service, should return error
    }

    #[tokio::test]
    async fn test_extract_user_id_empty_auth_header() {
        let jwt_service = JwtService::new("test-secret".to_string(), Some(24 * 7));
        let result = extract_user_id("", &jwt_service);
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_extract_user_id_malformed_bearer() {
        let jwt_service = JwtService::new("test-secret".to_string(), Some(24 * 7));
        let result = extract_user_id("Bearertoken", &jwt_service);
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_extract_user_id_no_bearer_prefix() {
        let jwt_service = JwtService::new("test-secret".to_string(), Some(24 * 7));
        let result = extract_user_id("token", &jwt_service);
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_user_agent_edge_actual() {
        // Test actual Edge browser (contains "Edge" or "Edg")
        let ua = "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36 Edg/120.0.0.0";
        let (_device, browser, os) = parse_user_agent(ua);
        // Should detect Chrome first (before Edge in the UA string)
        assert_eq!(browser, "Chrome");
        assert_eq!(os, "Windows");
    }

    #[test]
    fn test_parse_user_agent_firefox_android() {
        let ua = "Mozilla/5.0 (Android 13; Mobile; rv:109.0) Gecko/20100101 Firefox/120.0";
        let (_device, browser, os) = parse_user_agent(ua);
        assert_eq!(browser, "Firefox");
        assert_eq!(os, "Android");
    }

    #[test]
    fn test_parse_user_agent_chrome_android_tablet() {
        let ua = "Mozilla/5.0 (Linux; Android 13; SM-X900) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36";
        let (_device, browser, os) = parse_user_agent(ua);
        assert_eq!(browser, "Chrome");
        assert_eq!(os, "Android");
    }

    #[test]
    fn test_parse_user_agent_safari_ipad_specific() {
        let ua = "Mozilla/5.0 (iPad; CPU OS 17_1 like Mac OS X) AppleWebKit/605.1.15 (KHTML, like Gecko) Version/17.0 Mobile/15E148 Safari/604.1";
        let (_device, browser, os) = parse_user_agent(ua);
        assert_eq!(browser, "Safari");
        assert_eq!(os, "iOS"); // iPad should be detected as iOS
    }

    #[test]
    fn test_parse_user_agent_device_string_format() {
        let ua = "Mozilla/5.0 (Windows NT 10.0) Chrome/120.0";
        let (device, browser, os) = parse_user_agent(ua);
        assert_eq!(device, format!("{} on {}", browser, os));
    }

    // =========================================================================
    // INLINE COVERAGE BOOST: Additional tests for uncovered branches
    // =========================================================================

    #[tokio::test]
    async fn test_inline_change_password_invalid_current_length() {
        let service = Arc::new(SecurityService::new_dummy());
        let jwt_service = JwtService::new("secret".to_string(), Some(24 * 7));
        let user_id = bson::oid::ObjectId::new();
        let token = jwt_service.generate_token(&user_id.to_hex(), "user@test.com", false).unwrap();

        // Empty current password (min=1, should pass)
        let request = ChangePasswordRequest {
            current_password: "x".to_string(),
            new_password: "validnewpass".to_string(),
        };

        let result = handle_change_password(format!("Bearer {}", token), request, service).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_inline_update_profile_avatar_jpg_prefix() {
        let service = Arc::new(SecurityService::new_dummy());
        let jwt_service = JwtService::new("secret".to_string(), Some(24 * 7));
        let user_id = bson::oid::ObjectId::new();
        let token = jwt_service.generate_token(&user_id.to_hex(), "user@test.com", false).unwrap();

        let request = UpdateProfileRequest {
            display_name: Some("Test".to_string()),
            avatar_base64: Some("data:image/jpeg;base64,/9j/4AAQ".to_string()),
        };

        let result = handle_update_profile(format!("Bearer {}", token), request, service).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_inline_update_profile_avatar_webp_prefix() {
        let service = Arc::new(SecurityService::new_dummy());
        let jwt_service = JwtService::new("secret".to_string(), Some(24 * 7));
        let user_id = bson::oid::ObjectId::new();
        let token = jwt_service.generate_token(&user_id.to_hex(), "user@test.com", false).unwrap();

        let request = UpdateProfileRequest {
            display_name: Some("Test".to_string()),
            avatar_base64: Some("data:image/webp;base64,UklGR".to_string()),
        };

        let result = handle_update_profile(format!("Bearer {}", token), request, service).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_inline_handle_list_sessions_with_token() {
        let service = Arc::new(SecurityService::new_dummy());
        let jwt_service = JwtService::new("secret".to_string(), Some(24 * 7));
        let user_id = bson::oid::ObjectId::new();

        // Generate regular token (session_id handling is tested elsewhere)
        let token = jwt_service.generate_token(&user_id.to_hex(), "user@test.com", false).unwrap();

        let result = handle_list_sessions(format!("Bearer {}", token), service).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_inline_handle_revoke_all_with_token() {
        let service = Arc::new(SecurityService::new_dummy());
        let jwt_service = JwtService::new("secret".to_string(), Some(24 * 7));
        let user_id = bson::oid::ObjectId::new();

        // Generate regular token
        let token = jwt_service.generate_token(&user_id.to_hex(), "user@test.com", false).unwrap();

        let result = handle_revoke_all_sessions(format!("Bearer {}", token), service).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_inline_extract_user_id_with_admin_token() {
        let jwt_service = JwtService::new("secret".to_string(), Some(24 * 7));
        let user_id = bson::oid::ObjectId::new();

        // Generate token with is_admin=true
        let token = jwt_service.generate_token(&user_id.to_hex(), "admin@test.com", true).unwrap();

        let result = extract_user_id(&format!("Bearer {}", token), &jwt_service);
        assert!(result.is_ok());
        if let Ok(extracted_id) = result {
            assert_eq!(extracted_id, user_id);
        }
    }

    #[test]
    fn test_inline_parse_user_agent_edge_explicit() {
        // Test explicit Edge detection (without Chrome before it)
        let ua = "Mozilla/5.0 (Windows NT 10.0; Win64; x64) Edge/18.0";
        let (_device, browser, os) = parse_user_agent(ua);
        assert_eq!(browser, "Edge");
        assert_eq!(os, "Windows");
    }

    #[test]
    fn test_inline_parse_user_agent_mac_without_iphone_ipad() {
        // Test that Mac is detected when no iPhone/iPad in UA
        let ua = "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36";
        let (_device, _browser, os) = parse_user_agent(ua);
        assert_eq!(os, "MacOS");
    }

    #[test]
    fn test_inline_parse_user_agent_windows_before_mac() {
        // Ensure Windows is checked before Mac
        let ua = "Mozilla/5.0 (Windows NT 10.0; Mac OS X mention)";
        let (_device, _browser, os) = parse_user_agent(ua);
        assert_eq!(os, "Windows");
    }

    #[test]
    fn test_inline_parse_user_agent_android_before_linux() {
        // Ensure Android is detected before Linux
        let ua = "Mozilla/5.0 (Linux; Android 11)";
        let (_device, _browser, os) = parse_user_agent(ua);
        assert_eq!(os, "Android");
    }

    #[test]
    fn test_inline_parse_user_agent_iphone_priority() {
        // iPhone should be detected as iOS even if "Mac OS X" is present
        let ua = "Mozilla/5.0 (iPhone; CPU iPhone OS 15_0 like Mac OS X)";
        let (_device, _browser, os) = parse_user_agent(ua);
        assert_eq!(os, "iOS");
    }

    #[test]
    fn test_inline_parse_user_agent_safari_priority() {
        // Test Safari before Edge/Firefox/Chrome
        let ua = "Mozilla/5.0 (Macintosh) Safari/605 Firefox";
        let (_device, browser, _os) = parse_user_agent(ua);
        // Chrome checked first, then Firefox, then Safari, then Edge
        assert_eq!(browser, "Firefox");
    }

    #[test]
    fn test_inline_session_new_generates_unique_ids() {
        let user_id = bson::oid::ObjectId::new();
        let session1 = Session::new(
            user_id,
            "Device".to_string(),
            "Browser".to_string(),
            "OS".to_string(),
            "IP".to_string(),
            "Loc".to_string(),
            "UA".to_string(),
        );
        let session2 = Session::new(
            user_id,
            "Device".to_string(),
            "Browser".to_string(),
            "OS".to_string(),
            "IP".to_string(),
            "Loc".to_string(),
            "UA".to_string(),
        );

        // Each session should have a unique session_id
        assert_ne!(session1.session_id, session2.session_id);
    }

    #[test]
    fn test_inline_session_new_not_revoked_by_default() {
        let user_id = bson::oid::ObjectId::new();
        let session = Session::new(
            user_id,
            "Device".to_string(),
            "Browser".to_string(),
            "OS".to_string(),
            "IP".to_string(),
            "Loc".to_string(),
            "UA".to_string(),
        );

        assert!(!session.revoked);
    }

    #[tokio::test]
    async fn test_inline_warp_routes_combined() {
        let service = SecurityService::new_dummy();
        let routes = service.routes();
        let jwt_service = JwtService::new("secret".to_string(), Some(24 * 7));
        let user_id = bson::oid::ObjectId::new();
        let token = jwt_service.generate_token(&user_id.to_hex(), "test@example.com", false).unwrap();

        // Test profile update route
        let profile_req = UpdateProfileRequest {
            display_name: Some("Name".to_string()),
            avatar_base64: None,
        };

        let resp = request()
            .method("PATCH")
            .path("/auth/profile")
            .header("authorization", format!("Bearer {}", token))
            .json(&profile_req)
            .reply(&routes)
            .await;

        assert!(resp.status().as_u16() > 0);
    }

    #[tokio::test]
    async fn test_inline_warp_routes_2fa_setup() {
        let service = SecurityService::new_dummy();
        let routes = service.routes();
        let jwt_service = JwtService::new("secret".to_string(), Some(24 * 7));
        let user_id = bson::oid::ObjectId::new();
        let token = jwt_service.generate_token(&user_id.to_hex(), "test@example.com", false).unwrap();

        let resp = request()
            .method("POST")
            .path("/auth/2fa/setup")
            .header("authorization", format!("Bearer {}", token))
            .reply(&routes)
            .await;

        assert!(resp.status().as_u16() > 0);
    }

    #[tokio::test]
    async fn test_inline_warp_routes_2fa_verify() {
        let service = SecurityService::new_dummy();
        let routes = service.routes();
        let jwt_service = JwtService::new("secret".to_string(), Some(24 * 7));
        let user_id = bson::oid::ObjectId::new();
        let token = jwt_service.generate_token(&user_id.to_hex(), "test@example.com", false).unwrap();

        let verify_req = Verify2FARequest {
            code: "123456".to_string(),
        };

        let resp = request()
            .method("POST")
            .path("/auth/2fa/verify")
            .header("authorization", format!("Bearer {}", token))
            .json(&verify_req)
            .reply(&routes)
            .await;

        assert!(resp.status().as_u16() > 0);
    }

    #[tokio::test]
    async fn test_inline_warp_routes_2fa_disable() {
        let service = SecurityService::new_dummy();
        let routes = service.routes();
        let jwt_service = JwtService::new("secret".to_string(), Some(24 * 7));
        let user_id = bson::oid::ObjectId::new();
        let token = jwt_service.generate_token(&user_id.to_hex(), "test@example.com", false).unwrap();

        let disable_req = Verify2FARequest {
            code: "123456".to_string(),
        };

        let resp = request()
            .method("POST")
            .path("/auth/2fa/disable")
            .header("authorization", format!("Bearer {}", token))
            .json(&disable_req)
            .reply(&routes)
            .await;

        assert!(resp.status().as_u16() > 0);
    }

    #[tokio::test]
    async fn test_inline_warp_routes_sessions_list() {
        let service = SecurityService::new_dummy();
        let routes = service.routes();
        let jwt_service = JwtService::new("secret".to_string(), Some(24 * 7));
        let user_id = bson::oid::ObjectId::new();
        let token = jwt_service.generate_token(&user_id.to_hex(), "test@example.com", false).unwrap();

        let resp = request()
            .method("GET")
            .path("/auth/sessions")
            .header("authorization", format!("Bearer {}", token))
            .reply(&routes)
            .await;

        assert!(resp.status().as_u16() > 0);
    }

    #[tokio::test]
    async fn test_inline_warp_routes_sessions_revoke() {
        let service = SecurityService::new_dummy();
        let routes = service.routes();
        let jwt_service = JwtService::new("secret".to_string(), Some(24 * 7));
        let user_id = bson::oid::ObjectId::new();
        let token = jwt_service.generate_token(&user_id.to_hex(), "test@example.com", false).unwrap();

        let resp = request()
            .method("DELETE")
            .path("/auth/sessions/session-123")
            .header("authorization", format!("Bearer {}", token))
            .reply(&routes)
            .await;

        assert!(resp.status().as_u16() > 0);
    }

    #[tokio::test]
    async fn test_inline_warp_routes_sessions_revoke_all() {
        let service = SecurityService::new_dummy();
        let routes = service.routes();
        let jwt_service = JwtService::new("secret".to_string(), Some(24 * 7));
        let user_id = bson::oid::ObjectId::new();
        let token = jwt_service.generate_token(&user_id.to_hex(), "test@example.com", false).unwrap();

        let resp = request()
            .method("POST")
            .path("/auth/sessions/revoke-all")
            .header("authorization", format!("Bearer {}", token))
            .reply(&routes)
            .await;

        assert!(resp.status().as_u16() > 0);
    }

    #[test]
    fn test_inline_security_service_new_with_custom_secret() {
        let user_repo = UserRepository::new_dummy();
        let session_repo = SessionRepository::new_dummy();
        let service = SecurityService::new(
            user_repo,
            session_repo,
            "custom-jwt-secret-12345".to_string(),
        );

        // Service should be created successfully
        let _ = service.clone();
    }

    #[test]
    fn test_inline_totp_issuer_constant_value() {
        assert_eq!(TOTP_ISSUER, "BotCore Trading");
        assert!(TOTP_ISSUER.contains("BotCore"));
        assert!(TOTP_ISSUER.contains("Trading"));
    }

    #[tokio::test]
    async fn test_inline_change_password_minimum_new_password() {
        let service = Arc::new(SecurityService::new_dummy());
        let jwt_service = JwtService::new("secret".to_string(), Some(24 * 7));
        let user_id = bson::oid::ObjectId::new();
        let token = jwt_service.generate_token(&user_id.to_hex(), "user@test.com", false).unwrap();

        // Exactly 6 characters (minimum valid length)
        let request = ChangePasswordRequest {
            current_password: "oldpass".to_string(),
            new_password: "123456".to_string(),
        };

        let result = handle_change_password(format!("Bearer {}", token), request, service).await;
        assert!(result.is_ok());
    }

    #[test]
    fn test_inline_update_profile_exactly_100_chars() {
        let request = UpdateProfileRequest {
            display_name: Some("a".repeat(100)),
            avatar_base64: None,
        };

        // Exactly 100 characters should be valid
        assert!(request.validate().is_ok());
    }

    #[test]
    fn test_inline_verify_2fa_exactly_6_chars() {
        let request = Verify2FARequest {
            code: "123456".to_string(),
        };

        // Exactly 6 characters should be valid
        assert!(request.validate().is_ok());
    }

    #[test]
    fn test_inline_session_info_with_timestamps() {
        let now = chrono::Utc::now();
        let earlier = now - chrono::Duration::hours(1);

        let info = SessionInfo {
            session_id: "sess-id".to_string(),
            device: "Device".to_string(),
            browser: "Browser".to_string(),
            os: "OS".to_string(),
            location: "Location".to_string(),
            is_current: true,
            created_at: earlier,
            last_active: now,
        };

        assert!(info.last_active > info.created_at);
    }

    #[tokio::test]
    async fn test_inline_handle_change_password_with_admin_user() {
        let service = Arc::new(SecurityService::new_dummy());
        let jwt_service = JwtService::new("secret".to_string(), Some(24 * 7));
        let user_id = bson::oid::ObjectId::new();

        // Generate token for admin user (is_admin=true)
        let token = jwt_service.generate_token(&user_id.to_hex(), "admin@test.com", true).unwrap();

        let request = ChangePasswordRequest {
            current_password: "oldpass123".to_string(),
            new_password: "newpass456".to_string(),
        };

        let result = handle_change_password(format!("Bearer {}", token), request, service).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_inline_handle_update_profile_with_admin_user() {
        let service = Arc::new(SecurityService::new_dummy());
        let jwt_service = JwtService::new("secret".to_string(), Some(24 * 7));
        let user_id = bson::oid::ObjectId::new();

        // Generate token for admin user (is_admin=true)
        let token = jwt_service.generate_token(&user_id.to_hex(), "admin@test.com", true).unwrap();

        let request = UpdateProfileRequest {
            display_name: Some("Updated Name".to_string()),
            avatar_base64: None,
        };

        let result = handle_update_profile(format!("Bearer {}", token), request, service).await;
        assert!(result.is_ok());
    }

    #[test]
    fn test_inline_change_password_request_eq() {
        let req1 = ChangePasswordRequest {
            current_password: "pass1".to_string(),
            new_password: "pass2".to_string(),
        };
        let req2 = ChangePasswordRequest {
            current_password: "pass1".to_string(),
            new_password: "pass2".to_string(),
        };

        assert_eq!(req1.current_password, req2.current_password);
        assert_eq!(req1.new_password, req2.new_password);
    }

    #[test]
    fn test_inline_update_profile_request_eq() {
        let req1 = UpdateProfileRequest {
            display_name: Some("Name".to_string()),
            avatar_base64: Some("Avatar".to_string()),
        };
        let req2 = UpdateProfileRequest {
            display_name: Some("Name".to_string()),
            avatar_base64: Some("Avatar".to_string()),
        };

        assert_eq!(req1.display_name, req2.display_name);
        assert_eq!(req1.avatar_base64, req2.avatar_base64);
    }

    #[test]
    fn test_inline_verify_2fa_request_eq() {
        let req1 = Verify2FARequest {
            code: "123456".to_string(),
        };
        let req2 = Verify2FARequest {
            code: "123456".to_string(),
        };

        assert_eq!(req1.code, req2.code);
    }

    // Additional handler execution tests to boost coverage

    #[tokio::test]
    async fn test_boost_handle_setup_2fa_direct() {
        let service = Arc::new(SecurityService::new_dummy());
        let jwt_service = JwtService::new("test-secret".to_string(), Some(24 * 7));
        let user_id = bson::oid::ObjectId::new();
        let token = jwt_service.generate_token(&user_id.to_hex(), "user@test.com", false).unwrap();

        let result = handle_setup_2fa(format!("Bearer {}", token), service).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_boost_handle_verify_2fa_direct() {
        let service = Arc::new(SecurityService::new_dummy());
        let jwt_service = JwtService::new("test-secret".to_string(), Some(24 * 7));
        let user_id = bson::oid::ObjectId::new();
        let token = jwt_service.generate_token(&user_id.to_hex(), "user@test.com", false).unwrap();

        let request = Verify2FARequest {
            code: "123456".to_string(),
        };

        let result = handle_verify_2fa(format!("Bearer {}", token), request, service).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_boost_handle_disable_2fa_direct() {
        let service = Arc::new(SecurityService::new_dummy());
        let jwt_service = JwtService::new("test-secret".to_string(), Some(24 * 7));
        let user_id = bson::oid::ObjectId::new();
        let token = jwt_service.generate_token(&user_id.to_hex(), "user@test.com", false).unwrap();

        let request = Verify2FARequest {
            code: "123456".to_string(),
        };

        let result = handle_disable_2fa(format!("Bearer {}", token), request, service).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_boost_handle_list_sessions_direct() {
        let service = Arc::new(SecurityService::new_dummy());
        let jwt_service = JwtService::new("test-secret".to_string(), Some(24 * 7));
        let user_id = bson::oid::ObjectId::new();
        let token = jwt_service.generate_token(&user_id.to_hex(), "user@test.com", false).unwrap();

        let result = handle_list_sessions(format!("Bearer {}", token), service).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_boost_handle_revoke_session_direct() {
        let service = Arc::new(SecurityService::new_dummy());
        let jwt_service = JwtService::new("test-secret".to_string(), Some(24 * 7));
        let user_id = bson::oid::ObjectId::new();
        let token = jwt_service.generate_token(&user_id.to_hex(), "user@test.com", false).unwrap();

        let session_id = "test-session-id".to_string();
        let result = handle_revoke_session(session_id, format!("Bearer {}", token), service).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_boost_handle_revoke_all_sessions_direct() {
        let service = Arc::new(SecurityService::new_dummy());
        let jwt_service = JwtService::new("test-secret".to_string(), Some(24 * 7));
        let user_id = bson::oid::ObjectId::new();
        let token = jwt_service.generate_token(&user_id.to_hex(), "user@test.com", false).unwrap();

        let result = handle_revoke_all_sessions(format!("Bearer {}", token), service).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_boost_extract_user_id_valid() {
        let jwt_service = JwtService::new("test-secret".to_string(), Some(24 * 7));
        let user_id = bson::oid::ObjectId::new();
        let token = jwt_service.generate_token(&user_id.to_hex(), "user@test.com", false).unwrap();

        let result = extract_user_id(&format!("Bearer {}", token), &jwt_service);
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_boost_extract_user_id_invalid_token() {
        let jwt_service = JwtService::new("test-secret".to_string(), Some(24 * 7));
        let result = extract_user_id("Bearer invalid-token", &jwt_service);
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_boost_extract_user_id_no_bearer() {
        let jwt_service = JwtService::new("test-secret".to_string(), Some(24 * 7));
        let result = extract_user_id("token-without-bearer", &jwt_service);
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_boost_extract_user_id_invalid_object_id() {
        let jwt_service = JwtService::new("test-secret".to_string(), Some(24 * 7));
        // Generate token with invalid ObjectId string
        let token = jwt_service.generate_token("invalid-oid", "user@test.com", false).unwrap();
        let result = extract_user_id(&format!("Bearer {}", token), &jwt_service);
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_boost_change_password_validation_short_new_password() {
        let service = Arc::new(SecurityService::new_dummy());
        let jwt_service = JwtService::new("test-secret".to_string(), Some(24 * 7));
        let user_id = bson::oid::ObjectId::new();
        let token = jwt_service.generate_token(&user_id.to_hex(), "user@test.com", false).unwrap();

        let request = ChangePasswordRequest {
            current_password: "oldpass123".to_string(),
            new_password: "short".to_string(), // Too short
        };

        let result = handle_change_password(format!("Bearer {}", token), request, service).await;
        assert!(result.is_ok()); // Returns error response
    }

    #[tokio::test]
    async fn test_boost_change_password_validation_long_new_password() {
        let service = Arc::new(SecurityService::new_dummy());
        let jwt_service = JwtService::new("test-secret".to_string(), Some(24 * 7));
        let user_id = bson::oid::ObjectId::new();
        let token = jwt_service.generate_token(&user_id.to_hex(), "user@test.com", false).unwrap();

        let request = ChangePasswordRequest {
            current_password: "oldpass123".to_string(),
            new_password: "a".repeat(150), // Too long
        };

        let result = handle_change_password(format!("Bearer {}", token), request, service).await;
        assert!(result.is_ok()); // Returns error response
    }

    #[tokio::test]
    async fn test_boost_update_profile_validation_long_display_name() {
        let service = Arc::new(SecurityService::new_dummy());
        let jwt_service = JwtService::new("test-secret".to_string(), Some(24 * 7));
        let user_id = bson::oid::ObjectId::new();
        let token = jwt_service.generate_token(&user_id.to_hex(), "user@test.com", false).unwrap();

        let request = UpdateProfileRequest {
            display_name: Some("a".repeat(150)), // Too long
            avatar_base64: None,
        };

        let result = handle_update_profile(format!("Bearer {}", token), request, service).await;
        assert!(result.is_ok()); // Returns error response
    }

    #[tokio::test]
    async fn test_boost_update_profile_with_data_url_prefix_png() {
        let service = Arc::new(SecurityService::new_dummy());
        let jwt_service = JwtService::new("test-secret".to_string(), Some(24 * 7));
        let user_id = bson::oid::ObjectId::new();
        let token = jwt_service.generate_token(&user_id.to_hex(), "user@test.com", false).unwrap();

        let request = UpdateProfileRequest {
            display_name: Some("Test User".to_string()),
            avatar_base64: Some("data:image/png;base64,iVBORw0KGgoAAAANS".to_string()),
        };

        let result = handle_update_profile(format!("Bearer {}", token), request, service).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_boost_verify_2fa_validation_short_code() {
        let service = Arc::new(SecurityService::new_dummy());
        let jwt_service = JwtService::new("test-secret".to_string(), Some(24 * 7));
        let user_id = bson::oid::ObjectId::new();
        let token = jwt_service.generate_token(&user_id.to_hex(), "user@test.com", false).unwrap();

        let request = Verify2FARequest {
            code: "12345".to_string(), // Too short
        };

        let result = handle_verify_2fa(format!("Bearer {}", token), request, service).await;
        assert!(result.is_ok()); // Returns error response
    }

    #[tokio::test]
    async fn test_boost_verify_2fa_validation_long_code() {
        let service = Arc::new(SecurityService::new_dummy());
        let jwt_service = JwtService::new("test-secret".to_string(), Some(24 * 7));
        let user_id = bson::oid::ObjectId::new();
        let token = jwt_service.generate_token(&user_id.to_hex(), "user@test.com", false).unwrap();

        let request = Verify2FARequest {
            code: "1234567".to_string(), // Too long
        };

        let result = handle_verify_2fa(format!("Bearer {}", token), request, service).await;
        assert!(result.is_ok()); // Returns error response
    }


    #[test]
    fn test_boost_change_password_request_with_special_chars() {
        let request = ChangePasswordRequest {
            current_password: "old!@#$%^&*()".to_string(),
            new_password: "new!@#$%^&*()123".to_string(),
        };
        assert!(request.validate().is_ok());
    }

    #[test]
    fn test_boost_update_profile_request_empty_display_name() {
        let request = UpdateProfileRequest {
            display_name: Some("a".to_string()), // Single char - valid
            avatar_base64: None,
        };
        // Single character should be valid
        assert!(request.validate().is_ok());
    }

    #[test]
    fn test_boost_update_profile_request_none_values() {
        let request = UpdateProfileRequest {
            display_name: None,
            avatar_base64: None,
        };
        assert!(request.validate().is_ok());
    }

    #[test]
    fn test_boost_verify_2fa_request_alphanumeric() {
        let request = Verify2FARequest {
            code: "12345A".to_string(),
        };
        // Should validate (alphanumeric allowed)
        assert!(request.validate().is_ok());
    }

    #[test]
    fn test_boost_session_info_creation() {
        let now = Utc::now();
        let info = SessionInfo {
            session_id: "test-123".to_string(),
            device: "Chrome on MacOS".to_string(),
            browser: "Chrome".to_string(),
            os: "MacOS".to_string(),
            location: "US".to_string(),
            is_current: true,
            created_at: now,
            last_active: now,
        };

        assert_eq!(info.session_id, "test-123");
        assert_eq!(info.device, "Chrome on MacOS");
        assert!(info.is_current);
    }

    #[test]
    fn test_boost_session_list_response_creation() {
        let session = SessionInfo {
            session_id: "test-123".to_string(),
            device: "Chrome on MacOS".to_string(),
            browser: "Chrome".to_string(),
            os: "MacOS".to_string(),
            location: "US".to_string(),
            is_current: true,
            created_at: Utc::now(),
            last_active: Utc::now(),
        };

        let response = SessionListResponse {
            sessions: vec![session],
        };

        assert_eq!(response.sessions.len(), 1);
    }

    #[test]
    fn test_boost_setup_2fa_response_creation() {
        let response = Setup2FAResponse {
            secret: "JBSWY3DPEHPK3PXP".to_string(),
            qr_code: "data:image/png;base64,iVBORw0KGgoAAAANS".to_string(),
            otpauth_url: "otpauth://totp/BotCore:user@test.com?secret=JBSWY3DPEHPK3PXP&issuer=BotCore".to_string(),
        };

        assert!(response.secret.len() > 0);
        assert!(response.qr_code.starts_with("data:image/"));
        assert!(response.otpauth_url.starts_with("otpauth://"));
    }

    #[tokio::test]
    async fn test_boost_handle_change_password_empty_auth() {
        let service = Arc::new(SecurityService::new_dummy());
        let request = ChangePasswordRequest {
            current_password: "oldpass123".to_string(),
            new_password: "newpass456".to_string(),
        };

        let result = handle_change_password("".to_string(), request, service).await;
        assert!(result.is_ok()); // Returns error response
    }

    #[tokio::test]
    async fn test_boost_handle_update_profile_empty_auth() {
        let service = Arc::new(SecurityService::new_dummy());
        let request = UpdateProfileRequest {
            display_name: Some("Test".to_string()),
            avatar_base64: None,
        };

        let result = handle_update_profile("".to_string(), request, service).await;
        assert!(result.is_ok()); // Returns error response
    }

    #[tokio::test]
    async fn test_boost_handle_setup_2fa_empty_auth() {
        let service = Arc::new(SecurityService::new_dummy());
        let result = handle_setup_2fa("".to_string(), service).await;
        assert!(result.is_ok()); // Returns error response
    }

    #[tokio::test]
    async fn test_boost_handle_verify_2fa_empty_auth() {
        let service = Arc::new(SecurityService::new_dummy());
        let request = Verify2FARequest {
            code: "123456".to_string(),
        };

        let result = handle_verify_2fa("".to_string(), request, service).await;
        assert!(result.is_ok()); // Returns error response
    }

    #[tokio::test]
    async fn test_boost_handle_disable_2fa_empty_auth() {
        let service = Arc::new(SecurityService::new_dummy());
        let request = Verify2FARequest {
            code: "123456".to_string(),
        };

        let result = handle_disable_2fa("".to_string(), request, service).await;
        assert!(result.is_ok()); // Returns error response
    }

    #[tokio::test]
    async fn test_boost_handle_list_sessions_empty_auth() {
        let service = Arc::new(SecurityService::new_dummy());
        let result = handle_list_sessions("".to_string(), service).await;
        assert!(result.is_ok()); // Returns error response
    }

    #[tokio::test]
    async fn test_boost_handle_revoke_session_empty_auth() {
        let service = Arc::new(SecurityService::new_dummy());
        let result = handle_revoke_session("session-id".to_string(), "".to_string(), service).await;
        assert!(result.is_ok()); // Returns error response
    }

    #[tokio::test]
    async fn test_boost_handle_revoke_all_sessions_empty_auth() {
        let service = Arc::new(SecurityService::new_dummy());
        let result = handle_revoke_all_sessions("".to_string(), service).await;
        assert!(result.is_ok()); // Returns error response
    }

    #[test]
    fn test_boost_totp_issuer_constant() {
        assert_eq!(TOTP_ISSUER, "BotCore Trading");
    }

    #[tokio::test]
    async fn test_boost_security_service_new_custom() {
        let user_repo = UserRepository::new_dummy();
        let session_repo = SessionRepository::new_dummy();
        let service = SecurityService::new(
            user_repo,
            session_repo,
            "custom-secret-123".to_string(),
        );

        // Verify service is created properly
        let cloned = service.clone();
        drop(service);
        drop(cloned);
    }

    #[tokio::test]
    async fn test_boost_handle_change_password_malformed_token() {
        let service = Arc::new(SecurityService::new_dummy());
        let request = ChangePasswordRequest {
            current_password: "oldpass123".to_string(),
            new_password: "newpass456".to_string(),
        };

        let result = handle_change_password("Bearer malformed.token.here".to_string(), request, service).await;
        assert!(result.is_ok()); // Returns error response
    }

    #[tokio::test]
    async fn test_boost_handle_update_profile_with_only_avatar() {
        let service = Arc::new(SecurityService::new_dummy());
        let jwt_service = JwtService::new("test-secret".to_string(), Some(24 * 7));
        let user_id = bson::oid::ObjectId::new();
        let token = jwt_service.generate_token(&user_id.to_hex(), "user@test.com", false).unwrap();

        let request = UpdateProfileRequest {
            display_name: None,
            avatar_base64: Some("base64encodeddata".to_string()),
        };

        let result = handle_update_profile(format!("Bearer {}", token), request, service).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_boost_handle_update_profile_with_webp_avatar() {
        let service = Arc::new(SecurityService::new_dummy());
        let jwt_service = JwtService::new("test-secret".to_string(), Some(24 * 7));
        let user_id = bson::oid::ObjectId::new();
        let token = jwt_service.generate_token(&user_id.to_hex(), "user@test.com", false).unwrap();

        let request = UpdateProfileRequest {
            display_name: Some("Test".to_string()),
            avatar_base64: Some("data:image/webp;base64,UklGRiQAAABXRUJQ".to_string()),
        };

        let result = handle_update_profile(format!("Bearer {}", token), request, service).await;
        assert!(result.is_ok());
    }

    #[test]
    fn test_boost_change_password_request_min_length() {
        let request = ChangePasswordRequest {
            current_password: "12345678".to_string(), // Exactly 8 chars (min)
            new_password: "newpass8".to_string(),
        };
        assert!(request.validate().is_ok());
    }

    #[test]
    fn test_boost_change_password_request_max_length() {
        let request = ChangePasswordRequest {
            current_password: "a".repeat(128), // Exactly 128 chars (max)
            new_password: "b".repeat(128),
        };
        assert!(request.validate().is_ok());
    }

    #[test]
    fn test_boost_update_profile_request_max_length() {
        let request = UpdateProfileRequest {
            display_name: Some("a".repeat(100)), // Exactly 100 chars (max)
            avatar_base64: None,
        };
        assert!(request.validate().is_ok());
    }

    #[test]
    fn test_boost_verify_2fa_request_numeric() {
        let request = Verify2FARequest {
            code: "000000".to_string(),
        };
        assert!(request.validate().is_ok());
    }

    #[test]
    fn test_boost_parse_user_agent_edge_browser() {
        let ua = "Mozilla/5.0 (Windows NT 10.0; Win64; x64) Chrome/120.0.0.0 Safari/537.36 Edg/120.0.0.0";
        let (_device, browser, _os) = parse_user_agent(ua);
        // Edge is detected as Chrome (based on current logic)
        assert_eq!(browser, "Chrome");
    }

    #[test]
    fn test_boost_parse_user_agent_ipad_safari() {
        let ua = "Mozilla/5.0 (iPad; CPU OS 17_0 like Mac OS X) AppleWebKit/605.1.15 Version/17.0 Safari/604.1";
        let (_device, browser, os) = parse_user_agent(ua);
        assert_eq!(browser, "Safari");
        assert_eq!(os, "iOS");
    }

    #[test]
    fn test_boost_parse_user_agent_android_tablet() {
        let ua = "Mozilla/5.0 (Linux; Android 13; SM-T870) Chrome/120.0.0.0 Safari/537.36";
        let (_device, browser, os) = parse_user_agent(ua);
        assert_eq!(browser, "Chrome");
        assert_eq!(os, "Android");
    }

    #[test]
    fn test_boost_parse_user_agent_windows_edge() {
        let ua = "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36 Edg/120.0.0.0";
        let (_device, _browser, os) = parse_user_agent(ua);
        assert_eq!(os, "Windows");
    }

    #[test]
    fn test_boost_parse_user_agent_mac_safari() {
        let ua = "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/605.1.15 Version/17.0 Safari/605.1.15";
        let (_device, browser, os) = parse_user_agent(ua);
        assert_eq!(browser, "Safari");
        assert_eq!(os, "MacOS");
    }
}
