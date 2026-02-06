# Scout Report: Full Codebase Structure Map
**Generated**: 2026-02-06 | **Task**: Map entire bot-core codebase structure

---

## Executive Summary

**bot-core** is a comprehensive cryptocurrency trading bot with three main services:
- **Rust Core Engine** (8080): Trading execution, WebSocket, paper/real trading, strategies
- **Python AI Service** (8000): ML models, GPT-4 analysis, technical indicators
- **NextJS Frontend** (3000): React dashboard, trading UI, real-time updates

All services communicate via REST APIs, WebSockets, and MongoDB database.

---

## 1. ROOT DIRECTORY STRUCTURE

```
bot-core/
├── rust-core-engine/          # Rust backend - trading engine (Actix-web)
├── python-ai-service/         # Python backend - AI/ML service (FastAPI)
├── nextjs-ui-dashboard/       # React frontend - trading dashboard (Vite)
├── infrastructure/            # Docker, Kubernetes, Terraform configs
├── scripts/                   # Shell scripts for orchestration
├── docs/                      # Operational documentation
├── specs/                     # Technical specifications
├── plans/                     # Planning & scout reports
├── examples/                  # Code examples
├── .github/                   # GitHub Actions CI/CD
├── Makefile                   # Build orchestration
├── CLAUDE.md                  # AI development guide
├── README.md                  # Project overview
├── package.json               # Root monorepo config
├── bun.lock                   # Dependency lock file
├── docker-compose.yml         # Local dev orchestration
└── .env                       # Environment variables
```

---

## 2. RUST CORE ENGINE (`rust-core-engine/src/`)

**Framework**: Actix-web + Tokio async runtime
**Database**: MongoDB 
**Key Features**: Paper trading, risk management, strategies, WebSocket

### 2.1 Module Structure

```
rust-core-engine/src/
├── main.rs                    # Entry point - bootstraps all services
├── lib.rs                     # Library exports & tests
├── config.rs                  # Configuration loader (TOML)
├── error.rs                   # Error types & handling
│
├── api/                       # REST API routes (Warp/Actix)
│   ├── mod.rs                # API server orchestration
│   ├── paper_trading.rs       # Paper trading endpoints
│   ├── real_trading.rs        # Real trading endpoints  
│   ├── settings.rs            # Settings persistence API
│   └── notifications.rs       # WebSocket broadcast API
│
├── auth/                      # Authentication & security
│   ├── mod.rs                # Module orchestration
│   ├── models.rs             # User, JWT models
│   ├── jwt.rs                # JWT token generation/validation
│   ├── handlers.rs           # Login/logout/register routes
│   ├── middleware.rs         # Auth middleware
│   ├── database.rs           # User DB operations
│   └── security_handlers.rs  # 2FA, API keys, etc.
│
├── binance/                   # Binance API integration
│   ├── mod.rs                # Module orchestration
│   ├── client.rs             # HTTP client for REST API
│   ├── websocket.rs          # WebSocket stream handler
│   ├── user_data_stream.rs   # Account balance updates
│   └── types.rs              # Order, ticker, account types
│
├── market_data/               # Real-time price data
│   ├── mod.rs                # Module orchestration
│   ├── processor.rs          # Candle aggregation & caching
│   ├── analyzer.rs           # Technical analysis helpers
│   ├── cache.rs              # In-memory price cache
│   └── market_data/          # Market data types
│
├── strategies/                # Trading signal generation
│   ├── mod.rs                # Strategy orchestration
│   ├── strategy_engine.rs     # Main strategy coordinator
│   ├── rsi_strategy.rs        # RSI-based strategy
│   ├── macd_strategy.rs       # MACD momentum strategy
│   ├── bollinger_strategy.rs  # Bollinger Bands strategy
│   ├── volume_strategy.rs     # Volume analysis strategy
│   ├── stochastic_strategy.rs # Stochastic oscillator
│   ├── trend_filter.rs        # Trend confirmation filter
│   ├── hybrid_filter.rs       # Combined technical filters
│   ├── ml_trend_predictor.rs  # ML-based trend prediction
│   ├── indicators.rs          # RSI, MACD, EMA calculations
│   ├── types.rs               # Signal, strategy types
│   └── tests.rs               # Strategy test suite
│
├── paper_trading/             # Simulated trading engine
│   ├── mod.rs                # Paper trading orchestration
│   ├── engine.rs             # Main execution engine (1200+ lines)
│   │                         # - Execution simulation (slippage, fills)
│   │                         # - Risk management (daily loss, cooldown)
│   │                         # - Trade lifecycle management
│   ├── portfolio.rs          # Portfolio state & positions
│   ├── trade.rs              # Individual trade representation
│   ├── settings.rs           # Per-symbol trading settings
│   └── strategy_optimizer.rs # ML strategy parameter tuning
│
├── real_trading/              # Live trading (disabled by default)
│   ├── mod.rs                # Real trading orchestration
│   ├── engine.rs             # Live execution engine
│   ├── config.rs             # Real trading config
│   ├── order.rs              # Order management
│   ├── position.rs           # Position tracking
│   └── risk.rs               # Risk controls for live trading
│
├── trading/                   # Shared trading logic
│   ├── mod.rs                # Module orchestration
│   ├── engine.rs             # Base trading engine
│   ├── position_manager.rs   # Position lifecycle
│   └── risk_manager.rs       # Risk calculation & limits
│
├── ai/                        # AI service integration
│   ├── mod.rs                # AI module orchestration
│   ├── client.rs             # HTTP client to Python service
│   └── types.rs              # AI response models
│
├── monitoring/                # System monitoring
│   └── mod.rs                # Metrics, health checks
│
└── storage/                   # MongoDB persistence
    └── mod.rs                # Database client & operations
```

### 2.2 Key Rust Files (Critical)

| File | Lines | Purpose |
|------|-------|---------|
| `paper_trading/engine.rs` | 1200+ | Core execution sim + risk mgmt |
| `strategies/strategy_engine.rs` | 800+ | Strategy orchestration |
| `api/mod.rs` | 600+ | REST API server setup |
| `binance/websocket.rs` | 500+ | Binance stream integration |
| `market_data/processor.rs` | 450+ | Candle aggregation |
| `auth/jwt.rs` | 300+ | JWT token handling |

### 2.3 Rust Dependencies (Top 20)

```toml
tokio = "1.48"                          # Async runtime
warp = "0.4"                            # HTTP server
serde/serde_json = "1.0"                # JSON serialization
mongodb/bson = "3.3/2.15"               # Database
reqwest = "0.12"                        # HTTP client
chrono = "0.4"                          # Date/time
uuid = "1.19"                           # IDs
tracing = "0.1"                         # Logging
tokio-tungstenite = "0.28"              # WebSocket
rust_decimal = "1.39"                   # Precise decimals
jsonwebtoken = "10.2"                   # JWT
bcrypt = "0.17"                         # Password hashing
base64/hex = "0.22/0.4"                 # Encoding
```

---

## 3. PYTHON AI SERVICE (`python-ai-service/`)

**Framework**: FastAPI + Uvicorn
**Database**: MongoDB + Redis cache
**Key Features**: GPT-4 analysis, ML models (LSTM/GRU/Transformer), technical indicators

### 3.1 Module Structure

```
python-ai-service/
├── main.py                    # FastAPI app entry point
├── config_loader.py           # YAML config reader
├── settings_manager.py        # Settings cache with Rust API sync
├── celery_app.py              # Celery task queue setup
├── requirements.txt           # Dependencies (50+ packages)
│
├── app/                       # FastAPI application
│   ├── __init__.py
│   ├── core/
│   │   ├── config.py          # App configuration
│   │   └── state.py           # Global app state
│   ├── models/
│   │   └── schemas.py         # Pydantic request/response models
│   ├── routers/               # API route handlers
│   ├── services/              # Business logic layer
│   └── websocket/
│       └── manager.py         # WebSocket connection management
│
├── models/                    # ML models
│   ├── __init__.py
│   ├── model_manager.py       # Model loading & inference
│   ├── lstm_model.py          # LSTM network (price prediction)
│   ├── gru_model.py           # GRU network (sequence modeling)
│   └── transformer_model.py   # Transformer (attention-based)
│
├── features/                  # Feature engineering
│   ├── __init__.py
│   ├── feature_engineering.py # Data preprocessing
│   └── technical_indicators.py# RSI, MACD, Bollinger, etc.
│
├── services/                  # Service layer
│   ├── __init__.py
│   └── project_chatbot.py     # RAG-based chatbot
│
├── tasks/                     # Celery async tasks
│   ├── __init__.py
│   ├── ml_tasks.py            # Model training/inference
│   ├── backtest_tasks.py      # Strategy backtesting
│   ├── monitoring.py          # Health monitoring
│   └── ai_improvement.py      # Model improvement tasks
│
├── utils/                     # Utility functions
│   ├── __init__.py
│   ├── logger.py              # Logging setup
│   ├── notifications.py       # Alert/notification system
│   ├── data_storage.py        # Data persistence
│   ├── redis_cache.py         # Cache operations
│   └── helpers.py             # General utilities
│
├── config/                    # Configuration module
│   ├── __init__.py
│   └── config.py              # Config models
│
├── tests/                     # Test suite (40+ test files)
│   ├── test_main.py           # Main API tests
│   ├── test_models.py         # Model tests
│   ├── test_feature_engineering.py
│   ├── test_technical_indicators.py
│   ├── test_api_endpoints.py
│   ├── conftest.py            # Pytest fixtures
│   └── [28 more test files]
│
├── examples/
│   └── example_client.py      # Usage examples
│
└── scripts/
    ├── test_setup.py          # Setup verification
    └── analyze_trades_now.py  # Trade analysis script
```

### 3.2 Key Python Files

| File | Lines | Purpose |
|------|-------|---------|
| `main.py` | 2000+ | FastAPI app with GPT-4 endpoints |
| `models/model_manager.py` | 800+ | ML model lifecycle |
| `features/technical_indicators.py` | 600+ | TA indicators |
| `tasks/ml_tasks.py` | 500+ | Async ML jobs |
| `services/project_chatbot.py` | 400+ | RAG chatbot |

### 3.3 Python Dependencies (Key 25)

```
tensorflow>=2.18.0              # Deep learning
torch>=2.9.0                    # PyTorch ML
scikit-learn>=1.6.0             # ML algorithms
pandas>=2.2.0                   # Data manipulation
numpy>=1.26.0                   # Numerical computing
fastapi>=0.115.0                # Web framework
uvicorn>=0.30.0                 # ASGI server
pydantic>=2.9.0                 # Data validation
openai>=1.50.0                  # GPT-4 API
ta>=0.11.0                      # Technical analysis
keras>=3.5.0                    # Deep learning API
h5py>=3.12.0                    # Model serialization
pymongo>=4.10.0                 # MongoDB driver
motor>=3.6.0                    # Async MongoDB
redis>=5.3.0                    # Cache
celery[redis]>=5.4.0            # Task queue
```

### 3.4 API Endpoints (Python)

```
POST   /predict                 # ML price prediction
POST   /analyze                 # GPT-4 market analysis  
POST   /sentiment               # Sentiment analysis
POST   /train                   # Model retraining
GET    /signals                 # Current signals
GET    /health                  # Health check
POST   /backtest                # Strategy backtest
WS     /ws                      # WebSocket feed
```

---

## 4. NEXTJS FRONTEND (`nextjs-ui-dashboard/src/`)

**Framework**: React 18 + Vite + TypeScript
**UI Library**: Shadcn/UI + TailwindCSS + Radix UI
**State**: React Context + TanStack Query (React Query)

### 4.1 Directory Structure

```
nextjs-ui-dashboard/src/
├── main.tsx                   # React entry point
├── index.css                  # Global styles
├── App.tsx                    # Root app component
├── i18n.ts                    # Internationalization setup
│
├── pages/                     # Page components (26 pages)
│   ├── Index.tsx              # Home page
│   ├── Dashboard.tsx          # Main trading dashboard
│   ├── PaperTrading.tsx       # Paper trading page
│   ├── RealTrading.tsx        # Real trading page (locked)
│   ├── AISignals.tsx          # AI signals page
│   ├── Portfolio.tsx          # Portfolio overview
│   ├── Settings.tsx           # User settings
│   ├── Login.tsx              # Authentication
│   ├── Register.tsx           # User signup
│   ├── Profile.tsx            # User profile
│   ├── TradeAnalyses.tsx      # Trade analysis
│   ├── HowItWorks.tsx         # Tutorial
│   ├── Features.tsx           # Feature showcase
│   ├── Pricing.tsx            # Pricing page
│   ├── API.tsx                # API documentation
│   ├── Documentation.tsx      # Help docs
│   ├── Blog.tsx               # Blog/news
│   ├── About.tsx              # About page
│   ├── Contact.tsx            # Contact form
│   ├── Careers.tsx            # Job listings
│   ├── Compliance.tsx         # Legal/compliance
│   ├── Privacy.tsx            # Privacy policy
│   ├── Terms.tsx              # Terms of service
│   ├── SecurityPage.tsx       # Security center
│   ├── Error.tsx              # Error page
│   └── NotFound.tsx           # 404 page
│
├── components/                # Reusable components (71 total)
│   ├── ui/                    # Shadcn/UI primitives (40 files)
│   │   ├── button.tsx         # Button component
│   │   ├── card.tsx           # Card container
│   │   ├── dialog.tsx         # Modal dialog
│   │   ├── form.tsx           # Form wrapper
│   │   ├── input.tsx          # Text input
│   │   ├── select.tsx         # Dropdown selector
│   │   ├── tabs.tsx           # Tab interface
│   │   ├── table.tsx          # Data table
│   │   ├── chart.tsx          # Chart component
│   │   ├── toast.tsx          # Notifications
│   │   ├── dropdown-menu.tsx  # Context menu
│   │   ├── sidebar.tsx        # Sidebar layout
│   │   ├── accordion.tsx      # Collapsible sections
│   │   ├── alert.tsx          # Alert boxes
│   │   ├── badge.tsx          # Status badges
│   │   ├── avatar.tsx         # User avatars
│   │   ├── sheet.tsx          # Mobile drawer
│   │   ├── pagination.tsx     # Page navigation
│   │   └── [30+ more UI components]
│   │
│   ├── dashboard/             # Dashboard components (14 files)
│   │   ├── DashboardHeader.tsx    # Header bar
│   │   ├── DashboardContentHeader.tsx
│   │   ├── BotSettings.tsx        # Settings panel
│   │   ├── TradingSettings.tsx     # Trading config
│   │   ├── PerSymbolSettings.tsx  # Per-symbol rules
│   │   ├── TradingCharts.tsx      # Charts/graphs
│   │   ├── PerformanceChart.tsx   # P&L chart
│   │   ├── StrategyComparison.tsx # Strategy stats
│   │   ├── AISignals.tsx          # AI signal display
│   │   ├── AISignalsNew.tsx       # New signal format
│   │   ├── BotStatus.tsx          # Bot status widget
│   │   ├── SystemMonitoring.tsx   # System health
│   │   ├── QuickActionsBar.tsx    # Quick actions
│   │   ├── BentoGrid.tsx          # Dashboard grid
│   │   ├── PortfolioSummaryCard.tsx
│   │   ├── PortfolioQuickActions.tsx
│   │   ├── PriceTickerRow.tsx     # Price ticker
│   │   ├── TransactionHistory.tsx # Trade log
│   │   ├── AIStrategySelector.tsx # Strategy picker
│   │   ├── StrategyTuningSettings.tsx
│   │   ├── MobileNav.tsx          # Mobile navigation
│   │   └── PerSymbolSettings.example.tsx
│   │
│   ├── trading/               # Trading-related components (12 files)
│   │   ├── TradingInterface.tsx    # Main trading panel
│   │   ├── TradingLayout.tsx       # Layout wrapper
│   │   ├── OrderForm.tsx           # Order entry form
│   │   ├── OpenPositions.tsx       # Active trades
│   │   ├── OpenPositionsTable.tsx  # Positions table
│   │   ├── ClosedTradesTable.tsx   # Trade history
│   │   ├── TradeHistory.tsx        # Historical trades
│   │   ├── TradeConfirmationDialog.tsx
│   │   ├── RiskMetrics.tsx         # Risk stats
│   │   ├── RiskWarningCard.tsx     # Risk alerts
│   │   ├── PortfolioStats.tsx      # Portfolio summary
│   │   ├── TradingChartPanel.tsx   # Chart display
│   │   ├── TradingViewChart.tsx    # Chart integration
│   │   ├── AIInsightsPanel.tsx     # AI insights
│   │   ├── ModeToggle.tsx          # Paper/Real switch
│   │   ├── ModeSwitchDialog.tsx    # Mode confirmation
│   │   └── RealModeWarningBanner.tsx
│   │
│   ├── ai/                    # AI-specific components (4 files)
│   │   ├── AISignalsDashboard.tsx  # Signals overview
│   │   ├── SignalCard.tsx          # Signal display
│   │   ├── DetailedSignalDialog.tsx# Signal details
│   │   └── StrategyExplanation.tsx  # Strategy explanation
│   │
│   ├── landing/               # Landing page sections (5 files)
│   │   ├── FeaturesSection.tsx
│   │   ├── TestimonialsSection.tsx
│   │   ├── FAQSection.tsx
│   │   ├── PartnersSection.tsx
│   │   └── LandingFooter.tsx
│   │
│   ├── TradingInterface.tsx   # Primary trading component
│   ├── ErrorBoundary.tsx      # Error handling
│   ├── ProtectedRoute.tsx     # Auth guard
│   ├── ChatBot.tsx            # AI chatbot widget
│   ├── ProductTour.tsx        # Onboarding tour
│   ├── LivePriceTicker.tsx    # Price ticker
│   ├── ScrollToTop.tsx        # Scroll behavior
│   ├── ThemeToggle.tsx        # Dark/light mode
│   ├── LanguageSelector.tsx   # Language picker
│   ├── BotCoreLogo.tsx        # Logo component
│   └── [additional components]
│
├── hooks/                     # Custom React hooks (26 files)
│   ├── useWebSocket.ts        # WebSocket connection
│   ├── useWebSocket.enhanced.test.tsx
│   ├── useTradingApi.ts       # Trading API calls
│   ├── usePaperTrading.ts     # Paper trading hooks
│   ├── useRealTrading.ts      # Real trading hooks
│   ├── useMarketData.ts       # Market data fetching
│   ├── useAIAnalysis.ts       # AI analysis calls
│   ├── useAccount.ts          # Account data
│   ├── usePositions.ts        # Position tracking
│   ├── useTrades.ts           # Trade history
│   ├── useSecurity.ts         # Security operations
│   ├── useNotification.ts     # Notifications
│   ├── usePushNotifications.ts# Push notifications
│   ├── useNotificationPreferences.ts
│   ├── useThemeColors.ts      # Theme colors
│   ├── useOnlineStatus.ts     # Network status
│   ├── useDeviceCapability.ts # Device features
│   ├── useDebouncedValue.ts   # Debouncing
│   ├── useBreakpoint.ts       # Responsive breakpoints
│   ├── useMediaQuery.ts       # Media queries
│   ├── useMobile.ts           # Mobile detection
│   ├── use-mobile.tsx         # Mobile hook variant
│   ├── use-toast.ts           # Toast notifications
│   ├── useTradingMode.ts      # Trading mode (paper/real)
│   ├── useSidebar.ts          # Sidebar state
│   └── index.ts               # Exports
│
├── contexts/                  # React Context providers (8 files)
│   ├── AuthContext.tsx        # Authentication state
│   ├── WebSocketContext.tsx   # WebSocket state
│   ├── PaperTradingContext.tsx# Paper trading state
│   ├── AIAnalysisContext.tsx  # AI analysis state
│   ├── TradingModeContext.tsx # Mode (paper/real) state
│   ├── ThemeContext.tsx       # Dark/light theme
│   ├── LanguageContext.tsx    # i18n language
│   ├── NotificationContext.tsx# Notifications
│   ├── SidebarContext.tsx     # Sidebar state
│   └── index.ts               # Context exports
│
├── services/                  # API client layer (2 files)
│   ├── api.ts                 # REST client (axios)
│   └── chatbot.ts             # Chatbot API calls
│
├── lib/                       # Utilities (1 file)
│   └── utils.ts               # Helper functions
│
├── utils/                     # Utility functions (3 files)
│   ├── formatters.ts          # Data formatting
│   ├── logger.ts              # Logging
│   └── [additional utilities]
│
├── constants/                 # Constants (1 file)
│   └── trading.ts             # Trading constants
│
├── types/                     # Type definitions (2 files)
│   ├── env.d.ts               # Environment types
│   └── trading.ts             # Trading types
│
├── styles/                    # Global styles (2 files)
│   ├── tokens/
│   │   ├── typography.ts      # Font tokens
│   │   └── spacing.ts         # Spacing tokens
│   └── [additional styles]
│
├── test/                      # Test utilities
│   └── setup.ts               # Test setup
│
├── __tests__/                 # Test files (30+ tests)
│   ├── hooks/
│   │   ├── useAccount.test.ts
│   │   ├── usePositions.test.ts
│   │   ├── useTrades.test.ts
│   │   ├── useMarketData.test.ts
│   │   ├── useTradingApi.test.ts
│   │   ├── useWebSocket.test.tsx
│   │   ├── useWebSocket.enhanced.test.tsx
│   │   ├── usePaperTrading.test.ts
│   │   ├── use-mobile.test.tsx
│   │   └── [additional hook tests]
│   ├── contexts/
│   │   └── AuthContext.test.tsx
│   ├── components/
│   │   ├── TradingInterface.test.tsx
│   │   └── dashboard/
│   │       ├── BotSettings.test.tsx
│   │       └── TradingCharts.test.tsx
│   ├── services/
│   │   └── api.test.ts
│   ├── utils/
│   │   └── formatters.test.ts
│   └── integration/
│       └── component-integration.test.tsx
│
├── main.tsx                   # Vite entry point
├── vite-env.d.ts             # Vite types
└── i18n.ts                   # i18n configuration
```

### 4.2 Key Frontend Files

| File | Lines | Purpose |
|------|-------|---------|
| `pages/Dashboard.tsx` | 500+ | Main dashboard page |
| `pages/PaperTrading.tsx` | 400+ | Paper trading UI |
| `hooks/useWebSocket.ts` | 400+ | WebSocket connection |
| `components/TradingInterface.tsx` | 400+ | Trading panel |
| `contexts/AuthContext.tsx` | 300+ | Auth state mgmt |
| `services/api.ts` | 300+ | REST client |

### 4.3 Frontend Dependencies (Top 25)

```json
react = "^18.3.1"              # UI library
typescript = "^5.8.0"          # Type checking
vite = "^6.3.0"                # Build tool
tailwindcss = "^4.1.0"         # Styling
@radix-ui/react-*              # Component primitives
shadcn/ui                      # Pre-built components
@tanstack/react-query          # Data fetching
axios = "^1.6.2"               # HTTP client
recharts = "^2.15.0"           # Charting
react-i18next = "^15.2.0"      # i18n
zustand = "^5.0.0"             # State management
date-fns = "^3.6.0"            # Date utilities
```

---

## 5. INFRASTRUCTURE & DEVOPS

### 5.1 Docker Setup

```
Dockerfiles:
├── rust-core-engine/
│   ├── Dockerfile             # Production build
│   ├── Dockerfile.dev         # Dev build (hot reload)
│   └── Dockerfile.production
├── python-ai-service/
│   ├── Dockerfile             # Production
│   ├── Dockerfile.dev         # Dev
│   ├── Dockerfile.ci          # CI/CD tests
│   └── Dockerfile.vps         # VPS deployment
└── nextjs-ui-dashboard/
    ├── Dockerfile             # Production
    └── Dockerfile.dev         # Dev

Compose:
├── docker-compose.yml         # Local dev (3 services + MongoDB + Redis)
├── docker-compose.prod.yml    # Production
└── docker-compose-vps.yml     # VPS config
```

### 5.2 Infrastructure Directories

```
infrastructure/
├── docker/                    # Docker compose files
├── kubernetes/                # K8s manifests (prod)
├── terraform/                 # IaC for cloud resources
├── monitoring/                # Prometheus, Grafana
├── grafana/                   # Dashboard configs
├── nginx/                     # Reverse proxy
├── kong/                      # API gateway
├── mongodb/                   # DB init scripts
├── rabbitmq/                  # Message queue (optional)
├── cron/                      # Scheduled tasks
├── secrets/                   # Secret management
└── fly/                       # Fly.io deployment
```

### 5.3 CI/CD Pipelines (GitHub Actions)

```
.github/workflows/
├── lint.yml                   # Code quality checks
├── test-coverage.yml          # Unit/integration tests
├── security-scan.yml          # Vulnerability scanning
├── mutation-testing.yml       # Mutation score tests
├── docker-build-push.yml      # Docker image build
├── integration-tests.yml      # End-to-end tests
└── deploy-vps.yml            # Production deployment
```

---

## 6. DATABASE SCHEMA (MongoDB)

### 6.1 Collections (17 total)

| Collection | Purpose | Key Fields |
|------------|---------|-----------|
| `users` | User accounts | `_id`, `email`, `password_hash`, `api_keys`, `created_at` |
| `paper_portfolios` | Paper trading accounts | `user_id`, `balance`, `equity`, `roi%`, `settings` |
| `paper_trades` | Executed paper trades | `portfolio_id`, `symbol`, `entry`, `exit`, `pnl`, `status` |
| `real_portfolios` | Real trading accounts | `user_id`, `api_key_encrypted`, `balance`, `settings` |
| `real_trades` | Real executed trades | `portfolio_id`, `symbol`, `qty`, `pnl`, `fees` |
| `strategies` | Strategy configs | `name`, `type`, `parameters`, `enabled`, `win_rate` |
| `market_data` | OHLCV candles | `symbol`, `timestamp`, `open/high/low/close`, `volume` |
| `signals` | Generated signals | `symbol`, `signal_type`, `confidence`, `timestamp` |
| `ai_analysis_results` | GPT-4 analysis cache | `symbol`, `analysis`, `timestamp`, `expires_at` |
| `settings` | User preferences | `user_id`, `trading_mode`, `risk_level`, `indicators` |
| `notifications` | Alerts & logs | `user_id`, `type`, `message`, `read`, `created_at` |
| `api_keys` | API credentials | `user_id`, `name`, `key_hash`, `permissions` |
| `audit_logs` | Activity tracking | `user_id`, `action`, `timestamp`, `details` |
| `performance_metrics` | Strategy metrics | `strategy_id`, `roi`, `sharpe`, `win_rate` |
| `user_symbols` | Custom watchlist | `user_id`, `symbols`, `created_at` |
| `technical_indicators` | TA cache | `symbol`, `timeframe`, `rsi`, `macd`, `timestamp` |
| `websocket_connections` | Active sessions | `user_id`, `connection_id`, `connected_at` |

### 6.2 Indexes (37 total)

See `specs/02-design/2.2-database/DB-INDEXES.md` for complete index list.

---

## 7. API ROUTES SUMMARY

### 7.1 Rust Core Engine (Port 8080)

**Auth Routes** (`/api/auth/`):
```
POST   /login              # Email + password
POST   /register           # Create account
POST   /logout             # Sign out
POST   /refresh            # Refresh JWT token
GET    /me                 # Current user (protected)
POST   /2fa/enable         # Two-factor auth
POST   /2fa/verify         # Verify 2FA code
```

**Paper Trading Routes** (`/api/paper-trading/`):
```
GET    /portfolio          # Portfolio summary
GET    /trades             # Trade history
POST   /execute            # Execute trade
POST   /close              # Close position
GET    /settings           # Get settings
POST   /settings           # Update settings
GET    /signals/:symbol    # Get trading signals
GET    /risk-metrics       # Risk statistics
```

**Real Trading Routes** (`/api/real-trading/`):
```
POST   /connect            # Connect Binance API
GET    /positions          # Active positions
POST   /order              # Place order
DELETE /order/:id          # Cancel order
GET    /balance            # Account balance
```

**Market Data Routes** (`/api/market/`):
```
GET    /ticker/:symbol     # Current price
GET    /candles/:symbol    # Historical data
GET    /24h-stats          # 24h change
```

**Notification Routes** (`/api/notifications/`):
```
GET    /alerts             # Alert list
POST   /alerts             # Create alert
DELETE /alerts/:id         # Delete alert
GET    /ws                 # WebSocket upgrade
```

### 7.2 Python AI Service (Port 8000)

```
POST   /predict            # Price prediction (ML)
POST   /analyze            # Market analysis (GPT-4)
POST   /sentiment          # Sentiment analysis
POST   /train              # Model training
GET    /signals            # Current signals
GET    /health             # Health status
POST   /backtest           # Strategy backtest
WS     /ws                 # WebSocket stream
```

### 7.3 Frontend (Port 3000)

Routes via React Router:
```
/                          # Home/landing
/login                     # Login page
/dashboard                 # Main dashboard
/paper-trading             # Paper trading UI
/real-trading              # Real trading UI
/ai-signals                # AI signals
/portfolio                 # Portfolio view
/settings                  # Settings page
/profile                   # User profile
/api-docs                  # API documentation
```

---

## 8. KEY TECHNOLOGY STACK

| Layer | Technology | Version |
|-------|-----------|---------|
| **Language** | Rust | 1.86+ |
| **Async Runtime** | Tokio | 1.48 |
| **Web Framework** | Actix-web/Warp | 0.4 |
| **Database** | MongoDB | 3.3+ |
| **Cache** | Redis | 5.3+ |
| **Python Runtime** | Python | 3.11+ |
| **Python Framework** | FastAPI | 0.115+ |
| **ML Frameworks** | TensorFlow/PyTorch | 2.18/2.9 |
| **Frontend** | React | 18.3 |
| **Build Tool** | Vite | 6.3 |
| **UI Library** | Shadcn/UI + TailwindCSS | 4.1 |
| **WebSocket** | Tokio-tungstenite | 0.28 |
| **Container** | Docker | 25+ |

---

## 9. DEVELOPMENT WORKFLOW

### 9.1 Local Development

```bash
# Start all services
./scripts/bot.sh start --memory-optimized

# View logs
./scripts/bot.sh logs --service rust-core-engine

# Run tests
make test                 # All tests
make test-fast            # Sequential, memory-safe

# Build
make build                # Production build
make build-fast           # Sequential build
```

### 9.2 Build Targets

```makefile
build                     # Build all services
build-fast               # Sequential build
test                     # Run all tests
lint                     # Code quality
quality-metrics          # Quality score
security-check           # Vulnerability scan
fmt                      # Format code
clean                    # Remove artifacts
```

---

## 10. CODE METRICS

| Metric | Value | Status |
|--------|-------|--------|
| **Overall Quality** | 94/100 | A |
| **Security** | 98/100 | A+ |
| **Test Coverage** | 90.4% | A+ |
| **Lines of Code** | 50,000+ | - |
| **Total Tests** | 2,202+ | - |
| **Mutation Score** | 84% | A |
| **Documentation** | 96/100 | A+ |

---

## 11. FILE COUNT SUMMARY

| Component | Files | Types |
|-----------|-------|-------|
| **Rust Core** | 70+ | .rs |
| **Python AI** | 90+ | .py |
| **React Frontend** | 120+ | .tsx/.ts |
| **Infrastructure** | 30+ | .yml/.tf/.json |
| **Tests** | 60+ | .rs/.py/.tsx |
| **Specs** | 75+ | .md |
| **Docs** | 50+ | .md |

**Total**: ~600+ source files

---

## 12. CRITICAL PATHS & ENTRY POINTS

### 12.1 Startup Flow

```
1. Rust: main.rs
   ├── Load config → config.rs
   ├── Init storage → storage/mod.rs
   ├── Init market data → market_data/processor.rs
   ├── Init trading engine → trading/engine.rs
   ├── Init paper trading → paper_trading/engine.rs
   ├── Init API server → api/mod.rs
   └── Start all components (async)

2. Python: main.py
   ├── Load config → config_loader.py
   ├── Init FastAPI app
   ├── Load models → models/model_manager.py
   ├── Connect MongoDB & Redis
   ├── Setup WebSocket manager → app/websocket/manager.py
   ├── Setup Celery → celery_app.py
   └── Start Uvicorn server

3. Frontend: main.tsx
   ├── Load React app → App.tsx
   ├── Setup contexts (Auth, WebSocket, Theme)
   ├── Connect WebSocket → hooks/useWebSocket.ts
   ├── Fetch initial data → services/api.ts
   └── Render dashboard → pages/Dashboard.tsx
```

### 12.2 Trade Execution Flow

```
Frontend (Order Form)
  ↓ POST /api/paper-trading/execute
Rust API (api/paper_trading.rs)
  ↓ Validate & risk check
Paper Trading Engine (paper_trading/engine.rs)
  ↓ Execute with slippage/fills
  ↓ Update portfolio
Database (MongoDB)
  ↓ Store trade
WebSocket Broadcaster
  ↓ Notify connected clients
Frontend (Real-time updates)
  ↓ useWebSocket hook
Dashboard (Trade updated)
```

### 12.3 AI Signal Flow

```
Market Data (Binance)
  ↓ Strategy Engine (rust-core-engine/src/strategies/)
  ↓ Generate signal
Rust API (calls Python)
  ↓ POST /analyze
Python AI Service (main.py)
  ↓ GPT-4 + ML models
  ↓ Generate analysis
Rust API (receives response)
  ↓ Store in MongoDB
  ↓ WebSocket broadcast
Frontend (useAIAnalysis hook)
  ↓ Display signal
Dashboard (AISignals component)
```

---

## 13. KEY UNRESOLVED QUESTIONS / NOTES

- Real trading is **DISABLED by default** (requires explicit config + API keys)
- Paper trading uses **configurable risk limits** (daily loss, cooldown, leverage)
- AI service integrates with **OpenAI GPT-4** (cost tracking implemented)
- WebSocket connection status monitored on frontend (auto-reconnect enabled)
- Database uses **MongoDB Atlas** connection string in .env
- Redis used for **cache** (AI analysis, settings, rate limiting)
- Tests: **2,202+ tests** across all services (90.4% coverage)
- CI/CD: **7 GitHub Actions workflows** for quality gates

---

## 14. QUICK REFERENCE

### Service Ports
- Frontend: **3000** (Vite dev server)
- Rust API: **8080** (Actix-web)
- Python AI: **8000** (FastAPI/Uvicorn)
- MongoDB: **27017** (local dev)
- Redis: **6379** (local dev)

### Key Configuration Files
- **Rust**: `rust-core-engine/config.toml`
- **Python**: `python-ai-service/config/config.yaml`
- **Frontend**: `nextjs-ui-dashboard/.env`
- **All**: Root `.env` file

### Important Directories
- Specs: `/specs/` (75 documents, requirements)
- Docs: `/docs/` (operational guides)
- Infrastructure: `/infrastructure/` (Docker, K8s, Terraform)
- Scripts: `/scripts/` (orchestration)

---

**Report Generated**: 2026-02-06
**Scout Task**: Complete codebase structure mapping
**Status**: COMPREHENSIVE - Ready for development

