# Test Cases - Integration Module

**Document ID:** TC-INTEGRATION-001
**Version:** 1.0
**Last Updated:** 2025-10-11
**Status:** Active
**Related FR:** FR-INTEGRATION (Functional Requirements - Integration)

---

## Table of Contents

1. [Test Case Summary](#test-case-summary)
2. [Rust ↔ Python AI Integration](#rust--python-ai-integration)
3. [Rust ↔ Frontend API Integration](#rust--frontend-api-integration)
4. [Rust ↔ Binance API Integration](#rust--binance-api-integration)
5. [Rust ↔ MongoDB Integration](#rust--mongodb-integration)
6. [WebSocket Real-Time Updates](#websocket-real-time-updates)
7. [End-to-End Workflows](#end-to-end-workflows)
8. [Cross-Service Authentication](#cross-service-authentication)
9. [Error Propagation](#error-propagation)

---

## Test Case Summary

| Category | Total Tests | Priority | Coverage |
|----------|-------------|----------|----------|
| Rust ↔ Python AI | 6 | Critical | 100% |
| Rust ↔ Frontend | 7 | Critical | 100% |
| Rust ↔ Binance | 5 | Critical | 100% |
| Rust ↔ MongoDB | 5 | Critical | 100% |
| WebSocket Integration | 6 | Critical | 100% |
| End-to-End Workflows | 8 | Critical | 100% |
| Cross-Service Auth | 4 | High | 100% |
| Error Propagation | 4 | High | 100% |
| **TOTAL** | **45** | - | **100%** |

**Test File Locations:**
- Rust: `/Users/dungngo97/Documents/bot-core/rust-core-engine/tests/test_cross_service.rs`
- Rust: `/Users/dungngo97/Documents/bot-core/rust-core-engine/tests/test_service_integration.rs`
- Python: `/Users/dungngo97/Documents/bot-core/python-ai-service/tests/test_integration.py`
- Python: `/Users/dungngo97/Documents/bot-core/python-ai-service/tests/test_full_integration.py`
- E2E: `/Users/dungngo97/Documents/bot-core/tests/e2e-cross-service/test_full_system.py`

---

## Rust ↔ Python AI Integration

### TC-INT-001: Request AI Analysis from Rust

**Priority:** Critical
**Test Type:** Integration
**Related FR:** FR-INT-001

**Test Scenario (Gherkin):**
```gherkin
Feature: Rust to Python AI Service Integration
  As the Rust trading engine
  I want to request AI analysis from Python service
  So that I can incorporate ML predictions into trading decisions

  Scenario: Successfully request AI analysis
    Given Python AI service is running on port 8000
    And I have market data for BTC/USDT
    When Rust engine makes POST request to /api/analyze
    With symbol="BTCUSDT", interval="1h", limit=100
    Then Python service should return analysis
    And response should include signal, confidence, reasoning
    And response time should be < 5000ms
    And HTTP status should be 200 OK
```

**Code Location:**
- Rust: `/Users/dungngo97/Documents/bot-core/rust-core-engine/src/ai/client.rs`
- Test: `/Users/dungngo97/Documents/bot-core/rust-core-engine/tests/test_cross_service.rs`

---

### TC-INT-002: Handle Python Service Timeout

**Priority:** High
**Test Type:** Integration
**Related FR:** FR-INT-001

**Test Scenario (Gherkin):**
```gherkin
  Scenario: Handle Python service timeout
    Given Python service is slow to respond
    When request takes > 30 seconds
    Then Rust should timeout gracefully
    And return fallback technical analysis
    And log timeout error
```

---

### TC-INT-003: Handle Python Service Unavailable

**Priority:** Critical
**Test Type:** Integration
**Related FR:** FR-INT-001

**Test Scenario (Gherkin):**
```gherkin
  Scenario: Handle Python service unavailable
    Given Python service is down
    When Rust attempts to request analysis
    Then Rust should catch connection error
    And fall back to local indicators
    And continue trading with degraded AI
```

---

### TC-INT-004: Retry Failed Python Requests

**Priority:** High
**Test Type:** Integration
**Related FR:** FR-INT-001

**Test Scenario (Gherkin):**
```gherkin
  Scenario: Retry failed requests with exponential backoff
    Given Python service returns 500 Internal Server Error
    When Rust receives error
    Then Rust should retry up to 3 times
    With exponential backoff (1s, 2s, 4s)
    If all retries fail, use fallback
```

---

### TC-INT-005: Validate AI Response Schema

**Priority:** High
**Test Type:** Integration
**Related FR:** FR-INT-001

**Test Scenario (Gherkin):**
```gherkin
  Scenario: Validate AI response matches expected schema
    Given Python returns AI analysis
    When Rust parses response
    Then all required fields must be present:
      - signal (string)
      - confidence (f64, 0-1)
      - reasoning (string)
      - timestamp (i64)
    And signal must be one of: Long, Short, Neutral
```

---

### TC-INT-006: Multiple Concurrent AI Requests

**Priority:** Medium
**Test Type:** Integration
**Related FR:** FR-INT-001

**Test Scenario (Gherkin):**
```gherkin
  Scenario: Handle multiple concurrent AI requests
    Given Rust needs analysis for BTC, ETH, SOL simultaneously
    When Rust makes 3 concurrent requests to Python
    Then all requests should complete successfully
    And responses should not interfere with each other
    And total time should be < 6000ms (parallel processing)
```

---

## Rust ↔ Frontend API Integration

### TC-INT-007: Frontend Login via Rust API

**Priority:** Critical
**Test Type:** Integration
**Related FR:** FR-INT-002

**Test Scenario (Gherkin):**
```gherkin
Feature: Frontend to Rust API Integration
  As the frontend application
  I want to communicate with Rust backend
  So that users can interact with the trading system

  Scenario: User logs in via frontend
    Given frontend is running on port 3000
    And Rust API is running on port 8080
    When user submits login form with email and password
    Then frontend makes POST to /api/auth/login
    And Rust validates credentials
    And returns JWT token
    And frontend stores token in localStorage
```

---

### TC-INT-008: Fetch Account Balance

**Priority:** Critical
**Test Type:** Integration
**Related FR:** FR-INT-002

**Test Scenario (Gherkin):**
```gherkin
  Scenario: Frontend fetches user account balance
    Given user is authenticated with JWT
    When frontend makes GET /api/account
    With Authorization: Bearer <token>
    Then Rust validates JWT
    And returns account balance
    And frontend displays balance
```

---

### TC-INT-009: Place Order from Frontend

**Priority:** Critical
**Test Type:** Integration
**Related FR:** FR-INT-002

**Test Scenario (Gherkin):**
```gherkin
  Scenario: User places trade from frontend
    Given user is on trading interface
    When user submits market buy order for 0.1 BTC
    Then frontend makes POST /api/orders
    And Rust executes trade
    And returns order confirmation
    And frontend updates UI with new position
```

---

### TC-INT-010: CORS Configuration

**Priority:** High
**Test Type:** Integration
**Related FR:** FR-INT-002

**Test Scenario (Gherkin):**
```gherkin
  Scenario: CORS allows frontend requests
    Given frontend is on http://localhost:3000
    And Rust API is on http://localhost:8080
    When frontend makes CORS preflight request
    Then Rust should return proper CORS headers
    And allow Origin: http://localhost:3000
```

---

### TC-INT-011: API Rate Limiting

**Priority:** High
**Test Type:** Integration
**Related FR:** FR-INT-002

**Test Scenario (Gherkin):**
```gherkin
  Scenario: API rate limiting protects backend
    Given rate limit is 100 requests/minute per user
    When frontend makes 101 requests in 1 minute
    Then request #101 should return 429 Too Many Requests
    And frontend should show rate limit message
```

---

### TC-INT-012: Frontend Pagination

**Priority:** Medium
**Test Type:** Integration
**Related FR:** FR-INT-002

**Test Scenario (Gherkin):**
```gherkin
  Scenario: Paginate large trade history
    Given user has 1000 trades
    When frontend requests trades with ?page=1&limit=50
    Then Rust returns 50 trades
    And includes pagination metadata
    And frontend renders pagination controls
```

---

### TC-INT-013: Real-Time Error Display

**Priority:** High
**Test Type:** Integration
**Related FR:** FR-INT-002

**Test Scenario (Gherkin):**
```gherkin
  Scenario: Display API errors to user
    Given user attempts invalid operation
    When Rust API returns 400 Bad Request
    With error message "Insufficient balance"
    Then frontend should display error toast
    And show user-friendly message
```

---

## Rust ↔ Binance API Integration

### TC-INT-014: Fetch Market Data from Binance

**Priority:** Critical
**Test Type:** Integration
**Related FR:** FR-INT-003

**Test Scenario (Gherkin):**
```gherkin
Feature: Rust to Binance API Integration
  As the Rust trading engine
  I want to fetch market data from Binance
  So that I can execute trades at current prices

  Scenario: Fetch current price from Binance
    Given Binance API is accessible
    When Rust requests ticker price for BTCUSDT
    Then Binance returns current price
    And Rust caches price for 1 second
    And price is used for trading decisions
```

**Code Location:**
- Rust: `/Users/dungngo97/Documents/bot-core/rust-core-engine/src/binance/client.rs`
- Test: `/Users/dungngo97/Documents/bot-core/rust-core-engine/tests/test_binance_client.rs`

---

### TC-INT-015: Execute Order on Binance

**Priority:** Critical
**Test Type:** Integration
**Related FR:** FR-INT-003

**Test Scenario (Gherkin):**
```gherkin
  Scenario: Place market order on Binance
    Given user has sufficient balance
    When Rust places market buy order via Binance API
    Then Binance executes order
    And returns order ID and fill price
    And Rust updates local database
```

---

### TC-INT-016: Handle Binance API Errors

**Priority:** High
**Test Type:** Integration
**Related FR:** FR-INT-003

**Test Scenario (Gherkin):**
```gherkin
  Scenario Outline: Handle Binance error codes
    When Binance API returns error <code>
    Then Rust should handle with <action>

    Examples:
      | code  | action                                  |
      | -1021 | Sync timestamp and retry                |
      | -2010 | Reject order (insufficient balance)     |
      | 429   | Back off and retry after delay          |
```

---

### TC-INT-017: Binance Signature Validation

**Priority:** Critical
**Test Type:** Unit
**Related FR:** FR-INT-003

**Test Scenario (Gherkin):**
```gherkin
  Scenario: Generate valid Binance API signature
    Given API key and secret
    When Rust constructs signed request
    Then signature should be HMAC-SHA256
    And Binance should accept request
```

---

### TC-INT-018: Testnet vs Production Toggle

**Priority:** Critical
**Test Type:** Integration
**Related FR:** FR-INT-003

**Test Scenario (Gherkin):**
```gherkin
  Scenario: Use Binance testnet by default
    Given BINANCE_TESTNET=true
    When Rust initializes Binance client
    Then client should connect to testnet.binance.vision
    And no real funds at risk
```

---

## Rust ↔ MongoDB Integration

### TC-INT-019: Save Trade to MongoDB

**Priority:** Critical
**Test Type:** Integration
**Related FR:** FR-INT-004

**Test Scenario (Gherkin):**
```gherkin
Feature: Rust to MongoDB Integration
  As the Rust trading engine
  I want to persist data to MongoDB
  So that trade history is not lost

  Scenario: Save trade execution to database
    Given trade was executed successfully
    When Rust saves trade to MongoDB
    Then trade document should be inserted
    And trade ID should be returned
    And document should be retrievable
```

**Code Location:**
- Rust: `/Users/dungngo97/Documents/bot-core/rust-core-engine/src/storage/mongodb.rs`
- Test: `/Users/dungngo97/Documents/bot-core/rust-core-engine/tests/test_storage.rs`

---

### TC-INT-020: Query Trades from MongoDB

**Priority:** High
**Test Type:** Integration
**Related FR:** FR-INT-004

**Test Scenario (Gherkin):**
```gherkin
  Scenario: Query user's trade history
    Given user has 50 trades in database
    When Rust queries trades with filter: user_id, date_range
    Then matching trades should be returned
    And sorted by timestamp descending
```

---

### TC-INT-021: Update Position in MongoDB

**Priority:** Critical
**Test Type:** Integration
**Related FR:** FR-INT-004

**Test Scenario (Gherkin):**
```gherkin
  Scenario: Update position PnL in database
    Given open position exists in database
    When current price changes
    Then Rust calculates new unrealized PnL
    And updates position document
    And updates timestamp
```

---

### TC-INT-022: MongoDB Connection Failure

**Priority:** High
**Test Type:** Integration
**Related FR:** FR-INT-004

**Test Scenario (Gherkin):**
```gherkin
  Scenario: Handle MongoDB connection failure
    Given MongoDB is unavailable
    When Rust attempts database operation
    Then operation should fail gracefully
    And error should be logged
    And trading should pause until connection restored
```

---

### TC-INT-023: MongoDB Transaction Support

**Priority:** High
**Test Type:** Integration
**Related FR:** FR-INT-004

**Test Scenario (Gherkin):**
```gherkin
  Scenario: Use MongoDB transactions for atomic operations
    Given trade execution updates multiple collections
    When Rust starts MongoDB transaction
    Then all updates should succeed or all rollback
    And data consistency should be maintained
```

---

## WebSocket Real-Time Updates

### TC-INT-024: WebSocket Connection Establishment

**Priority:** Critical
**Test Type:** Integration
**Related FR:** FR-INT-005

**Test Scenario (Gherkin):**
```gherkin
Feature: WebSocket Real-Time Updates
  As the frontend
  I want to receive real-time updates via WebSocket
  So that users see live data

  Scenario: Establish WebSocket connection
    Given frontend loads trading page
    When frontend connects to ws://localhost:8080/ws
    Then WebSocket connection should be established
    And Rust should send welcome message
```

**Code Location:**
- Rust: `/Users/dungngo97/Documents/bot-core/rust-core-engine/src/websocket/server.rs`
- Test: `/Users/dungngo97/Documents/bot-core/rust-core-engine/tests/test_websocket.rs`

---

### TC-INT-025: Subscribe to Market Data Stream

**Priority:** Critical
**Test Type:** Integration
**Related FR:** FR-INT-005

**Test Scenario (Gherkin):**
```gherkin
  Scenario: Subscribe to BTC/USDT price updates
    Given WebSocket is connected
    When frontend sends subscribe message: {"type": "subscribe", "symbol": "BTCUSDT"}
    Then Rust subscribes to Binance WebSocket
    And frontend receives price updates every second
```

---

### TC-INT-026: Broadcast Trade Executions

**Priority:** High
**Test Type:** Integration
**Related FR:** FR-INT-005

**Test Scenario (Gherkin):**
```gherkin
  Scenario: Broadcast trade to all connected clients
    Given multiple users are connected via WebSocket
    When trade is executed
    Then Rust broadcasts trade event to all clients
    And each frontend updates their UI
```

---

### TC-INT-027: WebSocket Reconnection

**Priority:** High
**Test Type:** Integration
**Related FR:** FR-INT-005

**Test Scenario (Gherkin):**
```gherkin
  Scenario: Automatically reconnect on connection loss
    Given WebSocket connection drops
    When frontend detects disconnect
    Then frontend should attempt reconnection
    With exponential backoff
    And restore subscriptions after reconnect
```

---

### TC-INT-028: WebSocket Authentication

**Priority:** Critical
**Test Type:** Integration
**Related FR:** FR-INT-005

**Test Scenario (Gherkin):**
```gherkin
  Scenario: Authenticate WebSocket connection
    Given user has JWT token
    When WebSocket connects with token in query param
    Then Rust validates JWT
    And allows connection if valid
    And sends user-specific data
```

---

### TC-INT-029: WebSocket Message Rate Limiting

**Priority:** Medium
**Test Type:** Integration
**Related FR:** FR-INT-005

**Test Scenario (Gherkin):**
```gherkin
  Scenario: Throttle WebSocket messages
    Given high frequency price updates
    When updates exceed 10 messages/second
    Then Rust should throttle to reasonable rate
    And avoid overwhelming client
```

---

## End-to-End Workflows

### TC-INT-030: Complete Trading Workflow

**Priority:** Critical
**Test Type:** End-to-End
**Related FR:** FR-INT-006

**Test Scenario (Gherkin):**
```gherkin
Feature: End-to-End Trading Workflows
  As a user
  I want to complete full trading workflows
  So that I can buy and sell assets

  Scenario: End-to-end trade execution
    Given user logs in via frontend
    When user views market data (WebSocket updates)
    And requests AI analysis (Rust → Python)
    And AI returns "Buy" signal
    And user places market buy order
    Then Rust executes on Binance (Rust → Binance)
    And saves trade to MongoDB (Rust → MongoDB)
    And broadcasts update via WebSocket
    And frontend updates portfolio display
```

**Code Location:**
- Test: `/Users/dungngo97/Documents/bot-core/tests/e2e-cross-service/test_full_system.py`

---

### TC-INT-031: User Registration to First Trade

**Priority:** High
**Test Type:** End-to-End
**Related FR:** FR-INT-006

**Test Scenario (Gherkin):**
```gherkin
  Scenario: New user complete journey
    Given new user visits platform
    When user registers account
    And verifies email
    And logs in
    And enables paper trading
    And views AI signals
    And places first paper trade
    Then entire workflow should complete successfully
    And user sees trade in history
```

---

### TC-INT-032: Automated Trading Workflow

**Priority:** High
**Test Type:** End-to-End
**Related FR:** FR-INT-006

**Test Scenario (Gherkin):**
```gherkin
  Scenario: Automated trading based on AI signals
    Given bot is configured with auto-trade enabled
    When Rust receives market data
    And requests AI analysis every 5 minutes
    And AI returns "Buy" signal with confidence > 0.75
    Then Rust automatically places trade
    And sets stop-loss
    And monitors position
    And closes position when target reached
```

---

### TC-INT-033: Portfolio Rebalancing Workflow

**Priority:** Medium
**Test Type:** End-to-End
**Related FR:** FR-INT-006

**Test Scenario (Gherkin):**
```gherkin
  Scenario: Rebalance portfolio to target allocation
    Given user has 70% BTC, 30% USDT
    And target is 50% BTC, 50% USDT
    When user clicks "Rebalance"
    Then Rust calculates required trades
    And executes sell orders for BTC
    And updates portfolio
    And achieves target allocation
```

---

### TC-INT-034: Stop-Loss Trigger Workflow

**Priority:** Critical
**Test Type:** End-to-End
**Related FR:** FR-INT-006

**Test Scenario (Gherkin):**
```gherkin
  Scenario: Stop-loss triggers and closes position
    Given user has BTC position with stop-loss at 49000
    When price drops to 49000 (via WebSocket)
    Then Rust detects stop-loss trigger
    And executes market sell immediately
    And closes position
    And notifies user via WebSocket
```

---

### TC-INT-035: Multiple Concurrent Users

**Priority:** High
**Test Type:** End-to-End
**Related FR:** FR-INT-006

**Test Scenario (Gherkin):**
```gherkin
  Scenario: Handle multiple users trading simultaneously
    Given 10 users are logged in
    When all users place trades at same time
    Then all trades should execute correctly
    And no data corruption should occur
    And each user sees only their own data
```

---

### TC-INT-036: Data Consistency Across Services

**Priority:** Critical
**Test Type:** End-to-End
**Related FR:** FR-INT-006

**Test Scenario (Gherkin):**
```gherkin
  Scenario: Ensure data consistency across all services
    When trade is executed
    Then Rust database should have trade record
    And Binance should show filled order
    And Frontend should display updated balance
    And All balances should match exactly
```

---

### TC-INT-037: Service Recovery After Crash

**Priority:** High
**Test Type:** End-to-End
**Related FR:** FR-INT-006

**Test Scenario (Gherkin):**
```gherkin
  Scenario: Recover gracefully from service crash
    Given Rust service crashes during trade
    When Rust restarts
    Then positions should be restored from database
    And pending orders should be reconciled with Binance
    And trading should resume normally
```

---

## Cross-Service Authentication

### TC-INT-038: JWT Validation Across Services

**Priority:** High
**Test Type:** Integration
**Related FR:** FR-INT-007

**Test Scenario (Gherkin):**
```gherkin
Feature: Cross-Service Authentication
  As the system
  I want consistent authentication across services
  So that users are properly authorized

  Scenario: JWT token validated by all services
    Given user logs in and receives JWT
    When user makes request to Frontend → Rust → Python
    Then all services should validate JWT
    And extract same user_id from token
```

---

### TC-INT-039: Token Expiration Handling

**Priority:** High
**Test Type:** Integration
**Related FR:** FR-INT-007

**Test Scenario (Gherkin):**
```gherkin
  Scenario: Handle expired JWT across services
    Given user's JWT expires
    When user attempts operation
    Then all services should reject expired token
    And frontend should redirect to login
```

---

### TC-INT-040: Service-to-Service Authentication

**Priority:** Medium
**Test Type:** Integration
**Related FR:** FR-INT-007

**Test Scenario (Gherkin):**
```gherkin
  Scenario: Rust authenticates to Python service
    Given Rust needs to call Python API
    When Rust includes service token in request
    Then Python validates service token
    And allows request
```

---

### TC-INT-041: Admin-Only Cross-Service Operations

**Priority:** Medium
**Test Type:** Integration
**Related FR:** FR-INT-007

**Test Scenario (Gherkin):**
```gherkin
  Scenario: Admin operations require admin JWT
    Given user has regular JWT (is_admin=false)
    When user attempts admin operation
    Then all services should check is_admin claim
    And reject non-admin requests with 403
```

---

## Error Propagation

### TC-INT-042: Error Propagation Frontend ← Rust ← Python

**Priority:** High
**Test Type:** Integration
**Related FR:** FR-INT-008

**Test Scenario (Gherkin):**
```gherkin
Feature: Error Propagation
  As the system
  I want errors to propagate correctly
  So that users see meaningful error messages

  Scenario: Python error propagates to frontend
    Given Python service returns 500 error
    When Rust receives error
    Then Rust should log error
    And return 503 Service Unavailable to frontend
    And frontend should display "AI service temporarily unavailable"
```

---

### TC-INT-043: Binance Error Propagation

**Priority:** High
**Test Type:** Integration
**Related FR:** FR-INT-008

**Test Scenario (Gherkin):**
```gherkin
  Scenario: Binance error shown to user
    Given Binance rejects order due to insufficient balance
    When Rust receives Binance error -2010
    Then Rust translates to user-friendly message
    And returns to frontend
    And frontend shows "Insufficient balance for this trade"
```

---

### TC-INT-044: Database Error Handling

**Priority:** High
**Test Type:** Integration
**Related FR:** FR-INT-008

**Test Scenario (Gherkin):**
```gherkin
  Scenario: Handle database write failure
    Given MongoDB is unavailable
    When Rust attempts to save trade
    Then Rust should retry 3 times
    If all fail, return 500 to frontend
    And log error for investigation
```

---

### TC-INT-045: Partial Failure Handling

**Priority:** High
**Test Type:** Integration
**Related FR:** FR-INT-008

**Test Scenario (Gherkin):**
```gherkin
  Scenario: Handle partial failure gracefully
    Given trade executes on Binance successfully
    But database save fails
    Then Rust should queue trade for retry
    And return success to user (trade did execute)
    And background job retries database save
```

---

## Acceptance Criteria

Integration module is considered complete when:

- [ ] All 45 integration test cases pass
- [ ] All services communicate successfully
- [ ] Error handling works across service boundaries
- [ ] WebSocket updates work in real-time
- [ ] End-to-end workflows complete successfully
- [ ] No data inconsistencies between services
- [ ] Authentication works across all services
- [ ] System recovers from service failures
- [ ] Performance: E2E trade < 2000ms
- [ ] No critical integration bugs

---

**Document Control:**
- **Created by**: Integration Team
- **Reviewed by**: Architecture Team
- **Approved by**: Product Owner
- **Next Review Date**: 2025-11-11

---

*End of Integration Test Cases Document*
