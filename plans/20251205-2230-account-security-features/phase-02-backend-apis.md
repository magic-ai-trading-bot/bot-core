# Phase 02: Backend API Endpoints

**Status**: Pending
**Priority**: High

## Context

- Existing endpoints: /auth/register, /auth/login, /auth/verify, /auth/profile
- Need: 7 new endpoints for security features

## New Endpoints

### 2.1 Change Password
```
POST /api/auth/change-password
Authorization: Bearer <token>
Body: { current_password, new_password }
Response: { success: true }
```

### 2.2 Profile Update
```
PATCH /api/auth/profile
Authorization: Bearer <token>
Body: { display_name }
Response: { success: true, data: UserProfile }
```

### 2.3 2FA Setup
```
POST /api/auth/2fa/setup
Authorization: Bearer <token>
Response: { success: true, data: { secret, qr_code } }
```

### 2.4 2FA Verify (Enable)
```
POST /api/auth/2fa/verify
Authorization: Bearer <token>
Body: { code: "123456" }
Response: { success: true }
```

### 2.5 2FA Disable
```
POST /api/auth/2fa/disable
Authorization: Bearer <token>
Body: { code: "123456" }  // Require code to disable
Response: { success: true }
```

### 2.6 List Sessions
```
GET /api/auth/sessions
Authorization: Bearer <token>
Response: { success: true, data: [Session] }
```

### 2.7 Revoke Session
```
DELETE /api/auth/sessions/:session_id
Authorization: Bearer <token>
Response: { success: true }
```

### 2.8 Revoke All Sessions
```
POST /api/auth/sessions/revoke-all
Authorization: Bearer <token>
Response: { success: true }
```

## Implementation (handlers.rs)

```rust
// Add routes to AuthService::routes()
let change_password = self.change_password_route();
let update_profile = self.update_profile_route();
let setup_2fa = self.setup_2fa_route();
let verify_2fa = self.verify_2fa_route();
let disable_2fa = self.disable_2fa_route();
let list_sessions = self.list_sessions_route();
let revoke_session = self.revoke_session_route();
let revoke_all = self.revoke_all_sessions_route();
```

## Files to Modify

- `rust-core-engine/src/auth/handlers.rs`
- `rust-core-engine/src/auth/mod.rs` (exports)

## Success Criteria

- [ ] All 8 endpoints respond correctly
- [ ] Password change verifies current password
- [ ] 2FA generates valid QR codes
- [ ] Session list returns real data
- [ ] Revoke invalidates sessions immediately
