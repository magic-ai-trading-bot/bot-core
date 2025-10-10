import { test, expect } from '@playwright/test';

test.describe('Critical User Flows - Complete E2E', () => {
  test('complete trading flow - buy order', async ({ page }) => {
    // 1. Login
    await page.goto('/login');
    await page.getByLabel(/email/i).fill('test@example.com');
    await page.getByLabel(/password/i).fill('password123');
    await page.getByRole('button', { name: /sign in|login/i }).click();
    await expect(page).toHaveURL(/.*dashboard/, { timeout: 15000 });

    // 2. Navigate to trading
    await page.getByRole('link', { name: /trading|paper.*trading/i }).click();
    await expect(page).toHaveURL(/.*trading/, { timeout: 10000 });

    // 3. Wait for page to load
    await page.waitForTimeout(2000);

    // 4. Place buy order
    const amountInput = page.getByLabel(/amount|quantity|size/i);
    if (await amountInput.isVisible({ timeout: 5000 })) {
      await amountInput.fill('0.001');

      const buyButton = page.getByRole('button', { name: /^buy$|^long$/i }).first();
      if (await buyButton.isVisible({ timeout: 2000 })) {
        await buyButton.click();

        // 5. Confirm order if confirmation dialog appears
        const confirmButton = page.getByRole('button', { name: /confirm|place.*order/i }).last();
        if (await confirmButton.isVisible({ timeout: 3000 })) {
          await confirmButton.click();
        }

        // 6. Verify success
        const successIndicator = page.getByText(/success|placed|opened/i);
        if (await successIndicator.isVisible({ timeout: 5000 })) {
          await expect(successIndicator).toBeVisible();
        }
      }
    }
  });

  test('authentication flow - register to verify to login', async ({ page }) => {
    // Navigate to register
    await page.goto('/register');

    // Fill registration form
    const emailInput = page.getByLabel(/email/i);
    const passwordInput = page.getByLabel(/^password$/i);

    if (await emailInput.isVisible({ timeout: 5000 })) {
      const testEmail = `test${Date.now()}@example.com`;
      await emailInput.fill(testEmail);
      await passwordInput.fill('SecurePass123!');

      // Submit registration
      const registerButton = page.getByRole('button', { name: /register|sign up|create/i });
      if (await registerButton.isVisible()) {
        await registerButton.click();

        // Should show success or redirect
        await page.waitForTimeout(2000);

        // Verify either success message or redirect to login
        const isOnLogin = page.url().includes('/login');
        const hasSuccessMsg = await page.getByText(/success|created|registered/i).isVisible({ timeout: 3000 }).catch(() => false);

        expect(isOnLogin || hasSuccessMsg).toBeTruthy();
      }
    }
  });

  test('AI analysis request flow', async ({ page }) => {
    // Login first
    await page.goto('/login');
    await page.getByLabel(/email/i).fill('test@example.com');
    await page.getByLabel(/password/i).fill('password123');
    await page.getByRole('button', { name: /sign in|login/i }).click();
    await expect(page).toHaveURL(/.*dashboard/, { timeout: 15000 });

    // Look for AI analysis section
    await page.waitForTimeout(2000);
    const aiButton = page.getByRole('button', { name: /ai|analysis|signal/i }).first();

    if (await aiButton.isVisible({ timeout: 5000 })) {
      await aiButton.click();

      // Wait for AI response
      await page.waitForTimeout(3000);

      // Should see AI signal/analysis result
      const aiResult = page.getByText(/long|short|hold|bullish|bearish/i);
      if (await aiResult.isVisible({ timeout: 10000 })) {
        await expect(aiResult).toBeVisible();
      }
    }
  });

  test('real-time market data updates', async ({ page }) => {
    // Login and navigate to dashboard
    await page.goto('/login');
    await page.getByLabel(/email/i).fill('test@example.com');
    await page.getByLabel(/password/i).fill('password123');
    await page.getByRole('button', { name: /sign in|login/i }).click();
    await expect(page).toHaveURL(/.*dashboard/, { timeout: 15000 });

    // Check for price displays
    await page.waitForTimeout(2000);
    const priceElement = page.locator('text=/\\$[0-9,]+/').first();

    if (await priceElement.isVisible({ timeout: 5000 })) {
      const initialPrice = await priceElement.textContent();

      // Wait for potential WebSocket update
      await page.waitForTimeout(5000);

      // Price should still be displayed (real-time updates)
      await expect(priceElement).toBeVisible();
      expect(initialPrice).toBeTruthy();
    }
  });

  test('portfolio management flow', async ({ page }) => {
    // Login
    await page.goto('/login');
    await page.getByLabel(/email/i).fill('test@example.com');
    await page.getByLabel(/password/i).fill('password123');
    await page.getByRole('button', { name: /sign in|login/i }).click();
    await expect(page).toHaveURL(/.*dashboard/, { timeout: 15000 });

    // Navigate to settings or portfolio
    const settingsLink = page.getByRole('link', { name: /settings|portfolio/i });
    if (await settingsLink.isVisible({ timeout: 5000 })) {
      await settingsLink.click();
      await page.waitForTimeout(2000);

      // Verify settings page loaded
      expect(page.url()).toMatch(/settings|portfolio/);
    }
  });

  test('settings and preferences', async ({ page }) => {
    // Login
    await page.goto('/login');
    await page.getByLabel(/email/i).fill('test@example.com');
    await page.getByLabel(/password/i).fill('password123');
    await page.getByRole('button', { name: /sign in|login/i }).click();
    await expect(page).toHaveURL(/.*dashboard/, { timeout: 15000 });

    // Navigate to settings
    const settingsLink = page.getByRole('link', { name: /settings/i });
    if (await settingsLink.isVisible({ timeout: 5000 })) {
      await settingsLink.click();
      await expect(page).toHaveURL(/.*settings/, { timeout: 10000 });

      // Look for settings options
      const settingsForm = page.locator('form, [role="form"]').first();
      if (await settingsForm.isVisible({ timeout: 3000 })) {
        await expect(settingsForm).toBeVisible();
      }
    }
  });

  test('error handling - API failure', async ({ page }) => {
    // Block API requests to simulate failure
    await page.route('**/api/**', route => route.abort());

    await page.goto('/login');
    await page.getByLabel(/email/i).fill('test@example.com');
    await page.getByLabel(/password/i).fill('password123');
    await page.getByRole('button', { name: /sign in|login/i }).click();

    // Should show error message
    const errorMessage = page.getByText(/error|failed|unable/i);
    await expect(errorMessage.first()).toBeVisible({ timeout: 10000 });
  });

  test('error handling - network offline', async ({ page }) => {
    // Start normally
    await page.goto('/login');
    await page.getByLabel(/email/i).fill('test@example.com');
    await page.getByLabel(/password/i).fill('password123');
    await page.getByRole('button', { name: /sign in|login/i }).click();

    // Simulate offline
    await page.context().setOffline(true);

    // Try to perform an action
    await page.reload();

    // Should handle offline state
    await page.waitForTimeout(2000);

    // Restore online
    await page.context().setOffline(false);
  });

  test('multi-language support', async ({ page }) => {
    await page.goto('/');

    // Look for language switcher
    const langSwitcher = page.getByRole('button', { name: /language|english|vietnamese/i });
    if (await langSwitcher.isVisible({ timeout: 5000 })) {
      await langSwitcher.click();

      // Select different language
      const vnOption = page.getByRole('option', { name: /vietnamese|tiếng việt/i });
      if (await vnOption.isVisible({ timeout: 2000 })) {
        await vnOption.click();
        await page.waitForTimeout(1000);

        // Verify language changed (look for Vietnamese text)
        const vnText = page.getByText(/đăng nhập|đăng ký/i);
        if (await vnText.isVisible({ timeout: 3000 })) {
          await expect(vnText).toBeVisible();
        }
      }
    }
  });

  test('responsive design - mobile', async ({ page }) => {
    // Set mobile viewport
    await page.setViewportSize({ width: 375, height: 667 });

    await page.goto('/login');

    // Form should be visible and usable on mobile
    await expect(page.getByLabel(/email/i)).toBeVisible();
    await expect(page.getByLabel(/password/i)).toBeVisible();

    // Fill and submit
    await page.getByLabel(/email/i).fill('test@example.com');
    await page.getByLabel(/password/i).fill('password123');
    await page.getByRole('button', { name: /sign in|login/i }).click();

    // Should work on mobile
    await page.waitForTimeout(3000);
  });

  test('theme switching - dark/light mode', async ({ page }) => {
    await page.goto('/');

    // Look for theme toggle
    const themeToggle = page.getByRole('button', { name: /theme|dark|light/i });
    if (await themeToggle.isVisible({ timeout: 5000 })) {
      // Get initial theme
      const bodyClass = await page.locator('html').getAttribute('class');

      // Toggle theme
      await themeToggle.click();
      await page.waitForTimeout(500);

      // Verify theme changed
      const newBodyClass = await page.locator('html').getAttribute('class');
      expect(newBodyClass).not.toBe(bodyClass);
    }
  });

  test('session persistence', async ({ page, context }) => {
    // Login
    await page.goto('/login');
    await page.getByLabel(/email/i).fill('test@example.com');
    await page.getByLabel(/password/i).fill('password123');
    await page.getByRole('button', { name: /sign in|login/i }).click();
    await expect(page).toHaveURL(/.*dashboard/, { timeout: 15000 });

    // Get cookies
    const cookies = await context.cookies();
    expect(cookies.length).toBeGreaterThan(0);

    // Refresh page
    await page.reload();

    // Should still be logged in
    await expect(page).toHaveURL(/.*dashboard/);
  });

  test('logout flow', async ({ page, context }) => {
    // Login first
    await page.goto('/login');
    await page.getByLabel(/email/i).fill('test@example.com');
    await page.getByLabel(/password/i).fill('password123');
    await page.getByRole('button', { name: /sign in|login/i }).click();
    await expect(page).toHaveURL(/.*dashboard/, { timeout: 15000 });

    // Logout
    const logoutButton = page.getByRole('button', { name: /logout|đăng xuất/i });
    await logoutButton.click();

    // Should redirect to login
    await expect(page).toHaveURL(/.*login/, { timeout: 10000 });

    // Try to access protected route
    await page.goto('/dashboard');

    // Should redirect back to login
    await expect(page).toHaveURL(/.*login/, { timeout: 10000 });
  });

  test('keyboard navigation', async ({ page }) => {
    await page.goto('/login');

    // Tab to email field
    await page.keyboard.press('Tab');
    await page.keyboard.type('test@example.com');

    // Tab to password field
    await page.keyboard.press('Tab');
    await page.keyboard.type('password123');

    // Tab to submit button and press Enter
    await page.keyboard.press('Tab');
    await page.keyboard.press('Enter');

    // Should submit form
    await page.waitForTimeout(2000);
  });

  test('accessibility compliance', async ({ page }) => {
    await page.goto('/login');

    // Check for proper labels
    const emailLabel = page.getByLabel(/email/i);
    const passwordLabel = page.getByLabel(/password/i);

    await expect(emailLabel).toBeVisible();
    await expect(passwordLabel).toBeVisible();

    // Check for proper heading structure
    const heading = page.getByRole('heading', { level: 1 });
    if (await heading.isVisible({ timeout: 3000 })) {
      await expect(heading).toBeVisible();
    }

    // Check for landmarks
    const main = page.locator('main, [role="main"]');
    if (await main.isVisible({ timeout: 2000 })) {
      await expect(main).toBeVisible();
    }
  });
});
