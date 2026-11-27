// Rockoon Battle: the page's game. Rules live in logic.mjs, online rooms in room.mjs and net.ts,
// and the canvas drawing in render.ts. Hot-seat and online play share one path: every move is
// written into a player's snapshot and room.mjs decides what happens next, so both modes run
// the very same rules.
import * as G from './logic.mjs';
import * as Room from './room.mjs';
import { Renderer, rocketSvg, type Scene } from './render';
import type { RoomLink, LinkStatus, Snapshot } from './net';

type Snap = { cs: string; joined: number; creator?: boolean; turns: Record<number, { angle: number; power: number; weapon: string }>; shops: Record<number, string[]>; host?: Host; log?: Record<number, { angle: number; power: number; weapon: string } | 'skip'>; shopLog?: Record<number, string[][]> };
type Host = { seed: number; rounds: number; order: string[]; names: Record<string, string> };
type Match = ReturnType<typeof G.createMatch>;
type Field = NonNullable<Match['field']>;
type Events = { type: string; points?: number }[];

const ROUNDS = 3;
const HOWTO_KEY = 'rockoon-battle:how-to-seen';
const FLIGHT_SPEED = 1.6; // path points are 1/60 s of game time; play a little faster than real
const LIFT_TIME = 1.1;
const BOOM_TIME = 0.45, SETTLE_TIME = 0.5;

const root = document.querySelector<HTMLElement>('[data-rw]');
if (root) start(root);

function start(root: HTMLElement) {
  const $ = <T extends HTMLElement>(sel: string) => root.querySelector<T>(sel)!;
  const canvas = $<HTMLCanvasElement>('canvas');
  const overlay = $<HTMLDivElement>('[data-overlay]');
  const status = $<HTMLParagraphElement>('[data-status]');
  const net = $<HTMLParagraphElement>('[data-net]');
  const toast = $<HTMLDivElement>('[data-toast]');
  const hud = {
    air: $('[data-air]'), angle: $('[data-angle]'), power: $('[data-power]'), rocket: $('[data-rocket]'), rocketIcon: $('[data-rocket-icon]'),
    round: $('[data-round]'), board: $('[data-board]'), controls: $('[data-controls]'), wait: $('[data-wait]'),
  };
  const pauseBtn = $<HTMLButtonElement>('[data-pause]');
  const soundBtn = $<HTMLButtonElement>('[data-sound]');
  const quitBtn = $<HTMLButtonElement>('[data-quit]');
  const helpBtn = $<HTMLButtonElement>('[data-help]');
  const fullBtn = $<HTMLButtonElement>('[data-full]');
  const reduced = matchMedia('(prefers-reduced-motion: reduce)').matches;
  const view = new Renderer(canvas, reduced);
  new ResizeObserver(() => view.resize()).observe(canvas);
  view.resize();

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
  let anim: null | { phase: 'lift' | 'flight' | 'boom' | 'settle' | 'rest'; t: number; pos: number; path: number[][]; before: Field; blast: { x: number; y: number; r: number; dirt: boolean } | null; events: Events; shooter: number } = null;
  /** Each player's last shot this round, drawn faintly so they can correct for the wind. */
  let ghosts: { round: number; paths: Record<number, number[][]> } = { round: -1, paths: {} };
  let banner: null | { title: string; text: string } = null; // round or match results, until dismissed
  let shopPick: Record<string, string[]> = {}; // rockets being bought in the shop, by player id
  let drag: { x: number; y: number } | null = null;
  let announced = '';
  let menuNote = '';
  let netDetail = '';
  let joinCode = new URLSearchParams(location.hash.slice(1)).get('room') ?? '';
  let lastTurnKey = '';
  let liveSince = 0; // when this screen's room connection last went live
  let shotNote = ''; // what the last shot did, shown with the next turn

  function seenHowto() { try { return localStorage.getItem(HOWTO_KEY) === '1'; } catch { return false; } }
  function closeHowto() { howto = false; try { localStorage.setItem(HOWTO_KEY, '1'); } catch { /* private mode */ } render(); }
  helpBtn.addEventListener('click', () => { howto = true; render(); });
  fullBtn.hidden = !document.fullscreenEnabled || !root.requestFullscreen;
  fullBtn.addEventListener('click', async () => {
    try {
      if (document.fullscreenElement === root) await document.exitFullscreen();
      else await root.requestFullscreen();
    } catch { say('Full screen is unavailable. You can keep playing here.'); }
  });
  document.addEventListener('fullscreenchange', () => {
    const active = document.fullscreenElement === root;
    fullBtn.setAttribute('aria-pressed', String(active));
    fullBtn.title = active ? 'Exit full screen' : 'Full screen';
  });

  // ---- sound (off until asked for) ----
  let audio: AudioContext | null = null;
  let sound = false;
  function beep(kind: 'fire' | 'boom' | 'turn') {
    if (!sound) return;
    audio ??= new AudioContext();
    const t = audio.currentTime, gain = audio.createGain();
    gain.connect(audio.destination);
    if (kind === 'boom') {
      // A short burst of filtered noise sounds much more like a bang than a tone.
      const len = Math.floor(audio.sampleRate * 0.5), buf = audio.createBuffer(1, len, audio.sampleRate), d = buf.getChannelData(0);
      for (let i = 0; i < len; i++) d[i] = (Math.random() * 2 - 1) * Math.pow(1 - i / len, 2);
      const src = audio.createBufferSource(), lp = audio.createBiquadFilter();
      lp.type = 'lowpass'; lp.frequency.setValueAtTime(900, t); lp.frequency.exponentialRampToValueAtTime(120, t + 0.5);
      src.buffer = buf; src.connect(lp).connect(gain); gain.gain.setValueAtTime(0.5, t); src.start(t);
      return;
    }
    const osc = audio.createOscillator();
    osc.type = kind === 'fire' ? 'triangle' : 'sine';
    osc.frequency.setValueAtTime(kind === 'fire' ? 420 : 660, t);
    osc.frequency.exponentialRampToValueAtTime(kind === 'fire' ? 160 : 880, t + 0.2);
    gain.gain.setValueAtTime(0.08, t); gain.gain.exponentialRampToValueAtTime(0.001, t + 0.25);
    osc.connect(gain); osc.start(t); osc.stop(t + 0.3);
  }
  soundBtn.addEventListener('click', () => toggleSound());
  function toggleSound() {
    sound = !sound;
    soundBtn.setAttribute('aria-pressed', String(sound));
    soundBtn.setAttribute('aria-label', sound ? 'Sound on' : 'Sound off');
    soundBtn.textContent = sound ? '🔊' : '🔇';
    if (sound) void (audio ??= new AudioContext()).resume();
  }
  pauseBtn.addEventListener('click', () => togglePause());
  function togglePause() {
    // Online, the match runs on everyone's screens, so one player cannot pause it.
    if (mode === 'menu' || (mode === 'online' && !paused)) return;
    paused = !paused;
    pauseBtn.setAttribute('aria-pressed', String(paused));
    say(paused ? 'Paused.' : 'Playing again.');
    render();
  }
  // Leaving a match in progress takes a second tap, so a stray tap beside Pause is harmless.
  let quitArmed: ReturnType<typeof setTimeout> | undefined;
  quitBtn.addEventListener('click', () => {
    const inMatch = mode !== 'menu' && !!match && match.phase !== 'over';
    if (inMatch && !quitArmed) {
      quitBtn.dataset.armed = 'true';
      quitBtn.setAttribute('aria-label', 'Tap again to leave the match');
      say('Tap ⌂ again to leave this match.');
      quitArmed = setTimeout(disarmQuit, 3000);
      return;
    }
    disarmQuit();
    quit();
  });
  function disarmQuit() {
    clearTimeout(quitArmed); quitArmed = undefined;
    delete quitBtn.dataset.armed;
    quitBtn.setAttribute('aria-label', quitBtn.title || 'Leave');
  }

  function say(text: string, repeat = false) {
    if (text === announced) {
      if (!repeat) return;
      // The same words again (a second "Missed by 10 m"): clear first so screen readers repeat them.
      status.textContent = '';
      requestAnimationFrame(() => { status.textContent = text; });
      return;
    }
    announced = text;
    status.textContent = text;
  }
  const esc = (s: string) => s.replace(/[&<>"']/g, c => `&#${c.charCodeAt(0)};`);

  // ---- the shared move path ----
  const snapOf = (id: string) => snaps.get(id)!;
  function publish() {
    if (mode === 'online' && link) link.publish(snapOf(me));
  }
  const waitingOn = (): ReturnType<typeof Room.nextAction> | null => (match && host ? Room.nextAction(match, host, snaps) : null);
  const myTurn = () => {
    const a = waitingOn();
    return !!a && a.kind === 'wait' && !a.sent && match!.phase === 'play' && mine.includes(a.ids[0]);
  };
  const shooter = () => match!.field!.units[match!.turn];

  /** Runs every action that is ready. Shots animate one at a time. */
  function step() {
    if (!match || !host || anim || paused || banner) return render();
    for (let guard = 0; guard < 50; guard++) {
      // The host records each turn once; everyone, the host included, then plays that record.
      if (amHost() && Room.commit(match, host, snaps, goneIds())) publish();
      const action = Room.nextAction(match, host, snaps);
      if (action.kind === 'fire') {
        const before = structuredClone(match.field!) as Field;
        before.units[match.turn].angle = action.shot.angle;
        const shooterIndex = match.turn;
        const result = Room.apply(match, action) as ReturnType<typeof G.fire>;
        if (ghosts.round !== match.round) ghosts = { round: match.round, paths: {} };
        ghosts.paths[shooterIndex] = result.path;
        // match.field is the same object until the next round starts, so keep a copy to draw.
        anim = { phase: 'lift', t: 0, pos: 0, path: result.path, before, blast: result.blast, events: result.events, shooter: shooterIndex };
        drag = null;
        return render();
      }
      if (action.kind === 'shop' || action.kind === 'skip') {
        Room.apply(match, action); shopPick = {};
        if (action.kind === 'skip' && match.phase !== 'play') { shotNote = 'Turn skipped.'; showRoundResult(); break; }
        continue;
      }
      break;
    }
    render();
  }

  /** In local games this screen is the host. Online, the room creator is. */
  const amHost = () => !!host && mine.includes(host.order[0]);

  /** Players the host gives up on after 30 seconds without a snapshot. Their turns are skipped. */
  function goneIds() {
    if (mode !== 'online' || !host) return [];
    const now = Date.now();
    // Just (re)connected: give everyone time to be heard again before deciding anyone left.
    if (now - liveSince < 10_000) return [];
    return host.order.filter(id => id !== me && now - (seen.get(id) ?? 0) > 30_000);
  }

  const canAct = () => myTurn() && !anim && !paused && !banner && !howto;

  function fireNow() {
    if (!match || !canAct()) return;
    const u = shooter(), id = host!.order[match.turn];
    snapOf(id).turns[match.log.length] = { angle: u.angle, power: u.power, weapon: u.weapon };
    publish();
    step();
  }

  // ---- the animation loop: always running, so the sky, clouds and effects move smoothly ----
  let last = performance.now();
  function frame(now: number) {
    requestAnimationFrame(frame);
    const dt = Math.min(0.05, (now - last) / 1000);
    last = now;
    if (anim && !paused) advance(dt);
    view.draw(scene(), paused ? 0 : dt, sceneSeed());
  }
  requestAnimationFrame(frame);

  const sceneSeed = () => (match ? (match.seed + match.round) >>> 0 : 7);

  function advance(dt: number) {
    const a = anim!;
    a.t += dt;
    if (a.phase === 'lift') {
      if (a.t >= LIFT_TIME) { a.phase = 'flight'; a.t = 0; beep('fire'); renderHud(); }
    } else if (a.phase === 'flight') {
      const prev = Math.floor(a.pos);
      a.pos = Math.min(a.path.length - 1, a.pos + dt * 60 * (reduced ? 4 : FLIGHT_SPEED));
      for (let i = prev + 1; i <= Math.floor(a.pos); i += 2) view.trailPuff(a.path[i][0], a.path[i][1]);
      if (a.pos >= a.path.length - 1) {
        a.phase = 'boom'; a.t = 0;
        if (a.blast) { view.boom(a.blast.x, a.blast.y, a.blast.r, a.blast.dirt); beep('boom'); }
        hitFeedback(a);
      }
    } else if (a.phase === 'boom' && a.t >= (a.blast ? BOOM_TIME : 0.2)) { a.phase = 'settle'; a.t = 0; }
    else if (a.phase === 'settle' && a.t >= SETTLE_TIME) { a.phase = 'rest'; a.t = 0; }
    else if (a.phase === 'rest' && a.t >= 0.35) finishShot();
  }

  function hitFeedback(a: NonNullable<typeof anim>) {
    const points = a.events.find(e => e.type === 'scan')?.points ?? 0;
    const { x, y } = a.before.target;
    view.float(x, y + 22, points ? `+${points}` : 'Miss');
  }

  /** What the canvas shows right now, including a shot in progress. */
  function scene(): Scene | null {
    if (!match?.field) return null;
    const names = match.players.map(p => p.name), colors = match.players.map(p => p.color);
    if (anim) {
      const { before, phase, t, shooter } = anim;
      const lift = phase === 'lift' ? Math.min(1, t / LIFT_TIME) : 1;
      const units = before.units.map((u, i) => i === shooter ? { ...u, ...G.balloonPosition(u, before.wind, lift) } : u);
      return { ...before, units, colors, names, released: phase === 'lift' ? undefined : shooter, active: -1, preview: null, drag: null,
        shot: phase === 'flight' ? { path: anim.path, pos: anim.pos } : null };
    }
    const act = canAct() && match.phase === 'play';
    return {
      target: match.field.target, terrain: match.field.terrain, units: match.field.units, wind: match.field.wind, colors, names,
      active: match.phase === 'play' ? match.turn : -1,
      preview: act ? G.aimPreview(match, 40) : null, drag: act ? drag : null, shot: null,
      ghost: act && ghosts.round === match.round ? ghosts.paths[match.turn] ?? null : null,
    };
  }
  function finishShot() {
    const points = anim!.events.find(e => e.type === 'scan')?.points ?? 0;
    shotNote = points === 3 ? 'Through the ring! +3 points.' : points === 1 ? 'Outer scan! +1 point.' : 'Missed the ring. Check your last trail.';
    anim = null;
    if (match!.phase !== 'play') showRoundResult();
    say(shotNote, true);
    step();
  }

  function showRoundResult() {
    const top = match!.winners.map(i => match!.players[i].name);
    const title = match!.phase === 'over'
      ? (top.length > 1 ? `${top.join(' and ')} share the win!` : `${top[0]} wins!`)
      : `Round ${match!.round} complete`;
    banner = { title, text: [shotNote, match!.phase === 'shop' ? 'Each crew gets 100 credits for the next round.' : ''].filter(Boolean).join(' ') };
  }

  // ---- the HTML around the canvas (changed only when the game changes, never per frame) ----
  function render() {
    // For checking that online players stay in step: how many turns, and a hash of the match.
    root.dataset.turns = match ? String(match.log.length) : '';
    root.dataset.fp = match && !anim ? G.fingerprint(match) : '';
    root.dataset.mode = mode;
    pauseBtn.disabled = mode === 'online';
    pauseBtn.title = mode === 'online' ? 'An online match cannot be paused' : 'Pause';
    renderHud();
    renderOverlay();
    turnToast();
  }

  function renderHud() {
    const playing = !!match?.field && match.phase === 'play';
    const turn = playing ? match!.turn : -1;
    const u = playing ? shooter() : null;
    const p = turn >= 0 ? match!.players[turn] : null;
    const field = anim?.before ?? match?.field;
    const pad = anim ? anim.before.units[anim.shooter] : u;
    hud.air.hidden = !pad;
    if (field && pad) {
      const ground = pad.y * G.METRES_PER_PIXEL;
      const release = ground + G.LIFT * G.METRES_PER_PIXEL;
      const wind = field.wind === 0 ? 'calm' : `${Math.abs(field.wind)}/10 ${field.wind < 0 ? 'left' : 'right'}`;
      hud.air.textContent = `Simulated wind · ${wind} | Standard pressure · pad ${ground} m: ${Math.round(G.pressureHpa(ground))} hPa → release ${release} m: ${Math.round(G.pressureHpa(release))} hPa`;
    }
    hud.angle.textContent = u ? `${u.angle}°` : '—';
    hud.power.textContent = u ? String(u.power) : '—';
    if (u && p) {
      const n = p.ammo[u.weapon as keyof typeof p.ammo];
      hud.rocket.textContent = `${G.WEAPON[u.weapon].name}${n === Infinity ? '' : ` ×${n}`}`;
      if (hud.rocketIcon.dataset.w !== u.weapon) { hud.rocketIcon.innerHTML = rocketSvg(u.weapon, 30); hud.rocketIcon.dataset.w = u.weapon; }
    }
    hud.round.textContent = match ? `Round ${Math.max(1, match.round)}/${match.rounds}` : '';
    const board = match ? match.players.map((pl, i) => {
      const cls = i === turn && !anim ? 'now' : '';
      return `<li${cls ? ` class="${cls}"` : ''} style="--c:${pl.color}"><b>${esc(pl.name)}</b><span class="cr" title="Credits">🪙 ${pl.credits}</span><span class="pt" title="Points">⭐ ${pl.score}</span></li>`;
    }).join('') : '';
    if (hud.board.dataset.html !== board) { hud.board.innerHTML = board; hud.board.dataset.html = board; }
    const act = canAct();
    for (const b of hud.controls.querySelectorAll('button')) (b as HTMLButtonElement).disabled = !act;
    root.dataset.playing = String(playing);
    // When it is not this screen's turn, the controls make way for a short note.
    let wait = '';
    if (playing && !act) {
      const a = waitingOn();
      if (anim) wait = anim.phase === 'lift' ? 'Balloon rising…' : 'Rocket released';
      else if (paused) wait = 'Paused';
      else if (a?.kind === 'wait' && a.sent) wait = 'Sending the shot…';
      else if (a?.kind === 'wait' && !mine.includes(a.ids[0])) wait = `Waiting for ${p!.name}…`;
    }
    hud.controls.dataset.wait = wait ? 'true' : 'false';
    hud.wait.textContent = wait;
  }

  /** A big, short "whose turn" card when the turn changes. */
  function turnToast() {
    const playing = !!match?.field && match.phase === 'play' && !anim && !banner && !howto;
    const key = playing ? `${match!.round}:${match!.log.length}` : '';
    if (!key || key === lastTurnKey) return;
    lastTurnKey = key;
    const p = match!.players[match!.turn];
    const mineNow = mine.includes(host!.order[match!.turn]);
    const text = mode === 'online' ? (mineNow ? `Your turn, ${p.name}!` : `${p.name}'s turn`) : `${p.name}'s turn`;
    // The last shot's result rides along under the name, so a miss note is seen, not lost.
    const note = shotNote; shotNote = '';
    toast.innerHTML = esc(text) + (note ? `<small>${esc(note)}</small>` : '');
    toast.style.setProperty('--c', p.color);
    say(`${note ? note + ' ' : ''}${mineNow ? `Your turn: ${p.name}` : `Waiting for ${p.name}`}`);
    if (mineNow) beep('turn');
    toast.getAnimations().forEach(a => a.cancel());
    toast.animate(reduced
      ? [{ opacity: 0 }, { opacity: 1, offset: 0.1 }, { opacity: 1, offset: 0.8 }, { opacity: 0 }]
      : [{ opacity: 0, transform: 'translate(-50%, -14px) scale(.9)' }, { opacity: 1, transform: 'translate(-50%, 0) scale(1)', offset: 0.12 }, { opacity: 1, transform: 'translate(-50%, 0) scale(1)', offset: 0.8 }, { opacity: 0, transform: 'translate(-50%, -8px) scale(.98)' }],
      { duration: note ? 2600 : 1500, easing: 'ease-out', fill: 'forwards' });
  }

  function renderOverlay() {
    let html = '', kind = '';
    if (howto) { kind = 'howto'; html = howtoHtml(); }
    else if (paused) { kind = 'pause'; html = `<h3>Paused</h3><div class="row"><button type="button" class="go big" data-act="resume">▶ Keep playing</button></div>`; }
    else if (mode === 'menu') { kind = 'menu'; html = menuHtml(); }
    else if (mode === 'online' && !host) { kind = 'lobby'; html = lobbyHtml(); }
    else if (banner) { kind = 'banner'; html = `<h3>${esc(banner.title)}</h3><p>${esc(banner.text)}</p>${scoresHtml()}<div class="row"><button type="button" class="go big" data-act="banner">${match?.phase === 'over' ? 'Back to the start' : 'Next ▶'}</button></div>`; }
    else if (match && !anim && match.phase === 'shop') { kind = 'shop'; html = shopHtml(); }
    const changedPanel = overlay.dataset.kind !== kind;
    const focusWasInside = overlay.contains(document.activeElement);
    overlay.hidden = !html;
    overlay.dataset.kind = kind;
    canvas.inert = !!html;
    if (overlay.dataset.html !== html) {
      const focused = focusWasInside ? (document.activeElement as HTMLElement).dataset.act : null;
      overlay.innerHTML = html ? `<div class="panel">${html}</div>` : '';
      overlay.dataset.html = html;
      if (html && (focusWasInside || (changedPanel && root.contains(document.activeElement)))) {
        const target = (focused && overlay.querySelector<HTMLElement>(`[data-act="${focused}"]:not(:disabled)`)) || overlay.querySelector<HTMLElement>('input, .go:not(:disabled), button:not(:disabled)');
        target?.focus({ preventScroll: true });
      } else if (!html && (focusWasInside || root.contains(document.activeElement))) canvas.focus({ preventScroll: true });
    }
  }

  function howtoHtml() {
    return `<h3>How to play</h3>
      <ol class="steps">
        <li><span class="pic" aria-hidden="true">👆</span><b>1. Aim</b><span>Drag from your pad toward the sky ring. Farther means more power. Or use the ◀ ▶ and − + buttons.</span></li>
        <li><span class="pic" aria-hidden="true">🌬️</span><b>2. Mind the wind</b><span>The arrow at the top shows the wind. It carries your balloon sideways and pushes your rocket. The yellow dots show how it starts.</span></li>
        <li><span class="pic" aria-hidden="true">🚀</span><b>3. Launch</b><span>Press Launch. Your balloon lifts off, then releases its rocket. Fly through the ring for 3 points.</span></li>
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
      return `<h3>Join a room</h3><p>Type the 5-letter code your friend sees.</p>
        <div class="row"><input data-code class="code-in" aria-label="Room code" maxlength="24" autocomplete="off" autocapitalize="characters" spellcheck="false" enterkeyhint="go" placeholder="ABC12" value="${esc(joinCode)}" /></div>
        <div class="row"><button type="button" class="go big" data-act="join">Join ▶</button></div>
        <p class="small" role="alert">${esc(menuNote)}</p>
        <div class="row"><button type="button" data-act="back">◀ Back</button></div>`;
    }
    return `<p class="logo" aria-hidden="true">🎈🚀</p><h3 class="title">Rockoon Battle</h3>
      <p>Lift off from the hillside. Send your rocket through the sky ring. Highest score after three rounds wins.</p>
      <div class="row col"><button type="button" class="go big" data-act="to-local">🖥️ Play on this computer</button><button type="button" class="go big alt" data-act="to-online">🌐 Play online</button></div>`;
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
      .map(p => `<li><i style="background:${p.color}"></i> ${esc(p.name)}: ⭐ ${p.score} points · 🪙 ${p.credits} credits</li>`).join('')}</ol>`;
  }

  function shopHtml() {
    const a = waitingOn();
    if (a?.kind !== 'wait') return '';
    const id = a.ids.find(x => mine.includes(x));
    if (!id) return a.sent || !a.ids.length
      ? `<h3>🛒 Shopping time</h3><p>Everyone is done. Starting the round…</p>`
      : `<h3>🛒 Shopping time</h3><p>Waiting for ${esc(a.ids.map(x => host!.names[x]).join(', '))} to finish shopping.</p>`;
    const i = host!.order.indexOf(id), p = match!.players[i];
    const picks = shopPick[id] ?? [];
    const credits = p.credits - picks.reduce((s, w) => s + G.WEAPON[w].price, 0);
    const cards = G.WEAPONS.filter(w => w.price).map(w => {
      const have = p.ammo[w.id as keyof typeof p.ammo] + picks.filter(x => x === w.id).length * w.pack;
      const poor = credits < w.price;
      return `<li class="card${poor ? ' poor' : ''}">${rocketSvg(w.id, 48)}<b>${esc(w.name)}</b><span class="say">${esc(`Scans ${w.radius} pixels beyond the ring.`)}</span>
        <span class="price">${w.pack} for 🪙 ${w.price}</span><span class="have">You have ${have}</span>
        <button type="button" data-act="buy" data-w="${w.id}"${poor ? ' disabled' : ''} aria-label="Buy ${w.pack} ${esc(w.name)} for ${w.price} credits">${poor ? 'Not enough' : 'Buy'}</button></li>`;
    }).join('');
    return `<h3><span class="dot" style="background:${p.color}"></span>${esc(p.name)}'s shop</h3><p class="wallet">🪙 ${credits} credits to spend</p>
      <ul class="shop">${cards}</ul>
      <div class="row">${picks.length ? '<button type="button" data-act="undo">↩ Undo</button>' : ''}<button type="button" class="go big" data-act="done" data-id="${id}">Done ✓</button></div>`;
  }

  function joinTyped() {
    joinCode = overlay.querySelector<HTMLInputElement>('[data-code]')?.value ?? '';
    // Easy entry: ignore spaces and dashes, any case, and the look-alikes O/0 and I/1 are never used.
    const code = joinCode.toUpperCase().replace(/[\s-]/g, '');
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
  const bringIntoView = () => {
    const r = root.getBoundingClientRect();
    if (Math.abs(r.top) > 4) root.scrollIntoView({ behavior: reduced ? 'auto' : 'smooth', block: 'start' });
  };

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
    banner = null; shopPick = {}; paused = false; lastTurnKey = '';
    say('New game. Each player shops first, then round 1 starts.');
    bringIntoView();
    step();
  }

  let joining = false;
  async function startOnline(code: string, creator: boolean) {
    // One room at a time: a double tap on Join must not open two connections.
    if (joining || link) return;
    joining = true;
    const { RoomLink } = await import('./net').catch(() => ({ RoomLink: null })).finally(() => { joining = false; });
    if (mode !== 'menu' || link) return; // left, or already joined, while the code loaded
    if (!RoomLink) { menuNote = 'Online play could not load. Try playing on this computer.'; return renderOverlay(); }
    snaps.clear(); seen.clear();
    host = null; match = null; banner = null; shopPick = {}; paused = false; lastTurnKey = '';
    mode = 'online';
    history.replaceState(null, '', `#room=${code}`);
    const thisLink: RoomLink = link = new RoomLink(code, {
      status(s: LinkStatus, detail?: string) {
        if (link !== thisLink) return; // a room this screen has already left
        if (s === 'live') liveSince = Date.now();
        netDetail = detail ?? '';
        net.textContent = s === 'live' ? `🌐 Room ${code} · you are ${thisLink.callsign}` : s === 'connecting' ? (detail || 'Connecting…') : s === 'offline' ? 'Offline' : '';
        render();
      },
      peer(id: string, snap: Snapshot) {
        snaps.set(id, snap as Snap);
        seen.set(id, Date.now());
        adoptHost();
        step();
      },
      // A player's broadcast also goes away for a moment whenever a connection renews, so
      // leaving is judged only by how long it has been since their last snapshot (goneIds).
      leave() { render(); },
    });
    bringIntoView();
    const ok = await thisLink.start();
    if (!ok || link !== thisLink) return;
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
    snapOf(me).log = {};
    snapOf(me).shopLog = {};
    publish();
    adoptHost();
    step();
  }

  function quit() {
    link?.stop(); link = null;
    mode = 'menu'; menuStep = 'home'; host = null; match = null; anim = null; banner = null; paused = false; drag = null;
    snaps.clear(); seen.clear(); mine = []; me = ''; lastTurnKey = '';
    pauseBtn.setAttribute('aria-pressed', 'false');
    net.textContent = '';
    if (location.hash) history.replaceState(null, '', location.pathname);
    render();
  }

  // Re-check the room now and then: players who have gone, and the host's committed turns.
  setInterval(() => { if (mode === 'online') { if (!anim) step(); else renderOverlay(); } }, 5_000);

  // ---- controls ----
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
    renderHud();
  }

  // Drag to aim: press anywhere on the playfield on your turn, and point where you want to shoot.
  let dragId = -1;
  const aimAt = (e: PointerEvent) => {
    const w = view.toWorld(e.clientX, e.clientY);
    const u = shooter();
    const { angle, power } = G.aimFromPoint(u, w.x, w.y);
    u.angle = angle; u.power = power;
    drag = { x: w.x, y: w.y };
    renderHud();
  };
  canvas.addEventListener('pointerdown', e => {
    if (!canAct() || e.button !== 0 || dragId !== -1) return;
    e.preventDefault();
    dragId = e.pointerId;
    canvas.setPointerCapture(e.pointerId);
    aimAt(e);
  });
  canvas.addEventListener('pointermove', e => { if (e.pointerId === dragId && canAct()) aimAt(e); });
  const endDrag = (e: PointerEvent) => {
    if (e.pointerId !== dragId) return;
    dragId = -1; drag = null;
    if (canAct()) say(`Aim ${shooter().angle} degrees, power ${shooter().power}.`);
  };
  canvas.addEventListener('pointerup', endDrag);
  canvas.addEventListener('pointercancel', endDrag);
  canvas.addEventListener('lostpointercapture', endDrag);

  // The buttons: hold to keep turning.
  const parseAdj = (b: HTMLElement) => { const [k, d] = (b.dataset.adj ?? '').split(':'); return [k as 'angle' | 'power' | 'weapon', Number(d)] as const; };
  let hold: ReturnType<typeof setTimeout> | undefined;
  const stopHold = () => { clearTimeout(hold); hold = undefined; };
  hud.controls.addEventListener('pointerdown', e => {
    const btn = (e.target as HTMLElement).closest<HTMLButtonElement>('button[data-adj]');
    if (!btn || btn.disabled || e.button !== 0 || !e.isPrimary) return;
    stopHold();
    const [kind, d] = parseAdj(btn);
    adjust(kind, d);
    if (kind === 'weapon') return;
    let n = 0;
    const again = () => {
      if (!canAct()) return stopHold();
      adjust(kind, d * (n++ > 12 ? 2 : 1));
      hold = setTimeout(again, 60);
    };
    btn.setPointerCapture(e.pointerId);
    hold = setTimeout(again, 350);
  });
  for (const ev of ['pointerup', 'pointercancel', 'lostpointercapture']) hud.controls.addEventListener(ev, stopHold);
  window.addEventListener('blur', stopHold);
  hud.controls.addEventListener('click', e => {
    const btn = (e.target as HTMLElement).closest<HTMLButtonElement>('button');
    if (!btn) return;
    if (btn.dataset.fire !== undefined) return fireNow();
    // A tap or mouse click was already handled on pointerdown; this is the keyboard.
    if (e.detail > 0) return;
    const [kind, d] = parseAdj(btn);
    if (kind) adjust(kind, d);
  });

  // Keys pressed inside the game, and also those with nothing focused: Safari and Firefox on
  // macOS do not focus a button when it is clicked, so after a tap on ◀ ▶ focus is on the page.
  root.addEventListener('keydown', onKey);
  document.addEventListener('keydown', e => { if (e.target === document.body && mode !== 'menu') onKey(e); });
  function onKey(e: KeyboardEvent) {
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
        if (t !== canvas || e.repeat) { handled = false; break; }
        fireNow(); break;
      case 'p': case 'P': case 'Escape': if (e.repeat || document.fullscreenElement && e.key === 'Escape') { handled = false; break; } togglePause(); break;
      case 'm': case 'M': if (!e.repeat) toggleSound(); break;
      default: handled = false;
    }
    if (handled) e.preventDefault();
  }

  render();
}
