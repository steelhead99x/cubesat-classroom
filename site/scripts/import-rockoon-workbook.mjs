// Refresh the simulator engine from the Rockoon workbook. Classroom owns the page
// and browser code because its observation storage and navigation differ from upstream.
// Run the workbook smoke test after importing a new engine.
import { readFile, writeFile } from 'node:fs/promises';

const source = process.argv[2];
if (!source) throw new Error('usage: node scripts/import-rockoon-workbook.mjs <workbook.html>');
const target = new URL('../public/rockoon/workbook/workbook.js', import.meta.url);
const engine = /const WASM = "[A-Za-z0-9+/=]+";/g;
const [html, workbook] = await Promise.all([readFile(source, 'utf8'), readFile(target, 'utf8')]);
const incoming = html.match(engine);
if (incoming?.length !== 1 || workbook.match(engine)?.length !== 1) {
  throw new Error('expected exactly one embedded simulator in both files');
}
const bytes = Buffer.from(incoming[0].split('"')[1], 'base64');
const exports = new Set(WebAssembly.Module.exports(new WebAssembly.Module(bytes)).map(entry => entry.name));
if (['memory', 'alloc', 'dealloc', 'ground_report'].some(name => !exports.has(name))) {
  throw new Error('the incoming simulator uses a different interface');
}
await writeFile(target, workbook.replace(engine, () => incoming[0]));
console.log('Updated the engine in public/rockoon/workbook/workbook.js.');
