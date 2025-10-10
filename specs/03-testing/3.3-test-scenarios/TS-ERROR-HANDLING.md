# Test Scenarios - Error Handling

**Document ID:** TS-ERROR-001
**Version:** 1.0
**Last Updated:** 2025-10-11
**Status:** Active

---

## Overview

This document contains error handling test scenarios that verify the system properly handles various error conditions, failures, and exceptions.

---

## Scenario Index

| ID | Scenario Name | Priority | Status |
|----|---------------|----------|--------|
| TS-ERROR-001 | Invalid JWT Token | Critical | ✅ |
| TS-ERROR-002 | Binance API Failure | Critical | ✅ |
| TS-ERROR-003 | Database Write Failure | Critical | ✅ |
| TS-ERROR-004 | WebSocket Disconnection | High | ✅ |
| TS-ERROR-005 | OpenAI API Rate Limit | High | ✅ |
| TS-ERROR-006 | Insufficient Balance Error | Critical | ✅ |
| TS-ERROR-007 | Risk Limit Exceeded | Critical | ✅ |
| TS-ERROR-008 | Invalid Order Parameters | High | ✅ |
| TS-ERROR-009 | Service Timeout Error | High | ✅ |
| TS-ERROR-010 | Malformed JSON Response | High | ✅ |
| TS-ERROR-011 | Concurrent Modification Error | Medium | ✅ |
| TS-ERROR-012 | Cache Failure Fallback | Medium | ✅ |
| TS-ERROR-013 | Authentication Failure Chain | High | ✅ |
| TS-ERROR-014 | Orphaned Position Recovery | High | ✅ |
| TS-ERROR-015 | Partial System Failure | Critical | ✅ |

---

## TS-ERROR-001: Invalid JWT Token

### Gherkin Scenario

```gherkin
Feature: Invalid JWT Token Handling
  As the authentication system
  I want to reject invalid tokens
  So that unauthorized access is prevented

  Scenario: Expired JWT token
    Given user logged in 25 hours ago
    And JWT token expired after 24 hours
    When user attempts to access /api/account
    Then system should reject with 401 Unauthorized
    And error message: "Token expired. Please log in again."
    And frontend should redirect to login page
    And localStorage should clear token

  Scenario: Malformed JWT token
    Given user modifies JWT token manually
    When user makes API request with invalid token
    Then Rust should detect invalid signature
    And return 401 Unauthorized
    And error: "Invalid authentication token"
    And log security event

  Scenario: Token signed with wrong secret
    Given attacker creates token with different secret
    When attacker attempts API access
    Then signature verification should fail
    And access should be denied
    And security alert should be raised
```

---

## TS-ERROR-002: Binance API Failure

### Gherkin Scenario

```gherkin
Feature: Binance API Error Handling
  As the trading system
  I want to handle Binance errors gracefully
  So that trading continues when possible

  Scenario: Binance returns 500 Internal Server Error
    Given user places order
    When Binance API returns 500 error
    Then Rust should retry 3 times with exponential backoff
    And delays: 1s, 2s, 4s
    If all retries fail, return error to user:
      "Trading temporarily unavailable. Please try again."
    And log error for investigation

  Scenario: Binance API timeout
    Given Binance API is slow
    When request takes >30 seconds
    Then Rust should cancel request
    And return timeout error to user
    And suggest checking network connection

  Scenario: Invalid API signature (error -1022)
    Given API secret is incorrect in config
    When system attempts Binance request
    Then Binance rejects with error -1022
    And system should alert admin
    And disable trading until fixed
    And show maintenance message to users
```

---

## TS-ERROR-003: Database Write Failure

### Gherkin Scenario

```gherkin
Feature: Database Failure Handling
  As the persistence layer
  I want to handle database failures
  So that critical data isn't lost

  Scenario: MongoDB connection lost during trade save
    Given trade executed successfully on Binance
    When Rust attempts to save to MongoDB
    And connection is lost
    Then Rust should:
      1. Acknowledge trade execution to user (it did happen)
      2. Queue trade data for retry
      3. Background job retries save every 30 seconds
      4. Log failure for monitoring
      5. Alert if retry fails 10 times

  Scenario: Database disk full
    Given MongoDB disk is full
    When write operation fails
    Then system should:
      - Return error to user
      - Alert DevOps team immediately
      - Switch to read-only mode
      - Prevent new trades until resolved
```

---

## TS-ERROR-004: WebSocket Disconnection

### Gherkin Scenario

```gherkin
Feature: WebSocket Disconnection Handling
  As the real-time update system
  I want to handle WebSocket drops
  So that users maintain connectivity

  Scenario: Client-side WebSocket disconnect
    Given WebSocket is connected
    When connection drops (network issue)
    Then client should detect disconnect
    And automatically attempt reconnection
    With exponential backoff: 1s, 2s, 4s, 8s, 16s, max 30s
    And restore subscriptions after reconnect
    And show connectivity status to user

  Scenario: Server-side WebSocket crash
    Given Rust WebSocket server crashes
    When clients try to connect
    Then clients should receive connection refused
    And retry with backoff
    When server restarts
    Then clients should successfully reconnect
    And resume receiving updates

  Scenario: WebSocket authentication fails after reconnect
    Given WebSocket reconnects after disconnect
    But JWT token expired during disconnect
    Then reconnection should fail authentication
    And client should prompt user to log in again
```

---

## TS-ERROR-005: OpenAI API Rate Limit

### Gherkin Scenario

```gherkin
Feature: OpenAI Rate Limit Handling
  As the AI service
  I want to handle OpenAI rate limits
  So that analysis continues despite limits

  Scenario: Hit OpenAI rate limit (429 error)
    Given I make 61 GPT-4 requests in 1 minute
    When request #61 returns 429 Too Many Requests
    Then Python should:
      1. Check Redis cache for recent analysis (<5 min old)
      2. If cache hit, return cached result
      3. If cache miss, fall back to technical analysis only
      4. Log rate limit event
      5. Implement backoff before next GPT-4 call

  Scenario: OpenAI quota exceeded
    Given monthly OpenAI quota is exhausted
    When AI analysis is requested
    Then system should:
      - Permanently fall back to technical analysis
      - Notify admin of quota exhaustion
      - Continue trading with degraded AI
      - Update UI to show "AI analysis unavailable"
```

---

## TS-ERROR-006: Insufficient Balance Error

### Gherkin Scenario

```gherkin
Feature: Insufficient Balance Handling
  As the trading system
  I want to prevent overdraft
  So that users can't spend more than they have

  Scenario: Attempt trade with insufficient funds
    Given my balance is 1000 USDT
    When I attempt to buy BTC worth 5000 USDT
    Then validation should fail before API call
    And error: "Insufficient balance: 1000 USDT available, 5000 USDT required"
    And frontend should highlight balance
    And suggest deposit or reduce order size

  Scenario: Balance consumed by concurrent trade
    Given my balance is 1000 USDT
    And I submit trade A for 600 USDT
    And I submit trade B for 600 USDT (before A completes)
    Then system should:
      - Lock balance for trade A
      - Detect insufficient balance for trade B
      - Reject trade B with "Insufficient available balance"
      - Execute only trade A
```

---

## TS-ERROR-007: Risk Limit Exceeded

### Gherkin Scenario

```gherkin
Feature: Risk Limit Enforcement
  As the risk management system
  I want to block trades exceeding limits
  So that users don't take excessive risk

  Scenario: Trade exceeds max risk per trade
    Given max risk per trade is 2% (200 USDT on 10000 balance)
    When I attempt trade risking 500 USDT (5%)
    Then system should reject with:
      "Risk limit exceeded: 500 USDT > 200 USDT maximum"
    And suggest reducing position size

  Scenario: Daily loss limit reached
    Given daily loss limit is 5% (500 USDT)
    And I already lost 500 USDT today
    When I attempt another trade
    Then system should block with:
      "Daily loss limit reached. Trading disabled until tomorrow."
    And prevent all trades for 24 hours
```

---

## TS-ERROR-008: Invalid Order Parameters

### Gherkin Scenario

```gherkin
Feature: Order Validation Errors
  As the trading system
  I want to validate orders
  So that invalid orders are rejected

  Scenario Outline: Invalid order parameters
    When user submits order with <parameter> = <value>
    Then system should reject with error "<error>"

    Examples:
      | parameter | value      | error                                |
      | quantity  | 0          | Quantity must be greater than 0      |
      | quantity  | -0.5       | Quantity cannot be negative          |
      | price     | 0          | Price must be greater than 0         |
      | symbol    | "INVALID"  | Invalid trading pair                 |
      | side      | "MIDDLE"   | Side must be BUY or SELL             |
```

---

## TS-ERROR-009: Service Timeout Error

### Gherkin Scenario

```gherkin
Feature: Service Timeout Handling
  As the microservices system
  I want to handle timeouts gracefully
  So that slow services don't block everything

  Scenario: Python AI service timeout
    Given Python service is processing heavy ML model
    And takes >30 seconds to respond
    When Rust times out waiting
    Then Rust should:
      - Cancel request
      - Return fallback response (technical analysis)
      - Log timeout for monitoring
      - Continue trading without blocking

  Scenario: Cascading timeout prevention
    Given Frontend → Rust → Python chain
    When Python times out
    Then Rust should timeout before Frontend
    And Frontend should receive error within 35 seconds total
    And no request should hang indefinitely
```

---

## TS-ERROR-010: Malformed JSON Response

### Gherkin Scenario

```gherkin
Feature: Invalid JSON Handling
  As the API consumer
  I want to handle malformed responses
  So that parsing errors don't crash system

  Scenario: GPT-4 returns non-JSON text
    Given GPT-4 returns: "I think the market will go up"
    Instead of: {"signal": "Long", "confidence": 0.75}
    When Python parses response
    Then JSON parsing should fail
    And system should catch exception
    And fall back to technical analysis
    And log malformed response

  Scenario: Binance returns invalid JSON
    Given network corruption causes invalid JSON
    When Rust parses Binance response
    Then parsing should fail gracefully
    And retry request
    And don't crash application
```

---

## TS-ERROR-011: Concurrent Modification Error

### Gherkin Scenario

```gherkin
Feature: Concurrent Modification Handling
  As the database layer
  I want to handle concurrent updates
  So that data remains consistent

  Scenario: Two processes update same position
    Given position exists with PnL = 100
    When Process A updates PnL to 150
    And Process B updates PnL to 120 (at same time)
    Then database should use optimistic locking
    And one update should succeed
    And other should retry with latest data
    And final PnL should be correct
```

---

## TS-ERROR-012: Cache Failure Fallback

### Gherkin Scenario

```gherkin
Feature: Cache Failure Handling
  As the caching layer
  I want to handle Redis failures
  So that system works without cache

  Scenario: Redis is unavailable
    Given Redis server is down
    When system attempts to cache AI analysis
    Then cache write should fail silently
    And system should log error
    And continue without caching
    And analysis should still complete

  Scenario: Cache read failure
    Given Redis connection is unstable
    When system attempts cache read
    And read fails
    Then system should treat as cache miss
    And fetch fresh data
    And continue normally
```

---

## TS-ERROR-013: Authentication Failure Chain

### Gherkin Scenario

```gherkin
Feature: Authentication Failure Propagation
  As the security system
  I want auth failures to propagate correctly
  So that users see helpful messages

  Scenario: Frontend → Rust → Python auth chain failure
    Given user's JWT is invalid
    When Frontend calls Rust /api/analyze
    And Rust calls Python /api/ai/analyze
    Then Rust should reject with 401
    And Frontend should catch 401
    And redirect user to login
    And show message: "Session expired. Please log in again."
    And not call Python (auth failed at Rust)
```

---

## TS-ERROR-014: Orphaned Position Recovery

### Gherkin Scenario

```gherkin
Feature: Orphaned Position Recovery
  As the trading system
  I want to recover from incomplete trades
  So that positions don't get orphaned

  Scenario: Trade succeeds on Binance but DB save fails
    Given trade executes on Binance: order ID 12345
    But MongoDB save fails
    And system crashes before retry
    When system restarts
    Then recovery process should:
      1. Query Binance for recent orders
      2. Find order 12345 (FILLED)
      3. Detect missing in local database
      4. Reconcile and save to database
      5. Update user's positions
      6. Ensure consistency

  Scenario: Position exists in DB but not on Binance
    Given position in database shows open position
    But Binance shows no such position
    Then system should:
      - Detect discrepancy
      - Mark position as "reconciliation_needed"
      - Alert admin for manual review
      - Don't auto-close (might be data issue)
```

---

## TS-ERROR-015: Partial System Failure

### Gherkin Scenario

```gherkin
Feature: Graceful Degradation
  As the system
  I want to operate with partial failures
  So that entire platform doesn't go down

  Scenario: Python AI service is down
    Given Python service crashes
    When users request AI analysis
    Then Rust should:
      - Detect Python unavailable
      - Fall back to local technical indicators
      - Return analysis without ML models
      - Show warning: "AI analysis unavailable - using technical analysis"
      - Allow trading to continue

  Scenario: MongoDB is down but Binance works
    Given MongoDB is unreachable
    When users attempt to trade
    Then system should:
      - Block trading (can't record trades)
      - Allow viewing (read-only mode)
      - Show: "Trading temporarily disabled - maintenance in progress"
      - Queue user requests for when database returns

  Scenario: All external services down
    Given Binance, MongoDB, Redis all down
    When user accesses platform
    Then system should:
      - Show maintenance page
      - Display "We'll be back soon" message
      - Log all failures
      - Alert DevOps team
      - Health check endpoints return 503
```

---

## Error Handling Checklist

For each error scenario:

- [ ] Error is caught and doesn't crash system
- [ ] User-friendly error message displayed
- [ ] Technical error logged for debugging
- [ ] Appropriate HTTP status code returned
- [ ] System falls back gracefully
- [ ] Critical errors alert DevOps
- [ ] Users can recover from error
- [ ] Error tracked in monitoring system

---

**Document Control:**
- **Created by**: QA Team
- **Reviewed by**: Engineering Team
- **Approved by**: Product Owner
- **Next Review Date**: 2025-11-11

---

*End of Error Handling Scenarios Document*
