import puppeteer from 'puppeteer';

const pageName = process.argv[2] || '/';
const theme = process.argv[3] || 'light';

async function checkPage() {
  const browser = await puppeteer.launch({ 
    headless: true,
    defaultViewport: { width: 1440, height: 900 }
  });
  const page = await browser.newPage();
  
  // Set theme via localStorage before navigating
  await page.evaluateOnNewDocument((t) => {
    localStorage.setItem('theme', t);
    localStorage.setItem('vite-ui-theme', t);
  }, theme);
  
  const url = `http://localhost:3003${pageName}`;
  console.log(`🔗 Navigating to: ${url} (${theme} mode)`);
  
  await page.goto(url, { waitUntil: 'networkidle2' });
  await new Promise(r => setTimeout(r, 1500));
  
  // Force theme class on html element
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
  
  // Get theme info
  const info = await page.evaluate(() => {
    const html = document.documentElement;
    const body = document.body;
    const mainContent = document.querySelector('main') || body.firstElementChild;
    
    return {
      htmlClass: html.className,
      localStorageTheme: localStorage.getItem('theme'),
      bodyBg: getComputedStyle(body).backgroundColor,
      bodyColor: getComputedStyle(body).color,
      mainBg: mainContent ? getComputedStyle(mainContent).backgroundColor : 'N/A',
      mainColor: mainContent ? getComputedStyle(mainContent).color : 'N/A',
      pageTitle: document.title,
      url: window.location.pathname
    };
  });
  
  console.log('📊 Theme Info:', JSON.stringify(info, null, 2));
  
  // Check for issues
  const issues = [];
  
  // Light mode should have light background
  if (theme === 'light') {
    if (info.bodyBg === 'rgb(0, 0, 0)' || info.bodyBg.includes('rgba(0, 0, 0, 1)')) {
      issues.push('❌ Body background is still dark (black)');
    }
    if (info.mainBg === 'rgb(0, 0, 0)' || (info.mainBg && info.mainBg.includes('rgba(0, 0, 0, 1)'))) {
      issues.push('❌ Main content background is still dark');
    }
    // Check if text is white (hard to see on light bg)
    if (info.bodyColor === 'rgb(255, 255, 255)' || info.mainColor === 'rgb(255, 255, 255)') {
      issues.push('⚠️ Text is white - may be invisible on light background');
    }
  }
  
  // Dark mode should have dark background  
  if (theme === 'dark') {
    if (info.bodyBg === 'rgb(255, 255, 255)' || info.bodyBg === 'rgb(250, 248, 245)') {
      issues.push('❌ Body background is still light');
    }
  }
  
  // Take screenshot
  const safeName = pageName.replace(/\//g, '-').replace(/^-/, '') || 'home';
  const screenshotPath = `/tmp/page-${safeName}-${theme}.png`;
  
  // Scroll to trigger animations
  const totalHeight = await page.evaluate(() => document.body.scrollHeight);
  for (let y = 0; y < totalHeight; y += 500) {
    await page.evaluate((pos) => window.scrollTo(0, pos), y);
    await new Promise(r => setTimeout(r, 100));
  }
  await page.evaluate(() => window.scrollTo(0, 0));
  await new Promise(r => setTimeout(r, 300));
  
  await page.screenshot({ path: screenshotPath, fullPage: true });
  console.log(`📸 Screenshot: ${screenshotPath}`);
  
  if (issues.length > 0) {
    console.log('\n🚨 ISSUES FOUND:');
    issues.forEach(i => console.log('  ' + i));
  } else {
    console.log('\n✅ Theme looks correct!');
  }
  
  await browser.close();
}

checkPage().catch(console.error);
