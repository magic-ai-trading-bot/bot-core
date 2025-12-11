import puppeteer from 'puppeteer';

const footerPages = [
  // Product
  { name: 'Features', path: '/features' },
  { name: 'Pricing', path: '/pricing' },
  { name: 'API', path: '/api' },
  { name: 'Documentation', path: '/docs' },
  // Company
  { name: 'About', path: '/about' },
  { name: 'Blog', path: '/blog' },
  { name: 'Careers', path: '/careers' },
  { name: 'Contact', path: '/contact' },
  // Legal
  { name: 'Privacy', path: '/privacy' },
  { name: 'Terms', path: '/terms' },
  { name: 'Security', path: '/security' },
  { name: 'Compliance', path: '/compliance' },
];

async function testFooterPages() {
  const browser = await puppeteer.launch({ headless: true });
  const page = await browser.newPage();
  await page.setViewport({ width: 1440, height: 900 });
  
  console.log('Testing all footer pages in LIGHT mode...\n');
  
  for (const p of footerPages) {
    await page.goto('http://localhost:3003' + p.path, { waitUntil: 'networkidle2' });
    
    // Set light mode
    await page.evaluate(() => {
      localStorage.setItem('theme', 'light');
      localStorage.setItem('vite-ui-theme', 'light');
      document.documentElement.classList.remove('dark');
      document.documentElement.classList.add('light');
    });
    await page.reload({ waitUntil: 'networkidle2' });
    await new Promise(r => setTimeout(r, 1000));
    
    // Scroll to trigger animations
    const totalHeight = await page.evaluate(() => document.body.scrollHeight);
    for (let y = 0; y < totalHeight; y += 500) {
      await page.evaluate((pos) => window.scrollTo(0, pos), y);
      await new Promise(r => setTimeout(r, 100));
    }
    await page.evaluate(() => window.scrollTo(0, 0));
    await new Promise(r => setTimeout(r, 300));
    
    // Check background color
    const bgColor = await page.evaluate(() => {
      return getComputedStyle(document.body).backgroundColor;
    });
    
    // Check if it's a 404 page
    const is404 = await page.evaluate(() => {
      const h1 = document.querySelector('h1');
      return h1 && (h1.textContent.includes('404') || h1.textContent.includes('Not Found'));
    });
    
    const status = is404 ? '⚠️ 404' : (bgColor.includes('250') || bgColor.includes('252') || bgColor.includes('248') ? '✅' : '❓');
    console.log(`${status} ${p.name.padEnd(15)} (${p.path.padEnd(12)}) - bg: ${bgColor}`);
    
    // Take screenshot
    const filename = `/tmp/footer-${p.name.toLowerCase()}-light.png`;
    await page.screenshot({ path: filename, fullPage: true });
  }
  
  await browser.close();
  console.log('\n✅ Done! Screenshots saved to /tmp/footer-*.png');
}

testFooterPages().catch(console.error);
