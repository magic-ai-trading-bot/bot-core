import puppeteer from 'puppeteer';

async function fullVerify() {
  const browser = await puppeteer.launch({ headless: true });
  const page = await browser.newPage();

  // Desktop light mode verification
  console.log('=== DESKTOP LIGHT MODE ===');
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

  // Scroll through each section with more time for animations
  const sections = ['features', 'stats', 'pricing', 'testimonials', 'faq'];
  for (const sectionId of sections) {
    await page.evaluate((id) => {
      const el = document.getElementById(id);
      if (el) el.scrollIntoView({ block: 'center', behavior: 'smooth' });
    }, sectionId);
    await new Promise(r => setTimeout(r, 1000)); // Wait for animations
    await page.screenshot({ path: `/tmp/verify-${sectionId}.png` });
    console.log(`${sectionId}: /tmp/verify-${sectionId}.png`);
  }

  // Full page with animations triggered
  await page.evaluate(() => window.scrollTo(0, 0));
  await new Promise(r => setTimeout(r, 500));
  await page.screenshot({ path: '/tmp/verify-fullpage.png', fullPage: true });
  console.log('Full page: /tmp/verify-fullpage.png');

  // Mobile verification
  console.log('\n=== MOBILE (375x812) ===');
  await page.setViewport({ width: 375, height: 812 });
  await page.reload({ waitUntil: 'networkidle2' });
  await new Promise(r => setTimeout(r, 1500));
  
  // Set light via localStorage (more reliable)
  await page.evaluate(() => localStorage.setItem('theme', 'light'));
  await page.reload({ waitUntil: 'networkidle2' });
  await new Promise(r => setTimeout(r, 1500));

  // Scroll mobile through sections
  for (const sectionId of sections) {
    await page.evaluate((id) => {
      const el = document.getElementById(id);
      if (el) el.scrollIntoView({ block: 'center' });
    }, sectionId);
    await new Promise(r => setTimeout(r, 800));
  }
  
  await page.evaluate(() => window.scrollTo(0, 0));
  await new Promise(r => setTimeout(r, 500));
  await page.screenshot({ path: '/tmp/verify-mobile-full.png', fullPage: true });
  console.log('Mobile full: /tmp/verify-mobile-full.png');

  await browser.close();
  console.log('\n✅ Verification complete!');
}

fullVerify().catch(console.error);
