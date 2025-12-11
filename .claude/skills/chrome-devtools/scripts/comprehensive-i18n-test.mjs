import puppeteer from 'puppeteer';

const allPages = [
  { name: 'Landing', path: '/', namespace: 'landing' },
  { name: 'Login', path: '/login', namespace: 'auth' },
  { name: 'Register', path: '/register', namespace: 'auth' },
  { name: 'Features', path: '/features', namespace: 'landing' },
  { name: 'Pricing', path: '/pricing', namespace: 'landing' },
  { name: 'API', path: '/api', namespace: 'landing' },
  { name: 'Documentation', path: '/docs', namespace: 'landing' },
  { name: 'HowItWorks', path: '/how-it-works', namespace: 'landing' },
  { name: 'About', path: '/about', namespace: 'landing' },
  { name: 'Blog', path: '/blog', namespace: 'landing' },
  { name: 'Careers', path: '/careers', namespace: 'landing' },
  { name: 'Contact', path: '/contact', namespace: 'landing' },
  { name: 'Privacy', path: '/privacy', namespace: 'landing' },
  { name: 'Terms', path: '/terms', namespace: 'landing' },
  { name: 'Security', path: '/security', namespace: 'landing' },
  { name: 'Compliance', path: '/compliance', namespace: 'landing' },
  { name: 'Dashboard', path: '/dashboard', namespace: 'dashboard', protected: true },
  { name: 'PaperTrading', path: '/trading/paper', namespace: 'trading', protected: true },
  { name: 'RealTrading', path: '/trading/real', namespace: 'trading', protected: true },
  { name: 'AISignals', path: '/signals', namespace: 'trading', protected: true },
  { name: 'TradeAnalyses', path: '/trade-analyses', namespace: 'trading', protected: true },
  { name: 'Portfolio', path: '/portfolio', namespace: 'trading', protected: true },
  { name: 'Settings', path: '/settings', namespace: 'settings', protected: true },
  { name: 'Profile', path: '/profile', namespace: 'settings', protected: true },
];

async function testAllPages() {
  const browser = await puppeteer.launch({ headless: true });
  const page = await browser.newPage();
  await page.setViewport({ width: 1440, height: 900 });

  console.log('Logging in...');
  await page.goto('http://localhost:3003/login', { waitUntil: 'networkidle2' });
  await page.type('input[type="email"]', 'trader@botcore.com');
  await page.type('input[type="password"]', 'password123');
  await page.click('button[type="submit"]');
  await new Promise(r => setTimeout(r, 2000));

  console.log('Setting Vietnamese...');
  await page.evaluate(() => { localStorage.setItem('language', 'vi'); });

  console.log('\n=== TESTING ALL PAGES ===\n');

  const results = [];

  for (const p of allPages) {
    process.stdout.write(p.name.padEnd(15) + ' ');
    
    try {
      await page.goto('http://localhost:3003' + p.path, { waitUntil: 'networkidle2' });
      await new Promise(r => setTimeout(r, 1200));
      
      const analysis = await page.evaluate(() => {
        const bodyText = document.body.innerText;
        
        // Find untranslated keys
        const keyMatches = bodyText.match(/\b[a-z]+[A-Z][a-zA-Z]*\.[a-z]+/g) || [];
        const upperMatches = bodyText.match(/\b[A-Z]{3,}[A-Z_]*\.[A-Z_]+/g) || [];
        const foundKeys = [...new Set([...keyMatches, ...upperMatches])];
        
        // Check Vietnamese
        const hasVi = /[àáạảãâầấậẩẫăằắặẳẵèéẹẻẽêềếệểễìíịỉĩòóọỏõôồốộổỗơờớợởỡùúụủũưừứựửữỳýỵỷỹđ]/i.test(bodyText);
        
        return { foundKeys: foundKeys.slice(0, 5), hasVi };
      });
      
      let status = '✅';
      let msg = '';
      
      if (analysis.foundKeys.length > 0) {
        status = '❌';
        msg = 'Keys: ' + analysis.foundKeys.join(', ');
      } else if (!analysis.hasVi) {
        status = '⚠️';
        msg = 'No Vietnamese text';
      }
      
      console.log(status + ' ' + msg);
      results.push({ name: p.name, status, msg });
      
    } catch (err) {
      console.log('❌ Error: ' + err.message);
      results.push({ name: p.name, status: '❌', msg: err.message });
    }
  }
  
  await browser.close();
  
  console.log('\n=== SUMMARY ===');
  const passed = results.filter(r => r.status === '✅').length;
  const warned = results.filter(r => r.status === '⚠️').length;
  const failed = results.filter(r => r.status === '❌').length;
  console.log('Passed: ' + passed + ' | Warnings: ' + warned + ' | Failed: ' + failed);
  
  if (failed > 0) {
    console.log('\nFailed pages:');
    results.filter(r => r.status === '❌').forEach(r => console.log('  - ' + r.name + ': ' + r.msg));
  }
  if (warned > 0) {
    console.log('\nWarning pages (no Vietnamese):');
    results.filter(r => r.status === '⚠️').forEach(r => console.log('  - ' + r.name));
  }
}

testAllPages().catch(console.error);
