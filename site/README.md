# CubeSat Classroom website

The public site connects K–12 classrooms and volunteer mentors to the ground station, balloon lesson, 3D-print files, and contribution quests. The Astro pages and Tailwind styles live in `src/`. The CubeSat mark lives in `public/mark.svg`. The banner uses the project's Rev B CAD preview.

## Preview locally

Use Node.js 24.

```sh
cd site
npm ci
npm run dev
```

## Publish

For GitHub Pages, run `npm run build:github` from `site/` and commit the generated files in `docs/` along with the source changes. GitHub Pages serves `docs/` on `main`, including the classroom installer at `start.sh`.

For Cloudflare, run `npm run build` from `site/`. Deploy `dist/` as [Worker Static Assets](https://developers.cloudflare.com/workers/static-assets/direct-upload/) on the `cubesat-classroom` Worker. Include `src/worker.mjs` as the Worker entry point and `src/hubs.mjs` as a module. Set the static assets binding to `ASSETS` and run the Worker first for `/`. Other paths serve the assets directly.

| Custom domain | Role |
| --- | --- |
| `cubesatclass.org` | Classroom home, curriculum, and teacher pilot |
| `cubesatclass.com` | Redirect to `cubesatclass.org` |
| `cubesat.app` | iOS and Android template |
| `cubesatclass.io` | Ground station and radio |
| `cubesatclass.dev` | Software development and MoQ.pro testing |
| `cubesat.club` | Quick start, volunteer contributions, and fundraising links |

## Quest guide

The quest guide on the home and quest pages works without any setup: it shows the prewritten answers in `src/guide/answers.mjs`. Visitors choose from fixed questions, and there is no typing box.

To switch on the live guide (Claude Sonnet 5.5 through the Anthropic API) on Cloudflare:

1. Create an API key in the [Claude Console](https://platform.claude.com/). Do not use a personal Claude subscription.
2. Store it as a Worker secret: `npx wrangler secret put ANTHROPIC_API_KEY`. Never put it in the site code or a `PUBLIC_` variable.
3. Optional but recommended: bind a KV namespace as `GUIDE_LIMITS` and set the secret `GUIDE_SALT`. This turns on per-device daily limits, counted with a salted hash that rotates daily, and caches answers for a day.
4. Run the Worker first for `/api/*` as well as `/`.
5. Rebuild the site with `PUBLIC_GUIDE_ENDPOINT=/api/guide npm run build` and deploy.

Optional settings: `GUIDE_MODEL` (default `claude-sonnet-5-5`), `GUIDE_EFFORT` (default `low`), `GUIDE_DAILY_LIMIT` (default 10 per device), `GUIDE_GLOBAL_LIMIT` (default 2000 a day for the site), `GUIDE_ALLOWED_ORIGINS` (comma-separated). If the API fails or a limit is reached, visitors get the prewritten answer. The guide's rules mirror [AGENTS.md](../AGENTS.md); review them with a teacher before switching it on for students.

The [Worker Custom Domains](https://developers.cloudflare.com/workers/configuration/routing/custom-domains/) share one build. The guided installer is published at `https://cubesat.club/start.sh` and mirrored by GitHub Pages. The Rust station and its MoQ/QUIC relays run locally during the lesson.
