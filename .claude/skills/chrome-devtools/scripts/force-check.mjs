import puppeteer from 'puppeteer';

async function check() {
  const browser = await puppeteer.launch({ headless: true });
  const page = await browser.newPage();
  
  // Disable cache
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

  // Scroll to features section
  await page.evaluate(() => document.getElementById('features')?.scrollIntoView({ block: 'start' }));
  await new Promise(r => setTimeout(r, 800));
  
  // Check the actual margin value from computed styles
  const marginInfo = await page.evaluate(() => {
    const badge = document.querySelector('#features .inline-flex'); // Badge element
    if (badge) {
      const style = window.getComputedStyle(badge);
      return {
        marginBottom: style.marginBottom,
        className: badge.className
      };
    }
    return null;
  });
  
  console.log('Badge margin-bottom:', marginInfo);
  
  await page.screenshot({ path: '/tmp/force-features.png' });
  console.log('Screenshot: /tmp/force-features.png');
  
  await browser.close();
}

check().catch(console.error);
