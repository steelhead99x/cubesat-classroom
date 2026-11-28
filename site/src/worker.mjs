import { classroomHost, hubs, redirects } from './hubs.mjs';
import { answerFor, byQuest, questionText } from './guide/answers.mjs';
import { gameToken } from './games-token.mjs';
import { rockoonRoom } from './rockoon-room-api.mjs';

// Hubs with their own `path` live on another host (GitHub Pages or the classroom home), not here.
const homeByHost = new Map(hubs.filter(hub => !hub.path).map(({ host, slug }) => [host, '/hub/' + slug + '/']));

// Redirect domains (see `redirects` in hubs.mjs) keep the path and query, so old links and
// the old `curl -fsSL https://cubesat.club/start.sh | bash` keep working. Checked before the preview
// password, so a redirect domain never shows the password page.
// www on those domains follows the same rule (one hop to the public site). www on the classroom
// host or a hub host goes to the same path on the apex.
function redirected(request) {
  const url = new URL(request.url);
  const host = url.hostname;
  const apex = host.replace(/^www\./, '');
  const target = redirects[host] ?? redirects[apex];
  if (target) {
    return new Response(null, {
      status: 301,
      headers: { location: `${target}${url.pathname}${url.search}`, 'cache-control': 'public, max-age=3600' },
    });
  }
  if (host !== apex && (apex === classroomHost || hubs.some(hub => hub.host === apex))) {
    return new Response(null, {
      status: 301,
      headers: { location: `https://${apex}${url.pathname}${url.search}`, 'cache-control': 'public, max-age=3600' },
    });
  }
  return null;
}

// Classroom guardrails for the live quest guide. Mirrors AGENTS.md.
const SYSTEM = `You are the CubeSat Classroom quest guide, an AI helping students aged 13 and up, and the teachers, guardians and amateur radio mentors who guide them, with a practice satellite ground station.
Rules:
- Only discuss CubeSats, orbits, amateur radio and AMSAT, sensors and units, measurement and data, the practice ground station and the current quest. If asked anything else, say in one sentence that you only help with the mission.
- Teach, do not do the work: give hints and ask questions. Never write a student's crew log or answer their "talk it over" question for them.
- Never ask for or repeat names, ages, schools, locations, photos or contact details.
- Do not discuss violence, self-harm, dating, drugs or other unsafe topics; suggest talking to a trusted adult.
- Transmitting on amateur bands needs a licensed operator. Never help anyone get around radio rules.
- If you are not sure, say so.
- Answer in at most 90 words, in plain text, at the reader's level.`;

const LEVEL_NOTE = {
  beginner: 'The reader is new to satellites and may be 13. Use short sentences and everyday words.',
  intermediate: 'The reader is in middle or high school. Mention units, timing and simple numbers.',
  professional: 'The reader is an advanced student, mentor or engineer. Technical terms such as MoQ groups and tracks are fine.',
};

const json = (body, status = 200) => new Response(JSON.stringify(body), { status, headers: { 'content-type': 'application/json', 'cache-control': 'no-store' } });

async function sha256(text) {
  const bytes = await crypto.subtle.digest('SHA-256', new TextEncoder().encode(text));
  return [...new Uint8Array(bytes)].map(b => b.toString(16).padStart(2, '0')).join('');
}

async function bump(kv, key, limit) {
  const used = Number((await kv.get(key)) ?? 0);
  if (used >= limit) return false;
  await kv.put(key, String(used + 1), { expirationTtl: 60 * 60 * 25 });
  return true;
}

// POST /api/guide { quest, question, level }
// Off unless ANTHROPIC_API_KEY is set. Only fixed question ids are accepted, so
// nothing a visitor types reaches the model. Optional KV binding GUIDE_LIMITS adds
// per-device daily limits (salted, daily-rotating hash) and caches answers.
async function guide(request, env) {
  if (!env.ANTHROPIC_API_KEY) return json({ error: 'The live guide is not switched on yet.' }, 503);
  // Browsers always send Origin on a POST from a page. Accept only this site's own origin,
  // or the list in GUIDE_ALLOWED_ORIGINS, so other sites cannot spend the guide's budget.
  const origin = request.headers.get('origin');
  const allowed = env.GUIDE_ALLOWED_ORIGINS ? env.GUIDE_ALLOWED_ORIGINS.split(',').map(o => o.trim()) : [new URL(request.url).origin];
  if (!origin || !allowed.includes(origin)) return json({ error: 'Forbidden' }, 403);

  // Only fixed ids are accepted. `quest` is checked on its own because general questions
  // ignore it, and it goes into the prompt and the cache key.
  if (Number(request.headers.get('content-length') ?? 0) > 1024) return json({ error: 'Request too large.' }, 413);
  const text = await request.text().catch(() => '');
  if (new TextEncoder().encode(text).length > 1024) return json({ error: 'Request too large.' }, 413);
  let body = null;
  try { body = JSON.parse(text); } catch { body = null; }
  const { quest = '', question = '', level = '' } = body ?? {};
  if (typeof quest !== 'string' || (quest !== '' && !Object.hasOwn(byQuest, quest))) return json({ error: 'Unknown quest.' }, 400);
  const fallback = answerFor(quest, question, level);
  const asked = questionText(quest, question);
  if (!fallback || !asked) return json({ error: 'Unknown question.' }, 400);

  // Without the KV limits and a salt for the hashes, the model is never called: an unlimited
  // guide could run up any bill, and unsalted address hashes could be reversed.
  const kv = env.GUIDE_LIMITS;
  if (!kv || !env.GUIDE_SALT) return json({ answer: fallback, fallback: true });
  const day = new Date().toISOString().slice(0, 10);
  const cacheKey = `answer:${day}:${quest}:${question}:${level}`;
  {
    const cached = await kv.get(cacheKey);
    if (cached) return json({ answer: cached, cached: true });
    // The device id is sent by the browser, so anyone can change it. The network cap is
    // keyed on the connecting address, which a visitor cannot pick, and is set high enough
    // for a whole class behind one school network. Workers KV is not transactional, so
    // requests that arrive together can each slip past a limit: treat all three as soft.
    // KV has no atomic increment and a Durable Object would be new infrastructure, so the
    // default caps are kept low to bound what a burst of parallel requests can overshoot by.
    const client = (request.headers.get('x-guide-client') ?? '').slice(0, 64) || 'anonymous';
    const address = request.headers.get('cf-connecting-ip') ?? 'unknown';
    const device = await sha256(`${env.GUIDE_SALT}:${day}:${client}`);
    const network = await sha256(`${env.GUIDE_SALT}:${day}:net:${address}`);
    const perDevice = Number(env.GUIDE_DAILY_LIMIT ?? 10);
    const perNetwork = Number(env.GUIDE_NETWORK_LIMIT ?? 150);
    const perSite = Number(env.GUIDE_GLOBAL_LIMIT ?? 1000);
    if (!(await bump(kv, `device:${device}`, perDevice))) {
      return json({ message: 'That is all the questions for today. Keep going with the quest steps, and come back tomorrow.' }, 429);
    }
    if (!(await bump(kv, `network:${network}`, perNetwork))) return json({ answer: fallback, limited: 'network' });
    if (!(await bump(kv, `site:${day}`, perSite))) return json({ answer: fallback, limited: 'site' });
  }

  const res = await fetch('https://api.anthropic.com/v1/messages', {
    method: 'POST',
    headers: {
      'x-api-key': env.ANTHROPIC_API_KEY,
      'anthropic-version': env.ANTHROPIC_VERSION ?? '2026-10-21',
      'content-type': 'application/json',
    },
    body: JSON.stringify({
      model: env.GUIDE_MODEL ?? 'claude-sonnet-5-5',
      max_tokens: 300,
      output_config: { effort: env.GUIDE_EFFORT ?? 'low' },
      system: `${SYSTEM}\n${LEVEL_NOTE[level]}`,
      messages: [{
        role: 'user',
        content: `Current quest: ${quest || 'none, general question'}.\nQuestion: ${asked}\nA good answer looks like this example; write your own in the same spirit:\n${fallback}`,
      }],
    }),
  }).catch(error => {
    console.error('Guide: Anthropic API request failed', error);
    return null;
  });

  if (!res || !res.ok) {
    if (res) console.error('Guide: Anthropic API returned', res.status, (await res.text().catch(() => '')).slice(0, 500));
    return json({ answer: fallback, fallback: true });
  }
  const data = await res.json().catch(() => null);
  const answer = data?.content?.find(block => block.type === 'text')?.text?.trim();
  if (!answer) return json({ answer: fallback, fallback: true });
  await kv.put(cacheKey, answer, { expirationTtl: 60 * 60 * 24 }).catch(error => console.error('Guide: cache write failed', error));
  return json({ answer });
}

// Private preview. When the SITE_PASSWORD secret is set, every page, file and API
// route on every domain shows an "under construction" page with the mission statement
// and a password box. The right password sets a cookie for 30 days. The password lives
// only in the Worker secret, never in this public repository. Delete the secret to open
// the site. Scripts can send the password as HTTP Basic auth (any user name).
const GATE_COOKIE = '__Host-preview';
const sameText = (a, b) => {
  let diff = a.length ^ b.length;
  for (let i = 0; i < a.length; i++) diff |= a.charCodeAt(i) ^ b.charCodeAt(i % (b.length || 1));
  return diff === 0;
};
// Hashes are compared, so the check takes the same time whatever was typed.
const passToken = password => sha256('preview-password:' + password);

// The cookie is "<expiry>.<signature>", signed with PREVIEW_COOKIE_SECRET (a long random
// value), so the password cannot be worked out from a copied cookie. Without that secret
// no cookie is issued or accepted: only HTTP Basic auth opens the preview.
async function sign(expiry, env) {
  const key = await crypto.subtle.importKey('raw', new TextEncoder().encode(env.PREVIEW_COOKIE_SECRET), { name: 'HMAC', hash: 'SHA-256' }, false, ['sign']);
  const mac = await crypto.subtle.sign('HMAC', key, new TextEncoder().encode('preview:' + expiry));
  return [...new Uint8Array(mac)].map(b => b.toString(16).padStart(2, '0')).join('');
}

async function validCookie(request, env) {
  if (!env.PREVIEW_COOKIE_SECRET) return false;
  const value = (request.headers.get('cookie') ?? '').split(/;\s*/).find(c => c.startsWith(GATE_COOKIE + '='))?.slice(GATE_COOKIE.length + 1) ?? '';
  const [expiry, mac] = value.split('.');
  if (!/^\d{10,13}$/.test(expiry ?? '') || !mac || Number(expiry) < Date.now()) return false;
  return sameText(mac, await sign(expiry, env));
}

function landing(wrong = false, unconfigured = false, limited = false) {
  const html = `<!doctype html>
<html lang="en">
<head>
<meta charset="utf-8">
<meta name="viewport" content="width=device-width, initial-scale=1">
<meta name="robots" content="noindex, nofollow">
<title>CubeSat Classroom | Under construction</title>
<style>
  :root { color-scheme: light; }
  * { box-sizing: border-box; }
  body { margin: 0; font: 1.05rem/1.6 system-ui, -apple-system, "Segoe UI", sans-serif; color: #172b43; background: #fff; }
  main { width: min(100% - 2rem, 40rem); margin: 0 auto; padding: clamp(2rem, 8vw, 5rem) 0 3rem; }
  .status { display: inline-block; margin: 0 0 1rem; padding: .25rem .7rem; border-radius: 2rem; background: #fff3dc; color: #7a4700; font-size: .88rem; font-weight: 700; }
  h1 { margin: 0 0 1rem; font-size: clamp(1.9rem, 6vw, 2.7rem); line-height: 1.12; letter-spacing: -.01em; }
  h2 { margin: 2.2rem 0 .5rem; font-size: 1.15rem; }
  p { margin: 0 0 1rem; color: #3d5372; }
  ul { margin: 0 0 1rem; padding-left: 1.2rem; color: #3d5372; }
  li { margin-bottom: .35rem; }
  a { color: #214da5; font-weight: 700; }
  form { margin-top: 1rem; padding: 1.2rem; border: 1px solid #d5deeb; border-radius: .7rem; background: #f4f7fb; }
  label { display: block; margin-bottom: .4rem; font-weight: 700; }
  .row { display: flex; flex-wrap: wrap; gap: .6rem; }
  input { flex: 1 1 12rem; min-width: 0; padding: .62rem .75rem; border: 1px solid #8a9bb5; border-radius: .5rem; font: inherit; }
  button { padding: .62rem 1.1rem; border: 0; border-radius: .5rem; background: #214da5; color: #fff; font: inherit; font-weight: 700; cursor: pointer; }
  button:hover { background: #193d86; }
  :focus-visible { outline: 3px solid #214da5; outline-offset: 2px; }
  .error { margin: .7rem 0 0; color: #a7283a; font-weight: 700; }
  footer { margin-top: 2.5rem; padding-top: 1rem; border-top: 1px solid #d5deeb; font-size: .9rem; }
</style>
</head>
<body>
<main>
  <p class="status">Under construction</p>
  <h1>An open starting kit for classroom space projects</h1>
  <p>CubeSat Classroom is a free, open-source set of lessons, a practice ground station and printable hardware for learners 13 and up. Any school, club or mission can copy it and make it their own.</p>
  <h2>Our mission</h2>
  <ul>
    <li><strong>Share, don't compete.</strong> This is a framework to build on, not a rival to other space education projects. We point to their work and hope they reuse ours.</li>
    <li><strong>Make starting easy.</strong> Fork the code, change one settings file, and you have a site for your own club or project.</li>
    <li><strong>Be ready for the next flight.</strong> The same lessons and tools can serve balloons, gliders, research spaceflights and satellites to come.</li>
  </ul>
  <p>The website is private while it is being reviewed. The code is already public.</p>
  <p><a href="https://github.com/steelhead99x/cubesat-classroom">See the project on GitHub</a></p>
  <form method="post" action="/preview-unlock">
    <label for="password">Preview password</label>
    <div class="row">
      <input id="password" name="password" type="password" autocomplete="current-password" required>
      <button type="submit">Open the preview</button>
    </div>
    ${wrong ? '<p class="error" role="alert">That password did not match. Check it and try again.</p>' : ''}
    ${unconfigured ? '<p class="error" role="alert">The password is right, but the preview is not fully set up yet. Ask the site owner to finish the setup.</p>' : ''}
    ${limited ? '<p class="error" role="alert">Too many wrong passwords from this network today. Try again tomorrow.</p>' : ''}
  </form>
  <footer><p>An independent volunteer project.</p></footer>
</main>
</body>
</html>`;
  return new Response(html, {
    status: 401,
    headers: { 'content-type': 'text/html; charset=utf-8', 'cache-control': 'no-store', 'x-robots-tag': 'noindex, nofollow' },
  });
}

// Wrong passwords, from the form and from Basic auth alike, are counted under one key per
// network address and day, when the KV store is bound.
async function unlockKey(request, env) {
  if (!env.GUIDE_LIMITS || !env.GUIDE_SALT) return null;
  const day = new Date().toISOString().slice(0, 10);
  return 'unlock:' + (await sha256(`${env.GUIDE_SALT}:${day}:${request.headers.get('cf-connecting-ip') ?? 'unknown'}`));
}

async function tooManyGuesses(request, env) {
  // The UNLOCK_LIMIT rate-limit binding (wrangler.jsonc) caps password attempts per network
  // address per minute, so the gate is not open to fast guessing even when the KV store
  // below is not bound. It counts every attempt, right or wrong, so it is used only when
  // PREVIEW_COOKIE_SECRET is set: then a right password sets the signed cookie and later
  // requests skip this check. Without the cookie, a browser resends Basic credentials with
  // every file of every page and would soon lock itself out.
  if (env.UNLOCK_LIMIT && env.PREVIEW_COOKIE_SECRET) {
    const { success } = await env.UNLOCK_LIMIT.limit({ key: request.headers.get('cf-connecting-ip') ?? 'unknown' });
    if (!success) return true;
  }
  const key = await unlockKey(request, env);
  if (!key) return false;
  return Number((await env.GUIDE_LIMITS.get(key)) ?? 0) >= Number(env.PREVIEW_UNLOCK_LIMIT ?? 20);
}

async function countWrongGuess(request, env) {
  const key = await unlockKey(request, env);
  if (!key) return;
  const wrong = Number((await env.GUIDE_LIMITS.get(key)) ?? 0);
  await env.GUIDE_LIMITS.put(key, String(wrong + 1), { expirationTtl: 60 * 60 * 25 });
}

/** The signed preview cookie, good for 30 days. */
async function gateCookie(env) {
  const expiry = String(Date.now() + 30 * 24 * 60 * 60 * 1000);
  return `${GATE_COOKIE}=${expiry}.${await sign(expiry, env)}; Path=/; Max-Age=2592000; HttpOnly; Secure; SameSite=Lax`;
}

/**
 * The preview gate. Returns null to let the request through, a Response to answer it
 * instead, or `{ setCookie }` to let it through and sign the visitor in.
 */
async function locked(request, env) {
  if (!env.SITE_PASSWORD) return null;
  const url = new URL(request.url);
  const good = await passToken(env.SITE_PASSWORD);
  const hasCookie = await validCookie(request, env);

  if (url.pathname === '/preview-unlock' && request.method === 'POST') {
    if (await tooManyGuesses(request, env)) return landing(false, false, true);
    const form = await request.formData().catch(() => null);
    const given = String(form?.get('password') ?? '').slice(0, 200);
    if (!sameText(await passToken(given), good)) {
      await countWrongGuess(request, env);
      return hasCookie ? Response.redirect(new URL('/', url), 303) : landing(true);
    }
    if (!env.PREVIEW_COOKIE_SECRET) {
      console.error('Preview: set PREVIEW_COOKIE_SECRET (npx wrangler secret put PREVIEW_COOKIE_SECRET) before the password form can sign anyone in.');
      return landing(false, true);
    }
    return new Response(null, {
      status: 303,
      headers: { location: '/', 'set-cookie': await gateCookie(env), 'cache-control': 'no-store' },
    });
  }

  if (hasCookie) return null;

  const header = request.headers.get('authorization') ?? '';
  if (header.startsWith('Basic ')) {
    let given = '';
    try {
      const decoded = new TextDecoder().decode(Uint8Array.from(atob(header.slice(6)), c => c.charCodeAt(0)));
      given = decoded.slice(decoded.indexOf(':') + 1);
    } catch {
      given = '';
    }
    if (await tooManyGuesses(request, env)) return landing(false, false, true);
    if (sameText(await passToken(given), good)) {
      // Hand out the signed cookie too, so the next requests skip the password check.
      return env.PREVIEW_COOKIE_SECRET ? { setCookie: await gateCookie(env) } : null;
    }
    await countWrongGuess(request, env);
  }
  return landing();
}

async function privately(response, env) {
  if (!env.SITE_PASSWORD) return response;
  const out = new Response(response.body, response);
  // Pages are never stored. Fingerprinted files may stay in the visitor's own browser cache.
  const html = (out.headers.get('content-type') ?? '').includes('text/html');
  const noStore = /no-store|no-cache/.test(out.headers.get('cache-control') ?? '');
  out.headers.set('cache-control', html || noStore ? 'private, no-store' : 'private, max-age=86400');
  out.headers.set('x-robots-tag', 'noindex, nofollow');
  return out;
}

// Defence in depth for every response. The site's scripts are all files on its own origin
// (no inline scripts), so scripts may load only from 'self'. The rockoon workbook runs its
// simulator as WebAssembly, so its pages may also compile WebAssembly ('wasm-unsafe-eval').
// Pages may connect only to this site and to the games' MoQ relay, over WebTransport (https:)
// or its WebSocket fallback (wss:).
const CONNECT_SRC = "connect-src 'self' https://cdn.moq.pro wss://cdn.moq.pro";
function secured(response, pathname = '') {
  const out = new Response(response.body, response);
  const wasm = pathname.startsWith('/rockoon/workbook/') ? " 'wasm-unsafe-eval'" : '';
  out.headers.set('content-security-policy', `script-src 'self'${wasm}; ${CONNECT_SRC}; frame-ancestors 'none'; base-uri 'none'; object-src 'none'; form-action 'self'`);
  out.headers.set('strict-transport-security', 'max-age=31536000; includeSubDomains');
  out.headers.set('x-content-type-options', 'nosniff');
  out.headers.set('referrer-policy', 'strict-origin-when-cross-origin');
  out.headers.set('permissions-policy', 'camera=(), microphone=(), geolocation=()');
  out.headers.set('cross-origin-opener-policy', 'same-origin');
  return out;
}

// Astro puts a content hash in every file name under /_astro/, so a changed file always gets
// a new URL and these can be cached for a year. Pages and other files keep the assets
// default (revalidate every time), so a deploy shows up at once.
function cachedLong(request, response) {
  if (response.status !== 200 || !new URL(request.url).pathname.startsWith('/_astro/')) return response;
  const out = new Response(response.body, response);
  out.headers.set('cache-control', 'public, max-age=31536000, immutable');
  return out;
}

export default {
  async fetch(request, env) {
    const moved = redirected(request);
    if (moved) return secured(moved);
    const gate = await locked(request, env);
    if (gate instanceof Response) return secured(gate);
    const response = await privately(cachedLong(request, await route(request, env)), env);
    if (gate?.setCookie) response.headers.append('set-cookie', gate.setCookie);
    return secured(response, new URL(request.url).pathname);
  },
};

async function route(request, env) {
  {
    const url = new URL(request.url);
    if (url.pathname === '/api/games/moq-token') return gameToken(request, env);
    if (url.pathname === '/api/games/rockoon-room') return rockoonRoom(request, env);
    if (url.pathname === '/api/guide') {
      if (request.method !== 'POST') return json({ error: 'Use POST.' }, 405);
      return guide(request, env);
    }
    const home = homeByHost.get(url.hostname);
    if (url.pathname === '/' && home) {
      url.pathname = home;
      return pageOrMissing(new Request(url, request), env);
    }
    if (url.pathname === '/start.sh') {
      // Show the installer as plain text. Browsers such as Firefox warn about, or download,
      // files served as shell scripts; `curl ... | bash` reads it the same either way.
      const script = await env.ASSETS.fetch(request);
      const out = new Response(script.body, script);
      out.headers.set('content-type', 'text/plain; charset=utf-8');
      out.headers.set('content-disposition', 'inline');
      out.headers.set('x-content-type-options', 'nosniff');
      out.headers.set('cache-control', 'no-cache');
      return out;
    }
    return pageOrMissing(request, env);
  }
}

// Browsers ask for a page with Accept: text/html. A missing stylesheet or script
// stays a plain 404 so the browser does not treat HTML as code.
function wantsPage(request) {
  const path = new URL(request.url).pathname;
  if (/\.(?:css|gif|ico|jpe?g|js|map|pdf|png|sh|svg|txt|webp|woff2?|xml)$/i.test(path)) return false;
  const accept = request.headers.get('accept') ?? '';
  return accept === '' || accept.includes('text/html') || accept.includes('*/*');
}

async function pageOrMissing(request, env) {
  const response = await env.ASSETS.fetch(request);
  if (response.status !== 404 || request.method !== 'GET' || !wantsPage(request)) return response;
  const missing = new URL(request.url);
  if (missing.pathname === '/404.html') return response;
  missing.pathname = '/404.html';
  missing.search = '';
  const fallback = await env.ASSETS.fetch(new Request(missing, { headers: request.headers }));
  if (!fallback.ok) return response;
  const headers = new Headers(fallback.headers);
  headers.set('content-type', 'text/html; charset=utf-8');
  headers.set('cache-control', 'no-store');
  return new Response(fallback.body, { status: 404, headers });
}
