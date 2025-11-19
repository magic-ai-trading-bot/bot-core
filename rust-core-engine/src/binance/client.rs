use anyhow::Result;
use chrono::Utc;
use hmac::{Hmac, Mac};
use reqwest::{Client, Method};
use serde::de::DeserializeOwned;
use sha2::Sha256;
use std::collections::HashMap;
use tracing::{error, trace};
use url::Url;

use super::types::*;
use crate::config::BinanceConfig;

// @spec:FR-TRADING-005 - Binance Integration
// @ref:specs/02-design/2.5-components/COMP-RUST-TRADING.md
// @test:TC-TRADING-030, TC-TRADING-031, TC-TRADING-032

type HmacSha256 = Hmac<Sha256>;

#[derive(Clone)]
pub struct BinanceClient {
    config: BinanceConfig,
    client: Client,
}

impl BinanceClient {
    pub fn new(config: BinanceConfig) -> Result<Self> {
        let client = Client::builder()
            .timeout(std::time::Duration::from_secs(30))
            .build()
            .map_err(|e| anyhow::anyhow!("Failed to create HTTP client: {}", e))?;

        Ok(Self { config, client })
    }

    // Authentication helpers
    fn sign_request(&self, query_string: &str) -> Result<String> {
        let mut mac = HmacSha256::new_from_slice(self.config.secret_key.as_bytes())
            .map_err(|e| anyhow::anyhow!("Failed to create HMAC instance: {}", e))?;
        mac.update(query_string.as_bytes());
        Ok(hex::encode(mac.finalize().into_bytes()))
    }

    fn get_timestamp() -> i64 {
        Utc::now().timestamp_millis()
    }

    async fn make_request<T>(
        &self,
        method: Method,
        endpoint: &str,
        params: Option<HashMap<String, String>>,
        signed: bool,
    ) -> Result<T>
    where
        T: DeserializeOwned,
    {
        let mut url = if endpoint.starts_with("/fapi/") {
            let futures_base_url = &self.config.futures_base_url;
            Url::parse(&format!("{futures_base_url}{endpoint}"))?
        } else {
            let base_url = &self.config.base_url;
            Url::parse(&format!("{base_url}/api/v3{endpoint}"))?
        };

        let mut query_params = params.unwrap_or_default();

        if signed {
            query_params.insert("timestamp".to_string(), Self::get_timestamp().to_string());

            let query_string = query_params
                .iter()
                .map(|(k, v)| format!("{k}={v}"))
                .collect::<Vec<_>>()
                .join("&");

            let signature = self.sign_request(&query_string)?;
            query_params.insert("signature".to_string(), signature);
        }

        // Add query parameters to URL
        for (key, value) in &query_params {
            url.query_pairs_mut().append_pair(key, value);
        }

        let mut request_builder = self.client.request(method, url.clone());

        // Add headers
        request_builder = request_builder.header("Content-Type", "application/json");

        if signed || !self.config.api_key.is_empty() {
            request_builder = request_builder.header("X-MBX-APIKEY", &self.config.api_key);
        }

        trace!("Making request to: {url}");

        let response = request_builder.send().await?;

        if !response.status().is_success() {
            let status = response.status();
            let error_text = response.text().await?;
            error!("Request failed with status {status}: {error_text}");
            return Err(anyhow::anyhow!(
                "API request failed: {} - {}",
                status,
                error_text
            ));
        }

        let response_text = response.text().await?;
        trace!("Response: {response_text}");

        let result: T = serde_json::from_str(&response_text)?;
        Ok(result)
    }

    // Public API endpoints
    pub async fn get_klines(
        &self,
        symbol: &str,
        interval: &str,
        limit: Option<u16>,
    ) -> Result<Vec<Kline>> {
        let mut params = HashMap::new();
        params.insert("symbol".to_string(), symbol.to_uppercase());
        params.insert("interval".to_string(), interval.to_string());

        if let Some(limit) = limit {
            params.insert("limit".to_string(), limit.to_string());
        }

        let response: Vec<serde_json::Value> = self
            .make_request(Method::GET, "/klines", Some(params), false)
            .await?;

        let klines: Vec<Kline> = response
            .into_iter()
            .map(|k| {
                if let serde_json::Value::Array(arr) = k {
                    Ok(Kline {
                        open_time: arr[0].as_i64().unwrap_or(0),
                        open: arr[1].as_str().unwrap_or("0").to_string(),
                        high: arr[2].as_str().unwrap_or("0").to_string(),
                        low: arr[3].as_str().unwrap_or("0").to_string(),
                        close: arr[4].as_str().unwrap_or("0").to_string(),
                        volume: arr[5].as_str().unwrap_or("0").to_string(),
                        close_time: arr[6].as_i64().unwrap_or(0),
                        quote_asset_volume: arr[7].as_str().unwrap_or("0").to_string(),
                        number_of_trades: arr[8].as_i64().unwrap_or(0),
                        taker_buy_base_asset_volume: arr[9].as_str().unwrap_or("0").to_string(),
                        taker_buy_quote_asset_volume: arr[10].as_str().unwrap_or("0").to_string(),
                        ignore: arr[11].as_str().unwrap_or("0").to_string(),
                    })
                } else {
                    Err(anyhow::anyhow!("Invalid kline data format"))
                }
            })
            .collect::<Result<Vec<_>>>()?;

        Ok(klines)
    }

    pub async fn get_futures_klines(
        &self,
        symbol: &str,
        interval: &str,
        limit: Option<u16>,
    ) -> Result<Vec<Kline>> {
        let mut params = HashMap::new();
        params.insert("symbol".to_string(), symbol.to_uppercase());
        params.insert("interval".to_string(), interval.to_string());

        if let Some(limit) = limit {
            params.insert("limit".to_string(), limit.to_string());
        }

        let response: Vec<serde_json::Value> = self
            .make_request(Method::GET, "/fapi/v1/klines", Some(params), false)
            .await?;

        let klines: Vec<Kline> = response
            .into_iter()
            .map(|k| {
                if let serde_json::Value::Array(arr) = k {
                    Ok(Kline {
                        open_time: arr[0].as_i64().unwrap_or(0),
                        open: arr[1].as_str().unwrap_or("0").to_string(),
                        high: arr[2].as_str().unwrap_or("0").to_string(),
                        low: arr[3].as_str().unwrap_or("0").to_string(),
                        close: arr[4].as_str().unwrap_or("0").to_string(),
                        volume: arr[5].as_str().unwrap_or("0").to_string(),
                        close_time: arr[6].as_i64().unwrap_or(0),
                        quote_asset_volume: arr[7].as_str().unwrap_or("0").to_string(),
                        number_of_trades: arr[8].as_i64().unwrap_or(0),
                        taker_buy_base_asset_volume: arr[9].as_str().unwrap_or("0").to_string(),
                        taker_buy_quote_asset_volume: arr[10].as_str().unwrap_or("0").to_string(),
                        ignore: arr[11].as_str().unwrap_or("0").to_string(),
                    })
                } else {
                    Err(anyhow::anyhow!("Invalid kline data format"))
                }
            })
            .collect::<Result<Vec<_>>>()?;

        Ok(klines)
    }

    // Private API endpoints
    pub async fn get_account_info(&self) -> Result<AccountInfo> {
        self.make_request(Method::GET, "/account", None, true).await
    }

    pub async fn get_futures_account(&self) -> Result<serde_json::Value> {
        self.make_request(Method::GET, "/fapi/v2/account", None, true)
            .await
    }

    pub async fn get_futures_positions(&self) -> Result<Vec<FuturesPosition>> {
        self.make_request(Method::GET, "/fapi/v2/positionRisk", None, true)
            .await
    }

    pub async fn get_open_orders(&self, symbol: Option<&str>) -> Result<Vec<FuturesOrder>> {
        let mut params = HashMap::new();
        if let Some(symbol) = symbol {
            params.insert("symbol".to_string(), symbol.to_uppercase());
        }

        self.make_request(Method::GET, "/fapi/v1/openOrders", Some(params), true)
            .await
    }

    pub async fn place_futures_order(&self, order: NewOrderRequest) -> Result<OrderResponse> {
        let mut params = HashMap::new();

        params.insert("symbol".to_string(), order.symbol);
        params.insert("side".to_string(), order.side);
        params.insert("type".to_string(), order.r#type);

        if let Some(quantity) = order.quantity {
            params.insert("quantity".to_string(), quantity);
        }

        if let Some(price) = order.price {
            params.insert("price".to_string(), price);
        }

        if let Some(time_in_force) = order.time_in_force {
            params.insert("timeInForce".to_string(), time_in_force);
        }

        if let Some(reduce_only) = order.reduce_only {
            params.insert("reduceOnly".to_string(), reduce_only.to_string());
        }

        if let Some(new_client_order_id) = order.new_client_order_id {
            params.insert("newClientOrderId".to_string(), new_client_order_id);
        }

        self.make_request(Method::POST, "/fapi/v1/order", Some(params), true)
            .await
    }

    pub async fn cancel_order(
        &self,
        symbol: &str,
        order_id: Option<i64>,
        orig_client_order_id: Option<&str>,
    ) -> Result<serde_json::Value> {
        let mut params = HashMap::new();
        params.insert("symbol".to_string(), symbol.to_uppercase());

        if let Some(order_id) = order_id {
            params.insert("orderId".to_string(), order_id.to_string());
        }

        if let Some(orig_client_order_id) = orig_client_order_id {
            params.insert(
                "origClientOrderId".to_string(),
                orig_client_order_id.to_string(),
            );
        }

        self.make_request(Method::DELETE, "/fapi/v1/order", Some(params), true)
            .await
    }

    pub async fn change_leverage(&self, symbol: &str, leverage: u8) -> Result<serde_json::Value> {
        let mut params = HashMap::new();
        params.insert("symbol".to_string(), symbol.to_uppercase());
        params.insert("leverage".to_string(), leverage.to_string());

        self.make_request(Method::POST, "/fapi/v1/leverage", Some(params), true)
            .await
    }

    pub async fn change_margin_type(
        &self,
        symbol: &str,
        margin_type: &str,
    ) -> Result<serde_json::Value> {
        let mut params = HashMap::new();
        params.insert("symbol".to_string(), symbol.to_uppercase());
        params.insert("marginType".to_string(), margin_type.to_string());

        self.make_request(Method::POST, "/fapi/v1/marginType", Some(params), true)
            .await
    }

    pub async fn get_symbol_price(&self, symbol: &str) -> Result<SymbolPrice> {
        let mut params = HashMap::new();
        params.insert("symbol".to_string(), symbol.to_uppercase());

        self.make_request(Method::GET, "/ticker/price", Some(params), false)
            .await
    }

    pub async fn get_funding_rate(&self, symbol: &str) -> Result<FundingRate> {
        let mut params = HashMap::new();
        params.insert("symbol".to_string(), symbol.to_uppercase());

        self.make_request(Method::GET, "/fapi/v1/fundingRate", Some(params), false)
            .await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::BinanceConfig;

    fn create_test_config() -> BinanceConfig {
        BinanceConfig {
            api_key: "test_api_key".to_string(),
            secret_key: "test_secret_key".to_string(),
            base_url: "https://api.binance.com".to_string(),
            ws_url: "wss://stream.binance.com:9443/ws".to_string(),
            futures_base_url: "https://fapi.binance.com".to_string(),
            futures_ws_url: "wss://fstream.binance.com".to_string(),
            testnet: false,
        }
    }

    #[test]
    fn test_client_creation() {
        let config = create_test_config();
        let client = BinanceClient::new(config.clone()).expect("Failed to create test client");
        assert_eq!(client.config.api_key, "test_api_key");
        assert_eq!(client.config.secret_key, "test_secret_key");
    }

    #[test]
    fn test_sign_request() {
        let config = create_test_config();
        let client = BinanceClient::new(config).expect("Failed to create test client");

        let query_string =
            "symbol=BTCUSDT&side=BUY&type=LIMIT&quantity=1&price=9000&timestamp=1499827319559";
        let signature = client
            .sign_request(query_string)
            .expect("Failed to sign request");

        // Signature should be a 64-character hex string (SHA256 produces 32 bytes = 64 hex chars)
        assert_eq!(signature.len(), 64);
        assert!(signature.chars().all(|c| c.is_ascii_hexdigit()));
    }

    #[test]
    fn test_sign_request_consistency() {
        let config = create_test_config();
        let client = BinanceClient::new(config).expect("Failed to create test client");

        let query_string = "symbol=BTCUSDT&timestamp=1499827319559";
        let signature1 = client
            .sign_request(query_string)
            .expect("Failed to sign request");
        let signature2 = client
            .sign_request(query_string)
            .expect("Failed to sign request");

        // Same input should produce same signature
        assert_eq!(signature1, signature2);
    }

    #[test]
    fn test_sign_request_different_inputs() {
        let config = create_test_config();
        let client = BinanceClient::new(config).expect("Failed to create test client");

        let query_string1 = "symbol=BTCUSDT&timestamp=1499827319559";
        let query_string2 = "symbol=ETHUSDT&timestamp=1499827319559";
        let signature1 = client
            .sign_request(query_string1)
            .expect("Failed to sign request");
        let signature2 = client
            .sign_request(query_string2)
            .expect("Failed to sign request");

        // Different inputs should produce different signatures
        assert_ne!(signature1, signature2);
    }

    #[test]
    fn test_get_timestamp() {
        let timestamp1 = BinanceClient::get_timestamp();
        std::thread::sleep(std::time::Duration::from_millis(10));
        let timestamp2 = BinanceClient::get_timestamp();

        // Timestamps should be different and in milliseconds
        assert!(timestamp2 > timestamp1);
        assert!(timestamp1 > 0);
        // Check that timestamp is reasonable (after 2020 and before 2100)
        assert!(timestamp1 > 1577836800000); // Jan 1, 2020
        assert!(timestamp1 < 4102444800000); // Jan 1, 2100
    }

    #[test]
    fn test_clone() {
        let config = create_test_config();
        let client1 = BinanceClient::new(config).expect("Failed to create test client");
        let client2 = client1.clone();

        assert_eq!(client1.config.api_key, client2.config.api_key);
        assert_eq!(client1.config.secret_key, client2.config.secret_key);
    }

    #[test]
    fn test_sign_empty_string() {
        let config = create_test_config();
        let client = BinanceClient::new(config).expect("Failed to create test client");

        let signature = client.sign_request("").expect("Failed to sign request");
        assert_eq!(signature.len(), 64);
        assert!(signature.chars().all(|c| c.is_ascii_hexdigit()));
    }

    #[test]
    fn test_sign_special_characters() {
        let config = create_test_config();
        let client = BinanceClient::new(config).expect("Failed to create test client");

        let query_string = "symbol=BTC%2FUSDT&quantity=1.5&price=50000.00";
        let signature = client
            .sign_request(query_string)
            .expect("Failed to sign request");

        assert_eq!(signature.len(), 64);
        assert!(signature.chars().all(|c| c.is_ascii_hexdigit()));
    }

    #[test]
    fn test_config_clone() {
        let config = create_test_config();
        let cloned = config.clone();

        assert_eq!(config.api_key, cloned.api_key);
        assert_eq!(config.base_url, cloned.base_url);
        assert_eq!(config.testnet, cloned.testnet);
    }

    #[test]
    fn test_timestamp_is_millis() {
        let timestamp = BinanceClient::get_timestamp();
        let timestamp_str = timestamp.to_string();

        // Timestamp in milliseconds should be 13 digits
        assert_eq!(timestamp_str.len(), 13);
    }

    #[test]
    fn test_client_with_testnet_config() {
        let mut config = create_test_config();
        config.testnet = true;
        config.base_url = "https://testnet.binance.vision".to_string();

        let client = BinanceClient::new(config.clone());
        assert_eq!(client.config.base_url, "https://testnet.binance.vision");
        assert!(client.config.testnet);
    }

    #[test]
    fn test_signature_with_different_keys() {
        let mut config1 = create_test_config();
        config1.secret_key = "secret_key_1".to_string();
        let client1 = BinanceClient::new(config1)?;

        let mut config2 = create_test_config();
        config2.secret_key = "secret_key_2".to_string();
        let client2 = BinanceClient::new(config2)?;

        let query_string = "symbol=BTCUSDT&timestamp=1499827319559";
        let signature1 = client1
            .sign_request(query_string)
            .expect("Failed to sign request");
        let signature2 = client2
            .sign_request(query_string)
            .expect("Failed to sign request");

        // Different keys should produce different signatures
        assert_ne!(signature1, signature2);
    }

    #[test]
    fn test_sign_request_with_unicode() {
        let config = create_test_config();
        let client = BinanceClient::new(config).expect("Failed to create test client");

        // Test that signature works with various strings
        let query_string = "symbol=BTCUSDT&note=test";
        let signature = client
            .sign_request(query_string)
            .expect("Failed to sign request");

        assert_eq!(signature.len(), 64);
        assert!(signature.chars().all(|c| c.is_ascii_hexdigit()));
    }

    #[test]
    fn test_timestamp_monotonic() {
        let mut timestamps = Vec::new();
        for _ in 0..5 {
            timestamps.push(BinanceClient::get_timestamp());
            std::thread::sleep(std::time::Duration::from_millis(2));
        }

        // Timestamps should be monotonically increasing
        for i in 1..timestamps.len() {
            assert!(timestamps[i] >= timestamps[i - 1]);
        }
    }

    #[test]
    fn test_client_with_empty_api_key() {
        let mut config = create_test_config();
        config.api_key = "".to_string();

        let client = BinanceClient::new(config).expect("Failed to create test client");
        assert_eq!(client.config.api_key, "");
    }

    #[test]
    fn test_sign_long_query_string() {
        let config = create_test_config();
        let client = BinanceClient::new(config).expect("Failed to create test client");

        let mut long_query = String::new();
        for i in 0..100 {
            long_query.push_str(&format!("param{i}=value{i}&"));
        }

        let signature = client
            .sign_request(&long_query)
            .expect("Failed to sign request");
        assert_eq!(signature.len(), 64);
        assert!(signature.chars().all(|c| c.is_ascii_hexdigit()));
    }

    #[test]
    fn test_hex_encoding_lowercase() {
        let config = create_test_config();
        let client = BinanceClient::new(config).expect("Failed to create test client");

        let signature = client.sign_request("test").expect("Failed to sign request");
        // Hex encoding should be lowercase
        assert!(signature
            .chars()
            .all(|c| !c.is_ascii_uppercase() || !c.is_alphabetic()));
    }

    #[test]
    fn test_config_urls() {
        let config = create_test_config();

        assert!(config.base_url.starts_with("https://"));
        assert!(config.ws_url.starts_with("wss://"));
        assert!(config.futures_base_url.starts_with("https://"));
        assert!(config.futures_ws_url.starts_with("wss://"));
    }

    // === Additional Comprehensive Tests ===

    // Test BinanceClient::new() variants
    #[test]
    fn test_client_new_with_production_config() {
        let config = BinanceConfig {
            api_key: "prod_api_key".to_string(),
            secret_key: "prod_secret_key".to_string(),
            base_url: "https://api.binance.com".to_string(),
            ws_url: "wss://stream.binance.com:9443/ws".to_string(),
            futures_base_url: "https://fapi.binance.com".to_string(),
            futures_ws_url: "wss://fstream.binance.com".to_string(),
            testnet: false,
        };

        let client = BinanceClient::new(config.clone());
        assert_eq!(client.config.api_key, "prod_api_key");
        assert!(!client.config.testnet);
        assert_eq!(client.config.base_url, "https://api.binance.com");
    }

    #[test]
    fn test_client_new_with_testnet_config() {
        let config = BinanceConfig {
            api_key: "test_api_key".to_string(),
            secret_key: "test_secret_key".to_string(),
            base_url: "https://testnet.binance.vision".to_string(),
            ws_url: "wss://testnet.binance.vision/ws".to_string(),
            futures_base_url: "https://testnet.binancefuture.com".to_string(),
            futures_ws_url: "wss://stream.binancefuture.com".to_string(),
            testnet: true,
        };

        let client = BinanceClient::new(config.clone());
        assert!(client.config.testnet);
        assert!(client.config.base_url.contains("testnet"));
    }

    #[test]
    fn test_client_new_with_empty_credentials() {
        let config = BinanceConfig {
            api_key: "".to_string(),
            secret_key: "".to_string(),
            base_url: "https://api.binance.com".to_string(),
            ws_url: "wss://stream.binance.com:9443/ws".to_string(),
            futures_base_url: "https://fapi.binance.com".to_string(),
            futures_ws_url: "wss://fstream.binance.com".to_string(),
            testnet: false,
        };

        let client = BinanceClient::new(config).expect("Failed to create test client");
        assert_eq!(client.config.api_key, "");
        assert_eq!(client.config.secret_key, "");
    }

    #[test]
    fn test_client_new_with_long_api_keys() {
        let long_key = "a".repeat(1000);
        let config = BinanceConfig {
            api_key: long_key.clone(),
            secret_key: long_key.clone(),
            base_url: "https://api.binance.com".to_string(),
            ws_url: "wss://stream.binance.com:9443/ws".to_string(),
            futures_base_url: "https://fapi.binance.com".to_string(),
            futures_ws_url: "wss://fstream.binance.com".to_string(),
            testnet: false,
        };

        let client = BinanceClient::new(config).expect("Failed to create test client");
        assert_eq!(client.config.api_key.len(), 1000);
        assert_eq!(client.config.secret_key.len(), 1000);
    }

    #[test]
    fn test_client_new_with_special_chars_in_keys() {
        let config = BinanceConfig {
            api_key: "key!@#$%^&*()".to_string(),
            secret_key: "secret+=-_{}[]|:;<>,.?/~`".to_string(),
            base_url: "https://api.binance.com".to_string(),
            ws_url: "wss://stream.binance.com:9443/ws".to_string(),
            futures_base_url: "https://fapi.binance.com".to_string(),
            futures_ws_url: "wss://fstream.binance.com".to_string(),
            testnet: false,
        };

        let client = BinanceClient::new(config.clone());
        assert_eq!(client.config.api_key, "key!@#$%^&*()");
        assert!(client.config.secret_key.contains("+=-_"));
    }

    // Test sign_request() edge cases
    #[test]
    fn test_sign_request_empty_query() {
        let config = create_test_config();
        let client = BinanceClient::new(config).expect("Failed to create test client");

        let signature = client.sign_request("").expect("Failed to sign request");

        // Should still produce valid signature for empty string
        assert_eq!(signature.len(), 64);
        assert!(signature.chars().all(|c| c.is_ascii_hexdigit()));
    }

    #[test]
    fn test_sign_request_single_param() {
        let config = create_test_config();
        let client = BinanceClient::new(config).expect("Failed to create test client");

        let signature = client
            .sign_request("symbol=BTCUSDT")
            .expect("Failed to sign request");
        assert_eq!(signature.len(), 64);
    }

    #[test]
    fn test_sign_request_multiple_params() {
        let config = create_test_config();
        let client = BinanceClient::new(config).expect("Failed to create test client");

        let query = "symbol=BTCUSDT&side=BUY&type=LIMIT&quantity=1&price=50000";
        let signature = client.sign_request(query).expect("Failed to sign request");
        assert_eq!(signature.len(), 64);
    }

    #[test]
    fn test_sign_request_with_encoded_params() {
        let config = create_test_config();
        let client = BinanceClient::new(config).expect("Failed to create test client");

        let query = "symbol=BTCUSDT&note=Hello%20World&tag=test%26tag";
        let signature = client.sign_request(query).expect("Failed to sign request");
        assert_eq!(signature.len(), 64);
    }

    #[test]
    fn test_sign_request_with_numbers() {
        let config = create_test_config();
        let client = BinanceClient::new(config).expect("Failed to create test client");

        let query = "timestamp=1234567890123&recvWindow=5000";
        let signature = client.sign_request(query).expect("Failed to sign request");
        assert_eq!(signature.len(), 64);
    }

    #[test]
    fn test_sign_request_with_decimals() {
        let config = create_test_config();
        let client = BinanceClient::new(config).expect("Failed to create test client");

        let query = "price=50000.50&quantity=0.001&stopPrice=49999.99";
        let signature = client.sign_request(query).expect("Failed to sign request");
        assert_eq!(signature.len(), 64);
    }

    #[test]
    fn test_sign_request_deterministic() {
        let config = create_test_config();
        let client = BinanceClient::new(config).expect("Failed to create test client");

        let query = "symbol=ETHUSDT&interval=1h&limit=100";
        let sig1 = client.sign_request(query).expect("Failed to sign request");
        let sig2 = client.sign_request(query).expect("Failed to sign request");
        let sig3 = client.sign_request(query).expect("Failed to sign request");

        assert_eq!(sig1, sig2);
        assert_eq!(sig2, sig3);
    }

    #[test]
    fn test_sign_request_order_matters() {
        let config = create_test_config();
        let client = BinanceClient::new(config).expect("Failed to create test client");

        let query1 = "symbol=BTCUSDT&side=BUY";
        let query2 = "side=BUY&symbol=BTCUSDT";
        let sig1 = client.sign_request(query1).expect("Failed to sign request");
        let sig2 = client.sign_request(query2).expect("Failed to sign request");

        // Different order should produce different signatures
        assert_ne!(sig1, sig2);
    }

    #[test]
    fn test_sign_request_case_sensitive() {
        let config = create_test_config();
        let client = BinanceClient::new(config).expect("Failed to create test client");

        let query1 = "symbol=BTCUSDT";
        let query2 = "symbol=btcusdt";
        let sig1 = client.sign_request(query1).expect("Failed to sign request");
        let sig2 = client.sign_request(query2).expect("Failed to sign request");

        // Case difference should produce different signatures
        assert_ne!(sig1, sig2);
    }

    #[test]
    fn test_sign_request_with_whitespace() {
        let config = create_test_config();
        let client = BinanceClient::new(config).expect("Failed to create test client");

        let query1 = "symbol=BTCUSDT&side=BUY";
        let query2 = "symbol=BTCUSDT&side=BUY ";
        let sig1 = client.sign_request(query1).expect("Failed to sign request");
        let sig2 = client.sign_request(query2).expect("Failed to sign request");

        // Trailing space should produce different signature
        assert_ne!(sig1, sig2);
    }

    #[test]
    fn test_sign_request_very_long_query() {
        let config = create_test_config();
        let client = BinanceClient::new(config).expect("Failed to create test client");

        let mut query = String::from("symbol=BTCUSDT");
        for i in 0..500 {
            query.push_str(&format!("&param{i}=value{i}"));
        }

        let signature = client.sign_request(&query).expect("Failed to sign request");
        assert_eq!(signature.len(), 64);
    }

    #[test]
    fn test_sign_request_with_equals_in_value() {
        let config = create_test_config();
        let client = BinanceClient::new(config).expect("Failed to create test client");

        let query = "symbol=BTCUSDT&note=test=value";
        let signature = client.sign_request(query).expect("Failed to sign request");
        assert_eq!(signature.len(), 64);
    }

    #[test]
    fn test_sign_request_with_ampersand_encoded() {
        let config = create_test_config();
        let client = BinanceClient::new(config).expect("Failed to create test client");

        let query = "symbol=BTCUSDT&tag=test%26encoded";
        let signature = client.sign_request(query).expect("Failed to sign request");
        assert_eq!(signature.len(), 64);
    }

    // Test signature with different secret keys
    #[test]
    fn test_different_secrets_produce_different_signatures() {
        let mut config1 = create_test_config();
        config1.secret_key = "secret1".to_string();
        let client1 = BinanceClient::new(config1)?;

        let mut config2 = create_test_config();
        config2.secret_key = "secret2".to_string();
        let client2 = BinanceClient::new(config2)?;

        let mut config3 = create_test_config();
        config3.secret_key = "secret3".to_string();
        let client3 = BinanceClient::new(config3)?;

        let query = "symbol=BTCUSDT&timestamp=1234567890";
        let sig1 = client1.sign_request(query).expect("Failed to sign request");
        let sig2 = client2.sign_request(query).expect("Failed to sign request");
        let sig3 = client3.sign_request(query).expect("Failed to sign request");

        assert_ne!(sig1, sig2);
        assert_ne!(sig2, sig3);
        assert_ne!(sig1, sig3);
    }

    #[test]
    fn test_signature_hex_format() {
        let config = create_test_config();
        let client = BinanceClient::new(config).expect("Failed to create test client");

        let signature = client
            .sign_request("test_query")
            .expect("Failed to sign request");

        // Should be all lowercase hex
        for c in signature.chars() {
            assert!(c.is_ascii_hexdigit());
            if c.is_alphabetic() {
                assert!(c.is_lowercase());
            }
        }
    }

    #[test]
    fn test_signature_with_numeric_only_query() {
        let config = create_test_config();
        let client = BinanceClient::new(config).expect("Failed to create test client");

        let signature = client
            .sign_request("1234567890")
            .expect("Failed to sign request");
        assert_eq!(signature.len(), 64);
    }

    #[test]
    fn test_signature_with_newlines() {
        let config = create_test_config();
        let client = BinanceClient::new(config).expect("Failed to create test client");

        let query1 = "symbol=BTCUSDT&side=BUY";
        let query2 = "symbol=BTCUSDT\n&side=BUY";
        let sig1 = client.sign_request(query1).expect("Failed to sign request");
        let sig2 = client.sign_request(query2).expect("Failed to sign request");

        // Newline should produce different signature
        assert_ne!(sig1, sig2);
    }

    // Test get_timestamp()
    #[test]
    fn test_timestamp_positive() {
        let timestamp = BinanceClient::get_timestamp();
        assert!(timestamp > 0);
    }

    #[test]
    fn test_timestamp_reasonable_range() {
        let timestamp = BinanceClient::get_timestamp();

        // After Jan 1, 2024 and before Jan 1, 2100
        assert!(timestamp > 1704067200000); // Jan 1, 2024
        assert!(timestamp < 4102444800000); // Jan 1, 2100
    }

    #[test]
    fn test_timestamp_millisecond_precision() {
        let timestamp = BinanceClient::get_timestamp();
        let timestamp_str = timestamp.to_string();

        // Should be 13 digits (milliseconds since epoch)
        assert_eq!(timestamp_str.len(), 13);
    }

    #[test]
    fn test_timestamp_sequential_calls() {
        let timestamps: Vec<i64> = (0..10)
            .map(|_| {
                let ts = BinanceClient::get_timestamp();
                std::thread::sleep(std::time::Duration::from_millis(1));
                ts
            })
            .collect();

        // Check all timestamps are unique and increasing
        for i in 1..timestamps.len() {
            assert!(timestamps[i] >= timestamps[i - 1]);
        }
    }

    #[test]
    fn test_timestamp_multiple_rapid_calls() {
        let ts1 = BinanceClient::get_timestamp();
        let ts2 = BinanceClient::get_timestamp();
        let ts3 = BinanceClient::get_timestamp();

        // Should be close to each other (within 100ms)
        assert!((ts3 - ts1).abs() < 100);

        // Should be non-decreasing
        assert!(ts2 >= ts1);
        assert!(ts3 >= ts2);
    }

    // Test client cloning
    #[test]
    fn test_client_clone_preserves_config() {
        let config = create_test_config();
        let client = BinanceClient::new(config).expect("Failed to create test client");
        let cloned = client.clone();

        assert_eq!(client.config.api_key, cloned.config.api_key);
        assert_eq!(client.config.secret_key, cloned.config.secret_key);
        assert_eq!(client.config.base_url, cloned.config.base_url);
        assert_eq!(client.config.ws_url, cloned.config.ws_url);
        assert_eq!(
            client.config.futures_base_url,
            cloned.config.futures_base_url
        );
        assert_eq!(client.config.futures_ws_url, cloned.config.futures_ws_url);
        assert_eq!(client.config.testnet, cloned.config.testnet);
    }

    #[test]
    fn test_client_clone_independent_signatures() {
        let config = create_test_config();
        let client1 = BinanceClient::new(config).expect("Failed to create test client");
        let client2 = client1.clone();

        let query = "symbol=BTCUSDT&timestamp=1234567890";
        let sig1 = client1.sign_request(query).expect("Failed to sign request");
        let sig2 = client2.sign_request(query).expect("Failed to sign request");

        // Should produce same signature
        assert_eq!(sig1, sig2);
    }

    // Test config variations
    #[test]
    fn test_config_with_custom_urls() {
        let config = BinanceConfig {
            api_key: "test_key".to_string(),
            secret_key: "test_secret".to_string(),
            base_url: "https://custom.binance.com".to_string(),
            ws_url: "wss://custom.binance.com/ws".to_string(),
            futures_base_url: "https://custom-futures.binance.com".to_string(),
            futures_ws_url: "wss://custom-futures.binance.com/ws".to_string(),
            testnet: false,
        };

        let client = BinanceClient::new(config.clone());
        assert_eq!(client.config.base_url, "https://custom.binance.com");
        assert_eq!(
            client.config.futures_base_url,
            "https://custom-futures.binance.com"
        );
    }

    #[test]
    fn test_config_urls_protocol() {
        let config = create_test_config();

        // HTTP URLs should use https
        assert!(config.base_url.starts_with("https://"));
        assert!(config.futures_base_url.starts_with("https://"));

        // WebSocket URLs should use wss
        assert!(config.ws_url.starts_with("wss://"));
        assert!(config.futures_ws_url.starts_with("wss://"));
    }

    #[test]
    fn test_config_testnet_flag() {
        let mut config = create_test_config();
        assert!(!config.testnet);

        config.testnet = true;
        let client = BinanceClient::new(config.clone());
        assert!(client.config.testnet);
    }

    // Test signature consistency across instances
    #[test]
    fn test_signature_consistency_across_instances() {
        let config = create_test_config();
        let client1 = BinanceClient::new(config.clone());
        let client2 = BinanceClient::new(config.clone());

        let query = "symbol=BTCUSDT&side=BUY&type=MARKET";
        let sig1 = client1.sign_request(query).expect("Failed to sign request");
        let sig2 = client2.sign_request(query).expect("Failed to sign request");

        assert_eq!(sig1, sig2);
    }

    #[test]
    fn test_signature_with_all_printable_ascii() {
        let config = create_test_config();
        let client = BinanceClient::new(config).expect("Failed to create test client");

        // Test with various ASCII characters
        let query = "data=!@#$%^&*()_+-=[]{}|;':\",./<>?`~";
        let signature = client.sign_request(query).expect("Failed to sign request");
        assert_eq!(signature.len(), 64);
    }

    #[test]
    fn test_client_multiple_clones() {
        let config = create_test_config();
        let client1 = BinanceClient::new(config).expect("Failed to create test client");
        let client2 = client1.clone();
        let client3 = client2.clone();
        let client4 = client3.clone();

        assert_eq!(client1.config.api_key, client4.config.api_key);

        let query = "test=query";
        assert_eq!(
            client1.sign_request(query).expect("Failed to sign request"),
            client4.sign_request(query).expect("Failed to sign request")
        );
    }

    // Test edge cases for query strings
    #[test]
    fn test_sign_request_query_with_only_ampersands() {
        let config = create_test_config();
        let client = BinanceClient::new(config).expect("Failed to create test client");

        let signature = client.sign_request("&&&").expect("Failed to sign request");
        assert_eq!(signature.len(), 64);
    }

    #[test]
    fn test_sign_request_query_with_only_equals() {
        let config = create_test_config();
        let client = BinanceClient::new(config).expect("Failed to create test client");

        let signature = client.sign_request("===").expect("Failed to sign request");
        assert_eq!(signature.len(), 64);
    }

    #[test]
    fn test_sign_request_mixed_case_params() {
        let config = create_test_config();
        let client = BinanceClient::new(config).expect("Failed to create test client");

        let query1 = "Symbol=BTCUSDT&Side=BUY";
        let query2 = "symbol=BTCUSDT&side=BUY";
        let sig1 = client.sign_request(query1).expect("Failed to sign request");
        let sig2 = client.sign_request(query2).expect("Failed to sign request");

        // Case should matter
        assert_ne!(sig1, sig2);
    }

    #[test]
    fn test_timestamp_format() {
        let timestamp = BinanceClient::get_timestamp();

        // Should be able to convert to string
        let _timestamp_str = timestamp.to_string();

        // Should be i64
        assert!(timestamp.is_positive());
    }

    #[test]
    fn test_client_config_immutability() {
        let config = BinanceConfig {
            api_key: "original_key".to_string(),
            secret_key: "original_secret".to_string(),
            base_url: "https://api.binance.com".to_string(),
            ws_url: "wss://stream.binance.com:9443/ws".to_string(),
            futures_base_url: "https://fapi.binance.com".to_string(),
            futures_ws_url: "wss://fstream.binance.com".to_string(),
            testnet: false,
        };

        let client = BinanceClient::new(config.clone());

        // Original config should be unchanged
        assert_eq!(config.api_key, "original_key");
        assert_eq!(client.config.api_key, "original_key");
    }

    #[test]
    fn test_signature_length_always_64() {
        let config = create_test_config();
        let client = BinanceClient::new(config).expect("Failed to create test client");

        let long_string = "x".repeat(10000);
        let test_queries = vec![
            "",
            "a",
            "short",
            "symbol=BTCUSDT",
            "symbol=BTCUSDT&side=BUY&type=LIMIT&quantity=1&price=50000",
            &long_string,
        ];

        for query in test_queries {
            let signature = client.sign_request(query).expect("Failed to sign request");
            assert_eq!(signature.len(), 64, "Failed for query: {}", query);
        }
    }

    #[test]
    fn test_signature_alphanumeric_only() {
        let config = create_test_config();
        let client = BinanceClient::new(config).expect("Failed to create test client");

        let signature = client.sign_request("test").expect("Failed to sign request");

        // Should only contain 0-9 and a-f (hex)
        for c in signature.chars() {
            assert!(
                ('0'..='9').contains(&c) || ('a'..='f').contains(&c),
                "Invalid character in signature: {c}"
            );
        }
    }

    #[test]
    fn test_client_creation_multiple_times() {
        let config = create_test_config();

        // Create multiple clients
        let _client1 = BinanceClient::new(config.clone());
        let _client2 = BinanceClient::new(config.clone());
        let _client3 = BinanceClient::new(config.clone());

        // Should not panic
        assert!(true);
    }

    #[test]
    fn test_timestamp_never_negative() {
        for _ in 0..100 {
            let timestamp = BinanceClient::get_timestamp();
            assert!(timestamp > 0, "Timestamp should never be negative");
        }
    }

    #[test]
    fn test_config_all_fields_present() {
        let config = create_test_config();

        // All fields should be non-empty
        assert!(!config.api_key.is_empty());
        assert!(!config.secret_key.is_empty());
        assert!(!config.base_url.is_empty());
        assert!(!config.ws_url.is_empty());
        assert!(!config.futures_base_url.is_empty());
        assert!(!config.futures_ws_url.is_empty());
    }

    #[test]
    fn test_signature_with_tab_character() {
        let config = create_test_config();
        let client = BinanceClient::new(config).expect("Failed to create test client");

        let query1 = "symbol=BTCUSDT&side=BUY";
        let query2 = "symbol=BTCUSDT\t&side=BUY";
        let sig1 = client.sign_request(query1).expect("Failed to sign request");
        let sig2 = client.sign_request(query2).expect("Failed to sign request");

        // Tab should produce different signature
        assert_ne!(sig1, sig2);
    }

    #[test]
    fn test_signature_with_null_byte_string() {
        let config = create_test_config();
        let client = BinanceClient::new(config).expect("Failed to create test client");

        // This should still work, as it's just a string
        let query = "symbol=BTCUSDT\0&side=BUY";
        let signature = client.sign_request(query).expect("Failed to sign request");
        assert_eq!(signature.len(), 64);
    }
}
