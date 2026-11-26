# [S] Redirect www hosts to their apex on the public Worker

## Goal

A request to `www.cubesatclass.org` (and www of the hub and redirect domains) receives a 301 to the same path on the apex host, with the custom domains kept in `site/wrangler.jsonc`.

## Plan

1. Read `redirected()` in `site/src/worker.mjs` and the `redirects` map in `site/src/hubs.mjs`. After the redirect-domain map, a www host whose apex is the classroom host or a hub host must 301 to `https://{apex}{pathname}{search}`.
2. Confirm `site/wrangler.jsonc` lists the apex and www custom-domain routes, and that `workers_dev` and `preview_urls` stay off, so the site is only served on its custom domains.
3. From a machine that can reach the live Worker, request `https://www.cubesatclass.org/` and `https://www.cubesat.club/start.sh` with `curl -sI` and check for a 301 (or a 308 from a zone rule) whose Location keeps the path and uses the apex host.

### How to check

- [ ] `www.cubesatclass.org` is not left unredirected on the branch that introduced the check (merged in pull request 21).
- [ ] `quest check` from the repository root still passes.
