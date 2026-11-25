# [S] Make classroom quest check pass on the existing lesson files

## Goal

`quest --root . check` (moq-dev/quest 0.23.1) exits 0 on `steelhead99x/cubesat-classroom`, including the pre-existing lesson files in `quest/m0/`.

## Plan

1. Run `quest --root . check` and verify no unknown `##` headings remain across `quest/`.
2. Keep all lesson sections intact as `###` subsections under `## Plan` (`### Background`, `### Steps`, `### Levels`, `### Try it with your own pet`, `### Buying a cover`, `### Talk it over`).
3. Re-run `quest --root . check` and confirm it exits 0.

### How to check

- [ ] `quest --root . check` prints a clean pass with exit code 0.
- [ ] All pre-existing lesson content remains intact under allowed headings.
- [ ] A pull request is open and has not been merged automatically.

