// Rockoon Wars online: how a room's published snapshots become one shared match.
//
// Every player publishes one snapshot over MoQ and keeps republishing it:
//   { p, cs, joined, creator, host?, turns: { [turnNumber]: { angle, power, weapon } },
//     shops: { [round]: [weaponId, ...] }, log?: {...}, shopLog?: {...} }
// The room creator adds `host` when the match starts: the seed, the player order and the rounds.
//
// Players' `turns` and `shops` are requests. One player at a time decides what each turn was:
// the committer, normally the room creator (order[0]). It copies the shot (or "skip" for a
// player who left) into `log`, and each round's purchases into `shopLog`, once, and never
// changes an entry. If the committer leaves, the next player in the order who is still here
// takes over (committerFor), so the match keeps going.
//
// Every player in the match republishes `host` and a copy of the history it has played, so a
// player who joins or reconnects after the creator has left can still catch up. Another
// player's copy only counts if that player is the committer or the copy matches what the
// players asked for, so nobody can commit a move for someone else (historyEntry). When copies
// disagree, the one earlier in the order wins; a screen that already played the other replays
// from the start (diverged).
import { WEAPON, createMatch, buy, startRound, fire, skip } from './logic.mjs';

/**
 * The room protocol. It names the relay namespace (rockoon-battle-v<N>) and every snapshot
 * carries it as `p`. A snapshot from another version is ignored, so a client that follows only
 * the creator (v3 and older) and one that hands the room on (v4) never share a match.
 */
export const ROOM_PROTOCOL = 4;
export const ROOM_NAMESPACE = `rockoon-battle-v${ROOM_PROTOCOL}`;

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
  if (raw.p !== ROOM_PROTOCOL) return null;
  if (typeof raw.cs !== 'string' || !CS.test(raw.cs)) return null;
  const joined = int(raw.joined, 0, 1e14);
  if (joined === null) return null;
  const out = { p: ROOM_PROTOCOL, cs: raw.cs, joined, creator: raw.creator === true, turns: {}, shops: {} };
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
  // Display only: the turn number whose balloon this player has lifted but not yet released,
  // so other screens can show it rising. It never changes the match.
  const rising = int(raw.rising, 0, 9999);
  if (rising !== null) out.rising = rising;
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

/** The player who records turns: the first in the order who hasn't left. `gone` excludes this screen's own players. */
export function committerFor(host, gone = []) {
  return host.order.find(id => !gone.includes(id)) ?? host.order[0];
}

/**
 * Whether a copy of a history entry matches what the players asked for: a shot equal to the
 * turn owner's own `turns` request, a shop list equal to each player's own `shops` (an empty
 * list only for a player whose picks never arrived). A skip can't be checked.
 */
function matchesRequests(host, get, field, key, entry, owner) {
  if (field === 'log') {
    const asked = owner && get(owner)?.turns?.[key];
    return entry !== 'skip' && !!asked && asked.angle === entry.angle && asked.power === entry.power && asked.weapon === entry.weapon;
  }
  return Array.isArray(entry) && entry.length === host.order.length && entry.every((list, i) => {
    const asked = get(host.order[i])?.shops?.[key];
    return Array.isArray(list) && (Array.isArray(asked) ? JSON.stringify(asked) === JSON.stringify(list) : list.length === 0);
  });
}

/**
 * A committed log entry for turn k (owned by `owner`), or a shop list for a round. Copies are
 * read in player order and the first believable one wins. A copy is believed when its author
 * is the creator, or the committer as this screen sees it (everyone earlier in the order has
 * gone), or when it matches what the players asked for. So the creator's history never
 * changes, and nobody else can commit a shot, a skip or a purchase for someone else.
 */
function historyEntry(host, get, field, key, owner, gone = []) {
  for (const [i, id] of host.order.entries()) {
    const entry = get(id)?.[field]?.[key];
    if (!entry) continue;
    const committer = host.order.slice(0, i).every(p => gone.includes(p));
    if (committer || matchesRequests(host, get, field, key, entry, owner)) return entry;
  }
  return undefined;
}

/**
 * What happens next in the match, from the committed history. One of:
 *   { kind: 'shop', buys: [[weaponId...] per player] }   the shop for the next round is closed
 *   { kind: 'fire', shot }   the current player's shot is committed
 *   { kind: 'skip' }          the current player was skipped
 *   { kind: 'wait', ids, sent? }   waiting for these players (sent: their move is on its way)
 *   { kind: 'over' }
 * Actions taken from history carry `ref`, so apply() can note what was played.
 * @returns {NextAction}
 */
export function nextAction(match, host, snapshots, gone = []) {
  const get = getter(snapshots);
  if (match.phase === 'over') return { kind: 'over' };
  if (match.phase === 'shop') {
    const round = match.round + 1;
    const buys = historyEntry(host, get, 'shopLog', round, undefined, gone);
    if (buys) return { kind: 'shop', buys, ref: `S${round}` };
    const missing = host.order.filter(id => !get(id)?.shops?.[round]);
    return missing.length ? { kind: 'wait', ids: missing } : { kind: 'wait', ids: [], sent: true };
  }
  const k = match.log.length;
  const owner = host.order[match.turn];
  const entry = historyEntry(host, get, 'log', k, owner, gone);
  if (entry === 'skip') return { kind: 'skip', ref: `L${k}`, owner };
  if (entry) return { kind: 'fire', shot: entry, ref: `L${k}`, owner };
  return get(owner)?.turns?.[k] ? { kind: 'wait', ids: [owner], sent: true } : { kind: 'wait', ids: [owner] };
}

/**
 * Copies the history this screen has played into `id`'s own snapshot, so it survives the
 * committer leaving. Only entries already checked and played are copied, so a forged entry
 * never spreads. Returns true when it added anything.
 */
export function mirror(match, host, snapshots, id) {
  const get = getter(snapshots);
  const mine = get(id);
  if (!mine || !host.order.includes(id)) return false;
  let added = false;
  if (!mine.host) { mine.host = structuredClone(host); added = true; }
  mine.log ??= {};
  mine.shopLog ??= {};
  for (const [ref, { entry }] of Object.entries(match.played ?? {})) {
    const field = ref[0] === 'S' ? 'shopLog' : 'log';
    const key = ref.slice(1);
    if (JSON.stringify(mine[field][key]) !== JSON.stringify(entry)) {
      mine[field][key] = structuredClone(entry);
      added = true;
    }
  }
  return added;
}

/**
 * The committer's side: records what the current turn or shop was, once. `gone` lists players
 * this screen has given up on. `id` is the committer (committerFor). Writes into that player's
 * own snapshot and returns true when it added an entry.
 */
export function commit(match, host, snapshots, gone = [], id = host.order[0]) {
  const get = getter(snapshots);
  const mine = get(id);
  if (!mine || match.phase === 'over') return false;
  mine.log ??= {};
  mine.shopLog ??= {};
  if (match.phase === 'shop') {
    const round = match.round + 1;
    if (historyEntry(host, get, 'shopLog', round, undefined, gone)) return false;
    if (host.order.some(p => !get(p)?.shops?.[round] && !gone.includes(p))) return false;
    mine.shopLog[round] = host.order.map(p => [...(get(p)?.shops?.[round] ?? [])]);
    return true;
  }
  const k = match.log.length;
  const owner = host.order[match.turn];
  if (historyEntry(host, get, 'log', k, owner, gone)) return false;
  const shot = get(owner)?.turns?.[k];
  if (shot) mine.log[k] = { angle: shot.angle, power: shot.power, weapon: shot.weapon };
  else if (gone.includes(owner)) mine.log[k] = 'skip';
  else return false;
  return true;
}

/** True when an entry this match has already played no longer matches the committed history. */
export function diverged(match, host, snapshots, gone = []) {
  const get = getter(snapshots);
  for (const [ref, played] of Object.entries(match.played ?? {})) {
    const entry = historyEntry(host, get, ref[0] === 'S' ? 'shopLog' : 'log', Number(ref.slice(1)), played.owner, gone);
    if (JSON.stringify(entry) !== JSON.stringify(played.entry)) return true;
  }
  return false;
}

/** @typedef {{kind:'shop',buys:string[][],ref?:string}|{kind:'fire',shot:{angle:number,power:number,weapon:string},ref?:string,owner?:string}|{kind:'skip',ref?:string,owner?:string}|{kind:'wait',ids:string[],sent?:boolean}|{kind:'over'}} NextAction */

/** Applies one action from nextAction. Returns fire()'s result for a shot, else null. */
export function apply(match, action) {
  if (action.ref) {
    match.played ??= {};
    match.played[action.ref] = { entry: structuredClone(action.kind === 'shop' ? action.buys : action.kind === 'fire' ? action.shot : 'skip'), owner: action.owner };
  }
  if (action.kind === 'shop') {
    action.buys.forEach((list, i) => { for (const w of list) buy(match, i, w); });
    startRound(match);
    return null;
  }
  if (action.kind === 'fire') return fire(match, action.shot);
  if (action.kind === 'skip') { skip(match); return null; }
  return null;
}

/** Replays the committed history from the start. A reconnecting client uses this to catch up. */
export function replay(host, snapshots, gone = []) {
  const match = matchFor(host);
  for (let guard = 0; guard < 5000; guard++) {
    const action = nextAction(match, host, snapshots, gone);
    if (action.kind === 'wait' || action.kind === 'over') return { match, action };
    apply(match, action);
  }
  throw new Error('replay did not settle');
}

