// Satellite pass model for the pedal trainer's pass mode (src/components/PedalLab.astro).
// The same formulas as the station simulator in src/receiver.rs, so the website, the
// phone app and `cubesat-classroom receiver-sim` agree. tests/pass.test.mjs checks the
// numbers against the Rust output. Everything here is a desk simulation, not a measurement.

const EARTH_RADIUS_KM = 6371;
const MU_KM3_S2 = 398600.4418;
const LIGHT_KM_S = 299792.458;

/** Defaults match `receiver-sim`: ISS height, a 45 degree pass, the CSC-R1P pedal. */
export const DEFAULTS = Object.freeze({
  altitudeKm: 420,
  maxElevationDeg: 45,
  nominalHz: 145_800_000,
  levelAt1000kmDbfs: -80,
  noiseAtZeroGainDbfs: -96,
});

/** Receiver passband half-width in Hz: within this the carrier is heard clearly. */
export const PASSBAND_HZ = 1500;
/** TUNE step sizes. Pressing the TUNE knob switches between them. */
export const STEPS_HZ = [5000, 1000];
/** SNR needed to decode the beacon. */
export const DECODE_SNR_DB = 10;

export function makePass(options = {}) {
  const o = { ...DEFAULTS, ...options };
  const r = EARTH_RADIUS_KM + o.altitudeKm;
  const rate = Math.sqrt(MU_KM3_S2 / r) / r;
  const ratio = EARTH_RADIUS_KM / r;
  const horizon = Math.acos(ratio);
  const el = (o.maxElevationDeg * Math.PI) / 180;
  const cross = Math.acos(ratio * Math.cos(el)) - el;
  const duration = (2 * Math.acos(Math.min(1, Math.max(-1, Math.cos(horizon) / Math.cos(cross))))) / rate;

  /** Geometry t seconds after the highest point. */
  function geometry(t) {
    const along = rate * t;
    const psi = Math.acos(Math.min(1, Math.max(-1, Math.cos(along) * Math.cos(cross))));
    const rangeKm = Math.sqrt(EARTH_RADIUS_KM ** 2 + r * r - 2 * EARTH_RADIUS_KM * r * Math.cos(psi));
    const elevationDeg = (Math.atan2(r * Math.cos(psi) - EARTH_RADIUS_KM, r * Math.sin(psi)) * 180) / Math.PI;
    return { rangeKm, elevationDeg };
  }

  function dopplerHz(t) {
    const rangeRate = (geometry(t + 0.5).rangeKm - geometry(t - 0.5).rangeKm) / 1;
    return (-o.nominalHz * rangeRate) / LIGHT_KM_S;
  }

  /**
   * State `passS` seconds after rise (negative before it).
   * @param {number} passS
   * @param {{gainDb?: number, tunedHz?: number}} rx
   */
  function at(passS, { gainDb = 20, tunedHz = o.nominalHz } = {}) {
    const t = passS - duration / 2;
    const geo = geometry(t);
    const inView = geo.elevationDeg >= 0 && passS >= 0 && passS <= duration;
    const doppler = inView ? dopplerHz(t) : 0;
    const receivedHz = o.nominalHz + doppler;
    const offsetHz = receivedHz - tunedHz;
    const noiseDbfs = o.noiseAtZeroGainDbfs + gainDb;
    // Off tune, the carrier falls outside the passband and is not heard.
    const heard = inView && Math.abs(offsetHz) <= PASSBAND_HZ;
    const signalDbfs = inView ? o.levelAt1000kmDbfs + gainDb + 20 * Math.log10(1000 / geo.rangeKm) : -Infinity;
    const noisePower = 10 ** (noiseDbfs / 10);
    const signalPower = 10 ** (signalDbfs / 10);
    const snrDb = heard ? signalDbfs - noiseDbfs : -Infinity;
    const adcDbfs = 10 * Math.log10(noisePower + signalPower);
    const total = 10 * Math.log10(noisePower + (heard ? signalPower : 0));
    const clipping = adcDbfs > -1;
    return {
      label: 'desk simulation',
      passS,
      inView,
      elevationDeg: geo.elevationDeg,
      rangeKm: geo.rangeKm,
      dopplerHz: doppler,
      receivedHz,
      offsetHz,
      heard,
      dbfs: Math.min(0, total),
      noiseDbfs,
      adcDbfs,
      snrDb,
      clipping,
      decoded: heard && !clipping && snrDb >= DECODE_SNR_DB,
    };
  }

  return { duration, geometry, dopplerHz, at, nominalHz: o.nominalHz };
}

/**
 * Score a finished pass: the share of in-view seconds the carrier was heard and decoded with
 * the pedal engaged (a muted second plays nothing, however well tuned).
 * @param {{inView: boolean, heard: boolean, decoded: boolean, engaged?: boolean}[]} log one entry per simulated second
 */
export function score(log) {
  const seen = log.filter((s) => s.inView);
  if (seen.length === 0) return { heard: 0, decoded: 0 };
  const on = s => s.engaged !== false;
  return {
    heard: Math.round((100 * seen.filter((s) => s.heard && on(s)).length) / seen.length),
    decoded: Math.round((100 * seen.filter((s) => s.decoded && on(s)).length) / seen.length),
  };
}

/**
 * After a pass: why seconds were missed. `log` has one entry per simulated second with
 * { passS, inView, dopplerHz, tunedOffsetHz, heard, engaged }, where `heard` is about tuning
 * alone and `engaged` says whether the output was on. Muted seconds are explained as muted,
 * never as tuning; tuning advice comes only from seconds when the pedal was engaged.
 * Returns the part of the pass with the most missed seconds and a sentence about it, or null.
 */
export function passDebrief(log) {
  const seen = log.filter(s => s.inView);
  if (!seen.length) return null;
  const on = s => s.engaged !== false;
  const missed = seen.filter(s => !(s.heard && on(s)));
  if (!missed.length) return { part: null, text: 'You kept the carrier in the passband for the whole pass. Notice where you had to retune most often: near the top of the pass, where the Doppler shift changes fastest.' };
  const muted = missed.filter(s => !on(s));
  const offTune = missed.filter(s => on(s));
  if (muted.length >= offTune.length) {
    const tuned = muted.filter(s => s.heard).length;
    return {
      part: 'muted',
      text: `The pedal was muted (ENGAGE off) for ${muted.length} s of the pass, so nothing played${tuned ? `, even though you were tuned to the carrier for ${tuned} s of that` : ''}. Tap ENGAGE before the satellite rises, and check its light stays on.`,
      missedS: missed.length,
    };
  }
  const start = seen[0].passS, len = seen[seen.length - 1].passS - start || 1;
  const parts = ['rising', 'overhead', 'setting'];
  const counts = [0, 0, 0];
  for (const s of offTune) counts[Math.min(2, Math.floor((3 * (s.passS - start)) / len))]++;
  const part = parts[counts.indexOf(Math.max(...counts))];
  const text = {
    rising: 'You missed most while the satellite was rising. It arrives about 3.4 kHz high, so start the pass tuned above 145.800 MHz instead of on it.',
    overhead: 'You missed most while the satellite was overhead. That is where the Doppler shift swings fastest, from high to low in about two minutes, so retune in small steps, often.',
    setting: 'You missed most while the satellite was setting. It ends about 3.4 kHz low, so keep turning TUNE down as it leaves.',
  }[part];
  return { part, text, missedS: missed.length };
}
