# Phase 01: Backend Models & Database

**Status**: Pending
**Priority**: High

## Context

- Existing: User model in models.rs, UserRepository in database.rs
- Need: Session model, 2FA fields on User, new DB operations

## Requirements

1. Add 2FA fields to User model
2. Create Session model for tracking active sessions
3. Add database operations for sessions

## Implementation Steps

### 1.1 Update User Model (models.rs)

Add fields:
```rust
pub struct User {
    // ... existing fields ...
    pub two_factor_enabled: bool,
    pub two_factor_secret: Option<String>,  // Encrypted
    pub display_name: Option<String>,
}
```

### 1.2 Create Session Model (models.rs)

```rust
pub struct Session {
    pub id: Option<ObjectId>,
    pub session_id: String,      // UUID
    pub user_id: ObjectId,
    pub device: String,          // "Chrome on MacOS"
    pub location: String,        // "San Francisco, US"
    pub ip_address: String,
    pub user_agent: String,
    pub created_at: DateTime,
    pub last_active: DateTime,
    pub expires_at: DateTime,
    pub revoked: bool,
}
```

### 1.3 Add Request/Response Types

```rust
pub struct ChangePasswordRequest {
    pub current_password: String,
    pub new_password: String,
}

pub struct Setup2FAResponse {
    pub secret: String,
    pub qr_code: String,  // base64
}

pub struct Verify2FARequest {
    pub code: String,
}

pub struct UpdateProfileRequest {
    pub display_name: Option<String>,
}
```

### 1.4 Database Operations (database.rs)

```rust
impl UserRepository {
    // Existing + new:
    async fn update_password(&self, user_id: &ObjectId, hash: String) -> Result<()>
    async fn update_2fa(&self, user_id: &ObjectId, enabled: bool, secret: Option<String>) -> Result<()>
    async fn update_profile(&self, user_id: &ObjectId, display_name: String) -> Result<()>
}

// New SessionRepository
impl SessionRepository {
    async fn create_session(&self, session: Session) -> Result<ObjectId>
    async fn find_by_user(&self, user_id: &ObjectId) -> Result<Vec<Session>>
    async fn find_by_session_id(&self, session_id: &str) -> Result<Option<Session>>
    async fn revoke_session(&self, session_id: &str) -> Result<()>
    async fn revoke_all_except(&self, user_id: &ObjectId, current_session_id: &str) -> Result<()>
    async fn update_last_active(&self, session_id: &str) -> Result<()>
}
```

## Files to Modify

- `rust-core-engine/src/auth/models.rs`
- `rust-core-engine/src/auth/database.rs`
- `rust-core-engine/Cargo.toml` (add totp-rs)

## Success Criteria

- [ ] User model has 2FA fields
- [ ] Session model defined
- [ ] All DB operations compile
- [ ] Unit tests pass
