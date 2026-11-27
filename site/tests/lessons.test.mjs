// Every browser lesson needs its guide answers, a patch colour and a patch icon.
import test from 'node:test';
import assert from 'node:assert/strict';
import { readdirSync, readFileSync } from 'node:fs';
import { byQuest } from '../src/guide/answers.mjs';

const dir = new URL('../src/content/quests/', import.meta.url);
const ids = readdirSync(dir).filter(f => f.endsWith('.md')).map(f => f.slice(0, -3));
const tones = readFileSync(new URL('../src/patches.ts', import.meta.url), 'utf8');
const icons = readFileSync(new URL('../src/components/Patch.astro', import.meta.url), 'utf8');

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
    assert.match(tones, new RegExp(`'${id}':`), `tone for ${id}`);
    assert.match(icons, new RegExp(`'${id}':`), `icon for ${id}`);
  }
});

test('lesson order numbers are 1 to n with no gaps', () => {
  const orders = ids.map(id => Number(readFileSync(new URL(`${id}.md`, dir), 'utf8').match(/^order: (\d+)$/m)[1])).sort((a, b) => a - b);
  assert.deepEqual(orders, orders.map((_, i) => i + 1));
});
