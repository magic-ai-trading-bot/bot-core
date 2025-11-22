# Mutation Testing: Before & After Examples

**Real-world examples showing weak vs strong tests**

---

## Example 1: RSI Calculation (Rust)

### ❌ BEFORE: Weak Test (Lets Mutants Survive)

```rust
#[test]
fn test_calculate_rsi() {
    let prices = vec![100.0, 101.0, 99.0, 102.0, 98.0];
    let result = calculate_rsi(&prices, 3);

    // WEAK ASSERTION: Only checks success
    assert!(result.is_ok());
}
```

**Mutations that SURVIVE this weak test:**
```rust
// MUTATION 1: Return empty vector
fn calculate_rsi() -> Result<Vec<f64>, String> {
    return Ok(vec![]);  // ❌ Test still passes!
}

// MUTATION 2: Return constant value
fn calculate_rsi() -> Result<Vec<f64>, String> {
    return Ok(vec![50.0]);  // ❌ Test still passes!
}

// MUTATION 3: Flip arithmetic operator
let rsi = 100.0 + (100.0 / (1.0 + rs));  // Was -
// ❌ Test still passes! (wrong value)

// MUTATION 4: Change comparison
if avg_gain < avg_loss {  // Was >
// ❌ Test still passes! (wrong logic)
```

**Mutation Score: ~15%** (85% of mutants survive)

---

### ✅ AFTER: Strong Test (Catches Mutants)

```rust
#[test]
fn test_calculate_rsi_exact_values() {
    // Known test data with verified RSI values
    let prices = vec![
        44.0, 44.25, 44.37, 44.12, 44.0, 43.87,
        43.75, 43.87, 44.0, 44.12, 44.25, 44.37,
        44.5, 44.62, 44.75
    ];
    let period = 14;

    let result = calculate_rsi(&prices, period).unwrap();

    // STRONG ASSERTION 1: Exact value check
    // Catches: arithmetic mutations, algorithm errors
    let last_rsi = result.last().unwrap();
    assert!(
        (last_rsi - 70.46).abs() < 0.1,
        "RSI should be ~70.46, got {}",
        last_rsi
    );

    // STRONG ASSERTION 2: Range check
    // Catches: constant return mutations
    assert!(
        *last_rsi >= 0.0 && *last_rsi <= 100.0,
        "RSI must be 0-100, got {}",
        last_rsi
    );

    // STRONG ASSERTION 3: Length check
    // Catches: empty return mutations
    assert_eq!(
        result.len(),
        prices.len() - period + 1,
        "RSI should have {} values",
        prices.len() - period + 1
    );

    // STRONG ASSERTION 4: All values in range
    // Catches: individual calculation errors
    for (i, rsi) in result.iter().enumerate() {
        assert!(
            *rsi >= 0.0 && *rsi <= 100.0,
            "RSI[{}] = {} is out of range",
            i,
            rsi
        );
    }
}

#[test]
fn test_rsi_boundary_all_gains() {
    // Edge case: strictly increasing prices
    let prices: Vec<f64> = (0..20).map(|i| 100.0 + i as f64).collect();
    let result = calculate_rsi(&prices, 14).unwrap();
    let rsi = result.last().unwrap();

    // Catches: comparison operator mutations
    assert!(
        *rsi > 95.0,
        "All gains should give RSI > 95, got {}",
        rsi
    );
}

#[test]
fn test_rsi_boundary_all_losses() {
    // Edge case: strictly decreasing prices
    let prices: Vec<f64> = (0..20).map(|i| 100.0 - i as f64).collect();
    let result = calculate_rsi(&prices, 14).unwrap();
    let rsi = result.last().unwrap();

    // Catches: comparison operator mutations
    assert!(
        *rsi < 5.0,
        "All losses should give RSI < 5, got {}",
        rsi
    );
}

#[test]
fn test_rsi_insufficient_data() {
    let prices = vec![100.0, 101.0];  // Only 2 prices
    let result = calculate_rsi(&prices, 14);

    // Catches: error path mutations
    assert!(result.is_err());
    let err = result.unwrap_err();
    assert!(err.contains("Insufficient"));
}

#[test]
fn test_rsi_empty_prices() {
    let prices: Vec<f64> = vec![];
    let result = calculate_rsi(&prices, 14);

    // Catches: edge case mutations
    assert!(result.is_err());
}
```

**Mutations that are NOW CAUGHT:**
```rust
// MUTATION 1: Return empty vector
fn calculate_rsi() -> Result<Vec<f64>, String> {
    return Ok(vec![]);
}
// ✅ CAUGHT by: assert_eq!(result.len(), expected)

// MUTATION 2: Return constant value
fn calculate_rsi() -> Result<Vec<f64>, String> {
    return Ok(vec![50.0]);
}
// ✅ CAUGHT by: assert!((last_rsi - 70.46).abs() < 0.1)

// MUTATION 3: Flip arithmetic operator
let rsi = 100.0 + (100.0 / (1.0 + rs));  // Was -
// ✅ CAUGHT by: exact value assertion

// MUTATION 4: Change comparison
if avg_gain < avg_loss {  // Was >
// ✅ CAUGHT by: boundary tests (all gains/losses)
```

**Mutation Score: ~85%** (85% of mutants caught)

**Improvement: +70 percentage points**

---

## Example 2: SMA Calculation (Python)

### ❌ BEFORE: Weak Test

```python
def test_calculate_sma():
    prices = [100, 102, 101, 103, 104]
    result = calculate_sma(prices, 3)

    # WEAK: Only checks existence
    assert result is not None
    assert len(result) > 0
```

**Mutations that SURVIVE:**
```python
# MUTATION 1: Wrong aggregation function
def calculate_sma(prices, period):
    return prices.rolling(window=period).max()  # Was mean()
# ❌ Test passes (returns values, not None)

# MUTATION 2: Off-by-one error
def calculate_sma(prices, period):
    return prices.rolling(window=period+1).mean()  # Wrong period
# ❌ Test passes (still returns values)

# MUTATION 3: Division error
def calculate_sma(prices, period):
    return sum(prices) * period  # Was division
# ❌ Test passes (returns a number)
```

**Mutation Score: ~20%**

---

### ✅ AFTER: Strong Test

```python
import pytest
import numpy as np

def test_calculate_sma_exact_values():
    """Test SMA with known calculations - catches mutations"""
    prices = np.array([100, 102, 101, 103, 104, 105])
    period = 3

    result = calculate_sma(prices, period)

    # STRONG ASSERTION 1: Exact values
    # Catches: arithmetic mutations, aggregation mutations
    expected = [
        (100 + 102 + 101) / 3,  # 101.0
        (102 + 101 + 103) / 3,  # 102.0
        (101 + 103 + 104) / 3,  # 102.67
        (103 + 104 + 105) / 3,  # 104.0
    ]

    assert len(result) == len(expected), \
        f"Expected {len(expected)} values, got {len(result)}"

    for i, (actual, exp) in enumerate(zip(result, expected)):
        assert abs(actual - exp) < 0.01, \
            f"SMA[{i}] should be {exp:.2f}, got {actual:.2f}"

def test_sma_boundary_period_equals_length():
    """Test boundary: period == data length"""
    prices = np.array([100, 102, 104])
    result = calculate_sma(prices, period=3)

    # Catches: boundary condition mutations
    assert len(result) == 1
    assert abs(result[0] - 102.0) < 0.01

def test_sma_period_greater_than_length():
    """Test edge case: period > data length"""
    prices = np.array([100, 102])

    # Catches: comparison operator mutations (< vs <=)
    with pytest.raises(ValueError, match="Insufficient data"):
        calculate_sma(prices, period=3)

def test_sma_empty_array():
    """Test edge case: empty array"""
    prices = np.array([])

    # Catches: return value mutations
    with pytest.raises(ValueError, match="Empty"):
        calculate_sma(prices, period=14)

def test_sma_all_same_values():
    """Test special case: constant prices"""
    prices = np.array([100.0] * 20)
    result = calculate_sma(prices, period=14)

    # Catches: arithmetic mutations
    for sma in result:
        assert abs(sma - 100.0) < 0.01, \
            f"SMA of constant prices should be 100.0, got {sma}"

@pytest.mark.parametrize("prices,period,expected_last", [
    ([100, 101, 102], 2, 101.5),   # Simple case
    ([50, 50, 50, 50], 3, 50.0),   # Constant
    ([100, 110, 90, 100], 2, 95.0),  # Volatile
])
def test_sma_parametrized(prices, period, expected_last):
    """Parametrized tests catch comparison mutations"""
    result = calculate_sma(np.array(prices), period)
    assert abs(result[-1] - expected_last) < 0.01
```

**Mutations NOW CAUGHT:**
```python
# MUTATION 1: Wrong aggregation
return prices.rolling(window=period).max()
# ✅ CAUGHT by: exact value assertions

# MUTATION 2: Off-by-one
return prices.rolling(window=period+1).mean()
# ✅ CAUGHT by: exact value assertions

# MUTATION 3: Arithmetic error
return sum(prices) * period
# ✅ CAUGHT by: exact value assertions
```

**Mutation Score: ~88%**

**Improvement: +68 percentage points**

---

## Example 3: WebSocket Connection (TypeScript)

### ❌ BEFORE: Weak Test

```typescript
test('WebSocket connects', () => {
  const { result } = renderHook(() => useWebSocket(url));

  // WEAK: Only checks that something was set
  expect(result.current).toBeDefined();
});
```

**Mutations that SURVIVE:**
```typescript
// MUTATION 1: Wrong initial state
const [isConnected, setConnected] = useState(true);  // Was false
// ❌ Test passes (result still defined)

// MUTATION 2: Missing connection logic
const connect = () => {
  // Do nothing
};
// ❌ Test passes (hook still defined)

// MUTATION 3: Wrong reconnect logic
if (shouldReconnect && attempts < maxAttempts) {  // Was >
// ❌ Test passes (no reconnect tested)
```

**Mutation Score: ~10%**

---

### ✅ AFTER: Strong Test

```typescript
import { renderHook, waitFor, act } from '@testing-library/react';
import { useWebSocket } from '../useWebSocket';

describe('useWebSocket Hook', () => {
  let mockWs: MockWebSocket;

  beforeEach(() => {
    mockWs = new MockWebSocket();
    global.WebSocket = jest.fn(() => mockWs) as any;
  });

  test('should initialize with disconnected state', () => {
    const { result } = renderHook(() => useWebSocket(url));

    // STRONG: Exact state check
    // Catches: initial state mutations
    expect(result.current.isConnected).toBe(false);
    expect(result.current.error).toBeNull();
    expect(result.current.lastMessage).toBeNull();
  });

  test('should connect and set state correctly', async () => {
    const { result } = renderHook(() => useWebSocket(url));

    // Initial state
    expect(result.current.isConnected).toBe(false);

    // Simulate connection
    act(() => mockWs.simulateOpen());

    // STRONG: Exact boolean check
    // Catches: boolean negation mutations
    await waitFor(() => {
      expect(result.current.isConnected).toBe(true);
      expect(result.current.error).toBeNull();
    });
  });

  test('should handle disconnection', async () => {
    const { result } = renderHook(() => useWebSocket(url));

    act(() => mockWs.simulateOpen());
    await waitFor(() => expect(result.current.isConnected).toBe(true));

    act(() => mockWs.simulateClose());

    // STRONG: State transition check
    // Catches: state update mutations
    await waitFor(() => {
      expect(result.current.isConnected).toBe(false);
    });
  });

  test('should receive and parse messages correctly', async () => {
    const { result } = renderHook(() => useWebSocket(url));
    act(() => mockWs.simulateOpen());

    const testData = { signal: 'LONG', price: 50000, confidence: 0.85 };
    act(() => mockWs.simulateMessage(testData));

    // STRONG: Exact value checks
    // Catches: object property mutations, string mutations, number mutations
    await waitFor(() => {
      expect(result.current.lastMessage).toEqual(testData);
      expect(result.current.lastMessage.signal).toBe('LONG');
      expect(result.current.lastMessage.price).toBe(50000);
      expect(result.current.lastMessage.confidence).toBe(0.85);
    });
  });

  test('should handle malformed JSON gracefully', async () => {
    const { result } = renderHook(() => useWebSocket(url));
    act(() => mockWs.simulateOpen());

    // Send malformed message
    act(() => {
      mockWs.onmessage?.(new MessageEvent('message', {
        data: '{invalid json}'
      }));
    });

    // STRONG: Error handling check
    // Catches: error path mutations
    await waitFor(() => {
      expect(result.current.error).toBeTruthy();
      expect(result.current.error?.message).toContain('JSON');
    });
  });

  test('should reconnect with exponential backoff', async () => {
    jest.useFakeTimers();
    const { result } = renderHook(() =>
      useWebSocket(url, {
        reconnect: true,
        reconnectInterval: 1000,
        maxReconnectAttempts: 3
      })
    );

    // Simulate disconnect
    act(() => {
      mockWs.simulateOpen();
      mockWs.simulateClose();
    });

    // STRONG: Exact timing check
    // Catches: arithmetic mutations in backoff calculation
    act(() => jest.advanceTimersByTime(1000));
    expect(global.WebSocket).toHaveBeenCalledTimes(2); // Attempt 1

    act(() => jest.advanceTimersByTime(2000));
    expect(global.WebSocket).toHaveBeenCalledTimes(3); // Attempt 2

    act(() => jest.advanceTimersByTime(4000));
    expect(global.WebSocket).toHaveBeenCalledTimes(4); // Attempt 3

    // STRONG: Stop after max attempts
    // Catches: comparison mutations (< vs <=)
    act(() => jest.advanceTimersByTime(8000));
    expect(global.WebSocket).toHaveBeenCalledTimes(4); // No more attempts

    jest.useRealTimers();
  });

  test('should not reconnect when disabled', async () => {
    jest.useFakeTimers();
    const { result } = renderHook(() =>
      useWebSocket(url, { reconnect: false })
    );

    act(() => {
      mockWs.simulateOpen();
      mockWs.simulateClose();
    });

    act(() => jest.advanceTimersByTime(10000));

    // STRONG: Exact call count
    // Catches: boolean logic mutations
    expect(global.WebSocket).toHaveBeenCalledTimes(1);

    jest.useRealTimers();
  });
});
```

**Mutations NOW CAUGHT:**
```typescript
// MUTATION 1: Wrong initial state
const [isConnected, setConnected] = useState(true);
// ✅ CAUGHT by: expect(result.current.isConnected).toBe(false)

// MUTATION 2: Missing connection logic
const connect = () => { /* nothing */ };
// ✅ CAUGHT by: connection state test

// MUTATION 3: Wrong comparison in reconnect
if (attempts < maxAttempts) {  // Was >
// ✅ CAUGHT by: exact reconnect attempt counting
```

**Mutation Score: ~82%**

**Improvement: +72 percentage points**

---

## Example 4: Error Handling (All Languages)

### ❌ BEFORE: No Error Path Testing

```rust
// Rust
#[test]
fn test_divide() {
    assert_eq!(divide(10.0, 2.0), 5.0);
    // Missing: divide by zero test
}

// Python
def test_divide():
    assert divide(10, 2) == 5
    # Missing: divide by zero test

// TypeScript
test('divide', () => {
    expect(divide(10, 2)).toBe(5);
    // Missing: divide by zero test
});
```

**Mutations that SURVIVE:**
```rust
// MUTATION: Remove error check
fn divide(a: f64, b: f64) -> Result<f64, String> {
    // if b == 0.0 { return Err(...) }  ← DELETED
    Ok(a / b)  // Division by zero!
}
// ❌ Test doesn't cover error path
```

---

### ✅ AFTER: Complete Error Path Coverage

```rust
// Rust
#[test]
fn test_divide_normal() {
    let result = divide(10.0, 2.0).unwrap();
    assert!((result - 5.0).abs() < 0.001);
}

#[test]
fn test_divide_by_zero() {
    let result = divide(10.0, 0.0);
    assert!(result.is_err());
    assert!(result.unwrap_err().contains("division by zero"));
}

#[test]
fn test_divide_negative() {
    let result = divide(-10.0, 2.0).unwrap();
    assert!((result + 5.0).abs() < 0.001);
}

// Python
def test_divide_normal():
    assert divide(10, 2) == 5

def test_divide_by_zero():
    with pytest.raises(ZeroDivisionError, match="division by zero"):
        divide(10, 0)

def test_divide_negative():
    assert divide(-10, 2) == -5

// TypeScript
test('divide - normal case', () => {
    expect(divide(10, 2)).toBe(5);
});

test('divide - by zero throws', () => {
    expect(() => divide(10, 0)).toThrow('Division by zero');
});

test('divide - negative numbers', () => {
    expect(divide(-10, 2)).toBe(-5);
});
```

**Mutations NOW CAUGHT:**
```rust
// MUTATION: Remove error check
fn divide(a: f64, b: f64) -> Result<f64, String> {
    Ok(a / b)  // Missing zero check
}
// ✅ CAUGHT by: test_divide_by_zero expects error
```

---

## Summary: Weak vs Strong Tests

### Characteristics of WEAK Tests

❌ Check only success: `assert!(result.is_ok())`
❌ Check existence: `assert!(result.is_some())`
❌ Check type: `assert!(result.is_empty() == false)`
❌ Vague assertions: `assert!(value > 0.0)`
❌ No edge cases tested
❌ No error paths tested
❌ No boundary conditions tested

**Result:** ~15-30% mutation score

### Characteristics of STRONG Tests

✅ Check exact values: `assert_eq!(result, expected)`
✅ Check precise ranges: `assert!((value - 70.46).abs() < 0.1)`
✅ Check all properties: `assert_eq!(obj.field, expected_value)`
✅ Test edge cases: empty, null, zero, negative
✅ Test boundaries: ==, <, <=, >, >=
✅ Test error paths: invalid input handling
✅ Use parametrized tests for variations

**Result:** ~75-90% mutation score

---

## Quick Reference: Common Weak Patterns

| Weak Pattern | Strong Alternative |
|--------------|-------------------|
| `assert!(x.is_ok())` | `assert_eq!(x.unwrap(), expected)` |
| `assert!(x.is_some())` | `assert_eq!(x.unwrap(), expected_value)` |
| `assert!(x > 0)` | `assert!((x - 42.5).abs() < 0.1)` |
| `assert!(!vec.is_empty())` | `assert_eq!(vec.len(), 5)` |
| `assert_ne!(x, 0)` | `assert_eq!(x, expected_nonzero_value)` |

---

## Mutation Testing Checklist

When reviewing tests, check for:

- [ ] ✅ Tests exact values, not just success/failure
- [ ] ✅ Tests boundary conditions (==, <, <=, >, >=)
- [ ] ✅ Tests edge cases (empty, null, zero, negative)
- [ ] ✅ Tests error paths explicitly
- [ ] ✅ Uses appropriate tolerance for floats
- [ ] ✅ Would catch arithmetic operator mutations (+, -, *, /)
- [ ] ✅ Would catch comparison operator mutations (<, <=, ==, >=, >)
- [ ] ✅ Would catch return value mutations (empty, zero, null)
- [ ] ✅ Would catch boolean logic mutations (&&, ||, !)

---

**See also:**
- [MUTATION_TESTING_REPORT.md](./MUTATION_TESTING_REPORT.md) - Complete analysis
- [MUTATION_TESTING_SUMMARY.md](./MUTATION_TESTING_SUMMARY.md) - Quick reference
- [MUTATION_TESTING_CI_SETUP.md](./MUTATION_TESTING_CI_SETUP.md) - CI/CD integration
