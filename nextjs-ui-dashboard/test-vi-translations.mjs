import puppeteer from 'puppeteer';

const pages = [
  { name: 'Login', path: '/login', requireAuth: false },
  { name: 'Register', path: '/register', requireAuth: false },
  { name: 'Dashboard', path: '/dashboard', requireAuth: true },
  { name: 'Settings', path: '/settings', requireAuth: true },
  { name: 'TradingPaper', path: '/trading/paper', requireAuth: true },
];

async function testVietnameseTranslations() {
  const browser = await puppeteer.launch({
    headless: true,
    defaultViewport: { width: 1440, height: 900 }
  });

  const page = await browser.newPage();

  console.log('🇻🇳 Testing Vietnamese translations on all pages...\n');

  // Test public pages first (Login, Register)
  for (const p of pages.filter(x => !x.requireAuth)) {
    console.log(`📄 Testing ${p.name}...`);
    await page.goto('http://localhost:3003' + p.path, { waitUntil: 'networkidle2' });
    await new Promise(r => setTimeout(r, 1000));

    // Set Vietnamese language
    await page.evaluate(() => {
      localStorage.setItem('i18nextLng', 'vi');
    });
    await page.reload({ waitUntil: 'networkidle2' });
    await new Promise(r => setTimeout(r, 1500));

    // Take screenshot
    const filename = '/tmp/vi-' + p.name.toLowerCase() + '.png';
    await page.screenshot({ path: filename, fullPage: true });
    console.log('  ✅ Screenshot: ' + filename);

    // Check for Vietnamese text
    const pageText = await page.evaluate(() => document.body.innerText);
    const hasVietnamese = /[àáạảãâầấậẩẫăằắặẳẵèéẹẻẽêềếệểễìíịỉĩòóọỏõôồốộổỗơờớợởỡùúụủũưừứựửữỳýỵỷỹđ]/i.test(pageText);
    console.log('  ' + (hasVietnamese ? '✅' : '❌') + ' Has Vietnamese characters: ' + hasVietnamese);
  }

  // Login first
  console.log('\n🔐 Logging in...');
  await page.goto('http://localhost:3003/login', { waitUntil: 'networkidle2' });
  await new Promise(r => setTimeout(r, 1000));

  // Fill login form
  await page.type('input[type="email"]', 'trader@botcore.com');
  await page.type('input[type="password"]', 'password123');
  await page.click('button[type="submit"]');
  await new Promise(r => setTimeout(r, 3000));

  console.log('📍 Current URL: ' + page.url());

  // Test protected pages
  for (const p of pages.filter(x => x.requireAuth)) {
    console.log('\n📄 Testing ' + p.name + '...');
    await page.goto('http://localhost:3003' + p.path, { waitUntil: 'networkidle2' });
    await new Promise(r => setTimeout(r, 1000));

    // Set Vietnamese language
    await page.evaluate(() => {
      localStorage.setItem('i18nextLng', 'vi');
    });
    await page.reload({ waitUntil: 'networkidle2' });
    await new Promise(r => setTimeout(r, 2000));

    // Take screenshot
    const filename = '/tmp/vi-' + p.name.toLowerCase() + '.png';
    await page.screenshot({ path: filename, fullPage: true });
    console.log('  ✅ Screenshot: ' + filename);

    // Check for Vietnamese text
    const pageText = await page.evaluate(() => document.body.innerText);
    const hasVietnamese = /[àáạảãâầấậẩẫăằắặẳẵèéẹẻẽêềếệểễìíịỉĩòóọỏõôồốộổỗơờớợởỡùúụủũưừứựửữỳýỵỷỹđ]/i.test(pageText);
    console.log('  ' + (hasVietnamese ? '✅' : '❌') + ' Has Vietnamese characters: ' + hasVietnamese);

    // Additional check for Settings tabs
    if (p.name === 'Settings') {
      const tabText = await page.evaluate(() => {
        const tabs = document.querySelectorAll('[role="tab"]');
        return Array.from(tabs).map(t => t.textContent).join(', ');
      });
      console.log('  📑 Tabs: ' + tabText);
    }

    // Additional check for Dashboard widgets
    if (p.name === 'Dashboard') {
      const widgetTitles = await page.evaluate(() => {
        const widgets = document.querySelectorAll('h3, h2');
        return Array.from(widgets).slice(0, 5).map(w => w.textContent).join(', ');
      });
      console.log('  📊 Widget titles: ' + widgetTitles);
    }
  }

  await browser.close();
  console.log('\n🎉 Done! Check screenshots in /tmp/vi-*.png');
}

testVietnameseTranslations().catch(console.error);
