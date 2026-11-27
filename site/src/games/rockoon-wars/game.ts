// Rockoon Wars: the page's game. Rules live in logic.mjs, online rooms in room.mjs and net.ts.
// Hot-seat and online play share one path: every move is written into a player's snapshot and
// room.mjs decides what happens next, so both modes run the very same rules.
import * as G from './logic.mjs';
import * as Room from './room.mjs';
import type { RoomLink, LinkStatus, Snapshot } from './net';

type Snap = { cs: string; joined: number; creator?: boolean; turns: Record<number, { angle: number; power: number; weapon: string }>; shops: Record<number, string[]>; host?: Host; skips?: number[]; shopSkips?: Record<number, string[]> };
type Host = { seed: number; rounds: number; order: string[]; names: Record<string, string> };
type Match = ReturnType<typeof G.createMatch>;
type Field = NonNullable<Match['field']>;

const root = document.querySelector<HTMLElement>('[data-rw]');
if (root) start(root);

function start(root: HTMLElement) {
  const $ = <T extends HTMLElement>(sel: string) => root.querySelector<T>(sel)!;
  const canvas = $<HTMLCanvasElement>('canvas');
  const ctx = canvas.getContext('2d')!;
  const overlay = $<HTMLDivElement>('[data-overlay]');
  const status = $<HTMLParagraphElement>('[data-status]');
  const net = $<HTMLParagraphElement>('[data-net]');
  const hud = {
    who: $('[data-who]'), angle: $('[data-angle]'), power: $('[data-power]'), weapon: $('[data-weapon]'),
    wind: $('[data-wind]'), round: $('[data-round]'), board: $('[data-board]'), controls: $('[data-controls]'),
  };
  const pauseBtn = $<HTMLButtonElement>('[data-pause]');
  const soundBtn = $<HTMLButtonElement>('[data-sound]');
  const quitBtn = $<HTMLButtonElement>('[data-quit]');
  const reduced = matchMedia('(prefers-reduced-motion: reduce)').matches;

  const dpr = Math.min(2, window.devicePixelRatio || 1);
  canvas.width = G.W * dpr; canvas.height = G.H * dpr;
  ctx.scale(dpr, dpr);

  // ---- state ----
  let mode: 'menu' | 'local' | 'online' = 'menu';
  let host: Host | null = null;
  let match: Match | null = null;
  const snaps = new Map<string, Snap>();
  const seen = new Map<string, number>();
  let mine: string[] = []; // the player ids this screen controls
  let link: RoomLink | null = null;
  let me = '';
  let paused = false;
  let anim: null | { path: number[][]; i: number; before: Field; blast: { x: number; y: number; r: number; dirt: boolean } | null; boom: number; events: { type: string; who?: number; winner?: number }[] } = null;
  let banner: null | { title: string; text: string } = null; // round or match results, until dismissed
  let shopPick: Record<string, string[]> = {}; // purchases being made in the shop, by player id
  let announced = '';

  // ---- sound (off until asked for) ----
  let audio: AudioContext | null = null;
  let sound = false;
  function beep(kind: 'fire' | 'boom') {
    if (!sound) return;
    audio ??= new AudioContext();
    const t = audio.currentTime, osc = audio.createOscillator(), gain = audio.createGain();
    osc.type = kind === 'fire' ? 'triangle' : 'sawtooth';
    osc.frequency.setValueAtTime(kind === 'fire' ? 520 : 110, t);
    osc.frequency.exponentialRampToValueAtTime(kind === 'fire' ? 180 : 40, t + (kind === 'fire' ? 0.25 : 0.5));
    gain.gain.setValueAtTime(0.12, t);
    gain.gain.exponentialRampToValueAtTime(0.001, t + (kind === 'fire' ? 0.25 : 0.5));
    osc.connect(gain).connect(audio.destination);
    osc.start(t); osc.stop(t + 0.55);
  }
  soundBtn.addEventListener('click', () => toggleSound());
  function toggleSound() {
    sound = !sound;
    soundBtn.setAttribute('aria-pressed', String(sound));
    soundBtn.textContent = sound ? 'Sound on' : 'Sound off';
    if (sound) void (audio ??= new AudioContext()).resume();
  }
  pauseBtn.addEventListener('click', () => togglePause());
  function togglePause() {
    if (mode === 'menu') return;
    paused = !paused;
    pauseBtn.setAttribute('aria-pressed', String(paused));
    pauseBtn.textContent = paused ? 'Resume' : 'Pause';
    say(paused ? 'Paused.' : 'Resumed.');
    render();
  }
  quitBtn.addEventListener('click', () => quit());

  function say(text: string) { if (text !== announced) { announced = text; status.textContent = text; } }
  const esc = (s: string) => s.replace(/[&<>"']/g, c => `&#${c.charCodeAt(0)};`);

  // ---- the shared move path ----
  const snapOf = (id: string) => snaps.get(id)!;
  function publish() {
    if (mode === 'online' && link) link.publish(snapOf(me));
  }
  const waitingOn = (): ReturnType<typeof Room.nextAction> | null => (match && host ? Room.nextAction(match, host, snaps) : null);
  const myTurn = () => {
    const a = waitingOn();
    return !!a && a.kind === 'wait' && match!.phase === 'play' && mine.includes(a.ids[0]);
  };
  const shooter = () => match!.field!.units[match!.turn];

  /** Runs every action that is ready. Shots animate one at a time. */
  function step() {
    if (!match || !host || anim || paused) return render();
    for (let guard = 0; guard < 50; guard++) {
      const action = Room.nextAction(match, host, snaps);
      if (action.kind === 'fire') {
        const before = structuredClone(match.field!) as Field;
        before.units[match.turn].angle = action.shot.angle;
        const result = Room.apply(match, action) as ReturnType<typeof G.fire>;
        anim = { path: result.path, i: 0, before, blast: result.blast, boom: 0, events: result.events };
        beep('fire');
        return render();
      }
      if (action.kind === 'shop' || action.kind === 'skip') { Room.apply(match, action); shopPick = {}; continue; }
      if (action.kind === 'wait') hostSkips(action.ids);
      break;
    }
    render();
  }

  /** Only the room's host skips players who have gone, so everyone skips the same turn. */
  function hostSkips(ids: string[]) {
    if (mode !== 'online' || !host || host.order[0] !== me || !match) return;
    const now = Date.now();
    const gone = ids.filter(id => id !== me && now - (seen.get(id) ?? 0) > 30_000);
    if (!gone.length) return;
    const s = snapOf(me);
    if (match.phase === 'shop') {
      const r = match.round + 1;
      s.shopSkips = { ...s.shopSkips, [r]: [...new Set([...(s.shopSkips?.[r] ?? []), ...gone])] };
    } else if (gone.includes(host.order[match.turn])) {
      s.skips = [...(s.skips ?? []), match.log.length];
    } else return;
    publish();
    step();
  }

  function fireNow() {
    if (!match || paused || anim || banner || !myTurn()) return;
    const u = shooter(), id = host!.order[match.turn];
    snapOf(id).turns[match.log.length] = { angle: u.angle, power: u.power, weapon: u.weapon };
    publish();
    step();
  }

  // ---- the animation loop ----
  let last = 0;
  function frame(t: number) {
    requestAnimationFrame(frame);
    if (!anim || paused) return;
    if (t - last < 14) return;
    last = t;
    if (anim.i < anim.path.length - 1) {
      anim.i = reduced ? anim.path.length - 1 : Math.min(anim.path.length - 1, anim.i + 3);
      if (anim.i === anim.path.length - 1 && anim.blast) beep('boom');
    } else if (anim.blast && anim.boom < 1) {
      anim.boom = reduced ? 1 : anim.boom + 0.08;
    } else {
      finishShot();
      return;
    }
    draw();
  }
  requestAnimationFrame(frame);

  function finishShot() {
    const events = anim!.events;
    anim = null;
    const names = (i: number) => match!.players[i]?.name ?? '';
    const said: string[] = [];
    for (const e of events) {
      if (e.type === 'kill') said.push(`${names(e.who!)} is out!`);
      if (e.type === 'round') {
        const over = match!.phase === 'over';
        const winner = e.winner! >= 0 ? `${names(e.winner!)} wins round ${match!.round}!` : `Round ${match!.round} is a draw.`;
        banner = over
          ? { title: `${names(match!.winner!)} wins the match!`, text: winner }
          : { title: winner, text: 'Everyone gets $100. The winner gets $300 more and 2 points. Time to shop.' };
      }
    }
    const hits = events.filter(e => e.type === 'hit').length;
    say(said.join(' ') || (hits ? 'Hit!' : 'Missed.'));
    step();
  }

  // ---- drawing ----
  const SKY = ctx.createLinearGradient(0, 0, 0, G.H);
  SKY.addColorStop(0, '#081028'); SKY.addColorStop(0.7, '#1d2b6b'); SKY.addColorStop(1, '#3a3f8f');
  const stars = Array.from({ length: 60 }, (_, i) => [(i * 97) % G.W, (i * 53) % 220]);

  function draw() {
    ctx.fillStyle = SKY; ctx.fillRect(0, 0, G.W, G.H);
    ctx.fillStyle = '#c9d2ff';
    for (const [x, y] of stars) ctx.fillRect(x, y, 1, 1);
    if (!match?.field) return;
    const field = anim ? anim.before : match.field;
    const sy = (y: number) => G.H - y;
    // Ground
    ctx.beginPath(); ctx.moveTo(0, G.H);
    for (let x = 0; x < G.W; x++) ctx.lineTo(x, sy(field.terrain[x]));
    ctx.lineTo(G.W, G.H); ctx.closePath();
    ctx.fillStyle = '#5a4a2e'; ctx.fill();
    ctx.strokeStyle = '#6fd3a0'; ctx.lineWidth = 3; ctx.stroke();
    // Rockoons
    field.units.forEach((u, i) => {
      if (!u.alive) return;
      const color = G.COLORS[i];
      const turn = !anim && match!.phase === 'play' && match!.turn === i;
      const bx = u.x, by = sy(u.y + 50);
      // balloon and tether
      ctx.strokeStyle = '#e8ecff'; ctx.lineWidth = 1;
      ctx.beginPath(); ctx.moveTo(u.x, sy(u.y + 22)); ctx.lineTo(bx, by + 10); ctx.stroke();
      ctx.fillStyle = '#e8ecff'; ctx.beginPath(); ctx.ellipse(bx, by, 9, 11, 0, 0, Math.PI * 2); ctx.fill();
      ctx.fillStyle = color; ctx.fillRect(bx - 9, by - 2, 18, 3);
      // gondola and mast
      ctx.fillStyle = color; ctx.fillRect(u.x - 8, sy(u.y + 10), 16, 10);
      ctx.fillStyle = '#081028'; ctx.fillRect(u.x - 4, sy(u.y + 7), 8, 3);
      ctx.strokeStyle = color; ctx.lineWidth = 2;
      ctx.beginPath(); ctx.moveTo(u.x, sy(u.y + 10)); ctx.lineTo(u.x, sy(u.y + 22)); ctx.stroke();
      // launch rail
      ctx.lineWidth = 4; ctx.lineCap = 'round';
      ctx.beginPath(); ctx.moveTo(u.x, sy(u.y + 22));
      ctx.lineTo(u.x + G.cosDeg(u.angle) * 14, sy(u.y + 22 + G.sinDeg(u.angle) * 14)); ctx.stroke();
      ctx.lineCap = 'butt';
      // health
      ctx.fillStyle = '#1b2550'; ctx.fillRect(u.x - 14, by - 22, 28, 4);
      ctx.fillStyle = u.hp > 30 ? '#6fd3a0' : '#ef6b6c'; ctx.fillRect(u.x - 14, by - 22, 28 * u.hp / 100, 4);
      if (turn) {
        ctx.fillStyle = '#ffd23f';
        ctx.beginPath(); ctx.moveTo(u.x, by - 26); ctx.lineTo(u.x - 5, by - 34); ctx.lineTo(u.x + 5, by - 34); ctx.fill();
      }
    });
    // Shot in flight, then the blast
    if (anim) {
      const p = anim.path[anim.i];
      ctx.fillStyle = '#ffd23f';
      ctx.beginPath(); ctx.arc(p[0], sy(p[1]), 3, 0, Math.PI * 2); ctx.fill();
      ctx.strokeStyle = 'rgb(255 210 63 / .35)'; ctx.lineWidth = 1; ctx.beginPath();
      for (let k = Math.max(0, anim.i - 40); k <= anim.i; k++) ctx.lineTo(anim.path[k][0], sy(anim.path[k][1]));
      ctx.stroke();
      if (anim.i >= anim.path.length - 1 && anim.blast) {
        const b = anim.blast;
        ctx.fillStyle = b.dirt ? 'rgb(140 100 50 / .9)' : `rgb(255 ${150 - anim.boom * 100} 60 / ${1 - anim.boom * 0.5})`;
        ctx.beginPath(); ctx.arc(b.x, sy(b.y), Math.max(2, b.r * anim.boom), 0, Math.PI * 2); ctx.fill();
      }
    }
    // Wind
    const wind = field.wind;
    ctx.fillStyle = '#e8ecff'; ctx.font = '600 13px system-ui, sans-serif'; ctx.textAlign = 'center';
    ctx.fillText(wind === 0 ? 'No wind' : `Wind ${wind < 0 ? '◀' : ''} ${Math.abs(wind)} ${wind > 0 ? '▶' : ''}`, G.W / 2, 20);
  }

  // ---- the HTML around the canvas ----
  function render() {
    // For checking that online players stay in step: how many turns, and a hash of the match.
    root.dataset.turns = match ? String(match.log.length) : '';
    root.dataset.fp = match && !anim ? G.fingerprint(match) : '';
    draw();
    renderHud();
    renderOverlay();
  }

  function renderHud() {
    const playing = !!match?.field && match.phase === 'play';
    const turn = playing ? match!.turn : -1;
    const u = playing ? shooter() : null;
    const p = turn >= 0 ? match!.players[turn] : null;
    hud.who.innerHTML = p ? `<i style="background:${p.color}"></i>${esc(p.name)}${host && mine.includes(host.order[turn]) && mode === 'online' ? ' (you)' : ''}` : '—';
    hud.angle.textContent = u ? `${u.angle}°` : '—';
    hud.power.textContent = u ? String(u.power) : '—';
    if (u && p) {
      const n = p.ammo[u.weapon as keyof typeof p.ammo];
      hud.weapon.textContent = `${G.WEAPON[u.weapon].name}${n === Infinity ? '' : ` ×${n}`}`;
    } else hud.weapon.textContent = '—';
    hud.wind.textContent = match?.field ? (match.field.wind === 0 ? 'none' : `${Math.abs(match.field.wind)} ${match.field.wind < 0 ? 'left' : 'right'}`) : '—';
    hud.round.textContent = match ? `${Math.max(1, match.round)} of ${match.rounds}` : '—';
    hud.board.innerHTML = match ? match.players.map((pl, i) => {
      const unit = match!.field?.units[i];
      const hp = unit ? (unit.alive ? unit.hp : 0) : 100;
      return `<li><i style="background:${pl.color}"></i><span>${esc(pl.name)}</span><span class="hp" aria-label="health ${hp}"><b style="width:${hp}%"></b></span><span>$${pl.money}</span><span>${pl.score} pts</span></li>`;
    }).join('') : '';
    const canAct = myTurn() && !anim && !paused && !banner;
    for (const b of hud.controls.querySelectorAll('button')) (b as HTMLButtonElement).disabled = !canAct;
    if (playing && !anim && !banner) {
      const a = waitingOn();
      if (a?.kind === 'wait' && !mine.includes(a.ids[0])) say(`Waiting for ${match!.players[match!.turn].name} to fire.`);
      else if (canAct && mode === 'local') say(`${p!.name}'s turn.`);
      else if (canAct) say('Your turn.');
    }
  }

  function renderOverlay() {
    let html = '';
    if (paused) html = `<h3>Paused</h3><div class="row"><button type="button" class="go" data-act="resume">Resume</button></div>`;
    else if (mode === 'menu') html = menuHtml();
    else if (mode === 'online' && !host) html = lobbyHtml();
    else if (banner) html = `<h3>${esc(banner.title)}</h3><p>${esc(banner.text)}</p>${scoresHtml()}<div class="row"><button type="button" class="go" data-act="banner">${match?.phase === 'over' ? 'Back to menu' : 'Continue'}</button></div>`;
    else if (match && !anim && match.phase === 'shop') html = shopHtml();
    overlay.hidden = !html;
    if (overlay.dataset.html !== html) {
      const focused = overlay.contains(document.activeElement) ? (document.activeElement as HTMLElement).dataset.act ?? '' : null;
      overlay.innerHTML = html;
      overlay.dataset.html = html;
      if (html) {
        const target = (focused && overlay.querySelector<HTMLElement>(`[data-act="${focused}"]`)) || overlay.querySelector<HTMLElement>('.go, button, input');
        target?.focus({ preventScroll: true });
      } else if (overlay.contains(document.activeElement) || document.activeElement === document.body) canvas.focus({ preventScroll: true });
    }
  }

  function menuHtml() {
    const code = new URLSearchParams(location.hash.slice(1)).get('room') ?? '';
    return `<h3>Rockoon Wars</h3>
      <p>Take turns launching rockets from balloon platforms. Set your angle and power, mind the wind, and be the last rockoon flying.</p>
      <div class="row"><label>Players <select data-players>${[2, 3, 4].map(n => `<option>${n}</option>`).join('')}</select></label>
      <label>Rounds <select data-rounds>${[1, 2, 3, 4, 5].map(n => `<option${n === 3 ? ' selected' : ''}>${n}</option>`).join('')}</select></label></div>
      <div class="row"><button type="button" class="go" data-act="local">Play on this computer</button></div>
      <p class="small">Or play online, each player on their own screen:</p>
      <div class="row"><button type="button" data-act="create">Create a room</button>
      <label>Room code <input data-code maxlength="5" autocomplete="off" spellcheck="false" value="${esc(code)}" pattern="[A-Za-z2-9]{5}" /></label>
      <button type="button" data-act="join">Join</button></div>
      <p class="small" data-menu-note>${esc(menuNote)}</p>`;
  }
  let menuNote = '';

  function present() {
    const now = Date.now();
    return [...snaps.entries()].filter(([id]) => id === me || now - (seen.get(id) ?? 0) < 10_000)
      .sort((a, b) => a[1].joined - b[1].joined || (a[0] < b[0] ? -1 : 1));
  }

  function lobbyHtml() {
    if (!link || link.status !== 'live') {
      const offline = link?.status === 'offline';
      return offline
        ? `<h3>Online play is not available</h3><p>${esc(netDetail || 'The relay could not be reached.')}</p><div class="row"><button type="button" class="go" data-act="quit">Play on this computer instead</button></div>`
        : `<h3>Connecting…</h3><p>Joining room ${esc(link?.room ?? '')}.</p><div class="row"><button type="button" data-act="quit">Cancel</button></div>`;
    }
    const people = present();
    const creator = snapOf(me).creator;
    const list = people.map(([id, s]) => `<li>${esc(s.cs)}${id === me ? ' (you)' : ''}${s.creator ? ' · host' : ''}</li>`).join('');
    const canStart = creator && people.length >= 2;
    return `<h3>Room <span class="code">${esc(link.room)}</span></h3>
      <p>Share this code. Two to four players can join; anyone after that watches.</p>
      <ol class="people">${list}</ol>
      <div class="row">${creator ? `<label>Rounds <select data-rounds>${[1, 2, 3, 4, 5].map(n => `<option${n === 3 ? ' selected' : ''}>${n}</option>`).join('')}</select></label><button type="button" class="go" data-act="start"${canStart ? '' : ' disabled'}>Start</button>` : '<p>Waiting for the host to start…</p>'}
      <button type="button" data-act="quit">Leave</button></div>`;
  }

  function scoresHtml() {
    return `<ol class="people">${[...match!.players].map((p, i) => ({ p, i })).sort((a, b) => b.p.score - a.p.score)
      .map(({ p }) => `<li>${esc(p.name)}: ${p.score} pts, $${p.money}</li>`).join('')}</ol>`;
  }

  function shopHtml() {
    const a = waitingOn();
    if (a?.kind !== 'wait') return '';
    const id = a.ids.find(x => mine.includes(x));
    if (!id) return `<h3>Shopping</h3><p>Waiting for ${esc(a.ids.map(x => host!.names[x]).join(', '))} to finish shopping.</p>`;
    const i = host!.order.indexOf(id), p = match!.players[i];
    const picks = shopPick[id] ?? [];
    const spent = picks.reduce((s, w) => s + G.WEAPON[w].price, 0);
    const money = p.money - spent;
    const rows = G.WEAPONS.filter(w => w.price).map(w => {
      const have = p.ammo[w.id as keyof typeof p.ammo] + picks.filter(x => x === w.id).length * w.pack;
      return `<li><span>${esc(w.name)}</span><span>${w.pack} for $${w.price}</span><span>have ${have}</span><button type="button" data-act="buy" data-w="${w.id}"${money < w.price ? ' disabled' : ''}>Buy</button></li>`;
    }).join('');
    return `<h3>${esc(p.name)}'s shop · round ${match!.round + 1}</h3><p>Money: $${money}. Baby Rockets are free and never run out.</p>
      <ul class="shop">${rows}</ul>
      <div class="row">${picks.length ? '<button type="button" data-act="undo">Undo</button>' : ''}<button type="button" class="go" data-act="done" data-id="${id}">Done shopping</button></div>`;
  }

  overlay.addEventListener('click', e => {
    const btn = (e.target as HTMLElement).closest<HTMLButtonElement>('button[data-act]');
    if (!btn || btn.disabled) return;
    const act = btn.dataset.act;
    if (act === 'resume') togglePause();
    else if (act === 'local') startLocal();
    else if (act === 'create') void startOnline(Room.newRoomCode(cryptoRandom), true);
    else if (act === 'join') {
      const code = overlay.querySelector<HTMLInputElement>('[data-code]')!.value.trim().toUpperCase();
      if (!Room.ROOM_CODE.test(code)) { menuNote = 'Room codes are five letters and numbers, like K7QX2.'; return renderOverlay(); }
      void startOnline(code, false);
    } else if (act === 'start') hostStart();
    else if (act === 'quit') quit();
    else if (act === 'banner') { banner = null; if (match?.phase === 'over') quit(); else step(); }
    else if (act === 'buy' || act === 'undo') {
      const a = waitingOn();
      const id = a?.kind === 'wait' ? a.ids.find(x => mine.includes(x)) : undefined;
      if (!id) return;
      const list = (shopPick[id] ??= []);
      if (act === 'undo') list.pop(); else list.push(btn.dataset.w!);
      render();
    } else if (act === 'done') {
      const id = btn.dataset.id!;
      snapOf(id).shops[match!.round + 1] = shopPick[id] ?? [];
      publish();
      step();
    }
  });

  const cryptoRandom = () => crypto.getRandomValues(new Uint32Array(1))[0] / 4294967296;

  // ---- starting and leaving ----
  function startLocal() {
    const n = Number(overlay.querySelector<HTMLSelectElement>('[data-players]')?.value ?? 2);
    const rounds = Number(overlay.querySelector<HTMLSelectElement>('[data-rounds]')?.value ?? 3);
    snaps.clear(); seen.clear();
    const order = Array.from({ length: n }, (_, i) => String(i + 1).padStart(12, '0'));
    const names: Record<string, string> = {};
    order.forEach((id, i) => { names[id] = `Player ${i + 1}`; snaps.set(id, { cs: names[id], joined: i, turns: {}, shops: {} }); });
    host = { seed: crypto.getRandomValues(new Uint32Array(1))[0], rounds, order, names };
    mine = order;
    mode = 'local';
    match = Room.matchFor(host);
    banner = null; shopPick = {}; paused = false;
    say('New game. Each player buys weapons, then the first round starts.');
    step();
  }

  let netDetail = '';
  async function startOnline(code: string, creator: boolean) {
    const { RoomLink } = await import('./net').catch(() => ({ RoomLink: null }));
    if (!RoomLink) { menuNote = 'Online play could not load. Play on this computer instead.'; return renderOverlay(); }
    snaps.clear(); seen.clear();
    host = null; match = null; banner = null; shopPick = {}; paused = false;
    mode = 'online';
    history.replaceState(null, '', `#room=${code}`);
    link = new RoomLink(code, {
      status(s: LinkStatus, detail?: string) {
        netDetail = detail ?? '';
        net.textContent = s === 'live' ? `Online · room ${code} · you are ${link?.callsign}` : s === 'connecting' ? (detail || 'Connecting…') : s === 'offline' ? 'Offline' : '';
        render();
      },
      peer(id: string, snap: Snapshot) {
        snaps.set(id, snap as Snap);
        seen.set(id, Date.now());
        adoptHost();
        step();
      },
      leave(id: string) { seen.set(id, 0); render(); },
    });
    const ok = await link.start();
    if (!ok || !link) return;
    me = link.player;
    mine = [me];
    if (!snaps.has(me)) snaps.set(me, { cs: link.callsign, joined: Date.now(), creator, turns: {}, shops: {} });
    publish();
    render();
  }

  /** Follows the room creator's match once it starts. */
  function adoptHost() {
    if (host) return;
    for (const [id, s] of snaps) {
      if (s.host && s.host.order[0] === id && s.creator) {
        host = s.host;
        // Catch up without animating everything that already happened.
        match = Room.replay(host, snaps).match;
        say(host.order.includes(me) ? 'The match has started.' : 'The match has started. You are watching.');
        return;
      }
    }
  }

  function hostStart() {
    const people = present().slice(0, G.MAX_PLAYERS);
    if (people.length < 2) return;
    const rounds = Number(overlay.querySelector<HTMLSelectElement>('[data-rounds]')?.value ?? 3);
    const order = [me, ...people.map(([id]) => id).filter(id => id !== me)].slice(0, G.MAX_PLAYERS);
    const names = Object.fromEntries(order.map(id => [id, snapOf(id).cs]));
    snapOf(me).host = { seed: crypto.getRandomValues(new Uint32Array(1))[0], rounds, order, names };
    snapOf(me).skips = [];
    snapOf(me).shopSkips = {};
    publish();
    adoptHost();
    step();
  }

  function quit() {
    link?.stop(); link = null;
    mode = 'menu'; host = null; match = null; anim = null; banner = null; paused = false;
    snaps.clear(); seen.clear(); mine = []; me = '';
    pauseBtn.setAttribute('aria-pressed', 'false'); pauseBtn.textContent = 'Pause';
    net.textContent = '';
    if (location.hash) history.replaceState(null, '', location.pathname);
    render();
  }

  // Re-check the room now and then: players who have gone, and the host's skips.
  setInterval(() => { if (mode === 'online') { if (!anim) step(); else renderOverlay(); } }, 5_000);

  // ---- controls ----
  function adjust(kind: 'angle' | 'power' | 'weapon', delta: number) {
    if (!match || paused || anim || banner || !myTurn()) return;
    const u = shooter(), p = match.players[match.turn];
    if (kind === 'angle') u.angle = Math.max(0, Math.min(180, u.angle + delta));
    if (kind === 'power') u.power = Math.max(0, Math.min(100, u.power + delta));
    if (kind === 'weapon') {
      const owned = G.WEAPONS.filter(w => p.ammo[w.id as keyof typeof p.ammo] > 0).map(w => w.id);
      const k = owned.indexOf(u.weapon);
      u.weapon = owned[(k + delta + owned.length) % owned.length] ?? 'baby';
      say(G.WEAPON[u.weapon].name);
    }
    render();
  }
  hud.controls.addEventListener('click', e => {
    const btn = (e.target as HTMLElement).closest<HTMLButtonElement>('button');
    if (!btn) return;
    const [kind, d] = (btn.dataset.adj ?? '').split(':');
    if (btn.dataset.fire !== undefined) fireNow();
    else if (kind) adjust(kind as 'angle' | 'power' | 'weapon', Number(d));
  });

  document.addEventListener('keydown', e => {
    const t = e.target as HTMLElement;
    if (t.closest('input, select, textarea') || e.ctrlKey || e.metaKey || e.altKey) return;
    if (mode === 'menu') return;
    const big = e.shiftKey ? 5 : 1;
    let handled = true;
    switch (e.key) {
      case 'ArrowLeft': adjust('angle', big); break;
      case 'ArrowRight': adjust('angle', -big); break;
      case 'ArrowUp': adjust('power', big); break;
      case 'ArrowDown': adjust('power', -big); break;
      case 'q': case 'Q': adjust('weapon', -1); break;
      case 'e': case 'E': adjust('weapon', 1); break;
      case ' ': case 'Enter':
        if (t.closest('button')) { handled = false; break; }
        fireNow(); break;
      case 'p': case 'P': case 'Escape': togglePause(); break;
      case 'm': case 'M': toggleSound(); break;
      default: handled = false;
    }
    if (handled) e.preventDefault();
  });

  render();
}
