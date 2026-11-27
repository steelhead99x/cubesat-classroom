// Behaviour tests for src/worker.mjs. Run with `npm test` (node:test, no extra packages).
import test from 'node:test';
import assert from 'node:assert/strict';
import worker from '../src/worker.mjs';

const ASSETS = {
  fetch: async (request) => {
    const { pathname } = new URL(request.url);
    if (pathname === '/404.html') {
      return new Response('<!doctype html><title>Not found</title><p>Page not found</p>', {
        headers: { 'content-type': 'text/html; charset=utf-8' },
      });
    }
    if (pathname === '/missing' || pathname === '/missing.js') {
      return new Response('nope', { status: 404, headers: { 'content-type': 'text/plain' } });
    }
    return new Response('<!doctype html><p>page</p>', { headers: { 'content-type': 'text/html' } });
  },
};
const call = (url, init = {}, env = {}) => worker.fetch(new Request(url, init), { ASSETS, ...env });

function fakeKv() {
  const store = new Map();
  return { store, get: async k => store.get(k) ?? null, put: async (k, v) => { store.set(k, String(v)); } };
}
const basic = pw => 'Basic ' + btoa('user:' + pw);
const post = (body, headers = {}) => ({ method: 'POST', headers: { 'content-type': 'application/json', ...headers }, body: JSON.stringify(body) });

test('www on the classroom host and hub hosts redirects to the apex, keeping path and query', async () => {
  for (const host of ['cubesatclass.org', 'cubesat.app', 'cubesatclass.io', 'cubesatclass.dev']) {
    const res = await call(`https://www.${host}/quests/x?a=1`);
    assert.equal(res.status, 301);
    assert.equal(res.headers.get('location'), `https://${host}/quests/x?a=1`);
  }
});

test('redirect domains and their www go to cubesatclass.org in one hop', async () => {
  for (const host of ['cubesat.club', 'www.cubesat.club', 'cubesatclass.com', 'www.cubesatclass.com']) {
    const res = await call(`https://${host}/start.sh?x=1`);
    assert.equal(res.status, 301);
    assert.equal(res.headers.get('location'), 'https://cubesatclass.org/start.sh?x=1');
  }
});

test('a missing page is the site 404, and a missing script stays a plain 404', async () => {
  const page = await call('https://cubesatclass.org/missing', { headers: { accept: 'text/html' } });
  assert.equal(page.status, 404);
  assert.match(page.headers.get('content-type'), /text\/html/);
  assert.match(await page.text(), /Page not found/);
  const script = await call('https://cubesatclass.org/missing.js', { headers: { accept: '*/*' } });
  assert.equal(script.status, 404);
  assert.equal(await script.text(), 'nope');
});

test('apex hosts are served, with security headers', async () => {
  const res = await call('https://cubesatclass.org/');
  assert.equal(res.status, 200);
  assert.match(res.headers.get('strict-transport-security'), /includeSubDomains/);
  assert.match(res.headers.get('content-security-policy'), /script-src 'self'/);
  assert.match(res.headers.get('content-security-policy'), /connect-src 'self' https:\/\/cdn\.moq\.pro wss:\/\/cdn\.moq\.pro;/);
  assert.equal(res.headers.get('x-content-type-options'), 'nosniff');
});

test('/start.sh is plain text, nosniff and no-cache', async () => {
  const res = await call('https://cubesatclass.org/start.sh');
  assert.match(res.headers.get('content-type'), /^text\/plain/);
  assert.equal(res.headers.get('x-content-type-options'), 'nosniff');
  assert.equal(res.headers.get('cache-control'), 'no-cache');
});

test('guide is off (503) without ANTHROPIC_API_KEY, and GET is refused', async () => {
  const res = await call('https://cubesatclass.org/api/guide', post({ quest: '', question: 'what-is-cubesat', level: 'beginner' }));
  assert.equal(res.status, 503);
  assert.equal((await call('https://cubesatclass.org/api/guide')).status, 405);
});

test('guide checks Origin when on', async () => {
  const env = { ANTHROPIC_API_KEY: 'test-key' };
  const body = { quest: '', question: 'what-is-cubesat', level: 'beginner' };
  assert.equal((await call('https://cubesatclass.org/api/guide', post(body), env)).status, 403);
  assert.equal((await call('https://cubesatclass.org/api/guide', post(body, { origin: 'https://evil.example' }), env)).status, 403);
  // Same origin passes the check; without KV and a salt it answers from the prewritten text.
  const ok = await call('https://cubesatclass.org/api/guide', post(body, { origin: 'https://cubesatclass.org' }), env);
  assert.equal(ok.status, 200);
  assert.equal((await ok.json()).fallback, true);
  const listed = { ...env, GUIDE_ALLOWED_ORIGINS: 'https://a.example, https://b.example' };
  assert.equal((await call('https://cubesatclass.org/api/guide', post(body, { origin: 'https://b.example' }), listed)).status, 200);
  assert.equal((await call('https://cubesatclass.org/api/guide', post(body, { origin: 'https://cubesatclass.org' }), listed)).status, 403);
});

test('guide rejects unknown questions and oversized bodies', async () => {
  const env = { ANTHROPIC_API_KEY: 'test-key' };
  const origin = { origin: 'https://cubesatclass.org' };
  assert.equal((await call('https://cubesatclass.org/api/guide', post({ question: 'nope' }, origin), env)).status, 400);
  assert.equal((await call('https://cubesatclass.org/api/guide', post({ question: 'x'.repeat(2000) }, origin), env)).status, 413);
});

test('preview gate: locked without a password, open with Basic auth, open when unset', async () => {
  const env = { SITE_PASSWORD: 'hunter2' };
  const locked = await call('https://cubesatclass.org/', {}, env);
  assert.equal(locked.status, 401);
  assert.match(await locked.text(), /Preview password/);
  assert.equal((await call('https://cubesatclass.org/', { headers: { authorization: basic('wrong') } }, env)).status, 401);
  const open = await call('https://cubesatclass.org/', { headers: { authorization: basic('hunter2') } }, env);
  assert.equal(open.status, 200);
  assert.match(open.headers.get('cache-control'), /no-store/);
  assert.equal((await call('https://cubesatclass.org/')).status, 200);
});

test('preview gate never blocks redirect domains', async () => {
  const res = await call('https://cubesat.club/', {}, { SITE_PASSWORD: 'hunter2' });
  assert.equal(res.status, 301);
});

test('preview form needs PREVIEW_COOKIE_SECRET to sign anyone in', async () => {
  const form = pw => ({ method: 'POST', body: new URLSearchParams({ password: pw }) });
  const env = { SITE_PASSWORD: 'hunter2' };
  assert.equal((await call('https://cubesatclass.org/preview-unlock', form('hunter2'), env)).status, 401);
  const res = await call('https://cubesatclass.org/preview-unlock', form('hunter2'), { ...env, PREVIEW_COOKIE_SECRET: 's'.repeat(32) });
  assert.equal(res.status, 303);
  assert.match(res.headers.get('set-cookie'), /^__Host-preview=\d+\.[0-9a-f]{64}; .*HttpOnly; Secure/);
});

test('wrong form and Basic guesses share one limit', async () => {
  const kv = fakeKv();
  const env = { SITE_PASSWORD: 'hunter2', GUIDE_LIMITS: kv, GUIDE_SALT: 'salt', PREVIEW_UNLOCK_LIMIT: '4' };
  const ip = { 'cf-connecting-ip': '203.0.113.9' };
  const form = { method: 'POST', headers: ip, body: new URLSearchParams({ password: 'bad' }) };
  for (let i = 0; i < 2; i++) await call('https://cubesatclass.org/preview-unlock', form, env);
  for (let i = 0; i < 2; i++) await call('https://cubesatclass.org/', { headers: { ...ip, authorization: basic('bad') } }, env);
  assert.equal([...kv.store.values()].join(), '4');
  // Now locked out, even for the right password, by either route.
  const byBasic = await call('https://cubesatclass.org/', { headers: { ...ip, authorization: basic('hunter2') } }, env);
  assert.equal(byBasic.status, 401);
  assert.match(await byBasic.text(), /Too many wrong passwords/);
  const byForm = await call('https://cubesatclass.org/preview-unlock', { ...form, body: new URLSearchParams({ password: 'hunter2' }) }, { ...env, PREVIEW_COOKIE_SECRET: 's'.repeat(32) });
  assert.match(await byForm.text(), /Too many wrong passwords/);
  // Another address is unaffected.
  const other = await call('https://cubesatclass.org/', { headers: { 'cf-connecting-ip': '198.51.100.1', authorization: basic('hunter2') } }, env);
  assert.equal(other.status, 200);
});

test('a right Basic password signs the visitor in with the cookie, so later files skip the check', async () => {
  const env = { SITE_PASSWORD: 'hunter2', PREVIEW_COOKIE_SECRET: 's'.repeat(32) };
  const res = await call('https://cubesatclass.org/', { headers: { authorization: basic('hunter2') } }, env);
  assert.equal(res.status, 200);
  const cookie = res.headers.get('set-cookie');
  assert.match(cookie, /^__Host-preview=\d+\.[0-9a-f]{64}; /);
  const next = await call('https://cubesatclass.org/style.css', { headers: { cookie: cookie.split(';')[0] } }, env);
  assert.equal(next.status, 200);
});

test('password attempts are rate-limited per address even without the KV store', async () => {
  let count = 0;
  const UNLOCK_LIMIT = { limit: async () => ({ success: ++count <= 3 }) };
  const env = { SITE_PASSWORD: 'hunter2', PREVIEW_COOKIE_SECRET: 's'.repeat(32), UNLOCK_LIMIT };
  const ip = { 'cf-connecting-ip': '203.0.113.7' };
  for (let i = 0; i < 3; i++) {
    const res = await call('https://cubesatclass.org/', { headers: { ...ip, authorization: basic('bad') } }, env);
    assert.doesNotMatch(await res.text(), /Too many/);
  }
  const blocked = await call('https://cubesatclass.org/', { headers: { ...ip, authorization: basic('hunter2') } }, env);
  assert.match(await blocked.text(), /Too many wrong passwords/);
});

test('without the cookie secret the limiter is not used, so Basic users are not locked out by their own page loads', async () => {
  const UNLOCK_LIMIT = { limit: async () => ({ success: false }) };
  const env = { SITE_PASSWORD: 'hunter2', UNLOCK_LIMIT };
  const res = await call('https://cubesatclass.org/', { headers: { authorization: basic('hunter2') } }, env);
  assert.equal(res.status, 200);
});

test('hashed /_astro/ files are cached for a year; pages revalidate', async () => {
  const file = await call('https://cubesatclass.org/_astro/app.abc123.css');
  assert.equal(file.headers.get('cache-control'), 'public, max-age=31536000, immutable');
  const page = await call('https://cubesatclass.org/hardware/');
  assert.notEqual(page.headers.get('cache-control'), 'public, max-age=31536000, immutable');
  assert.equal(page.headers.get('cross-origin-opener-policy'), 'same-origin');
});

test('the preview gate keeps hashed files private', async () => {
  const res = await call('https://cubesatclass.org/_astro/app.abc123.css', { headers: { authorization: 'Basic ' + btoa('u:pw') } }, { SITE_PASSWORD: 'pw' });
  assert.match(res.headers.get('cache-control'), /^private/);
});

test('only the rockoon workbook may compile WebAssembly', async () => {
  const workbook = await call('https://cubesatclass.org/rockoon/workbook/');
  assert.match(workbook.headers.get('content-security-policy'), /^script-src 'self' 'wasm-unsafe-eval'; /);
  const home = await call('https://cubesatclass.org/');
  assert.match(home.headers.get('content-security-policy'), /^script-src 'self'; /);
  assert.doesNotMatch(home.headers.get('content-security-policy'), /wasm-unsafe-eval|unsafe-inline/);
});
