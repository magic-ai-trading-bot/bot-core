# UI Component Library Documentation

**Document Version**: 1.0
**Last Updated**: 2025-10-10
**Owner**: Frontend Team
**Status**: Complete
**Total Components**: 71

---

## Table of Contents

1. [Overview](#overview)
2. [Component Categories](#component-categories)
3. [Page Components](#page-components)
4. [Dashboard Components](#dashboard-components)
5. [Shadcn/UI Components](#shadcnui-components)
6. [Custom Components](#custom-components)
7. [Hooks](#hooks)
8. [Services](#services)
9. [Component Usage Guidelines](#component-usage-guidelines)

---

## Overview

This document provides comprehensive documentation for all React components in the Bot Core Trading Platform. The application uses:
- **React 18** with TypeScript
- **Shadcn/UI** component library (50+ components)
- **Tailwind CSS** for styling
- **Recharts** for data visualization
- **Lucide React** for icons

**Related Documents**:
- **UI-WIREFRAMES.md** - Screen layouts and wireframes
- **UX-FLOWS.md** - User journey flows
- **FR-DASHBOARD.md** - Functional requirements

---

## Component Categories

### Component Breakdown

| Category | Count | Description |
|----------|-------|-------------|
| Page Components | 6 | Top-level route components |
| Dashboard Components | 6 | Domain-specific components |
| Shadcn/UI Components | 50+ | Reusable UI primitives |
| Custom Components | 3 | Specialized functionality |
| Hooks | 4 | Reusable state logic |

**Total: 71 components**

---

## Page Components

Page components are top-level route components that represent entire screens.

### 1. Login Page

**File**: `/Users/dungngo97/Documents/bot-core/nextjs-ui-dashboard/src/pages/Login.tsx`

**Description**: Authentication page for user login with form validation and demo credentials display.

**Route**: `/login`

**FR Reference**: FR-DASHBOARD-008

**Props**: None (route component)

**State Management**:
- `email`: string - User email input
- `password`: string - User password input
- `isLoading`: boolean - Form submission state

**Key Features**:
- Form validation (email format, required fields)
- Demo credentials display for testing
- Auto-redirect to dashboard on success
- Error handling with toast notifications
- ChatBot widget integration

**Dependencies**:
```typescript
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card";
import { Input } from "@/components/ui/input";
import { Button } from "@/components/ui/button";
import { Label } from "@/components/ui/label";
import ChatBot from "@/components/ChatBot";
import { useAuth } from "@/contexts/AuthContext";
import { toast } from "sonner";
```

**Example Usage**:
```tsx
// Used in routing configuration
<Route path="/login" element={<Login />} />
```

**Layout**:
- Centered card (max-w-sm md:max-w-md)
- Gradient background (radial-gradient from-primary/5)
- Responsive padding and spacing
- ChatBot widget at bottom-right

**Validation Rules**:
- Email: Required, must be valid email format
- Password: Required, minimum 6 characters

**Success Flow**:
1. User enters credentials
2. Form validation
3. API call to `/api/auth/login`
4. Success toast notification
5. Redirect to `/dashboard`

**Error States**:
- Invalid credentials: Toast error message
- Network error: Toast error with retry option
- Empty fields: Toast validation error

---

### 2. Register Page

**File**: `/Users/dungngo97/Documents/bot-core/nextjs-ui-dashboard/src/pages/Register.tsx`

**Description**: User registration page with form validation and auto-login on success.

**Route**: `/register`

**FR Reference**: FR-DASHBOARD-008

**Props**: None (route component)

**State Management**:
- `fullName`: string - User full name (optional)
- `email`: string - User email
- `password`: string - User password
- `confirmPassword`: string - Password confirmation
- `isLoading`: boolean - Form submission state

**Key Features**:
- Multi-field form with validation
- Password confirmation matching
- Auto-login after successful registration
- Feature list display
- ChatBot integration

**Dependencies**:
```typescript
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card";
import { Input } from "@/components/ui/input";
import { Button } from "@/components/ui/button";
import { Label } from "@/components/ui/label";
import ChatBot from "@/components/ChatBot";
import { useAuth } from "@/contexts/AuthContext";
import { toast } from "sonner";
```

**Validation Rules**:
- Email: Required, valid email format
- Password: Required, min 6 characters
- Confirm Password: Must match password
- Full Name: Optional

**Success Flow**:
1. User enters registration details
2. Form validation (password match check)
3. API call to `/api/auth/register`
4. Auto-login with new credentials
5. Redirect to `/dashboard`

**Error States**:
- Password mismatch: "Mật khẩu xác nhận không khớp"
- Password too short: "Mật khẩu phải có ít nhất 6 ký tự"
- Email already exists: Server error message in toast

---

### 3. Dashboard Page

**File**: `/Users/dungngo97/Documents/bot-core/nextjs-ui-dashboard/src/pages/Dashboard.tsx`

**Description**: Main dashboard home page with portfolio overview, charts, AI signals, and performance metrics.

**Route**: `/` or `/dashboard`

**FR Reference**: FR-DASHBOARD-001

**Props**: None (route component)

**State Management**:
- Uses multiple custom hooks:
  - `useWebSocket()` - Real-time price updates
  - `useAuth()` - User authentication state
  - `useAIAnalysis()` - AI signal data

**Key Features**:
- Bot status overview with portfolio metrics
- Real-time trading charts (lazy loaded)
- AI trading signals with confidence scores
- Performance chart (balance over time)
- Transaction history table
- AI strategy selector
- ChatBot integration

**Layout Structure**:
```
Dashboard
├── Header (Fixed top)
├── BotStatus (Portfolio metrics grid)
├── TradingCharts (Lazy loaded, 4-column grid)
├── AI Section
│   ├── AIStrategySelector (Sidebar)
│   └── AISignals (Main content)
├── PerformanceChart (Lazy loaded)
├── TransactionHistory (Table)
└── ChatBot (Floating widget)
```

**Dependencies**:
```typescript
import { TradingCharts } from "@/components/dashboard/TradingCharts";
import { AISignals } from "@/components/dashboard/AISignals";
import { TradingSettings } from "@/components/dashboard/TradingSettings";
import ChatBot from "@/components/ChatBot";
import { useWebSocket } from "@/hooks/useWebSocket";
import { useAuth } from "@/contexts/AuthContext";
```

**Lazy Loading**:
- TradingCharts component (heavy Recharts rendering)
- PerformanceChart (Recharts LineChart)
- ChatBot (AI chatbot widget)

**Real-time Features**:
- WebSocket connection for live prices
- Auto-refresh portfolio metrics every 5s
- Pulsing indicators for active bot status
- Live AI signal updates

---

### 4. Trading Paper Page

**File**: `/Users/dungngo97/Documents/bot-core/nextjs-ui-dashboard/src/pages/TradingPaper.tsx`

**Description**: Paper trading simulation page with real-time bot status, AI signals, trade history, and settings.

**Route**: `/trading-paper`

**FR Reference**: FR-DASHBOARD-004

**Props**: None (route component)

**State Management**:
- `activeTab`: string - Current tab (overview/signals/history/settings)
- `botStatus`: object - Bot running status and metrics
- `openTrades`: array - List of open positions
- `closedTrades`: array - Trade history
- `settings`: object - Paper trading configuration
- Uses custom hooks:
  - `usePaperTrading()` - Paper trading state and actions
  - `useWebSocket()` - Real-time updates
  - `useAIAnalysis()` - AI signal data

**Key Features**:
- **Tab 1 - Tổng quan**: Portfolio overview + Performance chart
- **Tab 2 - Tín hiệu AI**: Live AI signals with WebSocket updates
- **Tab 3 - Lịch sử giao dịch**: Open trades + Closed trades tables
- **Tab 4 - Cài đặt**: Bot config + Symbol settings + TradingSettings
- Real-time WebSocket connection status
- Interactive trade detail popups
- Symbol configuration dialog
- Comprehensive settings management

**Layout Structure**:
```
TradingPaper
├── Header (Fixed top)
├── Status Bar (WebSocket, Clock, Update counter)
├── Control Panel (Start/Stop/Update buttons)
├── Tabs Component
│   ├── Tab 1: Tổng quan
│   │   ├── Portfolio Overview (Metric cards grid)
│   │   └── Performance Chart (Recharts)
│   ├── Tab 2: Tín hiệu AI
│   │   └── AISignals Component
│   ├── Tab 3: Lịch sử giao dịch
│   │   ├── Open Trades Table
│   │   └── Closed Trades Table
│   └── Tab 4: Cài đặt
│       ├── Basic Settings Form
│       ├── Symbol Settings Dialog
│       └── TradingSettings (Embedded)
├── Trade Details Dialog
├── Symbol Config Dialog
└── ChatBot (Floating)
```

**Dependencies**:
```typescript
import { Tabs, TabsContent, TabsList, TabsTrigger } from "@/components/ui/tabs";
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card";
import { AISignals } from "@/components/dashboard/AISignals";
import { TradingSettings } from "@/components/dashboard/TradingSettings";
import { usePaperTrading } from "@/hooks/usePaperTrading";
import { useWebSocket } from "@/hooks/useWebSocket";
import { useAIAnalysis } from "@/hooks/useAIAnalysis";
```

**Real-time Features**:
- WebSocket connection indicator (🟢/🔴)
- Live update counter (Updates #142)
- Real-time clock (14:30:45)
- Auto-refresh portfolio metrics
- Live AI signal streaming
- Trade execution notifications

**Interactive Dialogs**:
1. **Trade Detail Dialog**: Shows full trade information with P&L
2. **Symbol Config Dialog**: Per-symbol settings (leverage, size, stop loss)
3. **Trading Settings Dialog**: Full strategy configuration

---

### 5. Settings Page

**File**: `/Users/dungngo97/Documents/bot-core/nextjs-ui-dashboard/src/pages/Settings.tsx`

**Description**: Comprehensive settings page with tabs for bot config, API keys, notifications, and security.

**Route**: `/settings`

**FR Reference**: FR-DASHBOARD (Settings sections)

**Props**: None (route component)

**State Management**:
- `activeTab`: string - Current settings tab
- `apiKeys`: object - Binance API credentials
- `notificationSettings`: object - Notification preferences
- `securitySettings`: object - Security configuration

**Key Features**:
- **Tab 1 - Bot Settings**: Embedded TradingSettings component
- **Tab 2 - API Keys**: Binance API configuration with test connection
- **Tab 3 - Thông báo**: Email, Telegram, Discord, Push notifications
- **Tab 4 - Bảo mật**: 2FA, password change, session management

**Layout Structure**:
```
Settings
├── Header
├── Tabs Component
│   ├── Tab 1: Bot Settings
│   │   └── TradingSettings (Full strategy config)
│   ├── Tab 2: API Keys
│   │   ├── API Key Input (masked)
│   │   ├── Secret Key Input (masked)
│   │   ├── Test Connection Button
│   │   └── Trading Permissions Checkboxes
│   ├── Tab 3: Thông báo
│   │   ├── Email Notifications (Toggle)
│   │   ├── Push Notifications (Toggle)
│   │   ├── Telegram Bot (Toggle + Token input)
│   │   └── Discord Webhook (Toggle + URL input)
│   └── Tab 4: Bảo mật
│       ├── 2FA Status Card
│       ├── Change Password Form
│       └── Active Sessions List
└── ChatBot
```

**Dependencies**:
```typescript
import { Tabs, TabsContent, TabsList, TabsTrigger } from "@/components/ui/tabs";
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card";
import { Input } from "@/components/ui/input";
import { Switch } from "@/components/ui/switch";
import { Button } from "@/components/ui/button";
import { TradingSettings } from "@/components/dashboard/TradingSettings";
```

**API Integration**:
- `GET /api/settings` - Load current settings
- `PUT /api/settings` - Save settings
- `POST /api/settings/test-api` - Test Binance API connection
- `POST /api/auth/change-password` - Change password

---

### 6. Not Found Page

**File**: `/Users/dungngo97/Documents/bot-core/nextjs-ui-dashboard/src/pages/NotFound.tsx`

**Description**: 404 error page displayed for unknown routes.

**Route**: `*` (catch-all)

**FR Reference**: FR-DASHBOARD-015

**Props**: None (route component)

**Key Features**:
- Simple centered layout with logo
- Large "404" text
- Friendly error message in Vietnamese
- Navigation buttons (Home, Dashboard)

**Layout**:
```
NotFound
├── Logo Badge
├── "404" (Large heading)
├── "Page Not Found"
├── Error message
└── Navigation Buttons
    ├── [🏠 Về trang chủ]
    └── [📊 Dashboard]
```

**Dependencies**:
```typescript
import { Button } from "@/components/ui/button";
import { useNavigate } from "react-router-dom";
```

---

## Dashboard Components

Domain-specific components for trading functionality.

### 1. TradingCharts Component

**File**: `/Users/dungngo97/Documents/bot-core/nextjs-ui-dashboard/src/components/dashboard/TradingCharts.tsx`

**Description**: Real-time candlestick charts with WebSocket updates for multiple trading pairs.

**Props**:
```typescript
interface TradingChartsProps {
  className?: string;
}
```

**State Management**:
- `charts`: ChartData[] - Array of chart data for multiple symbols
- `loading`: boolean - Initial data loading state
- `selectedTimeframe`: string - Current timeframe (1m, 5m, 15m, 1h, 4h, 1d)
- Uses `useWebSocket()` hook for real-time price updates

**Key Features**:
- Multi-symbol support (BTCUSDT, ETHUSDT, BNBUSDT, SOLUSDT)
- Timeframe selector (1m - 1d)
- Add/Remove symbols dynamically
- Real-time price updates via WebSocket
- Candlestick chart visualization (custom implementation)
- 24h price change and volume display
- Remove symbol functionality

**Sub-components**:
1. **CandlestickChart**: Custom candlestick renderer using DOM elements
2. **ChartCard**: Individual chart card with price info and chart
3. **AddSymbolDialog**: Dialog for adding new trading symbols

**Dependencies**:
```typescript
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card";
import { Badge } from "@/components/ui/badge";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from "@/components/ui/select";
import { Dialog, DialogContent, DialogHeader, DialogTitle, DialogTrigger } from "@/components/ui/dialog";
import { useWebSocket } from "@/hooks/useWebSocket";
import { apiClient } from "@/services/api";
import { toast } from "sonner";
```

**Data Flow**:
```
API (getSupportedSymbols) → Initial load
   ↓
API (getChartData) → Load historical candles
   ↓
WebSocket (ChartUpdate) → Real-time candle updates
   ↓
WebSocket (MarketData) → Real-time price updates
   ↓
State update → UI re-render
```

**ChartCard Props**:
```typescript
interface ChartCardProps {
  chartData: ChartData;
  onRemove: (symbol: string) => void;
}
```

**ChartData Interface**:
```typescript
interface ChartData {
  symbol: string;
  timeframe: string;
  latest_price: number;
  price_change_24h: number;
  price_change_percent_24h: number;
  volume_24h: number;
  candles: Candle[];
}

interface Candle {
  timestamp: number;
  open: number;
  high: number;
  low: number;
  close: number;
  volume: number;
}
```

**WebSocket Events**:
- `MarketData`: Real-time price updates (every second)
- `ChartUpdate`: Candle completion updates (per timeframe)

**Responsive Grid**:
```css
grid-cols-1 sm:grid-cols-2 lg:grid-cols-3 xl:grid-cols-4
gap-4
```

**Example Usage**:
```tsx
<TradingCharts className="mb-6" />
```

**Performance Optimizations**:
- Lazy loading with React.lazy()
- Memoized ChartCard with React.memo()
- Debounced price updates (avoid excessive re-renders)
- Slice candles to last 15 for chart display
- WebSocket connection shared across components

---

### 2. AISignals Component

**File**: `/Users/dungngo97/Documents/bot-core/nextjs-ui-dashboard/src/components/dashboard/AISignals.tsx`

**Description**: Displays live AI trading signals from both API and WebSocket with detailed analysis popups.

**Props**: None

**State Management**:
- Uses `useAIAnalysis()` hook for API signals
- Uses `useWebSocket()` hook for real-time WebSocket signals
- Combines and deduplicates signals (latest per symbol)
- Sorts by timestamp (newest first)

**Key Features**:
- Real-time AI signal streaming via WebSocket
- Detailed signal analysis dialog
- Strategy explanation dialogs (RSI, MACD, Volume, Bollinger)
- Confidence score visualization with progress bars
- Market analysis breakdown
- Risk assessment details
- Clickable strategy scores for education

**Sub-components**:
1. **DetailedSignalDialog**: Full signal analysis popup
2. **StrategyExplanationDialog**: Educational strategy information with SVG charts

**Signal Flow**:
```
AI Service (API) → useAIAnalysis() → signals
                        ↓
WebSocket Stream → useWebSocket() → aiSignals
                        ↓
                Combine & Deduplicate
                        ↓
           Sort by timestamp (newest first)
                        ↓
                  Display in UI
```

**Signal Interface**:
```typescript
interface FormattedSignal {
  id: string;
  signal: "LONG" | "SHORT" | "NEUTRAL";
  confidence: number;
  timestamp: string;
  pair: string;
  reason: string;
  active: boolean; // < 30 minutes old
  marketAnalysis?: AIMarketAnalysis;
  riskAssessment?: AIRiskAssessment;
  strategyScores?: Record<string, number>;
  source: "api" | "websocket";
  isWebSocket: boolean;
}
```

**AIMarketAnalysis Interface**:
```typescript
interface AIMarketAnalysis {
  trend_direction: string; // "Bullish" | "Bearish" | "Sideways"
  trend_strength: number; // 0.0 - 1.0
  support_levels: number[];
  resistance_levels: number[];
  volatility_level: string; // "Low" | "Medium" | "High"
  volume_analysis: string;
}
```

**AIRiskAssessment Interface**:
```typescript
interface AIRiskAssessment {
  overall_risk: string; // "Low" | "Medium" | "High"
  technical_risk: number; // 0.0 - 1.0
  market_risk: number; // 0.0 - 1.0
  recommended_position_size: number; // 0.0 - 1.0 (percentage)
  stop_loss_suggestion: number | null;
  take_profit_suggestion: number | null;
}
```

**DetailedSignalDialog Features**:
- Signal Overview Cards (Strength, Recommendation, Risk)
- Market Analysis Section (Trend, Volatility, Volume)
- Strategy Scores with Progress Bars (Clickable for explanation)
- Risk Assessment (SL/TP suggestions)
- Analysis Reasoning (Full AI explanation)

**StrategyExplanationDialog Content**:
- Strategy Description
- How it works explanation
- Buy/Sell signals
- Advantages and Disadvantages
- Best timeframes
- Chart illustrations (SVG visualizations)
- Educational explanations in Vietnamese

**Strategy Database** (STRATEGY_INFO):
```typescript
const STRATEGY_INFO = {
  "RSI Strategy": {
    name: "RSI Strategy",
    description: "Relative Strength Index - Xác định điều kiện quá mua/quá bán",
    how_it_works: "RSI dao động từ 0-100...",
    signals: { buy: "...", sell: "..." },
    advantages: [...],
    disadvantages: [...],
    best_timeframe: "1h, 4h, 1d",
    chart_description: "..."
  },
  // Similar for MACD, Volume, Bollinger Bands
};
```

**SVG Chart Illustrations**:
- RSI Indicator chart with overbought/oversold zones
- MACD chart with histogram and signal lines
- Volume & Price chart
- Bollinger Bands chart with squeeze patterns

**Dependencies**:
```typescript
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card";
import { Badge } from "@/components/ui/badge";
import { Button } from "@/components/ui/button";
import { Dialog, DialogContent, DialogHeader, DialogTitle, DialogTrigger } from "@/components/ui/dialog";
import { useAIAnalysis } from "@/hooks/useAIAnalysis";
import { useWebSocket } from "@/hooks/useWebSocket";
import { toast } from "sonner";
import { Wifi, WifiOff, AlertCircle, RefreshCw, TrendingUp, BarChart3, Target, Activity, Shield, ArrowUp, ArrowDown, Info } from "lucide-react";
```

**Color Coding**:
- 🟢 LONG signals: Green (profit class)
- 🔴 SHORT signals: Red (loss class)
- 🟡 NEUTRAL signals: Yellow (warning class)
- Confidence >= 80%: Green
- Confidence 60-80%: Yellow
- Confidence < 60%: Red

**Example Usage**:
```tsx
<AISignals />
```

**Performance**:
- Deduplication prevents duplicate signals for same symbol
- Only shows latest signal per symbol
- Active signals highlighted (< 30 min old)
- WebSocket connection status indicator

---

### 3. TradingSettings Component

**File**: `/Users/dungngo97/Documents/bot-core/nextjs-ui-dashboard/src/components/dashboard/TradingSettings.tsx`

**Description**: Comprehensive trading bot configuration dialog with market presets, strategy parameters, risk management, and engine settings.

**Props**: None

**State Management**:
- `isOpen`: boolean - Dialog open state
- `settings`: TradingSettingsData - All configuration data
- `selectedPreset`: string - Currently selected market preset
- `isLoading`: boolean - Loading settings from API
- `isSaving`: boolean - Saving settings to API

**Key Features**:
- **Tab 1 - Market Presets**: One-click configuration for different market conditions
- **Tab 2 - Strategies**: Individual strategy parameter tuning
- **Tab 3 - Risk Management**: Position and portfolio risk controls
- **Tab 4 - Engine Settings**: Signal processing and market condition config
- Real-time slider updates with value display
- Save/Reload functionality
- Preset application with toast confirmation

**Market Presets**:
```typescript
const MARKET_PRESETS = {
  low_volatility: {
    name: "Low Volatility",
    description: "Optimized for sideways/ranging markets",
    icon: "📊",
    settings: { /* Strategy + Risk + Engine config */ }
  },
  normal_volatility: {
    name: "Normal Volatility",
    description: "Balanced settings for typical markets",
    icon: "⚖️",
    settings: { /* ... */ }
  },
  high_volatility: {
    name: "High Volatility",
    description: "Conservative settings for volatile markets",
    icon: "🚀",
    settings: { /* ... */ }
  }
};
```

**Settings Interfaces**:
```typescript
interface TradingSettingsData {
  strategies: StrategySettings;
  risk: RiskSettings;
  engine: EngineSettings;
}

interface StrategySettings {
  rsi: {
    enabled: boolean;
    period: number;
    oversold_threshold: number;
    overbought_threshold: number;
    extreme_oversold: number;
    extreme_overbought: number;
  };
  macd: {
    enabled: boolean;
    fast_period: number;
    slow_period: number;
    signal_period: number;
    histogram_threshold: number;
  };
  volume: {
    enabled: boolean;
    sma_period: number;
    spike_threshold: number;
    correlation_period: number;
  };
  bollinger: {
    enabled: boolean;
    period: number;
    multiplier: number;
    squeeze_threshold: number;
  };
}

interface RiskSettings {
  max_risk_per_trade: number;
  max_portfolio_risk: number;
  stop_loss_percent: number;
  take_profit_percent: number;
  max_leverage: number;
  max_drawdown: number;
  daily_loss_limit: number;
  max_consecutive_losses: number;
}

interface EngineSettings {
  min_confidence_threshold: number;
  signal_combination_mode: string;
  enabled_strategies: string[];
  market_condition: string;
  risk_level: string;
}
```

**Tab 1 - Market Presets**:
- 3 preset cards in grid layout
- Click to apply preset
- Shows key metrics (confidence, risk, stop loss)
- Selected preset highlighted with ring
- Info box explaining low volatility settings

**Tab 2 - Strategies**:
- 4 strategy cards (RSI, MACD, Volume, Bollinger)
- Each with enable/disable switch
- Parameter sliders with real-time value display
- Grid layout (2 columns)

**Slider Components**:
```tsx
<Slider
  value={[settings.strategies.rsi.period]}
  onValueChange={([value]) => /* Update state */}
  min={5}
  max={30}
  step={1}
  className="w-full"
/>
```

**Tab 3 - Risk Management**:
- 2 card sections:
  - Position Risk (Max risk per trade, Stop Loss, Take Profit)
  - Portfolio Risk (Max portfolio risk, Drawdown, Consecutive losses)
- All with sliders and percentage/multiplier display

**Tab 4 - Engine Settings**:
- 2 card sections:
  - Signal Processing (Confidence threshold, Combination mode)
  - Market Conditions (Market condition, Risk level)
- Select dropdowns for modes:
  - Signal Combination: WeightedAverage, Consensus, BestConfidence, Conservative
  - Market Condition: Trending, Ranging, Volatile, LowVolume
  - Risk Level: Conservative, Moderate, Aggressive

**API Integration**:
```typescript
// Load settings
GET http://localhost:8080/api/paper-trading/strategy-settings

// Save settings
PUT http://localhost:8080/api/paper-trading/strategy-settings
{
  "settings": { /* TradingSettingsData */ }
}
```

**Dependencies**:
```typescript
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card";
import { Button } from "@/components/ui/button";
import { Badge } from "@/components/ui/badge";
import { Input } from "@/components/ui/input";
import { Label } from "@/components/ui/label";
import { Slider } from "@/components/ui/slider";
import { Switch } from "@/components/ui/switch";
import { Tabs, TabsContent, TabsList, TabsTrigger } from "@/components/ui/tabs";
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from "@/components/ui/select";
import { Dialog, DialogContent, DialogHeader, DialogTitle, DialogTrigger } from "@/components/ui/dialog";
import { toast } from "sonner";
import { Settings, Target, TrendingUp, AlertTriangle, BarChart3, Zap, Shield, Gauge, Save, RefreshCw } from "lucide-react";
```

**Example Usage**:
```tsx
<TradingSettings />
```

**User Flow**:
1. Click "Trading Settings" button
2. Dialog opens with default tab (Presets)
3. Select preset or customize manually
4. Switch tabs to adjust strategies/risk/engine
5. Click "Save Settings" button
6. Settings saved to backend
7. Success toast notification

**Loading States**:
- Initial load: Spinner + disabled form
- Saving: Button shows "Saving..." with spinner
- Reloading: "Loading..." button text

---

### 4. PortfolioOverview Component

**Description**: Displays key portfolio metrics in a responsive grid layout.

**Props**:
```typescript
interface PortfolioOverviewProps {
  balance: number;
  equity: number;
  totalPnL: number;
  totalTrades: number;
  openTrades: number;
  closedTrades: number;
  winRate: number;
  className?: string;
}
```

**Layout**:
- 4-column grid (responsive: 1 col mobile, 2 col tablet, 4 col desktop)
- Metric cards with large values and secondary info
- Color coding (profit/loss indicators)

**Metric Cards**:
1. Balance + Equity
2. Total P&L (with percentage and live indicator)
3. Total Trades (open/closed breakdown)
4. Win Rate (with fraction display)

---

### 5. PerformanceChart Component

**Description**: Recharts line chart showing balance over time with trade event markers.

**Props**:
```typescript
interface PerformanceChartProps {
  data: ChartDataPoint[];
  height?: number;
  className?: string;
}

interface ChartDataPoint {
  timestamp: number;
  balance: number;
  trade?: boolean; // Trade event marker
}
```

**Features**:
- Line chart with gradient fill
- Trade event markers (dots)
- Current status marker (large dot)
- Responsive height
- Tooltip on hover

**Dependencies**:
```typescript
import { LineChart, Line, XAxis, YAxis, CartesianGrid, Tooltip, ResponsiveContainer } from "recharts";
```

---

### 6. TransactionHistory Component

**Description**: Table displaying completed trades with sortable columns.

**Props**:
```typescript
interface TransactionHistoryProps {
  trades: Trade[];
  className?: string;
}

interface Trade {
  id: string;
  timestamp: number;
  symbol: string;
  type: "LONG" | "SHORT";
  quantity: number;
  entryPrice: number;
  exitPrice: number;
  pnl: number;
  duration: string;
}
```

**Features**:
- Sortable columns (timestamp, symbol, P&L)
- Color-coded P&L (green/red)
- Responsive table with horizontal scroll on mobile
- Click row to view trade details

---

## Shadcn/UI Components

Shadcn/UI provides 50+ reusable UI primitives. Below are the most commonly used components in the Bot Core platform.

### Core Components

#### 1. Button

**File**: `/Users/dungngo97/Documents/bot-core/nextjs-ui-dashboard/src/components/ui/button.tsx`

**Description**: Versatile button component with multiple variants and sizes.

**Props**:
```typescript
interface ButtonProps {
  variant?: "default" | "destructive" | "outline" | "secondary" | "ghost" | "link";
  size?: "default" | "sm" | "lg" | "icon";
  asChild?: boolean;
  disabled?: boolean;
  className?: string;
  children: React.ReactNode;
  onClick?: () => void;
}
```

**Variants**:
- **default**: Gradient primary button (bg-profit)
- **destructive**: Red button for dangerous actions
- **outline**: Border-only button for secondary actions
- **secondary**: Gray background button
- **ghost**: No background, hover effect only
- **link**: Text link styling

**Sizes**:
- **default**: h-10 px-4 py-2
- **sm**: h-9 px-3 text-sm
- **lg**: h-11 px-8
- **icon**: h-10 w-10 (square)

**Example Usage**:
```tsx
// Primary action
<Button>Save Settings</Button>

// Secondary action
<Button variant="outline">Cancel</Button>

// Destructive action
<Button variant="destructive">Delete Account</Button>

// Icon button
<Button variant="ghost" size="icon">
  <Settings className="h-4 w-4" />
</Button>

// Loading state
<Button disabled>
  <Loader2 className="mr-2 h-4 w-4 animate-spin" />
  Loading...
</Button>
```

---

#### 2. Card

**Files**:
- `card.tsx` - Card container
- Card components: CardHeader, CardTitle, CardDescription, CardContent, CardFooter

**Description**: Container component for grouping related content.

**Props**:
```typescript
interface CardProps {
  className?: string;
  children: React.ReactNode;
}
```

**Sub-components**:
- **CardHeader**: Top section with title and description
- **CardTitle**: Main heading (h3)
- **CardDescription**: Subtitle text
- **CardContent**: Main content area
- **CardFooter**: Bottom section for actions

**Example Usage**:
```tsx
<Card>
  <CardHeader>
    <CardTitle>Trading Charts</CardTitle>
    <CardDescription>Real-time market data</CardDescription>
  </CardHeader>
  <CardContent>
    <TradingCharts />
  </CardContent>
  <CardFooter>
    <Button>Refresh</Button>
  </CardFooter>
</Card>
```

---

#### 3. Input

**File**: `input.tsx`

**Description**: Text input field with consistent styling.

**Props**:
```typescript
interface InputProps extends React.InputHTMLAttributes<HTMLInputElement> {
  type?: string;
  placeholder?: string;
  value?: string;
  onChange?: (e: React.ChangeEvent<HTMLInputElement>) => void;
  disabled?: boolean;
  className?: string;
}
```

**Example Usage**:
```tsx
// Text input
<Input
  type="text"
  placeholder="Enter symbol (e.g., BTCUSDT)"
  value={symbol}
  onChange={(e) => setSymbol(e.target.value)}
/>

// Password input
<Input
  type="password"
  placeholder="Password"
  value={password}
  onChange={(e) => setPassword(e.target.value)}
/>

// Disabled input
<Input value="admin@tradingbot.com" disabled />
```

---

#### 4. Label

**File**: `label.tsx`

**Description**: Form label with accessibility support.

**Props**:
```typescript
interface LabelProps {
  htmlFor?: string;
  children: React.ReactNode;
  className?: string;
}
```

**Example Usage**:
```tsx
<Label htmlFor="email">Email Address</Label>
<Input id="email" type="email" />
```

---

#### 5. Badge

**File**: `badge.tsx`

**Description**: Small status indicator or label.

**Props**:
```typescript
interface BadgeProps {
  variant?: "default" | "secondary" | "destructive" | "outline";
  children: React.ReactNode;
  className?: string;
}
```

**Variants**:
- **default**: Primary color badge
- **secondary**: Gray badge
- **destructive**: Red badge
- **outline**: Border-only badge

**Example Usage**:
```tsx
// Status indicator
<Badge variant="outline" className="text-green-600 border-green-600">
  🟢 LIVE
</Badge>

// Trading signal
<Badge className="bg-profit">LONG</Badge>

// Confidence level
<Badge variant="secondary">82% Confidence</Badge>
```

---

#### 6. Dialog

**Files**:
- `dialog.tsx` - Main dialog component
- Sub-components: DialogTrigger, DialogContent, DialogHeader, DialogTitle, DialogDescription, DialogFooter

**Description**: Modal dialog for focused interactions.

**Props**:
```typescript
interface DialogProps {
  open?: boolean;
  onOpenChange?: (open: boolean) => void;
  children: React.ReactNode;
}
```

**Example Usage**:
```tsx
<Dialog open={isOpen} onOpenChange={setIsOpen}>
  <DialogTrigger asChild>
    <Button>Open Settings</Button>
  </DialogTrigger>
  <DialogContent className="max-w-4xl">
    <DialogHeader>
      <DialogTitle>Trading Settings</DialogTitle>
      <DialogDescription>
        Configure your bot parameters
      </DialogDescription>
    </DialogHeader>
    {/* Dialog content */}
    <DialogFooter>
      <Button variant="outline" onClick={() => setIsOpen(false)}>
        Cancel
      </Button>
      <Button onClick={saveSettings}>Save</Button>
    </DialogFooter>
  </DialogContent>
</Dialog>
```

---

#### 7. Tabs

**Files**:
- `tabs.tsx` - Tabs container
- Sub-components: TabsList, TabsTrigger, TabsContent

**Description**: Tabbed interface for organizing content.

**Props**:
```typescript
interface TabsProps {
  defaultValue?: string;
  value?: string;
  onValueChange?: (value: string) => void;
  children: React.ReactNode;
  className?: string;
}
```

**Example Usage**:
```tsx
<Tabs defaultValue="overview">
  <TabsList className="grid w-full grid-cols-4">
    <TabsTrigger value="overview">Tổng quan</TabsTrigger>
    <TabsTrigger value="signals">Tín hiệu AI</TabsTrigger>
    <TabsTrigger value="history">Lịch sử</TabsTrigger>
    <TabsTrigger value="settings">Cài đặt</TabsTrigger>
  </TabsList>
  <TabsContent value="overview">
    <PortfolioOverview />
  </TabsContent>
  <TabsContent value="signals">
    <AISignals />
  </TabsContent>
  {/* Other tabs */}
</Tabs>
```

---

#### 8. Select

**Files**:
- `select.tsx` - Select dropdown
- Sub-components: SelectTrigger, SelectValue, SelectContent, SelectItem

**Description**: Dropdown select component.

**Props**:
```typescript
interface SelectProps {
  value?: string;
  onValueChange?: (value: string) => void;
  disabled?: boolean;
  children: React.ReactNode;
}
```

**Example Usage**:
```tsx
<Select value={timeframe} onValueChange={setTimeframe}>
  <SelectTrigger className="w-24">
    <SelectValue />
  </SelectTrigger>
  <SelectContent>
    <SelectItem value="1m">1m</SelectItem>
    <SelectItem value="5m">5m</SelectItem>
    <SelectItem value="15m">15m</SelectItem>
    <SelectItem value="1h">1h</SelectItem>
    <SelectItem value="4h">4h</SelectItem>
    <SelectItem value="1d">1d</SelectItem>
  </SelectContent>
</Select>
```

---

#### 9. Slider

**File**: `slider.tsx`

**Description**: Range slider for numeric input.

**Props**:
```typescript
interface SliderProps {
  value?: number[];
  onValueChange?: (value: number[]) => void;
  min?: number;
  max?: number;
  step?: number;
  disabled?: boolean;
  className?: string;
}
```

**Example Usage**:
```tsx
<div>
  <Label>RSI Period: {rsiPeriod}</Label>
  <Slider
    value={[rsiPeriod]}
    onValueChange={([value]) => setRsiPeriod(value)}
    min={5}
    max={30}
    step={1}
    className="w-full"
  />
</div>
```

---

#### 10. Switch

**File**: `switch.tsx`

**Description**: Toggle switch for boolean states.

**Props**:
```typescript
interface SwitchProps {
  checked?: boolean;
  onCheckedChange?: (checked: boolean) => void;
  disabled?: boolean;
  className?: string;
}
```

**Example Usage**:
```tsx
<div className="flex items-center gap-2">
  <Switch
    checked={rsiEnabled}
    onCheckedChange={setRsiEnabled}
  />
  <Label>Enable RSI Strategy</Label>
</div>
```

---

#### 11. Table

**Files**:
- `table.tsx` - Table container
- Sub-components: TableHeader, TableBody, TableRow, TableHead, TableCell

**Description**: Styled table component.

**Example Usage**:
```tsx
<Table>
  <TableHeader>
    <TableRow>
      <TableHead>Time</TableHead>
      <TableHead>Symbol</TableHead>
      <TableHead>Type</TableHead>
      <TableHead>P&L</TableHead>
    </TableRow>
  </TableHeader>
  <TableBody>
    {trades.map((trade) => (
      <TableRow key={trade.id}>
        <TableCell>{formatTime(trade.timestamp)}</TableCell>
        <TableCell>{trade.symbol}</TableCell>
        <TableCell>
          <Badge className={trade.type === "LONG" ? "bg-profit" : "bg-loss"}>
            {trade.type}
          </Badge>
        </TableCell>
        <TableCell className={trade.pnl >= 0 ? "text-profit" : "text-loss"}>
          ${trade.pnl.toFixed(2)}
        </TableCell>
      </TableRow>
    ))}
  </TableBody>
</Table>
```

---

#### 12. ScrollArea

**File**: `scroll-area.tsx`

**Description**: Customizable scrollable area.

**Props**:
```typescript
interface ScrollAreaProps {
  className?: string;
  children: React.ReactNode;
}
```

**Example Usage**:
```tsx
<ScrollArea className="h-96">
  {/* Long content */}
</ScrollArea>
```

---

#### 13. Separator

**File**: `separator.tsx`

**Description**: Visual divider line.

**Props**:
```typescript
interface SeparatorProps {
  orientation?: "horizontal" | "vertical";
  decorative?: boolean;
  className?: string;
}
```

**Example Usage**:
```tsx
<div>
  <Section1 />
  <Separator className="my-4" />
  <Section2 />
</div>
```

---

#### 14. Toast (Sonner)

**Package**: `sonner`

**Description**: Toast notifications for user feedback.

**Usage**:
```tsx
import { toast } from "sonner";

// Success
toast.success("Settings saved successfully!");

// Error
toast.error("Failed to save settings");

// Info
toast.info("Trading bot started", {
  description: "Bot is now analyzing market signals",
  duration: 5000,
});

// Loading
toast.loading("Analyzing market...");

// Warning
toast.warning("High volatility detected");
```

---

### Additional Shadcn/UI Components

The following components are also available but less frequently used:

15. **Accordion** - Collapsible content panels
16. **Alert** - Alert messages with variants
17. **AlertDialog** - Confirmation dialogs
18. **AspectRatio** - Maintain aspect ratios
19. **Avatar** - User avatars with fallbacks
20. **Breadcrumb** - Navigation breadcrumbs
21. **Calendar** - Date picker calendar
22. **Checkbox** - Checkbox input
23. **Collapsible** - Collapsible content
24. **Command** - Command palette
25. **ContextMenu** - Right-click menus
26. **DatePicker** - Date selection
27. **DropdownMenu** - Dropdown menus
28. **Form** - Form wrapper with validation
29. **HoverCard** - Hover popup cards
30. **MenuBar** - Menu bar navigation
31. **NavigationMenu** - Complex navigation
32. **Pagination** - Page navigation
33. **Popover** - Popup content
34. **Progress** - Progress bars
35. **RadioGroup** - Radio button groups
36. **ResizablePanel** - Resizable layouts
37. **Sheet** - Side panel drawer
38. **Skeleton** - Loading skeletons
39. **Textarea** - Multi-line text input
40. **Toggle** - Toggle button
41. **ToggleGroup** - Toggle button groups
42. **Tooltip** - Hover tooltips

---

## Custom Components

### 1. ChatBot Component

**File**: `/Users/dungngo97/Documents/bot-core/nextjs-ui-dashboard/src/components/ChatBot.tsx`

**Description**: AI-powered chatbot widget for user assistance with Vietnamese language support.

**Props**:
```typescript
interface ChatBotProps {
  isOpen?: boolean;
  onToggle?: () => void;
  className?: string;
}
```

**State Management**:
- `messages`: ChatMessage[] - Chat history
- `inputMessage`: string - User input
- `isLoading`: boolean - AI processing state
- `isTyping`: boolean - Typing indicator
- `isMinimized`: boolean - Widget minimized state
- `soundEnabled`: boolean - Notification sound toggle

**Key Features**:
- Floating widget (bottom-right corner)
- Collapsible/expandable interface
- AI-powered responses via chatbot service
- Vietnamese language support
- Suggested questions
- Typing indicator animation
- Sound notifications
- Chat history persistence
- Confidence indicators for AI responses

**ChatMessage Interface**:
```typescript
interface ChatMessage {
  id: string;
  type: "user" | "bot";
  content: string;
  timestamp: Date;
}
```

**Widget States**:
1. **Collapsed**: Floating button (64x64px)
2. **Expanded**: Full chat interface (384x600px)
3. **Minimized**: Header only (384x64px)

**Layout (Expanded)**:
```
ChatBot Widget
├── Header (Gradient bg)
│   ├── AI Assistant Title
│   └── Controls (Sound, Minimize, Close)
├── Messages Area (Scrollable)
│   ├── Bot Messages (Left-aligned)
│   ├── User Messages (Right-aligned)
│   ├── Typing Indicator
│   └── Suggested Questions (Initial state)
└── Input Area
    ├── Text Input
    ├── Clear Button
    ├── Send Button
    └── Status Footer
```

**Suggested Questions**:
```typescript
const suggestedQuestions = [
  "Bot hoạt động thế nào?",
  "Cách cài đặt strategy?",
  "Giải thích về RSI indicator?",
  "Paper trading là gì?"
];
```

**AI Service Integration**:
```typescript
import { chatbotService, ChatMessage } from "@/services/chatbot";

// Process user message
const response = await chatbotService.processMessage(userMessage);

// Response interface
interface ChatbotResponse {
  message: string;
  type: "ai" | "rule_based";
  confidence: number;
}
```

**Sound Notifications**:
- Web Audio API for notification beeps
- 800Hz sine wave, 0.1s duration
- Toggle on/off with button

**Dependencies**:
```typescript
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { Badge } from "@/components/ui/badge";
import { ScrollArea } from "@/components/ui/scroll-area";
import { chatbotService } from "@/services/chatbot";
import { toast } from "sonner";
import { MessageCircle, Send, X, Minimize2, Maximize2, Bot, User, Loader2, Sparkles, HelpCircle, Trash2, Volume2, VolumeX } from "lucide-react";
```

**Example Usage**:
```tsx
// Controlled
<ChatBot isOpen={chatOpen} onToggle={() => setChatOpen(!chatOpen)} />

// Uncontrolled
<ChatBot />
```

**Confidence Indicators**:
- Low confidence (< 70%): Toast warning "Câu trả lời có thể chưa chính xác"
- Shows confidence percentage in toast

**Mobile Responsive**:
- Fixed position on all screen sizes
- z-index: 50 (above most content)
- Full-width on mobile (<384px screens)

---

### 2. BotStatus Component

**Description**: Displays current bot status with portfolio metrics in a grid layout.

**Props**:
```typescript
interface BotStatusProps {
  status: "active" | "inactive" | "paused";
  balance: number;
  equity: number;
  marginUsed: number;
  freeMargin: number;
  className?: string;
}
```

**Features**:
- 4-column metric grid (responsive)
- Status indicator (🟢 Active / 🔴 Inactive)
- Large metric values with currency formatting
- Color-coded status badge

**Layout**:
```
BotStatus Card
├── Header
│   ├── Title: "BOT STATUS"
│   └── Status Badge: "🟢 Active"
└── Metrics Grid (4 columns)
    ├── Balance
    ├── Equity
    ├── Margin Used
    └── Free Margin
```

---

### 3. AIStrategySelector Component

**Description**: Strategy enable/disable checkboxes for filtering AI signals.

**Props**:
```typescript
interface AIStrategySelectorProps {
  enabledStrategies: string[];
  onToggle: (strategy: string) => void;
  className?: string;
}
```

**Strategies**:
- RSI Strategy
- MACD Strategy
- Volume Strategy
- Bollinger Bands Strategy

**Features**:
- Checkbox list with icons
- Toggle individual strategies
- Settings button to open TradingSettings dialog

---

## Hooks

Custom React hooks for reusable state logic.

### 1. useWebSocket Hook

**File**: `/Users/dungngo97/Documents/bot-core/nextjs-ui-dashboard/src/hooks/useWebSocket.ts`

**Description**: Manages WebSocket connection to Rust backend for real-time updates.

**Returns**:
```typescript
interface WebSocketState {
  isConnected: boolean;
  isConnecting: boolean;
  error: string | null;
  lastMessage: WebSocketMessage | null;
  aiSignals: AISignal[];
}

interface WebSocketHook {
  state: WebSocketState;
  connect: () => void;
  disconnect: () => void;
  subscribe: (channel: string) => void;
  unsubscribe: (channel: string) => void;
}
```

**WebSocket Message Types**:
```typescript
type WebSocketMessage =
  | { type: "MarketData"; data: MarketDataUpdateData }
  | { type: "ChartUpdate"; data: ChartUpdateData }
  | { type: "AISignal"; data: AISignalData }
  | { type: "TradeExecution"; data: TradeExecutionData }
  | { type: "Error"; data: { message: string } };
```

**Usage Example**:
```tsx
const { state, connect, disconnect } = useWebSocket();

useEffect(() => {
  if (!state.isConnected && !state.isConnecting) {
    connect();
  }
  return () => disconnect();
}, []);

// Use state.lastMessage to trigger UI updates
useEffect(() => {
  if (state.lastMessage?.type === "MarketData") {
    // Update price display
  }
}, [state.lastMessage]);
```

**Features**:
- Auto-reconnect on disconnect
- Connection state management
- Message type discrimination
- AI signal accumulation
- Error handling

**WebSocket URL**:
```
ws://localhost:8080/ws
```

---

### 2. useAIAnalysis Hook

**File**: `/Users/dungngo97/Documents/bot-core/nextjs-ui-dashboard/src/hooks/useAIAnalysis.ts`

**Description**: Manages AI signal fetching and analysis state.

**Returns**:
```typescript
interface AIAnalysisState {
  signals: AISignal[];
  isLoading: boolean;
  error: string | null;
  serviceInfo: ServiceInfo | null;
  lastUpdate: Date | null;
}

interface AIAnalysisHook {
  state: AIAnalysisState;
  analyzeSymbol: (symbol: string) => Promise<void>;
  clearError: () => void;
}
```

**AISignal Interface**:
```typescript
interface AISignal {
  signal: "long" | "short" | "neutral";
  confidence: number;
  timestamp: string | number;
  symbol?: string;
  reasoning?: string;
  strategy_scores?: Record<string, number>;
  market_analysis?: AIMarketAnalysis;
  risk_assessment?: AIRiskAssessment;
}
```

**Usage Example**:
```tsx
const { state, analyzeSymbol, clearError } = useAIAnalysis();

// Trigger analysis
const handleAnalyze = async () => {
  await analyzeSymbol("BTCUSDT");
};

// Display signals
{state.signals.map((signal) => (
  <SignalCard key={signal.timestamp} signal={signal} />
))}
```

**API Endpoints**:
```
GET http://localhost:8000/api/ai/health - Service info
POST http://localhost:8000/api/ai/analyze - Analyze symbol
```

---

### 3. usePaperTrading Hook

**File**: `/Users/dungngo97/Documents/bot-core/nextjs-ui-dashboard/src/hooks/usePaperTrading.ts`

**Description**: Manages paper trading state, trades, and bot control.

**Returns**:
```typescript
interface PaperTradingState {
  status: "active" | "inactive" | "paused";
  portfolio: Portfolio;
  openTrades: Trade[];
  closedTrades: Trade[];
  settings: PaperTradingSettings;
  isLoading: boolean;
  error: string | null;
}

interface PaperTradingHook {
  state: PaperTradingState;
  startBot: () => Promise<void>;
  stopBot: () => Promise<void>;
  executeTrade: (trade: TradeRequest) => Promise<void>;
  closeTrade: (tradeId: string) => Promise<void>;
  updateSettings: (settings: PaperTradingSettings) => Promise<void>;
  refresh: () => Promise<void>;
}
```

**Portfolio Interface**:
```typescript
interface Portfolio {
  balance: number;
  equity: number;
  totalPnL: number;
  totalPnLPercent: number;
  totalTrades: number;
  winRate: number;
}
```

**Trade Interface**:
```typescript
interface Trade {
  id: string;
  symbol: string;
  type: "LONG" | "SHORT";
  entryPrice: number;
  exitPrice?: number;
  quantity: number;
  positionSize: number;
  leverage: number;
  marginRequired: number;
  unrealizedPnL?: number;
  realizedPnL?: number;
  stopLoss?: number;
  takeProfit?: number;
  openTime: number;
  closeTime?: number;
  status: "open" | "closed";
}
```

**Usage Example**:
```tsx
const { state, startBot, stopBot, executeTrade, closeTrade } = usePaperTrading();

// Start bot
<Button onClick={startBot} disabled={state.status === "active"}>
  {state.status === "active" ? "🟢 Đang hoạt động" : "Khởi động Bot"}
</Button>

// Execute trade
const handleTrade = async () => {
  await executeTrade({
    symbol: "BTCUSDT",
    type: "LONG",
    quantity: 0.05,
    leverage: 20,
    stopLoss: 26500,
    takeProfit: 28500
  });
};

// Close position
<Button onClick={() => closeTrade(trade.id)}>
  Đóng vị thế
</Button>
```

**API Endpoints**:
```
GET /api/paper-trading/status - Bot status
POST /api/paper-trading/start - Start bot
POST /api/paper-trading/stop - Stop bot
GET /api/paper-trading/portfolio - Portfolio data
GET /api/paper-trading/trades - Trade history
POST /api/paper-trading/execute - Execute trade
POST /api/paper-trading/close/{id} - Close trade
GET /api/paper-trading/settings - Get settings
PUT /api/paper-trading/settings - Update settings
```

---

### 4. useAuth Hook

**File**: `/Users/dungngo97/Documents/bot-core/nextjs-ui-dashboard/src/contexts/AuthContext.tsx`

**Description**: Authentication state management with login/logout functionality.

**Returns**:
```typescript
interface AuthState {
  isAuthenticated: boolean;
  user: User | null;
  isLoading: boolean;
}

interface AuthHook {
  state: AuthState;
  login: (email: string, password: string) => Promise<void>;
  register: (email: string, password: string, fullName?: string) => Promise<void>;
  logout: () => void;
}
```

**User Interface**:
```typescript
interface User {
  id: string;
  email: string;
  fullName?: string;
  createdAt: Date;
}
```

**Usage Example**:
```tsx
const { state, login, logout } = useAuth();

// Login
const handleLogin = async () => {
  await login(email, password);
  navigate("/dashboard");
};

// Protected route
if (!state.isAuthenticated) {
  return <Navigate to="/login" />;
}

// Display user
<div>Welcome, {state.user?.email}</div>

// Logout
<Button onClick={logout}>Đăng xuất</Button>
```

**API Endpoints**:
```
POST /api/auth/login - User login
POST /api/auth/register - User registration
POST /api/auth/logout - User logout
GET /api/auth/me - Get current user
```

---

## Services

### 1. API Client Service

**File**: `/Users/dungngo97/Documents/bot-core/nextjs-ui-dashboard/src/services/api.ts`

**Description**: Centralized API client for all backend communication.

**Exports**:
```typescript
export const apiClient = {
  rust: {
    getSupportedSymbols: () => Promise<SymbolsResponse>,
    getChartData: (symbol: string, timeframe: string, limit: number) => Promise<ChartData>,
    getLatestPrices: () => Promise<Record<string, number>>,
    addSymbol: (request: AddSymbolRequest) => Promise<void>,
    removeSymbol: (symbol: string) => Promise<void>,
    // ... more methods
  },
  python: {
    analyzeSymbol: (symbol: string) => Promise<AIAnalysis>,
    getServiceInfo: () => Promise<ServiceInfo>,
    // ... more methods
  },
  auth: {
    login: (email: string, password: string) => Promise<LoginResponse>,
    register: (email: string, password: string, fullName?: string) => Promise<RegisterResponse>,
    logout: () => Promise<void>,
    // ... more methods
  }
};
```

**Base URLs**:
```typescript
const RUST_API_URL = "http://localhost:8080";
const PYTHON_API_URL = "http://localhost:8000";
```

**Error Handling**:
- Network errors: Throws error with message
- HTTP errors: Throws error with status and message
- Timeout: 30s default timeout

---

### 2. Chatbot Service

**File**: `/Users/dungngo97/Documents/bot-core/nextjs-ui-dashboard/src/services/chatbot.ts`

**Description**: AI chatbot service with rule-based fallback and OpenAI integration.

**Exports**:
```typescript
export const chatbotService = {
  processMessage: (message: string) => Promise<ChatbotResponse>,
  addMessageToHistory: (message: ChatMessage) => void,
  clearHistory: () => void,
  getSuggestedQuestions: () => string[],
};

interface ChatbotResponse {
  message: string;
  type: "ai" | "rule_based";
  confidence: number;
}
```

**Features**:
- OpenAI GPT-3.5 integration (if API key available)
- Rule-based fallback for common questions
- Vietnamese language support
- Chat history management
- Suggested questions

**Rule-based Responses**:
```typescript
const RULE_BASED_RESPONSES = {
  "bot hoạt động thế nào": "Bot sử dụng AI và các chỉ báo kỹ thuật...",
  "cách cài đặt strategy": "Vào Settings → Bot Settings...",
  // ... more rules
};
```

**OpenAI Integration**:
```typescript
const response = await openai.chat.completions.create({
  model: "gpt-3.5-turbo",
  messages: [
    { role: "system", content: SYSTEM_PROMPT },
    ...chatHistory,
    { role: "user", content: userMessage }
  ],
  temperature: 0.7,
  max_tokens: 500
});
```

---

## Component Usage Guidelines

### 1. Naming Conventions

**Component Files**:
- PascalCase for component names: `TradingCharts.tsx`
- kebab-case for UI primitives: `button.tsx`, `card.tsx`

**Props Interfaces**:
- Component name + "Props" suffix: `TradingChartsProps`
- Export interfaces for public components

**State Variables**:
- camelCase: `isLoading`, `userData`
- Boolean prefix: `is`, `has`, `should`

### 2. Component Organization

**File Structure**:
```
ComponentName.tsx
├── Imports
├── Type Definitions (Props, State interfaces)
├── Constants
├── Helper Functions
├── Sub-components (if any)
├── Main Component
└── Export
```

**Example**:
```tsx
// Imports
import React, { useState, useEffect } from "react";
import { Button } from "@/components/ui/button";

// Type Definitions
interface MyComponentProps {
  title: string;
  onSubmit: () => void;
}

// Constants
const DEFAULT_VALUES = {
  timeout: 5000,
};

// Helper Functions
function formatData(data: any) {
  return data.toString();
}

// Sub-component
const SubComponent: React.FC = () => {
  return <div>...</div>;
};

// Main Component
export function MyComponent({ title, onSubmit }: MyComponentProps) {
  const [loading, setLoading] = useState(false);

  return (
    <div>
      <h1>{title}</h1>
      <SubComponent />
      <Button onClick={onSubmit}>Submit</Button>
    </div>
  );
}
```

### 3. Performance Best Practices

**Memoization**:
```tsx
// Memoize expensive components
const ExpensiveChart = React.memo(ChartComponent);

// Memoize callbacks
const handleSubmit = useCallback(() => {
  // ... logic
}, [dependencies]);

// Memoize computed values
const sortedData = useMemo(() => {
  return data.sort((a, b) => a.value - b.value);
}, [data]);
```

**Lazy Loading**:
```tsx
// Page-level lazy loading
const TradingCharts = React.lazy(() => import("@/components/dashboard/TradingCharts"));

// Usage with Suspense
<Suspense fallback={<LoadingSpinner />}>
  <TradingCharts />
</Suspense>
```

**Avoid Prop Drilling**:
```tsx
// Use context for deeply nested props
const ThemeContext = React.createContext();

// Or use state management library
import { useStore } from "@/store";
```

### 4. Accessibility Guidelines

**Keyboard Navigation**:
- All interactive elements must be keyboard accessible
- Use `tabIndex` for custom controls
- Implement `onKeyDown` handlers for custom interactions

**ARIA Labels**:
```tsx
<Button aria-label="Close dialog" onClick={onClose}>
  <X className="h-4 w-4" />
</Button>

<Input
  aria-label="Email address"
  aria-describedby="email-help"
  aria-invalid={hasError}
/>
```

**Focus Management**:
```tsx
// Auto-focus on dialog open
const inputRef = useRef<HTMLInputElement>(null);

useEffect(() => {
  if (isOpen) {
    inputRef.current?.focus();
  }
}, [isOpen]);

<Input ref={inputRef} />
```

### 5. Error Handling

**Try-Catch Blocks**:
```tsx
const handleSubmit = async () => {
  try {
    setLoading(true);
    await apiClient.rust.executeTrade(tradeData);
    toast.success("Trade executed successfully!");
  } catch (error) {
    logger.error("Trade execution failed:", error);
    toast.error("Failed to execute trade");
  } finally {
    setLoading(false);
  }
};
```

**Error Boundaries**:
```tsx
<ErrorBoundary fallback={<ErrorFallback />}>
  <TradingCharts />
</ErrorBoundary>
```

### 6. TypeScript Best Practices

**Strict Typing**:
```tsx
// Avoid 'any'
const data: unknown = await fetch();

// Use type guards
if (typeof data === "object" && data !== null) {
  // Safe to use
}

// Use generics
function processData<T>(data: T): T {
  return data;
}
```

**Union Types**:
```tsx
type Status = "idle" | "loading" | "success" | "error";

interface State {
  status: Status;
  data: Data | null;
  error: Error | null;
}
```

---

## Related Documents

- **UI-WIREFRAMES.md** - Screen layouts and wireframes
- **UX-FLOWS.md** - User journey flows
- **FR-DASHBOARD.md** - Functional requirements
- **DATA_MODELS.md** - Data structures and schemas
- **API_SPEC.md** - API endpoints and contracts

---

## Version History

| Version | Date | Author | Changes |
|---------|------|--------|---------|
| 1.0 | 2025-10-10 | Frontend Team | Initial documentation for 71 components |

---

**END OF UI-COMPONENTS.md**
