import puppeteer from 'puppeteer';

async function checkLayout() {
  const browser = await puppeteer.launch({ headless: true });
  const page = await browser.newPage();

  console.log('=== LAYOUT INSPECTION ===\n');

  // Desktop viewport
  console.log('1. Desktop (1440x900) - Full page scroll...');
  await page.setViewport({ width: 1440, height: 900 });
  await page.goto('http://localhost:3003/', { waitUntil: 'networkidle2' });
  await new Promise(r => setTimeout(r, 1500));

  // Set light mode for better visibility of layout issues
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
    await new Promise(r => setTimeout(r, 500));
  }

  // Take full page screenshot
  await page.screenshot({ path: '/tmp/layout-desktop-full.png', fullPage: true });
  console.log('   Saved: /tmp/layout-desktop-full.png');

  // Tablet viewport
  console.log('\n2. Tablet (768x1024)...');
  await page.setViewport({ width: 768, height: 1024 });
  await page.reload({ waitUntil: 'networkidle2' });
  await new Promise(r => setTimeout(r, 1000));
  await page.screenshot({ path: '/tmp/layout-tablet-full.png', fullPage: true });
  console.log('   Saved: /tmp/layout-tablet-full.png');

  // Mobile viewport
  console.log('\n3. Mobile (375x812 - iPhone X)...');
  await page.setViewport({ width: 375, height: 812 });
  await page.reload({ waitUntil: 'networkidle2' });
  await new Promise(r => setTimeout(r, 1000));
  await page.screenshot({ path: '/tmp/layout-mobile-full.png', fullPage: true });
  console.log('   Saved: /tmp/layout-mobile-full.png');

  // Get section spacing info
  console.log('\n4. Analyzing section spacing...');
  const spacingInfo = await page.evaluate(() => {
    const sections = document.querySelectorAll('section');
    const results = [];
    sections.forEach((section, i) => {
      const rect = section.getBoundingClientRect();
      const style = getComputedStyle(section);
      results.push({
        index: i,
        id: section.id || 'no-id',
        paddingTop: style.paddingTop,
        paddingBottom: style.paddingBottom,
        marginTop: style.marginTop,
        marginBottom: style.marginBottom,
        height: Math.round(rect.height),
      });
    });
    return results;
  });
  console.log('Section spacing:', JSON.stringify(spacingInfo, null, 2));

  await browser.close();
  console.log('\n✅ Layout check complete!');
}

checkLayout().catch(console.error);
