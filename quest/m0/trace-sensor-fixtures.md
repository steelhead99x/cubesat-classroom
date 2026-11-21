# [S] Trace a sensor value from file to display

## Goal

Show how a known input becomes a displayed reading, and check what the station does when an input goes missing.

## Plan

1. Run these commands from the repository. The three files are test fixtures.

   ```sh
   cubesat_fixture_dir=$(mktemp -d)
   printf '%s\n' -1250 > "$cubesat_fixture_dir/temperature_mc"
   printf '%s\n' 4100 > "$cubesat_fixture_dir/voltage_mv"
   printf '%s\n' 375 > "$cubesat_fixture_dir/light_lux"
   printf 'Fixture folder: %s\n' "$cubesat_fixture_dir"
   cargo run --locked -- classroom --sensor-dir "$cubesat_fixture_dir"
   ```

2. Open the printed address and request a reading. Record the display and check it against −1.25 °C, 4.10 V, and 375 lx. The display may round. Record its source label too.
3. From another terminal, use the printed fixture folder. Rename only its temporary `light_lux` file and request another reading. Verify that the station reports a failure and does not present a new sample as fresh. Restore the file and ask again.
4. Share a three-row table with the baseline, missing-input and recovery results. Include the exact fixture values and label the run as a software test.
