use anyhow::Result;
use chrono::{Duration, Utc};
use jsonwebtoken::{
    decode, encode, Algorithm, DecodingKey, EncodingKey, Header, TokenData, Validation,
};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    pub sub: String, // user id
    pub email: String,
    pub is_admin: bool,
    pub exp: i64, // expiration time
    pub iat: i64, // issued at
}

#[derive(Clone)]
pub struct JwtService {
    secret: String,
    expiration_hours: i64,
}

impl JwtService {
    pub fn new(secret: String, expiration_hours: Option<i64>) -> Self {
        Self {
            secret,
            expiration_hours: expiration_hours.unwrap_or(24), // Default 24 hours
        }
    }

    pub fn generate_token(&self, user_id: &str, email: &str, is_admin: bool) -> Result<String> {
        let now = Utc::now();
        let exp = now + Duration::hours(self.expiration_hours);

        let claims = Claims {
            sub: user_id.to_string(),
            email: email.to_string(),
            is_admin,
            exp: exp.timestamp(),
            iat: now.timestamp(),
        };

        let header = Header::new(Algorithm::HS256);
        let token = encode(
            &header,
            &claims,
            &EncodingKey::from_secret(self.secret.as_ref()),
        )?;

        Ok(token)
    }

    pub fn verify_token(&self, token: &str) -> Result<Claims> {
        let validation = Validation::new(Algorithm::HS256);
        let token_data: TokenData<Claims> = decode(
            token,
            &DecodingKey::from_secret(self.secret.as_ref()),
            &validation,
        )?;

        Ok(token_data.claims)
    }

    pub fn extract_token_from_header(auth_header: &str) -> Option<&str> {
        auth_header.strip_prefix("Bearer ")
    }
}

// Password hashing utilities
pub struct PasswordService;

impl PasswordService {
    pub fn hash_password(password: &str) -> Result<String> {
        let hashed = bcrypt::hash(password, bcrypt::DEFAULT_COST)?;
        Ok(hashed)
    }

    pub fn verify_password(password: &str, hash: &str) -> Result<bool> {
        let is_valid = bcrypt::verify(password, hash)?;
        Ok(is_valid)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_password_hashing() {
        let password = "test_password_123";
        let hashed = PasswordService::hash_password(password).unwrap();

        assert!(PasswordService::verify_password(password, &hashed).unwrap());
        assert!(!PasswordService::verify_password("wrong_password", &hashed).unwrap());
    }

    #[test]
    fn test_jwt_token() {
        let jwt_service = JwtService::new("test_secret".to_string(), Some(1));
        let token = jwt_service
            .generate_token("user123", "test@example.com", false)
            .unwrap();

        let claims = jwt_service.verify_token(&token).unwrap();
        assert_eq!(claims.sub, "user123");
        assert_eq!(claims.email, "test@example.com");
        assert!(!claims.is_admin);
    }

    #[test]
    fn test_jwt_service_new_with_default_expiration() {
        let service = JwtService::new("secret".to_string(), None);
        assert_eq!(service.expiration_hours, 24);
    }

    #[test]
    fn test_jwt_service_new_with_custom_expiration() {
        let service = JwtService::new("secret".to_string(), Some(48));
        assert_eq!(service.expiration_hours, 48);
    }

    #[test]
    fn test_generate_token_for_admin_user() {
        let jwt_service = JwtService::new("test_secret".to_string(), Some(24));
        let token = jwt_service
            .generate_token("admin123", "admin@example.com", true)
            .unwrap();

        let claims = jwt_service.verify_token(&token).unwrap();
        assert_eq!(claims.sub, "admin123");
        assert_eq!(claims.email, "admin@example.com");
        assert!(claims.is_admin);
    }

    #[test]
    fn test_verify_token_with_wrong_secret() {
        let jwt_service1 = JwtService::new("secret1".to_string(), Some(1));
        let jwt_service2 = JwtService::new("secret2".to_string(), Some(1));

        let token = jwt_service1
            .generate_token("user123", "test@example.com", false)
            .unwrap();

        // Should fail with different secret
        assert!(jwt_service2.verify_token(&token).is_err());
    }

    #[test]
    fn test_verify_token_with_invalid_token() {
        let jwt_service = JwtService::new("test_secret".to_string(), Some(1));

        // Invalid token format
        assert!(jwt_service.verify_token("invalid.token.here").is_err());
        assert!(jwt_service.verify_token("notavalidtoken").is_err());
        assert!(jwt_service.verify_token("").is_err());
    }

    #[test]
    fn test_extract_token_from_header_success() {
        let header = "Bearer eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9.test.token";
        let token = JwtService::extract_token_from_header(header);
        assert_eq!(
            token,
            Some("eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9.test.token")
        );
    }

    #[test]
    fn test_extract_token_from_header_no_bearer() {
        let header = "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9.test.token";
        let token = JwtService::extract_token_from_header(header);
        assert_eq!(token, None);
    }

    #[test]
    fn test_extract_token_from_header_empty() {
        let header = "Bearer ";
        let token = JwtService::extract_token_from_header(header);
        assert_eq!(token, Some(""));
    }

    #[test]
    fn test_extract_token_from_header_wrong_prefix() {
        let header = "Basic sometoken";
        let token = JwtService::extract_token_from_header(header);
        assert_eq!(token, None);
    }

    #[test]
    fn test_claims_contains_correct_timestamps() {
        let jwt_service = JwtService::new("test_secret".to_string(), Some(2));
        let token = jwt_service
            .generate_token("user123", "test@example.com", false)
            .unwrap();

        let claims = jwt_service.verify_token(&token).unwrap();

        // Verify exp is after iat
        assert!(claims.exp > claims.iat);

        // Verify exp is approximately 2 hours after iat (with 1 second tolerance)
        let diff_hours = (claims.exp - claims.iat) / 3600;
        assert_eq!(diff_hours, 2);
    }

    #[test]
    fn test_password_hash_is_different_each_time() {
        let password = "test_password";
        let hash1 = PasswordService::hash_password(password).unwrap();
        let hash2 = PasswordService::hash_password(password).unwrap();

        // Hashes should be different due to random salt
        assert_ne!(hash1, hash2);

        // But both should verify correctly
        assert!(PasswordService::verify_password(password, &hash1).unwrap());
        assert!(PasswordService::verify_password(password, &hash2).unwrap());
    }

    #[test]
    fn test_password_verify_with_empty_password() {
        let password = "test_password";
        let hash = PasswordService::hash_password(password).unwrap();

        assert!(!PasswordService::verify_password("", &hash).unwrap());
    }

    #[test]
    fn test_password_hash_empty_password() {
        let password = "";
        let hash = PasswordService::hash_password(password).unwrap();
        assert!(PasswordService::verify_password(password, &hash).unwrap());
    }

    #[test]
    fn test_jwt_service_clone() {
        let service1 = JwtService::new("secret".to_string(), Some(10));
        let service2 = service1.clone();

        let token = service1
            .generate_token("user1", "test@test.com", false)
            .unwrap();

        // Cloned service should be able to verify token from original
        assert!(service2.verify_token(&token).is_ok());
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

        // Test that claims can be serialized and deserialized
        let json = serde_json::to_string(&claims).unwrap();
        let deserialized: Claims = serde_json::from_str(&json).unwrap();

        assert_eq!(deserialized.sub, "user123");
        assert_eq!(deserialized.email, "test@example.com");
        assert_eq!(deserialized.is_admin, false);
        assert_eq!(deserialized.exp, 1234567890);
        assert_eq!(deserialized.iat, 1234567800);
    }
}
