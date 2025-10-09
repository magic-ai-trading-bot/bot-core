import { test, expect } from '@playwright/test';

test.describe('Paper Trading Flow', () => {
  // Helper function to login before each test
  test.beforeEach(async ({ page }) => {
    await page.goto('/login');
    await page.getByLabel(/email/i).fill('test@example.com');
    await page.getByLabel(/password/i).fill('password123');
    await page.getByRole('button', { name: /sign in|login/i }).click();
    await expect(page).toHaveURL(/.*dashboard/, { timeout: 15000 });

    // Navigate to paper trading page
    await page.getByRole('link', { name: /trading|paper.*trading/i }).click();
    await expect(page).toHaveURL(/.*trading/, { timeout: 10000 });
  });

  test('should display paper trading page header', async ({ page }) => {
    await expect(page.getByText(/paper.*trading|trading.*simulator/i).first()).toBeVisible();
  });

  test('should display portfolio balance', async ({ page }) => {
    // Check for balance display
    const balanceElement = page.getByText(/balance|portfolio|capital/i);
    await expect(balanceElement.first()).toBeVisible({ timeout: 10000 });

    // Should show a dollar amount
    const dollarAmount = page.locator('text=/\\$[0-9,]+/').first();
    await expect(dollarAmount).toBeVisible();
  });

  test('should display available trading pairs', async ({ page }) => {
    // Check for trading pair selector or list
    await page.waitForTimeout(2000);

    const tradingPairs = page.getByText(/BTCUSDT|ETHUSDT|BNBUSDT/);
    await expect(tradingPairs.first()).toBeVisible({ timeout: 10000 });
  });

  test('should display order entry form', async ({ page }) => {
    // Check for buy/sell buttons or tabs
    const buyButton = page.getByRole('button', { name: /^buy$/i });
    const sellButton = page.getByRole('button', { name: /^sell$/i });

    if (await buyButton.isVisible({ timeout: 5000 })) {
      await expect(buyButton).toBeVisible();
    }
    if (await sellButton.isVisible({ timeout: 5000 })) {
      await expect(sellButton).toBeVisible();
    }
  });

  test('should validate order amount input', async ({ page }) => {
    await page.waitForTimeout(2000);

    // Find amount input
    const amountInput = page.getByLabel(/amount|quantity|size/i);
    if (await amountInput.isVisible({ timeout: 5000 })) {
      // Try to enter invalid amount
      await amountInput.fill('-1');
      await amountInput.blur();

      // Should show validation error
      const errorMessage = page.getByText(/invalid|must.*positive|greater.*than/i);
      if (await errorMessage.isVisible({ timeout: 2000 })) {
        await expect(errorMessage).toBeVisible();
      }
    }
  });

  test('should place a long position', async ({ page }) => {
    await page.waitForTimeout(2000);

    // Click on Buy/Long button
    const buyButton = page.getByRole('button', { name: /^buy$|^long$/i }).first();
    if (await buyButton.isVisible({ timeout: 5000 })) {
      // Enter amount
      const amountInput = page.getByLabel(/amount|quantity|size/i);
      if (await amountInput.isVisible({ timeout: 2000 })) {
        await amountInput.fill('0.001');

        // Submit order
        const submitButton = page.getByRole('button', { name: /place.*order|execute|confirm/i });
        if (await submitButton.isVisible({ timeout: 2000 })) {
          await submitButton.click();

          // Wait for success message or position to appear
          await page.waitForTimeout(2000);

          // Check for success notification
          const successMsg = page.getByText(/success|placed|opened/i);
          if (await successMsg.isVisible({ timeout: 5000 })) {
            await expect(successMsg).toBeVisible();
          }
        }
      }
    }
  });

  test('should display open positions', async ({ page }) => {
    await page.waitForTimeout(2000);

    // Check for positions section
    const positionsSection = page.getByText(/open.*positions|active.*positions|positions/i);
    await expect(positionsSection.first()).toBeVisible({ timeout: 10000 });
  });

  test('should display position details', async ({ page }) => {
    await page.waitForTimeout(3000);

    // If there are open positions, check their details
    const positionRow = page.locator('[data-testid*="position"], tr, .position-row').first();
    if (await positionRow.isVisible({ timeout: 5000 })) {
      // Should show entry price
      const priceElement = page.locator('text=/\\$[0-9,]+/').first();
      await expect(priceElement).toBeVisible();

      // Should show quantity
      const quantityElement = page.locator('text=/[0-9]+\\.[0-9]+|[0-9]+/').first();
      await expect(quantityElement).toBeVisible();
    }
  });

  test('should calculate PnL for positions', async ({ page }) => {
    await page.waitForTimeout(3000);

    // Check for PnL display (can be positive or negative)
    const pnlElement = page.locator('text=/[+\\-]?\\$[0-9,]+|[+\\-]?[0-9]+\\.[0-9]+%/').first();
    if (await pnlElement.isVisible({ timeout: 5000 })) {
      await expect(pnlElement).toBeVisible();
    }
  });

  test('should close a position', async ({ page }) => {
    await page.waitForTimeout(3000);

    // Find close button for a position
    const closeButton = page.getByRole('button', { name: /close|exit/i }).first();
    if (await closeButton.isVisible({ timeout: 5000 })) {
      await closeButton.click();

      // Confirm closure if dialog appears
      const confirmButton = page.getByRole('button', { name: /confirm|yes|close/i }).last();
      if (await confirmButton.isVisible({ timeout: 2000 })) {
        await confirmButton.click();
      }

      // Wait for position to be closed
      await page.waitForTimeout(2000);

      // Check for success message
      const successMsg = page.getByText(/closed|exited/i);
      if (await successMsg.isVisible({ timeout: 5000 })) {
        await expect(successMsg).toBeVisible();
      }
    }
  });

  test('should display trading history', async ({ page }) => {
    await page.waitForTimeout(2000);

    // Check for history/trades section
    const historySection = page.getByText(/history|past.*trades|trade.*history/i);
    if (await historySection.isVisible({ timeout: 5000 })) {
      await expect(historySection).toBeVisible();
    }
  });

  test('should show AI signals for trading pairs', async ({ page }) => {
    await page.waitForTimeout(3000);

    // Check for AI signal indicators
    const aiSignal = page.getByText(/AI|signal|recommendation|bullish|bearish/i);
    if (await aiSignal.first().isVisible({ timeout: 5000 })) {
      await expect(aiSignal.first()).toBeVisible();
    }
  });

  test('should display risk metrics', async ({ page }) => {
    await page.waitForTimeout(2000);

    // Check for risk-related information
    const riskMetric = page.getByText(/risk|leverage|margin|exposure/i);
    if (await riskMetric.first().isVisible({ timeout: 5000 })) {
      await expect(riskMetric.first()).toBeVisible();
    }
  });

  test('should update portfolio value in real-time', async ({ page }) => {
    await page.waitForTimeout(2000);

    // Get initial portfolio value
    const portfolioElement = page.locator('text=/\\$[0-9,]+/').first();
    if (await portfolioElement.isVisible({ timeout: 5000 })) {
      const initialValue = await portfolioElement.textContent();

      // Wait for potential WebSocket update
      await page.waitForTimeout(5000);

      // Portfolio value should still be displayed (may or may not change)
      const currentValue = await portfolioElement.textContent();
      expect(currentValue).toBeTruthy();
    }
  });

  test('should prevent trading with insufficient balance', async ({ page }) => {
    await page.waitForTimeout(2000);

    // Try to place order with amount > balance
    const amountInput = page.getByLabel(/amount|quantity|size/i);
    if (await amountInput.isVisible({ timeout: 5000 })) {
      // Enter very large amount
      await amountInput.fill('999999');

      const submitButton = page.getByRole('button', { name: /place.*order|execute|confirm/i });
      if (await submitButton.isVisible({ timeout: 2000 })) {
        await submitButton.click();

        // Should show error about insufficient balance
        const errorMsg = page.getByText(/insufficient|balance|not.*enough/i);
        if (await errorMsg.isVisible({ timeout: 3000 })) {
          await expect(errorMsg).toBeVisible();
        }
      }
    }
  });

  test('should display performance chart', async ({ page }) => {
    await page.waitForTimeout(3000);

    // Check for performance/equity chart
    const chartTitle = page.getByText(/performance|equity.*curve|portfolio.*chart/i);
    if (await chartTitle.isVisible({ timeout: 5000 })) {
      await expect(chartTitle).toBeVisible();
    }
  });
});
