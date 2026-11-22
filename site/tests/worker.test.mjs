// Behaviour tests for src/worker.mjs. Run with `npm test` (node:test, no extra packages).
import test from 'node:test';
import assert from 'node:assert/strict';
import worker from '../src/worker.mjs';

const ASSETS = { fetch: async () => new Response('<!doctype html><p>page</p>', { headers: { 'content-type': 'text/html' } }) };
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

test('apex hosts are served, with security headers', async () => {
  const res = await call('https://cubesatclass.org/');
  assert.equal(res.status, 200);
  assert.match(res.headers.get('strict-transport-security'), /includeSubDomains/);
  assert.match(res.headers.get('content-security-policy'), /script-src 'self'/);
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
