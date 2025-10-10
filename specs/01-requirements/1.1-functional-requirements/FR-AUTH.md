# Authentication & Authorization - Functional Requirements

**Spec ID**: FR-AUTH
**Version**: 1.0
**Status**: ☑ Implemented
**Owner**: Backend Team
**Last Updated**: 2025-10-10

---

## Tasks Checklist

- [x] JWT token generation implemented
- [x] User registration with password hashing
- [x] User login with credential verification
- [x] Token verification middleware
- [x] Profile retrieval endpoint
- [x] Authorization middleware (user/admin)
- [x] Frontend authentication context
- [x] Login/Register UI pages
- [x] Token storage and expiry handling
- [x] Database user repository with MongoDB
- [ ] Password reset functionality
- [ ] Email verification
- [ ] Two-factor authentication (2FA)
- [ ] Session management and refresh tokens
- [ ] Rate limiting for auth endpoints
- [ ] Account lockout after failed attempts

---

## Metadata

**Related Specs**:
- Related API: [API_SPEC.md - Authentication Endpoints](../../02-technical-specs/2.1-api-specifications/API_SPEC.md)
- Related Data Models: [DATA_MODELS.md - User Models](../../02-technical-specs/2.2-data-models/DATA_MODELS.md)
- Related Security: [NFR-SEC - Security Requirements](../1.2-non-functional-requirements/NFR-SEC.md)

**Dependencies**:
- MongoDB Database (user collection with unique email index)
- JWT library (jsonwebtoken in Rust)
- bcrypt for password hashing
- Warp web framework for Rust
- React Context API for frontend state management

**Business Value**: Critical
**Technical Complexity**: Medium
**Priority**: ☑ Critical

---

## Overview

This specification defines all authentication and authorization functionality for the Bot Core trading platform. The system uses JWT (JSON Web Tokens) for stateless authentication, bcrypt for secure password hashing, and role-based access control (RBAC) for authorization. The implementation spans three layers: Rust backend API, frontend TypeScript/React UI, and MongoDB persistence.

**Architecture**: Token-based authentication with Bearer tokens, 7-day token expiration, and role-based authorization (User/Admin roles).

---

## Business Context

**Problem Statement**:
The trading platform requires secure user authentication to protect user accounts, trading data, and administrative functions. Without proper authentication, unauthorized users could access sensitive trading information, modify strategies, or execute fraudulent trades.

**Business Goals**:
- Secure access control for all platform features
- Prevent unauthorized access to user accounts and trading data
- Support role-based permissions (regular users vs administrators)
- Provide seamless user experience with persistent sessions
- Comply with security best practices for financial applications

**Success Metrics**:
- Zero unauthorized access incidents
- Login success rate > 99%
- Token verification latency < 10ms
- Password hashing time < 500ms
- User registration completion rate > 95%

---

## Functional Requirements

### FR-AUTH-001: JWT Token Generation

**Priority**: ☑ Critical
**Status**: ☑ Completed
**Code Tags**: `@spec:FR-AUTH-001`

**Description**:
Generate cryptographically secure JWT tokens for authenticated users containing user identification, email, admin status, and expiration time. Tokens are signed with HS256 algorithm using a secret key from environment variables.

**Acceptance Criteria**:
- [x] Token includes user_id (sub claim), email, is_admin flag
- [x] Token expires in 7 days (configurable: 24 * 7 hours)
- [x] Token is signed with HS256 algorithm
- [x] Token includes issued_at (iat) timestamp
- [x] Token includes expiration (exp) timestamp
- [x] Secret key is loaded from environment configuration
- [x] Token generation returns Result<String> for error handling
- [x] Generated tokens can be successfully verified

**Dependencies**: None
**Test Cases**: TC-AUTH-001, TC-AUTH-002, TC-AUTH-003

**Code Locations**:
- `/Users/dungngo97/Documents/bot-core/rust-core-engine/src/auth/jwt.rs:31-51` - Token generation function
- `/Users/dungngo97/Documents/bot-core/rust-core-engine/src/auth/jwt.rs:8-15` - Claims structure definition
- `/Users/dungngo97/Documents/bot-core/rust-core-engine/src/auth/jwt.rs:17-29` - JwtService initialization with configurable expiration

**Implementation Details**:
```rust
pub fn generate_token(&self, user_id: &str, email: &str, is_admin: bool) -> Result<String> {
    let now = Utc::now();
    let exp = now + Duration::hours(self.expiration_hours);

    let claims = Claims {
        sub: user_id.to_string(),
        email: email.to_string(),
        is_admin,
        exp: exp.timestamp(),
        iat: now.timestamp(),
    };

    let header = Header::new(Algorithm::HS256);
    let token = encode(&header, &claims, &EncodingKey::from_secret(self.secret.as_ref()))?;

    Ok(token)
}
```

---

### FR-AUTH-002: JWT Token Verification

**Priority**: ☑ Critical
**Status**: ☑ Completed
**Code Tags**: `@spec:FR-AUTH-002`

**Description**:
Verify JWT tokens received in API requests, validate signature, check expiration, and extract claims. Tokens must be provided in the Authorization header with "Bearer " prefix.

**Acceptance Criteria**:
- [x] Verify token signature using secret key
- [x] Validate token has not expired (exp > current_time)
- [x] Extract and return Claims (user_id, email, is_admin)
- [x] Return error for invalid or expired tokens
- [x] Support extraction from "Bearer {token}" authorization header
- [x] Case-sensitive "Bearer" prefix (not "bearer")
- [x] Return AuthError::InvalidToken for invalid tokens
- [x] Return AuthError::InvalidHeader for malformed headers

**Dependencies**: FR-AUTH-001
**Test Cases**: TC-AUTH-004, TC-AUTH-005, TC-AUTH-006

**Code Locations**:
- `/Users/dungngo97/Documents/bot-core/rust-core-engine/src/auth/jwt.rs:53-62` - Token verification function
- `/Users/dungngo97/Documents/bot-core/rust-core-engine/src/auth/jwt.rs:64-66` - Header token extraction
- `/Users/dungngo97/Documents/bot-core/rust-core-engine/src/auth/middleware.rs:31-43` - Authorization middleware
- `/Users/dungngo97/Documents/bot-core/rust-core-engine/src/auth/middleware.rs:78-86` - AuthError types

---

### FR-AUTH-003: Password Hashing (Bcrypt)

**Priority**: ☑ Critical
**Status**: ☑ Completed
**Code Tags**: `@spec:FR-AUTH-003`

**Description**:
Hash user passwords using bcrypt with default cost factor (10) for secure storage. Each password generates a unique hash due to random salt. Never store plain-text passwords.

**Acceptance Criteria**:
- [x] Hash passwords with bcrypt algorithm
- [x] Use DEFAULT_COST (cost factor 10)
- [x] Generate unique hash for same password (random salt)
- [x] Hash generation returns Result<String>
- [x] Hashing time < 500ms for acceptable UX
- [x] Verify password against hash returns Result<bool>
- [x] Wrong passwords return false, not error
- [x] Empty passwords can be hashed and verified

**Dependencies**: None
**Test Cases**: TC-AUTH-007, TC-AUTH-008

**Code Locations**:
- `/Users/dungngo97/Documents/bot-core/rust-core-engine/src/auth/jwt.rs:69-82` - PasswordService implementation
- `/Users/dungngo97/Documents/bot-core/rust-core-engine/src/auth/handlers.rs:133-145` - Password hashing in registration
- `/Users/dungngo97/Documents/bot-core/rust-core-engine/src/auth/handlers.rs:259-323` - Password verification in login

**Implementation Details**:
```rust
pub struct PasswordService;

impl PasswordService {
    pub fn hash_password(password: &str) -> Result<String> {
        let hashed = bcrypt::hash(password, bcrypt::DEFAULT_COST)?;
        Ok(hashed)
    }

    pub fn verify_password(password: &str, hash: &str) -> Result<bool> {
        let is_valid = bcrypt::verify(password, hash)?;
        Ok(is_valid)
    }
}
```

---

### FR-AUTH-004: User Registration

**Priority**: ☑ Critical
**Status**: ☑ Completed
**Code Tags**: `@spec:FR-AUTH-004`

**Description**:
Register new users with email, password, and optional full name. Validate input, check email uniqueness, hash password, create user in database, and return JWT token.

**Acceptance Criteria**:
- [x] Validate email format (RFC 5322 compliant)
- [x] Validate password minimum length (6 characters)
- [x] Check email uniqueness before registration
- [x] Return 409 CONFLICT if email already exists
- [x] Hash password with bcrypt before storage
- [x] Create user with default settings (trading_enabled: false)
- [x] Set is_active: true, is_admin: false for new users
- [x] Generate JWT token for immediate login
- [x] Return 201 CREATED on success with token and user profile
- [x] Return 400 BAD_REQUEST for validation errors
- [x] Support optional full_name field
- [x] Set created_at and updated_at timestamps

**Dependencies**: FR-AUTH-001, FR-AUTH-003
**Test Cases**: TC-AUTH-009, TC-AUTH-010, TC-AUTH-011

**API Endpoint**: `POST /api/auth/register`

**Code Locations**:
- `/Users/dungngo97/Documents/bot-core/rust-core-engine/src/auth/handlers.rs:85-200` - Registration handler
- `/Users/dungngo97/Documents/bot-core/rust-core-engine/src/auth/models.rs:115-122` - RegisterRequest model
- `/Users/dungngo97/Documents/bot-core/rust-core-engine/src/auth/models.rs:174-189` - User::new() constructor
- `/Users/dungngo97/Documents/bot-core/rust-core-engine/src/auth/database.rs:46-59` - UserRepository::create_user()
- `/Users/dungngo97/Documents/bot-core/nextjs-ui-dashboard/src/contexts/AuthContext.tsx:80-109` - Frontend register function
- `/Users/dungngo97/Documents/bot-core/nextjs-ui-dashboard/src/pages/Register.tsx:28-76` - Registration UI form

**Request Format**:
```json
{
  "email": "user@example.com",
  "password": "securePassword123",
  "full_name": "John Doe"  // optional
}
```

**Response Format** (201 CREATED):
```json
{
  "success": true,
  "data": {
    "token": "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9...",
    "user": {
      "id": "507f1f77bcf86cd799439011",
      "email": "user@example.com",
      "full_name": "John Doe",
      "is_active": true,
      "is_admin": false,
      "created_at": "2025-10-10T12:00:00Z",
      "last_login": null,
      "settings": {
        "trading_enabled": false,
        "risk_level": "Medium",
        "max_positions": 3,
        "default_quantity": 0.01,
        "notifications": {
          "email_alerts": true,
          "trade_notifications": true,
          "system_alerts": true
        }
      }
    }
  }
}
```

---

### FR-AUTH-005: User Login

**Priority**: ☑ Critical
**Status**: ☑ Completed
**Code Tags**: `@spec:FR-AUTH-005`

**Description**:
Authenticate users with email and password credentials. Verify password hash, check account status, update last login timestamp, and issue JWT token.

**Acceptance Criteria**:
- [x] Validate email format
- [x] Validate password is not empty
- [x] Find user by email in database
- [x] Return 401 UNAUTHORIZED if user not found
- [x] Check user.is_active flag
- [x] Return 403 FORBIDDEN if account deactivated
- [x] Verify password against stored hash
- [x] Return 401 UNAUTHORIZED for incorrect password
- [x] Update last_login timestamp on successful login
- [x] Generate JWT token with user claims
- [x] Return 200 OK with token and user profile
- [x] Generic error message for failed auth (don't reveal if email exists)
- [x] Log login attempts for security audit

**Dependencies**: FR-AUTH-001, FR-AUTH-003
**Test Cases**: TC-AUTH-012, TC-AUTH-013, TC-AUTH-014

**API Endpoint**: `POST /api/auth/login`

**Code Locations**:
- `/Users/dungngo97/Documents/bot-core/rust-core-engine/src/auth/handlers.rs:202-324` - Login handler
- `/Users/dungngo97/Documents/bot-core/rust-core-engine/src/auth/models.rs:124-130` - LoginRequest model
- `/Users/dungngo97/Documents/bot-core/rust-core-engine/src/auth/database.rs:61-70` - UserRepository::find_by_email()
- `/Users/dungngo97/Documents/bot-core/rust-core-engine/src/auth/database.rs:103-119` - UserRepository::update_last_login()
- `/Users/dungngo97/Documents/bot-core/nextjs-ui-dashboard/src/contexts/AuthContext.tsx:57-78` - Frontend login function
- `/Users/dungngo97/Documents/bot-core/nextjs-ui-dashboard/src/pages/Login.tsx:26-60` - Login UI form

**Request Format**:
```json
{
  "email": "user@example.com",
  "password": "securePassword123"
}
```

**Response Format** (200 OK):
```json
{
  "success": true,
  "data": {
    "token": "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9...",
    "user": {
      "id": "507f1f77bcf86cd799439011",
      "email": "user@example.com",
      "full_name": "John Doe",
      "is_active": true,
      "is_admin": false,
      "created_at": "2025-10-10T12:00:00Z",
      "last_login": "2025-10-10T14:30:00Z",
      "settings": { ... }
    }
  }
}
```

---

### FR-AUTH-006: Token Verification Endpoint

**Priority**: ☐ High
**Status**: ☑ Completed
**Code Tags**: `@spec:FR-AUTH-006`

**Description**:
Provide endpoint to verify JWT token validity and retrieve token claims without requiring database lookup. Used by frontend to check authentication status.

**Acceptance Criteria**:
- [x] Accept Authorization header with Bearer token
- [x] Verify token signature and expiration
- [x] Return token claims (user_id, email, is_admin, exp)
- [x] Return 401 UNAUTHORIZED for invalid/expired tokens
- [x] Return 401 for missing or malformed Authorization header
- [x] No database query required (stateless verification)
- [x] Response includes expiration timestamp

**Dependencies**: FR-AUTH-002
**Test Cases**: TC-AUTH-015, TC-AUTH-016

**API Endpoint**: `GET /api/auth/verify`

**Code Locations**:
- `/Users/dungngo97/Documents/bot-core/rust-core-engine/src/auth/handlers.rs:326-364` - Verify handler
- `/Users/dungngo97/Documents/bot-core/rust-core-engine/src/auth/handlers.rs:66-73` - Verify route definition
- `/Users/dungngo97/Documents/bot-core/nextjs-ui-dashboard/src/services/api.ts:669-684` - Frontend verifyToken()

**Response Format** (200 OK):
```json
{
  "success": true,
  "data": {
    "user_id": "507f1f77bcf86cd799439011",
    "email": "user@example.com",
    "is_admin": false,
    "exp": 1728604800
  }
}
```

---

### FR-AUTH-007: User Profile Retrieval

**Priority**: ☐ High
**Status**: ☑ Completed
**Code Tags**: `@spec:FR-AUTH-007`

**Description**:
Retrieve full user profile from database using authenticated JWT token. Returns complete user information including settings and preferences.

**Acceptance Criteria**:
- [x] Require valid JWT token in Authorization header
- [x] Extract user_id from token claims
- [x] Validate user_id is valid MongoDB ObjectId format
- [x] Query user from database by ID
- [x] Return 404 NOT_FOUND if user doesn't exist
- [x] Return 400 BAD_REQUEST for invalid ObjectId format
- [x] Return full UserProfile including settings
- [x] Exclude sensitive fields (password_hash)
- [x] Return 401 UNAUTHORIZED for invalid token

**Dependencies**: FR-AUTH-002, FR-AUTH-006
**Test Cases**: TC-AUTH-017, TC-AUTH-018

**API Endpoint**: `GET /api/auth/profile`

**Code Locations**:
- `/Users/dungngo97/Documents/bot-core/rust-core-engine/src/auth/handlers.rs:366-435` - Profile handler
- `/Users/dungngo97/Documents/bot-core/rust-core-engine/src/auth/handlers.rs:75-82` - Profile route definition
- `/Users/dungngo97/Documents/bot-core/rust-core-engine/src/auth/database.rs:72-81` - UserRepository::find_by_id()
- `/Users/dungngo97/Documents/bot-core/rust-core-engine/src/auth/models.rs:191-202` - User::to_profile()
- `/Users/dungngo97/Documents/bot-core/nextjs-ui-dashboard/src/services/api.ts:686-696` - Frontend getProfile()
- `/Users/dungngo97/Documents/bot-core/nextjs-ui-dashboard/src/contexts/AuthContext.tsx:36-55` - Profile loading on app init

**Response Format** (200 OK):
```json
{
  "success": true,
  "data": {
    "id": "507f1f77bcf86cd799439011",
    "email": "user@example.com",
    "full_name": "John Doe",
    "is_active": true,
    "is_admin": false,
    "created_at": "2025-10-10T12:00:00Z",
    "last_login": "2025-10-10T14:30:00Z",
    "settings": {
      "trading_enabled": false,
      "risk_level": "Medium",
      "max_positions": 3,
      "default_quantity": 0.01,
      "notifications": {
        "email_alerts": true,
        "trade_notifications": true,
        "system_alerts": true
      }
    }
  }
}
```

---

### FR-AUTH-008: Authorization Middleware (with_auth)

**Priority**: ☑ Critical
**Status**: ☑ Completed
**Code Tags**: `@spec:FR-AUTH-008`

**Description**:
Warp middleware filter that validates JWT tokens and extracts claims for protected endpoints. Returns Claims object on success, AuthError rejection on failure.

**Acceptance Criteria**:
- [x] Extract token from Authorization header
- [x] Verify token signature and expiration
- [x] Return Claims on successful verification
- [x] Reject with AuthError::InvalidHeader for missing/malformed header
- [x] Reject with AuthError::InvalidToken for invalid/expired token
- [x] Can be composed with other Warp filters
- [x] Provides Claims as extracted value to route handlers
- [x] Stateless verification (no database lookup)

**Dependencies**: FR-AUTH-002
**Test Cases**: TC-AUTH-019, TC-AUTH-020

**Code Locations**:
- `/Users/dungngo97/Documents/bot-core/rust-core-engine/src/auth/middleware.rs:7-13` - with_auth filter
- `/Users/dungngo97/Documents/bot-core/rust-core-engine/src/auth/middleware.rs:31-43` - authorize function
- `/Users/dungngo97/Documents/bot-core/rust-core-engine/src/auth/middleware.rs:78-86` - AuthError enum
- `/Users/dungngo97/Documents/bot-core/rust-core-engine/src/auth/middleware.rs:89-130` - Error handler

**Usage Example**:
```rust
let protected_route = warp::path("protected")
    .and(with_auth(jwt_service))
    .and_then(|claims: Claims| async move {
        // claims.sub = user_id
        // claims.email = user email
        // claims.is_admin = admin status
        Ok::<_, Rejection>(warp::reply::json(&json!({
            "user_id": claims.sub
        })))
    });
```

---

### FR-AUTH-009: Optional Authorization Middleware (with_optional_auth)

**Priority**: ☐ Medium
**Status**: ☑ Completed
**Code Tags**: `@spec:FR-AUTH-009`

**Description**:
Middleware for endpoints that support both authenticated and unauthenticated access. Returns Option<Claims> - Some(claims) for valid token, None for missing/invalid token.

**Acceptance Criteria**:
- [x] Accept missing Authorization header (returns None)
- [x] Accept invalid tokens (returns None, not error)
- [x] Return Some(Claims) for valid tokens
- [x] Never reject request (always Ok)
- [x] Route handler receives Option<Claims>
- [x] Can implement different logic for authenticated vs unauthenticated

**Dependencies**: FR-AUTH-002
**Test Cases**: TC-AUTH-021, TC-AUTH-022

**Code Locations**:
- `/Users/dungngo97/Documents/bot-core/rust-core-engine/src/auth/middleware.rs:15-21` - with_optional_auth filter
- `/Users/dungngo97/Documents/bot-core/rust-core-engine/src/auth/middleware.rs:45-63` - optional_authorize function

**Usage Example**:
```rust
let route = warp::path("data")
    .and(with_optional_auth(jwt_service))
    .and_then(|claims: Option<Claims>| async move {
        match claims {
            Some(c) => {
                // User authenticated - return personalized data
                Ok(warp::reply::json(&json!({ "user": c.sub })))
            },
            None => {
                // User not authenticated - return public data
                Ok(warp::reply::json(&json!({ "data": "public" })))
            }
        }
    });
```

---

### FR-AUTH-010: Admin Authorization Middleware (with_admin_auth)

**Priority**: ☐ High
**Status**: ☑ Completed
**Code Tags**: `@spec:FR-AUTH-010`

**Description**:
Middleware that requires valid JWT token AND admin privileges (is_admin: true). Used to protect administrative endpoints.

**Acceptance Criteria**:
- [x] Verify token validity (same as with_auth)
- [x] Check claims.is_admin == true
- [x] Return Claims on success
- [x] Reject with AuthError::InsufficientPermissions if not admin
- [x] Reject with AuthError::InvalidToken for invalid token
- [x] Return 403 FORBIDDEN for non-admin users
- [x] Return 401 UNAUTHORIZED for invalid tokens

**Dependencies**: FR-AUTH-008
**Test Cases**: TC-AUTH-023, TC-AUTH-024

**Code Locations**:
- `/Users/dungngo97/Documents/bot-core/rust-core-engine/src/auth/middleware.rs:23-29` - with_admin_auth filter
- `/Users/dungngo97/Documents/bot-core/rust-core-engine/src/auth/middleware.rs:65-76` - admin_authorize function
- `/Users/dungngo97/Documents/bot-core/rust-core-engine/src/auth/middleware.rs:100-103` - InsufficientPermissions error

**Usage Example**:
```rust
let admin_route = warp::path("admin")
    .and(warp::path("users"))
    .and(with_admin_auth(jwt_service))
    .and_then(|claims: Claims| async move {
        // Only admin users reach here
        // claims.is_admin is guaranteed to be true
        Ok::<_, Rejection>(warp::reply::json(&json!({
            "admin": claims.email
        })))
    });
```

---

### FR-AUTH-011: User Database Repository

**Priority**: ☑ Critical
**Status**: ☑ Completed
**Code Tags**: `@spec:FR-AUTH-011`

**Description**:
MongoDB repository for user CRUD operations with unique email index. Provides async methods for user creation, retrieval, updates, and queries.

**Acceptance Criteria**:
- [x] Create unique index on email field
- [x] Async create_user() returns ObjectId
- [x] Async find_by_email() returns Option<User>
- [x] Async find_by_id() returns Option<User>
- [x] Async email_exists() checks email uniqueness
- [x] Async update_last_login() updates timestamp
- [x] Async deactivate_user() sets is_active=false
- [x] Async count_users() returns total user count
- [x] Handle MongoDB errors gracefully
- [x] Support dummy repository for testing (no database)

**Dependencies**: MongoDB Database
**Test Cases**: TC-AUTH-025, TC-AUTH-026, TC-AUTH-027

**Code Locations**:
- `/Users/dungngo97/Documents/bot-core/rust-core-engine/src/auth/database.rs:10-159` - UserRepository implementation
- `/Users/dungngo97/Documents/bot-core/rust-core-engine/src/auth/database.rs:16-38` - Repository initialization with index
- `/Users/dungngo97/Documents/bot-core/rust-core-engine/src/auth/database.rs:40-44` - Dummy repository for testing

**Database Schema**:
```json
{
  "_id": ObjectId("507f1f77bcf86cd799439011"),
  "email": "user@example.com",  // UNIQUE INDEX
  "password_hash": "$2b$10$...",
  "full_name": "John Doe",
  "is_active": true,
  "is_admin": false,
  "created_at": ISODate("2025-10-10T12:00:00Z"),
  "updated_at": ISODate("2025-10-10T14:30:00Z"),
  "last_login": ISODate("2025-10-10T14:30:00Z"),
  "settings": {
    "trading_enabled": false,
    "risk_level": "Medium",
    "max_positions": 3,
    "default_quantity": 0.01,
    "notifications": {
      "email_alerts": true,
      "trade_notifications": true,
      "system_alerts": true
    }
  }
}
```

---

### FR-AUTH-012: User Data Models

**Priority**: ☑ Critical
**Status**: ☑ Completed
**Code Tags**: `@spec:FR-AUTH-012`

**Description**:
Comprehensive data models for user authentication and authorization with serde serialization, validation, and MongoDB compatibility.

**Acceptance Criteria**:
- [x] User model with MongoDB ObjectId
- [x] RegisterRequest with email/password validation
- [x] LoginRequest with field validation
- [x] LoginResponse with token and user profile
- [x] UserProfile without sensitive fields
- [x] UserSettings with trading preferences
- [x] NotificationSettings for alerts
- [x] RiskLevel enum (Low/Medium/High)
- [x] DateTime serialization for MongoDB
- [x] Validator traits for input validation
- [x] Default implementations for settings

**Dependencies**: None
**Test Cases**: TC-AUTH-028, TC-AUTH-029

**Code Locations**:
- `/Users/dungngo97/Documents/bot-core/rust-core-engine/src/auth/models.rs:74-209` - All model definitions
- `/Users/dungngo97/Documents/bot-core/rust-core-engine/src/auth/models.rs:7-72` - Custom DateTime serde modules
- `/Users/dungngo97/Documents/bot-core/rust-core-engine/src/auth/models.rs:115-122` - RegisterRequest with validation
- `/Users/dungngo97/Documents/bot-core/rust-core-engine/src/auth/models.rs:124-130` - LoginRequest with validation
- `/Users/dungngo97/Documents/bot-core/rust-core-engine/src/auth/models.rs:152-172` - Default settings

**Model Validation Rules**:
- **Email**: Must be valid RFC 5322 email format
- **Password (register)**: Minimum 6 characters
- **Password (login)**: Cannot be empty
- **Full name**: Optional, no validation

---

### FR-AUTH-013: Frontend Authentication Context

**Priority**: ☑ Critical
**Status**: ☑ Completed
**Code Tags**: `@spec:FR-AUTH-013`

**Description**:
React Context provider for global authentication state management. Handles login, logout, registration, token storage, and automatic authentication on app load.

**Acceptance Criteria**:
- [x] Global auth state (isAuthenticated, user, loading, error)
- [x] login() async function with email/password
- [x] register() async function with email/password/fullName
- [x] logout() function to clear authentication
- [x] Automatic token loading on app mount
- [x] Token expiry checking before API calls
- [x] Store token in localStorage
- [x] Fetch user profile after token load
- [x] Clear token on verification failure
- [x] useAuth() hook for consuming context

**Dependencies**: FR-AUTH-004, FR-AUTH-005, FR-AUTH-007
**Test Cases**: TC-AUTH-030, TC-AUTH-031

**Code Locations**:
- `/Users/dungngo97/Documents/bot-core/nextjs-ui-dashboard/src/contexts/AuthContext.tsx:24-142` - AuthProvider implementation
- `/Users/dungngo97/Documents/bot-core/nextjs-ui-dashboard/src/contexts/AuthContext.tsx:10-22` - AuthContextType interface
- `/Users/dungngo97/Documents/bot-core/nextjs-ui-dashboard/src/contexts/AuthContext.tsx:36-55` - Auto-authentication on mount
- `/Users/dungngo97/Documents/bot-core/nextjs-ui-dashboard/src/contexts/AuthContext.tsx:57-78` - Login function
- `/Users/dungngo97/Documents/bot-core/nextjs-ui-dashboard/src/contexts/AuthContext.tsx:80-109` - Register function
- `/Users/dungngo97/Documents/bot-core/nextjs-ui-dashboard/src/contexts/AuthContext.tsx:111-116` - Logout function
- `/Users/dungngo97/Documents/bot-core/nextjs-ui-dashboard/src/contexts/AuthContext.tsx:135-141` - useAuth hook

**Context State**:
```typescript
interface AuthContextType {
  isAuthenticated: boolean;
  user: UserProfile | null;
  login: (email: string, password: string) => Promise<boolean>;
  register: (email: string, password: string, fullName?: string) => Promise<boolean>;
  logout: () => void;
  loading: boolean;
  error: string | null;
}
```

---

### FR-AUTH-014: Frontend Login Page

**Priority**: ☑ Critical
**Status**: ☑ Completed
**Code Tags**: `@spec:FR-AUTH-014`

**Description**:
React login page with email/password form, validation, error handling, and redirect to dashboard on success. Includes demo credentials display and registration link.

**Acceptance Criteria**:
- [x] Email and password input fields
- [x] Form validation (required fields)
- [x] Submit button with loading state
- [x] Error toast notifications
- [x] Success toast on login
- [x] Redirect to /dashboard on successful login
- [x] Link to registration page
- [x] Demo credentials display
- [x] Automatic redirect if already authenticated
- [x] Responsive design (mobile/desktop)

**Dependencies**: FR-AUTH-005, FR-AUTH-013
**Test Cases**: TC-AUTH-032, TC-AUTH-033

**Code Locations**:
- `/Users/dungngo97/Documents/bot-core/nextjs-ui-dashboard/src/pages/Login.tsx:1-178` - Login page component
- `/Users/dungngo97/Documents/bot-core/nextjs-ui-dashboard/src/pages/Login.tsx:26-60` - handleLogin function
- `/Users/dungngo97/Documents/bot-core/nextjs-ui-dashboard/src/pages/Login.tsx:20-24` - Auto-redirect if authenticated

**Form Fields**:
- Email: type="email", required
- Password: type="password", required
- Submit button: Disabled during loading

**Demo Credentials**:
- Email: admin@tradingbot.com
- Password: demo123

---

### FR-AUTH-015: Frontend Registration Page

**Priority**: ☑ Critical
**Status**: ☑ Completed
**Code Tags**: `@spec:FR-AUTH-015`

**Description**:
React registration page with email/password/confirmPassword/fullName form. Client-side validation, password matching check, and automatic login after registration.

**Acceptance Criteria**:
- [x] Email, password, confirmPassword, fullName fields
- [x] Email format validation
- [x] Password minimum 6 characters validation
- [x] Password confirmation matching check
- [x] Full name is optional
- [x] Submit button with loading state
- [x] Error/success toast notifications
- [x] Automatic redirect to /dashboard on success
- [x] Link to login page
- [x] Auto-redirect if already authenticated

**Dependencies**: FR-AUTH-004, FR-AUTH-013
**Test Cases**: TC-AUTH-034, TC-AUTH-035

**Code Locations**:
- `/Users/dungngo97/Documents/bot-core/nextjs-ui-dashboard/src/pages/Register.tsx:1-206` - Register page component
- `/Users/dungngo97/Documents/bot-core/nextjs-ui-dashboard/src/pages/Register.tsx:28-76` - handleRegister function
- `/Users/dungngo97/Documents/bot-core/nextjs-ui-dashboard/src/pages/Register.tsx:31-50` - Client-side validation

**Form Validation**:
- Email: Required, valid email format
- Password: Required, minimum 6 characters
- Confirm Password: Required, must match password
- Full Name: Optional

---

### FR-AUTH-016: Frontend API Client Authentication

**Priority**: ☑ Critical
**Status**: ☑ Completed
**Code Tags**: `@spec:FR-AUTH-016`

**Description**:
TypeScript API client with authentication methods and automatic token injection in request headers. Handles token storage, expiry checking, and API error handling.

**Acceptance Criteria**:
- [x] AuthApiClient class with auth endpoints
- [x] login() returns LoginResponse with token and user
- [x] register() returns LoginResponse
- [x] verifyToken() checks token validity
- [x] getProfile() fetches user profile
- [x] setAuthToken() stores in localStorage
- [x] getAuthToken() retrieves from localStorage
- [x] removeAuthToken() clears from localStorage
- [x] isTokenExpired() decodes and checks expiry
- [x] Automatic Bearer token injection in requests
- [x] Request interceptor adds Authorization header
- [x] Response interceptor handles auth errors

**Dependencies**: FR-AUTH-001 through FR-AUTH-007
**Test Cases**: TC-AUTH-036, TC-AUTH-037

**Code Locations**:
- `/Users/dungngo97/Documents/bot-core/nextjs-ui-dashboard/src/services/api.ts:640-723` - AuthApiClient class
- `/Users/dungngo97/Documents/bot-core/nextjs-ui-dashboard/src/services/api.ts:645-655` - login()
- `/Users/dungngo97/Documents/bot-core/nextjs-ui-dashboard/src/services/api.ts:657-667` - register()
- `/Users/dungngo97/Documents/bot-core/nextjs-ui-dashboard/src/services/api.ts:669-684` - verifyToken()
- `/Users/dungngo97/Documents/bot-core/nextjs-ui-dashboard/src/services/api.ts:686-696` - getProfile()
- `/Users/dungngo97/Documents/bot-core/nextjs-ui-dashboard/src/services/api.ts:699-722` - Token management utilities
- `/Users/dungngo97/Documents/bot-core/nextjs-ui-dashboard/src/services/api.ts:324-331` - Request interceptor

**Token Storage**:
- Key: `authToken`
- Storage: localStorage
- Format: Raw JWT string (not JSON)

**Token Expiry Check**:
```typescript
isTokenExpired(token?: string): boolean {
    const authToken = token || this.getAuthToken();
    if (!authToken) return true;

    try {
        const payload = JSON.parse(atob(authToken.split(".")[1]));
        const exp = payload.exp * 1000; // Convert to milliseconds
        return Date.now() >= exp;
    } catch {
        return true;
    }
}
```

---

## Use Cases

### UC-AUTH-001: User Registration Flow

**Actor**: Anonymous User
**Preconditions**:
- User has valid email address
- User creates password meeting minimum requirements
- Email is not already registered

**Main Flow**:
1. User navigates to /register page
2. User enters email, password, confirm password, optional full name
3. System validates email format and password length (≥6)
4. System checks password and confirm password match
5. User clicks "Register" button
6. System sends POST /api/auth/register request
7. Backend validates input (email format, password length)
8. Backend checks email uniqueness in database
9. Backend hashes password with bcrypt
10. Backend creates user with default settings
11. Backend generates JWT token (7 days expiry)
12. Backend returns 201 CREATED with token and user profile
13. Frontend stores token in localStorage
14. Frontend updates AuthContext with authenticated state
15. Frontend redirects to /dashboard
16. User sees dashboard with welcome message

**Alternative Flows**:
- **Alt 1 - Email Already Exists**:
  1. Step 8 fails - email exists
  2. Backend returns 409 CONFLICT with "Email already registered"
  3. Frontend shows error toast
  4. User must use different email or try login

- **Alt 2 - Validation Failure**:
  1. Step 7 fails - invalid input
  2. Backend returns 400 BAD_REQUEST with validation details
  3. Frontend shows error toast with specific issue
  4. User corrects input and resubmits

- **Alt 3 - Password Mismatch**:
  1. Step 4 fails - passwords don't match
  2. Frontend shows error toast
  3. User re-enters confirm password
  4. No API request sent until validation passes

**Postconditions**:
- New user created in database
- User authenticated with JWT token
- User settings initialized to defaults
- User redirected to dashboard
- last_login remains null (not set on registration)

**Exception Handling**:
- Network Error: Show "Connection failed, please try again"
- Database Error: Show "Server error, please try again later"
- Bcrypt Error: Show "Internal error, please contact support"

---

### UC-AUTH-002: User Login Flow

**Actor**: Registered User
**Preconditions**:
- User has registered account
- User knows email and password
- Account is active (is_active: true)

**Main Flow**:
1. User navigates to /login page
2. User enters email and password
3. System validates email format and password not empty
4. User clicks "Login" button
5. System sends POST /api/auth/login request
6. Backend validates input format
7. Backend finds user by email in database
8. Backend verifies user.is_active == true
9. Backend verifies password against stored hash
10. Backend updates user.last_login timestamp
11. Backend generates JWT token (7 days expiry)
12. Backend returns 200 OK with token and user profile
13. Frontend stores token in localStorage
14. Frontend updates AuthContext with authenticated state
15. Frontend redirects to /dashboard
16. User sees dashboard with trading data

**Alternative Flows**:
- **Alt 1 - Invalid Credentials**:
  1. Step 7 or 9 fails - user not found or wrong password
  2. Backend returns 401 UNAUTHORIZED with "Invalid email or password"
  3. Frontend shows error toast
  4. User retries with correct credentials

- **Alt 2 - Account Deactivated**:
  1. Step 8 fails - user.is_active == false
  2. Backend returns 403 FORBIDDEN with "Account is deactivated"
  3. Frontend shows error toast
  4. User must contact support to reactivate

- **Alt 3 - Validation Error**:
  1. Step 6 fails - invalid email format or empty password
  2. Backend returns 400 BAD_REQUEST
  3. Frontend shows validation error
  4. User corrects input and resubmits

**Postconditions**:
- User authenticated with JWT token
- last_login timestamp updated
- User redirected to dashboard
- Trading features available based on settings

**Exception Handling**:
- Network Error: Show "Connection failed, please try again"
- Database Error: Show "Server error, please try again later"
- Bcrypt Verification Error: Show "Authentication failed"

---

### UC-AUTH-003: Automatic Authentication on App Load

**Actor**: System
**Preconditions**:
- User previously logged in
- Token stored in localStorage
- User refreshes page or reopens app

**Main Flow**:
1. App component mounts (React useEffect)
2. AuthProvider reads token from localStorage
3. System checks token expiry by decoding JWT
4. Token is not expired (exp > current_time)
5. System sends GET /api/auth/profile with token
6. Backend verifies token signature
7. Backend extracts user_id from token claims
8. Backend fetches user from database
9. Backend returns 200 OK with user profile
10. Frontend updates AuthContext (isAuthenticated: true)
11. Frontend sets user state with profile data
12. User sees authenticated app state
13. Protected routes become accessible

**Alternative Flows**:
- **Alt 1 - Token Expired**:
  1. Step 4 fails - token expired
  2. System removes token from localStorage
  3. System sets isAuthenticated: false
  4. User remains on current page or redirected to /login

- **Alt 2 - Invalid Token**:
  1. Step 6 fails - invalid signature or format
  2. Backend returns 401 UNAUTHORIZED
  3. Frontend removes token from localStorage
  4. Frontend sets isAuthenticated: false
  5. User redirected to /login

- **Alt 3 - User Not Found**:
  1. Step 8 fails - user deleted or ID invalid
  2. Backend returns 404 NOT_FOUND
  3. Frontend removes token from localStorage
  4. Frontend sets isAuthenticated: false
  5. User redirected to /login

- **Alt 4 - No Token**:
  1. Step 2 finds no token
  2. System sets isAuthenticated: false
  3. System sets loading: false
  4. User sees unauthenticated state

**Postconditions**:
- User authentication state determined
- App ready for user interaction
- Protected routes accessible if authenticated
- Loading state cleared

**Exception Handling**:
- Network Error: Keep user unauthenticated, allow retry
- Malformed Token: Remove token, force re-login

---

### UC-AUTH-004: Accessing Protected API Endpoint

**Actor**: Authenticated User
**Preconditions**:
- User logged in with valid token
- Token not expired
- User attempts to access protected endpoint

**Main Flow**:
1. Frontend makes API request to protected endpoint
2. Request interceptor adds Authorization header
3. Header format: "Bearer {token}"
4. Backend receives request with Authorization header
5. Middleware extracts token from header
6. Middleware verifies token signature with secret key
7. Middleware checks token expiration
8. Middleware extracts Claims (user_id, email, is_admin)
9. Middleware passes Claims to route handler
10. Route handler processes request with user context
11. Backend returns response with requested data
12. Frontend receives and processes response

**Alternative Flows**:
- **Alt 1 - Missing Token**:
  1. Step 2 skipped - no token in localStorage
  2. Request sent without Authorization header
  3. Backend middleware rejects request
  4. Backend returns 401 UNAUTHORIZED
  5. Frontend redirects to /login

- **Alt 2 - Invalid Token**:
  1. Step 6 fails - signature verification fails
  2. Middleware rejects with AuthError::InvalidToken
  3. Backend returns 401 UNAUTHORIZED with error
  4. Frontend removes invalid token
  5. Frontend redirects to /login

- **Alt 3 - Expired Token**:
  1. Step 7 fails - exp < current_time
  2. Middleware rejects with AuthError::InvalidToken
  3. Backend returns 401 UNAUTHORIZED
  4. Frontend removes expired token
  5. Frontend redirects to /login

**Postconditions**:
- API request processed with user context
- User identity verified
- Response data returned to frontend

**Exception Handling**:
- Malformed Header: Return 401 with "Invalid authorization header"
- Corrupted Token: Return 401 with "Invalid or expired token"

---

### UC-AUTH-005: Accessing Admin-Only Endpoint

**Actor**: Admin User
**Preconditions**:
- User logged in with valid token
- User has is_admin: true
- Token not expired

**Main Flow**:
1. Admin user attempts to access admin endpoint
2. Frontend makes API request with Authorization header
3. Backend receives request
4. Admin middleware extracts and verifies token
5. Middleware checks claims.is_admin == true
6. Admin verification passes
7. Middleware passes Claims to route handler
8. Route handler processes admin-level request
9. Backend returns admin data
10. Frontend displays admin interface

**Alternative Flows**:
- **Alt 1 - Non-Admin User**:
  1. Step 5 fails - claims.is_admin == false
  2. Middleware rejects with AuthError::InsufficientPermissions
  3. Backend returns 403 FORBIDDEN
  4. Frontend shows "Access denied" message
  5. User redirected to regular dashboard

- **Alt 2 - Invalid Token**:
  1. Step 4 fails - token invalid or expired
  2. Middleware rejects with AuthError::InvalidToken
  3. Backend returns 401 UNAUTHORIZED
  4. Frontend redirects to /login

**Postconditions**:
- Admin operation completed (if authorized)
- Admin interface displayed (if authorized)
- Regular user denied access (if not admin)

**Exception Handling**:
- Missing Token: Return 401 UNAUTHORIZED
- Regular User Attempt: Return 403 FORBIDDEN with clear message

---

### UC-AUTH-006: User Logout

**Actor**: Authenticated User
**Preconditions**:
- User logged in with valid token
- User in authenticated app state

**Main Flow**:
1. User clicks logout button
2. Frontend calls logout() from AuthContext
3. System removes token from localStorage
4. System clears user state (user: null)
5. System sets isAuthenticated: false
6. System clears any error state
7. Frontend redirects to /login page
8. User sees login form

**Alternative Flows**:
- **Alt 1 - Already Logged Out**:
  1. Step 2 called with no token
  2. System clears state anyway
  3. User remains on current page or redirected to /login

**Postconditions**:
- Token removed from storage
- User state cleared
- App in unauthenticated state
- User on login page

**Exception Handling**:
- None - logout is always successful (client-side only)

**Notes**:
- No backend API call required (stateless tokens)
- Token remains valid until expiration (backend has no revocation)
- Future enhancement: Add token revocation list

---

## Data Requirements

### Input Data

**RegisterRequest**:
- `email`: string, Required, email format validation
- `password`: string, Required, minimum 6 characters
- `full_name`: string, Optional, no validation

**LoginRequest**:
- `email`: string, Required, email format validation
- `password`: string, Required, non-empty validation

**Authorization Header**:
- Format: "Bearer {token}"
- Token: string, JWT format with 3 parts (header.payload.signature)
- Required for protected endpoints

---

### Output Data

**JWT Token Claims**:
- `sub`: string, User ObjectId as hex string
- `email`: string, User email address
- `is_admin`: boolean, Admin privilege flag
- `exp`: i64, Expiration timestamp (Unix seconds)
- `iat`: i64, Issued at timestamp (Unix seconds)

**UserProfile**:
- `id`: string, User ObjectId as hex
- `email`: string
- `full_name`: string | null
- `is_active`: boolean
- `is_admin`: boolean
- `created_at`: DateTime<Utc>
- `last_login`: DateTime<Utc> | null
- `settings`: UserSettings

**UserSettings**:
- `trading_enabled`: boolean
- `risk_level`: "Low" | "Medium" | "High"
- `max_positions`: u32
- `default_quantity`: f64
- `notifications`: NotificationSettings

**NotificationSettings**:
- `email_alerts`: boolean
- `trade_notifications`: boolean
- `system_alerts`: boolean

---

### Data Validation

**Email Validation**:
- Must match RFC 5322 email format
- Uses validator crate email validation
- Case-sensitive (stored as provided)
- No automatic lowercase conversion

**Password Validation**:
- **Registration**: Minimum 6 characters
- **Login**: Cannot be empty (minimum 1 character)
- No maximum length
- No complexity requirements (can be enhanced)
- Accepts any UTF-8 characters

**Token Validation**:
- Must have 3 parts separated by dots
- Header must specify HS256 algorithm
- Signature must match secret key
- Expiration must be in future
- Claims must deserialize correctly

**ObjectId Validation**:
- Must be 24 character hex string
- Valid MongoDB ObjectId format
- Used for user_id in tokens and database queries

---

## Interface Requirements

### API Endpoints

All authentication endpoints follow REST conventions with JSON payloads and standard HTTP status codes.

**Base URL**: `/api/auth`

#### POST /api/auth/register
- **Purpose**: Create new user account
- **Request Body**: RegisterRequest JSON
- **Success Response**: 201 CREATED with LoginResponse
- **Error Responses**:
  - 400 BAD_REQUEST - Validation failed
  - 409 CONFLICT - Email already exists
  - 500 INTERNAL_SERVER_ERROR - Server error
- **Reference**: FR-AUTH-004

#### POST /api/auth/login
- **Purpose**: Authenticate existing user
- **Request Body**: LoginRequest JSON
- **Success Response**: 200 OK with LoginResponse
- **Error Responses**:
  - 400 BAD_REQUEST - Validation failed
  - 401 UNAUTHORIZED - Invalid credentials
  - 403 FORBIDDEN - Account deactivated
  - 500 INTERNAL_SERVER_ERROR - Server error
- **Reference**: FR-AUTH-005

#### GET /api/auth/verify
- **Purpose**: Verify JWT token validity
- **Headers**: Authorization: Bearer {token}
- **Success Response**: 200 OK with token claims
- **Error Responses**:
  - 401 UNAUTHORIZED - Invalid/expired token or missing header
- **Reference**: FR-AUTH-006

#### GET /api/auth/profile
- **Purpose**: Retrieve user profile
- **Headers**: Authorization: Bearer {token}
- **Success Response**: 200 OK with UserProfile
- **Error Responses**:
  - 400 BAD_REQUEST - Invalid user ID format
  - 401 UNAUTHORIZED - Invalid/expired token
  - 404 NOT_FOUND - User not found
  - 500 INTERNAL_SERVER_ERROR - Database error
- **Reference**: FR-AUTH-007

---

### UI Screens

**Login Page** (`/login`):
- Email input field
- Password input field (hidden)
- Submit button with loading state
- Link to registration page
- Demo credentials display
- Reference: FR-AUTH-014

**Registration Page** (`/register`):
- Email input field
- Full name input field (optional)
- Password input field (hidden)
- Confirm password input field (hidden)
- Submit button with loading state
- Link to login page
- Reference: FR-AUTH-015

**Protected Routes**:
- Automatic redirect to /login if not authenticated
- Check isAuthenticated from AuthContext
- Show loading spinner during authentication check

---

### External Systems

**MongoDB**:
- Collection: `users`
- Index: `email` (unique)
- Operations: Create, Read, Update
- Reference: FR-AUTH-011

**localStorage (Browser)**:
- Key: `authToken`
- Stores JWT token string
- Cleared on logout
- Read on app initialization

---

## Non-Functional Requirements

### Performance
- **JWT Generation**: < 50ms per token
- **JWT Verification**: < 10ms per token
- **Password Hashing**: < 500ms (bcrypt cost 10)
- **Password Verification**: < 100ms
- **Login Endpoint**: < 1000ms total response time
- **Register Endpoint**: < 1500ms total response time
- **Token Verification Endpoint**: < 100ms (no database)
- **Profile Endpoint**: < 200ms (with database query)
- **Concurrent Logins**: Support 100 simultaneous login requests

**Rationale**: Authentication must be fast to provide good user experience. Bcrypt intentionally slow to prevent brute force attacks.

---

### Security

**Token Security**:
- HS256 algorithm (HMAC with SHA-256)
- Secret key minimum 32 characters (256 bits recommended)
- Secret stored in environment variables, never in code
- Tokens transmitted over HTTPS only in production
- 7-day expiration to limit exposure window

**Password Security**:
- Bcrypt hashing with cost factor 10
- Random salt per password (built into bcrypt)
- No plain-text password storage
- No password in logs or error messages
- Generic error messages to prevent user enumeration

**API Security**:
- All protected endpoints require valid JWT
- Admin endpoints require is_admin flag
- CORS configured for known origins
- Rate limiting recommended (not implemented)
- No SQL injection (using MongoDB query builders)

**Session Security**:
- Stateless tokens (no server-side session store)
- Logout is client-side only (clear token)
- No session fixation vulnerability
- Token revocation not implemented (future enhancement)

---

### Reliability

**Error Handling**:
- All async operations wrapped in Result<T, Error>
- Database errors return 500 INTERNAL_SERVER_ERROR
- Network errors handled gracefully with retry
- Token errors return appropriate 401/403 status codes
- Frontend shows user-friendly error messages

**Data Integrity**:
- Email unique index prevents duplicates
- MongoDB transactions for atomic operations
- Password hashes validated before verification
- User IDs validated as ObjectId format

**Availability**:
- No single point of failure (stateless tokens)
- Database connection pooling for scalability
- Graceful degradation if database unavailable
- Health check endpoints for monitoring

---

### Maintainability

**Code Organization**:
- Modular design: jwt.rs, handlers.rs, models.rs, database.rs, middleware.rs
- Clear separation of concerns
- Type safety with Rust and TypeScript
- Comprehensive unit tests (100+ test cases)

**Documentation**:
- Inline code comments for complex logic
- This specification document
- API documentation (OpenAPI/Swagger recommended)
- README with setup instructions

**Testing**:
- Unit tests for all auth functions
- Integration tests for API endpoints
- Frontend component tests for auth forms
- E2E tests for complete auth flows

---

## Implementation Notes

### Backend Architecture (Rust)

**Technology Stack**:
- **Warp**: Web framework for routing and middleware
- **jsonwebtoken**: JWT generation and verification
- **bcrypt**: Password hashing
- **MongoDB**: User persistence
- **serde**: JSON serialization/deserialization
- **validator**: Input validation
- **chrono**: DateTime handling
- **tracing**: Logging

**Code Structure**:
```
rust-core-engine/src/auth/
├── mod.rs              # Module exports
├── jwt.rs              # JWT service & password hashing
├── models.rs           # Data models with validation
├── database.rs         # MongoDB user repository
├── handlers.rs         # API route handlers
└── middleware.rs       # Auth middleware filters
```

**Design Patterns**:
- **Service Pattern**: JwtService, PasswordService, AuthService
- **Repository Pattern**: UserRepository for data access
- **Middleware Pattern**: with_auth, with_admin_auth filters
- **Result Pattern**: Error handling with anyhow::Result

**Configuration**:
- JWT_SECRET: Environment variable for token signing
- JWT_EXPIRATION_HOURS: Token expiry (default 168 = 7 days)
- MONGODB_URI: Database connection string
- MONGODB_DATABASE: Database name (default: "trading_bot")

---

### Frontend Architecture (TypeScript/React)

**Technology Stack**:
- **React**: UI framework
- **TypeScript**: Type safety
- **Axios**: HTTP client
- **React Router**: Navigation
- **Context API**: Global state
- **localStorage**: Token persistence
- **Sonner**: Toast notifications

**Code Structure**:
```
nextjs-ui-dashboard/src/
├── contexts/
│   └── AuthContext.tsx     # Auth state management
├── pages/
│   ├── Login.tsx           # Login page
│   └── Register.tsx        # Registration page
└── services/
    └── api.ts              # API client with auth methods
```

**Design Patterns**:
- **Context Pattern**: Global auth state with AuthProvider
- **Custom Hook**: useAuth() for consuming context
- **API Client Pattern**: Centralized API methods
- **Interceptor Pattern**: Automatic token injection

**Token Management**:
- Store: localStorage.setItem('authToken', token)
- Retrieve: localStorage.getItem('authToken')
- Remove: localStorage.removeItem('authToken')
- Expiry Check: Decode JWT payload and check exp claim

---

### Database Schema (MongoDB)

**Collection**: `users`

**Indexes**:
- `_id`: Primary key (ObjectId, automatic)
- `email`: Unique index for fast lookup and uniqueness

**Documents**:
```javascript
{
  _id: ObjectId("507f1f77bcf86cd799439011"),
  email: "user@example.com",
  password_hash: "$2b$10$N9qo8uLOickgx2ZMRZoMyeIjZAgcfl7p92ldGxad68LJZdL17lhWy",
  full_name: "John Doe",
  is_active: true,
  is_admin: false,
  created_at: ISODate("2025-10-10T12:00:00Z"),
  updated_at: ISODate("2025-10-10T14:30:00Z"),
  last_login: ISODate("2025-10-10T14:30:00Z"),
  settings: {
    trading_enabled: false,
    risk_level: "Medium",
    max_positions: 3,
    default_quantity: 0.01,
    notifications: {
      email_alerts: true,
      trade_notifications: true,
      system_alerts: true
    }
  }
}
```

**Field Constraints**:
- `email`: Required, unique, string
- `password_hash`: Required, string (bcrypt hash)
- `full_name`: Optional, string
- `is_active`: Required, boolean, default true
- `is_admin`: Required, boolean, default false
- `created_at`: Required, DateTime
- `updated_at`: Required, DateTime
- `last_login`: Optional, DateTime

---

### Security Best Practices

**Implemented**:
- JWT tokens for stateless auth
- Bcrypt for password hashing (cost 10)
- HTTPS in production (infrastructure)
- Authorization middleware on protected routes
- Input validation with validator crate
- Generic error messages (prevent user enumeration)
- Secure secret key storage (environment variables)
- Token expiration (7 days)

**Recommended Enhancements**:
- Rate limiting on auth endpoints (prevent brute force)
- Account lockout after N failed login attempts
- Password complexity requirements
- Email verification after registration
- Two-factor authentication (2FA)
- Refresh tokens (shorter access token expiry)
- Token revocation list (blacklist)
- Security headers (CSP, HSTS, X-Frame-Options)
- Audit logging of auth events
- IP-based access control

---

## Testing Strategy

### Unit Tests

**Rust Tests** (All implemented, 100+ test cases):
- `jwt.rs` tests: Token generation, verification, expiry, password hashing
- `models.rs` tests: Validation, serialization, model methods
- `handlers.rs` tests: Registration, login, verify, profile handlers
- `database.rs` tests: Repository methods, dummy repository
- `middleware.rs` tests: Auth filters, admin checks, error handling

**Frontend Tests** (Recommended):
- AuthContext tests: Login, logout, register, auto-auth
- Login page tests: Form validation, submission
- Register page tests: Form validation, password matching
- API client tests: Token storage, injection, expiry check

**Coverage Target**: 90%+ for all auth modules

---

### Integration Tests

**API Endpoint Tests**:
- Full registration flow with database
- Full login flow with token generation
- Token verification with valid/invalid tokens
- Profile retrieval with authentication
- Admin authorization checks
- Error scenarios (network, database, validation)

**Test Scenarios**:
1. Register new user → verify in database → login → access protected route
2. Login with wrong password → verify 401 error
3. Access protected route without token → verify 401 error
4. Access admin route as regular user → verify 403 error
5. Token expires → verify 401 error → re-login successful

---

### E2E Tests

**User Flows** (Playwright recommended):
1. **Registration Flow**:
   - Navigate to /register
   - Fill form with valid data
   - Submit and verify redirect to /dashboard
   - Verify token in localStorage
   - Verify user appears in database

2. **Login Flow**:
   - Navigate to /login
   - Enter credentials
   - Submit and verify redirect to /dashboard
   - Verify protected routes accessible
   - Logout and verify redirect to /login

3. **Auto-Authentication Flow**:
   - Login successfully
   - Refresh page
   - Verify user remains authenticated
   - Verify no login page shown

4. **Session Expiry Flow**:
   - Login successfully
   - Wait for token to expire (or mock expiry)
   - Make API request
   - Verify redirect to /login
   - Verify error message shown

---

### Security Tests

**Vulnerability Scanning**:
- SQL/NoSQL injection attempts
- XSS in form inputs
- CSRF token validation
- Rate limiting effectiveness
- Password brute force resistance

**Penetration Testing**:
- Token tampering attempts
- Unauthorized access attempts
- Privilege escalation attempts
- Session fixation tests

**Authentication Testing**:
- Invalid token formats
- Expired tokens
- Tokens from different secret keys
- Replay attack resistance

---

## Deployment

### Environment Configuration

**Required Environment Variables**:
```bash
# Rust Backend
JWT_SECRET=your-super-secret-key-min-32-chars
JWT_EXPIRATION_HOURS=168  # 7 days
MONGODB_URI=mongodb://localhost:27017
MONGODB_DATABASE=trading_bot

# Frontend (Vite)
VITE_RUST_API_URL=http://localhost:8080
```

**Secret Key Generation**:
```bash
# Generate secure random key
openssl rand -base64 32
```

---

### Database Setup

**MongoDB Initialization**:
```javascript
// Connect to database
use trading_bot;

// Create unique index on email
db.users.createIndex({ email: 1 }, { unique: true });

// Verify index
db.users.getIndexes();
```

**Database Migrations**:
- No migrations required (schema-less MongoDB)
- Index creation handled by UserRepository::new()
- Backward compatible schema changes

---

### Deployment Checklist

**Pre-Deployment**:
- [ ] Generate production JWT secret key
- [ ] Configure HTTPS/TLS certificates
- [ ] Set up MongoDB replica set (production)
- [ ] Configure CORS for production domain
- [ ] Enable rate limiting
- [ ] Set up monitoring and alerts
- [ ] Run security audit
- [ ] Test all auth flows in staging

**Deployment Steps**:
1. Deploy MongoDB with replica set
2. Apply database indexes
3. Deploy Rust backend with environment variables
4. Deploy frontend with API URL
5. Verify health checks pass
6. Test registration and login
7. Monitor logs for errors

**Rollback Plan**:
- Keep previous backend version running
- Switch traffic back to old version
- Investigate issues in staging
- No database rollback needed (additive changes only)

---

## Monitoring & Observability

### Metrics to Track

**Authentication Metrics**:
- Login success rate (target > 99%)
- Registration success rate (target > 95%)
- Token verification latency (target < 10ms)
- Password hashing time (target < 500ms)
- Failed login attempts per user (alert > 5)
- Active user sessions count

**Error Metrics**:
- 401 UNAUTHORIZED responses per minute
- 403 FORBIDDEN responses per minute
- 500 INTERNAL_SERVER_ERROR responses per minute
- Token expiration rate
- Database connection errors

**Performance Metrics**:
- API endpoint response times (p50, p95, p99)
- Database query latency
- Bcrypt hashing time
- Token generation time

---

### Logging

**Log Levels**:
- **INFO**: Successful login, registration, logout
- **WARN**: Failed login attempts, validation errors
- **ERROR**: Database errors, JWT errors, server errors

**Key Log Events**:
```rust
// Registration
info!("Register attempt for email: {}", request.email);
info!("User created successfully: {} (ID: {})", email, user_id);
warn!("Registration failed: email already exists: {}", email);

// Login
info!("Login attempt for email: {}", request.email);
info!("Login successful for user: {}", email);
warn!("Login failed: user not found: {}", email);
warn!("Login failed: invalid password for user: {}", email);
warn!("Login failed: user account deactivated: {}", email);

// Errors
error!("Database error checking email: {}", e);
error!("Password hashing failed: {}", e);
error!("Token generation failed: {}", e);
```

**Sensitive Data Handling**:
- Never log passwords (plain or hashed)
- Never log full JWT tokens
- Never log user_id in public logs (GDPR)
- Sanitize email addresses in production logs

---

### Alerts

**Critical Alerts** (Immediate Response):
- Auth service down (health check fails)
- Database connection lost
- JWT secret compromised (detected)
- Failed login rate > 1000/min (DDoS)

**Warning Alerts** (24h Response):
- Login success rate < 95%
- Token verification latency > 50ms
- Failed login attempts > 5 per user
- Database query slow (> 500ms)

**Info Alerts** (7d Review):
- New user registration spike
- Unusual login patterns
- Token expiration rate increase

---

### Dashboards

**Authentication Dashboard**:
- Login/Registration rate (requests per minute)
- Success/Failure ratio
- Active user count
- Token expiration timeline
- Failed login attempts by user

**Performance Dashboard**:
- API endpoint latency (all auth endpoints)
- Database query performance
- Bcrypt hashing time distribution
- P50/P95/P99 response times

---

## Traceability

### Requirements Traceability

**Business Requirements → Functional Requirements**:
- BR-001 (Secure User Access) → FR-AUTH-001 through FR-AUTH-016
- BR-002 (Role-Based Access) → FR-AUTH-010 (Admin middleware)
- BR-003 (Session Management) → FR-AUTH-001, FR-AUTH-002, FR-AUTH-013

### Design Traceability

**Functional Requirements → Code**:
- FR-AUTH-001 → `/rust-core-engine/src/auth/jwt.rs:31-51`
- FR-AUTH-002 → `/rust-core-engine/src/auth/jwt.rs:53-62`
- FR-AUTH-003 → `/rust-core-engine/src/auth/jwt.rs:69-82`
- FR-AUTH-004 → `/rust-core-engine/src/auth/handlers.rs:85-200`
- FR-AUTH-005 → `/rust-core-engine/src/auth/handlers.rs:202-324`
- FR-AUTH-006 → `/rust-core-engine/src/auth/handlers.rs:326-364`
- FR-AUTH-007 → `/rust-core-engine/src/auth/handlers.rs:366-435`
- FR-AUTH-008 → `/rust-core-engine/src/auth/middleware.rs:7-13`
- FR-AUTH-009 → `/rust-core-engine/src/auth/middleware.rs:15-21`
- FR-AUTH-010 → `/rust-core-engine/src/auth/middleware.rs:23-29`
- FR-AUTH-011 → `/rust-core-engine/src/auth/database.rs:10-159`
- FR-AUTH-012 → `/rust-core-engine/src/auth/models.rs:74-209`
- FR-AUTH-013 → `/nextjs-ui-dashboard/src/contexts/AuthContext.tsx:24-142`
- FR-AUTH-014 → `/nextjs-ui-dashboard/src/pages/Login.tsx:1-178`
- FR-AUTH-015 → `/nextjs-ui-dashboard/src/pages/Register.tsx:1-206`
- FR-AUTH-016 → `/nextjs-ui-dashboard/src/services/api.ts:640-723`

### Test Traceability

**Functional Requirements → Test Cases**:
- FR-AUTH-001 → TC-AUTH-001, TC-AUTH-002, TC-AUTH-003
- FR-AUTH-002 → TC-AUTH-004, TC-AUTH-005, TC-AUTH-006
- FR-AUTH-003 → TC-AUTH-007, TC-AUTH-008
- FR-AUTH-004 → TC-AUTH-009, TC-AUTH-010, TC-AUTH-011
- FR-AUTH-005 → TC-AUTH-012, TC-AUTH-013, TC-AUTH-014
- FR-AUTH-006 → TC-AUTH-015, TC-AUTH-016
- FR-AUTH-007 → TC-AUTH-017, TC-AUTH-018
- FR-AUTH-008 → TC-AUTH-019, TC-AUTH-020
- FR-AUTH-009 → TC-AUTH-021, TC-AUTH-022
- FR-AUTH-010 → TC-AUTH-023, TC-AUTH-024

**Test Files**:
- Rust: `rust-core-engine/src/auth/jwt.rs:84-268` (tests module)
- Rust: `rust-core-engine/src/auth/models.rs:210-851` (tests module)
- Rust: `rust-core-engine/src/auth/handlers.rs:437-1137` (tests module)
- Rust: `rust-core-engine/src/auth/database.rs:161-298` (tests module)
- Rust: `rust-core-engine/src/auth/middleware.rs:132-643` (tests module)

---

## Risks & Mitigations

| Risk | Impact | Probability | Mitigation |
|------|--------|-------------|------------|
| JWT secret key exposure | Critical | Low | Store in environment variables only, rotate periodically, use secrets manager in production |
| Token hijacking (XSS) | High | Medium | Use HttpOnly cookies (future), implement CSP headers, sanitize all inputs |
| Brute force password attacks | High | High | Implement rate limiting, account lockout after N failures, CAPTCHA on login |
| Token replay attacks | Medium | Low | Use short expiration times, implement refresh tokens, log token usage |
| Database connection failure | High | Low | Connection pooling, auto-reconnect, graceful degradation, health monitoring |
| Password hash collision (bcrypt) | Low | Very Low | Bcrypt with cost 10 is industry standard, probability negligible |
| User enumeration via error messages | Medium | Medium | Use generic error messages, same response time for valid/invalid users |
| Token secret in version control | Critical | Low | Use .gitignore for .env files, scan commits for secrets, use pre-commit hooks |
| MongoDB injection | Medium | Low | Use query builders (not string concatenation), validate all inputs |
| Session fixation | Low | Very Low | Stateless tokens prevent fixation, generate new token on privilege change |

---

## Open Questions

- [x] **Q1**: Should we implement refresh tokens with shorter access token expiry?
  - **Resolution**: Current 7-day expiry acceptable for MVP. Implement refresh tokens in v2.0

- [ ] **Q2**: Should we add email verification after registration?
  - **Status**: Not implemented yet. Required for production?
  - **Decision needed by**: Sprint planning

- [ ] **Q3**: What rate limiting strategy should we use for auth endpoints?
  - **Status**: Not implemented. Redis-based rate limiting recommended
  - **Decision needed by**: Before production deployment

- [ ] **Q4**: Should we implement account lockout after failed login attempts?
  - **Status**: Not implemented. Security requirement for production
  - **Decision needed by**: Security review

- [ ] **Q5**: Do we need password reset functionality for MVP?
  - **Status**: Not implemented. User requested feature
  - **Decision needed by**: Product roadmap review

---

## Changelog

| Version | Date | Author | Changes |
|---------|------|--------|---------|
| 1.0 | 2025-10-10 | Backend Team | Initial comprehensive specification covering all implemented authentication and authorization features |

---

## Appendix

### References

**External Documentation**:
- [JWT RFC 7519](https://tools.ietf.org/html/rfc7519) - JSON Web Token standard
- [bcrypt Algorithm](https://en.wikipedia.org/wiki/Bcrypt) - Password hashing
- [OWASP Authentication Cheat Sheet](https://cheatsheetseries.owasp.org/cheatsheets/Authentication_Cheat_Sheet.html)
- [MongoDB User Model Best Practices](https://docs.mongodb.com/manual/core/security-users/)
- [Warp Web Framework](https://docs.rs/warp/) - Rust async web framework

**Internal Documentation**:
- [API_SPEC.md](../../02-technical-specs/2.1-api-specifications/API_SPEC.md) - Complete API specification
- [DATA_MODELS.md](../../02-technical-specs/2.2-data-models/DATA_MODELS.md) - Data structure definitions
- [BUSINESS_RULES.md](../../specs/BUSINESS_RULES.md) - Business logic and rules

---

### Glossary

- **JWT (JSON Web Token)**: Compact, URL-safe token format for securely transmitting claims between parties
- **Claims**: Statements about an entity (user) and additional metadata in JWT
- **Bearer Token**: Authentication scheme where the token itself grants access (transmitted in Authorization header)
- **Bcrypt**: Password hashing function based on Blowfish cipher, designed to be slow
- **Salt**: Random data added to password before hashing to prevent rainbow table attacks
- **HMAC**: Hash-based Message Authentication Code, used for token signatures
- **HS256**: HMAC with SHA-256, the signing algorithm used for JWTs
- **ObjectId**: MongoDB's 12-byte unique identifier format
- **RBAC (Role-Based Access Control)**: Access control paradigm where permissions are assigned to roles
- **Middleware**: Software component that processes requests before they reach route handlers
- **Stateless Authentication**: Authentication where server doesn't store session state, all info in token
- **Token Expiry (exp)**: JWT claim indicating when token becomes invalid
- **Issued At (iat)**: JWT claim indicating when token was created
- **Subject (sub)**: JWT claim identifying the user (user_id)

---

### Code Examples

**Generate and Verify Token (Rust)**:
```rust
use crate::auth::jwt::JwtService;

// Initialize service
let jwt_service = JwtService::new(
    "my-secret-key".to_string(),
    Some(24 * 7) // 7 days
);

// Generate token
let token = jwt_service
    .generate_token("user_id_123", "user@example.com", false)
    .unwrap();

// Verify token
match jwt_service.verify_token(&token) {
    Ok(claims) => {
        println!("User ID: {}", claims.sub);
        println!("Email: {}", claims.email);
        println!("Admin: {}", claims.is_admin);
    },
    Err(e) => eprintln!("Invalid token: {}", e),
}
```

**Hash and Verify Password (Rust)**:
```rust
use crate::auth::jwt::PasswordService;

// Hash password
let password = "user_password_123";
let hash = PasswordService::hash_password(password).unwrap();

// Verify password
let is_valid = PasswordService::verify_password(password, &hash).unwrap();
assert!(is_valid);
```

**Use Auth Middleware (Rust)**:
```rust
use crate::auth::middleware::with_auth;
use warp::Filter;

let jwt_service = JwtService::new("secret".to_string(), Some(24));

let protected_route = warp::path("api")
    .and(warp::path("protected"))
    .and(with_auth(jwt_service))
    .and_then(|claims: Claims| async move {
        Ok::<_, Rejection>(warp::reply::json(&json!({
            "user_id": claims.sub,
            "email": claims.email
        })))
    });
```

**Frontend Auth Hook (TypeScript)**:
```typescript
import { useAuth } from '@/contexts/AuthContext';

function MyComponent() {
  const { isAuthenticated, user, login, logout } = useAuth();

  const handleLogin = async () => {
    const success = await login('user@example.com', 'password');
    if (success) {
      console.log('Logged in as:', user.email);
    }
  };

  return (
    <div>
      {isAuthenticated ? (
        <div>
          <p>Welcome, {user.email}</p>
          <button onClick={logout}>Logout</button>
        </div>
      ) : (
        <button onClick={handleLogin}>Login</button>
      )}
    </div>
  );
}
```

---

**End of Specification**

**Remember**: Update TRACEABILITY_MATRIX.md when implementation is complete!
