# Security Architecture

**Spec ID**: ARCH-SECURITY-001 to ARCH-SECURITY-009
**Version**: 1.0
**Status**: ☑ Approved
**Owner**: Security Team
**Last Updated**: 2025-10-11

---

## Tasks Checklist

- [x] Security requirements gathered
- [x] Threat modeling completed
- [x] Security controls designed
- [x] Authentication implemented
- [x] Authorization implemented
- [x] Encryption configured
- [ ] Penetration testing completed
- [ ] Security audit scheduled
- [ ] Compliance review pending

---

## Metadata

**Related Specs**:
- Related NFR: NFR-SECURITY-001 to NFR-SECURITY-005
- Related FR: FR-AUTH-001, FR-TRADING-001
- Related Design: ARCH-OVERVIEW-001, ARCH-DATA-FLOW-004

**Dependencies**:
- Depends on: API_SPEC.md, DATA_MODELS.md
- Blocks: DEPLOYMENT-SPEC.md, TC-SECURITY-001

**Business Value**: Critical
**Technical Complexity**: High
**Priority**: ☑ Critical

**Compliance Frameworks**:
- OWASP Top 10 (2021)
- NIST Cybersecurity Framework
- CIS Controls v8
- GDPR (EU data protection)

---

## Overview

This document provides a comprehensive security architecture for the Bot Core cryptocurrency trading platform. Security is paramount as the system handles user authentication, financial transactions, and sensitive trading data. The architecture follows defense-in-depth principles with multiple layers of security controls.

### Security Principles

1. **Least Privilege**: Users and services have minimum necessary permissions
2. **Defense in Depth**: Multiple layers of security controls
3. **Secure by Default**: Secure configurations out of the box
4. **Fail Secure**: System fails to a secure state on errors
5. **Separation of Duties**: Critical operations require multiple checks
6. **Audit Trail**: All security-relevant events are logged
7. **Privacy by Design**: User data protection built into architecture

### Security Objectives

**Confidentiality**:
- Protect user credentials (passwords, API keys)
- Encrypt sensitive data at rest and in transit
- Prevent unauthorized access to trading data

**Integrity**:
- Ensure trade data cannot be tampered with
- Validate all inputs to prevent injection attacks
- Maintain audit logs for all transactions

**Availability**:
- Protect against DDoS attacks
- Implement rate limiting to prevent abuse
- Ensure system uptime during security incidents

**Authentication**:
- Strong password policies
- JWT-based stateless authentication
- Secure session management

**Authorization**:
- Role-based access control (RBAC)
- Resource-level permissions
- API endpoint protection

---

## ARCH-SECURITY-001: Authentication Architecture

**Priority**: ☑ Critical
**Status**: ☑ Implemented
**Code Tags**: `@spec:ARCH-SECURITY-001`
**Implementation**: `rust-core-engine/src/auth/jwt.rs`, `rust-core-engine/src/auth/handlers.rs`

### Description

This section describes the complete authentication system, including user registration, login, password management, JWT token lifecycle, and session security.

### Authentication Flow Diagram

```mermaid
sequenceDiagram
    participant User
    participant Frontend
    participant AuthHandler as Auth Handler
    participant PasswordSvc as Password Service<br/>(bcrypt)
    participant JWTSvc as JWT Service<br/>(HMAC-SHA256)
    participant MongoDB
    participant AuditLog

    Note over User,AuditLog: User Registration
    User->>Frontend: Enter registration details
    Frontend->>Frontend: Client-side validation
    Frontend->>AuthHandler: POST /api/auth/register
    AuthHandler->>AuthHandler: Validate input (email, password)
    AuthHandler->>MongoDB: Check email uniqueness
    alt Email exists
        MongoDB-->>AuthHandler: User found
        AuthHandler->>AuditLog: Log failed registration (duplicate email)
        AuthHandler-->>Frontend: 409 Conflict
    else Email available
        MongoDB-->>AuthHandler: null
        AuthHandler->>PasswordSvc: hash_password(password)
        Note right of PasswordSvc: bcrypt(password, cost=12)<br/>Time: 100-200ms
        PasswordSvc-->>AuthHandler: password_hash
        AuthHandler->>MongoDB: Create user document
        MongoDB-->>AuthHandler: user_id
        AuthHandler->>JWTSvc: generate_token(user_id, email, is_admin)
        JWTSvc-->>AuthHandler: JWT token
        AuthHandler->>AuditLog: Log successful registration
        AuthHandler-->>Frontend: 201 Created {token, user}
    end

    Note over User,AuditLog: User Login
    User->>Frontend: Enter credentials
    Frontend->>AuthHandler: POST /api/auth/login
    AuthHandler->>MongoDB: Find user by email
    alt User not found
        MongoDB-->>AuthHandler: null
        AuthHandler->>AuditLog: Log failed login (user not found)
        AuthHandler-->>Frontend: 401 Unauthorized
    else User found
        MongoDB-->>AuthHandler: User document
        AuthHandler->>AuthHandler: Check account status
        alt Account deactivated
            AuthHandler->>AuditLog: Log login attempt (deactivated account)
            AuthHandler-->>Frontend: 403 Forbidden
        else Account active
            AuthHandler->>PasswordSvc: verify_password(password, hash)
            PasswordSvc-->>AuthHandler: verification result
            alt Password incorrect
                AuthHandler->>AuthHandler: Increment failed attempts
                AuthHandler->>AuditLog: Log failed login (wrong password)
                alt Failed attempts >= 5
                    AuthHandler->>MongoDB: Lock account (1 hour)
                    AuthHandler->>AuditLog: Log account lockout
                    AuthHandler-->>Frontend: 403 Account locked
                else Failed attempts < 5
                    AuthHandler-->>Frontend: 401 Unauthorized
                end
            else Password correct
                AuthHandler->>MongoDB: Reset failed attempts
                AuthHandler->>MongoDB: Update last_login
                AuthHandler->>JWTSvc: generate_token(user_id, email, is_admin)
                JWTSvc-->>AuthHandler: JWT token
                AuthHandler->>AuditLog: Log successful login
                AuthHandler-->>Frontend: 200 OK {token, user}
            end
        end
    end

    Note over User,AuditLog: Token Verification (Every Request)
    Frontend->>AuthHandler: GET /api/protected<br/>Authorization: Bearer {token}
    AuthHandler->>JWTSvc: verify_token(token)
    JWTSvc->>JWTSvc: Decode header
    JWTSvc->>JWTSvc: Verify HMAC-SHA256 signature
    alt Signature invalid
        JWTSvc-->>AuthHandler: Error: Invalid signature
        AuthHandler->>AuditLog: Log invalid token attempt
        AuthHandler-->>Frontend: 401 Unauthorized
    else Signature valid
        JWTSvc->>JWTSvc: Check expiration
        alt Token expired
            JWTSvc-->>AuthHandler: Error: Token expired
            AuthHandler->>AuditLog: Log expired token use
            AuthHandler-->>Frontend: 401 Token expired
        else Token valid
            JWTSvc-->>AuthHandler: Claims {user_id, email, is_admin}
            AuthHandler->>AuthHandler: Proceed with request
        end
    end
```

### JWT Token Structure

**Token Composition**:
A JWT token consists of three base64url-encoded parts separated by dots:
```
{header}.{payload}.{signature}
```

**Header**:
```json
{
  "alg": "HS256",
  "typ": "JWT"
}
```
- `alg`: Algorithm used for signing (HMAC-SHA256)
- `typ`: Token type (JWT)

**Payload (Claims)**:
```json
{
  "sub": "60f7b3b3e4b0c8f7d8f7b3b3",
  "email": "user@example.com",
  "is_admin": false,
  "iat": 1698765432,
  "exp": 1699370232
}
```
- `sub` (Subject): User ID (MongoDB ObjectId as hex string)
- `email`: User's email address
- `is_admin`: Admin flag (boolean)
- `iat` (Issued At): Token creation timestamp (Unix epoch)
- `exp` (Expiration): Token expiration timestamp (Unix epoch)

**Signature Calculation**:
```
HMACSHA256(
  base64UrlEncode(header) + "." + base64UrlEncode(payload),
  secret_key
)
```

**Complete Token Example**:
```
eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9.eyJzdWIiOiI2MGY3YjNiM2U0YjBjOGY3ZDhmN2IzYjMiLCJlbWFpbCI6InVzZXJAZXhhbXBsZS5jb20iLCJpc19hZG1pbiI6ZmFsc2UsImlhdCI6MTY5ODc2NTQzMiwiZXhwIjoxNjk5MzcwMjMyfQ.5J8n5F7qF7kZ8F8q5F8n5F7qF7kZ8F8q5F8n5F7qF7k
```

### Token Lifecycle

**Token Generation**:
1. User successfully authenticates (login or registration)
2. Server creates claims object with user info
3. Server signs claims with HMAC-SHA256 using secret key
4. Server returns token to client
5. **Latency**: 1-5ms

**Token Storage** (Client-Side):
- **Location**: Browser localStorage
- **Key**: `auth_token`
- **Format**: Plain text string
- **Security Considerations**:
  - Vulnerable to XSS attacks (mitigated by Content Security Policy)
  - Not vulnerable to CSRF (not sent in cookies)
  - Not HttpOnly (JavaScript needs access for API calls)

**Token Transmission**:
- **Header**: `Authorization: Bearer {token}`
- **Protocol**: HTTPS only (TLS 1.3)
- **Never in URL**: Tokens are never sent as query parameters

**Token Validation**:
1. Client includes token in Authorization header
2. Server extracts token from "Bearer {token}" format
3. Server verifies HMAC-SHA256 signature
4. Server checks expiration timestamp
5. Server extracts claims if valid
6. **Latency**: 1-5ms

**Token Expiration**:
- **Expiration Time**: 7 days (604800 seconds)
- **Renewal**: Not implemented (user must re-login after expiration)
- **Reason for 7 days**: Balance between security and user convenience
- **Future Enhancement**: Refresh tokens with shorter access token expiration (15 minutes)

**Token Revocation** (Not Currently Implemented):
- **Challenge**: Stateless JWTs cannot be revoked
- **Workarounds**:
  - Maintain blacklist of revoked tokens in Redis (future)
  - Check user's `is_active` flag on critical operations (current)
  - Short expiration time (future: 15 min access tokens + refresh tokens)

### Password Security

**Password Hashing Algorithm**: bcrypt

**Why bcrypt?**
- Adaptive: Cost factor can be increased as hardware improves
- Salted: Each password has a unique random salt
- Slow: Intentionally slow to prevent brute force attacks
- Proven: Battle-tested, recommended by OWASP

**Cost Factor**: 12
- **Iterations**: 2^12 = 4096 rounds
- **Time**: 100-200ms per hash/verification
- **Rationale**: Slow enough to deter brute force, fast enough for user experience

**Salt Generation**:
- **Length**: 16 bytes (128 bits)
- **Source**: Cryptographically secure random number generator (CSPRNG)
- **Uniqueness**: New salt generated for every password
- **Storage**: Embedded in bcrypt hash (automatic)

**Hash Format**:
```
$2b$12$R9h/cIPz0gi.URNNX3kh2OPST9/PgBkqquzi.Ss7KIUgO2t0jWMUW
│  │  │                       │
│  │  │                       └─ Hash (31 chars)
│  │  └─────────────────────────── Salt (22 chars)
│  └────────────────────────────── Cost factor (12)
└───────────────────────────────── Algorithm identifier ($2b$ = bcrypt)
```
- **Total Length**: 60 characters
- **Storage**: VARCHAR(60) in database

**Password Verification**:
```rust
// Pseudocode
fn verify_password(input: &str, stored_hash: &str) -> Result<bool> {
    // bcrypt automatically extracts salt from stored_hash
    // and re-hashes input with same salt
    let is_valid = bcrypt::verify(input, stored_hash)?;
    Ok(is_valid)
}
```
- **Timing**: Constant time to prevent timing attacks
- **Latency**: 100-200ms (same as hashing)

### Password Policy

**Complexity Requirements**:
- **Minimum Length**: 8 characters
- **Maximum Length**: 128 characters
- **Character Classes** (at least 3 of 4):
  - Uppercase letters (A-Z)
  - Lowercase letters (a-z)
  - Digits (0-9)
  - Special characters (!@#$%^&*()_+-=[]{}|;:,.<>?)
- **No Common Passwords**: Check against list of 10,000 most common passwords
- **No User Info**: Password cannot contain user's email or name

**Password Validation** (Frontend + Backend):
```typescript
// Frontend validation (immediate feedback)
function validatePassword(password: string): { valid: boolean; errors: string[] } {
  const errors: string[] = [];

  if (password.length < 8) errors.push("At least 8 characters required");
  if (password.length > 128) errors.push("Maximum 128 characters");
  if (!/[A-Z]/.test(password)) errors.push("At least one uppercase letter");
  if (!/[a-z]/.test(password)) errors.push("At least one lowercase letter");
  if (!/[0-9]/.test(password)) errors.push("At least one digit");
  if (!/[!@#$%^&*()_+\-=\[\]{}|;:,.<>?]/.test(password)) {
    errors.push("At least one special character");
  }

  return { valid: errors.length === 0, errors };
}
```

**Password Storage**:
- **Original Password**: NEVER stored (only hash)
- **Hash Storage**: MongoDB `users` collection, `password_hash` field
- **Hash Transmission**: NEVER transmitted (only verified server-side)

### Account Lockout

**Failed Login Attempts**:
- **Threshold**: 5 failed attempts within 15 minutes
- **Lockout Duration**: 1 hour
- **Tracking**: In-memory counter (per user email)
- **Reset**: Counter resets on successful login or after 15 minutes

**Lockout Mechanism**:
```rust
// Pseudocode
struct FailedAttempts {
    count: u32,
    first_attempt_at: DateTime<Utc>,
    locked_until: Option<DateTime<Utc>>,
}

fn check_account_lock(email: &str) -> Result<()> {
    let attempts = get_failed_attempts(email);

    // Check if currently locked
    if let Some(locked_until) = attempts.locked_until {
        if Utc::now() < locked_until {
            return Err(Error::AccountLocked {
                unlock_time: locked_until,
            });
        } else {
            // Lock expired, reset counter
            reset_failed_attempts(email);
        }
    }

    // Check if within 15-minute window
    if Utc::now() - attempts.first_attempt_at > Duration::minutes(15) {
        reset_failed_attempts(email);
    }

    // Check if threshold reached
    if attempts.count >= 5 {
        lock_account(email, Duration::hours(1));
        return Err(Error::AccountLocked {
            unlock_time: Utc::now() + Duration::hours(1),
        });
    }

    Ok(())
}
```

**Unlock Mechanisms**:
1. **Automatic**: After 1 hour lockout period
2. **Manual**: Admin can unlock via admin panel
3. **Password Reset**: Successful password reset unlocks account

### Email Enumeration Prevention

**Problem**: Attackers can determine which emails are registered by observing different error messages.

**Solution**: Generic error messages
- Login: "Invalid email or password" (same for both wrong email and wrong password)
- Registration: "Email already registered" (acceptable, as registration is public action)
- Password Reset: "If an account exists, a reset email has been sent" (always same message)

**Timing Attack Prevention**:
- Password verification takes constant time (bcrypt)
- Database queries are always performed (even if email doesn't exist)
- Response time is consistent (within 50ms variance)

### Multi-Factor Authentication (Future Enhancement)

**Planned Implementation**:
- **Method**: Time-based One-Time Password (TOTP)
- **Algorithm**: HMAC-SHA1
- **Standard**: RFC 6238
- **Apps**: Google Authenticator, Authy, 1Password
- **Backup Codes**: 10 single-use codes for recovery

**Flow**:
1. User enables MFA in settings
2. Server generates secret key and QR code
3. User scans QR code with authenticator app
4. User enters 6-digit code to confirm setup
5. On login: User enters password + 6-digit code
6. Server verifies code (within 30-second time window)

---

## ARCH-SECURITY-002: Authorization Architecture

**Priority**: ☑ Critical
**Status**: ☑ Implemented
**Code Tags**: `@spec:ARCH-SECURITY-002`
**Implementation**: `rust-core-engine/src/auth/middleware.rs`

### Description

This section describes the authorization system, including role-based access control (RBAC), permission enforcement, and resource-level authorization.

### Authorization Model

**RBAC (Role-Based Access Control)**:
The system uses a simple two-role model:

**Roles**:
1. **user** (default): Regular users
2. **admin**: Administrators

**Role Assignment**:
- **Default**: New users are assigned `user` role
- **Admin**: Manually assigned by existing admin in database
- **Storage**: `is_admin` boolean field in user document

### Permission Matrix

**User Role Permissions**:
| Resource | Action | Allowed |
|----------|--------|---------|
| Own Profile | Read | Yes |
| Own Profile | Update | Yes |
| Other Profiles | Read | No |
| Other Profiles | Update | No |
| Own Trades | Create | Yes |
| Own Trades | Read | Yes |
| Own Trades | Cancel | Yes |
| Other Trades | Read | No |
| Own Portfolio | Read | Yes |
| Other Portfolios | Read | No |
| Market Data | Read | Yes |
| AI Signals | Read | Yes |
| System Settings | Read | No |
| System Settings | Update | No |

**Admin Role Permissions**:
| Resource | Action | Allowed |
|----------|--------|---------|
| All Profiles | Read | Yes |
| All Profiles | Update | Yes |
| All Trades | Read | Yes |
| All Trades | Cancel | Yes |
| All Portfolios | Read | Yes |
| System Settings | Read | Yes |
| System Settings | Update | Yes |
| User Management | Create | Yes |
| User Management | Deactivate | Yes |
| Audit Logs | Read | Yes |

### Authorization Middleware

**Middleware Functions**:

**1. with_auth**: Requires valid JWT token
```rust
pub fn with_auth(
    jwt_service: JwtService,
) -> impl Filter<Extract = (Claims,), Error = Rejection> + Clone {
    warp::header::<String>("authorization")
        .and(warp::any().map(move || jwt_service.clone()))
        .and_then(authorize)
}

// Usage
let route = warp::path("portfolio")
    .and(with_auth(jwt_service.clone()))
    .and_then(get_portfolio);
```

**2. with_admin_auth**: Requires admin role
```rust
pub fn with_admin_auth(
    jwt_service: JwtService,
) -> impl Filter<Extract = (Claims,), Error = Rejection> + Clone {
    warp::header::<String>("authorization")
        .and(warp::any().map(move || jwt_service.clone()))
        .and_then(admin_authorize)
}

async fn admin_authorize(auth_header: String, jwt_service: JwtService) -> Result<Claims, Rejection> {
    let claims = authorize(auth_header, jwt_service).await?;
    if claims.is_admin {
        Ok(claims)
    } else {
        Err(warp::reject::custom(AuthError::InsufficientPermissions))
    }
}

// Usage
let route = warp::path("admin")
    .and(with_admin_auth(jwt_service.clone()))
    .and_then(admin_dashboard);
```

**3. with_optional_auth**: Auth is optional
```rust
pub fn with_optional_auth(
    jwt_service: JwtService,
) -> impl Filter<Extract = (Option<Claims>,), Error = Rejection> + Clone {
    warp::header::optional::<String>("authorization")
        .and(warp::any().map(move || jwt_service.clone()))
        .and_then(optional_authorize)
}

// Usage (e.g., public endpoint that shows more data if authenticated)
let route = warp::path("market-data")
    .and(with_optional_auth(jwt_service.clone()))
    .and_then(get_market_data);
```

### Resource-Level Authorization

**Pattern**: Ownership-based access control

**Example**: User can only access their own trades
```rust
async fn get_user_trades(claims: Claims, db: Database) -> Result<impl Reply> {
    let user_id = ObjectId::parse_str(&claims.sub)
        .map_err(|_| warp::reject::custom(AuthError::InvalidToken))?;

    // Query only trades belonging to authenticated user
    let trades = db
        .collection::<Trade>("trades")
        .find(doc! { "user_id": user_id }, None)
        .await?;

    Ok(warp::reply::json(&trades))
}
```

**Admin Override**:
```rust
async fn get_all_trades(claims: Claims, db: Database) -> Result<impl Reply> {
    // Check admin permission
    if !claims.is_admin {
        return Err(warp::reject::custom(AuthError::InsufficientPermissions));
    }

    // Admin can see all trades
    let trades = db
        .collection::<Trade>("trades")
        .find(doc! {}, None)
        .await?;

    Ok(warp::reply::json(&trades))
}
```

### Authorization Errors

**403 Forbidden** (Insufficient Permissions):
```json
{
  "success": false,
  "error": "Insufficient permissions",
  "error_code": "FORBIDDEN",
  "required_role": "admin",
  "user_role": "user"
}
```

**401 Unauthorized** (Not Authenticated):
```json
{
  "success": false,
  "error": "Authentication required",
  "error_code": "UNAUTHORIZED"
}
```

### Future Enhancements

**Fine-Grained Permissions**:
- Permission-based system instead of role-based
- Permissions: `trade:create`, `trade:read`, `trade:cancel`, etc.
- Roles as permission collections: `user = [trade:own:*]`, `admin = [trade:*]`
- Stored in database for dynamic updates

**Resource Policies**:
- Define policies in database
- Policy language (e.g., JSON-based rules)
- Example: "User can cancel trade if created_at < 1 hour ago"

---

## ARCH-SECURITY-003: API Security

**Priority**: ☑ Critical
**Status**: ☑ Implemented
**Code Tags**: `@spec:ARCH-SECURITY-003`

### Description

This section describes API-level security measures including rate limiting, input validation, CORS configuration, security headers, and protection against common attacks.

### Rate Limiting

**Implementation**: In-memory token bucket algorithm (Rust), slowapi (Python)

**Rate Limits by Endpoint Type**:

**Public Endpoints** (no authentication required):
- **Login**: 5 requests / 15 minutes per IP
- **Registration**: 3 requests / 15 minutes per IP
- **Password Reset**: 3 requests / 15 minutes per IP
- **Market Data** (unauthenticated): 100 requests / minute per IP

**Authenticated Endpoints**:
- **Trading**: 60 requests / minute per user
- **Portfolio**: 120 requests / minute per user
- **AI Analysis**: 20 requests / minute per user (expensive operation)
- **General API**: 600 requests / minute per user

**Admin Endpoints**:
- **No rate limit** (trusted users)
- **Audit logging** (all requests logged)

**Rate Limit Headers** (returned in response):
```http
X-RateLimit-Limit: 60
X-RateLimit-Remaining: 45
X-RateLimit-Reset: 1698765492
Retry-After: 60
```

**Rate Limit Exceeded Response**:
```http
HTTP/1.1 429 Too Many Requests
Content-Type: application/json
X-RateLimit-Limit: 60
X-RateLimit-Remaining: 0
X-RateLimit-Reset: 1698765492
Retry-After: 60

{
  "success": false,
  "error": "Rate limit exceeded",
  "error_code": "RATE_LIMIT_EXCEEDED",
  "limit": 60,
  "window": "60 seconds",
  "retry_after": 60
}
```

**Rate Limiting Algorithm** (Token Bucket):
```rust
struct TokenBucket {
    capacity: u32,        // Max tokens (e.g., 60)
    tokens: u32,          // Current tokens available
    refill_rate: u32,     // Tokens per second (e.g., 1)
    last_refill: Instant, // Last refill timestamp
}

impl TokenBucket {
    fn try_consume(&mut self, tokens: u32) -> bool {
        self.refill();
        if self.tokens >= tokens {
            self.tokens -= tokens;
            true
        } else {
            false
        }
    }

    fn refill(&mut self) {
        let now = Instant::now();
        let elapsed = (now - self.last_refill).as_secs() as u32;
        let new_tokens = elapsed * self.refill_rate;
        self.tokens = (self.tokens + new_tokens).min(self.capacity);
        self.last_refill = now;
    }
}
```

### CORS Configuration

**Allowed Origins**:
- **Development**: `http://localhost:3000`
- **Production**: `https://app.botcore.com`, `https://www.botcore.com`
- **NO Wildcards**: Never use `Access-Control-Allow-Origin: *`

**Allowed Methods**:
- `GET`, `POST`, `PUT`, `DELETE`, `OPTIONS`

**Allowed Headers**:
- `Content-Type`, `Authorization`, `X-Request-ID`

**Exposed Headers**:
- `X-RateLimit-Limit`, `X-RateLimit-Remaining`, `X-RateLimit-Reset`

**Credentials**:
- `Access-Control-Allow-Credentials: true` (for cookie-based sessions, if used)

**Preflight Cache**:
- `Access-Control-Max-Age: 3600` (1 hour)

**CORS Headers Example**:
```http
Access-Control-Allow-Origin: https://app.botcore.com
Access-Control-Allow-Methods: GET, POST, PUT, DELETE, OPTIONS
Access-Control-Allow-Headers: Content-Type, Authorization, X-Request-ID
Access-Control-Expose-Headers: X-RateLimit-Limit, X-RateLimit-Remaining
Access-Control-Allow-Credentials: true
Access-Control-Max-Age: 3600
```

### Security Headers

**Mandatory Security Headers** (all responses):

**1. Content-Security-Policy (CSP)**:
```
Content-Security-Policy: default-src 'self'; script-src 'self' 'unsafe-inline' https://cdn.jsdelivr.net; style-src 'self' 'unsafe-inline'; img-src 'self' data: https:; font-src 'self' data:; connect-src 'self' https://api.botcore.com wss://api.botcore.com https://api.binance.com wss://stream.binance.com https://api.openai.com; frame-ancestors 'none'; base-uri 'self'; form-action 'self';
```
- **default-src 'self'**: Only load resources from same origin by default
- **script-src**: Allow scripts from self and CDN (for libraries)
- **connect-src**: Allow API calls to backend, Binance, OpenAI
- **frame-ancestors 'none'**: Prevent clickjacking (equivalent to X-Frame-Options: DENY)

**2. X-Frame-Options**:
```
X-Frame-Options: DENY
```
- Prevents page from being embedded in iframe (clickjacking protection)
- Modern browsers prefer CSP `frame-ancestors`, but this is for legacy support

**3. X-Content-Type-Options**:
```
X-Content-Type-Options: nosniff
```
- Prevents browsers from MIME-sniffing content type
- Reduces XSS attack surface

**4. Strict-Transport-Security (HSTS)**:
```
Strict-Transport-Security: max-age=31536000; includeSubDomains; preload
```
- Forces HTTPS for 1 year (31536000 seconds)
- Applies to all subdomains
- Eligible for browser preload list

**5. X-XSS-Protection**:
```
X-XSS-Protection: 1; mode=block
```
- Enables browser's XSS filter (legacy browsers)
- Modern browsers rely on CSP

**6. Referrer-Policy**:
```
Referrer-Policy: strict-origin-when-cross-origin
```
- Only send origin (not full URL) when navigating to different origin
- Protects sensitive data in URLs

**7. Permissions-Policy** (formerly Feature-Policy):
```
Permissions-Policy: geolocation=(), microphone=(), camera=(), payment=()
```
- Disables unnecessary browser features
- Reduces attack surface

**8. X-Request-ID** (custom header for tracing):
```
X-Request-ID: req_abc123def456
```
- Unique ID for each request
- Used for logging and debugging

### Input Validation

**Validation Layers**:
1. **Frontend**: Immediate user feedback (JavaScript)
2. **Backend**: Security validation (Rust/Python)
3. **Database**: Schema validation (MongoDB)

**Validation Principles**:
- **Whitelist, not blacklist**: Define allowed inputs, reject everything else
- **Fail secure**: Reject on validation error, don't try to sanitize
- **Validate type, length, format**: Check all input properties
- **Context-specific**: Email format for emails, numeric for numbers, etc.

**Common Validations**:

**Email**:
```rust
fn validate_email(email: &str) -> Result<()> {
    let email_regex = Regex::new(r"^[a-zA-Z0-9._%+-]+@[a-zA-Z0-9.-]+\.[a-zA-Z]{2,}$")?;
    if !email_regex.is_match(email) {
        return Err(ValidationError::InvalidEmail);
    }
    if email.len() > 254 {
        return Err(ValidationError::EmailTooLong);
    }
    Ok(())
}
```

**Symbol** (trading pair):
```rust
fn validate_symbol(symbol: &str) -> Result<()> {
    // Only uppercase letters, 3-10 characters
    let symbol_regex = Regex::new(r"^[A-Z]{3,10}$")?;
    if !symbol_regex.is_match(symbol) {
        return Err(ValidationError::InvalidSymbol);
    }
    // Check against whitelist of supported symbols
    let supported = ["BTCUSDT", "ETHUSDT", "BNBUSDT", "SOLUSDT"];
    if !supported.contains(&symbol) {
        return Err(ValidationError::UnsupportedSymbol);
    }
    Ok(())
}
```

**Quantity** (trade amount):
```rust
fn validate_quantity(quantity: f64) -> Result<()> {
    if quantity <= 0.0 {
        return Err(ValidationError::QuantityMustBePositive);
    }
    if quantity > 1000000.0 {
        return Err(ValidationError::QuantityTooLarge);
    }
    if quantity.is_nan() || quantity.is_infinite() {
        return Err(ValidationError::InvalidNumber);
    }
    Ok(())
}
```

**Price**:
```rust
fn validate_price(price: f64) -> Result<()> {
    if price <= 0.0 {
        return Err(ValidationError::PriceMustBePositive);
    }
    if price > 10000000.0 {
        return Err(ValidationError::PriceTooLarge);
    }
    if price.is_nan() || price.is_infinite() {
        return Err(ValidationError::InvalidNumber);
    }
    // Check decimal places (max 8 for crypto)
    let decimal_places = format!("{:.8}", price).trim_end_matches('0').split('.').nth(1).map(|s| s.len()).unwrap_or(0);
    if decimal_places > 8 {
        return Err(ValidationError::TooManyDecimalPlaces);
    }
    Ok(())
}
```

**User ID** (MongoDB ObjectId):
```rust
fn validate_user_id(id: &str) -> Result<ObjectId> {
    ObjectId::parse_str(id)
        .map_err(|_| ValidationError::InvalidUserId)
}
```

### Injection Prevention

**SQL Injection**: Not applicable (using MongoDB, not SQL)

**NoSQL Injection** (MongoDB):
**Attack Example**:
```json
// Malicious login request
{
  "email": {"$ne": null},
  "password": {"$ne": null}
}
```
This would match any user if not properly validated.

**Prevention**:
- **Strongly typed inputs**: Use Rust structs, not raw JSON
- **Validation**: Reject objects where strings are expected
- **Parameterized queries**: Use MongoDB driver's query builders

**Safe Code**:
```rust
#[derive(Deserialize)]
struct LoginRequest {
    email: String,    // Must be a string, not an object
    password: String, // Must be a string, not an object
}

async fn login(request: LoginRequest, db: Database) -> Result<()> {
    // Safe: email and password are guaranteed to be strings
    let user = db
        .collection::<User>("users")
        .find_one(doc! { "email": request.email }, None)
        .await?;
    // ...
}
```

**XSS (Cross-Site Scripting)** Prevention:
- **React auto-escaping**: React escapes all strings by default
- **CSP**: Content-Security-Policy header blocks inline scripts
- **Input validation**: Reject HTML/JavaScript in text fields
- **Output encoding**: Encode data before rendering

**Command Injection**: Not applicable (no shell commands executed with user input)

**Path Traversal**: Not applicable (no file system access with user input)

### API Versioning

**Current Version**: v1

**URL Structure**: `/api/v1/{endpoint}`

**Versioning Strategy**:
- Major version in URL: `/api/v1/`, `/api/v2/`
- Minor/patch versions: Backward compatible, no URL change
- Deprecation: 6 months notice, maintain old version during transition

**Version Header** (optional):
```http
X-API-Version: 1.0.0
```

---

## ARCH-SECURITY-004: Network Security

**Priority**: ☑ Critical
**Status**: ☑ Partially Implemented
**Code Tags**: `@spec:ARCH-SECURITY-004`

### Description

This section describes network-level security including TLS configuration, Docker network isolation, firewall rules, and DDoS protection.

### TLS/SSL Configuration

**Protocol Version**: TLS 1.3 (minimum TLS 1.2)

**Cipher Suites** (TLS 1.3):
- `TLS_AES_256_GCM_SHA384`
- `TLS_CHACHA20_POLY1305_SHA256`
- `TLS_AES_128_GCM_SHA256`

**Certificate**:
- **Type**: Let's Encrypt (free, automated)
- **Algorithm**: RSA 2048-bit or ECDSA P-256
- **Renewal**: Automated via certbot (every 60 days)
- **Validity**: 90 days (Let's Encrypt standard)

**Certificate Pinning** (Future):
- Pin public key hash in mobile apps
- Backup pins for rotation
- 7-day pin expiration warning

**HTTPS Redirect**:
```nginx
# Redirect HTTP to HTTPS
server {
    listen 80;
    server_name api.botcore.com;
    return 301 https://$server_name$request_uri;
}
```

**HSTS Preload**:
- Domain submitted to https://hstspreload.org
- Browsers always use HTTPS (no redirect needed)

### Docker Network Isolation

**Network Architecture**:
```mermaid
graph TB
    Internet[Internet]
    LoadBalancer[Load Balancer<br/>nginx]
    BotNetwork[Docker Network: bot-network<br/>Bridge Mode]

    subgraph "Exposed Services"
        Frontend[Frontend :3000]
        RustAPI[Rust API :8080]
        PythonAPI[Python API :8000]
    end

    subgraph "Internal Services"
        MongoDB[MongoDB :27017]
        Redis[Redis :6379]
    end

    Internet --> LoadBalancer
    LoadBalancer --> Frontend
    LoadBalancer --> RustAPI
    LoadBalancer --> PythonAPI

    Frontend -.->|bot-network| RustAPI
    RustAPI -.->|bot-network| PythonAPI
    RustAPI -.->|bot-network| MongoDB
    RustAPI -.->|bot-network| Redis
    PythonAPI -.->|bot-network| MongoDB
    PythonAPI -.->|bot-network| Redis
```

**Docker Network Configuration**:
```yaml
# docker-compose.yml
networks:
  bot-network:
    driver: bridge
    ipam:
      config:
        - subnet: 172.20.0.0/16

services:
  rust-core-engine:
    networks:
      - bot-network
    ports:
      - "8080:8080"  # Exposed to host

  python-ai-service:
    networks:
      - bot-network
    ports:
      - "8000:8000"  # Exposed to host

  frontend:
    networks:
      - bot-network
    ports:
      - "3000:3000"  # Exposed to host

  mongodb:
    networks:
      - bot-network
    # No ports exposed to host (internal only)

  redis:
    networks:
      - bot-network
    # No ports exposed to host (internal only)
```

**Network Policies**:
- **Ingress**: Only load balancer can access services from Internet
- **Egress**: Services can only access specific external APIs (Binance, OpenAI)
- **Inter-service**: Services can communicate within bot-network
- **Database**: MongoDB and Redis not exposed to host network

### Firewall Rules

**Host Firewall** (iptables/ufw):

**Ingress Rules**:
```bash
# Allow HTTP (for Let's Encrypt verification)
ufw allow 80/tcp

# Allow HTTPS
ufw allow 443/tcp

# Allow SSH (admin access only, specific IPs)
ufw allow from 203.0.113.0/24 to any port 22 proto tcp

# Deny all other inbound traffic
ufw default deny incoming
```

**Egress Rules**:
```bash
# Allow all outbound traffic (services need to call external APIs)
ufw default allow outgoing

# Alternative: Restrict to specific destinations
# Allow HTTPS to Binance API
ufw allow out to 3.64.0.0/11 port 443 proto tcp

# Allow HTTPS to OpenAI API
ufw allow out to 104.18.0.0/15 port 443 proto tcp
```

**Docker Container Firewall**:
- Docker manages iptables rules automatically
- Services within bot-network can communicate freely
- Services cannot access host network directly

### DDoS Protection

**Layer 7 (Application Layer)**:
- **Rate Limiting**: 600 requests/minute per user, 100 requests/minute per IP (unauthenticated)
- **Connection Limits**: Max 1000 concurrent WebSocket connections
- **Request Size Limits**: Max 1 MB request body
- **Timeout**: 30 seconds per request

**Layer 4 (Transport Layer)**:
- **SYN Cookies**: Enable to prevent SYN flood attacks
- **Connection Tracking**: Limit new connections per second per IP

**Layer 3 (Network Layer)**:
- **Cloudflare** (recommended): DDoS protection, WAF, CDN
- **IP Blacklisting**: Block known malicious IPs
- **Geo-blocking**: Optional, block traffic from certain countries

**Mitigation Services** (Production):
- **Cloudflare**: Free plan includes basic DDoS protection
- **AWS Shield**: If hosting on AWS
- **Fail2Ban**: Automatically ban IPs after failed login attempts

### VPN Access (Production Recommendation)

**Admin Access**:
- **SSH**: Only accessible via VPN
- **MongoDB**: Only accessible via VPN
- **Redis**: Only accessible via VPN

**VPN Solution**: WireGuard
- **Performance**: Fast, modern protocol
- **Security**: Strong encryption (ChaCha20, Poly1305)
- **Ease of Use**: Simple configuration

**Configuration Example**:
```ini
# wg0.conf (server)
[Interface]
Address = 10.0.0.1/24
ListenPort = 51820
PrivateKey = <server_private_key>

[Peer]
# Admin 1
PublicKey = <admin1_public_key>
AllowedIPs = 10.0.0.2/32

[Peer]
# Admin 2
PublicKey = <admin2_public_key>
AllowedIPs = 10.0.0.3/32
```

---

## ARCH-SECURITY-005: Data Security

**Priority**: ☑ Critical
**Status**: ☑ Partially Implemented
**Code Tags**: `@spec:ARCH-SECURITY-005`

### Description

This section describes data protection measures including secrets management, encryption at rest and in transit, API key protection, and sensitive data masking in logs.

### Secrets Management

**Current Implementation**: Environment variables (`.env` file)

**Environment Variables**:
```bash
# .env file (NEVER commit to git)
JWT_SECRET=<random_64_char_string>
MONGODB_URI=mongodb://localhost:27017/botcore
BINANCE_API_KEY=<user_specific_encrypted>
BINANCE_SECRET_KEY=<user_specific_encrypted>
OPENAI_API_KEY=sk-...
RUST_LOG=info
```

**Secret Generation**:
```bash
# Generate strong random secret (64 characters)
openssl rand -base64 48

# Or using Python
python3 -c "import secrets; print(secrets.token_urlsafe(48))"
```

**Secret Rotation**:
- **JWT Secret**: Rotate every 90 days
- **Binance API Keys**: User-managed, encourage rotation
- **OpenAI API Key**: Rotate every 180 days
- **MongoDB Password**: Rotate every 90 days

**Future Enhancement: HashiCorp Vault**:
```yaml
# vault-config.hcl
storage "file" {
  path = "/vault/data"
}

listener "tcp" {
  address     = "0.0.0.0:8200"
  tls_disable = 0
  tls_cert_file = "/vault/certs/cert.pem"
  tls_key_file  = "/vault/certs/key.pem"
}

# Store secrets
vault kv put secret/botcore/jwt JWT_SECRET=abc123...
vault kv put secret/botcore/openai OPENAI_API_KEY=sk-...

# Retrieve secrets
vault kv get secret/botcore/jwt
```

**Benefits of Vault**:
- Centralized secret storage
- Audit logging (who accessed which secret when)
- Dynamic secrets (generate on-demand, expire after use)
- Secret versioning (rollback if needed)
- Encryption at rest (secrets encrypted in Vault storage)

### API Key Encryption

**User's Binance API Keys**:
- **Storage**: Encrypted in MongoDB
- **Encryption Algorithm**: AES-256-GCM
- **Key**: Derived from master secret using PBKDF2
- **IV**: Random 12 bytes per encrypted value (stored with ciphertext)

**Encryption Implementation**:
```rust
use aes_gcm::{Aes256Gcm, Key, Nonce};
use aes_gcm::aead::{Aead, NewAead};

fn encrypt_api_key(api_key: &str, master_secret: &[u8]) -> Result<String> {
    // Derive encryption key from master secret
    let key = Key::from_slice(&master_secret[..32]);
    let cipher = Aes256Gcm::new(key);

    // Generate random IV (nonce)
    let nonce_bytes: [u8; 12] = rand::random();
    let nonce = Nonce::from_slice(&nonce_bytes);

    // Encrypt
    let ciphertext = cipher.encrypt(nonce, api_key.as_bytes())
        .map_err(|_| Error::EncryptionFailed)?;

    // Combine nonce + ciphertext + tag
    let mut result = nonce_bytes.to_vec();
    result.extend_from_slice(&ciphertext);

    // Encode as base64 for storage
    Ok(base64::encode(&result))
}

fn decrypt_api_key(encrypted: &str, master_secret: &[u8]) -> Result<String> {
    // Decode from base64
    let data = base64::decode(encrypted)?;

    // Split nonce and ciphertext
    let (nonce_bytes, ciphertext) = data.split_at(12);
    let nonce = Nonce::from_slice(nonce_bytes);

    // Derive encryption key
    let key = Key::from_slice(&master_secret[..32]);
    let cipher = Aes256Gcm::new(key);

    // Decrypt
    let plaintext = cipher.decrypt(nonce, ciphertext)
        .map_err(|_| Error::DecryptionFailed)?;

    String::from_utf8(plaintext)
        .map_err(|_| Error::InvalidUtf8)
}
```

**API Key Storage in MongoDB**:
```json
{
  "_id": ObjectId,
  "user_id": ObjectId,
  "api_key_encrypted": "base64_encoded_nonce+ciphertext",
  "secret_key_encrypted": "base64_encoded_nonce+ciphertext",
  "created_at": ISODate,
  "last_used": ISODate
}
```

**Decryption Policy**:
- **On-Demand**: Keys decrypted only when needed (trade execution)
- **In-Memory**: Decrypted keys never stored, only used immediately
- **Short-Lived**: Decrypted keys discarded after use

### Encryption at Rest

**Database Encryption**:
- **MongoDB**: Not currently encrypted at rest (future enhancement)
- **Recommendation**: MongoDB Enterprise Edition with encryption
- **Alternative**: LUKS (Linux Unified Key Setup) for disk encryption

**MongoDB Encryption at Rest** (Enterprise Edition):
```yaml
# mongod.conf
security:
  enableEncryption: true
  encryptionKeyFile: /path/to/keyfile

# Generate encryption key
openssl rand -base64 32 > /path/to/keyfile
chmod 600 /path/to/keyfile
```

**File System Encryption** (LUKS):
```bash
# Encrypt disk partition
cryptsetup luksFormat /dev/sdb1
cryptsetup open /dev/sdb1 mongodb_encrypted

# Create filesystem
mkfs.ext4 /dev/mapper/mongodb_encrypted

# Mount
mount /dev/mapper/mongodb_encrypted /var/lib/mongodb
```

### Encryption in Transit

**TLS for All External Communication**:
- **Frontend ↔ Rust API**: HTTPS (TLS 1.3)
- **Rust API ↔ Python AI**: HTTP over internal Docker network (acceptable) or HTTPS (recommended)
- **Rust API ↔ MongoDB**: TLS (recommended)
- **Rust API ↔ Binance**: HTTPS (required by Binance)
- **Python AI ↔ OpenAI**: HTTPS (required by OpenAI)

**MongoDB TLS Configuration**:
```yaml
# mongod.conf
net:
  tls:
    mode: requireTLS
    certificateKeyFile: /path/to/mongodb.pem
    CAFile: /path/to/ca.pem
```

**Rust MongoDB Client with TLS**:
```rust
let client_options = ClientOptions::parse_with_resolver_config(
    "mongodb://localhost:27017/?tls=true&tlsCAFile=/path/to/ca.pem",
    ResolverConfig::cloudflare(),
)
.await?;

let client = Client::with_options(client_options)?;
```

### Sensitive Data Masking in Logs

**Data to Mask**:
- Passwords (should never be logged)
- JWT tokens
- API keys (Binance, OpenAI)
- Credit card numbers (if applicable)
- Email addresses (in some contexts)
- IP addresses (for GDPR compliance)

**Masking Implementation**:
```rust
fn mask_sensitive_data(log_message: &str) -> String {
    let mut masked = log_message.to_string();

    // Mask JWT tokens (eyJ... format)
    let jwt_regex = Regex::new(r"eyJ[a-zA-Z0-9_-]*\.eyJ[a-zA-Z0-9_-]*\.[a-zA-Z0-9_-]*").unwrap();
    masked = jwt_regex.replace_all(&masked, "***JWT_TOKEN***").to_string();

    // Mask API keys (sk-... format for OpenAI)
    let api_key_regex = Regex::new(r"sk-[a-zA-Z0-9]{32,}").unwrap();
    masked = api_key_regex.replace_all(&masked, "***API_KEY***").to_string();

    // Mask Binance API keys (64 alphanumeric characters)
    let binance_key_regex = Regex::new(r"\b[A-Za-z0-9]{64}\b").unwrap();
    masked = binance_key_regex.replace_all(&masked, "***BINANCE_KEY***").to_string();

    // Mask passwords in structured logs
    let password_regex = Regex::new(r#""password"\s*:\s*"[^"]+""#).unwrap();
    masked = password_regex.replace_all(&masked, r#""password":"***""#).to_string();

    masked
}

// Example usage in logging
info!("{}", mask_sensitive_data(&format!("User logged in with token: {}", token)));
// Output: User logged in with token: ***JWT_TOKEN***
```

**Structured Logging** (Recommended):
```rust
use tracing::{info, error};

// Safe: password not included in log
info!(
    user_id = %user_id,
    email = %email,
    "User logged in successfully"
);

// Unsafe: DON'T DO THIS
info!("User {} logged in with password {}", email, password);
```

### Data Retention

**Personal Data** (GDPR Compliance):
- **User Profile**: Retained until account deletion
- **Trade History**: 7 years (financial regulation requirement)
- **Audit Logs**: 90 days (security incident investigation)
- **Session Logs**: 30 days

**Right to Erasure** (GDPR Article 17):
- User can request account deletion
- Personal data deleted within 30 days
- Trade history anonymized (user_id replaced with random ID)
- Audit logs retained for legal compliance

**Data Anonymization**:
```rust
async fn anonymize_user_data(user_id: &ObjectId, db: &Database) -> Result<()> {
    let anonymous_id = ObjectId::new(); // Random ID

    // Update trades to use anonymous ID
    db.collection::<Trade>("trades")
        .update_many(
            doc! { "user_id": user_id },
            doc! { "$set": { "user_id": anonymous_id } },
            None,
        )
        .await?;

    // Delete user document
    db.collection::<User>("users")
        .delete_one(doc! { "_id": user_id }, None)
        .await?;

    Ok(())
}
```

---

## ARCH-SECURITY-006: Security Monitoring & Audit

**Priority**: ☑ High
**Status**: ☑ Partially Implemented
**Code Tags**: `@spec:ARCH-SECURITY-006`

### Description

This section describes security monitoring, audit logging, intrusion detection, and incident response procedures.

### Audit Logging

**Events to Log**:

**Authentication Events**:
- User registration (success/failure)
- User login (success/failure)
- User logout
- Password change
- Account lockout
- Token verification failure

**Authorization Events**:
- Access denied (403 Forbidden)
- Privilege escalation attempt
- Admin action (any action by admin user)

**Trading Events**:
- Trade executed
- Trade cancelled
- Risk check failure
- Position update
- Large trades (> $10,000)

**Configuration Events**:
- Settings changed
- API key added/removed
- User promoted to admin
- Feature flag toggled

**Security Events**:
- Rate limit exceeded
- Suspicious activity detected
- Firewall rule triggered
- TLS handshake failure

**Audit Log Format** (JSON):
```json
{
  "timestamp": "2025-10-11T10:30:00.123Z",
  "event_type": "USER_LOGIN",
  "severity": "INFO",
  "user_id": "60f7b3b3e4b0c8f7d8f7b3b3",
  "email": "user@example.com",
  "ip_address": "203.0.113.42",
  "user_agent": "Mozilla/5.0...",
  "request_id": "req_abc123",
  "result": "SUCCESS",
  "details": {
    "login_method": "password",
    "session_duration": null
  },
  "service": "rust-core-engine",
  "version": "1.0.0"
}
```

**Audit Log Storage**:
- **Destination**: MongoDB collection `audit_logs`
- **Retention**: 90 days (configurable)
- **Index**: `{timestamp: -1, event_type: 1, user_id: 1}`
- **TTL Index**: `{timestamp: 1}` with `expireAfterSeconds: 7776000` (90 days)

**Audit Log Immutability**:
- Append-only collection (no updates or deletes)
- Checksum for each log entry (future: blockchain-based)
- Separate database credentials (read-only for application)

### Security Metrics

**Key Security Metrics**:
- `auth_failed_login_attempts_total` (counter, labels: reason)
- `auth_account_lockouts_total` (counter)
- `auth_suspicious_activity_total` (counter, labels: type)
- `api_rate_limit_exceeded_total` (counter, labels: endpoint)
- `api_unauthorized_access_attempts_total` (counter, labels: endpoint)
- `trade_risk_violations_total` (counter, labels: rule)
- `security_incidents_total` (counter, labels: severity)

**Dashboards**:
- **Grafana**: Real-time security metrics visualization
- **Panels**:
  - Failed login attempts over time
  - Rate limit violations by IP
  - Unauthorized access attempts by endpoint
  - Active sessions by user
  - Suspicious activity timeline

### Security Alerts

**Alert Conditions**:

**Critical Alerts** (Immediate Response):
- 10+ failed login attempts from same IP in 5 minutes
- Admin account accessed from unknown IP
- Multiple accounts accessed from same IP (credential stuffing)
- Unusual trading pattern (e.g., 100+ trades in 1 minute)
- Database connection from unauthorized IP
- TLS certificate expiring in <7 days

**High Priority Alerts** (Response within 1 hour):
- 5+ account lockouts in 10 minutes (possible attack)
- Rate limit exceeded 100+ times in 5 minutes
- 10+ risk violations in 1 hour (bypass attempt)
- Unusual API key usage (e.g., API key used from multiple IPs)

**Medium Priority Alerts** (Response within 24 hours):
- Failed login rate >5% over 1 hour
- Unauthorized access attempts >10 in 1 hour
- Large number of password reset requests

**Alert Channels**:
- **Email**: security@botcore.com
- **Slack**: #security-alerts channel
- **PagerDuty**: On-call engineer (critical alerts only)
- **SMS**: Admin phone (critical alerts only)

### Intrusion Detection

**Intrusion Detection System (IDS)** (Future):
- **Tool**: Snort or Suricata
- **Deployment**: On host or in Docker container
- **Rules**: OWASP CRS (Core Rule Set)

**Anomaly Detection** (Future):
- **Machine Learning**: Detect unusual patterns
- **Metrics**: Login times, IP locations, API usage
- **Example**: User logs in from USA, then Russia 10 minutes later → Alert

**File Integrity Monitoring** (Future):
- **Tool**: AIDE (Advanced Intrusion Detection Environment)
- **Monitor**: `/etc`, `/usr/bin`, application directories
- **Alert**: File modification, unexpected new files

### Incident Response

**Incident Response Plan**:

**1. Detection**:
- Alert triggered (automated)
- User report (manual)
- Security researcher disclosure (responsible disclosure policy)

**2. Triage** (within 15 minutes):
- Assess severity (Critical, High, Medium, Low)
- Determine scope (how many users affected?)
- Identify impacted systems

**3. Containment** (within 1 hour):
- **Account Compromise**: Lock account, revoke tokens
- **API Key Leak**: Revoke key, notify user
- **DDoS Attack**: Enable Cloudflare protection, rate limiting
- **Data Breach**: Isolate affected database, take offline if necessary

**4. Eradication** (within 24 hours):
- Remove malicious code
- Patch vulnerability
- Change compromised credentials
- Reset all user passwords (if needed)

**5. Recovery** (within 48 hours):
- Restore service
- Monitor for recurrence
- Verify no backdoors remain

**6. Post-Incident** (within 7 days):
- Root cause analysis
- Update security controls
- Notify affected users (if required by law)
- Improve monitoring/alerting

**Incident Severity Levels**:

**Critical** (P0):
- Active data breach
- All users locked out
- Database compromised
- **Response Time**: Immediate (within 15 minutes)

**High** (P1):
- Single account compromised
- Vulnerability discovered (not yet exploited)
- Partial service outage
- **Response Time**: Within 1 hour

**Medium** (P2):
- Potential vulnerability
- Failed attack attempt
- Minor service degradation
- **Response Time**: Within 24 hours

**Low** (P3):
- Security best practice violation
- Minor misconfiguration
- **Response Time**: Within 7 days

---

## ARCH-SECURITY-007: Vulnerability Management

**Priority**: ☑ High
**Status**: ☑ Partially Implemented
**Code Tags**: `@spec:ARCH-SECURITY-007`

### Description

This section describes vulnerability scanning, dependency management, patch management, and security testing procedures.

### Dependency Scanning

**Tools**:

**Rust Dependencies**:
- **Tool**: `cargo audit`
- **Frequency**: Weekly (automated in CI/CD)
- **Action**: Update vulnerable dependencies within 7 days (High), 30 days (Medium)

**Python Dependencies**:
- **Tool**: `pip-audit` or `safety`
- **Frequency**: Weekly (automated in CI/CD)
- **Action**: Update vulnerable dependencies within 7 days (High), 30 days (Medium)

**Node.js Dependencies**:
- **Tool**: `npm audit`
- **Frequency**: Weekly (automated in CI/CD)
- **Action**: Update vulnerable dependencies within 7 days (High), 30 days (Medium)

**Example CI/CD Pipeline** (GitHub Actions):
```yaml
name: Security Scan

on:
  schedule:
    - cron: '0 0 * * 0'  # Weekly on Sunday
  push:
    branches: [main]

jobs:
  rust-audit:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3
      - name: Install cargo-audit
        run: cargo install cargo-audit
      - name: Run cargo audit
        run: cd rust-core-engine && cargo audit
      - name: Report vulnerabilities
        if: failure()
        uses: actions/github-script@v6
        with:
          script: |
            github.rest.issues.create({
              owner: context.repo.owner,
              repo: context.repo.repo,
              title: 'Security: Rust dependencies have vulnerabilities',
              body: 'cargo audit failed. Please review and update dependencies.',
              labels: ['security', 'dependencies']
            })

  python-audit:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3
      - name: Set up Python
        uses: actions/setup-python@v4
        with:
          python-version: '3.11'
      - name: Install pip-audit
        run: pip install pip-audit
      - name: Run pip-audit
        run: cd python-ai-service && pip-audit
      - name: Report vulnerabilities
        if: failure()
        uses: actions/github-script@v6
        with:
          script: |
            github.rest.issues.create({
              owner: context.repo.owner,
              repo: context.repo.repo,
              title: 'Security: Python dependencies have vulnerabilities',
              body: 'pip-audit failed. Please review and update dependencies.',
              labels: ['security', 'dependencies']
            })

  npm-audit:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3
      - name: Set up Node.js
        uses: actions/setup-node@v3
        with:
          node-version: '18'
      - name: Run npm audit
        run: cd nextjs-ui-dashboard && npm audit --audit-level=high
      - name: Report vulnerabilities
        if: failure()
        uses: actions/github-script@v6
        with:
          script: |
            github.rest.issues.create({
              owner: context.repo.owner,
              repo: context.repo.repo,
              title: 'Security: npm dependencies have vulnerabilities',
              body: 'npm audit failed. Please review and update dependencies.',
              labels: ['security', 'dependencies']
            })
```

**Current Status**:
- **Rust**: 0 HIGH/CRITICAL vulnerabilities (as of last scan)
- **Python**: 0 HIGH/CRITICAL vulnerabilities (as of last scan)
- **Node.js**: 0 HIGH/CRITICAL vulnerabilities (as of last scan)

### Patch Management

**Patch Prioritization**:

**Critical Vulnerabilities** (CVSS 9.0-10.0):
- **Action**: Emergency patch within 24 hours
- **Deployment**: Immediate production release
- **Communication**: Notify all users of scheduled maintenance

**High Vulnerabilities** (CVSS 7.0-8.9):
- **Action**: Patch within 7 days
- **Deployment**: Include in next regular release
- **Communication**: Internal team notification

**Medium Vulnerabilities** (CVSS 4.0-6.9):
- **Action**: Patch within 30 days
- **Deployment**: Include in monthly maintenance release
- **Communication**: Internal team notification

**Low Vulnerabilities** (CVSS 0.1-3.9):
- **Action**: Patch within 90 days or next major release
- **Deployment**: Batch with other updates
- **Communication**: Not required

**Patch Testing**:
1. Apply patch in development environment
2. Run automated tests (unit, integration, E2E)
3. Manual security testing (verify fix)
4. Deploy to staging
5. Monitor for 24 hours
6. Deploy to production

### Security Testing

**Testing Types**:

**1. Static Application Security Testing (SAST)**:
- **Rust**: `cargo clippy` (linter with security checks)
- **Python**: `bandit` (security linter)
- **Frequency**: Every commit (CI/CD)

**Example** (bandit for Python):
```bash
# Install
pip install bandit

# Run scan
bandit -r python-ai-service/ -f json -o bandit-report.json

# Common issues detected:
# - Use of assert in production code
# - Hardcoded passwords
# - SQL injection (not applicable for MongoDB)
# - Insecure cryptography
```

**2. Dynamic Application Security Testing (DAST)**:
- **Tool**: OWASP ZAP (Zed Attack Proxy)
- **Target**: Running application (staging environment)
- **Frequency**: Weekly automated scan
- **Checks**:
  - SQL injection (not applicable)
  - XSS (Cross-Site Scripting)
  - CSRF (Cross-Site Request Forgery)
  - Broken authentication
  - Sensitive data exposure
  - Missing security headers

**Example** (OWASP ZAP):
```bash
# Docker-based scan
docker run -v $(pwd):/zap/wrk/:rw owasp/zap2docker-stable zap-full-scan.py \
  -t https://staging.botcore.com \
  -r zap-report.html
```

**3. Penetration Testing**:
- **Frequency**: Annual
- **Scope**: Full application (frontend, backend, APIs)
- **Method**: External security firm (or internal red team)
- **Deliverable**: Detailed report with findings and recommendations

**Penetration Test Scope**:
- Authentication bypass attempts
- Authorization vulnerabilities
- API security (rate limiting, input validation)
- Session management
- Cryptography implementation
- Business logic flaws
- Infrastructure security

**4. Security Code Review**:
- **Frequency**: Every pull request (for security-sensitive code)
- **Reviewers**: Senior engineers + security specialist
- **Focus Areas**:
  - Authentication logic
  - Authorization checks
  - Cryptography usage
  - Input validation
  - SQL/NoSQL query construction
  - File uploads (if applicable)

**5. Fuzz Testing** (Future):
- **Tool**: AFL (American Fuzzy Lop) or libFuzzer
- **Target**: Input parsers (JSON, WebSocket messages)
- **Goal**: Discover crashes, hangs, memory leaks

### OWASP Top 10 Compliance

**OWASP Top 10 (2021)**:

**A01:2021 – Broken Access Control**:
- **Mitigation**: JWT-based authentication, RBAC authorization, resource-level checks
- **Status**: ✅ Implemented

**A02:2021 – Cryptographic Failures**:
- **Mitigation**: TLS 1.3, bcrypt password hashing, AES-256 for API keys
- **Status**: ✅ Implemented

**A03:2021 – Injection**:
- **Mitigation**: Parameterized MongoDB queries, input validation, no SQL
- **Status**: ✅ Implemented

**A04:2021 – Insecure Design**:
- **Mitigation**: Threat modeling, secure architecture review
- **Status**: ✅ Implemented

**A05:2021 – Security Misconfiguration**:
- **Mitigation**: Secure defaults, security headers, minimal services
- **Status**: ✅ Implemented

**A06:2021 – Vulnerable and Outdated Components**:
- **Mitigation**: Weekly dependency scanning, automated updates
- **Status**: ✅ Implemented

**A07:2021 – Identification and Authentication Failures**:
- **Mitigation**: Strong password policy, account lockout, MFA (future)
- **Status**: ✅ Implemented (MFA pending)

**A08:2021 – Software and Data Integrity Failures**:
- **Mitigation**: Audit logging, checksum verification
- **Status**: ⚠️ Partially implemented (checksum verification pending)

**A09:2021 – Security Logging and Monitoring Failures**:
- **Mitigation**: Comprehensive audit logging, security alerts
- **Status**: ✅ Implemented

**A10:2021 – Server-Side Request Forgery (SSRF)**:
- **Mitigation**: URL whitelist for external requests, no user-controlled URLs
- **Status**: ✅ Not applicable (no user-controlled external requests)

---

## ARCH-SECURITY-008: Compliance & Privacy

**Priority**: ☑ Medium
**Status**: ☐ Not Started
**Code Tags**: `@spec:ARCH-SECURITY-008`

### Description

This section describes compliance with regulations (GDPR, CCPA) and privacy protection measures.

### GDPR Compliance

**General Data Protection Regulation** (EU):

**Key Requirements**:

**1. Lawful Basis for Processing**:
- **Consent**: Users explicitly consent to data processing during registration
- **Contract**: Data processing necessary for providing trading services
- **Legitimate Interest**: Security logging, fraud prevention

**2. Data Minimization**:
- Collect only necessary data (email, password, trading history)
- Don't collect unnecessary PII (address, phone number, unless required)

**3. Purpose Limitation**:
- Data used only for stated purposes (trading, account management)
- No selling or sharing with third parties (except processors like Binance)

**4. Right to Access** (Article 15):
- Users can request copy of their data
- Provided in machine-readable format (JSON)
- Within 30 days of request

**5. Right to Erasure** (Article 17):
- Users can request account deletion
- Personal data deleted within 30 days
- Exceptions: Legal obligations (trade records for 7 years)

**6. Right to Data Portability** (Article 20):
- Users can export their data
- Provided in JSON format
- Includes: Profile, trades, positions, settings

**7. Data Breach Notification** (Article 33):
- Notify supervisory authority within 72 hours
- Notify affected users if high risk
- Document all breaches

**8. Data Protection Officer** (DPO):
- Appoint DPO if processing large amounts of personal data
- Contact: dpo@botcore.com (future)

**Implementation**:

**Consent Management**:
```typescript
// Registration form
<Checkbox required>
  I consent to the processing of my personal data as described in the
  <Link to="/privacy-policy">Privacy Policy</Link>
</Checkbox>
```

**Data Export**:
```rust
async fn export_user_data(user_id: &ObjectId, db: &Database) -> Result<UserDataExport> {
    let user = get_user(user_id, db).await?;
    let trades = get_user_trades(user_id, db).await?;
    let positions = get_user_positions(user_id, db).await?;

    Ok(UserDataExport {
        profile: user,
        trades,
        positions,
        exported_at: Utc::now(),
    })
}

// Endpoint
GET /api/account/export
Authorization: Bearer {token}

// Response
{
  "profile": {...},
  "trades": [...],
  "positions": [...],
  "exported_at": "2025-10-11T10:30:00Z"
}
```

**Data Deletion**:
```rust
async fn delete_user_account(user_id: &ObjectId, db: &Database) -> Result<()> {
    // Anonymize trade history (legal requirement to keep)
    anonymize_user_data(user_id, db).await?;

    // Delete user profile
    db.collection::<User>("users")
        .delete_one(doc! { "_id": user_id }, None)
        .await?;

    // Delete API keys
    db.collection::<ApiKey>("api_keys")
        .delete_many(doc! { "user_id": user_id }, None)
        .await?;

    // Audit log
    log_audit_event("USER_DELETED", user_id);

    Ok(())
}
```

### Privacy Policy

**Required Disclosures**:
- What data is collected (email, password, trading activity)
- Why data is collected (provide trading services)
- How data is used (account management, trade execution, AI analysis)
- Who data is shared with (Binance for trade execution, OpenAI for AI analysis)
- How long data is retained (trade history: 7 years, profile: until deletion)
- User rights (access, erasure, portability, objection)
- Contact information (privacy@botcore.com)

**Cookie Policy**:
- Authentication token stored in localStorage (not cookies)
- Analytics cookies (Google Analytics, if used) - require consent
- No third-party tracking cookies

---

## ARCH-SECURITY-009: Security Architecture Diagram

**Priority**: ☑ Critical
**Status**: ☑ Completed
**Code Tags**: `@spec:ARCH-SECURITY-009`

### Description

This section provides a comprehensive security architecture diagram showing all security controls and their relationships.

### Complete Security Architecture

```mermaid
graph TB
    Internet[Internet<br/>Attackers, Users]
    CloudFlare[Cloudflare<br/>DDoS Protection<br/>WAF<br/>CDN]
    LoadBalancer[Load Balancer<br/>nginx<br/>TLS Termination<br/>Rate Limiting]

    subgraph "DMZ - Exposed Services"
        Frontend[Frontend :3000<br/>React + TypeScript<br/>CSP Headers<br/>XSS Protection]
        RustAPI[Rust API :8080<br/>JWT Auth<br/>RBAC<br/>Input Validation]
        PythonAPI[Python API :8000<br/>FastAPI<br/>Rate Limiting<br/>Input Validation]
    end

    subgraph "Security Layer"
        AuthService[Authentication Service<br/>bcrypt Password Hashing<br/>JWT Generation<br/>Account Lockout]
        AuthzService[Authorization Service<br/>RBAC Enforcement<br/>Resource Access Control]
        RateLimiter[Rate Limiter<br/>Token Bucket<br/>Per-User, Per-IP]
        InputValidator[Input Validator<br/>Schema Validation<br/>Sanitization]
        Encryption[Encryption Service<br/>AES-256-GCM<br/>API Key Encryption]
    end

    subgraph "Internal Network - bot-network"
        MongoDB[(MongoDB<br/>No external access<br/>Audit Logs<br/>Encrypted API Keys)]
        Redis[(Redis<br/>Session Store<br/>Rate Limit Counters)]
    end

    subgraph "External Services"
        Binance[Binance API<br/>HTTPS Only<br/>Encrypted API Keys]
        OpenAI[OpenAI API<br/>HTTPS Only<br/>API Key in Env]
    end

    subgraph "Monitoring & Audit"
        AuditLog[Audit Logger<br/>All Auth Events<br/>All Trades<br/>Immutable]
        SecurityMetrics[Security Metrics<br/>Failed Logins<br/>Rate Limit Violations<br/>Unauthorized Access]
        AlertManager[Alert Manager<br/>Critical: PagerDuty<br/>High: Slack<br/>Medium: Email]
    end

    subgraph "Vulnerability Management"
        DepScanner[Dependency Scanner<br/>cargo audit<br/>pip-audit<br/>npm audit]
        SAST[SAST<br/>clippy<br/>bandit<br/>ESLint]
        DAST[DAST<br/>OWASP ZAP<br/>Weekly Scans]
    end

    Internet --> CloudFlare
    CloudFlare --> LoadBalancer
    LoadBalancer --> Frontend
    LoadBalancer --> RustAPI
    LoadBalancer --> PythonAPI

    Frontend --> AuthService
    RustAPI --> AuthService
    RustAPI --> AuthzService
    RustAPI --> RateLimiter
    RustAPI --> InputValidator
    RustAPI --> Encryption

    PythonAPI --> AuthService
    PythonAPI --> RateLimiter
    PythonAPI --> InputValidator

    AuthService --> MongoDB
    AuthzService --> MongoDB
    RateLimiter --> Redis
    Encryption --> MongoDB

    RustAPI --> MongoDB
    PythonAPI --> MongoDB

    RustAPI --> Binance
    PythonAPI --> OpenAI

    AuthService --> AuditLog
    AuthzService --> AuditLog
    RustAPI --> AuditLog
    PythonAPI --> AuditLog

    AuditLog --> SecurityMetrics
    SecurityMetrics --> AlertManager

    RustAPI -.->|Weekly Scan| DepScanner
    PythonAPI -.->|Weekly Scan| DepScanner
    Frontend -.->|Weekly Scan| DepScanner

    RustAPI -.->|Every Commit| SAST
    PythonAPI -.->|Every Commit| SAST

    RustAPI -.->|Weekly| DAST

    style Internet fill:#f9f,stroke:#333
    style CloudFlare fill:#9cf,stroke:#333
    style AuthService fill:#9f9,stroke:#333
    style AuthzService fill:#9f9,stroke:#333
    style Encryption fill:#9f9,stroke:#333
    style AuditLog fill:#fc9,stroke:#333
    style MongoDB fill:#ff9,stroke:#333
    style Binance fill:#f99,stroke:#333
    style OpenAI fill:#f99,stroke:#333
```

---

## Summary

This comprehensive security architecture document covers all critical aspects of security for the Bot Core cryptocurrency trading platform:

**Authentication** (ARCH-SECURITY-001):
- JWT-based stateless authentication
- bcrypt password hashing (cost 12)
- 7-day token expiration
- Account lockout after 5 failed attempts
- Email enumeration prevention

**Authorization** (ARCH-SECURITY-002):
- Role-based access control (user, admin)
- Resource-level authorization (users can only access own data)
- Middleware enforcement (with_auth, with_admin_auth)
- Permission matrix documented

**API Security** (ARCH-SECURITY-003):
- Rate limiting (60 trades/min, 600 API calls/min per user)
- CORS configuration (no wildcards)
- Security headers (CSP, HSTS, X-Frame-Options, etc.)
- Input validation (whitelist approach)
- Injection prevention (parameterized queries)

**Network Security** (ARCH-SECURITY-004):
- TLS 1.3 for all external communication
- Docker network isolation (bot-network)
- Firewall rules (allow 80, 443, 22 from specific IPs)
- DDoS protection (Cloudflare recommended)
- VPN access for admin (WireGuard recommended)

**Data Security** (ARCH-SECURITY-005):
- Secrets management (environment variables, future Vault)
- API key encryption (AES-256-GCM)
- Encryption at rest (future MongoDB Enterprise)
- Encryption in transit (TLS for all external communication)
- Sensitive data masking in logs (JWT tokens, API keys)

**Security Monitoring & Audit** (ARCH-SECURITY-006):
- Comprehensive audit logging (all auth events, trades, config changes)
- Security metrics (Prometheus + Grafana)
- Security alerts (email, Slack, PagerDuty)
- Intrusion detection (future: Snort/Suricata)
- Incident response plan (detect, triage, contain, eradicate, recover)

**Vulnerability Management** (ARCH-SECURITY-007):
- Dependency scanning (cargo audit, pip-audit, npm audit - weekly)
- Patch management (critical: 24h, high: 7d, medium: 30d)
- Security testing (SAST, DAST, penetration testing)
- OWASP Top 10 compliance (all items addressed)

**Compliance & Privacy** (ARCH-SECURITY-008):
- GDPR compliance (right to access, erasure, portability)
- Privacy policy (data collection, usage, retention)
- Cookie policy (localStorage for tokens, consent for analytics)

**Security Architecture Diagram** (ARCH-SECURITY-009):
- Comprehensive diagram showing all security controls
- Defense-in-depth layers
- External dependencies (Binance, OpenAI)

**Current Security Posture**:
- ✅ Strong authentication (bcrypt, JWT)
- ✅ Authorization enforced (RBAC)
- ✅ API security (rate limiting, CORS, headers)
- ✅ Input validation (all endpoints)
- ✅ Audit logging (comprehensive)
- ⚠️ Encryption at rest (future: MongoDB Enterprise)
- ⚠️ MFA (future enhancement)
- ⚠️ IDS (future: Snort/Suricata)
- ⚠️ Secrets management (future: HashiCorp Vault)

**Risk Assessment**: Medium
- **Strengths**: Strong authentication, comprehensive audit logging, no critical vulnerabilities
- **Weaknesses**: No encryption at rest, no MFA, secrets in environment variables
- **Recommendations**: Implement MongoDB encryption, add MFA, migrate to Vault

This security architecture ensures the Bot Core platform meets industry standards for cryptocurrency trading platforms while providing a clear roadmap for future security enhancements.
