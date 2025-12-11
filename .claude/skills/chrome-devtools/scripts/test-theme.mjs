import puppeteer from 'puppeteer';

async function testTheme() {
  const browser = await puppeteer.launch({ headless: true });
  const page = await browser.newPage();
  await page.setViewport({ width: 1440, height: 900 });
  
  // First, set light theme in localStorage before navigating
  await page.evaluateOnNewDocument(() => {
    localStorage.setItem('theme', 'light');
  });
  
  // Navigate to the page
  await page.goto('http://localhost:3003/', { waitUntil: 'networkidle2' });
  
  // Wait for React to hydrate
  await new Promise(r => setTimeout(r, 2000));
  
  // Take screenshot
  await page.screenshot({ path: '/tmp/theme-light-test.png', fullPage: false });
  
  // Get the theme class
  const htmlClass = await page.evaluate(() => document.documentElement.className);
  const bgColor = await page.evaluate(() => {
    const div = document.querySelector('div[style]');
    return div ? div.style.backgroundColor : 'N/A';
  });
  
  console.log(JSON.stringify({ htmlClass, bgColor }, null, 2));
  
  await browser.close();
}

testTheme().catch(console.error);
