// Mission patches earned in the example quests. Progress is stored only in this
// browser (localStorage) and never leaves the page.
import art from './patch-art.json';

// Each patch's colour comes from patch-art.json, which the printed patches use too.
export const patchTones: Record<string, string> = Object.fromEntries(
  Object.entries(art.patches).map(([id, patch]) => [id, patch.tone]),
);

const KEY = 'cubesat.patches';
// Kept in memory too, so patches still work for this visit if storage is blocked.
const session = new Set<string>();

export function earned(): Set<string> {
  // Count only known patches, so an old or damaged saved value cannot break the shelf
  // or inflate the total.
  let stored: unknown = [];
  try { stored = JSON.parse(localStorage.getItem(KEY) ?? '[]'); } catch {}
  const saved = Array.isArray(stored) ? stored.filter((id): id is string => typeof id === 'string' && id in patchTones) : [];
  return new Set([...saved, ...session]);
}

export function earn(id: string) {
  const set = earned();
  if (set.has(id)) return;
  session.add(id);
  set.add(id);
  try { localStorage.setItem(KEY, JSON.stringify([...set])); } catch {}
  document.dispatchEvent(new CustomEvent('patchearned', { detail: id }));
}

// Keep other open tabs in step (browser only; this module is also used at build time).
if (typeof window !== 'undefined') {
  window.addEventListener('storage', event => {
    if (event.key === KEY) document.dispatchEvent(new CustomEvent('patchearned', { detail: '' }));
  });
}

export function paint(justEarned?: string) {
  const set = earned();
  for (const patch of document.querySelectorAll<SVGElement>('[data-patch]')) {
    const id = patch.dataset.patch!;
    const has = set.has(id);
    patch.toggleAttribute('data-earned', has);
    patch.querySelector('title')!.textContent = `${patch.dataset.name}: ${has ? 'earned' : 'not earned yet'}`;
    if (id === justEarned) {
      patch.classList.remove('patch-just-earned');
      void patch.getBoundingClientRect();
      patch.classList.add('patch-just-earned');
    }
  }
}
