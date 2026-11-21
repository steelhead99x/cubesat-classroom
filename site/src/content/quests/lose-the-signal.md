---
title: Lose the signal on purpose
patch: Signal lost
order: 3
summary: Pause contact in the middle of a video burst and see what the ground keeps.
minutes: 10
skill: Communication loss
beginner:
  steps:
    - Press **Start video**.
    - After two or three pictures, press **Pause contact**. This is like the satellite flying out of sight.
    - Look at the mission log. How many pictures made it?
    - Press **Restore contact**, then ask for a new reading.
  check: You know how many pictures arrived before contact stopped, and what came back afterwards.
  discuss: A real satellite is only overhead for a few minutes. What would you do first when it comes back into view?
intermediate:
  steps:
    - Start a video burst and pause contact after two or three frames.
    - Record how many frames arrived and how the station marks the burst.
    - Restore contact and send a sensors request. Record how long it takes to recover.
  check: Your log shows a partial burst marked incomplete and a successful request after recovery.
  discuss: Should the ground station throw away an incomplete burst or keep it? Give one reason each way.
professional:
  steps:
    - Start a burst and pause contact mid-group. Note the last frame index received before the group stalls.
    - Restore contact and issue a new request. Confirm the new reply carries a new request ID.
    - Explain why the station deduplicates request IDs and expires commands after 5 seconds.
  check: You identified the stall point, the recovery request ID and the purpose of the 5 s expiry.
  discuss: The real station rejects commands older than 5 s. What could go wrong if a delayed command ran after contact returned?
---
