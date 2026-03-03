# FR-BINANCE: Binance Exchange Integration

**Spec ID**: FR-BINANCE
**Version**: 1.0
**Status**: Implemented
**Owner**: System
**Last Updated**: 2026-03-03

---

## Tasks Checklist

- [x] REST API client with HMAC-SHA256 signing
- [x] Rate limiting (max 10 concurrent, 100ms delay)
- [x] Retry logic for 429/403 rate-limit responses (3 attempts, exponential backoff)
- [x] Spot and Futures API support (separate keys, separate base URLs)
- [x] Testnet support
- [x] Kline/OHLCV data (spot + futures)
- [x] Account info and balance queries
- [x] Futures positions and open orders
- [x] Place/cancel futures orders
- [x] Place/cancel spot orders
- [x] Change leverage and margin type
- [x] Symbol price and funding rate
- [x] WebSocket market data streams (kline, ticker, depth)
- [x] Dynamic subscribe/unsubscribe at runtime
- [x] WebSocket reconnection with exponential backoff (max 10 attempts)
- [x] User Data Stream (listen key management, Spot + Futures)
- [x] User Data Stream Manager with keepalive (30-min interval)
- [ ] Order book snapshot REST endpoint

---

## Metadata

**Related Specs**:
- `FR-TRADING.md` — Paper/real trading that uses this integration
- `FR-REAL.md` — Real trading order placement

**Dependencies**:
- `reqwest` crate — async HTTP client
- `tokio-tungstenite` crate — async WebSocket client
- `hmac` + `sha2` crates — request signing
- `BinanceConfig` — configuration struct

**Spec Tags in code**: `@spec:FR-TRADING-005`, `@spec:FR-WEBSOCKET-001`, `@spec:FR-WEBSOCKET-002`, `@spec:FR-REAL-007`, `@spec:FR-REAL-008`

**Business Value**: Critical (all market data and trade execution)
**Technical Complexity**: High
**Priority**: Critical

---

## Overview

The Binance integration module (`rust-core-engine/src/binance/`) provides:
1. **REST API client** (`BinanceClient`) — HMAC-signed requests to Spot and Futures APIs
2. **WebSocket client** (`BinanceWebSocket`) — market data streams with dynamic subscription
3. **User Data Stream** (`UserDataStreamManager`) — real-time order/balance updates

All operations support testnet (`BINANCE_TESTNET=true`) and production environments.

---

## Functional Requirements

### FR-BINANCE-001: REST Client Authentication

**Priority**: Critical
**Status**: Completed
**Code**: `rust-core-engine/src/binance/client.rs:BinanceClient`

**Description**: All signed requests use HMAC-SHA256 with timestamp. Spot and Futures use separate API/secret key pairs.

**Implementation**:
- Header: `X-MBX-APIKEY: <api_key>`
- Query param: `signature=<hmac_sha256(query_string)>`
- Query param: `timestamp=<millis_since_epoch>`
- Keys sorted before signing (Binance requires consistent order)
- Futures endpoints (`/fapi/`) use `futures_api_key` / `futures_secret_key` if configured, else fall back to spot keys

**Constants**:
```rust
MAX_CONCURRENT_REQUESTS: 10   // Semaphore
REQUEST_DELAY_MS: 100         // Between requests
MAX_RETRIES: 3                // On 429/403
```

---

### FR-BINANCE-002: Rate Limiting

**Priority**: High
**Status**: Completed
**Code**: `rust-core-engine/src/binance/client.rs:make_request()`

**Description**: Prevent Binance API bans via concurrency semaphore and per-request delay.

**Mechanism**:
- `Arc<Semaphore>` with 10 permits — max 10 in-flight requests
- 100ms sleep after acquiring permit
- On 429 or 403: read `Retry-After` header (default 2s), multiply by attempt number, retry up to 3 times
- Non-rate-limit errors (other than 400): logged at ERROR, no retry
- 400 errors: logged at DEBUG (often benign, e.g. `-4046` "no need to change margin type")

---

### FR-BINANCE-003: Testnet vs Mainnet

**Priority**: High
**Status**: Completed
**Code**: `rust-core-engine/src/binance/mod.rs` tests, `BinanceConfig`

**Description**: Full testnet support for safe development and testing.

**Endpoints**:

| Environment | Spot REST | Futures REST | Spot WS | Futures WS |
|---|---|---|---|---|
| **Mainnet** | `https://api.binance.com` | `https://fapi.binance.com` | `wss://stream.binance.com:9443/ws` | `wss://fstream.binance.com` |
| **Testnet** | `https://testnet.binance.vision` | `https://testnet.binancefuture.com` | `wss://testnet.binance.vision/ws` | `wss://stream.binancefuture.com/ws` |

**Config field**: `BinanceConfig.testnet: bool`
**Trading modes**: `PaperTrading`, `RealTestnet`, `RealLive`

---

### FR-BINANCE-004: REST API Methods

**Priority**: Critical
**Status**: Completed
**Code**: `rust-core-engine/src/binance/client.rs`

#### Public Endpoints (no auth required)

| Method | Function | Binance Endpoint |
|---|---|---|
| GET | `get_klines(symbol, interval, limit)` | `/api/v3/klines` |
| GET | `get_futures_klines(symbol, interval, limit)` | `/fapi/v1/klines` |
| GET | `get_symbol_price(symbol)` | `/api/v3/ticker/price` |
| GET | `get_funding_rate(symbol)` | `/fapi/v1/fundingRate` |

#### Private Endpoints (HMAC signed)

| Method | Function | Binance Endpoint |
|---|---|---|
| GET | `get_account_info()` | `/api/v3/account` |
| GET | `get_futures_account()` | `/fapi/v2/account` |
| GET | `get_futures_positions()` | `/fapi/v2/positionRisk` |
| GET | `get_futures_user_trades(symbol, limit)` | `/fapi/v1/userTrades` |
| GET | `get_all_futures_orders(symbol, limit)` | `/fapi/v1/allOrders` |
| GET | `get_open_orders(symbol?)` | `/fapi/v1/openOrders` |
| POST | `place_futures_order(order)` | `/fapi/v1/order` |
| DELETE | `cancel_order(symbol, order_id?, client_order_id?)` | `/fapi/v1/order` |
| POST | `change_leverage(symbol, leverage)` | `/fapi/v1/leverage` |
| POST | `change_margin_type(symbol, margin_type)` | `/fapi/v1/marginType` |
| POST | `place_spot_order(order)` | `/api/v3/order` |

#### User Data Stream Management

| Method | Function | Binance Endpoint |
|---|---|---|
| POST | `create_listen_key()` | `/api/v3/userDataStream` |
| PUT | `keepalive_listen_key(key)` | `/api/v3/userDataStream` |
| DELETE | `close_listen_key(key)` | `/api/v3/userDataStream` |
| POST | `create_futures_listen_key()` | `/fapi/v1/listenKey` |
| PUT | `keepalive_futures_listen_key(key)` | `/fapi/v1/listenKey` |

---

### FR-BINANCE-005: Order Types

**Priority**: Critical
**Status**: Completed
**Code**: `rust-core-engine/src/binance/types.rs:NewOrderRequest`

**Supported order fields** (`NewOrderRequest`):
- `side`: `BUY` / `SELL`
- `type`: `MARKET`, `LIMIT`, `STOP_LOSS`, `STOP_LOSS_LIMIT`, `TAKE_PROFIT`, `TAKE_PROFIT_LIMIT`
- `time_in_force`: `GTC`, `IOC`, `FOK`
- `reduce_only`: bool (Futures)
- `position_side`: `LONG` / `SHORT` / `BOTH` (Futures hedge mode)
- `working_type`: `MARK_PRICE` / `CONTRACT_PRICE`
- `price_protect`: bool
- `close_position`: bool
- `new_client_order_id`: custom tracking ID

**Spot-specific** (`SpotOrderRequest`):
- `quote_order_qty`: buy by quote amount (market orders)
- `stop_price`: for stop-limit orders

---

### FR-BINANCE-006: WebSocket Market Data Streams

**Priority**: Critical
**Status**: Completed
**Code**: `rust-core-engine/src/binance/websocket.rs:BinanceWebSocket`
**Spec tags**: `@spec:FR-WEBSOCKET-001`, `@spec:FR-WEBSOCKET-002`

**Description**: Real-time market data via persistent WebSocket connection.

**Stream types per symbol**:

| Stream | Format | Data |
|---|---|---|
| Kline | `{symbol}@kline_{interval}` | OHLCV + close flag |
| 24hr Ticker | `{symbol}@ticker` | Price change, bid/ask, volume |
| Order Book | `{symbol}@depth@100ms` | Bids/asks delta updates |

**URL construction**:
- Single stream: `{ws_url}/{stream_name}`
- Multiple streams: `{ws_url_without_/ws}/stream?streams={s1}/{s2}/...`

**Dynamic subscriptions** (runtime add/remove):
```rust
ws.subscribe_symbol("ETHUSDT", vec!["1m", "5m"])
ws.unsubscribe_symbol("ETHUSDT", vec!["1m"])
```
Sends `{"method":"SUBSCRIBE","params":[...],"id":<n>}` over live connection.

**Reconnection**: exponential backoff `2^min(attempt,6)` seconds, max 10 attempts.

**Message handling**:
- Subscription ACK: `{"result":null,"id":N}` — ignored
- Combined stream: `{"stream":"...", "data":{...}}`
- Raw stream: direct data payload

---

### FR-BINANCE-007: WebSocket Reconnection

**Priority**: High
**Status**: Completed
**Code**: `rust-core-engine/src/binance/websocket.rs:start()`

**Reconnection behavior**:
- Max attempts: 10 (`max_reconnect_attempts`)
- Delay: `2^min(attempt, 6)` seconds (2s, 4s, 8s, 16s, 32s, 64s, 64s...)
- Normal close (server sends `Message::Close`): logs info, breaks loop
- Error: logs, increments counter, sleeps, reconnects
- Exhausted: returns `Err`

**Ping/pong**: incoming `Ping` messages answered with `Pong` automatically.

---

### FR-BINANCE-008: User Data Stream

**Priority**: High
**Status**: Completed
**Code**: `rust-core-engine/src/binance/user_data_stream.rs`
**Spec tags**: `@spec:FR-REAL-007`, `@spec:FR-REAL-008`

**Description**: Real-time private user events (orders, balances) via `listenKey`-authenticated WebSocket.

**UserDataStreamManager**:
- `.new(client)` — Spot mode
- `.new_futures(client)` — Futures mode
- `.with_config(client, config, use_futures)` — custom config

**Events broadcast** (`UserDataStreamEvent`):
| Event | Trigger |
|---|---|
| `Connected` | WebSocket established |
| `Disconnected` | WebSocket closed |
| `ExecutionReport(Box<ExecutionReport>)` | Order fill, cancel, new |
| `AccountPosition(OutboundAccountPosition)` | Balance change |
| `BalanceUpdate(BalanceUpdate)` | Deposit/withdrawal |
| `Error(String)` | Connection error |

**Listen key lifecycle**:
- Create: `POST /userDataStream` → `{listenKey: "..."}`
- Valid: 60 minutes
- Keepalive: `PUT /userDataStream?listenKey=...` every **30 minutes**
- Close: `DELETE /userDataStream?listenKey=...`

**UserDataStreamConfig defaults**:
```rust
keepalive_interval_secs: 1800   // 30 minutes
reconnect_delay_secs: 5
max_reconnect_attempts: 10
channel_buffer_size: 100
```

---

## Key Types

### BinanceConfig

```rust
pub struct BinanceConfig {
    pub api_key: String,
    pub secret_key: String,
    pub futures_api_key: String,      // Falls back to api_key if empty
    pub futures_secret_key: String,   // Falls back to secret_key if empty
    pub base_url: String,             // Spot REST
    pub ws_url: String,               // Spot WebSocket
    pub futures_base_url: String,     // Futures REST
    pub futures_ws_url: String,       // Futures WebSocket
    pub testnet: bool,
    pub trading_mode: TradingMode,    // PaperTrading | RealTestnet | RealLive
}
```

### StreamEvent enum (emitted by BinanceWebSocket)

- `StreamEvent::Kline(KlineEvent)` — kline bar update
- `StreamEvent::Ticker(TickerEvent)` — 24h rolling stats
- `StreamEvent::OrderBook(OrderBookEvent)` — bid/ask depth delta

### FuturesPosition fields

| Field | Meaning |
|---|---|
| `position_amt` | Quantity (negative = SHORT) |
| `unrealized_pnl` | Current P&L in USDT |
| `leverage` | Current leverage |
| `margin_type` | `cross` / `isolated` |
| `liquidation_price` | Price at forced liquidation |
| `position_side` | `LONG` / `SHORT` / `BOTH` |

---

## Module Structure

```
rust-core-engine/src/binance/
├── mod.rs              Re-exports: BinanceClient, BinanceWebSocket, UserDataStreamManager, types
├── client.rs           REST API client (HMAC signing, rate limiting, retries)
├── types.rs            Serde types: Kline, TickerEvent, OrderBookEvent, FuturesPosition, ...
├── websocket.rs        WebSocket market data streams
└── user_data_stream.rs User Data Stream manager (order/balance events)
```

---

## Dependencies

- `reqwest` ^0.12 — async HTTP, 30s timeout, custom user-agent
- `tokio-tungstenite` — async WebSocket
- `hmac` + `sha2` — HMAC-SHA256 signing
- `hex` — signature hex encoding
- `futures-util` — stream combinators
- `tokio::sync::Semaphore` — rate limiter
- `url` — URL construction

---

## Test Cases

- `TC-TRADING-030`, `TC-TRADING-031`, `TC-TRADING-032` — REST client
- `TC-INTEGRATION-008`, `TC-INTEGRATION-009` — WebSocket connection
- `TC-REAL-007`, `TC-REAL-008` — User data stream
- In `mod.rs` tests: constructor tests, type instantiation, stream event enum
