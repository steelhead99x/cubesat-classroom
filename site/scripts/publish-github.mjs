import { cp, rm } from 'node:fs/promises';

const build = new URL('../dist/', import.meta.url);
const pages = new URL('../../docs/', import.meta.url);

await rm(new URL('_astro', pages), { recursive: true, force: true });
await rm(new URL('quests', pages), { recursive: true, force: true });
await rm(new URL('plans', pages), { recursive: true, force: true });
await rm(new URL('hardware', pages), { recursive: true, force: true });
await rm(new URL('mission', pages), { recursive: true, force: true });
await rm(new URL('volunteer', pages), { recursive: true, force: true });
await rm(new URL('open-source', pages), { recursive: true, force: true });
await rm(new URL('hub', pages), { recursive: true, force: true });
for (const file of ['index.html', '404.html', 'robots.txt', 'sitemap.xml', 'level.js', 'favicon.svg', 'mark.svg', 'club-mark.svg', '_astro', 'quests', 'plans', 'hardware', 'mission', 'volunteer', 'open-source', 'hub']) {
  await cp(new URL(file, build), new URL(file, pages), { recursive: true });
}
// docs/CNAME is managed by hand (or in the repository's Pages settings), not by this build:
// add it only once the custom domain's DNS points at GitHub Pages.
