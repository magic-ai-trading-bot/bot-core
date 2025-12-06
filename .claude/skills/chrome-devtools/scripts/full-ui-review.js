import puppeteer from 'puppeteer';
import path from 'path';
import fs from 'fs';

const pages = [
  { name: '01-dashboard', path: '/dashboard', expectedH1: 'Dashboard' },
  { name: '02-paper-trading', path: '/trading/paper', expectedH1: 'Paper Trading' },
  { name: '03-real-trading', path: '/trading/real', expectedH1: 'Real Trading' },
  { name: '04-portfolio', path: '/portfolio', expectedH1: 'Portfolio' },
  { name: '05-ai-signals', path: '/signals', expectedH1: 'AI Signals' },
  { name: '06-settings', path: '/settings', expectedH1: 'Settings' },
];

const outputDir = '/Users/dungngo97/Documents/bot-core/docs/screenshots/full-review';

async function run() {
  // Ensure output directory exists
  if (!fs.existsSync(outputDir)) {
    fs.mkdirSync(outputDir, { recursive: true });
  }

  const browser = await puppeteer.launch({
    headless: false,
    defaultViewport: { width: 1920, height: 1080 }
  });
  const page = await browser.newPage();

  // Login first
  console.log('Logging in...');
  await page.goto('http://localhost:3000/login', { waitUntil: 'networkidle2' });
  await page.type('input[type="email"]', 'trader@botcore.com');
  await page.type('input[type="password"]', 'password123');
  await page.click('button[type="submit"]');
  await page.waitForNavigation({ waitUntil: 'networkidle2' });
  console.log('Logged in successfully\n');

  const results = [];

  for (const pageInfo of pages) {
    console.log('\n=== ' + pageInfo.name + ' ===');

    try {
      // Use JavaScript navigation to preserve React state
      await page.evaluate((path) => {
        window.history.pushState({}, '', path);
        window.dispatchEvent(new PopStateEvent('popstate'));
      }, pageInfo.path);
      await new Promise(r => setTimeout(r, 2000)); // Wait for React to re-render

      const currentUrl = page.url();

      // Get page info
      const pageData = await page.evaluate(() => {
        const h1s = Array.from(document.querySelectorAll('h1')).map(h => h.textContent?.trim());
        const h2s = Array.from(document.querySelectorAll('h2')).map(h => h.textContent?.trim());
        const emptyStates = Array.from(document.querySelectorAll('[class*="empty"], [class*="no-data"], .text-muted-foreground'))
          .map(el => el.textContent?.trim())
          .filter(t => t && t.length < 100);
        const errors = Array.from(document.querySelectorAll('[class*="error"], [class*="Error"]'))
          .map(el => el.textContent?.trim());
        const vietnamese = document.body.innerText.match(/[àáạảãâầấậẩẫăằắặẳẵèéẹẻẽêềếệểễìíịỉĩòóọỏõôồốộổỗơờớợởỡùúụủũưừứựửữỳýỵỷỹđ]/gi);

        return { h1s, h2s, emptyStates, errors, hasVietnamese: vietnamese?.length > 0 };
      });

      const screenshotPath = path.join(outputDir, pageInfo.name + '.png');
      await page.screenshot({ path: screenshotPath, fullPage: true });

      console.log('  URL: ' + currentUrl);
      console.log('  H1s: ' + JSON.stringify(pageData.h1s));
      console.log('  H2s: ' + JSON.stringify(pageData.h2s.slice(0, 5)));
      if (pageData.emptyStates.length > 0) {
        console.log('  Empty states: ' + JSON.stringify(pageData.emptyStates.slice(0, 3)));
      }
      if (pageData.errors.length > 0) {
        console.log('  WARNING Errors found: ' + JSON.stringify(pageData.errors));
      }
      if (pageData.hasVietnamese) {
        console.log('  WARNING Vietnamese text detected!');
      }
      console.log('  Screenshot: ' + screenshotPath);

      results.push({
        name: pageInfo.name,
        path: pageInfo.path,
        success: true,
        ...pageData,
        screenshot: screenshotPath
      });
    } catch (err) {
      console.log('  ERROR: ' + err.message);
      results.push({
        name: pageInfo.name,
        path: pageInfo.path,
        success: false,
        error: err.message
      });
    }
  }

  await browser.close();

  console.log('\n\n=== SUMMARY ===');
  for (const r of results) {
    const status = r.success ? 'OK' : 'FAIL';
    const issues = [];
    if (r.hasVietnamese) issues.push('Vietnamese');
    if (r.errors?.length > 0) issues.push('Errors');
    if (r.emptyStates?.length > 3) issues.push('Many empty states');

    console.log(status + ' ' + r.name + ': ' + (issues.length > 0 ? 'ISSUES: ' + issues.join(', ') : 'Clean'));
  }

  return results;
}

run().catch(console.error);
