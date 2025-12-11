import puppeteer from 'puppeteer';

async function verifyAuthTheme() {
  const browser = await puppeteer.launch({ 
    headless: true,
    defaultViewport: { width: 1440, height: 900 }
  });
  
  const page = await browser.newPage();
  
  console.log('🔐 Going to login page...');
  await page.goto('http://localhost:3003/login', { waitUntil: 'networkidle2' });
  await new Promise(r => setTimeout(r, 1000));
  
  // Click "Use Trader" button using evaluate
  console.log('📝 Clicking Use Trader button...');
  await page.evaluate(() => {
    const buttons = document.querySelectorAll('button');
    for (const btn of buttons) {
      if (btn.textContent && btn.textContent.includes('Use Trader')) {
        btn.click();
        return true;
      }
    }
    return false;
  });
  await new Promise(r => setTimeout(r, 500));
  
  // Click Sign In
  console.log('🚀 Clicking Sign In...');
  await page.evaluate(() => {
    const buttons = document.querySelectorAll('button');
    for (const btn of buttons) {
      if (btn.textContent && btn.textContent.includes('Sign In')) {
        btn.click();
        return true;
      }
    }
    return false;
  });
  await new Promise(r => setTimeout(r, 3000));
  
  console.log('📍 Current URL:', page.url());
  
  // Go to dashboard
  await page.goto('http://localhost:3003/dashboard', { waitUntil: 'networkidle2' });
  await new Promise(r => setTimeout(r, 2000));
  
  // Test DARK mode first
  console.log('\n🌙 Testing DARK mode...');
  await page.evaluate(() => {
    localStorage.setItem('theme', 'dark');
    document.documentElement.classList.remove('light');
    document.documentElement.classList.add('dark');
  });
  await page.reload({ waitUntil: 'networkidle2' });
  await new Promise(r => setTimeout(r, 1500));
  await page.screenshot({ path: '/tmp/verify-dashboard-dark.png' });
  console.log('  Screenshot: /tmp/verify-dashboard-dark.png');
  
  // Test LIGHT mode
  console.log('\n☀️ Testing LIGHT mode...');
  await page.evaluate(() => {
    localStorage.setItem('theme', 'light');
    document.documentElement.classList.remove('dark');
    document.documentElement.classList.add('light');
  });
  await page.reload({ waitUntil: 'networkidle2' });
  await new Promise(r => setTimeout(r, 1500));
  await page.screenshot({ path: '/tmp/verify-dashboard-light.png' });
  console.log('  Screenshot: /tmp/verify-dashboard-light.png');
  
  await browser.close();
  console.log('\n✅ Done! Check /tmp/verify-dashboard-dark.png and /tmp/verify-dashboard-light.png');
}

verifyAuthTheme().catch(console.error);
