// Catch the Pass: drawing, input, sound and the live sky. The rules are in logic.mjs.
import * as G from './logic.mjs';
import type { LiveSky, StationState, PacketNote, SkyStatus } from './net';

type Game = ReturnType<typeof G.createGame>;
type Level = (typeof G.LEVELS)[number] | typeof G.LIVE;

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
  const best: Record<string, number> = (() => { try { return JSON.parse(localStorage.getItem(bestKey) ?? '{}'); } catch { return {}; } })();

  let mode: 'missions' | 'live' = 'missions';
  let levelIndex = 0;
  let game: Game | null = null;
  let running = false, paused = false, countdown = 0;
  let slow = true; // Easy mode is on until the player turns it off.
  const howtoKey = 'catch-the-pass:how-to-seen';
  let liveSlot = -1;
  let last = performance.now();
  const keys = new Set<string>();
  let pointer: { az: number; el: number } | null = null;
  let pointerDown = false;
  const pops: { x: number; y: number; text: string; age: number }[] = [];
  const stations = new Map<string, { state: StationState; seen: number; demo?: boolean }>();
  const feed: string[] = [];

  // ---- Sound: off by default, made only after the player turns it on. ----
  let audio: AudioContext | null = null;
  let soundOn = false;
  function beep(freq: number, ms: number, type: OscillatorType = 'sine', gain = 0.06) {
    if (!soundOn || !audio) return;
    const o = audio.createOscillator(), g = audio.createGain();
    o.type = type; o.frequency.value = freq;
    g.gain.setValueAtTime(gain, audio.currentTime);
    g.gain.exponentialRampToValueAtTime(0.0001, audio.currentTime + ms / 1000);
    o.connect(g).connect(audio.destination);
    o.start(); o.stop(audio.currentTime + ms / 1000);
  }
  $('[data-sound]').addEventListener('click', e => {
    soundOn = !soundOn;
    if (soundOn && !audio) audio = new AudioContext();
    (e.currentTarget as HTMLElement).setAttribute('aria-pressed', String(soundOn));
    (e.currentTarget as HTMLElement).textContent = soundOn ? '🔊' : '🔇';
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
  function resize() {
    const w = canvas.clientWidth, h = Math.round(Math.min(w * (w < 560 ? 0.85 : 0.56), Math.max(260, innerHeight * 0.62)));
    canvas.style.height = h + 'px';
    canvas.width = Math.round(w * devicePixelRatio); canvas.height = Math.round(h * devicePixelRatio);
    ctx.setTransform(devicePixelRatio, 0, 0, devicePixelRatio, 0, 0);
    draw();
  }
  new ResizeObserver(resize).observe(canvas);

  const stars = Array.from({ length: 70 }, (_, i) => { const r = G.rng(i + 1); return { x: r(), y: r() * 0.85, s: r() * 1.4 + 0.3, p: r() * 6 }; });

  // ---- Levels and modes ----
  const levelButtons = [...root.querySelectorAll<HTMLButtonElement>('[data-level]')];
  function unlocked(i: number) { return i === 0 || (best[G.LEVELS[i - 1].id] ?? 0) > 0; }
  function renderLevels() {
    levelButtons.forEach((b, i) => {
      const s = best[G.LEVELS[i].id] ?? 0;
      b.disabled = !unlocked(i);
      b.setAttribute('aria-pressed', String(mode === 'missions' && i === levelIndex));
      b.querySelector('[data-stars]')!.textContent = b.disabled ? 'locked' : '★'.repeat(s) + '☆'.repeat(3 - s);
      b.setAttribute('aria-label', `Level ${i + 1}, ${G.LEVELS[i].name}, ${b.disabled ? 'locked' : s + ' of 3 stars'}`);
    });
  }
  levelButtons.forEach((b, i) => b.addEventListener('click', () => { mode = 'missions'; levelIndex = i; sky?.stop(); setTab(); ready(); }));

  const tabs = [...root.querySelectorAll<HTMLButtonElement>('[data-mode]')];
  function setTab() {
    tabs.forEach(t => t.setAttribute('aria-selected', String(t.dataset.mode === mode)));
    $('[data-missions]').hidden = mode !== 'missions';
    $('[data-live]').hidden = mode !== 'live';
    $<HTMLInputElement>('[data-slow]').disabled = mode === 'live';
    // The live pass runs on a shared clock, so it cannot be paused or restarted.
    $<HTMLButtonElement>('[data-pause]').disabled = mode === 'live';
    $<HTMLButtonElement>('[data-restart]').disabled = mode === 'live';
    $('[data-pause]').textContent = 'Start';
    renderLevels();
  }
  tabs.forEach(t => t.addEventListener('click', () => {
    mode = t.dataset.mode as 'missions' | 'live';
    setTab();
    if (mode === 'missions') { sky?.stop(); stations.clear(); ready(); } else void joinLive();
  }));
  $<HTMLInputElement>('[data-slow]').addEventListener('change', e => { slow = (e.target as HTMLInputElement).checked; if (mode === 'missions') ready(); });

  function currentLevel(): Level {
    const level = mode === 'live' ? G.LIVE : G.LEVELS[levelIndex];
    return slow && mode === 'missions' ? G.easyLevel(level) : level;
  }

  function ready() {
    game = G.createGame(currentLevel(), (Math.random() * 2 ** 31) | 0);
    running = false; paused = false; countdown = 0;
    if (mode === 'missions') $('[data-pause]').textContent = 'Start';
    show(`<h3>${mode === 'live' ? 'Live sky' : `Level ${levelIndex + 1}: ${G.LEVELS[levelIndex].name}`}</h3>
      <p>${currentLevel().tip}</p>
      <p class="goal">Goal: ${currentLevel().goal.map((g, i) => `${'★'.repeat(i + 1)} ${g} packets`).join(' · ')}</p>
      <button type="button" class="go" data-go>▶ Start</button>`);
    hud(); draw();
    if (!seenHowto()) showHowto();
  }

  // ---- The first-visit "How to play" card ----
  function seenHowto() { try { return localStorage.getItem(howtoKey) === '1'; } catch { return false; } }
  function showHowto() {
    if (running && mode === 'missions' && !paused) setPaused(true);
    const stage = canvas.parentElement!;
    stage.dataset.panel = 'howto';
    show(`<h3>How to play</h3>
      <ol class="steps">
        <li><span class="pic" aria-hidden="true">🛰️</span><b>1. Find the satellite</b><span>It rises on the left and crosses the sky.</span></li>
        <li><span class="pic" aria-hidden="true">📡</span><b>2. Keep the circle on it</b><span>Use the ▲ ◀ ▶ ▼ buttons, drag on the sky, or press the arrow keys.</span></li>
        <li><span class="pic" aria-hidden="true">📦</span><b>3. Catch packets</b><span>A green signal bar means you are catching packets and earning credits. Lost it? Try 🐢 Far.</span></li>
      </ol>
      <button type="button" class="go" data-howto>Got it!</button>`);
    const got = overlay.querySelector<HTMLButtonElement>('[data-howto]')!;
    got.addEventListener('click', () => {
      try { localStorage.setItem(howtoKey, '1'); } catch { /* private mode */ }
      stage.dataset.panel = '';
      if (mode === 'missions' && !running) ready(); else if (paused) setPaused(false); else overlay.hidden = true;
    });
    got.focus({ preventScroll: true });
  }
  $('[data-help]').addEventListener('click', showHowto);

  function show(html: string) {
    overlay.innerHTML = html;
    overlay.hidden = false;
    overlay.querySelector<HTMLButtonElement>('[data-go]')?.addEventListener('click', start);
    overlay.querySelector<HTMLButtonElement>('[data-next]')?.addEventListener('click', () => { levelIndex++; renderLevels(); ready(); start(); });
    overlay.querySelector<HTMLButtonElement>('[data-again]')?.addEventListener('click', () => { ready(); start(); });
  }

  function start() {
    if (mode === 'live') return;
    if (!game || game.done) ready();
    overlay.hidden = true;
    running = true; paused = false; countdown = 3;
    $('[data-pause]').textContent = 'Pause';
    $('[data-pause]').setAttribute('aria-pressed', 'false');
    canvas.focus();
    announce('Pass starting. Satellite rises on the left.');
  }

  function setPaused(p: boolean) {
    if (!running || mode === 'live') return;
    paused = p;
    $('[data-pause]').textContent = p ? 'Resume' : 'Pause';
    $('[data-pause]').setAttribute('aria-pressed', String(p));
    if (p) show(`<h3>Paused</h3><p>Press P or the button to keep going.</p><button type="button" class="go" data-resume>Resume</button>`);
    else overlay.hidden = true;
    overlay.querySelector('[data-resume]')?.addEventListener('click', () => setPaused(false));
    announce(p ? 'Paused.' : 'Resumed.');
  }
  $('[data-pause]').addEventListener('click', () => (running ? setPaused(!paused) : start()));
  $('[data-restart]').addEventListener('click', () => { if (mode === 'missions') { ready(); start(); } });
  document.addEventListener('visibilitychange', () => { if (document.hidden && running && !paused) setPaused(true); });

  // ---- SF and transmitter commands ----
  const sfButtons = [...root.querySelectorAll<HTMLButtonElement>('[data-sf]')];
  sfButtons.forEach(b => b.addEventListener('click', () => { if (game) { G.setSf(game, Number(b.dataset.sf)); hud(); } }));
  $('[data-tx]').addEventListener('click', () => { if (game) { G.setTransmit(game, !game.transmit); hud(); } });

  // ---- Keyboard: arrows or WASD aim, 1/2/3 SF, Space transmitter, P pause, Enter start. ----
  const aimKeys = ['ArrowUp', 'ArrowDown', 'ArrowLeft', 'ArrowRight', 'w', 'a', 's', 'd'];
  root.addEventListener('keydown', e => {
    if ((e.target as HTMLElement).matches('input, select, textarea')) return;
    const k = e.key.length === 1 ? e.key.toLowerCase() : e.key;
    const inCanvas = e.target === canvas;
    if (aimKeys.includes(k) && inCanvas) { keys.add(k); pointer = null; e.preventDefault(); }
    else if (k === '1' || k === '2' || k === '3') { if (game) G.setSf(game, G.SF_ORDER[Number(k) - 1]); hud(); }
    else if (k === ' ' && inCanvas) { if (game) G.setTransmit(game, !game.transmit); hud(); e.preventDefault(); }
    else if (k === 'p' || (k === 'Escape' && running)) setPaused(!paused);
    else if (k === 'Enter' && inCanvas && !running && mode === 'missions') { e.preventDefault(); start(); }
  });
  root.addEventListener('keyup', e => keys.delete(e.key.length === 1 ? e.key.toLowerCase() : e.key));
  canvas.addEventListener('blur', () => keys.clear());

  // ---- The big arrow buttons: hold one to keep turning (they work like the arrow keys). ----
  for (const b of root.querySelectorAll<HTMLButtonElement>('[data-aim]')) {
    const key = b.dataset.aim!;
    const down = (e: PointerEvent) => {
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
    b.addEventListener('contextmenu', e => e.preventDefault());
  }

  // ---- Pointer: drag on touch; on desktop the antenna follows the mouse over the sky. ----
  const at = (e: PointerEvent) => { const r = canvas.getBoundingClientRect(); return fromXY(e.clientX - r.left, e.clientY - r.top); };
  canvas.addEventListener('pointerdown', e => { pointerDown = true; pointer = at(e); canvas.setPointerCapture(e.pointerId); if (!running && mode === 'missions' && overlay.hidden) start(); });
  canvas.addEventListener('pointermove', e => { if (pointerDown || e.pointerType === 'mouse') pointer = at(e); });
  canvas.addEventListener('pointerup', e => { pointerDown = false; if (e.pointerType !== 'mouse') pointer = null; });
  canvas.addEventListener('pointerleave', () => { if (!pointerDown) pointer = null; });

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
    overlay.hidden = true;
    liveSlot = -1;
    running = true; paused = false;
    skyStatus.textContent = 'Loading the live sky…';
    // The MoQ client loads only when someone opens the live sky.
    const { LiveSky } = await import('./net').catch(() => ({ LiveSky: null }));
    if (mode !== 'live') return;
    if (!LiveSky) { skyStatus.textContent = 'The live sky could not load. Flying with demo stations instead.'; root.dataset.sky = 'offline'; addDemoStations(); return; }
    sky ??= new LiveSky({
      status: (s: SkyStatus, detail?: string) => {
        const text: Record<SkyStatus, string> = {
          off: 'Not connected.',
          connecting: 'Connecting to the MoQ relay…',
          live: `Live over MoQ as ${sky?.callsign}. Other stations appear as dashed circles.`,
          offline: `${detail ?? 'The live sky is offline.'} Flying with demo stations on this computer instead.`,
        };
        skyStatus.textContent = text[s];
        root.dataset.sky = s;
        if (s === 'offline') addDemoStations();
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
      s.snr = G.snrAt(game, s.az, s.el);
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

  function liveTick() {
    const { slot, elapsed, seed } = G.liveSlot(Date.now());
    if (slot !== liveSlot) {
      liveSlot = slot;
      game = G.createGame(G.LIVE, seed);
      if (elapsed > 1) { game.t = Math.min(elapsed, G.LIVE.duration); }
      pops.length = 0;
    }
    return elapsed;
  }

  // ---- Main loop ----
  function frame(now: number) {
    const dt = Math.min(0.1, (now - last) / 1000);
    last = now;
    if (mode === 'live' && running) {
      const elapsed = liveTick();
      if (game && !game.done && elapsed <= G.LIVE.duration) update(dt);
      else if (game) game.events.length = 0;
      moveDemoStations(dt);
      sky?.sendStation({ az: round(game!.ant.az), el: round(game!.ant.el), sf: game!.sf, snr: round(game!.snr), locked: game!.locked, rx: game!.rx, score: game!.score, slot: liveSlot });
    } else if (running && !paused && game) {
      if (countdown > 0) { countdown -= dt; if (countdown <= 0) announce('Acquisition of signal. Track the satellite.'); }
      else update(dt);
    }
    for (const [id, v] of stations) if (!v.demo && now - v.seen > 6000) stations.delete(id);
    for (const p of pops) p.age += dt;
    while (pops.length && pops[0].age > 1.2) pops.shift();
    draw();
    requestAnimationFrame(frame);
  }
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
    if (game.locked !== lastLock) { lastLock = game.locked; announce(game.locked ? `Locked on, SF${game.sf}.` : 'Signal lost. Re-aim the antenna or try a higher SF.', false); }
    hud();
  }

  function finish() {
    if (!game) return;
    const s = G.stars(game);
    beep(s ? 880 : 220, 300, 'triangle');
    if (mode === 'live') { announce(`Pass over. You caught ${game.rx} packets. The next live pass starts at the top of the minute.`); return; }
    running = false;
    $('[data-pause]').textContent = 'Start';
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
    show(`<h3>${s ? 'Pass complete!' : 'Loss of signal'}</h3>
      <p class="stars" aria-label="${s} of 3 stars">${'★'.repeat(s)}${'☆'.repeat(3 - s)}</p>
      <p><strong>${game.rx}</strong> packets · <strong>${game.lost}</strong> lost · best streak <strong>${game.bestStreak}</strong> · <strong>${game.score}</strong> credits${game.battery >= 20 ? ' (with +200 credits for a healthy battery)' : ''}</p>
      <p>${lessons[levelIndex]}</p>
      <div class="row">${s && more ? '<button type="button" class="go" data-next>Next level</button>' : ''}<button type="button" class="${s && more ? '' : 'go'}" data-again>Try again</button></div>`);
    overlay.querySelector<HTMLButtonElement>('button')?.focus();
    announce(`Pass complete. ${game.rx} packets, ${s} of 3 stars.`);
  }

  // ---- HUD (HTML, so it is readable and zoomable) ----
  const hudEl = {
    snr: $('[data-snr]'), bar: $<HTMLElement>('[data-snr-bar]'), floor: $<HTMLElement>('[data-snr-floor]'),
    battery: $('[data-battery]'), batteryBar: $<HTMLElement>('[data-battery-bar]'),
    rx: $('[data-rx]'), score: $('[data-score]'), streak: $('[data-streak]'), time: $('[data-time]'),
    tx: $('[data-tx]'), light: $('[data-light]'), log: $('[data-log]'), board: $('[data-board]'),
  };
  const pct = (snr: number) => Math.max(0, Math.min(100, (snr + 30) / 40 * 100));
  let lastBoard = 0;
  function hud() {
    if (!game) return;
    const left = Math.max(0, game.level.duration - game.t);
    hudEl.snr.textContent = (game.snr < -60 ? '—' : game.snr.toFixed(1) + ' dB') + (game.locked ? ' · locked' : '');
    hudEl.bar.style.width = pct(game.snr) + '%';
    hudEl.bar.dataset.ok = String(game.locked);
    hudEl.floor.style.left = pct(G.SF[game.sf as 7 | 9 | 12].minSnr) + '%';
    hudEl.battery.textContent = Math.round(game.battery) + '%' + (game.safe ? ' · SAFE MODE' : '');
    hudEl.batteryBar.style.width = game.battery + '%';
    hudEl.batteryBar.dataset.low = String(game.battery < 25);
    hudEl.rx.textContent = String(game.rx);
    hudEl.score.textContent = String(game.score);
    hudEl.streak.textContent = String(game.streak);
    hudEl.time.textContent = left.toFixed(0) + ' s';
    hudEl.light.textContent = G.inEclipse(game.level, game.t / game.level.duration) ? 'In Earth\u2019s shadow' : 'Sunlit';
    hudEl.tx.setAttribute('aria-pressed', String(game.transmit));
    hudEl.tx.textContent = game.transmit ? '📡 Sending: on' : '📡 Sending: resting';
    sfButtons.forEach(b => b.setAttribute('aria-pressed', String(Number(b.dataset.sf) === game!.sf)));
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
      ctx.beginPath(); ctx.arc(toX(s.az), toY(s.el), level.beam / 2 * pxPerDeg, 0, Math.PI * 2); ctx.stroke();
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
    const br = level.beam / 2 * pxPerDeg;
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
    // Floating points
    ctx.font = '800 16px Inter, sans-serif'; ctx.lineWidth = 4; ctx.strokeStyle = 'rgba(6,10,32,.85)'; ctx.lineJoin = 'round';
    for (const pop of pops) { ctx.globalAlpha = Math.max(0, 1 - pop.age / 1.2); ctx.fillStyle = '#7ff0a8'; const py = pop.y - pop.age * 28; ctx.strokeText(pop.text, pop.x - 14, py); ctx.fillText(pop.text, pop.x - 14, py); }
    ctx.globalAlpha = 1;
    // Countdown and live waiting
    ctx.font = '900 64px Inter, sans-serif'; ctx.fillStyle = '#fff'; ctx.textAlign = 'center';
    if (countdown > 0 && running) { const n = Math.ceil(countdown), f = countdown % 1; ctx.globalAlpha = 0.4 + 0.6 * f; ctx.lineWidth = 6; ctx.strokeStyle = 'rgba(6,10,32,.7)'; ctx.strokeText(String(n), w / 2, h / 2); ctx.fillText(String(n), w / 2, h / 2); ctx.globalAlpha = 1; }
    ctx.textAlign = 'left';
    if (mode === 'live') {
      const { elapsed } = G.liveSlot(Date.now());
      if (elapsed > G.LIVE.duration) {
        ctx.font = 'bold 22px Inter, sans-serif';
        ctx.fillText(`Next live pass in ${Math.ceil(G.LIVE_SLOT_SECONDS - elapsed)} s`, w / 2 - 120, h / 2);
      }
    }
    canvas.setAttribute('aria-label', `Sky view. Satellite at elevation ${Math.round(sat.el)} degrees. Your antenna at elevation ${Math.round(game.ant.el)}. Signal ${game.locked ? 'locked' : 'not locked'}.`);
  }

  setTab();
  ready();
  resize();
  requestAnimationFrame(frame);
}
