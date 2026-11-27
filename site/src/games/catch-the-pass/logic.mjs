// Catch the Pass: game rules with no DOM, so node:test can check them (tests/catch-the-pass.test.mjs).
//
// The model is a simplified but honest link budget for a LoRa CubeSat downlink at 915 MHz:
// - The antenna loses signal as it points away from the satellite (a parabolic main lobe,
//   -12 (offset / beamwidth)^2 dB, the usual textbook approximation).
// - A low satellite is farther away (slant range), so its signal is weaker (free-space loss).
// - Each LoRa spreading factor (SF) decodes down to a different signal-to-noise ratio
//   (SF7 -7.5 dB ... SF12 -20 dB). Higher SF reaches farther but each packet takes longer
//   and costs the satellite more battery.
// - The radio uses the same power at every SF, so slow packets cost more battery each,
//   and the ground can tell the satellite to hold its transmitter to save energy.
// - In Earth's shadow the solar panels stop charging, and a flat battery puts the
//   satellite into safe mode.
// Passes are sped up about 15 times, and SF12 airtime is shortened, so a pass fits in a minute.

export const EARTH_RADIUS_KM = 6371;
export const ALTITUDE_KM = 500;

/** LoRa spreading factors the player can command. minSnr: the demodulator floor in dB. */
export const SF = {
  7: { minSnr: -7.5, airtime: 0.6 },
  9: { minSnr: -12.5, airtime: 1.5 },
  12: { minSnr: -20, airtime: 3.2 },
};

/** The transmitter draws the same power at every SF, so a slow packet costs more energy. */
export const packetEnergy = (level, sf) => level.drain * SF[sf].airtime;
export const SF_ORDER = [7, 9, 12];

/** Below this charge the satellite stops sending data; it resumes above RESUME_AT. */
export const SAFE_AT = 10;
export const RESUME_AT = 20;

export const LEVELS = [
  { id: 1, name: 'First light', duration: 40, maxEl: 78, beam: 34, slew: 70, link: 0, battery: 90, charge: 1.2, drain: 1, interference: 0, bursts: 0, eclipse: null,
    goal: [10, 20, 30], tip: 'Keep the beam circle on the satellite. Packets only arrive while the signal bar is above the line for your SF.' },
  { id: 2, name: 'Low pass', duration: 44, maxEl: 24, beam: 34, slew: 70, link: -2, battery: 90, charge: 1.2, drain: 1, interference: 0, bursts: 0, eclipse: null,
    goal: [3, 6, 9], tip: 'A low pass is far away, so the signal is weak. SF7 will not reach. SF12 reaches farthest; SF9 may work near the top.' },
  { id: 3, name: 'Noisy neighbor', duration: 42, maxEl: 55, beam: 30, slew: 60, link: -2, battery: 85, charge: 1.2, drain: 1, interference: 14, bursts: 3, eclipse: null,
    goal: [6, 12, 18], tip: 'A busy 915 MHz device sits on the right-hand horizon. Pointing near it, or a burst of noise, drowns the satellite out.' },
  { id: 4, name: 'Into the shadow', duration: 44, maxEl: 62, beam: 30, slew: 60, link: -2, battery: 40, charge: 1.6, drain: 1.8, interference: 0, bursts: 2, eclipse: [0.45, 1],
    goal: [5, 10, 16], tip: 'Low battery, and the satellite enters Earth\u2019s shadow. Hold the transmitter (Space) when you cannot hear it, and use fast SFs.' },
  { id: 5, name: 'Yagi sprint', duration: 34, maxEl: 70, beam: 16, slew: 45, link: 4, battery: 60, charge: 1.4, drain: 1.5, interference: 12, bursts: 3, eclipse: [0.65, 1],
    goal: [8, 16, 25], tip: 'A narrow, high-gain Yagi antenna: more signal, but you must point it precisely and it turns slowly.' },
];

/** The shared live pass: everyone in the live sky flies the same one at the same time. */
export const LIVE = { id: 'live', name: 'Live sky', duration: 45, maxEl: 50, beam: 28, slew: 60, link: -2, battery: 70, charge: 1.3, drain: 1.4, interference: 12, bursts: 2, eclipse: [0.7, 1],
  goal: [6, 13, 19], tip: 'Every station in the live sky tracks this same satellite right now.' };
export const LIVE_SLOT_SECONDS = 60;

/** Small deterministic PRNG (mulberry32), so a seed always gives the same pass. */
export function rng(seed) {
  let a = seed >>> 0;
  return () => {
    a = (a + 0x6d2b79f5) >>> 0;
    let t = a;
    t = Math.imul(t ^ (t >>> 15), t | 1);
    t ^= t + Math.imul(t ^ (t >>> 7), t | 61);
    return ((t ^ (t >>> 14)) >>> 0) / 4294967296;
  };
}

const rad = d => d * Math.PI / 180;
const deg = r => r * 180 / Math.PI;
const clamp = (v, lo, hi) => Math.max(lo, Math.min(hi, v));

/** Distance from the station to a satellite at this elevation, in km. */
export function slantRangeKm(elDeg, altitude = ALTITUDE_KM) {
  const R = EARTH_RADIUS_KM, e = rad(clamp(elDeg, 0, 90));
  return Math.sqrt((R + altitude) ** 2 - (R * Math.cos(e)) ** 2) - R * Math.sin(e);
}

/** Extra free-space loss compared with the satellite straight overhead, in dB. */
export function rangeLossDb(elDeg, altitude = ALTITUDE_KM) {
  return 20 * Math.log10(slantRangeKm(elDeg, altitude) / altitude);
}

/** Angle between two sky directions (azimuth, elevation), in degrees. */
export function separationDeg(az1, el1, az2, el2) {
  const c = Math.sin(rad(el1)) * Math.sin(rad(el2)) + Math.cos(rad(el1)) * Math.cos(rad(el2)) * Math.cos(rad(az1 - az2));
  return deg(Math.acos(clamp(c, -1, 1)));
}

/** Antenna gain lost by pointing `offset` degrees off the satellite. beam: half-power beamwidth. */
export function pointingLossDb(offsetDeg, beamDeg) {
  return Math.min(25, 12 * (offsetDeg / beamDeg) ** 2);
}

/** Where the satellite is at progress p (0 = rises, 1 = sets) in the simplified sky view. */
export function satPosition(level, p) {
  const q = 0.03 + 0.94 * clamp(p, 0, 1);
  // Like a real pass, the satellite lingers low near the horizon and crosses the top quickly.
  return { az: -85 + 170 * q, el: level.maxEl * Math.sin(Math.PI * q) ** 1.8 };
}

/** The noisy neighbor sits low on the horizon at this azimuth. */
export const NEIGHBOR = { az: 52, el: 3 };

/** Noise bursts for a pass, from its seed: [{ start, end }] in seconds. */
export function burstsFor(level, seed) {
  const r = rng(seed ^ 0x9e3779b9);
  const out = [];
  for (let i = 0; i < level.bursts; i++) {
    const slot = level.duration / (level.bursts + 1);
    const start = slot * (i + 0.6 + r() * 0.6);
    out.push({ start, end: start + 2 + r() * 2 });
  }
  return out;
}

/** Signal-to-noise ratio in dB for an antenna pointing at (az, el), at time t. */
export function snrAt(game, az, el, t = game.t) {
  const { level } = game;
  const sat = satPosition(level, t / level.duration);
  const off = separationDeg(az, el, sat.az, sat.el);
  let noiseDb = 0;
  if (level.interference) {
    // Noise from the neighbor, received through the same antenna pattern.
    const toNeighbor = separationDeg(az, el, NEIGHBOR.az, NEIGHBOR.el);
    noiseDb = level.interference - pointingLossDb(toNeighbor, level.beam);
  }
  if (game.bursts.some(b => t >= b.start && t < b.end)) noiseDb = Math.max(noiseDb, 0) + 10;
  // Noise above the receiver floor adds to it in power: N = 10 log10(1 + 10^(I/10)).
  const noise = noiseDb > -30 ? 10 * Math.log10(1 + 10 ** (noiseDb / 10)) : 0;
  return level.link - pointingLossDb(off, level.beam) - rangeLossDb(sat.el) - noise + game.fade;
}

export function inEclipse(level, p) {
  return !!level.eclipse && p >= level.eclipse[0] && p < level.eclipse[1];
}

export function createGame(level, seed = 1) {
  return {
    level, seed, random: rng(seed), t: 0, done: false,
    ant: { az: -60, el: 20 }, sf: 9, transmit: true,
    battery: level.battery, safe: false, fade: 0,
    bursts: burstsFor(level, seed),
    packet: { elapsed: 0, sum: 0, n: 0, min: Infinity },
    seq: 0, rx: 0, lost: 0, streak: 0, bestStreak: 0, score: 0,
    snr: -99, locked: false, events: [], log: [],
  };
}

/** Commands the satellite to another spreading factor. The packet on air is abandoned. */
export function setSf(game, sf) {
  if (!SF[sf] || sf === game.sf) return;
  game.sf = sf;
  game.packet = { elapsed: 0, sum: 0, n: 0, min: Infinity };
  game.events.push({ type: 'sf', sf });
}

/** Tells the satellite to send packets (on) or hold its transmitter to save battery (off). */
export function setTransmit(game, on) {
  if (game.transmit === !!on) return;
  game.transmit = !!on;
  game.packet = { elapsed: 0, sum: 0, n: 0, min: Infinity };
  game.events.push({ type: on ? 'tx-on' : 'tx-off' });
}

/**
 * Advances the game by dt seconds.
 * input: { target?: { az, el } } to slew toward a point, or { rate?: { az, el } } in -1..1.
 * Events for this step (rx, lost, safe, resume, aos, los, done) are left in game.events.
 */
export function step(game, input = {}, dt = 1 / 60) {
  if (game.done) return game;
  const { level } = game;
  dt = clamp(dt, 0, 0.1);
  game.t += dt;
  const p = game.t / level.duration;

  // The rotator turns at a limited rate, so a fast pass is hard to follow.
  const maxMove = level.slew * dt;
  let dAz = 0, dEl = 0;
  if (input.target) {
    dAz = input.target.az - game.ant.az;
    dEl = input.target.el - game.ant.el;
  } else if (input.rate) {
    dAz = clamp(input.rate.az, -1, 1) * maxMove;
    dEl = clamp(input.rate.el, -1, 1) * maxMove;
  }
  const len = Math.hypot(dAz, dEl);
  if (len > maxMove) { dAz *= maxMove / len; dEl *= maxMove / len; }
  game.ant.az = clamp(game.ant.az + dAz, -90, 90);
  game.ant.el = clamp(game.ant.el + dEl, 0, 90);

  // Slow fading of about +-1 dB, as the satellite tumbles.
  game.fade = clamp(game.fade * 0.97 + (game.random() - 0.5) * 0.35, -1.5, 1.5);
  game.snr = snrAt(game, game.ant.az, game.ant.el);
  const sf = SF[game.sf];
  game.locked = game.snr >= sf.minSnr;

  // Power: solar panels charge in sunlight. Transmitting costs energy whether or not anyone hears it.
  if (!inEclipse(level, p)) game.battery = Math.min(100, game.battery + level.charge * dt);
  if (!game.safe && game.battery < SAFE_AT) { game.safe = true; game.packet = { elapsed: 0, sum: 0, n: 0, min: Infinity }; game.events.push({ type: 'safe' }); }
  if (game.safe && game.battery >= RESUME_AT) { game.safe = false; game.events.push({ type: 'resume' }); }

  if (!game.safe && game.transmit) {
    const pk = game.packet;
    pk.elapsed += dt; pk.sum += game.snr * dt; pk.n += dt; pk.min = Math.min(pk.min, game.snr);
    game.battery = Math.max(0, game.battery - level.drain * dt);
    if (pk.elapsed >= sf.airtime) {
      game.seq++;
      // A packet decodes if its average SNR clears the floor and it never dips far below it.
      const ok = pk.sum / pk.n >= sf.minSnr && pk.min >= sf.minSnr - 3;
      if (ok) {
        game.rx++; game.streak++; game.bestStreak = Math.max(game.bestStreak, game.streak);
        const points = Math.round(100 * (1 + Math.min(game.streak - 1, 10) * 0.1));
        game.score += points;
        const reading = telemetry(game, p);
        game.log.unshift(reading);
        if (game.log.length > 8) game.log.length = 8;
        game.events.push({ type: 'rx', seq: game.seq, points, reading });
      } else {
        game.lost++; game.streak = 0;
        game.events.push({ type: 'lost', seq: game.seq });
      }
      game.packet = { elapsed: 0, sum: 0, n: 0, min: Infinity };
    }
  }

  if (p >= 1) {
    game.done = true;
    if (game.battery >= 20) game.score += 200;
    game.events.push({ type: 'done', stars: stars(game), score: game.score });
  }
  return game;
}

/** What the satellite says in a packet: the same sensors as the classroom simulator. */
export function telemetry(game, p) {
  const shade = inEclipse(game.level, p);
  return {
    seq: game.seq,
    sf: game.sf,
    snr: Math.round(game.snr * 10) / 10,
    tempC: Math.round((shade ? -8 : 21) + (game.random() - 0.5) * 6),
    light: shade ? 0 : Math.round(600 + game.random() * 400),
    battery: Math.round(game.battery),
  };
}

export function stars(game) {
  return game.level.goal.filter(g => game.rx >= g).length;
}

/** Which live pass is on now, and how far into it we are, from a shared clock. */
export function liveSlot(nowMs) {
  const s = nowMs / 1000;
  const slot = Math.floor(s / LIVE_SLOT_SECONDS);
  return { slot, elapsed: s - slot * LIVE_SLOT_SECONDS, seed: (slot * 2654435761) >>> 0 };
}
