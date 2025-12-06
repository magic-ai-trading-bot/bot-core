# Phase 7: Settings & Profile Pages

## Context
- **Parent Plan**: [plan.md](./plan.md)
- **Dependencies**: Phase 1-6
- **Research**: [Landing & Navigation](./research/researcher-03-landing-navigation.md)

## Overview
| Field | Value |
|-------|-------|
| Priority | P1 - High |
| Status | Pending |
| Est. Time | 2-3 days |
| Description | Settings page with trading preferences, API keys, notifications, and user profile page |

## Key Insights
- Settings organized by category (tabs/sections)
- Critical settings require confirmation
- Profile page with avatar, stats, achievements
- API key management with security focus

## Requirements

### Functional
- Trading settings (default leverage, risk limits)
- Notification preferences (email, push, in-app)
- API key management (add, revoke, permissions)
- Theme settings (dark/light, accent colors)
- User profile (avatar, display name, stats)
- Account security (2FA, sessions, password)
- Trading history export (CSV, PDF)

### Non-Functional
- Settings save immediately (no submit button)
- Sensitive data masked by default
- Profile page loads < 1s
- Export handles large datasets

## Architecture

```
SettingsPage/
├── SettingsTabs/
│   ├── TradingSettings
│   │   ├── DefaultLeverageSlider
│   │   ├── RiskLimitsForm
│   │   └── StrategyPreferences
│   ├── NotificationSettings
│   │   ├── EmailNotifications
│   │   ├── PushNotifications
│   │   └── TradingAlerts
│   ├── APIKeySettings
│   │   ├── APIKeyList
│   │   ├── AddAPIKeyDialog
│   │   └── KeyPermissionsForm
│   ├── AppearanceSettings
│   │   ├── ThemeToggle
│   │   ├── AccentColorPicker
│   │   └── LayoutPreferences
│   └── SecuritySettings
│       ├── PasswordChange
│       ├── TwoFactorSetup
│       └── ActiveSessions

ProfilePage/
├── ProfileHeader
│   ├── AvatarUpload
│   ├── DisplayName
│   └── MemberSince
├── TradingStats
│   ├── TotalTrades
│   ├── WinRate
│   ├── TotalPnL
│   └── BestTrade
├── Achievements
│   └── AchievementBadge[]
└── ActivityTimeline
```

## Related Code Files

### Create
- `/Users/dungngo97/Documents/bot-core/nextjs-ui-dashboard/src/pages/Settings.tsx`
- `/Users/dungngo97/Documents/bot-core/nextjs-ui-dashboard/src/pages/Profile.tsx`
- `/Users/dungngo97/Documents/bot-core/nextjs-ui-dashboard/src/components/settings/SettingsTabs.tsx`
- `/Users/dungngo97/Documents/bot-core/nextjs-ui-dashboard/src/components/settings/TradingSettings.tsx`
- `/Users/dungngo97/Documents/bot-core/nextjs-ui-dashboard/src/components/settings/NotificationSettings.tsx`
- `/Users/dungngo97/Documents/bot-core/nextjs-ui-dashboard/src/components/settings/APIKeySettings.tsx`
- `/Users/dungngo97/Documents/bot-core/nextjs-ui-dashboard/src/components/settings/AppearanceSettings.tsx`
- `/Users/dungngo97/Documents/bot-core/nextjs-ui-dashboard/src/components/settings/SecuritySettings.tsx`
- `/Users/dungngo97/Documents/bot-core/nextjs-ui-dashboard/src/components/profile/ProfileHeader.tsx`
- `/Users/dungngo97/Documents/bot-core/nextjs-ui-dashboard/src/components/profile/TradingStats.tsx`
- `/Users/dungngo97/Documents/bot-core/nextjs-ui-dashboard/src/components/profile/Achievements.tsx`
- `/Users/dungngo97/Documents/bot-core/nextjs-ui-dashboard/src/components/profile/ActivityTimeline.tsx`

### Modify
- `/Users/dungngo97/Documents/bot-core/nextjs-ui-dashboard/src/App.tsx` - Add routes

## Implementation Steps

1. **Create SettingsTabs**
   ```tsx
   // Vertical tabs on desktop, horizontal on mobile
   // Each tab is a separate section
   // Changes auto-save with debounce
   ```

2. **Create TradingSettings**
   - Default leverage slider (1x-125x)
   - Max position size input
   - Default stop loss percentage
   - Auto-close settings
   - Paper/Real mode preferences

3. **Create NotificationSettings**
   - Toggle switches for each notification type
   - Email preferences (frequency)
   - Trading alert thresholds
   - Sound on/off

4. **Create APIKeySettings**
   - List existing keys (masked)
   - "Add Key" button → dialog
   - Permissions checkboxes per key
   - Revoke with confirmation
   - Copy key (show once on create)

5. **Create AppearanceSettings**
   - Dark/Light/System toggle
   - Accent color picker (with presets)
   - Compact/Comfortable density
   - Chart preferences

6. **Create SecuritySettings**
   - Password change form
   - 2FA setup wizard
   - Active sessions list
   - "Log out everywhere" button

7. **Create ProfileHeader**
   - Avatar with upload/change
   - Display name editable
   - Member since date
   - Verification badge (if verified)

8. **Create TradingStats**
   - Animated counter components
   - Stats grid layout
   - Time period selector
   - Compare to average

9. **Create Achievements**
   - Badge grid
   - Locked/unlocked states
   - Progress indicators
   - Hover for details

## Todo List

- [ ] Create SettingsTabs container
- [ ] Create TradingSettings section
- [ ] Create NotificationSettings section
- [ ] Create APIKeySettings with security
- [ ] Create AppearanceSettings
- [ ] Create SecuritySettings
- [ ] Create ProfileHeader with avatar
- [ ] Create TradingStats display
- [ ] Create Achievements grid
- [ ] Create ActivityTimeline
- [ ] Add auto-save functionality
- [ ] Add export functionality (CSV/PDF)
- [ ] Responsive testing
- [ ] Write component tests

## Success Criteria

- [ ] Settings save automatically
- [ ] API keys are properly masked
- [ ] Theme changes apply immediately
- [ ] 2FA setup works correctly
- [ ] Profile stats are accurate
- [ ] Avatar upload works
- [ ] Export generates valid files

## Risk Assessment

| Risk | Impact | Mitigation |
|------|--------|------------|
| API key exposure | CRITICAL | Mask keys, show once, confirm revoke |
| Settings sync issues | Medium | Optimistic updates, error rollback |
| Large export crash | Low | Pagination, background job |

## Security Considerations
- API keys never fully visible after creation
- Password change requires current password
- 2FA requires recovery codes backup
- Session management with device info
- All sensitive actions logged

## Next Steps
→ Phase 8: 3D Visualizations
