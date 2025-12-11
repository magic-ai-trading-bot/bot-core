import puppeteer from 'puppeteer';

async function loginAndCheck() {
  const browser = await puppeteer.launch({ 
    headless: false, // Show browser for debugging
    defaultViewport: { width: 1440, height: 900 }
  });
  
  const page = await browser.newPage();
  
  console.log('🔐 Going to login page...');
  await page.goto('http://localhost:3003/login', { waitUntil: 'networkidle2' });
  await new Promise(r => setTimeout(r, 1000));
  
  // Click "Use Trader" button to fill demo credentials
  console.log('📝 Clicking Use Trader button...');
  const useTraderBtn = await page.$('button:has-text("Use Trader")');
  if (useTraderBtn) {
    await useTraderBtn.click();
    await new Promise(r => setTimeout(r, 500));
  } else {
    // Manual fill if button not found
    console.log('📝 Filling credentials manually...');
    await page.type('input[type="email"]', 'trader@botcore.com');
    await page.type('input[type="password"]', 'password123');
  }
  
  // Click Sign In
  console.log('🚀 Clicking Sign In...');
  await page.click('button[type="submit"]');
  await new Promise(r => setTimeout(r, 3000));
  
  // Wait for redirect to dashboard
  await page.waitForNavigation({ waitUntil: 'networkidle2', timeout: 10000 }).catch(() => {});
  
  console.log('📍 Current URL:', page.url());
  await page.screenshot({ path: '/tmp/after-login.png' });
  console.log('Screenshot: /tmp/after-login.png');
  
  // Check protected pages
  const protectedPages = [
    { name: 'dashboard', url: '/dashboard' },
    { name: 'trading-paper', url: '/trading/paper' },
    { name: 'signals', url: '/signals' },
    { name: 'trade-analyses', url: '/trade-analyses' },
    { name: 'portfolio', url: '/portfolio' },
    { name: 'settings', url: '/settings' },
  ];
  
  for (const p of protectedPages) {
    console.log('\n📄 Checking ' + p.name + '...');
    await page.goto('http://localhost:3003' + p.url, { waitUntil: 'networkidle2' });
    await new Promise(r => setTimeout(r, 2000));
    
    // Set light mode
    await page.evaluate(() => {
      localStorage.setItem('theme', 'light');
      document.documentElement.classList.remove('dark');
      document.documentElement.classList.add('light');
    });
    await page.reload({ waitUntil: 'networkidle2' });
    await new Promise(r => setTimeout(r, 1500));
    
    await page.screenshot({ path: '/tmp/auth-' + p.name + '-light.png', fullPage: true });
    console.log('  Light: /tmp/auth-' + p.name + '-light.png');
    
    // Set dark mode
    await page.evaluate(() => {
      localStorage.setItem('theme', 'dark');
      document.documentElement.classList.remove('light');
      document.documentElement.classList.add('dark');
    });
    await page.reload({ waitUntil: 'networkidle2' });
    await new Promise(r => setTimeout(r, 1500));
    
    await page.screenshot({ path: '/tmp/auth-' + p.name + '-dark.png', fullPage: true });
    console.log('  Dark: /tmp/auth-' + p.name + '-dark.png');
  }
  
  console.log('\n✅ Done! Browser will stay open for inspection.');
  // Keep browser open
  // await browser.close();
}

loginAndCheck().catch(console.error);
