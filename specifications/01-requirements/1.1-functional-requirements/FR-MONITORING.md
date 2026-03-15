# FR-MONITORING: System Monitoring

**Spec ID**: FR-MONITORING
**Version**: 1.0
**Status**: Implemented
**Owner**: System
**Last Updated**: 2026-03-03

---

## Tasks Checklist

- [x] System metrics collection (CPU, memory, uptime, cache)
- [x] Trading metrics collection (PnL, win rate, drawdown, Sharpe)
- [x] Connection status tracking (WebSocket, API, reconnect count)
- [x] REST API endpoints for monitoring data (`/api/monitoring/*`)
- [x] MCP tool exposure (7 tools: 4 in `monitoring.ts` + 3 in `health.ts`)
- [x] Health check endpoints for all services
- [x] AI pipeline deep health check
- [x] Sysinfo integration for real CPU/memory readings
- [ ] Prometheus metrics export endpoint
- [ ] Alert thresholds with automatic notification
- [ ] Tool-level audit logging

---

## Metadata

**Related Specs**:
- `FR-MCP.md` — MCP tool exposure
- `FR-AUTH.md` — API authentication

**Dependencies**:
- `sysinfo` crate — OS-level CPU/memory readings
- `chrono` crate — timestamp handling
- MongoDB via Storage — PerformanceStats source

**Business Value**: High (operational visibility)
**Technical Complexity**: Low
**Priority**: High

---

## Overview

`MonitoringService` (Rust) collects and exposes system, trading, and connection metrics. Metrics are served via three REST endpoints on the Rust Core Engine at `:8080`. The MCP Server exposes these as 7 tools for AI-driven monitoring via OpenClaw/Claude (4 in `monitoring.ts`, 3 in `health.ts`).

---

## Functional Requirements

### FR-MONITORING-001: System Metrics

**Priority**: High
**Status**: Completed
**Code**: `rust-core-engine/src/monitoring/mod.rs:SystemMetrics`

**Description**: Collect and expose real-time system-level performance data.

**Metrics collected**:

| Metric | Type | Source | Notes |
|---|---|---|---|
| `uptime_seconds` | u64 | `Instant::elapsed()` | Service uptime |
| `active_positions` | usize | Engine state | Paper/real open trades |
| `total_trades` | u64 | Engine state | Cumulative trade count |
| `cache_size` | usize | Engine state | In-memory cache entries |
| `memory_usage_mb` | f64 | `sysinfo::System` | Stored as % of total |
| `cpu_usage_percent` | f64 | `sysinfo::System` | Average across all cores |
| `last_update` | i64 | `chrono::Utc` | Unix timestamp |

**API**: `GET /api/monitoring/system`
**MCP tool**: `get_system_monitoring`

**Acceptance Criteria**:
- [x] CPU = average of all core usages (`system.cpus()`)
- [x] Memory = `used_memory / total_memory * 100` (percentage)
- [x] Uptime calculated from service start (`start_time: Instant`)
- [x] Returned as JSON via warp route

---

### FR-MONITORING-002: Trading Metrics

**Priority**: High
**Status**: Completed
**Code**: `rust-core-engine/src/monitoring/mod.rs:TradingMetrics`

**Description**: Expose aggregated trading performance metrics updated from `PerformanceStats`.

**Metrics collected**:

| Metric | Type | Source |
|---|---|---|
| `total_pnl` | f64 | `PerformanceStats.total_pnl` |
| `win_rate` | f64 | `PerformanceStats.win_rate` |
| `avg_trade_duration_minutes` | f64 | Engine (placeholder) |
| `max_drawdown` | f64 | Engine (placeholder) |
| `sharpe_ratio` | Option<f64> | Engine (placeholder) |
| `total_volume` | f64 | Engine (placeholder) |

**API**: `GET /api/monitoring/trading`
**MCP tool**: `get_trading_metrics`

**Update path**: `ApiServer::update_monitoring()` → `monitor.update_trading_metrics(&stats)`

---

### FR-MONITORING-003: Connection Status

**Priority**: High
**Status**: Completed
**Code**: `rust-core-engine/src/monitoring/mod.rs:ConnectionStatus`

**Description**: Track liveness of all external service connections.

**Fields**:

| Field | Type | Meaning |
|---|---|---|
| `websocket_connected` | bool | Binance WebSocket live |
| `api_responsive` | bool | Binance REST API responding |
| `last_data_update` | i64 | Unix timestamp of last update |
| `reconnect_count` | u32 | Cumulative reconnect events |

**API**: `GET /api/monitoring/connection`
**MCP tool**: `get_connection_status`

**Reconnect tracking**: `record_reconnect()` increments counter with `saturating_add(1)` to prevent overflow.

---

### FR-MONITORING-004: Service Health Check

**Priority**: Critical
**Status**: Completed
**Code**: `mcp-server/src/tools/health.ts`

**Description**: Composite health check polling all services and returning combined status.

**Tool**: `check_system_health`

**Services checked**:

| Service | Endpoint | Port | Auth |
|---|---|---|---|
| Rust Core Engine | `GET /api/health` | 8080 | None (skipAuth) |


**Response shape**:
```json
{
  "overall_status": "healthy|degraded",
  "timestamp": "ISO8601",
  "services": {
    "rust_core_engine": { "status": "healthy|unhealthy|unreachable", "port": 8080 },
  }
}
```

---

### FR-MONITORING-005: AI Pipeline Deep Health

**Priority**: High
**Status**: Completed
**Code**: `mcp-server/src/tools/health.ts:check_market_condition_health`

**Description**: Deep health check testing the full AI pipeline — MongoDB candle fetch, indicator calculation, and model inference.

**Tool**: `check_market_condition_health`

**Timeout**: 15 seconds

**Response**:
```json
{ "healthy": true }
// or
{ "healthy": false, "error": "...", "action_required": "Stop paper engine..." }
```

**Used by**: `health-check` cron job (every 30 min) — auto-stops paper engine on failure.

---



**Priority**: Medium
**Status**: Completed
**Code**: `mcp-server/src/tools/monitoring.ts:get_python_health`



**Tool**: `get_python_health`

**Timeout**: 5 seconds

---

### FR-MONITORING-007: Service Logs Summary

**Priority**: Medium
**Status**: Completed
**Code**: `mcp-server/src/tools/health.ts:get_service_logs_summary`

**Description**: Aggregate recent error/warning logs from services.

**Tool**: `get_service_logs_summary`
**Input**: `{ service: "rust" | "python" | "all" }`
**Sources**:
- Rust: `GET /api/monitoring/system`


---

## Implementation Details

### Code Locations

| File | Purpose |
|---|---|
| `rust-core-engine/src/monitoring/mod.rs` | `MonitoringService` struct, all metrics types |
| `rust-core-engine/src/api/mod.rs` | REST routes (`monitoring_routes()`), `update_monitoring()` |
| `mcp-server/src/tools/monitoring.ts` | MCP tools: `get_system_monitoring`, `get_trading_metrics`, `get_connection_status`, `get_python_health` |
| `mcp-server/src/tools/health.ts` | MCP tools: `check_system_health`, `get_service_logs_summary`, `check_market_condition_health` |

### REST API Endpoints (Rust, port 8080)

| Method | Path | Auth | Description |
|---|---|---|---|
| GET | `/api/monitoring/system` | None | System metrics (CPU, memory, uptime) |
| GET | `/api/monitoring/trading` | None | Trading metrics (PnL, win rate) |
| GET | `/api/monitoring/connection` | None | Connection status |
| GET | `/api/health` | None | Basic health check |

### MCP Tools (7 total)

| Tool | Category | Endpoint |
|---|---|---|



| `get_system_monitoring` | monitoring | Rust `/api/monitoring/system` |
| `get_trading_metrics` | monitoring | Rust `/api/monitoring/trading` |
| `get_connection_status` | monitoring | Rust `/api/monitoring/connection` |


### Update Mechanism

```
PaperTradingEngine/TradingEngine
    ↓ (periodic)
ApiServer::update_monitoring(active_positions, cache_size, ws_connected, api_responsive)
    ↓
MonitoringService::update_system_metrics()    → reads sysinfo
MonitoringService::update_trading_metrics()   → reads PerformanceStats
MonitoringService::update_connection_status() → sets ws/api flags
```

---

## Dependencies

- `sysinfo` crate — CPU and memory readings
- `chrono` crate — timestamp generation
- `warp` — HTTP routing
- `tokio::sync::RwLock` — thread-safe metric access


---

## Test Cases

- `test_monitoring_system_route` — GET `/api/monitoring/system` returns 200
- `test_monitoring_trading_route` — GET `/api/monitoring/trading` returns 200
- `test_monitoring_connection_route` — GET `/api/monitoring/connection` returns 200
- `test_update_monitoring` — verify metric mutation via `update_monitoring()`
- `test_monitoring_routes_all` — all three routes in sequence
