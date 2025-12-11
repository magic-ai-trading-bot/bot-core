import puppeteer from 'puppeteer';

async function debugTheme() {
  const browser = await puppeteer.launch({ 
    headless: true,
    defaultViewport: { width: 1440, height: 900 }
  });
  
  const page = await browser.newPage();
  
  // Enable console logging
  page.on('console', msg => console.log('PAGE:', msg.text()));
  
  console.log('Going to login page...');
  await page.goto('http://localhost:3003/login', { waitUntil: 'networkidle2' });
  await new Promise(r => setTimeout(r, 1000));
  
  // Login
  await page.evaluate(() => {
    const buttons = document.querySelectorAll('button');
    for (const btn of buttons) {
      if (btn.textContent && btn.textContent.includes('Use Trader')) {
        btn.click();
        return;
      }
    }
  });
  await new Promise(r => setTimeout(r, 500));
  
  await page.evaluate(() => {
    const buttons = document.querySelectorAll('button');
    for (const btn of buttons) {
      if (btn.textContent && btn.textContent.includes('Sign In')) {
        btn.click();
        return;
      }
    }
  });
  await new Promise(r => setTimeout(r, 3000));
  
  console.log('On dashboard:', page.url());
  
  // Debug initial state
  const initialState = await page.evaluate(() => {
    return {
      localStorage_theme: localStorage.getItem('theme'),
      html_classList: document.documentElement.classList.toString(),
      html_className: document.documentElement.className,
    };
  });
  console.log('Initial state:', initialState);
  
  // Click theme toggle
  console.log('\nClicking theme toggle...');
  const toggleBtn = await page.$('button[aria-label="Toggle theme"]');
  if (toggleBtn) {
    await toggleBtn.click();
    await new Promise(r => setTimeout(r, 500));
    
    // Check dropdown items
    const menuItems = await page.evaluate(() => {
      const items = document.querySelectorAll('[role="menuitem"]');
      return Array.from(items).map(item => ({
        text: item.textContent,
        html: item.outerHTML.slice(0, 200)
      }));
    });
    console.log('Menu items found:', menuItems);
    
    // Click Light
    await page.evaluate(() => {
      const items = document.querySelectorAll('[role="menuitem"]');
      for (const item of items) {
        if (item.textContent && item.textContent.toLowerCase().includes('light')) {
          console.log('Clicking Light item');
          item.click();
          return true;
        }
      }
      return false;
    });
    await new Promise(r => setTimeout(r, 1000));
  } else {
    console.log('Theme toggle button NOT found!');
  }
  
  // Check state after clicking Light
  const afterLightState = await page.evaluate(() => {
    return {
      localStorage_theme: localStorage.getItem('theme'),
      html_classList: document.documentElement.classList.toString(),
      html_className: document.documentElement.className,
    };
  });
  console.log('After clicking Light:', afterLightState);
  
  await page.screenshot({ path: '/tmp/debug-light.png' });
  console.log('Screenshot: /tmp/debug-light.png');
  
  // Get computed background color
  const bgColor = await page.evaluate(() => {
    const dashboardEl = document.querySelector('.min-h-screen');
    if (dashboardEl) {
      return window.getComputedStyle(dashboardEl).backgroundColor;
    }
    return 'element not found';
  });
  console.log('Dashboard background color:', bgColor);
  
  await browser.close();
  console.log('\nDone!');
}

debugTheme().catch(console.error);
