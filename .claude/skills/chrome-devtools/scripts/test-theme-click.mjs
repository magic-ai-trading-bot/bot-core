import puppeteer from 'puppeteer';

async function testThemeToggle() {
  const browser = await puppeteer.launch({ headless: true });
  const page = await browser.newPage();
  await page.setViewport({ width: 1440, height: 900 });

  // Navigate to the page (dark mode is default)
  await page.goto('http://localhost:3003/', { waitUntil: 'networkidle2' });

  // Wait for React to hydrate
  await new Promise(r => setTimeout(r, 2000));

  // Take initial screenshot (dark mode)
  await page.screenshot({ path: '/tmp/theme-dark-before.png', fullPage: false });

  // Get initial state
  const initialState = await page.evaluate(() => {
    const html = document.documentElement;
    const mainDiv = document.querySelector('div[style*="background"]');
    return {
      htmlClass: html.className,
      bgColor: mainDiv ? getComputedStyle(mainDiv).backgroundColor : 'N/A',
      localStorage: localStorage.getItem('theme'),
    };
  });

  console.log('Initial state (dark):', JSON.stringify(initialState, null, 2));

  // Click the theme toggle dropdown trigger
  // The aria-label is "Toggle theme"
  const triggerButton = await page.$('button[aria-label="Toggle theme"]');

  if (triggerButton) {
    console.log('Found theme toggle button, clicking...');
    await triggerButton.click();

    // Wait for dropdown to appear
    await new Promise(r => setTimeout(r, 300));

    // Take screenshot of open dropdown
    await page.screenshot({ path: '/tmp/theme-dropdown-open.png', fullPage: false });

    // Click the "Light" option in the dropdown
    // Radix dropdown items typically don't have unique selectors, so we search by text
    const lightOption = await page.evaluate(() => {
      const items = document.querySelectorAll('[role="menuitem"]');
      for (const item of items) {
        if (item.textContent && item.textContent.toLowerCase().includes('light')) {
          item.click();
          return 'clicked light';
        }
      }
      // Also try looking for Sun icon items
      const allItems = document.querySelectorAll('[data-radix-collection-item]');
      for (const item of allItems) {
        if (item.textContent && item.textContent.toLowerCase().includes('light')) {
          item.click();
          return 'clicked light (radix item)';
        }
      }
      return 'light option not found';
    });

    console.log('Light option:', lightOption);

    // Wait for theme transition
    await new Promise(r => setTimeout(r, 500));

  } else {
    console.log('Theme toggle button not found');

    // Debug: list all buttons
    const buttons = await page.evaluate(() => {
      const btns = document.querySelectorAll('button');
      return Array.from(btns).map(btn => ({
        ariaLabel: btn.getAttribute('aria-label'),
        className: btn.className,
        innerText: btn.innerText?.substring(0, 50),
      }));
    });
    console.log('Available buttons:', JSON.stringify(buttons, null, 2));
  }

  // Take screenshot after toggle
  await page.screenshot({ path: '/tmp/theme-light-after.png', fullPage: false });

  // Get state after toggle
  const afterState = await page.evaluate(() => {
    const html = document.documentElement;
    const mainDiv = document.querySelector('div[style*="background"]');
    return {
      htmlClass: html.className,
      bgColor: mainDiv ? getComputedStyle(mainDiv).backgroundColor : 'N/A',
      localStorage: localStorage.getItem('theme'),
    };
  });

  console.log('After toggle (should be light):', JSON.stringify(afterState, null, 2));

  await browser.close();
}

testThemeToggle().catch(console.error);
