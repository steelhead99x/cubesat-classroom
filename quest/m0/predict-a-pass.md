# [S] Predict a satellite pass

## Goal

Find when a real amateur satellite will be above your school, then check the prediction. Label it as a desk simulation until someone hears the satellite.

## Plan

1. Download fresh orbit data from [CelesTrak's amateur radio group](https://celestrak.org/NORAD/elements/gp.php?GROUP=amateur&FORMAT=tle) and save it as `amateur.tle`. Write down the download date: predictions get worse as the data ages.
2. Look up your town's latitude and longitude to two decimal places. Use the town, not a home address.
3. Run the prediction for the next day and pick one satellite, such as the ISS:

   ```sh
   cargo run --locked -- passes --tle amateur.tle --satellite ISS --lat 39.95 --lon -75.16 --hours 24
   ```

4. Choose the pass with the highest elevation. Convert its UTC times to your local time and note where to look at rise and set.
5. Check your work: compare the rise time and highest elevation with another tracker, such as Gpredict or an online pass list. Set it to the same location, UTC and a 0° horizon so you compare like with like. Record the difference in seconds and suggest one reason for it.
6. Share the crew nickname, data download date, satellite, predicted times and the comparison. If a licensed operator or a receive-only station heard the pass, add that as a separate "received RF" result.

## Levels

- **Beginner:** find one pass and say which direction to face.
- **Intermediate:** explain why a pass that climbs to 60° is easier to hear than one that reaches 12°.
- **Professional:** add `--json`, plot elevation against time for three passes, and explain how element age affects the error.
