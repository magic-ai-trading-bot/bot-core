import puppeteer from 'puppeteer';

async function test() {
  const browser = await puppeteer.launch({ headless: true });
  const page = await browser.newPage();
  await page.setViewport({ width: 1440, height: 900 });
  
  // Go to Features page
  await page.goto('http://localhost:3003/features', { waitUntil: 'networkidle2', timeout: 10000 });
  await new Promise(r => setTimeout(r, 500));
  
  // Set light mode via localStorage
  await page.evaluate(() => {
    localStorage.setItem('vite-ui-theme', 'light');
    document.documentElement.classList.remove('dark');
    document.documentElement.classList.add('light');
  });
  await page.reload({ waitUntil: 'networkidle2' });
  await new Promise(r => setTimeout(r, 1000));
  
  // Check page state
  const info = await page.evaluate(() => {
    const body = document.body;
    const html = document.documentElement;
    const bg = getComputedStyle(body).backgroundColor;
    const hasError = document.body.innerHTML.includes('colors is not defined');
    const h1 = document.querySelector('h1');
    const h1Text = h1 ? h1.textContent : 'no-h1';
    const h1Color = h1 ? getComputedStyle(h1).color : 'none';
    return { 
      bg, 
      hasError, 
      h1Text: h1Text.substring(0, 50),
      h1Color,
      htmlClass: html.className 
    };
  });
  
  console.log('Features Page Test:');
  console.log('  Background:', info.bg);
  console.log('  Has Error:', info.hasError);
  console.log('  H1 Text:', info.h1Text);
  console.log('  H1 Color:', info.h1Color);
  console.log('  HTML Class:', info.htmlClass);
  
  await page.screenshot({ path: '/tmp/features-quick-test.png', fullPage: true });
  console.log('\nScreenshot: /tmp/features-quick-test.png');
  
  await browser.close();
}

test().catch(console.error);
