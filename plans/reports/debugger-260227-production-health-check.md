# Báo Cáo Kiểm Tra Hệ Thống Production VPS
**Thời gian kiểm tra**: 2026-02-27 20:25-20:30 (GMT+7)
**VPS**: 180.93.2.247 | **Uptime VPS**: 83 ngày 3 giờ 49 phút

---

## TÓM TẮT TỔNG THỂ

| Service | Tình trạng | Ghi chú |
|---------|-----------|---------|
| python-ai-service | DEGRADED | /predict-trend lỗi 500, thiếu 4H data |
| rust-core-engine | DEGRADED | 2702 WARN, analyzer lỗi liên tục |
| mcp-server | HEALTHY | 84MB RAM, hoạt động tốt |
| openclaw | WARNING | "fetch failed" định kỳ, cron OK |
| nextjs-ui-dashboard | HEALTHY | HTTP 200 |
| mongodb | HEALTHY | 19 connections, 18720 candle docs |
| redis-cache | HEALTHY | 1MB RAM, ping OK |

**Đánh giá tổng thể**: Hệ thống chạy được nhưng có 2 lỗi nghiêm trọng đang diễn ra liên tục, gây degraded performance và AI analysis không hoàn chỉnh.

---

## 1. CONTAINER STATUS & UPTIME

```
mcp-server            Up 3 hours (healthy)    port 8090
python-ai-service     Up 6 hours (healthy)    port 8000
rust-core-engine      Up 33 hours (healthy)   port 8080
nextjs-ui-dashboard   Up 2 days (healthy)     port 3000
openclaw              Up 6 hours (healthy)    port 18789
mongodb               Up 6 days (healthy)     port 27017
redis-cache           Up 8 days (healthy)     port 6379
```

**Lưu ý**: python-ai-service, mcp-server, openclaw đều restart trong vòng 6 giờ qua (trước đó đã chạy 2+ ngày). Nguyên nhân restart chưa rõ - có thể do OOM hoặc manual restart.

---

## 2. RESOURCE USAGE

| Container | CPU | RAM | RAM Limit | RAM % |
|-----------|-----|-----|-----------|-------|
| openclaw | 0.13% | **360MB** | 768MB | **46.9%** |
| nextjs-ui | 0.37% | 89MB | 512MB | 17.4% |
| mcp-server | 0.00% | 85MB | 512MB | 16.5% |
| mongodb | 0.76% | 174MB | 2GB | 8.5% |
| python-ai | 0.48% | 122MB | 2GB | 5.95% |
| rust-engine | 1.03% | **22MB** | 2GB | 1.08% |
| redis-cache | 0.66% | 6MB | 256MB | 2.34% |

**VPS tổng thể**:
- RAM: 1.5GB used / 5.8GB total (26%) — bình thường
- Disk: 24GB / 69GB (37%) — ổn
- Load average: 0.09, 0.08, 0.08 — rất thấp

**Điểm đáng chú ý**: openclaw dùng 360MB RAM (46.9%) — cao hơn trước (từng 69% nhưng với limit khác). Cần theo dõi.

---

## 3. LỖI NGHIÊM TRỌNG TÌM THẤY

### [BUG-001] CRITICAL: /predict-trend trả về 500 Error

**Service**: python-ai-service
**Endpoint**: POST /predict-trend
**Triệu chứng**:
```
ERROR:main:❌ Error predicting trend for BTCUSDT: 'close'
WARNING:main:⚠️ GPT-4 analysis failed, falling back to technical: 'close'
INFO: "POST /predict-trend HTTP/1.1" 500 Internal Server Error
```

**Root cause**: Field name mismatch giữa MongoDB và Python code.

MongoDB lưu candle với field: `close_price`, `open_price`, `high_price`, `low_price`

```json
{
  "symbol": "BTCUSDT",
  "timeframe": "1h",
  "open_time": 1770134400000,
  "open_price": 78094.2,
  "high_price": 78101.5,
  "low_price": 76448,
  "close_price": 76458.2,  // <-- field này
  "volume": 24540.659
}
```

Python code trong `_predict_trend_gpt4()` và `_predict_trend_technical()` dùng:
```python
df = pd.DataFrame(candles)
df["ema_200"] = df["close"].ewm(...)  # KeyError: 'close' vì field là 'close_price'
```

**Hệ quả**: Endpoint /predict-trend 100% thất bại. Mọi AI tool cần predict-trend đều trả về lỗi.

**Fix**: Rename field khi build DataFrame:
```python
df = pd.DataFrame(candles)
# Normalize field names
for old, new in [("close_price","close"),("open_price","open"),("high_price","high"),("low_price","low")]:
    if old in df.columns:
        df.rename(columns={old: new}, inplace=True)
```

---

### [BUG-002] HIGH: Rust Analyzer "error decoding response body" - 2702 lần

**Service**: rust-core-engine
**Module**: `market_data::analyzer`
**Tần suất**: 131 lần/mỗi symbol × 4 symbols × 4 timeframes = liên tục

```
WARN binance_trading_bot::market_data::analyzer: Failed to analyze BTCUSDT 1m: error decoding response body
WARN binance_trading_bot::market_data::analyzer: Failed to analyze BTCUSDT 5m: error decoding response body
WARN binance_trading_bot::market_data::analyzer: Failed to analyze BTCUSDT 1h: error decoding response body
WARN binance_trading_bot::market_data::processor: Analysis failed for BTCUSDT: All timeframe analyses failed for BTCUSDT
```

**Root cause**: Rust gọi `POST /ai/analyze` để phân tích market data. Python AI service trả về JSON response nhưng Rust không deserialize được — có thể do schema mismatch giữa `AnalysisResponse` struct (Rust) và response JSON thực tế của Python.

Test thực tế: Endpoint `/ai/analyze` hoạt động và trả về JSON hợp lệ với fields: `signal`, `confidence`, `reasoning`, `strategy_scores`, `market_analysis`, `risk_assessment`.

Rust struct `AnalysisResponse` expect:
```rust
pub struct AnalysisResponse {
    pub symbol: String,
    pub timeframe: String,
    pub timestamp: i64,
    pub signal: TradingSignal,
    pub confidence: f64,
    ...
}
```

Python trả về thiếu fields `symbol`, `timeframe`, `timestamp` hoặc format `signal` không khớp với `TradingSignal` enum.

**Hệ quả**: Multi-timeframe analysis hoàn toàn fail → paper trading engine chạy với partial data → signal quality giảm.

---

### [BUG-003] MEDIUM: 4H candle data không có từ Rust API

**Service**: python-ai-service
**Triệu chứng**:
```
WARNING: Insufficient 4H data for BTCUSDT: 0 candles (need 50+)
WARNING: Insufficient 4H data for ETHUSDT: 0 candles (need 50+)
WARNING: Insufficient 4H data for BNBUSDT: 0 candles (need 50+)
WARNING: Insufficient 4H data for SOLUSDT: 0 candles (need 50+)
```

Test xác nhận: `GET /api/market/candles/BTCUSDT/4h?limit=60` → trả về `data: []`

Tuy nhiên MongoDB có: BTCUSDT/4h: 500 candles, BTCUSDT/1d: 500 candles.

**Root cause**: Rust API in-memory cache (`market_data::cache`) không có 4H/1D candles. Cache chỉ có 1m/5m/15m/1h (500 mỗi loại). 4H/1D không được preload vào cache hoặc không được subscribe từ Binance WebSocket.

**Hệ quả**: GPT-4 weighted confidence bị hạ xuống 0.40 thay vì dùng 4H context. AI phân tích thiếu multi-timeframe view → tất cả signals bị đánh giá "Neutral" với confidence thấp.

---

### [BUG-004] MEDIUM: Rust Binance API 401 Unauthorized

**Service**: rust-core-engine
**Tần suất**: 8 lần

```
ERROR binance_trading_bot::binance::client: Request failed with status 401 Unauthorized:
  {"code":-2015,"msg":"Invalid API-key, IP, or permissions for action"}
WARN: Failed to set margin type for BNBUSDT: API request failed: 401 Unauthorized
WARN: Failed to set leverage for SOLUSDT: API request failed: 401 Unauthorized
```

**Thời điểm**: 2026-02-26T04:45:17 (tức là xảy ra lúc restart trước)

**Root cause**: `BINANCE_TESTNET=false` nhưng API key là testnet key (hoặc key thiếu permission futures). Khi engine startup cố set margin type/leverage cho futures.

**Hệ quả**: Real trading setup fail. Nhưng `TRADING_ENABLED=false` nên không trade thật, chỉ là paper trading mode.

---

### [BUG-005] LOW: Openclaw "TypeError: fetch failed" định kỳ

**Service**: openclaw
**Pattern**: Xuất hiện theo clusters, ~10 lần mỗi batch, cách nhau 10-30 phút

```
2026-02-27T16:40:51.893+07:00 [openclaw] Non-fatal unhandled rejection: TypeError: fetch failed
2026-02-27T16:41:02.480+07:00 [openclaw] Non-fatal unhandled rejection: TypeError: fetch failed
...5 lần liên tiếp cách nhau 10s...
```

**Root cause**: Openclaw đang polling hoặc gọi một external URL bị timeout/unreachable. Có thể là Claude.ai API, Telegram API, hoặc MCP server endpoint. Marked "Non-fatal" nên app vẫn chạy tiếp.

**Hệ quả**: Một số cron tasks có thể bị miss nếu fetch fail đúng lúc cron trigger. Cần xác định URL nào đang fail.

---

### [BUG-006] LOW: Rust WebSocket Connection Resets

**Service**: rust-core-engine
**Count**: 67 × "WebSocket error: IO error: Connection reset by peer"

```
ERROR warp::server::run: server connection error: hyper::Error(Parse(Method))  -- 26 lần
ERROR warp::server::run: server connection error: hyper::Error(IncompleteMessage) -- 11 lần
ERROR binance_trading_bot::api: WebSocket error: IO error: Connection reset by peer -- 67 lần
```

**Root cause**:
- `Parse(Method)`: Client (openclaw/health checker) gửi non-HTTP request (WebSocket upgrade không đúng format, hoặc raw TCP probe)
- `Connection reset by peer`: Binance WebSocket tự disconnect (bình thường, cần reconnect)
- `IncompleteMessage`: Client ngắt kết nối trước khi gửi xong request

**Hệ quả**: Thấp — Rust tự reconnect. Nhưng 67 lần reset là hơi nhiều.

---

### [NOTICE-001] Paper Trading Performance bất thường

**Data**:
```
win_rate: 46.34% (dưới 50%)
total_trades: 82
sharpe_ratio: 0.206 (thấp, target > 1.0)
total_pnl: +$360.95 (+3.61%)
profit_factor: 1.91 (tốt)
consecutive_loss cooldown: đã trigger 5 lần (3,5,6 consecutive losses)
```

**Nhận xét**: Win rate 46% với Sharpe 0.2 cho thấy hệ thống đang dùng AI signals kém (do BUG-001, BUG-002, BUG-003) → signals mostly Neutral → trades theo wrong direction nhiều.

---

## 4. HEALTH ENDPOINTS

| Endpoint | Status | Ghi chú |
|----------|--------|---------|
| GET /health (python-ai) | 200 OK | GPT-4 available, MongoDB connected |
| GET /ai/health/market-condition | 200 OK | candles_fetched: 50, pipeline ok |
| GET /api/health (rust) | 200 OK | "Bot is running" |
| GET /health (mcp) | 200 OK | service ok |
| GET / (frontend) | 200 OK | |
| POST /ai/market-condition | 200 OK | Trả về analysis đúng |
| POST /ai/strategy-recommendations | 422 Error | Missing required fields `timeframe_data`, `current_price` |
| POST /predict-trend | 500 Error | KeyError 'close' (BUG-001) |
| POST /ai/analyze | 200 OK | Trả về đúng, nhưng Rust không parse được |

---

## 5. MONGODB STATUS

- **Database**: `bot_core` (không phải `botcore` hay `bot-core`)
- **Connections**: current=19, active=4, available=838841
- **Collections**: 16 collections

**Candle data** (đủ):
```
BTCUSDT/1m: 670 | BTCUSDT/5m: 670 | BTCUSDT/15m: 670 | BTCUSDT/1h: 670
BTCUSDT/4h: 500 | BTCUSDT/1d: 500
ETHUSDT: tương tự | BNBUSDT: tương tự | SOLUSDT: tương tự
Total market_data: 18,720 documents
```

**Vấn đề**: MongoDB có đủ 4H/1D data nhưng Rust API cache không expose ra (BUG-003).

---

## 6. OPENCLAW CRON JOBS

10 cron jobs đăng ký thành công (0 failed):

| Job | Schedule | Ghi chú |
|-----|----------|---------|
| health-check | */30 * * * * | Mỗi 30 phút |
| market-regime | 0 */4 * * * | Mỗi 4 giờ |
| trade-guardian | 0 * * * * | Mỗi giờ |
| trade-manager | */30 * * * * | Mỗi 30 phút |
| morning-briefing | 0 2 * * 1-5 | 9AM GMT+7, thứ 2-6 |
| self-tuning | 0 2,8,16 * * * | 3 lần/ngày |
| daily-portfolio | 0 13 * * * | 8PM GMT+7 |
| hourly-pnl | 0 */6 * * * | Mỗi 6 giờ |
| loss-analysis | 0 */2 * * * | Mỗi 2 giờ |
| weekly-review | 0 3 * * 1 | Thứ 2 10AM GMT+7 |

Tất cả đều `no-deliver` → chỉ execute, không gửi Telegram trực tiếp.

---

## 7. MCP SERVER ANOMALY

MCP server tạo **65 sessions total** (21 trong 1 giờ qua). Mỗi cron job của openclaw tạo 1 session mới per request. Pattern này bình thường theo thiết kế per-session server.

---

## 8. REDIS STATUS

- PING: PONG (healthy)
- RAM: 1MB/256MB (0.4%)
- Keyspace: **empty** — không có key nào

**Điểm lạ**: Redis trống rỗng. Nếu Redis được dùng làm cache cho candle data hoặc session, thì cache miss sẽ xảy ra 100%. Cần kiểm tra xem service nào dùng Redis và có đang hoạt động đúng không.

---

## 9. ĐÁNH GIÁ NGUY CƠ

| Nguy cơ | Mức độ | Khả năng xảy ra |
|---------|--------|----------------|
| AI predict-trend tiếp tục fail 100% | HIGH | Đang xảy ra |
| Rust analyzer không nhận AI signal → trade blindly | HIGH | Đang xảy ra |
| 4H/1D candles không có trong cache → AI context thiếu | MEDIUM | Đang xảy ra |
| Openclaw fetch failed escalate → miss critical crons | MEDIUM | 20% |
| openclaw RAM tăng thêm → OOM | LOW-MEDIUM | 15% |
| MongoDB connection exhaust | LOW | <5% |

---

## 10. KHUYẾN NGHỊ

### Ưu tiên NGAY (P0)

**1. Fix BUG-001: predict-trend KeyError 'close'**
```python
# python-ai-service/main.py - trong predict_trend endpoint
# Sau khi fetch candles từ MongoDB, normalize field names:
for tf, candles in candles_by_tf.items():
    for candle in candles:
        if "close_price" in candle:
            candle["close"] = candle.pop("close_price")
            candle["open"] = candle.pop("open_price", candle.get("open", 0))
            candle["high"] = candle.pop("high_price", candle.get("high", 0))
            candle["low"] = candle.pop("low_price", candle.get("low", 0))
```

**2. Fix BUG-002: Rust AnalysisResponse schema mismatch**

Kiểm tra Rust struct `AnalysisResponse` vs Python response JSON. Python trả về:
```json
{"signal":"Neutral","confidence":0.5,"reasoning":"...","strategy_scores":{...},"market_analysis":{...},"risk_assessment":{...}}
```
Nhưng Rust expect: `symbol`, `timeframe`, `timestamp`, `signal` (TradingSignal enum), `confidence`. Cần thêm fields vào Python response hoặc dùng `#[serde(default)]` trong Rust struct.

### Ưu tiên CAO (P1)

**3. Fix BUG-003: Expose 4H/1D candles qua Rust API**

Kiểm tra tại sao cache không load 4H/1D. Có thể cần thêm subscription cho 4H/1D WebSocket streams hoặc backfill từ MongoDB khi startup.

**4. Fix BUG-004: Binance API Key permissions**

Nếu cần enable real trading, API key phải có futures permissions. Hiện tại `TRADING_ENABLED=false` nên không urgent nhưng cần resolve trước khi go live.

### Ưu tiên TRUNG BÌNH (P2)

**5. Điều tra Openclaw "fetch failed"**

Cần xem URL nào đang fail. Thêm logging chi tiết hơn hoặc kiểm tra network connectivity từ openclaw container.

**6. Kiểm tra Redis usage**

Redis hoàn toàn trống. Xác nhận service nào dùng Redis và liệu empty cache có gây vấn đề gì.

**7. Tăng logging cho predict-trend**

Thêm `logger.exception()` thay vì `logger.error(f"... {e}")` để có full traceback.

### Monitoring

Thêm alert cho:
- `/predict-trend` HTTP 5xx rate > 0
- Rust analyzer WARN count > 10/phút
- openclaw RAM > 600MB
- Paper trading win rate < 40% trong 24h

---

## 11. CÂU HỎI CÒN TỒN ĐỌNG

1. Tại sao python-ai-service, mcp-server, openclaw đều restart ~6 giờ trước cùng lúc? OOM? Manual? Có log trước restart không?

2. Openclaw "fetch failed" đang gọi URL nào? Telegram API? Claude.ai? MCP? — cần thêm stack trace đầy đủ.

3. Redis hoàn toàn trống — đây có phải bình thường không? Service nào có dependency vào Redis?

4. MCP server tạo 65 sessions trong vài giờ — memory leak tiềm ẩn không? Mỗi session có được cleanup không?

5. Rust WebSocket "Connection reset by peer" 67 lần — từ Binance hay từ internal clients? Reconnect policy có đủ mạnh không?

---

*Báo cáo tạo lúc: 2026-02-27 20:30 GMT+7*
*Inspector: Claude Sonnet 4.6 (debugger agent)*
