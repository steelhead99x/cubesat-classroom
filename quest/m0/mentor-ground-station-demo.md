# [S] Mentor: guide a first ground station demo

## Goal

A mentor guides a small group through asking a simulated satellite for a reading, receiving pictures and recovering from a lost signal. Every learner presses the buttons and writes down a result.

## Plan

Follow the [mentor rules](/quest/m0/mentor-get-ready.md#mentor-rules): stay the whole time, keep your login to yourself, and let every learner do each step.

**Time:** 30 minutes. **Group:** up to 6 learners per mentor. **Bring:** one computer per pair, paper and pencils. No login, no internet needed for the local station.

1. **Set up (mentor, 5 min).** Start the station with `cargo run --locked -- classroom` on each computer and open the address it prints. Say: "This computer is our ground station. A pretend satellite is running inside it."
2. **First reading (everyone, 5 min).** Each learner presses **Get sensor reading** once and writes down the temperature with its unit. Ask: "What do the letters after the number mean?"
3. **Pictures (everyone, 5 min).** Each pair presses **Start video** and counts the frames out loud. Ask: "Did all eight arrive?"
4. **Lose the signal (everyone, 5 min).** One learner presses **Pause contact** while pictures are arriving; their partner presses **Restore contact** and asks for a new reading. Swap roles. Ask: "Which readings are new, and which are old?"
5. **Talk it over (everyone, 5 min).** Each learner says one thing they measured and one thing they would test next. Do not answer for them: ask a question that helps instead.
6. **Pack up (mentor, 5 min).** Stop each station with Ctrl+C. Write a short mentor log: date, crew nickname, how many learners did every step, and one instruction to improve.

### Levels

- **Beginner:** the reading, the unit and "old or new?"
- **Intermediate:** time three readings and compare round trips.
- **Professional:** repeat the steps in the [browser practice station](https://cubesatclassroom.org/#demo), open **Raw telemetry** and match `sequence` and `captured_unix_ms` to the mission log.

### Check your work

Every learner did steps 2 to 5 themselves, and your log says so. Label results "desk simulation".

## Required

- [Mentor: get ready to guide a demo](/quest/m0/mentor-get-ready.md) - the mentor has run the station and agreed to the rules.
