import puppeteer from 'puppeteer';

async function verifySpacing() {
  const browser = await puppeteer.launch({ headless: true });
  const page = await browser.newPage();

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

  // Scroll through to trigger all animations
  const totalHeight = await page.evaluate(() => document.body.scrollHeight);
  for (let y = 0; y < totalHeight; y += 400) {
    await page.evaluate((pos) => window.scrollTo(0, pos), y);
    await new Promise(r => setTimeout(r, 200));
  }
  
  // Back to top for full page
  await page.evaluate(() => window.scrollTo(0, 0));
  await new Promise(r => setTimeout(r, 500));

  // Full page screenshot in light mode
  await page.screenshot({ path: '/tmp/verify-spacing-full.png', fullPage: true });
  console.log('Full page (light): /tmp/verify-spacing-full.png');

  // Screenshot each section header to check spacing
  const sections = ['features', 'pricing', 'reviews', 'faq'];
  
  for (const section of sections) {
    await page.evaluate((id) => {
      const el = document.getElementById(id);
      if (el) el.scrollIntoView({ block: 'start', behavior: 'instant' });
    }, section);
    await new Promise(r => setTimeout(r, 500));
    await page.screenshot({ path: `/tmp/verify-${section}.png` });
    console.log(`${section}: /tmp/verify-${section}.png`);
  }

  await browser.close();
  console.log('\nDone - spacing verification complete!');
}

verifySpacing().catch(console.error);
