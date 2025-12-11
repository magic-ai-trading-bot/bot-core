import puppeteer from 'puppeteer';

async function testDashboardVi() {
  const browser = await puppeteer.launch({ headless: true });
  const page = await browser.newPage();
  await page.setViewport({ width: 1440, height: 900 });
  
  // Go to login page
  await page.goto('http://localhost:3003/login', { waitUntil: 'networkidle2' });
  await new Promise(r => setTimeout(r, 1000));
  
  // Set Vietnamese language BEFORE login
  await page.evaluate(() => {
    localStorage.setItem('language', 'vi');
  });
  
  // Fill login credentials
  await page.type('input[type="email"]', 'trader@botcore.com');
  await page.type('input[type="password"]', 'password123');
  
  // Click Sign In
  await page.click('button[type="submit"]');
  await new Promise(r => setTimeout(r, 3000));
  
  // Navigate to dashboard
  await page.goto('http://localhost:3003/dashboard', { waitUntil: 'networkidle2' });
  await new Promise(r => setTimeout(r, 2000));
  
  // Take screenshot
  await page.screenshot({ path: '/tmp/dashboard-vi-final.png', fullPage: true });
  console.log('Screenshot saved: /tmp/dashboard-vi-final.png');
  
  // Check for Vietnamese text
  const text = await page.evaluate(() => document.body.innerText);
  const hasViText = text.includes('Chào mừng') || text.includes('Tổng số dư') || text.includes('Thị trường');
  console.log('Has Vietnamese text:', hasViText);
  console.log('Sample text:', text.substring(0, 500));
  
  await browser.close();
}

testDashboardVi().catch(console.error);
