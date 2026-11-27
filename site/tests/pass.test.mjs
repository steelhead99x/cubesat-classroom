// Tests for the pass model (src/lib/pass.mjs). Run with `npm test`.
import test from 'node:test';
import assert from 'node:assert/strict';
import { makePass, score, PASSBAND_HZ } from '../src/lib/pass.mjs';

// Reference values printed by `cargo run --bin cubesat-classroom -- receiver-sim --start-s N --seconds 1`
// (pass_s = N - 60). If the Rust model changes, regenerate these and keep both in step.
const RUST = [
  { passS: 0, elevationDeg: 0.0, rangeKm: 2351.2, dopplerHz: 3442.6, snrDb: 8.6 },
  { passS: 140, elevationDeg: 11.9, rangeKm: 1377.6, dopplerHz: 3259.7, snrDb: 13.2 },
  { passS: 270, elevationDeg: 38.5, rangeKm: 645.1, dopplerHz: 1615.9, snrDb: 19.8 },
  { passS: 440, elevationDeg: 17.2, rangeKm: 1128.7, dopplerHz: -3091.1, snrDb: 14.9 },
  { passS: 590, elevationDeg: 1.9, rangeKm: 2153.2, dopplerHz: -3430.4, snrDb: 9.3 },
];

test('matches the station simulator in src/receiver.rs', () => {
  const pass = makePass();
  for (const want of RUST) {
    // The Rust simulator does not track Doppler by default, so tune to the received carrier
    // here to get the full signal, as its snr_db assumes.
    const got = pass.at(want.passS, { tunedHz: pass.nominalHz + pass.dopplerHz(want.passS - pass.duration / 2) });
    assert.ok(Math.abs(got.elevationDeg - want.elevationDeg) < 0.06, `${want.passS}: ${got.elevationDeg}`);
    assert.ok(Math.abs(got.rangeKm - want.rangeKm) < 0.06, `${want.passS}: ${got.rangeKm}`);
    assert.ok(Math.abs(got.dopplerHz - want.dopplerHz) < 0.06, `${want.passS}: ${got.dopplerHz}`);
    assert.ok(Math.abs(got.snrDb - want.snrDb) < 0.06, `${want.passS}: ${got.snrDb}`);
  }
});

test('before rise there is nothing to hear', () => {
  const s = makePass().at(-30);
  assert.equal(s.inView, false);
  assert.equal(s.heard, false);
  assert.equal(s.label, 'desk simulation');
});

test('staying on the nominal frequency loses the carrier as Doppler grows', () => {
  const pass = makePass();
  const top = pass.at(pass.duration / 2);
  assert.ok(top.heard, 'near the top Doppler is small');
  const early = pass.at(30);
  assert.ok(Math.abs(early.offsetHz) > PASSBAND_HZ);
  assert.equal(early.heard, false);
  const followed = pass.at(30, { tunedHz: 145_803_000 });
  assert.ok(followed.heard, 'retuning 3 kHz up catches it');
});

test('too much gain clips and stops the decode', () => {
  const pass = makePass({ levelAt1000kmDbfs: -40 });
  const s = pass.at(pass.duration / 2, { gainDb: 42 });
  assert.ok(s.clipping);
  assert.ok(s.dbfs <= 0);
  assert.equal(s.decoded, false);
});

test('score counts only seconds in view', () => {
  assert.deepEqual(score([]), { heard: 0, decoded: 0 });
  const log = [
    { inView: false, heard: false, decoded: false },
    { inView: true, heard: true, decoded: true },
    { inView: true, heard: true, decoded: false },
    { inView: true, heard: false, decoded: false },
    { inView: true, heard: true, decoded: true },
  ];
  assert.deepEqual(score(log), { heard: 75, decoded: 50 });
});
