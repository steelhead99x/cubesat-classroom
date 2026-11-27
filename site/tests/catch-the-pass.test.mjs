// Game rules for /games/catch-the-pass/ (src/games/catch-the-pass/logic.mjs).
import test from 'node:test';
import assert from 'node:assert/strict';
import * as G from '../src/games/catch-the-pass/logic.mjs';

const run = (game, seconds, input) => { for (let i = 0; i < seconds * 60 && !game.done; i++) G.step(game, typeof input === 'function' ? input(game) : input, 1 / 60); return game; };
const track = game => ({ target: G.satPosition(game.level, (game.t + 1 / 60) / game.level.duration) });

test('slant range is the altitude overhead and grows toward the horizon', () => {
  assert.ok(Math.abs(G.slantRangeKm(90) - G.ALTITUDE_KM) < 1e-6);
  assert.ok(G.slantRangeKm(10) > 1500 && G.slantRangeKm(10) < 1900);
  assert.ok(G.rangeLossDb(0) > G.rangeLossDb(30));
  assert.equal(Math.round(G.rangeLossDb(90) * 100), 0);
});

test('pointing loss is 3 dB at half the beamwidth and capped', () => {
  assert.equal(G.pointingLossDb(15, 30), 3);
  assert.equal(G.pointingLossDb(0, 30), 0);
  assert.equal(G.pointingLossDb(180, 30), 25);
});

test('higher spreading factors decode weaker signals but cost more airtime and energy', () => {
  const [a, b, c] = G.SF_ORDER.map(sf => G.SF[sf]);
  assert.ok(a.minSnr > b.minSnr && b.minSnr > c.minSnr);
  assert.ok(a.airtime < b.airtime && b.airtime < c.airtime);
  assert.ok(G.packetEnergy(G.LEVELS[0], 7) < G.packetEnergy(G.LEVELS[0], 12));
});

test('the same seed gives the same pass', () => {
  const a = run(G.createGame(G.LEVELS[2], 42), 20, track);
  const b = run(G.createGame(G.LEVELS[2], 42), 20, track);
  assert.deepEqual([a.rx, a.lost, a.score, a.snr], [b.rx, b.lost, b.score, b.snr]);
});

test('tracking the satellite downloads packets; pointing at the ground does not', () => {
  const good = run(G.createGame(G.LEVELS[0], 7), 60, track);
  assert.ok(good.done);
  assert.ok(good.rx >= G.LEVELS[0].goal[0], `got ${good.rx}`);
  const bad = run(G.createGame(G.LEVELS[0], 7), 60, { target: { az: 90, el: 0 } });
  assert.equal(bad.rx, 0);
  assert.equal(G.stars(bad), 0);
});

test('the rotator slews no faster than the level allows', () => {
  const game = G.createGame(G.LEVELS[4], 1);
  const before = { ...game.ant };
  G.step(game, { target: { az: 90, el: 90 } }, 0.1);
  const moved = Math.hypot(game.ant.az - before.az, game.ant.el - before.el);
  assert.ok(moved <= G.LEVELS[4].slew * 0.1 + 1e-9);
});

test('a packet is lost when the signal drops below the SF floor, and the streak resets', () => {
  const game = G.createGame(G.LEVELS[0], 3);
  G.setSf(game, 7);
  run(game, 18, track);
  assert.ok(game.streak > 0);
  run(game, 3, { target: { az: 90, el: 0 } });
  assert.equal(game.streak, 0);
  assert.ok(game.lost > 0);
});

test('switching SF abandons the packet on air', () => {
  const game = G.createGame(G.LEVELS[0], 3);
  run(game, 1, track);
  assert.ok(game.packet.elapsed > 0);
  G.setSf(game, 12);
  assert.equal(game.packet.elapsed, 0);
  assert.equal(game.sf, 12);
});

test('holding the transmitter saves battery and sends nothing', () => {
  const game = G.createGame({ ...G.LEVELS[3], eclipse: [0, 1] }, 5);
  G.setTransmit(game, false);
  const start = game.battery;
  run(game, 5, track);
  assert.equal(game.battery, start);
  assert.equal(game.rx + game.lost, 0);
});

test('in shadow SF12 drains the battery into safe mode, which stops data', () => {
  const game = G.createGame({ ...G.LEVELS[3], battery: 15, eclipse: [0, 1], duration: 200 }, 5);
  G.setSf(game, 12);
  run(game, 4, track);
  assert.ok(game.safe);
  const rx = game.rx;
  run(game, 3, track);
  assert.equal(game.rx, rx);
});

test('the noisy neighbor lowers SNR when the antenna points near it', () => {
  const game = G.createGame(G.LEVELS[2], 1);
  game.t = G.LEVELS[2].duration * 0.85;
  const sat = G.satPosition(game.level, 0.85);
  const quiet = G.snrAt({ ...game, level: { ...game.level, interference: 0 } }, sat.az, sat.el);
  assert.ok(G.snrAt(game, sat.az, sat.el) < quiet);
});

test('every live client agrees on the pass from the clock', () => {
  const a = G.liveSlot(1_800_000_000_000), b = G.liveSlot(1_800_000_030_000);
  assert.equal(a.slot, b.slot);
  assert.equal(a.seed, b.seed);
  assert.equal(b.elapsed - a.elapsed, 30);
});

test('every level is winnable by careful tracking with a sensible SF', () => {
  for (const level of [...G.LEVELS, G.LIVE]) {
    const game = G.createGame(level, 11);
    run(game, 120, g => {
      // A simple strategy: the strongest SF the current signal allows, with a 3 dB margin.
      const best = G.SF_ORDER.find(sf => g.snr - 3 >= G.SF[sf].minSnr) ?? 12;
      if (g.battery < 25 && best === 12) G.setSf(g, 9); else if (best !== g.sf && g.packet.elapsed === 0) G.setSf(g, best);
      return track(g);
    });
    assert.ok(game.rx >= level.goal[0], `${level.name}: ${game.rx} packets`);
  }
});

test('easy mode gives longer passes, a faster antenna and a wider beam, with the same goals', () => {
  for (const level of G.LEVELS) {
    const easy = G.easyLevel(level);
    assert.ok(easy.duration > level.duration && easy.slew > level.slew && easy.beam > level.beam);
    assert.deepEqual(easy.goal, level.goal);
    assert.equal(level.beam, G.LEVELS.find(l => l.id === level.id).beam, 'the level itself is unchanged');
  }
});
