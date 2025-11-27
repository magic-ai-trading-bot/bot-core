# Python AI Service Settings Unification Plan

**Plan ID**: 251128-PYTHON-SETTINGS
**Created**: 2025-11-28
**Status**: READY FOR IMPLEMENTATION
**Priority**: HIGH (Finance App - Settings Consistency Critical)
**Estimated Effort**: 4-6 hours

---

## Executive Summary

Fix settings flow inconsistencies between Frontend → Rust → Python AI Service. Currently Python has hardcoded indicator values and duplicate config that can drift from Rust settings. Signal thresholds are not configurable from frontend.

**CRITICAL**: This is a FINANCE APP - incorrect settings = money loss. All changes must maintain backward compatibility and include rollback strategy.

---

## Problem Analysis

### Issue 1: Hardcoded Indicator Values in Python
**Location**: `python-ai-service/main.py:799, 2502-2503`
```python
# HARDCODED: RSI period = 14 (should use config)
indicators["rsi"] = ta.momentum.rsi(df["close"], window=14).iloc[-1]

# HARDCODED: RSI calculation window = 14
gain = (delta.where(delta > 0, 0)).rolling(window=14).mean()
loss = (-delta.where(delta < 0, 0)).rolling(window=14).mean()
```

**Impact**: If Rust changes RSI period from 14 to 20, Python continues using 14, causing signals to diverge.

### Issue 2: Duplicate Configs (Rust vs Python)
**Rust Config**: `rust-core-engine/src/strategies/rsi_strategy.rs:22`
```rust
config.parameters.insert("rsi_period".to_string(), json!(14));
```

**Python Config**: `python-ai-service/config.yaml:24-32`
```yaml
technical_indicators:
  rsi_period: 14
  macd_fast: 12
  macd_slow: 26
  macd_signal: 9
  ema_periods: [9, 21, 50]
  bollinger_period: 20
  bollinger_std: 2
  volume_sma_period: 20
```

**Impact**: Two sources of truth. Can drift. No guarantee they stay synchronized.

### Issue 3: Signal Thresholds Not Exposed to Frontend
**Current**: Python only (`config_loader.py:64-66`)
```python
SIGNAL_TREND_THRESHOLD = 0.8  # Not exposed via Rust API
SIGNAL_MIN_TIMEFRAMES = 3     # Not exposed via Rust API
SIGNAL_MIN_INDICATORS = 4     # Not exposed via Rust API
```

**Frontend Settings**: `TradingSettings.tsx` - No UI for these critical params

**Impact**: Users cannot tune signal generation sensitivity. More conservative users want higher thresholds, aggressive traders want lower.

---

## Solution Architecture

### Principle: **Single Source of Truth = MongoDB**

```
┌─────────────┐      ┌──────────────┐      ┌─────────────────┐      ┌──────────────┐
│  Frontend   │─────▶│  Rust API    │─────▶│    MongoDB      │◀────│  Python AI   │
│  Settings   │      │  (Gateway)   │      │  (Source of     │     │  (Consumer)  │
│     UI      │      │              │      │    Truth)       │     │              │
└─────────────┘      └──────────────┘      └─────────────────┘      └──────────────┘
                            │                                                │
                            │                                                │
                            └────────────────────────────────────────────────┘
                                  Rust validates & persists all settings
                                  Python fetches on startup + caches
```

### Data Flow

1. **User changes settings** → Frontend sends to Rust API
2. **Rust validates** → Saves to MongoDB `strategy_settings` collection
3. **Python fetches** → On startup + periodic refresh (every 5 min)
4. **Python caches** → In-memory dict for fast access during calculations

---

## Implementation Plan

### Phase 1: Extend Rust Settings Struct (1.5 hours)

#### 1.1 Add Indicator Config to `PaperTradingSettings`

**File**: `rust-core-engine/src/paper_trading/settings.rs`

**Add new struct**:
```rust
/// Technical indicator configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IndicatorSettings {
    /// RSI period (default: 14)
    pub rsi_period: u32,

    /// MACD fast period (default: 12)
    pub macd_fast: u32,

    /// MACD slow period (default: 26)
    pub macd_slow: u32,

    /// MACD signal period (default: 9)
    pub macd_signal: u32,

    /// EMA periods (default: [9, 21, 50])
    pub ema_periods: Vec<u32>,

    /// Bollinger Bands period (default: 20)
    pub bollinger_period: u32,

    /// Bollinger Bands std deviation multiplier (default: 2.0)
    pub bollinger_std: f64,

    /// Volume SMA period (default: 20)
    pub volume_sma_period: u32,

    /// Stochastic K period (default: 14)
    pub stochastic_k_period: u32,

    /// Stochastic D period (default: 3)
    pub stochastic_d_period: u32,
}

/// Signal generation thresholds
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SignalSettings {
    /// Trend threshold percentage (default: 0.8%)
    /// Lower = more signals, Higher = more conservative
    pub trend_threshold_percent: f64,

    /// Minimum timeframes required to agree (default: 3 out of 4)
    /// 2 = 50% agreement (aggressive), 3 = 75% (balanced), 4 = 100% (conservative)
    pub min_required_timeframes: u32,

    /// Minimum indicators per timeframe (default: 4 out of 5)
    /// 3 = 60% agreement (aggressive), 4 = 80% (balanced), 5 = 100% (conservative)
    pub min_required_indicators: u32,

    /// Base confidence when signal triggered (default: 0.5)
    pub confidence_base: f64,

    /// Confidence added per agreeing timeframe (default: 0.08, max ~0.82 with 4TF)
    pub confidence_per_timeframe: f64,
}

impl Default for IndicatorSettings {
    fn default() -> Self {
        Self {
            rsi_period: 14,
            macd_fast: 12,
            macd_slow: 26,
            macd_signal: 9,
            ema_periods: vec![9, 21, 50],
            bollinger_period: 20,
            bollinger_std: 2.0,
            volume_sma_period: 20,
            stochastic_k_period: 14,
            stochastic_d_period: 3,
        }
    }
}

impl Default for SignalSettings {
    fn default() -> Self {
        Self {
            trend_threshold_percent: 0.8,
            min_required_timeframes: 3,
            min_required_indicators: 4,
            confidence_base: 0.5,
            confidence_per_timeframe: 0.08,
        }
    }
}
```

**Update `PaperTradingSettings` struct**:
```rust
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct PaperTradingSettings {
    // ... existing fields ...

    /// Technical indicator configuration
    pub indicators: IndicatorSettings,

    /// Signal generation settings
    pub signal: SignalSettings,
}
```

**Add validation**:
```rust
impl PaperTradingSettings {
    pub fn validate(&self) -> Result<()> {
        // ... existing validation ...

        // Validate indicator settings
        if self.indicators.rsi_period == 0 || self.indicators.rsi_period > 100 {
            return Err(anyhow::anyhow!("RSI period must be 1-100"));
        }

        if self.indicators.macd_fast >= self.indicators.macd_slow {
            return Err(anyhow::anyhow!("MACD fast must be < slow"));
        }

        if self.indicators.ema_periods.is_empty() {
            return Err(anyhow::anyhow!("EMA periods cannot be empty"));
        }

        // Validate signal settings
        if self.signal.trend_threshold_percent <= 0.0 || self.signal.trend_threshold_percent > 10.0 {
            return Err(anyhow::anyhow!("Trend threshold must be 0.1-10.0%"));
        }

        if self.signal.min_required_timeframes == 0 || self.signal.min_required_timeframes > 4 {
            return Err(anyhow::anyhow!("Min timeframes must be 1-4"));
        }

        if self.signal.min_required_indicators == 0 || self.signal.min_required_indicators > 5 {
            return Err(anyhow::anyhow!("Min indicators must be 1-5"));
        }

        Ok(())
    }
}
```

#### 1.2 Update Rust API to Expose Settings

**File**: `rust-core-engine/src/api/paper_trading.rs`

**Add new endpoint handler**:
```rust
/// Get indicator settings
pub async fn get_indicator_settings(
    engine: Arc<PaperTradingEngine>,
) -> Result<impl Reply, Rejection> {
    let settings = engine.get_settings().await;

    Ok(warp::reply::json(&json!({
        "indicators": settings.indicators,
        "signal": settings.signal,
    })))
}

/// Update indicator settings
pub async fn update_indicator_settings(
    engine: Arc<PaperTradingEngine>,
    request: UpdateIndicatorSettingsRequest,
) -> Result<impl Reply, Rejection> {
    let mut settings = engine.get_settings().await;
    settings.indicators = request.indicators;
    settings.signal = request.signal;

    // Validate
    settings.validate()
        .map_err(|e| warp::reject::custom(ValidationError(e.to_string())))?;

    // Save to MongoDB
    engine.update_settings(settings).await
        .map_err(|e| warp::reject::custom(DatabaseError(e.to_string())))?;

    Ok(warp::reply::json(&json!({
        "success": true,
        "message": "Indicator settings updated successfully"
    })))
}

#[derive(Debug, Deserialize)]
pub struct UpdateIndicatorSettingsRequest {
    pub indicators: IndicatorSettings,
    pub signal: SignalSettings,
}
```

**Add routes** (in paper trading routes setup):
```rust
// GET /api/paper-trading/indicator-settings
let get_indicator_settings = warp::path!("api" / "paper-trading" / "indicator-settings")
    .and(warp::get())
    .and(with_engine(engine.clone()))
    .and_then(get_indicator_settings);

// PUT /api/paper-trading/indicator-settings
let update_indicator_settings = warp::path!("api" / "paper-trading" / "indicator-settings")
    .and(warp::put())
    .and(warp::body::json())
    .and(with_engine(engine.clone()))
    .and_then(update_indicator_settings);
```

---

### Phase 2: Python Fetch Settings from Rust (2 hours)

#### 2.1 Create Settings Manager in Python

**File**: `python-ai-service/settings_manager.py` (NEW)

```python
"""
Settings manager for Python AI Service
Fetches indicator and signal settings from Rust API
"""

import httpx
import logging
from typing import Dict, Any, Optional
from datetime import datetime, timedelta
import asyncio

logger = logging.getLogger(__name__)


class SettingsManager:
    """Manages settings fetched from Rust API with caching"""

    def __init__(self, rust_api_url: str = "http://rust-core-engine:8080"):
        self.rust_api_url = rust_api_url
        self.settings_cache: Optional[Dict[str, Any]] = None
        self.last_fetch: Optional[datetime] = None
        self.cache_duration = timedelta(minutes=5)  # Refresh every 5 minutes
        self.lock = asyncio.Lock()

    async def get_settings(self, force_refresh: bool = False) -> Dict[str, Any]:
        """Get settings with caching"""
        async with self.lock:
            # Check cache
            if not force_refresh and self.settings_cache is not None:
                if self.last_fetch and datetime.now() - self.last_fetch < self.cache_duration:
                    logger.debug("✅ Using cached settings")
                    return self.settings_cache

            # Fetch from Rust API
            try:
                async with httpx.AsyncClient(timeout=10.0) as client:
                    response = await client.get(
                        f"{self.rust_api_url}/api/paper-trading/indicator-settings"
                    )
                    response.raise_for_status()

                    settings = response.json()
                    self.settings_cache = settings
                    self.last_fetch = datetime.now()

                    logger.info(f"✅ Fetched settings from Rust API: {settings}")
                    return settings

            except Exception as e:
                logger.error(f"❌ Failed to fetch settings from Rust API: {e}")

                # Fallback to defaults if no cache
                if self.settings_cache is None:
                    logger.warning("⚠️ Using default settings as fallback")
                    return self._get_default_settings()

                # Return stale cache
                logger.warning("⚠️ Using stale cached settings")
                return self.settings_cache

    def _get_default_settings(self) -> Dict[str, Any]:
        """Default settings fallback"""
        return {
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

    def get_indicator_value(self, key: str, default: Any = None) -> Any:
        """Get a specific indicator value synchronously (for use in sync code)"""
        if self.settings_cache is None:
            return default
        return self.settings_cache.get("indicators", {}).get(key, default)

    def get_signal_value(self, key: str, default: Any = None) -> Any:
        """Get a specific signal value synchronously"""
        if self.settings_cache is None:
            return default
        return self.settings_cache.get("signal", {}).get(key, default)


# Global singleton
settings_manager = SettingsManager()
```

#### 2.2 Update main.py to Use Settings Manager

**File**: `python-ai-service/main.py`

**Add import**:
```python
from settings_manager import settings_manager
```

**Add startup event**:
```python
@app.on_event("startup")
async def startup_event():
    """Fetch settings on startup"""
    logger.info("🚀 Starting Python AI Service...")

    # Fetch settings from Rust API
    settings = await settings_manager.get_settings()
    logger.info(f"📊 Loaded settings: {settings}")
```

**Replace hardcoded values** (line 799):
```python
# BEFORE (HARDCODED):
indicators["rsi"] = (
    ta.momentum.rsi(df["close"], window=14).iloc[-1]
    if len(df) >= 14
    else 50.0
)

# AFTER (DYNAMIC):
rsi_period = settings_manager.get_indicator_value("rsi_period", 14)
indicators["rsi"] = (
    ta.momentum.rsi(df["close"], window=rsi_period).iloc[-1]
    if len(df) >= rsi_period
    else 50.0
)
```

**Replace hardcoded values** (line 2502-2503):
```python
# BEFORE (HARDCODED):
if len(df) >= 14:
    delta = df["close"].diff()
    gain = (delta.where(delta > 0, 0)).rolling(window=14).mean()
    loss = (-delta.where(delta < 0, 0)).rolling(window=14).mean()

# AFTER (DYNAMIC):
rsi_period = settings_manager.get_indicator_value("rsi_period", 14)
if len(df) >= rsi_period:
    delta = df["close"].diff()
    gain = (delta.where(delta > 0, 0)).rolling(window=rsi_period).mean()
    loss = (-delta.where(delta < 0, 0)).rolling(window=rsi_period).mean()
```

**Update signal threshold usage** (lines using `SIGNAL_*` constants):
```python
# BEFORE (from config_loader.py):
from config_loader import SIGNAL_TREND_THRESHOLD, SIGNAL_MIN_TIMEFRAMES, SIGNAL_MIN_INDICATORS

# AFTER (from settings_manager):
# At top of functions that need these values:
signal_settings = await settings_manager.get_settings()
SIGNAL_TREND_THRESHOLD = signal_settings["signal"]["trend_threshold_percent"]
SIGNAL_MIN_TIMEFRAMES = signal_settings["signal"]["min_required_timeframes"]
SIGNAL_MIN_INDICATORS = signal_settings["signal"]["min_required_indicators"]
```

#### 2.3 Add Periodic Refresh Background Task

**File**: `python-ai-service/main.py`

```python
import asyncio

async def refresh_settings_periodically():
    """Background task to refresh settings every 5 minutes"""
    while True:
        try:
            await asyncio.sleep(300)  # 5 minutes
            settings = await settings_manager.get_settings(force_refresh=True)
            logger.info(f"🔄 Refreshed settings: {settings}")
        except Exception as e:
            logger.error(f"❌ Failed to refresh settings: {e}")

@app.on_event("startup")
async def startup_event():
    """Start background tasks"""
    logger.info("🚀 Starting Python AI Service...")

    # Initial fetch
    settings = await settings_manager.get_settings()
    logger.info(f"📊 Loaded settings: {settings}")

    # Start periodic refresh in background
    asyncio.create_task(refresh_settings_periodically())
```

---

### Phase 3: Update Frontend Settings UI (1.5 hours)

#### 3.1 Add Signal Settings to TypeScript Types

**File**: `nextjs-ui-dashboard/src/components/dashboard/TradingSettings.tsx`

**Add new interfaces**:
```typescript
interface IndicatorSettings {
  rsi_period: number;
  macd_fast: number;
  macd_slow: number;
  macd_signal: number;
  ema_periods: number[];
  bollinger_period: number;
  bollinger_std: number;
  volume_sma_period: number;
  stochastic_k_period: number;
  stochastic_d_period: number;
}

interface SignalSettings {
  trend_threshold_percent: number;
  min_required_timeframes: number;
  min_required_indicators: number;
  confidence_base: number;
  confidence_per_timeframe: number;
}

interface TradingStrategySettings {
  strategies: StrategySettings;
  risk: RiskSettings;
  engine: EngineSettings;
  indicators: IndicatorSettings;  // NEW
  signal: SignalSettings;         // NEW
}
```

#### 3.2 Add UI Components for Signal Settings

**Add new tab in settings panel**:
```typescript
<Tabs defaultValue="strategies" className="w-full">
  <TabsList>
    <TabsTrigger value="strategies">Strategies</TabsTrigger>
    <TabsTrigger value="risk">Risk Management</TabsTrigger>
    <TabsTrigger value="indicators">Indicators</TabsTrigger>  {/* NEW */}
    <TabsTrigger value="signals">Signal Thresholds</TabsTrigger>  {/* NEW */}
    <TabsTrigger value="engine">Engine</TabsTrigger>
  </TabsList>

  {/* ... existing tabs ... */}

  {/* NEW: Indicators Tab */}
  <TabsContent value="indicators">
    <Card>
      <CardHeader>
        <CardTitle>Technical Indicator Periods</CardTitle>
      </CardHeader>
      <CardContent className="space-y-6">
        {/* RSI Period */}
        <div className="space-y-2">
          <Label>RSI Period (default: 14)</Label>
          <Input
            type="number"
            min={5}
            max={50}
            value={settings.indicators.rsi_period}
            onChange={(e) => setSettings({
              ...settings,
              indicators: {
                ...settings.indicators,
                rsi_period: parseInt(e.target.value)
              }
            })}
          />
          <p className="text-xs text-muted-foreground">
            Number of periods for RSI calculation (5-50)
          </p>
        </div>

        {/* MACD Periods */}
        <div className="space-y-2">
          <Label>MACD Fast Period (default: 12)</Label>
          <Input
            type="number"
            min={5}
            max={30}
            value={settings.indicators.macd_fast}
            onChange={(e) => setSettings({
              ...settings,
              indicators: {
                ...settings.indicators,
                macd_fast: parseInt(e.target.value)
              }
            })}
          />
        </div>

        <div className="space-y-2">
          <Label>MACD Slow Period (default: 26)</Label>
          <Input
            type="number"
            min={15}
            max={50}
            value={settings.indicators.macd_slow}
            onChange={(e) => setSettings({
              ...settings,
              indicators: {
                ...settings.indicators,
                macd_slow: parseInt(e.target.value)
              }
            })}
          />
        </div>

        <div className="space-y-2">
          <Label>MACD Signal Period (default: 9)</Label>
          <Input
            type="number"
            min={3}
            max={20}
            value={settings.indicators.macd_signal}
            onChange={(e) => setSettings({
              ...settings,
              indicators: {
                ...settings.indicators,
                macd_signal: parseInt(e.target.value)
              }
            })}
          />
        </div>

        {/* Similar inputs for other indicators... */}
      </CardContent>
    </Card>
  </TabsContent>

  {/* NEW: Signal Thresholds Tab */}
  <TabsContent value="signals">
    <Card>
      <CardHeader>
        <CardTitle>Signal Generation Thresholds</CardTitle>
        <p className="text-sm text-muted-foreground">
          Control sensitivity of AI trading signals
        </p>
      </CardHeader>
      <CardContent className="space-y-6">
        {/* Trend Threshold */}
        <div className="space-y-2">
          <Label>Trend Threshold (%) - Current: {settings.signal.trend_threshold_percent}%</Label>
          <Slider
            min={0.1}
            max={2.0}
            step={0.1}
            value={[settings.signal.trend_threshold_percent]}
            onValueChange={([value]) => setSettings({
              ...settings,
              signal: {
                ...settings.signal,
                trend_threshold_percent: value
              }
            })}
          />
          <p className="text-xs text-muted-foreground">
            Price movement % to qualify as trend (Lower = more signals, Higher = conservative)
          </p>
        </div>

        {/* Min Timeframes */}
        <div className="space-y-2">
          <Label>Min Timeframes Required (out of 4)</Label>
          <Select
            value={settings.signal.min_required_timeframes.toString()}
            onValueChange={(value) => setSettings({
              ...settings,
              signal: {
                ...settings.signal,
                min_required_timeframes: parseInt(value)
              }
            })}
          >
            <SelectTrigger>
              <SelectValue />
            </SelectTrigger>
            <SelectContent>
              <SelectItem value="2">2 (50% - Aggressive)</SelectItem>
              <SelectItem value="3">3 (75% - Balanced)</SelectItem>
              <SelectItem value="4">4 (100% - Conservative)</SelectItem>
            </SelectContent>
          </Select>
          <p className="text-xs text-muted-foreground">
            How many timeframes must agree for signal
          </p>
        </div>

        {/* Min Indicators */}
        <div className="space-y-2">
          <Label>Min Indicators Required (out of 5)</Label>
          <Select
            value={settings.signal.min_required_indicators.toString()}
            onValueChange={(value) => setSettings({
              ...settings,
              signal: {
                ...settings.signal,
                min_required_indicators: parseInt(value)
              }
            })}
          >
            <SelectTrigger>
              <SelectValue />
            </SelectTrigger>
            <SelectContent>
              <SelectItem value="3">3 (60% - Aggressive)</SelectItem>
              <SelectItem value="4">4 (80% - Balanced)</SelectItem>
              <SelectItem value="5">5 (100% - Conservative)</SelectItem>
            </SelectContent>
          </Select>
          <p className="text-xs text-muted-foreground">
            How many indicators must agree per timeframe
          </p>
        </div>
      </CardContent>
    </Card>
  </TabsContent>
</Tabs>
```

#### 3.3 Update API Calls

**Fetch settings**:
```typescript
const fetchSettings = async () => {
  try {
    const response = await fetch(`${API_BASE}/api/paper-trading/indicator-settings`);
    const data = await response.json();
    setSettings(prevSettings => ({
      ...prevSettings,
      indicators: data.indicators,
      signal: data.signal,
    }));
  } catch (error) {
    console.error("Failed to fetch indicator settings:", error);
    toast.error("Failed to load indicator settings");
  }
};
```

**Save settings**:
```typescript
const saveSettings = async () => {
  try {
    const response = await fetch(`${API_BASE}/api/paper-trading/indicator-settings`, {
      method: 'PUT',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({
        indicators: settings.indicators,
        signal: settings.signal,
      }),
    });

    if (!response.ok) throw new Error("Failed to save");

    toast.success("Settings saved successfully");
  } catch (error) {
    console.error("Failed to save settings:", error);
    toast.error("Failed to save settings");
  }
};
```

---

### Phase 4: Deprecate Python config.yaml (30 minutes)

#### 4.1 Update config_loader.py

**File**: `python-ai-service/config_loader.py`

**Mark as deprecated**:
```python
"""
Configuration loader for Python AI Service

DEPRECATED: technical_indicators section in config.yaml
These values are now fetched from Rust API via settings_manager.py

Only server and ai_cache settings are still read from config.yaml
"""

import yaml
import logging
from pathlib import Path
from typing import Dict, Any

logger = logging.getLogger(__name__)


def load_config(config_path: str = "config.yaml") -> Dict[str, Any]:
    """Load configuration from YAML file"""
    try:
        config_file = Path(config_path)
        if not config_file.exists():
            logger.warning(f"Config file {config_path} not found, using defaults")
            return get_default_config()

        with open(config_file, 'r') as f:
            config = yaml.safe_load(f)

        logger.info(f"✅ Loaded configuration from {config_path}")

        # Warn about deprecated sections
        if "technical_indicators" in config:
            logger.warning("⚠️ DEPRECATED: technical_indicators in config.yaml. Now fetched from Rust API.")

        if "signal" in config:
            logger.warning("⚠️ DEPRECATED: signal in config.yaml. Now fetched from Rust API.")

        return config

    except Exception as e:
        logger.error(f"❌ Failed to load config: {e}, using defaults")
        return get_default_config()


def get_default_config() -> Dict[str, Any]:
    """Default configuration fallback"""
    return {
        "ai_cache": {
            "enabled": True,
            "duration_minutes": 2,
            "max_entries": 100
        },
        "server": {
            "host": "0.0.0.0",
            "port": 8000
        }
    }


# Load config on module import
CONFIG = load_config()

# Export commonly used values (non-deprecated only)
AI_CACHE_DURATION_MINUTES = CONFIG.get("ai_cache", {}).get("duration_minutes", 2)
AI_CACHE_ENABLED = CONFIG.get("ai_cache", {}).get("enabled", True)
AI_CACHE_MAX_ENTRIES = CONFIG.get("ai_cache", {}).get("max_entries", 100)

logger.info(f"📊 AI Cache Config: duration={AI_CACHE_DURATION_MINUTES}min, enabled={AI_CACHE_ENABLED}")

# DEPRECATED: These are now fetched from Rust API via settings_manager.py
# Keeping for backward compatibility during migration only
logger.warning("⚠️ DEPRECATED exports: SIGNAL_TREND_THRESHOLD, SIGNAL_MIN_TIMEFRAMES, SIGNAL_MIN_INDICATORS")
logger.warning("⚠️ Use settings_manager.get_signal_value() instead")
```

#### 4.2 Add Migration Notice to config.yaml

**File**: `python-ai-service/config.yaml`

```yaml
# DEPRECATED: technical_indicators and signal sections
# These settings are now managed via Rust API and fetched dynamically
# They will be removed in a future version
#
# DO NOT EDIT THESE VALUES - they will be ignored
# Use the frontend settings UI instead

server:
  host: "0.0.0.0"
  port: 8000
  reload: false

model:
  type: "lstm"
  # ... rest of model config ...

ai_cache:
  enabled: true
  duration_minutes: 2
  max_entries: 100

# DEPRECATED - DO NOT EDIT
# Fetched from Rust API instead
technical_indicators:
  rsi_period: 14  # IGNORED
  macd_fast: 12   # IGNORED
  # ... etc ...

# DEPRECATED - DO NOT EDIT
# Fetched from Rust API instead
signal:
  trend_threshold_percent: 0.8  # IGNORED
  # ... etc ...
```

---

## MongoDB Schema Updates

### New Collection: `strategy_settings`

```javascript
{
  _id: ObjectId("..."),
  user_id: "system",  // Or specific user ID for multi-tenant

  indicators: {
    rsi_period: 14,
    macd_fast: 12,
    macd_slow: 26,
    macd_signal: 9,
    ema_periods: [9, 21, 50],
    bollinger_period: 20,
    bollinger_std: 2.0,
    volume_sma_period: 20,
    stochastic_k_period: 14,
    stochastic_d_period: 3
  },

  signal: {
    trend_threshold_percent: 0.8,
    min_required_timeframes: 3,
    min_required_indicators: 4,
    confidence_base: 0.5,
    confidence_per_timeframe: 0.08
  },

  updated_at: ISODate("2025-11-28T10:00:00Z"),
  updated_by: "admin"
}
```

### Indexes

```javascript
db.strategy_settings.createIndex({ user_id: 1 }, { unique: true })
db.strategy_settings.createIndex({ updated_at: -1 })
```

---

## Testing Strategy

### Unit Tests

#### Rust Tests
**File**: `rust-core-engine/tests/test_settings.rs`

```rust
#[test]
fn test_indicator_settings_validation() {
    let mut settings = IndicatorSettings::default();

    // Valid settings
    assert!(settings.validate().is_ok());

    // Invalid RSI period
    settings.rsi_period = 0;
    assert!(settings.validate().is_err());

    settings.rsi_period = 200;
    assert!(settings.validate().is_err());

    // MACD fast >= slow (invalid)
    settings.rsi_period = 14;
    settings.macd_fast = 26;
    settings.macd_slow = 12;
    assert!(settings.validate().is_err());
}

#[test]
fn test_signal_settings_validation() {
    let mut settings = SignalSettings::default();

    // Invalid trend threshold
    settings.trend_threshold_percent = 0.0;
    assert!(settings.validate().is_err());

    settings.trend_threshold_percent = 15.0;
    assert!(settings.validate().is_err());

    // Invalid min timeframes
    settings.trend_threshold_percent = 0.8;
    settings.min_required_timeframes = 5;
    assert!(settings.validate().is_err());
}
```

#### Python Tests
**File**: `python-ai-service/tests/test_settings_manager.py`

```python
import pytest
from settings_manager import SettingsManager


@pytest.mark.asyncio
async def test_settings_manager_fetch():
    manager = SettingsManager(rust_api_url="http://localhost:8080")
    settings = await manager.get_settings()

    assert "indicators" in settings
    assert "signal" in settings
    assert settings["indicators"]["rsi_period"] > 0


@pytest.mark.asyncio
async def test_settings_manager_cache():
    manager = SettingsManager()

    # First fetch
    settings1 = await manager.get_settings()

    # Second fetch (should use cache)
    settings2 = await manager.get_settings()

    assert settings1 == settings2
    assert manager.last_fetch is not None


def test_settings_manager_fallback():
    manager = SettingsManager(rust_api_url="http://invalid-url:9999")

    # Should not have cache
    assert manager.settings_cache is None

    # Should return defaults
    rsi_period = manager.get_indicator_value("rsi_period", 14)
    assert rsi_period == 14
```

### Integration Tests

**Test Settings Flow End-to-End**:
```bash
# 1. Update settings via Rust API
curl -X PUT http://localhost:8080/api/paper-trading/indicator-settings \
  -H "Content-Type: application/json" \
  -d '{
    "indicators": {
      "rsi_period": 20,
      "macd_fast": 10,
      "macd_slow": 30,
      "macd_signal": 8,
      "ema_periods": [12, 26, 50],
      "bollinger_period": 25,
      "bollinger_std": 2.5,
      "volume_sma_period": 25,
      "stochastic_k_period": 15,
      "stochastic_d_period": 4
    },
    "signal": {
      "trend_threshold_percent": 0.5,
      "min_required_timeframes": 2,
      "min_required_indicators": 3,
      "confidence_base": 0.6,
      "confidence_per_timeframe": 0.1
    }
  }'

# 2. Verify stored in MongoDB
mongo bot-core --eval 'db.strategy_settings.findOne()'

# 3. Restart Python service (should fetch new settings)
docker restart bot-python-ai

# 4. Check Python logs for settings fetch
docker logs bot-python-ai | grep "Loaded settings"

# 5. Verify Python uses new values
# Send AI analysis request and check if RSI calculation uses period=20
curl -X POST http://localhost:8000/ai/analyze \
  -H "Content-Type: application/json" \
  -d '{
    "symbol": "BTCUSDT",
    "timeframe": "1h",
    "candles": [...]
  }'
```

---

## Rollback Strategy

### If Issues Detected After Deployment

#### Step 1: Revert Python to Use config.yaml
```python
# In main.py, comment out settings_manager usage temporarily:
# settings = await settings_manager.get_settings()

# Use hardcoded defaults:
rsi_period = 14  # Hardcoded fallback
```

#### Step 2: Revert Rust API Changes
- Remove new endpoints `/api/paper-trading/indicator-settings`
- MongoDB data remains (no data loss)

#### Step 3: Frontend Rollback
- Hide new tabs (Indicators, Signal Thresholds)
- No API calls to removed endpoints

### Backward Compatibility During Migration

- Python continues to work with `config.yaml` if Rust API unavailable
- Settings manager has fallback to defaults
- No breaking changes to existing APIs
- MongoDB migration is additive only (no deletions)

---

## Security Considerations

### 1. Settings Validation
- **Rust validates ALL incoming settings** before persisting
- Invalid ranges rejected (e.g., RSI period > 100)
- MACD constraints enforced (fast < slow)

### 2. Access Control
- Settings endpoints require authentication (JWT token)
- Only admin users can modify global settings
- Rate limiting on settings updates (max 10/minute)

### 3. Audit Trail
- Log all settings changes with timestamp + user
- Store previous values in `settings_history` collection
- Alert if suspicious changes (e.g., trend threshold set to 0.01%)

---

## Performance Considerations

### 1. Caching Strategy
- **Python**: In-memory cache, 5-minute TTL
- **Rust**: Settings loaded once on startup
- **MongoDB**: Indexed by `user_id` for fast lookup

### 2. Settings Refresh Impact
- Python fetches asynchronously (non-blocking)
- Uses stale cache if fetch fails
- No performance impact on signal generation

### 3. MongoDB Load
- Settings updates rare (user-initiated only)
- Read-heavy workload (Python reads every 5 min)
- Index on `user_id` ensures O(1) lookup

---

## Migration Timeline

### Day 1: Phase 1 + 2 (Rust + Python)
- [ ] 09:00-10:30: Extend Rust settings struct (1.5h)
- [ ] 10:30-12:30: Python settings manager (2h)
- [ ] 12:30-13:30: Unit tests (Rust + Python)

### Day 2: Phase 3 + 4 (Frontend + Deprecation)
- [ ] 09:00-10:30: Frontend UI components (1.5h)
- [ ] 10:30-11:00: Deprecate config.yaml (30min)
- [ ] 11:00-12:00: Integration tests
- [ ] 12:00-13:00: Manual testing + bug fixes

### Day 3: Deployment
- [ ] 09:00-10:00: Deploy to staging
- [ ] 10:00-11:00: Smoke tests on staging
- [ ] 11:00-12:00: Deploy to production
- [ ] 12:00-13:00: Monitor logs + verify settings flow

---

## Success Criteria

### ✅ Functional
- [ ] Frontend can update indicator periods (RSI, MACD, etc.)
- [ ] Frontend can update signal thresholds (trend %, min TF, min indicators)
- [ ] Python fetches settings from Rust API on startup
- [ ] Python refreshes settings every 5 minutes automatically
- [ ] Settings persist to MongoDB correctly
- [ ] Python uses dynamic values (no hardcoded 14 for RSI)

### ✅ Non-Functional
- [ ] All unit tests pass (Rust + Python + Frontend)
- [ ] Integration tests pass (end-to-end settings flow)
- [ ] Backward compatible (Python works with config.yaml as fallback)
- [ ] Performance: No impact on AI analysis latency (<5s)
- [ ] Security: Settings validation enforced
- [ ] Audit: All changes logged with user + timestamp

---

## Risks & Mitigations

### Risk 1: Python Cannot Reach Rust API
**Likelihood**: Medium
**Impact**: High (Python cannot fetch settings)
**Mitigation**:
- Fallback to default settings if fetch fails
- Use stale cache if Rust API temporarily down
- Health check endpoint to verify connectivity

### Risk 2: Invalid Settings Cause Trading Errors
**Likelihood**: Low (validation prevents)
**Impact**: CRITICAL (money loss)
**Mitigation**:
- Comprehensive validation in Rust (before persist)
- Frontend pre-validation (prevent invalid values)
- Dry-run mode to test settings before applying

### Risk 3: Settings Drift During Migration
**Likelihood**: Low
**Impact**: Medium (inconsistent signals)
**Mitigation**:
- Deploy all 3 components simultaneously (Rust, Python, Frontend)
- Use feature flag to enable new settings flow
- Monitor logs for settings mismatch warnings

---

## Unresolved Questions

### Q1: Should settings be per-user or global?
**Current Plan**: Global (single `strategy_settings` doc)
**Alternative**: Per-user (multi-tenant support)
**Decision Needed**: Confirm with product owner

### Q2: Should we keep config.yaml for non-trading settings?
**Current Plan**: Yes (server, model, ai_cache remain)
**Alternative**: Move everything to MongoDB
**Recommendation**: Keep config.yaml for static infra settings

### Q3: Celery task for periodic refresh or FastAPI background task?
**Current Plan**: FastAPI background task (`asyncio.create_task`)
**Alternative**: Celery beat task (separate worker)
**Tradeoff**: FastAPI = simpler, Celery = more robust for production

### Q4: Should we validate indicator combinations (e.g., MACD fast < slow)?
**Current Plan**: Yes (in Rust validation)
**Edge Cases**: EMA periods order, Bollinger std > 0, etc.
**Decision**: Add comprehensive validation rules in Phase 1

---

## File Locations Summary

### New Files
- `python-ai-service/settings_manager.py` (NEW - 150 lines)
- `rust-core-engine/tests/test_settings.rs` (NEW - 100 lines)
- `python-ai-service/tests/test_settings_manager.py` (NEW - 80 lines)

### Modified Files
- `rust-core-engine/src/paper_trading/settings.rs` (+200 lines)
- `rust-core-engine/src/api/paper_trading.rs` (+100 lines)
- `python-ai-service/main.py` (~15 locations, replace hardcoded values)
- `python-ai-service/config_loader.py` (deprecation warnings)
- `python-ai-service/config.yaml` (add deprecation notices)
- `nextjs-ui-dashboard/src/components/dashboard/TradingSettings.tsx` (+300 lines UI)

### Database
- `bot-core.strategy_settings` collection (NEW)
- Indexes: `{ user_id: 1 }`, `{ updated_at: -1 }`

---

## Appendix: Code Snippets

### A.1 MongoDB Upsert Settings (Rust)

```rust
// In paper_trading_engine.rs
pub async fn update_settings(&self, settings: PaperTradingSettings) -> Result<()> {
    let collection = self.db.collection::<Document>("strategy_settings");

    let doc = doc! {
        "user_id": "system",
        "indicators": bson::to_document(&settings.indicators)?,
        "signal": bson::to_document(&settings.signal)?,
        "updated_at": DateTime::now(),
        "updated_by": "admin",
    };

    let filter = doc! { "user_id": "system" };
    let update = doc! { "$set": doc };

    collection
        .update_one(filter, update, UpdateOptions::builder().upsert(true).build())
        .await?;

    Ok(())
}
```

### A.2 Python Indicator Calculation with Dynamic Period

```python
async def calculate_technical_indicators(
    df: pd.DataFrame,
    settings: Dict[str, Any]
) -> Dict[str, float]:
    """Calculate indicators using settings from Rust API"""

    indicators = {}
    indicator_config = settings.get("indicators", {})

    # RSI (dynamic period)
    rsi_period = indicator_config.get("rsi_period", 14)
    if len(df) >= rsi_period:
        indicators["rsi"] = ta.momentum.rsi(df["close"], window=rsi_period).iloc[-1]
    else:
        indicators["rsi"] = 50.0

    # MACD (dynamic periods)
    macd_fast = indicator_config.get("macd_fast", 12)
    macd_slow = indicator_config.get("macd_slow", 26)
    macd_signal = indicator_config.get("macd_signal", 9)

    if len(df) >= macd_slow:
        macd_line = ta.trend.macd(df["close"], window_slow=macd_slow, window_fast=macd_fast)
        macd_sig = ta.trend.macd_signal(df["close"], window_slow=macd_slow, window_fast=macd_fast, window_sign=macd_signal)
        indicators["macd"] = macd_line.iloc[-1]
        indicators["macd_signal"] = macd_sig.iloc[-1]
    else:
        indicators["macd"] = 0.0
        indicators["macd_signal"] = 0.0

    return indicators
```

---

**END OF PLAN**

**Next Steps**:
1. Review plan with team
2. Get approval from tech lead
3. Create JIRA tickets for each phase
4. Begin implementation (estimated 4-6 hours total)

**Questions?** Contact: @planner-agent
