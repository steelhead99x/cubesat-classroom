// Rockoon Battle: everything drawn on the canvas. The world is 640 x 400 with y up (logic.mjs);
// the view scales it to fill the canvas at the screen's real pixel density, bottom-aligned,
// with sky above and the hills carried on past the edges.
import * as G from './logic.mjs';

type Unit = { x: number; y: number; angle: number; power: number; weapon: string };
export type Scene = {
  pads: Unit[];
  target: { x: number; y: number }; terrain: number[]; units: Unit[]; wind: number; colors: string[]; names: string[];
  released?: number;
  active: number; // whose turn it is (-1 for none)
  preview: number[][] | null; // aim dots
  ghost?: number[][] | null; // this player's last shot, faint, to correct from
  drag: { x: number; y: number } | null; // drag-to-aim point, world coordinates
  shot: { path: number[][]; pos: number } | null;
  settling?: boolean; // terrain is moving this frame: paint it directly instead of re-caching
};
type Particle = { x: number; y: number; vx: number; vy: number; life: number; max: number; size: number; color: string; kind: 'spark' | 'debris' | 'smoke' };
type Float = { x: number; y: number; text: string; color: string; t: number };
type Boom = { x: number; y: number; r: number; dirt: boolean; t: number };

const SOIL = ['#8a5a32', '#6b4426', '#4a2f1c', '#2e1d12'];

export class Renderer {
  readonly canvas: HTMLCanvasElement;
  readonly ctx: CanvasRenderingContext2D;
  reduced: boolean;
  dpr = 1; cw = 0; ch = 0; scale = 1; ox = 0; oy = 0;
  #sky: HTMLCanvasElement = document.createElement('canvas');
  #skySeed = -1;
  #ground: HTMLCanvasElement = document.createElement('canvas');
  #groundKey = '';
  #particles: Particle[] = [];
  #floats: Float[] = [];
  #booms: Boom[] = [];
  #shake = 0;
  #flash = 0;
  #clouds = Array.from({ length: 6 }, (_, i) => ({ x: (i * 173) % 640, y: 250 + ((i * 53) % 120), s: 0.7 + ((i * 37) % 10) / 15 }));
  #time = 0;
  #attract: number[] | null = null;

  constructor(canvas: HTMLCanvasElement, reduced: boolean) {
    this.canvas = canvas;
    this.ctx = canvas.getContext('2d')!;
    this.reduced = reduced;
  }

  /** Matches the canvas to its box on screen, at the screen's pixel density. */
  resize() {
    const r = this.canvas.getBoundingClientRect();
    const dpr = Math.min(3, window.devicePixelRatio || 1);
    const w = Math.max(1, Math.round(r.width * dpr)), h = Math.max(1, Math.round(r.height * dpr));
    if (w === this.canvas.width && h === this.canvas.height && dpr === this.dpr) return;
    this.canvas.width = w; this.canvas.height = h;
    this.dpr = dpr; this.cw = r.width; this.ch = r.height;
    // Fit the whole world, keep it on the ground, and leave sky above: all of its height plus
    // a strip for the wind sign, so the sky ring is never cut off at the top.
    this.scale = Math.min(this.cw / G.W, this.ch / (G.H + 40));
    this.ox = (this.cw - G.W * this.scale) / 2;
    this.oy = this.ch - G.H * this.scale;
    this.#skySeed = -1; this.#groundKey = '';
  }

  /** Sprites never get smaller than this many screen pixels per world unit. */
  get k() { return Math.max(1, 0.8 / this.scale); }

  toWorld(clientX: number, clientY: number) {
    const r = this.canvas.getBoundingClientRect();
    return { x: (clientX - r.left - this.ox) / this.scale, y: G.H - (clientY - r.top - this.oy) / this.scale };
  }

  #world(shx = 0, shy = 0) {
    const { dpr, scale } = this;
    this.ctx.setTransform(dpr * scale, 0, 0, -dpr * scale, dpr * (this.ox + shx), dpr * (this.oy + shy + G.H * scale));
  }
  #screen() { this.ctx.setTransform(this.dpr, 0, 0, this.dpr, 0, 0); }
  screenX(x: number) { return this.ox + x * this.scale; }
  screenY(y: number) { return this.oy + (G.H - y) * this.scale; }

  // ---- effects ----
  boom(x: number, y: number, r: number, dirt: boolean) {
    this.#booms.push({ x, y, r, dirt, t: 0 });
    if (this.reduced) return;
    this.#shake = Math.min(14, 3 + r / 5);
    this.#flash = dirt || this.reduced ? 0 : Math.min(0.35, r / 160); // no flash with reduced motion
    const n = Math.round(18 + r * 0.8);
    for (let i = 0; i < n; i++) {
      const a = Math.random() * Math.PI, sp = 60 + Math.random() * (120 + r * 3);
      this.#particles.push({ x, y, vx: Math.cos(a) * sp * (Math.random() < 0.5 ? -1 : 1), vy: Math.sin(a) * sp, life: 0, max: 0.8 + Math.random() * 0.8, size: 1.5 + Math.random() * 2.5, color: SOIL[i % 3], kind: 'debris' });
    }
    if (!dirt) for (let i = 0; i < 16; i++) {
      const a = Math.random() * Math.PI * 2, sp = 80 + Math.random() * 160;
      this.#particles.push({ x, y, vx: Math.cos(a) * sp, vy: Math.sin(a) * sp, life: 0, max: 0.35 + Math.random() * 0.3, size: 1.5, color: '#ffe27a', kind: 'spark' });
    }
    for (let i = 0; i < 8; i++) {
      this.#particles.push({ x: x + (Math.random() - 0.5) * r, y: y + Math.random() * r * 0.5, vx: (Math.random() - 0.5) * 20, vy: 15 + Math.random() * 25, life: 0, max: 1.4 + Math.random(), size: r * 0.35 + Math.random() * 6, color: dirt ? '#7a5636' : '#5d6680', kind: 'smoke' });
    }
  }
  /** A burst for a rockoon that is knocked out. */
  pop(x: number, y: number, color: string) {
    if (this.reduced) return;
    for (let i = 0; i < 22; i++) {
      const a = Math.random() * Math.PI * 2, sp = 60 + Math.random() * 120;
      this.#particles.push({ x, y, vx: Math.cos(a) * sp, vy: Math.sin(a) * sp + 40, life: 0, max: 0.9 + Math.random() * 0.5, size: 2 + Math.random() * 2, color, kind: 'debris' });
    }
  }
  float(x: number, y: number, text: string, color = '#ff6b6b') { this.#floats.push({ x, y, text, color, t: 0 }); }
  trailPuff(x: number, y: number) {
    if (this.reduced || Math.random() < 0.5) return;
    this.#particles.push({ x, y, vx: (Math.random() - 0.5) * 8, vy: 6, life: 0, max: 0.7, size: 2.5 + Math.random() * 2, color: '#c9d2ff', kind: 'smoke' });
  }
  get busy() { return this.#booms.length > 0 || this.#particles.length > 0 || this.#floats.length > 0; }

  // ---- drawing ----
  draw(scene: Scene | null, dt: number, seed: number) {
    const { ctx } = this;
    this.#time += dt;
    if (!this.cw) return;
    this.#tick(dt, scene);
    const shx = this.#shake ? (Math.random() - 0.5) * this.#shake : 0, shy = this.#shake ? (Math.random() - 0.5) * this.#shake : 0;
    this.#screen();
    ctx.drawImage(this.#skyFor(seed), 0, 0, this.cw, this.ch);
    this.#drawClouds(scene?.wind ?? 0, dt);
    const terrain = scene?.terrain ?? (this.#attract ??= G.makeTerrain(11));
    this.#screen();
    if (scene?.settling) { ctx.save(); ctx.translate(shx, shy); this.#paintGround(ctx, terrain, false); ctx.restore(); }
    else ctx.drawImage(this.#groundFor(terrain), shx, shy, this.cw, this.ch);
    if (!scene) return;
    this.#world(shx, shy);
    ctx.save();
    ctx.strokeStyle = '#ffe27a'; ctx.lineWidth = 3;
    ctx.beginPath(); ctx.arc(scene.target.x, scene.target.y, G.TARGET_RADIUS, 0, Math.PI * 2); ctx.stroke();
    ctx.fillStyle = '#fff'; ctx.beginPath(); ctx.arc(scene.target.x, scene.target.y, 2, 0, Math.PI * 2); ctx.fill();
    ctx.restore();
    if (scene.ghost) this.#drawGhost(scene.ghost);
    if (scene.preview) {
      if (scene.active >= 0) this.#drawAscent(scene.pads[scene.active], scene.wind, scene.colors[scene.active]);
      this.#drawPreview(scene.preview);
    }
    scene.pads.forEach(({ x, y }, i) => {
      ctx.fillStyle = '#141b42';
      ctx.fillRect(x - 10, y - 3, 20, 5);
      ctx.fillStyle = scene.colors[i];
      ctx.fillRect(x - 10, y + 1, 20, 2);
    });
    scene.units.forEach((u, i) => { this.#drawRockoon(u, scene.colors[i], i === scene.active, i !== scene.released); });
    if (scene.drag && scene.active >= 0) this.#drawDrag(G.releasePoint(scene.pads[scene.active], scene.wind), scene.drag, scene.colors[scene.active]);
    if (scene.shot) this.#drawShot(scene.shot);
    this.#drawBooms();
    this.#drawParticles();
    this.#screen();
    scene.units.forEach((u, i) => { this.#drawTag(u, scene.names[i], scene.colors[i], i === scene.active); });
    this.#drawFloats();
    this.#drawWind(scene.wind);
    if (this.#flash > 0) { ctx.fillStyle = `rgba(255,240,200,${this.#flash})`; ctx.fillRect(0, 0, this.cw, this.ch); }
  }

  #tick(dt: number, scene: Scene | null) {
    this.#shake = this.#shake > 0.3 ? this.#shake * Math.pow(0.0015, dt) : 0;
    this.#flash = Math.max(0, this.#flash - dt * 2.2);
    for (const b of this.#booms) b.t += dt;
    this.#booms = this.#booms.filter(b => b.t < 0.7);
    for (const f of this.#floats) f.t += dt;
    this.#floats = this.#floats.filter(f => f.t < 1.4);
    const terrain = scene?.terrain;
    for (const p of this.#particles) {
      p.life += dt;
      if (p.kind === 'smoke') { p.x += p.vx * dt; p.y += p.vy * dt; p.size += dt * 6; continue; }
      p.vy -= (p.kind === 'spark' ? 120 : 320) * dt;
      p.x += p.vx * dt; p.y += p.vy * dt;
      if (terrain && p.kind === 'debris') {
        const col = Math.max(0, Math.min(G.W - 1, Math.floor(p.x)));
        if (p.y <= terrain[col]) { p.y = terrain[col]; p.vx *= 0.3; p.vy = 0; p.life = Math.max(p.life, p.max - 0.25); }
      }
    }
    this.#particles = this.#particles.filter(p => p.life < p.max);
  }

  #skyFor(seed: number) {
    if (this.#skySeed === seed && this.#sky.width === this.canvas.width) return this.#sky;
    this.#skySeed = seed;
    const c = this.#sky; c.width = this.canvas.width; c.height = this.canvas.height;
    const x = c.getContext('2d')!;
    x.setTransform(this.dpr, 0, 0, this.dpr, 0, 0);
    const w = this.cw, h = this.ch;
    const g = x.createLinearGradient(0, 0, 0, h);
    g.addColorStop(0, '#0b1240'); g.addColorStop(0.45, '#26307a'); g.addColorStop(0.75, '#6a4f9e'); g.addColorStop(1, '#e98a6b');
    x.fillStyle = g; x.fillRect(0, 0, w, h);
    const r = G.rng(seed * 31 + 7);
    for (let i = 0; i < 140; i++) {
      const sx = r() * w, sy = r() * h * 0.6, a = (1 - sy / (h * 0.6)) * (0.4 + r() * 0.6);
      x.fillStyle = `rgba(255,255,255,${a.toFixed(2)})`;
      const s = r() < 0.1 ? 2 : 1; x.fillRect(sx, sy, s, s);
    }
    // A low sun glow
    const sun = x.createRadialGradient(w * 0.78, this.screenY(140), 0, w * 0.78, this.screenY(140), Math.max(w, h) * 0.45);
    sun.addColorStop(0, 'rgba(255,214,150,.55)'); sun.addColorStop(0.25, 'rgba(255,170,120,.18)'); sun.addColorStop(1, 'rgba(255,170,120,0)');
    x.fillStyle = sun; x.fillRect(0, 0, w, h);
    // Two ranges of far-off mountains
    for (const [layer, color, base, amp] of [[0, '#3b3576', 170, 90], [1, '#2c2a5e', 120, 70]] as const) {
      const pr = G.rng(seed * 13 + layer * 101);
      x.fillStyle = color; x.beginPath(); x.moveTo(0, h);
      let y = base + pr() * amp;
      for (let px = -20; px <= w + 20; px += 18) {
        y += (pr() - 0.5) * 26; y = Math.max(base - 10, Math.min(base + amp, y));
        x.lineTo(px, this.screenY(y));
      }
      x.lineTo(w, h); x.closePath(); x.fill();
    }
    return c;
  }

  #groundFor(terrain: number[]) {
    const key = `${this.canvas.width}x${this.canvas.height}:${terrain.join(',')}`;
    if (key === this.#groundKey) return this.#ground;
    this.#groundKey = key;
    const c = this.#ground; c.width = this.canvas.width; c.height = this.canvas.height;
    const x = c.getContext('2d')!;
    x.setTransform(this.dpr, 0, 0, this.dpr, 0, 0);
    this.#paintGround(x, terrain, true);
    return c;
  }

  #paintGround(x: CanvasRenderingContext2D, terrain: number[], pebbles: boolean) {
    const sx = (wx: number) => this.screenX(wx), sy = (wy: number) => this.screenY(wy);
    const path = new Path2D();
    path.moveTo(0, this.ch); path.lineTo(0, sy(terrain[0]));
    for (let i = 0; i < G.W; i++) path.lineTo(sx(i + 0.5), sy(terrain[i]));
    path.lineTo(this.cw, sy(terrain[G.W - 1])); path.lineTo(this.cw, this.ch); path.closePath();
    const g = x.createLinearGradient(0, sy(300), 0, this.ch);
    g.addColorStop(0, SOIL[0]); g.addColorStop(0.35, SOIL[1]); g.addColorStop(0.75, SOIL[2]); g.addColorStop(1, SOIL[3]);
    x.fillStyle = g; x.fill(path);
    // Pebbles and strata, clipped to the ground (skipped while the ground is settling)
    if (pebbles) {
    x.save(); x.clip(path);
    const r = G.rng(99);
    for (let i = 0; i < 260; i++) {
      x.fillStyle = r() < 0.5 ? 'rgba(0,0,0,.18)' : 'rgba(255,220,170,.10)';
      x.beginPath(); x.ellipse(r() * this.cw, sy(r() * 300), 1 + r() * 2.5, 0.8 + r() * 1.5, 0, 0, Math.PI * 2); x.fill();
    }
    x.restore();
    }
    // Grass on top: a bright edge over a darker band
    const top = new Path2D();
    top.moveTo(0, sy(terrain[0]));
    for (let i = 0; i < G.W; i++) top.lineTo(sx(i + 0.5), sy(terrain[i]));
    top.lineTo(this.cw, sy(terrain[G.W - 1]));
    x.lineJoin = 'round';
    x.strokeStyle = '#2f7d4a'; x.lineWidth = Math.max(5, 7 * this.scale); x.stroke(top);
    x.strokeStyle = '#6fdc8c'; x.lineWidth = Math.max(2, 2.5 * this.scale); x.stroke(top);
  }

  #drawClouds(wind: number, dt: number) {
    const { ctx } = this;
    for (const c of this.#clouds) {
      if (!this.reduced) c.x += wind * 3 * dt * c.s;
      const span = G.W + 160;
      c.x = ((c.x + 80) % span + span) % span - 80;
      const x = this.screenX(c.x), y = this.screenY(c.y), s = 26 * c.s * Math.max(0.7, this.scale);
      ctx.fillStyle = 'rgba(220,226,255,.13)';
      ctx.beginPath();
      ctx.ellipse(x, y, s * 1.6, s * 0.55, 0, 0, Math.PI * 2);
      ctx.ellipse(x + s * 0.8, y - s * 0.3, s * 0.9, s * 0.5, 0, 0, Math.PI * 2);
      ctx.ellipse(x - s * 0.7, y - s * 0.2, s * 0.8, s * 0.45, 0, 0, Math.PI * 2);
      ctx.fill();
    }
  }

  #drawRockoon(u: Unit, color: string, active: boolean, carryingRocket: boolean) {
    const { ctx } = this, k = this.k;
    ctx.save();
    ctx.translate(u.x, u.y); ctx.scale(k, k);
    if (active) {
      const pulse = this.reduced ? 0.5 : 0.5 + 0.5 * Math.sin(this.#time * 5);
      ctx.strokeStyle = `rgba(255,210,63,${0.35 + pulse * 0.4})`; ctx.lineWidth = 2;
      ctx.beginPath(); ctx.ellipse(0, 1, 16 + pulse * 3, 4 + pulse, 0, 0, Math.PI * 2); ctx.stroke();
    }
    // tethers
    ctx.strokeStyle = 'rgba(230,236,255,.75)'; ctx.lineWidth = 0.8;
    ctx.beginPath(); ctx.moveTo(-7, 9); ctx.lineTo(-3, 40); ctx.moveTo(7, 9); ctx.lineTo(3, 40); ctx.stroke();
    // balloon
    const bob = this.reduced ? 0 : Math.sin(this.#time * 1.6 + u.x) * 1.2;
    const bg = ctx.createRadialGradient(-4, 56 + bob, 2, 0, 52 + bob, 15);
    bg.addColorStop(0, '#ffffff'); bg.addColorStop(0.55, tint(color, 0.55)); bg.addColorStop(1, color);
    ctx.fillStyle = bg;
    ctx.beginPath(); ctx.ellipse(0, 52 + bob, 11, 13, 0, 0, Math.PI * 2); ctx.fill();
    ctx.strokeStyle = 'rgba(0,0,0,.35)'; ctx.lineWidth = 0.8; ctx.stroke();
    ctx.fillStyle = shade(color, 0.7);
    ctx.beginPath(); ctx.moveTo(-2.5, 39.5 + bob); ctx.lineTo(2.5, 39.5 + bob); ctx.lineTo(0, 42 + bob); ctx.fill();
    // gondola
    const gg = ctx.createLinearGradient(0, 10, 0, 0);
    gg.addColorStop(0, tint(color, 0.3)); gg.addColorStop(1, shade(color, 0.65));
    ctx.fillStyle = gg; roundRect(ctx, -9, 0, 18, 10, 2.5); ctx.fill();
    ctx.strokeStyle = 'rgba(0,0,0,.45)'; ctx.lineWidth = 0.8; ctx.stroke();
    ctx.fillStyle = 'rgba(10,16,40,.85)'; roundRect(ctx, -5, 3.5, 10, 3.5, 1.2); ctx.fill();
    // launcher: a mast up to the pivot, then the tube
    ctx.restore();
    ctx.save();
    ctx.translate(u.x, u.y + G.PIVOT_Y);
    ctx.strokeStyle = '#c9d2ff'; ctx.lineWidth = 1.6;
    ctx.beginPath(); ctx.moveTo(0, 0); ctx.lineTo(0, -(G.PIVOT_Y - 10)); ctx.stroke();
    ctx.rotate(u.angle * Math.PI / 180);
    ctx.fillStyle = shade(color, 0.5); roundRect(ctx, -2, -2.6, 16, 5.2, 1.6); ctx.fill();
    if (carryingRocket) {
      ctx.fillStyle = '#fff2ca'; ctx.fillRect(5, -2, 12, 4);
      ctx.fillStyle = color;
      ctx.beginPath(); ctx.moveTo(17, -3); ctx.lineTo(23, 0); ctx.lineTo(17, 3); ctx.fill();
    }
    ctx.restore();
    ctx.fillStyle = '#e8ecff'; ctx.beginPath(); ctx.arc(u.x, u.y + G.PIVOT_Y, 2.2, 0, Math.PI * 2); ctx.fill();
  }

  #drawTag(u: Unit, name: string, color: string, active: boolean) {
    const { ctx } = this, k = this.k;
    const x = this.screenX(u.x), y = this.screenY(u.y + 70 * k) ;
    ctx.font = `700 ${active ? 13 : 12}px system-ui, sans-serif`; ctx.textAlign = 'center'; ctx.textBaseline = 'bottom';
    ctx.lineWidth = 3; ctx.strokeStyle = 'rgba(8,12,36,.85)'; ctx.strokeText(name, x, y - 6);
    ctx.fillStyle = active ? '#ffe27a' : tint(color, 0.35); ctx.fillText(name, x, y - 6);
    ctx.textBaseline = 'alphabetic';
  }

  /** The player's previous shot as a faint dashed line, so they can see how far the wind took it. */
  #drawGhost(points: number[][]) {
    if (points.length < 2) return;
    const { ctx } = this;
    ctx.save();
    ctx.strokeStyle = 'rgba(255,255,255,.28)';
    ctx.lineWidth = 2;
    ctx.setLineDash([3, 5]);
    ctx.beginPath();
    ctx.moveTo(points[0][0], points[0][1]);
    for (let i = 1; i < points.length; i++) ctx.lineTo(points[i][0], points[i][1]);
    ctx.stroke();
    ctx.restore();
  }

  #drawPreview(points: number[][]) {
    const { ctx } = this;
    points.forEach(([x, y], i) => {
      if (i % 2) return;
      const a = 1 - i / points.length;
      ctx.fillStyle = `rgba(255,226,122,${(0.25 + a * 0.75).toFixed(2)})`;
      ctx.beginPath(); ctx.arc(x, y, 1.4 + a * 1.4, 0, Math.PI * 2); ctx.fill();
    });
  }

  /** The climb the balloon will make before it lets go: pad to release point, dashed. */
  #drawAscent(u: Unit, wind: number, color: string) {
    const { ctx } = this, r = G.releasePoint(u, wind);
    ctx.save();
    ctx.setLineDash([2, 4]); ctx.strokeStyle = tint(color, 0.5); ctx.lineWidth = 1.2;
    ctx.beginPath(); ctx.moveTo(u.x, u.y + G.PIVOT_Y); ctx.lineTo(r.x, r.y + G.PIVOT_Y); ctx.stroke();
    ctx.setLineDash([]);
    ctx.beginPath(); ctx.arc(r.x, r.y + G.PIVOT_Y, 3, 0, Math.PI * 2); ctx.stroke();
    ctx.restore();
  }

  /** The drag line, from where the balloon will release the rocket (`u`) to the pointer. */
  #drawDrag(u: { x: number; y: number }, p: { x: number; y: number }, color: string) {
    const { ctx } = this;
    ctx.setLineDash([4, 4]); ctx.strokeStyle = tint(color, 0.4); ctx.lineWidth = 1.2;
    ctx.beginPath(); ctx.moveTo(u.x, u.y + G.PIVOT_Y); ctx.lineTo(p.x, p.y); ctx.stroke();
    ctx.setLineDash([]);
    ctx.strokeStyle = 'rgba(255,255,255,.25)';
    ctx.beginPath(); ctx.arc(u.x, u.y + G.PIVOT_Y, G.DRAG_FULL, 0, Math.PI); ctx.stroke();
    ctx.fillStyle = color; ctx.beginPath(); ctx.arc(p.x, p.y, 4, 0, Math.PI * 2); ctx.fill();
  }

  #drawShot(shot: { path: number[][]; pos: number }) {
    const { ctx } = this, i = Math.min(shot.path.length - 1, Math.floor(shot.pos));
    const from = Math.max(0, i - 36);
    ctx.lineCap = 'round';
    for (let j = from + 1; j <= i; j++) {
      const a = (j - from) / (i - from + 1);
      ctx.strokeStyle = `rgba(255,${180 + Math.round(a * 60)},120,${(a * 0.8).toFixed(2)})`;
      ctx.lineWidth = 0.8 + a * 2.4;
      ctx.beginPath(); ctx.moveTo(shot.path[j - 1][0], shot.path[j - 1][1]); ctx.lineTo(shot.path[j][0], shot.path[j][1]); ctx.stroke();
    }
    const [x, y] = shot.path[i];
    const glow = ctx.createRadialGradient(x, y, 0, x, y, 9);
    glow.addColorStop(0, 'rgba(255,255,230,1)'); glow.addColorStop(0.3, 'rgba(255,210,90,.8)'); glow.addColorStop(1, 'rgba(255,150,60,0)');
    ctx.fillStyle = glow; ctx.beginPath(); ctx.arc(x, y, 9, 0, Math.PI * 2); ctx.fill();
    ctx.lineCap = 'butt';
  }

  #drawBooms() {
    const { ctx } = this;
    for (const b of this.#booms) {
      const t = b.t;
      if (b.dirt) {
        const r = b.r * Math.min(1, t / 0.2);
        ctx.fillStyle = `rgba(122,86,54,${(0.85 * (1 - t / 0.7)).toFixed(2)})`;
        ctx.beginPath(); ctx.arc(b.x, b.y, r, 0, Math.PI * 2); ctx.fill();
        continue;
      }
      const grow = Math.min(1, t / 0.16), fade = Math.max(0, 1 - Math.max(0, t - 0.16) / 0.5);
      const r = b.r * (0.4 + 0.75 * grow);
      const g = ctx.createRadialGradient(b.x, b.y, 0, b.x, b.y, r);
      g.addColorStop(0, `rgba(255,255,235,${fade})`); g.addColorStop(0.35, `rgba(255,214,90,${fade})`);
      g.addColorStop(0.7, `rgba(255,120,50,${0.8 * fade})`); g.addColorStop(1, 'rgba(200,60,40,0)');
      ctx.fillStyle = g; ctx.beginPath(); ctx.arc(b.x, b.y, r, 0, Math.PI * 2); ctx.fill();
      if (!this.reduced) {
        ctx.strokeStyle = `rgba(255,240,210,${(0.5 * (1 - t / 0.7)).toFixed(2)})`; ctx.lineWidth = 2;
        ctx.beginPath(); ctx.arc(b.x, b.y, b.r * (0.8 + t * 2.4), 0, Math.PI * 2); ctx.stroke();
      }
    }
  }

  #drawParticles() {
    const { ctx } = this;
    for (const p of this.#particles) {
      const a = 1 - p.life / p.max;
      if (p.kind === 'smoke') { ctx.fillStyle = hexA(p.color, a * 0.35); ctx.beginPath(); ctx.arc(p.x, p.y, p.size, 0, Math.PI * 2); ctx.fill(); }
      else { ctx.fillStyle = hexA(p.color, Math.min(1, a * 1.5)); ctx.fillRect(p.x - p.size / 2, p.y - p.size / 2, p.size, p.size); }
    }
  }

  #drawFloats() {
    const { ctx } = this;
    ctx.textAlign = 'center';
    for (const f of this.#floats) {
      const a = Math.min(1, 2.2 * (1 - f.t / 1.4)), lift = (this.reduced ? 0 : 36) * Math.min(1, f.t / 1.0);
      const pop = this.reduced ? 1 : 1 + 0.4 * Math.max(0, 1 - f.t / 0.15);
      ctx.font = `900 ${Math.round(20 * pop)}px system-ui, sans-serif`;
      ctx.lineWidth = 4; ctx.strokeStyle = `rgba(8,12,36,${a})`;
      const x = this.screenX(f.x), y = this.screenY(f.y) - lift;
      ctx.strokeText(f.text, x, y); ctx.fillStyle = hexA(f.color, a); ctx.fillText(f.text, x, y);
    }
  }

  #drawWind(wind: number) {
    const { ctx } = this;
    const cx = this.cw / 2, y = 22;
    ctx.fillStyle = 'rgba(8,12,36,.55)'; roundRect(ctx, cx - 70, y - 14, 140, 30, 15); ctx.fill();
    ctx.font = '700 13px system-ui, sans-serif'; ctx.textAlign = 'left'; ctx.textBaseline = 'middle'; ctx.fillStyle = '#e8ecff';
    if (wind === 0) { ctx.textAlign = 'center'; ctx.fillText('No wind', cx, y + 1); ctx.textBaseline = 'alphabetic'; return; }
    ctx.fillText('Wind', cx - 60, y + 1);
    const len = 10 + Math.abs(wind) * 4.5, dir = Math.sign(wind), ax = cx + 16;
    ctx.strokeStyle = '#9fe3ff'; ctx.fillStyle = '#9fe3ff'; ctx.lineWidth = 3; ctx.lineCap = 'round';
    ctx.beginPath(); ctx.moveTo(ax - dir * len / 2, y + 1); ctx.lineTo(ax + dir * len / 2, y + 1); ctx.stroke();
    ctx.beginPath(); ctx.moveTo(ax + dir * (len / 2 + 6), y + 1); ctx.lineTo(ax + dir * (len / 2 - 3), y - 5); ctx.lineTo(ax + dir * (len / 2 - 3), y + 7); ctx.closePath(); ctx.fill();
    ctx.lineCap = 'butt'; ctx.textBaseline = 'alphabetic';
  }
}

// ---- colour helpers ----
function rgb(hex: string) { const n = parseInt(hex.slice(1), 16); return [(n >> 16) & 255, (n >> 8) & 255, n & 255]; }
function tint(hex: string, t: number) { const [r, g, b] = rgb(hex); return `rgb(${Math.round(r + (255 - r) * t)},${Math.round(g + (255 - g) * t)},${Math.round(b + (255 - b) * t)})`; }
function shade(hex: string, t: number) { const [r, g, b] = rgb(hex); return `rgb(${Math.round(r * t)},${Math.round(g * t)},${Math.round(b * t)})`; }
function hexA(hex: string, a: number) { const [r, g, b] = rgb(hex); return `rgba(${r},${g},${b},${Math.max(0, a).toFixed(2)})`; }
function roundRect(ctx: CanvasRenderingContext2D, x: number, y: number, w: number, h: number, r: number) {
  ctx.beginPath(); ctx.roundRect(x, y, Math.max(0, w), h, Math.min(r, Math.abs(w) / 2, h / 2));
}

/** Small pictures of each rocket for the shop and the rocket button. */
export function rocketSvg(id: string, size = 40) {
  const look: Record<string, [string, number]> = { baby: ['#ffd23f', 0.62], rocket: ['#ef6b6c', 0.8], dirt: ['#6fd3a0', 0.86], big: ['#ff9f43', 0.92], mega: ['#b9a4ff', 1] };
  const [c, s] = look[id] ?? look.baby;
  return `<svg width="${size}" height="${size}" viewBox="0 0 40 40" aria-hidden="true"><g transform="translate(20 20) rotate(45) scale(${s}) translate(-20 -20)">
    <path d="M20 3c5 4 7 10 7 17v8h-14v-8c0-7 2-13 7-17z" fill="${c}"/><path d="M13 22l-5 7v4l5-3zM27 22l5 7v4l-5-3z" fill="#5b6ab8"/>
    <circle cx="20" cy="15" r="3.2" fill="#0b1240" stroke="#e8ecff" stroke-width="1.2"/><path d="M15 28h10l-2 6h-6z" fill="#ffe27a"/>${id === 'mega' || id === 'big' ? '<path d="M17 34h6l-3 5z" fill="#ff6b3d"/>' : ''}</g></svg>`;
}
