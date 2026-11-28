# [M] Replay a stand-in's turn after the stand-in leaves

## Goal

In Rockoon Battle, every client can replay a match's full history after a stand-in who played someone else's turn leaves, with that entry keeping its original authority. The fix lands on [PR 115](https://github.com/steelhead99x/cubesat-classroom/pull/115) (`claude/mirror-keeps-standin-authority`), not on a new branch.

## Plan

Review of PR 115 at `dbb0623` found a High stall: in a three-player match, the returning creator plays a stand-in's skip and then records another turn. After the stand-in leaves, the third player can replay only two of the four completed turns, because the only copy of the stand-in's entry left with the stand-in.

Commit `8968c9b` on PR 115 copies a stand-in's entry with its author attached and adds the three-player reproduction as a test. This quest checks that fix rather than rewriting it:

1. Fetch PR 115 and read its latest commits first. Another session has been pushing to it; never force-push, and rebase any change onto the current head.
2. Run the three-player test it adds and confirm it fails on `dbb0623` and passes on the head.
3. Confirm a mirrored entry still cannot be republished as the creator's own record, the property PR 115 exists for.
4. Only if a gap remains, push a fix to the PR 115 branch.

The quest finishes when PR 115 merges with the stall fixed; that PR, or a follow-up, deletes this file.
