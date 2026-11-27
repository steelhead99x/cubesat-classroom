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

import { amplitudeGain, STEP_KHZ } from '../../lib/pedal.mjs';

export const EARTH_RADIUS_KM = 6371;
export const ALTITUDE_KM = 500;

export const MISSION_SPEED = 1.12;
export const ANTENNAS = {
  crossed: { name: 'Crossed dipole', note: 'Wide beam · lower gain', beam: 52, slew: 82, gain: -3 },
  patch: { name: 'Patch', note: 'Medium beam · balanced gain', beam: 34, slew: 70, gain: 0 },
  yagi: { name: 'Yagi', note: 'Narrow beam · higher gain', beam: 16, slew: 45, gain: 6 },
};
export const DEFAULT_RECEIVER = { engaged: true, offset: 0, gainDb: 18, level: 50 };
export const receiverFrequency = offset => (915000 + offset * STEP_KHZ) / 1000;

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
  { id: 1, name: 'First light', duration: 40, maxEl: 78, antenna: 'patch', link: 0, battery: 90, charge: 1.2, drain: 1, interference: 0, bursts: 0, eclipse: null,
    goal: [10, 20, 30], tip: 'Keep the beam circle on the satellite. Packets only arrive while the signal bar is above the line for your SF.' },
  { id: 2, name: 'Low pass', duration: 44, maxEl: 24, antenna: 'patch', link: -2, battery: 90, charge: 1.2, drain: 1, interference: 0, bursts: 0, eclipse: null,
    goal: [3, 6, 9], tip: 'A low pass is far away, so the signal is weak. Try SF12 or a higher-gain antenna. SF9 may work near the top.' },
  { id: 3, name: 'Noisy neighbor', duration: 42, maxEl: 55, antenna: 'patch', link: -2, battery: 85, charge: 1.2, drain: 1, interference: 14, bursts: 3, eclipse: null,
    goal: [6, 12, 18], tip: 'A busy 915 MHz device sits on the right-hand horizon. Pointing near it, or a burst of noise, drowns the satellite out.' },
  { id: 4, name: 'Into the shadow', duration: 44, maxEl: 62, antenna: 'patch', link: -2, battery: 40, charge: 1.6, drain: 1.8, interference: 0, bursts: 2, eclipse: [0.45, 1],
    goal: [5, 10, 16], tip: 'Low battery, and the satellite enters Earth\u2019s shadow. Hold the transmitter (Space) when you cannot hear it, and use fast SFs.' },
  { id: 5, name: 'Yagi sprint', duration: 34, maxEl: 70, antenna: 'yagi', link: -2, battery: 60, charge: 1.4, drain: 1.5, interference: 12, bursts: 3, eclipse: [0.65, 1],
    goal: [8, 16, 25], tip: 'Try the Yagi for a stronger signal. Its narrow beam takes precise aiming.' },
];

/** The shared live pass: everyone in the live sky flies the same one at the same time. */
export const LIVE = { id: 'live', name: 'Live sky', duration: 45, maxEl: 50, antenna: 'patch', link: -2, battery: 70, charge: 1.3, drain: 1.4, interference: 12, bursts: 2, eclipse: [0.7, 1],
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

/** Beam and rotator properties for the current antenna, with the practice assist applied. */
export function antennaFor(level, choice = level.antenna) {
  const { beam, slew, gain } = ANTENNAS[choice];
  return level.easy ? { beam: beam * 1.3, slew: slew * 1.2, gain } : { beam, slew, gain };
}

function linkAt(game, az, el, t, choice) {
  const { level } = game;
  const { beam, gain } = antennaFor(level, choice);
  const sat = satPosition(level, t / level.duration);
  const off = separationDeg(az, el, sat.az, sat.el);
  let noiseDb = 0;
  if (level.interference) {
    // Noise from the neighbor, received through the same antenna pattern.
    const toNeighbor = separationDeg(az, el, NEIGHBOR.az, NEIGHBOR.el);
    noiseDb = level.interference - pointingLossDb(toNeighbor, beam);
  }
  if (game.bursts.some(b => t >= b.start && t < b.end)) noiseDb = Math.max(noiseDb, 0) + 10;
  const noise = noiseDb > -30 ? 10 * Math.log10(1 + 10 ** (noiseDb / 10)) : 0;
  const pointing = pointingLossDb(off, beam);
  const carrier = level.link + gain - pointing - rangeLossDb(sat.el) + game.fade;
  return { carrier, noise, pointing };
}

/** Why packets were lost, for the debrief after a pass. */
export const LOSS_CAUSES = ['pointing', 'weak', 'noise', 'receiver'];
const freshPacket = () => ({ elapsed: 0, sum: 0, n: 0, min: Infinity, pointing: 0, noise: 0, receiver: 0 });

/**
 * The main reason a packet failed, from what happened while it was on air: the receiver set
 * wrong, the antenna pointed away, noise, or (none of those) a signal too weak for this SF.
 */
export function lossCause(pk) {
  const n = pk.n || 1;
  if (pk.receiver / n > 0.5) return 'receiver';
  if (pk.pointing / n > 3) return 'pointing';
  // The noise term is about 3 dB with nothing nearby, so look for clearly more than that.
  if (pk.noise / n > 4.5) return 'noise';
  return 'weak';
}

/** Signal-to-noise ratio at the antenna, before the receiver pedal. */
export function snrAt(game, az, el, t = game.t, choice = game.antenna) {
  const { carrier, noise } = linkAt(game, az, el, t, choice);
  return carrier - noise;
}

/** Simplified pedal input. Below normal gain the receiver adds noise. Overload clips the ADC. */
export function readReceiver(game) {
  const { engaged, offset, gainDb } = game.receiver;
  if (!engaged) return { peak: 0, snr: -99, status: 'Bypassed' };
  const { carrier, noise } = linkAt(game, game.ant.az, game.ant.el, game.t, game.antenna);
  // Match the pedal trainer's gain curve. The LoRa input scale is calibrated for gameplay.
  const tuned = offset === 0;
  // A satellite holding its transmitter (or in safe mode) sends nothing to lock on to.
  const sending = game.transmit && !game.safe;
  const inputDb = Math.max(tuned && sending ? carrier : -30, noise - 18);
  const peak = 0.42 * amplitudeGain(inputDb + gainDb - DEFAULT_RECEIVER.gainDb);
  const lowGain = Math.max(0, DEFAULT_RECEIVER.gainDb - gainDb) / 3;
  const overload = peak > 1 ? 40 * Math.log10(peak) : 0;
  const status = peak >= 1 ? 'Clipping' : !tuned ? 'Off frequency' : !sending ? 'No signal' : lowGain >= 2 ? 'Low gain' : 'Ready';
  return { peak, snr: tuned && sending ? carrier - noise - lowGain - overload : -99, status };
}

export function inEclipse(level, p) {
  return !!level.eclipse && p >= level.eclipse[0] && p < level.eclipse[1];
}

/** Easy mode (the default): slower passes, a faster antenna and a wider beam. Goals stay the same. */
export function easyLevel(level) {
  return { ...level, duration: level.duration * 1.5, easy: true };
}

export function createGame(level, seed = 1, antenna = level.antenna, receiver = DEFAULT_RECEIVER) {
  return {
    level, seed, antenna, receiver: { ...receiver }, random: rng(seed), t: 0, done: false,
    ant: { az: -60, el: 20 }, sf: 9, transmit: true,
    battery: level.battery, safe: false, fade: 0,
    bursts: burstsFor(level, seed),
    packet: freshPacket(),
    /** For the debrief: lost packets by cause, seconds resting or in safe mode, and a trace. */
    causes: { pointing: 0, weak: 0, noise: 0, receiver: 0 },
    /** Too-weak losses by the spreading factor each packet was sent on, so the debrief names the right one. */
    weakBySf: { 7: 0, 9: 0, 12: 0 },
    restS: 0, safeS: 0,
    /** @type {{ t: number, snr: number, floor: number, el: number, on: boolean }[]} every half second */
    trace: [],
    /** @type {{ t: number, ok: boolean, cause?: string }[]} */
    packets: [],
    seq: 0, rx: 0, lost: 0, streak: 0, bestStreak: 0, score: 0,
    snr: -99, locked: false,
    /** @type {{ type: string, seq?: number, points?: number, sf?: number, stars?: number, score?: number, reading?: ReturnType<typeof telemetry> }[]} */
    events: [],
    /** @type {ReturnType<typeof telemetry>[]} */
    log: [],
  };
}

/** Skip time a live station missed while away, without crediting packets it never received. */
export function syncLiveTime(game, elapsed) {
  const target = Math.min(elapsed, game.level.duration);
  if (game.done || target <= game.t || (target - game.t < 0.75 && target < game.level.duration)) return false;
  game.t = target;
  game.packet = freshPacket();
  game.streak = 0;
  game.locked = false;
  return true;
}

/** Commands the satellite to another spreading factor. The packet on air is abandoned. */
export function setSf(game, sf) {
  if (!SF[sf] || sf === game.sf) return;
  game.sf = sf;
  game.packet = freshPacket();
  game.events.push({ type: 'sf', sf });
}

/** Tells the satellite to send packets (on) or hold its transmitter to save battery (off). */
export function setTransmit(game, on) {
  if (game.transmit === !!on) return;
  game.transmit = !!on;
  game.snr = readReceiver(game).snr;
  game.locked = game.snr >= SF[game.sf].minSnr;
  game.packet = freshPacket();
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
  const maxMove = antennaFor(level, game.antenna).slew * dt;
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
  // Scaled by dt so the fade behaves the same at any frame rate.
  const frames = dt * 60;
  game.fade = clamp(game.fade * 0.97 ** frames + (game.random() - 0.5) * 0.35 * Math.sqrt(frames), -1.5, 1.5);
  // Power: solar panels charge in sunlight. Transmitting costs energy whether or not anyone hears it.
  if (!inEclipse(level, p)) game.battery = Math.min(100, game.battery + level.charge * dt);
  if (!game.safe && game.battery < SAFE_AT) { game.safe = true; game.packet = freshPacket(); game.events.push({ type: 'safe' }); }
  if (game.safe && game.battery >= RESUME_AT) { game.safe = false; game.events.push({ type: 'resume' }); }

  const rx = readReceiver(game);
  game.snr = rx.snr;
  const sf = SF[game.sf];
  game.locked = game.snr >= sf.minSnr;
  if (game.safe) game.safeS += dt;
  else if (!game.transmit) game.restS += dt;
  // A point every half second for the debrief chart (at most about 140 for the longest pass).
  if (game.t >= game.trace.length * 0.5) {
    game.trace.push({ t: game.t, snr: Math.max(-40, game.snr), floor: sf.minSnr, el: satPosition(level, p).el, on: game.transmit && !game.safe });
  }

  if (!game.safe && game.transmit) {
    const pk = game.packet;
    pk.elapsed += dt; pk.sum += game.snr * dt; pk.n += dt; pk.min = Math.min(pk.min, game.snr);
    const link = linkAt(game, game.ant.az, game.ant.el, game.t, game.antenna);
    pk.pointing += link.pointing * dt;
    pk.noise += link.noise * dt;
    if (rx.status !== 'Ready') pk.receiver += dt;
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
        if (game.packets.length < 200) game.packets.push({ t: game.t, ok: true });
      } else {
        game.lost++; game.streak = 0;
        const cause = lossCause(pk);
        game.causes[cause]++;
        if (cause === 'weak') game.weakBySf[game.sf]++;
        game.events.push({ type: 'lost', seq: game.seq, cause });
        if (game.packets.length < 200) game.packets.push({ t: game.t, ok: false, cause });
      }
      game.packet = freshPacket();
    }
  }

  if (p >= 1) {
    game.done = true;
    // The healthy-battery bonus is for flying the pass: a station that sent nothing (for
    // example, one that joined a live pass after it ended) earns nothing for it.
    if (game.battery >= 20 && game.seq > 0) game.score += 200;
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
  // Math.imul keeps the hash in 32-bit integers; a plain product passes 2^53 and loses bits.
  return { slot, elapsed: s - slot * LIVE_SLOT_SECONDS, seed: Math.imul(slot, 2654435761) >>> 0 };
}

/**
 * What to try next time, from how the pass went: the biggest reason packets were lost, or
 * time the satellite could not send. Returns { cause, text }, or null after a clean pass.
 */
export function debrief(game) {
  // The setting the weak packets were sent on, not the one selected now.
  const weakSf = Object.entries(game.weakBySf ?? {}).sort((a, b) => b[1] - a[1] || Number(b[0]) - Number(a[0]))[0];
  const sentSf = weakSf && weakSf[1] > 0 ? Number(weakSf[0]) : game.sf;
  const sfName = { 7: 'Fast', 9: 'Middle', 12: 'Far' }[sentSf];
  const top = LOSS_CAUSES.map(c => [c, game.causes[c]]).sort((a, b) => b[1] - a[1])[0];
  const plural = n => (n === 1 ? '1 packet was' : `${n} packets were`);
  if (game.safeS >= 3 && (!top || game.safeS * 0.7 > top[1])) {
    return { cause: 'battery', text: `The battery ran flat, so the satellite sent nothing for ${Math.round(game.safeS)} s. In Earth\u2019s shadow, rest it (Sending: resting) when you cannot hear it, and use faster settings.` };
  }
  if (!top || top[1] === 0) {
    return game.restS >= 5
      ? { cause: 'rest', text: `The satellite rested for ${Math.round(game.restS)} s. Resting saves battery, but it sends nothing while it rests.` }
      : null;
  }
  const [cause, n] = top;
  const text = {
    pointing: `${plural(n)} lost while the antenna pointed away. Keep the circle on the satellite, and move ahead of it when it is high and fast.`,
    weak: sentSf === 12
      ? `${plural(n)} too weak to decode even at Far (SF12) with the antenna on target. Very low in the sky the signal is too faint for any setting; rest the satellite then to save battery.`
      : `${plural(n)} too weak to decode at ${sfName} even with the antenna on target. When the satellite is low and far away, switch to Far (SF12).`,
    noise: `${plural(n)} drowned out by noise. Keep the antenna away from the busy device on the right horizon, and expect short noise bursts.`,
    receiver: `${plural(n)} lost at the receiver. Keep the pedal engaged, tuned to 915.000 MHz, and the gain out of the red.`,
  }[cause];
  return { cause, text };
}
