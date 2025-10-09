import { test, expect } from '@playwright/test';

test.describe('Authentication Flow', () => {
  test.beforeEach(async ({ page }) => {
    await page.goto('/');
  });

  test('should redirect to login page when not authenticated', async ({ page }) => {
    // Should redirect to /login from root
    await expect(page).toHaveURL(/.*login/);
    await expect(page.getByRole('heading', { name: /login/i })).toBeVisible();
  });

  test('should display login form with all fields', async ({ page }) => {
    await page.goto('/login');

    // Check for email and password fields
    await expect(page.getByLabel(/email/i)).toBeVisible();
    await expect(page.getByLabel(/password/i)).toBeVisible();
    await expect(page.getByRole('button', { name: /sign in|login/i })).toBeVisible();
  });

  test('should show validation errors for empty fields', async ({ page }) => {
    await page.goto('/login');

    // Try to submit empty form
    await page.getByRole('button', { name: /sign in|login/i }).click();

    // Should show validation errors
    await expect(page.getByText(/email.*required|required.*email/i).first()).toBeVisible();
  });

  test('should show error for invalid credentials', async ({ page }) => {
    await page.goto('/login');

    // Fill in invalid credentials
    await page.getByLabel(/email/i).fill('invalid@example.com');
    await page.getByLabel(/password/i).fill('wrongpassword');
    await page.getByRole('button', { name: /sign in|login/i }).click();

    // Should show error message
    await expect(page.getByText(/invalid.*credentials|login.*failed/i).first()).toBeVisible({ timeout: 10000 });
  });

  test('should successfully login with valid credentials', async ({ page }) => {
    await page.goto('/login');

    // Fill in valid test credentials (adjust based on your test data)
    await page.getByLabel(/email/i).fill('test@example.com');
    await page.getByLabel(/password/i).fill('password123');
    await page.getByRole('button', { name: /sign in|login/i }).click();

    // Should redirect to dashboard
    await expect(page).toHaveURL(/.*dashboard/, { timeout: 15000 });

    // Verify dashboard elements are visible
    await expect(page.getByText(/trading.*bot|dashboard/i).first()).toBeVisible();
  });

  test('should persist login after page reload', async ({ page }) => {
    await page.goto('/login');

    // Login
    await page.getByLabel(/email/i).fill('test@example.com');
    await page.getByLabel(/password/i).fill('password123');
    await page.getByRole('button', { name: /sign in|login/i }).click();

    await expect(page).toHaveURL(/.*dashboard/, { timeout: 15000 });

    // Reload page
    await page.reload();

    // Should still be on dashboard
    await expect(page).toHaveURL(/.*dashboard/);
    await expect(page.getByText(/trading.*bot|dashboard/i).first()).toBeVisible();
  });

  test('should logout successfully', async ({ page, context }) => {
    // Login first
    await page.goto('/login');
    await page.getByLabel(/email/i).fill('test@example.com');
    await page.getByLabel(/password/i).fill('password123');
    await page.getByRole('button', { name: /sign in|login/i }).click();
    await expect(page).toHaveURL(/.*dashboard/, { timeout: 15000 });

    // Logout
    await page.getByRole('button', { name: /logout|đăng xuất/i }).click();

    // Should redirect to login
    await expect(page).toHaveURL(/.*login/, { timeout: 10000 });

    // Verify token is cleared
    const cookies = await context.cookies();
    const authToken = cookies.find(c => c.name === 'auth_token' || c.name === 'token');
    expect(authToken).toBeUndefined();
  });

  test('should not access protected routes when logged out', async ({ page }) => {
    // Try to access dashboard without login
    await page.goto('/dashboard');

    // Should redirect to login
    await expect(page).toHaveURL(/.*login/, { timeout: 10000 });
  });

  test('should navigate to register page from login', async ({ page }) => {
    await page.goto('/login');

    // Click on register link
    const registerLink = page.getByRole('link', { name: /sign up|register|create account/i });
    if (await registerLink.isVisible()) {
      await registerLink.click();
      await expect(page).toHaveURL(/.*register|signup/);
    }
  });
});
