# Settings Management - Functional Requirements

**Spec ID**: FR-SETTINGS
**Version**: 1.0
**Status**: ✅ Implemented
**Owner**: Backend Team
**Last Updated**: 2026-02-06

---

## Overview

This specification defines functional requirements for the Settings Management System, which provides unified configuration management across Rust Trading Engine and Python AI Service. The system ensures consistent indicator calculations, signal generation thresholds, and trading parameters.

**Key Capabilities**:
- Unified indicator settings (RSI, MACD, EMA, Bollinger Bands, Stochastic)
- Unified signal generation thresholds
- Settings persistence to database
- Settings validation and migration
- API endpoints for settings management
- Real-time settings updates without restart

**Shared Configuration**:
Settings are shared between:
- Rust Trading Engine (uses settings for strategy execution)
- Python AI Service (uses settings for signal generation)

---

## Requirements

### FR-SETTINGS-001: Unified Indicator Settings

**Priority**: High
**Status**: ✅ Implemented

**Description**:
The system shall provide unified technical indicator configuration shared between Rust Trading Engine and Python AI Service to ensure consistent calculations.

**Acceptance Criteria**:
- [ ] RSI period (default: 14, range: 5-50)
- [ ] MACD fast period (default: 12)
- [ ] MACD slow period (default: 26, must be > fast)
- [ ] MACD signal period (default: 9)
- [ ] EMA periods (default: [9, 21, 50])
- [ ] Bollinger Bands period (default: 20)
- [ ] Bollinger Bands std deviation (default: 2.0, range: 1.0-4.0)
- [ ] Volume SMA period (default: 20)
- [ ] Stochastic %K period (default: 14)
- [ ] Stochastic %D period (default: 3)
- [ ] Validate ranges on update
- [ ] Persist to database
- [ ] Apply changes without service restart

**Code Location**:
- `rust-core-engine/src/paper_trading/settings.rs:30-79` (IndicatorSettings struct)
- `rust-core-engine/src/paper_trading/settings.rs:697-745` (validation logic)
- `rust-core-engine/src/api/paper_trading.rs:239-500` (API endpoints)
- `rust-core-engine/src/api/settings.rs:1-100` (dedicated settings API)
- `python-ai-service/settings_manager.py:1-150` (Python implementation)

**Test Cases**: TC-SETTINGS-001, TC-SETTINGS-002, TC-SETTINGS-003

**Related Design**: COMP-RUST-TRADING.md, COMP-PYTHON-ML.md, API-RUST-CORE.md

---

### FR-SETTINGS-002: Unified Signal Generation Settings

**Priority**: High
**Status**: ✅ Implemented

**Description**:
The system shall provide unified signal generation thresholds to control AI trading signals across services.

**Acceptance Criteria**:
- [ ] Trend threshold percent (default: 0.8%, range: 0.1-10.0%)
  - Price movement must exceed this to qualify as trend
  - Lower = more signals (aggressive), Higher = fewer (conservative)
- [ ] Min required timeframes (default: 3 out of 4)
  - Timeframes: 15M, 30M, 1H, 4H
  - Range: 1-4 (1=25%, 2=50%, 3=75%, 4=100% agreement)
- [ ] Min indicators per timeframe (default: 4 out of 5)
  - Indicators: MACD, RSI, Bollinger, Stochastic, Volume
  - Range: 1-5
- [ ] Volume ratio threshold (default: 1.2)
  - Volume must exceed average by this ratio to confirm trend
  - Range: 1.0-3.0
- [ ] RSI overbought/oversold thresholds (default: 70/30)
- [ ] Validate thresholds on update
- [ ] Persist to database
- [ ] Apply changes immediately

**Code Location**:
- `rust-core-engine/src/paper_trading/settings.rs:84-150` (SignalGenerationSettings struct)
- `rust-core-engine/src/paper_trading/settings.rs:746-800` (validation logic)
- `rust-core-engine/src/api/paper_trading.rs:255-520` (API endpoints)
- `python-ai-service/main.py:43-100` (signal generation logic)
- `python-ai-service/tests/test_main.py:781-2357` (multi-timeframe tests)

**Test Cases**: TC-SETTINGS-010, TC-SETTINGS-011, TC-SETTINGS-012

**Related Design**: COMP-PYTHON-ML.md, API-PYTHON-AI.md

---

### FR-SETTINGS-003: Settings Persistence

**Priority**: High
**Status**: ✅ Implemented

**Description**:
The system shall persist all settings to MongoDB for durability and cross-service access.

**Acceptance Criteria**:
- [ ] Save settings to `paper_trading_settings` collection
- [ ] Store as single document per user
- [ ] Include timestamp (updated_at)
- [ ] Support atomic updates (prevent race conditions)
- [ ] Handle save failures gracefully
- [ ] Log all save operations
- [ ] Validate before save
- [ ] Return confirmation on save

**Code Location**:
- `rust-core-engine/src/paper_trading/engine.rs:3462-3500` (save_settings)
- `rust-core-engine/src/api/paper_trading.rs:600-650` (API save endpoint)
- Database: `paper_trading_settings` collection

**Test Cases**: TC-SETTINGS-020, TC-SETTINGS-021

**Related Design**: DB-SCHEMA.md

---

### FR-SETTINGS-004: Settings Validation

**Priority**: Critical
**Status**: ✅ Implemented

**Description**:
The system shall validate all settings before applying to prevent invalid configurations.

**Acceptance Criteria**:
- [ ] Validate indicator ranges (e.g., RSI period 5-50)
- [ ] Validate MACD: fast < slow
- [ ] Validate Bollinger: std 1.0-4.0
- [ ] Validate signal thresholds (trend 0.1-10.0%)
- [ ] Validate min required timeframes (1-4)
- [ ] Validate min indicators per TF (1-5)
- [ ] Reject invalid settings with error message
- [ ] Log validation failures
- [ ] Return detailed validation errors

**Validation Rules**:

**Indicator Settings**:
- `rsi_period`: 5 ≤ value ≤ 50
- `macd_fast`: 5 ≤ value < macd_slow
- `macd_slow`: macd_fast < value ≤ 100
- `macd_signal`: 3 ≤ value ≤ 20
- `ema_periods`: all values 5-200, no duplicates
- `bollinger_period`: 10 ≤ value ≤ 50
- `bollinger_std`: 1.0 ≤ value ≤ 4.0
- `volume_sma_period`: 5 ≤ value ≤ 100
- `stochastic_k_period`: 5 ≤ value ≤ 30
- `stochastic_d_period`: 1 ≤ value ≤ 10

**Signal Generation Settings**:
- `trend_threshold_percent`: 0.1 ≤ value ≤ 10.0
- `min_required_timeframes`: 1 ≤ value ≤ 4
- `min_indicators_per_timeframe`: 1 ≤ value ≤ 5
- `volume_ratio_threshold`: 1.0 ≤ value ≤ 3.0
- `rsi_overbought`: 60 ≤ value ≤ 90
- `rsi_oversold`: 10 ≤ value ≤ 40

**Code Location**:
- `rust-core-engine/src/paper_trading/settings.rs:697-850` (validate method)

**Test Cases**: TC-SETTINGS-030, TC-SETTINGS-031, TC-SETTINGS-032

**Related Design**: NFR-RELIABILITY.md

---

### FR-SETTINGS-005: Settings Migration

**Priority**: Medium
**Status**: ✅ Implemented

**Description**:
The system shall migrate old settings format to new format when schema changes.

**Acceptance Criteria**:
- [ ] Detect old settings format
- [ ] Apply default values for new fields
- [ ] Preserve existing values
- [ ] Save migrated settings
- [ ] Log migration
- [ ] Handle migration failures gracefully

**Code Location**:
- `rust-core-engine/src/paper_trading/settings.rs:900-1000` (migration logic)

**Test Cases**: TC-SETTINGS-040, TC-SETTINGS-041

**Related Design**: DB-SCHEMA.md

---

### FR-SETTINGS-006: Settings API Endpoints

**Priority**: High
**Status**: ✅ Implemented

**Description**:
The system shall provide REST API endpoints for settings management.

**Acceptance Criteria**:
- [ ] GET /api/paper-trading/settings - Get current settings
- [ ] POST /api/paper-trading/settings - Update settings
- [ ] GET /api/settings/indicators - Get indicator settings only
- [ ] POST /api/settings/indicators - Update indicator settings
- [ ] GET /api/settings/signals - Get signal generation settings
- [ ] POST /api/settings/signals - Update signal settings
- [ ] All endpoints require authentication
- [ ] Return validation errors with 400 status
- [ ] Return updated settings on success
- [ ] Log all API calls

**Code Location**:
- `rust-core-engine/src/api/paper_trading.rs:239-650` (integrated endpoints)
- `rust-core-engine/src/api/settings.rs:1-200` (dedicated endpoints)

**Test Cases**: TC-SETTINGS-050, TC-SETTINGS-051, TC-SETTINGS-052

**Related Design**: API-RUST-CORE.md

---

### FR-SETTINGS-007: Default Settings

**Priority**: High
**Status**: ✅ Implemented

**Description**:
The system shall provide sensible default settings for all indicators and thresholds.

**Acceptance Criteria**:
- [ ] Provide Default trait implementation
- [ ] Use industry-standard indicator periods
- [ ] Use balanced signal thresholds (not too aggressive/conservative)
- [ ] Document rationale for each default value
- [ ] Allow overriding defaults in config file

**Default Values**:

**Indicators**:
- RSI: 14 periods (standard)
- MACD: 12/26/9 (standard)
- EMA: [9, 21, 50] (short, medium, long-term trends)
- Bollinger: 20 periods, 2.0 std (standard)
- Volume SMA: 20 periods
- Stochastic: 14/%K, 3/%D (standard)

**Signal Generation**:
- Trend threshold: 0.8% (balanced)
- Min timeframes: 3 out of 4 (75% agreement)
- Min indicators: 4 out of 5 (80% confidence)
- Volume ratio: 1.2x (moderate confirmation)
- RSI: 70/30 (overbought/oversold)

**Code Location**:
- `rust-core-engine/src/paper_trading/settings.rs:850-950` (Default impl)

**Test Cases**: TC-SETTINGS-060

**Related Design**: COMP-RUST-TRADING.md

---

### FR-SETTINGS-008: Settings Synchronization

**Priority**: High
**Status**: ✅ Implemented

**Description**:
The system shall synchronize settings between Rust and Python services via database.

**Acceptance Criteria**:
- [ ] Rust saves settings to DB
- [ ] Python reads settings from DB
- [ ] Python caches settings in memory
- [ ] Python refreshes cache periodically (every 60s)
- [ ] Changes apply immediately without restart
- [ ] Handle DB connection failures gracefully
- [ ] Log synchronization events

**Code Location**:
- `python-ai-service/settings_manager.py:1-200` (SettingsManager class)
- `python-ai-service/main.py:43-100` (settings usage)

**Test Cases**: TC-SETTINGS-070, TC-SETTINGS-071

**Related Design**: ARCH-MICROSERVICES.md, DB-SCHEMA.md

---

## Data Requirements

### Input Data

**Settings Update Request**:
```json
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
    "stochastic_d_period": 3
  },
  "signal": {
    "trend_threshold_percent": 0.8,
    "min_required_timeframes": 3,
    "min_indicators_per_timeframe": 4,
    "volume_ratio_threshold": 1.2,
    "rsi_overbought": 70,
    "rsi_oversold": 30
  }
}
```

### Output Data

**Settings Response**:
```json
{
  "user_id": "user123",
  "indicators": { ... },
  "signal": { ... },
  "basic": { ... },
  "risk": { ... },
  "strategy": { ... },
  "ai": { ... },
  "execution": { ... },
  "notifications": { ... },
  "updated_at": "2026-02-06T10:00:00Z"
}
```

**Validation Error**:
```json
{
  "error": "Validation failed",
  "details": [
    "rsi_period must be between 5 and 50, got 3",
    "macd_fast (30) must be less than macd_slow (26)"
  ]
}
```

---

## Interface Requirements

### REST API

**Base URL**: `http://localhost:8080/api`

**Authentication**: JWT Bearer token required

**Endpoints**:

1. **Get All Settings**
   - `GET /paper-trading/settings`
   - Returns: Complete settings object

2. **Update All Settings**
   - `POST /paper-trading/settings`
   - Body: Complete or partial settings
   - Returns: Updated settings

3. **Get Indicator Settings**
   - `GET /settings/indicators`
   - Returns: Indicator settings only

4. **Update Indicator Settings**
   - `POST /settings/indicators`
   - Body: Indicator settings
   - Returns: Updated settings

5. **Get Signal Settings**
   - `GET /settings/signals`
   - Returns: Signal generation settings

6. **Update Signal Settings**
   - `POST /settings/signals`
   - Body: Signal settings
   - Returns: Updated settings

### Database Schema

**Collection**: `paper_trading_settings`

**Document Structure**:
```json
{
  "_id": "ObjectId(...)",
  "user_id": "user123",
  "basic": { ... },
  "risk": { ... },
  "strategy": { ... },
  "symbols": { ... },
  "ai": { ... },
  "execution": { ... },
  "notifications": { ... },
  "indicators": {
    "rsi_period": 14,
    "macd_fast": 12,
    ...
  },
  "signal": {
    "trend_threshold_percent": 0.8,
    ...
  },
  "created_at": "2026-01-01T00:00:00Z",
  "updated_at": "2026-02-06T10:00:00Z"
}
```

**Indexes**:
- `user_id` (unique)
- `updated_at`

---

## Testing Strategy

### Unit Tests

**Target Coverage**: >95%

**Test Files**:
- `rust-core-engine/tests/test_settings.rs` - Settings validation
- `python-ai-service/tests/test_settings_manager.py` - Python integration
- `python-ai-service/tests/test_main.py` - Signal generation with settings

**Test Scenarios**:
- Validate valid settings (should pass)
- Validate invalid settings (should reject with error)
- Save settings to database
- Load settings from database
- Default values applied
- Migration from old format
- API endpoints (get, update)
- Cross-service synchronization

### Integration Tests

**Test Scenarios**:
1. Update indicator settings in Rust → Python reads → generates signals
2. Update signal thresholds → fewer/more signals generated
3. Invalid settings rejected → error returned
4. Database failure → fallback to defaults
5. Concurrent updates → last write wins

### Manual Testing

**Checklist**:
- [ ] Update RSI period → verify calculations change
- [ ] Update MACD periods → verify crossover signals change
- [ ] Update trend threshold → verify signal count changes
- [ ] Update min timeframes → verify signal strength changes
- [ ] Set invalid values → verify rejection with clear error
- [ ] Restart services → verify settings persist

---

## Traceability

### Related Requirements

- **FR-AI-001 to FR-AI-011**: AI/ML predictions (use indicator settings)
- **FR-STRATEGIES-001 to FR-STRATEGIES-009**: Trading strategies (use indicator settings)
- **FR-PAPER-001 to FR-PAPER-006**: Paper trading (uses all settings)
- **NFR-RELIABILITY-001 to NFR-RELIABILITY-012**: Reliability requirements

### Design Documents

- **COMP-RUST-TRADING.md**: Settings struct design
- **COMP-PYTHON-ML.md**: Python settings integration
- **API-RUST-CORE.md**: Settings API specification
- **DB-SCHEMA.md**: Settings collection schema
- **ARCH-MICROSERVICES.md**: Cross-service settings sync

### Test Cases

- **TC-SETTINGS-001 to TC-SETTINGS-080**: Settings test cases
- **TC-INTEGRATION-001 to TC-INTEGRATION-045**: Integration tests
- **python-ai-service/tests/test_main.py**: 30+ tests with @spec:FR-SETTINGS-002 tags

### Code Locations

All code uses `@spec:FR-SETTINGS-XXX` tags:
- `rust-core-engine/src/paper_trading/settings.rs` - Core settings (26 tags)
- `rust-core-engine/src/api/paper_trading.rs` - API endpoints (15 tags)
- `rust-core-engine/src/api/settings.rs` - Dedicated API (1 tag)
- `python-ai-service/settings_manager.py` - Python integration (1 tag)
- `python-ai-service/tests/test_main.py` - Tests (11 tags)

---

## Changelog

| Date | Version | Changes | Author |
|------|---------|---------|--------|
| 2026-02-06 | 1.0 | Initial specification created from code analysis | Claude (Fullstack Dev) |

---

## Notes

**Design Decisions**:

1. **Why Unified Settings?**
   - Ensures consistent indicator calculations across services
   - Prevents drift between Rust and Python implementations
   - Single source of truth (database)

2. **Why Database Sync?**
   - Services can restart independently
   - Settings persist across deployments
   - No need for service-to-service communication

3. **Why Validation Critical?**
   - Invalid settings cause incorrect signals
   - Incorrect signals lead to bad trades
   - Bad trades lose money (finance risk)

**Common Issues**:

1. **Settings not applying**: Check database connection, verify save succeeded
2. **Different results between services**: Verify settings synced, check cache refresh
3. **Validation errors**: Review validation rules, check ranges

**Best Practices**:

- Always validate settings before applying
- Test with both aggressive and conservative thresholds
- Monitor signal count after changing thresholds
- Document rationale for custom settings

---

**Document Status**: ✅ Complete
**Specification Version**: 1.0
**Total Requirements**: 8 (FR-SETTINGS-001 to FR-SETTINGS-008)
**Implementation Status**: ✅ All Implemented
**Last Updated**: 2026-02-06
