---
title: Predict a space station pass
patch: Sky watcher
order: 9
summary: Use orbit data to find when the International Space Station next crosses your sky, and where to look.
minutes: 10
skill: Orbits and timing
tool:
  name: Passes on cubesat.app
  href: https://cubesat.app/learn/passes
  note: Predictions from orbit data, not observations. Use the coordinates of a nearby town centre or landmark, not your home. The app keeps them only until it closes.
beginner:
  steps:
    - Open Passes. Type the latitude and longitude of a nearby town centre, then press **Use this location**.
    - Press the button to download orbit data for the space station.
    - Look at the next pass. Write down when it rises, which direction it rises from, and how high it gets.
  check: You have a time, a compass direction and a highest angle in degrees for one pass.
  discuss: If a pass is just after sunset, you might see the station as a bright dot moving across the sky. Why can't you see it in the middle of the day?
intermediate:
  steps:
    - Copy the passes for the next 48 hours into a table with rise time, highest angle and duration in minutes.
    - Find the time between the start of one pass and the start of the next.
    - Look for a pattern between highest angle and duration.
  check: Your table has every pass listed, and you have written the pattern in one sentence.
  discuss: The station goes around Earth about every 92 minutes. Why don't you get a pass every 92 minutes?
professional:
  steps:
    - Note the age of the orbit data the app shows. Then refresh it and see whether the times move.
    - On a computer with the ground station, run `cargo run --locked -- passes --tle amateur.tle --satellite ISS --lat <lat> --lon <lon> --hours 48` with the same coordinates (see `quest/m0/predict-a-pass.md` for getting the orbit file).
    - Compare the two lists, pass by pass, in seconds.
  check: You have a side-by-side table with the difference for each pass and a reason for any difference over a minute.
  discuss: Both tools use SGP4. List what could still make them disagree, and what could make both of them wrong.
---
