# [S] Let only one tab hold a restored Rockoon seat

## Goal

Two same-origin tabs never both resume the same Rockoon Battle seat, so two clients never publish under one broadcast name. The fix lands on [PR 114](https://github.com/steelhead99x/cubesat-classroom/pull/114) (`fix/rockoon-seat-reload`).

## Plan

Review of PR 114 at `708870a` found that a tab opened from the game inherited the saved seat ticket from session storage and requested the original tab's player ID.

Commit `abcede4` on PR 114 adds a one-tab-per-seat check. This quest verifies it on the current head:

1. Fetch PR 114 and read its latest commits first; never force-push, and rebase onto the current head.
2. In Chrome, join a match, then open a second tab from the game page (which copies session storage). Confirm only one tab resumes the seat and the other falls back to join or watch.
3. Close the first tab and confirm the second can then resume.
4. Note that relay behavior with a duplicate broadcast name stays unverified unless a live relay run is possible.
5. Only if a gap remains, push a fix to the PR 114 branch.
