// Keeping a Rockoon Battle seat across a page reload (src/games/rockoon-wars/seat.mjs).
import test from 'node:test';
import assert from 'node:assert/strict';
import worker from '../src/worker.mjs';
import { loadSeat, saveSeat } from '../src/games/rockoon-wars/seat.mjs';
import { ROOM_NAMESPACE } from '../src/games/rockoon-wars/room.mjs';

/** A stand-in for sessionStorage: one tab's store, which a reload keeps. */
function tabStore() {
  const map = new Map();
  return {
    getItem: k => (map.has(k) ? map.get(k) : null),
    setItem: (k, v) => { map.set(k, String(v)); },
    removeItem: k => { map.delete(k); },
    map,
  };
}

const seat = { player: '0123456789ab', ticket: 'A'.repeat(43), creator: true, joined: 1_700_000_000_000 };

test('a saved seat reads back for the same room, and only that room', () => {
  const store = tabStore();
  assert.equal(saveSeat(store, 'K7QX2', seat), true);
  assert.deepEqual(loadSeat(store, 'K7QX2'), seat);
  assert.equal(loadSeat(store, 'M3RT8'), null);
});

test('two rooms keep their own seats in one tab', () => {
  const store = tabStore();
  saveSeat(store, 'K7QX2', seat);
  saveSeat(store, 'M3RT8', { ...seat, player: 'ba9876543210', creator: false });
  assert.equal(loadSeat(store, 'K7QX2').player, '0123456789ab');
  assert.equal(loadSeat(store, 'M3RT8').player, 'ba9876543210');
  assert.equal(loadSeat(store, 'M3RT8').creator, false);
});

test('a missing, blocked or damaged store reads as no seat', () => {
  assert.equal(loadSeat(undefined, 'K7QX2'), null);
  assert.equal(loadSeat({ getItem() { throw new Error('blocked'); } }, 'K7QX2'), null);
  const store = tabStore();
  for (const raw of ['', 'not json', 'null', '[]', '{"player":"xyz","ticket":"AAAAAAAAAAAAAAAAAAAA"}',
    JSON.stringify({ ...seat, ticket: 'bad ticket!' }), JSON.stringify({ ...seat, player: '0123456789AB' })]) {
    store.setItem('rockoon-seat:K7QX2', raw);
    assert.equal(loadSeat(store, 'K7QX2'), null, raw);
  }
});

test('only a real flag makes a room maker, and a bad join time is replaced', () => {
  const store = tabStore();
  store.setItem('rockoon-seat:K7QX2', JSON.stringify({ ...seat, creator: 'yes', joined: 'soon' }));
  const back = loadSeat(store, 'K7QX2');
  assert.equal(back.creator, false);
  assert.ok(Number.isFinite(back.joined) && back.joined > 0);
});

test('saving refuses a bad seat or room, and survives a full store', () => {
  const store = tabStore();
  assert.equal(saveSeat(store, 'K7QX2', { ...seat, player: '' }), false);
  assert.equal(saveSeat(store, 'K7QX2', { ...seat, ticket: '' }), false);
  assert.equal(saveSeat(store, '../x', seat), false);
  assert.equal(store.map.size, 0);
  assert.equal(saveSeat({ setItem() { throw new Error('QuotaExceededError'); } }, 'K7QX2', seat), false);
});

// Through the Worker, as the page does: join, save the seat, reload, ask again with the saved seat.
const pair = await crypto.subtle.generateKey({ name: 'ECDSA', namedCurve: 'P-256' }, true, ['sign', 'verify']);
const jwk = { ...(await crypto.subtle.exportKey('jwk', pair.privateKey)), kid: 'test-kid', alg: 'ES256', key_ops: ['sign', 'verify'] };
const env = {
  ASSETS: { fetch: async () => new Response('page') },
  MOQ_GAMES_SIGNING_KEY: Buffer.from(JSON.stringify(jwk)).toString('base64url'),
  MOQ_GAMES_RELAY: 'https://cdn.moq.pro/',
  MOQ_GAMES_ROOT: 'proj/games',
  GAME_TOKEN_LIMIT: { limit: async () => ({ success: true }) },
};
const token = async body => (await worker.fetch(new Request('https://cubesatclass.org/api/games/moq-token', {
  method: 'POST',
  headers: { origin: 'https://cubesatclass.org', 'sec-fetch-site': 'same-origin', 'content-type': 'application/json' },
  body: JSON.stringify({ game: ROOM_NAMESPACE, ...body }),
}), env)).json();

test('after a reload the saved seat gets the same player id back from the Worker', async () => {
  const store = tabStore();
  const first = await token({ room: 'K7QX2' });
  assert.equal(saveSeat(store, 'K7QX2', { player: first.player, ticket: first.ticket, creator: true, joined: 1 }), true);
  // ...the page reloads; the tab's session storage is still there...
  const saved = loadSeat(store, 'K7QX2');
  const again = await token({ room: 'K7QX2', player: saved.player, ticket: saved.ticket });
  assert.equal(again.player, first.player);
  assert.equal(again.callsign, first.callsign);
  // A fresh tab (no saved seat) is a new player.
  const other = await token({ room: 'K7QX2' });
  assert.notEqual(other.player, first.player);
});
