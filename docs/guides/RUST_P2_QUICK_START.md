# Rust P2 Improvements - Quick Start Guide

## What Changed?

### ✅ P2-1: Price Validation
- **Impact:** Prevents trading on invalid price data (0.0 could cause losses)
- **Location:** `src/trading/engine.rs`, `src/utils.rs`
- **Action Required:** None - automatic validation

### ✅ P2-5: CORS Security
- **Impact:** Only allows requests from approved origins
- **Location:** `src/api/mod.rs`, `src/api/paper_trading.rs`
- **Action Required:** Set environment variable in production

### ✅ P2-3: Circuit Breaker Monitoring
- **Impact:** New API endpoints to monitor and reset circuit breaker
- **Location:** `src/api/paper_trading.rs`, `src/paper_trading/engine.rs`
- **Action Required:** Optional - integrate with dashboard

---

## Quick Test Commands

### 1. Build and Run
```bash
cd /Users/dungngo97/Documents/bot-core/rust-core-engine
cargo build --lib
cargo run
```

### 2. Test Circuit Breaker Status (NEW)
```bash
curl http://localhost:8080/api/paper-trading/circuit-breaker/status | jq
```

**Expected Response:**
```json
{
  "success": true,
  "data": {
    "is_tripped": false,
    "trip_reason": null,
    "daily_loss": 0.0,
    "daily_loss_pct": 0.0,
    "daily_loss_limit_pct": 5.0,
    "current_drawdown_pct": 0.0,
    "max_drawdown_pct": 15.0,
    "current_equity": 10000.0,
    "peak_equity": 10000.0,
    "last_reset": "2025-11-19T..."
  }
}
```

### 3. Test Circuit Breaker Reset (NEW)
```bash
curl -X POST http://localhost:8080/api/paper-trading/circuit-breaker/reset | jq
```

**Expected Response:**
```json
{
  "success": true,
  "data": {
    "message": "Circuit breaker reset successfully",
    "warning": "Trading risk limits have been reset. Monitor carefully."
  }
}
```

### 4. Test CORS (Development)
```bash
# Should work - localhost allowed by default
curl -H "Origin: http://localhost:3000" \
     -H "Access-Control-Request-Method: POST" \
     -X OPTIONS \
     http://localhost:8080/api/health
```

### 5. Verify Health
```bash
curl http://localhost:8080/api/health | jq
```

---

## Environment Configuration

### Development (Default)
```bash
# No environment variable needed
# Defaults: http://localhost:3000, http://localhost:5173
```

### Production
```bash
# Add to .env or docker-compose.yml
export CORS_ALLOWED_ORIGINS="https://dashboard.yourdomain.com,https://api.yourdomain.com"
```

### Staging
```bash
export CORS_ALLOWED_ORIGINS="https://staging-dashboard.yourdomain.com"
```

---

## Dashboard Integration

### Add to Dashboard API Client

**TypeScript Example:**
```typescript
// services/apiClient.ts

export async function getCircuitBreakerStatus() {
  const response = await fetch('/api/paper-trading/circuit-breaker/status');
  return response.json();
}

export async function resetCircuitBreaker() {
  const response = await fetch('/api/paper-trading/circuit-breaker/reset', {
    method: 'POST',
  });
  return response.json();
}
```

### Dashboard Component Example

**React Component:**
```tsx
// components/CircuitBreakerWidget.tsx

import { useEffect, useState } from 'react';
import { getCircuitBreakerStatus, resetCircuitBreaker } from '@/services/apiClient';

export function CircuitBreakerWidget() {
  const [status, setStatus] = useState(null);
  const [loading, setLoading] = useState(false);

  useEffect(() => {
    const fetchStatus = async () => {
      const data = await getCircuitBreakerStatus();
      setStatus(data.data);
    };

    fetchStatus();
    const interval = setInterval(fetchStatus, 5000); // Refresh every 5s

    return () => clearInterval(interval);
  }, []);

  const handleReset = async () => {
    if (confirm('Are you sure you want to reset the circuit breaker?')) {
      setLoading(true);
      await resetCircuitBreaker();
      const data = await getCircuitBreakerStatus();
      setStatus(data.data);
      setLoading(false);
    }
  };

  if (!status) return <div>Loading...</div>;

  return (
    <div className="circuit-breaker-widget">
      <h3>Circuit Breaker Status</h3>
      
      <div className={status.is_tripped ? 'alert-danger' : 'alert-success'}>
        {status.is_tripped ? '🚨 TRIPPED' : '✅ NORMAL'}
      </div>

      <div className="metrics">
        <div>
          <span>Daily Loss:</span>
          <span>{status.daily_loss_pct.toFixed(2)}% / {status.daily_loss_limit_pct}%</span>
        </div>
        <div>
          <span>Drawdown:</span>
          <span>{status.current_drawdown_pct.toFixed(2)}% / {status.max_drawdown_pct}%</span>
        </div>
        <div>
          <span>Equity:</span>
          <span>${status.current_equity.toFixed(2)}</span>
        </div>
      </div>

      {status.is_tripped && (
        <div>
          <p className="trip-reason">{status.trip_reason}</p>
          <button onClick={handleReset} disabled={loading}>
            Reset Circuit Breaker
          </button>
        </div>
      )}
    </div>
  );
}
```

---

## Troubleshooting

### CORS Issues

**Problem:** Dashboard can't connect to API

**Solution:**
```bash
# Check current CORS setting
echo $CORS_ALLOWED_ORIGINS

# Set for development
export CORS_ALLOWED_ORIGINS="http://localhost:3000,http://localhost:5173"

# Restart Rust service
cargo run
```

### Price Validation Errors

**Problem:** Seeing "Invalid price" errors in logs

**What it means:** Exchange API returned invalid data (good catch!)

**Action:** Check exchange status, verify data quality

**Example Log:**
```
WARN Invalid entry price for BTCUSDT: Invalid entry_price: abc - skipping position
```

### Circuit Breaker Won't Reset

**Problem:** Reset endpoint returns error

**Solution:** Check if circuit breaker is actually tripped first:
```bash
curl http://localhost:8080/api/paper-trading/circuit-breaker/status | jq '.data.is_tripped'
```

---

## Build Verification

### Check Build Status
```bash
cargo build --lib 2>&1 | grep -E "(Finished|warning|error)"
```

**Expected:**
```
warning: value assigned to `reconnect_attempts` is never read
warning: value assigned to `last_successful_connect` is never read
warning: `binance-trading-bot` (lib) generated 2 warnings
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 2.82s
```

**Note:** 2 warnings are unrelated to P2 changes (in WebSocket module)

### Run Tests
```bash
cargo test --lib 2>&1 | tail -20
```

---

## Monitoring Checklist

### Development
- [ ] Test circuit breaker status endpoint
- [ ] Test circuit breaker reset endpoint
- [ ] Verify CORS allows localhost:3000
- [ ] Check logs for price validation warnings
- [ ] Verify dashboard can connect

### Staging
- [ ] Set CORS_ALLOWED_ORIGINS environment variable
- [ ] Test from staging dashboard
- [ ] Verify CORS blocks unauthorized origins
- [ ] Monitor circuit breaker status
- [ ] Test manual reset flow

### Production
- [ ] Set production CORS origins
- [ ] Add authentication to reset endpoint
- [ ] Set up monitoring alerts
- [ ] Configure log aggregation
- [ ] Test disaster recovery

---

## Next Steps

1. **Test Locally** (5 min)
   ```bash
   cargo run
   curl http://localhost:8080/api/paper-trading/circuit-breaker/status
   ```

2. **Update Dashboard** (30 min)
   - Add CircuitBreakerWidget component
   - Test integration

3. **Deploy to Staging** (1 hour)
   - Set CORS_ALLOWED_ORIGINS
   - Deploy and test

4. **Production Deployment** (1-2 hours)
   - Review security settings
   - Set production CORS
   - Add auth to reset endpoint
   - Deploy and monitor

---

## Support

### Documentation
- Full Report: `/Users/dungngo97/Documents/bot-core/RUST_P2_COMPLETION_REPORT.md`
- Metrics Design: `rust-core-engine/docs/P2_MONITORING_METRICS_DESIGN.md`
- Code: `rust-core-engine/src/`

### Key Files
- Price Validation: `src/utils.rs`
- CORS Config: `src/api/mod.rs`
- Circuit Breaker: `src/api/paper_trading.rs`

### Questions?
Review the full completion report for detailed documentation and examples.

---

**Last Updated:** 2025-11-19
**Status:** ✅ READY FOR TESTING
