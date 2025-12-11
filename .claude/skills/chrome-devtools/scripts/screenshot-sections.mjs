import puppeteer from 'puppeteer';

async function screenshotSections() {
  const browser = await puppeteer.launch({ headless: true });
  const page = await browser.newPage();
  await page.setViewport({ width: 1440, height: 900 });
  await page.goto('http://localhost:3003/', { waitUntil: 'networkidle2' });
  await new Promise(r => setTimeout(r, 2000));

  // Force all animations to complete immediately
  await page.evaluate(() => {
    // Set all framer-motion elements to their final state
    document.querySelectorAll('[style*="opacity"]').forEach(el => {
      el.style.opacity = '1';
      el.style.transform = 'none';
    });
  });

  // Scroll to features and screenshot
  await page.evaluate(() => document.getElementById('features')?.scrollIntoView({ block: 'start' }));
  await new Promise(r => setTimeout(r, 500));
  await page.screenshot({ path: '/tmp/section-features.png' });
  console.log('Features section: /tmp/section-features.png');

  // Scroll to pricing and screenshot
  await page.evaluate(() => document.getElementById('pricing')?.scrollIntoView({ block: 'start' }));
  await new Promise(r => setTimeout(r, 500));
  await page.screenshot({ path: '/tmp/section-pricing.png' });
  console.log('Pricing section: /tmp/section-pricing.png');

  // Mobile check
  await page.setViewport({ width: 375, height: 812 });
  await page.reload({ waitUntil: 'networkidle2' });
  await new Promise(r => setTimeout(r, 1500));
  
  // Scroll through mobile
  const mobileHeight = await page.evaluate(() => document.body.scrollHeight);
  for (let y = 0; y < mobileHeight; y += 400) {
    await page.evaluate((pos) => window.scrollTo(0, pos), y);
    await new Promise(r => setTimeout(r, 200));
  }
  await page.evaluate(() => window.scrollTo(0, 0));
  await new Promise(r => setTimeout(r, 300));
  
  await page.screenshot({ path: '/tmp/mobile-full.png', fullPage: true });
  console.log('Mobile full page: /tmp/mobile-full.png');

  await browser.close();
}

screenshotSections().catch(console.error);
