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
// players asked for, so nobody can commit a move for someone else (historyEntry). An entry a
// player believed only on a stand-in's word is copied with that stand-in named in `standin`
// ({ L<turn> | S<round>: authorId }), so republishing a copy never lends the stand-in's record the
// copier's authority, and the record outlives the stand-in leaving. When copies
// disagree, the one earlier in the order wins; a screen that already played the other replays
// from the start (diverged).
import { WEAPON, createMatch, buy, startRound, fire, skip } from './logic.mjs';

/**
 * The room protocol. It names the relay namespace (rockoon-battle-v<N>) and every snapshot
 * carries it as `p`. A snapshot from another version is ignored, so a client that follows only
 * the creator (v3 and older) and one that hands the room on (v4) never share a match. v5 marks
 * copies of a stand-in's word (`standin`); a v4 client would ignore the mark and take such a copy
 * on the copier's authority, so the two never share a match either.
 */
export const ROOM_PROTOCOL = 5;
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
  // A weapon must be a known id: an object here could break the lookup (or match a prototype key).
  const known = typeof t.weapon === 'string' && Object.hasOwn(WEAPON, t.weapon);
  return angle === null || power === null || !known ? null : { angle, power, weapon: t.weapon };
}
const cleanList = list => list.filter(w => typeof w === 'string' && Object.hasOwn(WEAPON, w)).slice(0, 40);

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
      // Which copied entries are a stand-in's word, and whose: never the creator's (order[0]),
      // who needs no stand-in, and only for an entry this snapshot carries.
      if (raw.standin && typeof raw.standin === 'object') {
        const standin = {};
        for (const [ref, by] of Object.entries(raw.standin).slice(0, 2020)) {
          const m = /^([LS])(\d{1,4})$/.exec(ref);
          if (!m || typeof by !== 'string' || order.indexOf(by) < 1) continue;
          if ((m[1] === 'L' ? out.log : out.shopLog)[Number(m[2])] !== undefined) standin[`${m[1]}${Number(m[2])}`] = by;
        }
        if (Object.keys(standin).length) out.standin = standin;
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

/** The name of a history entry in `played` and `standin`: L<turn> or S<round>. */
const refOf = (field, key) => `${field === 'log' ? 'L' : 'S'}${Number(key)}`;
const sameEntry = (a, b) => a !== undefined && JSON.stringify(a) === JSON.stringify(b);

/** Whether p carries `entry` as a copy of `author`'s word: it took it from that stand-in. */
function vouchedBy(get, p, field, key, author, entry) {
  const snap = get(p);
  return snap?.standin?.[refOf(field, key)] === author && (entry === undefined ? snap?.[field]?.[key] !== undefined : sameEntry(snap?.[field]?.[key], entry));
}

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
 *     the turns its stand-in recorded without it, even when other players lost theirs as well;
 *   - a stand-in earlier in the order than this screen, while it is catching up, only when every
 *     player before the stand-in carries the entry marked as taken from that stand-in. The entry
 *     keeps the stand-in's authority, never the copier's (see relayed for such copies).
 */
function speaksFor(host, get, i, view, field, key) {
  if (host.order.slice(0, i).every(p => view.gone.includes(p))) return true;
  if (!view.rejoined || !view.me) return false;
  const mine = host.order.indexOf(view.me);
  if (mine < 0) return false;
  // A player who carries this entry as a copy of this author's word took it from the author as
  // its stand-in: it was away for it, and says so.
  const tookFromAuthor = p => vouchedBy(get, p, field, key, host.order[i]);
  // A stand-in later in the order than this screen: only that counts. Each player before it has
  // itself taken this entry from it, so the author was the committer then.
  if (mine >= i) return committerFor(host, [...view.gone, view.me, ...host.order.filter((p, j) => j < i && tookFromAuthor(p))]) === host.order[i];
  // Away, for this entry: gone, this screen, anyone catching up, anyone who took it from this
  // author, and
  // anyone whose history stops short of it (they lost it on a reload, or were away, so they
  // can't have stood in for it). A player whose history runs right up to this entry is here and
  // current, so nobody after it in the order can speak over it. If the committer later publishes
  // a different entry, it is earlier in the order and wins (diverged).
  // A player who is here records within a turn or so of the newest entry, so only one with no
  // history at all, or more than a whole round of turns behind, counts as having lost it.
  const behind = p => {
    const keys = Object.keys(get(p)?.[field] ?? {}).map(Number);
    return !keys.length || Math.max(...keys) < Number(key) - host.order.length || tookFromAuthor(p);
  };
  const away = [...view.gone, view.me, ...host.order.filter((p, j) => j < i && (get(p)?.catchUp || behind(p)))];
  return committerFor(host, away) === host.order[i];
}

/**
 * A committed log entry for turn k (owned by `owner`), or a shop list for a round. Copies are
 * read in player order and the first believable one wins. A copy is believed when its author
 * speaks for the history (speaksFor), when it matches what the players asked for, or, for a log
 * entry, when the turn's owner carries the identical entry in its own history: an owner's word
 * on its own turn, which no one else can publish. A screen also believes its own snapshot, which
 * only ever holds what it committed or already believed. So nobody can commit a shot, a skip or a
 * purchase for someone else, and an entry that was believed and copied stays believable after
 * requests are gone (a reload, or a player leaving).
 */
function historyEntry(host, get, field, key, owner, view = {}) {
  return findEntry(host, get, field, key, owner, view)?.entry;
}

/**
 * Whether a live request says something different from an entry: the owner's own `turns`
 * request for that turn, or any player's own picks for that round. A stand-in's copy never
 * overrides what a player is asking for right now.
 */
function contradicted(host, get, field, key, entry, owner) {
  if (field === 'log') {
    const asked = owner && get(owner)?.turns?.[key];
    return !!asked && (entry === 'skip' || asked.angle !== entry.angle || asked.power !== entry.power || asked.weapon !== entry.weapon);
  }
  return Array.isArray(entry) && host.order.some((id, i) => {
    const asked = get(id)?.shops?.[key];
    return Array.isArray(asked) && JSON.stringify(asked) !== JSON.stringify(entry[i]);
  });
}

/**
 * Whether a copy marked as stand-in `by`'s word (`standin`) is believable, as that stand-in's
 * word: never the copier's. Only a player's screen catching up takes it, as it would from the
 * stand-in itself, or a watcher's screen (not in the match, so it never republishes history and
 * can only mislead itself), and only when every player before the stand-in is gone, is this screen, or carries the
 * same entry taken from the same stand-in (it was away, and the stand-in recorded it), and when
 * the stand-in itself, if it is still here, carries the same entry or none. The stand-in or an
 * earlier player's copy can vouch for it. If those players have left, the current committer can
 * supply its retained copy. The record keeps the original author's mark in every case.
 */
function relayed(host, get, field, key, entry, by, carrier, v) {
  if (!v.me) return false;
  // A watcher follows the match from whatever copies are left: after the stand-in has gone, a
  // marked copy may be the only record of its entry.
  if (host.order.includes(v.me) && !v.rejoined) return false;
  const j = host.order.indexOf(by);
  if (j < 1) return false;
  const own = get(by)?.[field]?.[key];
  if (own !== undefined && !sameEntry(own, entry)) return false;
  const before = host.order.slice(0, j);
  const vouches = p => vouchedBy(get, p, field, key, by, entry);
  if (!before.every(p => v.gone.includes(p) || p === v.me || vouches(p))) return false;
  if (sameEntry(own, entry) || before.some(vouches)) return true;
  if (!v.gone.includes(by)) return false;
  // A returning player may have been away when the remaining committer took over.
  const away = v.rejoined ? [...v.gone, v.me] : v.gone;
  return committerFor(host, away) === carrier;
}

/** historyEntry, saying how the copy was believed: 'standin' (with `by`, whose word it is) when only by the catch-up rule. */
function findEntry(host, get, field, key, owner, view = {}) {
  const v = viewOf(view);
  const ref = refOf(field, key);
  for (const [i, id] of host.order.entries()) {
    const entry = get(id)?.[field]?.[key];
    if (!entry) continue;
    // A copy of a stand-in's word: the copier's place in the order gives it no authority.
    const by = get(id)?.standin?.[ref];
    if (!by && host.order.slice(0, i).every(p => v.gone.includes(p))) return { entry, via: 'committer' };
    // This screen's own copy: it only ever holds what it committed or already believed.
    // A copy of a stand-in's word stays one, so mirroring it again keeps the mark.
    if (id === v.me) return by ? { entry, via: 'standin', by } : { entry, via: 'own' };
    if (matchesRequests(host, get, field, key, entry, owner, v.gone)) return { entry, via: 'requests' };
    if (field === 'log' && owner && JSON.stringify(get(owner)?.log?.[key]) === JSON.stringify(entry) && !get(owner)?.standin?.[ref]) return { entry, via: 'owner' };
    const live = owner === v.me || !contradicted(host, get, field, key, entry, owner);
    if (by) {
      if (relayed(host, get, field, key, entry, by, id, v) && live) return { entry, via: 'standin', by };
      continue;
    }
    // A stand-in can't override what another player is asking for right now. This screen's own
    // turn is different: it is catching up, and a request it made before seeing the stand-in's
    // record of that turn is just late.
    if (speaksFor(host, get, i, v, field, key) && live) return { entry, via: 'standin', by: id };
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
    const found = findEntry(host, get, 'shopLog', round, undefined, view);
    if (found) return { kind: 'shop', buys: found.entry, ref: `S${round}`, via: found.via, by: found.by };
    const missing = host.order.filter(id => !get(id)?.shops?.[round]);
    return missing.length ? { kind: 'wait', ids: missing } : { kind: 'wait', ids: [], sent: true };
  }
  const k = match.log.length;
  const owner = host.order[match.turn];
  const found = findEntry(host, get, 'log', k, owner, view);
  const entry = found?.entry;
  if (entry === 'skip') return { kind: 'skip', ref: `L${k}`, owner, via: found.via, by: found.by };
  if (entry) return { kind: 'fire', shot: entry, ref: `L${k}`, owner, via: found.via, by: found.by };
  return get(owner)?.turns?.[k] ? { kind: 'wait', ids: [owner], sent: true } : { kind: 'wait', ids: [owner] };
}

/**
 * Copies the history this screen has played into `id`'s own snapshot, so it survives the
 * committer leaving. Only entries already checked and played are copied. An entry taken only on
 * a stand-in's word is copied with that stand-in named in `standin`, unless it is `id`'s own
 * turn (an owner's word on its own turn is believable already), so the copy keeps the stand-in's
 * authority and never gains `id`'s: a forged entry can't spread on someone else's authority, and
 * the record stays replayable after the stand-in leaves. Returns true when it changed anything.
 */
export function mirror(match, host, snapshots, id) {
  const get = getter(snapshots);
  const mine = get(id);
  if (!mine || !host.order.includes(id)) return false;
  let added = false;
  if (!mine.host) { mine.host = structuredClone(host); added = true; }
  mine.log ??= {};
  mine.shopLog ??= {};
  for (const [ref, { entry, owner, via, by }] of Object.entries(match.played ?? {})) {
    const field = ref[0] === 'S' ? 'shopLog' : 'log';
    const key = ref.slice(1);
    // A shop entry covers everyone's picks, so it is never `id`'s alone.
    const author = via === 'standin' && by && by !== id && !(field === 'log' && owner === id) ? by : undefined;
    if (JSON.stringify(mine[field][key]) !== JSON.stringify(entry)) {
      mine[field][key] = structuredClone(entry);
      added = true;
    }
    if ((mine.standin?.[ref]) !== author) {
      if (author) (mine.standin ??= {})[ref] = author;
      else unmark(mine, ref);
      added = true;
    }
  }
  return added;
}

/** This entry is now the snapshot's own record, not a copy of a stand-in's word. */
function unmark(snap, ref) {
  if (!snap.standin?.[ref]) return;
  delete snap.standin[ref];
  if (!Object.keys(snap.standin).length) delete snap.standin;
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
    unmark(mine, `S${round}`);
    return true;
  }
  const k = match.log.length;
  const owner = host.order[match.turn];
  if (historyEntry(host, get, 'log', k, owner, gone)) return false;
  const shot = get(owner)?.turns?.[k];
  if (shot) mine.log[k] = { angle: shot.angle, power: shot.power, weapon: shot.weapon };
  else if (gone.includes(owner)) mine.log[k] = 'skip';
  else return false;
  unmark(mine, `L${k}`);
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

/** @typedef {{kind:'shop',buys:string[][],ref?:string,via?:string,by?:string}|{kind:'fire',shot:{angle:number,power:number,weapon:string},ref?:string,owner?:string,via?:string,by?:string}|{kind:'skip',ref?:string,owner?:string,via?:string,by?:string}|{kind:'wait',ids:string[],sent?:boolean}|{kind:'over'}} NextAction */

/** Applies one action from nextAction. Returns fire()'s result for a shot, else null. */
export function apply(match, action) {
  if (action.ref) {
    match.played ??= {};
    match.played[action.ref] = { entry: structuredClone(action.kind === 'shop' ? action.buys : action.kind === 'fire' ? action.shot : 'skip'), owner: action.owner, via: action.via, ...(action.by ? { by: action.by } : {}) };
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

