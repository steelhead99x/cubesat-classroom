import { hubs } from './hubs.mjs';
import { answerFor, questionText } from './guide/answers.mjs';

const homeByHost = new Map(hubs.filter(hub => hub.slug !== 'lessons').map(({ host, slug }) => [host, slug === 'volunteer' ? '/volunteer/' : '/hub/' + slug + '/']));

// Classroom guardrails for the live quest guide. Mirrors AGENTS.md.
const SYSTEM = `You are the CubeSat Classroom quest guide, an AI helping K-12 students, teachers and amateur radio mentors with a practice satellite ground station.
Rules:
- Only discuss CubeSats, orbits, amateur radio and AMSAT, sensors and units, measurement and data, the practice ground station and the current quest. If asked anything else, say in one sentence that you only help with the mission.
- Teach, do not do the work: give hints and ask questions. Never write a student's crew log or answer their "talk it over" question for them.
- Never ask for or repeat names, ages, schools, locations, photos or contact details.
- Do not discuss violence, self-harm, dating, drugs or other unsafe topics; suggest talking to a trusted adult.
- Transmitting on amateur bands needs a licensed operator. Never help anyone get around radio rules.
- If you are not sure, say so.
- Answer in at most 90 words, in plain text, at the reader's level.`;

const LEVEL_NOTE = {
  beginner: 'The reader is a beginner, possibly 9 years old. Use short sentences and everyday words.',
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
  await kv.put(key, String(used + 1), { expirationTtl: 60 * 60 * 48 });
  return true;
}

// POST /api/guide { quest, question, level }
// Off unless ANTHROPIC_API_KEY is set. Only fixed question ids are accepted, so
// nothing a visitor types reaches the model. Optional KV binding GUIDE_LIMITS adds
// per-device daily limits (salted, daily-rotating hash) and caches answers.
async function guide(request, env) {
  if (!env.ANTHROPIC_API_KEY) return json({ error: 'The live guide is not switched on yet.' }, 503);
  const origin = request.headers.get('origin');
  if (origin && env.GUIDE_ALLOWED_ORIGINS && !env.GUIDE_ALLOWED_ORIGINS.split(',').includes(origin)) return json({ error: 'Forbidden' }, 403);

  const body = await request.json().catch(() => null);
  const { quest = '', question = '', level = '' } = body ?? {};
  const fallback = answerFor(quest, question, level);
  const asked = questionText(quest, question);
  if (!fallback || !asked) return json({ error: 'Unknown question.' }, 400);

  const kv = env.GUIDE_LIMITS;
  const day = new Date().toISOString().slice(0, 10);
  const cacheKey = `answer:${day}:${quest}:${question}:${level}`;
  if (kv) {
    const cached = await kv.get(cacheKey);
    if (cached) return json({ answer: cached, cached: true });
    const client = (request.headers.get('x-guide-client') ?? '').slice(0, 64) || 'anonymous';
    const device = await sha256(`${env.GUIDE_SALT ?? ''}:${day}:${client}`);
    const perDevice = Number(env.GUIDE_DAILY_LIMIT ?? 10);
    const perSite = Number(env.GUIDE_GLOBAL_LIMIT ?? 2000);
    if (!(await bump(kv, `site:${day}`, perSite))) return json({ answer: fallback, limited: 'site' });
    if (!(await bump(kv, `device:${device}`, perDevice))) {
      return json({ message: 'That is all the questions for today. Keep going with the quest steps, and come back tomorrow.' }, 429);
    }
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
      max_tokens: 1024,
      output_config: { effort: env.GUIDE_EFFORT ?? 'low' },
      system: `${SYSTEM}\n${LEVEL_NOTE[level]}`,
      messages: [{
        role: 'user',
        content: `Current quest: ${quest || 'none, general question'}.\nQuestion: ${asked}\nA good answer looks like this example; write your own in the same spirit:\n${fallback}`,
      }],
    }),
  }).catch(() => null);

  if (!res || !res.ok) return json({ answer: fallback, fallback: true });
  const data = await res.json().catch(() => null);
  const answer = data?.content?.find(block => block.type === 'text')?.text?.trim();
  if (!answer) return json({ answer: fallback, fallback: true });
  if (kv) await kv.put(cacheKey, answer, { expirationTtl: 60 * 60 * 24 });
  return json({ answer });
}

export default {
  async fetch(request, env) {
    const url = new URL(request.url);
    if (url.pathname === '/api/guide') {
      if (request.method !== 'POST') return json({ error: 'Use POST.' }, 405);
      return guide(request, env);
    }
    const home = homeByHost.get(url.hostname);
    if (url.pathname === '/' && home) {
      url.pathname = home;
      return env.ASSETS.fetch(new Request(url, request));
    }
    return env.ASSETS.fetch(request);
  },
};
