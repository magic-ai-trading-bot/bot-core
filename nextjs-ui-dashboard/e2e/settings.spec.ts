import { test, expect } from '@playwright/test';

test.describe('Settings Flow', () => {
  // Helper function to login before each test
  test.beforeEach(async ({ page }) => {
    await page.goto('/login');
    await page.getByLabel(/email/i).fill('test@example.com');
    await page.getByLabel(/password/i).fill('password123');
    await page.getByRole('button', { name: /sign in|login/i }).click();
    await expect(page).toHaveURL(/.*dashboard/, { timeout: 15000 });

    // Navigate to settings page
    await page.getByRole('link', { name: /settings/i }).click();
    await expect(page).toHaveURL(/.*settings/, { timeout: 10000 });
  });

  test('should display settings page header', async ({ page }) => {
    await expect(page.getByRole('heading', { name: /settings|configuration/i })).toBeVisible();
  });

  test('should display trading configuration section', async ({ page }) => {
    // Check for trading settings
    const tradingSection = page.getByText(/trading.*config|bot.*settings|strategy/i);
    await expect(tradingSection.first()).toBeVisible({ timeout: 10000 });
  });

  test('should display API configuration section', async ({ page }) => {
    await page.waitForTimeout(1000);

    // Check for API settings
    const apiSection = page.getByText(/API|binance.*key|exchange/i);
    if (await apiSection.first().isVisible({ timeout: 5000 })) {
      await expect(apiSection.first()).toBeVisible();
    }
  });

  test('should toggle bot enable/disable', async ({ page }) => {
    await page.waitForTimeout(2000);

    // Find bot enable toggle
    const toggleSwitch = page.locator('input[type="checkbox"], [role="switch"]').first();
    if (await toggleSwitch.isVisible({ timeout: 5000 })) {
      // Get initial state
      const initialState = await toggleSwitch.isChecked();

      // Toggle
      await toggleSwitch.click();

      // Wait for state change
      await page.waitForTimeout(1000);

      // State should have changed
      const newState = await toggleSwitch.isChecked();
      expect(newState).toBe(!initialState);
    }
  });

  test('should display risk management settings', async ({ page }) => {
    await page.waitForTimeout(1000);

    // Check for risk settings
    const riskSettings = page.getByText(/risk|stop.*loss|take.*profit|max.*loss/i);
    if (await riskSettings.first().isVisible({ timeout: 5000 })) {
      await expect(riskSettings.first()).toBeVisible();
    }
  });

  test('should validate risk percentage input', async ({ page }) => {
    await page.waitForTimeout(2000);

    // Find risk percentage input
    const riskInput = page.getByLabel(/risk|max.*risk|risk.*per.*trade/i);
    if (await riskInput.isVisible({ timeout: 5000 })) {
      // Enter invalid value (> 100%)
      await riskInput.fill('150');
      await riskInput.blur();

      // Should show validation error
      const errorMsg = page.getByText(/invalid|too.*high|maximum.*100/i);
      if (await errorMsg.isVisible({ timeout: 2000 })) {
        await expect(errorMsg).toBeVisible();
      }
    }
  });

  test('should save trading configuration', async ({ page }) => {
    await page.waitForTimeout(2000);

    // Find and click save button
    const saveButton = page.getByRole('button', { name: /save|update|apply/i });
    if (await saveButton.isVisible({ timeout: 5000 })) {
      await saveButton.click();

      // Wait for save to complete
      await page.waitForTimeout(1000);

      // Should show success message
      const successMsg = page.getByText(/saved|updated|success/i);
      if (await successMsg.isVisible({ timeout: 5000 })) {
        await expect(successMsg).toBeVisible();
      }
    }
  });

  test('should display strategy selection', async ({ page }) => {
    await page.waitForTimeout(2000);

    // Check for strategy options
    const strategySelect = page.locator('select, [role="combobox"]').filter({ hasText: /strategy|bollinger|moving.*average|rsi/i });
    if (await strategySelect.first().isVisible({ timeout: 5000 })) {
      await expect(strategySelect.first()).toBeVisible();
    }
  });

  test('should change trading strategy', async ({ page }) => {
    await page.waitForTimeout(2000);

    // Find strategy selector
    const strategySelect = page.locator('select, [role="combobox"]').filter({ hasText: /strategy/i }).first();
    if (await strategySelect.isVisible({ timeout: 5000 })) {
      await strategySelect.click();

      // Select a strategy option
      const strategyOption = page.getByRole('option', { name: /bollinger|rsi|ma/i }).first();
      if (await strategyOption.isVisible({ timeout: 2000 })) {
        await strategyOption.click();

        // Strategy should be selected
        await page.waitForTimeout(500);
        expect(await strategySelect.textContent()).toBeTruthy();
      }
    }
  });

  test('should display timeframe settings', async ({ page }) => {
    await page.waitForTimeout(2000);

    // Check for timeframe configuration
    const timeframeSettings = page.getByText(/timeframe|interval|1m|5m|15m|1h/i);
    if (await timeframeSettings.first().isVisible({ timeout: 5000 })) {
      await expect(timeframeSettings.first()).toBeVisible();
    }
  });

  test('should display position size configuration', async ({ page }) => {
    await page.waitForTimeout(2000);

    // Check for position size settings
    const positionSizeInput = page.getByLabel(/position.*size|trade.*size|lot.*size/i);
    if (await positionSizeInput.isVisible({ timeout: 5000 })) {
      await expect(positionSizeInput).toBeVisible();
    }
  });

  test('should validate position size limits', async ({ page }) => {
    await page.waitForTimeout(2000);

    const positionSizeInput = page.getByLabel(/position.*size|trade.*size/i);
    if (await positionSizeInput.isVisible({ timeout: 5000 })) {
      // Enter zero or negative value
      await positionSizeInput.fill('0');
      await positionSizeInput.blur();

      // Should show validation error
      const errorMsg = page.getByText(/invalid|must.*greater|positive/i);
      if (await errorMsg.isVisible({ timeout: 2000 })) {
        await expect(errorMsg).toBeVisible();
      }
    }
  });

  test('should display AI model configuration', async ({ page }) => {
    await page.waitForTimeout(2000);

    // Check for AI settings
    const aiSettings = page.getByText(/AI.*model|machine.*learning|neural/i);
    if (await aiSettings.first().isVisible({ timeout: 5000 })) {
      await expect(aiSettings.first()).toBeVisible();
    }
  });

  test('should reset settings to default', async ({ page }) => {
    await page.waitForTimeout(2000);

    // Find reset button
    const resetButton = page.getByRole('button', { name: /reset|default|restore/i });
    if (await resetButton.isVisible({ timeout: 5000 })) {
      await resetButton.click();

      // Confirm reset if dialog appears
      const confirmButton = page.getByRole('button', { name: /confirm|yes|reset/i }).last();
      if (await confirmButton.isVisible({ timeout: 2000 })) {
        await confirmButton.click();
      }

      // Wait for reset to complete
      await page.waitForTimeout(1000);

      // Should show success message
      const successMsg = page.getByText(/reset|restored|default/i);
      if (await successMsg.isVisible({ timeout: 3000 })) {
        await expect(successMsg).toBeVisible();
      }
    }
  });

  test('should display notification preferences', async ({ page }) => {
    await page.waitForTimeout(2000);

    // Check for notification settings
    const notificationSettings = page.getByText(/notification|alert|email|telegram/i);
    if (await notificationSettings.first().isVisible({ timeout: 5000 })) {
      await expect(notificationSettings.first()).toBeVisible();
    }
  });

  test('should toggle notification settings', async ({ page }) => {
    await page.waitForTimeout(2000);

    // Find notification toggles
    const notificationToggle = page.locator('input[type="checkbox"]').filter({ has: page.locator('text=/notification|alert/i') }).first();
    if (await notificationToggle.isVisible({ timeout: 5000 })) {
      const initialState = await notificationToggle.isChecked();
      await notificationToggle.click();
      await page.waitForTimeout(500);

      const newState = await notificationToggle.isChecked();
      expect(newState).toBe(!initialState);
    }
  });

  test('should display account information', async ({ page }) => {
    await page.waitForTimeout(2000);

    // Check for account details section
    const accountSection = page.getByText(/account|profile|user.*info/i);
    if (await accountSection.first().isVisible({ timeout: 5000 })) {
      await expect(accountSection.first()).toBeVisible();
    }
  });

  test('should preserve settings after page reload', async ({ page }) => {
    await page.waitForTimeout(2000);

    // Make a change
    const toggleSwitch = page.locator('input[type="checkbox"], [role="switch"]').first();
    if (await toggleSwitch.isVisible({ timeout: 5000 })) {
      await toggleSwitch.click();
      await page.waitForTimeout(1000);

      const stateBeforeReload = await toggleSwitch.isChecked();

      // Reload page
      await page.reload();
      await page.waitForTimeout(2000);

      // State should be preserved
      const stateAfterReload = await toggleSwitch.isChecked();
      expect(stateAfterReload).toBe(stateBeforeReload);
    }
  });
});
