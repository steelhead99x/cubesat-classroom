---
title: Fresh or historical?
patch: Fresh data
order: 4
summary: Find out when a reading stops counting as current, and why the station cares.
minutes: 5
skill: Data quality
beginner:
  steps:
    - Press **Ask for a reading** and look at the line under the numbers.
    - Wait and watch. When does **Fresh** change to **Historical**?
    - Pause contact. What does the line say now?
  check: You found how many seconds a reading stays fresh.
  discuss: Why is old information sometimes worse than no information?
intermediate:
  steps:
    - Get a reading and record the age at which the label changes to **Historical**.
    - Repeat with contact paused and compare.
    - Write one rule the next crew can use to decide whether a reading is current.
  check: Your rule names the 5 second limit and what happens while contact is paused.
  discuss: Would a 5 second limit work for a weather balloon logging once a minute? What would you choose?
professional:
  steps:
    - Get a reading and compare `captured_unix_ms` in **Raw telemetry** with the moment the label changes.
    - Confirm the threshold matches `MAX_AGE` in the station source (5000 ms).
    - Read `Sample::age_ms` in `src/main.rs` and describe what the station does with a sample whose capture time is ahead of the receiver clock.
  check: You confirmed the threshold and explained the future-timestamp rejection.
  discuss: How would you add clock uncertainty to the sample format so the ground can reason about it?
---
