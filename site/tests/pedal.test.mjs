// Tests for the pedal trainer's state logic (src/lib/pedal.mjs). Run with `npm test`.
import test from 'node:test';
import assert from 'node:assert/strict';
import { check, clampOffset, frequencyMHz, inputPercent, knobAngle, signal, tuneAngle, TASKS } from '../src/lib/pedal.mjs';

const noise = Array.from({ length: 201 }, (_, i) => Math.sin(i * 1.7));

test('frequency starts one step below the ISS preset and moves in 5 kHz steps', () => {
  assert.equal(frequencyMHz('2', 0), 145.8);
  assert.equal(frequencyMHz('2', -1), 145.795);
  assert.equal(frequencyMHz('70', 2), 435.01);
  assert.equal(clampOffset(99), 20);
  assert.equal(clampOffset(-99), -20);
});

test('only 2 m at 145.800 MHz is on station, and GAIN raises the peak', () => {
  assert.equal(signal(noise, { band: '2', offset: 0, gainDb: 0 }).tuned, true);
  assert.equal(signal(noise, { band: '70', offset: 0, gainDb: 0 }).tuned, false);
  assert.equal(signal(noise, { band: '2', offset: 1, gainDb: 0 }).tuned, false);
  const low = signal(noise, { band: '2', offset: 0, gainDb: 0 }).peak;
  const high = signal(noise, { band: '2', offset: 0, gainDb: 20 }).peak;
  assert.ok(Math.abs(high / low - 10) < 1e-9, '20 dB is ten times the amplitude');
  assert.ok(signal(noise, { band: '2', offset: 0, gainDb: 42 }).peak > 1, 'full GAIN clips on station');
  assert.ok(signal(noise, { band: '2', offset: 5, gainDb: 42 }).peak < 1, 'noise alone does not clip');
});

test('knobs turn through 270 degrees and the encoder turns per step', () => {
  assert.equal(knobAngle(0, 0, 42), -135);
  assert.equal(knobAngle(21, 0, 42), 0);
  assert.equal(knobAngle(42, 0, 42), 135);
  assert.equal(knobAngle(500, 0, 100), 135);
  assert.equal(tuneAngle(-1), -18);
  assert.equal(tuneAngle(0), 0);
});

test('the input meter reads whole percent of full scale', () => {
  assert.equal(inputPercent(0.756), 76);
});

test('the three steps pass in order with the right hints', () => {
  const base = { engaged: false, tuned: false, sawClip: false, peak: 0, level: 25 };
  assert.deepEqual(check({ ...base, stage: 0 }), { stage: 0, feedback: 'Tap ENGAGE first.' });
  assert.equal(check({ ...base, stage: 0, engaged: true }).stage, 1);
  assert.equal(check({ ...base, stage: 1, engaged: true }).feedback, 'Check the band and frequency on the display.');
  assert.equal(check({ ...base, stage: 1, engaged: true, tuned: true }).stage, 2);
  const tuned = { ...base, stage: 2, engaged: true, tuned: true };
  assert.equal(check({ ...tuned, peak: 0.8 }).feedback, 'Turn GAIN up until the input meter says CLIPPING.');
  assert.equal(check({ ...tuned, sawClip: true, peak: 1.2 }).feedback, 'Check the input percentage and output LEVEL.');
  assert.equal(check({ ...tuned, sawClip: true, peak: 0.8, level: 50 }).stage, 2);
  assert.equal(check({ ...tuned, sawClip: true, peak: 0.8, level: 30 }).stage, 3);
  assert.equal(check({ ...tuned, stage: 3, sawClip: true, peak: 0.8 }).stage, 3);
  assert.equal(TASKS.length, 4);
});

import { chain, carrierOffsetKhz, EXPERIMENTS, gainTimes, judge } from '../src/lib/pedal.mjs';
import { passDebrief } from '../src/lib/pass.mjs';

const reading = (o = {}) => {
  const r = { band: '2', offset: 0, gainDb: 20, level: 30, engaged: true, ...o };
  const s = signal(noise, { band: r.band, offset: r.offset, gainDb: r.gainDb });
  return { ...r, peak: s.peak, tuned: s.tuned };
};
const blocks = r => Object.fromEntries(chain({ ...r, offsetKhz: carrierOffsetKhz(r.band, r.offset) }).map(b => [b.id, b]));

test('the signal chain shows where the signal is stopped', () => {
  const ok = blocks(reading({ gainDb: 16 }));
  assert.equal(ok.filter.status, 'ok');
  assert.equal(ok.mixer.status, 'ok');
  assert.equal(ok.out.now, 'clean beacon');
  assert.equal(blocks(reading({ band: '70' })).filter.status, 'stop');
  const off = blocks(reading({ offset: 1 }));
  assert.equal(off.mixer.status, 'stop');
  assert.match(off.mixer.why, /down/, 'carrier below the tuning: turn TUNE down');
  assert.equal(blocks(reading({ gainDb: 42 })).adc.status, 'stop');
  assert.equal(blocks(reading({ engaged: false })).out.now, 'muted');
  assert.equal(carrierOffsetKhz('2', -1), 5);
  assert.equal(gainTimes(20), 10);
  assert.equal(gainTimes(6), 2);
});

test('each experiment is recognised when it is carried out, and only then', () => {
  const ex = Object.fromEntries(EXPERIMENTS.map(e => [e.id, e]));
  assert.ok(ex['six-db'].done(reading({ gainDb: 10 }), reading({ gainDb: 16 })));
  assert.ok(!ex['six-db'].done(reading({ gainDb: 10 }), reading({ gainDb: 12 })));
  assert.ok(ex['level-clip'].done(reading({ gainDb: 42, level: 60 }), reading({ gainDb: 42, level: 30 })));
  assert.ok(!ex['level-clip'].done(reading({ gainDb: 10, level: 60 }), reading({ gainDb: 10, level: 30 })));
  assert.ok(ex.band.done(reading(), reading({ band: '70' })));
  assert.ok(!ex.band.done(reading({ offset: 2 }), reading({ band: '70', offset: 2 })));
  assert.ok(ex['off-tune'].done(reading(), reading({ offset: 1 })));
  assert.ok(!ex['off-tune'].done(reading({ offset: 1 }), reading({ offset: 2 })));
  for (const e of EXPERIMENTS) {
    assert.equal(judge(e, e.answer).right, true);
    assert.equal(judge(e, (e.answer + 1) % e.choices.length).right, false);
  }
});

test('the pass debrief names where the learner fell behind', () => {
  const log = Array.from({ length: 90 }, (_, i) => ({ passS: i, inView: true, dopplerHz: 0, tunedOffsetHz: 0, heard: i < 30 || i > 60 }));
  assert.equal(passDebrief(log).part, 'overhead');
  assert.equal(passDebrief(log.map(s => ({ ...s, heard: true }))).part, null);
  assert.equal(passDebrief([]), null);
});
