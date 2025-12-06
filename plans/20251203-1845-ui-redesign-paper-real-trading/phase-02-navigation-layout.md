# Phase 2: Navigation & Layout System

## Context
- **Parent Plan**: [plan.md](./plan.md)
- **Dependencies**: Phase 1 (Design System)
- **Research**: [Landing & Navigation](./research/researcher-03-landing-navigation.md)

## Overview
| Field | Value |
|-------|-------|
| Priority | P0 - Critical |
| Status | Pending |
| Est. Time | 2-3 days |
| Description | Create new collapsible sidebar, header with mode indicator, and responsive layout system |

## Key Insights
- Sidebar pattern: Collapsible with icon-only mode (40% UX improvement)
- Transition: 200-300ms for smooth collapse
- Mobile: Hamburger drawer pattern
- Header: Mode indicator badge, user menu, notifications

## Requirements

### Functional
- Collapsible sidebar (expanded/collapsed/mobile drawer)
- Persistent mode indicator in header
- Breadcrumb navigation for deep pages
- Mobile-first responsive design
- Keyboard navigation support

### Non-Functional
- Sidebar transition < 300ms
- Layout shift CLS < 0.1
- Accessible (ARIA labels)

## Architecture

```
MainLayout/
├── Sidebar/
│   ├── SidebarHeader (logo, collapse button)
│   ├── SidebarNav (navigation items)
│   ├── SidebarFooter (user info, logout)
│   └── MobileSidebarDrawer
├── Header/
│   ├── ModeIndicatorBadge
│   ├── Breadcrumbs
│   ├── NotificationBell
│   └── UserMenu
└── ContentArea/
    └── {children}
```

## Related Code Files

### Create
- `/Users/dungngo97/Documents/bot-core/nextjs-ui-dashboard/src/components/layout/MainLayout.tsx`
- `/Users/dungngo97/Documents/bot-core/nextjs-ui-dashboard/src/components/layout/Sidebar.tsx`
- `/Users/dungngo97/Documents/bot-core/nextjs-ui-dashboard/src/components/layout/SidebarNav.tsx`
- `/Users/dungngo97/Documents/bot-core/nextjs-ui-dashboard/src/components/layout/Header.tsx`
- `/Users/dungngo97/Documents/bot-core/nextjs-ui-dashboard/src/components/layout/ModeIndicatorBadge.tsx`
- `/Users/dungngo97/Documents/bot-core/nextjs-ui-dashboard/src/components/layout/Breadcrumbs.tsx`
- `/Users/dungngo97/Documents/bot-core/nextjs-ui-dashboard/src/hooks/useSidebar.ts`

### Modify
- `/Users/dungngo97/Documents/bot-core/nextjs-ui-dashboard/src/App.tsx` - Wrap with MainLayout

## Implementation Steps

1. **Install shadcn/ui sidebar** (if not exists)
   ```bash
   npx shadcn-ui@latest add sidebar
   ```

2. **Create useSidebar hook**
   - State: expanded, collapsed, mobile
   - Persist preference to localStorage
   - Auto-collapse on mobile

3. **Create Sidebar component**
   - Logo with collapse animation
   - Navigation items with icons
   - Active state indicators
   - Framer Motion transitions

4. **Create Header component**
   - ModeIndicatorBadge (Paper=blue, Real=red)
   - Breadcrumbs from react-router
   - User menu dropdown
   - Mobile menu button

5. **Create MainLayout**
   - Combine Sidebar + Header + Content
   - Responsive grid layout
   - Handle mobile drawer

6. **Update App.tsx**
   - Wrap authenticated routes with MainLayout
   - Keep landing page without layout

## Todo List

- [ ] Install sidebar component from shadcn/ui
- [ ] Create useSidebar hook with localStorage persistence
- [ ] Create Sidebar component with collapse animation
- [ ] Create navigation items configuration
- [ ] Create Header with mode badge
- [ ] Create Breadcrumbs component
- [ ] Create MainLayout wrapper
- [ ] Update App.tsx routing
- [ ] Add mobile responsive styles
- [ ] Test keyboard navigation
- [ ] Write component tests

## Success Criteria

- [ ] Sidebar collapses smoothly in < 300ms
- [ ] Mode indicator visible at all times
- [ ] Mobile drawer works on touch devices
- [ ] Keyboard navigation (Tab, Enter, Escape)
- [ ] Layout persists across page navigation
- [ ] No layout shift (CLS < 0.1)

## Risk Assessment

| Risk | Impact | Mitigation |
|------|--------|------------|
| Layout shift on route change | Medium | Use consistent layout wrapper |
| Mobile drawer z-index conflicts | Low | Use proper stacking context |
| Sidebar state sync issues | Low | Single source of truth in context |

## Security Considerations
- User info in sidebar from authenticated context only
- No sensitive data in localStorage sidebar state

## Next Steps
→ Phase 3: Landing Page
