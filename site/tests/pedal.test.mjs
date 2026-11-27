// Tests for the pedal trainer's state logic (src/lib/pedal.mjs). Run with `npm test`.
import test from 'node:test';
import assert from 'node:assert/strict';
import { check, clampOffset, frequencyMHz, inputPercent, knobAngle, signal, signalLed, tuneAngle, TASKS } from '../src/lib/pedal.mjs';

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

test('SIGNAL LED brightens with signal strength but never quite goes dark', () => {
  assert.equal(signalLed(0), 0.08);
  assert.equal(signalLed(0.5), 0.5);
  assert.equal(signalLed(3), 1);
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
