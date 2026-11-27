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
  const before = m.players[shooter].credits;
  const { events } = R.fire(m, { angle: 90, power: 30, weapon: 'baby' });
  assert.ok(events.some(e => e.type === 'kill' && e.who === target));
  assert.ok(events.some(e => e.type === 'round' && e.winner === shooter) || !m.field.units[shooter].alive);
  assert.ok(m.players[shooter].credits > before || !m.field.units[shooter].alive);
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
  assert.equal(m.players[0].credits, R.START_CREDITS - 150);
  assert.equal(R.buy(m, 1, 'mega'), false, 'not enough credits');
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

// The host's side of a room: commit whatever is ready, then replay the committed history.
function settle(s, gone = []) {
  for (let guard = 0; guard < 5000; guard++) {
    const { match } = Room.replay(host, s);
    if (!Room.commit(match, host, s, gone)) return Room.replay(host, s);
  }
  throw new Error('did not settle');
}

test('the match waits for every shop, then for the player whose turn it is', () => {
  const s = { [ids[0]]: snap('GS-AA1'), [ids[1]]: snap('GS-BB2') };
  let { match, action } = settle(s);
  assert.deepEqual(action, { kind: 'wait', ids: ids.slice(0, 2) });
  void match;
  s[ids[0]].shops[1] = ['rocket'];
  s[ids[1]].shops[1] = [];
  // Every shop is in, but nothing happens until the host closes the shop.
  ({ match, action } = Room.replay(host, s));
  assert.equal(match.phase, 'shop');
  assert.deepEqual(action, { kind: 'wait', ids: [], sent: true });
  ({ match, action } = settle(s));
  assert.equal(match.phase, 'play');
  assert.equal(match.players[0].ammo.rocket, 5);
  assert.deepEqual(action, { kind: 'wait', ids: [ids[match.turn]] });
});

test('two clients replaying the same snapshots agree shot for shot', () => {
  const s = { [ids[0]]: snap('GS-AA1', { shops: { 1: [], 2: [] } }), [ids[1]]: snap('GS-BB2', { shops: { 1: [], 2: [] } }) };
  const r = R.rng(3);
  for (let k = 0; k < 400; k++) {
    const { match, action } = settle(s);
    if (action.kind !== 'wait') break;
    const owner = host.order[match.turn];
    s[owner].turns[match.log.length] = { angle: Math.floor(r() * 181), power: 30 + Math.floor(r() * 60), weapon: 'baby' };
  }
  const a = Room.replay(host, JSON.parse(JSON.stringify(s))), b = Room.replay(host, JSON.parse(JSON.stringify(s)));
  assert.equal(R.fingerprint(a.match), R.fingerprint(b.match));
  assert.ok(a.match.log.length > 5);
});

test('only the host decides a turn, and a player\'s shot waits for it', () => {
  const s = { [ids[0]]: snap('GS-AA1', { shops: { 1: [] } }), [ids[1]]: snap('GS-BB2', { shops: { 1: [] } }) };
  let { match } = settle(s);
  const k = match.log.length;
  // A non-host log entry is dropped when the snapshot is cleaned, and does nothing.
  const fake = Room.cleanSnapshot({ ...s[ids[1]], log: { [k]: 'skip' } });
  assert.equal(fake.log, undefined);
  s[ids[1]].log = { [k]: 'skip' };
  assert.equal(Room.replay(host, s).match.log.length, k);
  // A shot from the current player is only a request until the host commits it.
  const owner = host.order[match.turn];
  s[owner].turns[k] = { angle: 60, power: 50, weapon: 'baby' };
  assert.deepEqual(Room.replay(host, s).action, { kind: 'wait', ids: [owner], sent: true });
  ({ match } = settle(s));
  assert.equal(match.log.length, k + 1);
});

test('a skip and a late shot give every screen the same match, in either order', () => {
  const base = () => ({ [ids[0]]: snap('GS-AA1', { shops: { 1: [] } }), [ids[1]]: snap('GS-BB2', { shops: { 1: [] } }) });
  // Find a turn that belongs to the non-host player.
  const s0 = base();
  let { match } = settle(s0);
  for (let i = 0; host.order[match.turn] !== ids[1] && i < 4; i++) {
    s0[host.order[match.turn]].turns[match.log.length] = { angle: 90, power: 10, weapon: 'baby' };
    ({ match } = settle(s0));
  }
  assert.equal(host.order[match.turn], ids[1]);
  const k = match.log.length;
  const late = { angle: 40, power: 80, weapon: 'baby' };

  // Order 1: the host gives up on the player first, then the shot arrives.
  const a = JSON.parse(JSON.stringify(s0));
  settle(a, [ids[1]]);
  a[ids[1]].turns[k] = late;
  const hostView = settle(a).match;

  // Order 2: a viewer receives the late shot before the host's skip, then the host's snapshot.
  const viewer = JSON.parse(JSON.stringify(s0));
  viewer[ids[1]].turns[k] = late;
  const early = Room.replay(host, viewer).match;
  assert.equal(early.log.length, k, 'the viewer does not apply a shot the host has not committed');
  viewer[ids[0]] = JSON.parse(JSON.stringify(a[ids[0]]));
  const viewerView = Room.replay(host, viewer).match;

  assert.equal(hostView.log[k].skip, true);
  assert.equal(R.fingerprint(viewerView), R.fingerprint(hostView));
  // A reconnecting client rebuilds the same match from the host's history alone, after the
  // snapshots pass through the same cleaning as anything from the relay.
  a[ids[0]].host = host;
  a[ids[0]].creator = true;
  const rejoin = Room.replay(host, Object.fromEntries(Object.entries(a).map(([id, v]) => [id, Room.cleanSnapshot(JSON.parse(JSON.stringify(v)))])));
  assert.equal(R.fingerprint(rejoin.match), R.fingerprint(hostView));
});

test('committed history never changes once written', () => {
  const s = { [ids[0]]: snap('GS-AA1', { shops: { 1: [] } }), [ids[1]]: snap('GS-BB2', { shops: { 1: [] } }) };
  const { match } = settle(s);
  const k = match.log.length, owner = host.order[match.turn];
  s[owner].turns[k] = { angle: 30, power: 40, weapon: 'baby' };
  settle(s);
  const first = JSON.stringify(s[ids[0]].log[k]);
  s[owner].turns[k] = { angle: 170, power: 99, weapon: 'baby' };
  const again = Room.replay(host, s).match;
  assert.equal(Room.commit(again, host, s), false);
  assert.equal(JSON.stringify(s[ids[0]].log[k]), first);
});

test('everyone starts with credits for a couple of better rockets', () => {
  const m = R.createMatch({ seed: 2, players: players(4) });
  for (const p of m.players) assert.equal(p.credits, R.START_CREDITS);
  assert.ok(R.START_CREDITS >= R.WEAPON.rocket.price + R.WEAPON.big.price);
  assert.ok(R.buy(m, 0, 'rocket') && R.buy(m, 0, 'big'));
  assert.equal(m.players[0].credits, R.START_CREDITS - 550);
  assert.equal(Object.hasOwn(m.players[0], 'money'), false);
});

test('the aim line is the start of the real shot', () => {
  const m = begin(2, 21);
  const u = m.field.units[m.turn];
  u.angle = 70; u.power = 60;
  const preview = R.aimPreview(m, 25);
  assert.equal(preview.length, 25);
  assert.deepEqual(preview, R.trajectory(m, u, 70, 60).path.slice(0, 25));
});

test('drag to aim: point at where you want to shoot, farther means more power', () => {
  const u = { x: 100, y: 50 };
  const py = 50 + R.PIVOT_Y;
  assert.deepEqual(R.aimFromPoint(u, 100 + R.DRAG_FULL, py), { angle: 0, power: 100 });
  assert.deepEqual(R.aimFromPoint(u, 100, py + 80), { angle: 90, power: 50 });
  assert.deepEqual(R.aimFromPoint(u, 100 - 400, py), { angle: 180, power: 100 }, 'power is capped');
  assert.equal(R.aimFromPoint(u, 140, py - 30).angle, 0, 'pointing below the launcher aims flat');
  assert.equal(R.aimFromPoint(u, 60, py - 30).angle, 180);
  const a = R.aimFromPoint(u, 160, py + 60);
  assert.ok(Number.isInteger(a.angle) && Number.isInteger(a.power) && a.angle === 45);
});

test('a Dirt Ball never buries a rockoon', () => {
  for (let seed = 1; seed <= 40; seed++) {
    const m = R.createMatch({ seed, players: players(2) });
    m.players[m.turn].ammo.dirt = 1;
    R.startRound(m);
    const shooter = m.turn, target = 1 - shooter;
    const t = m.field.units[target];
    // Pile dirt straight onto the target, as an aimed shot would.
    R.carve(m.field.terrain, t.x, t.y + 8, R.WEAPON.dirt.radius, true);
    R.settle(m, shooter);
    const u = m.field.units[target];
    let ground = 0;
    for (let dx = -6; dx <= 6; dx++) ground = Math.max(ground, m.field.terrain[Math.max(0, Math.min(R.W - 1, u.x + dx))]);
    assert.ok(u.y >= ground || !u.alive, `seed ${seed}: rockoon at ${u.y} under ground ${ground}`);
  }
});

test('a tied top score is a shared win', () => {
  const m = R.createMatch({ seed: 4, players: players(3), rounds: 1 });
  R.startRound(m);
  // Player 2 will win the round (+2 points) and tie player 1.
  m.players[0].score = 4; m.players[1].score = 2; m.players[2].score = 0;
  m.field.units.forEach((u, i) => { if (i !== 1) u.alive = false; });
  R.skip(m);
  assert.equal(m.phase, 'over');
  assert.deepEqual(m.winners, [0, 1]);
  assert.equal(m.winner, 0);
});
