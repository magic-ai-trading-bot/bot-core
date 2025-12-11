import puppeteer from 'puppeteer';

async function debugSections() {
  const browser = await puppeteer.launch({ headless: true });
  const page = await browser.newPage();

  // Capture console errors
  const errors = [];
  page.on('console', msg => {
    if (msg.type() === 'error') {
      errors.push(msg.text());
    }
  });
  page.on('pageerror', err => errors.push(err.message));

  await page.setViewport({ width: 1440, height: 900 });
  await page.goto('http://localhost:3003/', { waitUntil: 'networkidle2' });
  await new Promise(r => setTimeout(r, 2000));

  // Check section visibility
  const sectionInfo = await page.evaluate(() => {
    const ids = ['features', 'stats', 'pricing', 'testimonials', 'faq'];
    const results = {};
    
    ids.forEach(id => {
      const section = document.getElementById(id);
      if (section) {
        const rect = section.getBoundingClientRect();
        const style = getComputedStyle(section);
        results[id] = {
          exists: true,
          display: style.display,
          visibility: style.visibility,
          opacity: style.opacity,
          height: rect.height,
          top: rect.top,
          hasContent: section.innerHTML.length > 100,
          childCount: section.children.length,
        };
      } else {
        results[id] = { exists: false };
      }
    });

    // Also check if content is rendering
    const body = document.body;
    results.bodyHeight = body.scrollHeight;
    results.pageContent = document.documentElement.outerHTML.length;
    
    return results;
  });

  console.log('=== SECTION DEBUG ===\n');
  console.log('Section Info:', JSON.stringify(sectionInfo, null, 2));
  
  if (errors.length > 0) {
    console.log('\n=== CONSOLE ERRORS ===');
    errors.forEach(e => console.log('  -', e));
  } else {
    console.log('\nNo console errors detected.');
  }

  // Take screenshot with DevTools to see what's happening
  await page.screenshot({ path: '/tmp/debug-page.png', fullPage: true });

  await browser.close();
}

debugSections().catch(console.error);
