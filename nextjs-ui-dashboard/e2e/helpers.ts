import { Page, expect } from '@playwright/test';

/**
 * Helper function to login to the application
 */
export async function login(
  page: Page,
  email: string = 'test@example.com',
  password: string = 'password123'
) {
  await page.goto('/login');
  await page.getByLabel(/email/i).fill(email);
  await page.getByLabel(/password/i).fill(password);
  await page.getByRole('button', { name: /sign in|login/i }).click();
  await expect(page).toHaveURL(/.*dashboard/, { timeout: 15000 });
}

/**
 * Helper function to logout from the application
 */
export async function logout(page: Page) {
  await page.getByRole('button', { name: /logout|đăng xuất/i }).click();
  await expect(page).toHaveURL(/.*login/, { timeout: 10000 });
}

/**
 * Helper function to navigate to a specific page
 */
export async function navigateTo(page: Page, route: 'dashboard' | 'trading' | 'settings') {
  const linkText = {
    dashboard: /dashboard/i,
    trading: /trading|paper.*trading/i,
    settings: /settings/i,
  };

  await page.getByRole('link', { name: linkText[route] }).click();
  await expect(page).toHaveURL(new RegExp(`.*${route}`), { timeout: 10000 });
}

/**
 * Helper function to wait for WebSocket connection
 */
export async function waitForWebSocket(page: Page, timeout: number = 5000) {
  await page.waitForTimeout(timeout);
  const wsStatus = page.getByText(/live|connected/i);
  if (await wsStatus.first().isVisible({ timeout: 5000 })) {
    await expect(wsStatus.first()).toBeVisible();
  }
}

/**
 * Helper function to wait for charts to load
 */
export async function waitForCharts(page: Page, timeout: number = 5000) {
  await page.waitForTimeout(timeout);
  const chartSymbols = page.getByText(/BTCUSDT|ETHUSDT/);
  await expect(chartSymbols.first()).toBeVisible({ timeout: 10000 });
}

/**
 * Helper function to check if element is visible with timeout
 */
export async function isElementVisible(
  page: Page,
  selector: string | RegExp,
  timeout: number = 5000
): Promise<boolean> {
  try {
    const element = typeof selector === 'string'
      ? page.locator(selector)
      : page.getByText(selector);
    return await element.isVisible({ timeout });
  } catch {
    return false;
  }
}

/**
 * Helper function to fill form field if it exists
 */
export async function fillFieldIfExists(
  page: Page,
  label: string | RegExp,
  value: string,
  timeout: number = 5000
): Promise<boolean> {
  try {
    const field = page.getByLabel(label);
    if (await field.isVisible({ timeout })) {
      await field.fill(value);
      return true;
    }
    return false;
  } catch {
    return false;
  }
}

/**
 * Helper function to click button if it exists
 */
export async function clickButtonIfExists(
  page: Page,
  name: string | RegExp,
  timeout: number = 5000
): Promise<boolean> {
  try {
    const button = page.getByRole('button', { name });
    if (await button.isVisible({ timeout })) {
      await button.click();
      return true;
    }
    return false;
  } catch {
    return false;
  }
}
