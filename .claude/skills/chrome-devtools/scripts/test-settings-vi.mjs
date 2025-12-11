import puppeteer from 'puppeteer';

async function testSettingsVi() {
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
  
  // Navigate to settings page
  await page.goto('http://localhost:3003/settings', { waitUntil: 'networkidle2' });
  await new Promise(r => setTimeout(r, 2000));
  
  // Take screenshot
  await page.screenshot({ path: '/tmp/settings-vi.png', fullPage: true });
  console.log('Screenshot saved: /tmp/settings-vi.png');
  
  // Check page text
  const text = await page.evaluate(() => document.body.innerText);
  console.log('\n=== Settings Page Text Sample ===');
  console.log(text.substring(0, 2000));
  
  await browser.close();
}

testSettingsVi().catch(console.error);
