# Account Security Features Implementation Plan

**Date**: 2025-12-05
**Status**: In Progress
**Priority**: High

## Overview

Implement complete account security features: 2FA, password change, session management, and profile update.

## Phases

| Phase | Name | Status | Files |
|-------|------|--------|-------|
| 01 | Backend Models & Database | Pending | [phase-01](./phase-01-backend-models.md) |
| 02 | Backend API Endpoints | Pending | [phase-02](./phase-02-backend-apis.md) |
| 03 | Frontend Integration | Pending | [phase-03](./phase-03-frontend.md) |
| 04 | Testing & Validation | Pending | [phase-04](./phase-04-testing.md) |

## Tech Stack

- **Backend**: Rust (warp), MongoDB
- **2FA**: totp-rs crate with QR generation
- **Frontend**: React, TypeScript, Axios

## Key Dependencies (Rust)

```toml
totp-rs = { version = "5.6", features = ["qr", "gen_secret"] }
```

## Success Criteria

- [ ] Change password works with current password verification
- [ ] 2FA setup generates QR code, verify enables 2FA
- [ ] Sessions list shows real device/location data
- [ ] Revoke session invalidates that session immediately
- [ ] Sign out all devices works except current session
- [ ] Profile name update persists to database

## Architecture

```
Frontend (Settings.tsx)
    ↓ API calls
Backend (/api/auth/*)
    ↓
MongoDB (users, sessions collections)
```

## Risk Assessment

| Risk | Mitigation |
|------|------------|
| 2FA secret leak | Encrypt with AES-256-GCM before storing |
| Session hijacking | Include device fingerprint validation |
| Brute force | Rate limit password/2FA endpoints |
