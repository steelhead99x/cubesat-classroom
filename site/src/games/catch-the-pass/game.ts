// Catch the Pass: drawing, input, sound and the live sky. The rules are in logic.mjs.
import * as G from './logic.mjs';
import { inputPercent } from '../../lib/pedal.mjs';
import { STEPS } from './steps.mjs';
import { icon } from '../icons.mjs';
import { patch } from '../patches.mjs';
import type { LiveSky, StationState, PacketNote, SkyStatus } from './net';

type Game = ReturnType<typeof G.createGame>;
type Level = (typeof G.LEVELS)[number] | typeof G.LIVE;
type Antenna = keyof typeof G.ANTENNAS;

/** The site's base path ("/cubesat-classroom" on GitHub Pages, "" on cubesatclass.org). */
const siteBase = import.meta.env.BASE_URL.replace(/\/$/, '');
const root = document.querySelector<HTMLElement>('[data-ctp]');
if (root) init(root);

function init(root: HTMLElement) {
  const $ = <T extends Element = HTMLElement>(sel: string) => root.querySelector<T>(sel)!;
  const canvas = $<HTMLCanvasElement>('canvas');
  const ctx = canvas.getContext('2d')!;
  const statusLine = $('[data-status]');
  const overlay = $('[data-overlay]');
  const reduced = matchMedia('(prefers-reduced-motion: reduce)');
  const bestKey = 'catch-the-pass:best';
  // Stored stars are checked, so an edited or damaged value cannot stop the game from starting.
  const best: Record<string, number> = (() => {
    const out: Record<string, number> = {};
    try {
      const raw: unknown = JSON.parse(localStorage.getItem(bestKey) ?? '{}');
      if (raw && typeof raw === 'object' && !Array.isArray(raw)) {
        for (const l of G.LEVELS) {
          const v = (raw as Record<string, unknown>)[l.id];
          if (Number.isInteger(v) && (v as number) >= 0 && (v as number) <= 3) out[l.id] = v as number;
        }
      }
    } catch { /* private mode or bad JSON */ }
    return out;
  })();
  const prefsKey = 'catch-the-pass:prefs';
  const prefs = (() => { try { const p = JSON.parse(localStorage.getItem(prefsKey) ?? '{}'); return p && typeof p === 'object' ? p : {}; } catch { return {}; } })();
  const savePrefs = () => { try { localStorage.setItem(prefsKey, JSON.stringify({ easy: slow, level: savedLevel, antenna: antennaChoice })); } catch { /* private mode */ } };
  let antennaChoice = typeof prefs.antenna === 'string' && Object.hasOwn(G.ANTENNAS, prefs.antenna) ? prefs.antenna as Antenna : '';
  const receiver = { ...G.DEFAULT_RECEIVER };

  let mode: 'missions' | 'live' = 'missions';
  let levelIndex = 0, savedLevel = 0;
  let linkedEntry = false;
  let game: Game | null = null;
  let running = false, paused = false, countdown = 0;
  let helpOpen = false;
  let afterHelp: string | null = null; // the results card to show again when Help closes
  let unlockedFor = 0; // seconds without a lock, for the beginner coach
  let slow = prefs.easy !== false; // Easy mode is on until the player turns it off.
  const howtoKey = 'catch-the-pass:how-to-seen';
  let liveSlot = -1;
  let last = performance.now();
  const keys = new Set<string>();
  let pointer: { az: number; el: number } | null = null;
  let pointerId: number | null = null;
  const pops: { x: number; y: number; text: string; age: number }[] = [];
  const stations = new Map<string, { state: StationState; seen: number; demo?: boolean }>();
  const feed: string[] = [];

  // ---- Sound: off by default, made only after the player turns it on. ----
  let audio: AudioContext | null = null;
  let soundOn = false;
  function beep(freq: number, ms: number, type: OscillatorType = 'sine', gain = 0.06) {
    if (!soundOn || !audio || receiver.level === 0) return;
    const o = audio.createOscillator(), g = audio.createGain();
    o.type = type; o.frequency.value = freq;
    g.gain.setValueAtTime(gain * receiver.level / G.DEFAULT_RECEIVER.level, audio.currentTime);
    g.gain.exponentialRampToValueAtTime(0.000001, audio.currentTime + ms / 1000);
    o.connect(g).connect(audio.destination);
    o.start(); o.stop(audio.currentTime + ms / 1000);
  }
  $('[data-sound]').addEventListener('click', e => {
    soundOn = !soundOn;
    if (soundOn && !audio) audio = new AudioContext();
    (e.currentTarget as HTMLElement).setAttribute('aria-pressed', String(soundOn));
    (e.currentTarget as HTMLElement).innerHTML = icon(soundOn ? 'sound-on' : 'sound-off');
    (e.currentTarget as HTMLElement).setAttribute('aria-label', soundOn ? 'Sound on' : 'Sound off');
    beep(660, 80);
  });

  // ---- Screen geometry: azimuth -90..90 across, elevation 0..90 up. ----
  const pad = { l: 34, r: 14, t: 16, b: 40 };
  const W = () => canvas.width / devicePixelRatio, H = () => canvas.height / devicePixelRatio;
  const toX = (az: number) => pad.l + (az + 90) / 180 * (W() - pad.l - pad.r);
  const toY = (el: number) => H() - pad.b - el / 90 * (H() - pad.t - pad.b);
  const fromXY = (x: number, y: number) => ({
    az: Math.max(-90, Math.min(90, (x - pad.l) / (W() - pad.l - pad.r) * 180 - 90)),
    el: Math.max(0, Math.min(90, (H() - pad.b - y) / (H() - pad.t - pad.b) * 90)),
  });
  const dashEl = $<HTMLElement>('.dash'), deckEl = $<HTMLElement>('.deck');
  function resize() {
    // Leave room for the numbers above and the controls below, so a whole pass can be played
    // without scrolling: the signal bar, the sky and the buttons all stay on screen.
    const chrome = dashEl.offsetHeight + ($<HTMLElement>('[data-now]').offsetHeight) + deckEl.offsetHeight + 36;
    const w = canvas.clientWidth, h = Math.round(Math.min(w * (w < 560 ? 0.85 : 0.56), Math.max(220, innerHeight - chrome)));
    canvas.style.height = h + 'px';
    canvas.width = Math.round(w * devicePixelRatio); canvas.height = Math.round(h * devicePixelRatio);
    ctx.setTransform(devicePixelRatio, 0, 0, devicePixelRatio, 0, 0);
    draw();
  }
  new ResizeObserver(resize).observe(canvas);
  addEventListener('resize', resize);

  const stars = Array.from({ length: 70 }, (_, i) => { const r = G.rng(i + 1); return { x: r(), y: r() * 0.85, s: r() * 1.4 + 0.3, p: r() * 6 }; });

  // ---- Levels and modes ----
  const levelButtons = [...root.querySelectorAll<HTMLButtonElement>('[data-level]')];
  function unlocked(i: number) { return i === 0 || (best[G.LEVELS[i - 1].id] ?? 0) > 0; }
  if (Number.isInteger(prefs.level) && prefs.level > 0 && prefs.level < G.LEVELS.length && unlocked(prefs.level)) savedLevel = levelIndex = prefs.level;
  function renderLevels() {
    levelButtons.forEach((b, i) => {
      const s = best[G.LEVELS[i].id] ?? 0;
      b.disabled = !unlocked(i);
      b.setAttribute('aria-pressed', String(mode === 'missions' && i === levelIndex));
      b.querySelector('[data-stars]')!.textContent = b.disabled ? 'locked' : '★'.repeat(s) + '☆'.repeat(3 - s);
      b.setAttribute('aria-label', `Level ${i + 1}, ${G.LEVELS[i].name}, ${b.disabled ? 'locked' : s + ' of 3 stars'}`);
    });
  }
  levelButtons.forEach((b, i) => b.addEventListener('click', () => {
    if (i === levelIndex) return;
    // A pass in flight is not thrown away by a stray tap: pause it first to switch levels.
    if (running && !paused && game && !game.done) { announce('Pause the pass first to pick another level.'); return; }
    savedLevel = levelIndex = i;
    savePrefs();
    ready();
  }));

  const helpBtn = $<HTMLButtonElement>('[data-help]');
  const pauseBtn = $<HTMLButtonElement>('[data-pause]');
  const easyInput = $<HTMLInputElement>('[data-slow]');
  const tabs = [...root.querySelectorAll<HTMLButtonElement>('[data-mode]')];
  function setTab() {
    tabs.forEach(t => t.setAttribute('aria-pressed', String(t.dataset.mode === mode)));
    $('[data-missions]').hidden = mode !== 'missions';
    $('[data-live]').hidden = mode !== 'live';
    $('[data-live-board]').hidden = mode !== 'live';
    easyInput.disabled = mode === 'live';
    // The live pass runs on a shared clock, so it cannot be paused or restarted.
    pauseBtn.disabled = mode === 'live';
    $<HTMLButtonElement>('[data-restart]').disabled = mode === 'live';
    pauseBtn.textContent = 'Start';
    renderLevels();
  }
  tabs.forEach(t => t.addEventListener('click', () => {
    if (t.dataset.mode === mode) return;
    setHelp(false);
    clearAim();
    mode = t.dataset.mode as 'missions' | 'live';
    setTab();
    if (mode === 'missions') { sky?.stop(); stations.clear(); ready(); } else void joinLive();
  }));
  easyInput.checked = slow;
  easyInput.addEventListener('change', e => { slow = (e.target as HTMLInputElement).checked; savePrefs(); if (mode === 'missions') ready(); });

  function currentLevel(): Level {
    const level = mode === 'live' ? G.LIVE : G.LEVELS[levelIndex];
    return slow && mode === 'missions' ? G.easyLevel(level) : level;
  }

  function ready() {
    // Choosing a level or Easy while Help is open closes Help, so Start always works.
    if (helpOpen) setHelp(false);
    afterHelp = null;
    renderLevels();
    const level = currentLevel();
    game = G.createGame(level, (Math.random() * 2 ** 31) | 0, antennaChoice || level.antenna, receiver);
    running = false; paused = false; countdown = 0;
    clearAim();
    pauseBtn.textContent = 'Start';
    easyInput.disabled = false;
    show(`<h3>${mode === 'live' ? 'Live sky' : `Level ${levelIndex + 1}: ${G.LEVELS[levelIndex].name}`}</h3>
      <p>${currentLevel().tip}</p>
      <p class="goal">Goal: ${currentLevel().goal.map((g, i) => `${'★'.repeat(i + 1)} ${g} packets`).join(' · ')}</p>
      <button type="button" class="go" data-go>${icon('start')} Start</button>`);
    hud(); draw();
    // Shown on a first visit without moving focus: the page has only just loaded, so keyboard
    // and screen-reader users start at the top (skip link, header) like on every other page.
    if (!linkedEntry && !seenHowto() && !helpOpen) showHowto(false);
  }

  // ---- The first-visit "How to play" card ----
  function seenHowto() { try { return localStorage.getItem(howtoKey) === '1'; } catch { return false; } }
  function setHelp(open: boolean) {
    helpOpen = open;
    canvas.parentElement!.dataset.panel = open ? 'howto' : '';
  }

  function showHowto(takeFocus = true) {
    if (helpOpen) return;
    if (running && mode === 'missions' && !paused) setPaused(true, false);
    clearAim();
    // Opened from a results card: bring that card (and its Next level button) back afterwards.
    afterHelp = mode === 'missions' && !running && game?.done && !overlay.hidden ? overlay.innerHTML : null;
    setHelp(true);
    // The key steps from the page's step-by-step instructions (steps.mjs), with their numbers.
    const pick = ['start', 'track', 'radio'].map(id => [STEPS.findIndex(s => s.id === id), STEPS.find(s => s.id === id)!] as const);
    show(`<div class="logo">${patch('pass', 'Catch the Pass')}</div><h3>How to play</h3>
      <ol class="steps">
        ${pick.map(([i, s]) => `<li><span class="pic">${icon(s.pic, 34)}</span><b>Step ${i + 1}: ${s.title}</b><span>${s.text}</span></li>`).join('')}
      </ol>
      <p class="small"><a href="#steps">See all ${STEPS.length} steps</a>, each with its own link to share.</p>
      <button type="button" class="go" data-howto>Got it!</button>`);
    const got = overlay.querySelector<HTMLButtonElement>('[data-howto]')!;
    got.addEventListener('click', closeHowto);
    if (takeFocus) got.focus({ preventScroll: true });
  }
  function closeHowto() {
    try { localStorage.setItem(howtoKey, '1'); } catch { /* private mode */ }
    setHelp(false);
    if (afterHelp) {
      show(afterHelp);
      afterHelp = null;
      overlay.querySelector<HTMLButtonElement>('.go')?.focus({ preventScroll: true });
    } else if (mode === 'missions' && !running) {
      ready();
      overlay.querySelector<HTMLButtonElement>('[data-go]')?.focus({ preventScroll: true });
    } else if (paused) setPaused(true);
    else {
      hideOverlay();
      helpBtn.focus({ preventScroll: true });
    }
  }
  helpBtn.addEventListener('click', () => showHowto());

  function hideOverlay() {
    overlay.hidden = true;
    canvas.inert = false;
    delete root.dataset.results;
  }

  /** Shows a card over the sky. A results card (with its debrief) takes the sky's place instead,
   * at its own height, so nothing in it is cut off on a short screen. */
  function show(html: string, results = html.includes('class="debrief"')) {
    overlay.innerHTML = html;
    overlay.hidden = false;
    canvas.inert = true;
    if (results) root.dataset.results = 'true'; else delete root.dataset.results;
    overlay.querySelector<HTMLButtonElement>('[data-go]')?.addEventListener('click', start);
    overlay.querySelector<HTMLButtonElement>('[data-next]')?.addEventListener('click', () => { savedLevel = ++levelIndex; savePrefs(); ready(); start(); });
    overlay.querySelector<HTMLButtonElement>('[data-again]')?.addEventListener('click', () => { ready(); start(); });
  }

  function start() {
    if (mode === 'live' || helpOpen) return;
    if (!game || game.done) ready();
    hideOverlay();
    running = true; paused = false; countdown = 3;
    easyInput.disabled = true;
    pauseBtn.textContent = 'Pause';
    canvas.focus({ preventScroll: true });
    bringIntoView();
    announce('Pass starting. Satellite rises on the left.');
  }

  /** Scrolls so the numbers, the sky and the controls are on screen, if they are not already. */
  function bringIntoView() {
    const a = dashEl.getBoundingClientRect(), b = deckEl.getBoundingClientRect();
    if (a.top >= 0 && b.bottom <= innerHeight) return;
    dashEl.scrollIntoView({ block: 'start', behavior: reduced.matches ? 'auto' : 'smooth' });
  }

  function setPaused(p: boolean, focus = true) {
    if (!running || mode === 'live' || helpOpen) return;
    paused = p;
    clearAim();
    pauseBtn.textContent = p ? 'Resume' : 'Pause';
    if (p) {
      show(`<h3>Paused</h3><p>Press P or the button to keep going.</p><button type="button" class="go" data-resume>Resume</button>`);
      const resume = overlay.querySelector<HTMLButtonElement>('[data-resume]')!;
      resume.addEventListener('click', () => setPaused(false));
      if (focus) resume.focus({ preventScroll: true });
    } else {
      hideOverlay();
      canvas.focus({ preventScroll: true });
    }
    announce(p ? 'Paused.' : 'Resumed.');
  }
  pauseBtn.addEventListener('click', () => {
    if (helpOpen) { closeHowto(); return; }
    if (running) setPaused(!paused);
    else start();
  });
  $('[data-restart]').addEventListener('click', () => {
    if (mode === 'missions' && !helpOpen) { ready(); start(); }
  });
  document.addEventListener('visibilitychange', () => {
    if (!document.hidden) return;
    clearAim();
    if (running && !paused) setPaused(true, false);
  });

  // ---- SF and transmitter commands ----
  const sfButtons = [...root.querySelectorAll<HTMLButtonElement>('[data-sf]')];
  sfButtons.forEach(b => b.addEventListener('click', () => { if (game) { G.setSf(game, Number(b.dataset.sf)); hud(); } }));
  $('[data-tx]').addEventListener('click', () => { if (game) { G.setTransmit(game, !game.transmit); hud(); } });

  const antennaSelect = $<HTMLSelectElement>('[data-antenna]');
  const antennaNote = $('[data-antenna-note]');
  antennaSelect.value = antennaChoice;
  antennaSelect.addEventListener('change', () => {
    const value = antennaSelect.value;
    antennaChoice = Object.hasOwn(G.ANTENNAS, value) ? value as Antenna : '';
    savePrefs();
    if (game) {
      game.antenna = antennaChoice || game.level.antenna;
      refreshReceiver();
    }
  });

  const engageButton = $<HTMLButtonElement>('[data-rx-engage]');
  const tuneInput = $<HTMLInputElement>('[data-rx-tune]');
  const gainInput = $<HTMLInputElement>('[data-rx-gain]');
  const levelInput = $<HTMLInputElement>('[data-rx-level]');
  const receiverHud = {
    peak: $('[data-rx-input]'), status: $('[data-rx-state]'), meter: $('[data-rx-meter]'),
    tune: $('[data-rx-tune-label]'), gain: $('[data-rx-gain-label]'), level: $('[data-rx-level-label]'),
  };
  function refreshReceiver() {
    if (!game) return;
    Object.assign(game.receiver, receiver);
    game.snr = G.readReceiver(game).snr;
    game.locked = game.snr >= G.SF[game.sf as 7 | 9 | 12].minSnr;
    hud();
    draw();
  }
  engageButton.addEventListener('click', () => {
    receiver.engaged = !receiver.engaged;
    refreshReceiver();
  });
  tuneInput.addEventListener('input', () => {
    receiver.offset = tuneInput.valueAsNumber;
    refreshReceiver();
  });
  gainInput.addEventListener('input', () => {
    receiver.gainDb = gainInput.valueAsNumber;
    refreshReceiver();
  });
  levelInput.addEventListener('input', () => {
    receiver.level = levelInput.valueAsNumber;
    hud();
  });

  // ---- Keyboard: arrows or WASD aim, 1/2/3 SF, Space transmitter, P pause, Enter start. ----
  const aimKeys = ['ArrowUp', 'ArrowDown', 'ArrowLeft', 'ArrowRight', 'w', 'a', 's', 'd'];
  root.addEventListener('keydown', e => {
    const target = e.target as HTMLElement;
    if (e.altKey || e.ctrlKey || e.metaKey || target.matches('select, textarea, input:not([type=checkbox])')) return;
    // The Easy checkbox keeps Space for itself but still lets P and Esc pause.
    if (target.matches('input') && e.key !== 'p' && e.key !== 'P' && e.key !== 'Escape') return;
    if (helpOpen) {
      if (e.key === 'Escape') { e.preventDefault(); closeHowto(); }
      return;
    }
    const k = e.key.length === 1 ? e.key.toLowerCase() : e.key;
    const inCanvas = e.target === canvas;
    // Arrows also steer from the game's buttons (after a tap on the pad, say), not only the sky.
    const aimFrom = inCanvas || (target.matches('button') && !target.matches('[data-level], [data-mode], [data-rx-engage]'));
    if (aimKeys.includes(k) && aimFrom) { keys.add(k); pointer = null; e.preventDefault(); }
    else if (k === '1' || k === '2' || k === '3') { if (game) G.setSf(game, G.SF_ORDER[Number(k) - 1]); hud(); }
    else if (k === ' ' && inCanvas) { if (game) G.setTransmit(game, !game.transmit); hud(); e.preventDefault(); }
    else if (!e.repeat && (k === 'p' || k === 'Escape') && running && mode === 'missions') { e.preventDefault(); setPaused(!paused); }
    else if (k === 'Enter' && inCanvas && !running && mode === 'missions') { e.preventDefault(); start(); }
  });
  root.addEventListener('keyup', e => keys.delete(e.key.length === 1 ? e.key.toLowerCase() : e.key));
  canvas.addEventListener('blur', () => keys.clear());
  window.addEventListener('blur', clearAim);

  // ---- The big arrow buttons: hold one to keep turning (they work like the arrow keys). ----
  for (const b of root.querySelectorAll<HTMLButtonElement>('[data-aim]')) {
    const key = b.dataset.aim!;
    const down = (e: PointerEvent) => {
      if (e.button !== 0 || helpOpen) return;
      e.preventDefault();
      if (!running && mode === 'missions' && overlay.hidden) start();
      keys.add(key); pointer = null; b.dataset.held = 'true';
      b.setPointerCapture(e.pointerId);
    };
    const up = () => { keys.delete(key); b.dataset.held = 'false'; };
    b.addEventListener('pointerdown', down);
    for (const ev of ['pointerup', 'pointercancel', 'lostpointercapture']) b.addEventListener(ev, up);
    // Keyboard users: Enter or Space on a focused arrow button nudges the antenna.
    b.addEventListener('keydown', e => { if (e.key === 'Enter' || e.key === ' ') { e.preventDefault(); e.stopPropagation(); keys.add(key); } });
    b.addEventListener('keyup', e => { if (e.key === 'Enter' || e.key === ' ') up(); });
    b.addEventListener('blur', up);
    b.addEventListener('contextmenu', e => e.preventDefault());
  }

  // ---- Pointer: drag on touch; on desktop the antenna follows the mouse over the sky. ----
  const at = (e: PointerEvent) => { const r = canvas.getBoundingClientRect(); return fromXY(e.clientX - r.left, e.clientY - r.top); };
  canvas.addEventListener('pointerdown', e => {
    if (e.button !== 0 || pointerId !== null) return;
    pointerId = e.pointerId;
    pointer = at(e);
    canvas.setPointerCapture(e.pointerId);
  });
  canvas.addEventListener('pointermove', e => {
    if (e.pointerId === pointerId || (pointerId === null && e.pointerType === 'mouse')) pointer = at(e);
  });
  const endPointer = (e: PointerEvent) => {
    if (pointerId !== e.pointerId) return;
    pointerId = null;
    if (e.type !== 'pointerup' || e.pointerType !== 'mouse') pointer = null;
  };
  for (const event of ['pointerup', 'pointercancel', 'lostpointercapture'] as const) canvas.addEventListener(event, endPointer);
  canvas.addEventListener('pointerleave', () => { if (pointerId === null) pointer = null; });

  function clearAim() {
    keys.clear();
    pointerId = null;
    pointer = null;
    for (const b of root.querySelectorAll<HTMLElement>('[data-held="true"]')) b.dataset.held = 'false';
  }

  function input() {
    const r = { az: 0, el: 0 };
    if (keys.has('ArrowLeft') || keys.has('a')) r.az -= 1;
    if (keys.has('ArrowRight') || keys.has('d')) r.az += 1;
    if (keys.has('ArrowUp') || keys.has('w')) r.el += 1;
    if (keys.has('ArrowDown') || keys.has('s')) r.el -= 1;
    if (r.az || r.el) return { rate: r };
    return pointer ? { target: pointer } : {};
  }

  // ---- Status for screen readers, throttled so it is not chatty. ----
  let lastAnnounce = 0, lastLock = false;
  function announce(text: string, force = true) {
    const now = performance.now();
    if (!force && now - lastAnnounce < 2500) return;
    lastAnnounce = now;
    statusLine.textContent = text;
  }

  // ---- The live sky over MoQ ----
  let sky: LiveSky | null = null;
  const skyStatus = $('[data-sky-status]');
  async function joinLive() {
    stations.clear(); feed.length = 0;
    hideOverlay();
    liveSlot = -1;
    running = true; paused = false;
    skyStatus.textContent = 'Loading the live sky…';
    // The MoQ client loads only when someone opens the live sky.
    const { LiveSky } = await import('./net').catch(() => ({ LiveSky: null }));
    if (mode !== 'live') return;
    if (!LiveSky) { skyStatus.textContent = 'The live sky could not load. Flying with demo stations instead.'; root.dataset.sky = 'offline'; addDemoStations(); return; }
    sky ??= new LiveSky({
      status: (s: SkyStatus, detail?: string) => {
        if (mode !== 'live' && s !== 'off') return; // a late answer after leaving the live sky
        const text: Record<SkyStatus, string> = {
          off: 'Not connected.',
          connecting: 'Connecting to the MoQ relay…',
          live: `Live over MoQ as ${sky?.callsign}. Other stations appear as dashed circles.`,
          offline: `${detail ?? 'The live sky is offline.'} Flying with demo stations on this computer instead.`,
        };
        skyStatus.textContent = text[s];
        root.dataset.sky = s;
        if (s === 'offline' && mode === 'live') addDemoStations();
        if (s === 'live') for (const [id, v] of stations) if (v.demo) stations.delete(id);
      },
      station: (id, state) => stations.set(id, { state, seen: performance.now() }),
      packet: (id, note: PacketNote) => {
        const cs = stations.get(id)?.state.cs ?? 'A station';
        pushFeed(`${cs} caught packet #${note.seq} on SF${note.sf} (${note.snr} dB): ${note.tempC} °C, light ${note.light}`);
      },
      leave: id => stations.delete(id),
    });
    void sky.start();
  }
  function pushFeed(line: string) {
    feed.unshift(line); if (feed.length > 6) feed.length = 6;
    $('[data-feed]').replaceChildren(...feed.map(t => Object.assign(document.createElement('li'), { textContent: t })));
  }

  // Demo stations: simple robots that track with a lag, so the live sky can be tried offline.
  function addDemoStations() {
    ['GS-DM1', 'GS-DM2', 'GS-DM3'].forEach((cs, i) => stations.set('demo' + i, {
      demo: true, seen: performance.now(),
      state: { cs, az: -60, el: 10, sf: G.SF_ORDER[i], snr: -30, locked: false, rx: 0, score: 0, slot: liveSlot },
    }));
  }
  function moveDemoStations(dt: number) {
    if (!game) return;
    let i = 0;
    for (const [id, v] of stations) {
      if (!v.demo) continue;
      const s = v.state, lag = 0.04 + i * 0.03, wobble = Math.sin(game.t * (0.7 + i * 0.4)) * (6 + i * 3);
      const sat = G.satPosition(game.level, Math.max(0, game.t / game.level.duration - lag));
      s.az += (sat.az + wobble - s.az) * Math.min(1, dt * 3);
      s.el += (Math.max(0, sat.el - wobble / 2) - s.el) * Math.min(1, dt * 3);
      s.snr = G.snrAt(game, s.az, s.el, game.t, G.LIVE.antenna);
      const floor = G.SF[s.sf as 7 | 9 | 12].minSnr;
      s.locked = s.snr >= floor;
      if (s.slot !== liveSlot) { s.slot = liveSlot; s.rx = 0; s.score = 0; }
      if (s.locked && Math.random() < dt / G.SF[s.sf as 7 | 9 | 12].airtime) {
        s.rx++; s.score += 100;
        if (Math.random() < 0.3) pushFeed(`${s.cs} (demo) caught packet #${s.rx} on SF${s.sf}`);
      }
      v.seen = performance.now();
      void id; i++;
    }
  }

  /** The shared clock: this computer's time, corrected by the server's when online. */
  const sharedNow = () => Date.now() + (sky?.clockOffset ?? 0);

  function liveTick() {
    const { slot, elapsed, seed } = G.liveSlot(sharedNow());
    if (slot !== liveSlot) {
      liveSlot = slot;
      game = G.createGame(G.LIVE, seed, antennaChoice || G.LIVE.antenna, receiver);
      pops.length = 0;
      hud();
    }
    const skipped = G.syncLiveTime(game!, elapsed);
    return skipped ? 0 : Math.max(0, elapsed - game!.t);
  }

  // ---- Main loop ----
  function frame(now: number) {
    const dt = Math.min(0.1, (now - last) / 1000);
    last = now;
    if (mode === 'live' && running) {
      // Follow the shared clock: small lags (a slow frame) are caught up in short steps, so
      // they never build up into a skip that would throw away the packet on air.
      let pending = liveTick();
      if (game && !game.done && (pending > 0 || game.t >= G.LIVE.duration)) {
        for (let n = 0; n < 12 && !game.done && (pending > 0.001 || game.t >= G.LIVE.duration); n++) {
          const stepDt = Math.min(pending, 0.05);
          update(stepDt);
          pending -= stepDt;
          if (stepDt === 0) break;
        }
      } else if (game) game.events.length = 0;
      moveDemoStations(dt);
      sky?.sendStation({ az: round(game!.ant.az), el: round(game!.ant.el), sf: game!.sf, snr: round(game!.snr), locked: game!.locked, rx: game!.rx, score: game!.score, slot: liveSlot, antenna: game!.antenna as Antenna });
    } else if (running && !paused && game) {
      if (countdown > 0) { countdown -= dt; if (countdown <= 0) announce('Acquisition of signal. Track the satellite.'); }
      else update(dt * G.MISSION_SPEED);
    }
    for (const [id, v] of stations) if (!v.demo && now - v.seen > 6000) stations.delete(id);
    for (const p of pops) p.age += dt;
    while (pops.length && pops[0].age > 1.2) pops.shift();
    // Drawing the sky is the costly part: skip it while the game is scrolled out of view.
    if (skyVisible) draw();
    requestAnimationFrame(frame);
  }
  let skyVisible = true;
  new IntersectionObserver(([e]) => { skyVisible = e.isIntersecting; }).observe(canvas);
  const round = (v: number) => Math.round(v * 10) / 10;

  function update(dt: number) {
    if (!game) return;
    G.step(game, input(), dt);
    for (const e of game.events) {
      if (e.type === 'rx') {
        beep(1250, 70, 'square', 0.03);
        const sat = G.satPosition(game.level, game.t / game.level.duration);
        if (!reduced.matches) pops.push({ x: toX(sat.az), y: toY(sat.el) - 18, text: `+${e.points}`, age: 0 });
        if (mode === 'live' && e.reading) sky?.sendPacket({ ...e.reading, slot: liveSlot } as PacketNote);
      } else if (e.type === 'lost') {
        beep(180, 120, 'sawtooth', 0.03);
        announce(`Packet ${e.seq} lost. Signal too weak for SF${game.sf}.`, false);
      } else if (e.type === 'safe') {
        beep(120, 400, 'triangle'); announce('Battery low. The satellite is in safe mode and stopped sending data.');
      } else if (e.type === 'resume') {
        announce('Battery recovered. The satellite is sending data again.');
      } else if (e.type === 'done') {
        finish();
      }
    }
    game.events.length = 0;
    unlockedFor = game.locked ? 0 : unlockedFor + dt;
    if (game.locked !== lastLock) { lastLock = game.locked; announce(game.locked ? `Locked on, SF${game.sf}.` : 'Signal lost. Re-aim the antenna or try a higher SF.', false); }
    hud();
  }

  function finish() {
    if (!game) return;
    const s = G.stars(game);
    beep(s ? 880 : 220, 300, 'triangle');
    if (mode === 'live') {
      const liveTip = game.seq > 0 ? G.debrief(game) : null;
      announce(`Pass over. You caught ${game.rx} packets.${liveTip ? ' Next time: ' + liveTip.text : ''} The next live pass starts at the top of the minute.`);
      return;
    }
    running = false;
    easyInput.disabled = false;
    pauseBtn.textContent = 'Start';
    const id = G.LEVELS[levelIndex].id;
    if (s > (best[id] ?? 0)) { best[id] = s; try { localStorage.setItem(bestKey, JSON.stringify(best)); } catch { /* private mode */ } }
    renderLevels();
    const more = levelIndex < G.LEVELS.length - 1;
    const lessons = [
      'The antenna has to point at the satellite. A ground station follows each pass from horizon to horizon.',
      'Far-away satellites are weaker. A higher spreading factor trades speed for range.',
      'Radio noise matters as much as signal. SNR is signal compared with noise.',
      'A CubeSat lives on its solar panels. In Earth\u2019s shadow every packet spends stored energy.',
      'High-gain antennas hear farther, but only where they point.',
    ];
    const tip = G.debrief(game);
    show(`<h3>${s ? 'Pass complete!' : 'Loss of signal'}</h3>
      <p class="stars" aria-label="${s} of 3 stars">${'★'.repeat(s)}${'☆'.repeat(3 - s)}</p>
      <p><strong>${game.rx}</strong> packets · <strong>${game.lost}</strong> lost · best streak <strong>${game.bestStreak}</strong> · <strong>${game.score}</strong> credits${game.battery >= 20 && game.seq > 0 ? ' (with +200 credits for a healthy battery)' : ''}</p>
      ${debriefChart(game)}
      ${tip ? `<p class="tip"><strong>Next time:</strong> ${tip.text}</p>
      <p class="learn"><a href="#step-${tip.step}">How: ${tip.stepTitle}</a> · <a href="${siteBase}${tip.learn}" target="_blank" rel="noopener">${tip.learnTitle}<span class="sr-only"> (opens in a new tab)</span></a></p>` : '<p class="tip"><strong>Clean pass!</strong> Every packet that was sent got through.</p>'}
      <p>${lessons[levelIndex]}</p>
      <div class="row">${s && more ? '<button type="button" class="go" data-next>Next level</button>' : ''}<button type="button" class="${s && more ? '' : 'go'}" data-again>Try again</button></div>`);
    overlay.querySelector<HTMLButtonElement>('button')?.focus({ preventScroll: true });
    overlay.scrollIntoView({ block: 'nearest', behavior: reduced.matches ? 'auto' : 'smooth' });
    announce(`Pass complete. ${game.rx} packets, ${s} of 3 stars.${tip ? ' Next time: ' + tip.text : ''}`);
  }

  /**
   * The pass at a glance: signal over time against the line the radio setting needs, the
   * shadow, and a tick for every packet (green caught, red lost). Labelled as a simulation.
   */
  function debriefChart(g: NonNullable<typeof game>) {
    const Wc = 320, Hc = 112, x0 = 8, x1 = Wc - 8, top = 8, bottom = 86;
    const x = (t: number) => (x0 + (t / g.level.duration) * (x1 - x0)).toFixed(1);
    const y = (db: number) => (top + ((10 - Math.max(-40, Math.min(10, db))) / 50) * (bottom - top)).toFixed(1);
    const parts: string[] = [];
    if (g.level.eclipse) {
      const [a, b] = g.level.eclipse;
      parts.push(`<rect x="${x(a * g.level.duration)}" y="${top}" width="${(Number(x(b * g.level.duration)) - Number(x(a * g.level.duration))).toFixed(1)}" height="${bottom - top}" fill="#000" opacity=".35"/>`);
      parts.push(`<text x="${x(((a + b) / 2) * g.level.duration)}" y="${top + 11}" text-anchor="middle" font-size="9" fill="#b9c2ff">shadow</text>`);
    }
    // What the radio setting needs, as a step line (it moves when the setting changes).
    const floor = g.trace.map((p, i) => `${i ? 'L' : 'M'}${x(p.t)} ${y(p.floor)}`).join(' ');
    parts.push(`<path d="${floor}" fill="none" stroke="#ffd23f" stroke-width="1.5" stroke-dasharray="4 3"/>`);
    // Signal, broken where the satellite was resting or in safe mode.
    let d = '', pen = false;
    for (const p of g.trace) {
      if (!p.on || p.snr <= -39.5) { pen = false; continue; }
      d += `${pen ? 'L' : 'M'}${x(p.t)} ${y(p.snr)}`;
      pen = true;
    }
    parts.push(`<path d="${d}" fill="none" stroke="#7df0a6" stroke-width="2" stroke-linejoin="round"/>`);
    for (const k of g.packets) parts.push(`<rect x="${(Number(x(k.t)) - 1.5).toFixed(1)}" y="${bottom + 6}" width="3" height="9" rx="1" fill="${k.ok ? '#7df0a6' : '#ef6b6c'}"/>`);
    parts.push(`<line x1="${x0}" x2="${x1}" y1="${bottom}" y2="${bottom}" stroke="#ffffff33"/>`);
    parts.push(`<text x="${x0}" y="${Hc - 2}" font-size="9" fill="#b9c2ff">rise</text><text x="${x1}" y="${Hc - 2}" font-size="9" fill="#b9c2ff" text-anchor="end">set</text>`);
    const label = `Signal over the pass. ${g.rx} packets caught and ${g.lost} lost. The dashed line is the signal your radio setting needs.`;
    return `<figure class="debrief"><svg viewBox="0 0 ${Wc} ${Hc}" role="img" aria-label="${label}"><title>${label}</title>${parts.join('')}</svg>
      <figcaption><span class="k sig"></span>signal <span class="k need"></span>needed <span class="k ok"></span>caught <span class="k lost"></span>lost · desk simulation</figcaption></figure>`;
  }

  // ---- HUD (HTML, so it is readable and zoomable) ----
  const nowEl = $<HTMLParagraphElement>('[data-now]');
  /** The "what to do now" line: one instruction, and a link to that step on the page. */
  function sayNow(stepId: string, text: string) {
    const i = STEPS.findIndex(s => s.id === stepId);
    const html = `<span class="now-n">Step ${i + 1} of ${STEPS.length}</span> ${text} <a href="#step-${stepId}">How?</a>`;
    if (nowEl.dataset.html !== html) { nowEl.innerHTML = html; nowEl.dataset.html = html; }
  }
  const hudEl = {
    snr: $('[data-snr]'), bar: $<HTMLElement>('[data-snr-bar]'), floor: $<HTMLElement>('[data-snr-floor]'),
    battery: $('[data-battery]'), batteryBar: $<HTMLElement>('[data-battery-bar]'),
    rx: $('[data-rx]'), score: $('[data-score]'), streak: $('[data-streak]'), time: $('[data-time]'),
    tx: $('[data-tx]'), txLabel: $('[data-tx] [data-tx-label]'), light: $('[data-light]'), log: $('[data-log]'), board: $('[data-board]'),
  };
  const pct = (snr: number) => Math.max(0, Math.min(100, (snr + 30) / 40 * 100));
  let lastBoard = 0;
  /** Writes to the page only when a value changed: the HUD updates every frame. */
  function put(el: HTMLElement, text: string) { if (el.textContent !== text) el.textContent = text; }
  function css(el: HTMLElement, prop: 'width' | 'left', value: string) { if (el.style[prop] !== value) el.style[prop] = value; }
  function hud() {
    if (!game) return;
    const left = Math.max(0, game.level.duration - game.t) / (mode === 'missions' ? G.MISSION_SPEED : 1);
    put(hudEl.snr, (game.snr < -60 ? '—' : game.snr.toFixed(1) + ' dB') + (game.locked ? ' · locked' : ''));
    css(hudEl.bar, 'width', pct(game.snr) + '%');
    hudEl.bar.dataset.ok = String(game.locked);
    css(hudEl.floor, 'left', pct(G.SF[game.sf as 7 | 9 | 12].minSnr) + '%');
    put(hudEl.battery, Math.round(game.battery) + '%' + (game.safe ? ' · SAFE MODE' : ''));
    css(hudEl.batteryBar, 'width', game.battery + '%');
    hudEl.batteryBar.dataset.low = String(game.battery < 25);
    put(hudEl.rx, String(game.rx));
    put(hudEl.score, String(game.score));
    put(hudEl.streak, String(game.streak));
    put(hudEl.time, left.toFixed(0) + ' s');
    put(hudEl.light, G.inEclipse(game.level, game.t / game.level.duration) ? 'In Earth\u2019s shadow' : 'Sunlit');
    hudEl.tx.setAttribute('aria-pressed', String(game.transmit));
    put(hudEl.txLabel, game.transmit ? 'Sending: on' : 'Sending: resting');
    const antenna = G.ANTENNAS[game.antenna as Antenna];
    put(antennaNote, `${antenna.name} · ${antenna.note}`);
    const { peak, status } = G.readReceiver(game);
    put(receiverHud.peak, inputPercent(peak) + '%');
    put(receiverHud.status, status);
    css(receiverHud.meter, 'width', Math.min(100, peak * 100) + '%');
    receiverHud.meter.dataset.clip = String(peak >= 1);
    engageButton.setAttribute('aria-pressed', String(receiver.engaged));
    put(engageButton, receiver.engaged ? 'Engaged' : 'Bypassed');
    put(receiverHud.tune, G.receiverFrequency(receiver.offset).toFixed(3) + ' MHz');
    put(receiverHud.gain, receiver.gainDb + ' dB');
    put(receiverHud.level, receiver.level + '%');
    sfButtons.forEach(b => b.setAttribute('aria-pressed', String(Number(b.dataset.sf) === game!.sf)));
    if (!running) sayNow('start', 'Press Start when you are ready.');
    else if (countdown > 0) sayNow('track', 'Get ready: the satellite rises on the left.');
    else if (paused) sayNow('start', 'Paused. Press Resume to keep going.');
    else if (game.safe) sayNow('battery', 'The battery is low. Press Sending to let the satellite rest.');
    else if (!receiver.engaged) sayNow('receiver', 'The receiver is off. Press Engaged on the receiver pedal.');
    else if (game.locked) sayNow('track', 'You are catching packets. Keep the circle on the satellite.');
    else sayNow('track', 'Move the circle onto the satellite until the signal bar turns green.');
    if (game.log[0] && hudEl.log.dataset.seq !== String(game.log[0].seq)) {
      hudEl.log.dataset.seq = String(game.log[0].seq);
      hudEl.log.replaceChildren(...game.log.slice(0, 5).map(r => Object.assign(document.createElement('li'), {
        textContent: `#${r.seq} SF${r.sf} ${r.snr} dB · ${r.tempC} °C · light ${r.light} · battery ${r.battery}%`,
      })));
    }
    const now = performance.now();
    if (mode === 'live' && now - lastBoard > 500) {
      lastBoard = now;
      const rows = [...stations.values()].filter(v => v.state.slot === liveSlot).map(v => ({ cs: v.state.cs + (v.demo ? ' (demo)' : ''), rx: v.state.rx, score: v.state.score }));
      rows.push({ cs: (sky?.status === 'live' ? sky.callsign : 'You') + ' (you)', rx: game.rx, score: game.score });
      rows.sort((a, b) => b.score - a.score);
      const total = rows.reduce((n, r) => n + r.rx, 0);
      $('[data-total]').textContent = `${rows.length} station${rows.length === 1 ? '' : 's'} · ${total} packets caught together this pass`;
      hudEl.board.replaceChildren(...rows.slice(0, 8).map(r => Object.assign(document.createElement('li'), { textContent: `${r.cs}: ${r.rx} packets, ${r.score} credits` })));
    }
  }

  // ---- Drawing ----
  let skyLabel = '';
  let lastHint = 0;
  /** The coach's hint goes to the status line now and then, not every frame. */
  function coachHint(text: string) {
    const now = performance.now();
    if (now - lastHint < 4000) return;
    lastHint = now;
    announce(text, false);
  }
  /** The canvas label changes only when what it says changes, not every frame. */
  function setLabel(text: string) { if (text !== skyLabel) { skyLabel = text; canvas.setAttribute('aria-label', text); } }

  function draw() {
    const w = W(), h = H();
    if (!w || !h) return;
    const t = performance.now() / 1000;
    const gy = h - pad.b;
    const g = ctx.createLinearGradient(0, 0, 0, gy);
    g.addColorStop(0, '#070b2e'); g.addColorStop(0.55, '#1a2470'); g.addColorStop(0.85, '#4b3a8f'); g.addColorStop(1, '#c46a7a');
    ctx.fillStyle = g; ctx.fillRect(0, 0, w, h);
    const glow = ctx.createRadialGradient(w / 2, gy, 0, w / 2, gy, w * 0.6);
    glow.addColorStop(0, 'rgba(255,170,110,.35)'); glow.addColorStop(1, 'rgba(255,170,110,0)');
    ctx.fillStyle = glow; ctx.fillRect(0, 0, w, gy);
    for (const s of stars) {
      ctx.globalAlpha = reduced.matches ? 0.6 : 0.4 + 0.3 * Math.sin(t * 1.5 + s.p);
      ctx.fillStyle = '#fff'; ctx.fillRect(s.x * w, s.y * (h - pad.b), s.s, s.s);
    }
    ctx.globalAlpha = 1;
    // Elevation grid
    ctx.strokeStyle = 'rgba(255,255,255,.18)'; ctx.fillStyle = 'rgba(255,255,255,.6)'; ctx.font = '11px "IBM Plex Mono", monospace'; ctx.lineWidth = 1;
    ctx.strokeStyle = 'rgba(255,255,255,.12)'; ctx.setLineDash([2, 6]);
    for (const el of [30, 60]) { ctx.beginPath(); ctx.moveTo(pad.l, toY(el)); ctx.lineTo(w - pad.r, toY(el)); ctx.stroke(); ctx.fillText(el + '°', 6, toY(el) + 4); }
    ctx.setLineDash([]);
    // Ground: two rows of hills and a ground station dish that points where your antenna points
    ctx.fillStyle = '#2a2158';
    ctx.beginPath(); ctx.moveTo(0, gy);
    for (let x = 0; x <= w; x += 8) ctx.lineTo(x, gy - 10 - 8 * Math.sin(x / 70) - 5 * Math.sin(x / 23 + 1));
    ctx.lineTo(w, h); ctx.lineTo(0, h); ctx.fill();
    const gg = ctx.createLinearGradient(0, gy, 0, h);
    gg.addColorStop(0, '#141a44'); gg.addColorStop(1, '#0a0f2c');
    ctx.fillStyle = gg;
    ctx.beginPath(); ctx.moveTo(0, gy + 4);
    for (let x = 0; x <= w; x += 8) ctx.lineTo(x, gy + 2 - 4 * Math.sin(x / 45 + 2));
    ctx.lineTo(w, h); ctx.lineTo(0, h); ctx.fill();
    {
      const dx = w / 2, dy = gy + 6;
      const aim = game ? Math.atan2(toY(game.ant.el) - dy, toX(game.ant.az) - dx) : -Math.PI / 2;
      ctx.fillStyle = '#5f6aa8'; ctx.fillRect(dx - 2, dy, 4, 16); ctx.fillRect(dx - 9, dy + 14, 18, 4);
      ctx.save(); ctx.translate(dx, dy); ctx.rotate(aim + Math.PI / 2);
      ctx.fillStyle = '#d9def5'; ctx.beginPath(); ctx.ellipse(0, 0, 13, 6, 0, Math.PI, 0); ctx.fill();
      ctx.strokeStyle = '#d9def5'; ctx.lineWidth = 1.5; ctx.beginPath(); ctx.moveTo(0, -2); ctx.lineTo(0, -11); ctx.stroke();
      ctx.fillStyle = game && game.locked ? '#7ff0a8' : '#ffd23f'; ctx.beginPath(); ctx.arc(0, -12, 2.2, 0, Math.PI * 2); ctx.fill();
      ctx.restore();
    }
    ctx.fillStyle = 'rgba(255,255,255,.55)'; ctx.font = '10px "IBM Plex Mono", monospace'; ctx.fillText('horizon', 6, h - 8);
    ctx.font = '11px "IBM Plex Mono", monospace';
    if (!game) return;
    const level = game.level;
    // Interference source
    if (level.interference) {
      const x = toX(G.NEIGHBOR.az), y = h - pad.b;
      ctx.strokeStyle = '#ef6b6c'; ctx.lineWidth = 2;
      ctx.beginPath(); ctx.moveTo(x - 6, y); ctx.lineTo(x, y - 18); ctx.lineTo(x + 6, y); ctx.stroke();
      const rings = reduced.matches ? [10, 20] : [0, 1, 2].map(i => ((t * 18 + i * 10) % 30) + 6);
      for (const r of rings) { ctx.globalAlpha = 1 - r / 36; ctx.beginPath(); ctx.arc(x, y - 18, r, Math.PI * 1.1, Math.PI * 1.9); ctx.stroke(); }
      ctx.globalAlpha = 1; ctx.fillStyle = '#ef6b6c'; ctx.fillText('915 MHz noise', x - 40, y + 26);
    }
    // Predicted track (dotted), grey in Earth's shadow
    ctx.lineWidth = 2; ctx.setLineDash([3, 6]);
    for (let i = 0; i < 60; i++) {
      const a = G.satPosition(level, i / 60), b = G.satPosition(level, (i + 1) / 60);
      ctx.strokeStyle = G.inEclipse(level, i / 60) ? 'rgba(160,160,180,.45)' : 'rgba(255,210,63,.55)';
      ctx.beginPath(); ctx.moveTo(toX(a.az), toY(a.el)); ctx.lineTo(toX(b.az), toY(b.el)); ctx.stroke();
    }
    ctx.setLineDash([]);
    const p = game.t / level.duration;
    const sat = G.satPosition(level, p);
    const shade = G.inEclipse(level, p);
    const pxPerDeg = (w - pad.l - pad.r) / 180;
    // Other stations
    for (const v of stations.values()) {
      const s = v.state;
      ctx.strokeStyle = 'rgba(185,164,255,.75)'; ctx.setLineDash([4, 4]); ctx.lineWidth = 1.5;
      const radius = G.antennaFor(G.LIVE, s.antenna).beam / 2 * pxPerDeg;
      ctx.beginPath(); ctx.arc(toX(s.az), toY(s.el), radius, 0, Math.PI * 2); ctx.stroke();
      ctx.setLineDash([]); ctx.fillStyle = 'rgba(185,164,255,.9)'; ctx.fillText(s.cs, toX(s.az) + 6, toY(s.el) - 6);
    }
    // Satellite
    if (!game.done && p > 0) {
      const x = toX(sat.az), y = toY(sat.el);
      if (game.transmit && !game.safe) {
        const halo = ctx.createRadialGradient(x, y, 0, x, y, 26);
        halo.addColorStop(0, shade ? 'rgba(160,170,210,.35)' : 'rgba(255,220,120,.45)'); halo.addColorStop(1, 'rgba(255,220,120,0)');
        ctx.fillStyle = halo; ctx.beginPath(); ctx.arc(x, y, 26, 0, Math.PI * 2); ctx.fill();
        ctx.strokeStyle = 'rgba(255,255,255,.55)'; ctx.lineWidth = 1.5;
        const rr = reduced.matches ? [14] : [0, 1].map(i => 10 + ((t * 18 + i * 9) % 18));
        for (const r of rr) { ctx.globalAlpha = 1 - (r - 10) / 18; ctx.beginPath(); ctx.arc(x, y, r, 0, Math.PI * 2); ctx.stroke(); }
        ctx.globalAlpha = 1;
      }
      // Solar panels with cells, a gold body and a little antenna
      for (const side of [-1, 1]) {
        const px = side < 0 ? x - 22 : x + 8;
        ctx.fillStyle = shade ? '#3f4668' : '#2f63c9'; ctx.fillRect(px, y - 5, 14, 10);
        ctx.strokeStyle = shade ? '#59607e' : '#a4c9ff'; ctx.lineWidth = 1;
        ctx.strokeRect(px + .5, y - 4.5, 13, 9);
        ctx.beginPath(); ctx.moveTo(px + 7, y - 5); ctx.lineTo(px + 7, y + 5); ctx.moveTo(px, y); ctx.lineTo(px + 14, y); ctx.stroke();
        ctx.fillStyle = '#9aa3c8'; ctx.fillRect(side < 0 ? x - 8 : x + 6, y - 1, 2, 2);
      }
      const body = ctx.createLinearGradient(x - 6, y - 6, x + 6, y + 6);
      body.addColorStop(0, shade ? '#a0a6c0' : '#fff2b0'); body.addColorStop(1, shade ? '#6a7092' : '#e0a91c');
      ctx.fillStyle = body; ctx.fillRect(x - 6, y - 6, 12, 12);
      ctx.strokeStyle = 'rgba(0,0,0,.35)'; ctx.strokeRect(x - 5.5, y - 5.5, 11, 11);
      ctx.strokeStyle = '#e8ecff'; ctx.beginPath(); ctx.moveTo(x, y - 6); ctx.lineTo(x + 4, y - 13); ctx.stroke();
      if (game.safe) { ctx.fillStyle = '#ef6b6c'; ctx.fillText('SAFE MODE', x + 10, y + 18); }
    }
    // Noise burst
    if (game.bursts.some(b => game!.t >= b.start && game!.t < b.end)) {
      ctx.strokeStyle = '#ef6b6c'; ctx.lineWidth = 4; ctx.strokeRect(2, 2, w - 4, h - pad.b - 2);
      ctx.fillStyle = '#ef6b6c'; ctx.fillText('NOISE BURST', w / 2 - 40, 30);
      if (!reduced.matches) for (let i = 0; i < 120; i++) { ctx.fillStyle = 'rgba(255,255,255,.25)'; ctx.fillRect(Math.random() * w, Math.random() * (h - pad.b), 2, 2); }
    }
    // Your antenna beam
    const ax = toX(game.ant.az), ay = toY(game.ant.el);
    const br = G.antennaFor(level, game.antenna).beam / 2 * pxPerDeg;
    const col = game.locked ? '111,211,160' : '255,210,63';
    ctx.strokeStyle = `rgba(${col},.25)`; ctx.lineWidth = 2; ctx.setLineDash([4, 6]);
    ctx.beginPath(); ctx.moveTo(w / 2, h - pad.b + 2); ctx.lineTo(ax, ay); ctx.stroke(); ctx.setLineDash([]);
    const fill = ctx.createRadialGradient(ax, ay, br * 0.2, ax, ay, br);
    fill.addColorStop(0, `rgba(${col},.05)`); fill.addColorStop(1, `rgba(${col},${game.locked ? .32 : .2})`);
    ctx.fillStyle = fill; ctx.beginPath(); ctx.arc(ax, ay, br, 0, Math.PI * 2); ctx.fill();
    ctx.strokeStyle = `rgb(${col})`; ctx.lineWidth = 3;
    if (game.locked && !reduced.matches) { ctx.shadowColor = `rgb(${col})`; ctx.shadowBlur = 12; }
    ctx.beginPath(); ctx.arc(ax, ay, br, 0, Math.PI * 2); ctx.stroke(); ctx.shadowBlur = 0;
    ctx.lineWidth = 1.5; ctx.beginPath(); ctx.moveTo(ax - 7, ay); ctx.lineTo(ax + 7, ay); ctx.moveTo(ax, ay - 7); ctx.lineTo(ax, ay + 7); ctx.stroke();
    // Beginner coach: on the first two levels, after a few seconds without a lock, point from the
    // antenna circle toward the satellite and say which way to turn.
    const coach = mode === 'missions' && levelIndex <= 1 && running && !paused && countdown <= 0
      && unlockedFor > 2.5 && sat.el > 2 && !game.safe;
    if (coach) {
      const sx = toX(sat.az), sy = toY(sat.el);
      const dx = sx - ax, dy = sy - ay, dist = Math.hypot(dx, dy);
      if (dist > br + 24) {
        const ux = dx / dist, uy = dy / dist;
        const pulse = reduced.matches ? 0 : Math.sin(t * 6) * 4;
        const x0 = ax + ux * (br + 6), y0 = ay + uy * (br + 6);
        const x1 = ax + ux * Math.min(dist - 18, br + 46 + pulse), y1 = ay + uy * Math.min(dist - 18, br + 46 + pulse);
        ctx.strokeStyle = '#ffd23f'; ctx.fillStyle = '#ffd23f'; ctx.lineWidth = 4; ctx.lineCap = 'round';
        ctx.beginPath(); ctx.moveTo(x0, y0); ctx.lineTo(x1, y1); ctx.stroke();
        ctx.beginPath(); ctx.moveTo(x1 + ux * 10, y1 + uy * 10);
        ctx.lineTo(x1 - uy * 8, y1 + ux * 8); ctx.lineTo(x1 + uy * 8, y1 - ux * 8); ctx.closePath(); ctx.fill();
        ctx.lineCap = 'butt';
      }
      const way = [sat.el > game.ant.el + 3 ? 'up' : sat.el < game.ant.el - 3 ? 'down' : '', sat.az > game.ant.az + 3 ? 'right' : sat.az < game.ant.az - 3 ? 'left' : ''].filter(Boolean).join(' and ');
      if (way) coachHint(`Move the circle ${way} to find the satellite.`);
    }
    // Floating points
    ctx.font = '800 16px Inter, sans-serif'; ctx.lineWidth = 4; ctx.strokeStyle = 'rgba(6,10,32,.85)'; ctx.lineJoin = 'round';
    for (const pop of pops) { ctx.globalAlpha = Math.max(0, 1 - pop.age / 1.2); ctx.fillStyle = '#7ff0a8'; const py = pop.y - pop.age * 28; ctx.strokeText(pop.text, pop.x - 14, py); ctx.fillText(pop.text, pop.x - 14, py); }
    ctx.globalAlpha = 1;
    // Countdown and live waiting
    ctx.font = '900 64px Inter, sans-serif'; ctx.fillStyle = '#fff'; ctx.textAlign = 'center';
    if (countdown > 0 && running) { const n = Math.ceil(countdown), f = countdown % 1; ctx.globalAlpha = 0.4 + 0.6 * f; ctx.lineWidth = 6; ctx.strokeStyle = 'rgba(6,10,32,.7)'; ctx.strokeText(String(n), w / 2, h / 2); ctx.fillText(String(n), w / 2, h / 2); ctx.globalAlpha = 1; }
    ctx.textAlign = 'left';
    if (mode === 'live') {
      const { elapsed } = G.liveSlot(sharedNow());
      if (elapsed > G.LIVE.duration) {
        ctx.font = 'bold 22px Inter, sans-serif';
        ctx.fillText(`Next live pass in ${Math.ceil(G.LIVE_SLOT_SECONDS - elapsed)} s`, w / 2 - 120, h / 2);
      }
    }
    setLabel(`Sky view. Satellite at elevation ${Math.round(sat.el)} degrees. Your antenna at elevation ${Math.round(game.ant.el)}. Signal ${game.locked ? 'locked' : 'not locked'}.`);
  }

  // ---- Deep links ----
  // A lesson or teacher can link straight to a level or the live sky:
  //   #level=low-pass (or #level=2)   that level, if it is open in this browser
  //   #mode=live                       the shared live pass
  // Step links (#step-track) are ordinary anchors and are left alone.
  const slug = (name: string) => name.toLowerCase().replace(/[^a-z0-9]+/g, '-').replace(/^-|-$/g, '');
  function applyHash() {
    const params = new URLSearchParams(location.hash.slice(1));
    const want = params.get('level'), wantMode = params.get('mode');
    const i = want ? G.LEVELS.findIndex(l => String(l.id) === want || slug(l.name) === want) : -1;
    if (i < 0 && wantMode !== 'live' && wantMode !== 'levels') return false;
    linkedEntry = true;
    if (wantMode === 'live') {
      if (mode !== 'live') tabs.find(t => t.dataset.mode === 'live')?.click();
      bringIntoView();
      return true;
    }

    const locked = i >= 0 && !unlocked(i);
    if (i >= 0) {
      levelIndex = locked ? 0 : i;
      while (locked && levelIndex < i && unlocked(levelIndex + 1)) levelIndex++;
    }
    if (mode !== 'missions') tabs.find(t => t.dataset.mode === 'missions')?.click();
    else ready();

    if (locked) {
      show(`<h3>Level ${i + 1}: ${G.LEVELS[i].name} is locked</h3>
        <p>Earn at least one star on <strong>${G.LEVELS[i - 1].name}</strong> to open it. This browser remembers your stars.</p>
        <button type="button" class="go" data-go>Play level ${levelIndex + 1}: ${G.LEVELS[levelIndex].name}</button>`);
      announce(`${G.LEVELS[i].name} is locked. Play ${G.LEVELS[levelIndex].name} first.`);
    }
    bringIntoView();
    return true;
  }
  addEventListener('hashchange', () => { applyHash(); });

  setTab();
  if (!applyHash()) ready();
  resize();
  requestAnimationFrame(frame);
}
