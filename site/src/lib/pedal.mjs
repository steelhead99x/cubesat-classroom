// State logic for the satellite pedal trainer (src/components/PedalLab.astro).
// Kept free of the DOM so tests/pedal.test.mjs can check it with node:test.
// Everything here is a desk simulation of the proposed CSC-R1 Rev B pedal, not a measurement.

/** Off station there is only noise; on station a carrier rides above it. GAIN lifts both. */
export const NOISE = 0.006;
export const CARRIER = 0.12;
export const STEP_KHZ = 5;
export const MAX_OFFSET = 20;
export const GAIN_MAX_DB = 42;
/** Knob travel, like a real pot: 270 degrees from fully left to fully right. */
export const KNOB_SWEEP = 270;
/** Degrees the TUNE encoder turns per 5 kHz step (20 detents a turn). */
export const TUNE_DEGREES_PER_STEP = 18;

export const TASKS = [
  '1 of 3 · Tap ENGAGE to unmute the receiver.',
  '2 of 3 · Select 2 m. Tune to 145.800 MHz in 5 kHz steps.',
  '3 of 3 · Raise GAIN until the peaks clip. Then lower GAIN until the input meter reads 60–95%. Set LEVEL between 20% and 40%.',
  'Complete · Write down the frequency, gain and level that worked.',
];

/** @param {string} band '2' or '70' @param {number} offset steps from the band's preset */
export function frequencyMHz(band, offset) {
  return ((band === '2' ? 145800 : 435000) + STEP_KHZ * offset) / 1000;
}

export function clampOffset(offset) {
  return Math.max(-MAX_OFFSET, Math.min(MAX_OFFSET, offset));
}

/**
 * @param {number[]} noise fixed noise samples in -1..1
 * @param {{band: string, offset: number, gainDb: number}} s
 */
export function signal(noise, { band, offset, gainDb }) {
  const tuned = band === '2' && offset === 0;
  const g = 10 ** (gainDb / 20);
  const samples = noise.map((n, i) => ((tuned ? CARRIER * Math.sin((i * Math.PI) / 20) : 0) + NOISE * n) * g);
  return { tuned, samples, peak: Math.max(...samples.map(Math.abs)) };
}

/** Pot angle in degrees, -135 (fully left) to +135 (fully right). */
export function knobAngle(value, min, max) {
  const t = Math.max(0, Math.min(1, (value - min) / (max - min)));
  return -KNOB_SWEEP / 2 + KNOB_SWEEP * t;
}

export function tuneAngle(offset) {
  return offset * TUNE_DEGREES_PER_STEP;
}

/** SIGNAL LED brightness 0..1: it brightens with signal strength, as in the Rev B plan. */
export function signalLed(peak) {
  return Math.max(0.08, Math.min(1, peak));
}

/** Input meter reading, as a whole percent of the ADC's full scale. */
export function inputPercent(peak) {
  return Math.round(peak * 100);
}

/**
 * One press of "Check setting".
 * @param {{stage: number, engaged: boolean, tuned: boolean, sawClip: boolean, peak: number, level: number}} s
 * @returns {{stage: number, feedback: string}}
 */
export function check({ stage, engaged, tuned, sawClip, peak, level }) {
  if (stage >= 3) return { stage, feedback: '' };
  const pass = [
    engaged,
    engaged && tuned,
    engaged && tuned && sawClip && peak >= 0.6 && peak <= 0.95 && level >= 20 && level <= 40,
  ];
  if (pass[stage]) return { stage: stage + 1, feedback: 'Setting confirmed.' };
  const feedback = !engaged
    ? 'Tap ENGAGE first.'
    : !tuned
      ? 'Check the band and frequency on the display.'
      : !sawClip
        ? 'Turn GAIN up until the input meter says CLIPPING.'
        : 'Check the input percentage and output LEVEL.';
  return { stage, feedback };
}
