import puppeteer from 'puppeteer';

async function check() {
  const browser = await puppeteer.launch({ headless: true });
  const page = await browser.newPage();
  
  await page.setCacheEnabled(false);
  await page.setViewport({ width: 1440, height: 900 });
  await page.goto('http://localhost:3003/', { waitUntil: 'networkidle2' });
  await new Promise(r => setTimeout(r, 2000));

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
    await new Promise(r => setTimeout(r, 1000));
  }

  // Scroll to stats section
  await page.evaluate(() => document.getElementById('stats')?.scrollIntoView({ block: 'center' }));
  await new Promise(r => setTimeout(r, 1000));
  
  await page.screenshot({ path: '/tmp/stats-updated.png' });
  console.log('Stats: /tmp/stats-updated.png');
  
  await browser.close();
}

check().catch(console.error);
