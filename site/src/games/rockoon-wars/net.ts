// Rockoon Battle rooms over Media over QUIC (MoQ).
//
// The site's Worker hands out a short-lived token (POST /api/games/moq-token) scoped to one room,
// hikd/games/rockoon-battle-v6/<CODE>, that lets this player publish only under its own id. Each
// player publishes one broadcast per relay session, <id>/<session>, with one track, `state`: its
// whole snapshot (see room.mjs),
// rewritten every few seconds so players who join late or reconnect catch up. Everyone replays
// the same snapshots, so nothing but angles, powers, weapons and shop lists crosses the relay.
import * as Moq from '@moq/net';
import { cleanSnapshot, ROOM_NAMESPACE } from './room.mjs';

export type LinkStatus = 'off' | 'connecting' | 'live' | 'offline';
export interface LinkEvents {
  status(status: LinkStatus, detail?: string): void;
  peer(id: string, snapshot: Snapshot): void;
  leave(id: string): void;
}
// The shape room.mjs's cleanSnapshot returns.
export type Snapshot = NonNullable<ReturnType<typeof cleanSnapshot>>;

interface Token { url: string; player: string; callsign: string; expires: number; ticket?: string }

const HEARTBEAT_MS = 3_000;
const PLAYER_ID = /^[0-9a-f]{12}$/;
const SESSION_ID = /^[0-9a-f]{8}$/;
/** A fresh id for one relay session's broadcast. */
const sessionId = () => [...crypto.getRandomValues(new Uint8Array(4))].map(b => b.toString(16).padStart(2, '0')).join('');

export class RoomLink {
  status: LinkStatus = 'off';
  callsign = '';
  player = '';
  readonly room: string;
  #ticket = '';
  #events: LinkEvents;
  #endpoint: string;
  #stop = false;
  #connection?: Moq.Connection.Established;
  #closed = false;
  #origin?: Moq.Origin.Producer;
  #track?: Moq.Track.Producer;
  /** Followed broadcasts by <player>/<session>. */
  #peers = new Map<string, { id: string; close: () => void }>();
  #snapshot: unknown = null;
  #beat?: ReturnType<typeof setInterval>;
  #retry = 0;
  /** Each start() is one attempt; stop() or a newer start() makes older attempts give up. */
  #gen = 0;
  #abort?: AbortController;
  #ready?: (ok: boolean) => void;

  /** `seat` is a seat this tab held in this room before a reload (see seat.mjs). The Worker keeps
   * it only if the ticket is valid for this room; otherwise this player gets a new id. */
  constructor(room: string, events: LinkEvents, endpoint = '/api/games/moq-token', seat?: { player: string; ticket: string }) {
    this.room = room;
    this.#events = events;
    this.#endpoint = endpoint;
    if (seat) { this.player = seat.player; this.#ticket = seat.ticket; }
  }

  /** This player's seat ticket, known once connected: with `player`, it keeps the seat. */
  get ticket() { return this.#ticket; }

  #set(status: LinkStatus, detail?: string) {
    this.status = status;
    if (status === 'live') { this.#ready?.(true); this.#ready = undefined; }
    if (status === 'offline' || status === 'off') { this.#ready?.(false); this.#ready = undefined; }
    // A problem drawing the page must not tear down the relay session.
    try { this.#events.status(status, detail); } catch (error) { console.warn('room status handler failed', error); }
  }

  /** Resolves true once connected (the player id and call sign are then known), false if not. */
  start(): Promise<boolean> {
    this.#ready?.(false);
    const ready = new Promise<boolean>(r => { this.#ready = r; });
    void this.#run();
    return ready;
  }

  async #run() {
    const gen = this.#begin();
    const live = () => !this.#stop && gen === this.#gen;
    while (live()) {
      this.#set('connecting');
      let token: Token;
      try {
        token = await this.#token();
      } catch (error) {
        if (!live()) return; // cancelled while asking for a token
        this.#set('offline', (error as Error).message);
        return;
      }
      if (!live()) return;
      try {
        const expiresIn = token.expires * 1000 - Date.now();
        await this.#session(token, Math.max(30_000, expiresIn - 30_000), live);
        if (!live()) return;
        this.#retry = 0;
      } catch {
        if (!live()) return;
        this.#retry++;
        this.#cleanup();
        if (this.#retry > 3) { this.#set('offline', 'The relay could not be reached.'); return; }
        this.#set('connecting', 'Reconnecting…');
        await new Promise(r => setTimeout(r, 1000 * this.#retry));
      }
    }
  }

  /** Starts a new attempt and cancels any older one, including its token request. */
  #begin() {
    const generation = ++this.#gen;
    this.#abort?.abort();
    this.#cleanup();
    this.#abort = new AbortController();
    this.#retry = 0;
    this.#stop = false;
    return generation;
  }

  stop() {
    this.#stop = true;
    this.#gen++;
    this.#abort?.abort();
    this.#cleanup();
    this.#set('off');
  }

  async #token(): Promise<Token> {
    // After the first token (or after a reload, with the saved seat), keep the same seat (player
    // id) with the Worker's seat ticket.
    const seat = this.player && this.#ticket ? { player: this.player, ticket: this.#ticket } : {};
    const res = await fetch(this.#endpoint, {
      method: 'POST',
      headers: { 'content-type': 'application/json' },
      body: JSON.stringify({ game: ROOM_NAMESPACE, room: this.room, ...seat }),
      credentials: 'same-origin',
      cache: 'no-store',
      signal: this.#abort?.signal,
    });
    const body = await res.json().catch(() => ({}));
    if (!res.ok || typeof body.url !== 'string') throw new Error(body.error ?? 'Online play is not available.');
    if (new URL(body.url).protocol !== 'https:') throw new Error('The relay must use https.');
    return body as Token;
  }

  async #session(token: Token, lifetimeMs: number, live: () => boolean) {
    this.player = token.player;
    this.callsign = token.callsign;
    this.#ticket = token.ticket ?? '';
    const origin = new Moq.Origin.Producer();
    const connection = await Moq.Connection.connect({
      url: new URL(token.url), publish: origin.consume(), consume: origin, signal: AbortSignal.timeout(6_000),
    }).catch(error => { try { origin.close(); } catch { /* already closed */ } throw error; });
    if (!live()) {
      // The player left (or started again) while this connection was opening: drop it.
      try { connection.close(); } catch { /* already closed */ }
      try { origin.close(); } catch { /* already closed */ }
      return;
    }
    this.#origin = origin;
    this.#connection = connection;
    this.#closed = false;
    const markClosed = () => {
      if (this.#connection === connection) this.#closed = true;
    };
    void connection.closed.then(markClosed, markClosed);
    // Each session publishes under a path of its own, <player>/<session>. Reusing the bare player
    // id meant that a player who came back (after a reload, a dropped connection or a token
    // renewal) announced the same path again before the relay had retracted the old session's
    // copy. Every other player's MoQ client takes a second announce for a live path as a protocol
    // violation and closes its whole session; its reconnect then did the same to the first
    // player, and the room flickered for 15 to 50 seconds. The token lets a player publish under
    // its id and anything beneath it.
    const broadcast = origin.createBroadcast(Moq.Path.from(token.player, sessionId()));
    this.#track = broadcast.createTrack('state', { priority: 1 });
    broadcast.announce();
    this.#set('live');
    this.#write();
    this.#beat = setInterval(() => this.#write(), HEARTBEAT_MS);
    void this.#discover(origin, token.player);
    let timeout: ReturnType<typeof setTimeout> | undefined;
    const renewal = new Promise<'renew'>(resolve => {
      timeout = setTimeout(() => resolve('renew'), lifetimeMs);
    });
    let result;
    try {
      result = await Promise.race([connection.closed, renewal]);
    } finally {
      clearTimeout(timeout);
      // A stop or a newer start may have installed a different connection while we waited.
      if (this.#connection === connection) this.#cleanup();
    }
    if (result !== 'renew' && result) throw result;
  }

  async #discover(origin: Moq.Origin.Producer, self: string) {
    for await (const update of origin.announced()) {
      if (this.#origin !== origin) return;
      // <player>/<session>: a player who reconnects briefly has two, the old one retracting.
      const [id = '', session = '', ...rest] = Moq.Path.parts(update.prefix);
      if (!PLAYER_ID.test(id) || !SESSION_ID.test(session) || rest.length || id === self) continue;
      const key = `${id}/${session}`;
      if (update.kind === 'retracted') {
        this.#peers.get(key)?.close();
        this.#peers.delete(key);
        // Gone only when none of that player's sessions is left.
        if (![...this.#peers.values()].some(p => p.id === id)) this.#events.leave(id);
        continue;
      }
      if (!this.#peers.has(key)) this.#follow(origin, update.prefix, key, id);
    }
  }

  #follow(origin: Moq.Origin.Producer, path: Moq.Path.Valid, key: string, id: string) {
    const request = origin.request(path);
    const subs: { close(): void }[] = [];
    const stopWatch = request.active.subscribe(broadcast => {
      for (const s of subs.splice(0)) s.close();
      if (!broadcast) return;
      const state = broadcast.track('state').subscribe({ priority: 1 }).ordered();
      subs.push(state);
      void (async () => {
        for (;;) {
          const msg = await state.readJson().catch(() => undefined);
          if (msg === undefined || this.#origin !== origin || !subs.includes(state)) break;
          const clean = cleanSnapshot(msg);
          if (clean) try { this.#events.peer(id, clean as Snapshot); } catch (error) { console.warn('room peer handler failed', error); }
        }
      })();
    });
    this.#peers.set(key, { id, close: () => { stopWatch(); for (const s of subs.splice(0)) s.close(); request.close(); } });
  }

  /** Sets this player's snapshot and sends it now (and again with every heartbeat). */
  publish(snapshot: unknown) {
    this.#snapshot = snapshot;
    this.#write();
  }

  #write() {
    if (this.#track && this.status === 'live' && this.#snapshot) {
      try { this.#track.writeJson(this.#snapshot); } catch { /* the session is closing */ }
    }
  }

  #cleanup() {
    clearInterval(this.#beat);
    const ids = new Set([...this.#peers.values()].map(p => p.id));
    for (const p of this.#peers.values()) p.close();
    this.#peers.clear();
    for (const id of ids) this.#events.leave(id);
    try { this.#track?.close(); } catch { /* already closed */ }
    // Closing a session the relay already closed makes the library log an error, so skip it.
    if (!this.#closed) try { this.#connection?.close(); } catch { /* already closed */ }
    try { this.#origin?.close(); } catch { /* already closed */ }
    this.#track = undefined;
    this.#connection = undefined;
    this.#origin = undefined;
  }
}
