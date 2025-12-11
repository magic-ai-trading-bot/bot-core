import puppeteer from 'puppeteer';

const footerPages = [
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

async function test() {
  const browser = await puppeteer.launch({ headless: true });
  const page = await browser.newPage();
  await page.setViewport({ width: 1440, height: 900 });

  console.log('Testing all footer pages in LIGHT mode...\n');

  for (const p of footerPages) {
    try {
      await page.goto('http://localhost:3003' + p.path, { waitUntil: 'networkidle2', timeout: 10000 });

      // Click theme toggle and select light
      const btn = await page.$('button[aria-label="Toggle theme"]');
      if (btn) {
        await btn.click();
        await new Promise(r => setTimeout(r, 300));
        await page.evaluate(() => {
          const items = document.querySelectorAll('[role="menuitem"]');
          for (const item of items) {
            if (item.textContent && item.textContent.toLowerCase().includes('light')) {
              item.click();
              return;
            }
          }
        });
        await new Promise(r => setTimeout(r, 500));
      }

      // Scroll to trigger animations
      const totalHeight = await page.evaluate(() => document.body.scrollHeight);
      for (let y = 0; y < totalHeight; y += 500) {
        await page.evaluate((pos) => window.scrollTo(0, pos), y);
        await new Promise(r => setTimeout(r, 100));
      }
      await page.evaluate(() => window.scrollTo(0, 0));
      await new Promise(r => setTimeout(r, 200));

      // Check background color and text color
      const info = await page.evaluate(() => {
        const bg = getComputedStyle(document.body).backgroundColor;
        const h1 = document.querySelector('h1');
        const h1Color = h1 ? getComputedStyle(h1).color : 'no-h1';
        const is404 = h1 && (h1.textContent.includes('404') || h1.textContent.includes('Not Found'));
        return { bg, h1Color, is404 };
      });

      const isLight = info.bg.includes('252') || info.bg.includes('250') || info.bg.includes('248');
      const status = info.is404 ? '⚠️ 404' : (isLight ? '✅' : '❓');
      console.log(status + ' ' + p.name.padEnd(15) + ' (' + p.path.padEnd(12) + ') - bg: ' + info.bg.slice(0,25) + ', h1: ' + info.h1Color.slice(0,20));

      await page.screenshot({ path: '/tmp/static-' + p.name.toLowerCase() + '-light.png', fullPage: true });
    } catch (err) {
      console.log('❌ ' + p.name.padEnd(15) + ' - Error: ' + err.message.slice(0, 50));
    }
  }

  await browser.close();
  console.log('\n✅ Done! Screenshots: /tmp/static-*.png');
}

test().catch(console.error);
