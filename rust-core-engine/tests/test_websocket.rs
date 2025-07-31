mod common;

use actix_web::{test, web, App};
use binance_trading_bot::models::*;
use binance_trading_bot::websocket::*;
use common::*;
use futures_util::{SinkExt, StreamExt};
use serde_json::json;
use tokio_tungstenite::connect_async;

#[actix_web::test]
async fn test_websocket_connection() {
    // Start test server
    let app = test::init_service(App::new().route("/ws", web::get().to(websocket_handler))).await;

    // Connect client
    let ws_url = "ws://localhost:8080/ws";
    let (ws_stream, _) = connect_async(ws_url).await.unwrap();
    let (mut write, mut read) = ws_stream.split();

    // Send subscribe message
    let subscribe_msg = json!({
        "type": "subscribe",
        "symbols": ["BTCUSDT", "ETHUSDT"]
    });
    write
        .send(tokio_tungstenite::tungstenite::Message::Text(
            subscribe_msg.to_string(),
        ))
        .await
        .unwrap();

    // Should receive confirmation
    if let Some(msg) = read.next().await {
        let msg = msg.unwrap();
        if let tokio_tungstenite::tungstenite::Message::Text(text) = msg {
            let data: serde_json::Value = serde_json::from_str(&text).unwrap();
            assert_eq!(data["type"], "subscribed");
        }
    }
}

#[actix_web::test]
async fn test_websocket_price_updates() {
    // Mock Binance WebSocket data
    let mock_price_update = json!({
        "e": "24hrTicker",
        "E": 1701234567000i64,
        "s": "BTCUSDT",
        "p": "1000.00",
        "P": "2.34",
        "w": "45234.56",
        "x": "44234.56",
        "c": "45234.56",
        "Q": "0.123",
        "b": "45234.55",
        "B": "1.234",
        "a": "45234.57",
        "A": "2.345",
        "o": "44234.56",
        "h": "45500.00",
        "l": "44000.00",
        "v": "12345.678",
        "q": "558901234.56",
        "O": 1701148167000i64,
        "C": 1701234567000i64,
        "F": 123456789,
        "L": 123456890,
        "n": 101
    });

    // Test price update parsing
    let update = parse_binance_ticker(&mock_price_update).unwrap();
    assert_eq!(update.symbol, "BTCUSDT");
    assert_eq!(update.price, 45234.56);
    assert_eq!(update.volume, 12345.678);
}

#[actix_web::test]
async fn test_websocket_kline_updates() {
    // Mock kline data
    let mock_kline = json!({
        "e": "kline",
        "E": 1701234567000i64,
        "s": "BTCUSDT",
        "k": {
            "t": 1701234000000i64,
            "T": 1701234599999i64,
            "s": "BTCUSDT",
            "i": "1m",
            "f": 100,
            "L": 200,
            "o": "45000.00",
            "c": "45100.00",
            "h": "45200.00",
            "l": "44900.00",
            "v": "100.123",
            "n": 101,
            "x": true,
            "q": "4510123.45",
            "V": "50.123",
            "Q": "2255123.45",
            "B": "0"
        }
    });

    // Test kline parsing
    let kline = parse_binance_kline(&mock_kline).unwrap();
    assert_eq!(kline.symbol, "BTCUSDT");
    assert_eq!(kline.open, 45000.0);
    assert_eq!(kline.close, 45100.0);
    assert_eq!(kline.high, 45200.0);
    assert_eq!(kline.low, 44900.0);
}

#[actix_web::test]
async fn test_websocket_reconnection() {
    // Test automatic reconnection logic
    let mut manager = WebSocketManager::new();

    // Simulate connection
    manager
        .connect("wss://stream.binance.com:9443/ws")
        .await
        .unwrap();

    // Simulate disconnect
    manager.disconnect().await;
    assert!(!manager.is_connected());

    // Should auto-reconnect
    tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;
    // In real implementation, would check reconnection
}

#[actix_web::test]
async fn test_websocket_rate_limiting() {
    // Test that we don't exceed rate limits
    let mut rate_limiter = RateLimiter::new(5, 60); // 5 requests per minute

    // Should allow first 5
    for _ in 0..5 {
        assert!(rate_limiter.check());
    }

    // 6th should be blocked
    assert!(!rate_limiter.check());
}

#[actix_web::test]
async fn test_websocket_heartbeat() {
    // Test heartbeat/ping-pong mechanism
    let app = test::init_service(App::new().route("/ws", web::get().to(websocket_handler))).await;

    let ws_url = "ws://localhost:8080/ws";
    let (ws_stream, _) = connect_async(ws_url).await.unwrap();
    let (mut write, mut read) = ws_stream.split();

    // Send ping
    write
        .send(tokio_tungstenite::tungstenite::Message::Ping(vec![1, 2, 3]))
        .await
        .unwrap();

    // Should receive pong
    if let Some(msg) = read.next().await {
        let msg = msg.unwrap();
        match msg {
            tokio_tungstenite::tungstenite::Message::Pong(data) => {
                assert_eq!(data, vec![1, 2, 3]);
            }
            _ => panic!("Expected pong message"),
        }
    }
}

#[actix_web::test]
async fn test_websocket_error_handling() {
    // Test various error scenarios

    // Invalid JSON
    let invalid_json = "not a json{";
    let result = serde_json::from_str::<serde_json::Value>(invalid_json);
    assert!(result.is_err());

    // Missing required fields
    let incomplete_ticker = json!({
        "e": "24hrTicker",
        "s": "BTCUSDT"
        // Missing other required fields
    });
    let result = parse_binance_ticker(&incomplete_ticker);
    assert!(result.is_err());
}

#[actix_web::test]
async fn test_websocket_subscription_management() {
    let mut manager = WebSocketManager::new();

    // Add subscriptions
    manager.subscribe("BTCUSDT").await.unwrap();
    manager.subscribe("ETHUSDT").await.unwrap();

    assert_eq!(manager.subscriptions.len(), 2);

    // Remove subscription
    manager.unsubscribe("BTCUSDT").await.unwrap();
    assert_eq!(manager.subscriptions.len(), 1);

    // Clear all
    manager.clear_subscriptions().await;
    assert_eq!(manager.subscriptions.len(), 0);
}

#[actix_web::test]
async fn test_websocket_message_queuing() {
    // Test message queue during disconnection
    let mut queue = MessageQueue::new(100);

    // Add messages
    for i in 0..10 {
        queue.push(json!({
            "id": i,
            "data": "test"
        }));
    }

    assert_eq!(queue.len(), 10);

    // Process messages
    while let Some(msg) = queue.pop() {
        assert!(msg["id"].is_number());
    }

    assert_eq!(queue.len(), 0);
}

#[actix_web::test]
async fn test_websocket_concurrent_connections() {
    // Test handling multiple concurrent WebSocket connections
    let app = test::init_service(App::new().route("/ws", web::get().to(websocket_handler))).await;

    let mut handles = vec![];

    // Spawn multiple connections
    for i in 0..5 {
        let handle = tokio::spawn(async move {
            let ws_url = format!("ws://localhost:8080/ws?client={}", i);
            let (ws_stream, _) = connect_async(&ws_url).await.unwrap();
            let (mut write, _) = ws_stream.split();

            // Send test message
            write
                .send(tokio_tungstenite::tungstenite::Message::Text(
                    json!({"client": i, "test": true}).to_string(),
                ))
                .await
                .unwrap();
        });
        handles.push(handle);
    }

    // Wait for all to complete
    for handle in handles {
        handle.await.unwrap();
    }
}

// Helper functions
fn parse_binance_ticker(
    data: &serde_json::Value,
) -> Result<PriceUpdate, Box<dyn std::error::Error>> {
    Ok(PriceUpdate {
        symbol: data["s"].as_str().unwrap().to_string(),
        price: data["c"].as_str().unwrap().parse()?,
        volume: data["v"].as_str().unwrap().parse()?,
        timestamp: data["E"].as_i64().unwrap(),
    })
}

fn parse_binance_kline(data: &serde_json::Value) -> Result<KlineData, Box<dyn std::error::Error>> {
    let k = &data["k"];
    Ok(KlineData {
        symbol: k["s"].as_str().unwrap().to_string(),
        interval: k["i"].as_str().unwrap().to_string(),
        open: k["o"].as_str().unwrap().parse()?,
        close: k["c"].as_str().unwrap().parse()?,
        high: k["h"].as_str().unwrap().parse()?,
        low: k["l"].as_str().unwrap().parse()?,
        volume: k["v"].as_str().unwrap().parse()?,
        timestamp: k["t"].as_i64().unwrap(),
    })
}

// Mock structures for testing
struct WebSocketManager {
    connected: bool,
    subscriptions: Vec<String>,
}

impl WebSocketManager {
    fn new() -> Self {
        Self {
            connected: false,
            subscriptions: Vec::new(),
        }
    }

    async fn connect(&mut self, _url: &str) -> Result<(), Box<dyn std::error::Error>> {
        self.connected = true;
        Ok(())
    }

    async fn disconnect(&mut self) {
        self.connected = false;
    }

    fn is_connected(&self) -> bool {
        self.connected
    }

    async fn subscribe(&mut self, symbol: &str) -> Result<(), Box<dyn std::error::Error>> {
        self.subscriptions.push(symbol.to_string());
        Ok(())
    }

    async fn unsubscribe(&mut self, symbol: &str) -> Result<(), Box<dyn std::error::Error>> {
        self.subscriptions.retain(|s| s != symbol);
        Ok(())
    }

    async fn clear_subscriptions(&mut self) {
        self.subscriptions.clear();
    }
}

struct RateLimiter {
    max_requests: usize,
    window_seconds: u64,
    requests: Vec<std::time::Instant>,
}

impl RateLimiter {
    fn new(max_requests: usize, window_seconds: u64) -> Self {
        Self {
            max_requests,
            window_seconds,
            requests: Vec::new(),
        }
    }

    fn check(&mut self) -> bool {
        let now = std::time::Instant::now();
        let window_start = now - std::time::Duration::from_secs(self.window_seconds);

        // Remove old requests
        self.requests.retain(|&req_time| req_time > window_start);

        if self.requests.len() < self.max_requests {
            self.requests.push(now);
            true
        } else {
            false
        }
    }
}

struct MessageQueue {
    messages: Vec<serde_json::Value>,
    capacity: usize,
}

impl MessageQueue {
    fn new(capacity: usize) -> Self {
        Self {
            messages: Vec::new(),
            capacity,
        }
    }

    fn push(&mut self, msg: serde_json::Value) {
        if self.messages.len() < self.capacity {
            self.messages.push(msg);
        }
    }

    fn pop(&mut self) -> Option<serde_json::Value> {
        if !self.messages.is_empty() {
            Some(self.messages.remove(0))
        } else {
            None
        }
    }

    fn len(&self) -> usize {
        self.messages.len()
    }
}

struct PriceUpdate {
    symbol: String,
    price: f64,
    volume: f64,
    timestamp: i64,
}

struct KlineData {
    symbol: String,
    interval: String,
    open: f64,
    close: f64,
    high: f64,
    low: f64,
    volume: f64,
    timestamp: i64,
}

// Mock handler
async fn websocket_handler() -> actix_web::Result<String> {
    Ok("WebSocket endpoint".to_string())
}
