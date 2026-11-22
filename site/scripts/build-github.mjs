// Builds the GitHub Pages copy of the site into docs/.
//
// With docs/CNAME (the custom domain is live), the site is served at the domain root.
// Without it, GitHub Pages serves the site at https://<owner>.github.io/<repo>/, so the
// build uses that path for every link and asset, and for canonical URLs.
import { spawnSync } from 'node:child_process';
import { existsSync, readFileSync } from 'node:fs';
import { repo } from '../src/hubs.mjs';

const cnameFile = new URL('../../docs/CNAME', import.meta.url);
const cname = existsSync(cnameFile) ? readFileSync(cnameFile, 'utf8').trim() : '';
const [, owner, name] = new URL(repo).pathname.split('/');
const base = cname ? '/' : `/${name}`;
const origin = cname ? `https://${cname}` : `https://${owner}.github.io/${name}`;
console.log(`GitHub Pages build for ${origin}`);

const astro = spawnSync('npx', ['astro', 'build', '--base', base], {
  stdio: 'inherit',
  env: { ...process.env, PUBLIC_SITE_ORIGIN: origin },
});
if (astro.status !== 0) process.exit(astro.status ?? 1);
await import('./publish-github.mjs');
