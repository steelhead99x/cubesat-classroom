# [S] Move cubesat.app to the app Worker and align site branding

## Goal

`cubesatclass.org` matches the `cubesat-classroom-app` branding, links out to `https://cubesat.app/`, and releases the `cubesat.app` and `www.cubesat.app` custom-domain routes from `site/wrangler.jsonc` cleanly before the app Worker claims them.

## Plan

1. Add the `#app` ("Follow along on every screen") section and **App** nav link on `site/src/pages/index.astro`, linking to `https://cubesat.app/` and `https://cubesat.app/#learn`.
2. Update the `mobile` entry in `site/src/hubs.mjs` with `path: '/'` and `home: 'https://cubesat.app/'` so `homeByHost` in `site/src/worker.mjs` stops claiming `cubesat.app` and footer/hub links point to `https://cubesat.app/`.
3. Remove `cubesat.app` and `www.cubesat.app` custom domains from `site/wrangler.jsonc`.
4. Rebuild `site/` and `docs/` (`npm test`, `npm run build`, `npm run build:github`).
5. Deploy in safe order after merge: merge PR #35 -> deploy `cubesat-classroom` Worker (releasing `cubesat.app`) -> verify in Cloudflare -> deploy `cubesat-classroom-app` Worker (claiming `cubesat.app`). Do not merge automatically.

### How to check

- [ ] `site/wrangler.jsonc` no longer lists `cubesat.app` or `www.cubesat.app` custom domains.
- [ ] `site/src/hubs.mjs` sets `home: 'https://cubesat.app/'` and `path: '/'` on the `mobile` (`App`) hub.
- [ ] `cd site && npm test && npm run build && npm run build:github` passes with `docs/` in sync.
- [ ] `quest --root . check` exits 0.
- [ ] Review posted on PR #35 documenting the safe deploy sequence (merge #35 -> deploy classroom -> deploy app).
