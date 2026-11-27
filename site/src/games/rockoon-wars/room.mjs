// Rockoon Wars online: how a room's published snapshots become one shared match.
//
// Every player publishes one snapshot over MoQ and keeps republishing it:
//   { cs, joined, creator, host?, turns: { [turnNumber]: { angle, power, weapon } },
//     shops: { [round]: [weaponId, ...] }, log?: {...}, shopLog?: {...} }
// The room creator adds `host` when the match starts: the seed, the player order and the rounds.
//
// Players' `turns` and `shops` are requests. Only the host decides what each turn was: it copies
// the shot (or "skip" for a player who left) into `log`, and each round's purchases into
// `shopLog`, once, and never changes an entry. Every client, including one that reconnects, replays
// only that committed history through logic.mjs, so a late shot and a skip can never be applied in
// different orders on different screens.
import { WEAPON, createMatch, buy, startRound, fire, skip } from './logic.mjs';

const ID = /^[0-9a-f]{12}$/;
const CS = /^GS-[A-Z]{2}\d$/;
export const ROOM_CODE = /^[A-HJ-NP-Z2-9]{5}$/;
const int = (v, lo, hi) => (Number.isInteger(v) && v >= lo && v <= hi ? v : null);

export function newRoomCode(random = Math.random) {
  const abc = 'ABCDEFGHJKLMNPQRSTUVWXYZ23456789';
  return Array.from({ length: 5 }, () => abc[Math.floor(random() * abc.length)]).join('');
}

function cleanShot(t) {
  if (!t || typeof t !== 'object') return null;
  const angle = int(t.angle, 0, 180), power = int(t.power, 0, 100);
  return angle === null || power === null || !WEAPON[t.weapon] ? null : { angle, power, weapon: t.weapon };
}
const cleanList = list => list.filter(w => typeof w === 'string' && WEAPON[w]).slice(0, 40);

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
      const shot = cleanShot(t);
      if (shot) out.turns[n] = shot;
    }
  }
  if (raw.shops && typeof raw.shops === 'object') {
    for (const [k, list] of Object.entries(raw.shops).slice(0, 20)) {
      const r = int(Number(k), 1, 20);
      if (r === null || !Array.isArray(list)) continue;
      out.shops[r] = cleanList(list);
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
      out.log = {};
      if (raw.log && typeof raw.log === 'object') {
        for (const [k, t] of Object.entries(raw.log).slice(0, 2000)) {
          const n = int(Number(k), 0, 9999);
          if (n === null) continue;
          if (t === 'skip') { out.log[n] = 'skip'; continue; }
          const shot = cleanShot(t);
          if (shot) out.log[n] = shot;
        }
      }
      out.shopLog = {};
      if (raw.shopLog && typeof raw.shopLog === 'object') {
        for (const [k, lists] of Object.entries(raw.shopLog).slice(0, 20)) {
          const r = int(Number(k), 1, 20);
          if (r === null || !Array.isArray(lists) || lists.length !== order.length) continue;
          out.shopLog[r] = lists.map(list => (Array.isArray(list) ? cleanList(list) : []));
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

const getter = snapshots => id => (snapshots instanceof Map ? snapshots.get(id) : snapshots[id]);

/**
 * What happens next in the match, from the host's committed history. One of:
 *   { kind: 'shop', buys: [[weaponId...] per player] }   the host closed the shop for the next round
 *   { kind: 'fire', shot }   the host committed the current player's shot
 *   { kind: 'skip' }          the host skipped the current player
 *   { kind: 'wait', ids, sent? }   waiting for these players (sent: their move is on its way to the host)
 *   { kind: 'over' }
 * @returns {NextAction}
 */
export function nextAction(match, host, snapshots) {
  const get = getter(snapshots);
  const hostSnap = get(host.order[0]);
  if (match.phase === 'over') return { kind: 'over' };
  if (match.phase === 'shop') {
    const round = match.round + 1;
    const buys = hostSnap?.shopLog?.[round];
    if (buys) return { kind: 'shop', buys };
    const missing = host.order.filter(id => !get(id)?.shops?.[round]);
    return missing.length ? { kind: 'wait', ids: missing } : { kind: 'wait', ids: [], sent: true };
  }
  const k = match.log.length;
  const owner = host.order[match.turn];
  const entry = hostSnap?.log?.[k];
  if (entry === 'skip') return { kind: 'skip' };
  if (entry) return { kind: 'fire', shot: entry };
  return get(owner)?.turns?.[k] ? { kind: 'wait', ids: [owner], sent: true } : { kind: 'wait', ids: [owner] };
}

/**
 * The host's side: records what the current turn or shop was, once. `gone` lists players the host
 * has given up on. Writes into the host's own snapshot and returns true when it added an entry.
 */
export function commit(match, host, snapshots, gone = []) {
  const get = getter(snapshots);
  const hostSnap = get(host.order[0]);
  if (!hostSnap || match.phase === 'over') return false;
  hostSnap.log ??= {};
  hostSnap.shopLog ??= {};
  if (match.phase === 'shop') {
    const round = match.round + 1;
    if (hostSnap.shopLog[round]) return false;
    if (host.order.some(id => !get(id)?.shops?.[round] && !gone.includes(id))) return false;
    hostSnap.shopLog[round] = host.order.map(id => [...(get(id)?.shops?.[round] ?? [])]);
    return true;
  }
  const k = match.log.length;
  if (hostSnap.log[k]) return false;
  const owner = host.order[match.turn];
  const shot = get(owner)?.turns?.[k];
  if (shot) hostSnap.log[k] = { angle: shot.angle, power: shot.power, weapon: shot.weapon };
  else if (gone.includes(owner)) hostSnap.log[k] = 'skip';
  else return false;
  return true;
}

/** @typedef {{kind:'shop',buys:string[][]}|{kind:'fire',shot:{angle:number,power:number,weapon:string}}|{kind:'skip'}|{kind:'wait',ids:string[],sent?:boolean}|{kind:'over'}} NextAction */

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

/** Replays the host's committed history from the start. A reconnecting client uses this to catch up. */
export function replay(host, snapshots) {
  const match = matchFor(host);
  for (let guard = 0; guard < 5000; guard++) {
    const action = nextAction(match, host, snapshots);
    if (action.kind === 'wait' || action.kind === 'over') return { match, action };
    apply(match, action);
  }
  throw new Error('replay did not settle');
}

