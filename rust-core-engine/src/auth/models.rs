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
            bson::Bson::DateTime(dt) => Ok(DateTime::from_timestamp_millis(dt.timestamp_millis())
                .unwrap_or_else(|| Utc::now())),
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
                DateTime::from_timestamp_millis(dt.timestamp_millis())
                    .unwrap_or_else(|| Utc::now()),
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
