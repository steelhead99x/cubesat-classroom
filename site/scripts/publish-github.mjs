import { cp, rm } from 'node:fs/promises';

const build = new URL('../dist-github/', import.meta.url);
const pages = new URL('../../docs/', import.meta.url);

const directories = ['_astro', 'quests', 'plans', 'hardware', 'mission', 'volunteer', 'open-source', 'hub', 'wiki', 'games', 'rockoon', 'privacy', 'terms'];
for (const directory of directories) {
  await rm(new URL(directory, pages), { recursive: true, force: true });
}
for (const file of ['index.html', '404.html', 'robots.txt', 'sitemap.xml', 'level.js', 'favicon.ico', 'favicon.svg', 'apple-touch-icon.png', 'icon-192.png', 'icon-512.png', 'site.webmanifest', 'mark.svg', 'club-mark.svg', ...directories]) {
  await cp(new URL(file, build), new URL(file, pages), { recursive: true });
}
// docs/CNAME is managed by hand (or in the repository's Pages settings), not by this build:
// add it only once the custom domain's DNS points at GitHub Pages.
