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

test('the rotator slews no faster than the selected antenna allows', () => {
  const game = G.createGame(G.LEVELS[4], 1);
  const before = { ...game.ant };
  G.step(game, { target: { az: 90, el: 90 } }, 0.1);
  const moved = Math.hypot(game.ant.az - before.az, game.ant.el - before.el);
  assert.ok(moved <= G.ANTENNAS.yagi.slew * 0.1 + 1e-9);
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
    assert.ok(easy.duration > level.duration);
    for (const choice of Object.keys(G.ANTENNAS)) {
      const normal = G.antennaFor(level, choice);
      const assisted = G.antennaFor(easy, choice);
      assert.ok(assisted.slew > normal.slew && assisted.beam > normal.beam);
    }
    assert.deepEqual(easy.goal, level.goal);
  }
});

test('the Yagi beats a crossed dipole on axis but loses more signal when misaimed', () => {
  const game = G.createGame(G.LEVELS[0]);
  const time = game.level.duration / 2;
  const sat = G.satPosition(game.level, 0.5);
  const snr = (offset, antenna) => G.snrAt(game, sat.az, sat.el - offset, time, antenna);
  assert.ok(snr(0, 'yagi') > snr(0, 'patch'));
  assert.ok(snr(0, 'patch') > snr(0, 'crossed'));
  assert.ok(snr(30, 'crossed') > snr(30, 'yagi'));
});

test('the receiver needs engagement and correct tuning to receive packets', () => {
  assert.equal(G.receiverFrequency(1), 915.005);
  for (const settings of [{ engaged: false }, { offset: 1 }]) {
    const game = G.createGame(G.LEVELS[0], 7, 'patch', { ...G.DEFAULT_RECEIVER, ...settings });
    run(game, 14, track);
    assert.equal(game.rx, 0);
    assert.ok(game.lost > 0);
    Object.assign(game.receiver, G.DEFAULT_RECEIVER);
    run(game, 15, track);
    assert.ok(game.rx > 0);
  }
});

test('gain restores weak signals up to the point of overload; audio level is independent', () => {
  const game = G.createGame(G.LEVELS[0]);
  game.t = game.level.duration / 2;
  game.ant = G.satPosition(game.level, 0.5);
  const nominal = G.readReceiver(game);
  game.receiver.level = 0;
  assert.deepEqual(G.readReceiver(game), nominal);
  game.receiver.gainDb = 0;
  assert.ok(G.readReceiver(game).snr < nominal.snr);
  game.receiver.gainDb = 42;
  const clipped = G.readReceiver(game);
  assert.equal(clipped.status, 'Clipping');
  assert.ok(clipped.peak > 1 && clipped.snr < nominal.snr);
});

test('returning to a live pass skips unattended time without awarding packets', () => {
  const game = G.createGame(G.LIVE, 5);
  G.step(game, {}, 0.1);
  assert.ok(game.packet.elapsed > 0);
  game.streak = 3;
  assert.equal(G.syncLiveTime(game, 0.2), false);
  assert.equal(G.syncLiveTime(game, 18), true);
  assert.equal(game.t, 18);
  assert.equal(game.packet.elapsed, 0);
  assert.equal(game.streak, 0);
  assert.equal(game.seq, 0);
  assert.equal(game.score, 0);
  assert.equal(G.syncLiveTime(game, 17), false);
  assert.equal(G.syncLiveTime(game, G.LIVE_SLOT_SECONDS), true);
  G.step(game, {}, 0);
  assert.equal(game.done, true);
  assert.equal(game.seq, 0);
});

test('joining a live pass after it ended earns no completion bonus', () => {
  const game = G.createGame(G.LIVE, 9);
  assert.equal(G.syncLiveTime(game, G.LIVE_SLOT_SECONDS), true);
  G.step(game, {}, 0);
  assert.equal(game.done, true);
  assert.equal(game.score, 0);
  // A station that flew the pass still gets it.
  const flown = G.createGame(G.LIVE, 9);
  while (!flown.done) G.step(flown, {}, 0.05);
  assert.ok(flown.seq > 0);
  if (flown.battery >= 20) assert.ok(flown.score >= 200);
});

test('live seeds use every bit and the fade does not depend on frame rate', () => {
  const lows = new Set();
  for (let i = 0; i < 64; i++) lows.add(G.liveSlot(1_800_000_000_000 + i * G.LIVE_SLOT_SECONDS * 1000).seed & 15);
  assert.ok(lows.size > 1, 'the low bits of the seed vary');
  const fadeAt = dt => {
    const g = G.createGame(G.LEVELS[0], 3);
    let worst = 0;
    for (let t = 0; t < 20; t += dt) { G.step(g, {}, dt); worst = Math.max(worst, Math.abs(g.fade)); }
    return worst;
  };
  // Both stay inside the clamp and move by a similar amount at 30 and 120 fps.
  const slow = fadeAt(1 / 30), fast = fadeAt(1 / 120);
  assert.ok(slow <= 1.5 && fast <= 1.5);
  assert.ok(Math.abs(slow - fast) < 1.2);
});
