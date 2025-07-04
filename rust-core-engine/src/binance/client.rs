use anyhow::Result;
use chrono::Utc;
use hmac::{Hmac, Mac};
use reqwest::{Client, Method};
use serde::de::DeserializeOwned;
use sha2::Sha256;
use std::collections::HashMap;
use tracing::{error, trace};
use url::Url;

use crate::config::BinanceConfig;
use super::types::*;

type HmacSha256 = Hmac<Sha256>;

#[derive(Clone)]
pub struct BinanceClient {
    config: BinanceConfig,
    client: Client,
}

impl BinanceClient {
    pub fn new(config: BinanceConfig) -> Self {
        let client = Client::builder()
            .timeout(std::time::Duration::from_secs(30))
            .build()
            .expect("Failed to create HTTP client");

        Self { config, client }
    }

    // Authentication helpers
    fn sign_request(&self, query_string: &str) -> String {
        let mut mac = HmacSha256::new_from_slice(self.config.secret_key.as_bytes())
            .expect("HMAC can take key of any size");
        mac.update(query_string.as_bytes());
        hex::encode(mac.finalize().into_bytes())
    }

    fn get_timestamp() -> i64 {
        Utc::now().timestamp_millis()
    }

    async fn make_request<T>(&self, method: Method, endpoint: &str, params: Option<HashMap<String, String>>, signed: bool) -> Result<T>
    where
        T: DeserializeOwned,
    {
        let mut url = if endpoint.starts_with("/fapi/") {
            Url::parse(&format!("{}{}", self.config.futures_base_url, endpoint))?
        } else {
            Url::parse(&format!("{}/api/v3{}", self.config.base_url, endpoint))?
        };

        let mut query_params = params.unwrap_or_default();

        if signed {
            query_params.insert("timestamp".to_string(), Self::get_timestamp().to_string());
            
            let query_string = query_params
                .iter()
                .map(|(k, v)| format!("{}={}", k, v))
                .collect::<Vec<_>>()
                .join("&");

            let signature = self.sign_request(&query_string);
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

        trace!("Making request to: {}", url);

        let response = request_builder.send().await?;
        
        if !response.status().is_success() {
            let status = response.status();
            let error_text = response.text().await?;
            error!("Request failed with status {}: {}", status, error_text);
            return Err(anyhow::anyhow!("API request failed: {} - {}", status, error_text));
        }

        let response_text = response.text().await?;
        trace!("Response: {}", response_text);
        
        let result: T = serde_json::from_str(&response_text)?;
        Ok(result)
    }

    // Public API endpoints
    pub async fn get_klines(&self, symbol: &str, interval: &str, limit: Option<u16>) -> Result<Vec<Kline>> {
        let mut params = HashMap::new();
        params.insert("symbol".to_string(), symbol.to_uppercase());
        params.insert("interval".to_string(), interval.to_string());
        
        if let Some(limit) = limit {
            params.insert("limit".to_string(), limit.to_string());
        }

        let response: Vec<serde_json::Value> = self.make_request(Method::GET, "/klines", Some(params), false).await?;
        
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

    pub async fn get_futures_klines(&self, symbol: &str, interval: &str, limit: Option<u16>) -> Result<Vec<Kline>> {
        let mut params = HashMap::new();
        params.insert("symbol".to_string(), symbol.to_uppercase());
        params.insert("interval".to_string(), interval.to_string());
        
        if let Some(limit) = limit {
            params.insert("limit".to_string(), limit.to_string());
        }

        let response: Vec<serde_json::Value> = self.make_request(Method::GET, "/fapi/v1/klines", Some(params), false).await?;
        
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
        self.make_request(Method::GET, "/fapi/v2/account", None, true).await
    }

    pub async fn get_futures_positions(&self) -> Result<Vec<FuturesPosition>> {
        self.make_request(Method::GET, "/fapi/v2/positionRisk", None, true).await
    }

    pub async fn get_open_orders(&self, symbol: Option<&str>) -> Result<Vec<FuturesOrder>> {
        let mut params = HashMap::new();
        if let Some(symbol) = symbol {
            params.insert("symbol".to_string(), symbol.to_uppercase());
        }
        
        self.make_request(Method::GET, "/fapi/v1/openOrders", Some(params), true).await
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

        self.make_request(Method::POST, "/fapi/v1/order", Some(params), true).await
    }

    pub async fn cancel_order(&self, symbol: &str, order_id: Option<i64>, orig_client_order_id: Option<&str>) -> Result<serde_json::Value> {
        let mut params = HashMap::new();
        params.insert("symbol".to_string(), symbol.to_uppercase());
        
        if let Some(order_id) = order_id {
            params.insert("orderId".to_string(), order_id.to_string());
        }
        
        if let Some(orig_client_order_id) = orig_client_order_id {
            params.insert("origClientOrderId".to_string(), orig_client_order_id.to_string());
        }

        self.make_request(Method::DELETE, "/fapi/v1/order", Some(params), true).await
    }

    pub async fn change_leverage(&self, symbol: &str, leverage: u8) -> Result<serde_json::Value> {
        let mut params = HashMap::new();
        params.insert("symbol".to_string(), symbol.to_uppercase());
        params.insert("leverage".to_string(), leverage.to_string());

        self.make_request(Method::POST, "/fapi/v1/leverage", Some(params), true).await
    }

    pub async fn change_margin_type(&self, symbol: &str, margin_type: &str) -> Result<serde_json::Value> {
        let mut params = HashMap::new();
        params.insert("symbol".to_string(), symbol.to_uppercase());
        params.insert("marginType".to_string(), margin_type.to_string());

        self.make_request(Method::POST, "/fapi/v1/marginType", Some(params), true).await
    }
} 