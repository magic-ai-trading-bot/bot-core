// Comprehensive tests for API notifications and settings modules
// Target: api/notifications.rs, api/settings.rs
// Focus: Request/response types, serialization, validation

mod common;

use binance_trading_bot::api::notifications::*;
use binance_trading_bot::api::settings::{ApiKeyPermissions, ApiResponse, SaveApiKeysRequest};
use chrono::Utc;
use serde_json;

// ========== NOTIFICATIONS API TESTS ==========

#[test]
fn test_notification_preferences_default() {
    let prefs = NotificationPreferences {
        enabled: true,
        channels: ChannelSettings {
            email: true,
            push: PushSettings {
                enabled: false,
                vapid_public_key: None,
                vapid_private_key: None,
            },
            telegram: TelegramSettings {
                enabled: false,
                bot_token: None,
                chat_id: None,
            },
            discord: DiscordSettings {
                enabled: false,
                webhook_url: None,
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
        price_alert_threshold: 2.0,
        created_at: Utc::now(),
        updated_at: Utc::now(),
    };

    assert!(prefs.enabled);
    assert!(prefs.channels.email);
    assert!(!prefs.channels.push.enabled);
    assert_eq!(prefs.price_alert_threshold, 2.0);
}

#[test]
fn test_notification_preferences_serialization() {
    let prefs = NotificationPreferences {
        enabled: true,
        channels: ChannelSettings {
            email: true,
            push: PushSettings {
                enabled: true,
                vapid_public_key: Some("pub_key_123".to_string()),
                vapid_private_key: Some("priv_key_456".to_string()),
            },
            telegram: TelegramSettings {
                enabled: false,
                bot_token: None,
                chat_id: None,
            },
            discord: DiscordSettings {
                enabled: false,
                webhook_url: None,
            },
            sound: false,
        },
        alerts: AlertSettings {
            price_alerts: true,
            trade_alerts: false,
            system_alerts: true,
            signal_alerts: false,
            risk_alerts: true,
        },
        price_alert_threshold: 5.0,
        created_at: Utc::now(),
        updated_at: Utc::now(),
    };

    let json = serde_json::to_string(&prefs).unwrap();
    let deserialized: NotificationPreferences = serde_json::from_str(&json).unwrap();

    assert_eq!(deserialized.enabled, prefs.enabled);
    assert_eq!(deserialized.price_alert_threshold, 5.0);
    assert!(deserialized.channels.push.enabled);
    assert!(!deserialized.channels.telegram.enabled);
}

#[test]
fn test_channel_settings_all_enabled() {
    let channels = ChannelSettings {
        email: true,
        push: PushSettings {
            enabled: true,
            vapid_public_key: Some("vapid_pub".to_string()),
            vapid_private_key: Some("vapid_priv".to_string()),
        },
        telegram: TelegramSettings {
            enabled: true,
            bot_token: Some("telegram_token".to_string()),
            chat_id: Some("12345".to_string()),
        },
        discord: DiscordSettings {
            enabled: true,
            webhook_url: Some("https://discord.com/webhook/123".to_string()),
        },
        sound: true,
    };

    assert!(channels.email);
    assert!(channels.push.enabled);
    assert!(channels.telegram.enabled);
    assert!(channels.discord.enabled);
    assert!(channels.sound);
}

#[test]
fn test_channel_settings_all_disabled() {
    let channels = ChannelSettings {
        email: false,
        push: PushSettings::default(),
        telegram: TelegramSettings::default(),
        discord: DiscordSettings::default(),
        sound: false,
    };

    assert!(!channels.email);
    assert!(!channels.push.enabled);
    assert!(!channels.telegram.enabled);
    assert!(!channels.discord.enabled);
    assert!(!channels.sound);
}

#[test]
fn test_push_settings_with_keys() {
    let push = PushSettings {
        enabled: true,
        vapid_public_key: Some("BNmT5...public_key".to_string()),
        vapid_private_key: Some("Qh3...private_key".to_string()),
    };

    assert!(push.enabled);
    assert!(push.vapid_public_key.is_some());
    assert!(push.vapid_private_key.is_some());
}

#[test]
fn test_push_settings_default() {
    let push = PushSettings::default();

    assert!(!push.enabled);
    assert!(push.vapid_public_key.is_none());
    assert!(push.vapid_private_key.is_none());
}

#[test]
fn test_telegram_settings_with_credentials() {
    let telegram = TelegramSettings {
        enabled: true,
        bot_token: Some("1234567890:ABCdefGHIjklMNOpqrsTUVwxyz".to_string()),
        chat_id: Some("-1001234567890".to_string()),
    };

    assert!(telegram.enabled);
    assert!(telegram.bot_token.is_some());
    assert!(telegram.chat_id.is_some());
}

#[test]
fn test_telegram_settings_default() {
    let telegram = TelegramSettings::default();

    assert!(!telegram.enabled);
    assert!(telegram.bot_token.is_none());
    assert!(telegram.chat_id.is_none());
}

#[test]
fn test_discord_settings_with_webhook() {
    let discord = DiscordSettings {
        enabled: true,
        webhook_url: Some("https://discord.com/api/webhooks/123456/token".to_string()),
    };

    assert!(discord.enabled);
    assert!(discord.webhook_url.is_some());
}

#[test]
fn test_discord_settings_default() {
    let discord = DiscordSettings::default();

    assert!(!discord.enabled);
    assert!(discord.webhook_url.is_none());
}

#[test]
fn test_alert_settings_all_enabled() {
    let alerts = AlertSettings {
        price_alerts: true,
        trade_alerts: true,
        system_alerts: true,
        signal_alerts: true,
        risk_alerts: true,
    };

    assert!(alerts.price_alerts);
    assert!(alerts.trade_alerts);
    assert!(alerts.system_alerts);
    assert!(alerts.signal_alerts);
    assert!(alerts.risk_alerts);
}

#[test]
fn test_alert_settings_selective() {
    let alerts = AlertSettings {
        price_alerts: true,
        trade_alerts: true,
        system_alerts: false,
        signal_alerts: false,
        risk_alerts: true,
    };

    assert!(alerts.price_alerts);
    assert!(alerts.trade_alerts);
    assert!(!alerts.system_alerts);
    assert!(!alerts.signal_alerts);
    assert!(alerts.risk_alerts);
}

#[test]
fn test_notification_preferences_price_threshold() {
    let mut prefs = NotificationPreferences {
        enabled: true,
        channels: ChannelSettings {
            email: true,
            push: PushSettings::default(),
            telegram: TelegramSettings::default(),
            discord: DiscordSettings::default(),
            sound: false,
        },
        alerts: AlertSettings {
            price_alerts: true,
            trade_alerts: true,
            system_alerts: true,
            signal_alerts: true,
            risk_alerts: true,
        },
        price_alert_threshold: 1.0,
        created_at: Utc::now(),
        updated_at: Utc::now(),
    };

    assert_eq!(prefs.price_alert_threshold, 1.0);

    // Update threshold
    prefs.price_alert_threshold = 5.0;
    assert_eq!(prefs.price_alert_threshold, 5.0);
}

// ========== SETTINGS API TESTS ==========

#[test]
fn test_api_response_success() {
    let data = "test_data".to_string();
    let response: ApiResponse<String> = ApiResponse::success(data.clone());

    assert!(response.success);
    assert_eq!(response.data, Some(data));
    assert!(response.error.is_none());
    assert!(response.timestamp <= Utc::now());
}

#[test]
fn test_api_response_error() {
    let error_msg = "An error occurred".to_string();
    let response: ApiResponse<String> = ApiResponse::error(error_msg.clone());

    assert!(!response.success);
    assert!(response.data.is_none());
    assert_eq!(response.error, Some(error_msg));
}

#[test]
fn test_save_api_keys_request() {
    let request = SaveApiKeysRequest {
        api_key: "test_api_key".to_string(),
        api_secret: "test_api_secret".to_string(),
        use_testnet: true,
        permissions: ApiKeyPermissions {
            spot_trading: true,
            futures_trading: true,
            margin_trading: false,
            options_trading: false,
        },
    };

    assert_eq!(request.api_key, "test_api_key");
    assert_eq!(request.api_secret, "test_api_secret");
    assert!(request.use_testnet);
    assert!(request.permissions.spot_trading);
    assert!(request.permissions.futures_trading);
}

#[test]
fn test_save_api_keys_request_serialization() {
    let request = SaveApiKeysRequest {
        api_key: "key123".to_string(),
        api_secret: "secret456".to_string(),
        use_testnet: false,
        permissions: ApiKeyPermissions::default(),
    };

    let json = serde_json::to_string(&request).unwrap();
    let deserialized: SaveApiKeysRequest = serde_json::from_str(&json).unwrap();

    assert_eq!(deserialized.api_key, "key123");
    assert_eq!(deserialized.api_secret, "secret456");
    assert!(!deserialized.use_testnet);
}

#[test]
fn test_api_key_permissions_default() {
    let permissions = ApiKeyPermissions::default();

    assert!(!permissions.spot_trading);
    assert!(permissions.futures_trading); // Default for bot
    assert!(!permissions.margin_trading);
    assert!(!permissions.options_trading);
}

#[test]
fn test_api_key_permissions_custom() {
    let permissions = ApiKeyPermissions {
        spot_trading: true,
        futures_trading: true,
        margin_trading: true,
        options_trading: true,
    };

    assert!(permissions.spot_trading);
    assert!(permissions.futures_trading);
    assert!(permissions.margin_trading);
    assert!(permissions.options_trading);
}

#[test]
fn test_api_key_permissions_only_futures() {
    let permissions = ApiKeyPermissions {
        spot_trading: false,
        futures_trading: true,
        margin_trading: false,
        options_trading: false,
    };

    assert!(!permissions.spot_trading);
    assert!(permissions.futures_trading);
    assert!(!permissions.margin_trading);
    assert!(!permissions.options_trading);
}

#[test]
fn test_notification_preferences_update_timestamps() {
    let now = Utc::now();
    let prefs = NotificationPreferences {
        enabled: true,
        channels: ChannelSettings {
            email: true,
            push: PushSettings::default(),
            telegram: TelegramSettings::default(),
            discord: DiscordSettings::default(),
            sound: true,
        },
        alerts: AlertSettings {
            price_alerts: true,
            trade_alerts: true,
            system_alerts: true,
            signal_alerts: true,
            risk_alerts: true,
        },
        price_alert_threshold: 2.0,
        created_at: now,
        updated_at: now,
    };

    assert!(prefs.created_at <= now);
    assert!(prefs.updated_at <= now);
}

#[test]
fn test_notification_preferences_disabled() {
    let prefs = NotificationPreferences {
        enabled: false,
        channels: ChannelSettings {
            email: false,
            push: PushSettings::default(),
            telegram: TelegramSettings::default(),
            discord: DiscordSettings::default(),
            sound: false,
        },
        alerts: AlertSettings {
            price_alerts: false,
            trade_alerts: false,
            system_alerts: false,
            signal_alerts: false,
            risk_alerts: false,
        },
        price_alert_threshold: 0.0,
        created_at: Utc::now(),
        updated_at: Utc::now(),
    };

    assert!(!prefs.enabled);
    assert_eq!(prefs.price_alert_threshold, 0.0);
}

#[test]
fn test_telegram_settings_serialization() {
    let telegram = TelegramSettings {
        enabled: true,
        bot_token: Some("token123".to_string()),
        chat_id: Some("chat456".to_string()),
    };

    let json = serde_json::to_string(&telegram).unwrap();
    let deserialized: TelegramSettings = serde_json::from_str(&json).unwrap();

    assert_eq!(deserialized.enabled, telegram.enabled);
    assert_eq!(deserialized.bot_token, telegram.bot_token);
    assert_eq!(deserialized.chat_id, telegram.chat_id);
}

#[test]
fn test_discord_settings_serialization() {
    let discord = DiscordSettings {
        enabled: true,
        webhook_url: Some("https://example.com/webhook".to_string()),
    };

    let json = serde_json::to_string(&discord).unwrap();
    let deserialized: DiscordSettings = serde_json::from_str(&json).unwrap();

    assert_eq!(deserialized.enabled, discord.enabled);
    assert_eq!(deserialized.webhook_url, discord.webhook_url);
}

#[test]
fn test_push_settings_serialization() {
    let push = PushSettings {
        enabled: true,
        vapid_public_key: Some("public_key".to_string()),
        vapid_private_key: Some("private_key".to_string()),
    };

    let json = serde_json::to_string(&push).unwrap();
    let deserialized: PushSettings = serde_json::from_str(&json).unwrap();

    assert_eq!(deserialized.enabled, push.enabled);
    assert_eq!(deserialized.vapid_public_key, push.vapid_public_key);
    assert_eq!(deserialized.vapid_private_key, push.vapid_private_key);
}

#[test]
fn test_alert_settings_serialization() {
    let alerts = AlertSettings {
        price_alerts: true,
        trade_alerts: false,
        system_alerts: true,
        signal_alerts: false,
        risk_alerts: true,
    };

    let json = serde_json::to_string(&alerts).unwrap();
    let deserialized: AlertSettings = serde_json::from_str(&json).unwrap();

    assert_eq!(deserialized.price_alerts, alerts.price_alerts);
    assert_eq!(deserialized.trade_alerts, alerts.trade_alerts);
    assert_eq!(deserialized.system_alerts, alerts.system_alerts);
    assert_eq!(deserialized.signal_alerts, alerts.signal_alerts);
    assert_eq!(deserialized.risk_alerts, alerts.risk_alerts);
}

#[test]
fn test_api_key_permissions_serialization() {
    let permissions = ApiKeyPermissions {
        spot_trading: true,
        futures_trading: false,
        margin_trading: true,
        options_trading: false,
    };

    let json = serde_json::to_string(&permissions).unwrap();
    let deserialized: ApiKeyPermissions = serde_json::from_str(&json).unwrap();

    assert_eq!(deserialized.spot_trading, permissions.spot_trading);
    assert_eq!(deserialized.futures_trading, permissions.futures_trading);
    assert_eq!(deserialized.margin_trading, permissions.margin_trading);
    assert_eq!(deserialized.options_trading, permissions.options_trading);
}

#[test]
fn test_save_api_keys_testnet_vs_mainnet() {
    let testnet_request = SaveApiKeysRequest {
        api_key: "testnet_key".to_string(),
        api_secret: "testnet_secret".to_string(),
        use_testnet: true,
        permissions: ApiKeyPermissions::default(),
    };

    let mainnet_request = SaveApiKeysRequest {
        api_key: "mainnet_key".to_string(),
        api_secret: "mainnet_secret".to_string(),
        use_testnet: false,
        permissions: ApiKeyPermissions::default(),
    };

    assert!(testnet_request.use_testnet);
    assert!(!mainnet_request.use_testnet);
}

#[test]
fn test_notification_preferences_multiple_channels() {
    let prefs = NotificationPreferences {
        enabled: true,
        channels: ChannelSettings {
            email: true,
            push: PushSettings {
                enabled: true,
                vapid_public_key: Some("key1".to_string()),
                vapid_private_key: Some("key2".to_string()),
            },
            telegram: TelegramSettings {
                enabled: true,
                bot_token: Some("bot_token".to_string()),
                chat_id: Some("chat_id".to_string()),
            },
            discord: DiscordSettings {
                enabled: true,
                webhook_url: Some("webhook".to_string()),
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
        price_alert_threshold: 3.0,
        created_at: Utc::now(),
        updated_at: Utc::now(),
    };

    // All channels enabled
    assert!(prefs.channels.email);
    assert!(prefs.channels.push.enabled);
    assert!(prefs.channels.telegram.enabled);
    assert!(prefs.channels.discord.enabled);
    assert!(prefs.channels.sound);
}

#[test]
fn test_api_response_with_vec_data() {
    let data = vec![1, 2, 3, 4, 5];
    let response: ApiResponse<Vec<i32>> = ApiResponse::success(data.clone());

    assert!(response.success);
    assert_eq!(response.data, Some(data));
}
