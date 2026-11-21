import { cp, rm, writeFile } from 'node:fs/promises';
import { publicSite } from '../src/hubs.mjs';

const build = new URL('../dist/', import.meta.url);
const pages = new URL('../../docs/', import.meta.url);

await rm(new URL('_astro', pages), { recursive: true, force: true });
await rm(new URL('quests', pages), { recursive: true, force: true });
await rm(new URL('plans', pages), { recursive: true, force: true });
await rm(new URL('hardware', pages), { recursive: true, force: true });
await rm(new URL('mission', pages), { recursive: true, force: true });
await rm(new URL('volunteer', pages), { recursive: true, force: true });
for (const file of ['index.html', 'mark.svg', 'club-mark.svg', '_astro', 'quests', 'plans', 'hardware', 'mission', 'volunteer']) {
  await cp(new URL(file, build), new URL(file, pages), { recursive: true });
}
// GitHub Pages serves docs/ at this custom domain.
await writeFile(new URL('CNAME', pages), publicSite + '\n');
