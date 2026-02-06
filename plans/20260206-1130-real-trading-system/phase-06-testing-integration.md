# Phase 06: Testing & Integration

**Status**: Pending | **Estimated Time**: 1 day

## Context

All features implemented. Need comprehensive testing: unit tests, integration tests with testnet, E2E tests for frontend flows.

## Overview

Complete testing coverage: unit tests for all new code, integration tests with Binance testnet, E2E tests for critical user flows.

## Key Insights (From Research)

- Use `proptest` for property-based testing (order validation)
- Integration tests should use testnet directly
- E2E tests can mock WebSocket for deterministic behavior
- Existing test patterns in `rust-core-engine/tests/`

## Requirements

1. Unit tests for all new Rust code (>80% coverage)
2. Unit tests for React components and hooks
3. Integration tests with Binance testnet
4. E2E tests for order placement flow
5. Performance tests for WebSocket handling
6. Security tests for confirmation system

## Architecture

```
Tests
    ├── Rust Unit Tests
    │   ├── test_spot_order_placement
    │   ├── test_futures_order_placement
    │   ├── test_oco_order_handling
    │   ├── test_risk_validation
    │   └── test_order_state_machine
    ├── Rust Integration Tests
    │   ├── test_real_order_on_testnet
    │   ├── test_position_lifecycle
    │   └── test_user_data_stream
    ├── React Unit Tests
    │   ├── OrderForm.test.tsx
    │   ├── ConfirmationDialog.test.tsx
    │   └── useRealTrading.test.ts
    └── E2E Tests
        ├── place_market_order.spec.ts
        ├── place_limit_order.spec.ts
        └── cancel_order.spec.ts
```

## Related Files

| File | Action | Description |
|------|--------|-------------|
| `rust-core-engine/tests/test_real_trading.rs` | Create | Rust integration tests |
| `rust-core-engine/src/real_trading/*.rs` | Modify | Add unit tests (in-file) |
| `nextjs-ui-dashboard/src/components/trading/*.test.tsx` | Create | Component tests |
| `nextjs-ui-dashboard/src/hooks/useRealTrading.test.ts` | Create | Hook tests |
| `nextjs-ui-dashboard/e2e/real-trading.spec.ts` | Create | E2E tests |

## Implementation Steps

### 1. Rust Unit Tests
```rust
// Add to engine.rs
#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_execute_spot_buy_validation() {
        // Test that invalid params are rejected
    }

    #[tokio::test]
    async fn test_risk_validation_blocks_overlimit() {
        // Test that risk manager blocks orders exceeding limits
    }

    #[tokio::test]
    async fn test_order_state_transitions() {
        // Test Pending -> New -> Filled transitions
    }
}
```

### 2. Rust Integration Tests (Testnet)
```rust
// tests/test_real_trading.rs
#[tokio::test]
#[ignore] // Run with --ignored for testnet tests
async fn test_place_spot_order_on_testnet() {
    let client = create_testnet_client();
    let result = client.place_spot_order(SpotOrderRequest {
        symbol: "BTCUSDT".to_string(),
        side: OrderSide::Buy,
        order_type: SpotOrderType::Market,
        quantity: Some(0.001),
        ..Default::default()
    }).await;

    assert!(result.is_ok());
}

#[tokio::test]
#[ignore]
async fn test_futures_position_lifecycle() {
    // Open long -> Update SL/TP -> Close -> Verify PnL
}
```

### 3. React Component Tests
```tsx
// OrderForm.test.tsx
describe('OrderForm', () => {
  it('validates quantity is positive', () => {
    render(<OrderForm onSubmit={jest.fn()} isLoading={false} mode="spot" />);
    const input = screen.getByLabelText(/quantity/i);
    fireEvent.change(input, { target: { value: '-1' } });
    expect(screen.getByText(/must be positive/i)).toBeInTheDocument();
  });

  it('shows price input only for limit orders', () => {
    render(<OrderForm onSubmit={jest.fn()} isLoading={false} mode="spot" />);
    const typeSelect = screen.getByLabelText(/order type/i);
    fireEvent.change(typeSelect, { target: { value: 'LIMIT' } });
    expect(screen.getByLabelText(/price/i)).toBeInTheDocument();
  });
});
```

### 4. Hook Tests
```typescript
// useRealTrading.test.ts
describe('useRealTrading', () => {
  it('returns confirmation token on first order submit', async () => {
    const { result } = renderHook(() => useRealTrading());

    server.use(
      rest.post('/api/real-trading/orders', (req, res, ctx) => {
        return res(ctx.json({
          success: true,
          data: { confirmation_required: true, token: 'abc123' }
        }));
      })
    );

    await act(async () => {
      await result.current.placeOrder({ symbol: 'BTCUSDT', side: 'BUY', ... });
    });

    expect(result.current.confirmationData).toBeDefined();
  });
});
```

### 5. E2E Tests
```typescript
// e2e/real-trading.spec.ts
test.describe('Real Trading', () => {
  test('place market order with confirmation', async ({ page }) => {
    await page.goto('/real-trading');

    // Fill order form
    await page.fill('[data-testid="symbol-input"]', 'BTCUSDT');
    await page.click('[data-testid="buy-button"]');
    await page.fill('[data-testid="quantity-input"]', '0.001');
    await page.click('[data-testid="submit-order"]');

    // Verify confirmation dialog
    await expect(page.locator('[data-testid="confirmation-dialog"]')).toBeVisible();
    await expect(page.locator('[data-testid="order-summary"]')).toContainText('BTCUSDT');

    // Confirm order
    await page.click('[data-testid="confirm-button"]');

    // Verify success
    await expect(page.locator('[role="alert"]')).toContainText('Order placed');
  });
});
```

## Todo

### Rust Tests
- [ ] Unit tests for `place_spot_order`
- [ ] Unit tests for `place_futures_order`
- [ ] Unit tests for OCO order handling
- [ ] Unit tests for order state machine
- [ ] Unit tests for risk validation
- [ ] Integration test: spot order on testnet
- [ ] Integration test: futures order on testnet
- [ ] Integration test: cancel order on testnet
- [ ] Integration test: position lifecycle

### Frontend Tests
- [ ] OrderForm component tests
- [ ] ConfirmationDialog component tests
- [ ] OrdersTable component tests
- [ ] useRealTrading hook tests
- [ ] E2E: place market order
- [ ] E2E: place limit order
- [ ] E2E: cancel order
- [ ] E2E: modify SL/TP
- [ ] E2E: view order history

### Other
- [ ] Performance test: 100 concurrent WebSocket updates
- [ ] Security test: token expiration
- [ ] Security test: replay prevention
- [ ] Run all tests in CI
- [ ] Achieve >80% coverage

## Success Criteria

- [ ] All unit tests passing
- [ ] All integration tests passing (testnet)
- [ ] All E2E tests passing
- [ ] Code coverage >80%
- [ ] No critical security issues
- [ ] Performance: <100ms for order submission

## Risk Assessment

| Risk | Probability | Impact | Mitigation |
|------|-------------|--------|------------|
| Testnet unavailable | Low | Medium | Mock for CI, manual for testnet |
| Flaky E2E tests | Medium | Low | Add retries, increase timeouts |
| Coverage gaps | Medium | Low | Add tests iteratively |

## Security Considerations

- Integration tests use testnet only
- No real API keys in test files
- E2E tests mock sensitive operations
- CI secrets are encrypted

## Next Steps

After completion: System is ready for testnet release. Document deployment steps.
