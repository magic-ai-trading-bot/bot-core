// @spec:FR-SETTINGS-001 - API Settings Endpoints
// @ref:specs/02-design/2.3-api/API-RUST-CORE.md
// @test:TC-SETTINGS-001, TC-SETTINGS-002, TC-SETTINGS-003

//! Settings API Module
//!
//! Provides endpoints for managing API keys and trading settings.
//! API keys are stored encrypted in the database.

use base64::{engine::general_purpose::STANDARD as BASE64, Engine};
use ring::aead::{Aad, LessSafeKey, Nonce, UnboundKey, AES_256_GCM};
use ring::rand::{SecureRandom, SystemRandom};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::RwLock;
use warp::http::StatusCode;
use warp::{Filter, Rejection, Reply};

use crate::binance::BinanceClient;
use crate::config::BinanceConfig;
use crate::storage::Storage;

/// Encryption key for API secrets (should come from environment in production)
fn get_encryption_key() -> [u8; 32] {
    // In production, this should be loaded from environment/secrets manager
    // For now, derive from a secret phrase
    let secret = std::env::var("API_KEY_ENCRYPTION_SECRET")
        .unwrap_or_else(|_| "bot-core-default-encryption-key-32b".to_string());
    let mut key = [0u8; 32];
    let bytes = secret.as_bytes();
    for (i, byte) in bytes.iter().enumerate().take(32) {
        key[i] = *byte;
    }
    key
}

/// Settings API handler
pub struct SettingsApi {
    storage: Storage,
    binance_config: Arc<RwLock<BinanceConfig>>,
}

// ============================================================================
// REQUEST/RESPONSE TYPES
// ============================================================================

#[derive(Debug, Serialize, Deserialize)]
pub struct ApiResponse<T> {
    pub success: bool,
    pub data: Option<T>,
    pub error: Option<String>,
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

impl<T> ApiResponse<T> {
    pub fn success(data: T) -> Self {
        Self {
            success: true,
            data: Some(data),
            error: None,
            timestamp: chrono::Utc::now(),
        }
    }

    pub fn error(message: String) -> Self {
        Self {
            success: false,
            data: None,
            error: Some(message),
            timestamp: chrono::Utc::now(),
        }
    }
}

/// Request to save API keys
#[derive(Debug, Serialize, Deserialize)]
pub struct SaveApiKeysRequest {
    pub api_key: String,
    pub api_secret: String,
    pub use_testnet: bool,
    pub permissions: ApiKeyPermissions,
}

/// API key permissions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiKeyPermissions {
    pub spot_trading: bool,
    pub futures_trading: bool,
    pub margin_trading: bool,
    pub options_trading: bool,
}

impl Default for ApiKeyPermissions {
    fn default() -> Self {
        Self {
            spot_trading: false,
            futures_trading: true, // Default for this bot
            margin_trading: false,
            options_trading: false,
        }
    }
}

/// Response for API key status
#[derive(Debug, Serialize, Deserialize)]
pub struct ApiKeyStatusResponse {
    pub configured: bool,
    pub api_key_masked: Option<String>,
    pub use_testnet: bool,
    pub permissions: Option<ApiKeyPermissions>,
    pub last_updated: Option<chrono::DateTime<chrono::Utc>>,
    pub connection_status: Option<ConnectionStatus>,
}

/// Connection test status
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConnectionStatus {
    pub connected: bool,
    pub message: String,
    pub account_type: Option<String>,
    pub can_trade: Option<bool>,
    pub balances_count: Option<usize>,
}

/// Stored API key record (encrypted)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StoredApiKey {
    pub api_key: String,              // Plain text (not sensitive)
    pub api_secret_encrypted: String, // Base64 encoded encrypted secret
    pub api_secret_nonce: String,     // Base64 encoded nonce
    pub use_testnet: bool,
    pub permissions: ApiKeyPermissions,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

// ============================================================================
// ENCRYPTION HELPERS
// ============================================================================

/// Encrypt API secret using AES-256-GCM
fn encrypt_secret(secret: &str) -> Result<(String, String), String> {
    let key_bytes = get_encryption_key();
    let unbound_key = UnboundKey::new(&AES_256_GCM, &key_bytes)
        .map_err(|e| format!("Failed to create encryption key: {:?}", e))?;
    let key = LessSafeKey::new(unbound_key);

    let rng = SystemRandom::new();
    let mut nonce_bytes = [0u8; 12];
    rng.fill(&mut nonce_bytes)
        .map_err(|e| format!("Failed to generate nonce: {:?}", e))?;
    let nonce = Nonce::assume_unique_for_key(nonce_bytes);

    let mut in_out = secret.as_bytes().to_vec();
    key.seal_in_place_append_tag(nonce, Aad::empty(), &mut in_out)
        .map_err(|e| format!("Encryption failed: {:?}", e))?;

    Ok((BASE64.encode(&in_out), BASE64.encode(nonce_bytes)))
}

/// Decrypt API secret using AES-256-GCM
fn decrypt_secret(encrypted: &str, nonce_b64: &str) -> Result<String, String> {
    let key_bytes = get_encryption_key();
    let unbound_key = UnboundKey::new(&AES_256_GCM, &key_bytes)
        .map_err(|e| format!("Failed to create decryption key: {:?}", e))?;
    let key = LessSafeKey::new(unbound_key);

    let nonce_bytes = BASE64
        .decode(nonce_b64)
        .map_err(|e| format!("Invalid nonce: {:?}", e))?;
    let mut nonce_arr = [0u8; 12];
    nonce_arr.copy_from_slice(&nonce_bytes);
    let nonce = Nonce::assume_unique_for_key(nonce_arr);

    let mut encrypted_data = BASE64
        .decode(encrypted)
        .map_err(|e| format!("Invalid encrypted data: {:?}", e))?;

    let decrypted = key
        .open_in_place(nonce, Aad::empty(), &mut encrypted_data)
        .map_err(|e| format!("Decryption failed: {:?}", e))?;

    String::from_utf8(decrypted.to_vec()).map_err(|e| format!("Invalid UTF-8: {:?}", e))
}

/// Mask API key for display (show first 4 and last 4 characters)
fn mask_api_key(key: &str) -> String {
    if key.len() <= 8 {
        return "*".repeat(key.len());
    }
    let visible_chars = 4;
    let start = &key[..visible_chars];
    let end = &key[key.len() - visible_chars..];
    let masked_len = key.len() - (visible_chars * 2);
    format!("{}{}{}", start, "*".repeat(masked_len.min(20)), end)
}

// ============================================================================
// WARP HELPERS
// ============================================================================

fn with_api(
    api: Arc<SettingsApi>,
) -> impl Filter<Extract = (Arc<SettingsApi>,), Error = std::convert::Infallible> + Clone {
    warp::any().map(move || api.clone())
}

// ============================================================================
// API IMPLEMENTATION
// ============================================================================

impl SettingsApi {
    pub fn new(storage: Storage, binance_config: BinanceConfig) -> Self {
        Self {
            storage,
            binance_config: Arc::new(RwLock::new(binance_config)),
        }
    }

    /// Create settings API routes
    pub fn routes(self) -> impl Filter<Extract = impl Reply, Error = Rejection> + Clone {
        let api = Arc::new(self);

        let cors = warp::cors()
            .allow_any_origin()
            .allow_headers(vec!["content-type", "authorization"])
            .allow_methods(vec!["GET", "POST", "PUT", "DELETE", "OPTIONS"]);

        let base_path = warp::path("settings");

        // GET /api/settings/api-keys - Get API key status
        let get_keys_route = base_path
            .and(warp::path("api-keys"))
            .and(warp::path::end())
            .and(warp::get())
            .and(with_api(api.clone()))
            .and_then(get_api_key_status);

        // POST /api/settings/api-keys - Save API keys
        let save_keys_route = base_path
            .and(warp::path("api-keys"))
            .and(warp::path::end())
            .and(warp::post())
            .and(warp::body::json())
            .and(with_api(api.clone()))
            .and_then(save_api_keys);

        // DELETE /api/settings/api-keys - Delete API keys
        let delete_keys_route = base_path
            .and(warp::path("api-keys"))
            .and(warp::path::end())
            .and(warp::delete())
            .and(with_api(api.clone()))
            .and_then(delete_api_keys);

        // POST /api/settings/api-keys/test - Test connection
        let test_connection_route = base_path
            .and(warp::path("api-keys"))
            .and(warp::path("test"))
            .and(warp::path::end())
            .and(warp::post())
            .and(with_api(api.clone()))
            .and_then(test_connection);

        // Combine all routes
        get_keys_route
            .or(save_keys_route)
            .or(delete_keys_route)
            .or(test_connection_route)
            .with(cors)
    }

    /// Get stored API key from database
    pub async fn get_stored_api_key(&self) -> Option<StoredApiKey> {
        self.storage.load_api_key().await.ok().flatten()
    }

    /// Get decrypted API secret
    pub async fn get_decrypted_credentials(&self) -> Option<(String, String, bool)> {
        let stored = self.get_stored_api_key().await?;
        let secret = decrypt_secret(&stored.api_secret_encrypted, &stored.api_secret_nonce).ok()?;
        Some((stored.api_key, secret, stored.use_testnet))
    }

    /// Update BinanceConfig with stored credentials
    pub async fn apply_stored_credentials(&self) -> Result<BinanceConfig, String> {
        let (api_key, api_secret, use_testnet) = self
            .get_decrypted_credentials()
            .await
            .ok_or_else(|| "No API keys configured".to_string())?;

        let mut config = self.binance_config.write().await;
        config.api_key = api_key.clone();
        config.secret_key = api_secret.clone();
        config.testnet = use_testnet;
        config.base_url = if use_testnet {
            "https://testnet.binancefuture.com".to_string()
        } else {
            "https://fapi.binance.com".to_string()
        };

        Ok(config.clone())
    }
}

// ============================================================================
// ROUTE HANDLERS
// ============================================================================

/// GET /api/settings/api-keys - Get API key status
async fn get_api_key_status(api: Arc<SettingsApi>) -> Result<impl Reply, Rejection> {
    let stored = api.get_stored_api_key().await;

    let response = match stored {
        Some(key) => ApiKeyStatusResponse {
            configured: true,
            api_key_masked: Some(mask_api_key(&key.api_key)),
            use_testnet: key.use_testnet,
            permissions: Some(key.permissions),
            last_updated: Some(key.updated_at),
            connection_status: None, // Call /test endpoint for live status
        },
        None => ApiKeyStatusResponse {
            configured: false,
            api_key_masked: None,
            use_testnet: true, // Default to testnet
            permissions: None,
            last_updated: None,
            connection_status: None,
        },
    };

    Ok(warp::reply::with_status(
        warp::reply::json(&ApiResponse::success(response)),
        StatusCode::OK,
    ))
}

/// POST /api/settings/api-keys - Save API keys
async fn save_api_keys(
    request: SaveApiKeysRequest,
    api: Arc<SettingsApi>,
) -> Result<impl Reply, Rejection> {
    // Validate input
    if request.api_key.is_empty() || request.api_secret.is_empty() {
        return Ok(warp::reply::with_status(
            warp::reply::json(&ApiResponse::<()>::error(
                "API key and secret are required".to_string(),
            )),
            StatusCode::BAD_REQUEST,
        ));
    }

    // Encrypt the secret
    let (encrypted_secret, nonce) = match encrypt_secret(&request.api_secret) {
        Ok(result) => result,
        Err(e) => {
            tracing::error!("Failed to encrypt API secret: {}", e);
            return Ok(warp::reply::with_status(
                warp::reply::json(&ApiResponse::<()>::error(
                    "Failed to encrypt credentials".to_string(),
                )),
                StatusCode::INTERNAL_SERVER_ERROR,
            ));
        },
    };

    let now = chrono::Utc::now();
    let stored_key = StoredApiKey {
        api_key: request.api_key.clone(),
        api_secret_encrypted: encrypted_secret,
        api_secret_nonce: nonce,
        use_testnet: request.use_testnet,
        permissions: request.permissions,
        created_at: now,
        updated_at: now,
    };

    // Save to storage
    if let Err(e) = api.storage.save_api_key(&stored_key).await {
        tracing::error!("Failed to save API key: {}", e);
        return Ok(warp::reply::with_status(
            warp::reply::json(&ApiResponse::<()>::error(format!(
                "Failed to save API key: {}",
                e
            ))),
            StatusCode::INTERNAL_SERVER_ERROR,
        ));
    }

    tracing::info!(
        "API keys saved successfully (testnet: {}, permissions: {:?})",
        request.use_testnet,
        stored_key.permissions
    );

    Ok(warp::reply::with_status(
        warp::reply::json(&ApiResponse::success(serde_json::json!({
            "message": "API keys saved successfully",
            "api_key_masked": mask_api_key(&request.api_key),
            "use_testnet": request.use_testnet,
        }))),
        StatusCode::OK,
    ))
}

/// DELETE /api/settings/api-keys - Delete API keys
async fn delete_api_keys(api: Arc<SettingsApi>) -> Result<impl Reply, Rejection> {
    if let Err(e) = api.storage.delete_api_key().await {
        tracing::error!("Failed to delete API key: {}", e);
        return Ok(warp::reply::with_status(
            warp::reply::json(&ApiResponse::<()>::error(format!(
                "Failed to delete API key: {}",
                e
            ))),
            StatusCode::INTERNAL_SERVER_ERROR,
        ));
    }

    tracing::info!("API keys deleted successfully");

    Ok(warp::reply::with_status(
        warp::reply::json(&ApiResponse::success("API keys deleted successfully")),
        StatusCode::OK,
    ))
}

/// POST /api/settings/api-keys/test - Test connection with Binance
async fn test_connection(api: Arc<SettingsApi>) -> Result<impl Reply, Rejection> {
    // Get stored credentials
    let credentials = match api.get_decrypted_credentials().await {
        Some(creds) => creds,
        None => {
            return Ok(warp::reply::with_status(
                warp::reply::json(&ApiResponse::success(ConnectionStatus {
                    connected: false,
                    message: "No API keys configured. Please save your API keys first.".to_string(),
                    account_type: None,
                    can_trade: None,
                    balances_count: None,
                })),
                StatusCode::OK,
            ));
        },
    };

    let (api_key, api_secret, use_testnet) = credentials;

    // Create temporary BinanceClient to test connection
    let config = BinanceConfig {
        api_key: api_key.clone(),
        secret_key: api_secret.clone(),
        futures_api_key: String::new(),
        futures_secret_key: String::new(),
        testnet: use_testnet,
        base_url: if use_testnet {
            "https://testnet.binancefuture.com".to_string()
        } else {
            "https://fapi.binance.com".to_string()
        },
        ws_url: if use_testnet {
            "wss://stream.binancefuture.com".to_string()
        } else {
            "wss://fstream.binance.com".to_string()
        },
        futures_base_url: if use_testnet {
            "https://testnet.binancefuture.com".to_string()
        } else {
            "https://fapi.binance.com".to_string()
        },
        futures_ws_url: if use_testnet {
            "wss://stream.binancefuture.com".to_string()
        } else {
            "wss://fstream.binance.com".to_string()
        },
        trading_mode: Default::default(),
    };

    let client = match BinanceClient::new(config) {
        Ok(c) => c,
        Err(e) => {
            return Ok(warp::reply::with_status(
                warp::reply::json(&ApiResponse::success(ConnectionStatus {
                    connected: false,
                    message: format!("Failed to create Binance client: {}", e),
                    account_type: None,
                    can_trade: None,
                    balances_count: None,
                })),
                StatusCode::OK,
            ));
        },
    };

    // Test connection by fetching account info
    match client.get_account_info().await {
        Ok(account) => {
            let balances_with_value: Vec<_> = account
                .balances
                .iter()
                .filter(|b| {
                    let free = b.free.parse::<f64>().unwrap_or(0.0);
                    let locked = b.locked.parse::<f64>().unwrap_or(0.0);
                    free > 0.0 || locked > 0.0
                })
                .collect();

            Ok(warp::reply::with_status(
                warp::reply::json(&ApiResponse::success(ConnectionStatus {
                    connected: true,
                    message: format!(
                        "Successfully connected to Binance {}",
                        if use_testnet { "Testnet" } else { "Mainnet" }
                    ),
                    account_type: Some(if use_testnet {
                        "TESTNET".to_string()
                    } else {
                        "MAINNET".to_string()
                    }),
                    can_trade: Some(account.can_trade),
                    balances_count: Some(balances_with_value.len()),
                })),
                StatusCode::OK,
            ))
        },
        Err(e) => {
            let error_msg = format!("{}", e);
            let user_message = if error_msg.contains("Invalid API-key") {
                "Invalid API key. Please check your credentials.".to_string()
            } else if error_msg.contains("Signature") {
                "Invalid API secret. Please check your credentials.".to_string()
            } else if error_msg.contains("IP") {
                "IP not whitelisted. Please add your IP to the API key restrictions.".to_string()
            } else if error_msg.contains("permissions") {
                "Insufficient permissions. Please enable Futures Trading for this API key."
                    .to_string()
            } else {
                format!("Connection failed: {}", error_msg)
            };

            Ok(warp::reply::with_status(
                warp::reply::json(&ApiResponse::success(ConnectionStatus {
                    connected: false,
                    message: user_message,
                    account_type: None,
                    can_trade: None,
                    balances_count: None,
                })),
                StatusCode::OK,
            ))
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mask_api_key_short() {
        assert_eq!(mask_api_key("abc"), "***");
        assert_eq!(mask_api_key("abcdefgh"), "********");
    }

    #[test]
    fn test_mask_api_key_long() {
        let key = "abcdefghijklmnopqrstuvwxyz";
        let masked = mask_api_key(key);
        assert!(masked.starts_with("abcd"));
        assert!(masked.ends_with("wxyz"));
        assert!(masked.contains("*"));
    }

    #[test]
    fn test_encrypt_decrypt_secret() {
        let secret = "my-super-secret-api-key-12345";
        let (encrypted, nonce) = encrypt_secret(secret).expect("Encryption should succeed");

        // Encrypted should be different from original
        assert_ne!(encrypted, secret);

        // Should decrypt back to original
        let decrypted = decrypt_secret(&encrypted, &nonce).expect("Decryption should succeed");
        assert_eq!(decrypted, secret);
    }

    #[test]
    fn test_api_response_success() {
        let response = ApiResponse::success("test data");
        assert!(response.success);
        assert_eq!(response.data, Some("test data"));
        assert!(response.error.is_none());
    }

    #[test]
    fn test_api_response_error() {
        let response: ApiResponse<()> = ApiResponse::error("test error".to_string());
        assert!(!response.success);
        assert!(response.data.is_none());
        assert_eq!(response.error, Some("test error".to_string()));
    }

    #[test]
    fn test_api_key_permissions_default() {
        let permissions = ApiKeyPermissions::default();
        assert!(!permissions.spot_trading);
        assert!(permissions.futures_trading);
        assert!(!permissions.margin_trading);
        assert!(!permissions.options_trading);
    }

    #[test]
    fn test_stored_api_key_serialization() {
        let stored = StoredApiKey {
            api_key: "test_key".to_string(),
            api_secret_encrypted: "encrypted_data".to_string(),
            api_secret_nonce: "nonce_data".to_string(),
            use_testnet: true,
            permissions: ApiKeyPermissions::default(),
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
        };

        let json = serde_json::to_string(&stored).expect("Serialization should succeed");
        let deserialized: StoredApiKey =
            serde_json::from_str(&json).expect("Deserialization should succeed");

        assert_eq!(deserialized.api_key, "test_key");
        assert!(deserialized.use_testnet);
    }

    #[test]
    fn test_connection_status_serialization() {
        let status = ConnectionStatus {
            connected: true,
            message: "Connected successfully".to_string(),
            account_type: Some("TESTNET".to_string()),
            can_trade: Some(true),
            balances_count: Some(5),
        };

        let json = serde_json::to_string(&status).expect("Serialization should succeed");
        assert!(json.contains("\"connected\":true"));
        assert!(json.contains("TESTNET"));
    }
}
