# [S] Add Privacy and Terms pages and footer links

## Goal

The public site footer’s bottom row links to Privacy and Terms pages. Both pages name the operator and a contact, state that the site is for ages 13 and up and is not directed at children under 13, and use plain language with no bracketed placeholders.

## Plan

1. Add Privacy and Terms links to the bottom row of `site/src/components/SiteFooter.astro`, with a 44px minimum tap height.
2. Add `site/src/pages/privacy.astro` and `site/src/pages/terms.astro`. Name the operator as the CubeSat Classroom volunteer project and use the public GitHub repository as the contact path for general questions, until the project lead posts a different operator name or address. Requests about a particular person, including deleting a child's information, go to the private project mailbox (`privacyEmail` in `site/src/hubs.mjs`), never a public issue or a security form.
3. On both pages, say the site is for learners 13 and up and is not directed at children under 13, and that we do not knowingly collect personal information from anyone under 13.
4. List the new paths in `site/src/pages/sitemap.xml.ts`, and add `privacy` and `terms` to the copy list in `site/scripts/publish-github.mjs` so the GitHub Pages `docs/` mirror includes them.
5. Run `npm test` and `npm run build` in `site/`, then `npm run build:github` so `docs/` stays in sync. Do not add or modify `.github/workflows`.

### How to check

- [ ] The footer bottom row on the built site links to `/privacy/` and `/terms/`.
- [ ] Both pages name an operator and a contact, state the 13-and-up / under-13 rule, and contain no `[BRACKETED]` placeholders.
- [ ] The privacy page sends deletion and other personal requests to a private channel, not a public issue.
- [ ] `npm test` in `site/` passes, `npm run build` includes the two new pages, and `docs/` matches `npm run build:github`.
- [ ] `quest --root . check` exits 0, and the pull request was not merged automatically.
