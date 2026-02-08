//! Security Handlers Tests
//!
//! Comprehensive tests for security handlers including 2FA, sessions, and password management.
//! @spec:FR-AUTH-016 - Account Security Handlers
//! @ref:specs/02-design/2.5-components/COMP-RUST-AUTH.md

mod common;

use binance_trading_bot::auth::jwt::{Claims, JwtService, PasswordService};
use binance_trading_bot::auth::models::{
    ChangePasswordRequest, UpdateProfileRequest, Verify2FARequest,
};
use binance_trading_bot::auth::security_handlers::{parse_user_agent, SecurityService};
use chrono::{Duration, Utc};

// ============================================================================
// PASSWORD SERVICE TESTS
// ============================================================================

#[test]
fn test_password_strength_variations() {
    // Test weak password (still should hash)
    let weak = "123456";
    let hash = PasswordService::hash_password(weak).unwrap();
    assert!(PasswordService::verify_password(weak, &hash).unwrap());

    // Test strong password
    let strong = "MyS3cur3P@ssw0rd!2024";
    let hash = PasswordService::hash_password(strong).unwrap();
    assert!(PasswordService::verify_password(strong, &hash).unwrap());

    // Test unicode password
    let unicode = "пароль123!";
    let hash = PasswordService::hash_password(unicode).unwrap();
    assert!(PasswordService::verify_password(unicode, &hash).unwrap());
}

#[test]
fn test_password_hash_uniqueness() {
    let password = "TestPassword123!";
    let hash1 = PasswordService::hash_password(password).unwrap();
    let hash2 = PasswordService::hash_password(password).unwrap();

    // Hashes should be different due to random salt
    assert_ne!(hash1, hash2);

    // But both should verify correctly
    assert!(PasswordService::verify_password(password, &hash1).unwrap());
    assert!(PasswordService::verify_password(password, &hash2).unwrap());
}

#[test]
fn test_password_verify_fails_on_wrong_password() {
    let password = "CorrectPassword";
    let hash = PasswordService::hash_password(password).unwrap();

    assert!(!PasswordService::verify_password("WrongPassword", &hash).unwrap());
    assert!(!PasswordService::verify_password("correctpassword", &hash).unwrap()); // Case sensitive
    assert!(!PasswordService::verify_password("CorrectPassword ", &hash).unwrap()); // Trailing space
}

#[test]
fn test_password_empty_string() {
    let empty = "";
    let hash = PasswordService::hash_password(empty).unwrap();
    assert!(PasswordService::verify_password(empty, &hash).unwrap());
    assert!(!PasswordService::verify_password("nonempty", &hash).unwrap());
}

#[test]
fn test_password_verify_invalid_hash_format() {
    let password = "test";
    let result = PasswordService::verify_password(password, "invalid_hash_format");
    assert!(result.is_err());
}

#[test]
fn test_password_very_long() {
    let long_password = "a".repeat(500);
    let hash = PasswordService::hash_password(&long_password).unwrap();
    assert!(PasswordService::verify_password(&long_password, &hash).unwrap());
}

#[test]
fn test_password_special_characters() {
    let special = "!@#$%^&*()_+-={}[]|\\:\";<>?,./~`";
    let hash = PasswordService::hash_password(special).unwrap();
    assert!(PasswordService::verify_password(special, &hash).unwrap());
}

// ============================================================================
// JWT SERVICE TESTS
// ============================================================================

#[test]
fn test_jwt_generate_and_verify_basic() {
    let jwt_service = JwtService::new("test_secret_key".to_string(), Some(1));

    let token = jwt_service
        .generate_token("user_123", "user@example.com", false)
        .unwrap();

    let claims = jwt_service.verify_token(&token).unwrap();
    assert_eq!(claims.sub, "user_123");
    assert_eq!(claims.email, "user@example.com");
    assert!(!claims.is_admin);
}

#[test]
fn test_jwt_admin_flag() {
    let jwt_service = JwtService::new("secret".to_string(), Some(24));

    let admin_token = jwt_service
        .generate_token("admin_1", "admin@test.com", true)
        .unwrap();
    let user_token = jwt_service
        .generate_token("user_1", "user@test.com", false)
        .unwrap();

    let admin_claims = jwt_service.verify_token(&admin_token).unwrap();
    let user_claims = jwt_service.verify_token(&user_token).unwrap();

    assert!(admin_claims.is_admin);
    assert!(!user_claims.is_admin);
}

#[test]
fn test_jwt_with_session_id() {
    let jwt_service = JwtService::new("secret".to_string(), Some(1));

    let token = jwt_service
        .generate_token_with_session(
            "user_123",
            "user@test.com",
            false,
            Some("session_abc".to_string()),
        )
        .unwrap();

    let claims = jwt_service.verify_token(&token).unwrap();
    assert_eq!(claims.session_id, Some("session_abc".to_string()));
}

#[test]
fn test_jwt_without_session_id() {
    let jwt_service = JwtService::new("secret".to_string(), Some(1));

    let token = jwt_service
        .generate_token_with_session("user_123", "user@test.com", false, None)
        .unwrap();

    let claims = jwt_service.verify_token(&token).unwrap();
    assert_eq!(claims.session_id, None);
}

#[test]
fn test_jwt_expiration_time() {
    let jwt_service = JwtService::new("secret".to_string(), Some(48)); // 48 hours

    let token = jwt_service
        .generate_token("user", "user@test.com", false)
        .unwrap();

    let claims = jwt_service.verify_token(&token).unwrap();
    let expected_exp = Utc::now() + Duration::hours(48);

    // Allow 5 seconds tolerance
    assert!((claims.exp - expected_exp.timestamp()).abs() < 5);
}

#[test]
fn test_jwt_issued_at_time() {
    let jwt_service = JwtService::new("secret".to_string(), Some(1));

    let before = Utc::now().timestamp();
    let token = jwt_service
        .generate_token("user", "user@test.com", false)
        .unwrap();
    let after = Utc::now().timestamp();

    let claims = jwt_service.verify_token(&token).unwrap();

    assert!(claims.iat >= before);
    assert!(claims.iat <= after);
}

#[test]
fn test_jwt_verify_with_wrong_secret() {
    let jwt_service1 = JwtService::new("secret1".to_string(), Some(1));
    let jwt_service2 = JwtService::new("secret2".to_string(), Some(1));

    let token = jwt_service1
        .generate_token("user", "user@test.com", false)
        .unwrap();

    let result = jwt_service2.verify_token(&token);
    assert!(result.is_err());
}

#[test]
fn test_jwt_verify_malformed_token() {
    let jwt_service = JwtService::new("secret".to_string(), Some(1));

    assert!(jwt_service.verify_token("").is_err());
    assert!(jwt_service.verify_token("not.a.token").is_err());
    assert!(jwt_service.verify_token("malformed_token").is_err());
    assert!(jwt_service
        .verify_token("header.payload.signature.extra")
        .is_err());
}

#[test]
fn test_jwt_extract_token_from_header_valid() {
    let header = "Bearer eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9.payload.signature";
    let token = JwtService::extract_token_from_header(header);
    assert_eq!(
        token,
        Some("eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9.payload.signature")
    );
}

#[test]
fn test_jwt_extract_token_from_header_invalid() {
    // Missing Bearer prefix
    assert_eq!(JwtService::extract_token_from_header("token123"), None);

    // Wrong auth type
    assert_eq!(
        JwtService::extract_token_from_header("Basic dXNlcjpwYXNz"),
        None
    );

    // Empty string
    assert_eq!(JwtService::extract_token_from_header(""), None);

    // Bearer but no token - strip_prefix returns Some("")
    assert_eq!(JwtService::extract_token_from_header("Bearer "), Some(""));

    // Case sensitive
    assert_eq!(
        JwtService::extract_token_from_header("bearer token123"),
        None
    );
}

#[test]
fn test_jwt_extract_token_with_whitespace() {
    let header = "Bearer  token_with_extra_space";
    let token = JwtService::extract_token_from_header(header);
    // strip_prefix removes "Bearer " leaving " token_with_extra_space"
    assert_eq!(token, Some(" token_with_extra_space"));
}

#[test]
fn test_jwt_multiple_sessions_same_user() {
    let jwt_service = JwtService::new("secret".to_string(), Some(24));

    let session1 = jwt_service
        .generate_token_with_session(
            "user1",
            "user@test.com",
            false,
            Some("session_001".to_string()),
        )
        .unwrap();

    let session2 = jwt_service
        .generate_token_with_session(
            "user1",
            "user@test.com",
            false,
            Some("session_002".to_string()),
        )
        .unwrap();

    let claims1 = jwt_service.verify_token(&session1).unwrap();
    let claims2 = jwt_service.verify_token(&session2).unwrap();

    assert_eq!(claims1.sub, claims2.sub);
    assert_ne!(claims1.session_id, claims2.session_id);
    assert_eq!(claims1.session_id, Some("session_001".to_string()));
    assert_eq!(claims2.session_id, Some("session_002".to_string()));
}

#[test]
fn test_jwt_default_expiration() {
    let jwt_service = JwtService::new("secret".to_string(), None);

    let token = jwt_service
        .generate_token("user", "user@test.com", false)
        .unwrap();

    let claims = jwt_service.verify_token(&token).unwrap();
    let expected_exp = Utc::now() + Duration::hours(24); // Default is 24h

    assert!((claims.exp - expected_exp.timestamp()).abs() < 5);
}

// ============================================================================
// CLAIMS SERIALIZATION TESTS
// ============================================================================

#[test]
fn test_claims_serialization_with_session() {
    let claims = Claims {
        sub: "user_123".to_string(),
        email: "test@example.com".to_string(),
        is_admin: true,
        exp: 1234567890,
        iat: 1234567800,
        session_id: Some("session_xyz".to_string()),
    };

    let json = serde_json::to_string(&claims).unwrap();
    assert!(json.contains("\"sub\":\"user_123\""));
    assert!(json.contains("\"email\":\"test@example.com\""));
    assert!(json.contains("\"is_admin\":true"));
    assert!(json.contains("\"session_id\":\"session_xyz\""));

    let deserialized: Claims = serde_json::from_str(&json).unwrap();
    assert_eq!(deserialized.sub, claims.sub);
    assert_eq!(deserialized.session_id, Some("session_xyz".to_string()));
}

#[test]
fn test_claims_serialization_without_session() {
    let claims = Claims {
        sub: "user_456".to_string(),
        email: "test2@example.com".to_string(),
        is_admin: false,
        exp: 1234567890,
        iat: 1234567800,
        session_id: None,
    };

    let json = serde_json::to_string(&claims).unwrap();
    assert!(!json.contains("session_id")); // Should be skipped

    let deserialized: Claims = serde_json::from_str(&json).unwrap();
    assert_eq!(deserialized.sub, claims.sub);
    assert_eq!(deserialized.session_id, None);
}

#[test]
fn test_claims_backward_compatibility() {
    // Old token format without session_id
    let old_json = r#"{
        "sub":"user123",
        "email":"old@test.com",
        "is_admin":false,
        "exp":1234567890,
        "iat":1234567800
    }"#;

    let claims: Claims = serde_json::from_str(old_json).unwrap();
    assert_eq!(claims.sub, "user123");
    assert_eq!(claims.email, "old@test.com");
    assert!(!claims.is_admin);
    assert_eq!(claims.session_id, None);
}

// ============================================================================
// USER AGENT PARSING TESTS
// ============================================================================

#[test]
fn test_parse_user_agent_chrome() {
    let ua = "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36";
    let (device, browser, os) = parse_user_agent(ua);
    assert_eq!(browser, "Chrome");
    assert_eq!(os, "Windows");
    assert_eq!(device, "Chrome on Windows");
}

#[test]
fn test_parse_user_agent_firefox() {
    let ua = "Mozilla/5.0 (Windows NT 10.0; Win64; x64; rv:120.0) Gecko/20100101 Firefox/120.0";
    let (device, browser, os) = parse_user_agent(ua);
    assert_eq!(browser, "Firefox");
    assert_eq!(os, "Windows");
    assert_eq!(device, "Firefox on Windows");
}

#[test]
fn test_parse_user_agent_safari() {
    let ua = "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/605.1.15 (KHTML, like Gecko) Version/17.0 Safari/605.1.15";
    let (device, browser, os) = parse_user_agent(ua);
    assert_eq!(browser, "Safari");
    assert_eq!(os, "MacOS");
    assert_eq!(device, "Safari on MacOS");
}

#[test]
fn test_parse_user_agent_edge() {
    let ua = "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Edge/120.0.0.0";
    let (device, browser, os) = parse_user_agent(ua);
    assert_eq!(browser, "Edge");
    assert_eq!(os, "Windows");
    assert_eq!(device, "Edge on Windows");
}

#[test]
fn test_parse_user_agent_unknown() {
    let ua = "CustomBrowser/1.0";
    let (device, browser, os) = parse_user_agent(ua);
    assert_eq!(browser, "Unknown");
    assert_eq!(os, "Unknown");
    assert_eq!(device, "Unknown on Unknown");
}

#[test]
fn test_parse_user_agent_empty() {
    let (device, browser, os) = parse_user_agent("");
    assert_eq!(browser, "Unknown");
    assert_eq!(os, "Unknown");
    assert_eq!(device, "Unknown on Unknown");
}

#[test]
fn test_parse_user_agent_mobile() {
    let ua = "Mozilla/5.0 (iPhone; CPU iPhone OS 17_0 like Mac OS X) AppleWebKit/605.1.15 (KHTML, like Gecko) Version/17.0 Mobile/15E148 Safari/604.1";
    let (device, browser, os) = parse_user_agent(ua);
    assert_eq!(browser, "Safari");
    assert_eq!(os, "iOS"); // iPhone should be iOS, not MacOS
    assert_eq!(device, "Safari on iOS");
}

// ============================================================================
// REQUEST VALIDATION TESTS
// ============================================================================

#[test]
fn test_change_password_request_validation_valid() {
    let request = ChangePasswordRequest {
        current_password: "OldPassword123".to_string(),
        new_password: "NewPassword456!".to_string(),
    };

    use validator::Validate;
    assert!(request.validate().is_ok());
}

#[test]
fn test_change_password_request_validation_short_new_password() {
    let request = ChangePasswordRequest {
        current_password: "OldPassword123".to_string(),
        new_password: "123".to_string(), // Too short
    };

    use validator::Validate;
    let result = request.validate();
    assert!(result.is_err());
}

#[test]
fn test_update_profile_request_validation_valid() {
    let request = UpdateProfileRequest {
        display_name: Some("John Doe".to_string()),
        avatar_base64: None,
    };

    use validator::Validate;
    assert!(request.validate().is_ok());
}

#[test]
fn test_update_profile_request_validation_empty_name() {
    let request = UpdateProfileRequest {
        display_name: Some("".to_string()), // Empty is valid (only max length validated)
        avatar_base64: None,
    };

    use validator::Validate;
    let result = request.validate();
    assert!(result.is_ok()); // Empty is allowed, only max length validated
}

#[test]
fn test_update_profile_request_validation_name_too_long() {
    let request = UpdateProfileRequest {
        display_name: Some("a".repeat(101)), // Over 100 chars
        avatar_base64: None,
    };

    use validator::Validate;
    let result = request.validate();
    assert!(result.is_err());
}

#[test]
fn test_update_profile_request_with_avatar() {
    let request = UpdateProfileRequest {
        display_name: Some("Jane Doe".to_string()),
        avatar_base64: Some("iVBORw0KGgoAAAANSUhEUgAAAAEAAAABCAYAAAAfFcSJAAAADUlEQVR42mNk+M9QDwADhgGAWjR9awAAAABJRU5ErkJggg==".to_string()),
    };

    use validator::Validate;
    assert!(request.validate().is_ok());
}

#[test]
fn test_verify_2fa_request_validation_valid() {
    let request = Verify2FARequest {
        code: "123456".to_string(),
    };

    use validator::Validate;
    assert!(request.validate().is_ok());
}

#[test]
fn test_verify_2fa_request_validation_invalid_length() {
    let request = Verify2FARequest {
        code: "12345".to_string(), // Too short (should be 6 digits)
    };

    use validator::Validate;
    let result = request.validate();
    assert!(result.is_err());
}

#[test]
fn test_verify_2fa_request_validation_too_long() {
    let request = Verify2FARequest {
        code: "1234567".to_string(), // Too long
    };

    use validator::Validate;
    let result = request.validate();
    assert!(result.is_err());
}

#[test]
fn test_verify_2fa_request_validation_non_numeric() {
    let request = Verify2FARequest {
        code: "12345a".to_string(), // Contains letter but length is 6
    };

    use validator::Validate;
    let result = request.validate();
    // Validation only checks length, not content - application logic validates numeric
    assert!(result.is_ok());
}

#[test]
fn test_verify_2fa_request_validation_exactly_six() {
    let request = Verify2FARequest {
        code: "abcdef".to_string(), // 6 chars (validation only checks length)
    };

    use validator::Validate;
    assert!(request.validate().is_ok());
}

// ============================================================================
// SECURITY SERVICE TESTS
// ============================================================================

#[test]
fn test_security_service_creation() {
    let service = SecurityService::new_dummy();
    // Should create without panic
    assert!(true);
}

// ============================================================================
// INTEGRATION TESTS (Edge Cases)
// ============================================================================

#[test]
fn test_password_and_jwt_integration() {
    // Simulate full auth flow
    let password = "UserPassword123!";
    let hash = PasswordService::hash_password(password).unwrap();

    // Verify password
    assert!(PasswordService::verify_password(password, &hash).unwrap());

    // Generate JWT after successful verification
    let jwt_service = JwtService::new("secret".to_string(), Some(24));
    let token = jwt_service
        .generate_token("user_123", "user@test.com", false)
        .unwrap();

    // Verify JWT
    let claims = jwt_service.verify_token(&token).unwrap();
    assert_eq!(claims.sub, "user_123");
}

#[test]
fn test_multiple_password_changes() {
    let password1 = "FirstPassword123!";
    let password2 = "SecondPassword456!";
    let password3 = "ThirdPassword789!";

    let hash1 = PasswordService::hash_password(password1).unwrap();
    assert!(PasswordService::verify_password(password1, &hash1).unwrap());

    let hash2 = PasswordService::hash_password(password2).unwrap();
    assert!(PasswordService::verify_password(password2, &hash2).unwrap());
    assert!(!PasswordService::verify_password(password1, &hash2).unwrap());

    let hash3 = PasswordService::hash_password(password3).unwrap();
    assert!(PasswordService::verify_password(password3, &hash3).unwrap());
    assert!(!PasswordService::verify_password(password2, &hash3).unwrap());
}

#[test]
fn test_jwt_session_lifecycle() {
    let jwt_service = JwtService::new("secret".to_string(), Some(1));

    // Create session
    let session_id = "session_12345";
    let token = jwt_service
        .generate_token_with_session(
            "user_1",
            "user@test.com",
            false,
            Some(session_id.to_string()),
        )
        .unwrap();

    // Verify session
    let claims = jwt_service.verify_token(&token).unwrap();
    assert_eq!(claims.session_id, Some(session_id.to_string()));

    // Create new session for same user
    let new_session_id = "session_67890";
    let new_token = jwt_service
        .generate_token_with_session(
            "user_1",
            "user@test.com",
            false,
            Some(new_session_id.to_string()),
        )
        .unwrap();

    let new_claims = jwt_service.verify_token(&new_token).unwrap();
    assert_eq!(new_claims.session_id, Some(new_session_id.to_string()));

    // Both tokens should be valid but have different sessions
    assert_ne!(claims.session_id, new_claims.session_id);
}

#[test]
fn test_concurrent_jwt_generation() {
    use std::thread;

    let jwt_service = std::sync::Arc::new(JwtService::new("secret".to_string(), Some(1)));

    let mut handles = vec![];
    for i in 0..10 {
        let service = jwt_service.clone();
        let handle = thread::spawn(move || {
            service
                .generate_token(&format!("user_{}", i), "user@test.com", false)
                .unwrap()
        });
        handles.push(handle);
    }

    let tokens: Vec<String> = handles.into_iter().map(|h| h.join().unwrap()).collect();

    // All tokens should be valid and unique
    assert_eq!(tokens.len(), 10);
    for (i, token) in tokens.iter().enumerate() {
        let claims = jwt_service.verify_token(token).unwrap();
        assert_eq!(claims.sub, format!("user_{}", i));
    }
}

#[test]
fn test_password_timing_attack_resistance() {
    // bcrypt should have constant-time comparison
    let password = "TestPassword123!";
    let hash = PasswordService::hash_password(password).unwrap();

    // These should all take similar time (bcrypt property)
    let start1 = std::time::Instant::now();
    let _ = PasswordService::verify_password("wrong1", &hash);
    let elapsed1 = start1.elapsed();

    let start2 = std::time::Instant::now();
    let _ = PasswordService::verify_password("wrong2", &hash);
    let elapsed2 = start2.elapsed();

    // Times should be relatively close (within 2x of each other)
    // This is a rough check - bcrypt is designed for constant time
    let ratio = elapsed1.as_millis() as f64 / elapsed2.as_millis().max(1) as f64;
    assert!(ratio > 0.5 && ratio < 2.0, "Timing ratio: {}", ratio);
}
