// Same-origin link to the room's authoritative history. Each tab serializes its own requests,
// keeping a command queued until the Worker replies. The room checks the phase of every retry.
import type { NextAction } from './room.mjs';

export type LinkStatus = 'off' | 'connecting' | 'live' | 'offline';
export interface RoomState {
  creator: string;
  people: { player: string; callsign: string }[];
  host: { order: string[]; names: Record<string, string>; rounds: number; seed: number } | null;
  actions: Extract<NextAction, { kind: 'fire' | 'skip' | 'shop' }>[];
  waiting: Extract<NextAction, { kind: 'wait' | 'over' }>;
  rising: string | null;
  deadline: number | null;
}

type Command =
  | { kind: 'start' }
  | { kind: 'rise'; version: number }
  | { kind: 'fire'; version: number; shot: { angle: number; power: number; weapon: string } }
  | { kind: 'shop'; version: number; buys: string[] };

type Seat = { player: string; ticket: string; callsign: string };
type Reply = { state: RoomState; seat?: Seat; error?: string };
type Events = { status(status: LinkStatus, detail?: string): void; state(state: RoomState): void };
const POLL_MS = 2_500;

export class RoomLink {
  status: LinkStatus = 'off';
  player = '';
  callsign = '';
  ticket = '';
  readonly room: string;
  #events: Events;
  #abort = new AbortController();
  #timer?: ReturnType<typeof setTimeout>;
  #queue: Command[] = [];
  #busy = false;

  constructor(room: string, events: Events) {
    this.room = room;
    this.#events = events;
  }

  #set(status: LinkStatus, detail?: string) {
    if (this.status === status) return;
    this.status = status;
    this.#events.status(status, detail);
  }

  async #request(input: Command | { kind: 'create' | 'join' | 'sync' }): Promise<Reply> {
    const response = await fetch('/api/games/rockoon-room', {
      method: 'POST', headers: { 'content-type': 'application/json' },
      body: JSON.stringify({ ...input, room: this.room, ticket: this.ticket }),
      credentials: 'same-origin', cache: 'no-store',
      signal: AbortSignal.any([this.#abort.signal, AbortSignal.timeout(8_000)]),
    });
    const body = await response.json().catch(() => null);
    if (!response.ok || !body?.state) {
      const error = new Error(body?.error ?? 'We could not reach the game server.');
      if (response.status === 403 || response.status === 404 || response.status === 405) error.name = 'RoomClosed';
      throw error;
    }
    return body;
  }

  async start(create: boolean, ticket = '') {
    this.ticket = ticket;
    this.#set('connecting');
    try {
      const { seat, state } = await this.#request({ kind: create ? 'create' : 'join' });
      if (!seat) throw new Error('The game server did not provide a seat.');
      if (this.#abort.signal.aborted) return null;
      this.player = seat.player;
      this.ticket = seat.ticket;
      this.callsign = seat.callsign;
      this.#set('live');
      this.#schedule();
      return state;
    } catch (error) {
      if (!this.#abort.signal.aborted) this.#set('offline', (error as Error).message);
      return null;
    }
  }

  stop() {
    this.#abort.abort();
    clearTimeout(this.#timer);
  }

  send(command: Command) {
    if (this.status !== 'live' || this.#abort.signal.aborted) return;
    if (!this.#queue.some(queued => queued.kind === command.kind && JSON.stringify(queued) === JSON.stringify(command))) {
      this.#queue.push(command);
    }
    void this.#tick();
  }

  pending(kind: Command['kind'], version: number) {
    return this.#queue.some(command => command.kind === kind && 'version' in command && command.version === version);
  }

  #schedule() {
    this.#timer = setTimeout(() => void this.#tick(), POLL_MS);
  }

  async #tick() {
    if (this.#busy || this.#abort.signal.aborted) return;
    clearTimeout(this.#timer);
    this.#busy = true;
    let retry = true;
    try {
      do {
        const command = this.#queue[0];
        const { state } = await this.#request(command ?? { kind: 'sync' });
        if (this.#abort.signal.aborted) return;
        if (command) this.#queue.shift();
        this.#set('live');
        this.#events.state(state);
      } while (this.#queue.length && !this.#abort.signal.aborted);
    } catch (error) {
      if (this.#abort.signal.aborted) return;
      retry = (error as Error).name !== 'RoomClosed';
      this.#set(retry ? 'connecting' : 'offline', retry ? 'Reconnecting…' : (error as Error).message);
    } finally {
      this.#busy = false;
      if (retry && !this.#abort.signal.aborted) this.#schedule();
    }
  }
}
