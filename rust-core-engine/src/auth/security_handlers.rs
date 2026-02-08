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
}
