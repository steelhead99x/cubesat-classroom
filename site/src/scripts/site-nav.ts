// Site menu: the narrow-screen Menu button and the desktop "More" list.
// Loaded as a file (no inline script) so the CSP can keep script-src 'self'.
const NARROW = '(max-width: 56rem)';

export function initSiteNav() {
  for (const header of document.querySelectorAll<HTMLElement>('[data-site-header]')) {
    if (header.dataset.js) continue;
    header.dataset.js = 'true';
    const toggle = header.querySelector<HTMLButtonElement>('[data-menu-toggle]')!;
    const panel = header.querySelector<HTMLElement>('[data-menu-panel]')!;
    const narrow = matchMedia(NARROW);
    toggle.hidden = false;

    const focusables = () => [toggle, ...panel.querySelectorAll<HTMLElement>('a[href], button:not([hidden])')].filter(el => el.offsetParent !== null);
    const setMenu = (open: boolean, focusBack = false) => {
      toggle.setAttribute('aria-expanded', String(open));
      toggle.querySelector('span:last-child')!.textContent = open ? 'Close' : 'Menu';
      if (open) panel.dataset.open = 'true'; else delete panel.dataset.open;
      if (open) focusables()[1]?.focus();
      else if (focusBack) toggle.focus();
    };
    toggle.addEventListener('click', () => setMenu(toggle.getAttribute('aria-expanded') !== 'true'));
    narrow.addEventListener('change', () => setMenu(false));

    // "More" dropdowns on wide screens.
    const drops = [...header.querySelectorAll<HTMLElement>('[data-dropdown]')];
    const setDrop = (dd: HTMLElement, open: boolean, focusBack = false) => {
      const b = dd.querySelector<HTMLButtonElement>('.dd-toggle')!;
      b.setAttribute('aria-expanded', String(open));
      if (open) dd.dataset.open = 'true'; else delete dd.dataset.open;
      if (focusBack) b.focus();
    };
    for (const dd of drops) {
      const b = dd.querySelector<HTMLButtonElement>('.dd-toggle')!;
      b.addEventListener('click', () => setDrop(dd, !dd.dataset.open));
      b.addEventListener('keydown', e => {
        if (e.key === 'ArrowDown') { e.preventDefault(); e.stopPropagation(); setDrop(dd, true); dd.querySelector<HTMLElement>('.dd-menu a')?.focus(); }
      });
      dd.addEventListener('focusout', e => { if (!dd.contains(e.relatedTarget as Node)) setDrop(dd, false); });
      dd.addEventListener('keydown', e => {
        const items = [...dd.querySelectorAll<HTMLElement>('.dd-menu a')];
        const i = items.indexOf(document.activeElement as HTMLElement);
        if (i < 0) return;
        if (e.key === 'ArrowDown' || e.key === 'ArrowUp') { e.preventDefault(); items[(i + (e.key === 'ArrowDown' ? 1 : items.length - 1)) % items.length].focus(); }
        if (e.key === 'Home') { e.preventDefault(); items[0].focus(); }
        if (e.key === 'End') { e.preventDefault(); items[items.length - 1].focus(); }
      });
    }
    document.addEventListener('click', e => {
      for (const dd of drops) if (dd.dataset.open && !dd.contains(e.target as Node)) setDrop(dd, false);
      if (panel.dataset.open && !header.contains(e.target as Node)) setMenu(false);
    });

    header.addEventListener('keydown', e => {
      if (e.key === 'Escape') {
        const dd = drops.find(d => d.dataset.open);
        if (dd) { setDrop(dd, false, true); return; }
        if (panel.dataset.open) { e.preventDefault(); setMenu(false, true); }
        return;
      }
      // Keep Tab inside the open narrow-screen menu.
      if (e.key === 'Tab' && panel.dataset.open && narrow.matches) {
        const f = focusables(), first = f[0], last = f[f.length - 1];
        if (e.shiftKey && document.activeElement === first) { e.preventDefault(); last.focus(); }
        else if (!e.shiftKey && document.activeElement === last) { e.preventDefault(); first.focus(); }
      }
    });
    // Same-page links (#demo, #labs) close the menu.
    panel.addEventListener('click', e => { if ((e.target as Element).closest('a')) { setMenu(false); for (const dd of drops) setDrop(dd, false); } });
  }
}
