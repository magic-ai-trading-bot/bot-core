import puppeteer from 'puppeteer';

async function testTradingVi() {
  const browser = await puppeteer.launch({ headless: true });
  const page = await browser.newPage();
  await page.setViewport({ width: 1440, height: 900 });
  
  // Go to login page
  await page.goto('http://localhost:3003/login', { waitUntil: 'networkidle2' });
  await new Promise(r => setTimeout(r, 1000));
  
  // Set Vietnamese language
  await page.evaluate(() => {
    localStorage.setItem('language', 'vi');
  });
  
  // Fill login credentials
  await page.type('input[type="email"]', 'trader@botcore.com');
  await page.type('input[type="password"]', 'password123');
  
  // Click Sign In
  await page.click('button[type="submit"]');
  await new Promise(r => setTimeout(r, 3000));
  
  // Navigate to trading page
  await page.goto('http://localhost:3003/trading/paper', { waitUntil: 'networkidle2' });
  await new Promise(r => setTimeout(r, 2000));
  
  // Take screenshot
  await page.screenshot({ path: '/tmp/trading-vi.png', fullPage: true });
  console.log('Screenshot saved: /tmp/trading-vi.png');
  
  // Check page text
  const text = await page.evaluate(() => document.body.innerText);
  console.log('\n=== Trading Page Text Sample ===');
  console.log(text.substring(0, 1500));
  
  await browser.close();
}

testTradingVi().catch(console.error);
