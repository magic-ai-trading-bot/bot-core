import puppeteer from 'puppeteer';

async function testPaperTradingVietnamese() {
  const browser = await puppeteer.launch({ headless: true });
  const page = await browser.newPage();
  await page.setViewport({ width: 1440, height: 900 });

  // Login first
  console.log('Logging in...');
  await page.goto('http://localhost:3003/login', { waitUntil: 'networkidle2' });
  await new Promise(r => setTimeout(r, 1000));
  
  // Fill credentials
  await page.type('input[type="email"]', 'trader@botcore.com');
  await page.type('input[type="password"]', 'password123');
  await page.click('button[type="submit"]');
  await new Promise(r => setTimeout(r, 3000));

  // Go to Paper Trading page
  console.log('Going to Paper Trading...');
  await page.goto('http://localhost:3003/trading/paper', { waitUntil: 'networkidle2' });
  await new Promise(r => setTimeout(r, 2000));

  // Set Vietnamese language
  console.log('Setting Vietnamese language...');
  await page.evaluate(() => {
    localStorage.setItem('i18nextLng', 'vi');
    localStorage.setItem('language', 'vi');
  });
  await page.reload({ waitUntil: 'networkidle2' });
  await new Promise(r => setTimeout(r, 2000));

  // Screenshot
  await page.screenshot({ path: '/tmp/paper-trading-vi-fixed.png', fullPage: true });
  console.log('Screenshot saved: /tmp/paper-trading-vi-fixed.png');

  // Get all text on page to check for translation issues
  const pageText = await page.evaluate(() => {
    return document.body.innerText;
  });
  
  // Check for untranslated keys
  const untranslatedKeys = pageText.match(/paperTradingPage\.[a-zA-Z.]+|PAPERTRADINGPAGE\.[A-Z.]+/g);
  if (untranslatedKeys && untranslatedKeys.length > 0) {
    console.log('\n⚠️ Found untranslated keys:');
    [...new Set(untranslatedKeys)].forEach(key => console.log('  - ' + key));
  } else {
    console.log('\n✅ No untranslated keys found!');
  }

  await browser.close();
}

testPaperTradingVietnamese().catch(console.error);
