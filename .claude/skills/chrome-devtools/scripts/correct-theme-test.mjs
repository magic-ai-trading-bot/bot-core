import puppeteer from 'puppeteer';

async function test() {
  const browser = await puppeteer.launch({ headless: true });
  const page = await browser.newPage();
  await page.setViewport({ width: 1440, height: 900 });

  // Test all 12 footer pages with CORRECT localStorage key
  const pages = [
    { name: 'Features', path: '/features' },
    { name: 'Pricing', path: '/pricing' },
    { name: 'API', path: '/api' },
    { name: 'Documentation', path: '/docs' },
    { name: 'About', path: '/about' },
    { name: 'Blog', path: '/blog' },
    { name: 'Careers', path: '/careers' },
    { name: 'Contact', path: '/contact' },
    { name: 'Privacy', path: '/privacy' },
    { name: 'Terms', path: '/terms' },
    { name: 'Security', path: '/security' },
    { name: 'Compliance', path: '/compliance' },
  ];

  console.log('Testing all static pages with CORRECT theme key...\n');

  for (const p of pages) {
    await page.goto('http://localhost:3003' + p.path, { waitUntil: 'networkidle2', timeout: 10000 });

    // Set light mode with CORRECT key: 'theme' (not 'vite-ui-theme')
    await page.evaluate(() => {
      localStorage.setItem('theme', 'light');  // Correct key!
    });
    await page.reload({ waitUntil: 'networkidle2' });
    await new Promise(r => setTimeout(r, 800));

    // Check page state
    const info = await page.evaluate(() => {
      const bg = getComputedStyle(document.body).backgroundColor;
      const hasError = document.body.innerHTML.includes('colors is not defined') ||
                       document.body.innerHTML.includes('Something went wrong');
      const htmlClass = document.documentElement.className;
      const h1 = document.querySelector('h1');
      const is404 = h1 && (h1.textContent.includes('404') || h1.textContent.includes('Not Found'));
      return { bg, hasError, htmlClass, is404 };
    });

    // Check if light mode (cream background)
    const isLight = info.bg.includes('250') || info.bg.includes('252') || info.bg.includes('248');
    const status = info.hasError ? '❌ Error' : (info.is404 ? '⚠️ 404' : (isLight ? '✅' : '❓'));

    console.log(status + ' ' + p.name.padEnd(15) + ' - class: ' + info.htmlClass.padEnd(25) + ' bg: ' + info.bg.slice(0,20));

    await page.screenshot({ path: '/tmp/corrected-' + p.name.toLowerCase() + '.png', fullPage: true });
  }

  await browser.close();
  console.log('\n✅ Done!');
}

test().catch(console.error);
