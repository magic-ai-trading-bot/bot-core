# Health Check Endpoints Documentation

This document describes the health check system for the Bot-Core trading platform.

## Overview

The health check system provides comprehensive monitoring of all services, databases, and system resources. It consists of automated scripts and standardized endpoints across all services.

**Last Updated:** 2025-11-18

---

## Health Check Script

### Location
`scripts/health-check.sh`

### Usage

```bash
# Run health check with default settings
./scripts/health-check.sh

# Run health check with custom report location
./scripts/health-check.sh /path/to/report.txt

# Run from cron (automated monitoring)
*/5 * * * * cd /path/to/bot-core && ./scripts/health-check.sh >> /var/log/health-check.log 2>&1
```

### Exit Codes

- `0` - All services healthy
- `1` - One or more services unhealthy (critical)
- `2` - One or more services degraded (warning)

### Report Format

The script generates a detailed report including:
- Service-by-service health status
- Response time metrics
- Database connectivity
- System resource usage
- Overall health percentage

---

## Service Endpoints

### 1. Rust Core Engine (Port 8080)

#### Basic Health Check
```
GET /api/health
```

**Response (200 OK):**
```json
{
  "status": "healthy",
  "timestamp": "2025-11-18T10:30:00Z",
  "version": "1.0.0",
  "uptime": 3600
}
```

#### Readiness Probe
```
GET /api/health/ready
```

**Response (200 OK when ready):**
```json
{
  "status": "ready",
  "database": "connected",
  "dependencies": {
    "mongodb": "healthy",
    "binance_api": "reachable"
  }
}
```

**Response (503 when not ready):**
```json
{
  "status": "not_ready",
  "reason": "database_unavailable"
}
```

#### Liveness Probe
```
GET /api/health/live
```

**Response (200 OK):**
```json
{
  "status": "alive",
  "timestamp": "2025-11-18T10:30:00Z"
}
```

#### Ping Endpoint
```
GET /api/ping
```

**Response (200 OK):**
```json
{
  "pong": true,
  "timestamp": "2025-11-18T10:30:00Z"
}
```

#### WebSocket Health
```
WS /ws
```

Send `ping` message, expect `pong` response.

---

### 2. Python AI Service (Port 8000)

#### Basic Health Check
```
GET /health
```

**Response (200 OK):**
```json
{
  "status": "healthy",
  "service": "python-ai-service",
  "version": "1.0.0",
  "timestamp": "2025-11-18T10:30:00Z",
  "ml_models_loaded": true
}
```

#### Readiness Probe
```
GET /health/ready
```

**Response (200 OK when ready):**
```json
{
  "status": "ready",
  "models": {
    "lstm": "loaded",
    "gru": "loaded",
    "transformer": "loaded"
  },
  "database": "connected"
}
```

#### Liveness Probe
```
GET /health/live
```

**Response (200 OK):**
```json
{
  "status": "alive"
}
```

#### API Documentation
```
GET /docs
```

FastAPI automatic documentation (Swagger UI)

---

### 3. Frontend Dashboard (Port 3000)

#### Basic Health Check
```
GET /api/health
```

**Response (200 OK):**
```json
{
  "status": "healthy",
  "service": "frontend-dashboard",
  "version": "1.0.0"
}
```

#### Application Root
```
GET /
```

Returns the main application page (200 OK if healthy)

---

### 4. MongoDB Database (Port 27017)

#### Connection Test (using mongosh)

```bash
mongosh "$DATABASE_URL" --eval "db.adminCommand('ping')"
```

**Success Response:**
```json
{ "ok": 1 }
```

#### Database Stats

```bash
mongosh "$DATABASE_URL" --eval "db.stats()"
```

Returns database statistics including:
- Collections count
- Data size
- Index size
- Storage size

---

### 5. Redis Cache (Port 6379) [Optional]

#### Ping Test

```bash
redis-cli -h localhost -p 6379 ping
```

**Response:** `PONG`

#### Info Command

```bash
redis-cli -h localhost -p 6379 INFO
```

Returns comprehensive Redis server information

---

### 6. RabbitMQ Message Queue (Port 5672) [Optional]

#### Management API (Port 15672)

```
GET http://localhost:15672/api/healthchecks/node
```

**Response (200 OK):**
```json
{
  "status": "ok"
}
```

---

## Monitoring Thresholds

### CPU Usage
- **Healthy:** < 80%
- **Degraded:** 80-90%
- **Unhealthy:** > 90%

### Memory Usage
- **Healthy:** < 90%
- **Degraded:** 90-95%
- **Unhealthy:** > 95%

### Disk Usage
- **Healthy:** < 90%
- **Degraded:** 90-95%
- **Unhealthy:** > 95%

### Response Time
- **Healthy:** < 100ms
- **Degraded:** 100-500ms
- **Unhealthy:** > 500ms

---

## Kubernetes/Docker Health Checks

### Docker Compose Healthchecks

Each service in `docker-compose.yml` includes:

```yaml
healthcheck:
  test: ["CMD", "curl", "-f", "http://localhost:8080/api/health"]
  interval: 30s
  timeout: 10s
  retries: 3
  start_period: 40s
```

### Kubernetes Probes

Example for Rust Core Engine:

```yaml
livenessProbe:
  httpGet:
    path: /api/health/live
    port: 8080
  initialDelaySeconds: 30
  periodSeconds: 10
  timeoutSeconds: 5
  failureThreshold: 3

readinessProbe:
  httpGet:
    path: /api/health/ready
    port: 8080
  initialDelaySeconds: 10
  periodSeconds: 5
  timeoutSeconds: 3
  failureThreshold: 3
```

---

## Automated Monitoring

### Cron Job Setup

Add to crontab for automated health monitoring:

```cron
# Run health check every 5 minutes
*/5 * * * * cd /path/to/bot-core && ./scripts/health-check.sh >> /var/log/bot-core-health.log 2>&1

# Send alert if unhealthy
*/5 * * * * cd /path/to/bot-core && ./scripts/health-check.sh || echo "ALERT: Services unhealthy" | mail -s "Bot-Core Alert" admin@example.com
```

### GitHub Actions Monitoring

The CI/CD pipeline includes automated health checks:
- Post-deployment validation
- Integration test health verification
- Scheduled weekly health audits

---

## Troubleshooting

### Service Unreachable

1. Check if service is running:
   ```bash
   docker-compose ps
   ```

2. Check service logs:
   ```bash
   docker-compose logs [service-name]
   ```

3. Check port availability:
   ```bash
   lsof -i :[port]
   ```

### Database Connection Failed

1. Verify MongoDB is running:
   ```bash
   docker-compose ps mongodb
   ```

2. Test connection manually:
   ```bash
   mongosh "$DATABASE_URL" --eval "db.adminCommand('ping')"
   ```

3. Check DATABASE_URL in `.env`:
   ```bash
   grep DATABASE_URL .env
   ```

### High Resource Usage

1. Check Docker container stats:
   ```bash
   docker stats
   ```

2. Review resource limits in `docker-compose.yml`

3. Check for memory leaks:
   ```bash
   ./scripts/bot.sh logs --service [service-name] | grep -i "memory\|oom"
   ```

---

## Best Practices

### 1. Regular Monitoring
- Run health checks at least every 5 minutes in production
- Set up alerts for critical failures
- Monitor trends over time

### 2. Graceful Degradation
- Services should handle degraded dependencies
- Implement circuit breakers for external APIs
- Cache critical data when possible

### 3. Health Check Design
- Keep health checks lightweight (< 100ms)
- Don't include heavy operations in health checks
- Use separate readiness and liveness probes
- Include version information in responses

### 4. Logging
- Log all health check results
- Include timestamps and response times
- Separate logs by severity level

### 5. Testing
- Test health checks in CI/CD pipeline
- Simulate failure scenarios
- Verify alert mechanisms

---

## Integration Examples

### Monitoring Dashboard

Example integration with Grafana:

```yaml
# Prometheus scrape config
scrape_configs:
  - job_name: 'bot-core'
    static_configs:
      - targets:
          - 'localhost:8080'  # Rust
          - 'localhost:8000'  # Python
          - 'localhost:3000'  # Frontend
    metrics_path: '/metrics'
    scrape_interval: 15s
```

### Alert Manager

Example alert rules:

```yaml
groups:
  - name: bot-core-alerts
    rules:
      - alert: ServiceDown
        expr: up == 0
        for: 2m
        labels:
          severity: critical
        annotations:
          summary: "Service {{ $labels.job }} is down"

      - alert: HighMemoryUsage
        expr: memory_usage_percent > 90
        for: 5m
        labels:
          severity: warning
        annotations:
          summary: "High memory usage on {{ $labels.instance }}"
```

---

## Related Documentation

- [Pre-Deployment Checks](./PRE_DEPLOYMENT_GUIDE.md)
- [Deployment Procedures](../specs/04-deployment/4.2-deployment-procedures.md)
- [Monitoring & Operations](../specs/05-operations/5.1-monitoring.md)
- [Disaster Recovery](../specs/05-operations/5.3-disaster-recovery.md)

---

## Support

For issues or questions:
1. Check troubleshooting section above
2. Review service logs: `./scripts/bot.sh logs`
3. Run full health check: `./scripts/health-check.sh`
4. Contact DevOps team

**Emergency Contact:** devops@example.com
