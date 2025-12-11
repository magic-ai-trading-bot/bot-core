import puppeteer from 'puppeteer';

async function checkAuthPages() {
  const browser = await puppeteer.launch({
    headless: true,
    defaultViewport: { width: 1440, height: 900 }
  });

  const page = await browser.newPage();

  console.log('🔐 Logging in...');
  await page.goto('http://localhost:3003/login', { waitUntil: 'networkidle2' });
  await new Promise(r => setTimeout(r, 1000));

  // Fill credentials using page.evaluate to find button by text
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
  await page.click('button[type="submit"]');
  await new Promise(r => setTimeout(r, 2000));

  // Wait for redirect
  await page.waitForNavigation({ waitUntil: 'networkidle2', timeout: 10000 }).catch(() => {});

  console.log('📍 Current URL:', page.url());

  // Protected pages to check
  const pages = [
    '/dashboard',
    '/trading/paper',
    '/trading/real',
    '/signals',
    '/trade-analyses',
    '/portfolio',
    '/settings',
    '/profile'
  ];

  const results = [];

  for (const pagePath of pages) {
    const safeName = pagePath.replace(/\//g, '-').replace(/^-/, '');

    for (const theme of ['light', 'dark']) {
      console.log(`\n📄 Checking ${pagePath} (${theme})...`);

      // Set theme before navigation
      await page.evaluate((t) => {
        localStorage.setItem('theme', t);
        localStorage.setItem('vite-ui-theme', t);
      }, theme);

      await page.goto(`http://localhost:3003${pagePath}`, { waitUntil: 'networkidle2' });
      await new Promise(r => setTimeout(r, 1500));

      // Force theme class
      await page.evaluate((t) => {
        if (t === 'dark') {
          document.documentElement.classList.add('dark');
          document.documentElement.classList.remove('light');
        } else {
          document.documentElement.classList.remove('dark');
          document.documentElement.classList.add('light');
        }
      }, theme);
      await new Promise(r => setTimeout(r, 500));

      // Get info
      const info = await page.evaluate(() => {
        const body = document.body;
        const sidebar = document.querySelector('aside') || document.querySelector('[class*="sidebar"]');
        const main = document.querySelector('main');

        // Check for text visibility issues
        const allText = document.querySelectorAll('h1, h2, h3, p, span, label');
        let invisibleText = 0;
        allText.forEach(el => {
          const style = getComputedStyle(el);
          if (style.color === 'rgb(255, 255, 255)' && document.documentElement.classList.contains('light')) {
            invisibleText++;
          }
        });

        return {
          url: window.location.pathname,
          htmlClass: document.documentElement.className,
          bodyBg: getComputedStyle(body).backgroundColor,
          sidebarBg: sidebar ? getComputedStyle(sidebar).backgroundColor : 'N/A',
          mainBg: main ? getComputedStyle(main).backgroundColor : 'N/A',
          invisibleTextCount: invisibleText
        };
      });

      // Check for issues
      const issues = [];

      if (theme === 'light') {
        if (info.bodyBg === 'rgb(0, 0, 0)') issues.push('Body still dark');
        if (info.sidebarBg === 'rgb(0, 0, 0)') issues.push('Sidebar still dark');
        if (info.mainBg === 'rgb(0, 0, 0)') issues.push('Main content still dark');
        if (info.invisibleTextCount > 5) issues.push(info.invisibleTextCount + ' text elements may be invisible');
      }

      // Screenshot
      const screenshotPath = `/tmp/auth${safeName}-${theme}.png`;
      await page.screenshot({ path: screenshotPath, fullPage: true });

      const status = issues.length === 0 ? '✅' : '❌';
      console.log(`  ${status} ${info.bodyBg} | Sidebar: ${info.sidebarBg}`);
      if (issues.length > 0) {
        issues.forEach(i => console.log(`     ⚠️ ${i}`));
      }
      console.log(`  📸 ${screenshotPath}`);

      results.push({
        page: pagePath,
        theme,
        status: issues.length === 0 ? 'OK' : 'ISSUES',
        issues,
        screenshot: screenshotPath
      });
    }
  }

  // Summary
  console.log('\n' + '='.repeat(60));
  console.log('📊 SUMMARY');
  console.log('='.repeat(60));

  const issueResults = results.filter(r => r.issues.length > 0);
  if (issueResults.length === 0) {
    console.log('✅ All pages look correct!');
  } else {
    console.log('❌ ' + issueResults.length + ' pages have issues:');
    issueResults.forEach(r => {
      console.log('  - ' + r.page + ' (' + r.theme + '): ' + r.issues.join(', '));
    });
  }

  await browser.close();
}

checkAuthPages().catch(console.error);
