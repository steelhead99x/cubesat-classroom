// The live sky over Media over QUIC (MoQ).
//
// The site's Worker hands out a short-lived token (POST /api/games/moq-token) scoped to one
// game room on the relay. Each player publishes one broadcast, named by its anonymous player id,
// with two tracks:
//   station  ~5 times a second: where the antenna points, SF, signal, packets and score
//   packets  one frame for every packet the station decodes, with the satellite's readings
// Every player subscribes to the other stations in the room, so all classrooms share one sky.
// Without a token (offline, not configured, relay unreachable) the game keeps working alone.
import * as Moq from '@moq/net';

export interface StationState {
  cs: string; az: number; el: number; sf: number; snr: number; locked: boolean;
  rx: number; score: number; slot: number;
}
export interface PacketNote { seq: number; sf: number; snr: number; tempC: number; light: number; battery: number; slot: number }
export type SkyStatus = 'off' | 'connecting' | 'live' | 'offline';

export interface SkyEvents {
  status(status: SkyStatus, detail?: string): void;
  station(id: string, state: StationState): void;
  packet(id: string, note: PacketNote): void;
  leave(id: string): void;
}

interface Token { url: string; player: string; callsign: string; expires: number; now?: number }

/** Accepts only well-formed station updates from the relay; anything else is dropped. */
export function cleanStation(raw: unknown): StationState | null {
  if (!raw || typeof raw !== 'object') return null;
  const r = raw as Record<string, unknown>;
  const num = (v: unknown, lo: number, hi: number) => (typeof v === 'number' && Number.isFinite(v) ? Math.max(lo, Math.min(hi, v)) : null);
  const cs = typeof r.cs === 'string' && /^GS-[A-Z]{2}\d$/.test(r.cs) ? r.cs : null;
  const az = num(r.az, -90, 90), el = num(r.el, 0, 90), sf = num(r.sf, 7, 12), snr = num(r.snr, -99, 40);
  const rx = num(r.rx, 0, 1e4), score = num(r.score, 0, 1e7), slot = num(r.slot, 0, 1e12);
  if (!cs || az === null || el === null || sf === null || snr === null || rx === null || score === null || slot === null) return null;
  return { cs, az, el, sf, snr, locked: r.locked === true, rx, score, slot };
}

export function cleanPacket(raw: unknown): PacketNote | null {
  if (!raw || typeof raw !== 'object') return null;
  const r = raw as Record<string, unknown>;
  const keys = ['seq', 'sf', 'snr', 'tempC', 'light', 'battery', 'slot'] as const;
  if (!keys.every(k => typeof r[k] === 'number' && Number.isFinite(r[k]))) return null;
  return Object.fromEntries(keys.map(k => [k, r[k]])) as unknown as PacketNote;
}

export class LiveSky {
  status: SkyStatus = 'off';
  callsign = '';
  player = '';
  /** Server clock minus this computer's clock, in ms, so every station flies the same pass. */
  clockOffset = 0;
  #events: SkyEvents;
  #endpoint: string;
  #stop = false;
  #connection?: Moq.Connection.Established;
  #origin?: Moq.Origin.Producer;
  #station?: Moq.Track.Producer;
  #packets?: Moq.Track.Producer;
  #peers = new Map<string, () => void>();
  #lastSent = 0;
  #retry = 0;
  /** Each start() is one attempt; stop() or a newer start() makes older attempts give up. */
  #gen = 0;
  #abort?: AbortController;

  constructor(events: SkyEvents, endpoint = '/api/games/moq-token') {
    this.#events = events;
    this.#endpoint = endpoint;
  }

  #set(status: SkyStatus, detail?: string) {
    this.status = status;
    this.#events.status(status, detail);
  }

  async start() {
    const gen = this.#begin();
    const live = () => !this.#stop && gen === this.#gen;
    while (live()) {
      this.#set('connecting');
      let token: Token;
      try {
        token = await this.#token();
      } catch (error) {
        if (!live()) return; // cancelled while asking for a token
        // Not set up, rate limited or offline: play alone and do not keep asking.
        this.#set('offline', (error as Error).message);
        return;
      }
      if (!live()) return;
      try {
        const expiresIn = token.expires * 1000 - Date.now();
        await this.#session(token, Math.max(30_000, expiresIn - 30_000), live);
        this.#retry = 0;
      } catch (error) {
        if (!live()) return;
        this.#retry++;
        this.#cleanup();
        if (this.#retry > 2) { this.#set('offline', 'The relay could not be reached.'); return; }
        this.#set('connecting', 'Reconnecting…');
        await new Promise(r => setTimeout(r, 1000 * this.#retry));
        void error;
      }
    }
  }

  /** Starts a new attempt and cancels any older one, including its token request. */
  #begin() {
    this.#abort?.abort();
    this.#abort = new AbortController();
    this.#stop = false;
    return ++this.#gen;
  }

  stop() {
    this.#stop = true;
    this.#gen++;
    this.#abort?.abort();
    this.#cleanup();
    this.#set('off');
  }

  async #token(): Promise<Token> {
    const sent = Date.now();
    const res = await fetch(this.#endpoint, {
      method: 'POST',
      headers: { 'content-type': 'application/json' },
      body: JSON.stringify({ game: 'catch-the-pass' }),
      credentials: 'same-origin',
      cache: 'no-store',
      signal: this.#abort?.signal,
    });
    const body = await res.json().catch(() => ({}));
    if (!res.ok || typeof body.url !== 'string') throw new Error(body.error ?? 'The live sky is not available.');
    const url = new URL(body.url);
    if (url.protocol !== 'https:') throw new Error('The relay must use https.');
    // A classroom computer's clock can be minutes off. Take the server's time at the middle
    // of the request (ignoring anything absurd).
    const back = Date.now();
    if (Number.isFinite(body.now) && back - sent < 10_000) {
      const offset = body.now - (sent + back) / 2;
      if (Math.abs(offset) < 86_400_000) this.clockOffset = Math.round(offset);
    }
    return body as Token;
  }

  /** One relay session, until it closes or the token is about to expire. */
  async #session(token: Token, lifetimeMs: number, live: () => boolean) {
    this.player = token.player;
    this.callsign = token.callsign;
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
    const broadcast = origin.createBroadcast(Moq.Path.from(token.player));
    this.#station = broadcast.createTrack('station', { priority: 1 });
    this.#packets = broadcast.createTrack('packets', { priority: 2 });
    broadcast.announce();
    this.#set('live');
    void this.#discover(origin, token.player);
    const timer = new Promise<'renew'>(r => setTimeout(() => r('renew'), lifetimeMs));
    const result = await Promise.race([connection.closed, timer]);
    this.#cleanup();
    if (result !== 'renew' && result) throw result;
  }

  async #discover(origin: Moq.Origin.Producer, self: string) {
    const announced = origin.announced();
    for await (const update of announced) {
      const id = Moq.Path.parts(update.prefix)[0] ?? '';
      if (!/^[0-9a-f]{12}$/.test(id) || id === self) continue;
      if (update.kind === 'retracted') {
        this.#peers.get(id)?.();
        this.#peers.delete(id);
        this.#events.leave(id);
        continue;
      }
      if (this.#peers.has(id)) continue;
      this.#follow(origin, id);
    }
  }

  #follow(origin: Moq.Origin.Producer, id: string) {
    const request = origin.request(Moq.Path.from(id));
    const subs: { close(): void }[] = [];
    const stopWatch = request.active.subscribe(broadcast => {
      for (const s of subs.splice(0)) s.close();
      if (!broadcast) return;
      const station = broadcast.track('station').subscribe({ priority: 1 }).ordered();
      const packets = broadcast.track('packets').subscribe({ priority: 2 }).ordered();
      subs.push(station, packets);
      void (async () => {
        for (;;) {
          const msg = await station.readJson().catch(() => undefined);
          if (msg === undefined) break;
          const clean = cleanStation(msg);
          if (clean) this.#events.station(id, clean);
        }
      })();
      void (async () => {
        for (;;) {
          const msg = await packets.readJson().catch(() => undefined);
          if (msg === undefined) break;
          const clean = cleanPacket(msg);
          if (clean) this.#events.packet(id, clean);
        }
      })();
    });
    this.#peers.set(id, () => { stopWatch(); for (const s of subs.splice(0)) s.close(); request.close(); });
  }

  /** Publishes this station's state, at most five times a second. */
  sendStation(state: Omit<StationState, 'cs'>, force = false) {
    if (!this.#station || this.status !== 'live') return;
    const now = performance.now();
    if (!force && now - this.#lastSent < 200) return;
    this.#lastSent = now;
    this.#station.writeJson({ ...state, cs: this.callsign });
  }

  sendPacket(note: PacketNote) {
    if (this.#packets && this.status === 'live') this.#packets.writeJson(note);
  }

  #cleanup() {
    for (const [id, close] of this.#peers) { close(); this.#events.leave(id); }
    this.#peers.clear();
    try { this.#station?.close(); this.#packets?.close(); } catch { /* already closed */ }
    try { this.#connection?.close(); } catch { /* already closed */ }
    try { this.#origin?.close(); } catch { /* already closed */ }
    this.#station = this.#packets = undefined;
    this.#connection = undefined;
    this.#origin = undefined;
  }
}
