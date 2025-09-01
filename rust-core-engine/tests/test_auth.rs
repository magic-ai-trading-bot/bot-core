mod common;

use binance_trading_bot::auth::jwt::{Claims, JwtService, PasswordService};
use chrono::{Duration, Utc};

#[test]
fn test_password_hash_and_verify() {
    let password = "SecurePassword123!@#";
    let hash = PasswordService::hash_password(password).unwrap();
    
    // Verify correct password
    assert!(PasswordService::verify_password(password, &hash).unwrap());
    
    // Verify wrong password fails
    assert!(!PasswordService::verify_password("WrongPassword", &hash).unwrap());
    
    // Verify hash is different each time (due to salt)
    let hash2 = PasswordService::hash_password(password).unwrap();
    assert_ne!(hash, hash2);
}

#[test]
fn test_jwt_token_generation_and_verification() {
    let jwt_service = JwtService::new("super_secret_key".to_string(), Some(24));
    
    let token = jwt_service
        .generate_token("user_id_123", "user@example.com", true)
        .unwrap();
    
    // Verify token
    let claims = jwt_service.verify_token(&token).unwrap();
    assert_eq!(claims.sub, "user_id_123");
    assert_eq!(claims.email, "user@example.com");
    assert!(claims.is_admin);
    
    // Check expiration is set correctly (24 hours)
    let expected_exp = Utc::now() + Duration::hours(24);
    assert!((claims.exp - expected_exp.timestamp()).abs() < 5); // Allow 5 seconds difference
}

#[test]
fn test_jwt_token_expiration() {
    // Create token with 0 hours expiration (already expired)
    let jwt_service = JwtService::new("test_secret".to_string(), Some(0));
    
    let token = jwt_service
        .generate_token("user123", "test@test.com", false)
        .unwrap();
    
    // Token should still be verifiable immediately
    let claims = jwt_service.verify_token(&token).unwrap();
    assert_eq!(claims.sub, "user123");
}

#[test]
fn test_jwt_invalid_token() {
    let jwt_service = JwtService::new("secret_key".to_string(), None);
    
    // Test with invalid token
    let result = jwt_service.verify_token("invalid.token.here");
    assert!(result.is_err());
    
    // Test with token signed with different secret
    let other_service = JwtService::new("different_secret".to_string(), None);
    let token = other_service
        .generate_token("user1", "user@test.com", false)
        .unwrap();
    
    let result = jwt_service.verify_token(&token);
    assert!(result.is_err());
}

#[test]
fn test_extract_token_from_header() {
    // Valid Bearer token
    let header = "Bearer eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9";
    let token = JwtService::extract_token_from_header(header);
    assert_eq!(token, Some("eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9"));
    
    // Invalid header format
    let header = "Basic dXNlcjpwYXNz";
    let token = JwtService::extract_token_from_header(header);
    assert_eq!(token, None);
    
    // No Bearer prefix
    let header = "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9";
    let token = JwtService::extract_token_from_header(header);
    assert_eq!(token, None);
}

#[test]
fn test_jwt_with_different_users() {
    let jwt_service = JwtService::new("secret".to_string(), Some(24));
    
    // Generate tokens for different users
    let admin_token = jwt_service
        .generate_token("admin_001", "admin@company.com", true)
        .unwrap();
    
    let user_token = jwt_service
        .generate_token("user_002", "user@company.com", false)
        .unwrap();
    
    // Verify admin token
    let admin_claims = jwt_service.verify_token(&admin_token).unwrap();
    assert_eq!(admin_claims.sub, "admin_001");
    assert!(admin_claims.is_admin);
    
    // Verify user token
    let user_claims = jwt_service.verify_token(&user_token).unwrap();
    assert_eq!(user_claims.sub, "user_002");
    assert!(!user_claims.is_admin);
    
    // Tokens should be different
    assert_ne!(admin_token, user_token);
}

#[test]
fn test_password_hash_errors() {
    // Test empty password
    let result = PasswordService::hash_password("");
    assert!(result.is_ok()); // bcrypt allows empty passwords
    
    // Test very long password
    let long_password = "a".repeat(1000);
    let result = PasswordService::hash_password(&long_password);
    assert!(result.is_ok());
    
    // Test special characters
    let special = "!@#$%^&*()_+-=[]{}|;':,.<>?/~`";
    let hash = PasswordService::hash_password(special).unwrap();
    assert!(PasswordService::verify_password(special, &hash).unwrap());
}

#[test]
fn test_jwt_default_expiration() {
    // Test with default expiration (24 hours)
    let jwt_service = JwtService::new("secret".to_string(), None);
    
    let token = jwt_service
        .generate_token("user", "user@test.com", false)
        .unwrap();
    
    let claims = jwt_service.verify_token(&token).unwrap();
    
    // Default should be 24 hours
    let expected_exp = Utc::now() + Duration::hours(24);
    assert!((claims.exp - expected_exp.timestamp()).abs() < 5);
}

#[test]
fn test_claims_serialization() {
    let claims = Claims {
        sub: "user123".to_string(),
        email: "test@example.com".to_string(),
        is_admin: false,
        exp: 1234567890,
        iat: 1234567800,
    };
    
    // Test serialization
    let json = serde_json::to_string(&claims).unwrap();
    assert!(json.contains("\"sub\":\"user123\""));
    assert!(json.contains("\"email\":\"test@example.com\""));
    
    // Test deserialization
    let deserialized: Claims = serde_json::from_str(&json).unwrap();
    assert_eq!(deserialized.sub, claims.sub);
    assert_eq!(deserialized.email, claims.email);
    assert_eq!(deserialized.is_admin, claims.is_admin);
}
