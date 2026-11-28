# [XL] Settle a returning creator's history before it commits

## Goal

In Rockoon Battle, a creator who comes back to a match (after a reload or a dropped connection) never commits a turn or shop that a stand-in already settled while it was away, whatever order the other players' snapshots arrive in. Peers never have to replay a round because of it.

## Plan

Review of [PR 114](https://github.com/steelhead99x/cubesat-classroom/pull/114) at `c191c1e` reproduced two orderings with browser and relay stubs:

1. The stand-in's first snapshot arrives with no history, which marks that peer as met (`metSinceLive` in `site/src/games/rockoon-wars/game.ts`). Its committed shop arrives later, after the catch-up window, and no longer reopens catch-up. The creator presses Done and replaces the shop.
2. The creator submits picks before the stand-in's first snapshot arrives at all. When it arrives with a committed shop, the creator, as committer, records its own picks and peers replay the round.

The same race exists on main for a creator whose connection drops and comes back with the same id; PR 114 makes it more common because a reload now keeps the seat.

- Track new history per peer rather than only the first snapshot after going live, so any snapshot carrying unplayed `log` or `shopLog` reopens catch-up without heartbeats extending it forever.
- Treat a returning committer's own picks as provisional until every player in the order has been heard from since it went live, or has been judged gone, then commit.
- Add room.mjs tests for both orderings.

A first attempt ([PR 123](https://github.com/steelhead99x/cubesat-classroom/pull/123), draft) gated commits on hearing every peer and tracked new history per peer. Review showed that gating on "heard from" is not enough. An older snapshot, a short reconnect the peers saw as a long gap, or two returning players can each still release a stale commit. The patch also added regressions: divergence checks and replay picked different histories, a wrong-match heartbeat blocked the committer, and catch-up reopened during normal play. Snapshot timing can't settle this. It needs an explicit handoff in the room protocol, such as a committer epoch that each snapshot carries and a returning committer must see acknowledged before it records.

A second attempt on the same PR (`dafd679`, protocol v7) added a committer claim epoch and an explicit `follows` acknowledgement. Review still reproduced High findings. When each side's presence timeout calls the other gone, both write (split brain). An older relay session's snapshot can replace a newer acknowledgement. Settled history that survives only as a marked copy can't be replayed. A peer that keeps heartbeating without acknowledging blocks the match. Peer-to-peer snapshots with local timeouts can't give one agreed writer. That needs a shared authority.

## Waiting

- Maintainer: where Rockoon Battle rooms get one agreed writer
  - Make the room authoritative on the Worker (Recommended) - a Durable Object per room orders requests and records history, and clients only replay it, so no handoff race exists; a larger change to the Worker and the client
  - Keep peer-to-peer rooms and accept rare replays - close PR 123; main keeps working, and a returning creator can occasionally overwrite a settled shop
