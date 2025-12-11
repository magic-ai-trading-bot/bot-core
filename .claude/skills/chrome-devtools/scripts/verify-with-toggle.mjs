import puppeteer from 'puppeteer';

async function verifyWithToggle() {
  const browser = await puppeteer.launch({ 
    headless: true,
    defaultViewport: { width: 1440, height: 900 }
  });
  
  const page = await browser.newPage();
  
  console.log('🔐 Going to login page...');
  await page.goto('http://localhost:3003/login', { waitUntil: 'networkidle2' });
  await new Promise(r => setTimeout(r, 1000));
  
  // Click "Use Trader" button
  await page.evaluate(() => {
    const buttons = document.querySelectorAll('button');
    for (const btn of buttons) {
      if (btn.textContent && btn.textContent.includes('Use Trader')) {
        btn.click();
        return true;
      }
    }
  });
  await new Promise(r => setTimeout(r, 500));
  
  // Click Sign In
  await page.evaluate(() => {
    const buttons = document.querySelectorAll('button');
    for (const btn of buttons) {
      if (btn.textContent && btn.textContent.includes('Sign In')) {
        btn.click();
        return true;
      }
    }
  });
  await new Promise(r => setTimeout(r, 3000));
  
  console.log('📍 Current URL:', page.url());
  
  // Now on dashboard, use the actual theme toggle
  console.log('\n☀️ Clicking theme toggle to switch to LIGHT mode...');
  
  // Find and click theme toggle button
  const toggleBtn = await page.$('button[aria-label="Toggle theme"]');
  if (toggleBtn) {
    await toggleBtn.click();
    await new Promise(r => setTimeout(r, 500));
    
    // Click "Light" option in dropdown
    await page.evaluate(() => {
      const items = document.querySelectorAll('[role="menuitem"]');
      for (const item of items) {
        if (item.textContent && item.textContent.toLowerCase().includes('light')) {
          item.click();
          return true;
        }
      }
    });
    await new Promise(r => setTimeout(r, 1000));
  }
  
  await page.screenshot({ path: '/tmp/verify-toggle-light.png' });
  console.log('  Screenshot (Light): /tmp/verify-toggle-light.png');
  
  // Now switch to dark
  console.log('\n🌙 Clicking theme toggle to switch to DARK mode...');
  const toggleBtn2 = await page.$('button[aria-label="Toggle theme"]');
  if (toggleBtn2) {
    await toggleBtn2.click();
    await new Promise(r => setTimeout(r, 500));
    
    // Click "Dark" option in dropdown
    await page.evaluate(() => {
      const items = document.querySelectorAll('[role="menuitem"]');
      for (const item of items) {
        if (item.textContent && item.textContent.toLowerCase().includes('dark')) {
          item.click();
          return true;
        }
      }
    });
    await new Promise(r => setTimeout(r, 1000));
  }
  
  await page.screenshot({ path: '/tmp/verify-toggle-dark.png' });
  console.log('  Screenshot (Dark): /tmp/verify-toggle-dark.png');
  
  await browser.close();
  console.log('\n✅ Done!');
}

verifyWithToggle().catch(console.error);
