import { test, expect } from '@playwright/test';

/**
 * Visual Regression Tests
 *
 * These tests capture screenshots of key pages and compare them against
 * baseline images to detect unintended visual changes.
 *
 * Tag: @visual
 * Run with: npm run test:visual
 */

test.describe('Visual Regression Tests @visual', () => {
  test('Landing page visual snapshot', async ({ page }) => {
    await page.goto('/');
    await page.waitForLoadState('networkidle');

    // Wait for hero 3D animation to load
    await page.waitForSelector('canvas', { timeout: 10000 });

    expect(await page.screenshot({ fullPage: true })).toMatchSnapshot('landing-page.png');
  });

  test('Login page visual snapshot', async ({ page }) => {
    await page.goto('/login');
    await page.waitForLoadState('networkidle');

    expect(await page.screenshot({ fullPage: true })).toMatchSnapshot('login-page.png');
  });

  test('Dashboard page visual snapshot @visual', async ({ page }) => {
    // Login first
    await page.goto('/login');
    await page.fill('input[type="email"]', 'test@example.com');
    await page.fill('input[type="password"]', 'password123');
    await page.click('button[type="submit"]');

    // Wait for dashboard to load
    await page.waitForURL('/dashboard');
    await page.waitForLoadState('networkidle');

    // Wait for charts to render
    await page.waitForSelector('.recharts-wrapper', { timeout: 10000 });

    expect(await page.screenshot({ fullPage: true })).toMatchSnapshot('dashboard-page.png');
  });

  test('Paper Trading page visual snapshot @visual', async ({ page }) => {
    // Login first
    await page.goto('/login');
    await page.fill('input[type="email"]', 'test@example.com');
    await page.fill('input[type="password"]', 'password123');
    await page.click('button[type="submit"]');

    // Navigate to paper trading
    await page.goto('/trading/paper');
    await page.waitForLoadState('networkidle');

    expect(await page.screenshot({ fullPage: true })).toMatchSnapshot('paper-trading-page.png');
  });

  test('Settings modal visual snapshot @visual', async ({ page }) => {
    // Login first
    await page.goto('/login');
    await page.fill('input[type="email"]', 'test@example.com');
    await page.fill('input[type="password"]', 'password123');
    await page.click('button[type="submit"]');

    // Navigate to dashboard
    await page.waitForURL('/dashboard');

    // Open settings modal
    await page.click('button:has-text("Settings")');
    await page.waitForSelector('[role="dialog"]');

    expect(await page.screenshot()).toMatchSnapshot('settings-modal.png');
  });

  test('Mobile responsive - Landing page @visual', async ({ page }) => {
    // Set mobile viewport
    await page.setViewportSize({ width: 375, height: 667 }); // iPhone SE

    await page.goto('/');
    await page.waitForLoadState('networkidle');

    expect(await page.screenshot({ fullPage: true })).toMatchSnapshot('landing-page-mobile.png');
  });

  test('Mobile responsive - Dashboard @visual', async ({ page }) => {
    // Set mobile viewport
    await page.setViewportSize({ width: 375, height: 667 }); // iPhone SE

    // Login
    await page.goto('/login');
    await page.fill('input[type="email"]', 'test@example.com');
    await page.fill('input[type="password"]', 'password123');
    await page.click('button[type="submit"]');

    // Wait for dashboard
    await page.waitForURL('/dashboard');
    await page.waitForLoadState('networkidle');

    expect(await page.screenshot({ fullPage: true })).toMatchSnapshot('dashboard-mobile.png');
  });

  test('Dark mode visual snapshot @visual', async ({ page }) => {
    await page.goto('/');
    await page.waitForLoadState('networkidle');

    // Toggle dark mode
    await page.click('button[aria-label="Toggle theme"]');
    await page.waitForTimeout(500); // Wait for animation

    expect(await page.screenshot({ fullPage: true })).toMatchSnapshot('landing-page-dark.png');
  });
});
