# [XS] Resume a Rockoon seat on reload without pressing Join

## Goal

Reloading a tab that holds a saved seat for the room returns the player to their seat, either automatically or through one clear Resume match action, instead of the prefilled room screen. The fix lands on [PR 114](https://github.com/steelhead99x/cubesat-classroom/pull/114) (`fix/rockoon-seat-reload`).

## Plan

Review of PR 114 at `708870a` found that after a reload Chrome showed the prefilled room screen and made no token request until Join was pressed.

Commit `abcede4` on PR 114 resumes without Join. This quest verifies it on the current head:

1. Fetch PR 114 and read its latest commits first; never force-push.
2. In Chrome, join a match, reload, and confirm a token request goes out and the seat resumes with no click.
3. Confirm a tab with no saved seat, or a seat for a different room, still shows the normal join screen.
4. Only if a gap remains, push a fix to the PR 114 branch.

## Related

- [Let only one tab hold a restored Rockoon seat](/quest/m0/rockoon-one-tab-per-seat.md) - the same PR and the same seat restore code
