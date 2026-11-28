import assert from 'node:assert/strict';
import test from 'node:test';
import { handleRoom, ROOM_LIFETIME_MS } from '../src/games/rockoon-wars/authority.mjs';
import { matchFor, apply } from '../src/games/rockoon-wars/room.mjs';
import { fingerprint } from '../src/games/rockoon-wars/logic.mjs';
import worker from '../src/worker.mjs';

function lobby() {
  let room = null, now = 1_000_000;
  const send = (kind, ticket, args = {}) => {
    const result = handleRoom(structuredClone(room), { kind, ticket, ...args }, now);
    room = result.room;
    return result;
  };
  const a = send('create').body.seat;
  const b = send('join').body.seat;
  return { a, b, send, begin() { return send('start', a.ticket).body.state; },
    advance(ms) { now += ms; }, get room() { return room; } };
}

test('a settled shop survives creator reload and late or duplicate submissions', () => {
  const { a, b, send, begin } = lobby();
  const watcher = send('join').body.seat;
  assert.equal(send('start', b.ticket).body.state.host, null);
  assert.deepEqual(begin().host.order, [a.player, b.player, watcher.player]);
  send('shop', a.ticket, { version: 0, buys: ['rocket'] });
  send('shop', a.ticket, { version: 0, buys: ['big'] });
  send('shop', b.ticket, { version: 0, buys: [] });
  const committed = send('shop', watcher.ticket, { version: 0, buys: [] }).body.state;
  assert.deepEqual(committed.actions, [{ kind: 'shop', buys: [['rocket'], [], []] }]);
  const rejoined = send('join', a.ticket).body;
  assert.equal(rejoined.seat.player, a.player);
  assert.deepEqual(rejoined.state, committed);
  assert.deepEqual(send('shop', a.ticket, { version: 0, buys: ['dirt'] }).body.state, committed);
});

test('a scheduled skip wins over a late shot and every seat replays one history', () => {
  const game = lobby();
  const { a, b, send, begin } = game;
  begin();
  send('shop', a.ticket, { version: 0, buys: [] });
  let state = send('shop', b.ticket, { version: 0, buys: [] }).body.state;
  const who = state.waiting.ids[0] === a.player ? a : b;
  const other = who === a ? b : a;
  const shot = { angle: 90, power: 40, weapon: 'baby' };
  assert.equal(send('fire', who.ticket, { version: 1, shot }).body.state.actions.length, 1);
  send('rise', who.ticket, { version: 1 });
  game.advance(120_000);
  assert.equal(send('fire', who.ticket, { version: 1, shot }).body.state.actions.at(-1).kind, 'skip');
  send('rise', other.ticket, { version: 2 });
  assert.equal(send('fire', who.ticket, { version: 2, shot }).body.state.actions.length, 2);
  state = send('fire', other.ticket, { version: 2, shot }).body.state;
  assert.deepEqual(state.actions.at(-1), { kind: 'fire', shot });
  const retry = send('fire', other.ticket, { version: 2, shot: { ...shot, angle: 80 } }).body.state;
  assert.deepEqual(retry, state);
  assert.deepEqual(send('join', who.ticket).body.state, state);
  const replay = matchFor(state.host);
  for (const action of state.actions) apply(replay, action);
  assert.equal(fingerprint(replay), fingerprint(game.room.match));
});

test('a creator who leaves the lobby can be replaced, and the room expires', () => {
  const game = lobby();
  game.advance(25_000);
  game.send('sync', game.b.ticket);
  game.advance(5_001);
  const friend = game.send('join').body.seat;
  assert.equal(game.send('sync', game.a.ticket).status, 403);
  assert.deepEqual(game.send('start', game.b.ticket).body.state.host.order, [game.b.player, friend.player]);
  game.advance(ROOM_LIFETIME_MS);
  assert.equal(game.send('sync', game.b.ticket).status, 404);
  const replacement = game.send('create').body;
  assert.equal(replacement.state.host, null);
  assert.equal(game.send('sync', game.b.ticket).status, 403);
});

test('invalid shop payloads cannot settle the shop, and missing picks expire', () => {
  const game = lobby();
  const { a, b, send, begin } = game;
  begin();
  assert.deepEqual(send('shop', a.ticket, { version: 0, buys: ['__proto__'] }).body.state.waiting.ids, [a.player, b.player]);
  send('shop', b.ticket, { version: 0, buys: ['rocket'] });
  game.advance(120_000);
  assert.deepEqual(send('shop', a.ticket, { version: 0, buys: ['dirt'] }).body.state.actions,
    [{ kind: 'shop', buys: [[], ['rocket']] }]);
});

test('the Worker bounds and rate-limits the new same-origin endpoint', async () => {
  const calls = [];
  const limiter = { limit: async () => ({ success: true }) };
  const env = {
    GAME_TOKEN_LIMIT: limiter, ROCKOON_REQUEST_LIMIT: limiter,
    ROCKOON_ROOMS: { getByName: name => ({ request: async body => {
      calls.push({ name, body });
      return { status: 200, body: { ok: true } };
    } }) },
  };
  const post = (body, opts = {}) => worker.fetch(new Request('https://cubesatclass.org/api/games/rockoon-room', {
    method: 'POST', headers: { origin: 'https://cubesatclass.org', 'content-type': 'application/json', ...opts },
    body: JSON.stringify(body),
  }), env);
  assert.equal((await post({ room: 'K7QX2', kind: 'join' }, { origin: 'https://example.com' })).status, 403);
  assert.equal((await post({ room: 'K7QX2', kind: 'sync' })).status, 403);
  assert.equal((await post({ room: 'K7QX2', kind: 'join', extra: 'x'.repeat(1100) })).status, 413);
  assert.equal(calls.length, 0);
  assert.equal((await post({ room: 'K7QX2', kind: 'join' })).status, 200);
  assert.equal(calls[0].name, 'K7QX2');
  env.ROCKOON_REQUEST_LIMIT = { limit: async () => ({ success: false }) };
  assert.equal((await post({ room: 'K7QX2', kind: 'sync', ticket: 'a'.repeat(48) })).status, 429);
  assert.equal(calls.length, 1);
});
