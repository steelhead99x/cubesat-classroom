// Copies the rockoon kit's ground-flight workbook into public/rockoon/workbook/.
//
//   node scripts/import-rockoon-workbook.mjs <path to cubesat-classroom-rockoon>/web/dist/workbook.html
//
// The workbook is one HTML file with an inline script that carries the simulator as
// WebAssembly. The site's CSP allows scripts only from this origin, so the script moves to
// workbook.js. The Worker adds 'wasm-unsafe-eval' for /rockoon/workbook/ only (src/worker.mjs).
import { mkdir, readFile, writeFile } from 'node:fs/promises';

const source = process.argv[2];
if (!source) throw new Error('usage: node scripts/import-rockoon-workbook.mjs <workbook.html>');
const target = new URL('../public/rockoon/workbook/', import.meta.url);

let html = await readFile(source, 'utf8');
const scripts = [...html.matchAll(/<script>([\s\S]*?)<\/script>/g)];
if (scripts.length !== 1) throw new Error(`expected one inline script, found ${scripts.length}`);
html = html.replace(scripts[0][0], '<script src="/rockoon/workbook/workbook.js"></script>');
// A way back to the classroom site, and who the page is for.
html = html.replace(
  '<header>\n',
  '<header>\n  <p><a href="/quests/#play-and-learn" style="color: #a4c9ff">← CubeSat Classroom · Play &amp; learn</a></p>\n  <p class="label">Hands-on activity for older learners, with an adult. Model rockets fly only at a club launch, under its safety officer.</p>\n',
);
if (/<script(?![^>]*\ssrc=)/i.test(html)) throw new Error('an inline script is left');
if (/kyle|douglas|kdouglas/i.test(html + scripts[0][1])) throw new Error('personal name found');

await mkdir(target, { recursive: true });
await writeFile(new URL('index.html', target), html);
await writeFile(new URL('workbook.js', target), scripts[0][1].trim() + '\n');
console.log('Wrote public/rockoon/workbook/');
