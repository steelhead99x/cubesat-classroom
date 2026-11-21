// Copies the browser lessons from site/src/content/quests into the app, so the
// phone and the website teach the same steps. Run: npm run sync:quests
import { readdir, readFile, writeFile } from 'node:fs/promises';
import yaml from 'js-yaml';

const source = new URL('../../site/src/content/quests/', import.meta.url);
const quests = [];
for (const file of (await readdir(source)).filter(name => name.endsWith('.md')).sort()) {
  const text = await readFile(new URL(file, source), 'utf8');
  const front = text.split(/^---$/m)[1];
  quests.push({ id: file.replace(/\.md$/, ''), ...yaml.load(front) });
}
quests.sort((a, b) => a.order - b.order);
await writeFile(new URL('../src/data/quests.json', import.meta.url), JSON.stringify(quests, null, 2) + '\n');
console.log(`Wrote ${quests.length} quests.`);
