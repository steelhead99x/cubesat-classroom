// State logic for the satellite pedal trainer (src/components/PedalLab.astro).
// Kept free of the DOM so tests/pedal.test.mjs can check it with node:test.
// Everything here is a desk simulation of the CSC-R1P pedal (hardware/sdr-receiver/pedal), not a measurement.

/** Off station there is only noise; on station a carrier rides above it. GAIN lifts both. */
export const NOISE = 0.006;
export const CARRIER = 0.12;
export const STEP_KHZ = 5;
export const MAX_OFFSET = 20;
export const GAIN_MAX_DB = 42;
export const amplitudeGain = db => 10 ** (db / 20);
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
  const g = amplitudeGain(gainDb);
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

/** Amplitude factor for a GAIN setting, rounded for display: 20 dB is ×10, 6 dB about ×2. */
export const gainTimes = db => {
  const x = amplitudeGain(db);
  return x < 10 ? Math.round(x * 10) / 10 : Math.round(x);
};

/** Where the 145.800 MHz carrier sits relative to where the pedal is tuned, in kHz (positive: above). */
export function carrierOffsetKhz(band, offset) {
  return band === '2' ? -STEP_KHZ * offset : null;
}

/**
 * The receiver's signal chain, antenna to OUT, with what each stage is doing right now. The
 * trainer draws it as a row of blocks: `status` colours a block (ok, warn, stop or idle),
 * `now` is its live reading and `why` explains it.
 * @param {{band: string, offsetKhz: number | null, inBand?: boolean, peak: number, gainDb: number, level: number, engaged: boolean, passbandKhz?: number, noCarrier?: boolean}} s
 *   offsetKhz: carrier minus tuning in kHz (null when the band filter blocks it).
 *   noCarrier: there is no carrier to tune to yet (the satellite is below the horizon).
 */
export function chain({ band, offsetKhz, peak, gainDb, level, engaged, passbandKhz = 1.5, noCarrier = false }) {
  const onBand = band === '2';
  const waiting = onBand && noCarrier;
  const inPass = offsetKhz !== null && Math.abs(offsetKhz) <= passbandKhz;
  const clipping = peak > 1;
  const blocks = [
    { id: 'antenna', name: 'Antenna', status: 'ok', now: 'picking up 145.800 MHz and everything else',
      why: 'The antenna collects every signal around it: the satellite, FM radio, and noise. The stages after it pick out the one you want.' },
    { id: 'lna', name: 'Amplifier', status: 'ok', now: 'boosting the faint signal',
      why: 'The low-noise amplifier makes the signal stronger right at the start, before the later stages can add their own noise.' },
    { id: 'filter', name: 'Band filter', status: onBand ? 'ok' : 'stop',
      now: onBand ? '2 m filter: 145.800 MHz passes' : '70 cm filter: 145.800 MHz is blocked',
      why: onBand
        ? 'BAND picks a filter. The 2 m filter passes about 125–185 MHz, around the 144–148 MHz band, and blocks strong signals outside it, such as FM radio near 100 MHz.'
        : 'BAND is on 70 cm: that filter passes about 370–520 MHz, so it blocks the 145.800 MHz satellite before it reaches the tuner. Flip BAND to 2 m.' },
    { id: 'mixer', name: 'Tuner', status: !onBand || waiting ? 'idle' : inPass ? 'ok' : 'stop',
      now: !onBand ? 'nothing to tune to' : waiting ? 'no carrier yet: the satellite is below the horizon' : inPass ? `carrier ${fmtKhz(offsetKhz)} from the tuning: inside the passband` : `carrier ${fmtKhz(offsetKhz)} from the tuning: outside the ±${passbandKhz} kHz passband`,
      why: waiting
        ? `There is nothing to tune to until the satellite rises. It will arrive about 3.4 kHz high, so you can get ready by tuning a little above 145.800 MHz.`
        : inPass || !onBand
        ? `TUNE sets the frequency the mixer listens around. Only signals within about ±${passbandKhz} kHz of it are heard.`
        : `The carrier is ${fmtKhz(offsetKhz)} away, but only ±${passbandKhz} kHz around the tuning gets through. Turn TUNE ${offsetKhz > 0 ? 'up' : 'down'} toward it.` },
    { id: 'adc', name: `ADC · GAIN ${gainDb} dB`, status: clipping ? 'stop' : peak >= 0.6 ? 'ok' : peak >= 0.2 ? 'warn' : 'idle',
      now: clipping ? 'over full scale: CLIPPING' : `input ${inputPercent(peak)}% of full scale`,
      why: clipping
        ? 'The signal is bigger than the ADC can measure, so the tops of the wave are cut flat. That damage cannot be undone later: lower GAIN.'
        : `GAIN multiplies the signal before the ADC turns it into numbers: ${gainDb} dB is ×${gainTimes(gainDb)} in size. Aim for 60–95% so it is big but not clipped.` },
    { id: 'level', name: `LEVEL ${level}%`, status: !engaged ? 'idle' : level >= 20 && level <= 40 ? 'ok' : 'warn',
      now: engaged ? `output at ${level}%` : 'muted until you ENGAGE',
      why: 'LEVEL only sets how loud the OUT jack is. It comes after the ADC, so it cannot fix a clipped or missing signal.' },
    { id: 'out', name: 'OUT', status: !engaged ? 'idle' : (onBand && inPass && !clipping ? 'ok' : 'warn'),
      now: !engaged ? 'muted' : onBand && inPass && !waiting ? (clipping ? 'distorted beacon' : 'clean beacon') : 'only noise',
      why: 'The ¼ inch jack sends the sound to an amplifier or a computer. ENGAGE turns it on.' },
  ];
  return blocks;
}

function fmtKhz(k) {
  const v = Math.abs(k) < 10 ? Math.round(k * 10) / 10 : Math.round(k);
  return `${v > 0 ? '+' : ''}${v} kHz`;
}

/**
 * Predict, then test. Each experiment asks for a prediction, then watches the learner try it.
 * `done(prev, next)` gets two readings of the trainer (before and after a change) and is true
 * when the change is the experiment being carried out. If `keep(prev, next)` is true, the
 * trainer keeps comparing against the earlier reading (a change made in several small steps).
 * A reading is { band, offset, gainDb, level, engaged, peak, tuned }.
 */
export const EXPERIMENTS = [
  {
    id: 'six-db',
    question: 'Tuned to the satellite, you turn GAIN up by 6 dB. What happens to the input meter?',
    choices: ['It about doubles', 'It goes up by 6%', 'It stays the same'],
    answer: 0,
    task: 'Tune to 145.800 MHz on 2 m, set the input below 45%, then raise GAIN by 6 dB.',
    explain: 'Decibels multiply. Every 6 dB is about twice the size of the wave, and every 20 dB is ten times. That is why a small turn of GAIN can take a signal from quiet to clipping.',
    done: (a, b) => a.tuned && b.tuned && a.peak < 0.45 && b.gainDb - a.gainDb >= 6 && b.peak <= 1,
    // GAIN moves 1 dB at a time, so compare with where the turn started, not the last step.
    keep: (a, b) => a.tuned && b.tuned && a.band === b.band && a.offset === b.offset && b.gainDb >= a.gainDb,
  },
  {
    id: 'level-clip',
    question: 'The input is clipping. You turn LEVEL down. Does the wave get its round tops back?',
    choices: ['Yes, LEVEL fixes it', 'No, it just gets quieter'],
    answer: 1,
    task: 'Make the input clip with GAIN, ENGAGE, then turn LEVEL down.',
    explain: 'Clipping happens in the ADC, before LEVEL. Turning LEVEL down makes the already-flattened wave smaller, but it stays flat. Only lowering GAIN fixes it.',
    done: (a, b) => a.engaged && b.engaged && a.peak > 1 && b.peak > 1 && b.level < a.level,
  },
  {
    id: 'band',
    question: 'You are tuned to the satellite on 2 m. You flip BAND to 70 cm. What happens?',
    choices: ['The signal gets stronger', 'The satellite disappears', 'Nothing changes'],
    answer: 1,
    task: 'Tune to 145.800 MHz on 2 m, then flip BAND to 70 cm.',
    explain: 'The 70 cm filter only passes signals near 435 MHz, so the 145.800 MHz satellite is blocked before it is ever tuned. Filters keep strong signals from other bands out.',
    done: (a, b) => a.band === '2' && a.tuned && b.band === '70',
  },
  {
    id: 'off-tune',
    question: 'You are tuned to the satellite. You turn TUNE one step, 5 kHz. Can you still hear it?',
    choices: ['Yes, 5 kHz is tiny', 'No, it is outside the passband'],
    answer: 1,
    task: 'Tune to 145.800 MHz on 2 m, then turn TUNE one step either way.',
    explain: 'The receiver only hears about ±1.5 kHz around its tuning. 5 kHz is less than 0.004% of 145.8 MHz, but it is still outside that window. In pass mode the satellite’s own Doppler shift moves it that far.',
    done: (a, b) => a.band === '2' && a.tuned && b.band === '2' && !b.tuned && Math.abs(b.offset - a.offset) === 1,
  },
];

/** Feedback on a prediction: whether it was right, and the explanation either way. */
export function judge(experiment, choice) {
  const right = choice === experiment.answer;
  return { right, text: `${right ? 'Your prediction was right.' : `You predicted “${experiment.choices[choice]}”. The answer is “${experiment.choices[experiment.answer]}”.`} ${experiment.explain}` };
}
