# [S] Turn every site feature into a lesson

## Goal

A learner can reach every current feature of the public site and cubesat.app from a browser lesson: the practice ground station, Catch the Pass, Rockoon Battle, the CSC-R1P pedal trainer and its pass mode, ISS pass prediction, the Bands rule finder and the rocket flight workbook. Each lesson has three levels, a check and a discussion question, labels its results as desk simulation, and has a matching hint and "why it matters" in the quest guide.

## Plan

1. Add an optional `tool` (name, link, note) to the lesson schema in `site/src/content.config.ts`. A lesson with a tool shows an "Open" panel, first on phones, instead of the station demo.
2. Add six lessons in `site/src/content/quests/`, a patch colour in `site/src/patches.ts` and an icon in `site/src/components/Patch.astro` for each.
3. Add their hints and reasons to `site/src/guide/answers.mjs`, plus general answers on simulated versus real data and on Doppler shift.
4. Build the lesson lists on `/` and `/quests/` from the collection, so the count stays right.
5. Put "Try it in the browser first" at the top of `quest/README.md` and move the site maintenance quests to the end.
6. Do not add or modify `.github/workflows`.

### How to check

- [ ] `npm test` in `site/` passes, including `tests/lessons.test.mjs` (every lesson has guide answers, a patch colour and an icon, and the order numbers have no gaps).
- [ ] `npm run build` and `npm run audit:a11y` in `site/` pass.
- [ ] Each tool link opens the right page, and on a 390px phone the tool panel comes before the steps.
- [ ] Every step matches the control names on the page it sends you to.
- [ ] `quest --root . check` exits 0.
