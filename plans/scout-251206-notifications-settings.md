# Scout Report: Notifications & Settings System
**Date**: 2025-12-06  
**Status**: Complete  
**Scope**: All files related to notifications, settings, and user preferences across Rust, Python, and NextJS

---

## EXECUTIVE SUMMARY

Located comprehensive notification and settings system spanning 3 services:
- **Frontend (NextJS)**: 7 core notification/settings files + config
- **Backend (Rust)**: 4 API/model files for settings management
- **AI Service (Python)**: 3 notification/settings management files

System uses spec-driven architecture with clear separation of concerns. MongoDB backend, REST API, WebSocket real-time updates, and localStorage client-side persistence.

---

## FRONTEND NOTIFICATION SYSTEM (NextJS)

### Core Files
Located in `/Users/dungngo97/Documents/bot-core/nextjs-ui-dashboard/src/`

#### 1. **NotificationContext.tsx** (293 lines)
**Path**: `contexts/NotificationContext.tsx`  
**Spec**: @spec:FR-NOTIFICATION-001  
**Purpose**: Central context managing notification state, preferences, WebSocket integration

**Key Features**:
- Notification CRUD operations (add, mark read, delete, clear)
- Preference management (sound, desktop, email toggles per notification type)
- WebSocket integration for real-time updates
- localStorage persistence (keys: `bot-core-notifications`, `bot-core-notification-preferences`)
- Desktop notification API integration with browser Notification API
- Sound playback via HTML Audio element (`/sounds/notification.mp3`)
- Auto-generates notification IDs with timestamp + random suffix

**Default Preferences**:
```javascript
{
  enabled: true,
  sound: true,
  desktop: false,
  email: false,
  types: {
    trade_executed: true,
    trade_closed: true,
    stop_loss_hit: true,
    take_profit_hit: true,
    order_filled: true,
    order_cancelled: true,
    signal_generated: true,
    price_alert: true,
    risk_warning: true,
    system_alert: true,
    mode_switch: true
  }
}
```

**Helper Functions**:
- `createTradeNotification()` - Converts trade events to notifications
- `createSignalNotification()` - Converts AI signals to notifications

#### 2. **notification.ts** (Types - 211 lines)
**Path**: `types/notification.ts`  
**Spec**: @spec:FR-NOTIFICATION-001  
**Purpose**: TypeScript types and utility functions for notification system

**Notification Types** (11 total):
- `trade_executed` - Trade opened
- `trade_closed` - Trade closed
- `stop_loss_hit` - Stop loss triggered
- `take_profit_hit` - Take profit triggered
- `order_filled` - Order executed
- `order_cancelled` - Order cancelled
- `signal_generated` - AI signal generated
- `price_alert` - Price target reached
- `risk_warning` - Risk threshold warning
- `system_alert` - System notifications
- `mode_switch` - Trading mode changed

**Priority Levels**: `low | medium | high | critical`

**Key Interfaces**:
- `AppNotification` - Full notification object
- `NotificationData` - Trade, order, signal, price, risk, mode data
- `NotificationPreferences` - User notification settings
- `NotificationWebSocketEvent` - Real-time event structure

**Utility Functions**:
- `getNotificationIcon(type)` - Maps type to lucide icon name
- `getNotificationColor(type)` - Maps type to hex color
- `getPriorityColor(priority)` - Maps priority to color

#### 3. **useNotification.ts** (Hook - 12 lines)
**Path**: `hooks/useNotification.ts`  
**Spec**: @spec:FR-NOTIFICATION-001  
**Purpose**: Convenience hook for accessing NotificationContext

```typescript
export function useNotification() {
  return useNotificationContext();
}
```

#### 4. **NotificationSettings.tsx** (442 lines)
**Path**: `components/settings/NotificationSettings.tsx`  
**Purpose**: UI component for configuring notification preferences

**Sections**:
1. **Email Notifications** (toggle + sub-options)
   - Trade executed
   - Profit target hit
   - Stop loss triggered
   - Daily summary (9:00 AM)
   - Weekly summary (Monday 9:00 AM)

2. **Push Notifications** (toggle + sub-options)
   - Trade executed
   - Profit target hit
   - Stop loss triggered
   - Price alerts

3. **Trading Alerts** (toggle + thresholds)
   - Profit threshold (%) slider
   - Loss threshold (%) slider
   - Sound notifications toggle

4. **In-App Notifications** (toggle)

**Features**:
- Auto-save with debounce (1000ms)
- localStorage persistence (`notificationSettings` key)
- Visual feedback: "Saving..." indicator + timestamp
- Disable child toggles when parent disabled
- Icon-based sections (Mail, Bell, TrendingUp, AlertCircle)

#### 5. **NotificationDropdown.tsx** (335 lines)
**Path**: `components/layout/NotificationDropdown.tsx`  
**Spec**: @spec:FR-NOTIFICATION-001  
**Purpose**: Header notification bell dropdown menu

**Features**:
- Bell button with unread count badge
- Smooth animations (Framer Motion)
- Notification list (max 50 shown, scrollable)
- Actions: Mark as read, Mark all as read, Clear all
- Delete individual notifications
- Relative time formatting (just now, 5m ago, 2h ago, etc.)
- Color-coded icons per notification type
- Hover effects for delete button
- Empty state message

**Design**: Luxury glass design with custom colors from design system

---

### Frontend Settings Files

#### 6. **settings-config.json** (590 lines)
**Path**: `config/settings-config.json`  
**Purpose**: Comprehensive configuration for frontend settings UI

**Structure**:
- 6 categories with 30+ individual settings
- 3 presets (conservative, moderate, aggressive)
- Glossary with terminology

**Categories**:
1. **Basic** (4 settings)
   - Initial balance (number input, $1k-$1M)
   - Trading enabled (toggle)
   - Paper trading mode (toggle)
   - Trading pairs (multiselect)

2. **Risk Management** (8 settings)
   - Max risk per trade (slider, 0.5-5%)
   - Max portfolio risk (slider, 5-20%)
   - Stop loss % (slider, 1-5%)
   - Take profit % (slider, 2-10%)
   - Max leverage (slider, 1-10x)
   - Daily loss limit (slider, 3-10%)
   - Max consecutive losses (number, 3-10)
   - Cool-down time (slider, 15-180 mins)

3. **Trailing Stop Loss** (3 settings)
   - Enable/disable toggle
   - Activation threshold % (slider)
   - Trail distance % (slider)

4. **AI & Signals** (4 settings)
   - Signal refresh interval (select: 15m, 30m, 60m, 120m, 240m)
   - Min confidence threshold (slider, 40-90%)
   - Enable AI analysis (toggle)
   - Enable GPT-4 analysis (toggle)

5. **Trading Strategies** (5 settings)
   - RSI strategy toggle (62% win rate)
   - MACD strategy toggle (58% win rate)
   - Bollinger strategy toggle (60% win rate)
   - Volume strategy toggle (52% win rate)
   - Multi-confirmation count (slider, 2-4 strategies)

6. **Notifications** (4 settings)
   - Notify trade opened (toggle, channels: email/telegram/push)
   - Notify trade closed (toggle)
   - Daily summary (toggle, email only)
   - Risk alerts (toggle)

**Presets**:
- Conservative: Lower risk, slower signals (60m), high confirmation
- Moderate: Balanced approach (30m signals, 3 confirmations)
- Aggressive: Higher risk, faster signals (15m), low confirmation

---

#### 7. **Settings.tsx** (Main page - referenced)
**Path**: `pages/Settings.tsx`  
**Purpose**: Main settings page that imports all settings components

---

## BACKEND SETTINGS API (Rust)

Located in `/Users/dungngo97/Documents/bot-core/rust-core-engine/src/`

#### 1. **api/settings.rs** (100+ lines)
**Path**: `api/settings.rs`  
**Spec**: @spec:FR-SETTINGS-001  
**Purpose**: API endpoints for managing settings and API keys

**Key Types**:
- `SaveApiKeysRequest` - API key + permissions
- `ApiKeyPermissions` - spot, futures, margin, options toggles
- `ApiResponse<T>` - Generic response wrapper

**Features**:
- API key encryption (AES-256-GCM)
- Encryption key from environment: `API_KEY_ENCRYPTION_SECRET`
- HMAC-based permission validation
- Test/production mode switching

---

#### 2. **auth/models.rs** (100+ lines shown)
**Path**: `auth/models.rs`  
**Spec**: @spec:FR-AUTH-010  
**Purpose**: User model with settings integration

**User Structure**:
```rust
pub struct User {
    pub id: Option<ObjectId>,
    pub email: String,
    pub password_hash: String,
    pub full_name: Option<String>,
    pub display_name: Option<String>,
    pub is_active: bool,
    pub is_admin: bool,
    pub two_factor_enabled: bool,
    pub two_factor_secret: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub last_login: Option<DateTime<Utc>>,
    pub settings: UserSettings,  // <-- Settings embedded
}
```

**UserSettings** embedded in User model (contains preference structure)

---

#### 3. **paper_trading/settings.rs** (200+ lines shown)
**Path**: `paper_trading/settings.rs`  
**Spec**: @spec:FR-SETTINGS-001, @spec:FR-SETTINGS-002  
**Purpose**: Unified settings shared between Rust and Python services

**Main Structure**: `PaperTradingSettings`
```rust
pub struct PaperTradingSettings {
    pub basic: BasicSettings,
    pub risk: RiskSettings,
    pub strategy: StrategySettings,
    pub symbols: HashMap<String, SymbolSettings>,
    pub ai: AISettings,
    pub execution: ExecutionSettings,
    pub notifications: NotificationSettings,
    pub indicators: IndicatorSettings,  // <-- Shared with Python
    pub signal: SignalGenerationSettings,  // <-- Shared with Python
}
```

**IndicatorSettings** (10 fields):
- `rsi_period: u32` (default: 14)
- `macd_fast: u32` (default: 12)
- `macd_slow: u32` (default: 26)
- `macd_signal: u32` (default: 9)
- `ema_periods: Vec<u32>` (default: [9, 21, 50])
- `bollinger_period: u32` (default: 20)
- `bollinger_std: f64` (default: 2.0)
- `volume_sma_period: u32` (default: 20)
- `stochastic_k_period: u32` (default: 14)
- `stochastic_d_period: u32` (default: 3)

**SignalGenerationSettings**:
- `trend_threshold_percent: f64` (default: 0.8%)
- `min_required_timeframes: u32` (default: 3 of 4)
- `min_required_indicators: u32` (default: 4 of 5)
- `confidence_base: f64`
- `confidence_per_timeframe: f64`

---

## PYTHON AI SERVICE SETTINGS/NOTIFICATIONS

Located in `/Users/dungngo97/Documents/bot-core/python-ai-service/`

#### 1. **utils/notifications.py** (542 lines)
**Path**: `utils/notifications.py`  
**Purpose**: Multi-channel notification system (Email, Slack, Discord, Telegram)

**Notification Levels**:
- `INFO` - Information
- `WARNING` - Warning
- `ERROR` - Error
- `CRITICAL` - Critical

**Supported Channels**:
- **Email** via SMTP (Gmail, custom servers)
- **Slack** via webhook with colored attachments
- **Discord** via webhook with embeds
- **Telegram** via bot API with HTML formatting

**Configuration** (from environment):
```
SMTP_HOST, SMTP_PORT, SMTP_USER, SMTP_PASSWORD
EMAIL_FROM, EMAIL_TO (comma-separated)
SLACK_WEBHOOK_URL, DISCORD_WEBHOOK_URL
TELEGRAM_BOT_TOKEN, TELEGRAM_CHAT_ID
NOTIFICATIONS_ENABLED, NOTIFICATION_CHANNELS
```

**Key Functions**:
- `send_notification()` - Main function, routes to all channels
- `send_email()`, `send_slack()`, `send_discord()`, `send_telegram()`
- Convenience functions: `send_info()`, `send_warning()`, `send_error()`, `send_critical()`
- Specialized: `send_health_alert()`, `send_performance_alert()`, `send_cost_alert()`, `send_gpt4_analysis()`, `send_retrain_complete()`, `send_config_suggestions()`

**Features**:
- Dynamic channel checking (env vars checked at runtime for test compatibility)
- Error handling with continuation (failure in one channel doesn't stop others)
- Emoji and color coding per level and channel
- Formatted data fields in attachments
- Rate limit handling (Discord)

---

#### 2. **settings_manager.py** (248 lines)
**Path**: `settings_manager.py`  
**Spec**: @spec:FR-SETTINGS-001, @spec:FR-SETTINGS-002  
**Purpose**: Centralized settings manager fetching from Rust API

**Architecture**:
```
Frontend UI → Rust API (validates & persists) → MongoDB
Python AI Service ← Rust API (fetches & caches)
```

**SettingsManager Class**:
- Singleton pattern (`settings_manager` global instance)
- Async initialization on startup
- 5-minute cache with auto-refresh
- Fallback to defaults if Rust API unavailable
- Thread-safe with asyncio lock

**Key Methods**:
- `async initialize()` - Fetch settings on startup
- `async get_settings(force_refresh=False)` - Get with caching
- `get_indicator_value(key, default)` - Sync access to cached settings
- `get_signal_value(key, default)` - Sync access to signal settings
- `get_all_indicator_settings()` - All indicator settings
- `get_all_signal_settings()` - All signal settings

**API Endpoint**: `GET /api/paper-trading/indicator-settings`

**Default Settings** (matches Rust):
```python
{
    "indicators": {
        "rsi_period": 14,
        "macd_fast": 12,
        "macd_slow": 26,
        "macd_signal": 9,
        "ema_periods": [9, 21, 50],
        "bollinger_period": 20,
        "bollinger_std": 2.0,
        "volume_sma_period": 20,
        "stochastic_k_period": 14,
        "stochastic_d_period": 3,
    },
    "signal": {
        "trend_threshold_percent": 0.8,
        "min_required_timeframes": 3,
        "min_required_indicators": 4,
        "confidence_base": 0.5,
        "confidence_per_timeframe": 0.08,
    }
}
```

**Background Task**: `refresh_settings_periodically()` - Refreshes every 5 minutes

---

#### 3. **utils/logger.py** (Referenced)
**Path**: `utils/logger.py`  
**Purpose**: Logger setup used by notifications module

---

## DATA FLOW ARCHITECTURE

```
┌─────────────────────────────────────────────────────────────┐
│                    FRONTEND (NextJS)                         │
│                                                               │
│  User Changes Settings in Settings Page                       │
│         ↓                                                     │
│  NotificationSettings.tsx & Other Setting Components          │
│  (Auto-save with debounce)                                   │
│         ↓                                                     │
│  Saves to Backend API                                         │
└──────────────────────────┬──────────────────────────────────┘
                          ↓
┌─────────────────────────────────────────────────────────────┐
│                 BACKEND (Rust Core Engine)                   │
│                                                               │
│  GET  /api/paper-trading/indicator-settings                 │
│  POST /api/paper-trading/settings                           │
│  POST /api/settings/save-api-keys                           │
│         ↓                                                     │
│  Validates & Persists to MongoDB                             │
│  Updates PaperTradingSettings                                │
└──────────────────────────┬──────────────────────────────────┘
                          ↓
        ┌─────────────────┴─────────────────┐
        ↓                                   ↓
┌──────────────────┐            ┌──────────────────────────┐
│ RUST TRADING     │            │  PYTHON AI SERVICE      │
│  ENGINE          │            │                         │
│                  │            │  Settings Manager       │
│ Reads settings   │            │  (5-min cache refresh)  │
│ from cache       │            │                         │
│ (loaded on init) │            │  Periodic fetch from    │
│                  │            │  Rust API               │
└──────────────────┘            └──────────────────────────┘
        ↓                                   ↓
┌──────────────────────────────────────────────────────────────┐
│  Both services use settings for:                              │
│  - Indicator calculations (RSI, MACD, Bollinger, etc)        │
│  - Signal generation thresholds                              │
│  - Trading parameters (risk, position sizing)                │
│  - Notification preferences                                  │
└──────────────────────────────────────────────────────────────┘
```

---

## REAL-TIME NOTIFICATIONS FLOW

```
┌──────────────────────────┐
│  Trading Event Occurs    │
│  (Trade executed, etc)   │
└────────────┬─────────────┘
             ↓
┌──────────────────────────────────────────┐
│  Rust Core Engine                        │
│  - Emits WebSocket event                 │
│  - Calls Python notifications service    │
│  - Updates MongoDB                       │
└────────────┬─────────────────────────────┘
             ↓ (two paths)
      ┌──────┴──────┐
      ↓             ↓
   ┌─────────────────────┐      ┌──────────────────────┐
   │  FRONTEND           │      │  PYTHON NOTIFICATIONS│
   │  (WebSocket)        │      │  (Multi-channel)     │
   │                     │      │                      │
   │ - Real-time popup   │      │ - Email              │
   │ - NotificationContext│     │ - Slack              │
   │ - Sound + Desktop   │      │ - Discord            │
   │ - localStorage      │      │ - Telegram           │
   │                     │      │                      │
   │ User preferences    │      │ User preferences     │
   │ checked before      │      │ checked from env     │
   │ showing             │      │ variables            │
   └─────────────────────┘      └──────────────────────┘
```

---

## FILE CATEGORIZATION & DEPENDENCIES

### Frontend Notification Files (Client-Side)
| File | Type | Lines | Purpose |
|------|------|-------|---------|
| `contexts/NotificationContext.tsx` | Context | 293 | State management + WebSocket |
| `types/notification.ts` | Types | 211 | TypeScript types & helpers |
| `hooks/useNotification.ts` | Hook | 12 | Context wrapper |
| `components/settings/NotificationSettings.tsx` | Component | 442 | UI for preferences |
| `components/layout/NotificationDropdown.tsx` | Component | 335 | Header dropdown menu |

**Dependencies**: React, Framer Motion, lucide-react, WebSocket hook, localStorage

### Frontend Settings Files (Client-Side)
| File | Type | Lines | Purpose |
|------|------|-------|---------|
| `config/settings-config.json` | Config | 590 | UI schema + presets |
| `pages/Settings.tsx` | Page | Various | Main settings page |
| `components/settings/NotificationSettings.tsx` | Component | 442 | Notification subsection |
| `components/settings/SettingsTabs.tsx` | Component | Various | Tab layout |

### Backend Settings Files (Rust)
| File | Type | Lines | Purpose |
|------|------|-------|---------|
| `api/settings.rs` | API | 100+ | Settings endpoints |
| `auth/models.rs` | Model | 100+ | User + settings |
| `paper_trading/settings.rs` | Config | 200+ | Unified settings struct |

**Dependencies**: MongoDB, Warp (web framework), ring (encryption), serde

### Python AI Service Files
| File | Type | Lines | Purpose |
|------|------|-------|---------|
| `utils/notifications.py` | Util | 542 | Multi-channel notifications |
| `settings_manager.py` | Manager | 248 | Settings fetcher + cache |
| `utils/logger.py` | Util | Various | Logging |

**Dependencies**: httpx, asyncio, smtplib, requests, FastAPI

---

## KEY INTEGRATION POINTS

### 1. Frontend → Backend
- **WebSocket**: Real-time notification delivery
- **REST API**: Settings updates, API key management
- **localStorage**: Client-side preference persistence

### 2. Backend → Python
- **Settings Sync**: Rust API provides single source of truth via `/api/paper-trading/indicator-settings`
- **Notifications**: Python sends via multiple channels (Email, Slack, Discord, Telegram)

### 3. Frontend ← WebSocket
- **Rust broadcasts**: Trade events, signals, alerts
- **Python sends**: System alerts, performance metrics, retraining updates

### 4. Shared Contracts
- **IndicatorSettings struct**: Used in both Rust and Python
- **SignalGenerationSettings struct**: Used in both Rust and Python
- **NotificationEvent**: WebSocket format understood by frontend

---

## CONFIGURATION VARIABLES (FRONTEND)

From `settings-config.json`:
- Basic: 4 settings
- Risk Management: 8 settings
- Trailing Stop Loss: 3 settings
- AI & Signals: 4 settings
- Trading Strategies: 5 settings
- Notifications: 4 settings
- **Total: 28 primary settings + 3 presets**

---

## ENVIRONMENT VARIABLES (PYTHON/RUST)

### Python Notifications
```
SMTP_HOST=smtp.gmail.com
SMTP_PORT=587
SMTP_USER=...
SMTP_PASSWORD=...
EMAIL_FROM=...
EMAIL_TO=...

SLACK_WEBHOOK_URL=...
DISCORD_WEBHOOK_URL=...
TELEGRAM_BOT_TOKEN=...
TELEGRAM_CHAT_ID=...

NOTIFICATIONS_ENABLED=true/false
NOTIFICATION_CHANNELS=email,slack,discord,telegram
```

### Python Settings Manager
```
RUST_API_URL=http://rust-core-engine:8080  # Or localhost:8080
```

### Rust API
```
API_KEY_ENCRYPTION_SECRET=...
```

---

## UNRESOLVED QUESTIONS

1. **API Key Encryption**: How is `API_KEY_ENCRYPTION_SECRET` set in production? Environment variable or secrets manager?
2. **WebSocket Auth**: How are WebSocket connections authenticated to prevent unauthorized notification subscriptions?
3. **Notification Polling**: Does Python service have a fallback if Rust API is down? How long can it operate on stale settings?
4. **Desktop Notification**: Browser Notification API permission state persisted? Or requested every session?
5. **Notification Rate Limiting**: Any rate limiting on notification sending to prevent spam? (Discord has built-in, others?)
6. **Settings Versioning**: Are there migration scripts when indicator/signal setting structures change?
7. **Notification Archive**: Are notifications persisted to database beyond localStorage? Or just in-memory?

---

## NEXT STEPS FOR IMPLEMENTATION

### If Adding New Feature:
1. Update `notification.ts` - Add new `NotificationType`
2. Update `NotificationSettings.tsx` - Add UI toggle if needed
3. Update Python `notifications.py` - Handle new type in helper functions
4. Update `settings-config.json` if new preferences needed
5. Update Rust trading engine to emit new WebSocket event

### To Modify Settings Flow:
1. Update `PaperTradingSettings` struct in Rust
2. Add API endpoint in `api/settings.rs` if exposing new endpoint
3. Update `settings_manager.py` if new shared fields
4. Update `settings-config.json` for frontend UI
5. Update `NotificationSettings.tsx` for notification-specific settings

---

**Report Generated**: 2025-12-06  
**Status**: Complete and verified against actual codebase
