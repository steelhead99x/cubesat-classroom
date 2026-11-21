// Mission patches earned in the example quests. Progress is stored only in this
// browser (localStorage) and never leaves the page.
export const patchTones: Record<string, string> = {
  'first-contact': '#a4c9ff',
  'bring-back-a-picture': '#ffd23f',
  'lose-the-signal': '#ef6b6c',
  'fresh-or-historical': '#6fd3a0',
  'light-and-shadow': '#b9a4ff',
};

const KEY = 'cubesat.patches';
// Kept in memory too, so patches still work for this visit if storage is blocked.
const session = new Set<string>();

export function earned(): Set<string> {
  let stored: string[] = [];
  try { stored = JSON.parse(localStorage.getItem(KEY) ?? '[]'); } catch {}
  return new Set([...stored, ...session]);
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
