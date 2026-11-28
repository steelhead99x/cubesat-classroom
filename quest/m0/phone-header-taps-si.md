# [S] Stop the phone header jumping, make taps 44px, and apply the SI house rule

## Goal

On a 360px-wide phone, the site header stays on one row and the menu does not flash open then shut on load (layout shift stays well under 0.1). Standalone links are at least 44px tall. The first user-facing "AI" on the public pages is written "SI (Super Intelligence)", with "SI" after. Project policy names and link labels stay as they are.

## Plan

1. In `site/public/level.js` (loaded in the document head before first paint), mark `document.documentElement.dataset.nav = 'js'` so CSS can start the phone menu closed, instead of waiting for the header script.
2. In `site/src/components/SiteHeader.astro`, hide the menu when `html[data-nav]` is set, keep the header on one row under 400px by showing only the Menu icon (the word stays for screen readers), and keep Games and Menu at least 44px tall.
3. In `site/src/theme.css` and the footer, give standalone links (calls to action, lesson titles, card links, 3MF downloads, glossary terms, footer links and the footer credit) a minimum height of 44px (`2.75rem`). Leave links inside sentences inline.
4. Apply the SI house rule on `/`, `/open-source/`, `/plans/`, the quick-start heading and the quest guide note.
5. Rebuild the static site (`npm test` and `npm run build` in `site/`). `docs/` was rebuilt with `npm run build:github`. If pull request 82 is also open, whichever merges second needs that rebuild run again, because both rewrite `docs/`.
6. Do not add or modify `.github/workflows`.

### How to check

- [ ] `npm test` in `site/` reports 81 passing tests, and `npm run build` completes.
- [ ] At 360px, the header is one row and the phone menu starts closed (no open-then-shut flash). Layout shift is well under 0.1.
- [ ] Standalone links are at least 44px tall; in-sentence links stay inline.
- [ ] The first user-facing "AI" on the listed pages is "SI (Super Intelligence)". Policy names such as "AI policy" are unchanged.
- [ ] `quest --root . check` exits 0, and the pull request was not merged automatically.
