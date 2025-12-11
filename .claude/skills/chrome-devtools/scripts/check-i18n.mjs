import puppeteer from 'puppeteer';

async function checkI18n() {
  const browser = await puppeteer.launch({ 
    headless: false,
    defaultViewport: { width: 1440, height: 900 }
  });
  
  const page = await browser.newPage();
  
  // Set Vietnamese language
  await page.goto('http://localhost:3003/', { waitUntil: 'networkidle2' });
  await page.evaluate(() => {
    localStorage.setItem('language', 'vi');
  });
  await page.reload({ waitUntil: 'networkidle2' });
  await new Promise(r => setTimeout(r, 2000));
  
  // Take screenshot of landing page
  await page.screenshot({ path: '/tmp/i18n-landing-vi.png', fullPage: true });
  console.log('Landing page (VI): /tmp/i18n-landing-vi.png');
  
  // Go to login
  await page.goto('http://localhost:3003/login', { waitUntil: 'networkidle2' });
  await page.evaluate(() => localStorage.setItem('language', 'vi'));
  await page.reload({ waitUntil: 'networkidle2' });
  await new Promise(r => setTimeout(r, 1500));
  await page.screenshot({ path: '/tmp/i18n-login-vi.png' });
  console.log('Login page (VI): /tmp/i18n-login-vi.png');
  
  // Login and check dashboard
  await page.evaluate(() => {
    const buttons = document.querySelectorAll('button');
    for (const btn of buttons) {
      if (btn.textContent.includes('Trader') || btn.textContent.includes('Dùng')) {
        btn.click();
        return;
      }
    }
  });
  await new Promise(r => setTimeout(r, 500));
  await page.click('button[type="submit"]');
  await new Promise(r => setTimeout(r, 3000));
  
  // Dashboard
  await page.goto('http://localhost:3003/dashboard', { waitUntil: 'networkidle2' });
  await new Promise(r => setTimeout(r, 2000));
  await page.screenshot({ path: '/tmp/i18n-dashboard-vi.png', fullPage: true });
  console.log('Dashboard (VI): /tmp/i18n-dashboard-vi.png');
  
  // Settings
  await page.goto('http://localhost:3003/settings', { waitUntil: 'networkidle2' });
  await new Promise(r => setTimeout(r, 2000));
  await page.screenshot({ path: '/tmp/i18n-settings-vi.png', fullPage: true });
  console.log('Settings (VI): /tmp/i18n-settings-vi.png');
  
  // Paper Trading
  await page.goto('http://localhost:3003/trading/paper', { waitUntil: 'networkidle2' });
  await new Promise(r => setTimeout(r, 2000));
  await page.screenshot({ path: '/tmp/i18n-paper-trading-vi.png', fullPage: true });
  console.log('Paper Trading (VI): /tmp/i18n-paper-trading-vi.png');
  
  // Features page
  await page.goto('http://localhost:3003/features', { waitUntil: 'networkidle2' });
  await new Promise(r => setTimeout(r, 2000));
  await page.screenshot({ path: '/tmp/i18n-features-vi.png', fullPage: true });
  console.log('Features (VI): /tmp/i18n-features-vi.png');
  
  // About page  
  await page.goto('http://localhost:3003/about', { waitUntil: 'networkidle2' });
  await new Promise(r => setTimeout(r, 2000));
  await page.screenshot({ path: '/tmp/i18n-about-vi.png', fullPage: true });
  console.log('About (VI): /tmp/i18n-about-vi.png');

  await browser.close();
  console.log('\n✅ Done!');
}

checkI18n().catch(console.error);
