# P2-4 & P2-6: Monitoring Metrics Design

## Overview

This document outlines the design for Rate Limiter Metrics (P2-4) and Retry Metrics Tracking (P2-6) that were identified in the P2 improvement plan but deferred due to time constraints.

## P2-4: Rate Limiter Metrics

### Objective
Export rate limiter metrics for monitoring and operational visibility.

### Design

#### New Data Structure
```rust
// Add to src/binance/rate_limiter.rs

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RateLimiterMetrics {
    pub available_permits: usize,
    pub total_permits: usize,
    pub utilization_percent: f64,
    pub requests_throttled: u64,
    pub avg_wait_time_ms: f64,
    pub timestamp: DateTime<Utc>,
}

impl RateLimiter {
    pub async fn get_metrics(&self) -> RateLimiterMetrics {
        let available = self.semaphore.available_permits();
        let total = self.permits_per_minute;
        let utilization = ((total - available) as f64 / total as f64) * 100.0;

        RateLimiterMetrics {
            available_permits: available,
            total_permits: total,
            utilization_percent: utilization,
            requests_throttled: self.throttled_count.load(Ordering::Relaxed),
            avg_wait_time_ms: self.avg_wait_time.load(Ordering::Relaxed),
            timestamp: Utc::now(),
        }
    }
}
```

#### API Endpoint
```
GET /api/metrics/rate-limiter
```

Response:
```json
{
  "success": true,
  "data": {
    "available_permits": 45,
    "total_permits": 60,
    "utilization_percent": 25.0,
    "requests_throttled": 3,
    "avg_wait_time_ms": 12.5,
    "timestamp": "2025-11-19T10:30:00Z"
  }
}
```

#### Implementation Location
- File: `src/binance/rate_limiter.rs`
- API handler: `src/api/mod.rs` (add to monitoring_routes)

### Benefits
- Real-time visibility into API rate limit usage
- Early warning if approaching limits
- Performance tuning data
- Incident investigation support

---

## P2-6: Retry Metrics Tracking

### Objective
Track and expose retry policy metrics for monitoring failures and system health.

### Design

#### New Data Structures
```rust
// Add to src/binance/retry.rs

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RetryMetrics {
    pub total_attempts: u64,
    pub successful_retries: u64,
    pub failed_retries: u64,
    pub avg_attempts_per_call: f64,
    pub retry_rate_percent: f64,
    pub last_failure: Option<DateTime<Utc>>,
    pub last_failure_reason: Option<String>,
}

pub struct RetryPolicy {
    // ... existing fields ...
    metrics: Arc<RwLock<RetryMetrics>>,
}

impl RetryPolicy {
    pub async fn get_metrics(&self) -> RetryMetrics {
        self.metrics.read().await.clone()
    }

    // Update metrics in retry_with_backoff method
    async fn update_metrics(&self, attempts: u32, success: bool, error: Option<&str>) {
        let mut metrics = self.metrics.write().await;
        metrics.total_attempts += attempts as u64;

        if success {
            metrics.successful_retries += 1;
        } else {
            metrics.failed_retries += 1;
            metrics.last_failure = Some(Utc::now());
            metrics.last_failure_reason = error.map(|e| e.to_string());
        }

        let total = metrics.successful_retries + metrics.failed_retries;
        if total > 0 {
            metrics.avg_attempts_per_call =
                metrics.total_attempts as f64 / total as f64;
            metrics.retry_rate_percent =
                (metrics.successful_retries as f64 / total as f64) * 100.0;
        }
    }
}
```

#### API Endpoint
```
GET /api/metrics/retry
```

Response:
```json
{
  "success": true,
  "data": {
    "total_attempts": 150,
    "successful_retries": 12,
    "failed_retries": 2,
    "avg_attempts_per_call": 1.8,
    "retry_rate_percent": 85.7,
    "last_failure": "2025-11-19T09:45:00Z",
    "last_failure_reason": "Connection timeout"
  }
}
```

#### Implementation Location
- File: `src/binance/retry.rs`
- API handler: `src/api/mod.rs` (add to monitoring_routes)

### Benefits
- Identify problematic API calls requiring retries
- Measure system reliability
- Alert on retry rate spikes
- Debug intermittent failures

---

## Implementation Priority

**Status:** Deferred to P3 (Future Enhancement)

**Reason:** Core P2 improvements (price validation, CORS, circuit breaker) take priority.

**Estimated Effort:**
- P2-4: 2-3 hours (metrics struct + endpoint + tests)
- P2-6: 2-3 hours (metrics struct + tracking + endpoint + tests)
- Total: 4-6 hours

## Future Work

When implementing:
1. Add metrics structs to respective modules
2. Implement atomic counters for thread-safe tracking
3. Add API endpoints to monitoring routes
4. Write unit tests for metrics collection
5. Add integration tests for endpoints
6. Document in API documentation
7. Add Prometheus/Grafana export support (optional)

## References

- P2 Improvement Plan
- `src/binance/rate_limiter.rs` - Current rate limiter implementation
- `src/binance/retry.rs` - Current retry policy implementation
- `src/api/mod.rs` - API routes definition
