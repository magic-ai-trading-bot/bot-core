# Metrics & Monitoring Specification

**Document Version:** 1.0.0
**Last Updated:** 2025-10-11
**Status:** Active
**Owner:** Operations Team

---

## Table of Contents

- [1. Overview](#1-overview)
- [2. Monitoring Stack](#2-monitoring-stack)
- [3. Key Metrics](#3-key-metrics)
- [4. Prometheus Configuration](#4-prometheus-configuration)
- [5. Grafana Dashboards](#5-grafana-dashboards)
- [6. Alerting Rules](#6-alerting-rules)
- [7. Service Level Objectives](#7-service-level-objectives)

---

## 1. Overview

### 1.1 Purpose

Define metrics collection, monitoring infrastructure, and alerting for the Bot Core platform.

### 1.2 Monitoring Goals

- Real-time system health visibility
- Proactive issue detection
- Performance optimization insights
- Business metrics tracking
- SLO/SLA compliance monitoring

---

## 2. Monitoring Stack

### 2.1 Architecture

```
┌─────────────────────────────────────────────────────────┐
│              Monitoring Architecture                     │
├─────────────────────────────────────────────────────────┤
│                                                           │
│  ┌─────────┐  ┌─────────┐  ┌─────────┐                 │
│  │Frontend │  │  Rust   │  │ Python  │                 │
│  │  /metrics│  │ /metrics│  │/metrics │                 │
│  └────┬────┘  └────┬────┘  └────┬────┘                 │
│       │            │            │                        │
│       └────────────┼────────────┘                        │
│                    │                                      │
│            ┌───────▼────────┐                           │
│            │  Prometheus    │                           │
│            │  (Scraper +    │                           │
│            │   Storage)     │                           │
│            └───────┬────────┘                           │
│                    │                                      │
│          ┌─────────┼─────────┐                          │
│          │                   │                           │
│    ┌─────▼──────┐    ┌──────▼─────┐                    │
│    │  Grafana   │    │Alertmanager│                    │
│    │ (Visualize)│    │  (Alerts)  │                    │
│    └────────────┘    └──────┬─────┘                    │
│                              │                           │
│                    ┌─────────▼─────────┐                │
│                    │  Slack/PagerDuty  │                │
│                    │   (Notifications) │                │
│                    └───────────────────┘                │
│                                                           │
└─────────────────────────────────────────────────────────┘
```

### 2.2 Prometheus Setup

**Docker Compose:**
```yaml
# infrastructure/monitoring/docker-compose-prometheus.yml
version: '3.8'

services:
  prometheus:
    image: prom/prometheus:latest
    container_name: prometheus
    restart: unless-stopped
    ports:
      - "9090:9090"
    volumes:
      - ./prometheus/prometheus.yml:/etc/prometheus/prometheus.yml
      - ./prometheus/rules:/etc/prometheus/rules
      - prometheus_data:/prometheus
    command:
      - '--config.file=/etc/prometheus/prometheus.yml'
      - '--storage.tsdb.path=/prometheus'
      - '--storage.tsdb.retention.time=90d'
      - '--web.console.libraries=/usr/share/prometheus/console_libraries'
      - '--web.console.templates=/usr/share/prometheus/consoles'
      - '--web.enable-lifecycle'
    networks:
      - monitoring

  alertmanager:
    image: prom/alertmanager:latest
    container_name: alertmanager
    restart: unless-stopped
    ports:
      - "9093:9093"
    volumes:
      - ./alertmanager/config.yml:/etc/alertmanager/config.yml
      - alertmanager_data:/alertmanager
    command:
      - '--config.file=/etc/alertmanager/config.yml'
      - '--storage.path=/alertmanager'
    networks:
      - monitoring

  node_exporter:
    image: prom/node-exporter:latest
    container_name: node_exporter
    restart: unless-stopped
    ports:
      - "9100:9100"
    command:
      - '--path.rootfs=/host'
    pid: host
    volumes:
      - '/:/host:ro,rslave'
    networks:
      - monitoring

  cadvisor:
    image: gcr.io/cadvisor/cadvisor:latest
    container_name: cadvisor
    restart: unless-stopped
    ports:
      - "8081:8080"
    volumes:
      - /:/rootfs:ro
      - /var/run:/var/run:ro
      - /sys:/sys:ro
      - /var/lib/docker/:/var/lib/docker:ro
      - /dev/disk/:/dev/disk:ro
    privileged: true
    devices:
      - /dev/kmsg
    networks:
      - monitoring

volumes:
  prometheus_data:
  alertmanager_data:

networks:
  monitoring:
    driver: bridge
```

---

## 3. Key Metrics

### 3.1 System Metrics

**Node Exporter Metrics:**
- `node_cpu_seconds_total` - CPU usage
- `node_memory_MemAvailable_bytes` - Available memory
- `node_memory_MemTotal_bytes` - Total memory
- `node_disk_io_time_seconds_total` - Disk I/O
- `node_network_receive_bytes_total` - Network RX
- `node_network_transmit_bytes_total` - Network TX
- `node_filesystem_avail_bytes` - Disk space

**Container Metrics (cAdvisor):**
- `container_cpu_usage_seconds_total` - Container CPU
- `container_memory_usage_bytes` - Container memory
- `container_network_receive_bytes_total` - Container network RX
- `container_network_transmit_bytes_total` - Container network TX

### 3.2 Application Metrics

#### Rust Core Engine Metrics

**Implementation:**
```rust
use prometheus::{
    Counter, Histogram, IntGauge, Registry, Encoder, TextEncoder
};
use lazy_static::lazy_static;

lazy_static! {
    pub static ref REGISTRY: Registry = Registry::new();

    // HTTP metrics
    pub static ref HTTP_REQUESTS_TOTAL: Counter = Counter::new(
        "http_requests_total",
        "Total HTTP requests"
    ).unwrap();

    pub static ref HTTP_REQUEST_DURATION: Histogram = Histogram::with_opts(
        histogram_opts!(
            "http_request_duration_seconds",
            "HTTP request duration in seconds",
            vec![0.001, 0.005, 0.01, 0.05, 0.1, 0.5, 1.0, 2.0, 5.0]
        )
    ).unwrap();

    // Trading metrics
    pub static ref TRADES_TOTAL: Counter = Counter::new(
        "trades_total",
        "Total trades executed"
    ).unwrap();

    pub static ref TRADES_SUCCESS: Counter = Counter::new(
        "trades_success_total",
        "Successful trades"
    ).unwrap();

    pub static ref TRADES_FAILED: Counter = Counter::new(
        "trades_failed_total",
        "Failed trades"
    ).unwrap();

    pub static ref TRADE_DURATION: Histogram = Histogram::with_opts(
        histogram_opts!(
            "trade_execution_duration_seconds",
            "Trade execution duration",
            vec![0.1, 0.5, 1.0, 2.0, 5.0, 10.0]
        )
    ).unwrap();

    // WebSocket metrics
    pub static ref WS_CONNECTIONS: IntGauge = IntGauge::new(
        "websocket_connections_active",
        "Active WebSocket connections"
    ).unwrap();

    pub static ref WS_MESSAGES_SENT: Counter = Counter::new(
        "websocket_messages_sent_total",
        "WebSocket messages sent"
    ).unwrap();

    pub static ref WS_MESSAGES_RECEIVED: Counter = Counter::new(
        "websocket_messages_received_total",
        "WebSocket messages received"
    ).unwrap();

    // Database metrics
    pub static ref DB_QUERY_DURATION: Histogram = Histogram::with_opts(
        histogram_opts!(
            "db_query_duration_seconds",
            "Database query duration",
            vec![0.001, 0.005, 0.01, 0.05, 0.1, 0.5, 1.0]
        )
    ).unwrap();

    pub static ref DB_CONNECTIONS_ACTIVE: IntGauge = IntGauge::new(
        "db_connections_active",
        "Active database connections"
    ).unwrap();
}

pub fn register_metrics() {
    REGISTRY.register(Box::new(HTTP_REQUESTS_TOTAL.clone())).unwrap();
    REGISTRY.register(Box::new(HTTP_REQUEST_DURATION.clone())).unwrap();
    REGISTRY.register(Box::new(TRADES_TOTAL.clone())).unwrap();
    REGISTRY.register(Box::new(TRADES_SUCCESS.clone())).unwrap();
    REGISTRY.register(Box::new(TRADES_FAILED.clone())).unwrap();
    REGISTRY.register(Box::new(TRADE_DURATION.clone())).unwrap();
    REGISTRY.register(Box::new(WS_CONNECTIONS.clone())).unwrap();
    REGISTRY.register(Box::new(WS_MESSAGES_SENT.clone())).unwrap();
    REGISTRY.register(Box::new(WS_MESSAGES_RECEIVED.clone())).unwrap();
    REGISTRY.register(Box::new(DB_QUERY_DURATION.clone())).unwrap();
    REGISTRY.register(Box::new(DB_CONNECTIONS_ACTIVE.clone())).unwrap();
}

// Metrics endpoint
pub async fn metrics_handler() -> impl Responder {
    let encoder = TextEncoder::new();
    let metric_families = REGISTRY.gather();
    let mut buffer = vec![];
    encoder.encode(&metric_families, &mut buffer).unwrap();
    HttpResponse::Ok()
        .content_type("text/plain; charset=utf-8")
        .body(buffer)
}

// Usage
pub async fn execute_trade(trade: Trade) -> Result<(), Error> {
    let timer = TRADE_DURATION.start_timer();
    TRADES_TOTAL.inc();

    match perform_trade(trade).await {
        Ok(_) => {
            TRADES_SUCCESS.inc();
            timer.observe_duration();
            Ok(())
        }
        Err(e) => {
            TRADES_FAILED.inc();
            timer.observe_duration();
            Err(e)
        }
    }
}
```

#### Python AI Service Metrics

**Implementation:**
```python
from prometheus_client import Counter, Histogram, Gauge, generate_latest, CONTENT_TYPE_LATEST
from fastapi import Response

# Define metrics
http_requests_total = Counter(
    'http_requests_total',
    'Total HTTP requests',
    ['method', 'endpoint', 'status']
)

http_request_duration = Histogram(
    'http_request_duration_seconds',
    'HTTP request duration',
    ['method', 'endpoint'],
    buckets=[0.001, 0.005, 0.01, 0.05, 0.1, 0.5, 1.0, 2.0, 5.0]
)

ai_analysis_total = Counter(
    'ai_analysis_total',
    'Total AI analyses performed',
    ['symbol', 'model']
)

ai_analysis_duration = Histogram(
    'ai_analysis_duration_seconds',
    'AI analysis duration',
    ['model'],
    buckets=[0.1, 0.5, 1.0, 2.0, 5.0, 10.0]
)

ai_model_confidence = Histogram(
    'ai_model_confidence',
    'AI model confidence scores',
    ['symbol', 'signal'],
    buckets=[0.0, 0.1, 0.2, 0.3, 0.4, 0.5, 0.6, 0.7, 0.8, 0.9, 1.0]
)

ai_predictions_total = Counter(
    'ai_predictions_total',
    'Total predictions made',
    ['symbol', 'signal']
)

model_load_time = Gauge(
    'model_load_time_seconds',
    'Time to load AI model',
    ['model']
)

# Metrics endpoint
@app.get("/metrics")
async def metrics():
    return Response(
        content=generate_latest(),
        media_type=CONTENT_TYPE_LATEST
    )

# Middleware for request metrics
@app.middleware("http")
async def metrics_middleware(request: Request, call_next):
    start_time = time.time()
    response = await call_next(request)
    duration = time.time() - start_time

    http_requests_total.labels(
        method=request.method,
        endpoint=request.url.path,
        status=response.status_code
    ).inc()

    http_request_duration.labels(
        method=request.method,
        endpoint=request.url.path
    ).observe(duration)

    return response

# Usage
async def analyze_market(symbol: str):
    timer = ai_analysis_duration.labels(model="lstm").time()
    ai_analysis_total.labels(symbol=symbol, model="lstm").inc()

    with timer:
        result = await perform_analysis(symbol)

    ai_model_confidence.labels(
        symbol=symbol,
        signal=result.signal
    ).observe(result.confidence)

    ai_predictions_total.labels(
        symbol=symbol,
        signal=result.signal
    ).inc()

    return result
```

### 3.3 Business Metrics

**Trading Metrics:**
- `trades_total` - Total trades
- `trades_success_total` - Successful trades
- `trades_failed_total` - Failed trades
- `trade_volume_total` - Total trading volume
- `portfolio_value_usd` - Portfolio value in USD
- `profit_loss_total` - Total P&L

**User Metrics:**
- `users_active_total` - Active users
- `user_registrations_total` - New registrations
- `user_logins_total` - Login attempts
- `user_sessions_active` - Active sessions

**AI Metrics:**
- `ai_predictions_total` - Total predictions
- `ai_prediction_accuracy` - Prediction accuracy
- `ai_model_confidence` - Confidence scores
- `ai_signal_distribution` - Signal types (LONG/SHORT/NEUTRAL)

---

## 4. Prometheus Configuration

### 4.1 Prometheus Config

**File:** `infrastructure/monitoring/prometheus/prometheus.yml`

```yaml
global:
  scrape_interval: 15s
  evaluation_interval: 15s
  external_labels:
    cluster: 'bot-core-production'
    environment: 'production'

# Alertmanager configuration
alerting:
  alertmanagers:
    - static_configs:
        - targets:
            - 'alertmanager:9093'

# Load alerting rules
rule_files:
  - '/etc/prometheus/rules/*.yml'

# Scrape configurations
scrape_configs:
  # Prometheus itself
  - job_name: 'prometheus'
    static_configs:
      - targets: ['localhost:9090']

  # Node Exporter (system metrics)
  - job_name: 'node_exporter'
    static_configs:
      - targets: ['node_exporter:9100']

  # cAdvisor (container metrics)
  - job_name: 'cadvisor'
    static_configs:
      - targets: ['cadvisor:8080']

  # Rust Core Engine
  - job_name: 'rust-core-engine'
    static_configs:
      - targets: ['rust-core-engine:8080']
    metrics_path: '/metrics'
    scrape_interval: 10s

  # Python AI Service
  - job_name: 'python-ai-service'
    static_configs:
      - targets: ['python-ai-service:8000']
    metrics_path: '/metrics'
    scrape_interval: 10s

  # Frontend (if metrics enabled)
  - job_name: 'nextjs-ui-dashboard'
    static_configs:
      - targets: ['nextjs-ui-dashboard:3000']
    metrics_path: '/api/metrics'
    scrape_interval: 30s

  # MongoDB Exporter
  - job_name: 'mongodb'
    static_configs:
      - targets: ['mongodb-exporter:9216']

  # Redis Exporter
  - job_name: 'redis'
    static_configs:
      - targets: ['redis-exporter:9121']

  # Kubernetes pods (if using K8s)
  - job_name: 'kubernetes-pods'
    kubernetes_sd_configs:
      - role: pod
    relabel_configs:
      - source_labels: [__meta_kubernetes_pod_annotation_prometheus_io_scrape]
        action: keep
        regex: true
      - source_labels: [__meta_kubernetes_pod_annotation_prometheus_io_path]
        action: replace
        target_label: __metrics_path__
        regex: (.+)
      - source_labels: [__address__, __meta_kubernetes_pod_annotation_prometheus_io_port]
        action: replace
        regex: ([^:]+)(?::\d+)?;(\d+)
        replacement: $1:$2
        target_label: __address__
```

---

## 5. Grafana Dashboards

### 5.1 System Overview Dashboard

**Panels:**
1. **CPU Usage** (time series)
   ```promql
   100 - (avg by (instance) (rate(node_cpu_seconds_total{mode="idle"}[5m])) * 100)
   ```

2. **Memory Usage** (gauge)
   ```promql
   (node_memory_MemTotal_bytes - node_memory_MemAvailable_bytes) / node_memory_MemTotal_bytes * 100
   ```

3. **Disk Usage** (bar gauge)
   ```promql
   100 - ((node_filesystem_avail_bytes{mountpoint="/"} / node_filesystem_size_bytes{mountpoint="/"}) * 100)
   ```

4. **Network Traffic** (time series)
   ```promql
   rate(node_network_receive_bytes_total[5m])
   rate(node_network_transmit_bytes_total[5m])
   ```

### 5.2 Trading Performance Dashboard

**Panels:**
1. **Trades per Minute**
   ```promql
   rate(trades_total[1m])
   ```

2. **Trade Success Rate**
   ```promql
   (rate(trades_success_total[5m]) / rate(trades_total[5m])) * 100
   ```

3. **Trade Execution Latency (p50, p95, p99)**
   ```promql
   histogram_quantile(0.50, rate(trade_execution_duration_seconds_bucket[5m]))
   histogram_quantile(0.95, rate(trade_execution_duration_seconds_bucket[5m]))
   histogram_quantile(0.99, rate(trade_execution_duration_seconds_bucket[5m]))
   ```

4. **Portfolio Value**
   ```promql
   portfolio_value_usd
   ```

5. **Profit/Loss**
   ```promql
   profit_loss_total
   ```

### 5.3 AI Analysis Dashboard

**Panels:**
1. **AI Analyses per Minute**
   ```promql
   rate(ai_analysis_total[1m])
   ```

2. **AI Analysis Duration (avg)**
   ```promql
   rate(ai_analysis_duration_seconds_sum[5m]) / rate(ai_analysis_duration_seconds_count[5m])
   ```

3. **Model Confidence Distribution**
   ```promql
   histogram_quantile(0.50, rate(ai_model_confidence_bucket[5m]))
   ```

4. **Predictions by Signal**
   ```promql
   sum by (signal) (rate(ai_predictions_total[5m]))
   ```

5. **Prediction Accuracy**
   ```promql
   ai_prediction_accuracy
   ```

---

## 6. Alerting Rules

### 6.1 Alert Rules Configuration

**File:** `infrastructure/monitoring/prometheus/rules/alerts.yml`

```yaml
groups:
  - name: system_alerts
    interval: 30s
    rules:
      - alert: HighCPUUsage
        expr: 100 - (avg by (instance) (rate(node_cpu_seconds_total{mode="idle"}[5m])) * 100) > 80
        for: 5m
        labels:
          severity: warning
          component: system
        annotations:
          summary: "High CPU usage on {{ $labels.instance }}"
          description: "CPU usage is {{ $value }}%"

      - alert: HighMemoryUsage
        expr: (node_memory_MemTotal_bytes - node_memory_MemAvailable_bytes) / node_memory_MemTotal_bytes * 100 > 85
        for: 5m
        labels:
          severity: warning
          component: system
        annotations:
          summary: "High memory usage on {{ $labels.instance }}"
          description: "Memory usage is {{ $value }}%"

      - alert: DiskSpaceLow
        expr: (node_filesystem_avail_bytes{mountpoint="/"} / node_filesystem_size_bytes{mountpoint="/"}) * 100 < 15
        for: 5m
        labels:
          severity: warning
          component: system
        annotations:
          summary: "Low disk space on {{ $labels.instance }}"
          description: "Only {{ $value }}% disk space available"

  - name: application_alerts
    interval: 30s
    rules:
      - alert: HighErrorRate
        expr: rate(http_requests_total{status=~"5.."}[5m]) / rate(http_requests_total[5m]) > 0.05
        for: 5m
        labels:
          severity: critical
          component: application
        annotations:
          summary: "High error rate in {{ $labels.job }}"
          description: "Error rate is {{ $value | humanizePercentage }}"

      - alert: HighLatency
        expr: histogram_quantile(0.99, rate(http_request_duration_seconds_bucket[5m])) > 2
        for: 5m
        labels:
          severity: warning
          component: application
        annotations:
          summary: "High latency in {{ $labels.job }}"
          description: "P99 latency is {{ $value }}s"

      - alert: ServiceDown
        expr: up == 0
        for: 2m
        labels:
          severity: critical
          component: application
        annotations:
          summary: "Service {{ $labels.job }} is down"
          description: "{{ $labels.instance }} has been down for more than 2 minutes"

  - name: trading_alerts
    interval: 30s
    rules:
      - alert: HighTradeFailureRate
        expr: rate(trades_failed_total[5m]) / rate(trades_total[5m]) > 0.1
        for: 5m
        labels:
          severity: warning
          component: trading
        annotations:
          summary: "High trade failure rate"
          description: "{{ $value | humanizePercentage }} of trades are failing"

      - alert: TradingHalted
        expr: rate(trades_total[5m]) == 0
        for: 10m
        labels:
          severity: warning
          component: trading
        annotations:
          summary: "Trading activity has stopped"
          description: "No trades executed in the last 10 minutes"

  - name: ai_alerts
    interval: 30s
    rules:
      - alert: AIServiceSlow
        expr: histogram_quantile(0.95, rate(ai_analysis_duration_seconds_bucket[5m])) > 5
        for: 5m
        labels:
          severity: warning
          component: ai
        annotations:
          summary: "AI service is slow"
          description: "P95 analysis time is {{ $value }}s"

      - alert: LowModelConfidence
        expr: avg(ai_model_confidence) < 0.5
        for: 15m
        labels:
          severity: warning
          component: ai
        annotations:
          summary: "AI model confidence is low"
          description: "Average confidence is {{ $value }}"
```

### 6.2 Alertmanager Configuration

**File:** `infrastructure/monitoring/alertmanager/config.yml`

```yaml
global:
  resolve_timeout: 5m
  slack_api_url: 'https://hooks.slack.com/services/YOUR/SLACK/WEBHOOK'

route:
  group_by: ['alertname', 'cluster', 'service']
  group_wait: 10s
  group_interval: 10s
  repeat_interval: 12h
  receiver: 'default'
  routes:
    - match:
        severity: critical
      receiver: 'pagerduty'
      continue: true
    - match:
        severity: warning
      receiver: 'slack'
    - match:
        component: trading
      receiver: 'trading-team'

receivers:
  - name: 'default'
    slack_configs:
      - channel: '#bot-core-alerts'
        title: 'Alert: {{ .GroupLabels.alertname }}'
        text: '{{ range .Alerts }}{{ .Annotations.description }}{{ end }}'

  - name: 'pagerduty'
    pagerduty_configs:
      - service_key: 'YOUR_PAGERDUTY_KEY'
        description: '{{ .GroupLabels.alertname }}'

  - name: 'slack'
    slack_configs:
      - channel: '#bot-core-alerts'
        title: 'Warning: {{ .GroupLabels.alertname }}'
        text: '{{ range .Alerts }}{{ .Annotations.description }}{{ end }}'
        color: 'warning'

  - name: 'trading-team'
    slack_configs:
      - channel: '#trading-alerts'
        title: 'Trading Alert: {{ .GroupLabels.alertname }}'
        text: '{{ range .Alerts }}{{ .Annotations.description }}{{ end }}'

inhibit_rules:
  - source_match:
      severity: 'critical'
    target_match:
      severity: 'warning'
    equal: ['alertname', 'cluster', 'service']
```

---

## 7. Service Level Objectives

### 7.1 SLOs Definition

**API Availability:**
- **Objective:** 99.5% uptime
- **Measurement:** `(successful_requests / total_requests) * 100`
- **Error Budget:** 0.5% (3.6 hours/month)

**API Latency:**
- **Objective:** 95% of requests < 500ms
- **Measurement:** `histogram_quantile(0.95, http_request_duration_seconds)`

**Trade Execution Success:**
- **Objective:** 95% success rate
- **Measurement:** `(trades_success / trades_total) * 100`

### 7.2 SLO Monitoring Queries

```promql
# API Availability (last 30 days)
(sum(rate(http_requests_total{status!~"5.."}[30d])) / sum(rate(http_requests_total[30d]))) * 100

# API Latency P95
histogram_quantile(0.95, rate(http_request_duration_seconds_bucket[5m]))

# Trade Success Rate
(rate(trades_success_total[5m]) / rate(trades_total[5m])) * 100

# Error Budget Remaining
(1 - ((1 - (sum(rate(http_requests_total{status!~"5.."}[30d])) / sum(rate(http_requests_total[30d])))) / 0.005)) * 100
```

---

## Revision History

| Version | Date | Author | Changes |
|---------|------|--------|---------|
| 1.0.0 | 2025-10-11 | Operations Team | Initial version |

---

**Document End**
