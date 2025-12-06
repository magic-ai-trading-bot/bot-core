#!/usr/bin/env node
/**
 * UI Review Script - Login and capture screenshots of all pages
 * Uses SIDEBAR CLICKS to maintain session (not direct URL navigation)
 */

import { getBrowser, getPage, closeBrowser, outputJSON } from './lib/browser.js';

const OUTPUT_DIR = '/Users/dungngo97/Documents/bot-core/docs/screenshots/ui-review';
const BASE_URL = 'http://localhost:3000';

// Navigate via sidebar links (use nav selector to target sidebar specifically)
const PAGES = [
  {
    name: '03-dashboard',
    sidebarSelector: 'nav a[href="/dashboard"]',
    expectedH1: 'Dashboard'
  },
  {
    name: '04-paper-trading',
    sidebarSelector: 'nav a[href="/trading/paper"]',
    expectedH1: 'Paper Trading'
  },
  {
    name: '05-real-trading',
    sidebarSelector: 'nav a[href="/trading/real"]',
    expectedH1: 'Real Trading'
  },
  {
    name: '06-settings',
    sidebarSelector: 'nav a[href="/settings"]',
    expectedH1: 'Cài đặt Bot'
  },
];

async function waitForPageContent(page, timeout = 5000) {
  await page.waitForFunction(() => document.readyState === 'complete', { timeout });
  await new Promise(r => setTimeout(r, 2000)); // Wait for React render
}

async function main() {
  const browser = await getBrowser({ headless: true });
  const page = await getPage(browser);
  await page.setViewport({ width: 1920, height: 1080 });

  try {
    // 1. Login
    console.error('Navigating to login...');
    await page.goto(`${BASE_URL}/login`, { waitUntil: 'networkidle2' });
    await waitForPageContent(page);

    // Fill login form
    await page.type('input[type="email"]', 'trader@botcore.com');
    await page.type('input[type="password"]', 'password123');

    // Click submit and wait
    await Promise.all([
      page.waitForNavigation({ waitUntil: 'networkidle2', timeout: 15000 }).catch(() => {}),
      page.click('button[type="submit"]')
    ]);

    await waitForPageContent(page);
    console.error('Logged in, current URL:', page.url());

    // Check if we're on dashboard
    if (!page.url().includes('/dashboard')) {
      console.error('WARNING: Not redirected to dashboard after login!');
    }

    // 2. Capture each page via SIDEBAR CLICKS
    const results = [];

    for (const { name, sidebarSelector, expectedH1 } of PAGES) {
      console.error(`\n=== Capturing ${name} ===`);

      try {
        // Find and click sidebar link using JavaScript (better for React Router)
        const clicked = await page.evaluate((selector) => {
          const link = document.querySelector(selector);
          if (link) {
            // Trigger click event through JavaScript
            link.click();
            return true;
          }
          return false;
        }, sidebarSelector);

        if (clicked) {
          console.error(`  JS clicked ${sidebarSelector}`);
          // Wait for React Router to handle navigation
          await new Promise(r => setTimeout(r, 3000));
        } else {
          console.error(`  Sidebar link not found: ${sidebarSelector}`);
        }

        // Wait for page content
        await waitForPageContent(page);

        // Get ALL h1 elements to debug
        const allH1s = await page.evaluate(() => {
          return Array.from(document.querySelectorAll('h1')).map(h => h.textContent.trim());
        });

        // Get main content h1 (skip sidebar/header h1)
        const mainH1 = await page.evaluate(() => {
          // Try main content area first
          const main = document.querySelector('main h1');
          if (main) return main.textContent.trim();
          // Fallback to any h1 that's not in sidebar/header
          const h1s = document.querySelectorAll('h1');
          for (const h1 of h1s) {
            const text = h1.textContent.trim();
            if (text !== 'BotCore' && text.length > 0) return text;
          }
          return 'No main h1 found';
        });

        const currentUrl = page.url();
        console.error(`  URL: ${currentUrl}`);
        console.error(`  All H1s: ${JSON.stringify(allH1s)}`);
        console.error(`  Main H1: ${mainH1}`);

        // Take screenshot
        const outputPath = `${OUTPUT_DIR}/${name}.png`;
        await page.screenshot({ path: outputPath, fullPage: true });
        console.error(`  Screenshot saved: ${outputPath}`);

        results.push({
          name,
          url: currentUrl,
          h1: mainH1,
          allH1s,
          expectedH1,
          matched: mainH1.toLowerCase().includes(expectedH1.toLowerCase()),
          success: true,
          output: outputPath
        });
      } catch (err) {
        console.error(`  ERROR: ${err.message}`);
        results.push({ name, success: false, error: err.message });
      }
    }

    // Summary
    console.error('\n=== SUMMARY ===');
    const matched = results.filter(r => r.matched).length;
    console.error(`Pages matched: ${matched}/${results.length}`);
    results.forEach(r => {
      const status = r.matched ? '✅' : '❌';
      console.error(`  ${status} ${r.name}: "${r.h1}" (expected: "${r.expectedH1}")`);
    });

    outputJSON({ success: true, results });

  } catch (err) {
    console.error('FATAL ERROR:', err.message);
    outputJSON({ success: false, error: err.message });
  } finally {
    await closeBrowser(browser);
  }
}

main();
