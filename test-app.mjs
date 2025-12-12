import { chromium } from 'playwright';

async function testApp() {
  const browser = await chromium.launch();
  const page = await browser.newPage();

  // Collect console messages
  const consoleMessages = [];
  page.on('console', msg => {
    consoleMessages.push({
      type: msg.type(),
      text: msg.text()
    });
  });

  // Collect errors
  const pageErrors = [];
  page.on('pageerror', error => {
    pageErrors.push(error.toString());
  });

  try {
    console.log('Navigating to http://localhost:3000...');
    await page.goto('http://localhost:3000', {
      waitUntil: 'networkidle',
      timeout: 30000
    });

    // Wait for content to load
    await page.waitForLoadState('domcontentloaded');
    await new Promise(r => setTimeout(r, 2000));

    // Take screenshot
    await page.screenshot({ path: '/tmp/app-screenshot.png', fullPage: true });
    console.log('✓ Screenshot saved to /tmp/app-screenshot.png');

    // Check for content
    const bodyText = await page.textContent('body');
    const hasContent = bodyText && bodyText.length > 50;

    // Check page title
    const title = await page.title();
    console.log(`✓ Page title: "${title}"`);

    // Get HTML length
    const html = await page.content();
    console.log(`✓ HTML content length: ${html.length} characters`);

    // Check for common error patterns
    const errorPatterns = [
      'undefined is not defined',
      'Cannot read properties',
      'Reference error',
      'Type error'
    ];

    const hasErrors = errorPatterns.some(pattern =>
      consoleMessages.some(msg => msg.text.toLowerCase().includes(pattern.toLowerCase()))
    );

    console.log('\n=== RESULTS ===');
    console.log(`Content loaded: ${hasContent ? 'YES' : 'NO'}`);
    console.log(`Page has visible text: ${bodyText ? 'YES (' + bodyText.length + ' chars)' : 'NO'}`);
    console.log(`JavaScript errors: ${pageErrors.length > 0 ? 'YES' : 'NO'}`);
    console.log(`Console errors/warnings: ${hasErrors ? 'YES' : 'NO'}`);

    if (pageErrors.length > 0) {
      console.log('\nJavaScript Errors:');
      pageErrors.forEach(err => console.log('  -', err));
    }

    if (consoleMessages.filter(m => m.type === 'error').length > 0) {
      console.log('\nConsole Errors:');
      consoleMessages.filter(m => m.type === 'error').forEach(msg => {
        console.log('  -', msg.text);
      });
    }

    if (consoleMessages.filter(m => m.type === 'warning').length > 0) {
      console.log('\nConsole Warnings:');
      consoleMessages.filter(m => m.type === 'warning').forEach(msg => {
        console.log('  -', msg.text);
      });
    }

  } catch (error) {
    console.error('ERROR:', error.message);
  } finally {
    await browser.close();
  }
}

testApp();
