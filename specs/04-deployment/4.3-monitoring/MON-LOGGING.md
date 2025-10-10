# Logging Strategy Specification

**Document Version:** 1.0.0
**Last Updated:** 2025-10-11
**Status:** Active
**Owner:** Operations Team

---

## Table of Contents

- [1. Overview](#1-overview)
- [2. Logging Architecture](#2-logging-architecture)
- [3. Logging Standards](#3-logging-standards)
- [4. Service-Specific Logging](#4-service-specific-logging)
- [5. Log Aggregation](#5-log-aggregation)
- [6. Log Retention](#6-log-retention)
- [7. Sensitive Data Handling](#7-sensitive-data-handling)

---

## 1. Overview

### 1.1 Purpose

Define logging standards, formats, and infrastructure for the Bot Core platform to ensure consistent, searchable, and actionable logs.

### 1.2 Logging Goals

- Structured logging (JSON format)
- Centralized log aggregation
- Fast log search and analysis
- Compliance with data retention policies
- Sensitive data protection

---

## 2. Logging Architecture

```
┌─────────────────────────────────────────────────────────┐
│                Application Logs                          │
├─────────────────────────────────────────────────────────┤
│                                                           │
│  ┌─────────┐  ┌─────────┐  ┌─────────┐                 │
│  │Frontend │  │  Rust   │  │ Python  │                 │
│  │  Logs   │  │  Logs   │  │  Logs   │                 │
│  └────┬────┘  └────┬────┘  └────┬────┘                 │
│       │            │            │                        │
│       └────────────┼────────────┘                        │
│                    │                                      │
│            ┌───────▼────────┐                           │
│            │  Fluentd/      │                           │
│            │  Filebeat      │                           │
│            │  (Collector)   │                           │
│            └───────┬────────┘                           │
│                    │                                      │
│            ┌───────▼────────┐                           │
│            │  Elasticsearch │                           │
│            │  (Storage)     │                           │
│            └───────┬────────┘                           │
│                    │                                      │
│            ┌───────▼────────┐                           │
│            │    Kibana      │                           │
│            │  (Visualization)│                           │
│            └────────────────┘                           │
│                                                           │
└─────────────────────────────────────────────────────────┘
```

---

## 3. Logging Standards

### 3.1 Log Levels

| Level | Usage | Examples |
|-------|-------|----------|
| TRACE | Very detailed debugging | Function entry/exit, variable values |
| DEBUG | Debugging information | Query parameters, internal state |
| INFO | General information | Service started, request received |
| WARN | Warning conditions | Deprecated API used, retry attempt |
| ERROR | Error conditions | Request failed, exception caught |
| FATAL | Critical failures | Service crash, data corruption |

### 3.2 Structured Log Format

**Standard JSON Format:**
```json
{
  "timestamp": "2025-10-11T10:30:00.123Z",
  "level": "INFO",
  "service": "rust-core-engine",
  "version": "1.0.0",
  "environment": "production",
  "trace_id": "abc123def456",
  "span_id": "789ghi",
  "message": "Trade executed successfully",
  "context": {
    "user_id": "507f1f77bcf86cd799439011",
    "trade_id": "60a1f5e5e5f5e5f5e5f5e5f5",
    "symbol": "BTCUSDT",
    "quantity": 0.01,
    "price": 45000.00
  },
  "duration_ms": 125,
  "host": "rust-core-engine-7d8f9c-xyz"
}
```

### 3.3 Required Fields

**Mandatory fields for all logs:**
- `timestamp`: ISO 8601 format with milliseconds
- `level`: Log level (TRACE, DEBUG, INFO, WARN, ERROR, FATAL)
- `service`: Service name
- `message`: Human-readable message
- `host`: Container/pod name

**Recommended fields:**
- `version`: Service version
- `environment`: dev/staging/production
- `trace_id`: Distributed tracing ID
- `span_id`: Span ID for tracing
- `user_id`: User identifier (if applicable)
- `request_id`: Unique request identifier

---

## 4. Service-Specific Logging

### 4.1 Rust Core Engine Logging

**Configuration:** `rust-core-engine/src/logging.rs`

**Library:** `tracing` + `tracing-subscriber`

**Implementation:**
```rust
use tracing::{info, error, debug, warn, instrument};
use tracing_subscriber::fmt::json;

// Initialize logger
pub fn init_logger() {
    tracing_subscriber::fmt()
        .json()
        .with_env_filter(std::env::var("RUST_LOG").unwrap_or_else(|_| "info".to_string()))
        .with_current_span(true)
        .with_span_list(true)
        .with_target(true)
        .with_thread_ids(true)
        .with_thread_names(true)
        .init();
}

// Usage examples
#[instrument(skip(trade))]
pub async fn execute_trade(trade: &Trade) -> Result<(), TradeError> {
    info!(
        trade_id = %trade.id,
        symbol = %trade.symbol,
        quantity = trade.quantity,
        "Executing trade"
    );

    match binance_client.execute(&trade).await {
        Ok(result) => {
            info!(
                trade_id = %trade.id,
                execution_price = result.price,
                duration_ms = result.duration,
                "Trade executed successfully"
            );
            Ok(())
        }
        Err(e) => {
            error!(
                trade_id = %trade.id,
                error = ?e,
                "Trade execution failed"
            );
            Err(e)
        }
    }
}

// Log with context
debug!(
    user_id = %user_id,
    portfolio_value = portfolio.total_value,
    "Portfolio updated"
);

// Log errors with full context
error!(
    error = ?err,
    context = ?err_context,
    "Failed to connect to Binance API"
);
```

**Log Configuration:**
```toml
# rust-core-engine/Cargo.toml
[dependencies]
tracing = "0.1"
tracing-subscriber = { version = "0.3", features = ["json", "env-filter"] }
tracing-opentelemetry = "0.21"
```

**Environment Variables:**
```bash
RUST_LOG=info                    # Production
RUST_LOG=debug                   # Development
RUST_LOG=rust_core_engine=debug  # Service-specific
RUST_BACKTRACE=1                 # Include backtraces
```

### 4.2 Python AI Service Logging

**Configuration:** `python-ai-service/config.yaml`

**Library:** `loguru`

**Implementation:**
```python
from loguru import logger
import sys
import json

# Configure structured logging
def init_logger():
    logger.remove()  # Remove default handler

    # Add JSON structured logging
    logger.add(
        sys.stdout,
        format=json_formatter,
        level="INFO",
        serialize=True,
        enqueue=True,
        backtrace=True,
        diagnose=True
    )

    # Add file logging with rotation
    logger.add(
        "logs/trading_ai.log",
        rotation="10 MB",
        retention="7 days",
        compression="zip",
        format=json_formatter,
        level="DEBUG"
    )

def json_formatter(record):
    """Custom JSON formatter"""
    log_record = {
        "timestamp": record["time"].isoformat(),
        "level": record["level"].name,
        "service": "python-ai-service",
        "message": record["message"],
        "function": record["function"],
        "file": record["file"].name,
        "line": record["line"],
        "context": record.get("extra", {})
    }
    return json.dumps(log_record)

# Usage examples
logger.info(
    "AI analysis completed",
    symbol="BTCUSDT",
    confidence=0.87,
    signal="LONG",
    duration_ms=245
)

logger.warning(
    "Model confidence below threshold",
    symbol="ETHUSDT",
    confidence=0.42,
    threshold=0.45
)

logger.error(
    "Failed to load AI model",
    model_path="/app/models/lstm-v2.pkl",
    error=str(e)
)

# Context manager for request logging
@logger.catch
async def analyze_market(symbol: str):
    with logger.contextualize(
        symbol=symbol,
        user_id=user_id,
        request_id=request_id
    ):
        logger.info("Starting market analysis")
        result = await perform_analysis(symbol)
        logger.info("Analysis complete", confidence=result.confidence)
        return result
```

**Configuration File:**
```yaml
# python-ai-service/config.yaml
logging:
  level: "INFO"
  format: "{time:YYYY-MM-DD HH:mm:ss.SSS} | {level} | {name}:{function}:{line} | {message}"
  file: "./logs/trading_ai.log"
  rotation: "10 MB"
  retention: "7 days"
  compression: "zip"
  enqueue: true
  backtrace: true
  diagnose: true
```

### 4.3 Frontend Logging

**Configuration:** `nextjs-ui-dashboard/src/utils/logger.ts`

**Implementation:**
```typescript
// logger.ts
interface LogContext {
  [key: string]: any;
}

class Logger {
  private service = 'nextjs-ui-dashboard';
  private environment = process.env.NODE_ENV || 'development';
  private version = process.env.VITE_APP_VERSION || '1.0.0';

  private formatLog(level: string, message: string, context?: LogContext) {
    return {
      timestamp: new Date().toISOString(),
      level,
      service: this.service,
      version: this.version,
      environment: this.environment,
      message,
      context,
      userAgent: navigator.userAgent,
      url: window.location.href,
    };
  }

  private sendLog(logData: any) {
    // Send to backend logging service
    if (this.environment === 'production') {
      fetch('/api/logs', {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify(logData),
      }).catch(err => console.error('Failed to send log:', err));
    }

    // Also log to console in development
    if (this.environment === 'development') {
      console.log(JSON.stringify(logData, null, 2));
    }
  }

  info(message: string, context?: LogContext) {
    const log = this.formatLog('INFO', message, context);
    this.sendLog(log);
  }

  warn(message: string, context?: LogContext) {
    const log = this.formatLog('WARN', message, context);
    this.sendLog(log);
  }

  error(message: string, error?: Error, context?: LogContext) {
    const log = this.formatLog('ERROR', message, {
      ...context,
      error: error?.message,
      stack: error?.stack,
    });
    this.sendLog(log);
  }

  debug(message: string, context?: LogContext) {
    if (this.environment === 'development') {
      const log = this.formatLog('DEBUG', message, context);
      this.sendLog(log);
    }
  }
}

export const logger = new Logger();

// Usage examples
logger.info('User logged in', { userId: user.id, email: user.email });

logger.warn('Trade execution delayed', {
  tradeId: trade.id,
  delayMs: 5000,
});

logger.error('Failed to fetch market data', error, {
  symbol: 'BTCUSDT',
  endpoint: '/api/market-data',
});

logger.debug('WebSocket message received', {
  messageType: 'trade_update',
  payload: data,
});
```

---

## 5. Log Aggregation

### 5.1 ELK Stack (Elasticsearch, Logstash, Kibana)

**Docker Compose Configuration:**
```yaml
# infrastructure/monitoring/docker-compose-logging.yml
version: '3.8'

services:
  elasticsearch:
    image: docker.elastic.co/elasticsearch/elasticsearch:8.10.0
    environment:
      - discovery.type=single-node
      - "ES_JAVA_OPTS=-Xms2g -Xmx2g"
    ports:
      - "9200:9200"
    volumes:
      - elasticsearch_data:/usr/share/elasticsearch/data
    networks:
      - logging

  logstash:
    image: docker.elastic.co/logstash/logstash:8.10.0
    volumes:
      - ./logstash/pipeline:/usr/share/logstash/pipeline
    ports:
      - "5000:5000/tcp"
      - "5000:5000/udp"
      - "9600:9600"
    environment:
      LS_JAVA_OPTS: "-Xms512m -Xmx512m"
    networks:
      - logging
    depends_on:
      - elasticsearch

  kibana:
    image: docker.elastic.co/kibana/kibana:8.10.0
    ports:
      - "5601:5601"
    environment:
      ELASTICSEARCH_HOSTS: "http://elasticsearch:9200"
    networks:
      - logging
    depends_on:
      - elasticsearch

  filebeat:
    image: docker.elastic.co/beats/filebeat:8.10.0
    user: root
    volumes:
      - ./filebeat/filebeat.yml:/usr/share/filebeat/filebeat.yml:ro
      - /var/lib/docker/containers:/var/lib/docker/containers:ro
      - /var/run/docker.sock:/var/run/docker.sock:ro
    networks:
      - logging
    depends_on:
      - elasticsearch
      - logstash

volumes:
  elasticsearch_data:

networks:
  logging:
    driver: bridge
```

**Filebeat Configuration:**
```yaml
# infrastructure/monitoring/filebeat/filebeat.yml
filebeat.inputs:
- type: container
  paths:
    - '/var/lib/docker/containers/*/*.log'
  processors:
    - add_docker_metadata:
        host: "unix:///var/run/docker.sock"
    - decode_json_fields:
        fields: ["message"]
        target: ""
        overwrite_keys: true

output.elasticsearch:
  hosts: ["elasticsearch:9200"]
  indices:
    - index: "rust-core-engine-%{+yyyy.MM.dd}"
      when.contains:
        container.name: "rust-core-engine"
    - index: "python-ai-service-%{+yyyy.MM.dd}"
      when.contains:
        container.name: "python-ai-service"
    - index: "nextjs-ui-dashboard-%{+yyyy.MM.dd}"
      when.contains:
        container.name: "nextjs-ui-dashboard"

setup.kibana:
  host: "kibana:5601"

logging.level: info
logging.to_files: true
logging.files:
  path: /var/log/filebeat
  name: filebeat
  keepfiles: 7
  permissions: 0644
```

### 5.2 Loki (Lightweight Alternative)

**Docker Compose:**
```yaml
services:
  loki:
    image: grafana/loki:2.9.0
    ports:
      - "3100:3100"
    volumes:
      - ./loki/loki-config.yml:/etc/loki/local-config.yaml
      - loki_data:/loki
    command: -config.file=/etc/loki/local-config.yaml
    networks:
      - logging

  promtail:
    image: grafana/promtail:2.9.0
    volumes:
      - ./promtail/promtail-config.yml:/etc/promtail/config.yml
      - /var/lib/docker/containers:/var/lib/docker/containers:ro
      - /var/run/docker.sock:/var/run/docker.sock
    command: -config.file=/etc/promtail/config.yml
    networks:
      - logging
    depends_on:
      - loki

  grafana:
    image: grafana/grafana:10.1.0
    ports:
      - "3001:3000"
    environment:
      - GF_AUTH_ANONYMOUS_ENABLED=true
      - GF_AUTH_ANONYMOUS_ORG_ROLE=Viewer
    volumes:
      - grafana_data:/var/lib/grafana
      - ./grafana/datasources:/etc/grafana/provisioning/datasources
    networks:
      - logging
    depends_on:
      - loki
```

---

## 6. Log Retention

### 6.1 Retention Policies

| Environment | Hot Storage | Warm Storage | Cold Storage | Archive |
|------------|-------------|--------------|--------------|---------|
| Development | 7 days | N/A | N/A | N/A |
| Staging | 30 days | N/A | N/A | N/A |
| Production | 30 days | 60 days | 90 days | 7 years |

### 6.2 Index Lifecycle Management

**Elasticsearch ILM Policy:**
```json
{
  "policy": "bot-core-logs",
  "phases": {
    "hot": {
      "min_age": "0ms",
      "actions": {
        "rollover": {
          "max_size": "50GB",
          "max_age": "1d"
        },
        "set_priority": {
          "priority": 100
        }
      }
    },
    "warm": {
      "min_age": "30d",
      "actions": {
        "shrink": {
          "number_of_shards": 1
        },
        "forcemerge": {
          "max_num_segments": 1
        },
        "set_priority": {
          "priority": 50
        }
      }
    },
    "cold": {
      "min_age": "60d",
      "actions": {
        "searchable_snapshot": {
          "snapshot_repository": "s3_repository"
        },
        "set_priority": {
          "priority": 0
        }
      }
    },
    "delete": {
      "min_age": "90d",
      "actions": {
        "delete": {}
      }
    }
  }
}
```

---

## 7. Sensitive Data Handling

### 7.1 Data Masking Rules

**Never log:**
- Passwords
- API keys (except last 4 characters)
- JWT tokens (except last 4 characters)
- Credit card numbers
- Social security numbers
- Private keys

**Masking Implementation:**
```rust
// Rust example
pub fn mask_api_key(api_key: &str) -> String {
    if api_key.len() <= 4 {
        return "***".to_string();
    }
    let last_four = &api_key[api_key.len()-4..];
    format!("***{}", last_four)
}

// Usage
info!(
    api_key = %mask_api_key(&api_key),
    "API key validated"
);
```

```python
# Python example
def mask_sensitive_data(value: str, show_chars: int = 4) -> str:
    """Mask sensitive data, showing only last N characters"""
    if len(value) <= show_chars:
        return "***"
    return f"***{value[-show_chars:]}"

# Usage
logger.info(
    "JWT token validated",
    token=mask_sensitive_data(token),
    user_id=user_id
)
```

### 7.2 PII Handling

**Personal Identifiable Information:**
- Hash user emails
- Mask IP addresses
- Anonymize location data

**Example:**
```python
import hashlib

def hash_pii(data: str) -> str:
    """Hash PII for logging"""
    return hashlib.sha256(data.encode()).hexdigest()[:16]

logger.info(
    "User registered",
    email_hash=hash_pii(user.email),
    ip_address=mask_ip(request.ip)
)
```

---

## Appendix: Log Queries

### Common Kibana/Elasticsearch Queries

**Find all errors in last hour:**
```
level: ERROR AND timestamp: [now-1h TO now]
```

**Find failed trades:**
```
service: rust-core-engine AND message: "Trade execution failed"
```

**Find slow requests (>1s):**
```
duration_ms: >1000 AND timestamp: [now-24h TO now]
```

**Group errors by service:**
```
{
  "aggs": {
    "errors_by_service": {
      "terms": {
        "field": "service.keyword",
        "size": 10
      }
    }
  },
  "query": {
    "bool": {
      "must": [
        { "match": { "level": "ERROR" } },
        { "range": { "timestamp": { "gte": "now-24h" } } }
      ]
    }
  }
}
```

---

## Revision History

| Version | Date | Author | Changes |
|---------|------|--------|---------|
| 1.0.0 | 2025-10-11 | Operations Team | Initial version |

---

**Document End**
