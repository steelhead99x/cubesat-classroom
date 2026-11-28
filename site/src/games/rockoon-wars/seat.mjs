// Keeps this tab's seat in a Rockoon Battle room across a page reload.
//
// The Worker gives each player an id and a seat ticket (an HMAC of game, room and id) with its
// first token. Sending both back keeps the same id, so a match in progress still knows the
// player. Without this, a reload would ask for a fresh id and a player who was in the match could
// only watch. The seat is kept in session storage, which survives a reload of the same tab and
// nothing else, under a key for one room, so a seat is never offered to a different room.

const KEY = 'rockoon-seat:';
const PLAYER = /^[0-9a-f]{12}$/;
const TICKET = /^[A-Za-z0-9_-]{16,128}$/;
const ROOM = /^[A-Z0-9]{1,16}$/;
// Peers drop a snapshot whose join time is not an integer in this range (cleanSnapshot in room.mjs).
const JOINED_MAX = 1e14;
const validJoined = joined => Number.isInteger(joined) && joined > 0 && joined <= JOINED_MAX;
const validCredentials = (player, ticket) =>
  typeof player === 'string' && PLAYER.test(player) && typeof ticket === 'string' && TICKET.test(ticket);

/**
 * The seat this tab held in `room`, or null. `storage` is sessionStorage (or a stand-in in tests);
 * a missing, blocked or damaged store reads as no seat.
 * @returns {{ player: string, ticket: string, creator: boolean, joined: number } | null}
 */
export function loadSeat(storage, room) {
  if (!ROOM.test(String(room))) return null;
  let raw;
  try { raw = storage?.getItem(KEY + room); } catch { return null; }
  if (typeof raw !== 'string') return null;
  let seat;
  try { seat = JSON.parse(raw); } catch { return null; }
  if (!seat || typeof seat !== 'object') return null;
  const { player, ticket, creator, joined } = seat;
  if (!validCredentials(player, ticket)) return null;
  return {
    player,
    ticket,
    creator: creator === true,
    joined: validJoined(joined) ? joined : Date.now(),
  };
}

/** Saves this tab's seat in `room`. Returns false if the store refused it (private mode, full). */
export function saveSeat(storage, room, seat) {
  if (!ROOM.test(String(room)) || !validCredentials(seat?.player, seat?.ticket)) return false;
  try {
    storage.setItem(KEY + room, JSON.stringify({
      player: seat.player, ticket: seat.ticket, creator: seat.creator === true,
      joined: validJoined(seat.joined) ? seat.joined : Date.now(),
    }));
    return true;
  } catch { return false; }
}

/**
 * Claims `player`'s seat in `room` for this tab with a Web Lock, so two tabs never use one seat.
 * (Opening a link from the game, or Duplicate Tab, copies session storage, saved seat and all.)
 * Resolves to a function that gives the seat up; null if another tab of this browser holds it; or
 * 'unavailable' if this browser has no Web Locks, or the lock request failed, so it can't tell
 * whether another tab holds it. A lock is also given up when its page closes or reloads, so the
 * reloaded page can claim it again. `locks` is navigator.locks.
 * @returns {Promise<(() => void) | null | 'unavailable'>}
 */
export function claimSeat(locks, room, player) {
  if (!ROOM.test(String(room)) || typeof player !== 'string' || !PLAYER.test(player)) return Promise.resolve(null);
  if (typeof locks?.request !== 'function') return Promise.resolve('unavailable');
  return new Promise(resolve => {
    let release = () => {};
    const held = new Promise(r => { release = r; });
    Promise.resolve()
      .then(() => locks.request(`${KEY}${room}:${player}`, { ifAvailable: true }, lock => {
        if (!lock) { resolve(null); return; }
        resolve(() => release());
        return held;
      }))
      .catch(() => resolve('unavailable'));
  });
}
