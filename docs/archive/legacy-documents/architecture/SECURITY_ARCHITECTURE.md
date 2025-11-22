# Security Architecture

## Overview

This document outlines the security measures, protocols, and best practices implemented in the Bot Core trading platform to protect user data, financial transactions, and system integrity.

## Security Layers

```mermaid
graph TB
    subgraph "Layer 7: Application Security"
        AUTH[Authentication & Authorization]
        VALID[Input Validation]
        AUDIT[Audit Logging]
    end

    subgraph "Layer 6: API Security"
        RATE[Rate Limiting]
        JWT[JWT Validation]
        CORS[CORS Policy]
    end

    subgraph "Layer 5: Transport Security"
        TLS[TLS 1.3]
        MTLS[mTLS between services]
        WSS[WSS for WebSockets]
    end

    subgraph "Layer 4: Network Security"
        FW[Firewall Rules]
        VPC[VPC Isolation]
        SG[Security Groups]
    end

    subgraph "Layer 3: Infrastructure Security"
        IAM[IAM Policies]
        SECRETS[Secrets Management]
        ENCRYPT[Encryption at Rest]
    end

    subgraph "Layer 2: Monitoring & Detection"
        IDS[Intrusion Detection]
        SIEM[SIEM System]
        ALERTS[Security Alerts]
    end

    subgraph "Layer 1: Physical Security"
        DC[Data Center Security]
        REDUNDANCY[Geographic Redundancy]
    end
```

## Authentication & Authorization

### JWT-Based Authentication

```mermaid
sequenceDiagram
    participant Client
    participant Auth Service
    participant Protected Resource
    participant Token Store

    Client->>Auth Service: POST /api/auth/login {email, password}
    Auth Service->>Auth Service: Verify credentials
    Auth Service->>Auth Service: Generate JWT (RS256)
    Auth Service->>Auth Service: Generate Refresh Token
    Auth Service->>Token Store: Store refresh token
    Auth Service-->>Client: {access_token, refresh_token}

    Client->>Protected Resource: GET /api/positions<br/>Authorization: Bearer {token}
    Protected Resource->>Protected Resource: Verify JWT signature
    Protected Resource->>Protected Resource: Check expiration
    Protected Resource->>Protected Resource: Extract user_id
    Protected Resource-->>Client: Protected data
```

**JWT Structure**:
```json
{
  "header": {
    "alg": "RS256",
    "typ": "JWT"
  },
  "payload": {
    "sub": "user_123",
    "email": "user@example.com",
    "role": "trader",
    "permissions": ["trade", "view_positions", "view_history"],
    "iat": 1701234567,
    "exp": 1701320967,
    "iss": "bot-core-auth",
    "aud": "bot-core-api"
  }
}
```

**Token Lifetimes**:
- Access Token: 24 hours
- Refresh Token: 7 days
- Session Token: 30 days (remember me)

### Role-Based Access Control (RBAC)

```mermaid
graph LR
    subgraph "Roles"
        ADMIN[Admin]
        TRADER[Trader]
        VIEWER[Viewer]
        API[API User]
    end

    subgraph "Permissions"
        P1[trade]
        P2[view_positions]
        P3[view_history]
        P4[manage_users]
        P5[manage_settings]
        P6[view_logs]
    end

    ADMIN --> P1 & P2 & P3 & P4 & P5 & P6
    TRADER --> P1 & P2 & P3
    VIEWER --> P2 & P3
    API --> P1 & P2 & P3
```

**Permission Matrix**:

| Resource | Admin | Trader | Viewer | API User |
|----------|-------|--------|--------|----------|
| Execute Trades | ✓ | ✓ | ✗ | ✓ |
| View Positions | ✓ | ✓ | ✓ | ✓ |
| View History | ✓ | ✓ | ✓ | ✓ |
| Modify Settings | ✓ | ✓ | ✗ | ✗ |
| Manage Users | ✓ | ✗ | ✗ | ✗ |
| View Audit Logs | ✓ | ✗ | ✗ | ✗ |

### Multi-Factor Authentication (2FA)

```mermaid
sequenceDiagram
    participant User
    participant Frontend
    participant Auth Service
    participant TOTP Service

    User->>Frontend: Enter email/password
    Frontend->>Auth Service: POST /api/auth/login
    Auth Service->>Auth Service: Verify credentials
    Auth Service-->>Frontend: Require 2FA
    Frontend-->>User: Prompt for 2FA code

    User->>Frontend: Enter TOTP code
    Frontend->>Auth Service: POST /api/auth/verify-2fa
    Auth Service->>TOTP Service: Verify code
    TOTP Service-->>Auth Service: Valid
    Auth Service-->>Frontend: JWT tokens
```

**2FA Requirements**:
- Required for: Withdrawals, settings changes, API key generation
- Optional for: Regular trading (recommended)
- Methods: TOTP (Google Authenticator), SMS (backup)

## API Security

### Rate Limiting

```mermaid
graph LR
    REQUEST[API Request] --> CHECK{Rate Check}
    CHECK -->|Within Limit| PROCESS[Process Request]
    CHECK -->|Exceeded| REJECT[429 Too Many Requests]

    subgraph "Rate Limiter"
        REDIS[(Redis Counter)]
        SLIDING[Sliding Window]
    end

    CHECK --> REDIS
    REDIS --> SLIDING
```

**Rate Limit Tiers**:

| Endpoint Category | Limit | Window | Per |
|------------------|-------|--------|-----|
| Authentication | 5 | 15 min | IP |
| Trade Execution | 10 | 1 sec | User |
| Market Data | 100 | 1 min | User |
| AI Analysis | 10 | 1 min | User |
| General API | 1000 | 1 min | User |

**Response Headers**:
```http
X-RateLimit-Limit: 100
X-RateLimit-Remaining: 45
X-RateLimit-Reset: 1701234567
Retry-After: 60
```

### Input Validation

```mermaid
graph LR
    INPUT[User Input] --> SANITIZE[Sanitize]
    SANITIZE --> VALIDATE[Validate]
    VALIDATE --> TYPE[Type Check]
    TYPE --> RANGE[Range Check]
    RANGE --> FORMAT[Format Check]
    FORMAT --> BUSINESS[Business Rules]
    BUSINESS --> SAFE[Safe Input]
```

**Validation Rules**:

```rust
// Rust example
pub struct OrderRequest {
    #[validate(length(equal = 6))]
    pub symbol: String,  // Must be 6 chars, e.g., "BTCUSDT"

    #[validate(custom = "validate_side")]
    pub side: OrderSide,  // Must be BUY or SELL

    #[validate(range(min = 0.00001, max = 1000000.0))]
    pub quantity: f64,  // Must be positive, reasonable

    #[validate(range(min = 0.01))]
    pub price: Option<f64>,  // If provided, must be positive
}
```

**Sanitization**:
- HTML encoding for display
- SQL parameter binding (no raw SQL)
- JSON schema validation
- Whitelist-based input filtering

### CORS Policy

```javascript
// Allowed origins
const allowedOrigins = [
  'https://app.botcore.com',
  'https://dashboard.botcore.com',
  'http://localhost:3000'  // Development only
];

// CORS headers
{
  'Access-Control-Allow-Origin': origin,
  'Access-Control-Allow-Methods': 'GET, POST, PUT, DELETE, OPTIONS',
  'Access-Control-Allow-Headers': 'Authorization, Content-Type',
  'Access-Control-Max-Age': '86400',
  'Access-Control-Allow-Credentials': 'true'
}
```

## Transport Security

### TLS/SSL Configuration

```yaml
tls_config:
  min_version: "TLS 1.3"
  cipher_suites:
    - TLS_AES_256_GCM_SHA384
    - TLS_AES_128_GCM_SHA256
    - TLS_CHACHA20_POLY1305_SHA256
  certificate: "/etc/ssl/certs/botcore.crt"
  private_key: "/etc/ssl/private/botcore.key"
  client_auth: "request"  # For mTLS
```

**Certificate Management**:
- Auto-renewal with Let's Encrypt
- 90-day rotation schedule
- Certificate pinning for mobile apps
- OCSP stapling enabled

### Mutual TLS (mTLS) for Inter-Service Communication

```mermaid
sequenceDiagram
    participant Rust Service
    participant Python Service

    Rust Service->>Python Service: TLS Handshake
    Python Service->>Rust Service: Present certificate
    Rust Service->>Rust Service: Verify certificate
    Rust Service->>Python Service: Present certificate
    Python Service->>Python Service: Verify certificate
    Note over Rust Service,Python Service: Encrypted channel established
    Rust Service->>Python Service: API request
    Python Service-->>Rust Service: API response
```

**Certificate Attributes**:
```
Subject: CN=rust-core-engine.bot-core.svc.cluster.local
Issuer: CN=Bot Core Internal CA
Valid: 2025-01-01 to 2025-12-31
Key Usage: Digital Signature, Key Encipherment
Extended Key Usage: Server Auth, Client Auth
```

## Data Security

### Encryption at Rest

```mermaid
graph TB
    subgraph "MongoDB"
        DATA[User Data]
        ENC1[AES-256 Encryption]
        KEY1[Encryption Key]
    end

    subgraph "Redis"
        CACHE[Cached Data]
        ENC2[AES-256 Encryption]
        KEY2[Encryption Key]
    end

    subgraph "Key Management"
        KMS[AWS KMS / HashiCorp Vault]
    end

    DATA --> ENC1
    CACHE --> ENC2
    ENC1 --> KEY1
    ENC2 --> KEY2
    KEY1 --> KMS
    KEY2 --> KMS
```

**Encrypted Fields**:
- User credentials (bcrypt for passwords)
- API keys (AES-256)
- Sensitive personal information
- Financial data
- Trading strategies

### Secrets Management

```mermaid
graph LR
    subgraph "Development"
        ENV[.env file<br/>Local only]
    end

    subgraph "Staging/Production"
        VAULT[HashiCorp Vault]
        KMS[AWS Secrets Manager]
    end

    subgraph "Application"
        SERVICE[Service Runtime]
    end

    ENV -.Development Only.-> SERVICE
    VAULT --> SERVICE
    KMS --> SERVICE
```

**Secret Rotation**:
```mermaid
graph LR
    A[Day 0: Secret A Active] --> B[Day 30: Generate Secret B]
    B --> C[Day 31: Both A & B Valid]
    C --> D[Day 32: Rotate to Secret B]
    D --> E[Day 33: Secret A Deprecated]
    E --> F[Day 60: Generate Secret C]
```

**Rotation Schedule**:
- Database passwords: 90 days
- API keys: 90 days
- JWT signing keys: 180 days
- TLS certificates: 90 days (auto-renewal)
- Admin passwords: 60 days (forced)

## Network Security

### VPC Architecture

```mermaid
graph TB
    subgraph "VPC: 10.0.0.0/16"
        subgraph "Public Subnet: 10.0.1.0/24"
            ALB[Load Balancer]
            NAT[NAT Gateway]
        end

        subgraph "Private Subnet 1: 10.0.10.0/24"
            RUST[Rust Service]
            PYTHON[Python Service]
            NEXT[Next.js Service]
        end

        subgraph "Private Subnet 2: 10.0.20.0/24"
            MONGO[(MongoDB)]
            REDIS[(Redis)]
        end

        subgraph "Management Subnet: 10.0.100.0/24"
            BASTION[Bastion Host]
        end
    end

    INTERNET[Internet] --> ALB
    ALB --> RUST & PYTHON & NEXT
    RUST & PYTHON --> MONGO & REDIS
    RUST & PYTHON --> NAT
    NAT --> INTERNET
    INTERNET -.SSH.-> BASTION
    BASTION -.Admin Access.-> MONGO & REDIS
```

### Security Groups

**Web Tier**:
```yaml
ingress:
  - port: 443
    protocol: tcp
    source: 0.0.0.0/0
    description: HTTPS from internet
  - port: 80
    protocol: tcp
    source: 0.0.0.0/0
    description: HTTP redirect to HTTPS

egress:
  - port: 8080
    protocol: tcp
    destination: sg-app-tier
    description: To application tier
```

**Application Tier**:
```yaml
ingress:
  - port: 8080
    source: sg-web-tier
    description: From web tier
  - port: 8000
    source: sg-app-tier
    description: Inter-service communication

egress:
  - port: 27017
    destination: sg-data-tier
    description: To MongoDB
  - port: 6379
    destination: sg-data-tier
    description: To Redis
  - port: 443
    destination: 0.0.0.0/0
    description: External APIs
```

### DDoS Protection

```mermaid
graph LR
    ATTACK[DDoS Attack] --> CF[CloudFront]
    CF --> SHIELD[AWS Shield]
    SHIELD --> WAF[AWS WAF]
    WAF --> ALB[Load Balancer]
    ALB --> APP[Application]

    CF -.Block.-> ATTACK
    SHIELD -.Mitigate.-> ATTACK
    WAF -.Filter.-> ATTACK
```

**WAF Rules**:
- Rate limiting (1000 req/min per IP)
- Geographic blocking (sanctioned countries)
- Known bad bot signatures
- SQL injection patterns
- XSS patterns
- Large payload blocking (> 1MB)

## Application Security

### Secure Coding Practices

**Rust: No `unwrap()` in Production**:
```rust
// Bad
let user = database.get_user(id).unwrap();  // Can panic!

// Good
let user = database.get_user(id)
    .map_err(|e| TradingError::DatabaseError(e))?;
```

**Python: Input Sanitization**:
```python
# Bad
query = f"SELECT * FROM users WHERE email = '{email}'"  # SQL injection!

# Good
query = "SELECT * FROM users WHERE email = ?"
cursor.execute(query, (email,))
```

**TypeScript: Type Safety**:
```typescript
// Bad
function processOrder(order: any) { ... }  // No type safety

// Good
function processOrder(order: Order): Result<Trade, Error> { ... }
```

### Security Headers

```http
Strict-Transport-Security: max-age=31536000; includeSubDomains; preload
X-Frame-Options: DENY
X-Content-Type-Options: nosniff
X-XSS-Protection: 1; mode=block
Content-Security-Policy: default-src 'self'; script-src 'self' 'unsafe-inline'; style-src 'self' 'unsafe-inline'
Referrer-Policy: strict-origin-when-cross-origin
Permissions-Policy: geolocation=(), microphone=(), camera=()
```

## Audit Logging

### Audit Trail

```mermaid
graph LR
    ACTION[User Action] --> LOG[Audit Logger]
    LOG --> SIEM[SIEM System]
    LOG --> S3[S3 Archive]
    SIEM --> ALERT[Security Alerts]
    SIEM --> DASHBOARD[Security Dashboard]
```

**Logged Events**:
```json
{
  "event_id": "evt_123456",
  "timestamp": "2025-10-10T12:00:00Z",
  "event_type": "trade_executed",
  "user_id": "user_123",
  "ip_address": "203.0.113.42",
  "user_agent": "Mozilla/5.0...",
  "action": "POST /api/trades/execute",
  "resource": "trade_789",
  "result": "success",
  "metadata": {
    "symbol": "BTCUSDT",
    "quantity": 0.001,
    "price": 45000.00
  },
  "session_id": "sess_456",
  "request_id": "req_789"
}
```

**Retention**:
- Security logs: 7 years
- Audit trail: 7 years
- Access logs: 1 year
- Debug logs: 30 days

## Vulnerability Management

### Security Scanning

```mermaid
graph TB
    CODE[Code] --> SAST[SAST Scan]
    DOCKER[Docker Image] --> CONTAINER[Container Scan]
    DEPS[Dependencies] --> SCA[SCA Scan]

    SAST --> REPORT[Security Report]
    CONTAINER --> REPORT
    SCA --> REPORT

    REPORT --> REVIEW[Security Review]
    REVIEW -->|Critical| BLOCK[Block Deployment]
    REVIEW -->|High| WARN[Warning]
    REVIEW -->|Medium/Low| PASS[Pass]
```

**Tools**:
- SAST: SonarQube, Semgrep
- Container: Trivy, Clair
- SCA: Dependabot, Snyk
- DAST: OWASP ZAP

**Vulnerability SLA**:
- Critical: 24 hours
- High: 7 days
- Medium: 30 days
- Low: 90 days

### Penetration Testing

**Schedule**:
- Annual third-party penetration test
- Quarterly internal security assessment
- Continuous automated scanning

**Scope**:
- External attack surface
- Internal network segmentation
- API security
- Authentication mechanisms
- Data protection

## Incident Response

### Incident Response Plan

```mermaid
stateDiagram-v2
    [*] --> Detection: Security Event
    Detection --> Analysis: Validate Threat
    Analysis --> Containment: Confirmed Incident
    Containment --> Eradication: Threat Isolated
    Eradication --> Recovery: Threat Removed
    Recovery --> PostIncident: System Restored
    PostIncident --> [*]: Lessons Learned

    Analysis --> [*]: False Positive
```

**Response Times**:
- Detection: < 5 minutes (automated)
- Analysis: < 15 minutes
- Containment: < 1 hour
- Communication: < 2 hours
- Recovery: < 24 hours

### Security Contacts

```yaml
security_team:
  email: security@botcore.com
  pagerduty: security-oncall
  emergency: +1-XXX-XXX-XXXX

escalation:
  - level: 1
    team: Security Engineers
    response_time: 15 min

  - level: 2
    team: Security Manager
    response_time: 30 min

  - level: 3
    team: CISO
    response_time: 1 hour
```

## Compliance

### Standards & Frameworks

- **SOC 2 Type II**: Annual audit
- **ISO 27001**: Information Security Management
- **PCI DSS**: Payment card data (if applicable)
- **GDPR**: EU user data protection
- **CCPA**: California privacy rights

### Data Privacy

**User Rights**:
- Right to access personal data
- Right to data portability
- Right to erasure ("right to be forgotten")
- Right to rectification
- Right to restrict processing

**Data Minimization**:
- Collect only necessary data
- Anonymize where possible
- Pseudonymize personal identifiers
- Regular data cleanup

## Security Checklist

### Pre-Deployment

- [ ] All secrets moved to Vault/KMS
- [ ] No hardcoded credentials in code
- [ ] TLS 1.3 configured
- [ ] Rate limiting enabled
- [ ] Input validation implemented
- [ ] RBAC configured
- [ ] Audit logging enabled
- [ ] Security headers set
- [ ] Dependencies scanned
- [ ] Container images scanned
- [ ] Penetration test completed

### Ongoing

- [ ] Daily security scans
- [ ] Weekly dependency updates
- [ ] Monthly access review
- [ ] Quarterly security training
- [ ] Annual penetration test
- [ ] Regular backup testing
- [ ] Incident response drills

## References

- [System Architecture](./SYSTEM_ARCHITECTURE.md)
- [Data Flow](./DATA_FLOW.md)
- [Deployment Guide](../../documents/DEPLOYMENT.md)
- [Disaster Recovery](../../documents/DISASTER_RECOVERY.md)
