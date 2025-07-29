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
        if auth_header.starts_with("Bearer ") {
            Some(&auth_header[7..])
        } else {
            None
        }
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
}
