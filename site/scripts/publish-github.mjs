import { cp, rm } from 'node:fs/promises';

const build = new URL('../dist/', import.meta.url);
const pages = new URL('../../docs/', import.meta.url);

await rm(new URL('_astro', pages), { recursive: true, force: true });
await rm(new URL('quests', pages), { recursive: true, force: true });
await rm(new URL('plans', pages), { recursive: true, force: true });
for (const file of ['index.html', 'mark.svg', '_astro', 'quests', 'plans']) {
  await cp(new URL(file, build), new URL(file, pages), { recursive: true });
}
