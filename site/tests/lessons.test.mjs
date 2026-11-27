// Every browser lesson needs its guide answers, a patch colour and a patch icon.
import test from 'node:test';
import assert from 'node:assert/strict';
import { existsSync, readdirSync, readFileSync } from 'node:fs';
import { byQuest } from '../src/guide/answers.mjs';

const dir = new URL('../src/content/quests/', import.meta.url);
const ids = readdirSync(dir).filter(f => f.endsWith('.md')).map(f => f.slice(0, -3));
const art = JSON.parse(readFileSync(new URL('../src/patch-art.json', import.meta.url), 'utf8'));
const shapes = JSON.parse(readFileSync(new URL('../src/patch-shapes.json', import.meta.url), 'utf8'));

test('each lesson has a hint and a why at every level', () => {
  assert.ok(ids.length >= 5);
  for (const id of ids) {
    for (const topic of ['hint', 'why']) {
      for (const level of ['beginner', 'intermediate', 'professional']) {
        assert.ok(byQuest[id]?.[topic]?.[level], `${id} ${topic} ${level}`);
      }
    }
  }
});

test('each lesson has a patch colour and icon', () => {
  for (const id of ids) {
    const patch = art.patches[id];
    assert.match(patch?.tone ?? '', /^#[0-9a-f]{6}$/i, `tone for ${id}`);
    assert.ok(patch.shapes.length > 0, `icon for ${id}`);
  }
});

test("each patch's label outline was generated from the quest's current patch name", () => {
  // Fails when a patch name changes without `python3 hardware/patches/gen/patches.py build`.
  for (const id of ids) {
    const label = readFileSync(new URL(`${id}.md`, dir), 'utf8').match(/^patch: (.+)$/m)[1].trim();
    assert.equal(shapes.labels[id]?.label, label, `label outline for ${id}`);
    assert.ok(shapes.labels[id].d.startsWith('M'), `label path for ${id}`);
  }
});

test('lesson order numbers are 1 to n with no gaps', () => {
  const orders = ids.map(id => Number(readFileSync(new URL(`${id}.md`, dir), 'utf8').match(/^order: (\d+)$/m)[1])).sort((a, b) => a - b);
  assert.deepEqual(orders, orders.map((_, i) => i + 1));
});

test('lesson inline links keep the site base and format only their labels', async () => {
  const { inline } = await import('../src/levels.ts');
  assert.equal(inline('[**Low pass**](/games/catch-the-pass/#level=2)', '/cubesat-classroom'),
    '<a href="/cubesat-classroom/games/catch-the-pass/#level=2"><strong>Low pass</strong></a>');
  assert.equal(inline('[`logic.mjs`](https://example.com/**path**?a=1&b=2)'),
    '<a href="https://example.com/**path**?a=1&amp;b=2"><code>logic.mjs</code></a>');
  assert.equal(inline('[relative](//example.com) and `**literal**`'), 'relative and <code>**literal**</code>');
});

test('each patch has a print file for the patches page to link', () => {
  for (const id of ids) {
    assert.ok(existsSync(new URL(`../../hardware/patches/print/${id}.3mf`, import.meta.url)), `print file for ${id}`);
  }
});
