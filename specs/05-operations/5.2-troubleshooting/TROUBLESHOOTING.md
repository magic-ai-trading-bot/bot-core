# Troubleshooting Guide

**Document Version:** 1.0.0
**Last Updated:** 2025-10-11
**Status:** Active
**Owner:** Operations Team

---

## Table of Contents

- [1. Overview](#1-overview)
- [2. Service Issues](#2-service-issues)
- [3. Performance Issues](#3-performance-issues)
- [4. Database Issues](#4-database-issues)
- [5. Network Issues](#5-network-issues)
- [6. Deployment Issues](#6-deployment-issues)
- [7. Trading Issues](#7-trading-issues)
- [8. Diagnostic Commands](#8-diagnostic-commands)

---

## 1. Overview

### 1.1 Purpose

This guide provides systematic troubleshooting procedures for common issues in the Bot Core platform.

### 1.2 General Troubleshooting Approach

1. **Identify** - Determine what's failing
2. **Isolate** - Narrow down the root cause
3. **Diagnose** - Understand why it's failing
4. **Resolve** - Apply fix
5. **Verify** - Confirm resolution
6. **Document** - Record for future reference

---

## 2. Service Issues

### 2.1 Service Won't Start

**Symptoms:**
- Container exits immediately with code 1
- Service shows as "Restarting" in `docker compose ps`
- Health checks never pass

**Diagnosis:**
```bash
# Check logs
docker compose logs rust-core-engine

# Check exit code
docker inspect rust-core-engine --format='{{.State.ExitCode}}'

# Check environment variables
docker inspect rust-core-engine --format='{{range .Config.Env}}{{println .}}{{end}}'

# Check configuration file
docker exec rust-core-engine cat /app/config.toml
```

**Common Causes & Solutions:**

**Missing Environment Variables:**
```bash
# Check required variables
./scripts/validate-env.sh

# Verify .env file exists
ls -la .env

# Check for missing secrets
grep "not set" docker-compose.yml | while read line; do
  echo "Checking: $line"
done

# Fix: Add missing variables to .env
vi .env
# Add missing values
# BINANCE_API_KEY=your_key_here
# DATABASE_URL=mongodb://...
```

**Port Conflicts:**
```bash
# Check if port is in use
lsof -i :8080
netstat -tulpn | grep :8080

# Fix: Kill process using port
kill -9 $(lsof -t -i:8080)

# Or change port in docker-compose.yml
ports:
  - "8081:8080"  # Map host 8081 to container 8080
```

**Configuration Errors:**
```bash
# Validate TOML syntax
cat rust-core-engine/config.toml | python -c "import sys, toml; toml.load(sys.stdin)"

# Validate YAML syntax
cat python-ai-service/config.yaml | python -c "import sys, yaml; yaml.safe_load(sys.stdin)"

# Fix: Correct syntax errors in config files
```

**Dependency Issues:**
```bash
# Check if dependency services are running
docker compose ps mongodb redis

# Start dependencies first
docker compose up -d mongodb redis
sleep 10
docker compose up -d rust-core-engine
```

### 2.2 Service Crashes/Restarts Frequently

**Symptoms:**
- Container restarts every few minutes
- `docker compose ps` shows restart count increasing
- OOMKilled status

**Diagnosis:**
```bash
# Check restart count
docker inspect rust-core-engine --format='{{.RestartCount}}'

# Check if OOM killed
docker inspect rust-core-engine --format='{{.State.OOMKilled}}'

# Check memory usage
docker stats rust-core-engine --no-stream

# Check logs for panic/crash
docker compose logs rust-core-engine | grep -i "panic\|error\|fatal"
```

**Common Causes & Solutions:**

**Out of Memory (OOM):**
```bash
# Check current memory limit
docker inspect rust-core-engine --format='{{.HostConfig.Memory}}'

# Fix: Increase memory limit
# Edit docker-compose.yml or use environment variable
export RUST_MEMORY_LIMIT="2G"
docker compose up -d rust-core-engine

# Or use memory-optimized mode
./scripts/bot.sh start --memory-optimized

# Long-term fix: Optimize application code
# - Fix memory leaks
# - Reduce cache size
# - Implement streaming for large datasets
```

**Panic/Unhandled Exception:**
```bash
# Get full panic trace
docker compose logs rust-core-engine | grep -A 50 "panic"

# Fix: Based on error message
# Common issues:
# - Null pointer dereference: Add null checks
# - Index out of bounds: Add bounds checking
# - Database connection lost: Implement retry logic
```

**Deadlock:**
```bash
# Check for SIGABRT (deadlock signal)
docker compose logs rust-core-engine | grep "SIGABRT"

# Get thread dump (if service supports it)
docker exec rust-core-engine kill -SIGUSR1 1

# Fix: Review code for:
# - Circular lock dependencies
# - Lock not released
# - Async/await issues
```

### 2.3 Service Responding Slowly

**Symptoms:**
- API requests taking >5 seconds
- Timeouts in client applications
- High CPU usage

**Diagnosis:**
```bash
# Check response times
time curl http://localhost:8080/api/health

# Check CPU usage
docker stats rust-core-engine --no-stream

# Check for slow queries
docker compose logs rust-core-engine | grep "duration_ms" | awk '{if ($NF > 1000) print}'

# Profile the application
# For Rust: Use flamegraph
# For Python: Use py-spy
docker exec -it python-ai-service py-spy top --pid 1
```

**Common Causes & Solutions:**

**High CPU Usage:**
```bash
# Check what's consuming CPU
docker exec rust-core-engine ps aux --sort=-pcpu | head -10

# Fix: Optimize hot paths
# - Add caching
# - Reduce computation in request path
# - Offload to background jobs
```

**Slow Database Queries:**
```bash
# Enable slow query logging
docker exec mongodb-primary mongosh
> db.setProfilingLevel(1, { slowms: 100 })

# Check slow queries
> db.system.profile.find().sort({ ts: -1 }).limit(10).pretty()

# Fix: Add indexes
> db.trades.createIndex({ user_id: 1, timestamp: -1 })
```

**Too Many Concurrent Requests:**
```bash
# Check connection pool
curl http://localhost:8080/metrics | grep connection_pool

# Fix: Scale horizontally
kubectl scale deployment rust-core-engine --replicas=5 -n bot-core-production

# Or increase connection pool size
# Edit config.toml
[database]
max_connections = 100  # Increase from 10
```

### 2.4 Health Check Failing

**Symptoms:**
- Container marked as unhealthy
- Service removed from load balancer
- No traffic reaching service

**Diagnosis:**
```bash
# Check health check config
docker inspect rust-core-engine --format='{{json .State.Health}}' | jq

# Test health endpoint manually
curl -v http://localhost:8080/api/health

# Check logs during health check
docker compose logs -f rust-core-engine
# In another terminal
curl http://localhost:8080/api/health
```

**Common Causes & Solutions:**

**Health Endpoint Not Responding:**
```bash
# Check if service is listening on correct port
docker exec rust-core-engine netstat -tulpn | grep 8080

# Fix: Ensure health endpoint is implemented
# rust-core-engine/src/main.rs
async fn health_check() -> impl Responder {
    HttpResponse::Ok().json(json!({
        "status": "healthy",
        "timestamp": Utc::now().to_rfc3339()
    }))
}
```

**Health Check Timeout Too Short:**
```bash
# Increase timeout in docker-compose.yml
healthcheck:
  test: ["CMD", "curl", "-f", "http://localhost:8080/api/health"]
  interval: 30s
  timeout: 10s  # Increase from 5s
  retries: 3
  start_period: 60s  # Increase grace period
```

---

## 3. Performance Issues

### 3.1 High Latency

**Symptoms:**
- API p95 latency > 500ms
- User complaints about slow response
- Increased timeout errors

**Diagnosis:**
```bash
# Check latency metrics
curl http://localhost:9090/api/v1/query?query='histogram_quantile(0.95, rate(http_request_duration_seconds_bucket[5m]))'

# Check for slow endpoints
docker compose logs rust-core-engine | grep "duration_ms" | sort -t: -k4 -n | tail -20

# Profile application
# Rust: Use cargo flamegraph
cd rust-core-engine
cargo flamegraph --bin rust-core-engine

# Python: Use py-spy
docker exec python-ai-service py-spy record -o profile.svg --pid 1 --duration 60
```

**Solutions:**

**Add Caching:**
```rust
// Rust example - Add caching
use cached::proc_macro::cached;

#[cached(time = 60, size = 1000)]
async fn get_market_data(symbol: String) -> Result<MarketData, Error> {
    // Expensive operation
    fetch_from_api(symbol).await
}
```

```python
# Python example - Add caching
from functools import lru_cache

@lru_cache(maxsize=1000)
def get_technical_indicators(symbol: str, timeframe: str):
    # Expensive calculation
    return calculate_indicators(symbol, timeframe)
```

**Database Query Optimization:**
```bash
# Add index for common queries
docker exec mongodb-primary mongosh
> use trading_bot
> db.trades.createIndex({ user_id: 1, timestamp: -1 })
> db.market_data.createIndex({ symbol: 1, timestamp: -1 })
```

**Connection Pooling:**
```toml
# rust-core-engine/config.toml
[database]
max_connections = 50  # Increase pool size
min_connections = 10
```

### 3.2 High Memory Usage

**Symptoms:**
- Memory usage > 90%
- Swapping to disk
- OOM kills

**Diagnosis:**
```bash
# Check memory usage
docker stats --no-stream

# Check memory by service
docker stats --format "table {{.Name}}\t{{.MemUsage}}\t{{.MemPerc}}"

# Profile memory usage
# Rust: Use valgrind or heaptrack
# Python: Use memory_profiler
docker exec python-ai-service python -m memory_profiler main.py
```

**Solutions:**

**Increase Memory Limit:**
```bash
# Temporary fix
export PYTHON_MEMORY_LIMIT="3G"
docker compose up -d python-ai-service
```

**Fix Memory Leaks:**
```python
# Python example - Explicit cleanup
def process_large_dataset():
    data = load_large_file()
    result = process(data)
    del data  # Explicit cleanup
    gc.collect()
    return result
```

**Reduce Cache Size:**
```yaml
# python-ai-service/config.yaml
ai_cache:
  enabled: true
  max_entries: 50  # Reduce from 100
```

### 3.3 High CPU Usage

**Symptoms:**
- CPU usage consistently > 80%
- Slow response times
- High load average

**Diagnosis:**
```bash
# Check CPU usage
docker stats --no-stream

# Check system load
uptime

# Profile CPU usage
docker exec rust-core-engine ps aux --sort=-pcpu | head -10
```

**Solutions:**

**Optimize Hot Paths:**
```rust
// Use more efficient algorithms
// Before: O(n²)
for i in 0..data.len() {
    for j in 0..data.len() {
        // ...
    }
}

// After: O(n)
use std::collections::HashMap;
let map: HashMap<_, _> = data.iter().map(|x| (x.id, x)).collect();
```

**Offload to Background Workers:**
```python
# Use Celery or RabbitMQ for async tasks
@celery.task
def expensive_analysis(data):
    return perform_analysis(data)

# In API endpoint
result = expensive_analysis.delay(data)
return {"task_id": result.id}
```

---

## 4. Database Issues

### 4.1 Connection Errors

**Symptoms:**
- "Connection refused" errors
- "Too many connections" errors
- Unable to reach database

**Diagnosis:**
```bash
# Check if MongoDB is running
docker compose ps mongodb

# Check connection from service
docker exec rust-core-engine curl mongodb:27017

# Check connection pool
docker compose logs rust-core-engine | grep "connection pool"

# Check MongoDB logs
docker compose logs mongodb | grep -i "error\|warn"
```

**Solutions:**

**MongoDB Not Running:**
```bash
# Start MongoDB
docker compose up -d mongodb

# Check health
docker exec mongodb-primary mongosh --eval "db.adminCommand('ping')"
```

**Connection String Incorrect:**
```bash
# Verify connection string
echo $DATABASE_URL

# Test connection manually
mongosh "$DATABASE_URL"

# Fix in .env
DATABASE_URL=mongodb://botuser:PASSWORD@mongodb:27017/trading_bot?authSource=admin
```

**Too Many Connections:**
```bash
# Check current connections
docker exec mongodb-primary mongosh --eval "db.serverStatus().connections"

# Close idle connections
docker exec mongodb-primary mongosh --eval "db.adminCommand({closeAllDatabases: true})"

# Increase max connections
# Edit MongoDB config
# maxIncomingConnections: 200
```

### 4.2 Slow Queries

**Symptoms:**
- Database queries taking >1 second
- High database CPU usage
- Application timeouts

**Diagnosis:**
```bash
# Enable profiling
docker exec mongodb-primary mongosh
> use trading_bot
> db.setProfilingLevel(1, { slowms: 100 })

# Check slow queries
> db.system.profile.find().sort({ ts: -1 }).limit(10).pretty()

# Explain query plan
> db.trades.find({ user_id: "xyz" }).explain("executionStats")
```

**Solutions:**

**Add Missing Indexes:**
```javascript
// Common indexes for trading bot
db.trades.createIndex({ user_id: 1, timestamp: -1 })
db.market_data.createIndex({ symbol: 1, timestamp: -1 })
db.users.createIndex({ email: 1 }, { unique: true })
```

**Optimize Query:**
```javascript
// Before: Full collection scan
db.trades.find({ status: "completed" })

// After: Use index
db.trades.createIndex({ status: 1 })
db.trades.find({ status: "completed" })
```

### 4.3 Replication Lag

**Symptoms:**
- Read queries return stale data
- Replica set status shows lag
- Secondary members behind primary

**Diagnosis:**
```bash
# Check replica set status
docker exec mongodb-primary mongosh --eval "rs.status()"

# Check replication lag
docker exec mongodb-primary mongosh --eval "rs.printReplicationInfo()"
docker exec mongodb-primary mongosh --eval "rs.printSecondaryReplicationInfo()"
```

**Solutions:**

**Network Issues:**
```bash
# Check network connectivity between nodes
docker exec mongodb-primary ping mongodb-secondary-1

# Check network latency
docker exec mongodb-primary ping -c 10 mongodb-secondary-1 | tail -1
```

**Secondary Overloaded:**
```bash
# Check secondary CPU/memory
docker stats mongodb-secondary-1 --no-stream

# Route read queries to primary
# In connection string
mongodb://.../?readPreference=primary
```

---

## 5. Network Issues

### 5.1 Service Cannot Reach Another Service

**Symptoms:**
- "Connection refused" errors
- DNS resolution failures
- Network timeout errors

**Diagnosis:**
```bash
# Check if services are on same network
docker network inspect bot-network

# Test DNS resolution
docker exec rust-core-engine nslookup python-ai-service

# Test connectivity
docker exec rust-core-engine curl http://python-ai-service:8000/health

# Check firewall rules
iptables -L -n
```

**Solutions:**

**Not on Same Network:**
```bash
# Add service to network
docker network connect bot-network rust-core-engine
```

**Wrong Service Name:**
```bash
# Use service name from docker-compose.yml
# Not container name
PYTHON_AI_SERVICE_URL=http://python-ai-service:8000  # Correct
# Not: http://python-ai-service-1:8000
```

**Port Not Exposed:**
```yaml
# Ensure port is exposed internally
# docker-compose.yml
python-ai-service:
  ports:
    - "8000:8000"  # Not needed for internal communication
  expose:
    - "8000"  # Sufficient for internal
```

### 5.2 External API Unreachable

**Symptoms:**
- Cannot connect to Binance API
- Timeout connecting to OpenAI
- DNS resolution fails for external hosts

**Diagnosis:**
```bash
# Check DNS resolution
docker exec rust-core-engine nslookup api.binance.com

# Test connectivity
docker exec rust-core-engine curl -I https://api.binance.com

# Check proxy settings
docker exec rust-core-engine env | grep -i proxy
```

**Solutions:**

**Firewall Blocking:**
```bash
# Allow outbound HTTPS
iptables -A OUTPUT -p tcp --dport 443 -j ACCEPT

# Or configure proxy
export HTTPS_PROXY=http://proxy.company.com:8080
```

**DNS Issues:**
```bash
# Use Google DNS
# In docker-compose.yml
services:
  rust-core-engine:
    dns:
      - 8.8.8.8
      - 8.8.4.4
```

---

## 6. Deployment Issues

### 6.1 Build Failures

**Symptoms:**
- Docker build fails
- Out of memory during build
- Dependency download errors

**Diagnosis:**
```bash
# Check build logs
docker compose build rust-core-engine 2>&1 | tee build.log

# Check disk space
df -h

# Check memory during build
docker stats
```

**Solutions:**

**Out of Memory:**
```bash
# Use sequential build
make build-fast

# Or increase Docker memory
# Docker Desktop > Settings > Resources > Memory: 8GB

# Or use multi-stage build to reduce layers
```

**Dependency Download Errors:**
```bash
# Use mirrors/proxies
# In rust-core-engine/.cargo/config.toml
[source.crates-io]
replace-with = "mirror"

[source.mirror]
registry = "https://mirror.example.com/crates.io-index"

# Or retry build
docker compose build --no-cache rust-core-engine
```

### 6.2 Deployment Fails

**Symptoms:**
- Kubernetes rollout stuck
- Pods in CrashLoopBackOff
- ImagePullBackOff errors

**Diagnosis:**
```bash
# Check rollout status
kubectl rollout status deployment/rust-core-engine -n bot-core-production

# Check pod events
kubectl describe pod -l app=rust-core-engine -n bot-core-production

# Check logs
kubectl logs -l app=rust-core-engine -n bot-core-production --tail=100
```

**Solutions:**

**ImagePullBackOff:**
```bash
# Check image exists
docker pull your-registry.com/rust-core-engine:latest

# Check image pull secrets
kubectl get secret regcred -n bot-core-production

# Create pull secret if missing
kubectl create secret docker-registry regcred \
  --docker-server=your-registry.com \
  --docker-username=username \
  --docker-password=password \
  -n bot-core-production
```

**CrashLoopBackOff:**
```bash
# Check pod logs
kubectl logs pod-name -n bot-core-production

# Check previous pod logs
kubectl logs pod-name -n bot-core-production --previous

# Rollback deployment
kubectl rollout undo deployment/rust-core-engine -n bot-core-production
```

---

## 7. Trading Issues

### 7.1 Trades Not Executing

**Symptoms:**
- No trades in logs
- Trade count metric stuck at 0
- API requests to Binance failing

**Diagnosis:**
```bash
# Check if trading is enabled
docker exec rust-core-engine cat /app/config.toml | grep "enabled"

# Check Binance API connectivity
docker exec rust-core-engine curl https://testnet.binance.vision/api/v3/ping

# Check API key validity
docker compose logs rust-core-engine | grep "API key"

# Check account balance
curl http://localhost:8080/api/account/balance
```

**Solutions:**

**Trading Disabled:**
```bash
# Enable in config
vi rust-core-engine/config.toml
[trading]
enabled = true

# Restart service
docker compose restart rust-core-engine
```

**Invalid API Keys:**
```bash
# Generate new keys from Binance
# Update .env
BINANCE_API_KEY=new_key_here
BINANCE_SECRET_KEY=new_secret_here

# Restart service
docker compose restart rust-core-engine
```

**Insufficient Balance:**
```bash
# Check balance
curl http://localhost:8080/api/account/balance

# Fund testnet account
# Go to https://testnet.binance.vision/
# Request test funds
```

### 7.2 High Trade Failure Rate

**Symptoms:**
- `trades_failed_total` metric increasing
- Error logs showing trade rejections
- Low success rate in dashboard

**Diagnosis:**
```bash
# Check failure rate
curl http://localhost:9090/api/v1/query?query='rate(trades_failed_total[5m]) / rate(trades_total[5m])'

# Check error messages
docker compose logs rust-core-engine | grep "trade.*failed"

# Check Binance API status
curl https://www.binance.com/api/v3/systemStatus
```

**Common Causes & Solutions:**

**Insufficient Balance:**
```bash
# Check balance before each trade
# Implement in code
async fn check_balance_before_trade(trade: &Trade) -> Result<(), Error> {
    let balance = get_account_balance().await?;
    if balance < trade.required_balance() {
        return Err(Error::InsufficientBalance);
    }
    Ok(())
}
```

**Price Out of Range:**
```bash
# Check price limits
# Binance has price filters
# Adjust order price to be within limits

# Get exchange info
curl https://api.binance.com/api/v3/exchangeInfo?symbol=BTCUSDT
```

**Rate Limiting:**
```bash
# Implement exponential backoff
# Add rate limiter
use governor::{Quota, RateLimiter};

let limiter = RateLimiter::direct(Quota::per_second(5));
limiter.until_ready().await;
place_order(trade).await?;
```

---

## 8. Diagnostic Commands

### 8.1 Quick Health Check

```bash
#!/bin/bash
# quick-health-check.sh

echo "=== Service Status ==="
docker compose ps

echo -e "\n=== Health Endpoints ==="
curl -sf http://localhost:3000/health && echo "✓ Frontend healthy" || echo "✗ Frontend unhealthy"
curl -sf http://localhost:8080/api/health && echo "✓ Rust healthy" || echo "✗ Rust unhealthy"
curl -sf http://localhost:8000/health && echo "✓ Python healthy" || echo "✗ Python unhealthy"

echo -e "\n=== Resource Usage ==="
docker stats --no-stream --format "table {{.Name}}\t{{.CPUPerc}}\t{{.MemUsage}}\t{{.MemPerc}}"

echo -e "\n=== Recent Errors ==="
docker compose logs --since 1h | grep -i "error\|fatal" | tail -10

echo -e "\n=== Disk Space ==="
df -h | grep -E "Filesystem|/dev/"
```

### 8.2 Collect Diagnostics

```bash
#!/bin/bash
# collect-diagnostics.sh

TIMESTAMP=$(date +%Y%m%d_%H%M%S)
OUTPUT_DIR="diagnostics_$TIMESTAMP"

mkdir -p $OUTPUT_DIR

echo "Collecting diagnostics..."

# Service status
docker compose ps > $OUTPUT_DIR/service_status.txt

# Logs
docker compose logs --tail=1000 > $OUTPUT_DIR/all_logs.txt
docker compose logs rust-core-engine --tail=500 > $OUTPUT_DIR/rust_logs.txt
docker compose logs python-ai-service --tail=500 > $OUTPUT_DIR/python_logs.txt

# Resource usage
docker stats --no-stream > $OUTPUT_DIR/resource_usage.txt

# Network
docker network inspect bot-network > $OUTPUT_DIR/network.json

# Configuration
cp docker-compose.yml $OUTPUT_DIR/
cp .env $OUTPUT_DIR/env.txt  # Be careful with secrets!

# System info
uname -a > $OUTPUT_DIR/system_info.txt
df -h > $OUTPUT_DIR/disk_space.txt
free -h > $OUTPUT_DIR/memory.txt

# Create tarball
tar -czf $OUTPUT_DIR.tar.gz $OUTPUT_DIR/
rm -rf $OUTPUT_DIR/

echo "Diagnostics collected: $OUTPUT_DIR.tar.gz"
```

---

## Appendix: Troubleshooting Flowchart

```mermaid
graph TD
    A[Issue Detected] --> B{Service Running?}
    B -->|No| C[Check Logs]
    B -->|Yes| D{Responding?}

    C --> E{Config Error?}
    E -->|Yes| F[Fix Config]
    E -->|No| G{Dependency Missing?}
    G -->|Yes| H[Start Dependencies]
    G -->|No| I[Check Resources]

    D -->|No| J{Health Check Pass?}
    D -->|Yes| K{Performance Issue?}

    J -->|No| L[Check Health Endpoint]
    J -->|Yes| K

    K -->|High Latency| M[Profile & Optimize]
    K -->|High Memory| N[Check for Leaks]
    K -->|High CPU| O[Optimize Code]

    F --> P[Restart Service]
    H --> P
    I --> P
    L --> P
    M --> Q[Monitor]
    N --> Q
    O --> Q
    P --> Q[Monitor]
```

---

## Revision History

| Version | Date | Author | Changes |
|---------|------|--------|---------|
| 1.0.0 | 2025-10-11 | Operations Team | Initial version |

---

**Document End**
