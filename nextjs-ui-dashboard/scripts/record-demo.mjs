#!/usr/bin/env node
/**
 * Bot Core - Demo Video Recording Script
 *
 * This script uses Puppeteer to navigate through the website
 * and create a screen recording for demo purposes.
 *
 * Usage: node scripts/record-demo.mjs
 *
 * Requirements:
 * - npm install puppeteer puppeteer-screen-recorder
 * - Website running on localhost:5173 (or change BASE_URL)
 */

import puppeteer from 'puppeteer';
import { PuppeteerScreenRecorder } from 'puppeteer-screen-recorder';
import path from 'path';
import { fileURLToPath } from 'url';

const __dirname = path.dirname(fileURLToPath(import.meta.url));

// Configuration
const CONFIG = {
  baseUrl: 'http://localhost:3004',
  outputDir: path.join(__dirname, '../docs/demo'),
  videoFile: 'bot-core-demo.mp4',
  viewport: { width: 1920, height: 1080 },
  recordingOptions: {
    followNewTab: false,
    fps: 30,
    videoFrame: {
      width: 1920,
      height: 1080,
    },
    videoCrf: 18,
    videoCodec: 'libx264',
    videoPreset: 'ultrafast',
    videoBitrate: 8000,
    aspectRatio: '16:9',
  },
};

// Demo flow - Pages and actions to record
const DEMO_FLOW = [
  {
    name: 'Landing Page - Hero',
    action: async (page) => {
      await page.goto(CONFIG.baseUrl, { waitUntil: 'networkidle0' });
      await sleep(3000); // Let 3D animation play
    },
  },
  {
    name: 'Landing Page - Scroll to Features',
    action: async (page) => {
      await smoothScroll(page, 800);
      await sleep(2000);
    },
  },
  {
    name: 'Landing Page - Scroll to Stats',
    action: async (page) => {
      await smoothScroll(page, 600);
      await sleep(2000);
    },
  },
  {
    name: 'Landing Page - Scroll to Pricing',
    action: async (page) => {
      await smoothScroll(page, 800);
      await sleep(2000);
    },
  },
  {
    name: 'Toggle Theme - Light Mode',
    action: async (page) => {
      await page.goto(CONFIG.baseUrl, { waitUntil: 'networkidle0' });
      await sleep(1000);
      // Click theme toggle button
      const themeBtn = await page.$('[data-testid="theme-toggle"], button[aria-label*="theme"], .theme-toggle');
      if (themeBtn) {
        await themeBtn.click();
        await sleep(2000);
      }
    },
  },
  {
    name: 'Toggle Theme - Back to Dark',
    action: async (page) => {
      const themeBtn = await page.$('[data-testid="theme-toggle"], button[aria-label*="theme"], .theme-toggle');
      if (themeBtn) {
        await themeBtn.click();
        await sleep(2000);
      }
    },
  },
  {
    name: 'Navigate to Features',
    action: async (page) => {
      await page.goto(`${CONFIG.baseUrl}/features`, { waitUntil: 'networkidle0' });
      await sleep(2000);
      await smoothScroll(page, 500);
      await sleep(1500);
    },
  },
  {
    name: 'Navigate to Pricing',
    action: async (page) => {
      await page.goto(`${CONFIG.baseUrl}/pricing`, { waitUntil: 'networkidle0' });
      await sleep(2000);
      await smoothScroll(page, 400);
      await sleep(1500);
    },
  },
  {
    name: 'Navigate to Login',
    action: async (page) => {
      await page.goto(`${CONFIG.baseUrl}/login`, { waitUntil: 'networkidle0' });
      await sleep(2000);
    },
  },
  {
    name: 'Navigate to Register',
    action: async (page) => {
      await page.goto(`${CONFIG.baseUrl}/register`, { waitUntil: 'networkidle0' });
      await sleep(2000);
    },
  },
  {
    name: 'Back to Landing - Final Shot',
    action: async (page) => {
      await page.goto(CONFIG.baseUrl, { waitUntil: 'networkidle0' });
      await sleep(3000);
    },
  },
];

// Helper functions
function sleep(ms) {
  return new Promise(resolve => setTimeout(resolve, ms));
}

async function smoothScroll(page, distance) {
  await page.evaluate(async (scrollDistance) => {
    const steps = 20;
    const stepDistance = scrollDistance / steps;
    for (let i = 0; i < steps; i++) {
      window.scrollBy(0, stepDistance);
      await new Promise(r => setTimeout(r, 30));
    }
  }, distance);
}

async function ensureDir(dirPath) {
  const fs = await import('fs');
  if (!fs.existsSync(dirPath)) {
    fs.mkdirSync(dirPath, { recursive: true });
  }
}

// Main recording function
async function recordDemo() {
  console.log('🎬 Bot Core Demo Recording');
  console.log('='.repeat(50));

  // Ensure output directory exists
  await ensureDir(CONFIG.outputDir);

  const outputPath = path.join(CONFIG.outputDir, CONFIG.videoFile);

  console.log(`\n📁 Output: ${outputPath}`);
  console.log(`🖥️  Viewport: ${CONFIG.viewport.width}x${CONFIG.viewport.height}`);
  console.log(`🎯 FPS: ${CONFIG.recordingOptions.fps}`);
  console.log(`📍 Base URL: ${CONFIG.baseUrl}\n`);

  // Launch browser
  console.log('🚀 Launching browser...');
  const browser = await puppeteer.launch({
    headless: false, // Show browser for visual feedback
    defaultViewport: CONFIG.viewport,
    args: [
      `--window-size=${CONFIG.viewport.width},${CONFIG.viewport.height}`,
      '--disable-web-security',
      '--disable-features=IsolateOrigins,site-per-process',
    ],
  });

  const page = await browser.newPage();
  await page.setViewport(CONFIG.viewport);

  // Initialize recorder
  console.log('🎥 Initializing screen recorder...\n');
  const recorder = new PuppeteerScreenRecorder(page, CONFIG.recordingOptions);

  try {
    // Start recording
    await recorder.start(outputPath);
    console.log('⏺️  Recording started!\n');

    // Execute demo flow
    for (let i = 0; i < DEMO_FLOW.length; i++) {
      const step = DEMO_FLOW[i];
      console.log(`[${i + 1}/${DEMO_FLOW.length}] ${step.name}`);
      await step.action(page);
    }

    // Stop recording
    await recorder.stop();
    console.log('\n⏹️  Recording stopped!');
    console.log(`\n✅ Demo video saved to: ${outputPath}`);

  } catch (error) {
    console.error('\n❌ Recording error:', error.message);
    await recorder.stop();
  } finally {
    await browser.close();
    console.log('\n🎬 Recording complete!');
  }
}

// Alternative: Screenshot-based approach (if video recording fails)
async function captureScreenshots() {
  console.log('📸 Bot Core Screenshot Capture (Fallback)');
  console.log('='.repeat(50));

  await ensureDir(CONFIG.outputDir);

  const browser = await puppeteer.launch({
    headless: 'new',
    defaultViewport: CONFIG.viewport,
  });

  const page = await browser.newPage();

  const screenshots = [
    { name: '01-landing-hero', url: '/', scroll: 0 },
    { name: '02-landing-features', url: '/', scroll: 800 },
    { name: '03-landing-stats', url: '/', scroll: 1600 },
    { name: '04-features', url: '/features', scroll: 0 },
    { name: '05-pricing', url: '/pricing', scroll: 0 },
    { name: '06-login', url: '/login', scroll: 0 },
    { name: '07-register', url: '/register', scroll: 0 },
  ];

  for (const shot of screenshots) {
    const url = `${CONFIG.baseUrl}${shot.url}`;
    console.log(`📸 Capturing: ${shot.name}`);

    await page.goto(url, { waitUntil: 'networkidle0' });
    if (shot.scroll > 0) {
      await page.evaluate((y) => window.scrollTo(0, y), shot.scroll);
    }
    await sleep(1000);

    await page.screenshot({
      path: path.join(CONFIG.outputDir, `${shot.name}.png`),
      fullPage: false,
    });
  }

  await browser.close();
  console.log(`\n✅ Screenshots saved to: ${CONFIG.outputDir}`);
  console.log('\n💡 Tip: Use ffmpeg to create video from screenshots:');
  console.log('   ffmpeg -framerate 1 -pattern_type glob -i "*.png" -c:v libx264 -pix_fmt yuv420p demo.mp4');
}

// Run
const mode = process.argv[2] || 'video';

if (mode === 'screenshots') {
  captureScreenshots().catch(console.error);
} else {
  recordDemo().catch(console.error);
}
