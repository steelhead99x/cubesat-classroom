// Rules for /games/rockoon-wars/ (src/games/rockoon-wars/logic.mjs).
import test from 'node:test';
import assert from 'node:assert/strict';
import * as R from '../src/games/rockoon-wars/logic.mjs';

const players = n => Array.from({ length: n }, (_, i) => ({ id: 'p' + i, name: 'P' + (i + 1) }));
const begin = (n = 2, seed = 42, rounds = 3) => R.startRound(R.createMatch({ seed, players: players(n), rounds }));

test('the sine polynomial matches Math.sin to 1e-9 on whole degrees', () => {
  for (let d = -360; d <= 360; d++) {
    assert.ok(Math.abs(R.sinDeg(d) - Math.sin(d * Math.PI / 180)) < 1e-9, `sin ${d}`);
    assert.ok(Math.abs(R.cosDeg(d) - Math.cos(d * Math.PI / 180)) < 1e-9, `cos ${d}`);
  }
});

test('terrain is the same for the same seed, in range, and different for another seed', () => {
  const a = R.makeTerrain(7), b = R.makeTerrain(7), c = R.makeTerrain(8);
  assert.deepEqual(a, b);
  assert.notDeepEqual(a, c);
  assert.equal(a.length, R.W);
  assert.ok(a.every(h => h >= 40 && h <= 300));
});

test('2 to 4 players only', () => {
  assert.throws(() => R.createMatch({ seed: 1, players: players(1) }));
  assert.throws(() => R.createMatch({ seed: 1, players: players(5) }));
  for (const n of [2, 3, 4]) assert.equal(begin(n).field.units.length, n);
});

test('rockoons start apart, on the ground, at full health', () => {
  const m = begin(4);
  const xs = m.field.units.map(u => u.x).sort((a, b) => a - b);
  for (let i = 1; i < xs.length; i++) assert.ok(xs[i] - xs[i - 1] > 60);
  for (const u of m.field.units) { assert.equal(u.hp, 100); assert.equal(u.y, m.field.terrain[u.x]); }
});

test('a straight-up shot with no wind comes back down near the shooter', () => {
  const m = begin(2, 3);
  m.field.wind = 0;
  const u = m.field.units[m.turn];
  const { end } = R.trajectory(m, u, 90, 40);
  assert.ok(end && Math.abs(end.x - u.x) < 2);
});

test('wind pushes a shot sideways', () => {
  const m = begin(2, 3);
  const u = m.field.units[m.turn];
  m.field.wind = 10; const right = R.trajectory(m, u, 90, 40).end;
  m.field.wind = -10; const left = R.trajectory(m, u, 90, 40).end;
  assert.ok(right.x > u.x + 5 && left.x < u.x - 5);
});

test('more power goes farther', () => {
  const m = begin(2, 5);
  m.field.wind = 0;
  m.field.terrain.fill(50);
  const u = m.field.units[m.turn]; u.y = 50;
  const others = m.field.units.filter(o => o !== u); for (const o of others) o.alive = false;
  u.x = 100;
  const near = R.trajectory(m, u, 45, 30).end, far = R.trajectory(m, u, 45, 50).end;
  assert.ok(Math.abs(far.x - u.x) > Math.abs(near.x - u.x));
});

test('a blast digs a crater and dirt builds a hill', () => {
  const t = new Array(R.W).fill(100);
  R.carve(t, 300, 100, 20);
  assert.equal(t[300], 80);
  assert.equal(t[100], 100);
  const d = new Array(R.W).fill(100);
  R.carve(d, 300, 100, 20, true);
  assert.equal(d[300], 120);
});

test('a direct hit damages, a kill pays, and the last one standing wins the round', () => {
  const m = begin(2, 11, 2);
  m.field.wind = 0;
  const [a, b] = m.field.units;
  // Put the target right under a straight-up shot.
  const shooter = m.turn, target = 1 - shooter;
  m.field.units[target].x = m.field.units[shooter].x; m.field.units[target].y = m.field.units[shooter].y;
  m.field.units[target].hp = 10;
  const before = m.players[shooter].money;
  const { events } = R.fire(m, { angle: 90, power: 30, weapon: 'baby' });
  assert.ok(events.some(e => e.type === 'kill' && e.who === target));
  assert.ok(events.some(e => e.type === 'round' && e.winner === shooter) || !m.field.units[shooter].alive);
  assert.ok(m.players[shooter].money > before || !m.field.units[shooter].alive);
  assert.equal(m.phase, 'shop');
  void a; void b;
});

test('turns go round the living players only', () => {
  const m = begin(3, 9);
  const first = m.turn;
  const dead = (first + 1) % 3;
  m.field.units[dead].alive = false;
  R.skip(m);
  assert.equal(m.turn, (first + 2) % 3);
  R.skip(m);
  assert.equal(m.turn, first);
});

test('ammo is spent, and an empty weapon fires a Baby Rocket', () => {
  const m = R.createMatch({ seed: 4, players: players(2) });
  assert.ok(R.buy(m, 0, 'rocket'));
  assert.equal(m.players[0].ammo.rocket, 5);
  assert.equal(m.players[0].money, R.START_MONEY - 150);
  assert.equal(R.buy(m, 1, 'mega'), false, 'not enough money');
  R.startRound(m);
  m.turn = 0;
  assert.equal(R.fire(m, { angle: 45, power: 50, weapon: 'rocket' }).weapon, 'rocket');
  assert.equal(m.players[0].ammo.rocket, 4);
  m.turn = 1;
  assert.equal(R.fire(m, { angle: 45, power: 50, weapon: 'mega' }).weapon, 'baby');
});

test('buying only between rounds', () => {
  const m = begin(2);
  assert.equal(R.buy(m, 0, 'rocket'), false);
});

test('a whole match replays identically from the seed and the shots', () => {
  const play = () => {
    const m = R.createMatch({ seed: 99, players: players(3), rounds: 2 });
    const r = R.rng(5);
    const prints = [];
    while (m.phase !== 'over') {
      if (m.phase === 'shop') { m.players.forEach((p, i) => R.buy(m, i, 'rocket')); R.startRound(m); }
      R.fire(m, { angle: Math.floor(r() * 181), power: Math.floor(r() * 101), weapon: r() < 0.5 ? 'rocket' : 'big' });
      prints.push(R.fingerprint(m));
      if (prints.length > 2000) throw new Error('match never ended');
    }
    return { prints, winner: m.winner, scores: m.players.map(p => p.score) };
  };
  assert.deepEqual(play(), play());
});

test('the match ends after the last round with the top score winning', () => {
  const m = begin(2, 1, 1);
  m.field.units[1 - m.turn].hp = 1;
  m.field.units[1 - m.turn].x = m.field.units[m.turn].x + 40;
  m.field.units[1 - m.turn].alive = false;
  R.skip(m);
  assert.equal(m.phase, 'over');
  assert.equal(m.players[m.winner].score, Math.max(...m.players.map(p => p.score)));
});

// ---- Online rooms (src/games/rockoon-wars/room.mjs) ----
import * as Room from '../src/games/rockoon-wars/room.mjs';

const ids = ['aaaaaaaaaaaa', 'bbbbbbbbbbbb', 'cccccccccccc'];
const host = { seed: 1234, rounds: 2, order: ids.slice(0, 2), names: { [ids[0]]: 'GS-AA1', [ids[1]]: 'GS-BB2' } };
const snap = (cs, extra = {}) => ({ cs, joined: 1, turns: {}, shops: {}, ...extra });

test('room codes are five unambiguous characters', () => {
  for (let i = 0; i < 50; i++) assert.match(Room.newRoomCode(), Room.ROOM_CODE);
  assert.doesNotMatch('ABCD0', Room.ROOM_CODE);
});

test('snapshots from the relay are checked and trimmed', () => {
  assert.equal(Room.cleanSnapshot({ cs: 'Kyle', joined: 1 }), null);
  const s = Room.cleanSnapshot({ cs: 'GS-AA1', joined: 5, turns: { 0: { angle: 45, power: 50, weapon: 'rocket' }, 1: { angle: 400, power: 50, weapon: 'rocket' }, 2: { angle: 1, power: 1, weapon: 'laser' } }, shops: { 1: ['rocket', 'nope'] }, chat: 'hi' });
  assert.deepEqual(s.turns, { 0: { angle: 45, power: 50, weapon: 'rocket' } });
  assert.deepEqual(s.shops, { 1: ['rocket'] });
  assert.equal(s.chat, undefined);
  const h = Room.cleanSnapshot({ cs: 'GS-AA1', joined: 5, host: { seed: 1, rounds: 3, order: [ids[0], ids[0]], names: {} } });
  assert.equal(h.host, undefined, 'a host with duplicate players is ignored');
});

test('the match waits for every shop, then for the player whose turn it is', () => {
  const s = { [ids[0]]: snap('GS-AA1'), [ids[1]]: snap('GS-BB2') };
  let { match, action } = Room.replay(host, s);
  assert.deepEqual(action, { kind: 'wait', ids: ids.slice(0, 2) });
  void match;
  s[ids[0]].shops[1] = ['rocket'];
  s[ids[1]].shops[1] = [];
  ({ match, action } = Room.replay(host, s));
  assert.equal(match.phase, 'play');
  assert.equal(match.players[0].ammo.rocket, 5);
  assert.deepEqual(action, { kind: 'wait', ids: [ids[match.turn]] });
});

test('two clients replaying the same snapshots agree shot for shot', () => {
  const s = { [ids[0]]: snap('GS-AA1', { shops: { 1: [], 2: [] } }), [ids[1]]: snap('GS-BB2', { shops: { 1: [], 2: [] } }) };
  const r = R.rng(3);
  for (let k = 0; k < 400; k++) {
    const { match, action } = Room.replay(host, s);
    if (action.kind !== 'wait') break;
    const owner = host.order[match.turn];
    s[owner].turns[match.log.length] = { angle: Math.floor(r() * 181), power: 30 + Math.floor(r() * 60), weapon: 'baby' };
  }
  const a = Room.replay(host, JSON.parse(JSON.stringify(s))), b = Room.replay(host, JSON.parse(JSON.stringify(s)));
  assert.equal(R.fingerprint(a.match), R.fingerprint(b.match));
  assert.ok(a.match.log.length > 5);
});

test('only the host can skip a player who left', () => {
  const s = { [ids[0]]: snap('GS-AA1', { shops: { 1: [] } }), [ids[1]]: snap('GS-BB2', { shops: { 1: [] } }) };
  let { match } = Room.replay(host, s);
  const k = match.log.length;
  // A non-host "skip" is not part of its snapshot after cleaning, and does nothing.
  s[ids[1]].skips = [k];
  assert.equal(Room.replay(host, s).match.log.length, k);
  s[ids[0]].skips = [k];
  ({ match } = Room.replay(host, s));
  assert.equal(match.log.length, k + 1);
  assert.equal(match.log[k].skip, true);
});
