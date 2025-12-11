import { chromium } from 'playwright';
import { mkdirSync } from 'fs';
import { join } from 'path';

const SCREENSHOTS_DIR = '/tmp/theme-test-screenshots';
mkdirSync(SCREENSHOTS_DIR, { recursive: true });

async function investigateTheme() {
  const browser = await chromium.launch();
  const page = await browser.newPage();

  console.log('🔍 THEME SYSTEM INVESTIGATION\n');

  try {
    // Set up browser to emulate light mode system preference
    const darkMode = false;
    const context = await browser.newContext({
      colorScheme: darkMode ? 'dark' : 'light',
    });
    const lightPage = await context.newPage();

    console.log('TEST 1: Check system preference detection');
    console.log('==========================================');
    console.log('Browser color scheme: LIGHT\n');

    // Navigate
    await lightPage.goto('http://localhost:3002/', { waitUntil: 'networkidle', timeout: 30000 });
    await lightPage.waitForTimeout(2000);

    // Get detailed theme info
    const themeInfo = await lightPage.evaluate(() => {
      const root = document.documentElement;
      const isDark = root.classList.contains('dark');
      const bgColor = getComputedStyle(root).getPropertyValue('--background').trim();
      const systemPreference = window.matchMedia('(prefers-color-scheme: dark)').matches ? 'dark' : 'light';
      const storedTheme = localStorage.getItem('theme');
      const classAttr = root.getAttribute('class') || '(empty)';

      return {
        isDark,
        bgColor,
        systemPreference,
        storedTheme,
        classAttr,
        documentClasses: root.className
      };
    });

    console.log('Initial state on LIGHT system:');
    console.log(`  HTML class="dark": ${themeInfo.isDark}`);
    console.log(`  HTML class attribute: "${themeInfo.classAttr}"`);
    console.log(`  CSS --background: ${themeInfo.bgColor}`);
    console.log(`  System preference: ${themeInfo.systemPreference}`);
    console.log(`  localStorage.theme: ${themeInfo.storedTheme || '(not set)'}`);
    console.log('');

    // Take screenshot
    const screenshot1 = join(SCREENSHOTS_DIR, 'light-system-initial.png');
    await lightPage.screenshot({ path: screenshot1, fullPage: true, maxHeight: 1000 });
    console.log(`Screenshot: ${screenshot1}\n`);

    // Now test with dark system preference
    console.log('TEST 2: Check with dark system preference');
    console.log('==========================================');
    console.log('Browser color scheme: DARK\n');

    const darkContext = await browser.newContext({
      colorScheme: 'dark',
    });
    const darkPage = await darkContext.newPage();

    await darkPage.goto('http://localhost:3002/', { waitUntil: 'networkidle', timeout: 30000 });
    await darkPage.waitForTimeout(2000);

    const darkThemeInfo = await darkPage.evaluate(() => {
      const root = document.documentElement;
      const isDark = root.classList.contains('dark');
      const bgColor = getComputedStyle(root).getPropertyValue('--background').trim();
      const systemPreference = window.matchMedia('(prefers-color-scheme: dark)').matches ? 'dark' : 'light';
      const storedTheme = localStorage.getItem('theme');
      const classAttr = root.getAttribute('class') || '(empty)';

      return {
        isDark,
        bgColor,
        systemPreference,
        storedTheme,
        classAttr
      };
    });

    console.log('Initial state on DARK system:');
    console.log(`  HTML class="dark": ${darkThemeInfo.isDark}`);
    console.log(`  HTML class attribute: "${darkThemeInfo.classAttr}"`);
    console.log(`  CSS --background: ${darkThemeInfo.bgColor}`);
    console.log(`  System preference: ${darkThemeInfo.systemPreference}`);
    console.log(`  localStorage.theme: ${darkThemeInfo.storedTheme || '(not set)'}`);
    console.log('');

    const screenshot2 = join(SCREENSHOTS_DIR, 'dark-system-initial.png');
    await darkPage.screenshot({ path: screenshot2, fullPage: true, maxHeight: 1000 });
    console.log(`Screenshot: ${screenshot2}\n`);

    // Test explicit theme toggle on light system
    console.log('TEST 3: Toggle theme on light system');
    console.log('=====================================\n');

    const themeToggle = await lightPage.$('button[aria-label="Toggle theme"]');
    console.log('Clicking theme toggle...');
    await themeToggle.click();
    await lightPage.waitForTimeout(500);

    const afterToggle = await lightPage.evaluate(() => {
      const root = document.documentElement;
      const isDark = root.classList.contains('dark');
      const bgColor = getComputedStyle(root).getPropertyValue('--background').trim();
      const storedTheme = localStorage.getItem('theme');
      const classAttr = root.getAttribute('class') || '(empty)';

      return {
        isDark,
        bgColor,
        storedTheme,
        classAttr
      };
    });

    console.log('After clicking toggle:');
    console.log(`  HTML class="dark": ${afterToggle.isDark}`);
    console.log(`  HTML class attribute: "${afterToggle.classAttr}"`);
    console.log(`  CSS --background: ${afterToggle.bgColor}`);
    console.log(`  localStorage.theme: ${afterToggle.storedTheme || '(not set)'}`);
    console.log('');

    const screenshot3 = join(SCREENSHOTS_DIR, 'light-system-after-toggle.png');
    await lightPage.screenshot({ path: screenshot3, fullPage: true, maxHeight: 1000 });
    console.log(`Screenshot: ${screenshot3}\n`);

    // Analysis
    console.log('ANALYSIS');
    console.log('========\n');

    const lightSystemStartsDark = themeInfo.isDark && themeInfo.systemPreference === 'light';
    const darkSystemStartsDark = darkThemeInfo.isDark && darkThemeInfo.systemPreference === 'dark';
    const toggleWorked = afterToggle.isDark !== themeInfo.isDark;
    const localStorageUpdated = afterToggle.storedTheme !== null;

    if (lightSystemStartsDark) {
      console.log('⚠️  Issue 1: Page starts in DARK mode even with LIGHT system preference');
      console.log('   This suggests the default theme is hardcoded to "dark"\n');
    }

    if (!toggleWorked) {
      console.log('❌ Issue 2: Theme toggle did NOT change the theme');
      console.log('   The "dark" class is not being added/removed from HTML element\n');
    } else {
      console.log('✅ Theme toggle IS working\n');
    }

    if (localStorageUpdated) {
      console.log('✅ localStorage.theme is being updated on toggle\n');
    } else {
      console.log('⚠️  Issue 3: localStorage.theme is NOT being updated\n');
    }

    console.log('CONCLUSION');
    console.log('==========\n');

    if (lightSystemStartsDark && !toggleWorked) {
      console.log('🔴 MAJOR ISSUE FOUND: Theme toggle system is broken');
      console.log('\nProblems:');
      console.log('1. Theme doesn\'t respect system preference');
      console.log('2. Clicking toggle doesn\'t change the theme');
      console.log('\nLikely causes:');
      console.log('- ThemeContext.useEffect() is not running');
      console.log('- setTheme() is not updating the DOM');
      console.log('- CSS transitions might be hiding the change');
    } else if (toggleWorked) {
      console.log('✅ Theme toggle system appears to be working correctly');
    }

  } catch (error) {
    console.error('❌ Error:', error.message);
  } finally {
    await browser.close();
  }
}

investigateTheme().catch(console.error);
