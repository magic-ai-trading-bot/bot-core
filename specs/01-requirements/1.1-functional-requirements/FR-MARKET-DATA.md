# Market Data Processing - Functional Requirements

**Spec ID**: FR-MARKET-DATA-001 to FR-MARKET-DATA-005
**Version**: 1.0
**Status**: ☑ Approved
**Owner**: Platform Team
**Last Updated**: 2025-10-10

---

## Tasks Checklist

- [x] Requirements gathered
- [x] Design completed
- [x] Implementation done
- [ ] Tests written
- [ ] Documentation updated
- [ ] Code reviewed
- [ ] Deployed to staging
- [ ] Deployed to production

---

## Metadata

**Related Specs**:
- Related FR: [FR-WEBSOCKET-001](./FR-WEBSOCKET.md)
- Related API: [API_SPEC.md](../../02-technical-specifications/API_SPEC.md)
- Related Data Model: [DATA_MODELS.md](../../02-technical-specifications/DATA_MODELS.md)

**Dependencies**:
- Depends on: MongoDB for storage
- Depends on: Binance API/WebSocket for real-time data
- Blocks: Trading strategy execution, AI analysis

**Business Value**: High
**Technical Complexity**: High
**Priority**: ☑ Critical

---

## Overview

The Market Data Processing system is responsible for ingesting, processing, storing, and distributing real-time and historical cryptocurrency market data from Binance. This system provides the foundation for all trading, analysis, and visualization features by maintaining an up-to-date, reliable stream of market information including candlestick data, trade ticks, order book updates, and price feeds across multiple symbols and timeframes.

---

## Business Context

**Problem Statement**:
Trading bots require accurate, real-time market data to make informed trading decisions. Without a robust data pipeline that can handle high-frequency updates, validate data integrity, aggregate across multiple timeframes, and provide low-latency access, the trading system cannot operate effectively or safely.

**Business Goals**:
- Provide sub-100ms latency for real-time price updates
- Support multiple cryptocurrency symbols (BTC, ETH, BNB, etc.)
- Maintain historical data for backtesting and analysis
- Enable multi-timeframe analysis (1m, 5m, 15m, 1h, 4h, 1d)
- Ensure 99.9% data availability and accuracy
- Support dynamic symbol addition/removal without system restart

**Success Metrics**:
- Data latency: < 100ms from Binance to clients
- Update frequency: Real-time (sub-second for 1m timeframe)
- Cache hit rate: > 95%
- Data accuracy: 100% (validated against Binance)
- System uptime: > 99.9%
- Concurrent symbol support: 50+ symbols
- WebSocket connection stability: < 5 reconnections per day

---

## Functional Requirements

### FR-MARKET-DATA-001: Real-Time Market Data Feed

**Priority**: ☑ Critical
**Status**: ☑ Completed
**Code Tags**: `@spec:FR-MARKET-DATA-001`

**Description**:
The system MUST establish and maintain persistent WebSocket connections to Binance Futures API to receive real-time market data updates. This includes candlestick (kline) data, 24-hour ticker statistics, and order book depth updates for all configured trading symbols and timeframes.

**Acceptance Criteria**:
- [x] System establishes WebSocket connection to Binance Futures stream endpoint
- [x] Subscription to kline streams for all configured symbols and timeframes
- [x] Subscription to 24hr ticker streams for price change statistics
- [x] Subscription to depth streams (@100ms) for order book updates
- [x] Handle combined stream format (multiple symbols/timeframes in single connection)
- [x] Support both testnet and production Binance endpoints
- [x] Automatic reconnection on connection loss with exponential backoff
- [x] Maximum reconnection attempts: 10 with delays: 2s, 4s, 8s, 16s, 32s, 64s
- [x] Message parsing for all event types: kline, 24hrTicker, depthUpdate
- [x] Heartbeat/ping-pong mechanism to keep connection alive
- [x] Graceful connection closure handling
- [x] Error logging for connection failures and message parsing errors

**Data Requirements**:
- **Input**:
  - Symbols list (e.g., ["BTCUSDT", "ETHUSDT"])
  - Timeframes list (e.g., ["1m", "5m", "15m", "1h", "4h", "1d"])
  - Binance WebSocket URL (testnet or production)
- **Output**:
  - StreamEvent::Kline - Real-time candlestick updates
  - StreamEvent::Ticker - 24-hour ticker statistics
  - StreamEvent::OrderBook - Order book depth updates

**Implementation Notes**:
- Code Location: `rust-core-engine/src/binance/websocket.rs`
- Uses: `tokio-tungstenite` for WebSocket client
- Stream format: `wss://stream.binance.com:9443/ws/stream?streams={stream1}/{stream2}/...`
- Single stream format: `wss://stream.binance.com:9443/ws/{stream}`
- Kline stream format: `{symbol}@kline_{timeframe}` (e.g., "btcusdt@kline_1m")
- Ticker stream format: `{symbol}@ticker` (e.g., "btcusdt@ticker")
- Depth stream format: `{symbol}@depth@100ms` (e.g., "btcusdt@depth@100ms")

**Dependencies**: Binance API availability, network connectivity
**Test Cases**: TC-MARKET-DATA-001, TC-MARKET-DATA-002, TC-MARKET-DATA-003

---

### FR-MARKET-DATA-002: Market Data Processing and Validation

**Priority**: ☑ Critical
**Status**: ☑ Completed
**Code Tags**: `@spec:FR-MARKET-DATA-002`

**Description**:
The system MUST process incoming market data by validating data integrity, converting data formats, aggregating across timeframes, and updating in-memory caches with minimal latency. The processor acts as the central hub for all market data operations.

**Acceptance Criteria**:
- [x] Validate incoming data structure and required fields
- [x] Convert string-based price/volume data to f64 numeric types
- [x] Handle invalid/malformed data gracefully with default values (0.0)
- [x] Update real-time price cache immediately on every kline update
- [x] Distinguish between closed and open candles (is_closed flag)
- [x] Update existing candle if same open_time (real-time updates)
- [x] Append new candle if different open_time
- [x] Maintain cache size limits per symbol/timeframe (configurable max_size)
- [x] Support multiple timeframe aggregation in single processor
- [x] Calculate 24-hour statistics: volume, price change, price change percentage
- [x] Thread-safe concurrent access using Arc<DashMap> and RwLock
- [x] Log all data updates at appropriate levels (debug, info, warn, error)
- [x] Periodic refresh for longer timeframes (1h, 4h, 1d) via REST API

**Data Structures**:

```rust
// CandleData - Internal representation
pub struct CandleData {
    pub open_time: i64,        // Millisecond timestamp
    pub close_time: i64,
    pub open: f64,
    pub high: f64,
    pub low: f64,
    pub close: f64,
    pub volume: f64,
    pub quote_volume: f64,
    pub trades: i64,
    pub is_closed: bool,       // True if candle period is complete
}

// ChartData - API response format
pub struct ChartData {
    pub symbol: String,
    pub timeframe: String,
    pub candles: Vec<CandleData>,
    pub latest_price: f64,
    pub volume_24h: f64,
    pub price_change_24h: f64,
    pub price_change_percent_24h: f64,
}
```

**Processing Rules**:
- **String to Float Conversion**: Use `.parse::<f64>().unwrap_or(0.0)` for safety
- **Candle Update Logic**:
  - If `last_candle.open_time == new_candle.open_time`: Replace (real-time update)
  - If `last_candle.open_time != new_candle.open_time`: Append (new candle)
- **Cache Eviction**: FIFO - remove oldest when exceeding max_size
- **Price Update**: Always update price cache, even for non-closed candles (for real-time feel)
- **Timeframe Priority**: Shorter timeframes (1m, 5m) update more frequently via WebSocket

**Implementation Notes**:
- Code Location: `rust-core-engine/src/market_data/processor.rs`
- Cache Implementation: `rust-core-engine/src/market_data/cache.rs`
- Uses: DashMap for concurrent hash map, VecDeque for candle storage
- Configuration: `MarketDataConfig` with cache_size, kline_limit, update_interval_ms

**Dependencies**: FR-MARKET-DATA-001 (WebSocket feed)
**Test Cases**: TC-MARKET-DATA-010 to TC-MARKET-DATA-020

---

### FR-MARKET-DATA-003: Historical Data Retrieval and Backfill

**Priority**: ☑ High
**Status**: ☑ Completed
**Code Tags**: `@spec:FR-MARKET-DATA-003`

**Description**:
The system MUST fetch historical candlestick data from Binance REST API on startup and support on-demand backfill operations. Historical data is essential for strategy initialization, backtesting, and ensuring continuous data availability.

**Acceptance Criteria**:
- [x] Fetch historical klines on system startup for all configured symbols/timeframes
- [x] Use Binance `/fapi/v1/klines` REST API endpoint
- [x] Support configurable kline limit (default: 100, max: 1000)
- [x] Check MongoDB for cached historical data before fetching from API
- [x] Use cached data if available and fetch only latest updates
- [x] Store fetched historical data to MongoDB for persistence
- [x] Rate limiting: 100ms delay between requests to avoid API limits
- [x] Handle API errors gracefully (log warning, continue with next symbol)
- [x] Support force refresh for specific symbols on demand
- [x] Backfill missing data gaps automatically
- [x] Load historical data before starting WebSocket connections
- [x] Provide API endpoint for manual historical data refresh

**Historical Data Loading Flow**:
1. **On Startup**:
   - For each symbol and timeframe:
     - Query MongoDB: `get_market_data(symbol, timeframe, limit)`
     - If cached data exists: Load into memory cache
     - Fetch latest 10 candles from Binance API to update cache
     - If no cached data: Fetch `kline_limit` candles from Binance API
     - Store fetched data to MongoDB
     - Wait 100ms (rate limiting)

2. **On-Demand Refresh**:
   - API endpoint: `POST /api/market-data/refresh/{symbol}`
   - Fetch latest klines for all timeframes
   - Update MongoDB and memory cache
   - Return success/failure status

**Binance Klines API Parameters**:
```
GET /fapi/v1/klines
Required: symbol, interval
Optional: startTime, endTime, limit (default 500, max 1500)
Returns: [
  [openTime, open, high, low, close, volume, closeTime,
   quoteAssetVolume, numberOfTrades, takerBuyBaseAssetVolume,
   takerBuyQuoteAssetVolume, ignore]
]
```

**Implementation Notes**:
- Code Location: `rust-core-engine/src/market_data/processor.rs:load_historical_data()`
- Binance Client: `rust-core-engine/src/binance/client.rs:get_futures_klines()`
- Storage: `rust-core-engine/src/storage/mod.rs:store_market_data()`
- Timeframes: "1m", "5m", "15m", "1h", "4h", "1d"
- Configuration: `kline_limit` in `MarketDataConfig`

**Dependencies**: MongoDB, Binance REST API
**Test Cases**: TC-MARKET-DATA-030 to TC-MARKET-DATA-040

---

### FR-MARKET-DATA-004: Market Data Storage and Persistence

**Priority**: ☑ High
**Status**: ☑ Completed
**Code Tags**: `@spec:FR-MARKET-DATA-004`

**Description**:
The system MUST persist market data to MongoDB for historical analysis, backtesting, and system recovery. The storage layer ensures data durability and provides efficient querying capabilities.

**Acceptance Criteria**:
- [x] Store candlestick data in MongoDB collection: `market_data`
- [x] Document structure includes: symbol, timeframe, kline data, timestamp
- [x] Create compound index on (symbol, timeframe, open_time) for fast queries
- [x] Support bulk insert operations for efficiency
- [x] Implement upsert logic to avoid duplicates (same symbol+timeframe+open_time)
- [x] Query historical data by symbol, timeframe, and optional limit
- [x] Support time-range queries for backtesting
- [x] Implement data retention policies (configurable)
- [x] Automatic cleanup of old data beyond retention period
- [x] Store 24-hour statistics alongside candle data
- [x] Provide API endpoints for historical data retrieval
- [x] Handle storage failures gracefully (log error, continue operation)

**MongoDB Schema**:
```json
{
  "_id": ObjectId,
  "symbol": "BTCUSDT",
  "timeframe": "1m",
  "open_time": 1609459200000,
  "close_time": 1609459260000,
  "open": 50000.0,
  "high": 50500.0,
  "low": 49500.0,
  "close": 50250.0,
  "volume": 1000.0,
  "quote_volume": 50000000.0,
  "trades": 100,
  "created_at": ISODate("2025-01-01T00:00:00Z"),
  "updated_at": ISODate("2025-01-01T00:01:00Z")
}
```

**Indexes**:
```javascript
// Primary query index
db.market_data.createIndex({ symbol: 1, timeframe: 1, open_time: -1 })

// Time-range query index
db.market_data.createIndex({ symbol: 1, timeframe: 1, open_time: 1, close_time: 1 })

// Cleanup index
db.market_data.createIndex({ created_at: 1 }, { expireAfterSeconds: 2592000 }) // 30 days
```

**Storage Operations**:
- **Insert/Update**: `storage.store_market_data(symbol, timeframe, &klines)`
- **Query**: `storage.get_market_data(symbol, timeframe, limit)`
- **Query Range**: `storage.get_market_data_range(symbol, timeframe, start_time, end_time)`
- **Cleanup**: Automatic TTL index or scheduled job

**Data Retention Policy**:
- 1-minute data: 7 days
- 5-minute data: 30 days
- 15-minute data: 90 days
- 1-hour data: 1 year
- 4-hour data: 2 years
- 1-day data: Indefinite

**Implementation Notes**:
- Code Location: `rust-core-engine/src/storage/mod.rs`
- MongoDB Driver: `mongodb` crate
- Connection Pool: max_connections = 10
- Database: `bot_core`
- Collection: `market_data`

**Dependencies**: MongoDB availability
**Test Cases**: TC-MARKET-DATA-050 to TC-MARKET-DATA-060

---

### FR-MARKET-DATA-005: Price Update Broadcasting

**Priority**: ☑ Critical
**Status**: ☑ Completed
**Code Tags**: `@spec:FR-MARKET-DATA-005`

**Description**:
The system MUST broadcast real-time price updates and chart data to connected WebSocket clients with minimal latency. This enables the frontend dashboard to display live market data and respond to market movements instantly.

**Acceptance Criteria**:
- [x] Broadcast price updates via WebSocket on every kline update
- [x] Support two message types: MarketData and ChartUpdate
- [x] MarketData: Sent on every price change (real-time)
- [x] ChartUpdate: Sent only when candle closes (is_closed = true)
- [x] Rate limiting: Max 100 updates per second per symbol
- [x] Message format: JSON with type, data, and timestamp fields
- [x] Broadcast to all connected clients (no filtering)
- [x] Handle broadcast failures gracefully (log warning, don't crash)
- [x] Calculate 24-hour statistics in real-time
- [x] Include latest price, volume, and price change percentage
- [x] Support subscription management (future: per-client filtering)
- [x] Maintain broadcast channel with capacity: 1000 messages

**WebSocket Message Formats**:

**MarketData Update** (every price update):
```json
{
  "type": "MarketData",
  "data": {
    "symbol": "BTCUSDT",
    "price": 50250.75,
    "price_change_24h": 250.50,
    "price_change_percent_24h": 0.5,
    "volume_24h": 10000.0,
    "timestamp": 1609459200000
  },
  "timestamp": "2025-01-01T00:00:00Z"
}
```

**ChartUpdate** (closed candles only):
```json
{
  "type": "ChartUpdate",
  "data": {
    "symbol": "BTCUSDT",
    "timeframe": "1m",
    "candle": {
      "timestamp": 1609459200000,
      "open": 50000.0,
      "high": 50500.0,
      "low": 49500.0,
      "close": 50250.0,
      "volume": 1000.0,
      "is_closed": true
    },
    "latest_price": 50250.0,
    "price_change_24h": 250.0,
    "price_change_percent_24h": 0.5,
    "volume_24h": 10000.0,
    "timestamp": 1609459200000
  },
  "timestamp": "2025-01-01T00:00:00Z"
}
```

**Broadcasting Logic**:
```rust
// On every kline update:
1. Update price cache with latest close price
2. Create MarketData message
3. Broadcast MarketData to all clients

// If candle is closed (is_this_kline_closed == true):
4. Create ChartUpdate message with full candle data
5. Broadcast ChartUpdate to all clients
```

**Performance Requirements**:
- Latency: < 50ms from Binance to frontend client
- Message rate: Handle 1000+ price updates per second
- Client capacity: Support 100+ concurrent WebSocket connections
- Message size: < 2KB per message
- Broadcast efficiency: O(n) where n = number of clients

**Implementation Notes**:
- Code Location: `rust-core-engine/src/market_data/processor.rs:handle_stream_event()`
- Broadcast Channel: `tokio::sync::broadcast` with capacity 1000
- WebSocket Handler: `rust-core-engine/src/api/mod.rs:handle_websocket()`
- Frontend Consumer: `nextjs-ui-dashboard/src/hooks/useWebSocket.ts`

**Error Handling**:
- If broadcast fails and receiver_count > 0: Log warning
- If broadcast fails and receiver_count == 0: Silent (no clients connected)
- Never panic on broadcast failure

**Dependencies**: FR-MARKET-DATA-002 (data processing), FR-WEBSOCKET-001 (WebSocket server)
**Test Cases**: TC-MARKET-DATA-070 to TC-MARKET-DATA-080

---

## Use Cases

### UC-MARKET-DATA-001: Real-Time Trading Dashboard

**Actor**: Trader using web dashboard
**Preconditions**:
- System is running and connected to Binance
- WebSocket connection established to backend
- Symbols configured: BTCUSDT, ETHUSDT

**Main Flow**:
1. User opens trading dashboard in browser
2. Frontend establishes WebSocket connection to `/ws`
3. System broadcasts real-time MarketData updates every second
4. Frontend receives price updates and updates UI chart
5. When 1-minute candle closes, system broadcasts ChartUpdate
6. Frontend appends new candle to chart
7. User sees live price movements and completed candles

**Alternative Flows**:
- **Alt 1**: WebSocket disconnection
  1. Frontend detects connection loss
  2. Automatic reconnection with exponential backoff
  3. On reconnect, fetch latest chart data via REST API
  4. Resume real-time updates

**Postconditions**:
- Dashboard displays current price within 100ms of Binance updates
- Chart shows completed candles for selected timeframe

**Exception Handling**:
- Connection timeout: Retry with backoff
- Invalid message: Log error, discard message, continue

---

### UC-MARKET-DATA-002: Historical Data Analysis

**Actor**: Trading algorithm or AI service
**Preconditions**:
- System has historical data for BTCUSDT 1-hour timeframe
- MongoDB contains at least 100 candles

**Main Flow**:
1. AI service requests historical data: `GET /api/chart/BTCUSDT/1h?limit=100`
2. System queries MongoDB for last 100 candles
3. System returns ChartData with candles, statistics, and latest price
4. AI service performs technical analysis on historical data
5. AI service generates trading signal
6. System stores analysis result

**Alternative Flows**:
- **Alt 1**: No historical data in MongoDB
  1. System detects empty cache
  2. Fetches historical klines from Binance API
  3. Stores to MongoDB
  4. Returns data to AI service

**Postconditions**:
- AI service has 100 hours of candlestick data
- Analysis results stored in MongoDB

---

### UC-MARKET-DATA-003: Multi-Timeframe Analysis

**Actor**: Paper trading engine
**Preconditions**:
- System configured with timeframes: 1m, 5m, 15m, 1h, 4h
- All timeframes have historical data loaded

**Main Flow**:
1. Paper trading engine requests analysis for BTCUSDT
2. Market data analyzer fetches data for all timeframes
3. System retrieves from cache: 1m (last 50), 5m (last 50), 15m (last 50), 1h (last 100), 4h (last 100)
4. Analyzer calculates multi-timeframe signal
5. System combines signals with weighted scores
6. Returns overall trading signal with confidence

**Alternative Flows**:
- **Alt 1**: Insufficient data for timeframe
  1. System fetches additional data from MongoDB or Binance API
  2. Updates cache
  3. Continues analysis

**Postconditions**:
- Trading engine receives multi-timeframe signal
- Signal confidence calculated from all timeframes

---

## Interface Requirements

**REST API Endpoints** (see API_SPEC.md for details):
```
GET  /api/prices                        - Latest prices for all symbols
GET  /api/overview                      - Market overview with latest analysis
GET  /api/candles/{symbol}/{timeframe}  - Raw candle data
GET  /api/chart/{symbol}/{timeframe}    - Comprehensive chart data
GET  /api/charts?symbols=...&timeframes=... - Multi-symbol chart data
POST /api/symbols                       - Add new symbol to track
DELETE /api/symbols/{symbol}            - Remove symbol
GET  /api/supported-symbols             - List configured symbols
```

**WebSocket Messages** (see FR-WEBSOCKET.md for protocol details):
- Incoming: None (server-to-client only for market data)
- Outgoing: MarketData, ChartUpdate

**External Systems**:
- Binance Futures API: WebSocket and REST endpoints
- MongoDB: Data persistence
- Python AI Service: Analysis requests

---

## Non-Functional Requirements

**Performance**:
- WebSocket latency: < 100ms end-to-end
- REST API response time: < 200ms (cached), < 1s (database query)
- Cache access time: < 1ms
- Throughput: 10,000 price updates per second
- Concurrent symbols: 50+

**Scalability**:
- Horizontal scaling: Yes (with shared MongoDB)
- Cache strategy: In-memory with LRU eviction
- Connection pooling: MongoDB connection pool (10 connections)

**Reliability**:
- Uptime target: 99.9%
- Error rate: < 0.1%
- Data accuracy: 100% (validated against Binance)
- Recovery time objective (RTO): 30 seconds
- Recovery point objective (RPO): 0 (real-time data, no data loss acceptable)

**Maintainability**:
- Code coverage: 70% (actual), target 80%
- Logging: Structured logging at debug, info, warn, error levels
- Monitoring: Metrics for cache size, update rate, latency, errors

---

## Monitoring & Observability

**Metrics to Track**:
- `market_data.cache.size` - Current cache size per symbol/timeframe (Gauge)
- `market_data.updates.total` - Total price updates received (Counter)
- `market_data.updates.rate` - Updates per second (Rate)
- `market_data.latency` - Time from Binance to cache update (Histogram)
- `market_data.errors.total` - Data processing errors (Counter)
- `market_data.websocket.reconnections` - Binance WebSocket reconnections (Counter)
- `market_data.storage.writes` - MongoDB write operations (Counter)
- `market_data.storage.errors` - Storage failures (Counter)
- `market_data.broadcast.sent` - Messages broadcast to clients (Counter)
- `market_data.broadcast.dropped` - Failed broadcast attempts (Counter)

**Logging**:
- Log level: INFO for operations, DEBUG for data flow, ERROR for failures
- Key log events:
  1. WebSocket connection established/closed
  2. Historical data loading started/completed
  3. Cache updates (debug level)
  4. Storage operations (info level)
  5. Broadcast failures (warn level)
  6. Data validation errors (error level)

**Alerts**:
- Alert 1: WebSocket disconnected for > 30 seconds - CRITICAL
- Alert 2: Data latency > 500ms - HIGH
- Alert 3: Storage write failures > 10 per minute - HIGH
- Alert 4: Cache size exceeds 90% of limit - MEDIUM
- Alert 5: Reconnection attempts > 5 in 1 hour - MEDIUM

**Dashboards**:
- Dashboard 1: Real-time data flow (updates/sec, latency, cache hit rate)
- Dashboard 2: Storage health (write rate, read rate, storage size, errors)
- Dashboard 3: WebSocket health (connections, reconnections, uptime)

---

## Traceability

**Requirements**:
- User Story: Trading system needs real-time market data
- Business Rule: [BUSINESS_RULES.md#market-data-requirements](../../BUSINESS_RULES.md)

**Design**:
- Architecture: Multi-layer data pipeline (ingest → process → store → broadcast)
- API Spec: [API_SPEC.md#market-data-endpoints](../../02-technical-specifications/API_SPEC.md)
- Data Model: [DATA_MODELS.md#candlestick-data](../../02-technical-specifications/DATA_MODELS.md)

**Test Cases**:
- Unit: TC-MARKET-DATA-001 to TC-MARKET-DATA-020
- Integration: TC-MARKET-DATA-030 to TC-MARKET-DATA-060
- E2E: TC-MARKET-DATA-070 to TC-MARKET-DATA-090
- Performance: TC-MARKET-DATA-100 to TC-MARKET-DATA-110

**Code Implementation**:
- Processor: `rust-core-engine/src/market_data/processor.rs` (@spec:FR-MARKET-DATA-001, @spec:FR-MARKET-DATA-002, @spec:FR-MARKET-DATA-005)
- Cache: `rust-core-engine/src/market_data/cache.rs` (@spec:FR-MARKET-DATA-002)
- Analyzer: `rust-core-engine/src/market_data/analyzer.rs` (@spec:FR-MARKET-DATA-002)
- Binance WebSocket: `rust-core-engine/src/binance/websocket.rs` (@spec:FR-MARKET-DATA-001)
- Binance Client: `rust-core-engine/src/binance/client.rs` (@spec:FR-MARKET-DATA-003)
- Storage: `rust-core-engine/src/storage/mod.rs` (@spec:FR-MARKET-DATA-004)
- API Routes: `rust-core-engine/src/api/mod.rs` (@spec:FR-MARKET-DATA-005)

---

## Risks & Mitigations

| Risk | Impact | Probability | Mitigation |
|------|--------|-------------|------------|
| Binance API downtime | High | Medium | Automatic reconnection, fallback to cached data, alerts |
| Data corruption | High | Low | Validation on ingest, checksums, data verification against multiple sources |
| Memory overflow | Medium | Medium | Cache size limits, LRU eviction, memory monitoring alerts |
| Network latency | Medium | Medium | Connection pooling, timeout configuration, multiple data centers |
| MongoDB unavailability | High | Low | In-memory cache can continue, queue writes for retry, replica sets |
| WebSocket message loss | Medium | Low | Sequence numbers (future), periodic full data sync, reconciliation |
| Rate limit exceeded | Medium | Medium | Request throttling (100ms delays), respect Binance limits, IP whitelisting |
| Incorrect timeframe aggregation | High | Low | Comprehensive unit tests, integration tests against known data |

---

## Open Questions

- [x] Should we implement message deduplication for WebSocket updates? **RESOLVED**: Not needed, handled by cache update logic
- [x] What is the optimal cache size per symbol/timeframe? **RESOLVED**: 100 candles (configurable)
- [x] Should we support symbol filtering for WebSocket broadcasts? **DEFERRED**: Future enhancement, currently broadcast all
- [ ] How to handle symbol delisting on Binance? **PENDING**: Need monitoring and graceful degradation strategy
- [ ] Should we support Level 2 order book (full depth)? **PENDING**: Evaluate bandwidth and storage requirements

---

## Changelog

| Version | Date | Author | Changes |
|---------|------|--------|---------|
| 1.0 | 2025-10-10 | Platform Team | Initial specification based on existing implementation |

---

## Appendix

**References**:
- [Binance Futures WebSocket API Documentation](https://binance-docs.github.io/apidocs/futures/en/#websocket-market-streams)
- [Binance Futures REST API Documentation](https://binance-docs.github.io/apidocs/futures/en/#kline-candlestick-data)

**Glossary**:
- **Kline**: Candlestick data representing OHLCV (Open, High, Low, Close, Volume) for a time period
- **Timeframe**: The period of each candle (1m = 1 minute, 1h = 1 hour, 1d = 1 day)
- **Symbol**: Trading pair identifier (e.g., BTCUSDT = Bitcoin/Tether)
- **Tick**: Individual trade event
- **Order Book**: List of buy (bids) and sell (asks) orders
- **24hr Ticker**: 24-hour rolling window statistics for a symbol
- **OHLCV**: Open, High, Low, Close, Volume - standard candle data

**Example WebSocket Stream Names**:
```
btcusdt@kline_1m          # Bitcoin 1-minute candles
ethusdt@kline_5m          # Ethereum 5-minute candles
btcusdt@ticker            # Bitcoin 24hr ticker
btcusdt@depth@100ms       # Bitcoin order book (100ms updates)
```

**Example API Request/Response**:

Request:
```http
GET /api/chart/BTCUSDT/1h?limit=24 HTTP/1.1
Host: localhost:8080
```

Response:
```json
{
  "success": true,
  "data": {
    "symbol": "BTCUSDT",
    "timeframe": "1h",
    "candles": [
      {
        "timestamp": 1609459200000,
        "open": 50000.0,
        "high": 50500.0,
        "low": 49500.0,
        "close": 50250.0,
        "volume": 1000.0
      }
    ],
    "latest_price": 50250.0,
    "volume_24h": 24000.0,
    "price_change_24h": 250.0,
    "price_change_percent_24h": 0.5
  },
  "error": null
}
```

---

**Remember**: Update TRACEABILITY_MATRIX.md when implementation is complete!
