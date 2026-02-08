// Comprehensive tests for auth module to increase coverage
// Target: auth/database.rs, auth/handlers.rs, auth/security_handlers.rs
// Focus: Data validation, error handling, edge cases, serialization

mod common;

use binance_trading_bot::auth::{
    jwt::{Claims, JwtService, PasswordService},
    models::{
        ChangePasswordRequest, LoginRequest, LoginResponse, NotificationSettings, RegisterRequest,
        RiskLevel, Session, UpdateProfileRequest, User, UserProfile, UserSettings,
        Verify2FARequest,
    },
};
use bson::oid::ObjectId;
use chrono::Utc;
use validator::Validate;

// ========== AUTH MODELS TESTS ==========

#[test]
fn test_register_request_validation_valid() {
    let request = RegisterRequest {
        email: "test@example.com".to_string(),
        password: "SecureP@ssw0rd".to_string(),
        full_name: Some("Test User".to_string()),
    };

    let result = request.validate();
    assert!(result.is_ok(), "Valid registration should pass validation");
}

#[test]
fn test_register_request_validation_invalid_email() {
    let request = RegisterRequest {
        email: "invalid-email".to_string(),
        password: "SecureP@ssw0rd".to_string(),
        full_name: Some("Test User".to_string()),
    };

    let result = request.validate();
    assert!(result.is_err(), "Invalid email should fail validation");
}

#[test]
fn test_register_request_validation_short_password() {
    let request = RegisterRequest {
        email: "test@example.com".to_string(),
        password: "short".to_string(), // Less than 6 characters
        full_name: Some("Test User".to_string()),
    };

    let result = request.validate();
    assert!(result.is_err(), "Short password should fail validation");
}

#[test]
fn test_login_request_validation_valid() {
    let request = LoginRequest {
        email: "user@example.com".to_string(),
        password: "ValidPassword123".to_string(),
    };

    let result = request.validate();
    assert!(result.is_ok(), "Valid login should pass validation");
}

#[test]
fn test_login_request_validation_invalid_email() {
    let request = LoginRequest {
        email: "not-an-email".to_string(),
        password: "password123".to_string(),
    };

    let result = request.validate();
    assert!(result.is_err());
}

#[test]
fn test_user_model_creation() {
    let user = User {
        id: Some(ObjectId::new()),
        email: "user@example.com".to_string(),
        password_hash: "$2b$12$hashedpassword".to_string(),
        full_name: Some("John Doe".to_string()),
        display_name: None,
        avatar_url: None,
        is_admin: false,
        is_active: true,
        two_factor_secret: None,
        two_factor_enabled: false,
        created_at: Utc::now(),
        updated_at: Utc::now(),
        last_login: None,
        settings: UserSettings {
            trading_enabled: true,
            risk_level: RiskLevel::Medium,
            max_positions: 5,
            default_quantity: 100.0,
            notifications: NotificationSettings {
                email_alerts: true,
                trade_notifications: true,
                system_alerts: true,
            },
        },
    };

    assert_eq!(user.email, "user@example.com");
    assert!(!user.is_admin);
    assert!(user.is_active);
    assert!(!user.two_factor_enabled);
}

#[test]
fn test_login_response_structure() {
    let response = LoginResponse {
        token: "jwt_token_here".to_string(),
        user: UserProfile {
            id: ObjectId::new().to_string(),
            email: "user@example.com".to_string(),
            full_name: Some("User".to_string()),
            display_name: None,
            avatar_url: None,
            is_admin: false,
            is_active: true,
            two_factor_enabled: false,
            created_at: Utc::now(),
            last_login: Some(Utc::now()),
            settings: UserSettings {
                trading_enabled: true,
                risk_level: RiskLevel::Low,
                max_positions: 3,
                default_quantity: 50.0,
                notifications: NotificationSettings {
                    email_alerts: false,
                    trade_notifications: true,
                    system_alerts: true,
                },
            },
        },
    };

    assert_eq!(response.token, "jwt_token_here");
    assert_eq!(response.user.email, "user@example.com");
}

#[test]
fn test_change_password_request_validation() {
    let request = ChangePasswordRequest {
        current_password: "OldPassword123".to_string(),
        new_password: "NewSecurePassword456".to_string(),
    };

    let result = request.validate();
    assert!(result.is_ok(), "Valid password change should pass");
}

#[test]
fn test_change_password_request_invalid_new_password() {
    let request = ChangePasswordRequest {
        current_password: "OldPassword123".to_string(),
        new_password: "short".to_string(), // Too short
    };

    let result = request.validate();
    assert!(result.is_err(), "Short new password should fail validation");
}

#[test]
fn test_update_profile_request_validation() {
    let request = UpdateProfileRequest {
        display_name: Some("New Display Name".to_string()),
        avatar_base64: None,
    };

    let result = request.validate();
    assert!(result.is_ok());
}

#[test]
fn test_verify_2fa_request_validation() {
    let request = Verify2FARequest {
        code: "123456".to_string(),
    };

    let result = request.validate();
    assert!(result.is_ok());
}

#[test]
fn test_verify_2fa_request_invalid_code_length() {
    let request = Verify2FARequest {
        code: "12345".to_string(), // Only 5 digits
    };

    let result = request.validate();
    assert!(result.is_err(), "5-digit code should fail validation");
}

#[test]
fn test_session_model_creation() {
    let now = Utc::now();
    let session = Session {
        id: Some(ObjectId::new()),
        session_id: "session_123".to_string(),
        user_id: ObjectId::new(),
        device: "Desktop".to_string(),
        browser: "Chrome".to_string(),
        os: "MacOS".to_string(),
        ip_address: "192.168.1.1".to_string(),
        location: "US".to_string(),
        user_agent: "Mozilla/5.0".to_string(),
        created_at: now,
        last_active: now,
        expires_at: now + chrono::Duration::days(7),
        revoked: false,
    };

    assert!(session.id.is_some());
    assert!(!session.session_id.is_empty());
    assert!(!session.ip_address.is_empty());
}

// ========== JWT SERVICE TESTS ==========

#[test]
fn test_jwt_service_token_lifecycle() {
    let jwt_service = JwtService::new("test_secret_key".to_string(), Some(24));

    // Generate token
    let token = jwt_service
        .generate_token("user_123", "user@test.com", false)
        .unwrap();

    // Verify token
    let claims = jwt_service.verify_token(&token).unwrap();
    assert_eq!(claims.sub, "user_123");
    assert_eq!(claims.email, "user@test.com");
    assert!(!claims.is_admin);
}

#[test]
fn test_jwt_service_admin_token() {
    let jwt_service = JwtService::new("admin_secret".to_string(), Some(48));

    let token = jwt_service
        .generate_token("admin_456", "admin@test.com", true)
        .unwrap();

    let claims = jwt_service.verify_token(&token).unwrap();
    assert!(claims.is_admin);
}

#[test]
fn test_jwt_extract_token_from_header_valid() {
    let header = "Bearer eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9.test.signature";
    let token = JwtService::extract_token_from_header(header);
    assert_eq!(
        token,
        Some("eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9.test.signature")
    );
}

#[test]
fn test_jwt_extract_token_no_bearer() {
    let header = "Basic dXNlcjpwYXNzd29yZA==";
    let token = JwtService::extract_token_from_header(header);
    assert_eq!(token, None);
}

#[test]
fn test_jwt_extract_token_empty_header() {
    let token = JwtService::extract_token_from_header("");
    assert_eq!(token, None);
}

#[test]
fn test_jwt_extract_token_only_bearer() {
    let header = "Bearer ";
    let token = JwtService::extract_token_from_header(header);
    assert_eq!(token, Some(""));
}

// ========== PASSWORD SERVICE TESTS ==========

#[test]
fn test_password_service_hash_verify_success() {
    let password = "MySecurePassword123!";
    let hash = PasswordService::hash_password(password).unwrap();

    // Verify correct password
    assert!(PasswordService::verify_password(password, &hash).unwrap());
}

#[test]
fn test_password_service_hash_verify_failure() {
    let password = "CorrectPassword";
    let wrong_password = "WrongPassword";
    let hash = PasswordService::hash_password(password).unwrap();

    // Verify wrong password fails
    assert!(!PasswordService::verify_password(wrong_password, &hash).unwrap());
}

#[test]
fn test_password_service_hash_uniqueness() {
    let password = "SamePassword123";

    let hash1 = PasswordService::hash_password(password).unwrap();
    let hash2 = PasswordService::hash_password(password).unwrap();

    // Hashes should be different due to different salts
    assert_ne!(hash1, hash2);

    // But both should verify correctly
    assert!(PasswordService::verify_password(password, &hash1).unwrap());
    assert!(PasswordService::verify_password(password, &hash2).unwrap());
}

#[test]
fn test_password_service_empty_password() {
    let password = "";
    let result = PasswordService::hash_password(password);
    // Empty password should still hash (validation happens at request level)
    assert!(result.is_ok());
}

#[test]
fn test_password_service_long_password() {
    let password = "a".repeat(1000); // Very long password
    let result = PasswordService::hash_password(&password);
    assert!(result.is_ok());
}

#[test]
fn test_password_service_special_characters() {
    let password = "P@$$w0rd!#%&*()_+-=[]{}|;:,.<>?/~`";
    let hash = PasswordService::hash_password(password).unwrap();
    assert!(PasswordService::verify_password(password, &hash).unwrap());
}

#[test]
fn test_password_service_unicode_password() {
    let password = "パスワード123"; // Japanese characters
    let hash = PasswordService::hash_password(password).unwrap();
    assert!(PasswordService::verify_password(password, &hash).unwrap());
}

#[test]
fn test_password_verify_invalid_hash_format() {
    let password = "TestPassword";
    let invalid_hash = "not_a_valid_bcrypt_hash";

    let result = PasswordService::verify_password(password, invalid_hash);
    assert!(result.is_err());
}

// ========== CLAIMS TESTS ==========

#[test]
fn test_claims_serialization() {
    let claims = Claims {
        sub: "user_789".to_string(),
        email: "user@example.com".to_string(),
        is_admin: true,
        exp: Utc::now().timestamp() + 3600,
        iat: Utc::now().timestamp(),
        session_id: Some("session_abc".to_string()),
    };

    let json = serde_json::to_string(&claims).unwrap();
    assert!(json.contains("user_789"));
    assert!(json.contains("user@example.com"));

    let deserialized: Claims = serde_json::from_str(&json).unwrap();
    assert_eq!(deserialized.sub, claims.sub);
    assert_eq!(deserialized.email, claims.email);
    assert_eq!(deserialized.is_admin, claims.is_admin);
}

// ========== EDGE CASES & ERROR HANDLING ==========

#[test]
fn test_register_request_empty_email() {
    let request = RegisterRequest {
        email: "".to_string(),
        password: "Password123".to_string(),
        full_name: None,
    };

    let result = request.validate();
    assert!(result.is_err());
}

#[test]
fn test_register_request_no_full_name() {
    let request = RegisterRequest {
        email: "user@example.com".to_string(),
        password: "Password123".to_string(),
        full_name: None,
    };

    let result = request.validate();
    assert!(result.is_ok(), "Full name should be optional");
}

#[test]
fn test_login_request_empty_password() {
    let request = LoginRequest {
        email: "user@example.com".to_string(),
        password: "".to_string(),
    };

    let result = request.validate();
    assert!(result.is_err(), "Empty password should fail");
}

#[test]
fn test_user_with_2fa_enabled() {
    let user = User {
        id: Some(ObjectId::new()),
        email: "secure@example.com".to_string(),
        password_hash: "hash".to_string(),
        full_name: Some("Secure User".to_string()),
        display_name: None,
        avatar_url: None,
        is_admin: false,
        is_active: true,
        two_factor_secret: Some("SECRET123".to_string()),
        two_factor_enabled: true,
        created_at: Utc::now(),
        updated_at: Utc::now(),
        last_login: None,
        settings: UserSettings {
            trading_enabled: true,
            risk_level: RiskLevel::High,
            max_positions: 10,
            default_quantity: 500.0,
            notifications: NotificationSettings {
                email_alerts: true,
                trade_notifications: true,
                system_alerts: true,
            },
        },
    };

    assert!(user.two_factor_enabled);
    assert!(user.two_factor_secret.is_some());
}

#[test]
fn test_session_expiration_check() {
    let now = Utc::now();
    let expired_session = Session {
        id: Some(ObjectId::new()),
        session_id: "expired_123".to_string(),
        user_id: ObjectId::new(),
        device: "Mobile".to_string(),
        browser: "Safari".to_string(),
        os: "iOS".to_string(),
        ip_address: "10.0.0.1".to_string(),
        location: "US".to_string(),
        user_agent: "Mozilla/5.0".to_string(),
        created_at: now - chrono::Duration::days(10),
        last_active: now - chrono::Duration::days(5),
        expires_at: now - chrono::Duration::days(3), // Expired 3 days ago
        revoked: false,
    };

    assert!(
        expired_session.expires_at < now,
        "Session should be expired"
    );
}

#[test]
fn test_session_valid_check() {
    let now = Utc::now();
    let valid_session = Session {
        id: Some(ObjectId::new()),
        session_id: "valid_456".to_string(),
        user_id: ObjectId::new(),
        device: "Desktop".to_string(),
        browser: "Firefox".to_string(),
        os: "Windows".to_string(),
        ip_address: "127.0.0.1".to_string(),
        location: "UK".to_string(),
        user_agent: "Test Agent".to_string(),
        created_at: now - chrono::Duration::hours(1),
        last_active: now,
        expires_at: now + chrono::Duration::days(6), // Expires in 6 days
        revoked: false,
    };

    assert!(valid_session.expires_at > now, "Session should be valid");
}

#[test]
fn test_jwt_service_different_secrets() {
    let service1 = JwtService::new("secret1".to_string(), Some(24));
    let service2 = JwtService::new("secret2".to_string(), Some(24));

    let token = service1
        .generate_token("user", "user@test.com", false)
        .unwrap();

    // Token from service1 should not verify with service2
    let result = service2.verify_token(&token);
    assert!(
        result.is_err(),
        "Token should not verify with different secret"
    );
}

#[test]
fn test_jwt_service_zero_expiration() {
    let service = JwtService::new("secret".to_string(), Some(0));

    let token = service
        .generate_token("user", "user@test.com", false)
        .unwrap();

    // Token should still be created and verifiable immediately
    let result = service.verify_token(&token);
    assert!(result.is_ok());
}

#[test]
fn test_change_password_same_passwords() {
    let request = ChangePasswordRequest {
        current_password: "SamePassword123".to_string(),
        new_password: "SamePassword123".to_string(),
    };

    // Validation allows same passwords (business logic should reject)
    let result = request.validate();
    assert!(result.is_ok());
}

#[test]
fn test_verify_2fa_non_numeric_code() {
    let request = Verify2FARequest {
        code: "abcdef".to_string(),
    };

    let result = request.validate();
    // Validator checks length, not numeric (TOTP library handles that)
    assert!(result.is_ok());
}

#[test]
fn test_user_settings_risk_levels() {
    let low_risk = UserSettings {
        trading_enabled: true,
        risk_level: RiskLevel::Low,
        max_positions: 2,
        default_quantity: 10.0,
        notifications: NotificationSettings {
            email_alerts: true,
            trade_notifications: true,
            system_alerts: true,
        },
    };

    let high_risk = UserSettings {
        trading_enabled: true,
        risk_level: RiskLevel::High,
        max_positions: 20,
        default_quantity: 1000.0,
        notifications: NotificationSettings {
            email_alerts: false,
            trade_notifications: false,
            system_alerts: true,
        },
    };

    assert!(matches!(low_risk.risk_level, RiskLevel::Low));
    assert!(matches!(high_risk.risk_level, RiskLevel::High));
}

#[test]
fn test_notification_settings() {
    let settings = NotificationSettings {
        email_alerts: true,
        trade_notifications: false,
        system_alerts: true,
    };

    assert!(settings.email_alerts);
    assert!(!settings.trade_notifications);
    assert!(settings.system_alerts);
}

#[test]
fn test_session_revoked() {
    let now = Utc::now();
    let revoked_session = Session {
        id: Some(ObjectId::new()),
        session_id: "revoked_789".to_string(),
        user_id: ObjectId::new(),
        device: "Tablet".to_string(),
        browser: "Edge".to_string(),
        os: "Android".to_string(),
        ip_address: "172.16.0.1".to_string(),
        location: "CA".to_string(),
        user_agent: "Mobile Browser".to_string(),
        created_at: now - chrono::Duration::days(1),
        last_active: now - chrono::Duration::hours(12),
        expires_at: now + chrono::Duration::days(6),
        revoked: true,
    };

    assert!(revoked_session.revoked);
}
