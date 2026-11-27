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

/** A picture and a plain-words line for each rocket, for the shop cards and the rocket button. */
export const ROCKET_LOOK: Record<string, { icon: string; say: string }> = {
  baby: { icon: '🔸', say: 'Small boom. Free, never runs out.' },
  rocket: { icon: '🚀', say: 'Bigger boom.' },
  big: { icon: '💥', say: 'Big boom.' },
  mega: { icon: '☄️', say: 'Huge boom!' },
  dirt: { icon: '🟤', say: 'Makes a hill instead of a hole.' },
};
const ROUNDS = 3;
const HOWTO_KEY = 'rockoon-wars:how-to-seen';

const root = document.querySelector<HTMLElement>('[data-rw]');
if (root) start(root);

function start(root: HTMLElement) {
  const $ = <T extends HTMLElement>(sel: string) => root.querySelector<T>(sel)!;
  const canvas = $<HTMLCanvasElement>('canvas');
  const ctx = canvas.getContext('2d')!;
  const stage = $<HTMLDivElement>('.stage');
  const overlay = $<HTMLDivElement>('[data-overlay]');
  const status = $<HTMLParagraphElement>('[data-status]');
  const net = $<HTMLParagraphElement>('[data-net]');
  const turnBanner = $<HTMLParagraphElement>('[data-turn]');
  const hud = {
    angle: $('[data-angle]'), power: $('[data-power]'), rocket: $('[data-rocket]'),
    wind: $('[data-wind]'), round: $('[data-round]'), board: $('[data-board]'), controls: $('[data-controls]'),
  };
  const pauseBtn = $<HTMLButtonElement>('[data-pause]');
  const soundBtn = $<HTMLButtonElement>('[data-sound]');
  const quitBtn = $<HTMLButtonElement>('[data-quit]');
  const helpBtn = $<HTMLButtonElement>('[data-help]');
  const info = $<HTMLElement>('.info');
  const reduced = matchMedia('(prefers-reduced-motion: reduce)').matches;

  const dpr = Math.min(2, window.devicePixelRatio || 1);
  canvas.width = G.W * dpr; canvas.height = G.H * dpr;
  ctx.scale(dpr, dpr);

  // ---- state ----
  let mode: 'menu' | 'local' | 'online' = 'menu';
  let menuStep: 'home' | 'local' | 'online' | 'join' = new URLSearchParams(location.hash.slice(1)).get('room') ? 'join' : 'home';
  let host: Host | null = null;
  let match: Match | null = null;
  const snaps = new Map<string, Snap>();
  const seen = new Map<string, number>();
  let mine: string[] = []; // the player ids this screen controls
  let link: RoomLink | null = null;
  let me = '';
  let paused = false;
  let howto = !seenHowto();
  let anim: null | { path: number[][]; i: number; before: Field; blast: { x: number; y: number; r: number; dirt: boolean } | null; boom: number; events: { type: string; who?: number; winner?: number }[] } = null;
  let banner: null | { title: string; text: string } = null; // round or match results, until dismissed
  let shopPick: Record<string, string[]> = {}; // rockets being bought in the shop, by player id
  let announced = '';
  let menuNote = '';
  let netDetail = '';

  function seenHowto() { try { return localStorage.getItem(HOWTO_KEY) === '1'; } catch { return false; } }
  function closeHowto() { howto = false; try { localStorage.setItem(HOWTO_KEY, '1'); } catch { /* private mode */ } render(); }
  helpBtn.addEventListener('click', () => { howto = true; render(); });

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
    soundBtn.textContent = sound ? '🔊 Sound on' : '🔇 Sound off';
    if (sound) void (audio ??= new AudioContext()).resume();
  }
  pauseBtn.addEventListener('click', () => togglePause());
  function togglePause() {
    if (mode === 'menu') return;
    paused = !paused;
    pauseBtn.setAttribute('aria-pressed', String(paused));
    pauseBtn.textContent = paused ? '▶ Keep playing' : '⏸ Pause';
    say(paused ? 'Paused.' : 'Playing again.');
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

  const canAct = () => myTurn() && !anim && !paused && !banner && !howto;

  function fireNow() {
    if (!match || !canAct()) return;
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
        const winner = e.winner! >= 0 ? `${names(e.winner!)} wins round ${match!.round}!` : `Round ${match!.round} is a tie.`;
        banner = over
          ? { title: `🏆 ${names(match!.winner!)} wins the game!`, text: winner }
          : { title: `🏆 ${winner}`, text: 'Everyone gets 100 credits. The winner gets 300 more credits and 2 points. Time to shop!' };
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
  const sy = (y: number) => G.H - y;

  function draw() {
    ctx.fillStyle = SKY; ctx.fillRect(0, 0, G.W, G.H);
    ctx.fillStyle = '#c9d2ff';
    for (const [x, y] of stars) ctx.fillRect(x, y, 1, 1);
    if (!match?.field) return;
    const field = anim ? anim.before : match.field;
    // Ground
    ctx.beginPath(); ctx.moveTo(0, G.H);
    for (let x = 0; x < G.W; x++) ctx.lineTo(x, sy(field.terrain[x]));
    ctx.lineTo(G.W, G.H); ctx.closePath();
    ctx.fillStyle = '#5a4a2e'; ctx.fill();
    ctx.strokeStyle = '#6fd3a0'; ctx.lineWidth = 3; ctx.stroke();
    // Aim line: the first part of the shot, on your turn
    if (canAct() && match.phase === 'play') {
      ctx.fillStyle = '#ffd23f';
      G.aimPreview(match, 32).forEach(([x, y], k) => { if (k % 3 === 0) { ctx.beginPath(); ctx.arc(x, sy(y), 2.2, 0, Math.PI * 2); ctx.fill(); } });
    }
    // Rockoons
    field.units.forEach((u, i) => {
      if (!u.alive) return;
      const color = G.COLORS[i];
      const turn = !anim && match!.phase === 'play' && match!.turn === i;
      const bx = u.x, by = sy(u.y + 50);
      ctx.strokeStyle = '#e8ecff'; ctx.lineWidth = 1;
      ctx.beginPath(); ctx.moveTo(u.x, sy(u.y + 22)); ctx.lineTo(bx, by + 10); ctx.stroke();
      ctx.fillStyle = '#e8ecff'; ctx.beginPath(); ctx.ellipse(bx, by, 9, 11, 0, 0, Math.PI * 2); ctx.fill();
      ctx.fillStyle = color; ctx.fillRect(bx - 9, by - 2, 18, 3);
      ctx.fillStyle = color; ctx.fillRect(u.x - 8, sy(u.y + 10), 16, 10);
      ctx.fillStyle = '#081028'; ctx.fillRect(u.x - 4, sy(u.y + 7), 8, 3);
      ctx.strokeStyle = color; ctx.lineWidth = 2;
      ctx.beginPath(); ctx.moveTo(u.x, sy(u.y + 10)); ctx.lineTo(u.x, sy(u.y + 22)); ctx.stroke();
      ctx.lineWidth = 4; ctx.lineCap = 'round';
      ctx.beginPath(); ctx.moveTo(u.x, sy(u.y + 22));
      ctx.lineTo(u.x + G.cosDeg(u.angle) * 14, sy(u.y + 22 + G.sinDeg(u.angle) * 14)); ctx.stroke();
      ctx.lineCap = 'butt';
      ctx.fillStyle = '#1b2550'; ctx.fillRect(u.x - 14, by - 22, 28, 4);
      ctx.fillStyle = u.hp > 30 ? '#6fd3a0' : '#ef6b6c'; ctx.fillRect(u.x - 14, by - 22, 28 * u.hp / 100, 4);
      if (turn) {
        ctx.fillStyle = '#ffd23f';
        ctx.beginPath(); ctx.moveTo(u.x, by - 26); ctx.lineTo(u.x - 6, by - 36); ctx.lineTo(u.x + 6, by - 36); ctx.fill();
      }
    });
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
    drawWind(field.wind);
  }

  /** The wind as an arrow at the top: longer for stronger wind. */
  function drawWind(wind: number) {
    const cx = G.W / 2, y = 26;
    ctx.fillStyle = '#e8ecff'; ctx.strokeStyle = '#e8ecff';
    ctx.font = '700 14px system-ui, sans-serif'; ctx.textAlign = 'center';
    if (wind === 0) { ctx.fillText('No wind', cx, y + 5); return; }
    const len = 14 + Math.abs(wind) * 5, dir = Math.sign(wind);
    ctx.fillText('Wind', cx, y - 8);
    ctx.lineWidth = 4;
    ctx.beginPath(); ctx.moveTo(cx - dir * len / 2, y + 6); ctx.lineTo(cx + dir * len / 2, y + 6); ctx.stroke();
    ctx.beginPath(); ctx.moveTo(cx + dir * (len / 2 + 8), y + 6); ctx.lineTo(cx + dir * (len / 2 - 4), y - 1); ctx.lineTo(cx + dir * (len / 2 - 4), y + 13); ctx.closePath(); ctx.fill();
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

  const windWords = (w: number) => (w === 0 ? 'No wind' : `${w < 0 ? '⬅' : '➡'} ${Math.abs(w)}`);

  function renderHud() {
    const playing = !!match?.field && match.phase === 'play';
    const turn = playing ? match!.turn : -1;
    const u = playing ? shooter() : null;
    const p = turn >= 0 ? match!.players[turn] : null;
    hud.angle.textContent = u ? `${u.angle}°` : '—';
    hud.power.textContent = u ? String(u.power) : '—';
    if (u && p) {
      const n = p.ammo[u.weapon as keyof typeof p.ammo];
      hud.rocket.textContent = `${ROCKET_LOOK[u.weapon].icon} ${G.WEAPON[u.weapon].name}${n === Infinity ? '' : ` (${n} left)`}`;
    } else hud.rocket.textContent = '—';
    hud.wind.textContent = match?.field ? windWords(match.field.wind) : '—';
    hud.round.textContent = match ? `${Math.max(1, match.round)} of ${match.rounds}` : '—';
    hud.board.innerHTML = match ? match.players.map((pl, i) => {
      const unit = match!.field?.units[i];
      const hp = unit ? (unit.alive ? unit.hp : 0) : 100;
      return `<li${i === turn ? ' class="now"' : ''}><i style="background:${pl.color}"></i><span>${esc(pl.name)}</span><span class="hp" title="Health ${hp}" aria-label="health ${hp}"><b style="width:${hp}%"></b></span><span>🪙 ${pl.credits} credits</span><span>⭐ ${pl.score} points</span></li>`;
    }).join('') : '';
    const act = canAct();
    for (const b of hud.controls.querySelectorAll('button')) (b as HTMLButtonElement).disabled = !act;
    // Only show the aim and fire buttons while a round is being played.
    hud.controls.hidden = !playing || mode === 'menu';
    info.hidden = !match;

    // The big turn banner
    let text = '', color = '';
    if (playing && !anim && !banner) {
      const a = waitingOn();
      const name = p!.name;
      color = p!.color;
      if (a?.kind === 'wait' && !mine.includes(a.ids[0])) text = `Waiting for ${name}…`;
      else text = `Your turn: ${name}`;
    } else if (anim) text = '🚀 Whoosh…';
    else if (match?.phase === 'shop' && !banner) text = '🛒 Shopping time';
    turnBanner.textContent = text;
    turnBanner.hidden = !text;
    turnBanner.style.setProperty('--who', color || '#ffd23f');
    if (text && !anim) say(text);
  }

  function renderOverlay() {
    let html = '', kind = '';
    if (howto) { kind = 'howto'; html = howtoHtml(); }
    else if (paused) { kind = 'pause'; html = `<h3>⏸ Paused</h3><div class="row"><button type="button" class="go big" data-act="resume">▶ Keep playing</button></div>`; }
    else if (mode === 'menu') { kind = 'menu'; html = menuHtml(); }
    else if (mode === 'online' && !host) { kind = 'lobby'; html = lobbyHtml(); }
    else if (banner) { kind = 'banner'; html = `<h3>${esc(banner.title)}</h3><p>${esc(banner.text)}</p>${scoresHtml()}<div class="row"><button type="button" class="go big" data-act="banner">${match?.phase === 'over' ? 'Back to the start' : 'Next ▶'}</button></div>`; }
    else if (match && !anim && match.phase === 'shop') { kind = 'shop'; html = shopHtml(); }
    overlay.hidden = !html;
    // Big panels (menus and the shop) take the canvas's place, so they fit on a phone.
    stage.dataset.panel = ['howto', 'menu', 'lobby', 'shop'].includes(kind) ? kind : '';
    overlay.dataset.kind = kind;
    if (overlay.dataset.html !== html) {
      const focused = overlay.contains(document.activeElement) ? (document.activeElement as HTMLElement).dataset.act ?? '' : null;
      overlay.innerHTML = html;
      overlay.dataset.html = html;
      if (html) {
        const target = (focused && overlay.querySelector<HTMLElement>(`[data-act="${focused}"]:not(:disabled)`)) || overlay.querySelector<HTMLElement>('input, .go:not(:disabled), button:not(:disabled)');
        target?.focus({ preventScroll: true });
      } else if (overlay.contains(document.activeElement) || document.activeElement === document.body) canvas.focus({ preventScroll: true });
    }
  }

  function howtoHtml() {
    return `<h3>How to play</h3>
      <ol class="steps">
        <li><span class="pic" aria-hidden="true">🎯</span><b>1. Aim</b><span>Tap ◀ and ▶ to tilt your launcher. The yellow dots show which way your rocket starts.</span></li>
        <li><span class="pic" aria-hidden="true">💪</span><b>2. Pick your power</b><span>Tap − for a short shot and + for a long one. Watch the wind arrow: it pushes your rocket.</span></li>
        <li><span class="pic" aria-hidden="true">🚀</span><b>3. Fire!</b><span>Press FIRE and try to hit the other rockoons. The last one flying wins.</span></li>
      </ol>
      <div class="row"><button type="button" class="go big" data-act="howto">Got it!</button></div>`;
  }

  function menuHtml() {
    if (menuStep === 'local') return `<h3>How many players?</h3><p>Take turns on this computer.</p>
      <div class="row">${[2, 3, 4].map(n => `<button type="button" class="go big num" data-act="local" data-n="${n}">${n}</button>`).join('')}</div>
      <div class="row"><button type="button" data-act="back">◀ Back</button></div>`;
    if (menuStep === 'online') return `<h3>Play online</h3><p>Each player uses their own computer or tablet.</p>
      <div class="row"><button type="button" class="go big" data-act="create">✨ Make a room</button><button type="button" class="go big" data-act="to-join">🔑 Join a room</button></div>
      <p class="small">${esc(menuNote)}</p>
      <div class="row"><button type="button" data-act="back">◀ Back</button></div>`;
    if (menuStep === 'join') {
      const code = new URLSearchParams(location.hash.slice(1)).get('room') ?? '';
      return `<h3>Join a room</h3><p>Type the 5-letter code your friend sees.</p>
        <div class="row"><input data-code class="code-in" aria-label="Room code" maxlength="6" autocomplete="off" autocapitalize="characters" spellcheck="false" enterkeyhint="go" placeholder="ABC12" value="${esc(code)}" /></div>
        <div class="row"><button type="button" class="go big" data-act="join">Join ▶</button></div>
        <p class="small" role="alert">${esc(menuNote)}</p>
        <div class="row"><button type="button" data-act="back">◀ Back</button></div>`;
    }
    return `<h3>Rockoon Wars</h3>
      <p>Launch rockets from balloons. Hit the other rockoons. The last one flying wins!</p>
      <div class="row"><button type="button" class="go big" data-act="to-local">🖥️ Play on this computer</button><button type="button" class="go big" data-act="to-online">🌐 Play online</button></div>`;
  }

  function present() {
    const now = Date.now();
    return [...snaps.entries()].filter(([id]) => id === me || now - (seen.get(id) ?? 0) < 10_000)
      .sort((a, b) => a[1].joined - b[1].joined || (a[0] < b[0] ? -1 : 1));
  }

  function lobbyHtml() {
    if (!link || link.status !== 'live' || !snaps.has(me)) {
      const offline = link?.status === 'offline';
      return offline
        ? `<h3>Online play is not working right now</h3><p>${esc(netDetail || 'We could not reach the game server.')}</p><div class="row"><button type="button" class="go big" data-act="quit">🖥️ Play on this computer</button></div>`
        : `<h3>Connecting…</h3><div class="row"><button type="button" data-act="quit">Cancel</button></div>`;
    }
    const people = present();
    const creator = snapOf(me).creator;
    const list = people.map(([id, s]) => `<li>${id === me ? '🙂 ' : '👤 '}${esc(s.cs)}${id === me ? ' (you)' : ''}</li>`).join('');
    const canStart = creator && people.length >= 2;
    const code = esc(link.room).split('').map(c => `<span>${c}</span>`).join('');
    return `<p class="t-label">Room code</p><p class="bigcode" aria-label="Room code ${esc(link.room.split('').join(' '))}">${code}</p>
      <p>${creator ? 'Tell your friends this code so they can join.' : 'You are in! Waiting for the room maker to press Start.'}</p>
      <ol class="people" aria-label="Players here">${list}</ol>
      <div class="row">${creator ? `<button type="button" class="go big" data-act="start"${canStart ? '' : ' disabled'}>${canStart ? '▶ Start' : 'Waiting for a friend…'}</button>` : ''}
      <button type="button" data-act="quit">Leave</button></div>`;
  }

  function scoresHtml() {
    return `<ol class="people">${[...match!.players].sort((a, b) => b.score - a.score)
      .map(p => `<li>${esc(p.name)}: ⭐ ${p.score} points · 🪙 ${p.credits} credits</li>`).join('')}</ol>`;
  }

  function shopHtml() {
    const a = waitingOn();
    if (a?.kind !== 'wait') return '';
    const id = a.ids.find(x => mine.includes(x));
    if (!id) return `<h3>🛒 Shopping time</h3><p>Waiting for ${esc(a.ids.map(x => host!.names[x]).join(', '))} to finish shopping.</p>`;
    const i = host!.order.indexOf(id), p = match!.players[i];
    const picks = shopPick[id] ?? [];
    const credits = p.credits - picks.reduce((s, w) => s + G.WEAPON[w].price, 0);
    const cards = G.WEAPONS.filter(w => w.price).map(w => {
      const have = p.ammo[w.id as keyof typeof p.ammo] + picks.filter(x => x === w.id).length * w.pack;
      const look = ROCKET_LOOK[w.id];
      return `<li class="card"><span class="pic" aria-hidden="true">${look.icon}</span><b>${esc(w.name)}</b><span>${esc(look.say)}</span>
        <span>${w.pack} for <strong>${w.price} credits</strong></span><span>You have ${have}</span>
        <button type="button" data-act="buy" data-w="${w.id}"${credits < w.price ? ' disabled' : ''} aria-label="Buy ${w.pack} ${esc(w.name)} for ${w.price} credits">${credits < w.price ? 'Not enough' : 'Buy'}</button></li>`;
    }).join('');
    return `<h3>🛒 ${esc(p.name)}'s shop</h3><p class="wallet">🪙 ${credits} credits to spend</p>
      <ul class="shop">${cards}</ul>
      <div class="row">${picks.length ? '<button type="button" data-act="undo">↩ Undo</button>' : ''}<button type="button" class="go big" data-act="done" data-id="${id}">Done ✓</button></div>`;
  }

  function joinTyped() {
    const raw = overlay.querySelector<HTMLInputElement>('[data-code]')?.value ?? '';
    // Easy entry: ignore spaces and dashes, any case, and the look-alikes O/0 and I/1 are never used.
    const code = raw.toUpperCase().replace(/[\s-]/g, '');
    if (!Room.ROOM_CODE.test(code)) { menuNote = 'Room codes have 5 letters and numbers, like K7QX2.'; return renderOverlay(); }
    menuNote = '';
    void startOnline(code, false);
  }

  overlay.addEventListener('keydown', e => {
    if (e.key === 'Enter' && (e.target as HTMLElement).matches('[data-code]')) { e.preventDefault(); joinTyped(); }
  });
  overlay.addEventListener('click', e => {
    const btn = (e.target as HTMLElement).closest<HTMLButtonElement>('button[data-act]');
    if (!btn || btn.disabled) return;
    const act = btn.dataset.act;
    if (act === 'howto') closeHowto();
    else if (act === 'resume') togglePause();
    else if (act === 'to-local') { menuStep = 'local'; render(); }
    else if (act === 'to-online') { menuStep = 'online'; menuNote = ''; render(); }
    else if (act === 'to-join') { menuStep = 'join'; menuNote = ''; render(); }
    else if (act === 'back') { menuStep = menuStep === 'join' ? 'online' : 'home'; menuNote = ''; render(); }
    else if (act === 'local') startLocal(Number(btn.dataset.n));
    else if (act === 'create') void startOnline(Room.newRoomCode(cryptoRandom), true);
    else if (act === 'join') joinTyped();
    else if (act === 'start') hostStart();
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
  function startLocal(n: number) {
    snaps.clear(); seen.clear();
    const order = Array.from({ length: Math.max(2, Math.min(4, n || 2)) }, (_, i) => String(i + 1).padStart(12, '0'));
    const names: Record<string, string> = {};
    order.forEach((id, i) => { names[id] = `Player ${i + 1}`; snaps.set(id, { cs: names[id], joined: i, turns: {}, shops: {} }); });
    host = { seed: crypto.getRandomValues(new Uint32Array(1))[0], rounds: ROUNDS, order, names };
    mine = order;
    mode = 'local';
    match = Room.matchFor(host);
    banner = null; shopPick = {}; paused = false;
    say('New game. Each player shops first, then round 1 starts.');
    step();
  }

  async function startOnline(code: string, creator: boolean) {
    const { RoomLink } = await import('./net').catch(() => ({ RoomLink: null }));
    if (!RoomLink) { menuNote = 'Online play could not load. Try playing on this computer.'; return renderOverlay(); }
    snaps.clear(); seen.clear();
    host = null; match = null; banner = null; shopPick = {}; paused = false;
    mode = 'online';
    history.replaceState(null, '', `#room=${code}`);
    link = new RoomLink(code, {
      status(s: LinkStatus, detail?: string) {
        netDetail = detail ?? '';
        net.textContent = s === 'live' ? `🌐 Online · room ${code} · you are ${link?.callsign}` : s === 'connecting' ? (detail || 'Connecting…') : s === 'offline' ? 'Offline' : '';
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
        say(host.order.includes(me) ? 'The game has started.' : 'The game has started. You are watching.');
        return;
      }
    }
  }

  function hostStart() {
    const people = present().slice(0, G.MAX_PLAYERS);
    if (people.length < 2) return;
    const order = [me, ...people.map(([id]) => id).filter(id => id !== me)].slice(0, G.MAX_PLAYERS);
    const names = Object.fromEntries(order.map(id => [id, snapOf(id).cs]));
    snapOf(me).host = { seed: crypto.getRandomValues(new Uint32Array(1))[0], rounds: ROUNDS, order, names };
    snapOf(me).skips = [];
    snapOf(me).shopSkips = {};
    publish();
    adoptHost();
    step();
  }

  function quit() {
    link?.stop(); link = null;
    mode = 'menu'; menuStep = 'home'; host = null; match = null; anim = null; banner = null; paused = false;
    snaps.clear(); seen.clear(); mine = []; me = '';
    pauseBtn.setAttribute('aria-pressed', 'false'); pauseBtn.textContent = '⏸ Pause';
    net.textContent = '';
    if (location.hash) history.replaceState(null, '', location.pathname);
    render();
  }

  // Re-check the room now and then: players who have gone, and the host's skips.
  setInterval(() => { if (mode === 'online') { if (!anim) step(); else renderOverlay(); } }, 5_000);

  // ---- controls: big buttons (hold to keep turning) and the keyboard ----
  function adjust(kind: 'angle' | 'power' | 'weapon', delta: number) {
    if (!match || !canAct()) return;
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
  const parseAdj = (b: HTMLElement) => { const [k, d] = (b.dataset.adj ?? '').split(':'); return [k as 'angle' | 'power' | 'weapon', Number(d)] as const; };
  let hold: ReturnType<typeof setTimeout> | undefined;
  let lastPointer = 0;
  const stopHold = () => { clearTimeout(hold); hold = undefined; };
  hud.controls.addEventListener('pointerdown', e => {
    const btn = (e.target as HTMLElement).closest<HTMLButtonElement>('button[data-adj]');
    if (!btn || btn.disabled) return;
    lastPointer = Date.now();
    const [kind, d] = parseAdj(btn);
    adjust(kind, d);
    if (kind === 'weapon') return;
    // Keep going while held, a little faster after a moment.
    let n = 0;
    const again = () => { adjust(kind, d * (n++ > 12 ? 2 : 1)); hold = setTimeout(again, 70); };
    stopHold();
    hold = setTimeout(again, 380);
  });
  for (const ev of ['pointerup', 'pointercancel', 'pointerleave']) hud.controls.addEventListener(ev, stopHold);
  hud.controls.addEventListener('click', e => {
    const btn = (e.target as HTMLElement).closest<HTMLButtonElement>('button');
    if (!btn) return;
    if (btn.dataset.fire !== undefined) return fireNow();
    // A tap or mouse click was already handled on pointerdown; this is the keyboard.
    if (Date.now() - lastPointer < 800) return;
    const [kind, d] = parseAdj(btn);
    if (kind) adjust(kind, d);
  });

  document.addEventListener('keydown', e => {
    const t = e.target as HTMLElement;
    if (t.closest('input, select, textarea') || e.ctrlKey || e.metaKey || e.altKey) return;
    if (mode === 'menu' || howto) return;
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
