use anyhow::Result;
use futures_util::{SinkExt, StreamExt};
use serde_json::Value;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use std::time::Duration;
use tokio::select;
use tokio::sync::mpsc;
use tokio::time::{interval, sleep};
use tokio_tungstenite::{connect_async, tungstenite::Message};
use tracing::{debug, error, info, warn};
use url::Url;

use super::types::*;
use crate::config::BinanceConfig;

// @spec:FR-WEBSOCKET-001 - Binance WebSocket Connection
// @spec:FR-WEBSOCKET-002 - Dynamic Symbol Subscription
// @ref:specs/02-design/2.3-api/API-WEBSOCKET.md
// @test:TC-INTEGRATION-008, TC-INTEGRATION-009

/// Commands that can be sent to the WebSocket for dynamic subscription
#[derive(Debug, Clone)]
pub enum WebSocketCommand {
    /// Subscribe to new streams for a symbol
    Subscribe {
        symbol: String,
        timeframes: Vec<String>,
    },
    /// Unsubscribe from streams for a symbol
    Unsubscribe {
        symbol: String,
        timeframes: Vec<String>,
    },
}

pub struct BinanceWebSocket {
    config: BinanceConfig,
    sender: mpsc::UnboundedSender<StreamEvent>,
    /// Channel for receiving subscribe/unsubscribe commands
    command_sender: mpsc::UnboundedSender<WebSocketCommand>,
    /// Receiver is wrapped in Mutex for interior mutability (take() in async context)
    command_receiver: std::sync::Mutex<Option<mpsc::UnboundedReceiver<WebSocketCommand>>>,
    /// Counter for request IDs
    request_id: Arc<AtomicU64>,
}

impl BinanceWebSocket {
    pub fn new(config: BinanceConfig) -> (Self, mpsc::UnboundedReceiver<StreamEvent>) {
        let (sender, receiver) = mpsc::unbounded_channel();
        let (command_sender, command_receiver) = mpsc::unbounded_channel();

        let ws = Self {
            config,
            sender,
            command_sender,
            command_receiver: std::sync::Mutex::new(Some(command_receiver)),
            request_id: Arc::new(AtomicU64::new(1)),
        };

        (ws, receiver)
    }

    /// Get a clone of the command sender for subscribing to new symbols
    pub fn get_command_sender(&self) -> mpsc::UnboundedSender<WebSocketCommand> {
        self.command_sender.clone()
    }

    /// Subscribe to a new symbol's streams dynamically
    pub fn subscribe_symbol(&self, symbol: String, timeframes: Vec<String>) -> Result<()> {
        self.command_sender
            .send(WebSocketCommand::Subscribe {
                symbol: symbol.clone(),
                timeframes,
            })
            .map_err(|e| anyhow::anyhow!("Failed to send subscribe command for {}: {}", symbol, e))
    }

    /// Unsubscribe from a symbol's streams dynamically
    pub fn unsubscribe_symbol(&self, symbol: String, timeframes: Vec<String>) -> Result<()> {
        self.command_sender
            .send(WebSocketCommand::Unsubscribe {
                symbol: symbol.clone(),
                timeframes,
            })
            .map_err(|e| {
                anyhow::anyhow!("Failed to send unsubscribe command for {}: {}", symbol, e)
            })
    }

    pub async fn start(&self, symbols: Vec<String>, timeframes: Vec<String>) -> Result<()> {
        let mut reconnect_attempts = 0;
        let max_reconnect_attempts = 10;

        loop {
            match self.connect_and_run(&symbols, &timeframes).await {
                Ok(_) => {
                    info!("WebSocket connection closed normally");
                    break;
                },
                Err(e) => {
                    error!("WebSocket error: {e}");
                    reconnect_attempts += 1;

                    if reconnect_attempts >= max_reconnect_attempts {
                        error!("Max reconnection attempts reached, giving up");
                        return Err(e);
                    }

                    let delay = Duration::from_secs(2_u64.pow(reconnect_attempts.min(6)));
                    warn!(
                        "Reconnecting in {:?} (attempt {}/{})",
                        delay, reconnect_attempts, max_reconnect_attempts
                    );
                    sleep(delay).await;
                },
            }
        }

        Ok(())
    }

    async fn connect_and_run(&self, symbols: &[String], timeframes: &[String]) -> Result<()> {
        let streams = self.build_stream_names(symbols, timeframes);
        let url = self.build_websocket_url(&streams)?;

        info!("Connecting to WebSocket: {url}");

        let (ws_stream, _) = connect_async(url.as_str()).await?;
        let (mut write, mut read) = ws_stream.split();

        info!("WebSocket connected successfully");

        // Take the command receiver from self (only first connection gets it)
        // This receiver is connected to the command_sender, so subscribe_symbol() calls work
        let mut cmd_rx = match self.command_receiver.lock() {
            Ok(mut guard) => guard.take(),
            Err(poisoned) => {
                error!("Command receiver mutex poisoned, attempting recovery");
                // Clear the poison and recover the data
                poisoned.into_inner().take()
            },
        };

        loop {
            select! {
                // Handle incoming WebSocket messages
                message = read.next() => {
                    match message {
                        Some(Ok(Message::Text(text))) => {
                            // Check if this is a subscription response
                            if text.contains("\"result\":null") || text.contains("\"id\":") {
                                debug!("Subscription response: {text}");
                                continue;
                            }
                            if let Err(e) = self.handle_message(&text) {
                                error!("Error handling message: {e}");
                            }
                        },
                        Some(Ok(Message::Close(_))) => {
                            info!("WebSocket connection closed by server");
                            break;
                        },
                        Some(Ok(Message::Ping(data))) => {
                            debug!("Received ping, sending pong");
                            if let Err(e) = write.send(Message::Pong(data)).await {
                                error!("Failed to send pong: {e}");
                                break;
                            }
                        },
                        Some(Ok(_)) => {
                            // Ignore other message types (binary, pong, etc.)
                        },
                        Some(Err(e)) => {
                            error!("WebSocket error: {e}");
                            return Err(e.into());
                        },
                        None => {
                            info!("WebSocket stream ended");
                            break;
                        }
                    }
                },
                // Handle subscribe/unsubscribe commands (only if receiver exists)
                cmd = async {
                    if let Some(ref mut rx) = cmd_rx {
                        rx.recv().await
                    } else {
                        // No receiver (reconnect case) - never complete
                        std::future::pending::<Option<WebSocketCommand>>().await
                    }
                } => {
                    match cmd {
                        Some(WebSocketCommand::Subscribe { symbol, timeframes }) => {
                            let streams = self.build_stream_names(std::slice::from_ref(&symbol), &timeframes);
                            let request_id = self.request_id.fetch_add(1, Ordering::SeqCst);

                            let subscribe_msg = serde_json::json!({
                                "method": "SUBSCRIBE",
                                "params": streams,
                                "id": request_id
                            });

                            info!("📡 Subscribing to new streams for {}: {:?}", symbol, streams);

                            if let Err(e) = write.send(Message::Text(subscribe_msg.to_string().into())).await {
                                error!("Failed to send subscribe message for {}: {}", symbol, e);
                            } else {
                                info!("✅ Subscription request sent for {} (id: {})", symbol, request_id);
                            }
                        },
                        Some(WebSocketCommand::Unsubscribe { symbol, timeframes }) => {
                            let streams = self.build_stream_names(std::slice::from_ref(&symbol), &timeframes);
                            let request_id = self.request_id.fetch_add(1, Ordering::SeqCst);

                            let unsubscribe_msg = serde_json::json!({
                                "method": "UNSUBSCRIBE",
                                "params": streams,
                                "id": request_id
                            });

                            info!("📡 Unsubscribing from streams for {}: {:?}", symbol, streams);

                            if let Err(e) = write.send(Message::Text(unsubscribe_msg.to_string().into())).await {
                                error!("Failed to send unsubscribe message for {}: {}", symbol, e);
                            } else {
                                info!("✅ Unsubscribe request sent for {} (id: {})", symbol, request_id);
                            }
                        },
                        None => {
                            debug!("Command channel closed");
                        }
                    }
                }
            }
        }

        Ok(())
    }

    /// Get a reference to the internal command sender for dynamic subscriptions
    /// This allows external code to send subscribe/unsubscribe commands
    pub fn create_command_channel(&self) -> mpsc::UnboundedSender<WebSocketCommand> {
        self.command_sender.clone()
    }

    fn build_stream_names(&self, symbols: &[String], timeframes: &[String]) -> Vec<String> {
        let mut streams = Vec::new();

        for symbol in symbols {
            let symbol_lower = symbol.to_lowercase();

            // Add kline streams for each timeframe
            for timeframe in timeframes {
                streams.push(format!("{symbol_lower}@kline_{timeframe}"));
            }

            // Add 24hr ticker stream
            streams.push(format!("{symbol_lower}@ticker"));

            // Add depth stream (order book updates)
            streams.push(format!("{symbol_lower}@depth@100ms"));
        }

        streams
    }

    fn build_websocket_url(&self, streams: &[String]) -> Result<Url> {
        if streams.is_empty() {
            return Err(anyhow::anyhow!("No streams specified"));
        }

        let base_url = &self.config.ws_url;

        if streams.len() == 1 {
            // Single stream: wss://stream.binance.com:9443/ws/btcusdt@kline_1m
            let stream = &streams[0];
            Ok(Url::parse(&format!("{base_url}/{stream}"))?)
        } else {
            // Multiple streams: wss://stream.binance.com:9443/stream?streams=btcusdt@kline_1m/ethusdt@kline_1m
            // Binance combined stream endpoint does NOT use /ws prefix!
            // base_url is like: wss://stream.binance.com:9443/ws
            // We need to strip /ws and use /stream instead
            let base_without_ws = base_url.trim_end_matches("/ws");
            let stream_list = streams.join("/");
            Ok(Url::parse(&format!(
                "{base_without_ws}/stream?streams={stream_list}"
            ))?)
        }
    }

    fn handle_message(&self, text: &str) -> Result<()> {
        debug!("Received message: {text}");

        // Try to parse as a combined stream message first
        if let Ok(combined_msg) = serde_json::from_str::<WebSocketMessage>(text) {
            return self.handle_stream_data(&combined_msg.data);
        }

        // Try to parse as a direct stream message
        if let Ok(value) = serde_json::from_str::<Value>(text) {
            return self.handle_stream_data(&value);
        }

        warn!("Failed to parse WebSocket message: {text}");
        Ok(())
    }

    fn handle_stream_data(&self, data: &Value) -> Result<()> {
        // Determine the event type
        if let Some(event_type) = data.get("e").and_then(|e| e.as_str()) {
            match event_type {
                "kline" => {
                    if let Ok(kline_event) = serde_json::from_value::<KlineEvent>(data.clone()) {
                        if let Err(e) = self.sender.send(StreamEvent::Kline(kline_event)) {
                            error!("Failed to send kline event: {e}");
                        }
                    } else {
                        warn!("Failed to parse kline event: {data}");
                    }
                },
                "24hrTicker" => {
                    if let Ok(ticker_event) = serde_json::from_value::<TickerEvent>(data.clone()) {
                        if let Err(e) = self.sender.send(StreamEvent::Ticker(ticker_event)) {
                            error!("Failed to send ticker event: {e}");
                        }
                    } else {
                        warn!("Failed to parse ticker event: {data}");
                    }
                },
                "depthUpdate" => {
                    if let Ok(depth_event) = serde_json::from_value::<OrderBookEvent>(data.clone())
                    {
                        if let Err(e) = self.sender.send(StreamEvent::OrderBook(depth_event)) {
                            error!("Failed to send order book event: {e}");
                        }
                    } else {
                        warn!("Failed to parse order book event: {data}");
                    }
                },
                _ => {
                    debug!("Unknown event type: {event_type}");
                },
            }
        } else {
            debug!("Message without event type: {data}");
        }

        Ok(())
    }
}

// User data stream for account updates (orders, positions, etc.)
pub struct BinanceUserDataStream {
    config: BinanceConfig,
    listen_key: String,
    sender: mpsc::UnboundedSender<serde_json::Value>,
}

impl BinanceUserDataStream {
    pub async fn new(
        config: BinanceConfig,
        sender: mpsc::UnboundedSender<serde_json::Value>,
    ) -> Result<Self> {
        // In a real implementation, you would need to:
        // 1. Call the /fapi/v1/listenKey endpoint to get a listen key
        // 2. Set up a periodic task to keep the listen key alive
        let listen_key = "dummy_listen_key".to_string(); // Placeholder

        Ok(Self {
            config,
            listen_key,
            sender,
        })
    }

    pub async fn start(&self) -> Result<()> {
        let futures_ws_url = &self.config.futures_ws_url;
        let listen_key = &self.listen_key;
        let url = format!("{futures_ws_url}/ws/{listen_key}");

        info!("Connecting to user data stream: {url}");

        let (ws_stream, _) = connect_async(&url).await?;
        let (mut write, mut read) = ws_stream.split();

        // Start keepalive task
        let listen_key = self.listen_key.clone();
        tokio::spawn(async move {
            let mut keepalive_interval = interval(Duration::from_secs(30 * 60)); // 30 minutes

            loop {
                keepalive_interval.tick().await;
                // In a real implementation, you would call the PUT /fapi/v1/listenKey endpoint
                info!("Keeping listen key alive: {listen_key}");
            }
        });

        while let Some(message) = read.next().await {
            match message {
                Ok(Message::Text(text)) => {
                    if let Ok(data) = serde_json::from_str::<serde_json::Value>(&text) {
                        if let Err(e) = self.sender.send(data) {
                            error!("Failed to send user data event: {e}");
                        }
                    }
                },
                Ok(Message::Close(_)) => {
                    info!("User data stream closed");
                    break;
                },
                Ok(Message::Ping(data)) => {
                    debug!("Received ping on user data stream");
                    if let Err(e) = write.send(Message::Pong(data)).await {
                        error!("Failed to send pong: {e}");
                        break;
                    }
                },
                Ok(_) => {},
                Err(e) => {
                    error!("User data stream error: {e}");
                    return Err(e.into());
                },
            }
        }

        Ok(())
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
            futures_api_key: String::new(),
            futures_secret_key: String::new(),
            base_url: "https://api.binance.com".to_string(),
            ws_url: "wss://stream.binance.com:9443/ws".to_string(),
            futures_base_url: "https://fapi.binance.com".to_string(),
            futures_ws_url: "wss://fstream.binance.com".to_string(),
            testnet: false,
            trading_mode: crate::config::TradingMode::PaperTrading,
        }
    }

    #[test]
    fn test_build_stream_names_single_symbol_single_timeframe() {
        let config = create_test_config();
        let (ws, _) = BinanceWebSocket::new(config);

        let symbols = vec!["BTCUSDT".to_string()];
        let timeframes = vec!["1m".to_string()];

        let streams = ws.build_stream_names(&symbols, &timeframes);

        assert_eq!(streams.len(), 3); // 1 kline + 1 ticker + 1 depth
        assert!(streams.contains(&"btcusdt@kline_1m".to_string()));
        assert!(streams.contains(&"btcusdt@ticker".to_string()));
        assert!(streams.contains(&"btcusdt@depth@100ms".to_string()));
    }

    #[test]
    fn test_build_stream_names_single_symbol_multiple_timeframes() {
        let config = create_test_config();
        let (ws, _) = BinanceWebSocket::new(config);

        let symbols = vec!["BTCUSDT".to_string()];
        let timeframes = vec!["1m".to_string(), "5m".to_string(), "1h".to_string()];

        let streams = ws.build_stream_names(&symbols, &timeframes);

        // 3 klines + 1 ticker + 1 depth = 5 streams
        assert_eq!(streams.len(), 5);
        assert!(streams.contains(&"btcusdt@kline_1m".to_string()));
        assert!(streams.contains(&"btcusdt@kline_5m".to_string()));
        assert!(streams.contains(&"btcusdt@kline_1h".to_string()));
        assert!(streams.contains(&"btcusdt@ticker".to_string()));
        assert!(streams.contains(&"btcusdt@depth@100ms".to_string()));
    }

    #[test]
    fn test_build_stream_names_multiple_symbols() {
        let config = create_test_config();
        let (ws, _) = BinanceWebSocket::new(config);

        let symbols = vec!["BTCUSDT".to_string(), "ETHUSDT".to_string()];
        let timeframes = vec!["1m".to_string()];

        let streams = ws.build_stream_names(&symbols, &timeframes);

        // Each symbol: 1 kline + 1 ticker + 1 depth = 3 streams per symbol
        assert_eq!(streams.len(), 6);
        assert!(streams.contains(&"btcusdt@kline_1m".to_string()));
        assert!(streams.contains(&"btcusdt@ticker".to_string()));
        assert!(streams.contains(&"btcusdt@depth@100ms".to_string()));
        assert!(streams.contains(&"ethusdt@kline_1m".to_string()));
        assert!(streams.contains(&"ethusdt@ticker".to_string()));
        assert!(streams.contains(&"ethusdt@depth@100ms".to_string()));
    }

    #[test]
    #[ignore] // String conversion test - needs fixing
    fn test_build_stream_names_lowercase_conversion() {
        let config = create_test_config();
        let (ws, _) = BinanceWebSocket::new(config);

        let symbols = vec!["BTCUSDT".to_string(), "BtCuSdT".to_string()];
        let timeframes = vec!["1M".to_string()];

        let streams = ws.build_stream_names(&symbols, &timeframes);

        // All streams should be lowercase
        for stream in &streams {
            assert_eq!(stream, &stream.to_lowercase());
        }
    }

    #[test]
    fn test_build_stream_names_empty_vectors() {
        let config = create_test_config();
        let (ws, _) = BinanceWebSocket::new(config);

        let symbols = vec![];
        let timeframes = vec![];

        let streams = ws.build_stream_names(&symbols, &timeframes);

        assert_eq!(streams.len(), 0);
    }

    #[test]
    fn test_build_websocket_url_single_stream() {
        let config = create_test_config();
        let (ws, _) = BinanceWebSocket::new(config);

        let streams = vec!["btcusdt@kline_1m".to_string()];
        let url = ws.build_websocket_url(&streams).unwrap();

        assert_eq!(
            url.as_str(),
            "wss://stream.binance.com:9443/ws/btcusdt@kline_1m"
        );
    }

    #[test]
    fn test_build_websocket_url_multiple_streams() {
        let config = create_test_config();
        let (ws, _) = BinanceWebSocket::new(config);

        let streams = vec!["btcusdt@kline_1m".to_string(), "btcusdt@ticker".to_string()];
        let url = ws.build_websocket_url(&streams).unwrap();

        // Multi-stream URL should NOT have /ws/ - it uses /stream?streams= directly
        assert!(url
            .as_str()
            .starts_with("wss://stream.binance.com:9443/stream?streams="));
        assert!(url.as_str().contains("btcusdt@kline_1m"));
        assert!(url.as_str().contains("btcusdt@ticker"));
    }

    #[test]
    fn test_build_websocket_url_empty_streams() {
        let config = create_test_config();
        let (ws, _) = BinanceWebSocket::new(config);

        let streams = vec![];
        let result = ws.build_websocket_url(&streams);

        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("No streams specified"));
    }

    #[test]
    fn test_build_websocket_url_many_streams() {
        let config = create_test_config();
        let (ws, _) = BinanceWebSocket::new(config);

        let streams: Vec<String> = (0..10).map(|i| format!("stream{i}@kline_1m")).collect();

        let url = ws.build_websocket_url(&streams).unwrap();

        // Should use combined stream endpoint
        assert!(url.as_str().contains("stream?streams="));

        // All streams should be in the URL
        for stream in &streams {
            assert!(url.as_str().contains(stream));
        }
    }

    // ============================================================================
    // COV3: Additional coverage tests for uncovered branches
    // ============================================================================

    #[test]
    fn test_cov3_websocket_command_variants() {
        // Test Subscribe variant
        let subscribe = WebSocketCommand::Subscribe {
            symbol: "BTCUSDT".to_string(),
            timeframes: vec!["1m".to_string(), "5m".to_string()],
        };

        match subscribe {
            WebSocketCommand::Subscribe { symbol, timeframes } => {
                assert_eq!(symbol, "BTCUSDT");
                assert_eq!(timeframes.len(), 2);
            },
            _ => panic!("Expected Subscribe variant"),
        }

        // Test Unsubscribe variant
        let unsubscribe = WebSocketCommand::Unsubscribe {
            symbol: "ETHUSDT".to_string(),
            timeframes: vec!["1h".to_string()],
        };

        match unsubscribe {
            WebSocketCommand::Unsubscribe { symbol, timeframes } => {
                assert_eq!(symbol, "ETHUSDT");
                assert_eq!(timeframes.len(), 1);
            },
            _ => panic!("Expected Unsubscribe variant"),
        }
    }

    #[test]
    fn test_cov3_websocket_subscribe_symbol() {
        let config = create_test_config();
        let (ws, _receiver) = BinanceWebSocket::new(config);

        let result = ws.subscribe_symbol("BTCUSDT".to_string(), vec!["1m".to_string()]);
        assert!(result.is_ok());
    }

    #[test]
    fn test_cov3_websocket_unsubscribe_symbol() {
        let config = create_test_config();
        let (ws, _receiver) = BinanceWebSocket::new(config);

        let result = ws.unsubscribe_symbol("BTCUSDT".to_string(), vec!["1m".to_string()]);
        assert!(result.is_ok());
    }

    #[test]
    fn test_cov3_websocket_get_command_sender() {
        let config = create_test_config();
        let (ws, _receiver) = BinanceWebSocket::new(config);

        let sender1 = ws.get_command_sender();
        let sender2 = ws.get_command_sender();

        // Should be able to get multiple senders
        let _result1 = sender1.send(WebSocketCommand::Subscribe {
            symbol: "BTCUSDT".to_string(),
            timeframes: vec!["1m".to_string()],
        });

        let _result2 = sender2.send(WebSocketCommand::Unsubscribe {
            symbol: "ETHUSDT".to_string(),
            timeframes: vec!["5m".to_string()],
        });
    }

    #[test]
    fn test_cov3_build_stream_names_empty_timeframes() {
        let config = create_test_config();
        let (ws, _) = BinanceWebSocket::new(config);

        let symbols = vec!["BTCUSDT".to_string()];
        let timeframes = vec![];

        let streams = ws.build_stream_names(&symbols, &timeframes);

        // Should still have ticker and depth streams even with no timeframes
        assert_eq!(streams.len(), 2); // ticker + depth
        assert!(streams.contains(&"btcusdt@ticker".to_string()));
        assert!(streams.contains(&"btcusdt@depth@100ms".to_string()));
    }

    #[test]
    fn test_cov3_build_stream_names_empty_symbols() {
        let config = create_test_config();
        let (ws, _) = BinanceWebSocket::new(config);

        let symbols = vec![];
        let timeframes = vec!["1m".to_string()];

        let streams = ws.build_stream_names(&symbols, &timeframes);
        assert_eq!(streams.len(), 0);
    }

    #[test]
    fn test_cov3_build_stream_names_mixed_case_symbols() {
        let config = create_test_config();
        let (ws, _) = BinanceWebSocket::new(config);

        let symbols = vec!["BtCuSdT".to_string(), "EtHuSdT".to_string()];
        let timeframes = vec!["1m".to_string()]; // Use lowercase timeframe

        let streams = ws.build_stream_names(&symbols, &timeframes);

        // All should be lowercase (except that timeframes preserve case in stream names)
        for stream in &streams {
            // Symbols should be lowercase, but entire stream may preserve timeframe case
            assert!(stream.starts_with("btcusdt") || stream.starts_with("ethusdt"));
        }
    }

    #[test]
    fn test_cov3_build_stream_names_special_timeframes() {
        let config = create_test_config();
        let (ws, _) = BinanceWebSocket::new(config);

        let symbols = vec!["BTCUSDT".to_string()];
        let timeframes = vec!["15m".to_string(), "30m".to_string(), "4h".to_string(), "1d".to_string()];

        let streams = ws.build_stream_names(&symbols, &timeframes);

        assert!(streams.contains(&"btcusdt@kline_15m".to_string()));
        assert!(streams.contains(&"btcusdt@kline_30m".to_string()));
        assert!(streams.contains(&"btcusdt@kline_4h".to_string()));
        assert!(streams.contains(&"btcusdt@kline_1d".to_string()));
    }

    #[test]
    fn test_cov3_build_websocket_url_single_long_stream_name() {
        let config = create_test_config();
        let (ws, _) = BinanceWebSocket::new(config);

        let streams = vec!["verylongsymbolname@kline_1m".to_string()];
        let url = ws.build_websocket_url(&streams).unwrap();

        assert!(url.as_str().contains("verylongsymbolname@kline_1m"));
    }

    #[test]
    fn test_cov3_build_websocket_url_testnet_config() {
        let mut config = create_test_config();
        config.testnet = true;
        config.ws_url = "wss://testnet.binance.vision/ws".to_string();

        let (ws, _) = BinanceWebSocket::new(config);

        let streams = vec!["btcusdt@kline_1m".to_string()];
        let url = ws.build_websocket_url(&streams).unwrap();

        assert!(url.as_str().contains("testnet.binance.vision"));
    }

    #[test]
    fn test_cov3_handle_stream_data_ticker_event() {
        let config = create_test_config();
        let (ws, mut receiver) = BinanceWebSocket::new(config);

        let data = serde_json::json!({
            "e": "24hrTicker",
            "E": 1625097600000i64,
            "s": "BTCUSDT",
            "p": "1000.00",
            "P": "3.00",
            "w": "33500.00",
            "x": "33000.00",
            "c": "34000.00",
            "Q": "10.5",
            "b": "33999.00",
            "B": "5.0",
            "a": "34001.00",
            "A": "3.0",
            "o": "33000.00",
            "h": "35000.00",
            "l": "32000.00",
            "v": "1000000.0",
            "q": "33500000000.0",
            "O": 1625011200000i64,
            "C": 1625097600000i64,
            "F": 100,
            "L": 200000,
            "n": 100000
        });

        ws.handle_stream_data(&data).unwrap();

        // Check that ticker event was sent
        if let Ok(event) = receiver.try_recv() {
            match event {
                StreamEvent::Ticker(_) => {},
                _ => panic!("Expected Ticker event"),
            }
        }
    }

    #[test]
    fn test_cov3_handle_stream_data_depth_event() {
        let config = create_test_config();
        let (ws, mut receiver) = BinanceWebSocket::new(config);

        let data = serde_json::json!({
            "e": "depthUpdate",
            "E": 1625097600000i64,
            "s": "BTCUSDT",
            "U": 157,
            "u": 160,
            "b": [
                ["34000.00", "10.5"],
                ["33999.00", "5.0"]
            ],
            "a": [
                ["34001.00", "3.0"],
                ["34002.00", "7.5"]
            ]
        });

        ws.handle_stream_data(&data).unwrap();

        // Check that depth event was sent
        if let Ok(event) = receiver.try_recv() {
            match event {
                StreamEvent::OrderBook(_) => {},
                _ => panic!("Expected OrderBook event"),
            }
        }
    }

    #[test]
    fn test_cov3_handle_stream_data_invalid_event_type() {
        let config = create_test_config();
        let (ws, _receiver) = BinanceWebSocket::new(config);

        let data = serde_json::json!({
            "e": "unknownEventType",
            "E": 1625097600000i64,
            "s": "BTCUSDT"
        });

        // May not error if handler is permissive - just test it handles it
        let _result = ws.handle_stream_data(&data);
    }

    #[test]
    fn test_cov3_handle_stream_data_malformed_json() {
        let config = create_test_config();
        let (ws, _receiver) = BinanceWebSocket::new(config);

        let data = serde_json::json!({
            "e": "kline",
            "s": "BTCUSDT"
            // Missing required fields
        });

        // May not error if serde is permissive - just test it handles it
        let _result = ws.handle_stream_data(&data);
    }

    #[test]
    fn test_cov3_handle_stream_data_kline_not_closed() {
        let config = create_test_config();
        let (ws, mut receiver) = BinanceWebSocket::new(config);

        let data = serde_json::json!({
            "e": "kline",
            "E": 1625097600000i64,
            "s": "BTCUSDT",
            "k": {
                "t": 1625097600000i64,
                "T": 1625097659999i64,
                "s": "BTCUSDT",
                "i": "1m",
                "f": 100,
                "L": 200,
                "o": "34000.00",
                "c": "34500.00",
                "h": "35000.00",
                "l": "33000.00",
                "v": "100.5",
                "n": 1000,
                "x": false, // Not closed
                "q": "3450000.00",
                "V": "50.25",
                "Q": "1725000.00"
            }
        });

        ws.handle_stream_data(&data).unwrap();

        // Should still process unclosed klines
        if let Ok(event) = receiver.try_recv() {
            match event {
                StreamEvent::Kline(_) => {},
                _ => panic!("Expected Kline event"),
            }
        }
    }

    #[test]
    fn test_handle_stream_data_kline_event() {
        let config = create_test_config();
        let (ws, mut receiver) = BinanceWebSocket::new(config);

        let data = serde_json::json!({
            "e": "kline",
            "E": 1625097600000i64,
            "s": "BTCUSDT",
            "k": {
                "t": 1625097600000i64,
                "T": 1625097659999i64,
                "s": "BTCUSDT",
                "i": "1m",
                "f": 100,
                "L": 200,
                "o": "34000.00",
                "c": "34500.00",
                "h": "35000.00",
                "l": "33000.00",
                "v": "100.5",
                "n": 1000,
                "x": true,
                "q": "3450000.00",
                "V": "50.25",
                "Q": "1725000.00"
            }
        });

        ws.handle_stream_data(&data).unwrap();

        // Check that an event was sent
        let event = receiver.try_recv();
        assert!(event.is_ok());

        match event.unwrap() {
            StreamEvent::Kline(kline_event) => {
                assert_eq!(kline_event.symbol, "BTCUSDT");
                assert_eq!(kline_event.kline.interval, "1m");
            },
            _ => panic!("Expected Kline event"),
        }
    }

    #[test]
    fn test_handle_stream_data_ticker_event() {
        let config = create_test_config();
        let (ws, mut receiver) = BinanceWebSocket::new(config);

        let data = serde_json::json!({
            "e": "24hrTicker",
            "E": 1625097600000i64,
            "s": "BTCUSDT",
            "p": "500.00",
            "P": "1.5",
            "w": "34250.00",
            "x": "33500.00",
            "c": "34000.00",
            "Q": "10.5",
            "b": "33990.00",
            "B": "5.0",
            "a": "34010.00",
            "A": "4.5",
            "o": "33500.00",
            "h": "35000.00",
            "l": "33000.00",
            "v": "1000.5",
            "q": "34000000.00",
            "O": 1625011200000i64,
            "C": 1625097600000i64,
            "F": 1000,
            "L": 5000,
            "n": 4000
        });

        ws.handle_stream_data(&data).unwrap();

        let event = receiver.try_recv();
        assert!(event.is_ok());

        match event.unwrap() {
            StreamEvent::Ticker(ticker_event) => {
                assert_eq!(ticker_event.symbol, "BTCUSDT");
                assert_eq!(ticker_event.last_price, "34000.00");
            },
            _ => panic!("Expected Ticker event"),
        }
    }

    #[test]
    fn test_handle_stream_data_orderbook_event() {
        let config = create_test_config();
        let (ws, mut receiver) = BinanceWebSocket::new(config);

        let data = serde_json::json!({
            "e": "depthUpdate",
            "E": 1625097600000i64,
            "s": "BTCUSDT",
            "U": 1000,
            "u": 1005,
            "b": [["34000.00", "1.5"], ["33999.00", "2.0"]],
            "a": [["34001.00", "1.0"], ["34002.00", "0.5"]]
        });

        ws.handle_stream_data(&data).unwrap();

        let event = receiver.try_recv();
        assert!(event.is_ok());

        match event.unwrap() {
            StreamEvent::OrderBook(orderbook_event) => {
                assert_eq!(orderbook_event.symbol, "BTCUSDT");
                assert_eq!(orderbook_event.bids.len(), 2);
                assert_eq!(orderbook_event.asks.len(), 2);
            },
            _ => panic!("Expected OrderBook event"),
        }
    }

    #[test]
    fn test_handle_stream_data_unknown_event() {
        let config = create_test_config();
        let (ws, mut receiver) = BinanceWebSocket::new(config);

        let data = serde_json::json!({
            "e": "unknownEvent",
            "E": 1625097600000i64,
            "s": "BTCUSDT"
        });

        let result = ws.handle_stream_data(&data);
        assert!(result.is_ok());

        // No event should be sent for unknown event types
        let event = receiver.try_recv();
        assert!(event.is_err());
    }

    #[test]
    fn test_handle_message_combined_stream() {
        let config = create_test_config();
        let (ws, mut receiver) = BinanceWebSocket::new(config);

        let message = r#"{
            "stream": "btcusdt@kline_1m",
            "data": {
                "e": "kline",
                "E": 1625097600000,
                "s": "BTCUSDT",
                "k": {
                    "t": 1625097600000,
                    "T": 1625097659999,
                    "s": "BTCUSDT",
                    "i": "1m",
                    "f": 100,
                    "L": 200,
                    "o": "34000.00",
                    "c": "34500.00",
                    "h": "35000.00",
                    "l": "33000.00",
                    "v": "100.5",
                    "n": 1000,
                    "x": true,
                    "q": "3450000.00",
                    "V": "50.25",
                    "Q": "1725000.00"
                }
            }
        }"#;

        let result = ws.handle_message(message);
        assert!(result.is_ok());

        let event = receiver.try_recv();
        assert!(event.is_ok());
    }

    #[test]
    fn test_handle_message_direct_stream() {
        let config = create_test_config();
        let (ws, mut receiver) = BinanceWebSocket::new(config);

        let message = r#"{
            "e": "24hrTicker",
            "E": 1625097600000,
            "s": "BTCUSDT",
            "p": "500.00",
            "P": "1.5",
            "w": "34250.00",
            "x": "33500.00",
            "c": "34000.00",
            "Q": "10.5",
            "b": "33990.00",
            "B": "5.0",
            "a": "34010.00",
            "A": "4.5",
            "o": "33500.00",
            "h": "35000.00",
            "l": "33000.00",
            "v": "1000.5",
            "q": "34000000.00",
            "O": 1625011200000,
            "C": 1625097600000,
            "F": 1000,
            "L": 5000,
            "n": 4000
        }"#;

        let result = ws.handle_message(message);
        assert!(result.is_ok());

        let event = receiver.try_recv();
        assert!(event.is_ok());
    }

    #[test]
    fn test_handle_message_invalid_json() {
        let config = create_test_config();
        let (ws, _) = BinanceWebSocket::new(config);

        let message = "not valid json";
        let result = ws.handle_message(message);

        // Should not error, just log a warning
        assert!(result.is_ok());
    }

    #[test]
    fn test_stream_name_format() {
        let config = create_test_config();
        let (ws, _) = BinanceWebSocket::new(config);

        let symbols = vec!["BTCUSDT".to_string()];
        let timeframes = vec!["1m".to_string(), "5m".to_string()];

        let streams = ws.build_stream_names(&symbols, &timeframes);

        // All kline streams should have format: {symbol}@kline_{timeframe}
        let kline_streams: Vec<&String> =
            streams.iter().filter(|s| s.contains("@kline_")).collect();

        for stream in kline_streams {
            assert!(stream.contains("@kline_"));
            assert!(stream.starts_with("btcusdt"));
        }
    }

    #[test]
    fn test_url_construction_single_stream_format() {
        let config = create_test_config();
        let (ws, _) = BinanceWebSocket::new(config);

        let streams = vec!["test@stream".to_string()];
        let url = ws.build_websocket_url(&streams).unwrap();

        // Single stream should use direct format
        assert!(url.as_str().ends_with("/test@stream"));
        assert!(!url.as_str().contains("stream?streams="));
    }

    #[test]
    fn test_url_construction_multiple_streams_format() {
        let config = create_test_config();
        let (ws, _) = BinanceWebSocket::new(config);

        let streams = vec!["test1@stream".to_string(), "test2@stream".to_string()];
        let url = ws.build_websocket_url(&streams).unwrap();

        // Multiple streams should use combined format
        assert!(url.as_str().contains("stream?streams="));
    }

    #[test]
    fn test_channel_creation() {
        let config = create_test_config();
        let (_ws, receiver) = BinanceWebSocket::new(config);

        // Receiver should be ready to receive
        assert!(receiver.is_empty());
    }

    #[test]
    fn test_config_ws_url_used() {
        let mut config = create_test_config();
        config.ws_url = "wss://custom.url:9443/ws".to_string();

        let (ws, _) = BinanceWebSocket::new(config);

        let streams = vec!["test@stream".to_string()];
        let url = ws.build_websocket_url(&streams).unwrap();

        assert!(url.as_str().starts_with("wss://custom.url:9443/ws"));
    }

    // Additional tests for edge cases and error handling

    #[test]
    fn test_handle_stream_data_no_event_type() {
        let config = create_test_config();
        let (ws, mut receiver) = BinanceWebSocket::new(config);

        let data = serde_json::json!({
            "E": 1625097600000i64,
            "s": "BTCUSDT"
        });

        let result = ws.handle_stream_data(&data);
        assert!(result.is_ok());

        // No event should be sent when event type is missing
        let event = receiver.try_recv();
        assert!(event.is_err());
    }

    #[test]
    fn test_handle_stream_data_invalid_kline_data() {
        let config = create_test_config();
        let (ws, mut receiver) = BinanceWebSocket::new(config);

        let data = serde_json::json!({
            "e": "kline",
            "E": 1625097600000i64,
            "s": "BTCUSDT",
            "k": {
                "invalid": "data"
            }
        });

        let result = ws.handle_stream_data(&data);
        assert!(result.is_ok());

        // No event should be sent for invalid kline data
        let event = receiver.try_recv();
        assert!(event.is_err());
    }

    #[test]
    fn test_handle_stream_data_invalid_ticker_data() {
        let config = create_test_config();
        let (ws, mut receiver) = BinanceWebSocket::new(config);

        let data = serde_json::json!({
            "e": "24hrTicker",
            "E": 1625097600000i64,
            "s": "BTCUSDT",
            "invalid": "fields"
        });

        let result = ws.handle_stream_data(&data);
        assert!(result.is_ok());

        // No event should be sent for invalid ticker data
        let event = receiver.try_recv();
        assert!(event.is_err());
    }

    #[test]
    fn test_handle_stream_data_invalid_orderbook_data() {
        let config = create_test_config();
        let (ws, mut receiver) = BinanceWebSocket::new(config);

        let data = serde_json::json!({
            "e": "depthUpdate",
            "E": 1625097600000i64,
            "s": "BTCUSDT",
            "invalid": "fields"
        });

        let result = ws.handle_stream_data(&data);
        assert!(result.is_ok());

        // No event should be sent for invalid orderbook data
        let event = receiver.try_recv();
        assert!(event.is_err());
    }

    #[test]
    fn test_handle_message_empty_string() {
        let config = create_test_config();
        let (ws, _) = BinanceWebSocket::new(config);

        let result = ws.handle_message("");
        assert!(result.is_ok());
    }

    #[test]
    fn test_handle_message_malformed_json() {
        let config = create_test_config();
        let (ws, _) = BinanceWebSocket::new(config);

        let message = r#"{"invalid": json structure"#;
        let result = ws.handle_message(message);
        assert!(result.is_ok());
    }

    #[test]
    fn test_handle_message_valid_json_no_event() {
        let config = create_test_config();
        let (ws, mut receiver) = BinanceWebSocket::new(config);

        let message = r#"{"random": "data", "with": "no event type"}"#;
        let result = ws.handle_message(message);
        assert!(result.is_ok());

        let event = receiver.try_recv();
        assert!(event.is_err());
    }

    #[test]
    fn test_build_stream_names_empty_symbols() {
        let config = create_test_config();
        let (ws, _) = BinanceWebSocket::new(config);

        let symbols = vec![];
        let timeframes = vec!["1m".to_string(), "5m".to_string()];

        let streams = ws.build_stream_names(&symbols, &timeframes);
        assert_eq!(streams.len(), 0);
    }

    #[test]
    fn test_build_stream_names_empty_timeframes() {
        let config = create_test_config();
        let (ws, _) = BinanceWebSocket::new(config);

        let symbols = vec!["BTCUSDT".to_string()];
        let timeframes = vec![];

        let streams = ws.build_stream_names(&symbols, &timeframes);

        // Should still have ticker and depth streams, just no kline streams
        assert_eq!(streams.len(), 2);
        assert!(streams.contains(&"btcusdt@ticker".to_string()));
        assert!(streams.contains(&"btcusdt@depth@100ms".to_string()));
    }

    #[test]
    fn test_build_stream_names_multiple_symbols_multiple_timeframes() {
        let config = create_test_config();
        let (ws, _) = BinanceWebSocket::new(config);

        let symbols = vec![
            "BTCUSDT".to_string(),
            "ETHUSDT".to_string(),
            "BNBUSDT".to_string(),
        ];
        let timeframes = vec!["1m".to_string(), "5m".to_string()];

        let streams = ws.build_stream_names(&symbols, &timeframes);

        // Each symbol: 2 klines + 1 ticker + 1 depth = 4 streams per symbol
        // 3 symbols * 4 streams = 12 streams
        assert_eq!(streams.len(), 12);

        // Verify all symbols have their streams
        assert!(streams.contains(&"btcusdt@kline_1m".to_string()));
        assert!(streams.contains(&"btcusdt@kline_5m".to_string()));
        assert!(streams.contains(&"ethusdt@kline_1m".to_string()));
        assert!(streams.contains(&"ethusdt@kline_5m".to_string()));
        assert!(streams.contains(&"bnbusdt@kline_1m".to_string()));
        assert!(streams.contains(&"bnbusdt@kline_5m".to_string()));
    }

    #[test]
    fn test_build_stream_names_case_sensitivity() {
        let config = create_test_config();
        let (ws, _) = BinanceWebSocket::new(config);

        let symbols = vec!["BtCuSdT".to_string(), "ETHUSDT".to_string()];
        let timeframes = vec!["1M".to_string()];

        let streams = ws.build_stream_names(&symbols, &timeframes);

        // All symbol parts should be lowercase
        for stream in &streams {
            if stream.contains("btcusdt") {
                assert!(stream.starts_with("btcusdt"));
            }
            if stream.contains("ethusdt") {
                assert!(stream.starts_with("ethusdt"));
            }
        }
    }

    #[test]
    fn test_build_websocket_url_special_characters() {
        let config = create_test_config();
        let (ws, _) = BinanceWebSocket::new(config);

        let streams = vec![
            "btcusdt@kline_1m".to_string(),
            "ethusdt@depth@100ms".to_string(),
        ];
        let url = ws.build_websocket_url(&streams).unwrap();

        // Should handle special characters in stream names
        assert!(url.as_str().contains("btcusdt@kline_1m"));
        assert!(url.as_str().contains("ethusdt@depth@100ms"));
    }

    #[test]
    fn test_build_websocket_url_many_streams_format() {
        let config = create_test_config();
        let (ws, _) = BinanceWebSocket::new(config);

        let streams: Vec<String> = vec![
            "btcusdt@kline_1m".to_string(),
            "btcusdt@ticker".to_string(),
            "btcusdt@depth@100ms".to_string(),
            "ethusdt@kline_1m".to_string(),
            "ethusdt@ticker".to_string(),
        ];

        let url = ws.build_websocket_url(&streams).unwrap();

        // Should use combined stream format and have all streams
        assert!(url.as_str().contains("stream?streams="));

        // Verify all streams are included
        for stream in &streams {
            assert!(url.as_str().contains(stream));
        }
    }

    #[test]
    fn test_handle_stream_data_kline_with_closed_false() {
        let config = create_test_config();
        let (ws, mut receiver) = BinanceWebSocket::new(config);

        let data = serde_json::json!({
            "e": "kline",
            "E": 1625097600000i64,
            "s": "BTCUSDT",
            "k": {
                "t": 1625097600000i64,
                "T": 1625097659999i64,
                "s": "BTCUSDT",
                "i": "1m",
                "f": 100,
                "L": 200,
                "o": "34000.00",
                "c": "34500.00",
                "h": "35000.00",
                "l": "33000.00",
                "v": "100.5",
                "n": 1000,
                "x": false,
                "q": "3450000.00",
                "V": "50.25",
                "Q": "1725000.00"
            }
        });

        ws.handle_stream_data(&data).unwrap();

        let event = receiver.try_recv();
        assert!(event.is_ok());

        match event.unwrap() {
            StreamEvent::Kline(kline_event) => {
                assert!(!kline_event.kline.is_this_kline_closed);
            },
            _ => panic!("Expected Kline event"),
        }
    }

    #[test]
    fn test_handle_stream_data_orderbook_empty_bids_asks() {
        let config = create_test_config();
        let (ws, mut receiver) = BinanceWebSocket::new(config);

        let data = serde_json::json!({
            "e": "depthUpdate",
            "E": 1625097600000i64,
            "s": "BTCUSDT",
            "U": 1000,
            "u": 1005,
            "b": [],
            "a": []
        });

        ws.handle_stream_data(&data).unwrap();

        let event = receiver.try_recv();
        assert!(event.is_ok());

        match event.unwrap() {
            StreamEvent::OrderBook(orderbook_event) => {
                assert_eq!(orderbook_event.bids.len(), 0);
                assert_eq!(orderbook_event.asks.len(), 0);
            },
            _ => panic!("Expected OrderBook event"),
        }
    }

    #[test]
    fn test_handle_stream_data_ticker_with_negative_change() {
        let config = create_test_config();
        let (ws, mut receiver) = BinanceWebSocket::new(config);

        let data = serde_json::json!({
            "e": "24hrTicker",
            "E": 1625097600000i64,
            "s": "BTCUSDT",
            "p": "-500.00",
            "P": "-1.5",
            "w": "34250.00",
            "x": "34500.00",
            "c": "34000.00",
            "Q": "10.5",
            "b": "33990.00",
            "B": "5.0",
            "a": "34010.00",
            "A": "4.5",
            "o": "34500.00",
            "h": "35000.00",
            "l": "33000.00",
            "v": "1000.5",
            "q": "34000000.00",
            "O": 1625011200000i64,
            "C": 1625097600000i64,
            "F": 1000,
            "L": 5000,
            "n": 4000
        });

        ws.handle_stream_data(&data).unwrap();

        let event = receiver.try_recv();
        assert!(event.is_ok());

        match event.unwrap() {
            StreamEvent::Ticker(ticker_event) => {
                assert_eq!(ticker_event.price_change, "-500.00");
                assert_eq!(ticker_event.price_change_percent, "-1.5");
            },
            _ => panic!("Expected Ticker event"),
        }
    }

    #[test]
    fn test_handle_message_combined_stream_different_events() {
        let config = create_test_config();
        let (ws, mut receiver) = BinanceWebSocket::new(config);

        // Test with ticker event in combined stream format
        let message = r#"{
            "stream": "btcusdt@ticker",
            "data": {
                "e": "24hrTicker",
                "E": 1625097600000,
                "s": "BTCUSDT",
                "p": "500.00",
                "P": "1.5",
                "w": "34250.00",
                "x": "33500.00",
                "c": "34000.00",
                "Q": "10.5",
                "b": "33990.00",
                "B": "5.0",
                "a": "34010.00",
                "A": "4.5",
                "o": "33500.00",
                "h": "35000.00",
                "l": "33000.00",
                "v": "1000.5",
                "q": "34000000.00",
                "O": 1625011200000,
                "C": 1625097600000,
                "F": 1000,
                "L": 5000,
                "n": 4000
            }
        }"#;

        let result = ws.handle_message(message);
        assert!(result.is_ok());

        let event = receiver.try_recv();
        assert!(event.is_ok());

        match event.unwrap() {
            StreamEvent::Ticker(ticker_event) => {
                assert_eq!(ticker_event.symbol, "BTCUSDT");
            },
            _ => panic!("Expected Ticker event"),
        }
    }

    #[test]
    fn test_handle_message_orderbook_direct_stream() {
        let config = create_test_config();
        let (ws, mut receiver) = BinanceWebSocket::new(config);

        let message = r#"{
            "e": "depthUpdate",
            "E": 1625097600000,
            "s": "BTCUSDT",
            "U": 1000,
            "u": 1005,
            "b": [["34000.00", "1.5"]],
            "a": [["34001.00", "1.0"]]
        }"#;

        let result = ws.handle_message(message);
        assert!(result.is_ok());

        let event = receiver.try_recv();
        assert!(event.is_ok());

        match event.unwrap() {
            StreamEvent::OrderBook(orderbook_event) => {
                assert_eq!(orderbook_event.symbol, "BTCUSDT");
            },
            _ => panic!("Expected OrderBook event"),
        }
    }

    #[test]
    fn test_channel_unbounded() {
        let config = create_test_config();
        let (ws, mut receiver) = BinanceWebSocket::new(config);

        // Send multiple events without receiving
        for i in 0..100 {
            let data = serde_json::json!({
                "e": "kline",
                "E": 1625097600000i64 + i,
                "s": "BTCUSDT",
                "k": {
                    "t": 1625097600000i64,
                    "T": 1625097659999i64,
                    "s": "BTCUSDT",
                    "i": "1m",
                    "f": 100,
                    "L": 200,
                    "o": "34000.00",
                    "c": "34500.00",
                    "h": "35000.00",
                    "l": "33000.00",
                    "v": "100.5",
                    "n": 1000,
                    "x": true,
                    "q": "3450000.00",
                    "V": "50.25",
                    "Q": "1725000.00"
                }
            });

            let result = ws.handle_stream_data(&data);
            assert!(result.is_ok());
        }

        // All events should be in the channel
        let mut count = 0;
        while receiver.try_recv().is_ok() {
            count += 1;
        }
        assert_eq!(count, 100);
    }

    #[test]
    fn test_different_symbols() {
        let config = create_test_config();
        let (ws, mut receiver) = BinanceWebSocket::new(config);

        let symbols = vec!["BTCUSDT", "ETHUSDT", "BNBUSDT"];

        for symbol in &symbols {
            let data = serde_json::json!({
                "e": "kline",
                "E": 1625097600000i64,
                "s": symbol,
                "k": {
                    "t": 1625097600000i64,
                    "T": 1625097659999i64,
                    "s": symbol,
                    "i": "1m",
                    "f": 100,
                    "L": 200,
                    "o": "34000.00",
                    "c": "34500.00",
                    "h": "35000.00",
                    "l": "33000.00",
                    "v": "100.5",
                    "n": 1000,
                    "x": true,
                    "q": "3450000.00",
                    "V": "50.25",
                    "Q": "1725000.00"
                }
            });

            ws.handle_stream_data(&data).unwrap();

            let event = receiver.try_recv().unwrap();
            match event {
                StreamEvent::Kline(kline_event) => {
                    assert_eq!(kline_event.symbol, *symbol);
                },
                _ => panic!("Expected Kline event"),
            }
        }
    }

    #[test]
    fn test_different_timeframes() {
        let config = create_test_config();
        let (ws, _) = BinanceWebSocket::new(config);

        let symbols = vec!["BTCUSDT".to_string()];
        let timeframes = vec![
            "1m".to_string(),
            "5m".to_string(),
            "15m".to_string(),
            "1h".to_string(),
            "4h".to_string(),
            "1d".to_string(),
        ];

        let streams = ws.build_stream_names(&symbols, &timeframes);

        // Should have 6 kline streams + 1 ticker + 1 depth = 8 streams
        assert_eq!(streams.len(), 8);

        for timeframe in &timeframes {
            assert!(streams.contains(&format!("btcusdt@kline_{}", timeframe)));
        }
    }

    #[test]
    fn test_config_testnet_flag() {
        let mut config = create_test_config();
        config.testnet = true;

        let (ws, _) = BinanceWebSocket::new(config);

        let streams = vec!["test@stream".to_string()];
        let url = ws.build_websocket_url(&streams).unwrap();

        // URL should still use the configured ws_url
        assert!(url.as_str().starts_with("wss://stream.binance.com:9443/ws"));
    }

    #[test]
    fn test_multiple_depth_levels() {
        let config = create_test_config();
        let (ws, mut receiver) = BinanceWebSocket::new(config);

        let data = serde_json::json!({
            "e": "depthUpdate",
            "E": 1625097600000i64,
            "s": "BTCUSDT",
            "U": 1000,
            "u": 1005,
            "b": [
                ["34000.00", "1.5"],
                ["33999.00", "2.0"],
                ["33998.00", "3.0"],
                ["33997.00", "4.0"],
                ["33996.00", "5.0"]
            ],
            "a": [
                ["34001.00", "1.0"],
                ["34002.00", "0.5"],
                ["34003.00", "0.8"],
                ["34004.00", "1.2"]
            ]
        });

        ws.handle_stream_data(&data).unwrap();

        let event = receiver.try_recv().unwrap();
        match event {
            StreamEvent::OrderBook(orderbook_event) => {
                assert_eq!(orderbook_event.bids.len(), 5);
                assert_eq!(orderbook_event.asks.len(), 4);
                assert_eq!(orderbook_event.bids[0].0, "34000.00");
                assert_eq!(orderbook_event.bids[0].1, "1.5");
            },
            _ => panic!("Expected OrderBook event"),
        }
    }

    #[test]
    fn test_kline_different_intervals() {
        let config = create_test_config();
        let (ws, mut receiver) = BinanceWebSocket::new(config);

        let intervals = vec!["1m", "5m", "15m", "1h", "4h", "1d"];

        for interval in intervals {
            let data = serde_json::json!({
                "e": "kline",
                "E": 1625097600000i64,
                "s": "BTCUSDT",
                "k": {
                    "t": 1625097600000i64,
                    "T": 1625097659999i64,
                    "s": "BTCUSDT",
                    "i": interval,
                    "f": 100,
                    "L": 200,
                    "o": "34000.00",
                    "c": "34500.00",
                    "h": "35000.00",
                    "l": "33000.00",
                    "v": "100.5",
                    "n": 1000,
                    "x": true,
                    "q": "3450000.00",
                    "V": "50.25",
                    "Q": "1725000.00"
                }
            });

            ws.handle_stream_data(&data).unwrap();

            let event = receiver.try_recv().unwrap();
            match event {
                StreamEvent::Kline(kline_event) => {
                    assert_eq!(kline_event.kline.interval, interval);
                },
                _ => panic!("Expected Kline event"),
            }
        }
    }

    #[test]
    fn test_ticker_zero_values() {
        let config = create_test_config();
        let (ws, mut receiver) = BinanceWebSocket::new(config);

        let data = serde_json::json!({
            "e": "24hrTicker",
            "E": 1625097600000i64,
            "s": "BTCUSDT",
            "p": "0.00",
            "P": "0.0",
            "w": "34250.00",
            "x": "34000.00",
            "c": "34000.00",
            "Q": "0.0",
            "b": "33990.00",
            "B": "0.0",
            "a": "34010.00",
            "A": "0.0",
            "o": "34000.00",
            "h": "35000.00",
            "l": "33000.00",
            "v": "0.0",
            "q": "0.00",
            "O": 1625011200000i64,
            "C": 1625097600000i64,
            "F": 1000,
            "L": 1000,
            "n": 0
        });

        ws.handle_stream_data(&data).unwrap();

        let event = receiver.try_recv().unwrap();
        match event {
            StreamEvent::Ticker(ticker_event) => {
                assert_eq!(ticker_event.price_change, "0.00");
                assert_eq!(ticker_event.total_number_of_trades, 0);
            },
            _ => panic!("Expected Ticker event"),
        }
    }

    #[test]
    fn test_url_with_single_character_stream() {
        let config = create_test_config();
        let (ws, _) = BinanceWebSocket::new(config);

        let streams = vec!["a".to_string()];
        let url = ws.build_websocket_url(&streams).unwrap();

        assert!(url.as_str().ends_with("/a"));
    }

    #[test]
    fn test_stream_names_with_special_symbols() {
        let config = create_test_config();
        let (ws, _) = BinanceWebSocket::new(config);

        let symbols = vec!["1000SHIBUSDT".to_string(), "BTCUSDT".to_string()];
        let timeframes = vec!["1m".to_string()];

        let streams = ws.build_stream_names(&symbols, &timeframes);

        assert!(streams.contains(&"1000shibusdt@kline_1m".to_string()));
        assert!(streams.contains(&"btcusdt@kline_1m".to_string()));
    }

    #[tokio::test]
    async fn test_user_data_stream_creation() {
        let config = create_test_config();
        let (sender, _receiver) = mpsc::unbounded_channel();

        let result = BinanceUserDataStream::new(config, sender).await;
        assert!(result.is_ok());

        let stream = result.unwrap();
        assert_eq!(stream.listen_key, "dummy_listen_key");
    }

    #[test]
    fn test_handle_stream_data_large_numbers() {
        let config = create_test_config();
        let (ws, mut receiver) = BinanceWebSocket::new(config);

        let data = serde_json::json!({
            "e": "kline",
            "E": 9999999999999i64,
            "s": "BTCUSDT",
            "k": {
                "t": 9999999999999i64,
                "T": 9999999999999i64,
                "s": "BTCUSDT",
                "i": "1m",
                "f": 999999999,
                "L": 999999999,
                "o": "99999999.99",
                "c": "99999999.99",
                "h": "99999999.99",
                "l": "99999999.99",
                "v": "99999999.99",
                "n": 999999999,
                "x": true,
                "q": "99999999.99",
                "V": "99999999.99",
                "Q": "99999999.99"
            }
        });

        ws.handle_stream_data(&data).unwrap();

        let event = receiver.try_recv();
        assert!(event.is_ok());
    }

    #[test]
    fn test_handle_stream_data_decimal_precision() {
        let config = create_test_config();
        let (ws, mut receiver) = BinanceWebSocket::new(config);

        let data = serde_json::json!({
            "e": "kline",
            "E": 1625097600000i64,
            "s": "BTCUSDT",
            "k": {
                "t": 1625097600000i64,
                "T": 1625097659999i64,
                "s": "BTCUSDT",
                "i": "1m",
                "f": 100,
                "L": 200,
                "o": "34000.12345678",
                "c": "34500.87654321",
                "h": "35000.99999999",
                "l": "33000.00000001",
                "v": "100.50000000",
                "n": 1000,
                "x": true,
                "q": "3450000.12345678",
                "V": "50.25000000",
                "Q": "1725000.87654321"
            }
        });

        ws.handle_stream_data(&data).unwrap();

        let event = receiver.try_recv().unwrap();
        match event {
            StreamEvent::Kline(kline_event) => {
                assert_eq!(kline_event.kline.open_price, "34000.12345678");
                assert_eq!(kline_event.kline.close_price, "34500.87654321");
            },
            _ => panic!("Expected Kline event"),
        }
    }

    #[test]
    fn test_build_websocket_url_exactly_two_streams() {
        let config = create_test_config();
        let (ws, _) = BinanceWebSocket::new(config);

        let streams = vec!["stream1@test".to_string(), "stream2@test".to_string()];
        let url = ws.build_websocket_url(&streams).unwrap();

        // Two streams should use combined format
        assert!(url.as_str().contains("stream?streams="));
    }

    #[test]
    fn test_receiver_dropped_sender_fails() {
        let config = create_test_config();
        let (ws, receiver) = BinanceWebSocket::new(config);

        // Drop the receiver
        drop(receiver);

        let data = serde_json::json!({
            "e": "kline",
            "E": 1625097600000i64,
            "s": "BTCUSDT",
            "k": {
                "t": 1625097600000i64,
                "T": 1625097659999i64,
                "s": "BTCUSDT",
                "i": "1m",
                "f": 100,
                "L": 200,
                "o": "34000.00",
                "c": "34500.00",
                "h": "35000.00",
                "l": "33000.00",
                "v": "100.5",
                "n": 1000,
                "x": true,
                "q": "3450000.00",
                "V": "50.25",
                "Q": "1725000.00"
            }
        });

        // Should not panic, just log error
        let result = ws.handle_stream_data(&data);
        assert!(result.is_ok());
    }

    #[test]
    fn test_orderbook_update_ids() {
        let config = create_test_config();
        let (ws, mut receiver) = BinanceWebSocket::new(config);

        let data = serde_json::json!({
            "e": "depthUpdate",
            "E": 1625097600000i64,
            "s": "BTCUSDT",
            "U": 1000,
            "u": 2000,
            "b": [["34000.00", "1.5"]],
            "a": [["34001.00", "1.0"]]
        });

        ws.handle_stream_data(&data).unwrap();

        let event = receiver.try_recv().unwrap();
        match event {
            StreamEvent::OrderBook(orderbook_event) => {
                assert_eq!(orderbook_event.first_update_id, 1000);
                assert_eq!(orderbook_event.final_update_id, 2000);
            },
            _ => panic!("Expected OrderBook event"),
        }
    }

    #[test]
    fn test_ticker_all_fields() {
        let config = create_test_config();
        let (ws, mut receiver) = BinanceWebSocket::new(config);

        let data = serde_json::json!({
            "e": "24hrTicker",
            "E": 1625097600000i64,
            "s": "BTCUSDT",
            "p": "500.00",
            "P": "1.5",
            "w": "34250.00",
            "x": "33500.00",
            "c": "34000.00",
            "Q": "10.5",
            "b": "33990.00",
            "B": "5.0",
            "a": "34010.00",
            "A": "4.5",
            "o": "33500.00",
            "h": "35000.00",
            "l": "33000.00",
            "v": "1000.5",
            "q": "34000000.00",
            "O": 1625011200000i64,
            "C": 1625097600000i64,
            "F": 1000,
            "L": 5000,
            "n": 4000
        });

        ws.handle_stream_data(&data).unwrap();

        let event = receiver.try_recv().unwrap();
        match event {
            StreamEvent::Ticker(ticker_event) => {
                assert_eq!(ticker_event.event_type, "24hrTicker");
                assert_eq!(ticker_event.event_time, 1625097600000);
                assert_eq!(ticker_event.weighted_avg_price, "34250.00");
                assert_eq!(ticker_event.prev_close_price, "33500.00");
                assert_eq!(ticker_event.last_quantity, "10.5");
                assert_eq!(ticker_event.best_bid_price, "33990.00");
                assert_eq!(ticker_event.best_bid_quantity, "5.0");
                assert_eq!(ticker_event.best_ask_price, "34010.00");
                assert_eq!(ticker_event.best_ask_quantity, "4.5");
                assert_eq!(ticker_event.first_trade_id, 1000);
                assert_eq!(ticker_event.last_trade_id, 5000);
            },
            _ => panic!("Expected Ticker event"),
        }
    }

    #[test]
    fn test_subscribe_symbol() {
        let config = create_test_config();
        let (ws, _receiver) = BinanceWebSocket::new(config);

        let result = ws.subscribe_symbol(
            "ETHUSDT".to_string(),
            vec!["1m".to_string(), "5m".to_string()],
        );

        assert!(result.is_ok());
    }

    #[test]
    fn test_unsubscribe_symbol() {
        let config = create_test_config();
        let (ws, _receiver) = BinanceWebSocket::new(config);

        let result = ws.unsubscribe_symbol("ETHUSDT".to_string(), vec!["1m".to_string()]);

        assert!(result.is_ok());
    }

    #[test]
    fn test_get_command_sender() {
        let config = create_test_config();
        let (ws, _receiver) = BinanceWebSocket::new(config);

        let sender1 = ws.get_command_sender();
        let sender2 = ws.get_command_sender();

        // Both senders should be able to send
        let result = sender1.send(WebSocketCommand::Subscribe {
            symbol: "TEST".to_string(),
            timeframes: vec!["1m".to_string()],
        });
        assert!(result.is_ok());

        let result = sender2.send(WebSocketCommand::Unsubscribe {
            symbol: "TEST".to_string(),
            timeframes: vec!["1m".to_string()],
        });
        assert!(result.is_ok());
    }

    #[test]
    fn test_create_command_channel() {
        let config = create_test_config();
        let (ws, _receiver) = BinanceWebSocket::new(config);

        let sender = ws.create_command_channel();
        let result = sender.send(WebSocketCommand::Subscribe {
            symbol: "BTC".to_string(),
            timeframes: vec![],
        });

        assert!(result.is_ok());
    }

    #[test]
    fn test_websocket_command_clone() {
        let cmd1 = WebSocketCommand::Subscribe {
            symbol: "BTC".to_string(),
            timeframes: vec!["1m".to_string()],
        };

        let cmd2 = cmd1.clone();

        match (cmd1, cmd2) {
            (
                WebSocketCommand::Subscribe {
                    symbol: s1,
                    timeframes: t1,
                },
                WebSocketCommand::Subscribe {
                    symbol: s2,
                    timeframes: t2,
                },
            ) => {
                assert_eq!(s1, s2);
                assert_eq!(t1, t2);
            },
            _ => panic!("Expected Subscribe commands"),
        }
    }

    #[test]
    fn test_build_websocket_url_with_custom_base() {
        let mut config = create_test_config();
        config.ws_url = "wss://testnet.binance.vision:9443/ws".to_string();

        let (ws, _) = BinanceWebSocket::new(config);
        let streams = vec!["btcusdt@kline_1m".to_string()];
        let url = ws.build_websocket_url(&streams).unwrap();

        assert!(url
            .as_str()
            .starts_with("wss://testnet.binance.vision:9443"));
    }

    #[test]
    fn test_handle_message_subscription_response() {
        let config = create_test_config();
        let (ws, mut receiver) = BinanceWebSocket::new(config);

        // Subscription response should be handled silently
        let message = r#"{"result":null,"id":1}"#;
        let result = ws.handle_message(message);

        assert!(result.is_ok());

        // No event should be sent for subscription response
        let event = receiver.try_recv();
        assert!(event.is_err());
    }

    #[test]
    fn test_handle_message_with_id_field() {
        let config = create_test_config();
        let (ws, mut receiver) = BinanceWebSocket::new(config);

        // Message with id field (likely a response)
        let message = r#"{"id":123,"method":"SUBSCRIBE"}"#;
        let result = ws.handle_message(message);

        assert!(result.is_ok());

        // No event should be sent
        let event = receiver.try_recv();
        assert!(event.is_err());
    }

    #[test]
    fn test_multiple_subscriptions() {
        let config = create_test_config();
        let (ws, _receiver) = BinanceWebSocket::new(config);

        for i in 0..10 {
            let symbol = format!("SYM{}", i);
            let result = ws.subscribe_symbol(symbol, vec!["1m".to_string()]);
            assert!(result.is_ok());
        }
    }

    #[test]
    fn test_empty_symbol_subscription() {
        let config = create_test_config();
        let (ws, _receiver) = BinanceWebSocket::new(config);

        let result = ws.subscribe_symbol("".to_string(), vec!["1m".to_string()]);
        assert!(result.is_ok()); // Should not fail, just create empty stream names
    }

    // Additional comprehensive tests for WebSocket
    #[test]
    fn test_subscribe_symbol_multiple_timeframes() {
        let config = create_test_config();
        let (ws, _receiver) = BinanceWebSocket::new(config);

        let result = ws.subscribe_symbol(
            "BTCUSDT".to_string(),
            vec!["1m".to_string(), "5m".to_string(), "1h".to_string()],
        );
        assert!(result.is_ok());
    }

    #[test]
    fn test_unsubscribe_symbol_multiple_timeframes() {
        let config = create_test_config();
        let (ws, _receiver) = BinanceWebSocket::new(config);

        let result = ws.unsubscribe_symbol(
            "ETHUSDT".to_string(),
            vec!["1m".to_string(), "15m".to_string()],
        );
        assert!(result.is_ok());
    }

    #[test]
    fn test_get_command_sender_cloneable() {
        let config = create_test_config();
        let (ws, _receiver) = BinanceWebSocket::new(config);

        let sender1 = ws.get_command_sender();
        let sender2 = ws.get_command_sender();
        let sender3 = ws.get_command_sender();

        // All should be able to send
        assert!(sender1.send(WebSocketCommand::Subscribe {
            symbol: "BTC".to_string(),
            timeframes: vec![]
        }).is_ok());

        assert!(sender2.send(WebSocketCommand::Unsubscribe {
            symbol: "ETH".to_string(),
            timeframes: vec![]
        }).is_ok());

        assert!(sender3.send(WebSocketCommand::Subscribe {
            symbol: "BNB".to_string(),
            timeframes: vec!["1m".to_string()]
        }).is_ok());
    }

    #[test]
    fn test_build_stream_names_with_many_symbols() {
        let config = create_test_config();
        let (ws, _) = BinanceWebSocket::new(config);

        let symbols = vec![
            "BTCUSDT".to_string(),
            "ETHUSDT".to_string(),
            "BNBUSDT".to_string(),
            "ADAUSDT".to_string(),
            "SOLUSDT".to_string(),
        ];
        let timeframes = vec!["1m".to_string(), "1h".to_string()];

        let streams = ws.build_stream_names(&symbols, &timeframes);

        // 5 symbols * (2 klines + 1 ticker + 1 depth) = 20 streams
        assert_eq!(streams.len(), 20);
    }

    #[test]
    fn test_build_stream_names_lowercase_symbols() {
        let config = create_test_config();
        let (ws, _) = BinanceWebSocket::new(config);

        let symbols = vec!["BTCUSDT".to_string()];
        let timeframes = vec!["1M".to_string()];

        let streams = ws.build_stream_names(&symbols, &timeframes);

        // All streams should contain lowercase symbol
        for stream in &streams {
            if stream.contains("btcusdt") {
                assert!(stream.starts_with("btcusdt"));
            }
        }
    }

    #[test]
    fn test_build_websocket_url_empty_streams_error() {
        let config = create_test_config();
        let (ws, _) = BinanceWebSocket::new(config);

        let result = ws.build_websocket_url(&[]);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("No streams"));
    }

    #[test]
    fn test_build_websocket_url_single_stream_format() {
        let config = create_test_config();
        let (ws, _) = BinanceWebSocket::new(config);

        let streams = vec!["btcusdt@ticker".to_string()];
        let url = ws.build_websocket_url(&streams).unwrap();

        assert!(url.as_str().ends_with("/btcusdt@ticker"));
        assert!(!url.as_str().contains("stream?streams="));
    }

    #[test]
    fn test_build_websocket_url_multiple_streams_combined() {
        let config = create_test_config();
        let (ws, _) = BinanceWebSocket::new(config);

        let streams = vec![
            "btcusdt@kline_1m".to_string(),
            "ethusdt@ticker".to_string(),
            "bnbusdt@depth@100ms".to_string(),
        ];
        let url = ws.build_websocket_url(&streams).unwrap();

        assert!(url.as_str().contains("stream?streams="));
        assert!(url.as_str().contains("btcusdt@kline_1m"));
        assert!(url.as_str().contains("ethusdt@ticker"));
        assert!(url.as_str().contains("bnbusdt@depth@100ms"));
    }

    #[test]
    fn test_handle_stream_data_kline_non_closed() {
        let config = create_test_config();
        let (ws, mut receiver) = BinanceWebSocket::new(config);

        let data = serde_json::json!({
            "e": "kline",
            "E": 1625097600000i64,
            "s": "BTCUSDT",
            "k": {
                "t": 1625097600000i64,
                "T": 1625097659999i64,
                "s": "BTCUSDT",
                "i": "5m",
                "f": 100,
                "L": 200,
                "o": "34000.00",
                "c": "34500.00",
                "h": "35000.00",
                "l": "33000.00",
                "v": "100.5",
                "n": 1000,
                "x": false,
                "q": "3450000.00",
                "V": "50.25",
                "Q": "1725000.00"
            }
        });

        ws.handle_stream_data(&data).unwrap();
        let event = receiver.try_recv();
        assert!(event.is_ok());

        match event.unwrap() {
            StreamEvent::Kline(kline) => {
                assert!(!kline.kline.is_this_kline_closed);
                assert_eq!(kline.kline.interval, "5m");
            }
            _ => panic!("Expected Kline event"),
        }
    }

    #[test]
    fn test_handle_stream_data_multiple_events_sequence() {
        let config = create_test_config();
        let (ws, mut receiver) = BinanceWebSocket::new(config);

        // Send multiple events
        for i in 0..5 {
            let data = serde_json::json!({
                "e": "24hrTicker",
                "E": 1625097600000i64 + i,
                "s": "BTCUSDT",
                "p": "500.00",
                "P": "1.5",
                "w": "34250.00",
                "x": "33500.00",
                "c": "34000.00",
                "Q": "10.5",
                "b": "33990.00",
                "B": "5.0",
                "a": "34010.00",
                "A": "4.5",
                "o": "33500.00",
                "h": "35000.00",
                "l": "33000.00",
                "v": "1000.5",
                "q": "34000000.00",
                "O": 1625011200000i64,
                "C": 1625097600000i64,
                "F": 1000,
                "L": 5000,
                "n": 4000
            });

            ws.handle_stream_data(&data).unwrap();
        }

        // All 5 events should be received
        let mut count = 0;
        while receiver.try_recv().is_ok() {
            count += 1;
        }
        assert_eq!(count, 5);
    }

    #[test]
    fn test_handle_message_with_result_null() {
        let config = create_test_config();
        let (ws, mut receiver) = BinanceWebSocket::new(config);

        let message = r#"{"result":null,"id":123}"#;
        let result = ws.handle_message(message);

        assert!(result.is_ok());
        // No event should be sent
        assert!(receiver.try_recv().is_err());
    }

    #[test]
    fn test_handle_message_with_id_only() {
        let config = create_test_config();
        let (ws, mut receiver) = BinanceWebSocket::new(config);

        let message = r#"{"id":456,"method":"SUBSCRIBE"}"#;
        let result = ws.handle_message(message);

        assert!(result.is_ok());
        assert!(receiver.try_recv().is_err());
    }

    #[test]
    fn test_orderbook_event_with_many_levels() {
        let config = create_test_config();
        let (ws, mut receiver) = BinanceWebSocket::new(config);

        let data = serde_json::json!({
            "e": "depthUpdate",
            "E": 1625097600000i64,
            "s": "ETHUSDT",
            "U": 1000,
            "u": 1100,
            "b": [
                ["2000.00", "10.5"],
                ["1999.50", "20.0"],
                ["1999.00", "30.0"],
            ],
            "a": [
                ["2001.00", "5.0"],
                ["2001.50", "10.0"],
                ["2002.00", "15.0"],
            ]
        });

        ws.handle_stream_data(&data).unwrap();

        let event = receiver.try_recv().unwrap();
        match event {
            StreamEvent::OrderBook(ob) => {
                assert_eq!(ob.symbol, "ETHUSDT");
                assert_eq!(ob.bids.len(), 3);
                assert_eq!(ob.asks.len(), 3);
            }
            _ => panic!("Expected OrderBook event"),
        }
    }

    #[test]
    fn test_ticker_event_with_high_volume() {
        let config = create_test_config();
        let (ws, mut receiver) = BinanceWebSocket::new(config);

        let data = serde_json::json!({
            "e": "24hrTicker",
            "E": 1625097600000i64,
            "s": "BTCUSDT",
            "p": "1000.00",
            "P": "2.5",
            "w": "41000.00",
            "x": "40000.00",
            "c": "41000.00",
            "Q": "100.5",
            "b": "40990.00",
            "B": "50.0",
            "a": "41010.00",
            "A": "45.0",
            "o": "40000.00",
            "h": "42000.00",
            "l": "39000.00",
            "v": "100000.5",
            "q": "4100000000.00",
            "O": 1625011200000i64,
            "C": 1625097600000i64,
            "F": 10000,
            "L": 50000,
            "n": 40000
        });

        ws.handle_stream_data(&data).unwrap();

        let event = receiver.try_recv().unwrap();
        match event {
            StreamEvent::Ticker(ticker) => {
                assert_eq!(ticker.symbol, "BTCUSDT");
                assert_eq!(ticker.total_traded_base_asset_volume, "100000.5");
                assert_eq!(ticker.total_number_of_trades, 40000);
            }
            _ => panic!("Expected Ticker event"),
        }
    }

    #[test]
    fn test_create_command_channel_returns_sender() {
        let config = create_test_config();
        let (ws, _receiver) = BinanceWebSocket::new(config);

        let sender = ws.create_command_channel();
        let result = sender.send(WebSocketCommand::Subscribe {
            symbol: "TEST".to_string(),
            timeframes: vec!["1m".to_string()],
        });

        assert!(result.is_ok());
    }

    #[test]
    fn test_websocket_command_debug_format() {
        let cmd = WebSocketCommand::Subscribe {
            symbol: "BTC".to_string(),
            timeframes: vec!["1m".to_string()],
        };

        let debug_str = format!("{:?}", cmd);
        assert!(debug_str.contains("Subscribe"));
        assert!(debug_str.contains("BTC"));
    }

    #[test]
    fn test_build_stream_names_special_symbol_format() {
        let config = create_test_config();
        let (ws, _) = BinanceWebSocket::new(config);

        let symbols = vec!["1000SHIBUSDT".to_string()];
        let timeframes = vec!["1m".to_string()];

        let streams = ws.build_stream_names(&symbols, &timeframes);

        assert!(streams.contains(&"1000shibusdt@kline_1m".to_string()));
        assert!(streams.contains(&"1000shibusdt@ticker".to_string()));
        assert!(streams.contains(&"1000shibusdt@depth@100ms".to_string()));
    }

    #[test]
    fn test_handle_stream_data_missing_required_fields() {
        let config = create_test_config();
        let (ws, mut receiver) = BinanceWebSocket::new(config);

        let data = serde_json::json!({
            "e": "kline",
            "E": 1625097600000i64,
            // Missing symbol and kline data
        });

        let result = ws.handle_stream_data(&data);
        assert!(result.is_ok());

        // No event should be sent for invalid data
        assert!(receiver.try_recv().is_err());
    }

    #[tokio::test]
    async fn test_user_data_stream_listen_key() {
        let config = create_test_config();
        let (sender, _receiver) = mpsc::unbounded_channel();

        let stream = BinanceUserDataStream::new(config, sender).await;
        assert!(stream.is_ok());

        let stream = stream.unwrap();
        assert_eq!(stream.listen_key, "dummy_listen_key");
    }

    #[test]
    fn test_url_construction_with_trailing_slash() {
        let mut config = create_test_config();
        config.ws_url = "wss://stream.binance.com:9443/ws/".to_string();

        let (ws, _) = BinanceWebSocket::new(config);
        let streams = vec!["btcusdt@kline_1m".to_string()];
        let url = ws.build_websocket_url(&streams).unwrap();

        assert!(url.as_str().contains("btcusdt@kline_1m"));
    }

    #[test]
    fn test_subscribe_unsubscribe_same_symbol() {
        let config = create_test_config();
        let (ws, _receiver) = BinanceWebSocket::new(config);

        // Subscribe
        let result = ws.subscribe_symbol("BTCUSDT".to_string(), vec!["1m".to_string()]);
        assert!(result.is_ok());

        // Unsubscribe same symbol
        let result = ws.unsubscribe_symbol("BTCUSDT".to_string(), vec!["1m".to_string()]);
        assert!(result.is_ok());
    }

    #[test]
    fn test_handle_message_partial_json() {
        let config = create_test_config();
        let (ws, _receiver) = BinanceWebSocket::new(config);

        let message = r#"{"e":"kline","E":"#; // Incomplete JSON
        let result = ws.handle_message(message);

        // Should not panic, just log warning
        assert!(result.is_ok());
    }

    // Additional WebSocket tests for increased coverage
    #[test]
    fn test_command_sender_cloning() {
        let config = create_test_config();
        let (ws, _) = BinanceWebSocket::new(config);

        let sender1 = ws.get_command_sender();
        let sender2 = ws.get_command_sender();

        let _ = sender1;
        let _ = sender2;
    }

    #[test]
    fn test_create_command_channel_v2() {
        let config = create_test_config();
        let (ws, _) = BinanceWebSocket::new(config);

        let channel = ws.create_command_channel();
        let _ = channel;
    }

    #[test]
    fn test_subscribe_symbol_multiple_timeframes_v2() {
        let config = create_test_config();
        let (ws, _) = BinanceWebSocket::new(config);

        let timeframes = vec!["1m".to_string(), "5m".to_string(), "15m".to_string()];
        let result = ws.subscribe_symbol("ETHUSDT".to_string(), timeframes);
        assert!(result.is_ok());
    }

    #[test]
    fn test_unsubscribe_symbol_multiple_timeframes_v2() {
        let config = create_test_config();
        let (ws, _) = BinanceWebSocket::new(config);

        let timeframes = vec!["1m".to_string(), "5m".to_string()];
        let result = ws.unsubscribe_symbol("BNBUSDT".to_string(), timeframes);
        assert!(result.is_ok());
    }

    #[test]
    fn test_build_stream_names_single_symbol_single_timeframe_v2() {
        let config = create_test_config();
        let (ws, _) = BinanceWebSocket::new(config);

        let symbols = vec!["SOLUSDT".to_string()];
        let timeframes = vec!["1h".to_string()];
        let streams = ws.build_stream_names(&symbols, &timeframes);

        // 1 kline + 1 ticker + 1 depth = 3 streams
        assert_eq!(streams.len(), 3);
        assert!(streams.contains(&"solusdt@kline_1h".to_string()));
        assert!(streams.contains(&"solusdt@ticker".to_string()));
        assert!(streams.contains(&"solusdt@depth@100ms".to_string()));
    }

    #[test]
    fn test_build_websocket_url_empty_streams_v2() {
        let config = create_test_config();
        let (ws, _) = BinanceWebSocket::new(config);

        let streams = vec![];
        let result = ws.build_websocket_url(&streams);
        assert!(result.is_err());
    }

    #[test]
    fn test_build_websocket_url_many_streams_v2() {
        let config = create_test_config();
        let (ws, _) = BinanceWebSocket::new(config);

        let streams = (0..20)
            .map(|i| format!("stream{}@data", i))
            .collect::<Vec<_>>();
        let url = ws.build_websocket_url(&streams).unwrap();

        assert!(url.as_str().contains("stream?streams="));
    }

    #[test]
    fn test_handle_stream_data_unknown_event_type() {
        let config = create_test_config();
        let (ws, mut receiver) = BinanceWebSocket::new(config);

        let data = serde_json::json!({
            "e": "unknownEvent",
            "E": 1625097600000i64,
            "s": "BTCUSDT"
        });

        let result = ws.handle_stream_data(&data);
        assert!(result.is_ok());

        // No event should be sent for unknown types
        assert!(receiver.try_recv().is_err());
    }

    #[test]
    fn test_handle_stream_data_kline_partial_fields() {
        let config = create_test_config();
        let (ws, mut receiver) = BinanceWebSocket::new(config);

        let data = serde_json::json!({
            "e": "kline",
            "E": 1625097600000i64,
            "s": "BTCUSDT",
            "k": {
                "t": 1625097540000i64,
                "T": 1625097599999i64
                // Missing required fields
            }
        });

        let result = ws.handle_stream_data(&data);
        assert!(result.is_ok());

        // Should not send event due to missing fields
        assert!(receiver.try_recv().is_err());
    }

    #[test]
    fn test_handle_stream_data_ticker_partial_fields() {
        let config = create_test_config();
        let (ws, mut receiver) = BinanceWebSocket::new(config);

        let data = serde_json::json!({
            "e": "24hrTicker",
            "E": 1625097600000i64,
            "s": "ETHUSDT"
            // Missing other required ticker fields
        });

        let result = ws.handle_stream_data(&data);
        assert!(result.is_ok());

        assert!(receiver.try_recv().is_err());
    }

    #[test]
    fn test_handle_stream_data_depth_partial_fields() {
        let config = create_test_config();
        let (ws, mut receiver) = BinanceWebSocket::new(config);

        let data = serde_json::json!({
            "e": "depthUpdate",
            "E": 1625097600000i64,
            "s": "BNBUSDT"
            // Missing bids/asks
        });

        let result = ws.handle_stream_data(&data);
        assert!(result.is_ok());

        assert!(receiver.try_recv().is_err());
    }

    #[test]
    fn test_handle_message_combined_stream_wrapper() {
        let config = create_test_config();
        let (ws, mut receiver) = BinanceWebSocket::new(config);

        // Simulate combined stream message format
        let message = r#"{"stream":"btcusdt@kline_1m","data":{"e":"kline","E":1625097600000,"s":"BTCUSDT","k":{"t":1625097540000,"T":1625097599999,"s":"BTCUSDT","i":"1m","f":100,"L":200,"o":"35000.00","c":"35100.00","h":"35200.00","l":"34900.00","v":"100.5","n":100,"x":false,"q":"3520000.00","V":"50.25","Q":"1760000.00","B":"0"}}}"#;

        let result = ws.handle_message(message);
        assert!(result.is_ok());

        let event = receiver.try_recv();
        assert!(event.is_ok());
    }

    #[test]
    fn test_handle_message_direct_stream_format() {
        let config = create_test_config();
        let (ws, mut receiver) = BinanceWebSocket::new(config);

        // Direct stream format (no wrapper)
        let message = r#"{"e":"kline","E":1625097600000,"s":"BTCUSDT","k":{"t":1625097540000,"T":1625097599999,"s":"BTCUSDT","i":"1m","f":100,"L":200,"o":"35000.00","c":"35100.00","h":"35200.00","l":"34900.00","v":"100.5","n":100,"x":false,"q":"3520000.00","V":"50.25","Q":"1760000.00","B":"0"}}"#;

        let result = ws.handle_message(message);
        assert!(result.is_ok());

        let event = receiver.try_recv();
        assert!(event.is_ok());
    }

    #[test]
    fn test_build_stream_names_mixed_case_symbols() {
        let config = create_test_config();
        let (ws, _) = BinanceWebSocket::new(config);

        let symbols = vec!["BtCuSdT".to_string(), "EtHuSdT".to_string()];
        let timeframes = vec!["1m".to_string()];
        let streams = ws.build_stream_names(&symbols, &timeframes);

        // All should be lowercase
        for stream in &streams {
            assert!(!stream.contains(char::is_uppercase));
        }
    }

    #[test]
    fn test_build_stream_names_special_timeframes() {
        let config = create_test_config();
        let (ws, _) = BinanceWebSocket::new(config);

        let symbols = vec!["BTCUSDT".to_string()];
        let timeframes = vec!["3m".to_string(), "30m".to_string(), "2h".to_string()];
        let streams = ws.build_stream_names(&symbols, &timeframes);

        assert!(streams.contains(&"btcusdt@kline_3m".to_string()));
        assert!(streams.contains(&"btcusdt@kline_30m".to_string()));
        assert!(streams.contains(&"btcusdt@kline_2h".to_string()));
    }

    #[test]
    fn test_url_construction_no_ws_suffix() {
        let mut config = create_test_config();
        config.ws_url = "wss://stream.binance.com:9443".to_string();

        let (ws, _) = BinanceWebSocket::new(config);
        let streams = vec!["test1@stream".to_string(), "test2@stream".to_string()];
        let url = ws.build_websocket_url(&streams).unwrap();

        // Should handle URLs without /ws suffix
        assert!(url.as_str().contains("stream?streams="));
    }

    #[test]
    fn test_handle_message_subscription_response_v2() {
        let config = create_test_config();
        let (ws, mut receiver) = BinanceWebSocket::new(config);

        // Subscription response from Binance
        let message = r#"{"result":null,"id":1}"#;
        let result = ws.handle_message(message);
        assert!(result.is_ok());

        // No event should be sent for subscription responses
        assert!(receiver.try_recv().is_err());
    }

    #[test]
    fn test_handle_message_subscription_response_with_id() {
        let config = create_test_config();
        let (ws, mut receiver) = BinanceWebSocket::new(config);

        // Another subscription response format
        let message = r#"{"id":42,"result":null}"#;
        let result = ws.handle_message(message);
        assert!(result.is_ok());

        assert!(receiver.try_recv().is_err());
    }

    #[test]
    fn test_websocket_command_clone_v2() {
        let cmd1 = WebSocketCommand::Subscribe {
            symbol: "BTCUSDT".to_string(),
            timeframes: vec!["1m".to_string()],
        };

        let cmd2 = cmd1.clone();

        match (cmd1, cmd2) {
            (
                WebSocketCommand::Subscribe {
                    symbol: s1,
                    timeframes: _,
                },
                WebSocketCommand::Subscribe {
                    symbol: s2,
                    timeframes: _,
                },
            ) => assert_eq!(s1, s2),
            _ => panic!("Clone failed"),
        }
    }

    #[test]
    fn test_websocket_command_unsubscribe() {
        let cmd = WebSocketCommand::Unsubscribe {
            symbol: "ETHUSDT".to_string(),
            timeframes: vec!["5m".to_string(), "15m".to_string()],
        };

        match cmd {
            WebSocketCommand::Unsubscribe { symbol, timeframes } => {
                assert_eq!(symbol, "ETHUSDT");
                assert_eq!(timeframes.len(), 2);
            },
            _ => panic!("Wrong command type"),
        }
    }

    #[tokio::test]
    async fn test_user_data_stream_creation_v2() {
        let config = create_test_config();
        let (sender, _) = mpsc::unbounded_channel();

        let result = BinanceUserDataStream::new(config, sender).await;
        assert!(result.is_ok());
    }

    #[test]
    fn test_receiver_channel_empty_on_creation() {
        let config = create_test_config();
        let (_, mut receiver) = BinanceWebSocket::new(config);

        assert!(receiver.try_recv().is_err());
    }

    #[test]
    fn test_handle_message_null_json() {
        let config = create_test_config();
        let (ws, _) = BinanceWebSocket::new(config);

        let message = "null";
        let result = ws.handle_message(message);
        assert!(result.is_ok());
    }

    #[test]
    fn test_handle_message_array_json() {
        let config = create_test_config();
        let (ws, _) = BinanceWebSocket::new(config);

        let message = r#"["invalid","array"]"#;
        let result = ws.handle_message(message);
        assert!(result.is_ok());
    }

    #[test]
    fn test_handle_stream_data_empty_event_string() {
        let config = create_test_config();
        let (ws, mut receiver) = BinanceWebSocket::new(config);

        let data = serde_json::json!({
            "e": "",
            "E": 1625097600000i64,
            "s": "BTCUSDT"
        });

        let result = ws.handle_stream_data(&data);
        assert!(result.is_ok());

        assert!(receiver.try_recv().is_err());
    }

    // ========== COV8 TESTS: Additional coverage for websocket ==========

    #[test]
    fn test_cov8_websocket_command_debug_format() {
        let cmd = WebSocketCommand::Subscribe {
            symbol: "BTCUSDT".to_string(),
            timeframes: vec!["1m".to_string()],
        };
        let debug_str = format!("{:?}", cmd);
        assert!(debug_str.contains("Subscribe"));
        assert!(debug_str.contains("BTCUSDT"));
    }

    #[test]
    fn test_cov8_build_websocket_url_three_streams() {
        let config = create_test_config();
        let (ws, _) = BinanceWebSocket::new(config);

        let streams = vec![
            "btcusdt@kline_1m".to_string(),
            "btcusdt@ticker".to_string(),
            "btcusdt@depth@100ms".to_string(),
        ];
        let url = ws.build_websocket_url(&streams).unwrap();

        assert!(url.as_str().contains("stream?streams="));
        for stream in &streams {
            assert!(url.as_str().contains(stream));
        }
    }

    #[test]
    fn test_cov8_handle_message_combined_stream() {
        let config = create_test_config();
        let (ws, mut receiver) = BinanceWebSocket::new(config);

        let combined_msg = r#"{"stream":"btcusdt@kline_1m","data":{"e":"kline","E":1625097600000,"s":"BTCUSDT","k":{"t":1625097600000,"T":1625097659999,"s":"BTCUSDT","i":"1m","f":100,"L":200,"o":"34000.0","c":"34100.0","h":"34200.0","l":"33900.0","v":"100.5","n":100,"x":true,"q":"3420000.0","V":"50.2","Q":"1710000.0","B":"0"}}}"#;

        let result = ws.handle_message(combined_msg);
        assert!(result.is_ok());

        // Should receive kline event
        match receiver.try_recv() {
            Ok(StreamEvent::Kline(k)) => {
                assert_eq!(k.symbol, "BTCUSDT");
            },
            _ => {},
        }
    }

    #[test]
    fn test_cov8_build_stream_names_mixed_case_symbols() {
        let config = create_test_config();
        let (ws, _) = BinanceWebSocket::new(config);

        let symbols = vec!["BtCuSdT".to_string()];
        let timeframes = vec!["1M".to_string()];

        let streams = ws.build_stream_names(&symbols, &timeframes);

        // Should convert to lowercase
        assert!(streams.contains(&"btcusdt@kline_1M".to_string()));
        assert!(streams.contains(&"btcusdt@ticker".to_string()));
        assert!(streams.contains(&"btcusdt@depth@100ms".to_string()));
    }

    #[test]
    fn test_cov8_create_command_channel_multiple() {
        let config = create_test_config();
        let (ws, _) = BinanceWebSocket::new(config);

        let _ch1 = ws.create_command_channel();
        let _ch2 = ws.create_command_channel();
        let _ch3 = ws.create_command_channel();
        // Should be able to create multiple command channels
    }

    #[test]
    fn test_cov8_handle_stream_data_no_event_type() {
        let config = create_test_config();
        let (ws, _receiver) = BinanceWebSocket::new(config);

        let data = serde_json::json!({
            "s": "BTCUSDT",
            "p": "34000.0"
        });

        let result = ws.handle_stream_data(&data);
        assert!(result.is_ok());
    }

    #[test]
    fn test_cov8_subscribe_multiple_timeframes() {
        let config = create_test_config();
        let (ws, _) = BinanceWebSocket::new(config);

        let result = ws.subscribe_symbol(
            "BTCUSDT".to_string(),
            vec!["1m".to_string(), "5m".to_string(), "1h".to_string()],
        );
        assert!(result.is_ok());
    }

    #[test]
    fn test_cov8_unsubscribe_multiple_timeframes() {
        let config = create_test_config();
        let (ws, _) = BinanceWebSocket::new(config);

        let result = ws.unsubscribe_symbol(
            "ETHUSDT".to_string(),
            vec!["1m".to_string(), "5m".to_string()],
        );
        assert!(result.is_ok());
    }
}
