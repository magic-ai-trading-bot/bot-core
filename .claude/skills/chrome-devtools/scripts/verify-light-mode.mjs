import puppeteer from 'puppeteer';

async function verifyLightMode() {
  const browser = await puppeteer.launch({ headless: true });
  const page = await browser.newPage();
  await page.setViewport({ width: 1440, height: 900 });
  await page.goto('http://localhost:3003/', { waitUntil: 'networkidle2' });
  await new Promise(r => setTimeout(r, 2000));

  // Switch to light mode
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

  // Scroll to features to trigger animation
  await page.evaluate(() => document.getElementById('features')?.scrollIntoView({ block: 'center' }));
  await new Promise(r => setTimeout(r, 1500));

  // Check the theme and computed styles
  const debugInfo = await page.evaluate(() => {
    const html = document.documentElement;
    const body = document.body;
    
    // Check first feature card
    const featuresSection = document.getElementById('features');
    const firstCard = featuresSection?.querySelector('.rounded-2xl');
    const h3 = firstCard?.querySelector('h3');
    const p = firstCard?.querySelector('p');
    const icon = firstCard?.querySelector('svg');
    
    return {
      htmlClass: html.className,
      localStorage: localStorage.getItem('theme'),
      bodyBgColor: getComputedStyle(body).backgroundColor,
      card: firstCard ? {
        background: getComputedStyle(firstCard).backgroundColor,
        opacity: getComputedStyle(firstCard).opacity,
        h3: h3 ? {
          text: h3.textContent?.substring(0, 30),
          color: getComputedStyle(h3).color,
          opacity: getComputedStyle(h3).opacity,
        } : null,
        p: p ? {
          text: p.textContent?.substring(0, 30),
          color: getComputedStyle(p).color,
          opacity: getComputedStyle(p).opacity,
        } : null,
        icon: icon ? {
          color: getComputedStyle(icon).color,
          width: getComputedStyle(icon).width,
        } : null,
      } : null,
    };
  });

  console.log('=== LIGHT MODE DEBUG ===\n');
  console.log(JSON.stringify(debugInfo, null, 2));

  // Screenshot the feature section
  await page.screenshot({ path: '/tmp/light-features.png' });
  console.log('\nScreenshot: /tmp/light-features.png');

  await browser.close();
}

verifyLightMode().catch(console.error);
