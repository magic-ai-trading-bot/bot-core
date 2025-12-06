# Implementation Plan: Missing Features

## Executive Summary

After comprehensive audit, the following features have UI but are **NOT fully functional**:

| Feature | Status | Priority | Effort |
|---------|--------|----------|--------|
| Real Trading Page | API exists, engine disabled | FUTURE | - |
| Profile Update | ✅ FIXED - Connected to API | DONE | - |
| Avatar Upload | ✅ FIXED - Base64 upload to API | DONE | - |
| TradingSettings Component | ✅ FIXED - Connected to API | DONE | - |
| In-App Notifications | ✅ Works with localStorage | DONE | - |
| Demo Video (Landing) | ✅ FIXED - Product Preview | DONE | - |

---

## 1. Real Trading Page

### Current State
- **Frontend**: Shows "Coming Soon" overlay at `src/pages/RealTrading.tsx:1267`
- **Backend**: Full API exists at `/api/real-trading/*` in `rust-core-engine/src/api/real_trading.rs`
- **Issue**: Engine is `None` in main.rs line 181 - returns "Real trading service is not configured"

### Root Cause
```rust
// main.rs:181
None, // Real trading engine - configure via ENABLE_REAL_TRADING env var
```

### Implementation Options

#### Option A: Enable Real Trading (Requires Binance API Keys)
1. Set `ENABLE_REAL_TRADING=true` in environment
2. Configure valid Binance API keys with trading permissions
3. Remove Coming Soon overlay from frontend
4. Add proper safety warnings and confirmations

#### Option B: Keep as Coming Soon (Safer)
1. Keep overlay but improve messaging
2. Explain that Paper Trading is recommended first
3. Add contact form for users wanting real trading access

### Recommendation
**Option B** for now - Real trading with real money needs extensive testing and legal considerations.

---

## 2. Profile Update (Display Name)

### Current State
- **Frontend**: `src/pages/Profile.tsx:91` - Has TODO comment
- **Backend**: API EXISTS at `PUT /api/auth/profile` in `security_handlers.rs:276`

### Implementation

```typescript
// Profile.tsx - Replace TODO with actual API call
const handleSave = async () => {
  try {
    const response = await fetch(`${API_BASE}/api/auth/profile`, {
      method: 'PUT',
      headers: {
        'Content-Type': 'application/json',
        'Authorization': `Bearer ${token}`,
      },
      body: JSON.stringify({ display_name: tempName }),
    });

    if (response.ok) {
      setDisplayName(tempName);
      setIsEditing(false);
      toast.success('Profile updated successfully');
    }
  } catch (error) {
    toast.error('Failed to update profile');
  }
};
```

### Effort: LOW (1-2 hours)

---

## 3. Avatar Upload

### Current State
- **Frontend**: `src/pages/Profile.tsx:102` - Has TODO comment
- **Backend**: NO file upload endpoint exists

### Implementation Options

#### Option A: Base64 in Profile (Simple)
1. Convert image to base64 on client
2. Store in user profile document
3. Limit size (e.g., 500KB max)

#### Option B: File Storage (Better for scale)
1. Add file upload endpoint
2. Store in filesystem or cloud storage (S3, Cloudflare R2)
3. Return URL, save in profile

### Recommendation
Start with **Option A** for simplicity, migrate to Option B later if needed.

### Effort: MEDIUM (4-6 hours)

---

## 4. TradingSettings Component

### Current State
- **Component**: `src/components/settings/TradingSettings.tsx:50-67`
- **Issue**: Uses localStorage instead of API
- **Backend**: `/api/paper-trading/settings` API exists

### Implementation

```typescript
// TradingSettings.tsx - Connect to real API
useEffect(() => {
  const loadSettings = async () => {
    try {
      const response = await fetch(`${API_BASE}/api/paper-trading/settings`);
      const data = await response.json();
      if (data.success) {
        setSettings(mapApiToLocal(data.data));
      }
    } catch (error) {
      console.error('Failed to load settings:', error);
    }
  };
  loadSettings();
}, []);

const saveSettings = async (newSettings: TradingSettingsData) => {
  try {
    await fetch(`${API_BASE}/api/paper-trading/settings`, {
      method: 'PUT',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify(mapLocalToApi(newSettings)),
    });
    setLastSaved(new Date());
  } catch (error) {
    console.error('Failed to save settings:', error);
  }
};
```

### Effort: LOW (2-3 hours)

---

## 5. In-App Notifications

### Current State
- **Frontend**: `src/contexts/NotificationContext.tsx:270` - TODO for backend API
- **Status**: ✅ FUNCTIONAL with localStorage persistence (survives page refresh)
- **What works**: Empty state UI, localStorage persistence, WebSocket real-time updates, sound/desktop notifications
- **What's missing**: Backend API for cross-browser/device persistence (optional enhancement)

### Implementation

#### Backend (Rust)
1. Create `notifications` collection in MongoDB
2. Add endpoints:
   - `GET /api/notifications` - List user notifications
   - `POST /api/notifications/mark-read` - Mark as read
   - `DELETE /api/notifications/:id` - Delete notification

#### Frontend
1. Fetch notifications on mount
2. Real-time updates via WebSocket
3. Store locally with sync to backend

### Effort: MEDIUM (6-8 hours)

---

## 6. Demo Video (Landing Page)

### Current State
- **Location**: `src/components/landing/HeroSection.tsx:154-159`
- **Shows**: "Demo Video Coming Soon" placeholder

### Options
1. **Create demo video** - Record screen capture of dashboard
2. **Remove placeholder** - Hide until video is ready
3. **Add screenshot carousel** - Show static images instead

### Recommendation
Remove placeholder for now, add video later.

### Effort: LOW (remove) / HIGH (create video)

---

## Implementation Priority

### Phase 1: Quick Wins (1-2 days) - ✅ COMPLETED
1. [x] Profile Update - Connected to existing API (`PUT /api/auth/profile`)
2. [x] TradingSettings - Connected to paper trading API (`/api/paper-trading/basic-settings`)
3. [x] Demo Video - Replaced with interactive Product Preview modal
4. [x] In-App Notifications - Already works with localStorage persistence

### Phase 2: Medium Effort (3-5 days) - ✅ COMPLETED
4. [x] Avatar Upload - Base64 approach (backend + frontend implemented)
5. [x] In-App Notifications - Already works with localStorage (backend API optional)

### Phase 3: Future Consideration
6. [ ] Real Trading - Enable when ready for production

---

## Files to Modify

### Phase 1
- `src/pages/Profile.tsx` - Add API call for profile update
- `src/components/settings/TradingSettings.tsx` - Connect to API
- `src/components/landing/HeroSection.tsx` - Fix placeholder

### Phase 2
- `src/pages/Profile.tsx` - Add avatar upload logic
- `src/contexts/NotificationContext.tsx` - Add API integration
- `rust-core-engine/src/api/mod.rs` - Add notifications routes
- `rust-core-engine/src/api/notifications.rs` - Add list/read endpoints

---

## Summary

**Most critical**: Profile update and TradingSettings are easy fixes with existing APIs.

**Medium priority**: In-app notifications need backend work but improve UX significantly.

**Low priority**: Avatar upload and demo video are nice-to-have.

**Intentionally disabled**: Real Trading - keep Coming Soon until ready for production with proper risk management.
