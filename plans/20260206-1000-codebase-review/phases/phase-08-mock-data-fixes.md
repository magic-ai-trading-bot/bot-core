# Phase 08: Mock Data & Feature Completeness Fixes

**Parent Plan**: [plan.md](../plan.md)
**Dependencies**: Phase 01 (Security), Phase 03 (Python Quality)
**Priority**: P0-CRITICAL
**Effort**: Large (40-60 hours)

---

## Overview

**Date**: 2026-02-06
**Description**: Fix all mock/fake data implementations and ensure UI-to-Backend feature completeness
**Status**: Pending
**Review Status**: Not Started

---

## Key Insights

From reports `07-ui-feature-completeness.md` and `08-backend-mock-detection.md`:

### CRITICAL Findings

1. **Backtest Returns FAKE Random Data** - Users think they're testing strategies but results are completely made up
2. **Strategy Optimization Uses Random Parameters** - "Optimized" parameters are not actually optimal
3. **These features appear to work** - UI shows results, but data is meaningless

### Production Impact
- Users making trading decisions based on fake backtest results
- Potential significant financial loss
- Trust/reputation damage when discovered

---

## Requirements

### MUST FIX (P0)

| ID | Issue | File | Impact |
|----|-------|------|--------|
| MOCK-01 | Backtest returns random results | `python-ai-service/tasks/backtest_tasks.py:74-128` | Users get fake strategy validation |
| MOCK-02 | Optimization uses random params | `python-ai-service/tasks/backtest_tasks.py:204-218` | "Best" params are random |

### SHOULD FIX (P1)

| ID | Issue | File | Impact |
|----|-------|------|--------|
| MOCK-03 | Real trading uptime not tracked | Multiple files | No monitoring data |
| MOCK-04 | Portfolio report not sent | `python-ai-service/tasks/report_tasks.py` | Users don't get notifications |

### NICE TO HAVE (P2)

| ID | Issue | File | Impact |
|----|-------|------|--------|
| MOCK-05 | Order book simulated depth | `nextjs-ui-dashboard/src/components/` | Cosmetic only |

---

## Architecture

### Real Backtest Engine Requirements

```
Historical Data → Strategy Engine → Trade Simulation → Results
     |                  |                  |              |
     v                  v                  v              v
  MongoDB         Apply signals      Execute virtual    Calculate
  market_data     based on            trades with       metrics:
  collection      indicator values    slippage model    - Win rate
                                                        - Sharpe
                                                        - Max DD
                                                        - P&L
```

### Components Needed

1. **Historical Data Fetcher** - Get klines from MongoDB/Binance API
2. **Strategy Executor** - Apply strategy logic to historical data
3. **Virtual Order Engine** - Simulate order execution with:
   - Slippage simulation
   - Fee calculation
   - Partial fill handling
   - Position tracking
4. **Metrics Calculator** - Compute performance statistics
5. **Optimization Engine** - Grid search or genetic algorithm

---

## Related Code Files

### Backtest Task (TO FIX)
```
python-ai-service/tasks/backtest_tasks.py
├── async_backtest_strategy()      # Line 74-128 - MOCK DATA
├── BacktestResult model           # Line 30-50
└── async_optimize_strategy()      # Line 204-218 - RANDOM PARAMS
```

### Existing Real Components (REUSE)
```
rust-core-engine/src/
├── paper_trading/engine.rs        # Real execution simulation - REUSE
├── strategies/                    # Real strategy implementations
│   ├── rsi_strategy.rs
│   ├── macd_strategy.rs
│   └── bollinger_strategy.rs
└── binance/api.rs                 # Historical data API

python-ai-service/
├── services/market_data.py        # Market data service
├── models/                        # ML models (real)
└── main.py                        # API endpoints
```

---

## Implementation Steps

### Step 1: Add Warning Labels (Immediate - 1 hour)

```python
# In python-ai-service/routers/backtest_router.py
@router.post("/backtest")
async def backtest_strategy(...):
    """
    ⚠️ EXPERIMENTAL: Returns simulated results. NOT production-ready.
    """
    # Add warning to response
    return {
        "warning": "EXPERIMENTAL: Results are simulated, not real backtests",
        "results": results
    }
```

### Step 2: Implement Real Historical Data Fetcher (8 hours)

```python
# python-ai-service/services/backtest_service.py

class BacktestService:
    async def fetch_historical_data(
        self,
        symbol: str,
        interval: str,
        start_time: datetime,
        end_time: datetime
    ) -> list[Kline]:
        """Fetch real historical klines from MongoDB or Binance API"""
        # 1. Try MongoDB first (faster)
        klines = await self.db.market_data.find({
            "symbol": symbol,
            "interval": interval,
            "timestamp": {"$gte": start_time, "$lte": end_time}
        }).to_list(length=None)

        # 2. If not enough data, fetch from Binance
        if len(klines) < required_count:
            klines = await self.binance_client.get_historical_klines(
                symbol, interval, start_time, end_time
            )
            # Store for future use
            await self.db.market_data.insert_many(klines)

        return klines
```

### Step 3: Implement Strategy Executor (16 hours)

```python
class StrategyExecutor:
    def __init__(self, strategy_type: str, params: dict):
        self.strategy = self._load_strategy(strategy_type, params)

    def generate_signals(self, klines: list[Kline]) -> list[Signal]:
        """Apply strategy to historical data and generate signals"""
        signals = []
        for i, kline in enumerate(klines):
            # Calculate indicators at this point
            indicators = self._calculate_indicators(klines[:i+1])

            # Get strategy decision
            signal = self.strategy.evaluate(kline, indicators)
            if signal:
                signals.append(Signal(
                    timestamp=kline.timestamp,
                    action=signal.action,  # BUY/SELL/HOLD
                    price=kline.close,
                    confidence=signal.confidence
                ))
        return signals
```

### Step 4: Implement Virtual Order Engine (16 hours)

```python
class VirtualOrderEngine:
    def __init__(self, initial_balance: float, fee_rate: float = 0.001):
        self.balance = initial_balance
        self.positions = []
        self.trades = []
        self.fee_rate = fee_rate

    def execute_signal(self, signal: Signal, market_price: float) -> Trade:
        """Execute a signal with realistic simulation"""
        # Apply slippage
        slippage = self._calculate_slippage(signal, market_price)
        execution_price = market_price * (1 + slippage)

        # Calculate position size (risk management)
        position_size = self._calculate_position_size(signal.confidence)

        # Execute trade
        trade = Trade(
            timestamp=signal.timestamp,
            action=signal.action,
            price=execution_price,
            quantity=position_size,
            fee=position_size * execution_price * self.fee_rate,
            slippage=slippage
        )

        # Update balance and positions
        self._update_portfolio(trade)
        self.trades.append(trade)

        return trade

    def _calculate_slippage(self, signal: Signal, price: float) -> float:
        """Simulate realistic slippage based on order size and volatility"""
        base_slippage = 0.0005  # 0.05% base
        size_impact = min(signal.quantity / 10000, 0.001)  # Max 0.1%
        return base_slippage + size_impact
```

### Step 5: Implement Metrics Calculator (8 hours)

```python
class BacktestMetrics:
    @staticmethod
    def calculate(trades: list[Trade], initial_balance: float) -> dict:
        """Calculate comprehensive backtest metrics"""

        # Basic metrics
        total_trades = len(trades)
        winning_trades = len([t for t in trades if t.pnl > 0])
        win_rate = winning_trades / total_trades if total_trades > 0 else 0

        # P&L metrics
        total_pnl = sum(t.pnl for t in trades)
        total_return = total_pnl / initial_balance

        # Risk metrics
        returns = [t.pnl / initial_balance for t in trades]
        sharpe_ratio = calculate_sharpe(returns)
        max_drawdown = calculate_max_drawdown(trades)

        # Advanced metrics
        profit_factor = calculate_profit_factor(trades)
        avg_win = sum(t.pnl for t in trades if t.pnl > 0) / winning_trades if winning_trades > 0 else 0
        avg_loss = sum(t.pnl for t in trades if t.pnl < 0) / (total_trades - winning_trades) if (total_trades - winning_trades) > 0 else 0

        return {
            "total_trades": total_trades,
            "win_rate": round(win_rate * 100, 2),
            "total_pnl": round(total_pnl, 2),
            "total_return_pct": round(total_return * 100, 2),
            "sharpe_ratio": round(sharpe_ratio, 2),
            "max_drawdown_pct": round(max_drawdown * 100, 2),
            "profit_factor": round(profit_factor, 2),
            "avg_win": round(avg_win, 2),
            "avg_loss": round(avg_loss, 2),
            "trades": [t.to_dict() for t in trades]
        }
```

### Step 6: Implement Real Optimization (16 hours)

```python
class StrategyOptimizer:
    def __init__(self, strategy_type: str, param_ranges: dict):
        self.strategy_type = strategy_type
        self.param_ranges = param_ranges

    async def grid_search(
        self,
        historical_data: list[Kline],
        metric: str = "sharpe_ratio"
    ) -> OptimizationResult:
        """Perform grid search to find optimal parameters"""

        results = []
        param_combinations = self._generate_param_grid()

        for params in param_combinations:
            # Run backtest with these params
            executor = StrategyExecutor(self.strategy_type, params)
            signals = executor.generate_signals(historical_data)

            engine = VirtualOrderEngine(initial_balance=10000)
            for signal in signals:
                engine.execute_signal(signal, signal.price)

            metrics = BacktestMetrics.calculate(engine.trades, 10000)
            results.append({
                "params": params,
                "metrics": metrics
            })

        # Sort by optimization metric
        results.sort(key=lambda x: x["metrics"][metric], reverse=True)

        return OptimizationResult(
            best_params=results[0]["params"],
            best_metrics=results[0]["metrics"],
            all_results=results[:10]  # Top 10 results
        )
```

### Step 7: Update API Endpoints (4 hours)

```python
# python-ai-service/routers/backtest_router.py

@router.post("/backtest/run")
async def run_backtest(request: BacktestRequest) -> BacktestResponse:
    """Run real backtest with historical data"""

    # Fetch historical data
    historical_data = await backtest_service.fetch_historical_data(
        symbol=request.symbol,
        interval=request.interval,
        start_time=request.start_time,
        end_time=request.end_time
    )

    # Execute strategy
    executor = StrategyExecutor(request.strategy, request.params)
    signals = executor.generate_signals(historical_data)

    # Run virtual trading
    engine = VirtualOrderEngine(initial_balance=request.initial_balance)
    for signal in signals:
        price = historical_data[signal.index].close
        engine.execute_signal(signal, price)

    # Calculate metrics
    metrics = BacktestMetrics.calculate(engine.trades, request.initial_balance)

    return BacktestResponse(
        success=True,
        metrics=metrics,
        trades=engine.trades
    )
```

### Step 8: Update Frontend (4 hours)

- Remove "EXPERIMENTAL" warnings once backend is ready
- Add loading states for longer backtest operations
- Add visualization for backtest results (equity curve, trade markers)

---

## Todo List

### Immediate (Day 1)
- [ ] Add warning labels to backtest endpoints
- [ ] Disable optimization endpoint temporarily
- [ ] Create GitHub issues for tracking

### Week 1: Historical Data & Strategy Executor
- [ ] Implement `BacktestService.fetch_historical_data()`
- [ ] Implement `StrategyExecutor` class
- [ ] Add tests for historical data fetching
- [ ] Add tests for signal generation

### Week 2: Virtual Order Engine & Metrics
- [ ] Implement `VirtualOrderEngine` class
- [ ] Implement slippage simulation
- [ ] Implement fee calculation
- [ ] Implement `BacktestMetrics` calculator
- [ ] Add tests for trade execution
- [ ] Add tests for metrics calculation

### Week 3: Optimization & Integration
- [ ] Implement `StrategyOptimizer.grid_search()`
- [ ] Update API endpoints
- [ ] Update async tasks
- [ ] Integration testing
- [ ] Performance testing (large datasets)

### Week 4: Frontend & Documentation
- [ ] Update frontend UI
- [ ] Add equity curve visualization
- [ ] Add trade markers on chart
- [ ] Update API documentation
- [ ] Create user guide for backtesting

---

## Success Criteria

| Metric | Requirement | Verification |
|--------|-------------|--------------|
| Historical data accuracy | 100% match with Binance | Compare 1000 random klines |
| Backtest reproducibility | Same inputs = same outputs | Run 10 times, compare results |
| Slippage simulation | Within 0.01-0.5% realistic range | Validate against real trades |
| Metrics accuracy | Match manual calculation | Spot-check 10 backtests |
| Performance | <30s for 1 year backtest | Load testing |
| Test coverage | >90% for new code | Coverage report |

---

## Risk Assessment

| Risk | Likelihood | Impact | Mitigation |
|------|------------|--------|------------|
| Incorrect backtest results | Medium | HIGH | Extensive testing, comparison with known results |
| Performance issues with large datasets | High | Medium | Implement pagination, caching |
| Historical data gaps | Medium | Medium | Fallback to Binance API |
| Users trust fake results until fix | HIGH | HIGH | Add prominent warnings immediately |

---

## Security Considerations

1. **Data Integrity**: Ensure historical data cannot be manipulated
2. **Rate Limiting**: Prevent abuse of backtest API (expensive operation)
3. **Resource Limits**: Cap backtest duration and data size
4. **Audit Logging**: Log all backtest requests for debugging

---

## Next Steps

After this phase:
1. Remove "EXPERIMENTAL" warnings
2. Update documentation
3. Announce feature completion to users
4. Monitor for issues in production
5. Collect feedback for improvements
