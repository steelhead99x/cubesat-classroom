import { cp, rm } from 'node:fs/promises';

const build = new URL('../dist/', import.meta.url);
const pages = new URL('../../docs/', import.meta.url);

await rm(new URL('_astro', pages), { recursive: true, force: true });
for (const file of ['index.html', 'mark.svg', '_astro']) {
  await cp(new URL(file, build), new URL(file, pages), { recursive: true });
}
