# [XL] Give Rockoon Battle rooms one authoritative history

## Goal

A returning crew can reload or reconnect and keep its seat. Every browser gets the same committed turns and purchases, regardless of request order, dropped responses, or how long it was offline.

## Plan

- Replace the browser committer and stand-in with one SQLite-backed Durable Object per room. Keep the deterministic game rules shared with play on one computer.
- Give each seat a server-issued credential. Serialize lobby starts, shots, timeouts and purchases, and reject requests for a phase that has already closed.
- Prove that a stale shot or purchase cannot replace settled history. Check reload, interrupted requests, deadlines, and two independent rooms in a local Worker.
- Exercise two browsers against a preview before updating production. Keep the old relay namespace available for cached clients during rollout.

The browser handoff attempt in [PR 123](https://github.com/steelhead99x/cubesat-classroom/pull/123) could elect two writers. The decision recorded in [PR 117](https://github.com/steelhead99x/cubesat-classroom/pull/117) is resolved by choosing Worker authority.

### Verification

- The room tests cover stale purchases, a late shot after a timeout, a returning creator, room expiry, and a request from another room.
- Two separate Chromium sessions played a shot through local Wrangler. Both reached the same match fingerprint. A reload restored the player's seat and match state. Shutting down and restarting Wrangler preserved the history in SQLite.
- The remaining release check is the same flow on a Cloudflare preview with its own room storage, followed by deployment approval.
