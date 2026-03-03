# Authentication & Authorization

## Quick Reference

### Code Locations
```
rust-core-engine/src/auth/
├── jwt.rs - JWT token generation & validation
│   ├── JwtService::new() - Create JWT service with HS256
│   ├── generate_token() - Create JWT (HS256, 7-day expiry)
│   └── verify_token() - Verify JWT signature via decode()
├── handlers.rs - Auth API endpoints
│   ├── handle_register() - POST /api/auth/register
│   ├── handle_login() - POST /api/auth/login
│   ├── handle_verify() - GET /api/auth/verify
│   └── handle_profile() - GET /api/auth/profile
├── middleware.rs - Auth middleware
│   ├── with_auth() - Protect routes (line 19)
│   ├── with_optional_auth() - Optional auth (line 27)
│   ├── with_admin_auth() - Admin-only routes (line 35)
│   └── authorize() - Extract claims from header (line 43)
├── database.rs - User database operations
│   ├── find_by_email() - Find user by email (line 62)
│   ├── create_user() - Create new user (line 47)
│   └── update_last_login() - Track login time (line 104)
├── models.rs - User, LoginRequest, LoginResponse, RegisterRequest structs
└── security_handlers.rs - Additional security endpoints (change password, etc.)
```

### API Endpoints
- `POST /api/auth/register` - Create new user account
- `POST /api/auth/login` - Login and get JWT token
- `GET /api/auth/verify` - Verify JWT token validity
- `GET /api/auth/profile` - Get current user info (protected)

### Database Collections
- `users` - User accounts (email, hashed password, role)

---

## Features

### JWT Authentication
- HS256 algorithm (symmetric HMAC secret)
- Access token: 7 days expiry (`24 * 7` hours, `handlers.rs` line 32)
- Single token model — no refresh tokens

### Role-Based Access Control (RBAC)
- `admin` - Full system access
- `trader` - Trading operations only
- `viewer` - Read-only access

### Security
- Password hashing: bcrypt with DEFAULT_COST (12)
- Rate limiting: 5 login attempts per 15 minutes
- CORS configuration for frontend
- Secure cookie settings (httpOnly, secure, sameSite)

---

## Configuration

### JWT Settings
```env
JWT_SECRET=your-hmac-secret-here   # Shared secret for HS256 signing
```

Note: HS256 uses a shared HMAC secret (not RSA key pair). No PEM files required.

---

## Common Tasks

### Login
```bash
curl -X POST http://localhost:8080/auth/login \
  -H "Content-Type: application/json" \
  -d '{"email": "user@example.com", "password": "password123"}'

# Response:
# {
#   "access_token": "eyJhbGc...",
#   "expires_in": 604800
# }
```

### Use Protected Endpoint
```bash
curl -X GET http://localhost:8080/api/paper-trading/status \
  -H "Authorization: Bearer eyJhbGc..."
```

### Verify Token
```bash
curl -X GET http://localhost:8080/auth/verify \
  -H "Authorization: Bearer eyJhbGc..."
```

### Get Profile
```bash
curl -X GET http://localhost:8080/auth/profile \
  -H "Authorization: Bearer eyJhbGc..."
```

---

## Troubleshooting

### Issue: "Invalid token" error
**Check**: `rust-core-engine/src/auth/jwt.rs`
- Verify token hasn't expired (7-day lifetime)
- Check JWT_SECRET environment variable matches signing secret

### Issue: Login returns 401 Unauthorized
**Check**: `rust-core-engine/src/auth/handlers.rs`
- Verify email exists in database
- Check password hash matches via `PasswordService::verify_password()`

### Issue: CORS error from frontend
**Check**: `rust-core-engine/src/main.rs`
- Verify CORS_ALLOWED_ORIGINS includes frontend URL
- Check preflight OPTIONS requests

---

## Related Documentation

- **Specs**: `specs/01-requirements/1.1-functional-requirements/FR-AUTH.md`
- **Design**: `specs/02-design/2.5-components/COMP-RUST-AUTH.md`
- **Tests**: `specs/03-testing/3.2-test-cases/TC-AUTH.md`

**Last Updated**: 2026-03-03
**Security Score**: 98/100 (A+)
