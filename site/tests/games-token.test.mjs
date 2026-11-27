// POST /api/games/moq-token (src/games-token.mjs), through the Worker as a browser would call it.
import test from 'node:test';
import assert from 'node:assert/strict';
import worker from '../src/worker.mjs';

const ASSETS = { fetch: async () => new Response('page', { headers: { 'content-type': 'text/html' } }) };
const allow = { limit: async () => ({ success: true }) };
const deny = { limit: async () => ({ success: false }) };

const pair = await crypto.subtle.generateKey({ name: 'ECDSA', namedCurve: 'P-256' }, true, ['sign', 'verify']);
const privateJwk = { ...(await crypto.subtle.exportKey('jwk', pair.privateKey)), kid: 'test-kid', alg: 'ES256', key_ops: ['sign', 'verify'] };
// `moq auth generate` writes the JWK as base64url text; accept that form.
const keyText = Buffer.from(JSON.stringify(privateJwk)).toString('base64url');
const configured = { MOQ_GAMES_SIGNING_KEY: keyText, MOQ_GAMES_RELAY: 'https://cdn.moq.pro/', MOQ_GAMES_ROOT: 'proj/games', GAME_TOKEN_LIMIT: allow };

const ask = (env, { origin = 'https://cubesatclass.org', site = 'same-origin', body = { game: 'catch-the-pass' }, method = 'POST', type = 'application/json' } = {}) =>
  worker.fetch(new Request('https://cubesatclass.org/api/games/moq-token', {
    method,
    headers: { ...(origin ? { origin } : {}), ...(site ? { 'sec-fetch-site': site } : {}), 'content-type': type },
    ...(method === 'POST' ? { body: JSON.stringify(body) } : {}),
  }), { ASSETS, ...env });

const part = (jwt, i) => JSON.parse(Buffer.from(jwt.split('.')[i], 'base64url').toString());

test('a same-site request gets a short-lived token scoped to its own station', async () => {
  const res = await ask(configured);
  assert.equal(res.status, 200);
  assert.equal(res.headers.get('cache-control'), 'no-store');
  const body = await res.json();
  assert.match(body.player, /^[0-9a-f]{12}$/);
  assert.match(body.callsign, /^GS-[A-Z]{2}\d$/);
  const url = new URL(body.url);
  assert.equal(url.origin, 'https://cdn.moq.pro');
  assert.equal(url.pathname, '/proj/games/catch-the-pass/live');
  const jwt = url.searchParams.get('jwt');
  assert.deepEqual(part(jwt, 0), { typ: 'JWT', alg: 'ES256', kid: 'test-kid' });
  const claims = part(jwt, 1);
  assert.equal(claims.root, 'proj/games/catch-the-pass/live');
  assert.deepEqual(claims.put, [body.player]);
  assert.deepEqual(claims.get, ['']);
  assert.ok(claims.exp - claims.iat <= 600 && claims.exp > Date.now() / 1000);
  const [h, p, s] = jwt.split('.');
  const ok = await crypto.subtle.verify({ name: 'ECDSA', hash: 'SHA-256' }, pair.publicKey, Buffer.from(s, 'base64url'), new TextEncoder().encode(h + '.' + p));
  assert.ok(ok, 'signature verifies with the public key');
  assert.doesNotMatch(JSON.stringify(body), new RegExp(privateJwk.d), 'the private key is never returned');
});

test('each request gets a different player', async () => {
  const a = await (await ask(configured)).json();
  const b = await (await ask(configured)).json();
  assert.notEqual(a.player, b.player);
});

test('other sites, missing origins and cross-site fetches are refused', async () => {
  assert.equal((await ask(configured, { origin: 'https://evil.example' })).status, 403);
  assert.equal((await ask(configured, { origin: null })).status, 403);
  assert.equal((await ask(configured, { site: 'cross-site' })).status, 403);
  assert.equal((await ask(configured, { type: 'text/plain' })).status, 415);
  assert.equal((await ask(configured, { method: 'GET' })).status, 405);
});

test('unknown games and the rate limit are refused', async () => {
  assert.equal((await ask(configured, { body: { game: 'other' } })).status, 400);
  assert.equal((await ask({ ...configured, GAME_TOKEN_LIMIT: deny })).status, 429);
});

test('without the key, relay or limiter the game is told to play offline', async () => {
  for (const missing of ['MOQ_GAMES_SIGNING_KEY', 'MOQ_GAMES_RELAY', 'MOQ_GAMES_ROOT', 'GAME_TOKEN_LIMIT']) {
    const res = await ask({ ...configured, [missing]: undefined });
    assert.equal(res.status, 503, missing);
    assert.equal((await res.json()).offline, true);
  }
  const res = await ask({ ...configured, MOQ_GAMES_RELAY: 'http://insecure.example' });
  assert.equal(res.status, 503);
});

test('a broken key fails closed without leaking it', async () => {
  const errors = [];
  const original = console.error;
  console.error = (...args) => errors.push(args.join(' '));
  try {
    const res = await ask({ ...configured, MOQ_GAMES_SIGNING_KEY: '{"kty":"EC","crv":"P-256","d":"bad-secret-value","x":"1","y":"2"}' });
    assert.equal(res.status, 503);
    assert.doesNotMatch(await res.text(), /bad-secret-value/);
    assert.doesNotMatch(errors.join('\n'), /bad-secret-value/);
  } finally {
    console.error = original;
  }
});

test('HS256 and Ed25519 keys work too', async () => {
  const hs = await crypto.subtle.exportKey('jwk', await crypto.subtle.generateKey({ name: 'HMAC', hash: 'SHA-256' }, true, ['sign']));
  const ed = await crypto.subtle.exportKey('jwk', (await crypto.subtle.generateKey({ name: 'Ed25519' }, true, ['sign', 'verify'])).privateKey);
  for (const [key, alg] of [[hs, 'HS256'], [ed, 'EdDSA']]) {
    const res = await ask({ ...configured, MOQ_GAMES_SIGNING_KEY: JSON.stringify(key) });
    assert.equal(res.status, 200, alg);
    assert.equal(part(new URL((await res.json()).url).searchParams.get('jwt'), 0).alg, alg);
  }
});

test('Rockoon Battle tokens are scoped to one room, joined by code', async () => {
  const room = 'K7QX2';
  const body = await (await ask(configured, { body: { game: 'rockoon-battle', room } })).json();
  const url = new URL(body.url);
  assert.equal(url.pathname, `/proj/games/rockoon-battle/${room}`);
  const claims = part(url.searchParams.get('jwt'), 1);
  assert.equal(claims.root, `proj/games/rockoon-battle/${room}`);
  assert.deepEqual(claims.put, [body.player]);
  assert.equal(body.room, room);
  assert.match(body.ticket, /^[A-Za-z0-9_-]{43}$/);
  for (const bad of [undefined, '', 'k7qx2', 'K7QX', 'K7QX2/../x', 'O0O0O', '../live']) {
    assert.equal((await ask(configured, { body: { game: 'rockoon-battle', room: bad } })).status, 400, String(bad));
  }
});

test('a seat ticket keeps the same player id in the same room, and only there', async () => {
  const first = await (await ask(configured, { body: { game: 'rockoon-battle', room: 'K7QX2' } })).json();
  const again = await (await ask(configured, { body: { game: 'rockoon-battle', room: 'K7QX2', player: first.player, ticket: first.ticket } })).json();
  assert.equal(again.player, first.player);
  assert.equal(again.callsign, first.callsign);
  const forged = await (await ask(configured, { body: { game: 'rockoon-battle', room: 'K7QX2', player: first.player, ticket: 'x'.repeat(43) } })).json();
  assert.notEqual(forged.player, first.player);
  const otherRoom = await (await ask(configured, { body: { game: 'rockoon-battle', room: 'ZZZZ2', player: first.player, ticket: first.ticket } })).json();
  assert.notEqual(otherRoom.player, first.player);
});

test('Catch the Pass stays on its one shared sky', async () => {
  const body = await (await ask(configured, { body: { game: 'catch-the-pass', room: 'K7QX2' } })).json();
  assert.equal(new URL(body.url).pathname, '/proj/games/catch-the-pass/live');
  assert.equal(body.ticket, undefined);
});

test('an oversized request is refused before its JSON is parsed', async () => {
  const big = { game: 'catch-the-pass', pad: 'x'.repeat(5000) };
  const res = await ask(configured, { body: big });
  assert.equal(res.status, 413);
  // A streamed body with no length header is cut off at the limit too.
  const stream = new ReadableStream({
    start(c) { for (let i = 0; i < 64; i++) c.enqueue(new TextEncoder().encode('x'.repeat(1024))); c.close(); },
  });
  const res2 = await worker.fetch(new Request('https://cubesatclass.org/api/games/moq-token', {
    method: 'POST', duplex: 'half', body: stream,
    headers: { origin: 'https://cubesatclass.org', 'sec-fetch-site': 'same-origin', 'content-type': 'application/json' },
  }), { ASSETS, ...configured });
  assert.equal(res2.status, 413);
  // A normal request still works.
  assert.equal((await ask(configured)).status, 200);
});

test('the token reply carries the server time for the shared clock', async () => {
  const before = Date.now();
  const body = await (await ask(configured)).json();
  assert.ok(body.now >= before && body.now <= Date.now());
});
