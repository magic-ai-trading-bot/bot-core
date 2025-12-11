import puppeteer from 'puppeteer';

const PAGES = [
  { name: 'dashboard', url: '/dashboard' },
  { name: 'login', url: '/login' },
  { name: 'register', url: '/register' },
  { name: 'settings', url: '/settings' },
  { name: 'portfolio', url: '/portfolio' },
  { name: 'signals', url: '/signals' },
  { name: 'trade-analyses', url: '/trade-analyses' },
  { name: 'trading-paper', url: '/trading/paper' },
  { name: 'notfound', url: '/nonexistent-page' },
];

async function setLightMode(page) {
  await page.evaluate(() => {
    localStorage.setItem('theme', 'light');
    document.documentElement.classList.remove('dark');
    document.documentElement.classList.add('light');
    document.documentElement.style.colorScheme = 'light';
  });
  await new Promise(r => setTimeout(r, 500));
}

async function setDarkMode(page) {
  await page.evaluate(() => {
    localStorage.setItem('theme', 'dark');
    document.documentElement.classList.remove('light');
    document.documentElement.classList.add('dark');
    document.documentElement.style.colorScheme = 'dark';
  });
  await new Promise(r => setTimeout(r, 500));
}

async function checkPage(browser, pageInfo, theme) {
  const page = await browser.newPage();
  await page.setCacheEnabled(false);
  await page.setViewport({ width: 1440, height: 900 });

  const url = 'http://localhost:3003' + pageInfo.url;
  console.log('\n📄 Checking ' + pageInfo.name + ' (' + theme + ' mode)...');

  try {
    await page.goto(url, { waitUntil: 'networkidle2', timeout: 15000 });
    await new Promise(r => setTimeout(r, 1000));

    // Set theme
    if (theme === 'light') {
      await setLightMode(page);
    } else {
      await setDarkMode(page);
    }

    // Reload to apply theme
    await page.reload({ waitUntil: 'networkidle2' });
    await new Promise(r => setTimeout(r, 1500));

    // Check background color
    const bgColor = await page.evaluate(() => {
      const body = document.body;
      const main = document.querySelector('main') || document.querySelector('[class*="min-h-screen"]') || body;
      return window.getComputedStyle(main).backgroundColor;
    });

    const filename = '/tmp/theme-' + pageInfo.name + '-' + theme + '.png';
    await page.screenshot({ path: filename, fullPage: true });

    console.log('  ✅ Screenshot: ' + filename);
    console.log('  📊 Background: ' + bgColor);

  } catch (err) {
    console.log('  ❌ Error: ' + err.message);
  }

  await page.close();
}

async function main() {
  console.log('🎨 Theme Testing for All Pages\n');
  console.log('================================');

  const browser = await puppeteer.launch({ headless: true });

  // Test each page in both themes
  for (const pageInfo of PAGES) {
    await checkPage(browser, pageInfo, 'dark');
    await checkPage(browser, pageInfo, 'light');
  }

  await browser.close();

  console.log('\n================================');
  console.log('✅ Done! Check /tmp/theme-*.png for screenshots');
}

main().catch(console.error);
