import { chromium } from 'playwright';
import { writeFileSync, mkdirSync } from 'fs';
import { join } from 'path';

const SCREENSHOTS_DIR = '/tmp/theme-test-screenshots';
mkdirSync(SCREENSHOTS_DIR, { recursive: true });

async function testThemeToggle() {
  const browser = await chromium.launch();
  const context = await browser.newContext();
  const page = await context.newPage();

  console.log('🌐 Navigating to http://localhost:3002/...');

  try {
    // Capture console messages
    page.on('console', msg => {
      if (msg.type() === 'error') {
        console.log(`  🔴 Console Error: ${msg.text()}`);
      }
    });

    page.on('pageerror', err => {
      console.log(`  ❌ Page Error: ${err.message}`);
    });

    await page.goto('http://localhost:3002/', { waitUntil: 'domcontentloaded', timeout: 30000 });
    console.log('✓ Page DOM loaded');

    // Wait longer for React to hydrate
    console.log('⏳ Waiting for React hydration...');
    await page.waitForTimeout(3000);

    // Check page content
    const bodyHTML = await page.innerHTML('body');
    console.log(`✓ Body HTML length: ${bodyHTML.length} chars`);

    if (bodyHTML.length < 500) {
      console.log('⚠️  Page content seems minimal, checking for errors...');
      const pageContent = await page.content();
      console.log(`Page content preview: ${pageContent.substring(0, 500)}`);
    }

    // Take screenshot of blank page
    const screenshot1 = join(SCREENSHOTS_DIR, '1-initial-page.png');
    await page.screenshot({ path: screenshot1, fullPage: true });
    console.log(`✓ Screenshot 1: ${screenshot1}`);

    // Check if there's a root element
    const rootExists = await page.$('div#root');
    console.log(`✓ Root element exists: ${rootExists ? 'yes' : 'no'}`);

    // Get page structure
    const mainExists = await page.$('main');
    const headerExists = await page.$('header');
    const appExists = await page.$('[role="application"]');

    console.log(`✓ main element exists: ${mainExists ? 'yes' : 'no'}`);
    console.log(`✓ header element exists: ${headerExists ? 'yes' : 'no'}`);
    console.log(`✓ [role="application"] exists: ${appExists ? 'yes' : 'no'}`);

    // Count all buttons
    const allButtons = await page.$$('button');
    console.log(`✓ Total buttons found: ${allButtons.length}`);

    if (allButtons.length > 0) {
      console.log('🔍 Button details:');
      for (let i = 0; i < Math.min(allButtons.length, 10); i++) {
        const ariaLabel = await allButtons[i].getAttribute('aria-label');
        const className = await allButtons[i].getAttribute('class');
        const text = await allButtons[i].textContent();
        console.log(`  Button ${i}: aria-label="${ariaLabel}", text="${text?.trim()}", class="${className?.substring(0, 50)}..."`);
      }
    }

    // Try to find ThemeToggle by looking for sun/moon icons
    const svgs = await page.$$('svg');
    console.log(`✓ Total SVG elements found: ${svgs.length}`);

    // Check computed theme
    const theme = await page.evaluate(() => {
      return document.documentElement.classList.contains('dark') ? 'dark' : 'light';
    });
    console.log(`✓ Initial theme class: ${theme}`);

    // Get all classes on html element
    const htmlClasses = await page.getAttribute('html', 'class');
    console.log(`✓ HTML element classes: "${htmlClasses}"`);

    // Check if styles are loaded
    const styleCount = await page.$$('style');
    console.log(`✓ Style tags found: ${styleCount.length}`);

    // Try navigating to dashboard (protected route) - might redirect to login
    console.log('\n🔄 Navigating to dashboard...');
    await page.goto('http://localhost:3002/dashboard', { waitUntil: 'domcontentloaded', timeout: 30000 });
    await page.waitForTimeout(2000);

    const dashboardScreenshot = join(SCREENSHOTS_DIR, '2-dashboard.png');
    await page.screenshot({ path: dashboardScreenshot, fullPage: true });
    console.log(`✓ Screenshot 2 (Dashboard): ${dashboardScreenshot}`);

    const dashboardButtons = await page.$$('button');
    console.log(`✓ Buttons on dashboard: ${dashboardButtons.length}`);

  } catch (error) {
    console.error('❌ Error during testing:', error.message);
    console.error(error.stack);
  } finally {
    await browser.close();
  }
}

testThemeToggle().catch(console.error);
