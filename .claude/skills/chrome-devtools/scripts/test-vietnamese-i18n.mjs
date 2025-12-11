import puppeteer from 'puppeteer';

const footerPages = [
  { name: 'Features', path: '/features', key: 'features.badge' },
  { name: 'Pricing', path: '/pricing', key: 'pricing.badge' },
  { name: 'API', path: '/api', key: 'api.badge' },
  { name: 'Documentation', path: '/docs', key: 'docs.badge' },
  { name: 'About', path: '/about', key: 'about.badge' },
  { name: 'Blog', path: '/blog', key: 'blog.badge' },
  { name: 'Careers', path: '/careers', key: 'careers.badge' },
  { name: 'Contact', path: '/contact', key: 'contact.badge' },
  { name: 'Privacy', path: '/privacy', key: 'privacy.badge' },
  { name: 'Terms', path: '/terms', key: 'terms.badge' },
  { name: 'Security', path: '/security', key: 'security.badge' },
  { name: 'Compliance', path: '/compliance', key: 'compliance.badge' },
];

// Vietnamese badge texts to verify
const vietnameseBadges = {
  'features.badge': 'Tính năng nền tảng',
  'pricing.badge': 'Bảng giá đơn giản',
  'api.badge': 'API cho nhà phát triển',
  'docs.badge': 'Tài liệu',
  'about.badge': 'Về chúng tôi',
  'blog.badge': 'Blog',
  'careers.badge': 'Tuyển dụng',
  'contact.badge': 'Liên hệ',
  'privacy.badge': 'Chính sách bảo mật',
  'terms.badge': 'Điều khoản dịch vụ',
  'security.badge': 'Bảo mật',
  'compliance.badge': 'Tuân thủ',
};

async function testVietnamese() {
  const browser = await puppeteer.launch({ headless: true });
  const page = await browser.newPage();
  await page.setViewport({ width: 1440, height: 900 });

  console.log('🇻🇳 Testing Vietnamese translations for all footer pages...\n');

  let passed = 0;
  let failed = 0;

  for (const p of footerPages) {
    try {
      await page.goto('http://localhost:3003' + p.path, { waitUntil: 'networkidle2', timeout: 10000 });

      // Set Vietnamese language
      await page.evaluate(() => {
        localStorage.setItem('language', 'vi');
        localStorage.setItem('i18nextLng', 'vi');
      });
      await page.reload({ waitUntil: 'networkidle2', timeout: 10000 });
      await new Promise(r => setTimeout(r, 1500));

      // Get page content
      const pageContent = await page.evaluate(() => document.body.innerText);
      const expectedBadge = vietnameseBadges[p.key];

      // Check if Vietnamese badge text exists
      const hasVietnamese = pageContent.includes(expectedBadge);

      // Take screenshot
      const filename = '/tmp/vi-' + p.name.toLowerCase() + '.png';
      await page.screenshot({ path: filename });

      if (hasVietnamese) {
        console.log('✅ ' + p.name.padEnd(15) + ' - Found: "' + expectedBadge + '"');
        passed++;
      } else {
        console.log('❌ ' + p.name.padEnd(15) + ' - Missing: "' + expectedBadge + '"');
        console.log('   Screenshot: ' + filename);
        failed++;
      }
    } catch (err) {
      console.log('⚠️  ' + p.name.padEnd(15) + ' - Error: ' + err.message);
      failed++;
    }
  }

  await browser.close();

  console.log('\n' + '='.repeat(50));
  console.log('Results: ' + passed + ' passed, ' + failed + ' failed');
  console.log('='.repeat(50));

  if (failed === 0) {
    console.log('🎉 All Vietnamese translations working correctly!');
  }
}

testVietnamese().catch(console.error);
