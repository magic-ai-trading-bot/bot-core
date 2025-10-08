use bson::oid::ObjectId;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use validator::Validate;

// Custom serde module for handling MongoDB DateTime
mod date_time_serde {
    use chrono::{DateTime, Utc};
    use serde::{self, Deserialize, Deserializer, Serializer};

    pub fn serialize<S>(date: &DateTime<Utc>, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        // Serialize as BSON DateTime for MongoDB
        serializer.serialize_str(&date.to_rfc3339())
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<DateTime<Utc>, D::Error>
    where
        D: Deserializer<'de>,
    {
        use serde::de::Error;

        // Try to deserialize from different formats
        let value = bson::Bson::deserialize(deserializer)?;

        match value {
            // BSON DateTime (from MongoDB)
            bson::Bson::DateTime(dt) => {
                Ok(DateTime::from_timestamp_millis(dt.timestamp_millis()).unwrap_or_else(Utc::now))
            }
            // String format (RFC 3339)
            bson::Bson::String(s) => s.parse().map_err(D::Error::custom),
            _ => Err(D::Error::custom("Expected DateTime or String")),
        }
    }
}

// Optional DateTime serde
mod optional_date_time_serde {
    use chrono::{DateTime, Utc};
    use serde::{self, Deserialize, Deserializer, Serializer};

    pub fn serialize<S>(date: &Option<DateTime<Utc>>, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        match date {
            Some(dt) => serializer.serialize_some(&dt.to_rfc3339()),
            None => serializer.serialize_none(),
        }
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<Option<DateTime<Utc>>, D::Error>
    where
        D: Deserializer<'de>,
    {
        use serde::de::Error;

        let value: Option<bson::Bson> = Option::deserialize(deserializer)?;

        match value {
            Some(bson::Bson::DateTime(dt)) => Ok(Some(
                DateTime::from_timestamp_millis(dt.timestamp_millis()).unwrap_or_else(Utc::now),
            )),
            Some(bson::Bson::String(s)) => Ok(Some(s.parse().map_err(D::Error::custom)?)),
            Some(_) => Err(D::Error::custom("Expected DateTime or String")),
            None => Ok(None),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct User {
    #[serde(rename = "_id", skip_serializing_if = "Option::is_none")]
    pub id: Option<ObjectId>,
    pub email: String,
    pub password_hash: String,
    pub full_name: Option<String>,
    pub is_active: bool,
    pub is_admin: bool,
    #[serde(with = "date_time_serde")]
    pub created_at: DateTime<Utc>,
    #[serde(with = "date_time_serde")]
    pub updated_at: DateTime<Utc>,
    #[serde(with = "optional_date_time_serde")]
    pub last_login: Option<DateTime<Utc>>,
    pub settings: UserSettings,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserSettings {
    pub trading_enabled: bool,
    pub risk_level: RiskLevel,
    pub max_positions: u32,
    pub default_quantity: f64,
    pub notifications: NotificationSettings,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RiskLevel {
    Low,
    Medium,
    High,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NotificationSettings {
    pub email_alerts: bool,
    pub trade_notifications: bool,
    pub system_alerts: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct RegisterRequest {
    #[validate(email(message = "Invalid email format"))]
    pub email: String,
    #[validate(length(min = 6, message = "Password must be at least 6 characters"))]
    pub password: String,
    pub full_name: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct LoginRequest {
    #[validate(email(message = "Invalid email format"))]
    pub email: String,
    #[validate(length(min = 1, message = "Password cannot be empty"))]
    pub password: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoginResponse {
    pub token: String,
    pub user: UserProfile,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserProfile {
    pub id: String,
    pub email: String,
    pub full_name: Option<String>,
    pub is_active: bool,
    pub is_admin: bool,
    #[serde(with = "date_time_serde")]
    pub created_at: DateTime<Utc>,
    #[serde(with = "optional_date_time_serde")]
    pub last_login: Option<DateTime<Utc>>,
    pub settings: UserSettings,
}

impl Default for UserSettings {
    fn default() -> Self {
        Self {
            trading_enabled: false,
            risk_level: RiskLevel::Medium,
            max_positions: 3,
            default_quantity: 0.01,
            notifications: NotificationSettings::default(),
        }
    }
}

impl Default for NotificationSettings {
    fn default() -> Self {
        Self {
            email_alerts: true,
            trade_notifications: true,
            system_alerts: true,
        }
    }
}

impl User {
    pub fn new(email: String, password_hash: String, full_name: Option<String>) -> Self {
        let now = Utc::now();
        Self {
            id: None,
            email,
            password_hash,
            full_name,
            is_active: true,
            is_admin: false,
            created_at: now,
            updated_at: now,
            last_login: None,
            settings: UserSettings::default(),
        }
    }

    pub fn to_profile(&self) -> UserProfile {
        UserProfile {
            id: self.id.as_ref().map(|id| id.to_hex()).unwrap_or_default(),
            email: self.email.clone(),
            full_name: self.full_name.clone(),
            is_active: self.is_active,
            is_admin: self.is_admin,
            created_at: self.created_at,
            last_login: self.last_login,
            settings: self.settings.clone(),
        }
    }

    pub fn update_last_login(&mut self) {
        self.last_login = Some(Utc::now());
        self.updated_at = Utc::now();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_user_new_creates_valid_user() {
        let email = "test@example.com".to_string();
        let password_hash = "hashed_password".to_string();
        let full_name = Some("Test User".to_string());

        let user = User::new(email.clone(), password_hash.clone(), full_name.clone());

        assert_eq!(user.email, email);
        assert_eq!(user.password_hash, password_hash);
        assert_eq!(user.full_name, full_name);
        assert!(user.is_active);
        assert!(!user.is_admin);
        assert_eq!(user.id, None);
        assert_eq!(user.last_login, None);
    }

    #[test]
    fn test_user_new_without_full_name() {
        let user = User::new(
            "test@example.com".to_string(),
            "hashed_password".to_string(),
            None,
        );

        assert_eq!(user.full_name, None);
        assert!(user.is_active);
    }

    #[test]
    fn test_user_to_profile_without_id() {
        let user = User::new(
            "test@example.com".to_string(),
            "hashed_password".to_string(),
            Some("Test User".to_string()),
        );

        let profile = user.to_profile();

        assert_eq!(profile.id, "");
        assert_eq!(profile.email, user.email);
        assert_eq!(profile.full_name, user.full_name);
        assert_eq!(profile.is_active, user.is_active);
        assert_eq!(profile.is_admin, user.is_admin);
    }

    #[test]
    fn test_user_to_profile_with_id() {
        let mut user = User::new(
            "test@example.com".to_string(),
            "hashed_password".to_string(),
            Some("Test User".to_string()),
        );

        let object_id = ObjectId::new();
        user.id = Some(object_id);

        let profile = user.to_profile();

        assert_eq!(profile.id, object_id.to_hex());
        assert_eq!(profile.email, user.email);
    }

    #[test]
    fn test_user_update_last_login() {
        let mut user = User::new(
            "test@example.com".to_string(),
            "hashed_password".to_string(),
            None,
        );

        assert_eq!(user.last_login, None);

        let before_update = Utc::now();
        user.update_last_login();

        assert!(user.last_login.is_some());
        assert!(user.last_login.unwrap() >= before_update);
        assert!(user.updated_at >= before_update);
    }

    #[test]
    fn test_user_settings_default() {
        let settings = UserSettings::default();

        assert!(!settings.trading_enabled);
        assert_eq!(settings.max_positions, 3);
        assert_eq!(settings.default_quantity, 0.01);
        assert!(matches!(settings.risk_level, RiskLevel::Medium));
    }

    #[test]
    fn test_notification_settings_default() {
        let notifications = NotificationSettings::default();

        assert!(notifications.email_alerts);
        assert!(notifications.trade_notifications);
        assert!(notifications.system_alerts);
    }

    #[test]
    fn test_register_request_validation_valid_email() {
        let request = RegisterRequest {
            email: "test@example.com".to_string(),
            password: "password123".to_string(),
            full_name: None,
        };

        assert!(request.validate().is_ok());
    }

    #[test]
    fn test_register_request_validation_invalid_email() {
        let request = RegisterRequest {
            email: "invalid-email".to_string(),
            password: "password123".to_string(),
            full_name: None,
        };

        assert!(request.validate().is_err());
    }

    #[test]
    fn test_register_request_validation_short_password() {
        let request = RegisterRequest {
            email: "test@example.com".to_string(),
            password: "12345".to_string(),
            full_name: None,
        };

        assert!(request.validate().is_err());
    }

    #[test]
    fn test_register_request_validation_minimum_password() {
        let request = RegisterRequest {
            email: "test@example.com".to_string(),
            password: "123456".to_string(),
            full_name: None,
        };

        assert!(request.validate().is_ok());
    }

    #[test]
    fn test_login_request_validation_valid() {
        let request = LoginRequest {
            email: "test@example.com".to_string(),
            password: "password".to_string(),
        };

        assert!(request.validate().is_ok());
    }

    #[test]
    fn test_login_request_validation_invalid_email() {
        let request = LoginRequest {
            email: "not-an-email".to_string(),
            password: "password".to_string(),
        };

        assert!(request.validate().is_err());
    }

    #[test]
    fn test_login_request_validation_empty_password() {
        let request = LoginRequest {
            email: "test@example.com".to_string(),
            password: "".to_string(),
        };

        assert!(request.validate().is_err());
    }

    #[test]
    fn test_user_serialization() {
        let user = User::new(
            "test@example.com".to_string(),
            "hashed_password".to_string(),
            Some("Test User".to_string()),
        );

        let json = serde_json::to_string(&user).unwrap();
        let deserialized: User = serde_json::from_str(&json).unwrap();

        assert_eq!(deserialized.email, user.email);
        assert_eq!(deserialized.password_hash, user.password_hash);
        assert_eq!(deserialized.full_name, user.full_name);
    }

    #[test]
    fn test_login_response_serialization() {
        let user = User::new(
            "test@example.com".to_string(),
            "hashed_password".to_string(),
            Some("Test User".to_string()),
        );

        let response = LoginResponse {
            token: "test_token".to_string(),
            user: user.to_profile(),
        };

        let json = serde_json::to_string(&response).unwrap();
        let deserialized: LoginResponse = serde_json::from_str(&json).unwrap();

        assert_eq!(deserialized.token, "test_token");
        assert_eq!(deserialized.user.email, "test@example.com");
    }

    #[test]
    fn test_risk_level_serialization() {
        let low = RiskLevel::Low;
        let medium = RiskLevel::Medium;
        let high = RiskLevel::High;

        let low_json = serde_json::to_string(&low).unwrap();
        let medium_json = serde_json::to_string(&medium).unwrap();
        let high_json = serde_json::to_string(&high).unwrap();

        let low_deserialized: RiskLevel = serde_json::from_str(&low_json).unwrap();
        let medium_deserialized: RiskLevel = serde_json::from_str(&medium_json).unwrap();
        let high_deserialized: RiskLevel = serde_json::from_str(&high_json).unwrap();

        assert!(matches!(low_deserialized, RiskLevel::Low));
        assert!(matches!(medium_deserialized, RiskLevel::Medium));
        assert!(matches!(high_deserialized, RiskLevel::High));
    }

    #[test]
    fn test_user_profile_clone() {
        let user = User::new(
            "test@example.com".to_string(),
            "hashed_password".to_string(),
            Some("Test User".to_string()),
        );

        let profile1 = user.to_profile();
        let profile2 = profile1.clone();

        assert_eq!(profile1.email, profile2.email);
        assert_eq!(profile1.full_name, profile2.full_name);
    }

    #[test]
    fn test_user_settings_clone() {
        let settings1 = UserSettings::default();
        let settings2 = settings1.clone();

        assert_eq!(settings1.trading_enabled, settings2.trading_enabled);
        assert_eq!(settings1.max_positions, settings2.max_positions);
        assert_eq!(settings1.default_quantity, settings2.default_quantity);
    }

    #[test]
    fn test_notification_settings_clone() {
        let notif1 = NotificationSettings::default();
        let notif2 = notif1.clone();

        assert_eq!(notif1.email_alerts, notif2.email_alerts);
        assert_eq!(notif1.trade_notifications, notif2.trade_notifications);
        assert_eq!(notif1.system_alerts, notif2.system_alerts);
    }

    #[test]
    fn test_register_request_with_full_name() {
        let request = RegisterRequest {
            email: "test@example.com".to_string(),
            password: "password123".to_string(),
            full_name: Some("John Doe".to_string()),
        };

        assert!(request.validate().is_ok());
        assert_eq!(request.full_name, Some("John Doe".to_string()));
    }

    #[test]
    fn test_date_time_serde_serialize() {
        let user = User::new(
            "test@example.com".to_string(),
            "hash".to_string(),
            None,
        );

        // Test that created_at and updated_at are serialized
        let json = serde_json::to_string(&user).unwrap();
        assert!(json.contains("created_at"));
        assert!(json.contains("updated_at"));
    }

    #[test]
    fn test_date_time_serde_deserialize_from_string() {
        use serde_json::json;

        let json_value = json!({
            "email": "test@example.com",
            "password_hash": "hash",
            "full_name": null,
            "is_active": true,
            "is_admin": false,
            "created_at": "2024-01-01T12:00:00Z",
            "updated_at": "2024-01-01T12:00:00Z",
            "last_login": null,
            "settings": {
                "trading_enabled": false,
                "risk_level": "Medium",
                "max_positions": 3,
                "default_quantity": 0.01,
                "notifications": {
                    "email_alerts": true,
                    "trade_notifications": true,
                    "system_alerts": true
                }
            }
        });

        let user: User = serde_json::from_value(json_value).unwrap();
        assert_eq!(user.email, "test@example.com");
    }

    #[test]
    fn test_optional_date_time_serde_serialize_some() {
        let mut user = User::new(
            "test@example.com".to_string(),
            "hash".to_string(),
            None,
        );
        user.update_last_login();

        let json = serde_json::to_string(&user).unwrap();
        assert!(json.contains("last_login"));
    }

    #[test]
    fn test_optional_date_time_serde_serialize_none() {
        let user = User::new(
            "test@example.com".to_string(),
            "hash".to_string(),
            None,
        );

        let json = serde_json::to_string(&user).unwrap();
        // last_login should be present but null
        assert!(json.contains("last_login"));
    }

    #[test]
    fn test_optional_date_time_serde_deserialize_some() {
        use serde_json::json;

        let json_value = json!({
            "email": "test@example.com",
            "password_hash": "hash",
            "full_name": null,
            "is_active": true,
            "is_admin": false,
            "created_at": "2024-01-01T12:00:00Z",
            "updated_at": "2024-01-01T12:00:00Z",
            "last_login": "2024-01-02T12:00:00Z",
            "settings": {
                "trading_enabled": false,
                "risk_level": "Medium",
                "max_positions": 3,
                "default_quantity": 0.01,
                "notifications": {
                    "email_alerts": true,
                    "trade_notifications": true,
                    "system_alerts": true
                }
            }
        });

        let user: User = serde_json::from_value(json_value).unwrap();
        assert!(user.last_login.is_some());
    }

    #[test]
    fn test_optional_date_time_serde_deserialize_none() {
        use serde_json::json;

        let json_value = json!({
            "email": "test@example.com",
            "password_hash": "hash",
            "full_name": null,
            "is_active": true,
            "is_admin": false,
            "created_at": "2024-01-01T12:00:00Z",
            "updated_at": "2024-01-01T12:00:00Z",
            "last_login": null,
            "settings": {
                "trading_enabled": false,
                "risk_level": "Medium",
                "max_positions": 3,
                "default_quantity": 0.01,
                "notifications": {
                    "email_alerts": true,
                    "trade_notifications": true,
                    "system_alerts": true
                }
            }
        });

        let user: User = serde_json::from_value(json_value).unwrap();
        assert!(user.last_login.is_none());
    }

    #[test]
    fn test_user_debug_trait() {
        let user = User::new(
            "test@example.com".to_string(),
            "hash".to_string(),
            Some("Test User".to_string()),
        );

        let debug_str = format!("{:?}", user);
        assert!(debug_str.contains("test@example.com"));
        assert!(debug_str.contains("Test User"));
    }

    #[test]
    fn test_user_profile_debug_trait() {
        let user = User::new(
            "test@example.com".to_string(),
            "hash".to_string(),
            None,
        );
        let profile = user.to_profile();

        let debug_str = format!("{:?}", profile);
        assert!(debug_str.contains("test@example.com"));
    }

    #[test]
    fn test_user_settings_debug_trait() {
        let settings = UserSettings::default();
        let debug_str = format!("{:?}", settings);
        assert!(debug_str.contains("UserSettings"));
    }

    #[test]
    fn test_notification_settings_debug_trait() {
        let notif = NotificationSettings::default();
        let debug_str = format!("{:?}", notif);
        assert!(debug_str.contains("NotificationSettings"));
    }

    #[test]
    fn test_risk_level_debug_trait() {
        let low = RiskLevel::Low;
        let medium = RiskLevel::Medium;
        let high = RiskLevel::High;

        assert!(format!("{:?}", low).contains("Low"));
        assert!(format!("{:?}", medium).contains("Medium"));
        assert!(format!("{:?}", high).contains("High"));
    }

    #[test]
    fn test_register_request_debug_trait() {
        let request = RegisterRequest {
            email: "test@example.com".to_string(),
            password: "password123".to_string(),
            full_name: None,
        };

        let debug_str = format!("{:?}", request);
        assert!(debug_str.contains("RegisterRequest"));
    }

    #[test]
    fn test_login_request_debug_trait() {
        let request = LoginRequest {
            email: "test@example.com".to_string(),
            password: "password".to_string(),
        };

        let debug_str = format!("{:?}", request);
        assert!(debug_str.contains("LoginRequest"));
    }

    #[test]
    fn test_login_response_debug_trait() {
        let user = User::new(
            "test@example.com".to_string(),
            "hash".to_string(),
            None,
        );
        let response = LoginResponse {
            token: "token".to_string(),
            user: user.to_profile(),
        };

        let debug_str = format!("{:?}", response);
        assert!(debug_str.contains("LoginResponse"));
    }

    #[test]
    fn test_user_clone() {
        let user1 = User::new(
            "test@example.com".to_string(),
            "hash".to_string(),
            Some("Test".to_string()),
        );
        let user2 = user1.clone();

        assert_eq!(user1.email, user2.email);
        assert_eq!(user1.password_hash, user2.password_hash);
        assert_eq!(user1.full_name, user2.full_name);
    }

    #[test]
    fn test_register_request_clone() {
        let req1 = RegisterRequest {
            email: "test@example.com".to_string(),
            password: "password123".to_string(),
            full_name: Some("Test".to_string()),
        };
        let req2 = req1.clone();

        assert_eq!(req1.email, req2.email);
        assert_eq!(req1.password, req2.password);
        assert_eq!(req1.full_name, req2.full_name);
    }

    #[test]
    fn test_login_request_clone() {
        let req1 = LoginRequest {
            email: "test@example.com".to_string(),
            password: "password".to_string(),
        };
        let req2 = req1.clone();

        assert_eq!(req1.email, req2.email);
        assert_eq!(req1.password, req2.password);
    }

    #[test]
    fn test_login_response_clone() {
        let user = User::new(
            "test@example.com".to_string(),
            "hash".to_string(),
            None,
        );
        let resp1 = LoginResponse {
            token: "token".to_string(),
            user: user.to_profile(),
        };
        let resp2 = resp1.clone();

        assert_eq!(resp1.token, resp2.token);
        assert_eq!(resp1.user.email, resp2.user.email);
    }

    #[test]
    fn test_user_settings_custom_values() {
        let mut settings = UserSettings::default();
        settings.trading_enabled = true;
        settings.risk_level = RiskLevel::High;
        settings.max_positions = 10;
        settings.default_quantity = 0.5;

        assert!(settings.trading_enabled);
        assert!(matches!(settings.risk_level, RiskLevel::High));
        assert_eq!(settings.max_positions, 10);
        assert_eq!(settings.default_quantity, 0.5);
    }

    #[test]
    fn test_notification_settings_custom_values() {
        let mut notif = NotificationSettings::default();
        notif.email_alerts = false;
        notif.trade_notifications = false;
        notif.system_alerts = false;

        assert!(!notif.email_alerts);
        assert!(!notif.trade_notifications);
        assert!(!notif.system_alerts);
    }

    #[test]
    fn test_user_profile_with_last_login() {
        let mut user = User::new(
            "test@example.com".to_string(),
            "hash".to_string(),
            None,
        );
        user.update_last_login();

        let profile = user.to_profile();
        assert!(profile.last_login.is_some());
    }

    #[test]
    fn test_user_multiple_last_login_updates() {
        let mut user = User::new(
            "test@example.com".to_string(),
            "hash".to_string(),
            None,
        );

        user.update_last_login();
        let first_login = user.last_login;

        // Small delay to ensure different timestamp
        std::thread::sleep(std::time::Duration::from_millis(10));

        user.update_last_login();
        let second_login = user.last_login;

        assert!(first_login.is_some());
        assert!(second_login.is_some());
        assert!(second_login.unwrap() > first_login.unwrap());
    }

    #[test]
    fn test_user_with_id_serialization() {
        let mut user = User::new(
            "test@example.com".to_string(),
            "hash".to_string(),
            None,
        );
        user.id = Some(ObjectId::new());

        let json = serde_json::to_string(&user).unwrap();
        assert!(json.contains("_id"));
    }

    #[test]
    fn test_user_without_id_serialization() {
        let user = User::new(
            "test@example.com".to_string(),
            "hash".to_string(),
            None,
        );

        let json = serde_json::to_string(&user).unwrap();
        // When id is None, it should be skipped in serialization
        assert!(!json.contains("_id"));
    }

    #[test]
    fn test_register_request_with_empty_full_name() {
        let request = RegisterRequest {
            email: "test@example.com".to_string(),
            password: "password123".to_string(),
            full_name: Some("".to_string()),
        };

        assert!(request.validate().is_ok());
        assert_eq!(request.full_name, Some("".to_string()));
    }

    #[test]
    fn test_risk_level_low_serialization() {
        let risk = RiskLevel::Low;
        let json = serde_json::to_string(&risk).unwrap();
        let deserialized: RiskLevel = serde_json::from_str(&json).unwrap();
        assert!(matches!(deserialized, RiskLevel::Low));
    }

    #[test]
    fn test_user_inactive_state() {
        let mut user = User::new(
            "test@example.com".to_string(),
            "hash".to_string(),
            None,
        );
        user.is_active = false;

        let profile = user.to_profile();
        assert!(!profile.is_active);
    }

    #[test]
    fn test_user_admin_state() {
        let mut user = User::new(
            "test@example.com".to_string(),
            "hash".to_string(),
            None,
        );
        user.is_admin = true;

        let profile = user.to_profile();
        assert!(profile.is_admin);
    }
}
