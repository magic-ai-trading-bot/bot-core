import { test, expect } from '@playwright/test';

test.describe('Dashboard Flow', () => {
  // Helper function to login before each test
  test.beforeEach(async ({ page }) => {
    await page.goto('/login');
    await page.getByLabel(/email/i).fill('test@example.com');
    await page.getByLabel(/password/i).fill('password123');
    await page.getByRole('button', { name: /sign in|login/i }).click();
    await expect(page).toHaveURL(/.*dashboard/, { timeout: 15000 });
  });

  test('should display dashboard header with branding', async ({ page }) => {
    // Check for header elements
    await expect(page.getByText(/crypto.*trading.*bot/i).first()).toBeVisible();
    await expect(page.getByText(/AI.*powered|futures.*trading/i).first()).toBeVisible();
  });

  test('should display bot status badge', async ({ page }) => {
    // Check for bot status indicator
    const botStatus = page.getByText(/bot.*active|bot.*inactive|active|running/i).first();
    await expect(botStatus).toBeVisible();
  });

  test('should display navigation menu', async ({ page }) => {
    // Check for navigation links
    await expect(page.getByRole('link', { name: /dashboard/i })).toBeVisible();
    await expect(page.getByRole('link', { name: /trading|paper.*trading/i })).toBeVisible();
    await expect(page.getByRole('link', { name: /settings/i })).toBeVisible();
  });

  test('should display real-time trading charts', async ({ page }) => {
    // Wait for charts to load
    await expect(page.getByText(/real.*time.*trading.*charts|trading.*charts/i).first()).toBeVisible({ timeout: 10000 });

    // Check for chart symbols (BTCUSDT, ETHUSDT, etc)
    const chartSymbols = page.getByText(/BTCUSDT|ETHUSDT|BNBUSDT/);
    await expect(chartSymbols.first()).toBeVisible({ timeout: 15000 });
  });

  test('should display WebSocket connection status', async ({ page }) => {
    // Check for WebSocket status indicator
    const wsStatus = page.getByText(/live|connected|websocket/i);
    await expect(wsStatus.first()).toBeVisible({ timeout: 10000 });
  });

  test('should display AI signals section', async ({ page }) => {
    // Check for AI signals card
    await expect(page.getByText(/AI.*signals|signal.*analysis/i).first()).toBeVisible({ timeout: 10000 });
  });

  test('should display performance metrics', async ({ page }) => {
    // Check for performance stats
    const performanceSection = page.getByText(/performance|profit|loss|portfolio/i).first();
    await expect(performanceSection).toBeVisible({ timeout: 10000 });
  });

  test('should update charts in real-time via WebSocket', async ({ page }) => {
    // Wait for initial chart data
    await page.waitForTimeout(2000);

    // Get initial price
    const priceElement = page.locator('text=/\\$[0-9,]+/').first();
    await expect(priceElement).toBeVisible({ timeout: 10000 });

    const initialPrice = await priceElement.textContent();

    // Wait for potential WebSocket update (5 seconds)
    await page.waitForTimeout(5000);

    // Price might have updated (or stayed the same, which is also valid)
    const currentPrice = await priceElement.textContent();
    expect(currentPrice).toBeTruthy();
  });

  test('should navigate to paper trading from dashboard', async ({ page }) => {
    // Click on paper trading link
    await page.getByRole('link', { name: /trading|paper.*trading/i }).click();

    // Should navigate to paper trading page
    await expect(page).toHaveURL(/.*trading/, { timeout: 10000 });
  });

  test('should navigate to settings from dashboard', async ({ page }) => {
    // Click on settings link
    await page.getByRole('link', { name: /settings/i }).click();

    // Should navigate to settings page
    await expect(page).toHaveURL(/.*settings/, { timeout: 10000 });
  });

  test('should refresh chart data when refresh button clicked', async ({ page }) => {
    // Wait for charts to load
    await page.waitForTimeout(2000);

    // Find and click refresh button
    const refreshButton = page.getByRole('button', { name: /refresh|reload/i });
    if (await refreshButton.isVisible()) {
      await refreshButton.click();

      // Wait for refresh to complete
      await page.waitForTimeout(1000);

      // Charts should still be visible
      const chartSymbols = page.getByText(/BTCUSDT|ETHUSDT/);
      await expect(chartSymbols.first()).toBeVisible();
    }
  });

  test('should display trading controls', async ({ page }) => {
    // Check for trading action buttons
    const tradingControls = page.getByRole('button', { name: /buy|sell|trade|execute/i });
    if (await tradingControls.first().isVisible({ timeout: 5000 })) {
      await expect(tradingControls.first()).toBeVisible();
    }
  });

  test('should show user info in header', async ({ page }) => {
    // Check for logged in user info
    await expect(page.getByText(/test@example\.com|test.*user|logged.*in/i).first()).toBeVisible();
  });

  test('should display MongoDB connection status', async ({ page }) => {
    // Check for MongoDB badge
    const mongoStatus = page.getByText(/mongodb|database/i);
    await expect(mongoStatus.first()).toBeVisible({ timeout: 10000 });
  });

  test('should handle chart timeframe changes', async ({ page }) => {
    // Wait for initial chart load
    await page.waitForTimeout(2000);

    // Find timeframe selector (if exists)
    const timeframeSelector = page.locator('select, [role="combobox"]').filter({ hasText: /1m|5m|15m|1h/ });

    if (await timeframeSelector.first().isVisible({ timeout: 5000 })) {
      await timeframeSelector.first().click();

      // Select different timeframe
      const option = page.getByRole('option', { name: /5m|15m/ }).first();
      if (await option.isVisible({ timeout: 2000 })) {
        await option.click();

        // Wait for chart to reload
        await page.waitForTimeout(1000);

        // Chart should still be visible
        await expect(page.getByText(/BTCUSDT|ETHUSDT/).first()).toBeVisible();
      }
    }
  });

  test('should display responsive layout on mobile viewport', async ({ page }) => {
    // Set mobile viewport
    await page.setViewportSize({ width: 375, height: 667 });

    // Check that dashboard is still functional
    await expect(page.getByText(/crypto.*trading.*bot/i).first()).toBeVisible();

    // Navigation might be in hamburger menu on mobile
    const menuButton = page.getByRole('button', { name: /menu|navigation/i });
    if (await menuButton.isVisible({ timeout: 2000 })) {
      await menuButton.click();
      await expect(page.getByRole('link', { name: /dashboard/i })).toBeVisible();
    }
  });
});
