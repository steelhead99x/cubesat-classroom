import { MAX_BODY_BYTES, readSmallJson } from './games-token.mjs';
import { ROOM_CODE } from './games/rockoon-wars/room.mjs';

const SEAT = /^[0-9a-f]{48}$/;
const KINDS = new Set(['create', 'join', 'sync', 'start', 'shop', 'rise', 'fire']);
const reply = (body, status) => Response.json(body, { status, headers: { 'cache-control': 'no-store' } });

export async function rockoonRoom(request, env) {
  if (request.method !== 'POST') return reply({ error: 'Use POST.' }, 405);
  const site = request.headers.get('sec-fetch-site');
  if (request.headers.get('origin') !== new URL(request.url).origin || (site && site !== 'same-origin')) {
    return reply({ error: 'Only this site can play.' }, 403);
  }
  if (!(request.headers.get('content-type') ?? '').startsWith('application/json')) return reply({ error: 'Send JSON.' }, 415);
  if (!env.ROCKOON_ROOMS || !env.GAME_TOKEN_LIMIT || !env.ROCKOON_REQUEST_LIMIT) return reply({ error: 'Online play is unavailable.' }, 503);
  const input = await readSmallJson(request, MAX_BODY_BYTES).catch(() => null);
  if (input === undefined) return reply({ error: 'Request too large.' }, 413);
  if (typeof input?.room !== 'string' || !ROOM_CODE.test(input.room) || !KINDS.has(input.kind)) {
    return reply({ error: 'That room request is not valid.' }, 400);
  }
  const { kind, version, buys, shot } = input;
  const ticket = typeof input.ticket === 'string' && SEAT.test(input.ticket) ? input.ticket : null;
  const joining = kind === 'join' || kind === 'create';
  if (!joining && !ticket) return reply({ error: 'Join the room first.' }, 403);
  const limiter = joining ? env.GAME_TOKEN_LIMIT : env.ROCKOON_REQUEST_LIMIT;
  const { success } = await limiter.limit({ key: `rockoon:${request.headers.get('cf-connecting-ip') ?? 'unknown'}` });
  if (!success) return reply({ error: 'Too many requests. Try again in a minute.' }, 429);
  const result = await env.ROCKOON_ROOMS.getByName(input.room).request({ kind, ticket, version, buys, shot });
  return reply(result.body, result.status);
}
