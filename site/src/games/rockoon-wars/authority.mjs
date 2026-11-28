// The room service owns the lobby, clock, and append-only action list. The browser uses
// the same simulation for rendering, but sends only a request for the phase it sees.
import { callsignFor } from '../../games-token.mjs';
import { apply, matchFor } from './room.mjs';
import { MAX_PLAYERS, WEAPON } from './logic.mjs';

export const ROOM_LIFETIME_MS = 3 * 60 * 60_000;
const LOBBY_LEASE_MS = 30_000;
const PHASE_MS = 120_000;
const MAX_SEATS = 16;
const randomHex = size => Array.from(crypto.getRandomValues(new Uint8Array(size)), byte => byte.toString(16).padStart(2, '0')).join('');
const knownWeapon = id => typeof id === 'string' && Object.hasOwn(WEAPON, id);
const inRange = (value, hi) => Number.isInteger(value) && value >= 0 && value <= hi;
const error = (status, error) => ({ status, body: { error } });

function newRoom(now) {
  return { expires: now + ROOM_LIFETIME_MS, creator: null, seats: [], host: null, match: null,
    actions: [], picks: {}, rising: null, deadline: null };
}

function addSeat(room, now) {
  let player;
  do { player = randomHex(6); } while (room.seats.some(seat => seat.player === player));
  const seat = { player, ticket: randomHex(24), callsign: callsignFor(player), lastSeen: now };
  room.seats.push(seat);
  room.creator ??= player;
  return seat;
}

function commit(room, action, now) {
  apply(room.match, action);
  room.actions.push(action);
  room.rising = null;
  room.picks = {};
  room.deadline = room.match.phase === 'over' ? null : now + PHASE_MS;
}

function advance(room, now) {
  const phase = room.match?.phase;
  if (phase === 'shop') {
    const order = room.host.order;
    if (now >= room.deadline || order.every(player => Object.hasOwn(room.picks, player))) {
      commit(room, { kind: 'shop', buys: order.map(player => room.picks[player] ?? []) }, now);
    }
  }
  if (phase === 'play' && now >= room.deadline) commit(room, { kind: 'skip' }, now);
}

function command(room, seat, input, now) {
  if (input.kind === 'start' && !room.host && seat.player === room.creator) {
    const order = room.seats.slice(0, MAX_PLAYERS).map(s => s.player);
    if (order.length < 2) return;
    const names = Object.fromEntries(room.seats.slice(0, MAX_PLAYERS).map(s => [s.player, s.callsign]));
    room.host = { seed: crypto.getRandomValues(new Uint32Array(1))[0], order, names, rounds: Math.max(3, order.length) };
    room.match = matchFor(room.host);
    room.deadline = now + PHASE_MS;
  } else if (room.match && input.version === room.actions.length) {
    if (input.kind === 'shop' && room.match.phase === 'shop' && room.host.order.includes(seat.player)) {
      if (Array.isArray(input.buys) && input.buys.length <= 40 && input.buys.every(knownWeapon)
          && !Object.hasOwn(room.picks, seat.player)) room.picks[seat.player] = input.buys;
    } else if (room.match.phase === 'play' && room.host.order[room.match.turn] === seat.player) {
      if (input.kind === 'rise') room.rising ??= seat.player;
      const shot = input.shot;
      if (input.kind === 'fire' && room.rising === seat.player && shot && inRange(shot.angle, 180)
          && inRange(shot.power, 100) && knownWeapon(shot.weapon)) {
        commit(room, { kind: 'fire', shot: { angle: shot.angle, power: shot.power, weapon: shot.weapon } }, now);
      }
    }
  }
}

function snapshot(room) {
  const phase = room.match?.phase;
  const waiting = phase === 'over' ? { kind: 'over' } : { kind: 'wait',
    ids: phase === 'shop' ? room.host.order.filter(id => !Object.hasOwn(room.picks, id))
      : phase === 'play' ? [room.host.order[room.match.turn]] : [] };
  return { creator: room.creator, people: room.seats.map(({ player, callsign }) => ({ player, callsign })),
    host: room.host, actions: room.actions, waiting, rising: room.rising, deadline: room.deadline };
}

/** Processes one RPC synchronously. The adapter persists the updated room before returning. */
export function handleRoom(previous, input, now) {
  let room = previous && previous.expires > now ? previous : null;
  if (!room && input.kind !== 'create') return { room, ...error(404, 'That room is closed or does not exist.') };
  if (room && input.kind === 'create') return { room, ...error(409, 'That room code is in use. Try creating a room again.') };
  room ??= newRoom(now);
  const joining = input.kind === 'join' || input.kind === 'create';
  const existing = room.seats.find(s => s.ticket === input.ticket);
  if (!joining && !existing) return { room, ...error(403, 'This seat could not be found. Join the room again.') };

  if (!room.host) {
    room.seats = room.seats.filter(s => s === existing || now - s.lastSeen < LOBBY_LEASE_MS);
    if (!room.seats.some(s => s.player === room.creator)) room.creator = room.seats[0]?.player ?? null;
  }
  if (!existing && room.seats.length >= MAX_SEATS) return { room, ...error(409, 'This room is full.') };
  const seat = existing ?? addSeat(room, now);
  if (!room.host) seat.lastSeen = now;

  // At an expired deadline the timeout wins, even if the old request is next in the queue.
  advance(room, now);
  command(room, seat, input, now);
  advance(room, now);
  return { room, status: 200, body: { state: snapshot(room), ...(joining && { seat }) } };
}
