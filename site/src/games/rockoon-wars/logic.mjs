// Rockoon Wars: the rules, with no DOM. A turn-based artillery game in the style of the
// classic DOS Tank Wars, with rockoons (balloon-lifted rocket platforms) in place of tanks.
//
// Everything here is deterministic, so every online player can simulate each shot from the
// same seed and the same inputs and get exactly the same result. Only + - * / and
// Math.floor/round/abs/min/max are used in the simulation: Math.sin, Math.sqrt and friends may
// differ in the last bit between browsers, so sine comes from a fixed polynomial below.

export const W = 640;
export const H = 400;
export const GRAVITY = 150; // px/s^2
export const SPEED = 3.4; // px/s per point of power
export const WIND_ACCEL = 6; // px/s^2 per point of wind
export const DT = 1 / 120;
export const MAX_STEPS = 120 * 30;
export const UNIT_R = 10;
export const MAX_PLAYERS = 4;
// Everyone starts with enough credits for a couple of better rockets (a Rocket pack and a Big Bang).
export const START_CREDITS = 600;

/** Weapons, in shop order. The Baby Rocket is free and never runs out. */
export const WEAPONS = [
  { id: 'baby', name: 'Baby Rocket', radius: 12, damage: 30, price: 0, pack: 0 },
  { id: 'rocket', name: 'Rocket', radius: 20, damage: 45, price: 150, pack: 5 },
  { id: 'big', name: 'Big Bang', radius: 38, damage: 70, price: 400, pack: 2 },
  { id: 'mega', name: 'Mega Bang', radius: 64, damage: 100, price: 900, pack: 1 },
  { id: 'dirt', name: 'Dirt Ball', radius: 34, damage: 0, price: 200, pack: 3, dirt: true },
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
    field: null, winner: null, log: [],
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

/** Starts the next round: new hills, new wind, everyone back to full health. */
export function startRound(match) {
  if (match.phase !== 'shop') return match;
  match.round++;
  const r = rng(match.seed + match.round * 7919);
  const terrain = makeTerrain(match.seed ^ (match.round * 2654435761));
  const n = match.players.length, slot = W / n;
  // Shuffle who starts where, then spread the rockoons out with a little jitter.
  const order = match.players.map((_, i) => i);
  for (let i = n - 1; i > 0; i--) { const j = Math.floor(r() * (i + 1)); [order[i], order[j]] = [order[j], order[i]]; }
  const units = match.players.map(() => null);
  order.forEach((pi, k) => {
    const x = Math.round(slot * k + slot * (0.3 + r() * 0.4));
    // Flatten a small pad for the platform.
    const y = terrain[x];
    for (let dx = -8; dx <= 8; dx++) if (x + dx >= 0 && x + dx < W) terrain[x + dx] = y;
    units[pi] = { x, y, hp: 100, alive: true, angle: x < W / 2 ? 60 : 120, power: 55, weapon: 'baby' };
  });
  match.field = { terrain, units, wind: Math.round((r() - 0.5) * 20) };
  match.phase = 'play';
  match.turn = (match.round - 1) % n;
  while (!units[match.turn].alive) match.turn = (match.turn + 1) % n;
  return match;
}

export const alive = match => match.field.units.filter(u => u.alive).length;

/** The first part of the current player's shot, for the aim line: a list of [x, y] points. */
export function aimPreview(match, points = 30) {
  const u = match.field.units[match.turn];
  return trajectory(match, u, u.angle, u.power).path.slice(0, points);
}

/** Flies a shot and returns the path (for animation) and where it ended. Does not change the match. */
export function trajectory(match, from, angle, power) {
  const { terrain, units, wind } = match.field;
  const a = Math.max(0, Math.min(180, Math.round(angle)));
  const v = Math.max(0, Math.min(100, Math.round(power))) * SPEED;
  let x = from.x + cosDeg(a) * 14, y = from.y + 22 + sinDeg(a) * 14;
  let vx = cosDeg(a) * v, vy = sinDeg(a) * v;
  const path = [[x, y]];
  for (let i = 0; i < MAX_STEPS; i++) {
    vx += wind * WIND_ACCEL * DT; vy -= GRAVITY * DT;
    x += vx * DT; y += vy * DT;
    if (i % 2 === 0) path.push([x, y]);
    if (x < 0 || x >= W) return { path, end: null };
    const col = Math.floor(x);
    if (y <= terrain[col]) return { path, end: { x, y: Math.max(y, 0) } };
    for (const u of units) {
      if (!u.alive || u === from) continue;
      const dx = x - u.x, dy = y - (u.y + 8);
      if (dx * dx + dy * dy <= UNIT_R * UNIT_R) return { path, end: { x, y } };
    }
    // Leaving through the top is fine; the shot comes back down. Below the field it is gone.
    if (y < -50) return { path, end: null };
  }
  return { path, end: null };
}

/** Removes (or for dirt, adds) a circle of ground. The ground has no caves: dirt above a hole falls in. */
export function carve(terrain, cx, cy, r, dirt = false) {
  for (let x = Math.max(0, Math.ceil(cx - r)); x <= Math.min(W - 1, Math.floor(cx + r)); x++) {
    const dx = x - cx, span2 = r * r - dx * dx;
    if (span2 < 0) continue;
    // Half-height of the circle at this column, by Newton's method (no Math.sqrt).
    let s = r;
    for (let k = 0; k < 8; k++) s = (s + span2 / s) / 2;
    const bottom = cy - s, top = cy + s;
    // Dirt fills the part of the circle that is above the ground; it all settles onto it.
    if (dirt) terrain[x] = Math.min(H - 20, Math.round(terrain[x] + Math.max(0, top - Math.max(bottom, terrain[x]))));
    else terrain[x] = Math.max(0, Math.round(terrain[x] - Math.max(0, Math.min(terrain[x], top) - Math.max(0, bottom))));
  }
}

/**
 * Fires the current player's shot: { angle, power, weapon }. Applies damage, falls, kills,
 * credits and the next turn. Returns { path, blast, events } for the screen.
 */
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
  const { path, end } = trajectory(match, unit, unit.angle, unit.power);
  const events = [];
  let blast = null;
  if (end) {
    blast = { x: end.x, y: end.y, r: w.radius, dirt: !!w.dirt };
    carve(match.field.terrain, end.x, end.y, w.radius, !!w.dirt);
    if (w.damage) {
      const reach = w.radius + UNIT_R;
      match.field.units.forEach((u, i) => {
        if (!u.alive) return;
        const dx = u.x - end.x, dy = u.y + 8 - end.y, d2 = dx * dx + dy * dy;
        if (d2 >= reach * reach) return;
        const dmg = Math.round(w.damage * (1 - d2 / (reach * reach)));
        if (dmg > 0) hurt(match, i, dmg, shooter, events);
      });
    }
  }
  // Rockoons drop onto whatever ground is left under them.
  match.field.units.forEach((u, i) => {
    if (!u.alive) return;
    let ground = 0;
    for (let dx = -6; dx <= 6; dx++) ground = Math.max(ground, match.field.terrain[Math.max(0, Math.min(W - 1, u.x + dx))]);
    if (ground < u.y) {
      const drop = u.y - ground;
      u.y = ground;
      if (drop > 12) hurt(match, i, Math.floor(drop / 4), shooter, events);
    }
    if (u.alive && u.y <= 2) hurt(match, i, u.hp, shooter, events);
  });
  match.log.push({ round: match.round, player: shooter, angle: unit.angle, power: unit.power, weapon });
  endTurn(match, events);
  return { path, blast, events, weapon };
}

function hurt(match, i, dmg, by, events) {
  const u = match.field.units[i];
  if (!u.alive) return;
  const real = Math.min(u.hp, dmg);
  u.hp -= real;
  events.push({ type: 'hit', who: i, dmg: real, by });
  if (by !== i) match.players[by].credits += real * 5;
  if (u.hp <= 0) {
    u.alive = false;
    events.push({ type: 'kill', who: i, by });
    if (by !== i) { match.players[by].credits += 200; match.players[by].score += 1; }
  }
}

function endTurn(match, events) {
  const n = match.players.length;
  if (alive(match) <= 1) {
    const last = match.field.units.findIndex(u => u.alive);
    if (last >= 0) { match.players[last].score += 2; match.players[last].credits += 300; }
    for (const p of match.players) p.credits += 100;
    events.push({ type: 'round', winner: last });
    if (match.round >= match.rounds) {
      match.phase = 'over';
      const top = Math.max(...match.players.map(p => p.score));
      match.winner = match.players.findIndex(p => p.score === top);
      events.push({ type: 'over', winner: match.winner });
    } else match.phase = 'shop';
    return;
  }
  do match.turn = (match.turn + 1) % n; while (!match.field.units[match.turn].alive);
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
    for (const u of match.field.units) { add(u.x); add(u.y); add(u.hp); }
    for (let x = 0; x < W; x += 4) add(match.field.terrain[x]);
  }
  return (h >>> 0).toString(16);
}
