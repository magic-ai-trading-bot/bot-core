# Dashboard UI - Functional Requirements

**Spec ID**: FR-DASHBOARD-001
**Version**: 1.0
**Status**: ✓ Implemented
**Owner**: Frontend Team
**Last Updated**: 2025-10-10

---

## Tasks Checklist

- [x] Requirements gathered
- [x] Design completed
- [x] Implementation done
- [ ] Tests written
- [ ] Documentation updated
- [ ] Code reviewed
- [ ] Deployed to staging
- [ ] Deployed to production

---

## Metadata

**Related Specs**:
- Related FR: [FR-API-001](../../../02-design/2.1-api-design/API_SPEC.md)
- Related Design: [ARCH-FRONTEND-001](../../../02-design/2.2-architecture/ARCHITECTURE.md)
- Related Data: [DATA_MODELS.md](../../../02-design/2.3-data-models/DATA_MODELS.md)

**Dependencies**:
- Depends on: FR-API-001 (Rust Core Engine API), FR-PYTHON-AI-001 (AI Service API)
- Blocks: None

**Business Value**: High
**Technical Complexity**: High
**Priority**: ☑ Critical

---

## Overview

This specification covers the complete functional requirements for the Next.js-based Trading Bot Dashboard UI, including real-time trading charts, AI analysis displays, paper trading management, authentication flows, WebSocket integration, chatbot interface, and responsive design patterns.

---

## Business Context

**Problem Statement**:
Users need a comprehensive, real-time dashboard to monitor trading bot performance, analyze AI-generated signals, manage paper trading portfolios, configure trading strategies, and interact with the system through an intuitive interface.

**Business Goals**:
- Provide real-time visibility into trading bot operations
- Enable users to configure and control trading strategies
- Visualize AI-powered trading signals and market analysis
- Support paper trading for risk-free strategy testing
- Deliver responsive, mobile-friendly user experience
- Ensure secure authentication and authorization

**Success Metrics**:
- Dashboard load time: < 2 seconds
- Real-time update latency: < 500ms via WebSocket
- User engagement: > 80% of features used
- Mobile responsiveness: 100% feature parity
- WebSocket uptime: > 99.5%

---

## Functional Requirements

### FR-DASHBOARD-001: Dashboard Home Page

**Priority**: ☑ Critical
**Status**: ☑ Completed
**Code Tags**: `@spec:FR-DASHBOARD-001`

**Description**:
Display comprehensive overview of trading bot status, portfolio metrics, trading charts, AI signals, and performance analytics on the main dashboard page.

**Implementation**:
- File: `nextjs-ui-dashboard/src/pages/Dashboard.tsx` (lines 17-60)
- Components:
  - `DashboardHeader` - Header with navigation and user controls
  - `BotStatus` - Bot operational status and metrics
  - `TradingCharts` - Real-time candlestick charts (lazy loaded)
  - `AIStrategySelector` - Strategy configuration UI
  - `AISignals` - Real-time AI trading signals
  - `PerformanceChart` - Portfolio performance visualization (lazy loaded)
  - `TransactionHistory` - Trade history table
  - `ChatBot` - AI assistant widget (lazy loaded)

**Acceptance Criteria**:
- [x] Dashboard displays bot status (running/stopped/error)
- [x] Portfolio overview shows current balance, equity, margin used, free margin
- [x] Real-time P&L updates with color coding (green for profit, red for loss)
- [x] Total trades counter with breakdown (open/closed)
- [x] Win rate percentage calculation
- [x] Lazy loading for heavy components (TradingCharts, PerformanceChart, ChatBot)
- [x] Loading skeleton states for async data
- [x] Responsive grid layout (1 column mobile, 2-4 columns desktop)
- [x] Auto-refresh via WebSocket connection
- [x] Error boundary handling

**Dependencies**: FR-API-001 (Rust Core API), FR-WEBSOCKET-001
**Test Cases**: TC-DASH-001, TC-DASH-002

---

### FR-DASHBOARD-002: Real-Time Trading Charts

**Priority**: ☑ Critical
**Status**: ☑ Completed
**Code Tags**: `@spec:FR-DASHBOARD-002`

**Description**:
Display real-time candlestick charts for multiple cryptocurrency trading pairs with WebSocket price updates, technical indicators, and interactive features.

**Implementation**:
- File: `nextjs-ui-dashboard/src/components/dashboard/TradingCharts.tsx` (lines 518-821)
- Custom Candlestick Chart: Lines 69-254
- Chart Card Component: Lines 260-411
- Add Symbol Dialog: Lines 415-516

**Features**:
- Candlestick visualization using DOM elements (custom implementation)
- Real-time price updates via WebSocket (MarketData and ChartUpdate messages)
- Multiple timeframe support: 1m, 5m, 15m, 1h, 4h, 1d
- Symbol management (add/remove trading pairs)
- 24h price change percentage with trend indicators
- Volume display
- Latest candle OHLC data display
- Interactive hover tooltips
- Connection status indicator (LIVE/DISCONNECTED)

**Acceptance Criteria**:
- [x] Candlestick chart renders with OHLC data
- [x] Bullish candles (close ≥ open) rendered in green, bearish in red
- [x] Real-time price updates via WebSocket
- [x] Timeframe selector (1m, 5m, 15m, 1h, 4h, 1d)
- [x] Add/remove symbols dynamically
- [x] Hover tooltip shows timestamp, OHLC values
- [x] Price scaling with dynamic range calculation
- [x] Grid lines for price reference
- [x] WebSocket connection status badge
- [x] Hot reload support (no HTTP polling)
- [x] Displays last 15 candles for each symbol
- [x] 24h price change with percentage and trend icon
- [x] MongoDB data source badge
- [x] Remove symbol button with confirmation
- [x] Responsive grid (1-4 columns based on screen size)

**Data Flow**:
1. Initial load: HTTP GET `/api/charts/{symbol}?timeframe={tf}&limit=100`
2. Real-time updates: WebSocket messages `MarketData` and `ChartUpdate`
3. Add symbol: HTTP POST `/api/symbols/add`
4. Remove symbol: HTTP DELETE `/api/symbols/remove`

**Dependencies**: FR-API-001, FR-WEBSOCKET-001, FR-MONGODB-001
**Test Cases**: TC-CHART-001, TC-CHART-002, TC-CHART-003

---

### FR-DASHBOARD-003: Trading Strategy Settings

**Priority**: ☑ High
**Status**: ☑ Completed
**Code Tags**: `@spec:FR-DASHBOARD-003`

**Description**:
Comprehensive trading strategy configuration UI with market presets, individual strategy parameters, risk management settings, and engine configuration.

**Implementation**:
- File: `nextjs-ui-dashboard/src/components/dashboard/TradingSettings.tsx` (lines 277-1103)
- Market Presets: Lines 99-275
- Strategy Configuration: Lines 451-802
- Risk Management: Lines 804-939
- Engine Settings: Lines 941-1071

**Strategy Components**:
1. **RSI Strategy**:
   - Enable/disable toggle
   - Period slider (5-30, default 14)
   - Oversold threshold slider (20-50, default 30)
   - Overbought threshold slider (50-80, default 70)
   - Extreme oversold/overbought thresholds

2. **MACD Strategy**:
   - Enable/disable toggle
   - Fast period slider (5-20, default 12)
   - Slow period slider (15-35, default 26)
   - Signal period slider (3-15, default 9)
   - Histogram threshold input

3. **Volume Strategy**:
   - Enable/disable toggle
   - Volume spike threshold slider (1.0-5.0x, default 2.0x)
   - SMA period slider (10-30, default 20)
   - Correlation period input

4. **Bollinger Bands Strategy**:
   - Enable/disable toggle
   - Period slider (10-30, default 20)
   - Multiplier slider (1.0-3.0, default 2.0)
   - Squeeze threshold input

**Market Presets**:
- **Low Volatility**: Lower thresholds, faster MACD, lower confidence (45%)
- **Normal Volatility**: Balanced settings, standard parameters
- **High Volatility**: Higher thresholds, conservative risk, higher confidence (75%)

**Risk Management**:
- Max risk per trade (0.5-5.0%)
- Max portfolio risk (5-50%)
- Stop loss percentage (0.5-5.0%)
- Take profit percentage (1.0-10.0%)
- Max leverage (1-50x)
- Max drawdown (5-25%)
- Max consecutive losses (2-10)

**Engine Settings**:
- Min confidence threshold (0.3-0.9, default 0.65)
- Signal combination mode (WeightedAverage, Consensus, BestConfidence, Conservative)
- Market condition (Trending, Ranging, Volatile, LowVolume)
- Risk level (Conservative, Moderate, Aggressive)

**Acceptance Criteria**:
- [x] Dialog-based settings interface
- [x] Tabbed navigation (Presets, Strategies, Risk, Engine)
- [x] Market preset cards with one-click apply
- [x] Individual strategy enable/disable switches
- [x] Slider controls with live value display
- [x] Settings persistence via API
- [x] Load current settings on dialog open
- [x] Save button with loading state
- [x] Reload button to refresh from backend
- [x] Preset selection visual indicator (ring on selected)
- [x] Preset preview (confidence, risk, stop loss)
- [x] Tooltips explaining parameters
- [x] Validation for numeric inputs
- [x] Toast notifications for save success/error
- [x] Low volatility recommendations info box

**API Endpoints**:
- GET `/api/paper-trading/strategy-settings` - Load current settings
- PUT `/api/paper-trading/strategy-settings` - Save settings

**Dependencies**: FR-API-001
**Test Cases**: TC-SETTINGS-001, TC-SETTINGS-002

---

### FR-DASHBOARD-004: Paper Trading Management

**Priority**: ☑ Critical
**Status**: ☑ Completed
**Code Tags**: `@spec:FR-DASHBOARD-004`

**Description**:
Complete paper trading simulation interface with portfolio management, trade execution, real-time P&L tracking, and settings configuration.

**Implementation**:
- File: `nextjs-ui-dashboard/src/pages/TradingPaper.tsx` (lines 69-2354)
- Hook: `nextjs-ui-dashboard/src/hooks/usePaperTrading.ts` (lines 165-838)

**Key Features**:

1. **Portfolio Overview** (lines 680-834):
   - Current balance display
   - Real-time equity calculation
   - Total P&L with percentage
   - Total trades counter (open + closed)
   - Win rate percentage (for closed trades)
   - Margin usage percentage
   - Free margin available
   - Real-time update indicator (pulsing dot)

2. **Performance Chart** (lines 956-1137):
   - Line chart showing balance over time
   - Simulated 24h progression (8 data points)
   - Trade event markers
   - Current status marker
   - Tooltips with balance, P&L, margin used, positions
   - WebSocket live data badge
   - Update counter

3. **Open Trades Table** (lines 1297-1502):
   - Symbol with leverage badge
   - Trade type badge (Long/Short) with color coding
   - Entry price
   - Quantity with asset symbol
   - Position size (notional value)
   - Margin required with leverage calculation
   - Leverage badge
   - Unrealized P&L with percentage
   - Stop loss price
   - Take profit price
   - Open time with formatting
   - Close button with hover effect
   - Click row to open detailed popup
   - Real-time P&L updates via WebSocket

4. **Trade Details Popup** (lines 1874-2124):
   - Trade type badge
   - Symbol display
   - Live status indicator
   - Unrealized P&L (large display)
   - Position size with leverage
   - Trade information section:
     - Symbol
     - Type (Long/Short)
     - Entry price
     - Quantity
     - Leverage
     - Position value
   - Risk management section:
     - Stop loss with percentage from entry
     - Take profit with percentage from entry
     - Open time with formatted display
     - Duration calculation
   - Action buttons:
     - Close position (with confirmation)
     - Close popup
   - Real-time updates while open
   - Auto-close when trade is closed

5. **Closed Trades History** (lines 1505-1616):
   - Symbol
   - Trade type badge
   - Entry price
   - Exit price
   - Quantity
   - Realized P&L with color
   - P&L percentage with color
   - Duration in minutes
   - Close reason badge

6. **AI Signals Display** (lines 1141-1292):
   - Signal badge (LONG/SHORT/NEUTRAL) with color
   - Symbol display
   - Active status (< 30 minutes)
   - WebSocket source badge
   - Confidence percentage with color
   - Reasoning text
   - Timestamp
   - Confidence bar visualization
   - Live Analysis badge

7. **Settings Tab** (lines 1619-1843):
   - Initial balance input (USDT)
   - Max leverage slider
   - Position size percentage
   - Default stop loss percentage
   - Default take profit percentage
   - Trading fee rate
   - Save settings button
   - Reset portfolio button with confirmation
   - Symbol-specific settings dialog
   - Trading strategy settings (embedded TradingSettings component)

8. **Symbol Configuration** (lines 2127-2345):
   - Per-symbol enable/disable
   - Leverage configuration (1-50x)
   - Position size percentage (0.1-100%)
   - Max positions per symbol (1-10)
   - Stop loss percentage (0.1-50%)
   - Take profit percentage (0.1-100%)
   - Load/save symbol settings
   - Bulk update all symbols

9. **Control Panel** (lines 549-642):
   - Real-time status indicators:
     - WebSocket connected/disconnected with icon
     - Current time (HH:MM:SS)
     - Last update timestamp
   - Control buttons:
     - Start/Stop bot toggle with status badge
     - Refresh button
   - Status badge (active/paused) with animation

10. **WebSocket Integration**:
    - Real-time price updates (MarketData messages)
    - Portfolio metrics updates (performance_update messages)
    - Trade execution notifications (trade_executed messages)
    - Trade closure updates (trade_closed messages)
    - AI signal streaming (AISignalReceived messages)
    - Connection status monitoring
    - Automatic P&L recalculation on price changes
    - Heartbeat messages every 30 seconds

**Acceptance Criteria**:
- [x] Portfolio overview with all key metrics
- [x] Real-time balance and P&L updates
- [x] Open trades table with all details
- [x] Close trade functionality
- [x] Trade details popup with live updates
- [x] Closed trades history
- [x] AI signals display with confidence levels
- [x] Performance chart with historical data
- [x] Settings configuration UI
- [x] Symbol-specific configuration
- [x] Start/stop bot controls
- [x] Reset portfolio with confirmation
- [x] WebSocket status indicator
- [x] Real-time clock display
- [x] Last update timestamp
- [x] Toast notifications for actions
- [x] Error handling and display
- [x] Loading states for async operations
- [x] Responsive layout (mobile, tablet, desktop)
- [x] Vietnamese language UI
- [x] Currency formatting (VND locale, USD currency)
- [x] Date/time formatting (vi-VN locale)
- [x] Position size and margin calculations
- [x] Win rate calculation (only for closed trades)
- [x] Margin usage percentage
- [x] Trade detail popup auto-close on trade closure

**Data Models**:
- PaperTradingSettings (basic + risk)
- PaperTrade (id, symbol, type, status, prices, P&L, timestamps)
- PortfolioMetrics (balances, P&L, metrics, margin)
- AISignal (signal, symbol, confidence, reasoning, market analysis, risk assessment)

**API Endpoints**:
- GET `/api/paper-trading/status` - Bot running status
- GET `/api/paper-trading/portfolio` - Portfolio metrics
- GET `/api/paper-trading/trades/open` - Open trades
- GET `/api/paper-trading/trades/closed` - Closed trades
- GET `/api/paper-trading/basic-settings` - Current settings
- POST `/api/paper-trading/start` - Start bot
- POST `/api/paper-trading/stop` - Stop bot
- POST `/api/paper-trading/trades/{id}/close` - Close specific trade
- PUT `/api/paper-trading/basic-settings` - Update settings
- POST `/api/paper-trading/reset` - Reset portfolio
- GET `/api/paper-trading/symbols` - Symbol-specific settings
- PUT `/api/paper-trading/symbols` - Update symbol settings
- POST `/api/ai/analyze` - Get AI analysis (for signals)

**Dependencies**: FR-API-001, FR-WEBSOCKET-001, FR-AI-001
**Test Cases**: TC-PAPER-001 to TC-PAPER-010

---

### FR-DASHBOARD-005: AI Analysis Display

**Priority**: ☑ High
**Status**: ☑ Completed
**Code Tags**: `@spec:FR-DASHBOARD-005`

**Description**:
Display AI-generated trading signals, market analysis, confidence scores, and strategy recommendations with real-time updates.

**Implementation**:
- Component: `nextjs-ui-dashboard/src/components/dashboard/AISignals.tsx`
- Hook: `nextjs-ui-dashboard/src/hooks/useAIAnalysis.ts` (lines 42-354)
- Integration in TradingPaper: Lines 1141-1292

**Signal Display Features**:
- Signal badge (LONG/SHORT/NEUTRAL) with color coding:
  - LONG: Green background (bg-profit)
  - SHORT: Red background (bg-loss)
  - NEUTRAL: Yellow background (bg-warning)
- Symbol display with USDT pair formatting
- Active status indicator (< 30 minutes old)
- WebSocket source badge
- Confidence percentage display:
  - ≥ 80%: Green (high confidence)
  - 60-79%: Yellow (medium confidence)
  - < 60%: Red (low confidence)
- Reasoning text explanation
- Timestamp with Vietnamese locale
- Confidence indicator dot
- Confidence progress bar visualization

**AI Analysis Components**:
1. **Signal Data**:
   - Signal type (long/short/neutral)
   - Symbol (e.g., BTCUSDT)
   - Confidence score (0.0-1.0)
   - Timestamp
   - Reasoning text
   - Strategy scores (RSI, MACD, Volume, Bollinger)

2. **Market Analysis**:
   - Trend direction
   - Trend strength
   - Support levels (array)
   - Resistance levels (array)
   - Volatility level
   - Volume analysis

3. **Risk Assessment**:
   - Overall risk (Low/Medium/High)
   - Technical risk score
   - Market risk score
   - Recommended position size
   - Stop loss suggestion
   - Take profit suggestion

**Acceptance Criteria**:
- [x] Display AI signals with all metadata
- [x] Color-coded signal badges
- [x] Confidence percentage visualization
- [x] Active/expired signal indication
- [x] WebSocket source identification
- [x] Reasoning explanation
- [x] Timestamp formatting
- [x] Confidence bar animation
- [x] Real-time signal updates via WebSocket
- [x] Signal deduplication (one per symbol)
- [x] Signal expiration (30 minutes)
- [x] Empty state message
- [x] Loading state
- [x] Error handling
- [x] Maximum 8 signals displayed
- [x] Auto-scroll to new signals

**WebSocket Messages**:
- Type: `AISignalReceived`
- Data: AISignalReceivedData with signal, confidence, timestamp, model_type, timeframe, reasoning, strategy_scores

**Dependencies**: FR-API-001, FR-WEBSOCKET-001, FR-PYTHON-AI-001
**Test Cases**: TC-AI-001, TC-AI-002

---

### FR-DASHBOARD-006: AI Chatbot Interface

**Priority**: ☑ Medium
**Status**: ☑ Completed
**Code Tags**: `@spec:FR-DASHBOARD-006`

**Description**:
Interactive AI chatbot widget for user support, trading guidance, and system assistance with conversation history and suggested questions.

**Implementation**:
- Component: `nextjs-ui-dashboard/src/components/ChatBot.tsx` (lines 57-448)
- Service: `nextjs-ui-dashboard/src/services/chatbot.ts`

**Features**:
1. **Widget Display**:
   - Floating action button (bottom-right)
   - Gradient button styling (blue-cyan gradient)
   - Expand/collapse animation
   - Minimize/maximize functionality
   - Close button
   - Sound enable/disable toggle

2. **Chat Interface**:
   - Header with bot avatar and status indicator (online/offline)
   - Message scroll area with auto-scroll
   - User/bot message differentiation:
     - User messages: Blue background, right-aligned
     - Bot messages: Gray background, left-aligned
   - Avatar icons (User/Bot)
   - Timestamp display (HH:MM format)
   - Typing indicator with animated dots

3. **Welcome Message** (lines 86-101):
   - Greeting in Vietnamese
   - Feature list (bot operation, usage guide, strategy explanation, troubleshooting)
   - Friendly tone

4. **Suggested Questions** (lines 363-387):
   - Displayed after welcome message
   - 4 preset questions
   - Click to populate input field
   - Icon indicator (HelpCircle)

5. **Message Input**:
   - Text input field
   - Send button with loading state
   - Clear chat button (trash icon)
   - Enter key to send
   - Disabled during loading

6. **Features**:
   - Message history persistence
   - Typing simulation (1-2 second delay)
   - Sound notification on bot reply
   - Confidence score display for low confidence (<70%)
   - Toast notification for low confidence responses
   - Vietnamese language support
   - Status badges (AI, Vietnamese, Online)
   - Clear chat functionality

**Acceptance Criteria**:
- [x] Floating action button toggle
- [x] Expand/collapse animation
- [x] Minimize/maximize functionality
- [x] Welcome message on first open
- [x] Suggested questions display
- [x] User message sending
- [x] Bot response with delay
- [x] Typing indicator animation
- [x] Message history display
- [x] Scroll to latest message
- [x] Avatar icons for user/bot
- [x] Timestamp formatting
- [x] Sound notification (optional)
- [x] Sound enable/disable toggle
- [x] Clear chat functionality
- [x] Enter key to send
- [x] Loading state during processing
- [x] Error handling
- [x] Confidence score notification
- [x] Vietnamese language support
- [x] Status indicator (online)

**Chatbot Service Features**:
- Message processing
- Response generation
- History management
- Suggested questions
- Confidence calculation
- Knowledge base integration

**Dependencies**: FR-API-001 (chatbot service integration)
**Test Cases**: TC-CHAT-001, TC-CHAT-002

---

### FR-DASHBOARD-007: WebSocket Real-Time Updates

**Priority**: ☑ Critical
**Status**: ☑ Completed
**Code Tags**: `@spec:FR-DASHBOARD-007`

**Description**:
Real-time data synchronization via WebSocket connection for trading prices, portfolio updates, AI signals, and bot status.

**Implementation**:
- Hook: `nextjs-ui-dashboard/src/hooks/useWebSocket.ts` (lines 128-418)
- Integration: Multiple components use WebSocket for live updates

**WebSocket Message Types**:

1. **MarketData** (lines 654-674 in TradingCharts):
   - Symbol price updates
   - 24h price change
   - 24h price change percentage
   - 24h volume
   - Timestamp
   - Used for: Real-time chart price updates, portfolio P&L recalculation

2. **ChartUpdate** (lines 677-696 in TradingCharts):
   - Symbol
   - Timeframe
   - Candle data (OHLC)
   - Latest price
   - 24h metrics
   - Timestamp
   - Used for: New candle formation, chart data refresh

3. **PositionUpdate** (lines 145-159):
   - Symbol
   - Side (LONG/SHORT)
   - Current price
   - Unrealized P&L
   - Timestamp
   - Used for: Open position P&L updates

4. **TradeExecuted** (lines 161-182):
   - Symbol
   - Side (BUY/SELL)
   - Quantity
   - Price
   - Timestamp
   - P&L (if closing trade)
   - Used for: Trade history updates, portfolio refresh

5. **AISignalReceived** (lines 184-202):
   - Symbol
   - Signal (long/short/neutral)
   - Confidence score
   - Timestamp
   - Model type
   - Timeframe
   - Reasoning
   - Strategy scores
   - Used for: Real-time AI signal display

6. **BotStatusUpdate** (lines 204-218):
   - Status (running/stopped/error)
   - Active positions count
   - Total P&L
   - Total trades
   - Uptime
   - Used for: Bot status indicator updates

7. **Connected** (line 229):
   - Connection confirmation
   - Initial handshake

8. **Pong** (line 232):
   - Keep-alive response
   - Heartbeat acknowledgment

9. **Error** (lines 256-263):
   - Error message
   - Error code
   - Error details
   - Used for: Error display, reconnection logic

**Connection Management**:
- Auto-connect on mount
- Reconnection with exponential backoff
- Max reconnection attempts: 10
- Reconnection interval: 5 seconds (with backoff)
- Keep-alive ping every 30 seconds
- Connection status tracking (isConnected, isConnecting)
- Error state management

**Features**:
- Automatic reconnection
- Connection status indicator
- Last message tracking
- State management for positions, trades, signals, bot status
- Message parsing with error handling
- Cleanup on unmount
- Manual connect/disconnect methods
- Send message functionality

**Acceptance Criteria**:
- [x] WebSocket connection establishment
- [x] Auto-reconnect on disconnect
- [x] Exponential backoff for reconnection
- [x] Connection status tracking
- [x] Message type handling (all types)
- [x] Position updates in real-time
- [x] Trade execution updates
- [x] AI signal streaming
- [x] Bot status updates
- [x] Chart price updates
- [x] Keep-alive mechanism
- [x] Error handling and display
- [x] State synchronization
- [x] Manual connect/disconnect
- [x] Send message capability
- [x] Cleanup on unmount
- [x] Visual connection indicator

**WebSocket URL**: `ws://localhost:8080/ws` (configurable via VITE_WS_URL)

**Dependencies**: FR-API-001 (WebSocket endpoint)
**Test Cases**: TC-WS-001 to TC-WS-005

---

### FR-DASHBOARD-008: Authentication & Authorization

**Priority**: ☑ Critical
**Status**: ☑ Completed
**Code Tags**: `@spec:FR-DASHBOARD-008`

**Description**:
User authentication flow with login, registration, token management, and protected route access.

**Implementation**:
- Context: `nextjs-ui-dashboard/src/contexts/AuthContext.tsx` (lines 1-142)
- Login Page: `nextjs-ui-dashboard/src/pages/Login.tsx` (lines 13-178)
- Register Page: `nextjs-ui-dashboard/src/pages/Register.tsx`
- Protected Route: `nextjs-ui-dashboard/src/components/ProtectedRoute.tsx`

**Authentication Flow**:

1. **Login** (lines 57-78 in AuthContext):
   - Email and password input validation
   - API call to `/api/auth/login`
   - JWT token storage in localStorage
   - User profile storage in state
   - Automatic redirect to dashboard on success
   - Error handling with toast notifications
   - Loading state management

2. **Register** (lines 80-109):
   - Email, password, full name input
   - API call to `/api/auth/register`
   - JWT token storage
   - User profile storage
   - Auto-login after registration
   - Error handling
   - Loading state

3. **Token Management**:
   - Token stored in localStorage
   - Token expiration check
   - Automatic token refresh (if implemented)
   - Token removal on logout
   - Token validation on app initialization

4. **Profile Management** (lines 36-55):
   - Load user profile on mount
   - Check existing token
   - Validate token not expired
   - Fetch user profile from API
   - Store user data in state
   - Clear invalid tokens

5. **Logout** (lines 111-116):
   - Remove token from localStorage
   - Clear user state
   - Clear error state
   - Redirect to login

**Login Page Features** (Login.tsx):
- Email input field
- Password input field
- Submit button with loading state
- Demo credentials display
- Register link
- Features preview (AI signals, analytics, risk management)
- Security badge (E2E encryption, 2FA)
- Background gradient pattern
- Logo/brand display
- Vietnamese language UI
- Toast notifications
- Chatbot widget

**Protected Routes**:
- Redirect to login if not authenticated
- Check authentication status
- Display loading state during auth check
- Preserve intended destination after login

**Acceptance Criteria**:
- [x] Login form with email/password
- [x] Registration form with validation
- [x] JWT token storage and retrieval
- [x] Token expiration check
- [x] User profile loading
- [x] Protected route wrapper
- [x] Automatic redirect after login
- [x] Logout functionality
- [x] Error handling and display
- [x] Loading states
- [x] Toast notifications
- [x] Demo credentials display
- [x] Register navigation link
- [x] Password field masking
- [x] Email validation
- [x] Form submission handling
- [x] Responsive layout
- [x] Brand logo display
- [x] Security features preview

**API Endpoints**:
- POST `/api/auth/login` - User login
- POST `/api/auth/register` - User registration
- GET `/api/auth/profile` - Get user profile
- POST `/api/auth/logout` - User logout (optional)

**Data Models**:
- LoginRequest: { email, password }
- RegisterRequest: { email, password, full_name? }
- UserProfile: { id, email, full_name, created_at, ... }
- AuthResponse: { token, user }

**Dependencies**: FR-API-001 (Auth endpoints)
**Test Cases**: TC-AUTH-001 to TC-AUTH-005

---

### FR-DASHBOARD-009: Responsive Design & Layout

**Priority**: ☑ High
**Status**: ☑ Completed
**Code Tags**: `@spec:FR-DASHBOARD-009`

**Description**:
Responsive, mobile-first design that adapts to different screen sizes with optimal user experience across devices.

**Implementation**:
- Utility: Tailwind CSS responsive breakpoints (sm, md, lg, xl, 2xl)
- Component: `nextjs-ui-dashboard/src/components/ui/responsive-container.tsx`
- All components use responsive classes

**Breakpoints**:
- Mobile: < 640px (default)
- Tablet: 640px - 768px (sm)
- Desktop: 768px - 1024px (md)
- Large Desktop: 1024px - 1280px (lg)
- XL Desktop: 1280px+ (xl)

**Responsive Patterns**:

1. **Grid Layouts**:
   - Portfolio metrics: `grid-cols-1 md:grid-cols-2 lg:grid-cols-4`
   - Charts: `grid-cols-1 sm:grid-cols-2 lg:grid-cols-3 xl:grid-cols-4`
   - Settings: `grid-cols-1 md:grid-cols-2 lg:grid-cols-3`

2. **Spacing**:
   - Padding: `p-4 lg:p-6` (16px mobile, 24px desktop)
   - Gap: `gap-4 lg:gap-6` (16px mobile, 24px desktop)
   - Space: `space-y-4 lg:space-y-6`

3. **Typography**:
   - Headings: `text-2xl lg:text-3xl` (24px mobile, 30px desktop)
   - Body: `text-sm lg:text-base` (14px mobile, 16px desktop)
   - Small: `text-xs` (12px all screens)

4. **Component Sizing**:
   - Cards: Full width on mobile, grid on desktop
   - Tables: Horizontal scroll on mobile, full table on desktop
   - Dialogs: `max-w-sm md:max-w-md lg:max-w-4xl`
   - Charts: `h-40 md:h-64 lg:h-96`

5. **Navigation**:
   - Mobile: Hamburger menu or bottom navigation
   - Desktop: Full header with navigation links
   - Responsive header: `flex-col sm:flex-row`

6. **Forms**:
   - Stacked inputs on mobile
   - Side-by-side on desktop
   - Full-width buttons on mobile
   - Auto-width on desktop

**Acceptance Criteria**:
- [x] Mobile-first approach
- [x] Responsive grid layouts
- [x] Adaptive spacing and typography
- [x] Touch-friendly controls (44px min)
- [x] Horizontal scroll for tables on mobile
- [x] Responsive dialogs and modals
- [x] Adaptive chart sizes
- [x] Mobile navigation pattern
- [x] Desktop navigation pattern
- [x] Responsive images and icons
- [x] Media query breakpoints
- [x] Flexible containers
- [x] Responsive padding/margins
- [x] Stack on mobile, grid on desktop
- [x] Testing on multiple devices

**Browser Support**:
- Chrome (latest 2 versions)
- Firefox (latest 2 versions)
- Safari (latest 2 versions)
- Edge (latest 2 versions)
- Mobile Safari iOS (latest 2 versions)
- Chrome Mobile Android (latest 2 versions)

**Dependencies**: Tailwind CSS
**Test Cases**: TC-RESPONSIVE-001 to TC-RESPONSIVE-005

---

### FR-DASHBOARD-010: Theme & Dark Mode

**Priority**: ☐ Medium
**Status**: ☐ Not Started
**Code Tags**: `@spec:FR-DASHBOARD-010`

**Description**:
Dark mode support with theme toggle and persistent user preference.

**Implementation** (Planned):
- Theme provider context
- Theme toggle component
- CSS variable-based theming
- localStorage persistence

**Acceptance Criteria**:
- [ ] Dark mode theme
- [ ] Light mode theme
- [ ] Theme toggle button
- [ ] Persistent theme preference
- [ ] Smooth theme transition
- [ ] All components support both themes
- [ ] Proper contrast ratios

**Dependencies**: None
**Test Cases**: TC-THEME-001, TC-THEME-002

---

### FR-DASHBOARD-011: Internationalization (i18n)

**Priority**: ☐ Medium
**Status**: ☐ Partially Implemented
**Code Tags**: `@spec:FR-DASHBOARD-011`

**Description**:
Multi-language support with language switcher and translation management.

**Implementation**:
- Component: `nextjs-ui-dashboard/src/components/LanguageSelector.tsx`
- Current: Hardcoded Vietnamese UI text in components

**Supported Languages** (Planned):
- Vietnamese (vi-VN) - Currently hardcoded
- English (en-US) - Planned

**Translation Areas**:
- UI labels and buttons
- Error messages
- Toast notifications
- Form validation messages
- Date/time formatting (already locale-aware)
- Currency formatting (already locale-aware)

**Acceptance Criteria**:
- [ ] Language selector component
- [ ] Translation files (JSON)
- [ ] Translation keys in components
- [ ] Language persistence
- [ ] Fallback language (English)
- [ ] RTL support (future)
- [ ] Number/date/currency formatting per locale
- [x] Vietnamese UI (hardcoded)
- [x] Vietnamese date formatting (vi-VN locale)
- [x] Vietnamese currency formatting (VND locale, USD currency)

**Dependencies**: react-i18next or similar
**Test Cases**: TC-I18N-001, TC-I18N-002

---

### FR-DASHBOARD-012: Error Handling & User Feedback

**Priority**: ☑ High
**Status**: ☑ Completed
**Code Tags**: `@spec:FR-DASHBOARD-012`

**Description**:
Comprehensive error handling, user notifications, and feedback mechanisms across all dashboard features.

**Implementation**:
- Toast Library: Sonner (`nextjs-ui-dashboard/src/components/ui/sonner.tsx`)
- Logger: `nextjs-ui-dashboard/src/utils/logger.ts`
- Error States: Component-level error state management

**Error Handling Patterns**:

1. **API Errors**:
   - Try-catch blocks around all API calls
   - Error logging via logger utility
   - Toast error notifications
   - Error state display in UI
   - Retry mechanisms where appropriate

2. **WebSocket Errors**:
   - Connection error handling
   - Reconnection logic
   - Connection status display
   - Error message parsing
   - Fallback to HTTP polling (optional)

3. **Form Validation**:
   - Client-side validation
   - Required field checks
   - Format validation (email, numbers)
   - Error message display
   - Inline error indicators

4. **Loading States**:
   - Loading spinners/skeletons
   - Disabled buttons during processing
   - Loading indicators for async operations
   - Shimmer effects for content loading

**Toast Notifications**:

1. **Success Notifications**:
   - Green color scheme
   - Success icon
   - Short duration (3-4 seconds)
   - Examples:
     - "Đăng nhập thành công"
     - "Cài đặt đã được lưu thành công"
     - "Bot trading đã được khởi động"

2. **Error Notifications**:
   - Red color scheme
   - Error icon
   - Longer duration (4-5 seconds)
   - Error description
   - Examples:
     - "Lỗi đăng nhập"
     - "Lỗi khi lưu cài đặt"
     - "Không thể tải dữ liệu"

3. **Info Notifications**:
   - Blue color scheme
   - Info icon
   - Medium duration (3 seconds)
   - Examples:
     - "Giao dịch đã được đóng"
     - "Portfolio đã được reset"

4. **Loading Notifications**:
   - Loading spinner
   - Persistent (until action completes)
   - ID for dismissal
   - Example: "Đang đăng nhập..."

**Logger Utility**:
- Console logging wrapper
- Log levels: info, warn, error, debug
- Structured logging format
- Production log filtering
- Error stack traces

**Acceptance Criteria**:
- [x] Toast notifications for all user actions
- [x] Error logging for debugging
- [x] Loading states for async operations
- [x] Error state display in components
- [x] Retry mechanisms for failed requests
- [x] Connection error handling
- [x] Form validation with error messages
- [x] Success confirmations
- [x] User-friendly error messages
- [x] Vietnamese error messages
- [x] Consistent error handling patterns
- [x] Error boundary (React)
- [x] Logger utility integration

**Dependencies**: Sonner toast library
**Test Cases**: TC-ERROR-001 to TC-ERROR-005

---

### FR-DASHBOARD-013: Performance Optimization

**Priority**: ☑ High
**Status**: ☑ Completed
**Code Tags**: `@spec:FR-DASHBOARD-013`

**Description**:
Performance optimizations including lazy loading, code splitting, memoization, and efficient rendering.

**Implementation**:

1. **Lazy Loading** (Dashboard.tsx lines 8-11):
   - TradingCharts component lazy loaded
   - PerformanceChart component lazy loaded
   - ChatBot component lazy loaded
   - Loading fallback (skeleton)

2. **Code Splitting**:
   - Dynamic imports for heavy components
   - Route-based splitting
   - Vendor chunk splitting (Vite config)

3. **React Optimization**:
   - React.memo for ChartCard component
   - useCallback for event handlers
   - useMemo for expensive calculations
   - Avoid unnecessary re-renders

4. **WebSocket Optimization**:
   - Disabled HTTP polling (replaced with WebSocket)
   - Real-time updates only when data changes
   - Debounced updates for high-frequency data
   - Conditional rendering based on connection status

5. **Data Optimization**:
   - Signal deduplication (one per symbol)
   - Limited history (last 20 signals/trades)
   - Efficient state updates
   - Minimal re-renders

6. **Asset Optimization**:
   - Vite build optimization
   - Tree shaking
   - CSS purging (Tailwind)
   - Image optimization (future)

**Performance Metrics**:
- Initial load: < 2 seconds
- Time to interactive: < 3 seconds
- WebSocket latency: < 500ms
- Component render: < 16ms (60fps)
- Memory usage: < 100MB (excluding charts)

**Acceptance Criteria**:
- [x] Lazy loaded components
- [x] Code splitting implemented
- [x] React.memo for expensive components
- [x] useCallback for callbacks
- [x] useMemo for calculations
- [x] WebSocket replaces polling
- [x] Efficient state management
- [x] Data deduplication
- [x] Limited history storage
- [x] Build optimization (Vite)
- [x] Tree shaking enabled
- [x] CSS purging (Tailwind)
- [ ] Performance monitoring
- [ ] Lighthouse score > 90

**Dependencies**: Vite, React optimization hooks
**Test Cases**: TC-PERF-001 to TC-PERF-005

---

### FR-DASHBOARD-014: Data Visualization Components

**Priority**: ☑ High
**Status**: ☑ Completed
**Code Tags**: `@spec:FR-DASHBOARD-014`

**Description**:
Comprehensive data visualization using charts, graphs, tables, and custom visualizations.

**Implementation**:

1. **Candlestick Chart** (TradingCharts.tsx lines 69-254):
   - Custom DOM-based implementation
   - OHLC data visualization
   - Bullish/bearish color coding
   - Wick and body rendering
   - Price scaling
   - Hover tooltips
   - Grid lines
   - Time labels

2. **Line Chart** (TradingPaper.tsx lines 996-1122):
   - Recharts library
   - Balance over time
   - P&L visualization
   - Trade event markers
   - Custom tooltips
   - Responsive container
   - Smooth animations

3. **Tables**:
   - Open trades table
   - Closed trades history
   - Shadcn/UI table components
   - Sortable columns (future)
   - Pagination (future)
   - Row actions
   - Responsive overflow

4. **Progress Bars**:
   - Confidence level bars
   - Loading indicators
   - Percentage displays

5. **Badges & Icons**:
   - Status badges
   - Signal type badges
   - Color-coded indicators
   - Icon indicators (Lucide icons)

6. **Cards & Metrics**:
   - Portfolio metric cards
   - Animated value changes
   - Real-time pulse effects
   - Color-coded values

**Chart Library**:
- Recharts for standard charts (Line, Area, Bar)
- Custom implementation for Candlestick
- Responsive charts
- Interactive tooltips
- Theme support

**Acceptance Criteria**:
- [x] Candlestick chart implementation
- [x] Line chart for performance
- [x] Data tables with actions
- [x] Progress bars
- [x] Badge components
- [x] Icon integration
- [x] Metric cards
- [x] Responsive charts
- [x] Interactive tooltips
- [x] Color-coded visualizations
- [x] Animation effects
- [x] Real-time data updates

**Dependencies**: Recharts, Lucide React icons
**Test Cases**: TC-VISUAL-001 to TC-VISUAL-005

---

### FR-DASHBOARD-015: Navigation & Routing

**Priority**: ☑ Critical
**Status**: ☑ Completed
**Code Tags**: `@spec:FR-DASHBOARD-015`

**Description**:
Client-side routing with protected routes, navigation components, and page transitions.

**Implementation**:
- Router: React Router (react-router-dom)
- Routes: `nextjs-ui-dashboard/src/App.tsx`
- Header: `nextjs-ui-dashboard/src/components/dashboard/DashboardHeader.tsx`

**Routes**:
- `/` - Landing page (Index.tsx)
- `/login` - Login page
- `/register` - Registration page
- `/dashboard` - Main dashboard (protected)
- `/trading-paper` - Paper trading page (protected)
- `/settings` - Settings page (protected)
- `*` - 404 Not Found page

**Navigation Components**:

1. **Dashboard Header**:
   - Logo/brand
   - Navigation links
   - User profile dropdown
   - Logout button
   - Theme toggle (future)
   - Language selector
   - Responsive menu

2. **Protected Route**:
   - Authentication check
   - Redirect to login
   - Loading state
   - Preserve intended route

3. **Navigation Patterns**:
   - Link component (no page reload)
   - useNavigate hook for programmatic navigation
   - Redirect after login/logout
   - Back button support
   - Browser history management

**Acceptance Criteria**:
- [x] Client-side routing
- [x] Protected routes
- [x] Public routes (login, register)
- [x] Navigation header
- [x] Navigation links
- [x] User menu/dropdown
- [x] Logout functionality
- [x] Redirect after auth
- [x] 404 page
- [x] Browser back/forward support
- [x] Programmatic navigation
- [x] Responsive navigation menu

**Dependencies**: React Router
**Test Cases**: TC-NAV-001 to TC-NAV-005

---

## Use Cases

### UC-DASH-001: View Real-Time Trading Performance

**Actor**: Authenticated User
**Preconditions**:
- User is logged in
- Trading bot is running
- WebSocket connection established

**Main Flow**:
1. User navigates to Dashboard page
2. System loads portfolio metrics from API
3. System establishes WebSocket connection
4. System displays current balance, P&L, positions
5. System receives MarketData updates via WebSocket
6. System recalculates P&L in real-time
7. User views updated metrics without page refresh

**Alternative Flows**:
- **Alt 1**: WebSocket disconnected
  1. System shows disconnection warning
  2. System attempts reconnection
  3. System falls back to periodic HTTP polling (if enabled)
- **Alt 2**: Bot not running
  1. System displays "Bot Stopped" status
  2. User can start bot via control panel

**Postconditions**:
- Dashboard displays accurate, real-time data
- User has visibility into trading performance

**Exception Handling**:
- API Error: Display error toast, retry after 5 seconds
- WebSocket Error: Show connection status, auto-reconnect

---

### UC-DASH-002: Configure Trading Strategy

**Actor**: Authenticated User
**Preconditions**:
- User is logged in
- User has access to settings

**Main Flow**:
1. User navigates to TradingPaper page
2. User clicks Settings tab
3. System loads current settings from API
4. User opens Trading Strategy Settings dialog
5. User selects market preset (Low/Normal/High Volatility)
6. System applies preset to all strategy parameters
7. User fine-tunes individual strategy settings (RSI, MACD, etc.)
8. User adjusts risk management parameters
9. User clicks Save Settings
10. System validates input values
11. System sends PUT request to API
12. System displays success toast
13. System refreshes portfolio with new settings

**Alternative Flows**:
- **Alt 1**: Validation error
  1. System highlights invalid field
  2. System shows error message
  3. User corrects input
  4. System re-validates
- **Alt 2**: Save error
  1. System displays error toast
  2. User can retry or cancel

**Postconditions**:
- Strategy settings updated in backend
- New trades use updated parameters

**Exception Handling**:
- API Error: Display error toast with retry option
- Network Error: Queue settings for retry

---

### UC-DASH-003: Monitor AI Trading Signals

**Actor**: Authenticated User
**Preconditions**:
- User is logged in
- AI service is running
- WebSocket connection active

**Main Flow**:
1. User navigates to TradingPaper page
2. User clicks AI Signals tab
3. System displays recent AI signals
4. System receives AISignalReceived WebSocket message
5. System parses signal data (symbol, type, confidence)
6. System deduplicates signals (one per symbol)
7. System displays new signal at top of list
8. User views signal details (reasoning, confidence, timestamp)
9. User sees confidence bar visualization
10. System marks signals older than 30 minutes as expired

**Alternative Flows**:
- **Alt 1**: Low confidence signal (<70%)
  1. System displays signal with yellow/red indicator
  2. System shows toast notification about low confidence
  3. User reviews signal with caution
- **Alt 2**: No signals available
  1. System displays empty state message
  2. User can refresh signals manually

**Postconditions**:
- User has visibility into AI-generated trading signals
- Signals are sorted by recency

**Exception Handling**:
- WebSocket Error: Signals remain static until reconnection
- AI Service Error: Display error message, allow manual refresh

---

### UC-DASH-004: Close Open Position

**Actor**: Authenticated User
**Preconditions**:
- User is logged in
- At least one position is open
- Paper trading mode active

**Main Flow**:
1. User navigates to TradingPaper page
2. User clicks Trades tab
3. System displays open trades table
4. User clicks on a trade row
5. System opens trade details popup
6. System displays position details, P&L, margin
7. User clicks "Close Position" button
8. System shows confirmation toast
9. System sends POST request to close trade
10. System receives trade_closed WebSocket message
11. System updates open trades list (removes trade)
12. System updates closed trades list (adds trade)
13. System recalculates portfolio metrics
14. System auto-closes popup
15. User sees updated portfolio and trade history

**Alternative Flows**:
- **Alt 1**: User cancels close
  1. User clicks "Close popup" button
  2. System keeps position open
- **Alt 2**: Close fails
  1. System displays error toast
  2. Position remains open
  3. User can retry

**Postconditions**:
- Position is closed
- Portfolio metrics updated
- Trade appears in closed trades history

**Exception Handling**:
- API Error: Display error, keep position open
- Network Error: Retry with exponential backoff

---

### UC-DASH-005: Interact with AI Chatbot

**Actor**: Authenticated User
**Preconditions**:
- User is on any dashboard page

**Main Flow**:
1. User clicks chatbot floating action button
2. System expands chatbot widget
3. System displays welcome message and suggested questions
4. User types question or clicks suggested question
5. User presses Enter or clicks Send button
6. System shows typing indicator
7. System sends message to chatbot service
8. System receives bot response
9. System displays bot message with timestamp
10. System plays notification sound (if enabled)
11. User views response
12. User can continue conversation

**Alternative Flows**:
- **Alt 1**: Low confidence response
  1. System displays response
  2. System shows toast with confidence percentage
  3. System suggests refining question
- **Alt 2**: User clears chat
  1. User clicks clear chat button
  2. System removes all messages except welcome
  3. User can start fresh conversation

**Postconditions**:
- User receives guidance or information
- Chat history maintained during session

**Exception Handling**:
- Chatbot Error: Display error message, allow retry
- Network Error: Show connection error, queue message

---

## Data Requirements

**Input Data**:
- User credentials: email (string, required, email format), password (string, required, min 8 chars)
- Trading settings: initial_balance (number, required, > 0), leverage (number, 1-50), position_size (number, 0.1-100)
- Symbol configuration: symbol (string, required, format: {BASE}USDT), timeframe (string, enum: 1m/5m/15m/1h/4h/1d)
- Strategy parameters: All numeric with validation ranges
- Trade actions: trade_id (string, required, UUID format), action (string, enum: close/modify)

**Output Data**:
- Portfolio metrics: balance, equity, P&L, margin (all numbers, 2 decimal precision)
- Trade data: All fields from PaperTrade model
- AI signals: All fields from AISignal model
- Chart data: OHLC candles with timestamp, price, volume

**Data Validation**:
- Email: RFC 5322 format
- Password: Min 8 characters, alphanumeric
- Numbers: Range validation, non-negative where applicable
- Percentages: 0-100 range
- Leverage: 1-50 integer
- Symbols: Uppercase, ends with USDT

**Data Models** (reference to DATA_MODELS.md):
- UserProfile
- PaperTradingSettings
- PaperTrade
- PortfolioMetrics
- AISignal
- ChartData

---

## Interface Requirements

**API Endpoints** (reference to API_SPEC.md):
```
# Authentication
POST /api/auth/login
POST /api/auth/register
GET /api/auth/profile

# Paper Trading
GET /api/paper-trading/status
GET /api/paper-trading/portfolio
GET /api/paper-trading/trades/open
GET /api/paper-trading/trades/closed
POST /api/paper-trading/start
POST /api/paper-trading/stop
POST /api/paper-trading/reset
POST /api/paper-trading/trades/{id}/close
GET /api/paper-trading/basic-settings
PUT /api/paper-trading/basic-settings
GET /api/paper-trading/symbols
PUT /api/paper-trading/symbols
GET /api/paper-trading/strategy-settings
PUT /api/paper-trading/strategy-settings

# Charts
GET /api/charts/{symbol}?timeframe={tf}&limit={n}
GET /api/charts/prices
POST /api/symbols/add
DELETE /api/symbols/remove
GET /api/symbols/supported

# AI
POST /api/ai/analyze
GET /api/ai/service-info
GET /api/ai/strategies

# WebSocket
WS /ws
```

**UI Screens**:
- Login Screen: Authentication form
- Register Screen: User registration
- Dashboard: Main overview page
- TradingPaper: Paper trading management
- Settings: Configuration page
- NotFound: 404 error page

**External Systems** (reference to INTEGRATION_SPEC.md):
- Rust Core Engine: Port 8080
- Python AI Service: Port 8000
- MongoDB: Trading data persistence
- Binance WebSocket: Market data stream

---

## Non-Functional Requirements

**Performance**:
- Page load time: < 2 seconds
- Time to interactive: < 3 seconds
- WebSocket message latency: < 500ms
- Chart render time: < 100ms
- API response time: < 1 second (95th percentile)

**Security**:
- Authentication: JWT tokens
- Authorization: Protected routes
- Data encryption: HTTPS in production
- Token expiration: 24 hours
- XSS protection: React built-in
- CSRF protection: Token-based

**Scalability**:
- Horizontal scaling: Stateless frontend
- CDN support: Static asset hosting
- WebSocket scaling: Multiple connections per server
- Caching: Service worker (future)

**Reliability**:
- Uptime target: 99.9%
- Error rate: < 0.1%
- WebSocket reconnection: Automatic with backoff
- API retry logic: 3 attempts with exponential backoff

**Maintainability**:
- Code coverage: Target 80%
- Component library: Shadcn/UI
- Linting: ESLint
- Type checking: TypeScript strict mode
- Documentation: JSDoc comments

**Usability**:
- Mobile responsive: 100% feature parity
- Touch targets: Min 44x44px
- Contrast ratio: WCAG AA compliance
- Keyboard navigation: Full support
- Screen reader: ARIA labels

**Browser Compatibility**:
- Chrome: Latest 2 versions
- Firefox: Latest 2 versions
- Safari: Latest 2 versions
- Edge: Latest 2 versions
- Mobile browsers: Latest 2 versions

---

## Implementation Notes

**Code Locations**:
- Dashboard: `nextjs-ui-dashboard/src/pages/Dashboard.tsx`
- Trading Paper: `nextjs-ui-dashboard/src/pages/TradingPaper.tsx`
- Trading Charts: `nextjs-ui-dashboard/src/components/dashboard/TradingCharts.tsx`
- Trading Settings: `nextjs-ui-dashboard/src/components/dashboard/TradingSettings.tsx`
- Chatbot: `nextjs-ui-dashboard/src/components/ChatBot.tsx`
- Auth Context: `nextjs-ui-dashboard/src/contexts/AuthContext.tsx`
- WebSocket Hook: `nextjs-ui-dashboard/src/hooks/useWebSocket.ts`
- Paper Trading Hook: `nextjs-ui-dashboard/src/hooks/usePaperTrading.ts`
- AI Analysis Hook: `nextjs-ui-dashboard/src/hooks/useAIAnalysis.ts`
- Login Page: `nextjs-ui-dashboard/src/pages/Login.tsx`
- Register Page: `nextjs-ui-dashboard/src/pages/Register.tsx`
- Logger: `nextjs-ui-dashboard/src/utils/logger.ts`
- API Client: `nextjs-ui-dashboard/src/services/api.ts`
- Chatbot Service: `nextjs-ui-dashboard/src/services/chatbot.ts`

**Dependencies**:
- React: 18.3.1
- React Router DOM: 6.x
- Vite: 6.0.1
- Tailwind CSS: 3.x
- Shadcn/UI: Latest
- Recharts: 2.x
- Sonner: 1.x (toast notifications)
- Lucide React: Latest (icons)
- TypeScript: 5.6.2

**Design Patterns**:
- Custom Hooks: Encapsulate logic (useWebSocket, usePaperTrading, useAIAnalysis)
- Context API: Global state (AuthContext)
- Component Composition: Reusable UI components
- Lazy Loading: Code splitting for performance
- Memoization: React.memo, useCallback, useMemo for optimization
- Error Boundaries: React error boundaries for crash recovery

**Configuration**:
- Vite config: `nextjs-ui-dashboard/vite.config.ts`
- Environment variables:
  - `VITE_RUST_API_URL`: Rust backend URL (default: http://localhost:8080)
  - `VITE_WS_URL`: WebSocket URL (default: ws://localhost:8080/ws)
  - `VITE_ENABLE_REALTIME`: Enable WebSocket (default: true)
- Tailwind config: `nextjs-ui-dashboard/tailwind.config.js`
- ESLint config: `nextjs-ui-dashboard/eslint.config.js`

---

## Testing Strategy

**Unit Tests**:
- Test location: `nextjs-ui-dashboard/src/test/`
- Coverage target: 80%
- Key test scenarios:
  1. Component rendering
  2. Hook state management
  3. Utility functions
  4. WebSocket message handling
  5. Authentication flow

**Integration Tests**:
- Test suite: `nextjs-ui-dashboard/src/test/integration/`
- Integration points tested:
  1. API client with backend
  2. WebSocket connection
  3. Auth flow with token storage
  4. Chart data fetching
  5. Paper trading operations

**E2E Tests**:
- Test framework: Playwright (configured)
- Test location: `nextjs-ui-dashboard/e2e/`
- User flows tested:
  1. Login → Dashboard → View Metrics
  2. Configure Strategy → Save → Verify Applied
  3. View Charts → Add Symbol → Remove Symbol
  4. Paper Trading → Open Trade → Close Trade
  5. Chatbot → Ask Question → Receive Response

**Performance Tests**:
- Load test: Lighthouse CI
- Metrics: FCP, LCP, TTI, CLS
- Target: Score > 90

**Security Tests**:
- Authentication: Token validation
- Authorization: Protected route access
- XSS: Input sanitization
- CSRF: Token verification

---

## Deployment

**Environment Requirements**:
- Node.js: 18.x or higher
- npm: 9.x or higher
- Build tool: Vite
- Deployment: Static hosting (Vercel, Netlify, AWS S3+CloudFront)

**Configuration Changes**:
- Environment variables in `.env` file
- API URLs configured per environment
- WebSocket URLs configured per environment

**Build Process**:
```bash
npm install
npm run build
npm run preview  # Test production build
```

**Rollout Strategy**:
- Build static assets
- Deploy to CDN/hosting
- No database migrations required (frontend only)
- Rollback: Revert to previous build

---

## Monitoring & Observability

**Metrics to Track**:
- Page load time (target: < 2s)
- WebSocket connection uptime (target: > 99%)
- API error rate (target: < 0.1%)
- User session duration
- Feature usage (most/least used features)

**Logging**:
- Log level: INFO in production, DEBUG in development
- Key log events:
  1. Authentication success/failure
  2. WebSocket connection/disconnection
  3. API errors
  4. Chart rendering errors
  5. User actions (page views, clicks)

**Alerts**:
- WebSocket disconnection: Alert if down > 1 minute
- API error spike: Alert if error rate > 5%
- Page load slow: Alert if > 5 seconds

**Dashboards**:
- User analytics: Page views, session duration, feature usage
- Performance metrics: Load times, API latency, WebSocket uptime
- Error tracking: Error rates, error types, stack traces

---

## Traceability

**Requirements**:
- User Story: Dashboard UI for trading bot management
- Business Rule: Real-time updates, paper trading simulation, AI integration

**Design**:
- Architecture: [ARCHITECTURE.md](../../../02-design/2.2-architecture/ARCHITECTURE.md)
- API Spec: [API_SPEC.md](../../../02-design/2.1-api-design/API_SPEC.md)
- Data Model: [DATA_MODELS.md](../../../02-design/2.3-data-models/DATA_MODELS.md)

**Test Cases**:
- Unit: TC-DASH-UNIT-001 to TC-DASH-UNIT-050
- Integration: TC-DASH-INT-001 to TC-DASH-INT-020
- E2E: TC-DASH-E2E-001 to TC-DASH-E2E-010

---

## Risks & Mitigations

| Risk | Impact | Probability | Mitigation |
|------|--------|-------------|------------|
| WebSocket connection instability | High | Medium | Implement auto-reconnect with exponential backoff, fallback to HTTP polling |
| Real-time data overload | Medium | Medium | Implement data deduplication, limit history, debounce updates |
| Mobile performance issues | High | Low | Lazy loading, code splitting, responsive images, performance monitoring |
| Browser compatibility issues | Medium | Low | Polyfills, browser testing, feature detection |
| API latency affecting UX | Medium | Medium | Loading states, optimistic updates, caching strategies |
| Token expiration during session | Medium | High | Token refresh mechanism, auto-logout warning |
| Chart rendering performance | Medium | Medium | Virtual scrolling, limit visible candles, debounce updates |

---

## Open Questions

- [x] Dark mode implementation priority? - Medium priority, not started
- [x] Full i18n support or Vietnamese only? - Partial implementation, full i18n planned
- [ ] Performance monitoring setup (Sentry, LogRocket)? - Resolution needed
- [ ] CDN selection for production deployment? - Resolution needed
- [ ] Service worker for offline support? - Resolution needed

---

## Changelog

| Version | Date | Author | Changes |
|---------|------|--------|---------|
| 1.0 | 2025-10-10 | Frontend Team | Initial comprehensive specification based on codebase analysis |

---

## Appendix

**References**:
- React Documentation: https://react.dev
- Vite Documentation: https://vitejs.dev
- Tailwind CSS: https://tailwindcss.com
- Shadcn/UI: https://ui.shadcn.com
- Recharts: https://recharts.org
- React Router: https://reactrouter.com

**Glossary**:
- WebSocket: Full-duplex communication protocol for real-time updates
- Paper Trading: Simulated trading without real money
- P&L: Profit and Loss
- OHLC: Open, High, Low, Close (candlestick data)
- JWT: JSON Web Token for authentication
- AI Signal: Machine learning-generated trading recommendation
- Leverage: Borrowed capital for trading (multiplier)
- Margin: Collateral required for leveraged positions
- RSI: Relative Strength Index (technical indicator)
- MACD: Moving Average Convergence Divergence (technical indicator)

**Examples**:
```typescript
// WebSocket connection
const { state, connect, disconnect } = useWebSocket();

// Paper trading operations
const { portfolio, openTrades, startTrading, stopTrading, closeTrade } = usePaperTrading();

// AI analysis
const { state: aiState, analyzeSymbol } = useAIAnalysis();
await analyzeSymbol('BTCUSDT', ['RSI Strategy', 'MACD Strategy']);

// Authentication
const { login, logout, isAuthenticated } = useAuth();
await login('user@example.com', 'password123');
```

---

**Remember**: Update TRACEABILITY_MATRIX.md when implementation is complete!
