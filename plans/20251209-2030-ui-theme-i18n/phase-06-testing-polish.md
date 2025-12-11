# Phase 06: Testing & Polish

**Priority**: Medium | **Status**: Pending | **Est. Effort**: 4 hours

---

## Context Links

- [Main Plan](./plan.md)
- [Phase 01: Theme Infrastructure](./phase-01-theme-infrastructure.md)
- [Phase 02: Light Mode Design](./phase-02-light-mode-design.md)
- [Phase 03: i18n Restructure](./phase-03-i18n-restructure.md)
- [Phase 04: Apply Translations](./phase-04-apply-translations.md)
- [Phase 05: Theme Toggle UI](./phase-05-theme-toggle-ui.md)

---

## Overview

Final testing, accessibility validation, performance optimization, and polish. Verify all features work across browsers, devices, and assistive technologies.

---

## Key Insights

1. **Cross-browser testing** - Chrome, Firefox, Safari, Edge
2. **Mobile testing** - iOS Safari, Chrome Android
3. **Accessibility** - WCAG AA compliance required
4. **Performance** - Bundle size increase < 25KB acceptable
5. **E2E tests** - Critical user flows must have coverage

---

## Requirements

| ID | Requirement | Priority |
|----|-------------|----------|
| R1 | All test suites pass (unit + integration) | Critical |
| R2 | WCAG AA contrast validation for both themes | Critical |
| R3 | No console errors/warnings | Critical |
| R4 | Screen reader compatibility (NVDA, VoiceOver) | High |
| R5 | Bundle size < 25KB increase | High |
| R6 | Cross-browser compatibility | High |
| R7 | Mobile responsive | High |
| R8 | E2E test coverage for theme + language flows | Medium |

---

## Related Code Files

| File | Action | Purpose |
|------|--------|---------|
| `/Users/dungngo97/Documents/bot-core/nextjs-ui-dashboard/src/contexts/ThemeContext.test.tsx` | CREATE | Unit tests for ThemeContext |
| `/Users/dungngo97/Documents/bot-core/nextjs-ui-dashboard/src/contexts/LanguageContext.test.tsx` | CREATE | Unit tests for LanguageContext |
| `/Users/dungngo97/Documents/bot-core/nextjs-ui-dashboard/src/components/ThemeToggle.test.tsx` | CREATE | Unit tests for ThemeToggle |
| `/Users/dungngo97/Documents/bot-core/nextjs-ui-dashboard/src/i18n/config.test.ts` | CREATE | Tests for i18n configuration |
| `/Users/dungngo97/Documents/bot-core/nextjs-ui-dashboard/e2e/theme-language.spec.ts` | CREATE | E2E tests (if Playwright/Cypress exists) |

---

## Implementation Steps

### Step 1: Run Existing Test Suite

```bash
cd nextjs-ui-dashboard
npm run test
npm run lint
npm run type-check
```

Fix any failures before proceeding.

### Step 2: Create ThemeContext Unit Tests

```typescript
// src/contexts/ThemeContext.test.tsx
import { renderHook, act } from '@testing-library/react';
import { ThemeProvider, useTheme } from './ThemeContext';

describe('ThemeContext', () => {
  beforeEach(() => {
    localStorage.clear();
    document.documentElement.classList.remove('dark');
  });

  it('defaults to system preference when no localStorage', () => {
    const { result } = renderHook(() => useTheme(), {
      wrapper: ThemeProvider,
    });
    expect(result.current.theme).toBe('system');
  });

  it('persists theme to localStorage', () => {
    const { result } = renderHook(() => useTheme(), {
      wrapper: ThemeProvider,
    });
    act(() => result.current.setTheme('dark'));
    expect(localStorage.getItem('theme')).toBe('dark');
  });

  it('applies dark class when dark theme', () => {
    const { result } = renderHook(() => useTheme(), {
      wrapper: ThemeProvider,
    });
    act(() => result.current.setTheme('dark'));
    expect(document.documentElement.classList.contains('dark')).toBe(true);
  });

  it('throws error when useTheme called outside provider', () => {
    expect(() => renderHook(() => useTheme())).toThrow(
      'useTheme must be used within ThemeProvider'
    );
  });
});
```

### Step 3: Create LanguageContext Unit Tests

```typescript
// src/contexts/LanguageContext.test.tsx
import { renderHook, act, waitFor } from '@testing-library/react';
import { LanguageProvider, useLanguage } from './LanguageContext';

describe('LanguageContext', () => {
  beforeEach(() => {
    localStorage.clear();
  });

  it('provides supported languages list', () => {
    const { result } = renderHook(() => useLanguage(), {
      wrapper: LanguageProvider,
    });
    expect(result.current.supportedLanguages).toHaveLength(5);
    expect(result.current.supportedLanguages.map(l => l.code))
      .toEqual(['en', 'vi', 'fr', 'zh', 'ja']);
  });

  it('changes language and persists to localStorage', async () => {
    const { result } = renderHook(() => useLanguage(), {
      wrapper: LanguageProvider,
    });
    await act(async () => {
      await result.current.setLanguage('ja');
    });
    expect(localStorage.getItem('preferredLanguage')).toBe('ja');
  });
});
```

### Step 4: Create ThemeToggle Component Tests

```typescript
// src/components/ThemeToggle.test.tsx
import { render, screen, fireEvent } from '@testing-library/react';
import { ThemeProvider } from '@/contexts/ThemeContext';
import { ThemeToggle } from './ThemeToggle';

describe('ThemeToggle', () => {
  it('renders theme toggle button', () => {
    render(
      <ThemeProvider>
        <ThemeToggle />
      </ThemeProvider>
    );
    expect(screen.getByRole('button')).toBeInTheDocument();
  });

  it('opens dropdown menu on click', () => {
    render(
      <ThemeProvider>
        <ThemeToggle />
      </ThemeProvider>
    );
    fireEvent.click(screen.getByRole('button'));
    expect(screen.getByText('Light')).toBeInTheDocument();
    expect(screen.getByText('Dark')).toBeInTheDocument();
    expect(screen.getByText('System')).toBeInTheDocument();
  });

  it('has accessible aria-label', () => {
    render(
      <ThemeProvider>
        <ThemeToggle />
      </ThemeProvider>
    );
    expect(screen.getByLabelText(/theme/i)).toBeInTheDocument();
  });
});
```

### Step 5: WCAG Contrast Validation

Test critical color combinations with tools:

| Element | Light Mode | Dark Mode | Min Ratio | Tool |
|---------|-----------|-----------|-----------|------|
| Body text | #141414 on #FCFCFC | #FFFFFF on #000000 | 4.5:1 | WebAIM |
| Muted text | #727285 on #FCFCFC | #8B8BA0 on #000000 | 4.5:1 | WebAIM |
| Primary button | #FFFFFF on #1E5AE8 | #FFFFFF on #2962FF | 4.5:1 | WebAIM |
| Profit text | #0F9964 on #FCFCFC | #10B981 on #000000 | 4.5:1 | WebAIM |
| Loss text | #E8313D on #FCFCFC | #EF4444 on #000000 | 4.5:1 | WebAIM |

```bash
# DevTools check
# Open Chrome DevTools > Elements > Styles
# Click on any color swatch to see contrast ratio
```

### Step 6: Cross-Browser Testing

| Browser | Version | Desktop | Mobile | Status |
|---------|---------|---------|--------|--------|
| Chrome | Latest | Test | Android | Pending |
| Firefox | Latest | Test | - | Pending |
| Safari | Latest | Test | iOS | Pending |
| Edge | Latest | Test | - | Pending |

Test checklist per browser:
- [ ] Theme toggle works
- [ ] FOUC prevention works
- [ ] Language switch works
- [ ] localStorage persists
- [ ] CSS transitions smooth
- [ ] No visual glitches

### Step 7: Screen Reader Testing

**VoiceOver (macOS)**:
1. Enable: System Preferences > Accessibility > VoiceOver
2. Navigate to theme toggle
3. Verify: "Theme button, popup button" announced
4. Open dropdown, verify options announced
5. Select option, verify change announced

**NVDA (Windows)**:
1. Download from nvaccess.org
2. Run same verification steps

### Step 8: Performance Audit

```bash
# Check bundle size increase
npm run build
ls -la dist/assets/*.js | head -5

# Compare with baseline (before theme/i18n changes)
# Target: < 25KB total increase
```

Lighthouse audit targets:
- Performance: > 90
- Accessibility: > 95
- Best Practices: > 90

### Step 9: Write E2E Tests (If Playwright/Cypress Exists)

```typescript
// e2e/theme-language.spec.ts (Playwright example)
import { test, expect } from '@playwright/test';

test.describe('Theme and Language', () => {
  test('theme persists after reload', async ({ page }) => {
    await page.goto('/');
    await page.click('[aria-label="Theme"]');
    await page.click('text=Dark');

    // Reload
    await page.reload();

    // Verify dark class present
    const html = page.locator('html');
    await expect(html).toHaveClass(/dark/);
  });

  test('language change updates all text', async ({ page }) => {
    await page.goto('/');
    const origText = await page.textContent('h1');

    await page.click('[aria-label="Language"]');
    await page.click('text=日本語');

    const newText = await page.textContent('h1');
    expect(newText).not.toBe(origText);
  });
});
```

### Step 10: Final Polish

- [ ] Verify all 22 pages render without errors in both themes
- [ ] Check animation smoothness (60fps target)
- [ ] Remove any console.log statements
- [ ] Update design guidelines doc with new colors
- [ ] Add @spec tags to new code
- [ ] Update CLAUDE.md if needed

---

## Todo List

### Unit Tests
- [ ] Create `ThemeContext.test.tsx`
- [ ] Create `LanguageContext.test.tsx`
- [ ] Create `ThemeToggle.test.tsx`
- [ ] Create `i18n/config.test.ts`
- [ ] Run full test suite, fix failures

### Accessibility
- [ ] WCAG contrast check - light mode
- [ ] WCAG contrast check - dark mode
- [ ] VoiceOver testing
- [ ] Keyboard navigation testing
- [ ] Focus indicator visibility

### Cross-Browser
- [ ] Chrome desktop + Android
- [ ] Firefox desktop
- [ ] Safari desktop + iOS
- [ ] Edge desktop

### Performance
- [ ] Measure bundle size increase
- [ ] Run Lighthouse audit
- [ ] Verify < 25KB increase

### E2E
- [ ] Theme persistence test
- [ ] Language persistence test
- [ ] FOUC prevention test

### Documentation
- [ ] Update design guidelines
- [ ] Add @spec tags to new files
- [ ] Update any affected docs

---

## Success Criteria

- [ ] All unit tests pass (100% of new code covered)
- [ ] All E2E tests pass
- [ ] WCAG AA contrast for all text (4.5:1)
- [ ] WCAG AA contrast for UI elements (3:1)
- [ ] No console errors/warnings in any browser
- [ ] Bundle size increase < 25KB
- [ ] Lighthouse accessibility > 95
- [ ] Works in Chrome, Firefox, Safari, Edge
- [ ] Screen reader compatible

---

## Risk Assessment

| Risk | Likelihood | Impact | Mitigation |
|------|-----------|--------|-----------|
| Test failures | Medium | Medium | Fix issues before merge |
| Contrast failures | Low | High | Adjust colors, re-test |
| Browser incompatibility | Low | Medium | Use standard CSS, polyfills if needed |
| Screen reader issues | Medium | High | Manual testing, fix aria labels |

---

## Security Considerations

- **No new security surface** - testing phase only
- **No production data** - use mock/test data
- **No credentials** in test files

---

## Test Cases Summary

| Category | Count | Status |
|----------|-------|--------|
| ThemeContext | 4 | Pending |
| LanguageContext | 3 | Pending |
| ThemeToggle | 3 | Pending |
| i18n config | 3 | Pending |
| E2E theme | 2 | Pending |
| E2E language | 2 | Pending |
| **Total** | **17** | Pending |

---

## Acceptance Checklist (Before Merge)

- [ ] All CI checks pass
- [ ] Code review completed
- [ ] Manual QA on staging environment
- [ ] No regressions in existing functionality
- [ ] Documentation updated
- [ ] Performance benchmarks met
- [ ] Accessibility audit passed

---

## Next Steps

After this phase:
1. **Merge to main** - Create PR, get review, merge
2. **Deploy to staging** - Verify in staging environment
3. **Monitor** - Check error tracking for any issues
4. **Iterate** - Address any feedback or issues discovered
