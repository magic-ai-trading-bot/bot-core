import puppeteer from 'puppeteer';

async function loginAndCheck() {
  const browser = await puppeteer.launch({
    headless: false,
    defaultViewport: { width: 1440, height: 900 }
  });

  const page = await browser.newPage();

  console.log('1. Going to login page...');
  await page.goto('http://localhost:3003/login', { waitUntil: 'networkidle2' });
  await new Promise(r => setTimeout(r, 1000));

  // Use page.evaluate to click Use Trader button
  console.log('2. Clicking Use Trader button...');
  await page.evaluate(() => {
    const buttons = document.querySelectorAll('button');
    for (const btn of buttons) {
      if (btn.textContent.includes('Use Trader')) {
        btn.click();
        return true;
      }
    }
    return false;
  });
  await new Promise(r => setTimeout(r, 500));

  // Click Sign In
  console.log('3. Clicking Sign In...');
  const signInBtn = await page.$('button[type="submit"]');
  if (signInBtn) {
    await signInBtn.click();
  }
  await new Promise(r => setTimeout(r, 3000));

  console.log('4. Current URL:', page.url());
  await page.screenshot({ path: '/tmp/after-login.png' });
  console.log('   Screenshot: /tmp/after-login.png');

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
    console.log('\n5. Checking ' + p.name + '...');
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
    console.log('   Light: /tmp/auth-' + p.name + '-light.png');

    // Set dark mode
    await page.evaluate(() => {
      localStorage.setItem('theme', 'dark');
      document.documentElement.classList.remove('light');
      document.documentElement.classList.add('dark');
    });
    await page.reload({ waitUntil: 'networkidle2' });
    await new Promise(r => setTimeout(r, 1500));

    await page.screenshot({ path: '/tmp/auth-' + p.name + '-dark.png', fullPage: true });
    console.log('   Dark: /tmp/auth-' + p.name + '-dark.png');
  }

  console.log('\nDone! Browser stays open.');
}

loginAndCheck().catch(console.error);
