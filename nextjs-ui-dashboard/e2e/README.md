# E2E Tests with Playwright

End-to-end tests for the Crypto Trading Bot dashboard using Playwright.

## 📋 Test Coverage

### Authentication Flow (10 tests)
- Login/logout functionality
- Form validation
- Token persistence
- Protected route access
- Registration navigation

### Dashboard Flow (17 tests)
- UI rendering and layout
- Real-time chart updates
- WebSocket connection status
- Navigation between pages
- Performance metrics display
- Responsive design

### Paper Trading Flow (16 tests)
- Portfolio balance display
- Order placement (long/short positions)
- Position management (open/close)
- PnL calculations
- Trading history
- AI signal integration
- Risk validation

### Settings Flow (17 tests)
- Configuration management
- Bot enable/disable
- Risk management settings
- Strategy selection
- API configuration
- Notification preferences
- Settings persistence

**Total: 60 E2E tests**

## 🚀 Running Tests

### Prerequisites
```bash
# Install dependencies
npm install

# Install Playwright browsers (if not already installed)
npx playwright install chromium
```

### Run Commands

```bash
# Run all E2E tests (headless)
npm run test:e2e

# Run with UI mode (interactive)
npm run test:e2e:ui

# Run in headed mode (see browser)
npm run test:e2e:headed

# Debug tests
npm run test:e2e:debug

# Run specific test file
npx playwright test e2e/auth.spec.ts

# Run tests matching pattern
npx playwright test --grep "login"

# View test report
npm run test:e2e:report
```

## 🛠️ Configuration

Test configuration is in `playwright.config.ts`:

- **Base URL**: `http://localhost:3000`
- **Browser**: Chromium (can add Firefox, Safari)
- **Retries**: 2 on CI, 0 locally
- **Timeout**: 30s per test
- **Screenshots**: On failure
- **Videos**: On failure
- **Traces**: On first retry

## 📝 Writing Tests

### Test Structure
```typescript
import { test, expect } from '@playwright/test';
import { login } from './helpers';

test.describe('Feature Name', () => {
  test.beforeEach(async ({ page }) => {
    // Setup before each test
    await login(page);
  });

  test('should do something', async ({ page }) => {
    // Test implementation
    await expect(page.getByText('Something')).toBeVisible();
  });
});
```

### Using Helpers
```typescript
import { login, logout, navigateTo, waitForCharts } from './helpers';

// Login
await login(page, 'user@example.com', 'password');

// Navigate to page
await navigateTo(page, 'trading');

// Wait for charts to load
await waitForCharts(page);

// Logout
await logout(page);
```

## 🔍 Best Practices

1. **Use Data Attributes**: Prefer `data-testid` for stable selectors
   ```typescript
   await page.locator('[data-testid="login-button"]').click();
   ```

2. **Wait for Elements**: Use `waitFor` or `expect().toBeVisible()`
   ```typescript
   await expect(page.getByText('Dashboard')).toBeVisible({ timeout: 10000 });
   ```

3. **Isolate Tests**: Each test should be independent
   ```typescript
   test.beforeEach(async ({ page }) => {
     // Fresh state for each test
   });
   ```

4. **Handle Timing**: Use proper waits instead of fixed timeouts
   ```typescript
   // Good
   await page.waitForSelector('[data-testid="chart"]');

   // Avoid
   await page.waitForTimeout(5000);
   ```

5. **Mock External APIs**: Use `page.route()` for API mocking
   ```typescript
   await page.route('**/api/market/**', route => {
     route.fulfill({ json: mockData });
   });
   ```

## 🐛 Debugging

### Debug Mode
```bash
npm run test:e2e:debug
```
Opens Playwright Inspector for step-by-step debugging.

### Screenshots & Videos
Failed tests automatically capture:
- Screenshots: `test-results/*/screenshot.png`
- Videos: `test-results/*/video.webm`
- Traces: `test-results/*/trace.zip`

### View Trace
```bash
npx playwright show-trace test-results/*/trace.zip
```

## 📊 CI/CD Integration

### GitHub Actions Example
```yaml
- name: Install Playwright
  run: npx playwright install --with-deps chromium

- name: Run E2E tests
  run: npm run test:e2e

- name: Upload test results
  if: always()
  uses: actions/upload-artifact@v3
  with:
    name: playwright-report
    path: playwright-report/
```

## 🔐 Test Data

For local testing, use these credentials:
- Email: `test@example.com`
- Password: `password123`

**Note**: These tests expect the backend services (Rust API, Python AI) to be running locally or mocked.

## 📚 Resources

- [Playwright Documentation](https://playwright.dev)
- [Best Practices](https://playwright.dev/docs/best-practices)
- [API Reference](https://playwright.dev/docs/api/class-test)
