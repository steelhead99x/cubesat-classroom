# [S] Mentor: guide a satellite pass prediction

## Goal

A mentor guides a small group to predict when a real satellite will be overhead, then go outside or compare with another tracker to check it. Every learner finds one pass and says which way to face.

## Plan

Follow the [mentor rules](/quest/m0/mentor-get-ready.md#mentor-rules). Use the town, never a home address, for the location.

**Time:** 30 minutes, plus the pass if you watch it. **Group:** up to 6 learners per mentor. **Bring:** a computer with the station installed, fresh orbit data from [CelesTrak's amateur group](https://celestrak.org/NORAD/elements/gp.php?GROUP=amateur&FORMAT=tle) saved as `amateur.tle`, and a compass or phone compass. No login needed.

1. **Explain (mentor, 5 min).** Say: "Satellites move in predictable paths. We can work out when one will rise over our town."
2. **Predict (learners take turns, 10 min).** One learner types:

   ```sh
   cargo run --locked -- passes --tle amateur.tle --satellite ISS --lat 39.95 --lon -75.16 --hours 24
   ```

   using the town's latitude and longitude. Each learner reads one row aloud: rise time, direction and highest point.
3. **Convert (everyone, 5 min).** Each learner turns one UTC time into local time and points the compass where the satellite will rise.
4. **Check (everyone, 10 min).** Compare one pass with another tracker set to the same place and a 0° horizon. Each learner writes down the difference in seconds and one reason it might differ.
5. **Optional watch.** If the pass is after dark and high, go outside together at the time. The mentor stays with the group the whole time.
6. **Log (mentor).** Record the data download date, the satellite, how many learners did every step, and whether anyone saw the pass. Label predictions "desk simulation" and sightings separately.

### Check your work

Every learner found a pass, converted a time and pointed the right way.

## Required

- [Mentor: get ready to guide a demo](/quest/m0/mentor-get-ready.md) - the mentor has agreed to the rules.
- [Predict a satellite pass](/quest/m0/predict-a-pass.md) - the mentor has made a prediction on their own first.
