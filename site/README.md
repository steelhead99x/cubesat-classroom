# CubeSat Classroom website

The public site connects learners 13 and up, their teachers and guardians, and volunteer mentors to the ground station, balloon lesson, 3D-print files, and contribution quests. The Astro pages and Tailwind styles live in `src/`. The CubeSat mark lives in `public/mark.svg`. The banner uses the project's Rev B CAD preview.

## Preview locally

Use Node.js 24.

```sh
cd site
npm ci
npm run dev
```

## Publish

For GitHub Pages, run `npm run build:github` from `site/` and commit the generated files in `docs/` along with the source changes. GitHub Pages serves `docs/` on `main` at the custom domain in `publicSite` (`cubesatclassroom.org`), including the classroom installer at `start.sh`. Once the domain's DNS points at GitHub Pages, add `docs/CNAME` containing the domain (or set it in the repository's Pages settings). The build does not create it, so the `github.io` address keeps working until then. In the repository's Pages settings, set the custom domain to the same name and turn on **Enforce HTTPS**. At the domain registrar, point the apex at GitHub Pages (A records `185.199.108.153`, `185.199.109.153`, `185.199.110.153`, `185.199.111.153`) and `www` at `steelhead99x.github.io` with a CNAME record.

For Cloudflare, run `npm run build` from `site/`. Deploy `dist/` as [Worker Static Assets](https://developers.cloudflare.com/workers/static-assets/direct-upload/) on the `cubesat-classroom` Worker. Include `src/worker.mjs` as the Worker entry point and `src/hubs.mjs` as a module. Set the static assets binding to `ASSETS` and run the Worker first for `/`. Other paths serve the assets directly.

| Custom domain | Role |
| --- | --- |
| `cubesatclass.org` | Classroom home, curriculum, and teacher pilot |
| `cubesatclass.com` | Redirect to `cubesatclass.org` |
| `cubesat.app` | iOS and Android template |
| `cubesatclass.io` | Ground station and radio |
| `cubesatclass.dev` | Software development and MoQ.pro testing |
| `cubesat.club` | Permanent redirect to the same path on `cubesatclassroom.org` (set in `redirects` in `src/hubs.mjs`) |

`cubesatclassroom.org` is not a Worker domain. It is the GitHub Pages copy of this repository, so it changes only through reviewed pull requests to `main`.

## Quest guide

The quest guide on the home and quest pages works without any setup: it shows the prewritten answers in `src/guide/answers.mjs`. Visitors choose from fixed questions, and there is no typing box.

To switch on the live guide (Claude Sonnet 5.5 through the Anthropic API) on Cloudflare:

1. Create an API key in the [Claude Console](https://platform.claude.com/). Do not use a personal Claude subscription.
2. Store it as a Worker secret: `npx wrangler secret put ANTHROPIC_API_KEY`. Never put it in the site code or a `PUBLIC_` variable.
3. Required for live answers: bind a KV namespace as `GUIDE_LIMITS` and set the secret `GUIDE_SALT` to a long random value. Without both, the guide only shows prewritten answers. They turn on daily limits per device, per network address and for the whole site, and cache answers for a day. Devices and network addresses are stored only as salted hashes that change every day and expire after 25 hours.
4. Run the Worker first for `/api/*` as well as `/`.
5. Rebuild the site with `PUBLIC_GUIDE_ENDPOINT=/api/guide npm run build` and deploy.

Optional settings: `GUIDE_MODEL` (default `claude-sonnet-5-5`), `GUIDE_EFFORT` (default `low`), `GUIDE_DAILY_LIMIT` (default 10 per device), `GUIDE_NETWORK_LIMIT` (default 300 a day per network address, so a changed device id cannot get around the device limit), `GUIDE_GLOBAL_LIMIT` (default 2000 a day for the site), `GUIDE_ALLOWED_ORIGINS` (comma-separated; by default only the site's own origin may call the guide). If the API fails, or the network or site-wide limit is reached, visitors get the prewritten answer. A device that reaches its own daily limit is asked to come back tomorrow. The guide's rules mirror [AGENTS.md](../AGENTS.md); review them with a teacher before switching it on for students.

The [Worker Custom Domains](https://developers.cloudflare.com/workers/configuration/routing/custom-domains/) share one build. The guided installer is published by GitHub Pages at `https://cubesatclassroom.org/start.sh`; the old `https://cubesat.club/start.sh` address redirects there. The Rust station and its MoQ/QUIC relays run locally during the lesson.

## Private preview (password)

The Worker can keep every domain behind one password until the site is ready. Visitors see an "under construction" page with the mission statement and a password box. The password is a Worker secret. Never write it in this public repository.

1. Set the secret: `npx wrangler secret put SITE_PASSWORD`, then type the password when asked.
2. Run the Worker first for **every** path, not only `/`. `wrangler.jsonc` in this folder sets `run_worker_first` for you; if you deploy another way, set it yourself. Otherwise pages other than the home page are served without the check. After deploying, open a page such as `/quests/` in a private window and confirm it shows the under-construction page.
3. Required: `npx wrangler secret put PREVIEW_COOKIE_SECRET` with a long random value, so the sign-in cookie cannot be used to guess the password. Without it the password form does not sign anyone in. Add a Cloudflare rate-limiting rule on `POST /preview-unlock` to slow down guessing.
4. Deploy. The right password sets a cookie for 30 days on that domain. Each domain asks once.
5. To open the site, delete the secret (`npx wrangler secret delete SITE_PASSWORD`) and deploy again.

The wording of the under-construction page is in `landing()` in `src/worker.mjs`.

The published installer address, `https://cubesatclassroom.org/start.sh` (and the old `cubesat.club` address that redirects there), is served by GitHub Pages and never needs the password. While the password is on, `curl https://cubesatclass.org/start.sh` on a Worker domain does need it (`curl -u :PASSWORD ...`). People without it can clone the repository and run `bash docs/start.sh`. The GitHub Pages copy under `docs/` is a public part of the repository and is not covered by the password; turn Pages off in the repository settings if it should not be reachable. One shared password keeps casual visitors out; it is not strong protection, so don't put anything private on the site.
