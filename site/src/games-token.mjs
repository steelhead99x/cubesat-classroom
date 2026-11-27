// Short-lived MoQ relay tokens for the browser games (POST /api/games/moq-token).
//
// The relay signing key never leaves the Worker. It is the secret MOQ_GAMES_SIGNING_KEY: a
// private JWK (ES256, EdDSA or HS256) as JSON or as the base64url text that `moq auth generate`
// writes. Each player gets a token that lasts TOKEN_TTL seconds and is scoped to one game room:
// it may publish only its own station broadcast (`<root>/<player id>`) and subscribe inside the
// room. Nothing else on the relay is reachable with it.
//
// Settings (wrangler.jsonc vars, not secrets): MOQ_GAMES_RELAY (https URL of the relay, for
// example https://cdn.moq.pro) and MOQ_GAMES_ROOT (the path the key may sign under, for example
// the moq.pro project name followed by /games). Without the key, the relay or the rate limiter,
// the endpoint answers 503 and the games play offline.
//
// Only pages on this site may ask: the Origin header must be this site's origin, cross-site
// fetches are refused, and the GAME_TOKEN_LIMIT rate limiter caps requests per network address.

export const TOKEN_TTL = 600;
// Catch the Pass has one shared sky. Rockoon Battle has rooms, joined by a five-letter code;
// Keep older namespaces for cached clients. Terrain rules replay in their own rooms.
const GAMES = { 'catch-the-pass': { rooms: false }, 'rockoon-wars': { rooms: true }, 'rockoon-battle': { rooms: true }, 'rockoon-battle-v2': { rooms: true } };
const ROOM = 'live';
const ROOM_CODE = /^[A-HJ-NP-Z2-9]{5}$/;
const PLAYER_ID = /^[0-9a-f]{12}$/;

/** The largest request body read. A real one ({ game, room, player, ticket }) is under 200 bytes. */
export const MAX_BODY_BYTES = 1024;

const enc = new TextEncoder();
const b64url = bytes => btoa(String.fromCharCode(...new Uint8Array(bytes))).replace(/\+/g, '-').replace(/\//g, '_').replace(/=+$/, '');
const b64urlText = text => b64url(enc.encode(text));

/**
 * Reads a JSON body of at most `max` bytes, without buffering more than that. Returns
 * undefined when the body is too large and null when it is not valid JSON.
 */
async function readSmallJson(request, max) {
  const declared = Number(request.headers.get('content-length') ?? NaN);
  if (declared > max) return undefined;
  if (!request.body) return null;
  const reader = request.body.getReader();
  const chunks = [];
  let size = 0;
  for (;;) {
    const { done, value } = await reader.read();
    if (done) break;
    size += value.byteLength;
    if (size > max) { await reader.cancel().catch(() => {}); return undefined; }
    chunks.push(value);
  }
  const bytes = new Uint8Array(size);
  let at = 0;
  for (const c of chunks) { bytes.set(c, at); at += c.byteLength; }
  try { return JSON.parse(new TextDecoder().decode(bytes)); } catch { return null; }
}

function reply(body, status = 200) {
  return new Response(JSON.stringify(body), {
    status,
    headers: { 'content-type': 'application/json; charset=utf-8', 'cache-control': 'no-store', 'x-robots-tag': 'noindex' },
  });
}

/** Reads the signing key secret. Throws without saying anything about its contents. */
export function parseKey(raw) {
  const text = String(raw ?? '').trim();
  if (!text) throw new Error('no key');
  let jwk;
  try {
    jwk = JSON.parse(text.startsWith('{') ? text : atob(text.replace(/-/g, '+').replace(/_/g, '/')));
  } catch {
    throw new Error('unreadable key');
  }
  if (jwk.kty === 'EC' && jwk.crv === 'P-256' && jwk.d) return { jwk, alg: 'ES256', import: { name: 'ECDSA', namedCurve: 'P-256' }, sign: { name: 'ECDSA', hash: 'SHA-256' } };
  if (jwk.kty === 'OKP' && jwk.crv === 'Ed25519' && jwk.d) return { jwk, alg: 'EdDSA', import: { name: 'Ed25519' }, sign: { name: 'Ed25519' } };
  if (jwk.kty === 'oct' && jwk.k) return { jwk, alg: 'HS256', import: { name: 'HMAC', hash: 'SHA-256' }, sign: { name: 'HMAC' } };
  throw new Error('unsupported key');
}

export async function signJwt(claims, raw) {
  const key = parseKey(raw);
  // WebCrypto refuses JWKs whose alg or key_ops disagree with the import, so pass only key material.
  const { alg: _alg, key_ops: _ops, use: _use, kid, ...material } = key.jwk;
  const cryptoKey = await crypto.subtle.importKey('jwk', material, key.import, false, ['sign']);
  const header = { typ: 'JWT', alg: key.alg, ...(kid ? { kid } : {}) };
  const body = b64urlText(JSON.stringify(header)) + '.' + b64urlText(JSON.stringify(claims));
  const sig = await crypto.subtle.sign(key.sign, cryptoKey, enc.encode(body));
  return body + '.' + b64url(sig);
}

/** The call sign that goes with a player id. No names: players are anonymous. */
export function callsignFor(id) {
  const bytes = id.match(/../g).map(h => parseInt(h, 16));
  const letters = 'ABCDEFGHJKLMNPRSTUVWXYZ';
  return 'GS-' + letters[bytes[0] % letters.length] + letters[bytes[1] % letters.length] + (bytes[2] % 10);
}

function newPlayerId() {
  return [...crypto.getRandomValues(new Uint8Array(6))].map(b => b.toString(16).padStart(2, '0')).join('');
}

/**
 * A seat ticket lets a player who renews a token (or reconnects) keep the same id in the same
 * room, so a match in progress still knows them. It is an HMAC made with the signing secret, so
 * nobody can claim another player's id.
 */
export async function seatTicket(secret, game, room, id) {
  const key = await crypto.subtle.importKey('raw', enc.encode('seat:' + String(secret)), { name: 'HMAC', hash: 'SHA-256' }, false, ['sign']);
  return b64url(await crypto.subtle.sign('HMAC', key, enc.encode(`${game}/${room}/${id}`)));
}

function sameText(a, b) {
  if (typeof a !== 'string' || typeof b !== 'string' || a.length !== b.length) return false;
  let diff = 0;
  for (let i = 0; i < a.length; i++) diff |= a.charCodeAt(i) ^ b.charCodeAt(i);
  return diff === 0;
}

export async function gameToken(request, env) {
  if (request.method !== 'POST') return reply({ error: 'Use POST.' }, 405);
  const url = new URL(request.url);
  const origin = request.headers.get('origin');
  const fetchSite = request.headers.get('sec-fetch-site');
  if (origin !== url.origin || (fetchSite && fetchSite !== 'same-origin')) return reply({ error: 'Only this site can ask for a game token.' }, 403);
  if (!(request.headers.get('content-type') ?? '').startsWith('application/json')) return reply({ error: 'Send JSON.' }, 415);
  if (Number(request.headers.get('content-length') ?? 0) > MAX_BODY_BYTES) return reply({ error: 'Request too large.' }, 413);

  const relay = String(env.MOQ_GAMES_RELAY ?? '').replace(/\/+$/, '');
  const root = String(env.MOQ_GAMES_ROOT ?? '').replace(/^\/+|\/+$/g, '');
  if (!env.MOQ_GAMES_SIGNING_KEY || !relay.startsWith('https://') || !root || !env.GAME_TOKEN_LIMIT) {
    return reply({ offline: true, error: 'The live sky is not set up yet.' }, 503);
  }
  const { success } = await env.GAME_TOKEN_LIMIT.limit({ key: request.headers.get('cf-connecting-ip') ?? 'unknown' });
  if (!success) return reply({ error: 'Too many requests. Try again in a minute.' }, 429);

  const body = await readSmallJson(request, MAX_BODY_BYTES).catch(() => null);
  if (body === undefined) return reply({ error: 'Request too large.' }, 413);
  const game = String(body?.game ?? '');
  if (!Object.hasOwn(GAMES, game)) return reply({ error: 'Unknown game.' }, 400);
  let room = ROOM;
  if (GAMES[game].rooms) {
    room = String(body?.room ?? '');
    if (!ROOM_CODE.test(room)) return reply({ error: 'That room code is not valid.' }, 400);
  }

  let id = newPlayerId();
  if (GAMES[game].rooms && PLAYER_ID.test(String(body?.player ?? ''))) {
    // Keep the same seat only with a valid ticket for this room; otherwise start a new one.
    const expected = await seatTicket(env.MOQ_GAMES_SIGNING_KEY, game, room, body.player);
    if (sameText(String(body?.ticket ?? ''), expected)) id = body.player;
  }
  const callsign = callsignFor(id);
  const now = Math.floor(Date.now() / 1000);
  const scope = `${root}/${game}/${room}`;
  // The older moq-token claim form (put/get prefixes), which every relay version accepts:
  // publish only under this player's id, subscribe to the whole room.
  const claims = { root: scope, put: [id], get: [''], iat: now, exp: now + TOKEN_TTL };
  let jwt;
  try {
    jwt = await signJwt(claims, env.MOQ_GAMES_SIGNING_KEY);
  } catch (error) {
    console.error(JSON.stringify({ event: 'game_token_failed', error: String(error?.name ?? 'Error') }));
    return reply({ offline: true, error: 'The live sky is not available right now.' }, 503);
  }
  const ticket = GAMES[game].rooms ? await seatTicket(env.MOQ_GAMES_SIGNING_KEY, game, room, id) : undefined;
  // `now` lets a client line its clock up with the server's, for the shared live pass.
  return reply({ url: `${relay}/${scope}?jwt=${jwt}`, player: id, callsign, room, expires: claims.exp, ticket, now: Date.now() });
}
