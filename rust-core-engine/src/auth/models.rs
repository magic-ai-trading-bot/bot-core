use bson::oid::ObjectId;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use validator::Validate;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct User {
    #[serde(rename = "_id", skip_serializing_if = "Option::is_none")]
    pub id: Option<ObjectId>,
    pub email: String,
    pub password_hash: String,
    pub full_name: Option<String>,
    pub is_active: bool,
    pub is_admin: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
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
    pub created_at: DateTime<Utc>,
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