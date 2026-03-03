# WebSocket & Real-Time Communication

## Quick Reference

### Code Locations
```
rust-core-engine/src/
├── binance/websocket.rs - Binance WebSocket client
│   ├── subscribe_symbol() - Subscribe to market data streams (line 70)
│   ├── connect_and_run() - Establish connection and run loop (line 123)
│   └── handle_message() - Process incoming messages (line 292)
└── market_data/processor.rs - Real-time data processing

nextjs-ui-dashboard/src/
├── hooks/useWebSocket.ts - React WebSocket hook
└── contexts/WebSocketContext.tsx - WebSocket provider
```

Note: There is NO separate `rust-core-engine/src/websocket/` directory. Event broadcasting
is handled within the paper trading engine and Binance WebSocket module directly.

### WebSocket Endpoints

#### Binance WebSocket (Input)
```
wss://stream.binance.com:9443/ws
```
Streams:
- `btcusdt@trade` - Real-time trades
- `btcusdt@kline_1m` - 1-minute candles
- `btcusdt@depth` - Order book updates
- `!ticker@arr` - All tickers

#### Frontend WebSocket (Output)
```
ws://localhost:8080/ws
```
Events sent to frontend:
- `price_update` - Real-time price changes
- `signal_generated` - New AI trading signal
- `trade_executed` - Trade execution notification
- `portfolio_update` - Portfolio state change
- `risk_event` - Risk limit triggered
- `market_data` - New candle/ticker data

---

## Features

### Real-Time Market Data
- Price updates: ~100ms latency
- Candle updates: Every 1 minute
- Order book: Real-time L2 depth
- 24/7 connection with auto-reconnect

### Event Broadcasting
- Multi-client support (100+ concurrent connections)
- Event filtering by subscription
- Automatic reconnection on disconnect

### Connection Management
- Heartbeat/ping-pong (30s interval)
- Auto-reconnect with exponential backoff
- Connection state tracking
- Error recovery

---

## Common Tasks

### Frontend: Connect to WebSocket
```typescript
import { useWebSocket } from '@/hooks/useWebSocket';

function Dashboard() {
  const { socket, connected, subscribe } = useWebSocket();

  useEffect(() => {
    subscribe('price_update', (data) => {
      console.log('Price:', data.symbol, data.price);
    });

    subscribe('signal_generated', (signal) => {
      console.log('New signal:', signal);
    });
  }, [subscribe]);

  return <div>Status: {connected ? 'Connected' : 'Disconnected'}</div>;
}
```

### Monitor WebSocket Activity
```bash
docker logs -f rust-core-engine-dev | grep -E "WebSocket|ws://"
```

---

## Troubleshooting

### Issue: WebSocket disconnects frequently
**Check**: `rust-core-engine/src/binance/websocket.rs`
- Verify heartbeat interval (should be 30s)
- Check for network instability
- Look for "ping timeout" in logs

### Issue: No price updates received
**Check**: `rust-core-engine/src/binance/websocket.rs`
- Verify `connect_and_run()` is active (line 123)
- Check `subscribe_symbol()` was called for the symbol (line 70)
- Review market data processing logs

### Issue: Events not reaching frontend
**Check**: `nextjs-ui-dashboard/src/contexts/WebSocketContext.tsx`
- Verify client is subscribed to event type
- Check WebSocket URL: `ws://localhost:8080/ws`
- Monitor browser console for connection errors

### Issue: "Connection refused" error
- Verify URL is `ws://localhost:8080/ws` (not `wss://`)
- Check CORS settings in Rust backend
- Ensure port 8080 is not blocked

---

## Event Types Reference

### price_update
```json
{
  "event_type": "price_update",
  "data": {
    "symbol": "BTCUSDT",
    "price": 50000.00,
    "volume": 1234.56,
    "timestamp": "2026-03-03T10:00:00Z"
  }
}
```

### signal_generated
```json
{
  "event_type": "signal_generated",
  "data": {
    "symbol": "BTCUSDT",
    "signal": "LONG",
    "entry_price": 50000.00,
    "strategy": "rsi",
    "confidence": 0.85
  }
}
```

### trade_executed
```json
{
  "event_type": "trade_executed",
  "data": {
    "trade_id": "abc123",
    "symbol": "BTCUSDT",
    "type": "LONG",
    "entry_price": 50035.00,
    "quantity": 0.1,
    "leverage": 10
  }
}
```

---

## Performance

| Metric | Value |
|--------|-------|
| Binance to Backend | ~50ms |
| Backend to Frontend | ~20ms |
| End-to-end | ~70ms |
| Max concurrent clients | 100+ |

---

## Related Documentation

- **Specs**: `specs/01-requirements/1.1-functional-requirements/FR-WEBSOCKET.md`
- **Frontend Hook**: `nextjs-ui-dashboard/src/hooks/useWebSocket.ts`
- **Tests**: `specs/03-testing/3.2-test-cases/TC-WEBSOCKET.md`

**Last Updated**: 2026-03-03
**Latency**: <100ms end-to-end
