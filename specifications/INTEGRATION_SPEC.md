# Integration Specification

## Overview
This document defines how different services integrate and communicate within the trading bot ecosystem.

## Service Integration Architecture

```
┌─────────────────┐     ┌──────────────────┐     ┌─────────────────┐
│                 │     │                  │     │                 │
│  Frontend       │────▶│  Rust Core       │────▶│  Python AI      │
│  Dashboard      │◀────│  Engine          │◀────│  Service        │
│  (Port 3000)    │     │  (Port 8080)     │     │  (Port 8000)    │
│                 │     │                  │     │                 │
└────────┬────────┘     └────────┬─────────┘     └────────┬────────┘
         │                       │                         │
         │                       │                         │
         └───────────────────────┴─────────────────────────┘
                                 │
                                 ▼
                        ┌─────────────────┐
                        │                 │
                        │  MongoDB Atlas  │
                        │  (Shared Data)  │
                        │                 │
                        └─────────────────┘
```

## Integration Patterns

### 1. Frontend → Rust Core Engine

#### Authentication Flow
```sequence
Frontend -> Rust: POST /api/auth/login
Rust -> MongoDB: Verify credentials
MongoDB -> Rust: User data
Rust -> Frontend: JWT token + user profile
Frontend -> Frontend: Store token in localStorage
```

#### Trading Flow
```sequence
Frontend -> Rust: GET /api/account (with JWT)
Rust -> Frontend: Account balance + positions
Frontend -> Rust: POST /api/trades/execute
Rust -> Python: GET /ai/analyze (internal)
Python -> Rust: AI signal + confidence
Rust -> Binance: Execute trade
Binance -> Rust: Order confirmation
Rust -> MongoDB: Store trade
Rust -> Frontend: Trade result
```

### 2. Rust Core Engine → Python AI Service

#### AI Analysis Request
```yaml
Trigger: User requests trade or automated analysis
Method: HTTP REST
Authentication: Internal service token
Retry: 3 times with exponential backoff
Timeout: 30 seconds
Fallback: Use cached analysis if available
```

**Request Flow:**
```python
# Rust sends to Python
POST http://python-ai-service:8000/ai/analyze
Headers:
  Authorization: Bearer {INTER_SERVICE_TOKEN}
  X-Request-ID: {uuid}
Body:
  {
    "symbol": "BTCUSDT",
    "timeframe": "1h",
    "candles": [...],
    "technical_indicators": {...}
  }

# Python processes and responds
Response:
  {
    "signal": "Long",
    "confidence": 0.75,
    "reasoning": "...",
    "processing_time_ms": 1234
  }
```

### 3. Python AI Service → MongoDB

#### Store Analysis Results
```yaml
Trigger: After each AI analysis
Connection: MongoDB Atlas via connection string
Collection: ai_analysis_results
Retention: 30 days
Index: symbol + timestamp (compound)
```

**Storage Flow:**
```javascript
// Python stores analysis
{
  _id: ObjectId("..."),
  symbol: "BTCUSDT",
  signal: "Long",
  confidence: 0.75,
  timestamp: ISODate("2025-07-31T18:33:29.169Z"),
  technical_indicators: {...},
  metadata: {
    model_version: "gpt-4o-mini",
    processing_time_ms: 1234
  }
}
```

### 4. WebSocket Integration

#### Real-time Updates Flow
```yaml
Frontend WebSocket: ws://localhost:3000/ws
Python AI WebSocket: ws://localhost:8000/ws
Rust Core WebSocket: ws://localhost:8080/ws
```

**Message Broadcasting:**
```typescript
// Python broadcasts AI signal
{
  "type": "ai_signal",
  "data": {
    "symbol": "BTCUSDT",
    "signal": "Long",
    "confidence": 0.75,
    "timestamp": "2025-07-31T18:33:29.169Z"
  }
}

// Rust forwards to Frontend
{
  "type": "trade_signal",
  "data": {
    "symbol": "BTCUSDT",
    "action": "BUY",
    "price": 45000.00,
    "ai_confidence": 0.75
  }
}
```

## Service Dependencies

### 1. Startup Order
```yaml
1. MongoDB: Must be accessible
2. Python AI Service: Independent startup
3. Rust Core Engine: Depends on MongoDB
4. Frontend Dashboard: Depends on Rust Core
```

### 2. Health Check Chain
```yaml
Frontend checks:
  - Own health
  - Rust Core health
  
Rust Core checks:
  - Own health
  - MongoDB connection
  - Python AI availability
  - Binance WebSocket

Python AI checks:
  - Own health
  - MongoDB connection
  - OpenAI API availability
```

## Data Synchronization

### 1. Account Data Sync
```yaml
Source of Truth: Binance API
Cache Layer: Rust Core Engine (5 minute TTL)
Update Trigger: 
  - Every 30 seconds
  - After each trade
  - On user request
```

### 2. Price Data Sync
```yaml
Source: Binance WebSocket
Distribution:
  - Rust Core: Primary receiver
  - Python AI: Via REST request
  - Frontend: Via WebSocket from Rust
Update Frequency: Real-time (100ms)
```

### 3. AI Analysis Sync
```yaml
Generation: Python AI Service
Storage: MongoDB Atlas
Cache: 
  - Python: 5 minutes in-memory
  - Rust: 2 minutes in-memory
Invalidation: On new market data
```

## Error Handling & Resilience

### 1. Service Unavailability

#### Python AI Service Down
```yaml
Detection: Health check failure (3 consecutive)
Rust Action:
  - Use cached AI signals (up to 15 minutes old)
  - Fallback to technical-only strategies
  - Notify user of degraded mode
  - Log incident
Recovery: Automatic retry every 30 seconds
```

#### MongoDB Down
```yaml
Detection: Connection timeout
Service Actions:
  - Python: Queue analyses in memory (max 1000)
  - Rust: Continue trading with cached data
  - Block new user registrations
Recovery: Bulk insert queued data on reconnection
```

### 2. Network Failures

#### Inter-Service Communication
```yaml
Timeout: 30 seconds
Retry Strategy:
  - Attempt 1: Immediate
  - Attempt 2: After 1 second
  - Attempt 3: After 3 seconds
Circuit Breaker:
  - Open after 5 failures
  - Half-open after 30 seconds
  - Close after 3 successes
```

### 3. Data Consistency

#### Trade Execution
```yaml
Transaction Flow:
  1. Rust validates trade locally
  2. Rust sends to Binance
  3. On success: Store in MongoDB
  4. On failure: Rollback and notify
  
Consistency Check:
  - Every 5 minutes
  - Compare Binance vs MongoDB
  - Auto-reconcile discrepancies
```

## Security Integration

### 1. Service Authentication

#### Internal Service Communication
```yaml
Method: Bearer token (shared secret)
Token: INTER_SERVICE_TOKEN from environment
Validation: Every request
Rotation: Monthly (manual)
```

#### External API Authentication
```yaml
Frontend -> Rust: JWT (24 hour expiry)
Rust -> Binance: API key + secret
Python -> OpenAI: API key
All -> MongoDB: Connection string with credentials
```

### 2. Data Encryption

#### In Transit
```yaml
Internal: HTTP (within Docker network)
External: HTTPS required
WebSocket: WSS for production
Database: TLS 1.2+ for MongoDB Atlas
```

#### At Rest
```yaml
MongoDB: Encrypted at rest (Atlas)
Logs: No sensitive data logged
Secrets: Environment variables only
```

## Monitoring & Observability

### 1. Distributed Tracing
```yaml
Trace ID: Generated at edge (Frontend/API)
Propagation: X-Request-ID header
Services: All must forward trace ID
Storage: Logs include trace ID
```

### 2. Metrics Collection
```yaml
Rust Core:
  - Request rate
  - Response time
  - Error rate
  - Active positions
  - PnL

Python AI:
  - AI requests/minute
  - Model confidence distribution
  - Cache hit rate
  - OpenAI API latency

Frontend:
  - Page load time
  - WebSocket reconnections
  - User actions/minute
```

### 3. Log Aggregation
```yaml
Format: JSON structured logs
Fields:
  - timestamp
  - service
  - level
  - message
  - trace_id
  - user_id (if applicable)
  - extra_data

Centralization: 
  - Development: Docker logs
  - Production: CloudWatch/Datadog
```

## Deployment Integration

### 1. Environment Configuration
```yaml
Development:
  - Docker Compose
  - Hot reload enabled
  - Debug logging
  - Testnet APIs

Production:
  - Kubernetes/ECS
  - Health checks required
  - Info logging
  - Mainnet APIs (when enabled)
```

### 2. Service Discovery
```yaml
Development:
  - Hard-coded service names
  - Docker network DNS

Production:
  - Service mesh (Istio/Consul)
  - Load balancer endpoints
  - Auto-scaling aware
```

### 3. Database Migration
```yaml
Strategy: 
  - Python runs migrations on startup
  - Rust validates schema
  - Backward compatible changes only

Rollback:
  - Keep 3 previous versions
  - Test rollback in staging
```

## Integration Testing

### 1. End-to-End Test Scenarios

#### Scenario: Complete Trade Flow
```yaml
Steps:
  1. Frontend requests AI analysis
  2. Rust forwards to Python
  3. Python analyzes and responds
  4. Rust executes trade
  5. All services update state
  6. Frontend receives confirmation

Validations:
  - Response times < 5 seconds
  - Data consistency across services
  - Proper error handling
  - WebSocket updates sent
```

### 2. Integration Test Environment
```yaml
Setup:
  - Isolated Docker network
  - Test MongoDB instance
  - Mock Binance API
  - Seeded test data

Execution:
  - Run on every PR
  - Nightly full suite
  - Performance benchmarks
```

## Performance Optimization

### 1. Caching Strategy
```yaml
Python AI:
  - Analysis results: 5 minutes
  - Technical indicators: 1 minute
  - Market data: 30 seconds

Rust Core:
  - Account data: 5 minutes
  - Position data: 30 seconds
  - Order book: Real-time

Frontend:
  - Static assets: 1 year
  - API responses: 10 seconds
  - WebSocket data: No cache
```

### 2. Load Balancing
```yaml
Python AI Service:
  - Horizontal scaling (3 instances)
  - Round-robin distribution
  - Sticky sessions for WebSocket

Rust Core Engine:
  - Vertical scaling preferred
  - Single instance per region
  - State shared via MongoDB
```

### 3. Resource Limits
```yaml
Development:
  - Python: 1.5GB RAM, 1.5 CPU
  - Rust: 1GB RAM, 1 CPU
  - Frontend: 512MB RAM, 0.5 CPU

Production:
  - Python: 4GB RAM, 2 CPU
  - Rust: 2GB RAM, 2 CPU
  - Frontend: 1GB RAM, 1 CPU
  - Auto-scale at 70% usage
```