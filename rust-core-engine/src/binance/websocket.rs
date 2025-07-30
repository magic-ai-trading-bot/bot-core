use anyhow::Result;
use futures_util::{SinkExt, StreamExt};
use serde_json::Value;
use std::time::Duration;
use tokio::sync::mpsc;
use tokio::time::{interval, sleep};
use tokio_tungstenite::{connect_async, tungstenite::Message};
use tracing::{debug, error, info, warn};
use url::Url;

use super::types::*;
use crate::config::BinanceConfig;

pub struct BinanceWebSocket {
    config: BinanceConfig,
    sender: mpsc::UnboundedSender<StreamEvent>,
}

impl BinanceWebSocket {
    pub fn new(config: BinanceConfig) -> (Self, mpsc::UnboundedReceiver<StreamEvent>) {
        let (sender, receiver) = mpsc::unbounded_channel();

        let ws = Self { config, sender };

        (ws, receiver)
    }

    pub async fn start(&self, symbols: Vec<String>, timeframes: Vec<String>) -> Result<()> {
        let mut reconnect_attempts = 0;
        let max_reconnect_attempts = 10;

        loop {
            match self.connect_and_run(&symbols, &timeframes).await {
                Ok(_) => {
                    info!("WebSocket connection closed normally");
                    break;
                }
                Err(e) => {
                    error!("WebSocket error: {e}");
                    reconnect_attempts += 1;

                    if reconnect_attempts >= max_reconnect_attempts {
                        error!("Max reconnection attempts reached, giving up");
                        return Err(e.into());
                    }

                    let delay = Duration::from_secs(2_u64.pow(reconnect_attempts.min(6)));
                    warn!(
                        "Reconnecting in {:?} (attempt {}/{})",
                        delay, reconnect_attempts, max_reconnect_attempts
                    );
                    sleep(delay).await;
                }
            }
        }

        Ok(())
    }

    async fn connect_and_run(&self, symbols: &[String], timeframes: &[String]) -> Result<()> {
        let streams = self.build_stream_names(symbols, timeframes);
        let url = self.build_websocket_url(&streams)?;

        info!("Connecting to WebSocket: {url}");

        let (ws_stream, _) = connect_async(&url).await?;
        let (mut write, mut read) = ws_stream.split();

        info!("WebSocket connected successfully");

        // Handle incoming messages
        while let Some(message) = read.next().await {
            match message {
                Ok(Message::Text(text)) => {
                    if let Err(e) = self.handle_message(&text) {
                        error!("Error handling message: {e}");
                    }
                }
                Ok(Message::Close(_)) => {
                    info!("WebSocket connection closed by server");
                    break;
                }
                Ok(Message::Ping(data)) => {
                    debug!("Received ping, sending pong");
                    if let Err(e) = write.send(Message::Pong(data)).await {
                        error!("Failed to send pong: {e}");
                        break;
                    }
                }
                Ok(_) => {
                    // Ignore other message types (binary, pong, etc.)
                }
                Err(e) => {
                    error!("WebSocket error: {e}");
                    return Err(e.into());
                }
            }
        }

        Ok(())
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
            // Single stream
            Ok(Url::parse(&format!("{base_url}/{}", streams[0]))?)
        } else {
            // Multiple streams using combined stream endpoint
            let stream_list = streams.join("/");
            Ok(Url::parse(&format!(
                "{base_url}/stream?streams={stream_list}"
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
                }
                "24hrTicker" => {
                    if let Ok(ticker_event) = serde_json::from_value::<TickerEvent>(data.clone()) {
                        if let Err(e) = self.sender.send(StreamEvent::Ticker(ticker_event)) {
                            error!("Failed to send ticker event: {e}");
                        }
                    } else {
                        warn!("Failed to parse ticker event: {data}");
                    }
                }
                "depthUpdate" => {
                    if let Ok(depth_event) = serde_json::from_value::<OrderBookEvent>(data.clone())
                    {
                        if let Err(e) = self.sender.send(StreamEvent::OrderBook(depth_event)) {
                            error!("Failed to send order book event: {e}");
                        }
                    } else {
                        warn!("Failed to parse order book event: {data}");
                    }
                }
                _ => {
                    debug!("Unknown event type: {event_type}");
                }
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
        let url = format!("{}/ws/{}", self.config.futures_ws_url, self.listen_key);

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
                }
                Ok(Message::Close(_)) => {
                    info!("User data stream closed");
                    break;
                }
                Ok(Message::Ping(data)) => {
                    debug!("Received ping on user data stream");
                    if let Err(e) = write.send(Message::Pong(data)).await {
                        error!("Failed to send pong: {e}");
                        break;
                    }
                }
                Ok(_) => {}
                Err(e) => {
                    error!("User data stream error: {e}");
                    return Err(e.into());
                }
            }
        }

        Ok(())
    }
}
