// @spec:FR-REAL-007, FR-REAL-008
// @ref:plans/20251203-1353-binance-real-trading-system/phase-01-binance-order-api.md
// @test:TC-REAL-007, TC-REAL-008

//! User Data Stream Manager for Binance WebSocket
//!
//! This module handles the WebSocket connection to Binance's User Data Stream,
//! which provides real-time updates for:
//! - Order executions (executionReport)
//! - Account balance changes (outboundAccountPosition)
//! - Balance updates (balanceUpdate)

use anyhow::Result;
use futures_util::{SinkExt, StreamExt};
use std::sync::Arc;
use tokio::sync::{broadcast, mpsc, RwLock};
use tokio::time::{interval, Duration};
use tokio_tungstenite::{connect_async, tungstenite::Message};
use tracing::{debug, error, info, warn};

use super::client::BinanceClient;
use super::types::{ExecutionReport, OutboundAccountPosition, UserDataEvent, UserDataStreamHandle};

/// Configuration for the user data stream manager
#[derive(Debug, Clone)]
pub struct UserDataStreamConfig {
    /// Interval for keepalive pings (default: 30 minutes)
    pub keepalive_interval_secs: u64,
    /// Reconnect delay after disconnect (default: 5 seconds)
    pub reconnect_delay_secs: u64,
    /// Maximum reconnect attempts (default: 10)
    pub max_reconnect_attempts: u32,
    /// Channel buffer size (default: 100)
    pub channel_buffer_size: usize,
}

impl Default for UserDataStreamConfig {
    fn default() -> Self {
        Self {
            keepalive_interval_secs: 30 * 60, // 30 minutes
            reconnect_delay_secs: 5,
            max_reconnect_attempts: 10,
            channel_buffer_size: 100,
        }
    }
}

/// Events broadcast from the user data stream
/// Note: ExecutionReport is boxed to reduce enum size difference between variants
#[derive(Debug, Clone)]
pub enum UserDataStreamEvent {
    /// Connection established
    Connected,
    /// Connection lost
    Disconnected,
    /// Order execution report received (boxed to reduce enum size)
    ExecutionReport(Box<ExecutionReport>),
    /// Account position update received
    AccountPosition(OutboundAccountPosition),
    /// Balance update received
    BalanceUpdate(super::types::BalanceUpdate),
    /// Error occurred
    Error(String),
}

/// Manager for Binance User Data Stream WebSocket connection
pub struct UserDataStreamManager {
    client: BinanceClient,
    config: UserDataStreamConfig,
    handle: Arc<RwLock<Option<UserDataStreamHandle>>>,
    event_tx: broadcast::Sender<UserDataStreamEvent>,
    is_running: Arc<RwLock<bool>>,
    shutdown_tx: Option<mpsc::Sender<()>>,
}

impl UserDataStreamManager {
    /// Create a new UserDataStreamManager
    pub fn new(client: BinanceClient) -> Self {
        Self::with_config(client, UserDataStreamConfig::default())
    }

    /// Create a new UserDataStreamManager with custom config
    pub fn with_config(client: BinanceClient, config: UserDataStreamConfig) -> Self {
        let (event_tx, _) = broadcast::channel(config.channel_buffer_size);

        Self {
            client,
            config,
            handle: Arc::new(RwLock::new(None)),
            event_tx,
            is_running: Arc::new(RwLock::new(false)),
            shutdown_tx: None,
        }
    }

    /// Subscribe to user data stream events
    pub fn subscribe(&self) -> broadcast::Receiver<UserDataStreamEvent> {
        self.event_tx.subscribe()
    }

    /// Start the user data stream connection
    pub async fn start(&mut self) -> Result<()> {
        // Check if already running
        {
            let is_running = self.is_running.read().await;
            if *is_running {
                warn!("User data stream already running");
                return Ok(());
            }
        }

        // Create listen key
        let stream_handle = self.client.create_user_data_stream().await?;
        info!(
            "Created user data stream with listen key: {}...",
            &stream_handle.listen_key[..8]
        );

        // Store handle
        {
            let mut handle = self.handle.write().await;
            *handle = Some(stream_handle.clone());
        }

        // Set running flag
        {
            let mut is_running = self.is_running.write().await;
            *is_running = true;
        }

        // Create shutdown channel
        let (shutdown_tx, shutdown_rx) = mpsc::channel(1);
        self.shutdown_tx = Some(shutdown_tx);

        // Spawn connection task
        let client = self.client.clone();
        let config = self.config.clone();
        let handle = self.handle.clone();
        let event_tx = self.event_tx.clone();
        let is_running = self.is_running.clone();

        tokio::spawn(async move {
            Self::run_connection_loop(client, config, handle, event_tx, is_running, shutdown_rx)
                .await;
        });

        Ok(())
    }

    /// Stop the user data stream connection
    pub async fn stop(&mut self) -> Result<()> {
        // Send shutdown signal
        if let Some(tx) = self.shutdown_tx.take() {
            let _ = tx.send(()).await;
        }

        // Set running flag to false
        {
            let mut is_running = self.is_running.write().await;
            *is_running = false;
        }

        // Close listen key
        let listen_key = {
            let handle = self.handle.read().await;
            handle.as_ref().map(|h| h.listen_key.clone())
        };

        if let Some(listen_key) = listen_key {
            if let Err(e) = self.client.close_listen_key(&listen_key).await {
                warn!("Failed to close listen key: {}", e);
            }
        }

        // Clear handle
        {
            let mut handle = self.handle.write().await;
            *handle = None;
        }

        info!("User data stream stopped");
        Ok(())
    }

    /// Check if the stream is running
    pub async fn is_running(&self) -> bool {
        *self.is_running.read().await
    }

    /// Get the current listen key
    pub async fn get_listen_key(&self) -> Option<String> {
        let handle = self.handle.read().await;
        handle.as_ref().map(|h| h.listen_key.clone())
    }

    /// Main connection loop
    async fn run_connection_loop(
        client: BinanceClient,
        config: UserDataStreamConfig,
        handle: Arc<RwLock<Option<UserDataStreamHandle>>>,
        event_tx: broadcast::Sender<UserDataStreamEvent>,
        is_running: Arc<RwLock<bool>>,
        mut shutdown_rx: mpsc::Receiver<()>,
    ) {
        let mut reconnect_attempts = 0;

        loop {
            // Check if we should stop
            if !*is_running.read().await {
                break;
            }

            // Get current handle
            let ws_url = {
                let h = handle.read().await;
                match h.as_ref() {
                    Some(h) => h.ws_url.clone(),
                    None => {
                        error!("No user data stream handle available");
                        break;
                    },
                }
            };

            info!("Connecting to user data stream: {}", ws_url);

            // Connect to WebSocket
            match connect_async(&ws_url).await {
                Ok((ws_stream, _)) => {
                    reconnect_attempts = 0;
                    let _ = event_tx.send(UserDataStreamEvent::Connected);
                    info!("Connected to user data stream");

                    let (mut write, mut read) = ws_stream.split();

                    // Create keepalive interval
                    let mut keepalive_interval =
                        interval(Duration::from_secs(config.keepalive_interval_secs));

                    loop {
                        tokio::select! {
                            // Check for shutdown signal
                            _ = shutdown_rx.recv() => {
                                info!("Received shutdown signal");
                                let _ = write.close().await;
                                return;
                            }

                            // Handle incoming messages
                            msg = read.next() => {
                                match msg {
                                    Some(Ok(Message::Text(text))) => {
                                        Self::handle_message(&text, &event_tx);
                                    }
                                    Some(Ok(Message::Ping(data))) => {
                                        if let Err(e) = write.send(Message::Pong(data)).await {
                                            warn!("Failed to send pong: {}", e);
                                        }
                                    }
                                    Some(Ok(Message::Close(_))) => {
                                        warn!("WebSocket closed by server");
                                        break;
                                    }
                                    Some(Err(e)) => {
                                        error!("WebSocket error: {}", e);
                                        break;
                                    }
                                    None => {
                                        warn!("WebSocket stream ended");
                                        break;
                                    }
                                    _ => {}
                                }
                            }

                            // Send keepalive
                            _ = keepalive_interval.tick() => {
                                let listen_key = {
                                    let h = handle.read().await;
                                    h.as_ref().map(|h| h.listen_key.clone())
                                };

                                if let Some(listen_key) = listen_key {
                                    match client.keepalive_listen_key(&listen_key).await {
                                        Ok(_) => {
                                            debug!("Listen key keepalive sent");
                                            // Update last keepalive time
                                            let mut h = handle.write().await;
                                            if let Some(ref mut h) = *h {
                                                h.update_keepalive();
                                            }
                                        }
                                        Err(e) => {
                                            warn!("Failed to send keepalive: {}", e);
                                        }
                                    }
                                }
                            }
                        }
                    }

                    let _ = event_tx.send(UserDataStreamEvent::Disconnected);
                },
                Err(e) => {
                    error!("Failed to connect to user data stream: {}", e);
                    let _ = event_tx.send(UserDataStreamEvent::Error(e.to_string()));
                },
            }

            // Check if we should stop
            if !*is_running.read().await {
                break;
            }

            // Reconnect logic
            reconnect_attempts += 1;
            if reconnect_attempts > config.max_reconnect_attempts {
                error!(
                    "Max reconnect attempts ({}) reached",
                    config.max_reconnect_attempts
                );
                let mut running = is_running.write().await;
                *running = false;
                break;
            }

            let delay =
                Duration::from_secs(config.reconnect_delay_secs * reconnect_attempts as u64);
            warn!(
                "Reconnecting in {:?} (attempt {}/{})",
                delay, reconnect_attempts, config.max_reconnect_attempts
            );
            tokio::time::sleep(delay).await;

            // Recreate listen key if needed
            let needs_new_key = {
                let h = handle.read().await;
                h.as_ref().map(|h| h.is_expired()).unwrap_or(true)
            };

            if needs_new_key {
                match client.create_user_data_stream().await {
                    Ok(new_handle) => {
                        info!("Created new listen key");
                        let mut h = handle.write().await;
                        *h = Some(new_handle);
                    },
                    Err(e) => {
                        error!("Failed to create new listen key: {}", e);
                    },
                }
            }
        }

        info!("Connection loop ended");
    }

    /// Handle incoming WebSocket message
    fn handle_message(text: &str, event_tx: &broadcast::Sender<UserDataStreamEvent>) {
        debug!("Received message: {}", text);

        // Try to parse as UserDataEvent
        match serde_json::from_str::<UserDataEvent>(text) {
            Ok(event) => match event {
                UserDataEvent::ExecutionReport(report) => {
                    info!(
                        "Execution report: {} {} {} - Status: {}, Exec: {}",
                        report.symbol,
                        report.side,
                        report.order_type,
                        report.order_status,
                        report.execution_type
                    );
                    // Keep the report boxed since UserDataStreamEvent also uses Box<ExecutionReport>
                    let _ = event_tx.send(UserDataStreamEvent::ExecutionReport(report));
                },
                UserDataEvent::AccountPosition(position) => {
                    info!(
                        "Account position update: {} balances",
                        position.balances.len()
                    );
                    let _ = event_tx.send(UserDataStreamEvent::AccountPosition(position));
                },
                UserDataEvent::BalanceUpdate(update) => {
                    info!(
                        "Balance update: {} delta {}",
                        update.asset, update.balance_delta
                    );
                    let _ = event_tx.send(UserDataStreamEvent::BalanceUpdate(update));
                },
            },
            Err(e) => {
                // Log but don't fail - might be a different message type
                debug!("Failed to parse user data event: {} - Message: {}", e, text);
            },
        }
    }
}

impl Drop for UserDataStreamManager {
    fn drop(&mut self) {
        // Note: Can't do async cleanup in Drop, but we'll try to signal shutdown
        if let Some(tx) = self.shutdown_tx.take() {
            let _ = tx.try_send(());
        }
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
            base_url: "https://testnet.binance.vision".to_string(),
            ws_url: "wss://testnet.binance.vision/ws".to_string(),
            futures_base_url: "https://testnet.binancefuture.com".to_string(),
            futures_ws_url: "wss://stream.binancefuture.com".to_string(),
            testnet: true,
            trading_mode: crate::config::TradingMode::RealTestnet,
        }
    }

    #[test]
    fn test_user_data_stream_config_default() {
        let config = UserDataStreamConfig::default();
        assert_eq!(config.keepalive_interval_secs, 30 * 60);
        assert_eq!(config.reconnect_delay_secs, 5);
        assert_eq!(config.max_reconnect_attempts, 10);
        assert_eq!(config.channel_buffer_size, 100);
    }

    #[test]
    fn test_user_data_stream_manager_creation() {
        let config = create_test_config();
        let client = BinanceClient::new(config).expect("Failed to create client");
        let manager = UserDataStreamManager::new(client);

        // Should not be running initially
        let rt = tokio::runtime::Runtime::new().unwrap();
        let is_running = rt.block_on(async { manager.is_running().await });
        assert!(!is_running);
    }

    #[test]
    fn test_user_data_stream_manager_subscribe() {
        let config = create_test_config();
        let client = BinanceClient::new(config).expect("Failed to create client");
        let manager = UserDataStreamManager::new(client);

        // Should be able to subscribe
        let _rx = manager.subscribe();
    }

    #[test]
    fn test_execution_report_parsing() {
        let json = r#"{
            "e": "executionReport",
            "E": 1499405658658,
            "s": "BTCUSDT",
            "c": "mUvoqJxFIILMdfAW5iGSOW",
            "S": "BUY",
            "o": "LIMIT",
            "f": "GTC",
            "q": "1.00000000",
            "p": "0.10264410",
            "P": "0.00000000",
            "F": "0.00000000",
            "C": "",
            "x": "NEW",
            "X": "NEW",
            "r": "NONE",
            "i": 4293153,
            "l": "0.00000000",
            "z": "0.00000000",
            "L": "0.00000000",
            "n": "0",
            "N": null,
            "T": 1499405658657,
            "t": -1,
            "w": true,
            "m": false,
            "O": 1499405658657,
            "Z": "0.00000000",
            "Y": "0.00000000",
            "Q": "0.00000000"
        }"#;

        let report: ExecutionReport = serde_json::from_str(json).expect("Failed to parse");
        assert_eq!(report.event_type, "executionReport");
        assert_eq!(report.symbol, "BTCUSDT");
        assert_eq!(report.side, "BUY");
        assert_eq!(report.order_type, "LIMIT");
        assert_eq!(report.execution_type, "NEW");
        assert_eq!(report.order_status, "NEW");
        assert!(report.is_new());
        assert!(!report.is_filled());
    }

    #[test]
    fn test_execution_report_methods() {
        let report = ExecutionReport {
            event_type: "executionReport".to_string(),
            event_time: 1499405658658,
            symbol: "BTCUSDT".to_string(),
            client_order_id: "test".to_string(),
            side: "BUY".to_string(),
            order_type: "LIMIT".to_string(),
            time_in_force: "GTC".to_string(),
            order_quantity: "10.00000000".to_string(),
            order_price: "100.00".to_string(),
            stop_price: "0.00000000".to_string(),
            iceberg_quantity: "0.00000000".to_string(),
            original_client_order_id: "".to_string(),
            execution_type: "TRADE".to_string(),
            order_status: "FILLED".to_string(),
            order_reject_reason: "NONE".to_string(),
            order_id: 12345,
            last_executed_quantity: "10.00000000".to_string(),
            cumulative_filled_quantity: "10.00000000".to_string(),
            last_executed_price: "100.00".to_string(),
            commission_amount: "0.001".to_string(),
            commission_asset: Some("BNB".to_string()),
            transaction_time: 1499405658657,
            trade_id: 67890,
            is_on_book: false,
            is_maker: false,
            order_creation_time: 1499405658657,
            cumulative_quote_qty: "1000.00".to_string(),
            last_quote_qty: "1000.00".to_string(),
            quote_order_qty: "0.00".to_string(),
        };

        assert!(report.is_filled());
        assert!(report.is_trade());
        assert!(!report.is_cancelled());
        assert!(!report.is_rejected());
        assert!(!report.is_new());
        assert!(!report.is_partially_filled());
        assert!((report.fill_percentage() - 100.0).abs() < 0.01);
    }

    #[test]
    fn test_account_position_parsing() {
        let json = r#"{
            "e": "outboundAccountPosition",
            "E": 1564034571105,
            "u": 1564034571073,
            "B": [
                {
                    "a": "BTC",
                    "f": "10.0",
                    "l": "0.0"
                },
                {
                    "a": "USDT",
                    "f": "50000.0",
                    "l": "1000.0"
                }
            ]
        }"#;

        let event: UserDataEvent = serde_json::from_str(json).expect("Failed to parse");
        match event {
            UserDataEvent::AccountPosition(pos) => {
                // Note: event_type is empty when deserialized via tagged enum
                // because serde consumes "e" for tag discrimination
                assert!(pos.event_type.is_empty());
                assert_eq!(pos.balances.len(), 2);
                assert_eq!(pos.balances[0].asset, "BTC");
                assert_eq!(pos.balances[0].free, "10.0");
                assert_eq!(pos.balances[1].asset, "USDT");
            },
            _ => panic!("Expected AccountPosition event"),
        }
    }

    #[test]
    fn test_balance_update_parsing() {
        let json = r#"{
            "e": "balanceUpdate",
            "E": 1573200697110,
            "a": "BTC",
            "d": "100.00000000",
            "T": 1573200697068
        }"#;

        let event: UserDataEvent = serde_json::from_str(json).expect("Failed to parse");
        match event {
            UserDataEvent::BalanceUpdate(update) => {
                // Note: event_type is empty when deserialized via tagged enum
                // because serde consumes "e" for tag discrimination
                assert!(update.event_type.is_empty());
                assert_eq!(update.asset, "BTC");
                assert_eq!(update.balance_delta, "100.00000000");
            },
            _ => panic!("Expected BalanceUpdate event"),
        }
    }
}
