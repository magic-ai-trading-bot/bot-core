import { chromium } from 'playwright';
import { writeFileSync, mkdirSync } from 'fs';
import { join } from 'path';

const SCREENSHOTS_DIR = '/tmp/theme-test-screenshots';
mkdirSync(SCREENSHOTS_DIR, { recursive: true });

async function testThemeToggle() {
  const browser = await chromium.launch();
  const page = await browser.newPage();

  console.log('🌐 Navigating to http://localhost:3002/...');

  try {
    await page.goto('http://localhost:3002/', { waitUntil: 'networkidle', timeout: 30000 });
    console.log('✓ Page loaded');

    // Take initial screenshot (default theme)
    const screenshot1 = join(SCREENSHOTS_DIR, '1-initial-page.png');
    await page.screenshot({ path: screenshot1, fullPage: true });
    console.log(`✓ Screenshot 1 (Initial page): ${screenshot1}`);

    // Check current theme
    const initialTheme = await page.evaluate(() => {
      return document.documentElement.classList.contains('dark') ? 'dark' : 'light';
    });
    console.log(`✓ Initial theme: ${initialTheme}`);

    // Wait for page to fully load and find theme toggle button
    console.log('\n🔍 Looking for theme toggle button...');

    // The theme toggle should be a button with a Sun or Moon icon
    // First, let's locate it by looking for buttons in the header
    const themeToggleButton = await page.$('button[aria-label*="theme"], button[aria-label*="Toggle"]');

    if (!themeToggleButton) {
      console.log('⚠️  Theme toggle button not found in expected location');
      console.log('📍 Searching for alternative selectors...');

      // Try to find by icon
      const allButtons = await page.$$('button');
      console.log(`Found ${allButtons.length} buttons on page`);

      // Look for buttons with aria-label containing "Toggle"
      for (let i = 0; i < allButtons.length; i++) {
        const ariaLabel = await allButtons[i].getAttribute('aria-label');
        if (ariaLabel && (ariaLabel.toLowerCase().includes('theme') || ariaLabel.toLowerCase().includes('toggle'))) {
          console.log(`✓ Found theme toggle at button index ${i}: aria-label="${ariaLabel}"`);

          // Take screenshot before clicking
          const beforeClick = join(SCREENSHOTS_DIR, '2-before-theme-toggle.png');
          await page.screenshot({ path: beforeClick, fullPage: true });
          console.log(`✓ Screenshot 2 (Before toggle): ${beforeClick}`);

          // Click the theme toggle
          console.log('\n🖱️  Clicking theme toggle button...');
          await allButtons[i].click();

          // Wait for transition
          await page.waitForTimeout(500);

          // Take screenshot after clicking
          const afterClick = join(SCREENSHOTS_DIR, '3-after-theme-toggle.png');
          await page.screenshot({ path: afterClick, fullPage: true });
          console.log(`✓ Screenshot 3 (After toggle): ${afterClick}`);

          // Check new theme
          const newTheme = await page.evaluate(() => {
            return document.documentElement.classList.contains('dark') ? 'dark' : 'light';
          });
          console.log(`✓ New theme: ${newTheme}`);

          if (newTheme !== initialTheme) {
            console.log(`✅ THEME TOGGLE SUCCESS: ${initialTheme} → ${newTheme}`);
          } else {
            console.log(`❌ THEME TOGGLE FAILED: Theme did not change (still ${initialTheme})`);
          }

          // Try clicking again to toggle back
          console.log('\n🖱️  Clicking theme toggle again to toggle back...');
          await allButtons[i].click();
          await page.waitForTimeout(500);

          const finalTheme = await page.evaluate(() => {
            return document.documentElement.classList.contains('dark') ? 'dark' : 'light';
          });
          console.log(`✓ Final theme: ${finalTheme}`);

          const screenshot4 = join(SCREENSHOTS_DIR, '4-after-second-toggle.png');
          await page.screenshot({ path: screenshot4, fullPage: true });
          console.log(`✓ Screenshot 4 (After 2nd toggle): ${screenshot4}`);

          if (finalTheme === initialTheme) {
            console.log(`✅ DOUBLE TOGGLE SUCCESS: ${initialTheme} → ${newTheme} → ${finalTheme}`);
          }

          break;
        }
      }
    } else {
      console.log('✓ Theme toggle button found!');

      // Take screenshot before clicking
      const beforeClick = join(SCREENSHOTS_DIR, '2-before-theme-toggle.png');
      await page.screenshot({ path: beforeClick, fullPage: true });
      console.log(`✓ Screenshot 2 (Before toggle): ${beforeClick}`);

      // Click the theme toggle
      console.log('\n🖱️  Clicking theme toggle button...');
      await themeToggleButton.click();

      // Wait for transition
      await page.waitForTimeout(500);

      // Take screenshot after clicking
      const afterClick = join(SCREENSHOTS_DIR, '3-after-theme-toggle.png');
      await page.screenshot({ path: afterClick, fullPage: true });
      console.log(`✓ Screenshot 3 (After toggle): ${afterClick}`);

      // Check new theme
      const newTheme = await page.evaluate(() => {
        return document.documentElement.classList.contains('dark') ? 'dark' : 'light';
      });
      console.log(`✓ New theme: ${newTheme}`);

      if (newTheme !== initialTheme) {
        console.log(`✅ THEME TOGGLE SUCCESS: ${initialTheme} → ${newTheme}`);
      } else {
        console.log(`❌ THEME TOGGLE FAILED: Theme did not change (still ${initialTheme})`);
      }
    }

    // Check localStorage
    console.log('\n💾 Checking localStorage...');
    const storedTheme = await page.evaluate(() => localStorage.getItem('theme'));
    console.log(`✓ Stored theme preference: ${storedTheme}`);

    // Check CSS variables
    console.log('\n🎨 Checking CSS variables...');
    const bgColor = await page.evaluate(() => {
      const root = document.documentElement;
      return getComputedStyle(root).getPropertyValue('--background');
    });
    console.log(`✓ CSS --background variable: ${bgColor}`);

    console.log('\n✅ All tests completed!');
    console.log(`📁 Screenshots saved to: ${SCREENSHOTS_DIR}`);

  } catch (error) {
    console.error('❌ Error during testing:', error);
  } finally {
    await browser.close();
  }
}

testThemeToggle().catch(console.error);
