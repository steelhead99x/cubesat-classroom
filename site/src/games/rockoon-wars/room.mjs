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
  // This player is catching up (it has just rejoined, or just met a player with history it
  // missed). Other screens then count it as away when working out who stood in for whom.
  if (raw.catchUp === true) out.catchUp = true;
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
 * turn owner's own `turns` request, or a shop list equal to each player's own `shops`. An empty
 * list stands in for a player only once that player has gone in this screen's view. A skip
 * can't be checked this way.
 */
function matchesRequests(host, get, field, key, entry, owner, gone = []) {
  if (field === 'log') {
    const asked = owner && get(owner)?.turns?.[key];
    return entry !== 'skip' && !!asked && asked.angle === entry.angle && asked.power === entry.power && asked.weapon === entry.weapon;
  }
  return Array.isArray(entry) && entry.length === host.order.length && entry.every((list, i) => {
    if (!Array.isArray(list)) return false;
    const asked = get(host.order[i])?.shops?.[key];
    if (Array.isArray(asked)) return JSON.stringify(asked) === JSON.stringify(list);
    return list.length === 0 && gone.includes(host.order[i]);
  });
}

/**
 * How this screen sees the room: `gone` lists players it has given up on (never itself), `me`
 * is its own player id, and `rejoined` is true for a short while after it (re)joined, when it
 * may have missed turns. A plain array is read as `gone`.
 * @typedef {{gone?: string[], me?: string, rejoined?: boolean, ownSkips?: boolean} | string[]} RoomView
 */
const viewOf = view => {
  const v = Array.isArray(view) ? { gone: view } : { gone: [], ...view };
  return { ...v, rejoined: !!(v.rejoined ?? v.ownSkips) };
};

/**
 * Who may speak for the history, from one screen's point of view. Every player publishes only
 * its own snapshot (the relay enforces that), so authority comes from who wrote a copy:
 *   - the creator (order[0]) always;
 *   - the committer: a player all of whose predecessors in the order have gone, as seen here;
 *   - the stand-in for this screen: while this screen is catching up (`rejoined`), the player
 *     who would have been the committer with this screen away, along with those gone, any
 *     player whose own snapshot says it is catching up too, and any player whose history stops
 *     short of the entry. That is how a returning host keeps
 *     the turns its stand-in recorded without it, even when other players lost theirs as well.
 */
function speaksFor(host, get, i, view, field, key) {
  if (host.order.slice(0, i).every(p => view.gone.includes(p))) return true;
  if (!view.rejoined || !view.me) return false;
  const mine = host.order.indexOf(view.me);
  if (mine < 0 || mine >= i) return false;
  // Away, for this entry: gone, this screen, anyone catching up, and anyone whose history stops
  // short of it (they lost it on a reload, or were away, so they can't have stood in for it). A
  // player whose history runs right up to this entry is here and current, so nobody after it in
  // the order can speak over it. If the committer later publishes a different entry, it is
  // earlier in the order and wins (diverged).
  // A player who is here records within a turn or so of the newest entry, so only one with no
  // history at all, or more than a whole round of turns behind, counts as having lost it.
  const behind = p => {
    const keys = Object.keys(get(p)?.[field] ?? {}).map(Number);
    return !keys.length || Math.max(...keys) < Number(key) - host.order.length;
  };
  const away = [...view.gone, view.me, ...host.order.filter((p, j) => j < i && (get(p)?.catchUp || behind(p)))];
  return committerFor(host, away) === host.order[i];
}

/**
 * A committed log entry for turn k (owned by `owner`), or a shop list for a round. Copies are
 * read in player order and the first believable one wins. A copy is believed when its author
 * speaks for the history (speaksFor), when it matches what the players asked for, or, for a log
 * entry, when the turn's owner carries the identical entry in its own history: an owner's word
 * on its own turn, which no one else can publish. So nobody can commit a shot, a skip or a
 * purchase for someone else, and an entry that was believed and copied stays believable after
 * requests are gone (a reload, or a player leaving).
 */
function historyEntry(host, get, field, key, owner, view = {}) {
  const v = viewOf(view);
  for (const [i, id] of host.order.entries()) {
    const entry = get(id)?.[field]?.[key];
    if (!entry) continue;
    if (speaksFor(host, get, i, v, field, key) || matchesRequests(host, get, field, key, entry, owner, v.gone)) return entry;
    if (field === 'log' && owner && JSON.stringify(get(owner)?.log?.[key]) === JSON.stringify(entry)) return entry;
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
export function nextAction(match, host, snapshots, view = {}) {
  const get = getter(snapshots);
  if (match.phase === 'over') return { kind: 'over' };
  if (match.phase === 'shop') {
    const round = match.round + 1;
    const buys = historyEntry(host, get, 'shopLog', round, undefined, view);
    if (buys) return { kind: 'shop', buys, ref: `S${round}` };
    const missing = host.order.filter(id => !get(id)?.shops?.[round]);
    return missing.length ? { kind: 'wait', ids: missing } : { kind: 'wait', ids: [], sent: true };
  }
  const k = match.log.length;
  const owner = host.order[match.turn];
  const entry = historyEntry(host, get, 'log', k, owner, view);
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

/** True when a believable copy of an entry this match has already played says something else. */
export function diverged(match, host, snapshots, view = {}) {
  const get = getter(snapshots);
  for (const [ref, played] of Object.entries(match.played ?? {})) {
    const entry = historyEntry(host, get, ref[0] === 'S' ? 'shopLog' : 'log', Number(ref.slice(1)), played.owner, view);
    // Only a believable copy that says something different counts. An entry nobody now
    // vouches for (say, the host came back without publishing anything for that turn) stays
    // played: a heartbeat alone never undoes a committed turn.
    if (entry !== undefined && JSON.stringify(entry) !== JSON.stringify(played.entry)) return true;
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
export function replay(host, snapshots, view = {}) {
  const match = matchFor(host);
  for (let guard = 0; guard < 5000; guard++) {
    const action = nextAction(match, host, snapshots, view);
    if (action.kind === 'wait' || action.kind === 'over') return { match, action };
    apply(match, action);
  }
  throw new Error('replay did not settle');
}

