# UI Wireframes and Layouts

**Document Version**: 1.0
**Last Updated**: 2025-10-10
**Owner**: Frontend Team
**Status**: Complete

---

## Table of Contents

1. [Overview](#overview)
2. [Design Principles](#design-principles)
3. [Screen Wireframes](#screen-wireframes)
   - [Authentication Screens](#authentication-screens)
   - [Dashboard Screens](#dashboard-screens)
   - [Trading Screens](#trading-screens)
   - [Settings Screens](#settings-screens)
4. [Layout Patterns](#layout-patterns)
5. [Responsive Breakpoints](#responsive-breakpoints)
6. [Component Hierarchy](#component-hierarchy)

---

## Overview

This document provides detailed wireframes and layout specifications for all screens in the Bot Core Trading Platform dashboard. Each wireframe includes:
- ASCII art representation of the layout
- Component breakdown
- Responsive behavior
- Links to functional requirements

**Related Documents**:
- **FR-DASHBOARD.md** - Functional requirements for all screens
- **UI-COMPONENTS.md** - Component library documentation
- **UX-FLOWS.md** - User journey flows

---

## Design Principles

### Visual Hierarchy
- **Primary actions** highlighted with gradient buttons
- **Critical information** displayed prominently (balance, P&L, bot status)
- **Secondary actions** use outline buttons
- **Destructive actions** require confirmation

### Consistency
- **Shadcn/UI** component library for unified design language
- **Tailwind CSS** utility classes for consistent spacing and colors
- **Color coding**: Green (profit/bullish), Red (loss/bearish), Yellow (warning), Blue (info)

### Accessibility
- **Minimum touch target**: 44x44px for mobile
- **Contrast ratio**: WCAG AA compliant
- **Keyboard navigation**: Full support with focus indicators
- **Screen reader**: ARIA labels on all interactive elements

---

## Screen Wireframes

## Authentication Screens

### 1. Login Page (`/login`)

**File**: `nextjs-ui-dashboard/src/pages/Login.tsx`
**FR Reference**: FR-DASHBOARD-008 (Authentication & Authorization)

```
┌────────────────────────────────────────────────────────────────┐
│                                                                │
│     [Background: Radial gradient from-primary/5]              │
│                                                                │
│                    ┌──────────────┐                           │
│                    │   🤖  BT    │   Logo Badge              │
│                    └──────────────┘                           │
│                                                                │
│              Crypto Trading Bot                                │
│          Đăng nhập để quản lý bot trading của bạn            │
│                                                                │
│        ┌────────────────────────────────────────┐            │
│        │    ĐĂNG NHẬP                          │  Card      │
│        ├────────────────────────────────────────┤            │
│        │                                        │            │
│        │  Email                                 │            │
│        │  [____________________________]        │  Input     │
│        │                                        │            │
│        │  Mật khẩu                             │            │
│        │  [____________________________]        │  Input     │
│        │                                        │  (password)│
│        │                                        │            │
│        │  [      ĐĂNG NHẬP      ]              │  Button    │
│        │                                        │  (gradient)│
│        │                                        │            │
│        │  Chưa có tài khoản? Đăng ký ngay      │  Link      │
│        │                                        │            │
│        │  ┌──────────────────────────────┐    │            │
│        │  │ Demo credentials:            │    │  Info Box  │
│        │  │ Email: admin@tradingbot.com  │    │            │
│        │  │ Password: demo123            │    │            │
│        │  └──────────────────────────────┘    │            │
│        │                                        │            │
│        │  • AI-Powered Trading Signals         │  Features  │
│        │  • Real-time Performance Analytics    │  List      │
│        │  • Advanced Risk Management           │            │
│        │                                        │            │
│        └────────────────────────────────────────┘            │
│                                                                │
│          Bảo mật với mã hóa end-to-end và xác thực 2FA       │
│                                                                │
│                 🤖 ChatBot Widget (bottom-right)              │
│                                                                │
└────────────────────────────────────────────────────────────────┘
```

**Layout Details**:
- **Container**: Centered card on gradient background
- **Card Width**: max-w-sm md:max-w-md (400px mobile, 448px tablet+)
- **Logo**: 48px mobile, 64px desktop
- **Form Spacing**: space-y-4 (16px between fields)
- **Button**: Full width, gradient bg-profit
- **Demo Box**: p-4 bg-muted/50 rounded-lg border

**Responsive Behavior**:
- **Mobile (<640px)**: Single column, padding p-4
- **Tablet (640px+)**: Larger logo and text
- **Desktop (768px+)**: Increased spacing

**State Variations**:
- **Loading**: Button shows "Đang đăng nhập..." with spinner
- **Error**: Toast notification appears with error message
- **Success**: Toast notification + redirect to dashboard
- **Empty fields**: Toast error "Vui lòng nhập email và mật khẩu"

**Components Used**:
- `Card` (Shadcn/UI)
- `Input` (Shadcn/UI)
- `Button` (Shadcn/UI)
- `Label` (Shadcn/UI)
- `ChatBot` (Custom)

---

### 2. Register Page (`/register`)

**File**: `nextjs-ui-dashboard/src/pages/Register.tsx`
**FR Reference**: FR-DASHBOARD-008

```
┌────────────────────────────────────────────────────────────────┐
│                                                                │
│     [Background: Radial gradient from-primary/5]              │
│                                                                │
│                    ┌──────────────┐                           │
│                    │   🤖  BT    │   Logo Badge              │
│                    └──────────────┘                           │
│                                                                │
│              Crypto Trading Bot                                │
│            Tạo tài khoản để bắt đầu giao dịch                │
│                                                                │
│        ┌────────────────────────────────────────┐            │
│        │    ĐĂNG KÝ                            │  Card      │
│        ├────────────────────────────────────────┤            │
│        │                                        │            │
│        │  Họ và tên (tùy chọn)                 │            │
│        │  [____________________________]        │  Input     │
│        │                                        │            │
│        │  Email                                 │            │
│        │  [____________________________]        │  Input     │
│        │                                        │  (required)│
│        │                                        │            │
│        │  Mật khẩu                             │            │
│        │  [____________________________]        │  Input     │
│        │                                        │  (password)│
│        │                                        │            │
│        │  Xác nhận mật khẩu                    │            │
│        │  [____________________________]        │  Input     │
│        │                                        │  (password)│
│        │                                        │            │
│        │  [      ĐĂNG KÝ       ]               │  Button    │
│        │                                        │  (gradient)│
│        │                                        │            │
│        │  Đã có tài khoản? Đăng nhập ngay      │  Link      │
│        │                                        │            │
│        │  • AI-Powered Trading Signals         │  Features  │
│        │  • Real-time Performance Analytics    │  List      │
│        │  • Advanced Risk Management           │            │
│        │                                        │            │
│        └────────────────────────────────────────┘            │
│                                                                │
│          Bảo mật với mã hóa end-to-end và xác thực 2FA       │
│                                                                │
│                 🤖 ChatBot Widget (bottom-right)              │
│                                                                │
└────────────────────────────────────────────────────────────────┘
```

**Validation Rules**:
- **Email**: Required, valid email format
- **Password**: Required, min 6 characters
- **Confirm Password**: Must match password
- **Full Name**: Optional

**State Variations**:
- **Password mismatch**: Toast error "Mật khẩu xác nhận không khớp"
- **Password too short**: Toast error "Mật khẩu phải có ít nhất 6 ký tự"
- **Success**: Auto-login + redirect to dashboard

---

## Dashboard Screens

### 3. Dashboard Home (`/dashboard`)

**File**: `nextjs-ui-dashboard/src/pages/Dashboard.tsx`
**FR Reference**: FR-DASHBOARD-001 (Dashboard Home Page)

```
┌──────────────────────────────────────────────────────────────────────────┐
│ ┌────────────────────────────────────────────────────────────────────┐  │
│ │  🤖 BOT CORE     [Search]      🔔 Notifications    👤 User ▼      │  │ Header
│ └────────────────────────────────────────────────────────────────────┘  │
├──────────────────────────────────────────────────────────────────────────┤
│                                                                          │
│ ┌──────────────────────────────────────────────────────────────────┐   │
│ │  BOT STATUS                                       🟢 Active      │   │ BotStatus
│ │  ┌──────────┐  ┌──────────┐  ┌──────────┐  ┌──────────┐       │   │
│ │  │Balance   │  │Equity    │  │ Margin   │  │Free      │       │   │
│ │  │$10,245.50│  │$12,500.00│  │ Used     │  │Margin    │       │   │
│ │  └──────────┘  └──────────┘  └──────────┘  └──────────┘       │   │
│ └──────────────────────────────────────────────────────────────────┘   │
│                                                                          │
│ ┌──────────────────────────────────────────────────────────────────┐   │
│ │  TRADING CHARTS                           🟢 LIVE | ⚡ MAINNET  │   │ TradingCharts
│ │  [Timeframe: 1m ▼]  [+ Add Symbol]  [Update Prices] [Refresh]   │   │ (Lazy loaded)
│ │                                                                   │   │
│ │  ┌────────┐  ┌────────┐  ┌────────┐  ┌────────┐               │   │
│ │  │BTCUSDT │  │ETHUSDT │  │BNBUSDT │  │SOLUSDT │               │   │
│ │  │$27,800 │  │$1,650  │  │$238    │  │$45.50  │               │   │
│ │  │+2.5%   │  │+1.8%   │  │-0.5%   │  │+5.2%   │               │   │
│ │  │        │  │        │  │        │  │        │               │   │
│ │  │[Chart] │  │[Chart] │  │[Chart] │  │[Chart] │               │   │
│ │  │        │  │        │  │        │  │        │               │   │
│ │  │[Remove]│  │[Remove]│  │[Remove]│  │[Remove]│               │   │
│ │  └────────┘  └────────┘  └────────┘  └────────┘               │   │
│ └──────────────────────────────────────────────────────────────────┘   │
│                                                                          │
│ ┌──────────────┐  ┌─────────────────────────────────────────────┐     │
│ │AI STRATEGY   │  │  AI SIGNALS                    🟢 Live     │     │
│ │SELECTOR      │  │  ┌────────────────────────────────────────┐ │     │
│ │              │  │  │ LONG BTCUSDT    82% ✓ 🟢 ACTIVE      │ │     │
│ │[RSI ✓]      │  │  │ Strong momentum, RSI oversold...      │ │     │
│ │[MACD ✓]     │  │  │ ████████████████░░░░ 82%              │ │     │
│ │[Volume ✓]   │  │  └────────────────────────────────────────┘ │     │
│ │[Bollinger ✓]│  │  ┌────────────────────────────────────────┐ │     │
│ │              │  │  │ SHORT ETHUSDT   65% 🟡               │ │     │
│ │[Settings]    │  │  │ MACD bearish divergence...           │ │     │
│ │              │  │  │ ████████████░░░░░░░░ 65%             │ │     │
│ │              │  │  └────────────────────────────────────────┘ │     │
│ └──────────────┘  └─────────────────────────────────────────────┘     │
│                                                                          │
│ ┌──────────────────────────────────────────────────────────────────┐   │
│ │  PERFORMANCE CHART                                                │   │ PerformanceChart
│ │  Balance: $10,245.50  |  P&L: +$245.50 (+2.5%)                  │   │ (Lazy loaded)
│ │  ┌────────────────────────────────────────────────────────────┐ │   │
│ │  │                                                             │ │   │
│ │  │  [Line chart showing balance over time]                    │ │   │
│ │  │                                                             │ │   │
│ │  └────────────────────────────────────────────────────────────┘ │   │
│ └──────────────────────────────────────────────────────────────────┘   │
│                                                                          │
│ ┌──────────────────────────────────────────────────────────────────┐   │
│ │  TRANSACTION HISTORY                                              │   │ TransactionHistory
│ │  ┌────────────────────────────────────────────────────────────┐ │   │
│ │  │ Time     | Symbol  | Type  | Qty    | Price   | P&L      │ │   │
│ │  ├────────────────────────────────────────────────────────────┤ │   │
│ │  │ 10:30 AM | BTCUSDT | LONG  | 0.05   | $27,500 | +$150    │ │   │
│ │  │ 11:45 AM | ETHUSDT | SHORT | 1.2    | $1,650  | +$50     │ │   │
│ │  └────────────────────────────────────────────────────────────┘ │   │
│ └──────────────────────────────────────────────────────────────────┘   │
│                                                                          │
│                   🤖 ChatBot Widget (bottom-right)                      │
│                                                                          │
└──────────────────────────────────────────────────────────────────────────┘
```

**Layout Structure**:
- **Header**: Fixed top, full width
- **Content**: Vertical stack, p-4 lg:p-6
- **Grid Layouts**:
  - BotStatus: grid-cols-1 md:grid-cols-2 lg:grid-cols-4
  - Charts: grid-cols-1 sm:grid-cols-2 lg:grid-cols-3 xl:grid-cols-4
  - AI Section: grid-cols-1 lg:grid-cols-3

**Lazy Loaded Components**:
- TradingCharts (heavy candlestick rendering)
- PerformanceChart (Recharts library)
- ChatBot (AI chatbot widget)

**Real-time Updates**:
- **WebSocket connection** for live price updates
- **Pulsing indicators** for active status
- **Auto-refresh** portfolio metrics

---

### 4. Trading Paper Page (`/trading-paper`)

**File**: `nextjs-ui-dashboard/src/pages/TradingPaper.tsx`
**FR Reference**: FR-DASHBOARD-004 (Paper Trading Management)

```
┌──────────────────────────────────────────────────────────────────────────────┐
│ ┌────────────────────────────────────────────────────────────────────────┐  │
│ │  🤖 BOT CORE     [Search]      🔔 Notifications    👤 User ▼          │  │ Header
│ └────────────────────────────────────────────────────────────────────────┘  │
├──────────────────────────────────────────────────────────────────────────────┤
│                                                                              │
│  🎯 Trading Paper                                                           │
│  Mô phỏng giao dịch với AI Bot - Kiểm thử chiến lược không rủi ro         │
│                                                                              │
│  🌐 WebSocket Connected | ⏰ 14:30:45 | Last update: 2s ago                │
│  [🟢 Đang hoạt động] [⏸️ Dừng Bot] [🔄 Cập nhật]                          │
│                                                                              │
│  ┌─────────────────────────────────────────────────────────────────────┐   │
│  │ [Tổng quan] [Tín hiệu AI] [Lịch sử giao dịch] [Cài đặt]           │   │ Tabs
│  └─────────────────────────────────────────────────────────────────────┘   │
│                                                                              │
│  ━━━━━━━━━━━━━━━━━ TAB: TỔNG QUAN ━━━━━━━━━━━━━━━━━━                       │
│                                                                              │
│  ┌──────────────────────────────────────────────────────────────────┐      │
│  │ PORTFOLIO OVERVIEW                                                │      │
│  │ ┌─────────────┐ ┌─────────────┐ ┌─────────────┐ ┌─────────────┐│      │
│  │ │ Số dư       │ │ Tổng P&L 🔴│ │ Tổng lệnh 🔵│ │ Tỷ lệ thắng││      │
│  │ │ $10,245.50  │ │ +$245.50    │ │ 5 lệnh      │ │ 65.0%       ││      │
│  │ │ Equity:     │ │ +2.5% 🔴   │ │ Đang mở: 3  │ │ 3/5         ││      │
│  │ │ $12,500.00  │ │ • Live      │ │ Đã đóng: 2  │ │             ││      │
│  │ └─────────────┘ └─────────────┘ └─────────────┘ └─────────────┘│      │
│  └──────────────────────────────────────────────────────────────────┘      │
│                                                                              │
│  ┌──────────────────────────────────────────────────────────────────┐      │
│  │ BIỂU ĐỒ HIỆU SUẤT      🟢 Live Data | Updates #142              │      │
│  │ ┌────────────────────────────────────────────────────────────┐  │      │
│  │ │                                                             │  │      │
│  │ │  [Line chart: Balance over 24h timeline]                   │  │      │
│  │ │  • 8 data points (3-hour intervals)                        │  │      │
│  │ │  • Trade event markers (🟢 dots)                          │  │      │
│  │ │  • Current status marker (🔴 large dot)                   │  │      │
│  │ │                                                             │  │      │
│  │ └────────────────────────────────────────────────────────────┘  │      │
│  └──────────────────────────────────────────────────────────────────┘      │
│                                                                              │
│  ━━━━━━━━━━━━━━━━━ TAB: TÍN HIỆU AI ━━━━━━━━━━━━━━━━━━                     │
│                                                                              │
│  ┌──────────────────────────────────────────────────────────────────┐      │
│  │ TÍN HIỆU AI GẦN ĐÂY              🟢 Live Analysis               │      │
│  │ Grok Trading AI v2.0.0 • Model: grok-4-1-fast-non-reasoning      │      │
│  │                                                                   │      │
│  │ ┌─────────────────────────────────────────────────────────────┐ │      │
│  │ │ [LONG] BTCUSDT 🟢 ACTIVE 📡 websocket         82%          │ │      │
│  │ │ Strong bullish momentum, RSI oversold, MACD golden cross   │ │      │
│  │ │ 10:30 AM                                    🟢 AI Confidence│ │      │
│  │ │ ████████████████░░░░ 82%                                   │ │      │
│  │ └─────────────────────────────────────────────────────────────┘ │      │
│  │ ┌─────────────────────────────────────────────────────────────┐ │      │
│  │ │ [SHORT] ETHUSDT 🟡 websocket                  65%          │ │      │
│  │ │ Bearish divergence on MACD, high volume selling pressure   │ │      │
│  │ │ 11:15 AM                                    🟡 AI Confidence│ │      │
│  │ │ ████████████░░░░░░░░ 65%                                   │ │      │
│  │ └─────────────────────────────────────────────────────────────┘ │      │
│  └──────────────────────────────────────────────────────────────────┘      │
│                                                                              │
│  ━━━━━━━━━━━━━━━━━ TAB: LỊCH SỬ GIAO DỊCH ━━━━━━━━━━━━━━━━━━               │
│                                                                              │
│  ┌──────────────────────────────────────────────────────────────────┐      │
│  │ LỆNH ĐANG MỞ (3) 🟢 Live                                        │      │
│  │ Tổng Position Size: $8,250 | Tổng Margin Required: $412.50      │      │
│  │                                                                   │      │
│  │ ┌─────────────────────────────────────────────────────────────┐ │      │
│  │ │ Symbol  | Type  | Entry   | Qty    | Position | Margin  │   │      │
│  │ │         |       | Price   |        | Size     | Required│   │      │
│  │ ├─────────────────────────────────────────────────────────────┤ │      │
│  │ │ BTCUSDT │[LONG] │$27,500  │ 0.05   │ $1,375   │ $68.75  │   │      │
│  │ │ (20x)   │ 🟢   │         │ BTC    │          │(20x lev)│   │      │
│  │ │ Unrealized P&L: +$150 (+10.91%)        [Đóng]             │   │      │
│  │ │ SL: $26,500 | TP: $28,500 | Open: 10:30 AM                │   │      │
│  │ ├─────────────────────────────────────────────────────────────┤ │      │
│  │ │ ETHUSDT │[SHORT]│$1,650   │ 1.2    │ $1,980   │ $198.00 │   │      │
│  │ │ (10x)   │ 🔴   │         │ ETH    │          │(10x lev)│   │      │
│  │ │ Unrealized P&L: +$50 (+2.53%)          [Đóng]             │   │      │
│  │ │ SL: $1,700 | TP: $1,600 | Open: 11:45 AM                 │   │      │
│  │ └─────────────────────────────────────────────────────────────┘ │      │
│  └──────────────────────────────────────────────────────────────────┘      │
│                                                                              │
│  ┌──────────────────────────────────────────────────────────────────┐      │
│  │ LỊCH SỬ GIAO DỊCH (2) 🟢 Live                                   │      │
│  │ ┌─────────────────────────────────────────────────────────────┐ │      │
│  │ │ Symbol  | Type  | Entry | Exit   | Qty  | P&L  | Duration │ │      │
│  │ ├─────────────────────────────────────────────────────────────┤ │      │
│  │ │ BNBUSDT │[LONG] │ $235  │ $238   │ 5    │+$45.5│ 45m     │ │      │
│  │ │ SOLUSDT │[LONG] │ $45   │ $45.50 │ 10   │+$50  │ 30m     │ │      │
│  │ └─────────────────────────────────────────────────────────────┘ │      │
│  └──────────────────────────────────────────────────────────────────┘      │
│                                                                              │
│  ━━━━━━━━━━━━━━━━━ TAB: CÀI ĐẶT ━━━━━━━━━━━━━━━━━━                         │
│                                                                              │
│  ┌──────────────────────────────────────────────────────────────────┐      │
│  │ CÀI ĐẶT PAPER TRADING                                            │      │
│  │ ┌─────────────────┐  ┌─────────────────┐                        │      │
│  │ │ Vốn ban đầu     │  │ Đòn bẩy tối đa  │                        │      │
│  │ │ [10000]   USDT  │  │ [20]        x   │                        │      │
│  │ └─────────────────┘  └─────────────────┘                        │      │
│  │ ┌─────────────────┐  ┌─────────────────┐                        │      │
│  │ │ Kích thước vị   │  │ Stop Loss mặc   │                        │      │
│  │ │ thế (%)         │  │ định (%)        │                        │      │
│  │ │ [2.0]       %   │  │ [2.0]       %   │                        │      │
│  │ └─────────────────┘  └─────────────────┘                        │      │
│  │                                                                   │      │
│  │ [Lưu cài đặt]  [🔄 Reset dữ liệu]                               │      │
│  └──────────────────────────────────────────────────────────────────┘      │
│                                                                              │
│  ┌──────────────────────────────────────────────────────────────────┐      │
│  │ CÀI ĐẶT SYMBOLS                        4 symbols                │      │
│  │ Cấu hình riêng cho từng symbol                                   │      │
│  │ [Mở cài đặt Symbols]                                             │      │
│  └──────────────────────────────────────────────────────────────────┘      │
│                                                                              │
│  ┌──────────────────────────────────────────────────────────────────┐      │
│  │ CÀI ĐẶT CHIẾN LƯỢC TRADING                                       │      │
│  │ [TradingSettings Component - Embedded]                            │      │
│  └──────────────────────────────────────────────────────────────────┘      │
│                                                                              │
│  🟢 WebSocket Active | Real-time updates: 142 | Last sync: 14:30:45 🚀    │
│                                                                              │
│                   🤖 ChatBot Widget (bottom-right)                          │
│                                                                              │
└──────────────────────────────────────────────────────────────────────────────┘
```

**Key Features**:
- **Real-time indicators**: WebSocket connection status, live clock, update counter
- **Tabbed interface**: 4 tabs (Tổng quan, Tín hiệu AI, Lịch sử giao dịch, Cài đặt)
- **Portfolio cards**: Animated with pulsing effects for live P&L
- **Interactive tables**: Click rows to open detailed trade popup
- **Performance chart**: 24h balance history with Recharts
- **AI signals**: WebSocket streaming with confidence bars
- **Settings**: Multi-section configuration UI

**Trade Details Popup**:
```
┌────────────────────────────────────────────────────────────┐
│  [LONG] BTCUSDT  Chi tiết giao dịch  🟢 Live              │
│  Thông tin chi tiết về vị thế đang mở                     │
├────────────────────────────────────────────────────────────┤
│                                                            │
│  ┌────────────────────┐  ┌────────────────────┐          │
│  │ Unrealized P&L     │  │ Position Size      │          │
│  │ +$150.00          │  │ $1,375.00          │          │
│  │ +10.91%           │  │ với 20x leverage   │          │
│  └────────────────────┘  └────────────────────┘          │
│                                                            │
│  ┌─────────────────────┐  ┌─────────────────────┐        │
│  │ THÔNG TIN GIAO DỊCH│  │ RISK MANAGEMENT     │        │
│  │                     │  │                     │        │
│  │ Symbol: BTCUSDT    │  │ Stop Loss:         │        │
│  │ Type: [LONG] 🟢   │  │ $26,500 (-3.64%)   │        │
│  │ Entry: $27,500.00  │  │                     │        │
│  │ Quantity: 0.05 BTC │  │ Take Profit:       │        │
│  │ Leverage: 20x      │  │ $28,500 (+3.64%)   │        │
│  │ Position: $1,375   │  │                     │        │
│  │                     │  │ Open Time:         │        │
│  │                     │  │ 10:30 AM           │        │
│  │                     │  │ Duration: 45 phút  │        │
│  └─────────────────────┘  └─────────────────────┘        │
│                                                            │
│  [❌ Đóng vị thế]  [Đóng popup]                          │
│                                                            │
└────────────────────────────────────────────────────────────┘
```

**Symbol Configuration Dialog**:
```
┌────────────────────────────────────────────────────────────────┐
│  🎯 Cài đặt Symbols                          4 symbols        │
├────────────────────────────────────────────────────────────────┤
│                                                                │
│  ┌──────────────────────────────────────────────────────────┐│
│  │  BTCUSDT                              [Bật] Kích hoạt ☑ ││
│  │  ┌──────────┐ ┌──────────┐ ┌──────────┐                 ││
│  │  │Đòn bẩy   │ │Kích thước│ │Số vị thế │                 ││
│  │  │[20]   x  │ │[2.0]   % │ │[3]       │                 ││
│  │  └──────────┘ └──────────┘ └──────────┘                 ││
│  │  ┌──────────┐ ┌──────────┐                               ││
│  │  │Stop Loss │ │Take Prof │                               ││
│  │  │[2.0]   % │ │[5.0]   % │                               ││
│  │  └──────────┘ └──────────┘                               ││
│  └──────────────────────────────────────────────────────────┘│
│                                                                │
│  [Similar cards for ETHUSDT, BNBUSDT, SOLUSDT...]            │
│                                                                │
│  [Lưu cài đặt Symbols]  [🔄 Tải lại]                         │
│                                                                │
└────────────────────────────────────────────────────────────────┘
```

---

### 5. Settings Page (`/settings`)

**File**: `nextjs-ui-dashboard/src/pages/Settings.tsx`
**FR Reference**: FR-DASHBOARD (Settings sections)

```
┌──────────────────────────────────────────────────────────────────────────┐
│ ┌────────────────────────────────────────────────────────────────────┐  │
│ │  🤖 BOT CORE     [Search]      🔔 Notifications    👤 User ▼      │  │
│ └────────────────────────────────────────────────────────────────────┘  │
├──────────────────────────────────────────────────────────────────────────┤
│                                                                          │
│  ⚙️ Cài đặt Bot                                                         │
│  Quản lý cấu hình và tùy chọn cho trading bot của bạn                  │
│                                                                          │
│  ┌─────────────────────────────────────────────────────────────────┐   │
│  │ [Bot Settings] [API Keys] [Thông báo] [Bảo mật]               │   │ Tabs
│  └─────────────────────────────────────────────────────────────────┘   │
│                                                                          │
│  ━━━━━━━━━━━━━━━━━ TAB: BOT SETTINGS ━━━━━━━━━━━━━━━━━━               │
│                                                                          │
│  ┌──────────────────────────────────────────────────────────────┐      │
│  │ TRADING STRATEGY CONFIGURATION                                │      │
│  │ [BotSettings Component - Full strategy config UI]             │      │
│  │ • RSI Strategy settings                                        │      │
│  │ • MACD Strategy settings                                       │      │
│  │ • Volume Strategy settings                                     │      │
│  │ • Bollinger Bands settings                                     │      │
│  │ • Risk management parameters                                   │      │
│  └──────────────────────────────────────────────────────────────┘      │
│                                                                          │
│  ━━━━━━━━━━━━━━━━━ TAB: API KEYS ━━━━━━━━━━━━━━━━━━                    │
│                                                                          │
│  ┌──────────────────────────────────────────────────────────────┐      │
│  │ BINANCE API CONFIGURATION         🟡 Testnet                  │      │
│  │                                                                │      │
│  │ API Key                                                        │      │
│  │ [************************************1234]  (password field)   │      │
│  │                                                                │      │
│  │ Secret Key                                                     │      │
│  │ [************************************5678]  (password field)   │      │
│  │                                                                │      │
│  │ ℹ️ Lưu ý bảo mật                                               │      │
│  │ API keys được mã hóa và lưu trữ an toàn.                     │      │
│  │ Chỉ cấp quyền Futures Trading cho bot.                       │      │
│  │                                                                │      │
│  │ [Test Connection]  [Lưu API Keys]                             │      │
│  └──────────────────────────────────────────────────────────────┘      │
│                                                                          │
│  ┌──────────────────────────────────────────────────────────────┐      │
│  │ QUYỀN HẠN TRADING                                             │      │
│  │ ┌────────────────────────────────────────────────────────┐   │      │
│  │ │ Spot Trading         ☐                                 │   │      │
│  │ │ Giao dịch spot cơ bản                                  │   │      │
│  │ ├────────────────────────────────────────────────────────┤   │      │
│  │ │ Futures Trading      ☑ (disabled - required)          │   │      │
│  │ │ Giao dịch futures với đòn bẩy                         │   │      │
│  │ ├────────────────────────────────────────────────────────┤   │      │
│  │ │ Margin Trading       ☐                                 │   │      │
│  │ │ Giao dịch ký quỹ                                       │   │      │
│  │ └────────────────────────────────────────────────────────┘   │      │
│  └──────────────────────────────────────────────────────────────┘      │
│                                                                          │
│  ━━━━━━━━━━━━━━━━━ TAB: THÔNG BÁO ━━━━━━━━━━━━━━━━━━                   │
│                                                                          │
│  ┌──────────────────────────────────────────────────────────────┐      │
│  │ TÙY CHỌN THÔNG BÁO                                            │      │
│  │ ┌────────────────────────────────────────────────────────┐   │      │
│  │ │ Email Notifications         [ON]  ☑                    │   │      │
│  │ │ Nhận thông báo qua email                               │   │      │
│  │ ├────────────────────────────────────────────────────────┤   │      │
│  │ │ Push Notifications          [OFF] ☐                    │   │      │
│  │ │ Thông báo đẩy trên trình duyệt                        │   │      │
│  │ ├────────────────────────────────────────────────────────┤   │      │
│  │ │ Telegram Bot                [ON]  ☑                    │   │      │
│  │ │ Thông báo qua Telegram                                 │   │      │
│  │ │ ┌──────────────────────────────────────────────────┐  │   │      │
│  │ │ │ Telegram Bot Token                                │  │   │      │
│  │ │ │ [Nhập bot token từ @BotFather]                   │  │   │      │
│  │ │ └──────────────────────────────────────────────────┘  │   │      │
│  │ ├────────────────────────────────────────────────────────┤   │      │
│  │ │ Discord Webhook             [OFF] ☐                    │   │      │
│  │ │ Thông báo qua Discord                                  │   │      │
│  │ └────────────────────────────────────────────────────────┘   │      │
│  │                                                                │      │
│  │ [Lưu cài đặt thông báo]                                       │      │
│  └──────────────────────────────────────────────────────────────┘      │
│                                                                          │
│  ━━━━━━━━━━━━━━━━━ TAB: BẢO MẬT ━━━━━━━━━━━━━━━━━━                     │
│                                                                          │
│  ┌──────────────────────────────────────────────────────────────┐      │
│  │ BẢO MẬT TÀI KHOẢN                                             │      │
│  │                                                                │      │
│  │ ┌────────────────────────────────────────────────────────┐   │      │
│  │ │ Two-Factor Authentication          🟢 Đã kích hoạt    │   │      │
│  │ │ Xác thực 2 yếu tố đã được bật                         │   │      │
│  │ └────────────────────────────────────────────────────────┘   │      │
│  │                                                                │      │
│  │ ┌────────────────────────────────────────────────────────┐   │      │
│  │ │ Đổi mật khẩu                                           │   │      │
│  │ │ [Mật khẩu hiện tại]                                    │   │      │
│  │ │ [Mật khẩu mới]                                         │   │      │
│  │ │ [Xác nhận mật khẩu mới]                               │   │      │
│  │ │ [Cập nhật mật khẩu]                                    │   │      │
│  │ └────────────────────────────────────────────────────────┘   │      │
│  │                                                                │      │
│  │ ┌────────────────────────────────────────────────────────┐   │      │
│  │ │ Phiên đăng nhập                                        │   │      │
│  │ │ Chrome on Windows           🟢 Active now              │   │      │
│  │ │ Mobile App                  2 hours ago                │   │      │
│  │ │ [Đăng xuất tất cả thiết bị]                           │   │      │
│  │ └────────────────────────────────────────────────────────┘   │      │
│  └──────────────────────────────────────────────────────────────┘      │
│                                                                          │
│                   🤖 ChatBot Widget (bottom-right)                      │
│                                                                          │
└──────────────────────────────────────────────────────────────────────────┘
```

**Layout Notes**:
- **Tabbed interface**: 4 tabs for different settings categories
- **BotSettings**: Embedded component with full strategy configuration
- **API Keys**: Password-masked inputs with Test Connection button
- **Notifications**: Toggle switches for each notification type
- **Security**: 2FA status, password change, session management

---

### 6. 404 Not Found Page (`*`)

**File**: `nextjs-ui-dashboard/src/pages/NotFound.tsx`
**FR Reference**: FR-DASHBOARD-015 (Navigation & Routing)

```
┌──────────────────────────────────────────────────────────────┐
│                                                              │
│                                                              │
│                    ┌──────────────┐                         │
│                    │   🤖  BT    │   Logo                  │
│                    └──────────────┘                         │
│                                                              │
│                        404                                   │
│                  Page Not Found                              │
│                                                              │
│              Trang bạn tìm không tồn tại                    │
│                                                              │
│                                                              │
│              [🏠 Về trang chủ] [📊 Dashboard]              │
│                                                              │
│                                                              │
└──────────────────────────────────────────────────────────────┘
```

---

## Layout Patterns

### 1. Application Shell Layout

**Structure**:
```
┌─────────────────────────────────────┐
│          Header (Fixed)             │ 64px height
├─────────────────────────────────────┤
│                                     │
│                                     │
│         Main Content Area           │ Scrollable
│         (p-4 lg:p-6)               │
│                                     │
│                                     │
└─────────────────────────────────────┘
     Chatbot (Floating, z-50)
```

**Header Components**:
- Logo/Brand (left)
- Search bar (center, desktop only)
- Notifications icon (right)
- User menu dropdown (right)

**Content Patterns**:
- **Full-width cards**: Used for main sections
- **Grid layouts**: Responsive breakpoints for metrics/charts
- **Tabbed content**: For multi-section pages

### 2. Card Layout Pattern

**Standard Card**:
```
┌────────────────────────────────┐
│ Card Title         [Action]    │ CardHeader
├────────────────────────────────┤
│                                │
│   Card Content                 │ CardContent
│   (p-6)                        │
│                                │
└────────────────────────────────┘
```

**Metric Card**:
```
┌────────────────────┐
│ Label      Icon   │ CardHeader (pb-2)
├────────────────────┤
│ $10,245.50        │ Large value
│ +2.5% today       │ Secondary info
└────────────────────┘
```

### 3. Grid Layout Pattern

**4-Column Grid (Desktop)**:
```
┌──────┐ ┌──────┐ ┌──────┐ ┌──────┐
│ Card │ │ Card │ │ Card │ │ Card │
│  1   │ │  2   │ │  3   │ │  4   │
└──────┘ └──────┘ └──────┘ └──────┘

grid-cols-1 md:grid-cols-2 lg:grid-cols-4
gap-4 lg:gap-6
```

**Responsive Collapse**:
- **Mobile**: 1 column
- **Tablet**: 2 columns
- **Desktop**: 4 columns

### 4. Table Layout Pattern

**Responsive Table**:
```
Desktop:
┌────────────────────────────────────────────┐
│ Header 1 | Header 2 | Header 3 | Actions │
├────────────────────────────────────────────┤
│ Data 1   | Data 2   | Data 3   | [Action]│
│ Data 1   | Data 2   | Data 3   | [Action]│
└────────────────────────────────────────────┘

Mobile: Horizontal scroll or card view
```

---

## Responsive Breakpoints

### Tailwind Breakpoints

| Breakpoint | Min Width | Devices | Columns |
|------------|-----------|---------|---------|
| `sm` | 640px | Large phones | 2 |
| `md` | 768px | Tablets | 2-3 |
| `lg` | 1024px | Small laptops | 3-4 |
| `xl` | 1280px | Laptops | 4 |
| `2xl` | 1536px | Desktops | 4+ |

### Spacing Scale

```
p-4   = 16px  (mobile)
p-6   = 24px  (desktop)
gap-4 = 16px  (mobile)
gap-6 = 24px  (desktop)
```

### Typography Scale

```
Mobile → Desktop
text-2xl → text-3xl  (Headings)
text-sm → text-base  (Body)
text-xs → text-sm    (Labels)
```

---

## Component Hierarchy

### Page Structure

```
Page Component
├── DashboardHeader (Navigation)
│   ├── Logo
│   ├── Search
│   ├── Notifications
│   └── UserMenu
├── Main Content
│   ├── Section 1 (Card)
│   │   ├── CardHeader
│   │   └── CardContent
│   ├── Section 2 (Grid)
│   │   ├── Card
│   │   ├── Card
│   │   └── Card
│   └── Section 3 (Table)
│       └── Data Table
└── ChatBot (Floating)
```

### Dashboard Page Hierarchy

```
Dashboard
├── DashboardHeader
├── BotStatus (Card with metrics grid)
├── TradingCharts (Lazy loaded)
│   └── Multiple ChartCard components
├── AI Section
│   ├── AIStrategySelector (Card)
│   └── AISignals (Card with signal list)
├── PerformanceChart (Lazy loaded)
│   └── Recharts LineChart
├── TransactionHistory
│   └── Table with trade rows
└── ChatBot (Lazy loaded)
```

### TradingPaper Page Hierarchy

```
TradingPaper
├── DashboardHeader
├── Control Panel
│   ├── Status indicators
│   └── Action buttons
├── Tabs
│   ├── Tab: Tổng quan
│   │   ├── Portfolio Overview (Grid of metric cards)
│   │   └── Performance Chart (Recharts)
│   ├── Tab: Tín hiệu AI
│   │   └── AISignals (Card list)
│   ├── Tab: Lịch sử giao dịch
│   │   ├── Open Trades Table
│   │   └── Closed Trades Table
│   └── Tab: Cài đặt
│       ├── Basic Settings (Form)
│       ├── Symbol Settings (Dialog)
│       └── TradingSettings (Embedded)
├── Trade Details Dialog (Popup on row click)
├── Symbol Config Dialog
└── ChatBot
```

---

## Wireframe Key

### Symbols
- `[Button]` - Clickable button
- `[Input]` - Text input field
- `[Dropdown ▼]` - Select dropdown
- `☑` - Checkbox (checked)
- `☐` - Checkbox (unchecked)
- `🟢` - Green indicator (active/profit)
- `🔴` - Red indicator (inactive/loss)
- `🟡` - Yellow indicator (warning)
- `🔵` - Blue indicator (info)

### Layout Boxes
- Solid lines: Primary containers
- Dashed lines: Optional/conditional elements
- `━━━` Separator: Section divider

### Interactive Elements
- `[Action]` - Primary action button
- `[Cancel]` - Secondary/cancel button
- `[×]` - Close/remove button
- `[≡]` - Menu/hamburger icon

---

## Additional Wireframes

### 7. ChatBot Widget

**File**: `nextjs-ui-dashboard/src/components/ChatBot.tsx`
**FR Reference**: FR-DASHBOARD-006 (AI Chatbot Interface)

**Collapsed State**:
```
Bottom-right corner:
    ┌──────┐
    │  💬  │  Floating button
    └──────┘
```

**Expanded State**:
```
┌────────────────────────────────────────┐
│ 🤖 AI Assistant          🔊 ▢ ×       │ Header
│ Trading Bot Helper                     │ (Gradient bg)
├────────────────────────────────────────┤
│                                        │
│ ┌────────────────────────────────────┐│
│ │ 🤖 Chào bạn! Tôi là AI Assistant..││ Scroll
│ │    10:30 AM                        ││ Area
│ └────────────────────────────────────┘│
│                                        │
│           ┌──────────────────────┐    │
│           │ Bot hoạt động thế nào?│   │ User
│           │    10:31 AM          │   │ Message
│           └──────────────────────┘    │
│                                        │
│ ┌────────────────────────────────────┐│
│ │ Bot sử dụng AI để phân tích...    ││ Bot
│ │    10:31 AM                        ││ Response
│ └────────────────────────────────────┘│
│                                        │
│ ● ● ●  (Typing indicator)             │
│                                        │
│ ✨ Câu hỏi gợi ý:                     │ Suggested
│ [❓ Bot hoạt động thế nào?]          │ Questions
│ [❓ Cách cài đặt strategy?]          │
├────────────────────────────────────────┤
│ [Hỏi về trading bot...] [🗑️] [📤]   │ Input
│ 🤖 AI • Vietnamese        🟢 Online   │ Footer
└────────────────────────────────────────┘
```

**Dimensions**:
- Width: 384px (w-96)
- Height: 600px (expanded), 64px (minimized)
- Position: fixed bottom-4 right-4 z-50

---

### 8. AI Signal Detail Dialog

**Triggered by**: Clicking on AI signal card
**Component**: AISignals.tsx DetailedSignalDialog

```
┌─────────────────────────────────────────────────────────────┐
│ 📊 Detailed AI Analysis: BTCUSDT          [LONG]  🟢        │
├─────────────────────────────────────────────────────────────┤
│                                                             │
│ ┌──────────────┐  ┌──────────────┐  ┌──────────────┐     │
│ │🎯 Signal     │  │🔼 Recomm.    │  │🛡️ Risk       │     │
│ │  Strength    │  │              │  │  Level       │     │
│ │  82.0%       │  │  BUY (LONG)  │  │  Medium      │     │
│ │  Confidence  │  │  📈 Giá tăng │  │  Overall     │     │
│ └──────────────┘  └──────────────┘  └──────────────┘     │
│                                                             │
│ ┌───────────────────────────────────────────────────────┐ │
│ │ 📈 Market Analysis                                    │ │
│ │ ┌────────────────┐  ┌────────────────┐              │ │
│ │ │ Trend: Bullish │  │ Strength: 82%  │              │ │
│ │ │ Volatility:    │  │ Volume: High   │              │ │
│ │ │   Medium       │  │                 │              │ │
│ │ └────────────────┘  └────────────────┘              │ │
│ └───────────────────────────────────────────────────────┘ │
│                                                             │
│ ┌───────────────────────────────────────────────────────┐ │
│ │ ⚙️ Strategy Analysis   Click để xem chi tiết          │ │
│ │ ┌─────────────────────────────────────────────────┐  │ │
│ │ │ RSI Strategy        ℹ️              82%        │  │ │ Clickable
│ │ │ ████████████████░░░░                           │  │ │ → Opens
│ │ │ Click để xem giải thích chi tiết về RSI        │  │ │ Strategy
│ │ └─────────────────────────────────────────────────┘  │ │ Explanation
│ │ ┌─────────────────────────────────────────────────┐  │ │
│ │ │ MACD Strategy       ℹ️              75%        │  │ │
│ │ │ ███████████████░░░░░                           │  │ │
│ │ └─────────────────────────────────────────────────┘  │ │
│ └───────────────────────────────────────────────────────┘ │
│                                                             │
│ ┌───────────────────────────────────────────────────────┐ │
│ │ 🛡️ Risk Assessment                                    │ │
│ │ Technical Risk: 50%  |  Market Risk: 50%             │ │
│ │ Position Size: 2%    |  Source: websocket            │ │
│ │                                                       │ │
│ │ Trading Levels:                                       │ │
│ │ Stop Loss: $26,500   |  Take Profit: $28,500        │ │
│ └───────────────────────────────────────────────────────┘ │
│                                                             │
│ ┌───────────────────────────────────────────────────────┐ │
│ │ ℹ️ Analysis Reasoning                                 │ │
│ │ Strong bullish momentum detected. RSI shows oversold  │ │
│ │ conditions with potential reversal. MACD golden cross │ │
│ │ indicates upward trend. High volume confirms buying   │ │
│ │ pressure. Recommended entry for long position.        │ │
│ │                                                       │ │
│ │ Generated: 10:30 AM | Status: Active                 │ │
│ └───────────────────────────────────────────────────────┘ │
│                                                             │
└─────────────────────────────────────────────────────────────┘
```

---

### 9. Strategy Explanation Dialog

**Triggered by**: Clicking on strategy score in AI signal detail
**Component**: AISignals.tsx StrategyExplanationDialog

```
┌─────────────────────────────────────────────────────────────┐
│ ⚙️ Giải thích Strategy: RSI Strategy                        │
├─────────────────────────────────────────────────────────────┤
│                                                             │
│ ┌───────────────────────────────────────────────────────┐ │
│ │ 📖 Mô tả Strategy                                     │ │
│ │ Relative Strength Index - Xác định điều kiện quá     │ │
│ │ mua/quá bán                                           │ │
│ │                                                       │ │
│ │ 🔧 Cách hoạt động:                                    │ │
│ │ RSI dao động từ 0-100. Trên 70 = quá mua, dưới 30   │ │
│ │ = quá bán                                             │ │
│ └───────────────────────────────────────────────────────┘ │
│                                                             │
│ ┌─────────────────┐  ┌─────────────────┐                 │
│ │ 🟢 Tín hiệu MUA │  │ 🔴 Tín hiệu BÁN │                 │
│ │ RSI < 30 và     │  │ RSI > 70 và     │                 │
│ │ bắt đầu tăng    │  │ bắt đầu giảm    │                 │
│ └─────────────────┘  └─────────────────┘                 │
│                                                             │
│ ┌─────────────────┐  ┌─────────────────┐                 │
│ │ ✅ Ưu điểm      │  │ ⚠️ Nhược điểm   │                 │
│ │ • Dễ hiểu       │  │ • Lag signal    │                 │
│ │ • Hiệu quả      │  │ • False signal  │                 │
│ │   sideway       │  │   trong trend   │                 │
│ └─────────────────┘  └─────────────────┘                 │
│                                                             │
│ ┌───────────────────────────────────────────────────────┐ │
│ │ ⚙️ Thông tin sử dụng                                  │ │
│ │ 🕐 Timeframe tốt nhất: 1h, 4h, 1d                     │ │
│ │ 📊 Mô tả biểu đồ: Đường RSI dao động với vùng quá    │ │
│ │    mua (70+) và quá bán (30-)                         │ │
│ └───────────────────────────────────────────────────────┘ │
│                                                             │
│ ┌───────────────────────────────────────────────────────┐ │
│ │ 📈 Hình minh họa                                      │ │
│ │ ┌─────────────────────────────────────────────────┐  │ │
│ │ │      RSI Indicator (14)                         │  │ │
│ │ │ 70 ─── Overbought ─────────────────            │  │ │
│ │ │                                                 │  │ │
│ │ │ 50 ─── Midline ────────────────────            │  │ │
│ │ │                RSI Line                         │  │ │
│ │ │ 30 ─── Oversold ───────────────────            │  │ │
│ │ │                                                 │  │ │
│ │ │        BUY                 SELL                 │  │ │
│ │ └─────────────────────────────────────────────────┘  │ │
│ │                                                       │ │
│ │ 💡 Giải thích biểu đồ:                               │ │
│ │ • Đường xanh: RSI line dao động từ 0-100            │ │
│ │ • Vùng đỏ (70+): Overbought - có thể bán           │ │
│ │ • Vùng xanh (30-): Oversold - có thể mua           │ │
│ └───────────────────────────────────────────────────────┘ │
│                                                             │
└─────────────────────────────────────────────────────────────┘
```

**Features**:
- **Educational content**: Explains each strategy in detail
- **Visual charts**: SVG illustrations of each indicator
- **Bilingual labels**: Vietnamese with English terms
- **Color coding**: Green (buy), Red (sell), Blue (neutral)
- **Interactive**: Nested dialog from AI signal detail

---

### 10. Trading Settings Dialog

**File**: `nextjs-ui-dashboard/src/components/dashboard/TradingSettings.tsx`
**Trigger**: Settings button in various pages
**FR Reference**: FR-DASHBOARD-003

```
┌─────────────────────────────────────────────────────────────┐
│ ⚙️ Trading Strategy Settings                                │
├─────────────────────────────────────────────────────────────┤
│ [Presets] [Strategies] [Risk Management] [Engine]          │ Tabs
├─────────────────────────────────────────────────────────────┤
│                                                             │
│ ━━━━━━━━━━━━━━━━━ TAB: PRESETS ━━━━━━━━━━━━━━━━━━        │
│                                                             │
│ ┌─────────────┐  ┌─────────────┐  ┌─────────────┐        │
│ │ Low Volatil │  │ Normal Vol. │  │ High Volatil│        │
│ │ 🟢 Selected │  │             │  │             │        │
│ │             │  │             │  │             │        │
│ │ Confidence: │  │ Confidence: │  │ Confidence: │        │
│ │    45%      │  │    65%      │  │    75%      │        │
│ │ Risk: Low   │  │ Risk: Med.  │  │ Risk: High  │        │
│ │ Stop Loss:  │  │ Stop Loss:  │  │ Stop Loss:  │        │
│ │    2%       │  │    3%       │  │    5%       │        │
│ │             │  │             │  │             │        │
│ │ [Apply]     │  │ [Apply]     │  │ [Apply]     │        │
│ └─────────────┘  └─────────────┘  └─────────────┘        │
│                                                             │
│ ℹ️ Thị trường ít biến động - Sử dụng Low Volatility preset │
│                                                             │
│ ━━━━━━━━━━━━━━━━━ TAB: STRATEGIES ━━━━━━━━━━━━━━━━━━       │
│                                                             │
│ ┌───────────────────────────────────────────────────────┐ │
│ │ RSI Strategy                                 [ON] ☑  │ │
│ │ Period: [14]  5 ━━━━━━━━━━●──────── 30              │ │
│ │ Oversold: [30]  20 ━━━━●──────────── 50             │ │
│ │ Overbought: [70]  50 ──────────●──── 80             │ │
│ └───────────────────────────────────────────────────────┘ │
│                                                             │
│ ┌───────────────────────────────────────────────────────┐ │
│ │ MACD Strategy                                [ON] ☑  │ │
│ │ Fast Period: [12]  5 ━━━━━●──────── 20              │ │
│ │ Slow Period: [26]  15 ───────●─────── 35            │ │
│ │ Signal Period: [9]  3 ━━━━●────── 15                │ │
│ └───────────────────────────────────────────────────────┘ │
│                                                             │
│ [Similar sections for Volume and Bollinger Bands...]       │
│                                                             │
│ ━━━━━━━━━━━━━━━━━ TAB: RISK MANAGEMENT ━━━━━━━━━━━━━━━━   │
│                                                             │
│ Max Risk per Trade: [2.0%]  0.5 ━━●────── 5.0             │
│ Max Portfolio Risk: [20%]  5 ───────●──── 50              │
│ Stop Loss: [2.0%]  0.5 ━━●────── 5.0                      │
│ Take Profit: [5.0%]  1.0 ────●──── 10.0                   │
│ Max Leverage: [20x]  1 ───────●──── 50                    │
│ Max Drawdown: [10%]  5 ━━━●────── 25                      │
│                                                             │
│ ━━━━━━━━━━━━━━━━━ TAB: ENGINE ━━━━━━━━━━━━━━━━━━          │
│                                                             │
│ Min Confidence: [0.65]  0.3 ─────●──── 0.9                │
│ Signal Mode: [WeightedAverage ▼]                          │
│ Market Condition: [Trending ▼]                             │
│ Risk Level: [Moderate ▼]                                   │
│                                                             │
├─────────────────────────────────────────────────────────────┤
│              [Save Settings]  [Reload]                      │
└─────────────────────────────────────────────────────────────┘
```

**Features**:
- **Market presets**: One-click configurations for different market conditions
- **Strategy controls**: Enable/disable + parameter sliders
- **Real-time preview**: Shows current values as sliders move
- **Risk parameters**: Comprehensive risk management settings
- **Engine config**: Signal combination and confidence thresholds

---

## Related Documents

- **FR-DASHBOARD.md** (`/Users/dungngo97/Documents/bot-core/specs/01-requirements/1.1-functional-requirements/FR-DASHBOARD.md`) - Complete functional requirements
- **UI-COMPONENTS.md** (`UI-COMPONENTS.md`) - React component library documentation
- **UX-FLOWS.md** (`UX-FLOWS.md`) - User journey flow diagrams
- **DATA_MODELS.md** - Data structures and schemas
- **API_SPEC.md** - API endpoints and contracts

---

## Version History

| Version | Date | Author | Changes |
|---------|------|--------|---------|
| 1.0 | 2025-10-10 | Frontend Team | Initial wireframes for all 10+ screens |

---

**END OF UI-WIREFRAMES.md**
