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
  for (const [offset, points] of [[0, 10], [6, 6], [11, 4], [20, 2], [27, 1], [50, 0]]) {
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
    assert.equal(m.players[0].bullseyes, points === 10 ? 1 : 0);
    assert.equal(result.hit.bullseye, points === 10);
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

test('target drift follows the same flight clock used for scoring and advances between shots', () => {
  const m = begin();
  m.field.wind = 7;
  const field = structuredClone(m.field);
  const positions = Array.from({ length: 5 }, (_, i) => R.targetPosition(field, i));
  assert.notDeepEqual(positions[0], positions[1]);
  assert.ok(positions.every(({ x, y }) => x > R.TARGET_RADIUS && x < R.W - R.TARGET_RADIUS && y < R.H - R.TARGET_RADIUS));
  const path = Array.from({ length: 61 }, (_, i) => {
    const target = R.targetPosition(field, i / R.FLIGHT_SAMPLES_PER_SECOND);
    return [target.x + 10 * (i - 40.5), target.y];
  });
  const hit = R.targetHit(field, path, 0);
  assert.equal(hit.at, 40.5);
  assert.equal(hit.bullseye, true, 'detects a bullseye between samples');
  const shot = { angle: 90, power: 25, weapon: 'baby' };
  const next = structuredClone(m);
  assert.deepEqual(R.fire(next, shot), R.fire(m, shot));
  assert.equal(R.fingerprint(next), R.fingerprint(m));
  assert.notDeepEqual(R.targetPosition(m.field), positions[0], 'the next shot starts from the target’s new position');
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
const snap = (cs, extra = {}) => ({ p: Room.ROOM_PROTOCOL, cs, joined: 1, turns: {}, shops: {}, ...extra });

test('snapshots from another room protocol are ignored', () => {
  assert.equal(Room.ROOM_NAMESPACE, `rockoon-battle-v${Room.ROOM_PROTOCOL}`);
  const ok = { cs: 'GS-AA1', joined: 5 };
  assert.equal(Room.cleanSnapshot(ok), null, 'an older client sends no protocol number');
  assert.equal(Room.cleanSnapshot({ ...ok, p: Room.ROOM_PROTOCOL - 1 }), null);
  assert.equal(Room.cleanSnapshot({ ...ok, p: Room.ROOM_PROTOCOL }).p, Room.ROOM_PROTOCOL);
});

test('room codes are five unambiguous characters', () => {
  for (let i = 0; i < 50; i++) assert.match(Room.newRoomCode(), Room.ROOM_CODE);
  assert.doesNotMatch('ABCD0', Room.ROOM_CODE);
});

test('snapshots from the relay are checked and trimmed', () => {
  assert.equal(Room.cleanSnapshot({ cs: 'Kyle', joined: 1 }), null);
  const s = Room.cleanSnapshot({ p: Room.ROOM_PROTOCOL, cs: 'GS-AA1', joined: 5, turns: { 0: { angle: 45, power: 50, weapon: 'rocket' }, 1: { angle: 400, power: 50, weapon: 'rocket' }, 2: { angle: 1, power: 1, weapon: 'laser' } }, shops: { 1: ['rocket', 'nope'] }, chat: 'hi' });
  assert.deepEqual(s.turns, { 0: { angle: 45, power: 50, weapon: 'rocket' } });
  assert.deepEqual(s.shops, { 1: ['rocket'] });
  assert.equal(s.chat, undefined);
  const h = Room.cleanSnapshot({ p: Room.ROOM_PROTOCOL, cs: 'GS-AA1', joined: 5, host: { seed: 1, rounds: 3, order: [ids[0], ids[0]], names: {} } });
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

// The stand-in closed the shop while the creator was away. The returning screen may still
// see a snapshot from before the shop closed.
function shopSettledWhileAway() {
  const [creator, standin] = ids;
  const early = snap('GS-BB2', { host, shops: { 1: ['rocket'] } });
  const settled = structuredClone(early);
  const peers = { [standin]: settled };
  assert.ok(Room.commit(Room.matchFor(host), host, peers, [creator], standin));
  const peerMatch = Room.replay(host, peers, { gone: [creator], me: standin }).match;
  return { creator, standin, early, settled, peerMatch };
}

test('a stand-in can bring a settled shop after its first snapshot and the catch-up window', () => {
  const { creator, standin, early, settled, peerMatch } = shopSettledWhileAway();
  const s = { [creator]: snap('GS-AA1'), [standin]: early };
  const match = Room.matchFor(host);
  const heard = new Map();
  assert.equal(Room.notePeerHistory(heard, standin, early, match), false);
  assert.ok(heard.has(standin), 'a snapshot without history still counts as hearing from the peer');

  s[standin] = settled;
  assert.equal(Room.notePeerHistory(heard, standin, settled, match), true, 'the later shop reopens catch-up');
  assert.equal(Room.notePeerHistory(heard, standin, early, match), false);
  assert.equal(Room.notePeerHistory(heard, standin, settled, match), false, 'heartbeats do not reopen catch-up again');

  // If the result banner was up, the remembered shop still counts after the flag times out.
  const view = { me: creator, heard, rejoined: false };
  s[creator].shops[1] = ['big'];
  assert.equal(Room.commit(match, host, s, view), false);
  const action = Room.nextAction(match, host, s, view);
  assert.deepEqual(action.buys, [[], ['rocket']]);
  Room.apply(match, action);
  Room.mirror(match, host, s, creator);
  assert.equal(s[creator].shops[1], undefined, 'the late picks are dropped');
  assert.equal(R.fingerprint(match), R.fingerprint(peerMatch));
  assert.equal(Room.diverged(peerMatch, host, s, { me: standin }), false);
});

test('a returning creator can submit picks before the stand-in is heard without replacing its shop', () => {
  const { creator, standin, early, settled, peerMatch } = shopSettledWhileAway();
  // After a dropped connection, the creator still has the stand-in's old requests cached.
  const s = { [creator]: snap('GS-AA1', { shops: { 1: ['big'] } }), [standin]: early };
  const match = Room.matchFor(host);
  const heard = new Map();
  const view = { me: creator, heard, rejoined: true };
  assert.equal(Room.commit(match, host, s, view), false, 'old snapshots cannot release the creator to commit');
  assert.equal(s[creator].shopLog?.[1], undefined);
  assert.equal(Room.commit(match, host, s, { ...view, rejoined: false }), false, 'the wait can outlast the catch-up flag');

  s[standin] = settled;
  assert.equal(Room.notePeerHistory(heard, standin, settled, match), true);
  assert.equal(Room.commit(match, host, s, view), false, 'the settled shop takes precedence over the late picks');
  const action = Room.nextAction(match, host, s, view);
  assert.deepEqual(action.buys, [[], ['rocket']]);
  Room.apply(match, action);
  Room.mirror(match, host, s, creator);
  assert.equal(s[creator].standin.S1, standin);
  assert.equal(R.fingerprint(match), R.fingerprint(peerMatch));
  assert.equal(Room.diverged(peerMatch, host, s, { me: standin }), false);
});

test('a new creator commits promptly and a returning creator can proceed once a peer is gone', () => {
  const [creator, other] = ids;
  const s = { [creator]: snap('GS-AA1', { shops: { 1: ['big'] } }), [other]: snap('GS-BB2', { shops: { 1: [] } }) };
  const match = Room.matchFor(host);
  assert.equal(Room.commit(match, host, structuredClone(s), { me: creator }), true);
  const heard = new Map();
  assert.equal(Room.commit(match, host, s, { me: creator, heard }), false);
  assert.equal(Room.commit(match, host, s, { me: creator, heard, gone: [other] }), true);
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

test('a later player\'s skip counts only once everyone before it has gone, and the host wins when it comes back', () => {
  const s = { [ids[0]]: snap('GS-AA1', { shops: { 1: [] } }), [ids[1]]: snap('GS-BB2', { shops: { 1: [] } }) };
  let { match } = settle(s);
  const k = match.log.length, owner = host.order[match.turn];
  // The second player thought the host had gone and committed a skip.
  s[ids[1]].log = { ...structuredClone(s[ids[0]].log), [k]: 'skip' };
  s[ids[1]].shopLog = structuredClone(s[ids[0]].shopLog);
  assert.equal(Room.replay(host, s).match.log.length, k, 'while the host is here, nobody else can skip');
  const screen = Room.replay(host, s, [ids[0]]).match;
  assert.equal(screen.log.length, k + 1, 'with the host gone, its successor\'s skip counts');
  assert.equal(Room.diverged(screen, host, s, [ids[0]]), false);
  // The host was only slow: it is back, and its shot for turn k arrives.
  s[owner].turns[k] = { angle: 60, power: 50, weapon: 'baby' };
  s[ids[0]].log[k] = { angle: 60, power: 50, weapon: 'baby' };
  assert.equal(Room.diverged(screen, host, s, []), true, 'the screen must replay');
  const fixed = Room.replay(host, s).match;
  assert.equal(fixed.log[k].skip, undefined);
  assert.equal(Room.diverged(fixed, host, s), false);
});

test('a returning host keeps the turns played without it', () => {
  // Review: the stand-in recorded a skip of the host's turn and its own shot; the host came back
  // without publishing anything new, and replay dropped both and went back to turn one.
  const s = { [ids[0]]: snap('GS-AA1', { shops: { 1: [], 2: [] } }), [ids[1]]: snap('GS-BB2', { shops: { 1: [], 2: [] } }) };
  let { match } = settle(s);
  // Play until it's the host's turn.
  for (let i = 0; i < 6 && host.order[match.turn] !== ids[0]; i++) {
    s[host.order[match.turn]].turns[match.log.length] = { angle: 80, power: 40, weapon: 'baby' };
    ({ match } = settle(s));
  }
  const k = match.log.length;
  assert.equal(host.order[match.turn], ids[0]);
  // The host goes away. Its stand-in skips the host's turn, then plays its own.
  const away = [ids[0]];
  const bView = { gone: away, me: ids[1] };
  let b = Room.replay(host, s, bView).match;
  Room.mirror(b, host, s, ids[1]);
  assert.ok(Room.commit(b, host, s, away, ids[1]));
  b = Room.replay(host, s, bView).match;
  s[ids[1]].turns[b.log.length] = { angle: 70, power: 50, weapon: 'baby' };
  assert.ok(Room.commit(b, host, s, away, ids[1]));
  b = Room.replay(host, s, bView).match;
  Room.mirror(b, host, s, ids[1]);
  assert.equal(b.log.length, k + 2);
  // The host is back (gone is empty again) and has published nothing new.
  const bBack = { gone: [], me: ids[1] };
  assert.equal(Room.diverged(b, host, s, bBack), false, 'a heartbeat alone never undoes a turn');
  // The host's own screen, just rejoined, accepts the skip of its missed turn and catches up.
  const hostView = { gone: [], me: ids[0], rejoined: true };
  const h = Room.replay(host, s, hostView).match;
  assert.equal(h.log.length, k + 2);
  assert.equal(R.fingerprint(h), R.fingerprint(b));
  // Once the host carries the skip, every other screen believes it, even with nobody gone.
  Room.mirror(h, host, s, ids[0]);
  assert.equal(R.fingerprint(Room.replay(host, s, []).match), R.fingerprint(b));
  // And the host records turns again.
  assert.equal(Room.committerFor(host, []), ids[0]);
});

test('a shot stays replayable after the player who took it leaves (three players)', () => {
  // Review: the host goes offline, the stand-in fires and leaves, the host returns. The third
  // player carries the shot; the host must replay it, not record a skip over it.
  const three = { ...host, order: ids, names: { ...host.names, [ids[2]]: 'GS-CC3' } };
  const s = { [ids[0]]: snap('GS-AA1', { shops: { 1: [], 2: [], 3: [] } }), [ids[1]]: snap('GS-BB2', { shops: { 1: [], 2: [], 3: [] } }), [ids[2]]: snap('GS-CC3', { shops: { 1: [], 2: [], 3: [] } }) };
  let match;
  for (let i = 0; i < 10; i++) { match = Room.replay(three, s).match; if (!Room.commit(match, three, s)) break; }
  // Play until it's the stand-in's (ids[1]) turn.
  for (let i = 0; i < 9 && three.order[match.turn] !== ids[1]; i++) {
    s[three.order[match.turn]].turns[match.log.length] = { angle: 80, power: 40, weapon: 'baby' };
    Room.commit(match, three, s); match = Room.replay(three, s).match;
  }
  const k = match.log.length;
  // The host goes away; the stand-in commits its own shot, and the third player plays and mirrors it.
  const hostAway = [ids[0]];
  s[ids[1]].turns[k] = { angle: 66, power: 55, weapon: 'baby' };
  Room.mirror(match, three, s, ids[1]);
  assert.ok(Room.commit(match, three, s, hostAway, ids[1]));
  const c = Room.replay(three, s, { gone: hostAway, me: ids[2] }).match;
  assert.equal(c.log.length, k + 1);
  Room.mirror(c, three, s, ids[2]);
  // The stand-in leaves for good, and the host returns. It has never seen the stand-in's request.
  delete s[ids[1]];
  const hostView = { gone: [ids[1]], me: ids[0], rejoined: true };
  const h = Room.replay(three, s, hostView).match;
  assert.equal(h.log.length, k + 1, 'the host replays the shot from the third player\'s copy');
  assert.equal(R.fingerprint(h), R.fingerprint(c));
  assert.equal(Room.commit(h, three, s, [ids[1]]), false, 'nothing to record over it');
  assert.equal(Room.diverged(c, three, s, { gone: [ids[1]], me: ids[2] }), false, 'the third player keeps its match');
});

// Review cases for the stand-in rules, three players: A creator, B, C.
function threeAt(turnOwner) {
  const three = { ...host, order: ids, names: { ...host.names, [ids[2]]: 'GS-CC3' } };
  const s = Object.fromEntries(ids.map((id, i) => [id, snap(['GS-AA1', 'GS-BB2', 'GS-CC3'][i], { shops: { 1: [], 2: [], 3: [] } })]));
  let match;
  for (let i = 0; i < 10; i++) { match = Room.replay(three, s).match; if (!Room.commit(match, three, s)) break; }
  for (let i = 0; i < 9 && three.order[match.turn] !== turnOwner; i++) {
    s[three.order[match.turn]].turns[match.log.length] = { angle: 80, power: 40, weapon: 'baby' };
    Room.commit(match, three, s); match = Room.replay(three, s).match;
  }
  for (const id of ids) Room.mirror(match, three, s, id);
  return { three, s, match, k: match.log.length };
}

test('a returning creator keeps a stand-in skip over its own late shot', () => {
  const [creator, standin, other] = ids;
  const { three, s, match, k } = threeAt(creator);
  const away = [creator];
  assert.ok(Room.commit(match, three, s, away, standin));
  const peerMatch = Room.replay(three, s, { gone: away, me: standin }).match;
  const settled = s[standin];
  delete s[standin];

  const heard = new Map();
  const view = { me: creator, heard, rejoined: true };
  Room.notePeerHistory(heard, other, s[other], match);
  s[creator].turns[k] = { angle: 60, power: 50, weapon: 'baby' };
  assert.equal(Room.commit(match, three, s, view), false, 'hearing from another player does not release the creator');
  assert.equal(s[creator].log[k], undefined);

  s[standin] = settled;
  assert.equal(Room.notePeerHistory(heard, standin, settled, match), true);
  assert.equal(Room.commit(match, three, s, view), false);
  const action = Room.nextAction(match, three, s, view);
  assert.equal(action.kind, 'skip');
  Room.apply(match, action);
  Room.mirror(match, three, s, creator);
  assert.equal(s[creator].turns[k], undefined);
  assert.equal(R.fingerprint(match), R.fingerprint(peerMatch));
  assert.equal(Room.diverged(peerMatch, three, s, { me: standin }), false);
});

test('a player catching up does not take a skip of its own turn from someone who did not stand in', () => {
  const { three, s, match, k } = threeAt(ids[1]);
  // C forges skips far ahead; B reloads (no requests, catching up). A is here and current.
  s[ids[2]].log = { ...s[ids[2]].log, ...Object.fromEntries(Array.from({ length: 30 }, (_, j) => [k + j, 'skip'])) };
  s[ids[1]].turns = {};
  assert.deepEqual(Room.nextAction(match, three, s, { gone: [], me: ids[1], rejoined: true }).kind, 'wait');
});

test('a carried shot for a player who has gone is not believed from just anyone', () => {
  const { three, s, match, k } = threeAt(ids[1]);
  s[ids[2]].log = { ...s[ids[2]].log, [k]: { angle: 1, power: 1, weapon: 'baby' } };
  delete s[ids[1]];
  assert.equal(Room.nextAction(match, three, s, { gone: [ids[1]], me: ids[0] }).kind, 'wait');
  assert.ok(Room.commit(match, three, s, [ids[1]]), 'the creator records a skip');
  assert.equal(Room.nextAction(match, three, s, { gone: [ids[1]], me: ids[0] }).kind, 'skip');
});

test('a copy of an entry taken on a stand-in\'s word keeps the stand-in\'s authority', () => {
  // Review: mirror() copied every played entry, so the returning creator republished B's skip of
  // C's turn as its own record, and every screen then believed it on the creator's authority.
  const { three, s, k } = threeAt(ids[2]);
  // A and C are away. B, the stand-in, records a skip of C's turn.
  const bView = { gone: [ids[0], ids[2]], me: ids[1] };
  let b = Room.replay(three, s, bView).match;
  assert.ok(Room.commit(b, three, s, bView.gone, ids[1]));
  b = Room.replay(three, s, bView).match;
  assert.equal(b.log.length, k + 1);
  Room.mirror(b, three, s, ids[1]);
  assert.equal(s[ids[1]].standin, undefined, 'the stand-in\'s own record is not a copy');
  // A comes back, catching up, and plays B's skip on the stand-in rule.
  const aView = { gone: [ids[2]], me: ids[0], rejoined: true };
  let a = Room.replay(three, s, aView).match;
  assert.equal(a.log.length, k + 1);
  assert.deepEqual([a.played[`L${k}`].via, a.played[`L${k}`].by], ['standin', ids[1]]);
  Room.mirror(a, three, s, ids[0]);
  assert.equal(s[ids[0]].log[k], 'skip', 'the creator keeps a replayable copy');
  assert.equal(s[ids[0]].standin[`L${k}`], ids[1], 'marked as the stand-in\'s word');
  // The mark survives the relay, and a mark naming the creator is dropped.
  const wire = Room.cleanSnapshot(JSON.parse(JSON.stringify({ ...s[ids[0]], standin: { ...s[ids[0]].standin, L0: ids[0], L9999: ids[1] } })));
  assert.deepEqual(wire.standin, { [`L${k}`]: ids[1] });
  // A screen that isn't catching up does not take the skip on the creator's word.
  assert.equal(Room.replay(three, s, []).match.log.length, k);
  // A plays on and records its own turn after the skip.
  s[ids[0]].turns[k + 1] = { angle: 70, power: 50, weapon: 'baby' };
  assert.ok(Room.commit(a, three, s, [ids[2]], ids[0]));
  a = Room.replay(three, s, aView).match;
  Room.mirror(a, three, s, ids[0]);
  assert.equal(a.log.length, k + 2);
  assert.equal(s[ids[0]].standin[`L${k + 1}`], undefined, 'the creator\'s own turn is its own record');
  // Review repro: the stand-in leaves for good, then C reloads and catches up. The only copy of
  // B's skip is the creator's, still marked as B's word, so C replays every completed turn.
  delete s[ids[1]];
  s[ids[2]] = snap('GS-CC3', { shops: { 1: [], 2: [], 3: [] } });
  const cView = { gone: [ids[1]], me: ids[2], rejoined: true };
  const c = Room.replay(three, s, cView).match;
  assert.equal(c.log.length, k + 2);
  assert.equal(R.fingerprint(c), R.fingerprint(a));
  // C now carries the skip of its own turn, unmarked, so every screen believes it, on C's word.
  Room.mirror(c, three, s, ids[2]);
  assert.equal(s[ids[2]].log[k], 'skip');
  assert.equal(s[ids[2]].standin, undefined);
  assert.equal(R.fingerprint(Room.replay(three, s, [ids[1]]).match), R.fingerprint(a));
});

test('a watcher replays a stand-in\'s entry from a marked copy after the stand-in leaves', () => {
  // Review: a watcher who joined after B left replayed only up to B's skip of C's turn, because
  // only a player catching up took marked copies, and the watcher never catches up as a player.
  const { three, s, k } = threeAt(ids[2]);
  const bView = { gone: [ids[0], ids[2]], me: ids[1] };
  let b = Room.replay(three, s, bView).match;
  assert.ok(Room.commit(b, three, s, bView.gone, ids[1]));
  b = Room.replay(three, s, bView).match;
  Room.mirror(b, three, s, ids[1]);
  const aView = { gone: [ids[2]], me: ids[0], rejoined: true };
  let a = Room.replay(three, s, aView).match;
  Room.mirror(a, three, s, ids[0]);
  s[ids[0]].turns[k + 1] = { angle: 70, power: 50, weapon: 'baby' };
  assert.ok(Room.commit(a, three, s, [ids[2]], ids[0]));
  a = Room.replay(three, s, aView).match;
  Room.mirror(a, three, s, ids[0]);
  delete s[ids[1]];
  delete s[ids[2]];
  // A fresh watcher, not catching up, with B and C gone: the creator's marked copy is enough.
  const watcher = 'eeeeeeeeeeee';
  const w = Room.replay(three, s, { gone: [ids[1], ids[2]], me: watcher }).match;
  assert.equal(w.log.length, k + 2);
  assert.equal(R.fingerprint(w), R.fingerprint(a));
  // A player screen that is not catching up still doesn't take it on the creator's word.
  assert.equal(Room.replay(three, s, { gone: [ids[1], ids[2]], me: ids[2] }).match.log.length, k);
});

test('the remaining committer can replay a stand-in copy after earlier players leave', () => {
  const [a, b, c] = ids;
  const d = 'dddddddddddd';
  const order = [...ids, d];
  const names = { ...host.names, [c]: 'GS-CC3', [d]: 'GS-DD4' };
  const four = { ...host, seed: 42, order, names };
  const snapshots = Object.fromEntries(order.map(id => [id, snap(names[id], { shops: { 1: [] } })]));
  assert.ok(Room.commit(Room.matchFor(four), four, snapshots));
  let match = Room.replay(four, snapshots).match;
  for (let i = 0; i < order.length && four.order[match.turn] !== c; i++) {
    snapshots[four.order[match.turn]].turns[match.log.length] = { angle: 60, power: 50, weapon: 'baby' };
    assert.ok(Room.commit(match, four, snapshots));
    match = Room.replay(four, snapshots).match;
  }
  assert.equal(four.order[match.turn], c);
  const turn = match.log.length;
  for (const id of order) Room.mirror(match, four, snapshots, id);

  assert.ok(Room.commit(match, four, snapshots, [a, c], b));
  const creator = Room.replay(four, snapshots, { gone: [c], me: a, rejoined: true }).match;
  Room.mirror(creator, four, snapshots, a);
  delete snapshots[b];

  snapshots[d] = snap(names[d], { shops: { 1: [] } });
  const remaining = Room.replay(four, snapshots, { gone: [b, c], me: d, rejoined: true }).match;
  Room.mirror(remaining, four, snapshots, d);
  assert.equal(snapshots[d].standin[`L${turn}`], b);
  delete snapshots[a];

  snapshots[c] = snap(names[c], { shops: { 1: [] } });
  const returning = Room.replay(four, snapshots, { gone: [a, b], me: c, rejoined: true }).match;
  assert.equal(R.fingerprint(returning), R.fingerprint(remaining));
  assert.equal(returning.played[`L${turn}`].by, b);
  assert.equal(Room.replay(four, snapshots, { gone: [a], me: c, rejoined: true }).match.log.length, turn);
  const watcher = 'eeeeeeeeeeee';
  assert.equal(Room.nextAction(match, four, snapshots, { gone: [a, b], me: watcher }).kind, 'wait');
  assert.equal(R.fingerprint(Room.replay(four, snapshots, { gone: [a, b, c], me: watcher }).match), R.fingerprint(remaining));
});

test('v6 rooms (a broadcast per session, stand-in marks) ignore v5 and v4 snapshots', () => {
  assert.equal(Room.ROOM_PROTOCOL, 6);
  assert.equal(Room.cleanSnapshot({ p: 5, cs: 'GS-AA1', joined: 1, turns: {}, shops: {} }), null);
  assert.equal(Room.cleanSnapshot({ p: 4, cs: 'GS-AA1', joined: 1, turns: {}, shops: {} }), null);
});

test('a stand-in mark never makes a forged copy believable', () => {
  const { three, s, match, k } = threeAt(ids[2]);
  const cView = { gone: [], me: ids[2], rejoined: true };
  // C catching up; B forges a skip of C's turn and claims it is a copy of its own word. A is
  // here, current and carries nothing for turn k, so nobody vouches that B stood in.
  s[ids[1]].log = { ...s[ids[1]].log, [k]: 'skip' };
  s[ids[1]].standin = { [`L${k}`]: ids[1] };
  assert.equal(Room.nextAction(match, three, s, cView).kind, 'wait');
  // A mark naming the creator is not a stand-in at all.
  s[ids[1]].standin = { [`L${k}`]: ids[0] };
  assert.equal(Room.nextAction(match, three, s, cView).kind, 'wait');
  // With A gone, B is the committer anyway; but its marked copy still needs someone to vouch.
  s[ids[1]].standin = { [`L${k}`]: ids[1] };
  delete s[ids[0]];
  assert.equal(Room.nextAction(match, three, s, { ...cView, gone: [ids[0]] }).kind, 'skip', 'the stand-in vouches for its own entry');
  // A stand-in's word never beats a live request from another player either.
  const { three: t2, s: s2, match: m2, k: k2 } = threeAt(ids[2]);
  s2[ids[0]].log = { ...s2[ids[0]].log, [k2]: 'skip' };
  s2[ids[0]].standin = { [`L${k2}`]: ids[1] };
  // B reloaded and is catching up; only A's marked copy carries the skip. Without a request from
  // C, B takes it on A's say-so that B stood in. With C asking for a shot, it doesn't.
  const reloaded = { gone: [], me: ids[1], rejoined: true };
  s2[ids[1]] = snap('GS-BB2', { shops: { 1: [], 2: [], 3: [] } });
  assert.equal(Room.nextAction(m2, t2, s2, reloaded).kind, 'skip');
  s2[ids[2]].turns[k2] = { angle: 60, power: 50, weapon: 'baby' };
  assert.notEqual(Room.nextAction(m2, t2, s2, reloaded).kind, 'skip');
});

test('nobody else can close the shop with empty picks for players still choosing', () => {
  const { three, s, match } = threeAt(ids[0]);
  const shop = { ...match, phase: 'shop', round: 1 };
  for (const id of ids) delete s[id].shops[2];
  s[ids[2]].shops[2] = ['rocket'];
  s[ids[2]].shopLog = { ...s[ids[2]].shopLog, 2: [[], [], ['rocket']] };
  assert.equal(Room.nextAction(shop, three, s).kind, 'wait');
});

test('an owner\'s own copy of its turn stays believable after its request is gone (reloads)', () => {
  const { three, s, k } = threeAt(ids[1]);
  // B's shot is committed and everyone mirrors it; then B and A reload (requests and A's log gone).
  s[ids[1]].turns[k] = { angle: 66, power: 55, weapon: 'baby' };
  let m = Room.replay(three, s).match;
  Room.commit(m, three, s); m = Room.replay(three, s).match;
  for (const id of ids) Room.mirror(m, three, s, id);
  s[ids[1]].turns = {};
  s[ids[0]] = snap('GS-AA1');
  const a = Room.replay(three, s, { gone: [], me: ids[0], rejoined: true }).match;
  assert.equal(a.log.length, k + 1);
  assert.equal(R.fingerprint(a), R.fingerprint(m));
});

test('a stand-in never overrides what another player is asking for right now', () => {
  const { three, s, match, k } = threeAt(ids[1]);
  // A reloaded (catching up, no history); C would be the stand-in. B asks for a real shot.
  s[ids[0]] = snap('GS-AA1');
  s[ids[1]].turns[k] = { angle: 60, power: 50, weapon: 'baby' };
  s[ids[2]].log = { ...s[ids[2]].log, [k]: 'skip' };
  const a = Room.nextAction(match, three, s, { gone: [], me: ids[0], rejoined: true });
  assert.notEqual(a.kind, 'skip', 'C\'s skip loses to B\'s live request');
  // But for the screen's own turn, a request it made before seeing the record is just late.
  const own = Room.nextAction(match, three, s, { gone: [], me: ids[1], rejoined: true });
  assert.equal(own.kind, 'skip');
});

test('nobody can commit a shot, a skip or a purchase for someone else', () => {
  const three = { ...host, order: ids, names: { ...host.names, [ids[2]]: 'GS-CC3' } };
  const s = { [ids[0]]: snap('GS-AA1', { shops: { 1: [] } }), [ids[1]]: snap('GS-BB2', { shops: { 1: [] } }), [ids[2]]: snap('GS-CC3', { shops: { 1: [] } }) };
  let match;
  for (let i = 0; i < 10; i++) {
    match = Room.replay(three, s).match;
    if (!Room.commit(match, three, s)) break;
  }
  const k = match.log.length, owner = three.order[match.turn];
  const forger = ids.find(id => id !== owner && id !== ids[0]);
  // A made-up shot, and a skip, for the owner's turn.
  s[forger].log = { ...structuredClone(s[ids[0]].log), [k]: { angle: 1, power: 1, weapon: 'baby' } };
  assert.deepEqual(Room.nextAction(match, three, s).kind, 'wait');
  s[forger].log[k] = 'skip';
  assert.deepEqual(Room.nextAction(match, three, s).kind, 'wait');
  // The real committer still records the owner's own shot.
  s[owner].turns[k] = { angle: 60, power: 50, weapon: 'baby' };
  assert.equal(Room.commit(match, three, s), true);
  assert.deepEqual(Room.nextAction(match, three, s).shot, { angle: 60, power: 50, weapon: 'baby' });
  // A made-up shop result for the next round is ignored too.
  s[forger].shopLog = { 2: [['big'], [], []] };
  const shop = { ...match, phase: 'shop', round: 1 };
  assert.equal(Room.nextAction(shop, three, s).kind, 'wait');
});

test('when the host leaves, the next player commits and the match keeps going', () => {
  const s = { [ids[0]]: snap('GS-AA1', { shops: { 1: [], 2: [] } }), [ids[1]]: snap('GS-BB2', { shops: { 1: [], 2: [] } }) };
  let { match } = settle(s);
  // Both players keep a copy of what they have played; then the host's snapshot disappears.
  Room.mirror(match, host, s, ids[1]);
  delete s[ids[0]];
  const gone = [ids[0]];
  assert.equal(Room.committerFor(host, gone), ids[1]);
  assert.equal(Room.committerFor(host, []), ids[0]);
  for (let i = 0; i < 40 && match.phase !== 'over'; i++) {
    const owner = host.order[match.turn];
    if (match.phase === 'play' && owner === ids[1]) s[ids[1]].turns[match.log.length] = { angle: 70, power: 60, weapon: 'baby' };
    Room.commit(match, host, s, gone, ids[1]);
    match = Room.replay(host, s, gone).match;
    Room.mirror(match, host, s, ids[1]);
  }
  assert.equal(match.phase, 'over', 'the host\'s turns are skipped and the match finishes');
  // A player who joins now rebuilds the same match from the second player's copy alone.
  const late = Object.fromEntries(Object.entries(s).map(([id, v]) => [id, Room.cleanSnapshot(JSON.parse(JSON.stringify(v)))]));
  assert.ok(late[ids[1]].host, 'the copy carries the match settings');
  assert.equal(R.fingerprint(Room.replay(late[ids[1]].host, late, gone).match), R.fingerprint(match));
});

test('mirroring copies only what this screen has played, and only into a player in the match', () => {
  const s = { [ids[0]]: snap('GS-AA1', { shops: { 1: [] } }), [ids[1]]: snap('GS-BB2', { shops: { 1: [] } }), [ids[2]]: snap('GS-CC3') };
  const { match } = settle(s);
  assert.equal(Room.mirror(match, host, s, ids[2]), false, 'a watcher keeps no copy');
  assert.equal(Room.mirror(match, host, s, ids[1]), true);
  assert.deepEqual(s[ids[1]].shopLog, s[ids[0]].shopLog);
  assert.equal(Room.mirror(match, host, s, ids[1]), false, 'nothing new the second time');
  // An entry in a snapshot that no screen has played is not copied.
  s[ids[0]].log[999] = 'skip';
  Room.mirror(match, host, s, ids[1]);
  assert.equal(s[ids[1]].log[999], undefined);
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
            if (hit && p <= 10) assert.fail(`n=${n} seed ${seed}: hits the ring at power ${p}`);
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
  const base = { p: Room.ROOM_PROTOCOL, cs: 'GS-AA1', joined: 1, turns: {}, shops: {} };
  assert.equal(Room.cleanSnapshot({ ...base, rising: 4 }).rising, 4);
  for (const bad of [-1, 1e6, 'x', 2.5, null]) assert.equal(Room.cleanSnapshot({ ...base, rising: bad }).rising, undefined, String(bad));
});

test('a shot or shop list with a weapon that is not a known id is dropped, without throwing', () => {
  const base = { p: Room.ROOM_PROTOCOL, cs: 'GS-AA1', joined: 1 };
  for (const [i, weapon] of [{ toString: 1 }, ['rocket'], 'toString', '__proto__', 7].entries()) {
    const s = Room.cleanSnapshot({ ...base, turns: { 0: { angle: 45, power: 50, weapon } }, shops: { 1: [weapon, 'rocket'] } });
    assert.deepEqual(s.turns, {}, `case ${i}`);
    assert.deepEqual(s.shops[1], ['rocket']);
  }
});
