import puppeteer from 'puppeteer';

async function verifyMobile() {
  const browser = await puppeteer.launch({ headless: true });
  const page = await browser.newPage();

  // Mobile viewport
  await page.setViewport({ width: 375, height: 812 });
  await page.goto('http://localhost:3003/', { waitUntil: 'networkidle2' });
  await new Promise(r => setTimeout(r, 1500));

  // Set light mode
  const btn = await page.$('button[aria-label="Toggle theme"]');
  if (btn) {
    await btn.click();
    await new Promise(r => setTimeout(r, 300));
    await page.evaluate(() => {
      const items = document.querySelectorAll('[role="menuitem"]');
      for (const item of items) {
        if (item.textContent && item.textContent.toLowerCase().includes('light')) {
          item.click();
          return;
        }
      }
    });
    await new Promise(r => setTimeout(r, 800));
  }

  // Scroll through to trigger all animations
  const totalHeight = await page.evaluate(() => document.body.scrollHeight);
  for (let y = 0; y < totalHeight; y += 400) {
    await page.evaluate((pos) => window.scrollTo(0, pos), y);
    await new Promise(r => setTimeout(r, 150));
  }
  
  await page.evaluate(() => window.scrollTo(0, 0));
  await new Promise(r => setTimeout(r, 500));

  // Full mobile screenshot
  await page.screenshot({ path: '/tmp/final-mobile.png', fullPage: true });
  console.log('Mobile layout: /tmp/final-mobile.png');

  await browser.close();
  console.log('Done!');
}

verifyMobile().catch(console.error);
