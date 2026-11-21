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

export function earned(): Set<string> {
  try { return new Set(JSON.parse(localStorage.getItem(KEY) ?? '[]')); } catch { return new Set(); }
}

export function earn(id: string) {
  const set = earned();
  if (set.has(id)) return;
  set.add(id);
  try { localStorage.setItem(KEY, JSON.stringify([...set])); } catch {}
  document.dispatchEvent(new CustomEvent('patchearned', { detail: id }));
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
