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
  if (!PLAYER.test(String(player)) || !TICKET.test(String(ticket))) return null;
  return {
    player,
    ticket,
    creator: creator === true,
    joined: Number.isFinite(joined) && joined > 0 ? joined : Date.now(),
  };
}

/** Saves this tab's seat in `room`. Returns false if the store refused it (private mode, full). */
export function saveSeat(storage, room, seat) {
  if (!ROOM.test(String(room)) || !PLAYER.test(String(seat?.player)) || !TICKET.test(String(seat?.ticket))) return false;
  try {
    storage.setItem(KEY + room, JSON.stringify({
      player: seat.player, ticket: seat.ticket, creator: seat.creator === true, joined: seat.joined,
    }));
    return true;
  } catch { return false; }
}
