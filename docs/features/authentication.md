# Authentication & Authorization

## 📍 Quick Reference

### Code Locations
```
rust-core-engine/src/auth/
├── jwt.rs - JWT token generation & validation
│   ├── generate_token() - Create JWT with RS256
│   ├── validate_token() - Verify JWT signature
│   └── refresh_token() - Refresh expired tokens
├── handlers.rs - Auth API endpoints
│   ├── login() - POST /api/auth/login
│   ├── logout() - POST /api/auth/logout
│   ├── register() - POST /api/auth/register
│   └── refresh() - POST /api/auth/refresh
├── middleware.rs - Auth middleware
│   ├── jwt_auth_middleware() - Protect routes
│   └── extract_user_from_token() - Get user from JWT
└── database.rs - User database operations
    ├── find_user_by_email()
    ├── create_user()
    └── update_last_login()
```

### API Endpoints
- `POST /api/auth/register` - Create new user account
- `POST /api/auth/login` - Login and get JWT token
- `POST /api/auth/logout` - Invalidate current session
- `POST /api/auth/refresh` - Refresh JWT token
- `GET /api/auth/me` - Get current user info (protected)

### Database Collections
- `users` - User accounts (email, hashed password, role)
- `sessions` - Active JWT sessions
- `refresh_tokens` - Refresh tokens for token renewal

---

## 🔐 Features

### JWT Authentication
- RS256 algorithm (asymmetric encryption)
- Access token: 15 minutes expiry
- Refresh token: 7 days expiry
- Automatic token rotation

### Role-Based Access Control (RBAC)
- `admin` - Full system access
- `trader` - Trading operations only
- `viewer` - Read-only access

### Security
- Password hashing: bcrypt with cost 12
- Rate limiting: 5 login attempts per 15 minutes
- CORS configuration for frontend
- Secure cookie settings (httpOnly, secure, sameSite)

---

## ⚙️ Configuration

### JWT Settings
```rust
// config.toml
[jwt]
secret_key = "path/to/private_key.pem"
public_key = "path/to/public_key.pem"
access_token_expiry = 900  # 15 minutes
refresh_token_expiry = 604800  # 7 days
```

### Generate Keys
```bash
# Generate RSA key pair
openssl genrsa -out private_key.pem 2048
openssl rsa -in private_key.pem -pubout -out public_key.pem
```

---

## 🚀 Common Tasks

### Login
```bash
curl -X POST http://localhost:8080/api/auth/login \
  -H "Content-Type: application/json" \
  -d '{"email": "user@example.com", "password": "password123"}'

# Response:
# {
#   "access_token": "eyJhbGc...",
#   "refresh_token": "eyJhbGc...",
#   "expires_in": 900
# }
```

### Use Protected Endpoint
```bash
curl -X GET http://localhost:8080/api/paper-trading/status \
  -H "Authorization: Bearer eyJhbGc..."
```

### Refresh Token
```bash
curl -X POST http://localhost:8080/api/auth/refresh \
  -H "Content-Type: application/json" \
  -d '{"refresh_token": "eyJhbGc..."}'
```

---

## 🔧 Troubleshooting

### Issue: "Invalid token" error
**Check**: `rust-core-engine/src/auth/jwt.rs`
- Verify token hasn't expired
- Check JWT_SECRET environment variable
- Ensure public_key.pem exists and is readable

### Issue: Login returns 401 Unauthorized
**Check**: `rust-core-engine/src/auth/handlers.rs`
- Verify email exists in database
- Check password hash matches
- Look for rate limiting logs

### Issue: CORS error from frontend
**Check**: `rust-core-engine/src/main.rs`
- Verify CORS_ALLOWED_ORIGINS includes frontend URL
- Check preflight OPTIONS requests

---

## 📚 Related Documentation

- **Specs**: `specs/01-requirements/1.1-functional-requirements/FR-AUTH.md`
- **Design**: `specs/02-design/2.5-components/COMP-RUST-AUTH.md`
- **Tests**: `specs/03-testing/3.2-test-cases/TC-AUTH.md`

**Last Updated**: 2025-11-20
**Security Score**: 98/100 (A+)
