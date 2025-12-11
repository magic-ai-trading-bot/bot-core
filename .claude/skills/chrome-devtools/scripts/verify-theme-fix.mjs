import puppeteer from 'puppeteer';

async function verifyThemeFix() {
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

  // Login using demo button
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

  // Check initial state
  const initialState = await page.evaluate(() => {
    return {
      localStorage_theme: localStorage.getItem('theme'),
      html_class: document.documentElement.className,
    };
  });
  console.log('Initial state:', initialState);

  // Screenshot BEFORE theme change
  await page.screenshot({ path: '/tmp/theme-before.png' });
  console.log('Screenshot before: /tmp/theme-before.png');

  // Find and click theme toggle button
  console.log('\n--- CHANGING TO LIGHT MODE ---');

  // Method 1: Try clicking the theme toggle button
  const toggleClicked = await page.evaluate(() => {
    // Find button with Toggle theme aria-label
    const btn = document.querySelector('button[aria-label*="Toggle"]');
    if (btn) {
      console.log('Found toggle button:', btn.outerHTML.slice(0, 100));
      btn.click();
      return true;
    }
    return false;
  });
  console.log('Toggle button clicked:', toggleClicked);

  // Wait for dropdown to appear (Radix UI portal)
  await new Promise(r => setTimeout(r, 500));

  // Check what's in the DOM
  const dropdownState = await page.evaluate(() => {
    // Radix dropdowns use data-radix-popper-content-wrapper
    const radixDropdown = document.querySelector('[data-radix-popper-content-wrapper]');
    const menuItems = document.querySelectorAll('[role="menuitem"]');
    const menuItemsRadix = document.querySelectorAll('[data-radix-collection-item]');

    return {
      radixDropdownFound: !!radixDropdown,
      radixDropdownHTML: radixDropdown ? radixDropdown.innerHTML.slice(0, 500) : null,
      menuItemsCount: menuItems.length,
      menuItemsRadixCount: menuItemsRadix.length,
      menuItemsText: Array.from(menuItems).map(m => m.textContent),
    };
  });
  console.log('Dropdown state:', JSON.stringify(dropdownState, null, 2));

  // Click Light option using different methods
  const lightClicked = await page.evaluate(() => {
    // Try role="menuitem"
    const items = document.querySelectorAll('[role="menuitem"]');
    for (const item of items) {
      const text = item.textContent?.toLowerCase() || '';
      if (text.includes('light') || text.includes('sáng')) {
        console.log('Clicking Light item:', item.textContent);
        item.click();
        return 'menuitem';
      }
    }

    // Try data-radix-collection-item
    const radixItems = document.querySelectorAll('[data-radix-collection-item]');
    for (const item of radixItems) {
      const text = item.textContent?.toLowerCase() || '';
      if (text.includes('light') || text.includes('sáng')) {
        console.log('Clicking Radix Light item:', item.textContent);
        item.click();
        return 'radix';
      }
    }

    // Try clicking any dropdown content that contains Light
    const allElements = document.querySelectorAll('*');
    for (const el of allElements) {
      if (el.childElementCount === 0) {
        const text = el.textContent?.toLowerCase() || '';
        if ((text.includes('light') || text.includes('sáng')) && el.closest('[data-radix-popper-content-wrapper]')) {
          console.log('Clicking element with Light text:', el.textContent);
          el.click();
          return 'text-element';
        }
      }
    }

    return false;
  });
  console.log('Light option clicked:', lightClicked);

  // Wait for theme change
  await new Promise(r => setTimeout(r, 1000));

  // Check state AFTER
  const afterState = await page.evaluate(() => {
    const dashboardEl = document.querySelector('.min-h-screen');
    return {
      localStorage_theme: localStorage.getItem('theme'),
      html_class: document.documentElement.className,
      bgColor: dashboardEl ? window.getComputedStyle(dashboardEl).backgroundColor : 'not found',
    };
  });
  console.log('After state:', afterState);

  // Screenshot AFTER theme change
  await page.screenshot({ path: '/tmp/theme-after-light.png' });
  console.log('Screenshot after (light): /tmp/theme-after-light.png');

  // Also try directly setting theme via localStorage and reloading
  console.log('\n--- FORCE LIGHT MODE VIA LOCALSTORAGE ---');
  await page.evaluate(() => {
    localStorage.setItem('theme', 'light');
  });
  await page.reload({ waitUntil: 'networkidle2' });
  await new Promise(r => setTimeout(r, 1500));

  const forceState = await page.evaluate(() => {
    const dashboardEl = document.querySelector('.min-h-screen');
    return {
      localStorage_theme: localStorage.getItem('theme'),
      html_class: document.documentElement.className,
      bgColor: dashboardEl ? window.getComputedStyle(dashboardEl).backgroundColor : 'not found',
    };
  });
  console.log('Force light state:', forceState);

  await page.screenshot({ path: '/tmp/theme-force-light.png' });
  console.log('Screenshot force light: /tmp/theme-force-light.png');

  await browser.close();
  console.log('\nDone!');
}

verifyThemeFix().catch(console.error);
