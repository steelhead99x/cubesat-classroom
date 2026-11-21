# [S] Wildfires: get a message out by satellite

## Goal

Work out when a fire crew far from phone service could pass a short message through an amateur radio satellite. Label the result a desk simulation.

## Background

Two kinds of satellite help with wildfires:

- **Spotting.** NASA and NOAA weather satellites see the heat of a fire. NASA's free [FIRMS fire map](https://firms.modaps.eosdis.nasa.gov) shows the hot spots, usually within about three hours.
- **Talking.** Fires burn phone towers and power lines. Licensed amateur radio volunteers can pass short messages by radio, and an amateur satellite can carry a message over mountains while it is overhead.

This quest is the "talking" part. Only a licensed operator may transmit. Students plan and listen.

## Plan

1. Pick a pretend fire camp: a town or park near you. Use the place, not a home address.
2. Download current orbit data from [CelesTrak's amateur radio group](https://celestrak.org/NORAD/elements/gp.php?GROUP=amateur&FORMAT=tle) and save it as `amateur.tle`.
3. Find the passes over the camp for the next 12 hours:

   ```sh
   cargo run --locked -- passes --tle amateur.tle --satellite ISS --lat 39.95 --lon -75.16 --hours 12
   ```

4. Choose the best pass. Write a message a crew could send in that time, in 20 words or fewer: where they are, what they need.
5. Check your work: you have a pass time in UTC and local time, how many minutes it lasts, and a message short enough to say twice in that time.
6. Share the crew nickname, the camp, the pass and your message.

## Talk it over

A pass lasts only a few minutes and the next may be hours away. What should the crew do in between?
