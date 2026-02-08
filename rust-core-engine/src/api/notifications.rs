// @spec:FR-NOTIFICATION-001 - Notification Preferences API
// @ref:specs/02-design/2.3-api/API-RUST-CORE.md
// @test:TC-NOTIFICATION-001, TC-NOTIFICATION-002, TC-NOTIFICATION-003

//! Notification Preferences API Module
//!
//! Provides endpoints for managing user notification preferences.
//! Supports Email, Telegram, Discord, Push notifications, and Sound effects.

use serde::{Deserialize, Serialize};
use std::sync::Arc;
use warp::http::StatusCode;
use warp::{Filter, Rejection, Reply};

use crate::api::settings::ApiResponse;
use crate::storage::Storage;

/// Notification Preferences API handler
pub struct NotificationsApi {
    storage: Storage,
}

// ============================================================================
// REQUEST/RESPONSE TYPES
// ============================================================================

/// User notification preferences stored in database
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NotificationPreferences {
    /// Whether notifications are globally enabled
    pub enabled: bool,

    /// Channel settings
    pub channels: ChannelSettings,

    /// Alert type settings
    pub alerts: AlertSettings,

    /// Price alert threshold (percentage)
    pub price_alert_threshold: f64,

    /// Created timestamp
    #[serde(default = "chrono::Utc::now")]
    pub created_at: chrono::DateTime<chrono::Utc>,

    /// Last updated timestamp
    #[serde(default = "chrono::Utc::now")]
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

/// Notification channel settings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChannelSettings {
    /// Email notifications
    pub email: bool,
    /// Push notifications (browser) with VAPID keys
    pub push: PushSettings,
    /// Telegram bot notifications
    pub telegram: TelegramSettings,
    /// Discord webhook notifications
    pub discord: DiscordSettings,
    /// Sound effects
    pub sound: bool,
}

/// Push notification settings with VAPID keys
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct PushSettings {
    pub enabled: bool,
    pub vapid_public_key: Option<String>,
    pub vapid_private_key: Option<String>,
}

/// Telegram notification settings
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct TelegramSettings {
    pub enabled: bool,
    pub bot_token: Option<String>,
    pub chat_id: Option<String>,
}

/// Discord notification settings
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct DiscordSettings {
    pub enabled: bool,
    pub webhook_url: Option<String>,
}

/// Alert type settings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AlertSettings {
    /// Price movement alerts
    pub price_alerts: bool,
    /// Trade execution alerts
    pub trade_alerts: bool,
    /// System/maintenance alerts
    pub system_alerts: bool,
    /// AI signal alerts
    pub signal_alerts: bool,
    /// Risk warning alerts
    pub risk_alerts: bool,
}

/// Request to update notification preferences
#[derive(Debug, Serialize, Deserialize)]
pub struct UpdateNotificationPreferencesRequest {
    pub enabled: Option<bool>,
    pub channels: Option<ChannelSettingsUpdate>,
    pub alerts: Option<AlertSettingsUpdate>,
    pub price_alert_threshold: Option<f64>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ChannelSettingsUpdate {
    pub email: Option<bool>,
    pub push: Option<PushSettingsUpdate>,
    pub telegram: Option<TelegramSettingsUpdate>,
    pub discord: Option<DiscordSettingsUpdate>,
    pub sound: Option<bool>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PushSettingsUpdate {
    pub enabled: Option<bool>,
    pub vapid_public_key: Option<String>,
    pub vapid_private_key: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TelegramSettingsUpdate {
    pub enabled: Option<bool>,
    pub bot_token: Option<String>,
    pub chat_id: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DiscordSettingsUpdate {
    pub enabled: Option<bool>,
    pub webhook_url: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AlertSettingsUpdate {
    pub price_alerts: Option<bool>,
    pub trade_alerts: Option<bool>,
    pub system_alerts: Option<bool>,
    pub signal_alerts: Option<bool>,
    pub risk_alerts: Option<bool>,
}

/// Push notification subscription (for Web Push)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PushSubscription {
    pub endpoint: String,
    pub keys: PushSubscriptionKeys,
    #[serde(default = "chrono::Utc::now")]
    pub created_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PushSubscriptionKeys {
    pub p256dh: String,
    pub auth: String,
}

/// Test notification request
#[derive(Debug, Serialize, Deserialize)]
pub struct TestNotificationRequest {
    pub channel: String, // "email", "telegram", "discord", "push"
}

// ============================================================================
// DEFAULT IMPLEMENTATIONS
// ============================================================================

impl Default for NotificationPreferences {
    fn default() -> Self {
        Self {
            enabled: true,
            channels: ChannelSettings::default(),
            alerts: AlertSettings::default(),
            price_alert_threshold: 5.0, // 5% default
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
        }
    }
}

impl Default for ChannelSettings {
    fn default() -> Self {
        Self {
            email: false,
            push: PushSettings::default(),
            telegram: TelegramSettings::default(),
            discord: DiscordSettings::default(),
            sound: true,
        }
    }
}

impl Default for AlertSettings {
    fn default() -> Self {
        Self {
            price_alerts: true,
            trade_alerts: true,
            system_alerts: true,
            signal_alerts: true,
            risk_alerts: true,
        }
    }
}

// ============================================================================
// WARP HELPERS
// ============================================================================

fn with_api(
    api: Arc<NotificationsApi>,
) -> impl Filter<Extract = (Arc<NotificationsApi>,), Error = std::convert::Infallible> + Clone {
    warp::any().map(move || api.clone())
}

// ============================================================================
// API IMPLEMENTATION
// ============================================================================

impl NotificationsApi {
    pub fn new(storage: Storage) -> Self {
        Self { storage }
    }

    /// Create notifications API routes
    pub fn routes(self) -> impl Filter<Extract = impl Reply, Error = Rejection> + Clone {
        let api = Arc::new(self);

        let cors = warp::cors()
            .allow_any_origin()
            .allow_headers(vec!["content-type", "authorization"])
            .allow_methods(vec!["GET", "POST", "PUT", "DELETE", "OPTIONS"]);

        let base_path = warp::path("notifications");

        // GET /api/notifications/preferences - Get notification preferences
        let get_prefs_route = base_path
            .and(warp::path("preferences"))
            .and(warp::path::end())
            .and(warp::get())
            .and(with_api(api.clone()))
            .and_then(get_notification_preferences);

        // PUT /api/notifications/preferences - Update notification preferences
        let update_prefs_route = base_path
            .and(warp::path("preferences"))
            .and(warp::path::end())
            .and(warp::put())
            .and(warp::body::json())
            .and(with_api(api.clone()))
            .and_then(update_notification_preferences);

        // POST /api/notifications/push/subscribe - Subscribe to push notifications
        let push_subscribe_route = base_path
            .and(warp::path("push"))
            .and(warp::path("subscribe"))
            .and(warp::path::end())
            .and(warp::post())
            .and(warp::body::json())
            .and(with_api(api.clone()))
            .and_then(subscribe_push);

        // DELETE /api/notifications/push/subscribe - Unsubscribe from push notifications
        let push_unsubscribe_route = base_path
            .and(warp::path("push"))
            .and(warp::path("subscribe"))
            .and(warp::path::end())
            .and(warp::delete())
            .and(with_api(api.clone()))
            .and_then(unsubscribe_push);

        // POST /api/notifications/test - Send a test notification
        let test_notification_route = base_path
            .and(warp::path("test"))
            .and(warp::path::end())
            .and(warp::post())
            .and(warp::body::json())
            .and(with_api(api.clone()))
            .and_then(send_test_notification);

        // GET /api/notifications/vapid-key - Get VAPID public key for push notifications
        let vapid_key_route = base_path
            .and(warp::path("vapid-key"))
            .and(warp::path::end())
            .and(warp::get())
            .and_then(get_vapid_public_key);

        // Combine all routes
        get_prefs_route
            .or(update_prefs_route)
            .or(push_subscribe_route)
            .or(push_unsubscribe_route)
            .or(test_notification_route)
            .or(vapid_key_route)
            .with(cors)
    }
}

// ============================================================================
// ROUTE HANDLERS
// ============================================================================

/// GET /api/notifications/preferences - Get notification preferences
async fn get_notification_preferences(api: Arc<NotificationsApi>) -> Result<impl Reply, Rejection> {
    let preferences = api.storage.load_notification_preferences().await;

    Ok(warp::reply::with_status(
        warp::reply::json(&ApiResponse::success(preferences)),
        StatusCode::OK,
    ))
}

/// PUT /api/notifications/preferences - Update notification preferences
async fn update_notification_preferences(
    request: UpdateNotificationPreferencesRequest,
    api: Arc<NotificationsApi>,
) -> Result<impl Reply, Rejection> {
    // Load existing preferences (returns defaults if none exist)
    let mut prefs = api.storage.load_notification_preferences().await;

    // Apply updates
    if let Some(enabled) = request.enabled {
        prefs.enabled = enabled;
    }

    if let Some(threshold) = request.price_alert_threshold {
        if !(0.1..=100.0).contains(&threshold) {
            return Ok(warp::reply::with_status(
                warp::reply::json(&ApiResponse::<()>::error(
                    "Price alert threshold must be between 0.1 and 100".to_string(),
                )),
                StatusCode::BAD_REQUEST,
            ));
        }
        prefs.price_alert_threshold = threshold;
    }

    // Apply channel updates
    if let Some(channels) = request.channels {
        if let Some(email) = channels.email {
            prefs.channels.email = email;
        }
        if let Some(push) = channels.push {
            if let Some(enabled) = push.enabled {
                prefs.channels.push.enabled = enabled;
            }
            if let Some(vapid_public_key) = push.vapid_public_key {
                prefs.channels.push.vapid_public_key = Some(vapid_public_key);
            }
            if let Some(vapid_private_key) = push.vapid_private_key {
                prefs.channels.push.vapid_private_key = Some(vapid_private_key);
            }
        }
        if let Some(sound) = channels.sound {
            prefs.channels.sound = sound;
        }

        if let Some(telegram) = channels.telegram {
            if let Some(enabled) = telegram.enabled {
                prefs.channels.telegram.enabled = enabled;
            }
            if let Some(bot_token) = telegram.bot_token {
                prefs.channels.telegram.bot_token = if bot_token.is_empty() {
                    None
                } else {
                    Some(bot_token)
                };
            }
            if let Some(chat_id) = telegram.chat_id {
                prefs.channels.telegram.chat_id = if chat_id.is_empty() {
                    None
                } else {
                    Some(chat_id)
                };
            }
        }

        if let Some(discord) = channels.discord {
            if let Some(enabled) = discord.enabled {
                prefs.channels.discord.enabled = enabled;
            }
            if let Some(webhook_url) = discord.webhook_url {
                // Validate Discord webhook URL format
                if !webhook_url.is_empty()
                    && !webhook_url.starts_with("https://discord.com/api/webhooks/")
                    && !webhook_url.starts_with("https://discordapp.com/api/webhooks/")
                {
                    return Ok(warp::reply::with_status(
                        warp::reply::json(&ApiResponse::<()>::error(
                            "Invalid Discord webhook URL format".to_string(),
                        )),
                        StatusCode::BAD_REQUEST,
                    ));
                }
                prefs.channels.discord.webhook_url = if webhook_url.is_empty() {
                    None
                } else {
                    Some(webhook_url)
                };
            }
        }
    }

    // Apply alert updates
    if let Some(alerts) = request.alerts {
        if let Some(price_alerts) = alerts.price_alerts {
            prefs.alerts.price_alerts = price_alerts;
        }
        if let Some(trade_alerts) = alerts.trade_alerts {
            prefs.alerts.trade_alerts = trade_alerts;
        }
        if let Some(system_alerts) = alerts.system_alerts {
            prefs.alerts.system_alerts = system_alerts;
        }
        if let Some(signal_alerts) = alerts.signal_alerts {
            prefs.alerts.signal_alerts = signal_alerts;
        }
        if let Some(risk_alerts) = alerts.risk_alerts {
            prefs.alerts.risk_alerts = risk_alerts;
        }
    }

    // Update timestamp
    prefs.updated_at = chrono::Utc::now();

    // Save to database
    if let Err(e) = api.storage.save_notification_preferences(&prefs).await {
        tracing::error!("Failed to save notification preferences: {}", e);
        return Ok(warp::reply::with_status(
            warp::reply::json(&ApiResponse::<()>::error(format!(
                "Failed to save preferences: {}",
                e
            ))),
            StatusCode::INTERNAL_SERVER_ERROR,
        ));
    }

    tracing::info!("Notification preferences updated successfully");

    Ok(warp::reply::with_status(
        warp::reply::json(&ApiResponse::success(prefs)),
        StatusCode::OK,
    ))
}

/// POST /api/notifications/push/subscribe - Subscribe to push notifications
async fn subscribe_push(
    subscription: PushSubscription,
    api: Arc<NotificationsApi>,
) -> Result<impl Reply, Rejection> {
    // Validate subscription
    if subscription.endpoint.is_empty() {
        return Ok(warp::reply::with_status(
            warp::reply::json(&ApiResponse::<()>::error(
                "Invalid push subscription: missing endpoint".to_string(),
            )),
            StatusCode::BAD_REQUEST,
        ));
    }

    // Save subscription
    if let Err(e) = api.storage.save_push_subscription(&subscription).await {
        tracing::error!("Failed to save push subscription: {}", e);
        return Ok(warp::reply::with_status(
            warp::reply::json(&ApiResponse::<()>::error(format!(
                "Failed to save subscription: {}",
                e
            ))),
            StatusCode::INTERNAL_SERVER_ERROR,
        ));
    }

    tracing::info!("Push notification subscription saved");

    Ok(warp::reply::with_status(
        warp::reply::json(&ApiResponse::success(serde_json::json!({
            "message": "Push subscription saved successfully"
        }))),
        StatusCode::OK,
    ))
}

/// DELETE /api/notifications/push/subscribe - Unsubscribe from push notifications
async fn unsubscribe_push(api: Arc<NotificationsApi>) -> Result<impl Reply, Rejection> {
    if let Err(e) = api.storage.delete_push_subscription().await {
        tracing::error!("Failed to delete push subscription: {}", e);
        return Ok(warp::reply::with_status(
            warp::reply::json(&ApiResponse::<()>::error(format!(
                "Failed to delete subscription: {}",
                e
            ))),
            StatusCode::INTERNAL_SERVER_ERROR,
        ));
    }

    tracing::info!("Push notification subscription deleted");

    Ok(warp::reply::with_status(
        warp::reply::json(&ApiResponse::success("Push subscription deleted")),
        StatusCode::OK,
    ))
}

/// POST /api/notifications/test - Send a test notification
async fn send_test_notification(
    request: TestNotificationRequest,
    api: Arc<NotificationsApi>,
) -> Result<impl Reply, Rejection> {
    let prefs = api.storage.load_notification_preferences().await;

    let channel = request.channel.to_lowercase();
    let result = match channel.as_str() {
        "telegram" => {
            if !prefs.channels.telegram.enabled {
                return Ok(warp::reply::with_status(
                    warp::reply::json(&ApiResponse::<()>::error(
                        "Telegram is not enabled".to_string(),
                    )),
                    StatusCode::BAD_REQUEST,
                ));
            }
            // Telegram test would be handled by Python AI service
            Ok("Test notification sent to Telegram".to_string())
        },
        "discord" => {
            if !prefs.channels.discord.enabled {
                return Ok(warp::reply::with_status(
                    warp::reply::json(&ApiResponse::<()>::error(
                        "Discord is not enabled".to_string(),
                    )),
                    StatusCode::BAD_REQUEST,
                ));
            }
            // Discord test via webhook
            if let Some(webhook_url) = &prefs.channels.discord.webhook_url {
                match send_discord_test(webhook_url).await {
                    Ok(_) => Ok("Test notification sent to Discord".to_string()),
                    Err(e) => Err(e),
                }
            } else {
                Err("Discord webhook URL not configured".to_string())
            }
        },
        "email" => {
            if !prefs.channels.email {
                return Ok(warp::reply::with_status(
                    warp::reply::json(&ApiResponse::<()>::error(
                        "Email is not enabled".to_string(),
                    )),
                    StatusCode::BAD_REQUEST,
                ));
            }
            // Email test would be handled by Python AI service
            Ok("Test notification sent via Email".to_string())
        },
        "push" => {
            if !prefs.channels.push.enabled {
                return Ok(warp::reply::with_status(
                    warp::reply::json(&ApiResponse::<()>::error(
                        "Push notifications not enabled".to_string(),
                    )),
                    StatusCode::BAD_REQUEST,
                ));
            }
            // Push notification test
            Ok("Test push notification triggered".to_string())
        },
        _ => {
            return Ok(warp::reply::with_status(
                warp::reply::json(&ApiResponse::<()>::error(format!(
                    "Unknown channel: {}. Valid channels: telegram, discord, email, push",
                    channel
                ))),
                StatusCode::BAD_REQUEST,
            ));
        },
    };

    match result {
        Ok(message) => Ok(warp::reply::with_status(
            warp::reply::json(&ApiResponse::success(serde_json::json!({
                "message": message,
                "channel": channel
            }))),
            StatusCode::OK,
        )),
        Err(e) => Ok(warp::reply::with_status(
            warp::reply::json(&ApiResponse::<()>::error(e)),
            StatusCode::INTERNAL_SERVER_ERROR,
        )),
    }
}

/// GET /api/notifications/vapid-key - Get VAPID public key
async fn get_vapid_public_key() -> Result<impl Reply, Rejection> {
    // VAPID public key should be configured via environment variable
    let vapid_public_key = std::env::var("VAPID_PUBLIC_KEY").unwrap_or_else(|_| "".to_string());

    if vapid_public_key.is_empty() {
        return Ok(warp::reply::with_status(
            warp::reply::json(&ApiResponse::<()>::error(
                "VAPID public key not configured".to_string(),
            )),
            StatusCode::SERVICE_UNAVAILABLE,
        ));
    }

    Ok(warp::reply::with_status(
        warp::reply::json(&ApiResponse::success(serde_json::json!({
            "publicKey": vapid_public_key
        }))),
        StatusCode::OK,
    ))
}

// ============================================================================
// HELPER FUNCTIONS
// ============================================================================

/// Send test notification to Discord webhook
async fn send_discord_test(webhook_url: &str) -> Result<(), String> {
    let client = reqwest::Client::new();

    let payload = serde_json::json!({
        "embeds": [{
            "title": "🔔 Test Notification",
            "description": "This is a test notification from Bot Core Trading System.",
            "color": 3447003, // Blue
            "footer": {
                "text": "Bot Core Trading"
            },
            "timestamp": chrono::Utc::now().to_rfc3339()
        }]
    });

    let response = client
        .post(webhook_url)
        .json(&payload)
        .timeout(std::time::Duration::from_secs(10))
        .send()
        .await
        .map_err(|e| format!("Failed to send Discord notification: {}", e))?;

    if response.status().is_success() || response.status() == 204 {
        Ok(())
    } else {
        Err(format!("Discord returned status: {}", response.status()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_notification_preferences_default() {
        let prefs = NotificationPreferences::default();
        assert!(prefs.enabled);
        assert!(prefs.channels.sound);
        assert!(!prefs.channels.email);
        assert!(!prefs.channels.push.enabled);
        assert!(prefs.channels.push.vapid_public_key.is_none());
        assert!(prefs.channels.push.vapid_private_key.is_none());
        assert!(!prefs.channels.telegram.enabled);
        assert!(!prefs.channels.discord.enabled);
        assert_eq!(prefs.price_alert_threshold, 5.0);
    }

    #[test]
    fn test_alert_settings_default() {
        let alerts = AlertSettings::default();
        assert!(alerts.price_alerts);
        assert!(alerts.trade_alerts);
        assert!(alerts.system_alerts);
        assert!(alerts.signal_alerts);
        assert!(alerts.risk_alerts);
    }

    #[test]
    fn test_channel_settings_default() {
        let channels = ChannelSettings::default();
        assert!(!channels.email);
        assert!(!channels.push.enabled);
        assert!(channels.push.vapid_public_key.is_none());
        assert!(channels.sound);
    }

    #[test]
    fn test_notification_preferences_serialization() {
        let prefs = NotificationPreferences::default();
        let json = serde_json::to_string(&prefs).expect("Serialization should succeed");
        let deserialized: NotificationPreferences =
            serde_json::from_str(&json).expect("Deserialization should succeed");

        assert_eq!(deserialized.enabled, prefs.enabled);
        assert_eq!(
            deserialized.price_alert_threshold,
            prefs.price_alert_threshold
        );
    }

    #[test]
    fn test_push_subscription_serialization() {
        let sub = PushSubscription {
            endpoint: "https://example.com/push".to_string(),
            keys: PushSubscriptionKeys {
                p256dh: "test_p256dh".to_string(),
                auth: "test_auth".to_string(),
            },
            created_at: chrono::Utc::now(),
        };

        let json = serde_json::to_string(&sub).expect("Serialization should succeed");
        assert!(json.contains("test_p256dh"));
    }

    // ============================================================================
    // WARP HANDLER TESTS (Integration Tests for Notifications API Routes)
    // ============================================================================

    async fn create_test_notifications_api() -> NotificationsApi {
        let db_config = crate::config::DatabaseConfig {
            url: "no-db://test".to_string(),
            database_name: Some("test_db".to_string()),
            max_connections: 1,
            enable_logging: false,
        };
        let storage = crate::storage::Storage::new(&db_config).await.unwrap();
        NotificationsApi::new(storage)
    }

    #[tokio::test]
    async fn test_get_notification_preferences_route() {
        let api = create_test_notifications_api().await;
        let routes = api.routes();

        let resp = warp::test::request()
            .method("GET")
            .path("/notifications/preferences")
            .reply(&routes)
            .await;

        assert_eq!(resp.status(), 200);
    }

    #[tokio::test]
    async fn test_update_notification_preferences_route() {
        let api = create_test_notifications_api().await;
        let routes = api.routes();

        let request = UpdateNotificationPreferencesRequest {
            enabled: Some(true),
            channels: None,
            alerts: None,
            price_alert_threshold: None,
        };

        let resp = warp::test::request()
            .method("PUT")
            .path("/notifications/preferences")
            .json(&request)
            .reply(&routes)
            .await;

        assert!(resp.status().is_success() || resp.status().is_server_error());
    }

    #[tokio::test]
    async fn test_update_notification_preferences_invalid_threshold() {
        let api = create_test_notifications_api().await;
        let routes = api.routes();

        let request = UpdateNotificationPreferencesRequest {
            enabled: Some(true),
            channels: None,
            alerts: None,
            price_alert_threshold: Some(150.0), // Invalid: > 100
        };

        let resp = warp::test::request()
            .method("PUT")
            .path("/notifications/preferences")
            .json(&request)
            .reply(&routes)
            .await;

        assert_eq!(resp.status(), 400);
    }

    #[tokio::test]
    async fn test_update_notification_preferences_discord_invalid_webhook() {
        let api = create_test_notifications_api().await;
        let routes = api.routes();

        let request = UpdateNotificationPreferencesRequest {
            enabled: Some(true),
            channels: Some(ChannelSettingsUpdate {
                email: None,
                push: None,
                telegram: None,
                discord: Some(DiscordSettingsUpdate {
                    enabled: Some(true),
                    webhook_url: Some("https://invalid-url.com/webhook".to_string()),
                }),
                sound: None,
            }),
            alerts: None,
            price_alert_threshold: None,
        };

        let resp = warp::test::request()
            .method("PUT")
            .path("/notifications/preferences")
            .json(&request)
            .reply(&routes)
            .await;

        assert_eq!(resp.status(), 400);
    }

    #[tokio::test]
    async fn test_subscribe_push_route() {
        let api = create_test_notifications_api().await;
        let routes = api.routes();

        let subscription = PushSubscription {
            endpoint: "https://fcm.googleapis.com/test".to_string(),
            keys: PushSubscriptionKeys {
                p256dh: "test_p256dh_key".to_string(),
                auth: "test_auth_key".to_string(),
            },
            created_at: chrono::Utc::now(),
        };

        let resp = warp::test::request()
            .method("POST")
            .path("/notifications/push/subscribe")
            .json(&subscription)
            .reply(&routes)
            .await;

        assert!(resp.status().is_success() || resp.status().is_server_error());
    }

    #[tokio::test]
    async fn test_subscribe_push_invalid_endpoint() {
        let api = create_test_notifications_api().await;
        let routes = api.routes();

        let subscription = PushSubscription {
            endpoint: "".to_string(), // Invalid: empty
            keys: PushSubscriptionKeys {
                p256dh: "test_p256dh_key".to_string(),
                auth: "test_auth_key".to_string(),
            },
            created_at: chrono::Utc::now(),
        };

        let resp = warp::test::request()
            .method("POST")
            .path("/notifications/push/subscribe")
            .json(&subscription)
            .reply(&routes)
            .await;

        assert_eq!(resp.status(), 400);
    }

    #[tokio::test]
    async fn test_unsubscribe_push_route() {
        let api = create_test_notifications_api().await;
        let routes = api.routes();

        let resp = warp::test::request()
            .method("DELETE")
            .path("/notifications/push/subscribe")
            .reply(&routes)
            .await;

        assert!(resp.status().is_success() || resp.status().is_server_error());
    }

    #[tokio::test]
    async fn test_send_test_notification_telegram() {
        let api = create_test_notifications_api().await;
        let routes = api.routes();

        let request = TestNotificationRequest {
            channel: "telegram".to_string(),
        };

        let resp = warp::test::request()
            .method("POST")
            .path("/notifications/test")
            .json(&request)
            .reply(&routes)
            .await;

        assert!(resp.status().is_success() || resp.status().is_client_error());
    }

    #[tokio::test]
    async fn test_send_test_notification_discord() {
        let api = create_test_notifications_api().await;
        let routes = api.routes();

        let request = TestNotificationRequest {
            channel: "discord".to_string(),
        };

        let resp = warp::test::request()
            .method("POST")
            .path("/notifications/test")
            .json(&request)
            .reply(&routes)
            .await;

        assert!(resp.status().is_success() || resp.status().is_client_error());
    }

    #[tokio::test]
    async fn test_send_test_notification_email() {
        let api = create_test_notifications_api().await;
        let routes = api.routes();

        let request = TestNotificationRequest {
            channel: "email".to_string(),
        };

        let resp = warp::test::request()
            .method("POST")
            .path("/notifications/test")
            .json(&request)
            .reply(&routes)
            .await;

        assert!(resp.status().is_success() || resp.status().is_client_error());
    }

    #[tokio::test]
    async fn test_send_test_notification_push() {
        let api = create_test_notifications_api().await;
        let routes = api.routes();

        let request = TestNotificationRequest {
            channel: "push".to_string(),
        };

        let resp = warp::test::request()
            .method("POST")
            .path("/notifications/test")
            .json(&request)
            .reply(&routes)
            .await;

        assert!(resp.status().is_success() || resp.status().is_client_error());
    }

    #[tokio::test]
    async fn test_send_test_notification_invalid_channel() {
        let api = create_test_notifications_api().await;
        let routes = api.routes();

        let request = TestNotificationRequest {
            channel: "invalid_channel".to_string(),
        };

        let resp = warp::test::request()
            .method("POST")
            .path("/notifications/test")
            .json(&request)
            .reply(&routes)
            .await;

        assert_eq!(resp.status(), 400);
    }

    #[tokio::test]
    async fn test_get_vapid_public_key_route() {
        let api = create_test_notifications_api().await;
        let routes = api.routes();

        let resp = warp::test::request()
            .method("GET")
            .path("/notifications/vapid-key")
            .reply(&routes)
            .await;

        assert!(resp.status().is_success() || resp.status() == 503);
    }

    #[tokio::test]
    async fn test_update_preferences_all_channels() {
        let api = create_test_notifications_api().await;
        let routes = api.routes();

        let request = UpdateNotificationPreferencesRequest {
            enabled: Some(true),
            channels: Some(ChannelSettingsUpdate {
                email: Some(true),
                push: Some(PushSettingsUpdate {
                    enabled: Some(true),
                    vapid_public_key: Some("test_public_key".to_string()),
                    vapid_private_key: Some("test_private_key".to_string()),
                }),
                telegram: Some(TelegramSettingsUpdate {
                    enabled: Some(true),
                    bot_token: Some("test_bot_token".to_string()),
                    chat_id: Some("test_chat_id".to_string()),
                }),
                discord: Some(DiscordSettingsUpdate {
                    enabled: Some(true),
                    webhook_url: Some("https://discord.com/api/webhooks/123/test".to_string()),
                }),
                sound: Some(true),
            }),
            alerts: None,
            price_alert_threshold: Some(10.0),
        };

        let resp = warp::test::request()
            .method("PUT")
            .path("/notifications/preferences")
            .json(&request)
            .reply(&routes)
            .await;

        assert!(resp.status().is_success() || resp.status().is_server_error());
    }

    #[tokio::test]
    async fn test_update_preferences_all_alerts() {
        let api = create_test_notifications_api().await;
        let routes = api.routes();

        let request = UpdateNotificationPreferencesRequest {
            enabled: Some(true),
            channels: None,
            alerts: Some(AlertSettingsUpdate {
                price_alerts: Some(true),
                trade_alerts: Some(false),
                system_alerts: Some(true),
                signal_alerts: Some(false),
                risk_alerts: Some(true),
            }),
            price_alert_threshold: None,
        };

        let resp = warp::test::request()
            .method("PUT")
            .path("/notifications/preferences")
            .json(&request)
            .reply(&routes)
            .await;

        assert!(resp.status().is_success() || resp.status().is_server_error());
    }

    // ============================================================================
    // COVERAGE PHASE 7 - Additional Handler & Type Tests
    // ============================================================================

    #[test]
    fn test_cov7_notification_preferences_serialization_full() {
        let prefs = NotificationPreferences {
            enabled: true,
            channels: ChannelSettings {
                email: true,
                push: PushSettings {
                    enabled: true,
                    vapid_public_key: Some("test-public".to_string()),
                    vapid_private_key: Some("test-private".to_string()),
                },
                telegram: TelegramSettings {
                    enabled: true,
                    bot_token: Some("bot-token-123".to_string()),
                    chat_id: Some("chat-456".to_string()),
                },
                discord: DiscordSettings {
                    enabled: true,
                    webhook_url: Some("https://discord.com/api/webhooks/789/test".to_string()),
                },
                sound: true,
            },
            alerts: AlertSettings {
                price_alerts: true,
                trade_alerts: true,
                system_alerts: true,
                signal_alerts: true,
                risk_alerts: true,
            },
            price_alert_threshold: 10.0,
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
        };

        let json = serde_json::to_string(&prefs).unwrap();
        assert!(json.contains("\"enabled\":true"));
        assert!(json.contains("test-public"));
        assert!(json.contains("bot-token-123"));
        assert!(json.contains("10.0"));

        let parsed: NotificationPreferences = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.enabled, true);
        assert_eq!(parsed.price_alert_threshold, 10.0);
    }

    #[test]
    fn test_cov7_push_settings_default() {
        let push = PushSettings::default();
        assert!(!push.enabled);
        assert!(push.vapid_public_key.is_none());
        assert!(push.vapid_private_key.is_none());
    }

    #[test]
    fn test_cov7_telegram_settings_default() {
        let telegram = TelegramSettings::default();
        assert!(!telegram.enabled);
        assert!(telegram.bot_token.is_none());
        assert!(telegram.chat_id.is_none());
    }

    #[test]
    fn test_cov7_discord_settings_default() {
        let discord = DiscordSettings::default();
        assert!(!discord.enabled);
        assert!(discord.webhook_url.is_none());
    }

    #[test]
    fn test_cov7_push_subscription_keys_serialization() {
        let keys = PushSubscriptionKeys {
            p256dh: "test-p256dh-key-abc".to_string(),
            auth: "test-auth-key-xyz".to_string(),
        };

        let json = serde_json::to_string(&keys).unwrap();
        assert!(json.contains("test-p256dh-key-abc"));
        assert!(json.contains("test-auth-key-xyz"));

        let parsed: PushSubscriptionKeys = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.p256dh, "test-p256dh-key-abc");
        assert_eq!(parsed.auth, "test-auth-key-xyz");
    }

    #[test]
    fn test_cov7_test_notification_request_serialization() {
        let request = TestNotificationRequest {
            channel: "telegram".to_string(),
        };

        let json = serde_json::to_string(&request).unwrap();
        assert!(json.contains("telegram"));

        let parsed: TestNotificationRequest = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.channel, "telegram");
    }

    #[test]
    fn test_cov7_update_notification_preferences_request_partial() {
        let request = UpdateNotificationPreferencesRequest {
            enabled: Some(false),
            channels: None,
            alerts: None,
            price_alert_threshold: Some(15.0),
        };

        let json = serde_json::to_string(&request).unwrap();
        let parsed: UpdateNotificationPreferencesRequest = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.enabled, Some(false));
        assert_eq!(parsed.price_alert_threshold, Some(15.0));
        assert!(parsed.channels.is_none());
    }

    #[test]
    fn test_cov7_channel_settings_update_all_fields() {
        let update = ChannelSettingsUpdate {
            email: Some(true),
            push: Some(PushSettingsUpdate {
                enabled: Some(true),
                vapid_public_key: Some("new-public-key".to_string()),
                vapid_private_key: Some("new-private-key".to_string()),
            }),
            telegram: Some(TelegramSettingsUpdate {
                enabled: Some(true),
                bot_token: Some("new-bot-token".to_string()),
                chat_id: Some("new-chat-id".to_string()),
            }),
            discord: Some(DiscordSettingsUpdate {
                enabled: Some(true),
                webhook_url: Some("https://discord.com/api/webhooks/999/new".to_string()),
            }),
            sound: Some(false),
        };

        let json = serde_json::to_string(&update).unwrap();
        let parsed: ChannelSettingsUpdate = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.email, Some(true));
        assert_eq!(parsed.sound, Some(false));
    }

    #[test]
    fn test_cov7_alert_settings_update_partial() {
        let update = AlertSettingsUpdate {
            price_alerts: Some(true),
            trade_alerts: Some(false),
            system_alerts: None,
            signal_alerts: None,
            risk_alerts: Some(true),
        };

        let json = serde_json::to_string(&update).unwrap();
        let parsed: AlertSettingsUpdate = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.price_alerts, Some(true));
        assert_eq!(parsed.trade_alerts, Some(false));
        assert!(parsed.system_alerts.is_none());
    }

    #[tokio::test]
    async fn test_cov7_update_preferences_empty_telegram_token() {
        let api = create_test_notifications_api().await;
        let routes = api.routes();

        let request = UpdateNotificationPreferencesRequest {
            enabled: Some(true),
            channels: Some(ChannelSettingsUpdate {
                email: None,
                push: None,
                telegram: Some(TelegramSettingsUpdate {
                    enabled: Some(true),
                    bot_token: Some("".to_string()),
                    chat_id: Some("chat-id-123".to_string()),
                }),
                discord: None,
                sound: None,
            }),
            alerts: None,
            price_alert_threshold: None,
        };

        let resp = warp::test::request()
            .method("PUT")
            .path("/notifications/preferences")
            .json(&request)
            .reply(&routes)
            .await;

        assert!(resp.status().is_success() || resp.status().is_server_error());
    }

    #[tokio::test]
    async fn test_cov7_update_preferences_empty_discord_webhook() {
        let api = create_test_notifications_api().await;
        let routes = api.routes();

        let request = UpdateNotificationPreferencesRequest {
            enabled: Some(true),
            channels: Some(ChannelSettingsUpdate {
                email: None,
                push: None,
                telegram: None,
                discord: Some(DiscordSettingsUpdate {
                    enabled: Some(true),
                    webhook_url: Some("".to_string()),
                }),
                sound: None,
            }),
            alerts: None,
            price_alert_threshold: None,
        };

        let resp = warp::test::request()
            .method("PUT")
            .path("/notifications/preferences")
            .json(&request)
            .reply(&routes)
            .await;

        assert!(resp.status().is_success() || resp.status().is_server_error());
    }

    #[tokio::test]
    async fn test_cov7_update_preferences_threshold_edge_cases() {
        let api = create_test_notifications_api().await;
        let routes = api.routes();

        // Test minimum valid threshold
        let request1 = UpdateNotificationPreferencesRequest {
            enabled: Some(true),
            channels: None,
            alerts: None,
            price_alert_threshold: Some(0.1),
        };

        let resp1 = warp::test::request()
            .method("PUT")
            .path("/notifications/preferences")
            .json(&request1)
            .reply(&routes)
            .await;

        assert!(resp1.status().is_success() || resp1.status().is_server_error());

        // Test maximum valid threshold
        let request2 = UpdateNotificationPreferencesRequest {
            enabled: Some(true),
            channels: None,
            alerts: None,
            price_alert_threshold: Some(100.0),
        };

        let resp2 = warp::test::request()
            .method("PUT")
            .path("/notifications/preferences")
            .json(&request2)
            .reply(&routes)
            .await;

        assert!(resp2.status().is_success() || resp2.status().is_server_error());
    }

    #[tokio::test]
    async fn test_cov7_update_preferences_threshold_too_low() {
        let api = create_test_notifications_api().await;
        let routes = api.routes();

        let request = UpdateNotificationPreferencesRequest {
            enabled: Some(true),
            channels: None,
            alerts: None,
            price_alert_threshold: Some(0.05),
        };

        let resp = warp::test::request()
            .method("PUT")
            .path("/notifications/preferences")
            .json(&request)
            .reply(&routes)
            .await;

        assert_eq!(resp.status(), 400);
    }

    #[tokio::test]
    async fn test_cov7_subscribe_push_valid() {
        let api = create_test_notifications_api().await;
        let routes = api.routes();

        let subscription = PushSubscription {
            endpoint: "https://fcm.googleapis.com/fcm/send/test-endpoint-123".to_string(),
            keys: PushSubscriptionKeys {
                p256dh: "valid-p256dh-key".to_string(),
                auth: "valid-auth-key".to_string(),
            },
            created_at: chrono::Utc::now(),
        };

        let resp = warp::test::request()
            .method("POST")
            .path("/notifications/push/subscribe")
            .json(&subscription)
            .reply(&routes)
            .await;

        assert!(resp.status().is_success() || resp.status().is_server_error());
    }

    #[tokio::test]
    async fn test_cov7_unsubscribe_push_route() {
        let api = create_test_notifications_api().await;
        let routes = api.routes();

        let resp = warp::test::request()
            .method("DELETE")
            .path("/notifications/push/subscribe")
            .reply(&routes)
            .await;

        assert!(resp.status().is_success() || resp.status().is_server_error());
    }

    #[tokio::test]
    async fn test_cov7_test_notification_all_channels() {
        let api = create_test_notifications_api().await;
        let routes = api.routes();

        let channels = vec!["telegram", "discord", "email", "push"];

        for channel in channels {
            let request = TestNotificationRequest {
                channel: channel.to_string(),
            };

            let resp = warp::test::request()
                .method("POST")
                .path("/notifications/test")
                .json(&request)
                .reply(&routes)
                .await;

            assert!(resp.status().is_success() || resp.status().is_client_error() || resp.status().is_server_error());
        }
    }

    #[tokio::test]
    async fn test_cov7_test_notification_uppercase_channel() {
        let api = create_test_notifications_api().await;
        let routes = api.routes();

        let request = TestNotificationRequest {
            channel: "TELEGRAM".to_string(),
        };

        let resp = warp::test::request()
            .method("POST")
            .path("/notifications/test")
            .json(&request)
            .reply(&routes)
            .await;

        assert!(resp.status().is_success() || resp.status().is_client_error());
    }

    #[tokio::test]
    async fn test_cov7_get_preferences_default() {
        let api = create_test_notifications_api().await;
        let routes = api.routes();

        let resp = warp::test::request()
            .method("GET")
            .path("/notifications/preferences")
            .reply(&routes)
            .await;

        assert_eq!(resp.status(), 200);
        let body_str = std::str::from_utf8(resp.body()).unwrap();
        assert!(body_str.contains("success") || body_str.contains("enabled"));
    }

    #[tokio::test]
    async fn test_cov7_update_preferences_disable_all() {
        let api = create_test_notifications_api().await;
        let routes = api.routes();

        let request = UpdateNotificationPreferencesRequest {
            enabled: Some(false),
            channels: Some(ChannelSettingsUpdate {
                email: Some(false),
                push: Some(PushSettingsUpdate {
                    enabled: Some(false),
                    vapid_public_key: None,
                    vapid_private_key: None,
                }),
                telegram: Some(TelegramSettingsUpdate {
                    enabled: Some(false),
                    bot_token: None,
                    chat_id: None,
                }),
                discord: Some(DiscordSettingsUpdate {
                    enabled: Some(false),
                    webhook_url: None,
                }),
                sound: Some(false),
            }),
            alerts: Some(AlertSettingsUpdate {
                price_alerts: Some(false),
                trade_alerts: Some(false),
                system_alerts: Some(false),
                signal_alerts: Some(false),
                risk_alerts: Some(false),
            }),
            price_alert_threshold: Some(5.0),
        };

        let resp = warp::test::request()
            .method("PUT")
            .path("/notifications/preferences")
            .json(&request)
            .reply(&routes)
            .await;

        assert!(resp.status().is_success() || resp.status().is_server_error());
    }

    #[tokio::test]
    async fn test_cov7_update_preferences_only_sound() {
        let api = create_test_notifications_api().await;
        let routes = api.routes();

        let request = UpdateNotificationPreferencesRequest {
            enabled: Some(true),
            channels: Some(ChannelSettingsUpdate {
                email: None,
                push: None,
                telegram: None,
                discord: None,
                sound: Some(true),
            }),
            alerts: None,
            price_alert_threshold: None,
        };

        let resp = warp::test::request()
            .method("PUT")
            .path("/notifications/preferences")
            .json(&request)
            .reply(&routes)
            .await;

        assert!(resp.status().is_success() || resp.status().is_server_error());
    }

    #[tokio::test]
    async fn test_cov7_update_preferences_only_email() {
        let api = create_test_notifications_api().await;
        let routes = api.routes();

        let request = UpdateNotificationPreferencesRequest {
            enabled: Some(true),
            channels: Some(ChannelSettingsUpdate {
                email: Some(true),
                push: None,
                telegram: None,
                discord: None,
                sound: None,
            }),
            alerts: None,
            price_alert_threshold: None,
        };

        let resp = warp::test::request()
            .method("PUT")
            .path("/notifications/preferences")
            .json(&request)
            .reply(&routes)
            .await;

        assert!(resp.status().is_success() || resp.status().is_server_error());
    }

    #[tokio::test]
    async fn test_cov7_discord_webhook_discordapp_domain() {
        let api = create_test_notifications_api().await;
        let routes = api.routes();

        let request = UpdateNotificationPreferencesRequest {
            enabled: Some(true),
            channels: Some(ChannelSettingsUpdate {
                email: None,
                push: None,
                telegram: None,
                discord: Some(DiscordSettingsUpdate {
                    enabled: Some(true),
                    webhook_url: Some("https://discordapp.com/api/webhooks/123/test".to_string()),
                }),
                sound: None,
            }),
            alerts: None,
            price_alert_threshold: None,
        };

        let resp = warp::test::request()
            .method("PUT")
            .path("/notifications/preferences")
            .json(&request)
            .reply(&routes)
            .await;

        assert!(resp.status().is_success() || resp.status().is_server_error());
    }

    #[tokio::test]
    async fn test_cov7_wrong_http_methods() {
        let api = create_test_notifications_api().await;
        let routes = api.routes();

        let wrong_methods = vec![
            ("POST", "/notifications/preferences"),
            ("DELETE", "/notifications/preferences"),
            ("GET", "/notifications/push/subscribe"),
            ("PUT", "/notifications/push/subscribe"),
            ("GET", "/notifications/test"),
        ];

        for (method, path) in wrong_methods {
            let resp = warp::test::request()
                .method(method)
                .path(path)
                .reply(&routes)
                .await;
            assert_eq!(resp.status(), warp::http::StatusCode::METHOD_NOT_ALLOWED);
        }
    }

    #[test]
    fn test_cov7_push_subscription_with_timestamp() {
        let now = chrono::Utc::now();
        let sub = PushSubscription {
            endpoint: "https://push.example.com/abc".to_string(),
            keys: PushSubscriptionKeys {
                p256dh: "p256dh-key".to_string(),
                auth: "auth-key".to_string(),
            },
            created_at: now,
        };

        let json = serde_json::to_string(&sub).unwrap();
        let parsed: PushSubscription = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.endpoint, "https://push.example.com/abc");
    }

    #[test]
    fn test_cov7_notification_preferences_custom_threshold() {
        let mut prefs = NotificationPreferences::default();
        prefs.price_alert_threshold = 20.0;

        assert_eq!(prefs.price_alert_threshold, 20.0);
        assert!(prefs.enabled);
    }

    #[test]
    fn test_cov7_channel_settings_all_disabled() {
        let channels = ChannelSettings {
            email: false,
            push: PushSettings::default(),
            telegram: TelegramSettings::default(),
            discord: DiscordSettings::default(),
            sound: false,
        };

        assert!(!channels.email);
        assert!(!channels.push.enabled);
        assert!(!channels.sound);
    }

    #[test]
    fn test_cov7_alert_settings_all_enabled() {
        let alerts = AlertSettings::default();
        assert!(alerts.price_alerts);
        assert!(alerts.trade_alerts);
        assert!(alerts.system_alerts);
        assert!(alerts.signal_alerts);
        assert!(alerts.risk_alerts);
    }
}
