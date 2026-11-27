// Rules for /games/rockoon-wars/ (src/games/rockoon-wars/logic.mjs).
import test from 'node:test';
import assert from 'node:assert/strict';
import * as R from '../src/games/rockoon-wars/logic.mjs';

const under = (terrain, x) => Math.max(...Array.from({ length: 13 }, (_, i) => terrain[Math.max(0, Math.min(R.W - 1, x + i - 6))]));
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

test('rockoons start apart, on the ground', () => {
  const m = begin(4);
  const xs = m.field.units.map(u => u.x).sort((a, b) => a - b);
  for (let i = 1; i < xs.length; i++) assert.ok(xs[i] - xs[i - 1] > 60);
  for (const u of m.field.units) { assert.equal(u.y, m.field.terrain[u.x]); }
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
  // Where each shot was last seen, whether it landed or flew off the side.
  m.field.wind = 10; const [rx] = R.trajectory(m, u, 90, 40).path.at(-1);
  m.field.wind = -10; const [lx] = R.trajectory(m, u, 90, 40).path.at(-1);
  assert.ok(rx > u.x + 5 && lx < u.x - 5);
});

test('more power goes farther', () => {
  const m = begin(2, 5);
  m.field.wind = 0;
  m.field.terrain.fill(50);
  const u = m.field.units[m.turn]; u.y = 50;
  u.x = 100;
  const near = R.trajectory(m, u, 45, 30).end, far = R.trajectory(m, u, 45, 50).end;
  assert.ok(Math.abs(far.x - u.x) > Math.abs(near.x - u.x));
});

test('rocket releases above its pad and scores against the ring only', () => {
  for (const [offset, points] of [[0, 3], [20, 1], [50, 0]]) {
    const m = begin();
    m.field.wind = 0;
    const [pad, other] = m.field.units;
    const otherPad = { ...other };
    const ground = [...m.field.terrain];
    const release = pad.y + R.LIFT + R.PIVOT_Y + 14;
    m.field.target = { x: pad.x + offset, y: release + 20 };
    const result = R.fire(m, { angle: 90, power: 50, weapon: 'baby' });
    assert.ok(Math.abs(result.path[0][1] - release) < 1e-8);
    assert.equal(m.players[0].score, points);
    assert.equal(m.players[0].credits, R.START_CREDITS + 50 * points);
    assert.equal(m.players[1].score, 0);
    assert.ok(result.blast, 'a landing makes a crater');
    const centre = Math.floor(result.blast.x);
    assert.ok(m.field.terrain[centre] < ground[centre]);
    assert.notEqual(pad.x, centre, 'the nearby pad slides away from the impact');
    assert.equal(pad.y, under(m.field.terrain, pad.x), 'the pad sits on the highest ground under it');
    assert.deepEqual(other, otherPad, 'the distant pad stays put');
  }
});

test('a rocket that leaves the field does not change terrain or pad positions', () => {
  const m = begin();
  const { terrain, units } = m.field;
  m.field.wind = 0;
  const pad = units[m.turn];
  pad.x = 12;
  pad.y = terrain[pad.x];
  const ground = [...terrain];
  const pads = units.map(({ x, y }) => ({ x, y }));
  assert.equal(R.fire(m, { angle: 180, power: 100, weapon: 'baby' }).blast, null);
  assert.deepEqual(terrain, ground);
  assert.deepEqual(units.map(({ x, y }) => ({ x, y })), pads);
});

test('ammo is spent, and an empty weapon uses the free Scout', () => {
  const m = R.createMatch({ seed: 4, players: players(2) });
  assert.ok(R.buy(m, 0, 'rocket'));
  assert.equal(m.players[0].ammo.rocket, 5);
  assert.equal(m.players[0].credits, R.START_CREDITS - 150);
  m.players[1].credits = R.WEAPON.mega.price - 1;
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
  for (let i = 0; i < 2 * R.LAUNCHES; i++) R.skip(m);
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
  // A shot from the current player is only a request until the host commits it.
  const owner = host.order[match.turn];
  s[owner].turns[k] = { angle: 60, power: 50, weapon: 'baby' };
  assert.deepEqual(Room.replay(host, s).action, { kind: 'wait', ids: [owner], sent: true });
  ({ match } = settle(s));
  assert.equal(match.log.length, k + 1);
});

test('the host\'s copy of an entry beats a later player\'s, and a screen that played the other replays', () => {
  const s = { [ids[0]]: snap('GS-AA1', { shops: { 1: [] } }), [ids[1]]: snap('GS-BB2', { shops: { 1: [] } }) };
  let { match } = settle(s);
  const k = match.log.length;
  // The second player commits a skip (it thought the host had gone), and a screen plays it.
  s[ids[1]].log = { ...structuredClone(s[ids[0]].log), [k]: 'skip' };
  s[ids[1]].shopLog = structuredClone(s[ids[0]].shopLog);
  const screen = Room.replay(host, s).match;
  assert.equal(screen.log.length, k + 1);
  assert.equal(Room.diverged(screen, host, s), false);
  // The host was only slow: its shot for turn k arrives, and it wins.
  s[ids[0]].log[k] = { angle: 60, power: 50, weapon: 'baby' };
  assert.equal(Room.diverged(screen, host, s), true, 'the screen must replay');
  const fixed = Room.replay(host, s).match;
  assert.equal(fixed.log[k].skip, undefined);
  assert.equal(Room.diverged(fixed, host, s), false);
});

test('when the host leaves, the next player commits and the match keeps going', () => {
  const s = { [ids[0]]: snap('GS-AA1', { shops: { 1: [], 2: [] } }), [ids[1]]: snap('GS-BB2', { shops: { 1: [], 2: [] } }) };
  let { match } = settle(s);
  // Both players keep a copy of the history; then the host's snapshot disappears.
  Room.mirror(host, s, ids[1]);
  delete s[ids[0]];
  const gone = [ids[0]];
  assert.equal(Room.committerFor(host, gone), ids[1]);
  assert.equal(Room.committerFor(host, []), ids[0]);
  for (let i = 0; i < 40 && match.phase !== 'over'; i++) {
    const owner = host.order[match.turn];
    if (match.phase === 'play' && owner === ids[1]) s[ids[1]].turns[match.log.length] = { angle: 70, power: 60, weapon: 'baby' };
    Room.commit(match, host, s, gone, ids[1]);
    match = Room.replay(host, s).match;
  }
  assert.equal(match.phase, 'over', 'the host\'s turns are skipped and the match finishes');
  // A player who joins now rebuilds the same match from the second player's copy alone.
  const late = Object.fromEntries(Object.entries(s).map(([id, v]) => [id, Room.cleanSnapshot(JSON.parse(JSON.stringify(v)))]));
  assert.ok(late[ids[1]].host, 'the copy carries the match settings');
  assert.equal(R.fingerprint(Room.replay(late[ids[1]].host, late).match), R.fingerprint(match));
});

test('mirroring copies only what is committed, and only into a player in the match', () => {
  const s = { [ids[0]]: snap('GS-AA1', { shops: { 1: [] } }), [ids[1]]: snap('GS-BB2', { shops: { 1: [] } }), [ids[2]]: snap('GS-CC3') };
  settle(s);
  assert.equal(Room.mirror(host, s, ids[2]), false, 'a watcher keeps no copy');
  assert.equal(Room.mirror(host, s, ids[1]), true);
  assert.deepEqual(s[ids[1]].log, s[ids[0]].log);
  assert.deepEqual(s[ids[1]].shopLog, s[ids[0]].shopLog);
  assert.equal(Room.mirror(host, s, ids[1]), false, 'nothing new the second time');
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
  const cost = R.WEAPON.rocket.price + R.WEAPON.big.price;
  assert.ok(R.START_CREDITS >= cost);
  assert.ok(R.buy(m, 0, 'rocket') && R.buy(m, 0, 'big'));
  assert.equal(m.players[0].credits, R.START_CREDITS - cost);
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
  const py = 50 + R.LIFT + R.PIVOT_Y; // the rocket leaves from the release point, no wind here
  assert.deepEqual(R.aimFromPoint(u, 100 + R.DRAG_FULL, py), { angle: 0, power: 100 });
  assert.deepEqual(R.aimFromPoint(u, 100, py + 80), { angle: 90, power: 50 });
  assert.deepEqual(R.aimFromPoint(u, 100 - 400, py), { angle: 180, power: 100 }, 'power is capped');
  assert.equal(R.aimFromPoint(u, 140, py - 30).angle, 0, 'pointing below the launcher aims flat');
  assert.equal(R.aimFromPoint(u, 60, py - 30).angle, 180);
  const a = R.aimFromPoint(u, 160, py + 60);
  assert.ok(Number.isInteger(a.angle) && Number.isInteger(a.power) && a.angle === 45);
});

test('a tied top score is a shared win', () => {
  const m = R.createMatch({ seed: 4, players: players(3), rounds: 1 });
  R.startRound(m);
  m.players[0].score = 4; m.players[1].score = 4;
  for (let i = 0; i < 3 * R.LAUNCHES; i++) {
    assert.equal(m.turn, i % 3);
    R.skip(m);
  }
  assert.equal(m.phase, 'over');
  assert.deepEqual(m.winners, [0, 1]);
  assert.equal(m.winner, 0);
});

test('barometer agrees with standard atmosphere reference pressures', () => {
  for (const [height, hPa] of [[0, 1013.25], [1000, 898.75], [3000, 701.09]]) {
    assert.ok(Math.abs(R.pressureHpa(height) - hPa) < 0.1);
    assert.ok(R.pressureHpa(height + R.LIFT * R.METRES_PER_PIXEL) < hPa);
  }
});

test('a windy ascent drifts from the pad to the shared preview and flight origin', () => {
  const m = begin();
  const pad = m.field.units[m.turn];
  for (const wind of [-10, 0, 10]) {
    m.field.wind = wind;
    assert.deepEqual(R.balloonPosition(pad, wind, 0), { x: pad.x, y: pad.y });
    const release = R.balloonPosition(pad, wind, 1);
    assert.equal(Math.sign(release.x - pad.x), Math.sign(wind));
    assert.equal(release.y, pad.y + R.LIFT);
    pad.angle = 90;
    assert.ok(Math.abs(R.aimPreview(m)[0][0] - release.x) < 1e-8);
  }
});

test('the ring is never next to a release point, and every crew can reach it', () => {
  // Review finding: in 3- and 4-player rounds the ring could sit over a crew's release point,
  // so a straight-up launch at power 0 scored 3 points.
  for (const n of [2, 3, 4]) {
    for (let seed = 1; seed <= 60; seed++) {
      const m = R.createMatch({ seed, players: players(n) });
      R.startRound(m);
      const { target, wind, units } = m.field;
      assert.ok(target.y + R.TARGET_RADIUS <= R.H, `seed ${seed}: ring off the top`);
      for (const u of units) {
        const release = R.releasePoint(u, wind);
        const dx = target.x - release.x, dy = target.y - (release.y + R.PIVOT_Y);
        assert.ok(dx * dx + dy * dy >= 32 * 32, `n=${n} seed ${seed}: ring next to a release`);
        let reach = false;
        for (let a = 0; a <= 180 && !reach; a += 2) {
          for (let p = 0; p <= 100 && !reach; p += 2) {
            const { path } = R.trajectory(m, u, a, p);
            const hit = path.some(([x, y]) => (x - target.x) * (x - target.x) + (y - target.y) * (y - target.y) <= R.TARGET_RADIUS * R.TARGET_RADIUS);
            if (hit && p <= 10) assert.fail(`n=${n} seed ${seed}: scores 3 at power ${p}`);
            reach ||= hit;
          }
        }
        assert.ok(reach, `n=${n} seed ${seed}: a crew can't reach the ring`);
      }
    }
  }
});

test('drag to aim points from the release point, where the dots start', () => {
  const u = { x: 100, y: 50 };
  for (const wind of [-10, 0, 10]) {
    const r = R.releasePoint(u, wind);
    assert.deepEqual(R.aimFromPoint(u, r.x, r.y + R.PIVOT_Y + 80, wind), { angle: 90, power: 50 });
    assert.deepEqual(R.aimFromPoint(u, r.x + R.DRAG_FULL, r.y + R.PIVOT_Y, wind), { angle: 0, power: 100 });
  }
});

test('the rules use no ** or Math.pow, which browsers may round differently', async () => {
  const { readFile } = await import('node:fs/promises');
  const src = await readFile(new URL('../src/games/rockoon-wars/logic.mjs', import.meta.url), 'utf8');
  const rules = src.replace(/export function pressureHpa[\s\S]*?\n}\n/, '');
  assert.ok(!/\*\*|Math\.pow/.test(rules.replace(/\/\*\*[\s\S]*?\*\//g, '').replace(/\/\/.*$/gm, '')));
});

test('a wider scan always costs more per rocket', () => {
  // Review finding: the Mapper (34 px, 3 for 200) beat the Surveyor (38 px, 2 for 400).
  const paid = R.WEAPONS.filter(w => w.price);
  for (let i = 1; i < paid.length; i++) {
    assert.ok(paid[i].radius > paid[i - 1].radius, `${paid[i].name} scans wider`);
    assert.ok(paid[i].price / paid[i].pack > paid[i - 1].price / paid[i - 1].pack, `${paid[i].name} costs more each`);
  }
  assert.ok(R.WEAPONS[0].price === 0 && R.WEAPONS[0].radius < paid[0].radius);
});

test('over n rounds every crew launches once from every slot', () => {
  // Review finding: crews in the middle slots usually started nearest the ring.
  for (const n of [2, 3, 4]) {
    for (const seed of [1, 9, 77]) {
      const m = R.createMatch({ seed, players: players(n), rounds: n });
      const slots = Array.from({ length: n }, () => new Set());
      for (let round = 1; round <= n; round++) {
        R.startRound(m);
        const byX = m.field.units.map((u, i) => [u.x, i]).sort((a, b) => a[0] - b[0]);
        byX.forEach(([, i], k) => slots[i].add(k));
        while (m.phase === 'play') R.skip(m);
      }
      for (const s of slots) assert.equal(s.size, n, `n=${n} seed ${seed}: a crew missed a slot`);
    }
  }
});

test('with two crews the ring is about as far from each', () => {
  for (let seed = 1; seed <= 40; seed++) {
    const m = R.createMatch({ seed, players: players(2) });
    R.startRound(m);
    const [a, b] = m.field.units.map(u => Math.abs(m.field.target.x - R.releasePoint(u, m.field.wind).x));
    assert.ok(Math.abs(a - b) <= 16, `seed ${seed}: ${a} vs ${b}`);
  }
});

test('the controls show the Scout once the picked rocket has run out', () => {
  // Review finding: after the last Observatory the controls still named it, then launched a Scout.
  const m = R.createMatch({ seed: 4, players: players(2) });
  assert.ok(R.buy(m, 0, 'mega'));
  R.startRound(m);
  m.turn = 0;
  m.field.units[0].weapon = 'mega';
  assert.equal(R.readyWeapon(m, 0), 'mega');
  assert.equal(R.fire(m, { angle: 45, power: 50, weapon: 'mega' }).weapon, 'mega');
  assert.equal(m.players[0].ammo.mega, 0);
  assert.equal(m.field.units[0].weapon, 'mega', 'the last pick is remembered');
  assert.equal(R.readyWeapon(m, 0), 'baby', 'but the Scout is what will launch');
  m.field.units[1].weapon = 'nonsense';
  assert.equal(R.readyWeapon(m, 1), 'baby');
});

test('each rocket cuts a different profile and the next balloon uses the pad it moved', () => {
  const profiles = new Map();
  for (const rocket of R.WEAPONS) {
    const m = begin();
    const crew = m.turn;
    const pad = m.field.units[crew];
    const before = pad.x;
    m.field.wind = 0;
    m.field.terrain.fill(100);
    for (const unit of m.field.units) unit.y = 100;
    if (rocket.price) m.players[crew].ammo[rocket.id] = 1;
    const result = R.fire(m, { angle: 90, power: 25, weapon: rocket.id });
    assert.ok(result.blast, rocket.name);
    const { x, cutDepth, moved } = result.blast;
    const width = m.field.terrain.filter(height => height < 100).length;
    profiles.set(rocket.id, { width, cutDepth });
    assert.equal(m.field.terrain[Math.floor(x)], 100 - rocket.impact.depth);
    assert.equal(cutDepth, rocket.impact.depth);
    assert.deepEqual(moved, [{ crew, dx: pad.x - before, dy: pad.y - 100 }]);
    assert.notEqual(pad.x, before);
    assert.equal(pad.y, under(m.field.terrain, pad.x));
    R.skip(m);
    assert.equal(m.turn, crew);
    assert.deepEqual(R.balloonPosition(pad, m.field.wind, 0), { x: pad.x, y: pad.y });
    const next = R.trajectory(m, m.field.units[crew], 90, 25);
    assert.ok(Math.abs(next.path[0][0] - pad.x) < 1e-8);
  }
  assert.equal(new Set([...profiles.values()].map(p => JSON.stringify(p))).size, R.WEAPONS.length);
  assert.ok(profiles.get('rocket').cutDepth > profiles.get('dirt').cutDepth);
  assert.ok(profiles.get('dirt').width > profiles.get('rocket').width);
});

test('an impact on exposed bedrock leaves the ground and pads unchanged', () => {
  const m = begin();
  m.field.wind = 0;
  m.field.terrain.fill(0);
  const before = m.field.units.map(unit => ({ x: unit.x, y: 0 }));
  for (const unit of m.field.units) unit.y = 0;
  const { blast } = R.fire(m, { angle: 90, power: 25, weapon: 'baby' });
  assert.equal(blast.cutDepth, 0);
  assert.deepEqual(blast.moved, []);
  assert.ok(m.field.terrain.every(height => height === 0));
  assert.deepEqual(m.field.units.map(({ x, y }) => ({ x, y })), before);
});

test('a pad moved by a crater never hangs over the edge', () => {
  for (let seed = 1; seed <= 30; seed++) {
    const m = R.createMatch({ seed, players: players(2) });
    R.startRound(m);
    m.field.wind = 0;
    for (let k = 0; k < 4 && m.phase === 'play'; k++) {
      R.fire(m, { angle: 60 + 20 * k, power: 18 + 5 * k, weapon: 'baby' });
      for (const pad of m.field.units) assert.equal(pad.y, under(m.field.terrain, pad.x), `seed ${seed}`);
    }
  }
});

test('the rising flag is display only and checked like everything else', () => {
  const base = { cs: 'GS-AA1', joined: 1, turns: {}, shops: {} };
  assert.equal(Room.cleanSnapshot({ ...base, rising: 4 }).rising, 4);
  for (const bad of [-1, 1e6, 'x', 2.5, null]) assert.equal(Room.cleanSnapshot({ ...base, rising: bad }).rising, undefined, String(bad));
});
