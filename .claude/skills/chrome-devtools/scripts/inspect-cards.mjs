import puppeteer from 'puppeteer';

async function inspectCards() {
  const browser = await puppeteer.launch({ headless: true });
  const page = await browser.newPage();
  await page.setViewport({ width: 1440, height: 900 });
  await page.goto('http://localhost:3003/', { waitUntil: 'networkidle2' });
  await new Promise(r => setTimeout(r, 2000));

  // Scroll to features section
  await page.evaluate(() => {
    document.getElementById('features')?.scrollIntoView();
  });
  await new Promise(r => setTimeout(r, 1000));

  // Inspect first feature card
  const cardInfo = await page.evaluate(() => {
    const featureSection = document.getElementById('features');
    if (!featureSection) return { error: 'No features section' };
    
    const cards = featureSection.querySelectorAll('[class*="GlassCard"], [class*="glass"], .rounded-2xl');
    const results = [];
    
    cards.forEach((card, i) => {
      if (i > 2) return; // Only first 3 cards
      const style = getComputedStyle(card);
      const h3 = card.querySelector('h3');
      const p = card.querySelector('p');
      
      results.push({
        index: i,
        cardClasses: card.className.substring(0, 100),
        cardBg: style.backgroundColor,
        cardOpacity: style.opacity,
        h3Text: h3?.textContent?.substring(0, 30),
        h3Color: h3 ? getComputedStyle(h3).color : 'N/A',
        h3Opacity: h3 ? getComputedStyle(h3).opacity : 'N/A',
        pText: p?.textContent?.substring(0, 30),
        pColor: p ? getComputedStyle(p).color : 'N/A',
        innerHTML: card.innerHTML.length,
      });
    });
    
    return results;
  });

  console.log('=== FEATURE CARDS INSPECTION ===\n');
  console.log(JSON.stringify(cardInfo, null, 2));

  await browser.close();
}

inspectCards().catch(console.error);
