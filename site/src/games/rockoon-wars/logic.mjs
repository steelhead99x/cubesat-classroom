// Rockoon Battle: the rules, with no DOM. Each crew's balloon lifts its rocket off a hillside
// pad, drifts with the wind, and releases the rocket to fly through a shared sky ring.
//
// Everything here is deterministic, so every online player can simulate each launch from the
// same seed and the same inputs and get exactly the same result. Only + - * / and
// Math.floor/round/abs/min/max are used in the simulation (no ** or Math.pow either): Math.sin,
// Math.sqrt and friends may differ in the last bit between browsers, so sine comes from a fixed
// polynomial below. pressureHpa is display only and never feeds back into a launch.
export const W = 640;
export const H = 400;
export const GRAVITY = 150; // px/s^2
export const SPEED = 3.4; // px/s per point of power
export const WIND_ACCEL = 6; // px/s^2 per point of wind
export const DT = 1 / 120;
export const MAX_STEPS = 120 * 30;
export const LIFT = 48;
// Map the vertical scenery to an illustrative altitude for the balloon barometer.
export const METRES_PER_PIXEL = 10;

// Standard troposphere (0–11 km), pressure in hectopascals.
export function pressureHpa(altitudeMetres) {
  return 1013.25 * (1 - 0.0065 * altitudeMetres / 288.15) ** 5.25588;
}

// Arcade drift in pixels. The animation and launch origin use the same ascent.
export function balloonPosition(pad, wind, progress) {
  return { x: pad.x + wind * 1.5 * progress, y: pad.y + LIFT * progress };
}

/** Where a crew's balloon releases its rocket this round (the launch tube's base). */
export const releasePoint = (pad, wind) => balloonPosition(pad, wind, 1);

export const TARGET_RADIUS = 16;
/** The ring keeps this far from every crew's release point whenever the field has room. */
export const RING_CLEARANCE = 80;
export const LAUNCHES = 3;
export const MAX_PLAYERS = 4;
// Everyone starts with enough credits for a couple of better rockets (a Tracker pack and a
// Surveyor pack).
export const START_CREDITS = 600;

/**
 * Rockets, in shop order, from the narrowest scan to the widest. The Scout is free and never
 * runs out; each step up costs more per rocket, so a wider scan is always a trade.
 * (The ids are the old ones, kept so rooms and tests still line up.)
 */
export const WEAPONS = [
  { id: 'baby', name: 'Scout', radius: 12, price: 0, pack: 0 },
  { id: 'rocket', name: 'Tracker', radius: 20, price: 150, pack: 5 },
  { id: 'dirt', name: 'Mapper', radius: 28, price: 150, pack: 3 },
  { id: 'big', name: 'Surveyor', radius: 38, price: 250, pack: 2 },
  { id: 'mega', name: 'Observatory', radius: 50, price: 300, pack: 1 },
];
export const WEAPON = Object.fromEntries(WEAPONS.map(w => [w.id, w]));
export const COLORS = ['#ef6b6c', '#6fd3a0', '#ffd23f', '#b9a4ff'];

/** Small deterministic PRNG (mulberry32). */
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

const PI = 3.141592653589793;
/** sin(x) for x in [-PI/2, PI/2] from a fixed Taylor polynomial, identical in every browser. */
function sinPoly(x) {
  const x2 = x * x;
  return x * (1 - x2 / 6 * (1 - x2 / 20 * (1 - x2 / 42 * (1 - x2 / 72 * (1 - x2 / 110 * (1 - x2 / 156 * (1 - x2 / 210)))))));
}
/** sin and cos of a whole number of degrees (0 = right, 90 = up, 180 = left). */
export function sinDeg(deg) {
  let d = ((Math.round(deg) % 360) + 360) % 360;
  if (d > 180) d -= 360;
  // Fold into -90..90, where the polynomial is accurate to better than 1e-12.
  if (d > 90) d = 180 - d; else if (d < -90) d = -180 - d;
  return sinPoly(d * PI / 180);
}
export const cosDeg = deg => sinDeg(deg + 90);

/** Rolling hills from a seed: midpoint displacement, then smoothed. Heights are from the bottom. */
export function makeTerrain(seed) {
  const r = rng(seed);
  const n = 64, pts = new Array(n + 1);
  pts[0] = 90 + r() * 160; pts[n] = 90 + r() * 160;
  let step = n, rough = 150;
  while (step > 1) {
    const half = step / 2;
    for (let i = half; i < n; i += step) pts[i] = (pts[i - half] + pts[i + half]) / 2 + (r() - 0.5) * rough;
    step = half; rough *= 0.55;
  }
  const h = new Array(W);
  for (let x = 0; x < W; x++) {
    const f = x / (W - 1) * n, i = Math.min(n - 1, Math.floor(f)), t = f - i;
    const s = t * t * (3 - 2 * t);
    h[x] = Math.round(Math.max(40, Math.min(300, pts[i] + (pts[i + 1] - pts[i]) * s)));
  }
  return h;
}

/** A new match: players [{ id, name }], rounds to play. */
export function createMatch({ seed, players, rounds = 3 }) {
  if (players.length < 2 || players.length > MAX_PLAYERS) throw new Error('2 to 4 players');
  return {
    seed: seed >>> 0, rounds, round: 0, phase: 'shop', turn: 0,
    players: players.map((p, i) => ({
      id: p.id, name: p.name, color: COLORS[i], credits: START_CREDITS, score: 0,
      ammo: { baby: Infinity, rocket: 0, big: 0, mega: 0, dirt: 0 },
    })),
    field: null, winner: null, winners: /** @type {number[]} */ ([]), log: [],
  };
}

/** Buys one pack of a weapon for a player, if they can afford it. Returns true if bought. */
export function buy(match, playerIndex, weaponId) {
  const p = match.players[playerIndex], w = WEAPON[weaponId];
  if (!p || !w || !w.price || match.phase !== 'shop' || p.credits < w.price) return false;
  p.credits -= w.price;
  p.ammo[weaponId] += w.pack;
  return true;
}

/** Starts the next round: new hills, wind and a shared sky ring. */
export function startRound(match) {
  if (match.phase !== 'shop') return match;
  match.round++;
  const r = rng(match.seed + match.round * 7919);
  const terrain = makeTerrain(match.seed ^ (match.round * 2654435761));
  const n = match.players.length, slot = W / n;
  // Who starts where: shuffled once per match, then moved along one slot each round, so over n
  // rounds every crew launches once from every slot (no crew keeps the spot nearest the ring).
  // Then the rockoons are spread out with a little jitter.
  const start = rng(match.seed ^ 0x5bd1e995);
  const first = match.players.map((_, i) => i);
  for (let i = n - 1; i > 0; i--) { const j = Math.floor(start() * (i + 1)); [first[i], first[j]] = [first[j], first[i]]; }
  const order = first.map((_, k) => first[(k + match.round - 1) % n]);
  const units = match.players.map(() => null);
  order.forEach((pi, k) => {
    const x = Math.round(slot * k + slot * (0.3 + r() * 0.4));
    // Flatten a small pad for the platform.
    const y = terrain[x];
    for (let dx = -8; dx <= 8; dx++) if (x + dx >= 0 && x + dx < W) terrain[x + dx] = y;
    units[pi] = { x, y, angle: x < W / 2 ? 60 : 120, power: 55, weapon: 'baby' };
  });
  const wind = Math.round((r() - 0.5) * 20);
  match.field = { terrain, units, target: placeRing(terrain, units, wind, r), launches: 0, wind };
  match.phase = 'play';
  match.turn = (match.round - 1) % n;
  return match;
}

/**
 * The sky ring, shared by every crew: above the hills, and at least RING_CLEARANCE from every
 * crew's release point (so nobody scores with a gentle launch straight up), in the middle half
 * of the field when there is room there, and as even as it can be: among those spots, the ones
 * where the nearest and farthest crews are closest in sideways distance (within 8 px of the
 * best). If no spot is that clear, the one farthest from every release point. Squared
 * distances only (see the note at the top).
 */
function placeRing(terrain, units, wind, r) {
  const releases = units.map(u => { const p = releasePoint(u, wind); return { x: p.x, y: p.y + PIVOT_Y }; });
  let top = 0;
  for (const p of releases) top = Math.max(top, p.y);
  const y = Math.min(H - TARGET_RADIUS - 4, Math.max(Math.max(...terrain) + LIFT + 36, top + 40));
  const room = x => Math.min(...releases.map(p => (x - p.x) * (x - p.x) + (y - p.y) * (y - p.y)));
  const spots = [];
  for (let x = 40; x <= W - 40; x++) spots.push(x);
  const far = RING_CLEARANCE * RING_CLEARANCE;
  const middle = spots.filter(x => x >= W / 4 && x <= W * 3 / 4 && room(x) >= far);
  const clear = middle.length ? middle : spots.filter(x => room(x) >= far);
  const spread = x => {
    let lo = Infinity, hi = 0;
    for (const p of releases) { const d = Math.abs(x - p.x); lo = Math.min(lo, d); hi = Math.max(hi, d); }
    return hi - lo;
  };
  let best = Infinity;
  for (const x of clear) best = Math.min(best, spread(x));
  const even = clear.filter(x => spread(x) <= best + 8);
  const x = even.length ? even[Math.floor(r() * even.length)] : spots.reduce((a, b) => (room(b) > room(a) ? b : a));
  return { x, y };
}

/** Where shots leave the launcher from (the pivot of the launch tube). */
export const PIVOT_Y = 22;
/** World distance that means full power when dragging to aim. */
export const DRAG_FULL = 160;

/**
 * Drag-to-aim: the angle and power for pointing from where the balloon releases the rocket
 * (the pad carried up and across by `wind`) at a world point. This is input only (it may use
 * Math.atan2); the shot itself uses the whole-number result.
 */
export function aimFromPoint(unit, x, y, wind = 0) {
  const from = releasePoint(unit, wind);
  const dx = x - from.x, dy = y - (from.y + PIVOT_Y);
  let angle = Math.round(Math.atan2(dy, dx) * 180 / Math.PI);
  if (angle < 0) angle = dx >= 0 ? 0 : 180;
  const power = Math.round(Math.min(100, Math.max(0, Math.hypot(dx, dy) / DRAG_FULL * 100)));
  return { angle: Math.max(0, Math.min(180, angle)), power };
}

/** The first part of the current player's shot, for the aim line: a list of [x, y] points. */
export function aimPreview(match, points = 30) {
  const u = match.field.units[match.turn];
  return trajectory(match, u, u.angle, u.power).path.slice(0, points);
}

/** Flies a shot and returns the path (for animation) and where it ended. Does not change the match. */
export function trajectory(match, from, angle, power) {
  const { terrain, wind } = match.field;
  const a = Math.max(0, Math.min(180, Math.round(angle)));
  const v = Math.max(0, Math.min(100, Math.round(power))) * SPEED;
  const release = releasePoint(from, wind);
  let x = release.x + cosDeg(a) * 14, y = release.y + PIVOT_Y + sinDeg(a) * 14;
  let vx = cosDeg(a) * v, vy = sinDeg(a) * v;
  const path = [[x, y]];
  for (let i = 0; i < MAX_STEPS; i++) {
    vx += wind * WIND_ACCEL * DT; vy -= GRAVITY * DT;
    x += vx * DT; y += vy * DT;
    if (i % 2 === 0) path.push([x, y]);
    if (x < 0 || x >= W) return { path, end: null };
    const col = Math.floor(x);
    if (y <= terrain[col]) return { path, end: { x, y: Math.max(y, 0) } };
    // Leaving through the top is fine; the shot comes back down. Below the field it is gone.
    if (y < -50) return { path, end: null };
  }
  return { path, end: null };
}

export function fire(match, shot) {
  if (match.phase !== 'play') throw new Error('not in play');
  const shooter = match.turn, me = match.players[shooter], unit = match.field.units[shooter];
  let weapon = WEAPON[shot.weapon] ? shot.weapon : 'baby';
  if (!(me.ammo[weapon] > 0)) weapon = 'baby';
  if (weapon !== 'baby') me.ammo[weapon]--;
  unit.angle = Math.max(0, Math.min(180, Math.round(shot.angle)));
  unit.power = Math.max(0, Math.min(100, Math.round(shot.power)));
  unit.weapon = weapon;
  const w = WEAPON[weapon];
  const { path } = trajectory(match, unit, unit.angle, unit.power);
  const target = match.field.target;
  let closest = Infinity;
  for (const [x, y] of path) {
    const dx = x - target.x, dy = y - target.y;
    closest = Math.min(closest, dx * dx + dy * dy);
  }
  const outer = TARGET_RADIUS + w.radius;
  const points = closest <= TARGET_RADIUS * TARGET_RADIUS ? 3 : closest <= outer * outer ? 1 : 0;
  me.score += points;
  me.credits += points * 50;
  const events = [{ type: 'scan', points }];
  const blast = points ? { ...target, r: TARGET_RADIUS, dirt: false } : null;
  match.log.push({ round: match.round, player: shooter, angle: unit.angle, power: unit.power, weapon });
  endTurn(match, events);
  return { path, blast, events, weapon };
}

function endTurn(match, events) {
  const n = match.players.length;
  match.field.launches++;
  if (match.field.launches === n * LAUNCHES) {
    for (const p of match.players) p.credits += 100;
    events.push({ type: 'round' });
    if (match.round >= match.rounds) {
      match.phase = 'over';
      const top = Math.max(...match.players.map(p => p.score));
      // Everyone on the top score shares the win; `winner` is the first of them.
      match.winners = match.players.flatMap((p, i) => (p.score === top ? [i] : []));
      match.winner = match.winners[0];
      events.push({ type: 'over', winner: match.winner, winners: match.winners });
    } else match.phase = 'shop';
    return;
  }
  match.turn = (match.turn + 1) % n;
}

/** Skips the current player's turn (used online when a player has left). */
export function skip(match) {
  if (match.phase !== 'play') return;
  match.log.push({ round: match.round, player: match.turn, skip: true });
  endTurn(match, []);
}

/** A short fingerprint of the match, to check that online players stay in step. */
export function fingerprint(match) {
  let h = 2166136261;
  const add = v => { h ^= Math.round(v) & 0xffff; h = Math.imul(h, 16777619) >>> 0; };
  add(match.round); add(match.turn);
  for (const p of match.players) { add(p.credits); add(p.score); }
  if (match.field) {
    add(match.field.launches); add(match.field.target.x); add(match.field.target.y);
    for (const u of match.field.units) { add(u.x); add(u.y); }
    for (let x = 0; x < W; x += 4) add(match.field.terrain[x]);
  }
  return (h >>> 0).toString(16);
}
