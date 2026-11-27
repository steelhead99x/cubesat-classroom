// Rockoon Wars online: how a room's published snapshots become one shared match.
//
// Every player publishes one snapshot over MoQ and keeps republishing it:
//   { cs, joined, creator, host?, turns: { [turnNumber]: { angle, power, weapon } },
//     shops: { [round]: [weaponId, ...] }, skips?: [turnNumber], shopSkips?: { [round]: [id] } }
// The room creator adds `host` when the match starts: the seed, the player order and the rounds.
// Every client replays the same inputs through logic.mjs in the same order, so all screens agree.
// Only the host may skip a player who has left (skips, shopSkips), so skips are agreed too.
import { WEAPON, createMatch, buy, startRound, fire, skip } from './logic.mjs';

const ID = /^[0-9a-f]{12}$/;
const CS = /^GS-[A-Z]{2}\d$/;
export const ROOM_CODE = /^[A-HJ-NP-Z2-9]{5}$/;
const int = (v, lo, hi) => (Number.isInteger(v) && v >= lo && v <= hi ? v : null);

export function newRoomCode(random = Math.random) {
  const abc = 'ABCDEFGHJKLMNPQRSTUVWXYZ23456789';
  return Array.from({ length: 5 }, () => abc[Math.floor(random() * abc.length)]).join('');
}

/** Checks a snapshot from the relay. Anything malformed is dropped or trimmed. */
export function cleanSnapshot(raw) {
  if (!raw || typeof raw !== 'object') return null;
  if (typeof raw.cs !== 'string' || !CS.test(raw.cs)) return null;
  const joined = int(raw.joined, 0, 1e14);
  if (joined === null) return null;
  const out = { cs: raw.cs, joined, creator: raw.creator === true, turns: {}, shops: {} };
  if (raw.turns && typeof raw.turns === 'object') {
    for (const [k, t] of Object.entries(raw.turns).slice(0, 2000)) {
      const n = int(Number(k), 0, 9999);
      if (n === null || !t) continue;
      const angle = int(t.angle, 0, 180), power = int(t.power, 0, 100);
      if (angle === null || power === null || !WEAPON[t.weapon]) continue;
      out.turns[n] = { angle, power, weapon: t.weapon };
    }
  }
  if (raw.shops && typeof raw.shops === 'object') {
    for (const [k, list] of Object.entries(raw.shops).slice(0, 20)) {
      const r = int(Number(k), 1, 20);
      if (r === null || !Array.isArray(list)) continue;
      out.shops[r] = list.filter(w => typeof w === 'string' && WEAPON[w]).slice(0, 40);
    }
  }
  if (raw.host && typeof raw.host === 'object') {
    const h = raw.host;
    const seed = int(h.seed, 0, 4294967295), rounds = int(h.rounds, 1, 10);
    const order = Array.isArray(h.order) ? h.order.filter(id => typeof id === 'string' && ID.test(id)) : [];
    const names = {};
    for (const id of order) if (h.names && typeof h.names[id] === 'string' && CS.test(h.names[id])) names[id] = h.names[id];
    if (seed !== null && rounds !== null && order.length >= 2 && order.length <= 4 && new Set(order).size === order.length && Object.keys(names).length === order.length) {
      out.host = { seed, rounds, order, names };
      out.skips = Array.isArray(raw.skips) ? raw.skips.filter(n => int(n, 0, 9999) !== null).slice(0, 2000) : [];
      out.shopSkips = {};
      if (raw.shopSkips && typeof raw.shopSkips === 'object') {
        for (const [k, ids] of Object.entries(raw.shopSkips).slice(0, 20)) {
          const r = int(Number(k), 1, 20);
          if (r !== null && Array.isArray(ids)) out.shopSkips[r] = ids.filter(id => order.includes(id));
        }
      }
    }
  }
  return out;
}

/** The match a room's host has started, with no moves applied yet. */
export function matchFor(host) {
  return createMatch({ seed: host.seed, rounds: host.rounds, players: host.order.map(id => ({ id, name: host.names[id] })) });
}

/**
 * What happens next in the match, given everyone's snapshots (a Map or object of id -> snapshot,
 * including your own). One of:
 *   { kind: 'shop', buys: [[weaponId...] per player] }   everyone has shopped for the next round
 *   { kind: 'fire', shot }   the current player's shot has arrived
 *   { kind: 'skip' }          the host skipped the current player
 *   { kind: 'wait', ids }     waiting for these players
 *   { kind: 'over' }
 * @returns {NextAction}
 */
export function nextAction(match, host, snapshots) {
  const get = id => (snapshots instanceof Map ? snapshots.get(id) : snapshots[id]);
  const hostSnap = get(host.order[0]);
  if (match.phase === 'over') return { kind: 'over' };
  if (match.phase === 'shop') {
    const round = match.round + 1;
    const skipped = hostSnap?.shopSkips?.[round] ?? [];
    const missing = host.order.filter(id => !get(id)?.shops?.[round] && !skipped.includes(id));
    if (missing.length) return { kind: 'wait', ids: missing };
    return { kind: 'shop', buys: host.order.map(id => (skipped.includes(id) ? [] : get(id)?.shops?.[round] ?? [])) };
  }
  const k = match.log.length;
  const owner = host.order[match.turn];
  const shot = get(owner)?.turns?.[k];
  if (shot) return { kind: 'fire', shot };
  if (hostSnap?.skips?.includes(k)) return { kind: 'skip' };
  return { kind: 'wait', ids: [owner] };
}

/** @typedef {{kind:'shop',buys:string[][]}|{kind:'fire',shot:{angle:number,power:number,weapon:string}}|{kind:'skip'}|{kind:'wait',ids:string[]}|{kind:'over'}} NextAction */

/** Applies one action from nextAction. Returns fire()'s result for a shot, else null. */
export function apply(match, action) {
  if (action.kind === 'shop') {
    action.buys.forEach((list, i) => { for (const w of list) buy(match, i, w); });
    startRound(match);
    return null;
  }
  if (action.kind === 'fire') return fire(match, action.shot);
  if (action.kind === 'skip') { skip(match); return null; }
  return null;
}

/** Replays a room from the start up to the first input not yet published. */
export function replay(host, snapshots) {
  const match = matchFor(host);
  for (let guard = 0; guard < 5000; guard++) {
    const action = nextAction(match, host, snapshots);
    if (action.kind === 'wait' || action.kind === 'over') return { match, action };
    apply(match, action);
  }
  throw new Error('replay did not settle');
}
