# Quick Reference - Bot-Core Codebase

## 🚀 Quick Navigation

### Rust Core Engine (Port 8080)
| Component | Location | Lines | Purpose |
|-----------|----------|-------|---------|
| Entry Point | `src/main.rs` | 214 | Bootstrap all services |
| Paper Trading | `src/paper_trading/engine.rs` | 1200+ | Execution simulation |
| Strategies | `src/strategies/strategy_engine.rs` | 800+ | Signal generation |
| API Routes | `src/api/mod.rs` | 600+ | REST endpoints |
| Authentication | `src/auth/jwt.rs` | 300+ | JWT tokens |
| Market Data | `src/market_data/processor.rs` | 450+ | Candle aggregation |

### Python AI Service (Port 8000)
| Component | Location | Lines | Purpose |
|-----------|----------|-------|---------|
| Entry Point | `main.py` | 2000+ | FastAPI app + GPT-4 |
| Model Manager | `models/model_manager.py` | 800+ | ML lifecycle |
| Tech Indicators | `features/technical_indicators.py` | 600+ | TA calculations |
| ML Tasks | `tasks/ml_tasks.py` | 500+ | Async training |
| Project Chatbot | `services/project_chatbot.py` | 400+ | RAG chatbot |

### React Frontend (Port 3000)
| Component | Location | Lines | Purpose |
|-----------|----------|-------|---------|
| Root | `src/App.tsx` | 300+ | App orchestration |
| Dashboard | `src/pages/Dashboard.tsx` | 500+ | Main UI |
| WebSocket | `src/hooks/useWebSocket.ts` | 400+ | Real-time connection |
| Trading | `src/components/TradingInterface.tsx` | 400+ | Trading panel |
| API Client | `src/services/api.ts` | 300+ | REST client |

---

## 📁 Finding Files

**Looking for...?**

| Feature | Files | Location |
|---------|-------|----------|
| **Paper Trading** | engine, portfolio, trade, settings | `rust-core-engine/src/paper_trading/` |
| **Real Trading** | engine, order, risk | `rust-core-engine/src/real_trading/` |
| **Authentication** | jwt, handlers, middleware | `rust-core-engine/src/auth/` |
| **Trading Signals** | RSI, MACD, Bollinger, Volume | `rust-core-engine/src/strategies/` |
| **AI Analysis** | ML models, GPT-4 | `python-ai-service/models/` + `main.py` |
| **API Routes** | Endpoints | `rust-core-engine/src/api/` + `python-ai-service/main.py` |
| **Dashboard UI** | Pages, components | `nextjs-ui-dashboard/src/pages/` + `src/components/` |
| **Database** | Collections, schema | `specs/02-design/2.2-database/DB-SCHEMA.md` |

---

## 🔌 API Quick Reference

### Rust Endpoints (Main)

```
Auth:
  POST   /api/auth/login                 # Email + password
  POST   /api/auth/register              # Create account
  POST   /api/auth/refresh               # Refresh JWT

Paper Trading:
  GET    /api/paper-trading/portfolio    # Portfolio summary
  POST   /api/paper-trading/execute      # Execute trade
  GET    /api/paper-trading/trades       # Trade history
  POST   /api/paper-trading/settings     # Update settings

Market:
  GET    /api/market/ticker/:symbol      # Current price
  GET    /api/market/candles/:symbol     # Historical candles

Notifications:
  GET    /ws                             # WebSocket upgrade
```

### Python Endpoints

```
AI:
  POST   /predict                        # ML price prediction
  POST   /analyze                        # GPT-4 market analysis
  POST   /sentiment                      # Sentiment analysis
  POST   /train                          # Model training
  GET    /signals                        # Current signals
```

---

## 📊 Database Collections (17 total)

```
Core:
  - users              (accounts)
  - paper_portfolios   (paper trading balance)
  - paper_trades       (executed trades)
  - real_portfolios    (live trading)
  - real_trades        (live executed)

Config:
  - strategies         (strategy settings)
  - settings           (user preferences)
  - api_keys           (API credentials)

Data:
  - market_data        (OHLCV candles)
  - signals            (generated signals)
  - ai_analysis_results (GPT-4 cache)
  - technical_indicators (TA cache)

Admin:
  - notifications      (alerts & logs)
  - audit_logs         (activity tracking)
  - performance_metrics (strategy stats)
  - user_symbols       (watchlist)
  - websocket_connections (sessions)
```

---

## 🎯 Common Tasks

### Add a New Trading Strategy

1. **Define signal** → `rust-core-engine/src/strategies/[strategy]_strategy.rs`
2. **Register** in `strategy_engine.rs`
3. **Add tests** → `rust-core-engine/tests/test_strategies.rs`
4. **Update docs** → Update relevant feature docs

### Add API Endpoint

1. **Rust**: Create handler in `rust-core-engine/src/api/`
2. **Python**: Add route in `python-ai-service/main.py`
3. **Frontend**: Add hook in `nextjs-ui-dashboard/src/hooks/`
4. **Test**: Unit tests + integration tests

### Fix UI Bug

1. **Find component** → `nextjs-ui-dashboard/src/components/`
2. **Locate bug** → Check hooks if state issue
3. **Fix** → Update component or context
4. **Test** → `npm run test` in dashboard directory

### Deploy to Production

1. **Build**: `make build` (all services)
2. **Tests**: `make test` (must pass)
3. **Quality**: `make quality-metrics` (must be 94+)
4. **Deploy**: Use GitHub Actions or `make deploy`

---

## 🧪 Testing

```bash
# Run all tests (2,202+ tests)
make test

# Run specific service
cd rust-core-engine && cargo test
cd python-ai-service && pytest
cd nextjs-ui-dashboard && npm test

# Check coverage
make test-coverage

# Mutation testing (code quality)
make mutation-testing

# Type checking
cd nextjs-ui-dashboard && npm run type-check
```

---

## 🏗️ Architecture Overview

```
┌─────────────────────────────────────────────────────────────┐
│                     REACT FRONTEND (3000)                    │
│  Dashboard │ Trading │ AI Signals │ Portfolio │ Settings     │
└──────────────────┬────────────────────────────────────────┬──┘
                   │ REST APIs                    WebSocket │
                   │                                         │
        ┌──────────▼──────────────────────────────▼────────┐
        │       RUST CORE ENGINE (8080)                     │
        │  ┌─────────────────────────────────────────────┐  │
        │  │  Paper Trading │ Strategies │ Risk Mgmt    │  │
        │  └─────────────────────────────────────────────┘  │
        │  ┌─────────────────────────────────────────────┐  │
        │  │  Binance API │ Market Data │ WebSocket     │  │
        │  └─────────────────────────────────────────────┘  │
        └──────────────┬──────────────────┬──────────────────┘
                       │                  │
            ┌──────────▼──────┐    ┌──────▼─────────────┐
            │ PYTHON AI (8000)│    │  MONGODB/REDIS     │
            │ ┌──────────────┐│    │  ┌──────────────┐ │
            │ │ GPT-4        ││    │  │ User Data    │ │
            │ │ ML Models    ││    │  │ Portfolios   │ │
            │ │ Tech Ind.    ││    │  │ Trades       │ │
            │ └──────────────┘│    │  │ Signals      │ │
            └──────────────────┘    │  └──────────────┘ │
                                    └────────────────────┘
```

---

## 💡 Key Concepts

### Paper Trading Simulation
- Realistic slippage (0.05-0.15%)
- Partial fills (depends on volume)
- Daily loss limits (5% max)
- Cool-down mechanism (60 min after 5 losses)
- Per-symbol leverage settings

### Risk Management
- Maximum drawdown tracking
- Correlation-based position limits
- Leverage multiplier (1x-10x)
- Stop loss & take profit automation

### AI Integration
- GPT-4 for market analysis
- ML models: LSTM, GRU, Transformer
- Technical indicators: RSI, MACD, Bollinger, Stochastic
- Signal confidence scoring

### WebSocket Real-Time
- Binance price streams (1s candles)
- Frontend live updates
- Position changes
- Trade notifications
- Risk alerts

---

## 🔐 Security Notes

- JWT tokens signed with RS256
- Passwords hashed with bcrypt
- API keys encrypted in database
- 2FA support (TOTP-based)
- Rate limiting on API endpoints
- CORS configured for frontend only
- Real trading disabled by default

---

## 📚 Important Links

| Resource | Path |
|----------|------|
| Specifications | `/specs/` (75 docs) |
| Documentation | `/docs/` (operational guides) |
| Configuration | `config.toml` (Rust), `config.yaml` (Python) |
| Examples | `/examples/` (code examples) |
| Scripts | `/scripts/` (orchestration) |
| Makefile | `/Makefile` (build targets) |
| Development Guide | `/CLAUDE.md` (AI development) |

---

## ⚡ Performance Notes

- Paper trading: <10ms execution
- API response: <100ms typical
- WebSocket: <50ms latency
- ML inference: <500ms per symbol
- Database queries: <20ms typical
- Frontend build: <30s dev, <60s prod

---

## 🚨 Critical Files (Do Not Break!)

1. `rust-core-engine/src/paper_trading/engine.rs` - Core trading logic
2. `rust-core-engine/src/auth/jwt.rs` - Authentication
3. `python-ai-service/main.py` - AI service
4. `nextjs-ui-dashboard/src/contexts/WebSocketContext.tsx` - Real-time connection
5. `specs/TRACEABILITY_MATRIX.md` - Requirement tracking

---

**Last Updated**: 2026-02-06
**For More Details**: See `scout-01-codebase-structure.md`

