# [M] Align the public site copy with the volunteer review

## Goal

The public `cubesat-classroom` site and README no longer present an unapproved $19.99 Crew Plus offer, an unapproved Teachers in Space Givebutter fundraiser, unapproved adult phone sign-in, or settled yearly workshop counts until the project lead and partners agree.

## Plan

1. Replace the `$19.99` Crew Plus price with `TBD` and remove `New quest packs first` in `site/src/pages/plans.astro` (and `docs/assistant-plans.md`).
2. Hold the unapproved Givebutter donate links from `site/src/hubs.mjs`, `site/src/components/Workshops.astro`, `site/src/pages/index.astro`, and `site/src/pages/volunteer.astro`.
3. Soften workshop wording in `site/src/components/Workshops.astro` and `site/src/pages/plans.astro` so it reads as proposed if partners agree, without a fixed three-a-year promise.
4. Use generic independent wording in `site/src/components/SiteFooter.astro`, `README.md`, and `CONTRIBUTING.md`, and remove the unapproved phone-number sign-in paragraph from `README.md`.
5. Rebuild the static site and GitHub Pages `docs/` copy (`npm run build` and `npm run build:github`).

### How to check

- [ ] `site/src/pages/plans.astro` no longer shows `$19.99` or `"New quest packs first"`.
- [ ] Unapproved Givebutter donate links are removed from `site/src/`.
- [ ] `README.md` no longer advertises phone-number sign-in.
- [ ] `npm run build` and `npm run build:github` in `site/` succeed with `docs/` up to date.
- [ ] Open decisions (Crew Plus price, TIS attribution/fundraiser, adult phone sign-in) are flagged for the project lead.
