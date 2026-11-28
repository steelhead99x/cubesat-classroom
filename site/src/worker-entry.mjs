import { DurableObject } from 'cloudflare:workers';
import { handleRoom } from './games/rockoon-wars/authority.mjs';

export { default } from './worker.mjs';

export class RockoonRoom extends DurableObject {
  async request(input) {
    const { storage } = this.ctx;
    const previous = storage.kv.get('room');
    const before = JSON.stringify(previous);
    const { room, ...result } = handleRoom(previous, input, Date.now());
    // The SQLite KV API uses structured clone, preserving the Scout's infinite ammo.
    // No await between reading and writing the one room record.
    if (room && JSON.stringify(room) !== before) storage.kv.put('room', room);
    else if (!room && previous) storage.kv.delete('room');
    if (input.kind === 'create' && result.status === 200) await storage.setAlarm(room.expires);
    return result;
  }

  async alarm() {
    const { storage } = this.ctx;
    const room = storage.kv.get('room');
    if (!room) return;
    if (room.expires <= Date.now()) storage.kv.delete('room');
    else await storage.setAlarm(room.expires);
  }
}
