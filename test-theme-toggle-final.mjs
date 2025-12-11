import { chromium } from 'playwright';
import { mkdirSync } from 'fs';
import { join } from 'path';

const SCREENSHOTS_DIR = '/tmp/theme-test-screenshots';
mkdirSync(SCREENSHOTS_DIR, { recursive: true });

async function testThemeToggle() {
  const browser = await chromium.launch();
  const page = await browser.newPage();

  console.log('🌐 THEME TOGGLE TEST SUITE');
  console.log('===========================\n');
  console.log('Testing the light/dark mode toggle functionality');
  console.log('');

  try {
    // Navigate to home page
    console.log('Step 1: Load homepage');
    console.log('→ Navigating to http://localhost:3002/');

    await page.goto('http://localhost:3002/', { waitUntil: 'networkidle', timeout: 30000 });
    await page.waitForTimeout(2000); // Wait for React hydration
    console.log('✓ Homepage loaded successfully\n');

    // Take initial screenshot
    const screenshot1 = join(SCREENSHOTS_DIR, '1-initial-page.png');
    await page.screenshot({ path: screenshot1, fullPage: true });
    console.log(`Screenshot 1 (Initial): ${screenshot1}`);

    // Check initial theme
    const initialTheme = await page.evaluate(() => {
      const isDark = document.documentElement.classList.contains('dark');
      return isDark ? 'dark' : 'light';
    });

    const initialBgColor = await page.evaluate(() => {
      const computed = getComputedStyle(document.documentElement);
      return computed.getPropertyValue('--background').trim();
    });

    console.log(`Initial theme: ${initialTheme.toUpperCase()}`);
    console.log(`CSS --background: ${initialBgColor}`);
    console.log('');

    // Find and click theme toggle
    console.log('Step 2: Locate and click theme toggle button');

    const themeToggleBtn = await page.$('button[aria-label="Toggle theme"]');

    if (!themeToggleBtn) {
      throw new Error('Theme toggle button not found');
    }

    console.log('✓ Theme toggle button found\n');
    console.log('→ Clicking theme toggle button...');

    await themeToggleBtn.click();
    await page.waitForTimeout(500); // Wait for CSS transition

    // Take screenshot after first toggle
    const screenshot2 = join(SCREENSHOTS_DIR, '2-after-first-toggle.png');
    await page.screenshot({ path: screenshot2, fullPage: true });
    console.log(`✓ Screenshot 2 (After 1st toggle): ${screenshot2}`);

    // Check new theme
    const newTheme = await page.evaluate(() => {
      const isDark = document.documentElement.classList.contains('dark');
      return isDark ? 'dark' : 'light';
    });

    const newBgColor = await page.evaluate(() => {
      const computed = getComputedStyle(document.documentElement);
      return computed.getPropertyValue('--background').trim();
    });

    console.log(`New theme: ${newTheme.toUpperCase()}`);
    console.log(`CSS --background: ${newBgColor}`);
    console.log('');

    // Verify theme changed
    if (newTheme !== initialTheme) {
      console.log(`✅ THEME TOGGLE WORKING: ${initialTheme.toUpperCase()} → ${newTheme.toUpperCase()}`);
    } else {
      console.log(`❌ THEME TOGGLE FAILED: Theme did not change (still ${initialTheme.toUpperCase()})`);
    }
    console.log('');

    // Test double toggle
    console.log('Step 3: Click again to toggle back');
    console.log('→ Clicking theme toggle button again...');

    const themeToggleBtn2 = await page.$('button[aria-label="Toggle theme"]');
    await themeToggleBtn2.click();
    await page.waitForTimeout(500);

    const finalTheme = await page.evaluate(() => {
      const isDark = document.documentElement.classList.contains('dark');
      return isDark ? 'dark' : 'light';
    });

    const finalBgColor = await page.evaluate(() => {
      const computed = getComputedStyle(document.documentElement);
      return computed.getPropertyValue('--background').trim();
    });

    const screenshot3 = join(SCREENSHOTS_DIR, '3-after-second-toggle.png');
    await page.screenshot({ path: screenshot3, fullPage: true });
    console.log(`✓ Screenshot 3 (After 2nd toggle): ${screenshot3}`);

    console.log(`Final theme: ${finalTheme.toUpperCase()}`);
    console.log(`CSS --background: ${finalBgColor}`);
    console.log('');

    if (finalTheme === initialTheme) {
      console.log(`✅ DOUBLE TOGGLE WORKING: ${initialTheme.toUpperCase()} → ${newTheme.toUpperCase()} → ${finalTheme.toUpperCase()}`);
    } else {
      console.log(`❌ DOUBLE TOGGLE ISSUE: Expected ${initialTheme.toUpperCase()}, got ${finalTheme.toUpperCase()}`);
    }
    console.log('');

    // Check localStorage
    console.log('Step 4: Verify persistence');
    const storedTheme = await page.evaluate(() => localStorage.getItem('theme'));
    console.log(`LocalStorage "theme": ${storedTheme || '(not set)'}`);
    console.log('');

    // Test dropdown menu
    console.log('Step 5: Test theme dropdown menu');
    console.log('→ Clicking theme toggle to open dropdown...');

    const dropdownBtn = await page.$('button[aria-label="Toggle theme"]');
    await dropdownBtn.click();
    await page.waitForTimeout(500);

    const menuItems = await page.$$('div[role="menuitem"], [role="menuitemradio"], [role="menuitemcheckbox"]');
    console.log(`Found ${menuItems.length} menu items in dropdown`);

    const dropdownContent = await page.evaluate(() => {
      // Try to find the dropdown menu content
      const menu = document.querySelector('[role="menu"]');
      if (menu) {
        const items = menu.querySelectorAll('[role="menuitem"]');
        return Array.from(items).map(item => ({
          text: item.textContent?.trim(),
          html: item.innerHTML
        }));
      }
      return null;
    });

    if (dropdownContent) {
      console.log('Dropdown content found:');
      dropdownContent.forEach((item, idx) => {
        console.log(`  ${idx + 1}. ${item.text}`);
      });
    } else {
      console.log('(Could not extract dropdown menu items)');
    }

    console.log('');

    // Summary
    console.log('═══════════════════════════════════════════════════════════');
    console.log('TEST RESULTS SUMMARY');
    console.log('═══════════════════════════════════════════════════════════');

    const themeToggleWorking = newTheme !== initialTheme;
    const doubleToggleWorking = finalTheme === initialTheme;
    const cssApplied = initialBgColor !== newBgColor;

    console.log(`Theme toggle button: FOUND ✓`);
    console.log(`Single toggle works: ${themeToggleWorking ? 'YES ✓' : 'NO ✗'}`);
    console.log(`Double toggle works: ${doubleToggleWorking ? 'YES ✓' : 'NO ✗'}`);
    console.log(`CSS variables applied: ${cssApplied ? 'YES ✓' : 'NO ✗'}`);
    console.log(`LocalStorage persistence: ${storedTheme ? 'YES ✓' : 'MISSING ✗'}`);
    console.log('');

    if (themeToggleWorking && doubleToggleWorking && cssApplied) {
      console.log('✅ THEME SYSTEM: WORKING CORRECTLY');
    } else {
      console.log('⚠️  THEME SYSTEM: ISSUES DETECTED');
      if (!cssApplied) {
        console.log('   - CSS variables not changing with theme');
      }
      if (!storedTheme) {
        console.log('   - LocalStorage not being set');
      }
    }

    console.log('');
    console.log(`Screenshots: ${SCREENSHOTS_DIR}`);
    console.log('');

  } catch (error) {
    console.error('❌ Test failed with error:');
    console.error(`   ${error.message}`);
  } finally {
    await browser.close();
  }
}

testThemeToggle().catch(console.error);
