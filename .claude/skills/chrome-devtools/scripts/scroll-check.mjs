import puppeteer from 'puppeteer';

async function scrollCheck() {
  const browser = await puppeteer.launch({ headless: true });
  const page = await browser.newPage();
  await page.setViewport({ width: 1440, height: 900 });
  await page.goto('http://localhost:3003/', { waitUntil: 'networkidle2' });
  await new Promise(r => setTimeout(r, 2000));

  // Scroll through the entire page to trigger whileInView animations
  console.log('Scrolling through page to trigger animations...');
  
  const totalHeight = await page.evaluate(() => document.body.scrollHeight);
  const viewportHeight = 900;
  let currentPosition = 0;
  
  while (currentPosition < totalHeight) {
    await page.evaluate((y) => window.scrollTo(0, y), currentPosition);
    await new Promise(r => setTimeout(r, 300)); // Wait for animations
    currentPosition += viewportHeight / 2;
  }
  
  // Scroll back to top
  await page.evaluate(() => window.scrollTo(0, 0));
  await new Promise(r => setTimeout(r, 500));

  // Now take full page screenshot
  await page.screenshot({ path: '/tmp/scroll-triggered.png', fullPage: true });
  console.log('Screenshot saved: /tmp/scroll-triggered.png');

  await browser.close();
}

scrollCheck().catch(console.error);
