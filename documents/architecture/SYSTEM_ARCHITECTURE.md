# System Architecture

## Overview

Bot Core is a microservices-based cryptocurrency trading platform designed for high performance, scalability, and reliability. This document provides a comprehensive view of the system architecture.

## High-Level Architecture

```mermaid
graph TB
    subgraph "Client Layer"
        WEB[Web Browser]
        MOBILE[Mobile App]
        API_CLIENT[API Clients]
    end

    subgraph "CDN & Gateway Layer"
        CDN[CloudFront CDN]
        KONG[Kong API Gateway]
    end

    subgraph "Service Mesh"
        ISTIO[Istio Service Mesh]
    end

    subgraph "Application Layer"
        RUST[Rust Core Engine<br/>Port 8080]
        PYTHON[Python AI Service<br/>Port 8000]
        NEXT[Next.js Dashboard<br/>Port 3000]
    end

    subgraph "Message Queue"
        RABBITMQ[RabbitMQ<br/>Port 5672]
    end

    subgraph "Data Layer"
        MONGO[(MongoDB<br/>Primary + Replicas)]
        REDIS[(Redis Cache<br/>Port 6379)]
    end

    subgraph "External Services"
        BINANCE[Binance API]
        OPENAI[OpenAI API]
    end

    subgraph "Monitoring"
        PROM[Prometheus]
        GRAF[Grafana]
    end

    WEB --> CDN
    MOBILE --> CDN
    API_CLIENT --> KONG
    CDN --> KONG
    KONG --> ISTIO

    ISTIO --> RUST
    ISTIO --> PYTHON
    ISTIO --> NEXT

    RUST --> MONGO
    RUST --> REDIS
    RUST --> RABBITMQ
    RUST --> BINANCE

    PYTHON --> MONGO
    PYTHON --> REDIS
    PYTHON --> RABBITMQ
    PYTHON --> OPENAI

    NEXT --> RUST

    RUST --> PROM
    PYTHON --> PROM
    PROM --> GRAF
```

## Component Architecture

### 1. Rust Core Engine

**Purpose**: High-performance trading execution and real-time market data processing

**Key Responsibilities**:
- Trading order execution
- WebSocket connections to exchanges
- Position management
- Risk management
- User authentication and authorization
- Paper trading simulation

**Technology Stack**:
- Language: Rust 1.75+
- Framework: Actix-web for REST API
- WebSocket: tokio-tungstenite
- Database: MongoDB (via mongodb crate)
- Caching: Redis (via redis-rs)

**Architecture Diagram**:
```mermaid
graph LR
    subgraph "Rust Core Engine"
        API[REST API Layer]
        WS[WebSocket Layer]
        AUTH[Auth Service]
        TRADE[Trading Service]
        RISK[Risk Manager]
        STRAT[Strategy Engine]
        PAPER[Paper Trading]
        DB[Database Layer]
    end

    API --> AUTH
    API --> TRADE
    WS --> TRADE
    TRADE --> RISK
    TRADE --> STRAT
    TRADE --> PAPER
    TRADE --> DB
    STRAT --> DB
```

**Key Features**:
- Sub-10ms order execution latency
- Concurrent WebSocket handling (10,000+ connections)
- Thread-safe state management
- Circuit breaker pattern for external APIs
- Rate limiting per user and global

### 2. Python AI Service

**Purpose**: AI-powered market analysis and trading signal generation

**Key Responsibilities**:
- GPT-4 market analysis
- ML model predictions (LSTM, GRU, Transformer)
- Technical indicator calculations
- Market sentiment analysis
- Strategy recommendations

**Technology Stack**:
- Language: Python 3.11+
- Framework: FastAPI
- ML Libraries: PyTorch, TensorFlow
- Technical Analysis: TA-Lib
- AI: OpenAI API, LangChain

**Architecture Diagram**:
```mermaid
graph LR
    subgraph "Python AI Service"
        FAPI[FastAPI Layer]
        GPT[GPT-4 Service]
        ML[ML Models]
        TA[TA-Lib Indicators]
        CACHE[Cache Manager]
        DB[MongoDB Client]
    end

    FAPI --> GPT
    FAPI --> ML
    FAPI --> TA
    FAPI --> CACHE
    GPT --> CACHE
    ML --> DB
```

**Key Features**:
- Asynchronous processing
- Response caching (5-minute TTL)
- Batch request optimization
- Fallback models for reliability
- Cost tracking and budget limits

### 3. Next.js Dashboard

**Purpose**: Modern web interface for trading and monitoring

**Key Responsibilities**:
- User interface for trading
- Real-time data visualization
- Portfolio management
- Settings and configuration
- Multi-language support (i18n)

**Technology Stack**:
- Framework: Next.js 14+ with App Router
- Build Tool: Vite
- UI Library: Shadcn/UI + Tailwind CSS
- Charts: TradingView Lightweight Charts, Three.js
- State Management: React Context + Hooks
- WebSocket: Socket.io client

**Architecture Diagram**:
```mermaid
graph LR
    subgraph "Next.js Dashboard"
        PAGES[Pages/Routes]
        COMP[Components]
        HOOKS[Custom Hooks]
        CTX[Context/State]
        API[API Client]
        WS[WebSocket Client]
    end

    PAGES --> COMP
    COMP --> HOOKS
    COMP --> CTX
    HOOKS --> API
    HOOKS --> WS
```

## Data Flow Architecture

### Trading Flow

```mermaid
sequenceDiagram
    participant User
    participant Dashboard
    participant Rust
    participant Python
    participant Binance
    participant MongoDB

    User->>Dashboard: View market data
    Dashboard->>Rust: WebSocket connection
    Rust->>Binance: Subscribe to market data
    Binance-->>Rust: Real-time prices
    Rust-->>Dashboard: Forward price updates

    User->>Dashboard: Request AI analysis
    Dashboard->>Python: POST /ai/analyze
    Python->>Python: Calculate indicators
    Python->>OpenAI: Request analysis
    OpenAI-->>Python: AI insights
    Python->>MongoDB: Store analysis
    Python-->>Dashboard: Return signal

    User->>Dashboard: Place trade
    Dashboard->>Rust: POST /api/trades/execute
    Rust->>Rust: Validate order
    Rust->>Rust: Check risk limits
    Rust->>Binance: Execute order
    Binance-->>Rust: Order confirmation
    Rust->>MongoDB: Store trade
    Rust->>RabbitMQ: Publish trade event
    Rust-->>Dashboard: Trade confirmed
```

### AI Analysis Flow

```mermaid
sequenceDiagram
    participant Rust
    participant Python
    participant OpenAI
    participant Redis
    participant MongoDB

    Rust->>Python: POST /ai/analyze
    Python->>Redis: Check cache
    alt Cache hit
        Redis-->>Python: Cached result
        Python-->>Rust: Return cached analysis
    else Cache miss
        Python->>Python: Calculate technical indicators
        Python->>OpenAI: Request GPT-4 analysis
        OpenAI-->>Python: AI response
        Python->>Python: Parse and validate
        Python->>Redis: Cache result (5 min TTL)
        Python->>MongoDB: Store analysis
        Python-->>Rust: Return analysis
    end
```

## Deployment Architecture

### Single Region Deployment

```mermaid
graph TB
    subgraph "AWS Region: us-east-1"
        subgraph "VPC"
            subgraph "Public Subnet"
                ALB[Application Load Balancer]
                NAT[NAT Gateway]
            end

            subgraph "Private Subnet 1"
                EKS1[EKS Node 1]
                RUST1[Rust Pod]
                PYTHON1[Python Pod]
                NEXT1[Next.js Pod]
            end

            subgraph "Private Subnet 2"
                EKS2[EKS Node 2]
                RUST2[Rust Pod]
                PYTHON2[Python Pod]
                NEXT2[Next.js Pod]
            end

            subgraph "Data Subnet"
                RDS[(RDS MongoDB)]
                ELASTICACHE[(ElastiCache Redis)]
            end
        end
    end

    INTERNET[Internet] --> ALB
    ALB --> EKS1
    ALB --> EKS2
    EKS1 --> RDS
    EKS2 --> RDS
    EKS1 --> ELASTICACHE
    EKS2 --> ELASTICACHE
    EKS1 --> NAT
    EKS2 --> NAT
```

### Multi-Region Deployment

```mermaid
graph TB
    subgraph "Global"
        R53[Route 53]
        CF[CloudFront]
    end

    subgraph "Region 1: us-east-1"
        ALB1[ALB]
        EKS1[EKS Cluster]
        DB1[(MongoDB Primary)]
    end

    subgraph "Region 2: eu-west-1"
        ALB2[ALB]
        EKS2[EKS Cluster]
        DB2[(MongoDB Secondary)]
    end

    subgraph "Region 3: ap-southeast-1"
        ALB3[ALB]
        EKS3[EKS Cluster]
        DB3[(MongoDB Secondary)]
    end

    R53 --> CF
    CF --> ALB1
    CF --> ALB2
    CF --> ALB3

    DB1 -.Replication.-> DB2
    DB1 -.Replication.-> DB3
```

## Network Architecture

### Docker Network (Development)

```mermaid
graph LR
    subgraph "Docker Network: bot-network"
        RUST[rust-core-engine:8080]
        PYTHON[python-ai-service:8000]
        NEXT[nextjs-ui:3000]
        MONGO[(MongoDB:27017)]
        REDIS[(Redis:6379)]
        RABBIT[(RabbitMQ:5672)]
    end

    HOST[Host Machine]

    HOST -->|3000| NEXT
    HOST -->|8000| PYTHON
    HOST -->|8080| RUST

    NEXT --> RUST
    RUST --> PYTHON
    RUST --> MONGO
    RUST --> REDIS
    RUST --> RABBIT
    PYTHON --> MONGO
    PYTHON --> REDIS
    PYTHON --> RABBIT
```

### Kubernetes Network

```mermaid
graph TB
    subgraph "Kubernetes Cluster"
        subgraph "Istio Service Mesh"
            INGRESS[Istio Ingress Gateway]
        end

        subgraph "Namespace: bot-core"
            RUST_SVC[rust-core-service]
            PYTHON_SVC[python-ai-service]
            NEXT_SVC[nextjs-service]

            RUST_POD[Rust Pods]
            PYTHON_POD[Python Pods]
            NEXT_POD[Next.js Pods]
        end

        subgraph "Namespace: data"
            MONGO_SVC[MongoDB Service]
            REDIS_SVC[Redis Service]
        end
    end

    INGRESS --> RUST_SVC
    INGRESS --> PYTHON_SVC
    INGRESS --> NEXT_SVC

    RUST_SVC --> RUST_POD
    PYTHON_SVC --> PYTHON_POD
    NEXT_SVC --> NEXT_POD

    RUST_POD --> MONGO_SVC
    RUST_POD --> REDIS_SVC
    PYTHON_POD --> MONGO_SVC
    PYTHON_POD --> REDIS_SVC
```

## Security Architecture

### Authentication Flow

```mermaid
sequenceDiagram
    participant User
    participant Dashboard
    participant Rust
    participant MongoDB

    User->>Dashboard: Enter credentials
    Dashboard->>Rust: POST /api/auth/login
    Rust->>MongoDB: Verify credentials
    MongoDB-->>Rust: User data
    Rust->>Rust: Generate JWT token
    Rust-->>Dashboard: Return JWT + Refresh token
    Dashboard->>Dashboard: Store tokens

    Note over Dashboard,Rust: Subsequent requests
    Dashboard->>Rust: API request + JWT
    Rust->>Rust: Validate JWT
    alt Valid token
        Rust-->>Dashboard: Authorized response
    else Invalid token
        Rust-->>Dashboard: 401 Unauthorized
        Dashboard->>Dashboard: Clear tokens
        Dashboard-->>User: Redirect to login
    end
```

### Inter-Service Authentication

```mermaid
graph LR
    subgraph "Service Communication"
        RUST[Rust Core]
        PYTHON[Python AI]
    end

    subgraph "Security"
        JWT[JWT Token Validation]
        MTLS[mTLS Certificates]
        ISTIO[Istio Service Mesh]
    end

    RUST -->|HTTP + JWT| PYTHON
    JWT -.Validates.-> RUST
    JWT -.Validates.-> PYTHON
    ISTIO -->|mTLS| RUST
    ISTIO -->|mTLS| PYTHON
```

## Scalability Architecture

### Horizontal Scaling

```mermaid
graph TB
    subgraph "Auto Scaling Group"
        HPA[Horizontal Pod Autoscaler]
    end

    subgraph "Service Pods"
        R1[Rust Pod 1]
        R2[Rust Pod 2]
        R3[Rust Pod 3]
        Rn[Rust Pod N]
    end

    subgraph "Load Balancer"
        LB[Service Load Balancer]
    end

    HPA -->|Scale Up/Down| R1
    HPA -->|Scale Up/Down| R2
    HPA -->|Scale Up/Down| R3
    HPA -->|Scale Up/Down| Rn

    LB --> R1
    LB --> R2
    LB --> R3
    LB --> Rn
```

**Scaling Metrics**:
- CPU utilization > 70%: Scale up
- Memory utilization > 80%: Scale up
- Request rate > 1000 req/s: Scale up
- CPU utilization < 30% for 5 minutes: Scale down

### Database Scaling

```mermaid
graph LR
    subgraph "MongoDB Replica Set"
        PRIMARY[(Primary)]
        SEC1[(Secondary 1)]
        SEC2[(Secondary 2)]
        ARB[Arbiter]
    end

    WRITE[Write Operations] --> PRIMARY
    PRIMARY -.Replication.-> SEC1
    PRIMARY -.Replication.-> SEC2

    READ1[Read Operations] --> PRIMARY
    READ2[Read Operations] --> SEC1
    READ3[Read Operations] --> SEC2

    ARB -.Election Voting.-> PRIMARY
```

## Performance Optimization

### Caching Strategy

```mermaid
graph LR
    CLIENT[Client Request]
    CDN[CDN Cache<br/>Static Assets]
    REDIS[Redis Cache<br/>API Responses]
    APP[Application]
    DB[(Database)]

    CLIENT --> CDN
    CDN -->|Cache Miss| REDIS
    REDIS -->|Cache Miss| APP
    APP --> DB
    DB --> APP
    APP --> REDIS
    REDIS --> CDN
    CDN --> CLIENT
```

**Cache TTLs**:
- Static assets: 1 week
- API responses: 5 minutes
- Market data: 1 second
- User sessions: 24 hours
- AI analysis: 5 minutes

### Connection Pooling

- **MongoDB**: 100 connections per service
- **Redis**: 50 connections per service
- **HTTP**: Keep-alive with max 1000 connections

## Disaster Recovery

### Backup Strategy

```mermaid
graph LR
    subgraph "Primary Database"
        MONGO[(MongoDB Primary)]
    end

    subgraph "Backups"
        HOURLY[(Hourly Snapshots<br/>24 hours)]
        DAILY[(Daily Snapshots<br/>30 days)]
        WEEKLY[(Weekly Snapshots<br/>1 year)]
    end

    subgraph "Storage"
        S3[(S3 Bucket<br/>us-east-1)]
        GCS[(GCS Bucket<br/>eu-west-1)]
    end

    MONGO -->|Every Hour| HOURLY
    MONGO -->|Every Day| DAILY
    MONGO -->|Every Week| WEEKLY

    HOURLY --> S3
    DAILY --> S3
    WEEKLY --> S3

    S3 -.Replication.-> GCS
```

### Failover Architecture

```mermaid
graph TB
    PRIMARY[Primary Region<br/>us-east-1]
    SECONDARY[Secondary Region<br/>eu-west-1]
    HEALTH[Health Checks]
    DNS[Route 53]

    HEALTH -->|Healthy| PRIMARY
    HEALTH -->|Unhealthy| SECONDARY
    DNS -->|Active| PRIMARY
    DNS -->|Failover| SECONDARY

    PRIMARY -.Continuous Replication.-> SECONDARY
```

## Monitoring Architecture

```mermaid
graph TB
    subgraph "Services"
        RUST[Rust Core]
        PYTHON[Python AI]
        NEXT[Next.js]
    end

    subgraph "Metrics Collection"
        PROM[Prometheus]
    end

    subgraph "Visualization"
        GRAF[Grafana Dashboards]
    end

    subgraph "Alerting"
        ALERT[Alert Manager]
        SLACK[Slack]
        EMAIL[Email]
    end

    RUST -->|/metrics| PROM
    PYTHON -->|/metrics| PROM
    NEXT -->|/metrics| PROM

    PROM --> GRAF
    PROM --> ALERT
    ALERT --> SLACK
    ALERT --> EMAIL
```

## Technology Stack Summary

| Component | Technology | Version |
|-----------|-----------|---------|
| Rust Core Engine | Rust, Actix-web | 1.75+ |
| Python AI Service | Python, FastAPI | 3.11+ |
| Frontend Dashboard | Next.js, React | 14+ |
| Database | MongoDB | 7.0+ |
| Cache | Redis | 7.2+ |
| Message Queue | RabbitMQ | 3.12+ |
| API Gateway | Kong | 3.4+ |
| Service Mesh | Istio | 1.19+ |
| Container Runtime | Docker | 24.0+ |
| Orchestration | Kubernetes | 1.28+ |
| Monitoring | Prometheus + Grafana | Latest |
| CI/CD | GitHub Actions | - |
| Cloud Provider | AWS (Primary), GCP (Secondary) | - |

## Architectural Decisions

### Why Microservices?

1. **Language Optimization**: Use best language for each task
   - Rust for performance-critical trading
   - Python for AI/ML workloads
   - TypeScript/React for modern UI

2. **Independent Scaling**: Scale services based on demand
3. **Fault Isolation**: Service failures don't cascade
4. **Team Autonomy**: Teams work independently

### Why Rust for Core Engine?

- Sub-millisecond execution latency
- Memory safety without garbage collection
- Excellent concurrency support
- Zero-cost abstractions

### Why Python for AI Service?

- Rich ML/AI ecosystem
- Easy integration with OpenAI
- Fast prototyping
- Extensive libraries (TA-Lib, NumPy, PyTorch)

### Why MongoDB?

- Flexible schema for trading data
- Excellent scaling capabilities
- Native JSON support
- Change streams for real-time updates
- Built-in replication

### Why Redis?

- In-memory speed for caching
- Pub/Sub for real-time updates
- Support for complex data structures
- Persistence options

## Future Architecture Improvements

1. **Event Sourcing**: Implement full event sourcing for audit trail
2. **CQRS**: Separate read and write models
3. **GraphQL**: Add GraphQL layer for frontend
4. **Serverless**: Move some workloads to Lambda/Cloud Functions
5. **Edge Computing**: Deploy closer to users globally
6. **Machine Learning Pipeline**: MLOps infrastructure for model training

## References

- [Data Flow Diagram](./DATA_FLOW.md)
- [Security Architecture](./SECURITY_ARCHITECTURE.md)
- [Deployment Guide](./DEPLOYMENT_ARCHITECTURE.md)
- [API Specification](../../specs/API_SPEC.md)
- [Integration Patterns](../../specs/INTEGRATION_SPEC.md)
