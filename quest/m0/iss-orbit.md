# [S] Find the ISS: orbit, passes and deployment

## Goal

Use real orbit data to work out how the International Space Station moves, when it passes your town, and how a CubeSat gets from the station into its own orbit. Label the result a desk simulation.

## Plan

1. Download current orbit data from [CelesTrak's space stations group](https://celestrak.org/NORAD/elements/gp.php?GROUP=stations&FORMAT=tle) and save it as `stations.tle`. Write down the date.
2. Read the line of the ISS entry that starts with `2`. After the `2` comes a five-digit catalogue number, and the next number is the tilt of the orbit in degrees. The last block of digits on the line begins with the orbits per day, such as `15.49`. Work out the minutes for one orbit: 1440 ÷ orbits per day.
3. Predict passes for a large town or city near you. Don't use a home or school address. Replace the example latitude and longitude:

   ```sh
   cargo run --locked -- passes --tle stations.tle --satellite "ISS (ZARYA)" --lat 39.95 --lon -75.16 --hours 48
   ```

4. Pick the highest pass. Convert it to local time and note where to look.
5. Read the first table of the [ISS deployment review](/docs/iss-deployment-review.md). Write down the size of a 1U CubeSat and two things a deployer checks before a satellite may ride as station cargo.
6. Check your work: one orbit should take a little over 90 minutes, and the tilt should be near 51.6°. If your town's latitude is higher than the tilt, explain why the ISS never passes directly overhead.
7. Share the crew nickname, the data date, your orbit time and the pass you chose. If someone saw or heard the pass, add that as a separate observation.

## Levels

- **Beginner:** say how many times the ISS circles Earth in one school day.
- **Intermediate:** explain why passes come in groups about 90 minutes apart, then stop for hours.
- **Professional:** a CubeSat released from the ISS starts in nearly the same orbit. Explain why it drifts away and why its orbit decays faster than the station's, then name the review item that covers end of life.
