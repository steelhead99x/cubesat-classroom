// Accessibility audit of the built site (dist/), against WCAG 2.2 AA.
//   npm run build && npm run audit:a11y
// Needs Chrome or Chromium: set CHROME=/path/to/chrome if it isn't at a usual place.
//
// For every page, at desktop and phone sizes: axe-core's WCAG 2.0–2.2 A and AA rules. Then,
// by hand-written checks: one h1, a lang attribute, a working skip link, unique titles, no
// sideways scrolling at 320 px wide (WCAG 1.4.10), and a visible focus ring on everything Tab
// reaches (2.4.7). Both games are also checked in each state a player sees: the first card,
// the menus and shop, and play. Exits 1 if anything fails.
import { createServer } from 'node:http';
import { existsSync, readFileSync, readdirSync, statSync } from 'node:fs';
import { extname, join, normalize } from 'node:path';
import { fileURLToPath } from 'node:url';
import { chromium } from 'playwright-core';

const dist = fileURLToPath(new URL('../dist/', import.meta.url));
const axe = readFileSync(fileURLToPath(new URL('../node_modules/axe-core/axe.min.js', import.meta.url)), 'utf8');
const chrome = process.env.CHROME || ['/usr/bin/google-chrome', '/usr/bin/chromium', '/usr/bin/chromium-browser',
  '/Applications/Google Chrome.app/Contents/MacOS/Google Chrome'].find(existsSync);
if (!chrome) { console.error('Chrome not found: set CHROME=/path/to/chrome.'); process.exit(1); }

const types = { '.html': 'text/html', '.js': 'text/javascript', '.mjs': 'text/javascript', '.css': 'text/css', '.svg': 'image/svg+xml',
  '.png': 'image/png', '.woff2': 'font/woff2', '.woff': 'font/woff', '.json': 'application/json', '.wasm': 'application/wasm', '.ico': 'image/x-icon' };
const server = createServer((req, res) => {
  let path = normalize(decodeURIComponent(new URL(req.url, 'http://x').pathname)).replace(/^(\.\.[/\\])+/, '');
  let file = join(dist, path);
  if (existsSync(file) && statSync(file).isDirectory()) file = join(file, 'index.html');
  if (!existsSync(file)) { res.writeHead(404); return res.end(); }
  res.writeHead(200, { 'content-type': types[extname(file)] || 'application/octet-stream' });
  res.end(readFileSync(file));
}).listen(0);
const origin = `http://localhost:${server.address().port}`;

const pages = [];
(function walk(dir, url) {
  for (const name of readdirSync(dir)) {
    const full = join(dir, name);
    if (statSync(full).isDirectory()) walk(full, `${url}${name}/`);
    else if (name === 'index.html') pages.push(url);
  }
})(dist, '/');
pages.sort();

const failures = [];
const fail = text => failures.push(text);
const browser = await chromium.launch({ executablePath: chrome, args: ['--no-sandbox'] });
const rules = { runOnly: ['wcag2a', 'wcag2aa', 'wcag21a', 'wcag21aa', 'wcag22aa'] };
async function axeOn(page, where, scope = 'document') {
  await page.addScriptTag({ content: axe });
  const result = await page.evaluate(async ([s, r]) => await window.axe.run(s === 'document' ? document : document.querySelector(s), r), [scope, rules]);
  for (const v of result.violations) for (const n of v.nodes.slice(0, 3)) fail(`${where}: ${v.id} at ${n.target.join(' ')}: ${(n.failureSummary || '').split('\n')[1]?.trim() ?? v.help}`);
}

const titles = new Map();
for (const [size, viewport] of [['desktop', { width: 1280, height: 900 }], ['phone', { width: 390, height: 844 }]]) {
  const context = await browser.newContext({ viewport, bypassCSP: true });
  const page = await context.newPage();
  for (const path of pages) {
    await page.goto(origin + path, { waitUntil: 'load' });
    await axeOn(page, `${path} (${size})`);
  }
  await context.close();
}

for (const path of pages) {
  const context = await browser.newContext({ viewport: { width: 320, height: 700 }, bypassCSP: true });
  const page = await context.newPage();
  await page.goto(origin + path, { waitUntil: 'load' });
  const r = await page.evaluate(() => ({
    lang: document.documentElement.lang, h1: document.querySelectorAll('h1').length, title: document.title,
    overflow: document.documentElement.scrollWidth - innerWidth,
    skip: (() => { const a = document.querySelector('a.skip[href^="#"]'); return !!a && !!document.querySelector(a.getAttribute('href')); })(),
  }));
  if (!r.lang) fail(`${path}: no lang attribute`);
  if (r.h1 !== 1) fail(`${path}: ${r.h1} h1 headings`);
  if (!r.skip) fail(`${path}: no working skip link`);
  if (r.overflow > 1) fail(`${path}: scrolls sideways by ${r.overflow}px at 320 px wide`);
  if (titles.has(r.title)) fail(`${path}: same title as ${titles.get(r.title)}`); else titles.set(r.title, path);
  await page.setViewportSize({ width: 1280, height: 900 });
  for (let i = 0; i < 40; i++) {
    await page.keyboard.press('Tab');
    const f = await page.evaluate(() => {
      const e = document.activeElement;
      if (!e || e === document.body) return null;
      const s = getComputedStyle(e);
      return { name: `${e.tagName.toLowerCase()} "${(e.textContent || e.getAttribute('aria-label') || '').trim().slice(0, 30)}"`, skip: e.classList.contains('skip'),
        ring: (s.outlineStyle !== 'none' && parseFloat(s.outlineWidth) >= 2) || (s.boxShadow && s.boxShadow !== 'none') };
    });
    if (!f) break;
    if (i === 0 && !f.skip) fail(`${path}: the first Tab goes to ${f.name}, not the skip link`);
    if (!f.ring) fail(`${path}: no visible focus on ${f.name}`);
  }
  await context.close();
}

// The games, in each state a player sees.
for (const path of ['/games/rockoon-wars/', '/games/catch-the-pass/']) {
  for (const viewport of [{ width: 1280, height: 900 }, { width: 390, height: 844 }]) {
    const context = await browser.newContext({ viewport, bypassCSP: true });
    const page = await context.newPage();
    await page.goto(origin + path); await page.evaluate(() => localStorage.clear()); await page.reload();
    const where = state => `${path} ${viewport.width}px, ${state}`;
    await axeOn(page, where('first card'), 'section[data-rw], section[data-ctp]');
    for (let i = 0; i < 8; i++) {
      const buttons = await page.locator('[data-overlay]:not([hidden]) button:not([disabled])').allInnerTexts();
      const pick = buttons.find(t => /got it/i.test(t)) ?? buttons.find(t => /this computer/i.test(t)) ?? buttons.find(t => t.trim() === '2') ?? buttons.find(t => /done|start/i.test(t));
      if (!pick) break;
      await page.locator('[data-overlay]:not([hidden]) button', { hasText: pick }).first().click();
      await page.waitForTimeout(300);
      await axeOn(page, where(`after "${pick.trim()}"`), 'section[data-rw], section[data-ctp]');
    }
    await page.waitForTimeout(2500);
    await axeOn(page, where('playing'), 'section[data-rw], section[data-ctp]');
    await context.close();
  }
}

await browser.close();
server.close();
console.log(`Checked ${pages.length} pages and both games.`);
if (failures.length) { console.error([...new Set(failures)].join('\n')); process.exit(1); }
console.log('No accessibility failures found.');
