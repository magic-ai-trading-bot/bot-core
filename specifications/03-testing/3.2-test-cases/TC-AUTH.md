# Test Cases - Authentication Module

**Document ID:** TC-AUTH-001
**Version:** 1.0
**Last Updated:** 2025-10-11
**Status:** Active
**Related FR:** FR-AUTH (Functional Requirements - Authentication)

---

## Table of Contents

1. [Test Case Summary](#test-case-summary)
2. [User Registration Test Cases](#user-registration-test-cases)
3. [User Login Test Cases](#user-login-test-cases)
4. [JWT Token Test Cases](#jwt-token-test-cases)
5. [Password Management Test Cases](#password-management-test-cases)
6. [Session Management Test Cases](#session-management-test-cases)
7. [Authorization Middleware Test Cases](#authorization-middleware-test-cases)
8. [Security Test Cases](#security-test-cases)
9. [Traceability Matrix](#traceability-matrix)

---

## Test Case Summary

| Category | Total Tests | Priority | Coverage |
|----------|-------------|----------|----------|
| User Registration | 8 | Critical | 100% |
| User Login | 7 | Critical | 100% |
| JWT Token Validation | 9 | Critical | 100% |
| Password Management | 6 | High | 100% |
| Session Management | 4 | High | 100% |
| Authorization Middleware | 5 | Critical | 100% |
| Security | 6 | Critical | 100% |
| **TOTAL** | **45** | - | **100%** |

**Test File Locations:**
- Rust: `/Users/dungngo97/Documents/bot-core/rust-core-engine/tests/test_auth.rs`
- Frontend: `/Users/dungngo97/Documents/bot-core/nextjs-ui-dashboard/src/__tests__/pages/Login.test.tsx`
- Frontend: `/Users/dungngo97/Documents/bot-core/nextjs-ui-dashboard/src/__tests__/pages/Register.test.tsx`
- Frontend: `/Users/dungngo97/Documents/bot-core/nextjs-ui-dashboard/src/__tests__/contexts/AuthContext.test.tsx`

---

## User Registration Test Cases

### TC-AUTH-001: Successful User Registration

**Priority:** Critical
**Test Type:** Integration
**Related FR:** FR-AUTH-001

**Prerequisites:**
- Registration endpoint is available
- Database is accessible
- Email service is configured

**Test Scenario (Gherkin):**
```gherkin
Feature: User Registration
  As a new user
  I want to register an account
  So that I can access the trading platform

  Scenario: Successful registration with valid data
    Given I am on the registration page
    And no user exists with email "newuser@example.com"
    When I enter username "trader123"
    And I enter email "newuser@example.com"
    And I enter password "SecurePass123!@#"
    And I confirm password "SecurePass123!@#"
    And I accept terms and conditions
    And I click "Register" button
    Then I should see success message "Account created successfully"
    And I should receive a verification email
    And I should be redirected to "/dashboard"
    And JWT token should be stored in localStorage
    And user should exist in database with email "newuser@example.com"
```

**Test Steps:**
1. Navigate to registration page
2. Fill in username field: "trader123"
3. Fill in email field: "newuser@example.com"
4. Fill in password field: "SecurePass123!@#"
5. Fill in confirm password field: "SecurePass123!@#"
6. Check "I accept terms and conditions" checkbox
7. Click "Register" button

**Expected Results:**
- ✅ Success message displayed
- ✅ Verification email sent
- ✅ User redirected to dashboard
- ✅ JWT token stored in localStorage
- ✅ User record created in database
- ✅ Password hashed with bcrypt
- ✅ User status set to "pending_verification"

**Actual Results:** [To be filled during execution]

**Status:** [ ] Pass [ ] Fail [ ] Blocked

**Code Location:**
- Rust: `/Users/dungngo97/Documents/bot-core/rust-core-engine/src/auth/handlers.rs::register_user`
- Frontend: `/Users/dungngo97/Documents/bot-core/nextjs-ui-dashboard/src/pages/Register.tsx`

---

### TC-AUTH-002: Registration with Duplicate Email

**Priority:** Critical
**Test Type:** Integration
**Related FR:** FR-AUTH-001

**Prerequisites:**
- User exists with email "existing@example.com"

**Test Scenario (Gherkin):**
```gherkin
  Scenario: Registration fails with duplicate email
    Given a user exists with email "existing@example.com"
    When I try to register with username "newuser"
    And I enter email "existing@example.com"
    And I enter password "SecurePass123!"
    And I click "Register"
    Then I should see error message "Email already exists"
    And no new account should be created
    And I should remain on registration page
```

**Test Steps:**
1. Create existing user with email "existing@example.com"
2. Navigate to registration page
3. Enter username: "newuser"
4. Enter email: "existing@example.com"
5. Enter password: "SecurePass123!"
6. Click "Register"

**Expected Results:**
- ✅ Error message: "Email already exists"
- ✅ No new user record created
- ✅ User remains on registration page
- ✅ HTTP 409 Conflict status returned

**Code Location:**
- Rust: `/Users/dungngo97/Documents/bot-core/rust-core-engine/src/auth/handlers.rs::register_user`

---

### TC-AUTH-003: Registration with Invalid Email Format

**Priority:** High
**Test Type:** Unit
**Related FR:** FR-AUTH-001

**Test Scenario (Gherkin):**
```gherkin
  Scenario: Registration fails with invalid email format
    Given I am on the registration page
    When I enter email "invalid-email"
    And I enter valid username and password
    And I click "Register"
    Then I should see error "Invalid email format"
    And registration should not proceed
```

**Test Steps:**
1. Navigate to registration page
2. Enter email: "invalid-email" (no @ symbol)
3. Enter username: "validuser"
4. Enter password: "SecurePass123!"
5. Click "Register"

**Expected Results:**
- ✅ Validation error displayed: "Invalid email format"
- ✅ Form submission blocked
- ✅ No API call made

**Invalid Email Examples:**
- "plaintext"
- "@example.com"
- "user@"
- "user@.com"
- "user space@example.com"

---

### TC-AUTH-004: Registration with Weak Password

**Priority:** High
**Test Type:** Unit
**Related FR:** FR-AUTH-002

**Test Scenario (Gherkin):**
```gherkin
  Scenario: Registration fails with weak password
    Given I am on the registration page
    When I enter email "user@example.com"
    And I enter password "123"
    And I click "Register"
    Then I should see error "Password must be at least 8 characters"
    And I should see error "Password must contain uppercase, lowercase, number, and special character"
```

**Password Requirements:**
- Minimum 8 characters
- At least one uppercase letter
- At least one lowercase letter
- At least one number
- At least one special character (!@#$%^&*)

**Weak Password Examples:**
| Password | Reason |
|----------|--------|
| "123" | Too short, no letters |
| "password" | No uppercase, number, or special char |
| "Password" | No number or special char |
| "Password123" | No special char |
| "Pass!23" | Too short |

---

### TC-AUTH-005: Registration with Mismatched Passwords

**Priority:** Medium
**Test Type:** Unit
**Related FR:** FR-AUTH-001

**Test Scenario (Gherkin):**
```gherkin
  Scenario: Registration fails when passwords don't match
    Given I am on the registration page
    When I enter password "SecurePass123!"
    And I enter confirm password "DifferentPass456!"
    And I click "Register"
    Then I should see error "Passwords do not match"
```

---

### TC-AUTH-006: Registration with Empty Fields

**Priority:** Medium
**Test Type:** Unit
**Related FR:** FR-AUTH-001

**Test Scenario (Gherkin):**
```gherkin
  Scenario Outline: Registration fails with empty required fields
    Given I am on the registration page
    When I leave <field> empty
    And I fill other fields correctly
    And I click "Register"
    Then I should see error "<error_message>"

    Examples:
      | field            | error_message              |
      | username         | Username is required       |
      | email            | Email is required          |
      | password         | Password is required       |
      | confirm_password | Please confirm password    |
```

---

### TC-AUTH-007: Registration with Username Already Taken

**Priority:** High
**Test Type:** Integration
**Related FR:** FR-AUTH-001

**Test Scenario (Gherkin):**
```gherkin
  Scenario: Registration fails with duplicate username
    Given a user exists with username "trader123"
    When I try to register with username "trader123"
    And I use different email "newemail@example.com"
    And I click "Register"
    Then I should see error "Username already taken"
```

---

### TC-AUTH-008: Registration with SQL Injection Attempt

**Priority:** Critical
**Test Type:** Security
**Related FR:** FR-AUTH-001, SEC-001

**Test Scenario (Gherkin):**
```gherkin
  Scenario: Registration sanitizes SQL injection attempts
    Given I am on the registration page
    When I enter email "'; DROP TABLE users; --"
    And I enter username "admin' OR '1'='1"
    And I enter valid password
    And I click "Register"
    Then input should be sanitized
    And no SQL injection should occur
    And database should remain intact
```

**SQL Injection Test Inputs:**
- `'; DROP TABLE users; --`
- `admin' OR '1'='1`
- `' OR 1=1 --`
- `admin'--`

---

## User Login Test Cases

### TC-AUTH-009: Successful Login with Email

**Priority:** Critical
**Test Type:** Integration
**Related FR:** FR-AUTH-003

**Test Scenario (Gherkin):**
```gherkin
Feature: User Login
  As a registered user
  I want to log in to my account
  So that I can access my trading dashboard

  Scenario: Successful login with valid credentials
    Given I am on the login page
    And a user exists with email "user@example.com" and password "SecurePass123!"
    When I enter email "user@example.com"
    And I enter password "SecurePass123!"
    And I click "Login" button
    Then I should see success message "Login successful"
    And I should be redirected to "/dashboard"
    And JWT token should be stored in localStorage
    And token should contain user_id, email, and is_admin claims
```

**Test Steps:**
1. Create test user with email "user@example.com" and password "SecurePass123!"
2. Navigate to login page
3. Enter email: "user@example.com"
4. Enter password: "SecurePass123!"
5. Click "Login" button

**Expected Results:**
- ✅ Success message displayed
- ✅ User redirected to dashboard
- ✅ JWT token stored in localStorage
- ✅ Token expiration set to 24 hours
- ✅ User session initiated

**Code Location:**
- Rust: `/Users/dungngo97/Documents/bot-core/rust-core-engine/src/auth/handlers.rs::login_user`
- Rust Test: `/Users/dungngo97/Documents/bot-core/rust-core-engine/tests/test_auth.rs::test_password_hash_and_verify`
- Frontend: `/Users/dungngo97/Documents/bot-core/nextjs-ui-dashboard/src/pages/Login.tsx`

---

### TC-AUTH-010: Login with Incorrect Password

**Priority:** Critical
**Test Type:** Integration
**Related FR:** FR-AUTH-003

**Test Scenario (Gherkin):**
```gherkin
  Scenario: Login fails with incorrect password
    Given a user exists with email "user@example.com"
    When I enter email "user@example.com"
    And I enter password "WrongPassword123!"
    And I click "Login"
    Then I should see error "Invalid credentials"
    And I should not be logged in
    And no JWT token should be issued
    And login attempt should be logged
```

**Expected Results:**
- ✅ Error message: "Invalid credentials"
- ✅ User remains on login page
- ✅ No token issued
- ✅ Failed login attempt logged

**Code Location:**
- Rust Test: `/Users/dungngo97/Documents/bot-core/rust-core-engine/tests/test_auth.rs::test_password_hash_and_verify` (line 15)

---

### TC-AUTH-011: Login with Non-Existent Email

**Priority:** High
**Test Type:** Integration
**Related FR:** FR-AUTH-003

**Test Scenario (Gherkin):**
```gherkin
  Scenario: Login fails with non-existent email
    Given no user exists with email "nonexistent@example.com"
    When I enter email "nonexistent@example.com"
    And I enter any password
    And I click "Login"
    Then I should see error "Invalid credentials"
    And I should not be logged in
```

**Security Note:** Error message should be generic to prevent email enumeration.

---

### TC-AUTH-012: Login with Empty Credentials

**Priority:** Medium
**Test Type:** Unit
**Related FR:** FR-AUTH-003

**Test Scenario (Gherkin):**
```gherkin
  Scenario Outline: Login fails with empty fields
    Given I am on the login page
    When I leave <field> empty
    And I click "Login"
    Then I should see error "<error_message>"
    And login should not proceed

    Examples:
      | field    | error_message        |
      | email    | Email is required    |
      | password | Password is required |
      | both     | All fields required  |
```

---

### TC-AUTH-013: Login Rate Limiting

**Priority:** Critical
**Test Type:** Security
**Related FR:** FR-AUTH-003, SEC-002

**Test Scenario (Gherkin):**
```gherkin
  Scenario: Login rate limiting prevents brute force attacks
    Given I am on the login page
    When I attempt to login 5 times with wrong password
    And I attempt to login the 6th time
    Then I should see error "Too many login attempts. Try again in 15 minutes"
    And my IP should be temporarily blocked
    And login should be prevented for 15 minutes
```

**Rate Limit Configuration:**
- Max attempts: 5
- Time window: 15 minutes
- Block duration: 15 minutes

---

### TC-AUTH-014: Login with Remember Me

**Priority:** Medium
**Test Type:** Integration
**Related FR:** FR-AUTH-003

**Test Scenario (Gherkin):**
```gherkin
  Scenario: Login with "Remember Me" extends token expiration
    Given I am on the login page
    When I enter valid credentials
    And I check "Remember Me" checkbox
    And I click "Login"
    Then JWT token expiration should be 7 days
    And token should be stored in localStorage
```

---

### TC-AUTH-015: Login After Account Locked

**Priority:** High
**Test Type:** Integration
**Related FR:** FR-AUTH-003

**Test Scenario (Gherkin):**
```gherkin
  Scenario: Login fails for locked account
    Given my account is locked due to suspicious activity
    When I enter valid credentials
    And I click "Login"
    Then I should see error "Account is locked. Contact support."
    And I should not be logged in
```

---

## JWT Token Test Cases

### TC-AUTH-016: JWT Token Generation

**Priority:** Critical
**Test Type:** Unit
**Related FR:** FR-AUTH-004

**Test Scenario (Gherkin):**
```gherkin
Feature: JWT Token Management
  As the authentication system
  I want to generate secure JWT tokens
  So that users can access protected resources

  Scenario: Generate valid JWT token
    Given JWT service is initialized with secret "super_secret_key"
    When I generate token for user_id "user_id_123"
    And email "user@example.com"
    And is_admin flag is true
    Then token should be a valid JWT string
    And token should contain subject "user_id_123"
    And token should contain email "user@example.com"
    And token should contain is_admin claim as true
    And token should have expiration timestamp
```

**Code Location:**
- Rust: `/Users/dungngo97/Documents/bot-core/rust-core-engine/src/auth/jwt.rs::JwtService::generate_token`
- Rust Test: `/Users/dungngo97/Documents/bot-core/rust-core-engine/tests/test_auth.rs::test_jwt_token_generation_and_verification` (line 23-39)

---

### TC-AUTH-017: JWT Token Verification

**Priority:** Critical
**Test Type:** Unit
**Related FR:** FR-AUTH-004

**Test Scenario (Gherkin):**
```gherkin
  Scenario: Verify valid JWT token
    Given a valid JWT token exists
    When I verify the token with correct secret
    Then verification should succeed
    And claims should be extracted correctly
    And user_id should match original value
    And email should match original value
```

**Code Location:**
- Rust Test: `/Users/dungngo97/Documents/bot-core/rust-core-engine/tests/test_auth.rs::test_jwt_token_generation_and_verification` (line 31-35)

---

### TC-AUTH-018: JWT Token with Invalid Signature

**Priority:** Critical
**Test Type:** Security
**Related FR:** FR-AUTH-004

**Test Scenario (Gherkin):**
```gherkin
  Scenario: Reject token with invalid signature
    Given a JWT token signed with secret "secret_A"
    When I verify token with different secret "secret_B"
    Then verification should fail
    And error should be "Invalid signature"
    And no claims should be returned
```

**Code Location:**
- Rust Test: `/Users/dungngo97/Documents/bot-core/rust-core-engine/tests/test_auth.rs::test_jwt_invalid_token` (line 56-71)

---

### TC-AUTH-019: JWT Token Expiration

**Priority:** Critical
**Test Type:** Unit
**Related FR:** FR-AUTH-004

**Test Scenario (Gherkin):**
```gherkin
  Scenario: Token expires after configured duration
    Given JWT service with 24 hour expiration
    When I generate a token
    Then token expiration should be 24 hours from now
    And token should be valid before expiration
    And token should be invalid after expiration
```

**Code Location:**
- Rust Test: `/Users/dungngo97/Documents/bot-core/rust-core-engine/tests/test_auth.rs::test_jwt_token_expiration` (line 42-53)

---

### TC-AUTH-020: JWT Token with Malformed Format

**Priority:** Critical
**Test Type:** Security
**Related FR:** FR-AUTH-004

**Test Scenario (Gherkin):**
```gherkin
  Scenario: Reject malformed JWT tokens
    Given I have a malformed token "invalid.token.here"
    When I attempt to verify the token
    Then verification should fail
    And error should indicate invalid format
```

**Test Inputs:**
- "invalid.token.here"
- "not-a-jwt"
- "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9" (incomplete)
- "" (empty string)

**Code Location:**
- Rust Test: `/Users/dungngo97/Documents/bot-core/rust-core-engine/tests/test_auth.rs::test_jwt_invalid_token` (line 59-61)

---

### TC-AUTH-021: Extract Token from Authorization Header

**Priority:** High
**Test Type:** Unit
**Related FR:** FR-AUTH-004

**Test Scenario (Gherkin):**
```gherkin
  Scenario: Extract JWT from Bearer token header
    Given Authorization header "Bearer eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9"
    When I extract token from header
    Then extracted token should be "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9"
```

**Test Cases:**
| Header | Expected Token |
|--------|----------------|
| "Bearer ABC123" | "ABC123" |
| "Basic dXNlcjpwYXNz" | None (invalid scheme) |
| "ABC123" | None (missing Bearer) |
| "" | None (empty) |

**Code Location:**
- Rust: `/Users/dungngo97/Documents/bot-core/rust-core-engine/src/auth/jwt.rs::JwtService::extract_token_from_header`
- Rust Test: `/Users/dungngo97/Documents/bot-core/rust-core-engine/tests/test_auth.rs::test_extract_token_from_header` (line 74-89)

---

### TC-AUTH-022: JWT Token for Admin vs Regular User

**Priority:** High
**Test Type:** Unit
**Related FR:** FR-AUTH-004

**Test Scenario (Gherkin):**
```gherkin
  Scenario: Generate tokens with different privilege levels
    Given I generate admin token for "admin@company.com" with is_admin=true
    And I generate user token for "user@company.com" with is_admin=false
    When I verify both tokens
    Then admin token should have is_admin claim as true
    And user token should have is_admin claim as false
    And tokens should be different
```

**Code Location:**
- Rust Test: `/Users/dungngo97/Documents/bot-core/rust-core-engine/tests/test_auth.rs::test_jwt_with_different_users` (line 92-116)

---

### TC-AUTH-023: JWT Token Default Expiration

**Priority:** Medium
**Test Type:** Unit
**Related FR:** FR-AUTH-004

**Test Scenario (Gherkin):**
```gherkin
  Scenario: JWT token uses default 24-hour expiration
    Given JWT service initialized without expiration parameter
    When I generate a token
    Then token expiration should default to 24 hours
```

**Code Location:**
- Rust Test: `/Users/dungngo97/Documents/bot-core/rust-core-engine/tests/test_auth.rs::test_jwt_default_expiration` (line 136-149)

---

### TC-AUTH-024: JWT Claims Serialization

**Priority:** Medium
**Test Type:** Unit
**Related FR:** FR-AUTH-004

**Test Scenario (Gherkin):**
```gherkin
  Scenario: JWT claims serialize and deserialize correctly
    Given a Claims object with user data
    When I serialize claims to JSON
    Then JSON should contain all claim fields
    When I deserialize JSON back to Claims
    Then deserialized object should match original
```

**Code Location:**
- Rust Test: `/Users/dungngo97/Documents/bot-core/rust-core-engine/tests/test_auth.rs::test_claims_serialization` (line 152-171)

---

## Password Management Test Cases

### TC-AUTH-025: Password Hashing with Bcrypt

**Priority:** Critical
**Test Type:** Unit
**Related FR:** FR-AUTH-002

**Test Scenario (Gherkin):**
```gherkin
Feature: Password Security
  As the authentication system
  I want to securely hash passwords
  So that plaintext passwords are never stored

  Scenario: Hash password with bcrypt
    Given plaintext password "SecurePassword123!@#"
    When I hash the password
    Then hashed value should not equal plaintext
    And hash should start with "$2" (bcrypt identifier)
    And I should be able to verify plaintext against hash
```

**Code Location:**
- Rust: `/Users/dungngo97/Documents/bot-core/rust-core-engine/src/auth/jwt.rs::PasswordService::hash_password`
- Rust Test: `/Users/dungngo97/Documents/bot-core/rust-core-engine/tests/test_auth.rs::test_password_hash_and_verify` (line 7-20)

---

### TC-AUTH-026: Password Hash Uniqueness

**Priority:** High
**Test Type:** Unit
**Related FR:** FR-AUTH-002

**Test Scenario (Gherkin):**
```gherkin
  Scenario: Same password generates different hashes
    Given plaintext password "MyPassword123!"
    When I hash the password twice
    Then both hashes should be different
    And both hashes should verify against plaintext
```

**Reason:** Bcrypt uses random salt, ensuring unique hashes

**Code Location:**
- Rust Test: `/Users/dungngo97/Documents/bot-core/rust-core-engine/tests/test_auth.rs::test_password_hash_and_verify` (line 17-19)

---

### TC-AUTH-027: Password Verification

**Priority:** Critical
**Test Type:** Unit
**Related FR:** FR-AUTH-002

**Test Scenario (Gherkin):**
```gherkin
  Scenario: Verify correct password succeeds
    Given password hash for "CorrectPassword123!"
    When I verify "CorrectPassword123!" against hash
    Then verification should succeed

  Scenario: Verify incorrect password fails
    Given password hash for "CorrectPassword123!"
    When I verify "WrongPassword456!" against hash
    Then verification should fail
```

**Code Location:**
- Rust: `/Users/dungngo97/Documents/bot-core/rust-core-engine/src/auth/jwt.rs::PasswordService::verify_password`
- Rust Test: `/Users/dungngo97/Documents/bot-core/rust-core-engine/tests/test_auth.rs::test_password_hash_and_verify` (line 11-15)

---

### TC-AUTH-028: Password Hash with Special Characters

**Priority:** High
**Test Type:** Unit
**Related FR:** FR-AUTH-002

**Test Scenario (Gherkin):**
```gherkin
  Scenario: Hash and verify password with special characters
    Given password "!@#$%^&*()_+-=[]{}|;':,.<>?/~`"
    When I hash the password
    And I verify the password against hash
    Then verification should succeed
```

**Code Location:**
- Rust Test: `/Users/dungngo97/Documents/bot-core/rust-core-engine/tests/test_auth.rs::test_password_hash_errors` (line 130-132)

---

### TC-AUTH-029: Password Hash Edge Cases

**Priority:** Medium
**Test Type:** Unit
**Related FR:** FR-AUTH-002

**Test Scenario (Gherkin):**
```gherkin
  Scenario Outline: Hash various password edge cases
    Given password "<password>"
    When I hash the password
    Then hashing should succeed
    And verification should work correctly

    Examples:
      | password              |
      | ""                    | # Empty password
      | "a"                   | # Single character
      | "aaa...aaa" (1000 chars) | # Very long password
      | "日本語パスワード"      | # Unicode characters
```

**Code Location:**
- Rust Test: `/Users/dungngo97/Documents/bot-core/rust-core-engine/tests/test_auth.rs::test_password_hash_errors` (line 119-133)

---

### TC-AUTH-030: Password Reset Flow

**Priority:** High
**Test Type:** Integration
**Related FR:** FR-AUTH-005

**Test Scenario (Gherkin):**
```gherkin
  Scenario: User resets forgotten password
    Given I am on the "Forgot Password" page
    When I enter my email "user@example.com"
    And I click "Send Reset Link"
    Then I should receive password reset email
    And email should contain reset token
    When I click reset link with valid token
    Then I should be redirected to "Reset Password" page
    When I enter new password "NewSecurePass123!"
    And I confirm new password
    And I click "Reset Password"
    Then my password should be updated
    And I should be able to login with new password
    And old password should no longer work
```

---

## Session Management Test Cases

### TC-AUTH-031: Session Creation on Login

**Priority:** High
**Test Type:** Integration
**Related FR:** FR-AUTH-006

**Test Scenario (Gherkin):**
```gherkin
Feature: Session Management
  As the authentication system
  I want to manage user sessions
  So that authenticated users maintain their login state

  Scenario: Create session on successful login
    Given I login with valid credentials
    When login succeeds
    Then a new session should be created
    And session should be stored in database
    And session ID should be included in JWT
    And session should have creation timestamp
```

---

### TC-AUTH-032: Session Validation on Protected Routes

**Priority:** Critical
**Test Type:** Integration
**Related FR:** FR-AUTH-006

**Test Scenario (Gherkin):**
```gherkin
  Scenario: Validate session on protected route access
    Given I am logged in with active session
    When I access a protected route "/api/trades"
    Then session should be validated
    And request should be authorized
    And route should return data

  Scenario: Reject access with invalid session
    Given I have an expired session
    When I access a protected route
    Then I should receive 401 Unauthorized
    And I should be redirected to login
```

---

### TC-AUTH-033: Session Logout

**Priority:** High
**Test Type:** Integration
**Related FR:** FR-AUTH-006

**Test Scenario (Gherkin):**
```gherkin
  Scenario: Logout invalidates session
    Given I am logged in
    When I click "Logout"
    Then my session should be invalidated
    And JWT token should be removed from localStorage
    And I should be redirected to login page
    And I should not be able to access protected routes
```

---

### TC-AUTH-034: Concurrent Session Limit

**Priority:** Medium
**Test Type:** Integration
**Related FR:** FR-AUTH-006

**Test Scenario (Gherkin):**
```gherkin
  Scenario: Limit concurrent sessions per user
    Given I am logged in on Device A
    When I login on Device B
    And concurrent session limit is 2
    Then both sessions should be active
    When I login on Device C
    Then oldest session (Device A) should be invalidated
```

---

## Authorization Middleware Test Cases

### TC-AUTH-035: Protected Route Authorization

**Priority:** Critical
**Test Type:** Integration
**Related FR:** FR-AUTH-007

**Test Scenario (Gherkin):**
```gherkin
Feature: Authorization Middleware
  As the API
  I want to protect routes with JWT authentication
  So that only authorized users can access them

  Scenario: Access protected route with valid token
    Given I have a valid JWT token
    When I make request to "/api/account" with Authorization header
    Then request should be authorized
    And endpoint should return data
    And status should be 200 OK

  Scenario: Access protected route without token
    Given I have no JWT token
    When I make request to "/api/account"
    Then request should be rejected
    And status should be 401 Unauthorized
    And error should be "Authentication required"
```

---

### TC-AUTH-036: Admin-Only Route Authorization

**Priority:** Critical
**Test Type:** Integration
**Related FR:** FR-AUTH-007

**Test Scenario (Gherkin):**
```gherkin
  Scenario: Admin accesses admin-only route
    Given I have admin JWT token (is_admin=true)
    When I access "/api/admin/users"
    Then request should be authorized
    And endpoint should return data

  Scenario: Regular user cannot access admin route
    Given I have regular user token (is_admin=false)
    When I access "/api/admin/users"
    Then request should be rejected
    And status should be 403 Forbidden
    And error should be "Admin privileges required"
```

---

### TC-AUTH-037: Token Expiration Handling

**Priority:** High
**Test Type:** Integration
**Related FR:** FR-AUTH-007

**Test Scenario (Gherkin):**
```gherkin
  Scenario: Expired token rejected by middleware
    Given I have an expired JWT token
    When I make request to protected route
    Then request should be rejected
    And status should be 401 Unauthorized
    And error should be "Token expired"
    And I should be redirected to login
```

---

### TC-AUTH-038: Middleware Performance

**Priority:** Medium
**Test Type:** Performance
**Related FR:** FR-AUTH-007

**Test Scenario (Gherkin):**
```gherkin
  Scenario: Authorization middleware has minimal latency
    Given I make 1000 requests to protected endpoint
    When I measure middleware latency
    Then P95 latency should be < 5ms
    And P99 latency should be < 10ms
```

---

### TC-AUTH-039: Token Refresh Mechanism

**Priority:** High
**Test Type:** Integration
**Related FR:** FR-AUTH-007

**Test Scenario (Gherkin):**
```gherkin
  Scenario: Refresh token before expiration
    Given my token expires in 5 minutes
    When I make request to "/api/refresh-token"
    Then I should receive new token
    And new token should have extended expiration
    And old token should be invalidated
```

---

## Security Test Cases

### TC-AUTH-040: SQL Injection Prevention

**Priority:** Critical
**Test Type:** Security
**Related FR:** SEC-001

**Test Scenario (Gherkin):**
```gherkin
Feature: Security Testing
  As the security team
  I want to ensure authentication is secure
  So that user accounts are protected

  Scenario: Prevent SQL injection in login
    Given I am on login page
    When I enter email "admin' OR '1'='1"
    And I enter password "' OR '1'='1"
    And I click "Login"
    Then input should be sanitized
    And no SQL injection should occur
    And login should fail with "Invalid credentials"
```

**Test Inputs:**
- `admin' OR '1'='1`
- `' OR 1=1 --`
- `'; DROP TABLE users; --`

---

### TC-AUTH-041: XSS Prevention in Registration

**Priority:** Critical
**Test Type:** Security
**Related FR:** SEC-002

**Test Scenario (Gherkin):**
```gherkin
  Scenario: Prevent XSS in username field
    Given I am on registration page
    When I enter username "<script>alert('XSS')</script>"
    And I submit registration
    Then script should be escaped
    And no JavaScript should execute
    And username should be stored as plain text
```

**XSS Test Inputs:**
- `<script>alert('XSS')</script>`
- `<img src=x onerror=alert('XSS')>`
- `javascript:alert('XSS')`

---

### TC-AUTH-042: CSRF Protection

**Priority:** High
**Test Type:** Security
**Related FR:** SEC-003

**Test Scenario (Gherkin):**
```gherkin
  Scenario: Prevent CSRF attacks on login
    Given attacker creates malicious page
    When victim clicks malicious link while logged in
    Then login endpoint should require CSRF token
    And request without CSRF token should be rejected
    And status should be 403 Forbidden
```

---

### TC-AUTH-043: Timing Attack Prevention

**Priority:** High
**Test Type:** Security
**Related FR:** SEC-004

**Test Scenario (Gherkin):**
```gherkin
  Scenario: Login response time is constant
    Given I measure login response time for valid user
    And I measure login response time for invalid user
    Then both response times should be similar
    And timing should not reveal user existence
```

---

### TC-AUTH-044: Brute Force Protection

**Priority:** Critical
**Test Type:** Security
**Related FR:** SEC-005

**Test Scenario (Gherkin):**
```gherkin
  Scenario: Rate limiting prevents brute force
    Given I attempt login 100 times in 1 minute
    Then after 5 attempts I should be rate limited
    And I should see "Too many attempts"
    And I should be blocked for 15 minutes
```

---

### TC-AUTH-045: Secure Password Storage

**Priority:** Critical
**Test Type:** Security
**Related FR:** SEC-006

**Test Scenario (Gherkin):**
```gherkin
  Scenario: Passwords never stored in plaintext
    Given I register with password "MyPassword123!"
    When I inspect database
    Then password field should be bcrypt hash
    And hash should start with "$2"
    And plaintext password should not exist anywhere
```

---

## Traceability Matrix

| Test Case ID | Related FR | Related Code | Priority | Status |
|--------------|------------|--------------|----------|--------|
| TC-AUTH-001 | FR-AUTH-001 | `auth/handlers.rs::register_user` | Critical | ✅ |
| TC-AUTH-002 | FR-AUTH-001 | `auth/handlers.rs::register_user` | Critical | ✅ |
| TC-AUTH-009 | FR-AUTH-003 | `auth/handlers.rs::login_user` | Critical | ✅ |
| TC-AUTH-010 | FR-AUTH-003 | `auth/handlers.rs::login_user` | Critical | ✅ |
| TC-AUTH-016 | FR-AUTH-004 | `auth/jwt.rs::generate_token` | Critical | ✅ |
| TC-AUTH-017 | FR-AUTH-004 | `auth/jwt.rs::verify_token` | Critical | ✅ |
| TC-AUTH-025 | FR-AUTH-002 | `auth/jwt.rs::hash_password` | Critical | ✅ |
| TC-AUTH-027 | FR-AUTH-002 | `auth/jwt.rs::verify_password` | Critical | ✅ |
| TC-AUTH-035 | FR-AUTH-007 | `middleware/auth.rs` | Critical | ✅ |
| TC-AUTH-040 | SEC-001 | Input sanitization | Critical | ✅ |
| TC-AUTH-044 | SEC-005 | Rate limiting middleware | Critical | ✅ |

---

## Acceptance Criteria

Authentication module is considered complete when:

- [ ] All 45 test cases pass
- [ ] Code coverage >= 95% for authentication module
- [ ] No critical security vulnerabilities
- [ ] JWT tokens properly signed and validated
- [ ] Passwords hashed with bcrypt (cost factor >= 10)
- [ ] Rate limiting prevents brute force (5 attempts per 15 min)
- [ ] Session management functional
- [ ] Admin authorization works correctly
- [ ] All edge cases handled gracefully
- [ ] Performance: Login completes in < 200ms (P95)
- [ ] Documentation updated

---

## Test Execution Summary

**Last Execution Date:** [To be filled]
**Executed By:** [Name]

| Category | Total | Passed | Failed | Blocked |
|----------|-------|--------|--------|---------|
| Registration | 8 | - | - | - |
| Login | 7 | - | - | - |
| JWT Tokens | 9 | - | - | - |
| Password Mgmt | 6 | - | - | - |
| Sessions | 4 | - | - | - |
| Authorization | 5 | - | - | - |
| Security | 6 | - | - | - |
| **TOTAL** | **45** | **-** | **-** | **-** |

---

**Document Control:**
- **Created by**: QA Team
- **Reviewed by**: Security Team, Engineering Team
- **Approved by**: Product Owner
- **Next Review Date**: 2025-11-11

---

*End of Authentication Test Cases Document*
