# Bot-Core Monitoring Guide

**Version:** 1.0.0
**Last Updated:** 2025-11-18
**Status:** Production-Ready
**@spec:** NFR-PERFORMANCE-001, NFR-RELIABILITY-001

---

## Table of Contents

1. [Overview](#overview)
2. [Quick Start](#quick-start)
3. [Architecture](#architecture)
4. [Accessing Monitoring](#accessing-monitoring)
5. [Dashboards](#dashboards)
6. [Alerts](#alerts)
7. [Querying](#querying)
8. [Troubleshooting](#troubleshooting)
9. [Best Practices](#best-practices)

---

## Overview

Bot-Core monitoring infrastructure provides complete observability using industry-standard tools:

- **Prometheus** - Metrics collection and storage
- **Grafana** - Visualization and dashboards
- **Loki** - Log aggregation
- **Alertmanager** - Alert routing and notifications

### Coverage

✅ All 3 application services (Rust, Python, Next.js)
✅ Infrastructure (CPU, Memory, Disk, Network)
✅ Databases (MongoDB, Redis)
✅ Message queues (RabbitMQ)
✅ External APIs (Binance)
✅ Business metrics (Trading, Users, Revenue)
✅ SLO/SLA tracking

### Metrics Collected

- **System:** 20+ host metrics (node_exporter)
- **Containers:** 30+ container metrics (cAdvisor)
- **Application:** 50+ custom metrics per service
- **Database:** 40+ MongoDB metrics
- **Business:** 15+ trading/user metrics

### Log Collection

- **Services:** All Docker container logs
- **Format:** Structured JSON
- **Retention:** 30 days hot, 60 days warm
- **Search:** Full-text with LogQL

---

## Quick Start

### Start Monitoring Stack

```bash
# Start all monitoring services
docker-compose --profile monitoring up -d

# Verify all services running
docker-compose --profile monitoring ps

# Expected output:
# prometheus        Up (healthy)
# grafana           Up (healthy)
# loki              Up (healthy)
# promtail          Up
# alertmanager      Up (healthy)
# node-exporter     Up
# cadvisor          Up
# mongodb-exporter  Up
# redis-exporter    Up
# blackbox-exporter Up
```

### Access UIs

```bash
# Prometheus
open http://localhost:9090

# Grafana (admin / your-password)
open http://localhost:3001

# Alertmanager
open http://localhost:9093
```

### Verify Data Collection

```bash
# Check Prometheus targets (all should be "up")
curl http://localhost:9090/api/v1/targets | jq '.data.activeTargets[] | {job: .labels.job, health: .health}'

# Query a metric
curl 'http://localhost:9090/api/v1/query?query=up' | jq '.data.result[] | {job: .metric.job, value: .value[1]}'

# Check logs in Loki
curl -G "http://localhost:3100/loki/api/v1/query_range" \
  --data-urlencode 'query={job="rust-core-engine"}' | jq '.data.result[0].values' | head -n 5
```

---

## Architecture

### Data Flow

```
┌─────────────────────────────────────────────────────────┐
│                   APPLICATION LAYER                      │
├─────────────────────────────────────────────────────────┤
│                                                           │
│  ┌─────────────┐  ┌─────────────┐  ┌─────────────┐     │
│  │ Rust Core   │  │ Python AI   │  │ Next.js UI  │     │
│  │ :8080       │  │ :8000       │  │ :3000       │     │
│  │ /metrics    │  │ /metrics    │  │ /api/metrics│     │
│  └──────┬──────┘  └──────┬──────┘  └──────┬──────┘     │
│         │                 │                 │            │
│         └─────────────────┼─────────────────┘            │
│                           │                              │
└───────────────────────────┼──────────────────────────────┘
                            │
┌───────────────────────────┼──────────────────────────────┐
│                   METRICS COLLECTION                      │
├───────────────────────────┼──────────────────────────────┤
│                           │                              │
│  ┌────────────────────────▼────────────────────────┐    │
│  │         Prometheus (:9090)                       │    │
│  │  • Scrapes metrics every 15s                     │    │
│  │  • Stores 90 days retention                      │    │
│  │  • Evaluates alert rules every 30s               │    │
│  └────────────┬──────────────┬──────────────────────┘    │
│               │              │                            │
└───────────────┼──────────────┼────────────────────────────┘
                │              │
┌───────────────┼──────────────┼────────────────────────────┐
│          VISUALIZATION & ALERTING                         │
├───────────────┼──────────────┼────────────────────────────┤
│               │              │                            │
│  ┌────────────▼───────┐  ┌──▼──────────────┐            │
│  │  Grafana (:3001)   │  │ Alertmanager    │            │
│  │  • 7 dashboards    │  │ (:9093)         │            │
│  │  • Real-time viz   │  │ • Routes alerts │            │
│  │  • Query editor    │  │ • Notifications │            │
│  └────────────────────┘  └───┬─────────────┘            │
│                              │                            │
└──────────────────────────────┼────────────────────────────┘
                               │
┌──────────────────────────────┼────────────────────────────┐
│                    NOTIFICATIONS                          │
├──────────────────────────────┼────────────────────────────┤
│                              │                            │
│  ┌───────────────┬───────────▼───────────┬────────────┐  │
│  │  Slack        │  PagerDuty            │  Email     │  │
│  │  #alerts      │  Critical only        │  Team      │  │
│  └───────────────┴───────────────────────┴────────────┘  │
│                                                           │
└─────────────────────────────────────────────────────────┘

┌─────────────────────────────────────────────────────────┐
│                    LOG COLLECTION                        │
├─────────────────────────────────────────────────────────┤
│                                                           │
│  ┌─────────────────────────────────────────┐            │
│  │ Promtail (Log Collector)                │            │
│  │ • Reads Docker container logs           │            │
│  │ • Parses JSON structured logs           │            │
│  │ • Labels by service, level              │            │
│  └──────────────────┬──────────────────────┘            │
│                     │                                     │
│  ┌──────────────────▼──────────────────────┐            │
│  │ Loki (:3100)                            │            │
│  │ • Indexes logs by labels                │            │
│  │ • 30-day retention                       │            │
│  │ • LogQL queries                          │            │
│  └──────────────────┬──────────────────────┘            │
│                     │                                     │
│  ┌──────────────────▼──────────────────────┐            │
│  │ Grafana (Explore Logs)                  │            │
│  │ • Log viewer                             │            │
│  │ • Correlation with metrics               │            │
│  └─────────────────────────────────────────┘            │
│                                                           │
└─────────────────────────────────────────────────────────┘
```

### Component Roles

| Component | Port | Purpose |
|-----------|------|---------|
| Prometheus | 9090 | Metrics storage, query engine |
| Grafana | 3001 | Dashboards, visualization |
| Loki | 3100 | Log aggregation |
| Promtail | - | Log collection from Docker |
| Alertmanager | 9093 | Alert routing, notifications |
| Node Exporter | 9100 | Host system metrics |
| cAdvisor | 8081 | Container metrics |
| MongoDB Exporter | 9216 | MongoDB metrics |
| Redis Exporter | 9121 | Redis metrics |
| Blackbox Exporter | 9115 | HTTP/WS health checks |

---

## Accessing Monitoring

### Grafana Dashboards

**URL:** http://localhost:3001

**Login:**
- Username: `admin`
- Password: Set in `.env` as `GRAFANA_PASSWORD`

**Available Dashboards:**

1. **System Overview** - `/d/system-overview`
   - Service health status
   - CPU, Memory, Disk usage
   - Network traffic
   - Active alerts

2. **Trading Performance** - `/d/trading-performance`
   - Trades per minute
   - Success rate
   - Execution latency
   - Portfolio value
   - P&L tracking

3. **API Performance** - `/d/api-performance`
   - Request rate
   - Error rate
   - Latency percentiles (p50, p95, p99)
   - Top endpoints

4. **AI Analysis** - `/d/ai-analysis`
   - Analysis rate
   - Model confidence
   - Predictions by signal
   - OpenAI API usage

5. **Database Performance** - `/d/database-performance`
   - Query rate and latency
   - Connection pool usage
   - Slow queries
   - Replication lag

6. **WebSocket Metrics** - `/d/websocket-metrics`
   - Active connections
   - Message rate
   - Latency
   - Error rate

7. **Business Metrics** - `/d/business-metrics`
   - Active users
   - Trading volume
   - Revenue
   - Portfolio growth

### Prometheus

**URL:** http://localhost:9090

**Use Cases:**
- Query metrics directly (PromQL)
- View targets health
- Check alert rules
- Explore time series data

**Common Queries:**

```promql
# Service uptime
up{job="rust-core-engine"}

# CPU usage
100 - (avg by (instance) (rate(node_cpu_seconds_total{mode="idle"}[5m])) * 100)

# Request rate
rate(http_requests_total[5m])

# Error rate
rate(http_requests_total{status=~"5.."}[5m]) / rate(http_requests_total[5m])

# Trade success rate
rate(trades_success_total[5m]) / rate(trades_total[5m])
```

### Loki (Logs)

**URL:** http://localhost:3100 (API)

**Access via Grafana:**
- Navigate to "Explore" in Grafana
- Select "Loki" datasource
- Use LogQL to query logs

**Common LogQL Queries:**

```logql
# All errors from Rust service (last 5m)
{job="rust-core-engine"} |= "ERROR"

# Trade execution logs
{job="rust-core-engine"} |~ "trade.*executed"

# AI analysis logs with confidence > 0.8
{job="python-ai-service"} | json | confidence > 0.8

# All logs from specific container
{container_name="rust-core-engine"}

# Count errors by service (last 1h)
sum by (job) (count_over_time({job=~".+"} |= "ERROR" [1h]))

# Slow queries (>1s duration)
{job="rust-core-engine"} | json | duration_ms > 1000
```

### Alertmanager

**URL:** http://localhost:9093

**Features:**
- View active alerts
- Silence alerts temporarily
- View notification history
- Test alert routing

---

## Dashboards

### 1. System Overview

**Purpose:** High-level infrastructure health

**Key Panels:**
- **Service Status** - Up/Down for all services
- **CPU Usage** - Time series by host
- **Memory Usage** - Utilization percentage
- **Disk Space** - Available space per mount point
- **Network Traffic** - RX/TX bytes
- **Active Alerts** - Current firing alerts

**When to Check:**
- Daily health check
- Before deployments
- During incidents
- Resource planning

**Red Flags:**
- Any service showing "Down"
- CPU > 90% for >5 minutes
- Memory > 95%
- Disk < 10% free
- Multiple active alerts

### 2. Trading Performance

**Purpose:** Monitor trading execution and portfolio

**Key Panels:**
- **Trades/Minute** - Trading activity
- **Success Rate** - Target: >95%
- **Execution Latency** - Target p95 < 1s
- **Portfolio Value** - Real-time value
- **Daily P&L** - Profit/Loss tracking
- **Open Positions** - Current positions
- **Trading Balance** - Available funds

**When to Check:**
- Before/during market hours
- After strategy changes
- When alerts fire
- End-of-day review

**Red Flags:**
- Success rate < 95%
- Execution latency > 1s (p95)
- Daily loss > 5% portfolio
- Balance < $100
- >10 open positions

### 3. API Performance

**Purpose:** Monitor API latency and errors

**Key Panels:**
- **Request Rate** - Requests per second
- **Error Rate** - 5xx errors
- **Latency Distribution** - Heatmap
- **p95/p99 Latency** - Percentile tracking
- **Status Codes** - Distribution
- **Slow Queries** - Queries > 1s

**When to Check:**
- High traffic periods
- After deployments
- Performance tuning
- SLO compliance

**Red Flags:**
- Error rate > 5%
- p95 latency > 500ms
- p99 latency > 2s
- Sudden traffic spikes
- Availability < 99.5%

### 4. AI Analysis

**Purpose:** Monitor AI/ML service performance

**Key Panels:**
- **Analyses/Minute** - Analysis rate
- **Analysis Duration** - Latency tracking
- **Model Confidence** - Confidence distribution
- **Predictions by Signal** - LONG/SHORT/NEUTRAL
- **Prediction Accuracy** - Accuracy over time
- **OpenAI Usage** - API request rate
- **OpenAI Errors** - API error rate

**When to Check:**
- Model changes
- High analysis volume
- Accuracy concerns
- OpenAI issues

**Red Flags:**
- Analysis duration > 5s (p95)
- Avg confidence < 45%
- OpenAI error rate > 5%
- Prediction accuracy < 55%

### 5. Database Performance

**Purpose:** Monitor MongoDB performance

**Key Panels:**
- **Query Rate** - Queries per second
- **Query Latency** - p95/p99
- **Active Connections** - Current connections
- **Connection Pool** - Utilization %
- **Slow Queries** - Queries > 100ms
- **Operations** - By type (read/write/command)
- **Replication Lag** - Replica set lag

**When to Check:**
- Database migrations
- Schema changes
- Performance issues
- Before scaling

**Red Flags:**
- Query latency > 100ms (avg)
- Connection pool > 80%
- Replication lag > 30s
- Slow query count increasing

### 6. WebSocket Metrics

**Purpose:** Monitor real-time connections

**Key Panels:**
- **Active Connections** - Current WS connections
- **Messages Sent/Received** - Message rate
- **Message Latency** - p95 latency
- **Error Rate** - WS errors
- **Connection Duration** - Histogram
- **Disconnect Reasons** - Why clients disconnect

**When to Check:**
- High user activity
- Connection issues
- Latency problems

**Red Flags:**
- Connections > 1000 (warning)
- Connections > 5000 (critical)
- Message latency > 100ms
- Error rate > 5%

### 7. Business Metrics

**Purpose:** Track business KPIs

**Key Panels:**
- **Active Users** - Current active users
- **New Registrations** - Daily signups
- **Trading Volume** - 24h volume
- **Revenue** - 24h revenue
- **Portfolio Growth** - % growth
- **User Engagement** - Sessions, logins
- **Top Strategies** - By trade count

**When to Check:**
- Business reviews
- Growth tracking
- Revenue analysis

---

## Alerts

### Alert Severity Levels

| Severity | Response Time | Notification | Examples |
|----------|--------------|--------------|----------|
| **Critical** | Immediate | PagerDuty + Email + Slack | Service down, High error rate, Trading halted |
| **Warning** | Within 1 hour | Slack | High CPU, Low balance, Slow queries |
| **Info** | Review daily | Log only | High traffic, Normal operations |

### Critical Alerts

**ServiceDown**
- **Trigger:** Service unreachable for 2 minutes
- **Impact:** Complete service outage
- **Action:** Check container status, restart if needed
- **Runbook:** `docs/runbooks/SERVICE_DOWN_RUNBOOK.md`

**HighErrorRate**
- **Trigger:** >5% error rate for 5 minutes
- **Impact:** Degraded user experience
- **Action:** Check logs, recent deployments, rollback if needed
- **Runbook:** `docs/runbooks/HIGH_ERROR_RATE_RUNBOOK.md`

**TradingHalted**
- **Trigger:** No trades for 10 minutes (when trading enabled)
- **Impact:** Trading not executing
- **Action:** Check Binance API, verify balance, check errors
- **Runbook:** `docs/runbooks/TRADING_HALTED_RUNBOOK.md`

**DiskSpaceLow**
- **Trigger:** <10% disk space available
- **Impact:** System may become unstable
- **Action:** Clean logs, remove old data, provision storage
- **Runbook:** `docs/runbooks/DISK_SPACE_LOW_RUNBOOK.md`

**MongoDBDown**
- **Trigger:** Database unreachable for 1 minute
- **Impact:** All data operations fail
- **Action:** Check MongoDB container, verify credentials
- **Runbook:** `docs/runbooks/MONGODB_DOWN_RUNBOOK.md`

### Warning Alerts

**HighCPUUsage**
- **Trigger:** >80% CPU for 5 minutes
- **Impact:** Performance degradation
- **Action:** Identify CPU-intensive processes, scale resources

**HighMemoryUsage**
- **Trigger:** >90% memory for 5 minutes
- **Impact:** Risk of OOM kills
- **Action:** Identify memory leaks, restart services, scale

**HighLatencyP95**
- **Trigger:** p95 latency >1s for 5 minutes
- **Impact:** Slow response times
- **Action:** Check slow queries, resource usage, external APIs

**LowTradingBalance**
- **Trigger:** Balance < $100
- **Impact:** May not execute trades
- **Action:** Add funds or adjust trading parameters

### Alert Actions

**View Active Alerts:**
```bash
# Via API
curl http://localhost:9093/api/v2/alerts | jq '.[] | {name: .labels.alertname, severity: .labels.severity, status: .status.state}'

# Via Grafana
# Navigate to Alerting → Alert Rules
```

**Silence an Alert:**
```bash
# Via Alertmanager UI
open http://localhost:9093/#/silences

# Or via API
curl -X POST http://localhost:9093/api/v2/silences \
  -H "Content-Type: application/json" \
  -d '{
    "matchers": [{"name": "alertname", "value": "HighCPUUsage", "isRegex": false}],
    "startsAt": "'$(date -u +%Y-%m-%dT%H:%M:%S.000Z)'",
    "endsAt": "'$(date -u -d '+1 hour' +%Y-%m-%dT%H:%M:%S.000Z)'",
    "createdBy": "admin",
    "comment": "Planned maintenance"
  }'
```

**Test Alert:**
```bash
# Stop a service to trigger ServiceDown alert
docker stop python-ai-service

# Wait 2 minutes for alert to fire
sleep 130

# Check alert fired
curl http://localhost:9093/api/v2/alerts | grep ServiceDown

# Restart service
docker start python-ai-service

# Alert will auto-resolve
```

---

## Querying

### PromQL Basics

**Rate Functions:**
```promql
# Request rate (per second)
rate(http_requests_total[5m])

# Trade rate (per minute)
rate(trades_total[1m]) * 60

# Error rate (percentage)
rate(http_requests_total{status=~"5.."}[5m]) / rate(http_requests_total[5m]) * 100
```

**Aggregations:**
```promql
# Sum across all instances
sum(rate(trades_total[5m]))

# Average by service
avg by (service) (http_request_duration_seconds)

# Count unique services
count(up == 1)

# Max CPU usage
max(instance:node_cpu_utilization:rate5m)
```

**Percentiles:**
```promql
# p50 latency
histogram_quantile(0.50, rate(http_request_duration_seconds_bucket[5m]))

# p95 latency
histogram_quantile(0.95, rate(http_request_duration_seconds_bucket[5m]))

# p99 latency
histogram_quantile(0.99, rate(http_request_duration_seconds_bucket[5m]))
```

**Time Comparisons:**
```promql
# Current vs 1 hour ago
http_requests_total - http_requests_total offset 1h

# Growth rate (last hour)
(http_requests_total - http_requests_total offset 1h) / http_requests_total offset 1h * 100
```

### LogQL Basics

**Label Filtering:**
```logql
# By job
{job="rust-core-engine"}

# By level
{job="rust-core-engine", level="ERROR"}

# Multiple jobs
{job=~"rust-core-engine|python-ai-service"}
```

**Line Filtering:**
```logql
# Contains "trade"
{job="rust-core-engine"} |= "trade"

# Doesn't contain "health"
{job="rust-core-engine"} != "health"

# Regex match
{job="rust-core-engine"} |~ "trade.*executed"
```

**JSON Parsing:**
```logql
# Parse and filter
{job="python-ai-service"} | json | confidence > 0.8

# Extract field
{job="rust-core-engine"} | json | line_format "{{.message}}"
```

**Aggregations:**
```logql
# Count logs
count_over_time({job="rust-core-engine"}[5m])

# Count errors by service
sum by (job) (count_over_time({level="ERROR"}[1h]))

# Rate of errors
rate({level="ERROR"}[5m])
```

---

## Troubleshooting

### Common Issues

#### 1. Prometheus Targets Down

**Symptom:** Targets showing as "Down" in Prometheus

**Check:**
```bash
# View target status
curl http://localhost:9090/api/v1/targets | jq '.data.activeTargets[] | select(.health=="down")'

# Check service health
docker ps | grep <service-name>
curl http://<service>:<port>/metrics
```

**Solutions:**
- Verify service is running: `docker restart <service>`
- Check network connectivity: `docker network inspect bot-network`
- Verify metrics endpoint: `curl http://<service>:<port>/metrics`
- Check Prometheus config: `docker logs prometheus`

#### 2. No Metrics Appearing

**Symptom:** Empty graphs in Grafana

**Check:**
```bash
# Verify Prometheus is scraping
curl 'http://localhost:9090/api/v1/query?query=up'

# Check scrape errors
curl http://localhost:9090/api/v1/targets | jq '.data.activeTargets[] | select(.lastError != "")'

# Verify datasource in Grafana
curl -u admin:$GRAFANA_PASSWORD http://localhost:3001/api/datasources
```

**Solutions:**
- Wait 1-2 minutes for first scrape
- Check Prometheus logs: `docker logs prometheus`
- Verify Grafana datasource: Settings → Data Sources → Prometheus
- Test query directly in Prometheus UI

#### 3. Logs Not Showing in Loki

**Symptom:** No logs in Grafana Explore

**Check:**
```bash
# Check Promtail is running
docker logs promtail

# Query Loki directly
curl -G "http://localhost:3100/loki/api/v1/query_range" \
  --data-urlencode 'query={job=~".+"}' | jq '.data.result'

# Check container logs exist
docker logs rust-core-engine
```

**Solutions:**
- Verify Promtail has access to Docker socket
- Check Promtail config: `docker logs promtail | grep ERROR`
- Ensure logs are in JSON format
- Restart Promtail: `docker restart promtail`

#### 4. Alerts Not Firing

**Symptom:** Expected alerts not triggering

**Check:**
```bash
# View alert rules
curl http://localhost:9090/api/v1/rules | jq '.data.groups[].rules[] | select(.type=="alerting")'

# Check rule evaluation
curl http://localhost:9090/api/v1/rules | jq '.data.groups[].rules[] | select(.name=="ServiceDown")'

# Test query manually
curl 'http://localhost:9090/api/v1/query?query=up==0'
```

**Solutions:**
- Verify alert rule syntax in Prometheus
- Check alert evaluation interval (30s default)
- Ensure Alertmanager is connected
- Test query returns expected results
- Check for silences: http://localhost:9093/#/silences

#### 5. Alerts Not Notifying

**Symptom:** Alerts firing but no notifications

**Check:**
```bash
# View Alertmanager config
docker exec alertmanager cat /etc/alertmanager/alertmanager.yml

# Check alert status
curl http://localhost:9093/api/v2/alerts

# View Alertmanager logs
docker logs alertmanager | grep -i error
```

**Solutions:**
- Verify Slack webhook URL in config
- Check email SMTP settings
- Test notification manually in Alertmanager UI
- Verify alert routing rules
- Check for silences blocking notifications

### Debugging Commands

**Check all monitoring services:**
```bash
docker-compose --profile monitoring ps
```

**View logs:**
```bash
docker logs prometheus
docker logs grafana
docker logs loki
docker logs promtail
docker logs alertmanager
```

**Restart monitoring stack:**
```bash
docker-compose --profile monitoring restart
```

**Reload Prometheus config (without restart):**
```bash
curl -X POST http://localhost:9090/-/reload
```

**Validate Prometheus config:**
```bash
docker run --rm -v $(pwd)/infrastructure/monitoring/prometheus:/prometheus prom/prometheus:latest \
  promtool check config /prometheus/prometheus.yml
```

---

## Best Practices

### Dashboard Usage

1. **Set Time Range**
   - Use "Last 5 minutes" for real-time monitoring
   - Use "Last 24 hours" for trend analysis
   - Use "Last 7 days" for capacity planning

2. **Use Variables**
   - Dashboards support service selection
   - Filter by environment (prod/staging)
   - Select specific instances

3. **Create Snapshots**
   - Save dashboard state during incidents
   - Share with team members
   - Document in postmortems

4. **Export/Import**
   - Export dashboards as JSON for version control
   - Import community dashboards for new metrics
   - Keep backups of custom dashboards

### Query Optimization

1. **Use Recording Rules**
   - Pre-computed for common queries
   - Faster dashboard loading
   - Located in `recording-rules/rules.yml`

2. **Limit Time Range**
   - Shorter ranges = faster queries
   - Use `[5m]` instead of `[1h]` when possible

3. **Use Label Matchers**
   - Filter early: `{job="rust-core-engine"}`
   - More specific = faster: `{job="rust-core-engine", instance="host1"}`

4. **Aggregate Appropriately**
   - `sum by (job)` instead of `sum`
   - Group by meaningful labels only

### Alert Management

1. **Tune Thresholds**
   - Adjust based on actual traffic patterns
   - Reduce false positives
   - Balance sensitivity vs noise

2. **Set Proper `for` Duration**
   - Avoid flapping: use `for: 5m` minimum
   - Critical alerts: shorter duration (`for: 2m`)
   - Warnings: longer duration (`for: 10m`)

3. **Use Inhibition Rules**
   - Critical suppresses warning for same service
   - ServiceDown suppresses all other alerts

4. **Document Runbooks**
   - Every alert should have a runbook
   - Include investigation steps
   - Provide clear resolution steps

### Maintenance

1. **Monitor Storage**
   ```bash
   # Check Prometheus disk usage
   du -sh /var/lib/docker/volumes/bot-core_prometheus_data

   # Check Loki disk usage
   du -sh /var/lib/docker/volumes/bot-core_loki_data
   ```

2. **Review Retention**
   - Prometheus: 90 days (configurable)
   - Loki: 30 days (configurable)
   - Adjust based on disk space

3. **Backup Dashboards**
   ```bash
   # Export all dashboards
   curl -u admin:$GRAFANA_PASSWORD \
     http://localhost:3001/api/search | jq -r '.[] | select(.type=="dash-db") | .uid' | \
     while read uid; do
       curl -u admin:$GRAFANA_PASSWORD \
         "http://localhost:3001/api/dashboards/uid/$uid" > "backup_${uid}.json"
     done
   ```

4. **Update Regularly**
   - Keep monitoring stack updated
   - Review security advisories
   - Test updates in staging first

### Security

1. **Network Isolation**
   - Monitoring services in `bot-network`
   - No external exposure except Grafana
   - Use reverse proxy for production

2. **Authentication**
   - Change default Grafana password
   - Use strong passwords (>20 characters)
   - Enable 2FA in Grafana (production)

3. **Secrets Management**
   - Store credentials in `.env`
   - Never commit secrets to Git
   - Rotate API keys regularly

4. **Access Control**
   - Limit Grafana admin users
   - Use viewer role for read-only access
   - Audit user actions

---

## Additional Resources

### Documentation

- [Prometheus Documentation](https://prometheus.io/docs/)
- [Grafana Documentation](https://grafana.com/docs/)
- [Loki Documentation](https://grafana.com/docs/loki/)
- [PromQL Cheat Sheet](https://promlabs.com/promql-cheat-sheet/)
- [LogQL Cheat Sheet](https://grafana.com/docs/loki/latest/logql/)

### Internal Docs

- **Implementation Plan:** `docs/plans/251118-monitoring-infrastructure-implementation-plan.md`
- **Runbooks:** `docs/runbooks/`
- **Specs:** `specs/04-deployment/4.3-monitoring/`
- **Alert Rules:** `infrastructure/monitoring/prometheus/alerts/`

### Support

- **Issues:** Create GitHub issue with `monitoring` label
- **Slack:** `#bot-core-monitoring` channel
- **On-call:** Check PagerDuty rotation

---

**Last Updated:** 2025-11-18
**Version:** 1.0.0
**Maintainer:** Operations Team
