import { chromium } from 'playwright';

async function testDropdown() {
  const browser = await chromium.launch();
  const context = await browser.newContext({ colorScheme: 'light' });
  const page = await context.newPage();

  console.log('Testing theme dropdown menu\n');

  try {
    await page.goto('http://localhost:3002/', { waitUntil: 'networkidle', timeout: 30000 });
    await page.waitForTimeout(2000);

    console.log('Step 1: Find theme toggle button');
    const toggleBtn = await page.$('button[aria-label="Toggle theme"]');
    console.log(`Found: ${toggleBtn ? 'YES' : 'NO'}\n`);

    if (!toggleBtn) {
      throw new Error('Theme toggle button not found');
    }

    console.log('Step 2: Click theme toggle to open dropdown');
    await toggleBtn.click();
    await page.waitForTimeout(300);

    console.log('Step 3: Check for dropdown menu');

    // Look for dropdown menu
    const menuContent = await page.$('[role="menu"]');
    console.log(`Menu visible: ${menuContent ? 'YES' : 'NO'}`);

    if (menuContent) {
      // Get all menu items
      const items = await page.$$('[role="menuitem"]');
      console.log(`Menu items found: ${items.length}`);

      for (let i = 0; i < items.length; i++) {
        const text = await items[i].textContent();
        const isVisible = await items[i].isVisible();
        console.log(`  ${i + 1}. "${text?.trim()}" - visible: ${isVisible}`);
      }

      console.log('\nStep 4: Click on "Dark" option');

      // Try to find and click the dark option
      const menuItems = await page.$$('[role="menuitem"]');
      let darkOption = null;

      for (let item of menuItems) {
        const text = await item.textContent();
        if (text?.includes('Dark') || text?.includes('dark')) {
          darkOption = item;
          break;
        }
      }

      if (darkOption) {
        console.log('Found dark option, clicking...');
        await darkOption.click();
        await page.waitForTimeout(500);

        const afterClick = await page.evaluate(() => ({
          isDark: document.documentElement.classList.contains('dark'),
          theme: localStorage.getItem('theme')
        }));

        console.log(`After click: isDark=${afterClick.isDark}, localStorage.theme="${afterClick.theme}"`);

        if (afterClick.isDark && afterClick.theme === 'dark') {
          console.log('\n✅ Theme toggle IS WORKING via dropdown!');
        } else {
          console.log('\n❌ Theme toggle is NOT responding to dropdown click');
        }
      } else {
        console.log('Could not find dark option in menu');
      }
    } else {
      console.log('\n❌ Dropdown menu did not open!');

      // Let's check what happens after the click
      const allElements = await page.$$('*');
      console.log(`\nTotal elements on page: ${allElements.length}`);

      // Try to find any visible dropdown/popover
      const popovers = await page.$$('[role="dialog"], [role="menu"], .dropdown, [class*="dropdown"]');
      console.log(`Found dropdowns/popovers: ${popovers.length}`);

      // Check if there's any menu hidden
      const hiddenMenus = await page.evaluate(() => {
        return document.querySelectorAll('[role="menu"], [class*="menu"]').length;
      });
      console.log(`Hidden menus in DOM: ${hiddenMenus}`);
    }

  } catch (error) {
    console.error(`Error: ${error.message}`);
  } finally {
    await browser.close();
  }
}

testDropdown().catch(console.error);
