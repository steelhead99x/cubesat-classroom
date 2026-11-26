# Rideshare FlatSat: ground-test fork

A fork of the [rideshare payload integration lab](../rideshare-flatsat/README.md) for ground testing. One logger node from that lab runs on its own power: a [1U solar face](../solar-face/README.md) charges a cell through a solar charger, and an INA226 records every milliamp in and out. The lab's other evidence (the onboard SD log, the ground feed, the review worksheet) carries over unchanged.

> **Status: started.** The test plan and the desk tools (`ground.py`, with tests) exist. No node has been run on solar power yet. Every number below is a desk simulation from the cell makers' figures and an estimated node current.

## What's on the bench

| Part | From | Notes |
| --- | --- | --- |
| One logger node | The rideshare lab | Either the all-in-one XIAO carrier or the paired MG24 Sense + ESP32S3 Sense. Same firmware and log format as the lab |
| 1U solar face, silicon (`si`) | [hardware/solar-face](../solar-face/README.md) | Its land pattern still has to be verified before boards are ordered |
| Solar charger board | New | Rated for at least 6.5 V input. A mentor picks it, checks it, and wires the cell |
| Protected LiPo cell, about 1000 mAh | New | Protected cells only. A mentor checks the cell and its polarity first |
| INA226 breakout and logger sketch | [Receiver bench kit](../sdr-receiver/bench/README.md) | In the cell's lead, wired so charging reads positive. Record with `power_log.py record` |

Never leave a charging LiPo where nobody can check on it. Run long tests somewhere a mentor approves, on a non-flammable surface.

## Ground tests

| Test | What you do | What you learn | Worksheet row |
| --- | --- | --- | --- |
| 1. Energy balance | Measure the node's average current, then run `ground.py simulate --use-ma <yours>` to predict | Whether the face can keep up, and with how much sun | §7 electrical |
| 2. Day and night | Light the face, then cover it, for set periods. Record with the INA226. Run `ground.py report power.csv` | Charge in and out per period, and the real break-even lit fraction | §§6.1–6.7 verification, §7 |
| 3. Brown-out and recovery | Let the cell run down until the node stops, then light the face | Does the node restart on its own? Does the SD log keep its earlier rows? `lab.py compare` finds any gaps in sequence numbers | §§6.1–6.7 |
| 4. Tumble | Turn the node and face slowly in sunlight, tapping the table at the start to mark a shared event | The charge current follows the IMU's angle to the Sun: attitude from power, as some real CubeSats do | §2, Appendix C (axes) |
| 5. Endurance | Leave it running for days | Resets, gaps, clock drift, and the cell's lowest voltage | §§6.1–6.7 |

Label each result "hardware bench". Keep the raw CSV files with their `.json` notes, like the lab's other evidence.

## Try it without hardware

```sh
python3 ground.py simulate run-1
python3 ground.py report run-1/power.csv
python3 -m unittest test_ground
```

The default run is desk simulation: a silicon face in 800 W/m² of sunlight, a node drawing 60 mA (an estimate: measure yours), and four cycles of 45 minutes lit and 45 dark. The face puts about 124 mA into the cell while lit, so the cell gains about 13 mAh over six hours. It breaks even when lit about 48% of the time. Change `--use-ma`, `--sun-w-m2`, `--face` and the periods to see what matters.

`ground.py` reads the INA226 logger's CSV (`t_ms,current_ma,bus_v,power_mw`), so a real run goes straight into `report`.

## What it doesn't cover

This is ground testing of one node's power. Flight acceptance for a rideshare still needs everything the lab's [review worksheet](../rideshare-flatsat/README.md#rideshare-review-worksheet) leaves open: the integrator's ICD, the provider's test levels, a safety review and the authorizations.

## Next steps

- Measure the real current of each node configuration, and replace the 60 mA estimate.
- Pick and check a solar charger board, and add it here with its input range.
- Run test 2 and record the first hardware bench numbers.
- The nano FlatSat has a matching fork in the rockoon repository (the 1U bench FlatSat), with the same solar figures in Rust.
