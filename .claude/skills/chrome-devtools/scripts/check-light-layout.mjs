import puppeteer from 'puppeteer';

async function checkLightLayout() {
  const browser = await puppeteer.launch({ headless: true });
  const page = await browser.newPage();

  // Desktop light mode
  await page.setViewport({ width: 1440, height: 900 });
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

  // Scroll to trigger animations
  const totalHeight = await page.evaluate(() => document.body.scrollHeight);
  for (let y = 0; y < totalHeight; y += 450) {
    await page.evaluate((pos) => window.scrollTo(0, pos), y);
    await new Promise(r => setTimeout(r, 200));
  }
  await page.evaluate(() => window.scrollTo(0, 0));
  await new Promise(r => setTimeout(r, 300));

  // Take full page screenshot in light mode
  await page.screenshot({ path: '/tmp/light-desktop-full.png', fullPage: true });
  console.log('Desktop light mode: /tmp/light-desktop-full.png');

  // Scroll to pricing section for detail shot
  await page.evaluate(() => document.getElementById('pricing')?.scrollIntoView({ block: 'start' }));
  await new Promise(r => setTimeout(r, 500));
  await page.screenshot({ path: '/tmp/light-pricing.png' });
  console.log('Pricing section (light): /tmp/light-pricing.png');

  // Mobile light mode
  await page.setViewport({ width: 375, height: 812 });
  await page.reload({ waitUntil: 'networkidle2' });
  await new Promise(r => setTimeout(r, 1500));

  // Set light mode again after reload
  const btn2 = await page.$('button[aria-label="Toggle theme"]');
  if (btn2) {
    await btn2.click();
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

  // Scroll mobile to trigger animations
  const mobileHeight = await page.evaluate(() => document.body.scrollHeight);
  for (let y = 0; y < mobileHeight; y += 400) {
    await page.evaluate((pos) => window.scrollTo(0, pos), y);
    await new Promise(r => setTimeout(r, 150));
  }
  await page.evaluate(() => window.scrollTo(0, 0));
  await new Promise(r => setTimeout(r, 300));

  await page.screenshot({ path: '/tmp/light-mobile-full.png', fullPage: true });
  console.log('Mobile light mode: /tmp/light-mobile-full.png');

  await browser.close();
  console.log('\nDone!');
}

checkLightLayout().catch(console.error);
