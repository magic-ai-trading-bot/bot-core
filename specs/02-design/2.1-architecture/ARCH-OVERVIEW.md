# System Architecture Overview

**Spec ID**: ARCH-OVERVIEW-001 to ARCH-OVERVIEW-004
**Version**: 1.0
**Status**: ✅ Approved
**Owner**: Architecture Team
**Last Updated**: 2025-10-10

---

## Tasks Checklist

- [x] Requirements gathered
- [x] Design completed
- [x] Implementation done
- [x] Tests written
- [ ] Documentation updated
- [ ] Code reviewed
- [ ] Deployed to staging
- [ ] Deployed to production

---

## Metadata

**Related Specs**:
- Related FR: [FR-TRADING-001](../../01-requirements/1.1-functional/FR-TRADING.md)
- Related Design: [ARCH-MICROSERVICES-001](./ARCH-MICROSERVICES.md)
- Related Tests: [TC-INTEGRATION-001](../../03-testing/3.2-integration/TC-INTEGRATION.md)

**Dependencies**:
- Depends on: [DATA-MODELS](../../DATA_MODELS.md)
- Blocks: [DEPLOYMENT-SPEC](../../04-deployment/DEPLOYMENT-SPEC.md)

**Business Value**: High
**Technical Complexity**: High
**Priority**: ☑ Critical

---

## Overview

This document provides a comprehensive overview of the Bot Core cryptocurrency trading platform's system architecture. The platform is built on a microservices architecture pattern using Rust for high-performance trading execution, Python for AI/ML analysis, and React for the user interface. The system is designed for scalability, reliability, and real-time performance in cryptocurrency trading operations.

---

## Business Context

**Problem Statement**:
The cryptocurrency trading market requires a robust, high-performance platform capable of executing trades in milliseconds while providing intelligent AI-driven market analysis and a user-friendly interface for monitoring and control. Traditional monolithic architectures cannot meet the demands for scalability, real-time processing, and independent service deployment.

**Business Goals**:
- Provide sub-second trade execution latency
- Enable AI-powered trading decisions with confidence scoring
- Support multiple concurrent users with real-time updates
- Ensure 99.9% uptime for trading operations
- Allow independent scaling of AI, trading, and UI components

**Success Metrics**:
- Trade execution time: < 500ms (Target: < 200ms)
- AI analysis latency: < 2 seconds (Target: < 1 second)
- System uptime: > 99.9% (Target: 99.99%)
- Concurrent WebSocket connections: > 100 (Target: > 500)
- API response time: < 100ms (Target: < 50ms)

---

## ARCH-OVERVIEW-001: System Architecture High-Level View

**Priority**: ☑ Critical
**Status**: ☑ Completed
**Code Tags**: `@spec:ARCH-OVERVIEW-001`

### Description

The Bot Core platform follows a **3-tier microservices architecture** pattern with clear separation of concerns:

1. **Presentation Layer**: React-based frontend dashboard
2. **Application Layer**: Rust Core Engine and Python AI Service
3. **Data Layer**: MongoDB Atlas database

### Architecture Diagram

```mermaid
graph TB
    subgraph "Presentation Layer - Port 3000"
        FE[Next.js Frontend Dashboard<br/>React 18 + TypeScript + Vite<br/>Shadcn/UI Components]
    end

    subgraph "Application Layer"
        subgraph "Rust Core Engine - Port 8080"
            RUST[Rust Trading Engine<br/>Actix-web + Tokio<br/>JWT Authentication]
            WS[WebSocket Server<br/>Real-time Updates]
            RISK[Risk Manager<br/>Position Manager]
            STRAT[Strategy Engine<br/>RSI, MACD, Bollinger]
        end

        subgraph "Python AI Service - Port 8000"
            PY[Python AI Service<br/>FastAPI + TensorFlow<br/>PyTorch + OpenAI]
            ML[ML Models<br/>LSTM, GRU, Transformer]
            TA[Technical Analyzer<br/>Indicators Calculator]
        end
    end

    subgraph "Data Layer"
        DB[(MongoDB Atlas<br/>Primary Database)]
        CACHE[Redis Cache<br/>Optional]
    end

    subgraph "External Services"
        BINANCE[Binance API<br/>WebSocket + REST]
        OPENAI[OpenAI GPT-4<br/>AI Analysis]
    end

    FE -->|REST API<br/>WebSocket| RUST
    RUST -->|HTTP REST| PY
    PY -->|HTTP REST| OPENAI
    RUST -->|MongoDB Driver| DB
    PY -->|Motor Driver| DB
    RUST -->|WebSocket<br/>REST API| BINANCE
    RUST -.->|Optional| CACHE
    PY -.->|Optional| CACHE

    style FE fill:#4A90E2
    style RUST fill:#E27C4A
    style PY fill:#4AE290
    style DB fill:#E2C44A
    style BINANCE fill:#9B59B6
    style OPENAI fill:#E74C3C
```

### Component Responsibilities

**Frontend Dashboard (Port 3000)**:
- User authentication and session management
- Real-time market data visualization
- Trading interface and controls
- AI signal display and analysis results
- Portfolio and position monitoring

**Rust Core Engine (Port 8080)**:
- High-performance trade execution
- WebSocket connection management
- Risk and position management
- Strategy execution orchestration
- User authentication (JWT)
- Market data processing

**Python AI Service (Port 8000)**:
- Machine learning model inference
- Technical indicator calculation
- OpenAI GPT-4 integration
- AI signal generation
- Market condition analysis

**MongoDB Atlas**:
- User accounts and authentication
- Trade history and positions
- Market data persistence
- AI analysis cache
- Configuration storage

### Acceptance Criteria

- [x] All three tiers are independently deployable
- [x] Services communicate via well-defined REST APIs
- [x] Real-time updates use WebSocket protocol
- [x] Database is shared across services with proper isolation
- [x] External API integrations are fault-tolerant

### Dependencies
- Related: [ARCH-MICROSERVICES-001](./ARCH-MICROSERVICES.md)

### Test Cases
- TC-ARCH-001: Verify 3-tier separation
- TC-ARCH-002: Test service independence
- TC-ARCH-003: Validate communication protocols

---

## ARCH-OVERVIEW-002: Technology Stack

**Priority**: ☑ Critical
**Status**: ☑ Completed
**Code Tags**: `@spec:ARCH-OVERVIEW-002`

### Description

The Bot Core platform uses a modern, high-performance technology stack optimized for real-time cryptocurrency trading:

### Technology Stack Diagram

```mermaid
graph LR
    subgraph "Frontend Stack"
        REACT[React 18.3.1]
        TS[TypeScript 5.x]
        VITE[Vite 6.0.1]
        SHADCN[Shadcn/UI]
        RECHARTS[Recharts 2.15.0]
        AXIOS[Axios 1.7.9]
    end

    subgraph "Backend Stack - Rust"
        RUST[Rust 1.86+]
        ACTIX[Actix-web 4.0]
        TOKIO[Tokio 1.0<br/>Async Runtime]
        SERDE[Serde 1.0<br/>Serialization]
        MONGODB_RUST[MongoDB Driver 3.3]
        REQWEST[Reqwest 0.11<br/>HTTP Client]
        WARP[Warp 0.3<br/>WebSocket]
        JWT_RUST[JWT 9.1]
    end

    subgraph "Backend Stack - Python"
        PYTHON[Python 3.11]
        FASTAPI[FastAPI 0.115.6]
        TENSORFLOW[TensorFlow 2.18.0]
        PYTORCH[PyTorch 2.5.1]
        OPENAI_SDK[OpenAI SDK 1.59.5]
        MOTOR[Motor 3.6.0<br/>Async MongoDB]
        PANDAS[Pandas 2.2.3]
        NUMPY[NumPy 2.1.3]
    end

    subgraph "Database & Cache"
        MONGO[MongoDB 6.0+]
        REDIS_OPT[Redis 7<br/>Optional]
    end

    subgraph "Infrastructure"
        DOCKER[Docker 24.0+]
        COMPOSE[Docker Compose 2.0+]
        K8S[Kubernetes<br/>Optional]
    end

    style RUST fill:#E27C4A
    style PYTHON fill:#4AE290
    style MONGO fill:#E2C44A
    style DOCKER fill:#2496ED
```

### Detailed Technology Specifications

#### Frontend Technologies

| Technology | Version | Purpose | Code Location |
|------------|---------|---------|---------------|
| React | 18.3.1 | UI Framework | `nextjs-ui-dashboard/` |
| TypeScript | 5.x | Type Safety | All `.ts/.tsx` files |
| Vite | 6.0.1 | Build Tool | `vite.config.ts` |
| Shadcn/UI | Latest | Component Library | `src/components/ui/` |
| Recharts | 2.15.0 | Data Visualization | `src/components/dashboard/` |
| Axios | 1.7.9 | HTTP Client | `src/services/api.ts` |
| i18next | 23.16.8 | Internationalization | `src/i18n/` |
| React Router | 7.1.1 | Routing | `src/main.tsx` |
| TailwindCSS | 3.4.17 | Styling | `tailwind.config.ts` |

#### Backend Technologies - Rust

| Technology | Version | Purpose | Code Location |
|------------|---------|---------|---------------|
| Rust | 1.86+ | Core Language | `rust-core-engine/` |
| Actix-web | 4.0 | Web Framework | `src/api/mod.rs` |
| Tokio | 1.0 | Async Runtime | Throughout |
| Serde | 1.0 | Serialization | Data models |
| MongoDB | 3.3 | Database Driver | `src/storage/` |
| Reqwest | 0.11 | HTTP Client | `src/binance/client.rs` |
| Warp | 0.3 | WebSocket | `src/binance/websocket.rs` |
| JWT | 9.1 | Authentication | `src/auth/jwt.rs` |
| BCrypt | 0.15 | Password Hashing | `src/auth/handlers.rs` |
| Rust Decimal | 1.33 | Precision Math | Trading calculations |

#### Backend Technologies - Python

| Technology | Version | Purpose | Code Location |
|------------|---------|---------|---------------|
| Python | 3.11 | Core Language | `python-ai-service/` |
| FastAPI | 0.115.6 | Web Framework | `main.py` |
| TensorFlow | 2.18.0 | Deep Learning | `models/lstm_model.py` |
| PyTorch | 2.5.1 | Deep Learning | `models/transformer_model.py` |
| OpenAI | 1.59.5 | GPT-4 Integration | `main.py` (DirectOpenAIClient) |
| Motor | 3.6.0 | Async MongoDB | Database operations |
| Pandas | 2.2.3 | Data Analysis | Feature engineering |
| NumPy | 2.1.3 | Numerical Computing | Calculations |
| TA-Lib | Latest | Technical Analysis | `features/technical_indicators.py` |
| Slowapi | 0.1.9 | Rate Limiting | `main.py` |

#### Database Technologies

| Technology | Version | Purpose | Configuration |
|------------|---------|---------|---------------|
| MongoDB Atlas | 6.0+ | Primary Database | Connection string in `.env` |
| Redis | 7-alpine | Cache (Optional) | Port 6379, Docker Compose |

#### Infrastructure Technologies

| Technology | Version | Purpose | Configuration |
|------------|---------|---------|---------------|
| Docker | 24.0+ | Containerization | `Dockerfile` in each service |
| Docker Compose | 2.0+ | Orchestration | `docker-compose.yml` |
| Kubernetes | Optional | Production Orchestration | Future enhancement |

### Technology Selection Rationale

**Rust for Trading Engine**:
- **Performance**: Zero-cost abstractions, no garbage collection
- **Safety**: Memory safety without runtime overhead
- **Concurrency**: Tokio async runtime for high-throughput operations
- **Reliability**: Strong type system prevents common bugs

**Python for AI Service**:
- **ML Ecosystem**: TensorFlow, PyTorch, extensive libraries
- **Rapid Development**: Quick iteration on ML models
- **OpenAI Integration**: Native SDK support
- **Data Processing**: Pandas, NumPy for efficient data manipulation

**React for Frontend**:
- **Component-Based**: Reusable UI components
- **Performance**: Virtual DOM for efficient updates
- **Ecosystem**: Rich library ecosystem (Shadcn, Recharts)
- **TypeScript**: Type safety in frontend code

**MongoDB for Database**:
- **Flexibility**: Schema-less design for evolving data models
- **Performance**: Horizontal scaling with sharding
- **JSON-Native**: Natural fit for REST APIs
- **Atlas**: Managed service reduces operational overhead

### Acceptance Criteria

- [x] All technologies are production-grade and actively maintained
- [x] Version compatibility is verified across services
- [x] Dependencies are locked to specific versions
- [x] Technology choices align with performance requirements
- [x] All services use async/await for non-blocking I/O

### Dependencies
- Related: [DEPLOYMENT-SPEC](../../04-deployment/DEPLOYMENT-SPEC.md)

### Test Cases
- TC-TECH-001: Verify version compatibility
- TC-TECH-002: Test async performance
- TC-TECH-003: Validate technology stack integration

---

## ARCH-OVERVIEW-003: Service Communication

**Priority**: ☑ Critical
**Status**: ☑ Completed
**Code Tags**: `@spec:ARCH-OVERVIEW-003`

### Description

The Bot Core platform uses multiple communication protocols optimized for different use cases:

### Communication Architecture Diagram

```mermaid
sequenceDiagram
    participant User
    participant Frontend as Frontend<br/>(React)
    participant Rust as Rust Core<br/>(Port 8080)
    participant Python as Python AI<br/>(Port 8000)
    participant MongoDB as MongoDB<br/>Atlas
    participant Binance as Binance<br/>API
    participant OpenAI as OpenAI<br/>GPT-4

    %% User Authentication
    User->>Frontend: Login Request
    Frontend->>Rust: POST /api/auth/login
    Rust->>MongoDB: Verify Credentials
    MongoDB-->>Rust: User Data
    Rust-->>Frontend: JWT Token
    Frontend->>Frontend: Store Token

    %% WebSocket Connection
    Frontend->>Rust: WebSocket Connect
    Rust-->>Frontend: Connection Established

    %% AI Analysis Request
    User->>Frontend: Request AI Analysis
    Frontend->>Rust: POST /api/ai/analyze (JWT)
    Rust->>Python: POST /ai/analyze (Inter-Service Token)
    Python->>MongoDB: Check Cache
    MongoDB-->>Python: Cache Miss
    Python->>OpenAI: GPT-4 Analysis Request
    OpenAI-->>Python: Analysis Result
    Python->>MongoDB: Store Result
    Python-->>Rust: AI Signal Response
    Rust-->>Frontend: Trading Signal

    %% Real-time Price Updates
    Binance->>Rust: WebSocket Price Update
    Rust->>MongoDB: Store Market Data
    Rust->>Frontend: WebSocket Broadcast
    Frontend->>User: Update UI

    %% Trade Execution
    User->>Frontend: Execute Trade
    Frontend->>Rust: POST /api/trades/execute (JWT)
    Rust->>Rust: Risk Validation
    Rust->>Binance: Place Order
    Binance-->>Rust: Order Confirmation
    Rust->>MongoDB: Store Trade
    Rust->>Frontend: WebSocket Trade Update
    Frontend->>User: Confirmation
```

### Communication Protocols

#### 1. REST APIs (HTTP/HTTPS)

**Frontend ↔ Rust Core Engine**:
- **Protocol**: HTTPS (production), HTTP (development)
- **Authentication**: JWT Bearer Token
- **Data Format**: JSON
- **Base URL**: `http://localhost:8080/api`
- **Endpoints**:
  - `POST /api/auth/login` - User authentication
  - `POST /api/auth/register` - User registration
  - `GET /api/account` - Account information
  - `POST /api/trades/execute` - Execute trade
  - `GET /api/positions` - Get positions
  - `GET /api/trades/history` - Trade history
  - `GET /api/market/data` - Market data
- **Timeout**: 10 seconds
- **Retry Strategy**: 3 retries with exponential backoff
- **Rate Limiting**: 100 requests/minute per user

**Rust Core Engine ↔ Python AI Service**:
- **Protocol**: HTTP (internal Docker network)
- **Authentication**: Inter-Service Token (Bearer)
- **Data Format**: JSON
- **Base URL**: `http://python-ai-service:8000`
- **Endpoints**:
  - `POST /ai/analyze` - AI signal analysis
  - `POST /ai/strategy-recommendations` - Strategy recommendations
  - `POST /ai/market-condition` - Market condition analysis
  - `GET /ai/info` - Service information
  - `GET /health` - Health check
- **Timeout**: 30 seconds
- **Retry Strategy**: 3 retries with circuit breaker
- **Rate Limiting**: No limit (internal service)

#### 2. WebSocket Connections

**Frontend ↔ Rust Core Engine**:
- **Protocol**: WSS (production), WS (development)
- **URL**: `ws://localhost:8080/ws`
- **Authentication**: JWT token in connection params
- **Message Format**: JSON
- **Heartbeat**: 30 seconds
- **Reconnection**: Automatic with exponential backoff
- **Message Types**:
  ```typescript
  {
    "type": "PriceUpdate",
    "data": {
      "symbol": "BTCUSDT",
      "price": 50000.00,
      "timestamp": 1697831609000
    }
  }
  ```
  ```typescript
  {
    "type": "TradeUpdate",
    "data": {
      "trade_id": "abc123",
      "status": "filled",
      "price": 50000.00
    }
  }
  ```
  ```typescript
  {
    "type": "AISignalReceived",
    "data": {
      "symbol": "BTCUSDT",
      "signal": "long",
      "confidence": 0.75
    }
  }
  ```

**Binance API ↔ Rust Core Engine**:
- **Protocol**: WSS (secure WebSocket)
- **URL**: `wss://stream.binance.com:9443/ws`
- **Streams**:
  - `btcusdt@kline_1m` - 1-minute candlestick
  - `btcusdt@ticker` - 24hr ticker
  - `btcusdt@depth` - Order book depth
  - `btcusdt@trade` - Real-time trades
- **Reconnection**: Automatic with backoff
- **Data Processing**: High-frequency (100ms intervals)

**Python AI Service ↔ Clients**:
- **Protocol**: WS
- **URL**: `ws://localhost:8000/ws`
- **Purpose**: AI signal broadcasting
- **Message Format**: JSON
- **Broadcast**: All connected clients receive AI signals

#### 3. Database Communication

**MongoDB Atlas Access**:
- **Driver**:
  - Rust: `mongodb` crate (sync/async)
  - Python: `motor` (async) / `pymongo` (sync)
- **Connection String**: `mongodb+srv://user:pass@cluster.mongodb.net/db?retryWrites=true&w=majority`
- **TLS**: Required (1.2+)
- **Connection Pool**:
  - Rust: 10-50 connections
  - Python: 10-100 connections
- **Collections**:
  - `users` - User accounts
  - `trades` - Trade history
  - `positions` - Active positions
  - `market_data` - Market data cache
  - `ai_analysis_results` - AI analysis cache

#### 4. External API Communication

**Binance REST API**:
- **Base URL**: `https://api.binance.com` (mainnet) / `https://testnet.binance.vision` (testnet)
- **Authentication**: HMAC SHA256 signature
- **Rate Limits**:
  - 1200 requests/minute (weight-based)
  - Order rate: 10 orders/second
- **Endpoints**:
  - `GET /api/v3/ticker/price` - Current price
  - `POST /api/v3/order` - Place order
  - `GET /api/v3/account` - Account information
  - `GET /api/v3/klines` - Candlestick data
- **Error Handling**: Retry on 429, fail on 4xx/5xx

**OpenAI API**:
- **Base URL**: `https://api.openai.com/v1`
- **Authentication**: Bearer token (API key)
- **Model**: `gpt-4o-mini`
- **Rate Limits**:
  - 20 requests/minute (configurable delay)
  - Automatic fallback to backup API keys
- **Endpoints**:
  - `POST /chat/completions` - GPT-4 analysis
- **Timeout**: 30 seconds
- **Retry Strategy**: Multiple API keys with fallback

### Service Discovery

**Development Environment**:
- **Method**: Hard-coded service names
- **DNS**: Docker Compose internal DNS
- **Configuration**:
  ```yaml
  RUST_API_URL: http://rust-core-engine:8080
  PYTHON_AI_URL: http://python-ai-service:8000
  ```

**Production Environment** (Future):
- **Method**: Service mesh (Istio/Consul)
- **Load Balancer**: Kubernetes Service
- **Configuration**: Dynamic service discovery

### Communication Security

**Authentication**:
- **User → Frontend → Rust**: JWT token (24-hour expiry)
- **Rust → Python**: Inter-service token (Bearer)
- **Rust/Python → MongoDB**: Connection string credentials
- **Rust → Binance**: API key + HMAC signature
- **Python → OpenAI**: API key (Bearer token)

**Encryption**:
- **External**: TLS 1.3 required
- **Internal**: HTTP (Docker network isolation)
- **Database**: TLS 1.2+ (MongoDB Atlas)
- **Secrets**: Environment variables, never hardcoded

### Communication Performance

| Route | Protocol | Avg Latency | Max Latency | Throughput |
|-------|----------|-------------|-------------|------------|
| Frontend → Rust | REST | 20ms | 50ms | 1000 req/s |
| Rust → Python | REST | 50ms | 200ms | 100 req/s |
| Rust → MongoDB | TCP | 5ms | 20ms | 10000 ops/s |
| Rust → Binance | HTTPS | 100ms | 500ms | 100 req/s |
| Python → OpenAI | HTTPS | 1000ms | 5000ms | 3 req/min |
| WebSocket Updates | WS | <10ms | 50ms | 1000 msg/s |

### Acceptance Criteria

- [x] All communication uses appropriate protocols
- [x] Authentication is enforced on all external endpoints
- [x] WebSocket connections support automatic reconnection
- [x] Circuit breakers prevent cascading failures
- [x] Rate limiting protects against abuse

### Dependencies
- Related: [ARCH-SECURITY-001](./ARCH-SECURITY.md)
- Related: [INTEGRATION_SPEC](../../INTEGRATION_SPEC.md)

### Test Cases
- TC-COMM-001: Test REST API communication
- TC-COMM-002: Verify WebSocket reconnection
- TC-COMM-003: Validate authentication flows

---

## ARCH-OVERVIEW-004: Deployment Architecture

**Priority**: ☑ High
**Status**: ☑ Completed
**Code Tags**: `@spec:ARCH-OVERVIEW-004`

### Description

The Bot Core platform supports multiple deployment configurations optimized for different environments:

### Deployment Architecture Diagram

```mermaid
graph TB
    subgraph "Development Environment"
        DEV_FE[Frontend Dev<br/>Bun + HMR<br/>768MB RAM]
        DEV_RUST[Rust Dev<br/>Cargo Watch<br/>1.5GB RAM]
        DEV_PY[Python Dev<br/>Uvicorn Reload<br/>1.5GB RAM]
        DEV_DB[(MongoDB<br/>Local/Atlas)]
    end

    subgraph "Production Environment (Docker Compose)"
        PROD_FE[Frontend<br/>Nginx Serve<br/>512MB RAM]
        PROD_RUST[Rust Core<br/>Release Build<br/>1GB RAM]
        PROD_PY[Python AI<br/>Gunicorn<br/>1.5GB RAM]
        PROD_DB[(MongoDB Atlas)]
        PROD_REDIS[Redis Cache<br/>Optional]
    end

    subgraph "Production Environment (Kubernetes - Future)"
        K8S_INGRESS[Ingress<br/>Load Balancer]
        K8S_FE[Frontend Pod<br/>Replicas: 2-5]
        K8S_RUST[Rust Pod<br/>Replicas: 2-10]
        K8S_PY[Python Pod<br/>Replicas: 3-10]
        K8S_DB[(MongoDB Atlas)]
        K8S_REDIS[Redis Cluster]
    end

    subgraph "External Services"
        EXT_BINANCE[Binance API]
        EXT_OPENAI[OpenAI API]
    end

    PROD_FE --> PROD_RUST
    PROD_RUST --> PROD_PY
    PROD_RUST --> PROD_DB
    PROD_PY --> PROD_DB
    PROD_RUST -.-> PROD_REDIS
    PROD_PY -.-> PROD_REDIS

    K8S_INGRESS --> K8S_FE
    K8S_FE --> K8S_RUST
    K8S_RUST --> K8S_PY
    K8S_RUST --> K8S_DB
    K8S_PY --> K8S_DB
    K8S_RUST --> K8S_REDIS
    K8S_PY --> K8S_REDIS

    PROD_RUST --> EXT_BINANCE
    PROD_PY --> EXT_OPENAI
    K8S_RUST --> EXT_BINANCE
    K8S_PY --> EXT_OPENAI

    style DEV_FE fill:#4A90E2
    style DEV_RUST fill:#E27C4A
    style DEV_PY fill:#4AE290
    style PROD_FE fill:#4A90E2
    style PROD_RUST fill:#E27C4A
    style PROD_PY fill:#4AE290
```

### Deployment Configurations

#### 1. Development Environment

**Technology**: Docker Compose with volume mounts

**Configuration**:
```yaml
# docker-compose.yml (dev profile)
services:
  nextjs-ui-dashboard-dev:
    build:
      dockerfile: Dockerfile.dev
    volumes:
      - ./nextjs-ui-dashboard/src:/app/src:delegated
    environment:
      - NODE_ENV=development
      - HMR_PORT=24678
    deploy:
      resources:
        limits:
          memory: 768M
          cpus: "1"

  rust-core-engine-dev:
    build:
      dockerfile: Dockerfile.dev
    volumes:
      - ./rust-core-engine/src:/app/src
      - rust_target_cache:/app/target
    environment:
      - RUST_LOG=debug
      - RUST_BACKTRACE=1
    deploy:
      resources:
        limits:
          memory: 1.5G
          cpus: "1.5"

  python-ai-service-dev:
    build:
      dockerfile: Dockerfile.dev
    volumes:
      - ./python-ai-service:/app
    command: ["uvicorn", "main:app", "--reload"]
    environment:
      - LOG_LEVEL=DEBUG
      - ENABLE_HOT_RELOAD=true
    deploy:
      resources:
        limits:
          memory: 1.5G
          cpus: "1.5"
```

**Features**:
- Hot module reloading (HMR) for frontend
- Cargo watch for Rust (manual restart recommended)
- Uvicorn auto-reload for Python
- Volume mounts for live code updates
- Debug logging enabled
- Lower resource limits

**Start Command**:
```bash
./scripts/bot.sh dev
# OR
docker compose --profile dev up
```

#### 2. Production Environment (Docker Compose)

**Technology**: Docker Compose with optimized builds

**Configuration**:
```yaml
# docker-compose.yml (prod profile)
services:
  nextjs-ui-dashboard:
    build:
      dockerfile: Dockerfile
    environment:
      - NODE_ENV=production
      - NODE_OPTIONS="--max-old-space-size=1024"
    deploy:
      resources:
        limits:
          memory: ${FRONTEND_MEMORY_LIMIT:-1G}
          cpus: "${FRONTEND_CPU_LIMIT:-1}"
        reservations:
          memory: 256M
          cpus: "0.5"
    healthcheck:
      test: ["CMD", "curl", "-f", "http://localhost:3000/health"]
      interval: 30s
      timeout: 10s
      retries: 3

  rust-core-engine:
    build:
      dockerfile: Dockerfile
    environment:
      - RUST_LOG=${RUST_LOG:-info}
    deploy:
      resources:
        limits:
          memory: ${RUST_MEMORY_LIMIT:-2G}
          cpus: "${RUST_CPU_LIMIT:-2}"
        reservations:
          memory: 1G
          cpus: "1"
    healthcheck:
      test: ["CMD", "curl", "-f", "http://localhost:8080/api/health"]
      interval: 30s
      timeout: 10s
      retries: 3

  python-ai-service:
    build:
      dockerfile: Dockerfile
    environment:
      - LOG_LEVEL=${LOG_LEVEL:-INFO}
    deploy:
      resources:
        limits:
          memory: ${PYTHON_MEMORY_LIMIT:-2G}
          cpus: "${PYTHON_CPU_LIMIT:-2}"
        reservations:
          memory: 1G
          cpus: "1"
    healthcheck:
      test: ["CMD", "curl", "-f", "http://localhost:8000/health"]
      interval: 30s
      timeout: 10s
      retries: 3
```

**Features**:
- Multi-stage Docker builds for smaller images
- Health checks for all services
- Resource limits and reservations
- Production logging (INFO level)
- Optimized builds (--release for Rust)

**Start Command**:
```bash
./scripts/bot.sh start --memory-optimized
# OR
docker compose --profile prod up -d
```

**Memory Optimization**:
- Python AI: 1.5GB limit (down from 2GB)
- Rust Core: 1GB limit (down from 2GB)
- Frontend: 512MB limit (down from 1GB)

#### 3. Production Environment (Kubernetes - Future)

**Technology**: Kubernetes with Helm charts

**Architecture**:
```yaml
# Conceptual K8s architecture
apiVersion: apps/v1
kind: Deployment
metadata:
  name: rust-core-engine
spec:
  replicas: 3
  selector:
    matchLabels:
      app: rust-core-engine
  template:
    spec:
      containers:
      - name: rust-core
        image: bot-core/rust-core-engine:latest
        resources:
          requests:
            memory: "1Gi"
            cpu: "1"
          limits:
            memory: "2Gi"
            cpu: "2"
        livenessProbe:
          httpGet:
            path: /api/health
            port: 8080
          initialDelaySeconds: 60
          periodSeconds: 30
        readinessProbe:
          httpGet:
            path: /api/health
            port: 8080
          initialDelaySeconds: 30
          periodSeconds: 10
---
apiVersion: autoscaling/v2
kind: HorizontalPodAutoscaler
metadata:
  name: rust-core-hpa
spec:
  scaleTargetRef:
    apiVersion: apps/v1
    kind: Deployment
    name: rust-core-engine
  minReplicas: 2
  maxReplicas: 10
  metrics:
  - type: Resource
    resource:
      name: cpu
      target:
        type: Utilization
        averageUtilization: 70
```

**Features**:
- Horizontal auto-scaling (HPA)
- Pod replicas for high availability
- Load balancing via Kubernetes Service
- Health checks (liveness/readiness probes)
- Rolling updates with zero downtime
- Resource quotas and limits
- Persistent volumes for logs

**Deployment Strategy**:
- Blue-green deployment for major releases
- Rolling updates for minor releases
- Canary deployment for A/B testing

#### 4. Service Discovery & Networking

**Docker Compose**:
- **Network**: `bot-network` (bridge driver)
- **Subnet**: `172.20.0.0/16`
- **DNS**: Docker Compose internal DNS
- **Service Names**:
  - `rust-core-engine` (or `rust-core-engine-dev`)
  - `python-ai-service` (or `python-ai-service-dev`)
  - `nextjs-ui-dashboard` (or `nextjs-ui-dashboard-dev`)

**Kubernetes**:
- **Service Discovery**: Kubernetes Service DNS
- **Load Balancing**: Kubernetes Service (ClusterIP)
- **Ingress**: Nginx Ingress Controller
- **Network Policy**: Restrict inter-pod communication

#### 5. High Availability Considerations

**Redundancy**:
- Multiple service replicas (K8s)
- Database replication (MongoDB Atlas)
- Multi-AZ deployment (Cloud)

**Failover**:
- Automatic pod restart on failure (K8s)
- Health check-based routing
- Circuit breakers in service communication

**Data Persistence**:
- MongoDB Atlas (managed replication)
- Persistent volumes for logs
- Redis for session persistence (optional)

**Disaster Recovery**:
- Database backups (MongoDB Atlas automated)
- Configuration backups (Git repository)
- Rollback capability (Docker tags, K8s rollout)

### Resource Requirements

#### Minimum Hardware (Development)

| Component | CPU | RAM | Storage | Network |
|-----------|-----|-----|---------|---------|
| Frontend | 1 core | 768MB | 1GB | 10 Mbps |
| Rust Core | 1.5 cores | 1.5GB | 2GB | 100 Mbps |
| Python AI | 1.5 cores | 1.5GB | 5GB | 100 Mbps |
| MongoDB | - | - | 10GB | - |
| **Total** | **4 cores** | **4GB** | **18GB** | **100 Mbps** |

#### Recommended Hardware (Production)

| Component | CPU | RAM | Storage | Network |
|-----------|-----|-----|---------|---------|
| Frontend | 1 core | 1GB | 2GB | 100 Mbps |
| Rust Core | 2 cores | 2GB | 5GB | 1 Gbps |
| Python AI | 2 cores | 2GB | 10GB | 1 Gbps |
| MongoDB | 2 cores | 4GB | 50GB | 1 Gbps |
| Redis | 1 core | 1GB | 5GB | 100 Mbps |
| **Total** | **8 cores** | **10GB** | **72GB** | **1 Gbps** |

#### Scalability Limits

| Metric | Development | Production | Maximum |
|--------|-------------|------------|---------|
| Concurrent Users | 10 | 100 | 1000+ (K8s) |
| Trades/Second | 1 | 10 | 100+ (K8s) |
| WebSocket Connections | 50 | 100 | 500+ (K8s) |
| API Requests/Min | 100 | 1000 | 10000+ (K8s) |
| AI Requests/Min | 5 | 10 | 30 (OpenAI limit) |

### Deployment Scripts

**Location**: `/Users/dungngo97/Documents/bot-core/scripts/bot.sh`

```bash
#!/usr/bin/env bash
# Main deployment script

# Start development environment
./scripts/bot.sh dev

# Start production environment
./scripts/bot.sh start

# Start with memory optimization
./scripts/bot.sh start --memory-optimized

# Stop all services
./scripts/bot.sh stop

# Clean volumes and restart
./scripts/bot.sh clean
./scripts/bot.sh start

# View logs
./scripts/bot.sh logs --service python-ai-service
./scripts/bot.sh logs --service rust-core-engine
./scripts/bot.sh logs --service nextjs-ui-dashboard

# Check status
./scripts/bot.sh status
```

### Acceptance Criteria

- [x] Docker Compose deployment supports dev and prod profiles
- [x] All services have health checks configured
- [x] Resource limits are enforced for production
- [x] Kubernetes manifests are documented (future enhancement)
- [x] Deployment scripts automate common operations
- [x] High availability architecture is documented

### Dependencies
- Related: [DEPLOYMENT-SPEC](../../04-deployment/DEPLOYMENT-SPEC.md)
- Related: [MONITORING-SPEC](../../05-operations/MONITORING-SPEC.md)

### Test Cases
- TC-DEPLOY-001: Test Docker Compose deployment
- TC-DEPLOY-002: Verify health checks work correctly
- TC-DEPLOY-003: Validate resource limits
- TC-DEPLOY-004: Test deployment script functionality

---

## Implementation Notes

**Code Locations**:
- Rust Core Engine: `/Users/dungngo97/Documents/bot-core/rust-core-engine/`
  - Main entry: `src/main.rs:44-169`
  - API server: `src/api/mod.rs`
  - WebSocket: `src/binance/websocket.rs`
- Python AI Service: `/Users/dungngo97/Documents/bot-core/python-ai-service/`
  - Main entry: `main.py:1-2087`
  - FastAPI app: `main.py:350-376`
  - WebSocket manager: `main.py:70-127`
- Frontend Dashboard: `/Users/dungngo97/Documents/bot-core/nextjs-ui-dashboard/`
  - Entry point: `src/main.tsx`
  - API client: `src/services/api.ts`
  - WebSocket hook: `src/hooks/useWebSocket.ts`
- Docker Orchestration: `/Users/dungngo97/Documents/bot-core/docker-compose.yml:1-460`
- Deployment Scripts: `/Users/dungngo97/Documents/bot-core/scripts/bot.sh`

**Design Patterns**:
- **Microservices**: Service independence and scalability
- **Event-Driven**: WebSocket for real-time updates
- **Repository Pattern**: Data access abstraction (MongoDB)
- **Circuit Breaker**: Fault tolerance in service communication
- **Pub/Sub**: WebSocket broadcasting for real-time events

**Configuration**:
- Environment Variables: `.env` file (from `config.env` template)
- Rust Config: `rust-core-engine/config.toml`
- Python Config: `python-ai-service/config.yaml`
- Frontend Config: Environment variables in `docker-compose.yml`

---

## Traceability

**Requirements**:
- User Story: [US-TRADING-001](../../01-requirements/1.2-user-stories/US-TRADING.md)
- Business Rule: [BUSINESS_RULES.md#trading-rules](../../BUSINESS_RULES.md)

**Design**:
- Architecture: [ARCH-MICROSERVICES-001](./ARCH-MICROSERVICES.md)
- API Spec: [API_SPEC.md](../../API_SPEC.md)
- Data Model: [DATA_MODELS.md](../../DATA_MODELS.md)

**Test Cases**:
- Unit: [TC-UNIT-001](../../03-testing/3.1-unit/TC-UNIT.md)
- Integration: [TC-INTEGRATION-001](../../03-testing/3.2-integration/TC-INTEGRATION.md)
- E2E: [TC-E2E-001](../../03-testing/3.3-e2e/TC-E2E.md)

---

## Risks & Mitigations

| Risk | Impact | Probability | Mitigation |
|------|--------|-------------|------------|
| Service communication failure | High | Medium | Circuit breakers, retry logic, fallback mechanisms |
| MongoDB connection loss | High | Low | Connection pooling, automatic reconnection, caching |
| Binance API rate limits | High | Medium | Rate limiting, request queuing, caching |
| OpenAI API failures | Medium | High | Multiple API keys, fallback to technical analysis |
| WebSocket disconnections | Medium | Medium | Automatic reconnection with exponential backoff |
| Resource exhaustion | High | Medium | Resource limits, auto-scaling, monitoring |
| Security vulnerabilities | High | Low | Regular updates, security scanning, HTTPS/TLS |

---

## Open Questions

- [x] Question 1: Should we implement Kubernetes deployment? (Resolution: Document for future, use Docker Compose now)
- [x] Question 2: Do we need Redis caching? (Resolution: Optional profile in Docker Compose)
- [x] Question 3: Should we use API Gateway (Kong)? (Resolution: Optional profile for advanced deployments)
- [ ] Question 4: What is the disaster recovery strategy? (Resolution needed by: 2025-11-01)

---

## Changelog

| Version | Date | Author | Changes |
|---------|------|--------|---------|
| 1.0 | 2025-10-10 | Architecture Team | Initial comprehensive architecture overview |

---

## Appendix

**References**:
- Docker Compose Documentation: https://docs.docker.com/compose/
- Kubernetes Documentation: https://kubernetes.io/docs/
- MongoDB Atlas: https://www.mongodb.com/atlas
- Binance API: https://binance-docs.github.io/apidocs/
- OpenAI API: https://platform.openai.com/docs/

**Glossary**:
- **Microservices**: Architectural style where application is composed of small, independent services
- **WebSocket**: Protocol for full-duplex communication over TCP
- **JWT**: JSON Web Token for stateless authentication
- **Circuit Breaker**: Design pattern to prevent cascading failures
- **HPA**: Horizontal Pod Autoscaler in Kubernetes
- **TLS**: Transport Layer Security for encrypted communication
- **HMR**: Hot Module Replacement for live code updates

**Diagrams Legend**:
- Blue: Frontend components
- Orange: Rust backend components
- Green: Python backend components
- Yellow: Database components
- Purple: External services
- Gray: Infrastructure components

---

**Remember**: Update TRACEABILITY_MATRIX.md when implementation is complete!
