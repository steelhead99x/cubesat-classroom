# Weather balloon lab

**Question.** How do air pressure and temperature change during a balloon flight? How do we know if part of the record is missing?

The ground lab uses the same timestamped file format the proposed balloon experiment would return. It works with a pressure sensor and a thermometer, or with two fixture files for a class without hardware. The first class does the work on a bench. A flight is a separate activity led by experienced adults.

## Build and rehearse on the ground

Give the class four jobs: payload lead, station operator, data recorder and reviewer. For younger grades, read values and units aloud and mark what changed. For older grades, graph both series against elapsed time and explain timing, units, calibration and missing points.

1. Start the [classroom ground station](../README.md). Get a reading, start the video, then practice a contact interruption and download the crew log. The station demonstrates communications and separately labels its simulated camera.
2. Connect an instructor-approved pressure sensor and thermometer using their supplied adapter or driver. Expose current integer values in two files, `pressure_pa` and `temperature_mc`, in one folder. Pressure is in pascals and temperature is in millidegrees Celsius. Check units against the instrument documentation. A local reference reading and a note on placement help the next group interpret the data.
3. Run the logger for one minute. It writes once per second, with a sequence, local UTC timestamp, elapsed time and data status. Observe the live instrument while it records. If one reading fails, its cell stays empty and the other reading remains available.

   ```sh
   cargo run --locked -- balloon-log --sensor-dir ./sensors --output ground-run.csv --samples 60
   ```

4. Repeat with a new output file. During this second test disconnect a **fixture input** or stop its adapter briefly under instructor supervision. Mark that time in the crew log, reconnect it, and verify the CSV records loss and recovery. If only software is available, create ordinary fixture files using the example below. Label their data as a simulation.
5. Open the CSV in a spreadsheet. Plot `pressure_pa / 100` as hPa and `temperature_mc / 1000` as °C. Keep missing values as gaps. Compare the shape with the May 18, 2026 Teachers in Space high-altitude balloon dataset and distinguish the bench and flight conditions. [Teachers in Space flight-data archive](https://www.tis.org/flightdata)

```sh
mkdir -p run/balloon-fixture
printf '101325\n' > run/balloon-fixture/pressure_pa
printf '21500\n' > run/balloon-fixture/temperature_mc
cargo run --locked -- balloon-log --sensor-dir run/balloon-fixture --output run/fixture.csv --samples 10
```

A passing ground rehearsal returns the planned sample count, explicit gaps for the deliberately disconnected input, and a new valid reading after it returns. The reviewed worksheet should include hardware or fixture provenance, units, run date, a chart, and one limitation. The current station reads temperature, voltage and light on command. The balloon logger records pressure and temperature independently of that link.

## Plan the flight hardware

These are planning figures for a first flight with an experienced launch partner. Prices are US prices as shown on the pages listed at the end of this guide in November 2026; confirm them before buying. Power and weight figures marked "estimate" have no measurement behind them yet: weigh and meter your own parts, then update the budget.

### Payload options

**A. Sensor logger (start here).** A Seeed XIAO ESP32-C3 with Wi-Fi off, a BME280 pressure and temperature sensor, a u-blox MAX-M10S GPS and a microSD card. It records the same `pressure_pa` and `temperature_mc` values as the ground lab, plus position. The [classroom sensor sketch](../firmware/classroom-sensors/classroom-sensors.ino) and `sensor-bridge` are the bench version of this logger. The flight version must log to the card on its own, with no laptop. Set the GPS to its airborne mode before flight: in the default mode many GPS modules stop reporting position at about 12 km (40,000 ft), far below burst altitude.

**B. Camera logger (add later).** A Raspberry Pi Zero 2 W with a camera module takes timed pictures. Fly it beside payload A, not instead of it, so a camera fault never costs the data record.

**Tracker (always).** Recovery depends on a GPS tracker that reports its position by radio. Common balloon trackers such as LightAPRS use amateur frequencies and need a licensed operator. If the team has no licensed operator, the launch partner provides and runs the tracker. Without working tracking there is no flight.

### Parts and estimated costs

| Item | Notes | Approx. price |
| --- | --- | --- |
| 600 g latex balloon (Totex/Kaymont) | Carries 1–2 lb; usually bursts around 75,000–105,000 ft | $70 |
| Helium, balloon grade | About 69 cu ft for a 600 g balloon; mixed helium bursts lower, so ask for pure | $100–250 (estimate; call a local gas supplier) |
| Parachute, about 36 in, plus shock cord | Keeps the parachute clear of the burst balloon | $40–80 (estimate) |
| Payload A: XIAO ESP32-C3, BME280, MAX-M10S breakout, microSD board | Sensor logger | $55–85 (XIAO $5; others estimates) |
| Four Energizer Ultimate Lithium AA | Rated −40 °C to 60 °C | $10–15 (estimate) |
| Insulated foam box, tape, bright paint, contact label | Holds and protects everything | $10–20 (estimate) |
| GPS tracker (LightAPRS) | Licensed operator required | $115 |
| **Total for payload A** | | **about $400–650** |
| Payload B add-on: Pi Zero 2 W, camera, eight lithium AA, 5 V buck-boost regulator, microSD | Camera logger | $70–110 (Pi Zero 2 W $15; others estimates) |

Ready-made kits exist: one supplier sells a 600 g package with balloon, cords, inflator, scale and fill hardware for $420. Compare what each kit includes before buying.

### Power budget

Run `cargo run --locked -- energy-budget` to recompute. One lithium AA is rated 3500 mAh at 100 mA; the budget counts 3000 mAh at 1.4 V, then keeps 25% back for cold and 15% for regulator losses (both estimates). Plan for six hours (preparation, about three hours of flight, recovery), then double it.

How to wire the cells: four in series give about 6 V fresh, which is too much for the boards directly. A small buck-boost regulator turns it into a steady 3.3 V for payload A or 5 V for payload B. Eight cells are two strings of four connected in parallel, which keeps 6 V and doubles the capacity. A mentor checks battery wiring and polarity before every power-up.

| Payload | Draw | Needed (6 h × 2) | Batteries | Run time after derating |
| --- | --- | --- | --- | --- |
| A: XIAO (25 mA at 3.3 V), GPS (under 25 mW while tracking; more while finding satellites), sensor and card (estimates) | about 0.18 W | 2.1 Wh | 4 × lithium AA | about 60 h |
| B: Pi Zero 2 W (up to 1.2 W) + camera (0.3 W estimate) | about 1.5 W | 18 Wh | 8 × lithium AA | about 14 h |

Four cells are not enough for payload B: they give about 7 hours, under the 12-hour target. Keep the cells inside the foam next to the electronics, where they stay warmest.

### Weight and the FAA thresholds

Weigh the finished payload. Estimates for a first flight: payload A with box and batteries about 250–350 g, tracker with its batteries about 50–100 g, parachute about 100 g. That is roughly 0.4–0.55 kg (about 1–1.2 lb), inside the 1–2 lb range of a 600 g balloon. Payload B adds about 150–200 g. The flight lead reviews the weighed balloon, line and payload together before launch.

In the United States, 14 CFR 101.1 brings an unmanned free balloon under the Part 101 balloon rules (operating limits, equipment, notices and position reports) if any of these is true:

- one package weighs more than 4 lb and more than 3 oz per square inch of its smallest face
- any one package weighs more than 6 lb
- two or more packages together weigh more than 12 lb
- the suspension line needs more than 50 lb of impact force to break away from the balloon

A small classroom payload usually falls below all four, but measure rather than assume. Whatever applies, treat the hazard rule in section 101.7 as the standard for every flight: no operation that creates a hazard to people, property or aircraft. This is not legal advice; the launch partner and flight lead confirm what applies to the actual flight.

## Prepare for a supervised flight

Use a partner experienced in high-altitude balloon launches and recovery. Ask them to define the payload interface, mounting and retention, pressure sensing exposure, sensor range, temperature limits, power and battery handling. Have the team prove that the logger keeps recording if the communications link is absent. Use the launch partner's approved tracking and recovery system independently of the experiment. A successful first flight is retrieval of a useful, timestamped data record. Treat optional radio reception as its own experiment.

Before any launch, the adult flight lead owns the go/no-go decision and confirms airspace and flight rules, notifications or approvals as applicable, ground hazards, acceptable weather and recovery access. For a U.S. launch, review the actual package and operation against the applicable provisions of 14 CFR Part 101 with the launch partner. Verify spectrum authorization separately before any transmitting configuration. [14 CFR Part 101](https://www.ecfr.gov/current/title-14/chapter-I/subchapter-F/part-101) · [FAA unmanned balloon procedures](https://www.faa.gov/air_traffic/publications/atpubs/atc_html/chap9_section_6.html)

The PLA and ASA parts supplied here are desktop mockups. An actual balloon payload needs its own host-approved mounting, materials, battery and recovery design.

## Sources for the flight plan

- Balloon size, payload range, burst altitude and helium volume: [The Rocketman balloons](https://www.the-rocketman.com/products/high-altitude-weather-balloons-hab), [Rocketman 600 g package](https://www.the-rocketman.com/products/hab-kit-complete-package), [High Altitude Science 600 g](https://highaltitudescience.com/products/600-g-near-space-balloon), [High Altitude Science on helium](https://highaltitudescience.com/pages/helium)
- Batteries: [Energizer L91 datasheet](https://data.energizer.com/pdfs/L91.pdf), [Energizer L91 capacity ratings](https://data.energizer.com/pdfs/lithiumintrinsicsafety.pdf)
- Power: [Seeed XIAO ESP32-C3](https://wiki.seeedstudio.com/XIAO_ESP32C3_Getting_Started/), [u-blox MAX-M10](https://content.u-blox.com/sites/default/files/MAX-M10_ProductSummary_UBX-20017987.pdf), [Raspberry Pi power by model](https://pip-assets.raspberrypi.com/categories/685-app-notes-guides-whitepapers/documents/RP-010139-WP-1-Use-case-specific%20thermal%20performance%20of%20Raspberry%20Pi%20SBCs.pdf)
- Tracker: [LightAPRS at QRP Labs](https://shop.qrp-labs.com/lightaprs), [LightAPRS 2.0 license note](https://github.com/lightaprs/LightAPRS-2.0)
- Rules: [14 CFR 101.1](https://www.law.cornell.edu/cfr/text/14/101.1), [14 CFR Part 101](https://www.ecfr.gov/current/title-14/chapter-I/subchapter-F/part-101)
