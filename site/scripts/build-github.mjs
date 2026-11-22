// Builds the GitHub Pages copy of the site into docs/. It is a mirror of the public site.
//
// GitHub Pages serves it at https://<owner>.github.io/<repo>/, so the build uses that path
// for every link and asset. If docs/CNAME gives it a domain of its own, it uses the root.
import { spawnSync } from 'node:child_process';
import { existsSync, readFileSync } from 'node:fs';
import { repo } from '../src/hubs.mjs';

const cnameFile = new URL('../../docs/CNAME', import.meta.url);
const cname = existsSync(cnameFile) ? readFileSync(cnameFile, 'utf8').trim() : '';
const [, owner, name] = new URL(repo).pathname.split('/');
const base = cname ? '/' : `/${name}`;
console.log(`GitHub Pages build for ${cname ? `https://${cname}` : `https://${owner}.github.io/${name}`}`);

const astro = spawnSync('npx', ['astro', 'build', '--base', base], {
  stdio: 'inherit',
});
if (astro.status !== 0) process.exit(astro.status ?? 1);
await import('./publish-github.mjs');
