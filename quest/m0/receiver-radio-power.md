# [M] Compare radio power: 2.4 GHz, 5 GHz and HaLow

## Goal

Measure how much energy each radio link uses to deliver the same stream, so a crew can pick one for a battery or solar station. This is the power comparison in the [Rev B plan](/hardware/sdr-receiver/rev-b-pedal-plan.md#power-comparison-24-ghz-vs-5-ghz-vs-halow), done on XIAO boards before the pedal exists. Label results "hardware bench".

## Plan

**Time:** two sessions of 60 minutes. **Bring:**
- an INA226 current-sensor breakout with a 0.1 Ω shunt (a few dollars), or a Nordic PPK2 if you have one
- a spare XIAO or Arduino to read it
- a XIAO ESP32-S3, a XIAO ESP32-C5, and the Seeed Wi-Fi HaLow board pair from the [HaLow noise quest](/quest/m0/receiver-halow-noise.md)
- a 5 V USB supply and jumper wires

A mentor checks the wiring before anything is powered.

### Steps

1. **Wire the meter.** The 5 V supply goes to INA226 IN+. IN− goes to the board under test's 5 V pin. All grounds are joined. The INA226 talks to the logger board over I²C.
2. **Upload the logger.** Upload [power-logger.ino](/hardware/sdr-receiver/bench/power-logger/power-logger.ino) to the logger board, then check it reads a steady current with the board under test idle.
3. **Record each case for 10 minutes:**

   ```sh
   python3 power_log.py record --port /dev/ttyACM0 --minutes 10 --label "S3, 2.4 GHz, streaming" --out s3-24-stream.csv
   ```

   | Board under test | Link | States to record |
   | --- | --- | --- |
   | XIAO ESP32-S3 | 2.4 GHz Wi-Fi | radio off, connected and idle, streaming |
   | XIAO ESP32-C5 | 2.4 GHz, then 5 GHz Wi-Fi | the same |
   | XIAO ESP32-S3 + HaLow board, Wi-Fi off | HaLow at 1, 2 and 4 MHz bandwidth | the same |

4. **Same stream every time.** Send the same thing at the same rate on every link. At the receiving end, write down how many megabytes arrived during the 10 minutes.
5. **Summarise.** Each run delivers its own amount of data, so give each file its own figure:

   ```sh
   python3 power_log.py summary s3-24-stream.csv --megabytes 180
   python3 power_log.py summary s3-24-stream.csv c5-5g-stream.csv \
       --delivered s3-24-stream.csv=180 --delivered c5-5g-stream.csv=210
   ```

   One `--megabytes` figure for several files is refused. With a PPK2, export CSV and add `--volts 5`.
6. Repeat every streaming case three times, on a different day if you can. Share a table of the averages and the spread.

### Check your work

- The INA226 averages over about 35 ms, so it gives good averages but can't show the shape of each radio burst. A PPK2 can.
- The HaLow case includes the XIAO running the HaLow link. That's fair, because a node needs both. Note it in your table.
- Distance and antennas stayed the same for every link. Write them down.

### Levels

- **Beginner:** which link has the lowest average power when idle?
- **Intermediate:** compute energy per megabyte for each link. Is the lowest-power link also the most efficient one?
- **Professional:** move the far end to two longer distances and repeat. Report where each link's energy per megabyte starts climbing as it retries and drops its rate.

### Talk it over

- Why measure the radio's own feed and not the whole station?
- A solar station has 5 Wh a day for its radio. How long could each link stream?

## Required

- [Record a XIAO bench sample](/quest/m0/xiao-first-reading.md) - the crew can upload a sketch and read serial output.
