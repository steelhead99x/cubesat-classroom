# Suborbital research flight: mission preparation

This is the working plan for getting a classroom experiment ready for a research flight on a Virgin Galactic spaceship. It gathers what is public, turns it into a timeline a class can rehearse, and lists what still has to come from the flight provider.

**Status.** On November 18, 2026, Teachers in Space announced plans to send two teachers on a Virgin Galactic research flight expected in 2027. CubeSat Classroom is an independent volunteer project. Nothing here is an agreement, a manifest or a flight plan. Teachers in Space and Virgin Galactic set the real requirements, and their documents replace anything on this page.

## What is public

| Topic | Published information | Source |
| --- | --- | --- |
| The flight | Two teachers, a Virgin Galactic research flight, expected in 2027. Teachers in Space is raising $2 million and reported $285,000 so far. | [Announcement](https://www.virgingalactic.com/news/teachers-in-space-plans-to-send-two-educators-on-2027-research-flight-with-virgin-galactic) |
| The experiment kit | A 3D-printed CubeSat about 3.5 × 3.5 inches (third dimension not given), an Arduino, a rechargeable lithium-polymer battery and a real-time clock. Sensors: temperature, pressure, acceleration, rotation, magnetic field, carbon dioxide, humidity and light. | Announcement |
| The data | Readings go back to the K–12 classrooms that designed the experiments, to study launch, weightlessness and landing. | Announcement |
| The vehicle | The first Delta-class spaceship, Apeiron, arrived at Spaceport America on October 28, 2026 to begin flight testing. First commercial spaceflight is targeted for early in the second quarter of 2027. The schedule has moved more than once. | [Coverage of the unveiling](https://tech.yahoo.com/science/articles/virgin-galactic-unveils-spaceship-apeiron-183008503.html) |
| The ride | Take-off under a carrier aircraft from Spaceport America, release near 45,000 ft, a rocket burn of about 60 s, apogee of at least 80 km, about 3 minutes of microgravity, a feathered re-entry and a runway landing. | [Payload User's Guide, May 2024](https://bynder.virgingalactic.com/m/79e8c34fe3c4d35d/original/Public-Payload-User-Guide_May-2024.pdf) |
| The cabin | Pressurised to the equivalent of 5,000–6,000 ft in flight. 0–30 °C while mated. No active heating or cooling after release. | Payload User's Guide |
| A past flight | 'Galactic 07', June 8, 2024: take-off 8:31, release at 44,562 ft, apogee 54.4 miles, top speed Mach 2.96, landing 9:41. | [Flight facts](https://press.virgingalactic.com/virgin-galactic-completes-12th-successful-spaceflight) |

The guide was written for the earlier spaceship, VSS Unity. Delta-class figures may differ. Ask for the current guide.

## The flight, second by second (desk simulation)

`flight-profile` turns those published numbers into a timeline:

```sh
cargo run --locked -- flight-profile --output run/flight-prediction.csv
```

No Rust installed? Open the ready-made [flight-prediction.csv](assets/flight-prediction.csv) in any spreadsheet.

Words used here: **apogee** is the highest point of the flight. **Microgravity** is the floating part, when everything falls together. A **g** is the push you feel from normal gravity; 2 g feels like double your weight.

| Starts | Phase | Height | What the sensor kit should see |
| --- | --- | --- | --- |
| T+00:00 | Climb under the carrier aircraft | 0 → 13.6 km | About 1 g. Cabin pressure settles near 82 kPa. Vibration from the jet engines. |
| T+50:00 | Release and rocket boost (60 s) | 13.6 → 42 km | A brief drop, then a strong push: about 2.6 g in this model. |
| T+51:00 | Microgravity coast (about 193 s) | 42 → 87.5 → 42 km | Acceleration near zero. Rotation as the ship turns. Light changes at the windows. |
| T+54:13 | Re-entry | 42 → 15 km | Rising g-load. The model leaves this blank because no figure is published. |
| T+57:13 | Glide | 15 → 0 km | About 1 g. Cabin pressure may drift. |
| T+70:00 | Landing | 0 km | Touchdown bump. |

How far to trust it:

- **From published figures:** release height, burn time, apogee, cabin pressure range and total flight time.
- **Worked out by the model:** engine cut-off speed (946 m/s, which is close to the published Mach 2.96), 193 s of microgravity (the guide says about 3 minutes) and the 2.6 g boost figure. The model assumes a straight vertical climb and no air drag, and its re-entry curve shows the shape of the descent, not its real speed.
- **Assumed:** release 50 minutes after take-off and three minutes for re-entry. The guide says the climb takes about an hour; Galactic 07 took 70 minutes in total.

Label everything from this command "desk simulation". After the flight, the provider supplies a standard data set (accelerations, rotation rates, cabin temperature and pressure, position). Use that for the real event times.

### What the pressure sensor will and won't show

The cabin stays pressurised. A pressure sensor inside it reads about 81–84 kPa for the whole flight, not the near-vacuum outside. That is the opposite of a balloon or the unpressurised science bay of the Perlan 2 glider, where the sensor follows the outside air. The quest [One sensor, three rides](../quest/m0/one-sensor-three-rides.md) makes that comparison.

### Sampling plan

| Channel | Why it matters on this flight | Suggested rate |
| --- | --- | --- |
| Acceleration and rotation | Mark release, boost, engine cut-off, weightlessness and re-entry | 10 readings a second or faster |
| Pressure, temperature, humidity, CO₂ | Slow cabin changes; six people in a sealed cabin with no CO₂ scrubbing after release | Once a second |
| Light, magnetic field | Window light as the ship turns; heading changes | Once a second or faster |

At one reading a second the kit collects about 190 readings in microgravity and 60 during the boost. Sixty readings is thin for the boost, which is why acceleration deserves a faster rate. Check the kit's real rates and storage with a full-length bench run: 70 minutes of flight plus time on the ground before and after.

## What the provider's guide asks of a payload

From the May 2024 Payload User's Guide. Confirm each against the current edition.

- **Two ways to fly.** Rack-mounted lockers (a single locker is 18.5 in wide and 21.5 in deep, up to 50 lb and 50 W), or wearable payloads: each researcher may carry two unpowered wearables. A pocket-sized sensor kit carried by a teacher fits the second route more naturally, but the provider decides.
- **Power.** Vehicle power is 24–28 V DC and is recommended, because custom batteries add a safety analysis. Small off-the-shelf batteries are likely acceptable; large or custom packs may be rejected. The kit's lithium-polymer cell needs the provider's review.
- **Switch-off.** One easy-to-find switch must cut all power. The payload must be safe if power fails at any moment.
- **Radio.** Transmitters and receivers must not interfere with the vehicle. Each payload gets an interference check before flight and can be shut down in flight. Plan for Wi-Fi and Bluetooth off, and log to a memory card.
- **Sound, loose parts, liquids.** Under 85 dB. Breakable and moving parts fully contained. Liquids and powders need two layers of containment.
- **Cleanliness.** Visibly clean, no debris. Printed parts must not shed.
- **Paperwork.** A Payload Information Package (the provider's form describing the payload) covering parts, interfaces, hazards, tests and procedures, reviewed by the provider's engineers and a safety board, with a final check during flight week. Flights are usually targeted 6–12 months after contract signature.

## Readiness gates

| Gate | Evidence a reviewer can check | Quest |
| --- | --- | --- |
| 1. Predict | A flight prediction CSV and a written guess for each sensor in each phase | [Predict the spaceflight](../quest/m0/predict-the-spaceflight.md) |
| 2. Bench | A 90-minute bench log with a deliberate gap, units and calibration notes | [Balloon bench log](../quest/m0/balloon-ground-log.md) |
| 3. Compare platforms | The same sensors explained on a glider, a spaceship and the ISS | [One sensor, three rides](../quest/m0/one-sensor-three-rides.md) |
| 4. Package | A draft Payload Information Package with open questions listed | [Draft the payload package](../quest/m1/suborbital-payload-package.md) |
| 5. Host review | Written answers from Teachers in Space or the provider | [Review the hosted experiment](../quest/m1/hosted-review.md) |
| 6. Rehearse | A timed run of switch-on, logging, switch-off and data hand-back, by the people who will do it | Flight-week checklist below |
| 7. Fly and share | A de-identified data set with units, event times and known gaps | [Data return](#after-landing) |

## Flight-week checklist (draft)

1. Fresh memory card, formatted and tested. Spare card labelled.
2. Battery charged per the provider's rule. Charge level recorded.
3. Clock set and checked against a reference. Offset written down.
4. Radios off. Confirmed by the interference check.
5. Two-minute test log read back on a laptop.
6. Kit compared with its paperwork by the provider. No last-minute changes.
7. Switch-on time and switch-off time written down by hand.

## After landing

1. Copy the card twice before anyone opens the file.
2. Keep the raw file unchanged. Work on a copy.
3. Line the log up with the provider's event times, and say how well the two clocks agree.
4. Mark gaps as gaps. Label the data "suborbital flight", with the kit, sensor models and units.
5. Remove names and other personal details before sharing with classrooms.
6. Publish the prediction next to the measurement, including where the prediction was wrong.

## Questions for Teachers in Space and Virgin Galactic

1. Is the kit flown as a wearable, in a locker, or another way? What are the full dimensions and mass limit?
2. Which battery, charge level and protection are accepted? Is vehicle power an option?
3. Must all radios be off? Is a real-time clock with a coin cell acceptable?
4. When is the kit switched on and off, and by whom?
5. Which printed materials are acceptable in the cabin?
6. Which flight data will be shared, at what rate, and what may be published?
7. What are the expected g-loads and vibration, so the mount and the accelerometer range can be chosen?
8. What is the paperwork schedule, counted back from the flight?
9. Can classrooms publish the data openly, and how should the flight be credited?

## Earlier flights to learn from

- **Virgin Galactic research flights.** Seven research missions flew on VSS Unity, with rack-mounted, human-tended and wearable payloads. Examples: seat accelerometers recording vibration on Galactic 01, a pocket radiation dosimeter on Galactic 07, and a suite measuring the electromagnetic environment inside the cabin on an early flight. [Research payload summary](https://bynder.virgingalactic.com/m/275562ddb238e7eb/original/Research_AllPayloads_2024.pdf)
- **Perlan 2.** Teachers in Space flew student CubeSat experiments in the glider's unpressurised science bay in 2015–2016 and 2022–2023. See [the Perlan 2 quest](../quest/m0/perlan-stratosphere.md).
- **Parabolic flights.** Teachers in Space teachers have run microgravity experiments on "zero-G" aircraft flights since 2016.
- **Other dedicated research flights.** Purdue University plans an all-Purdue research flight in 2027, a useful public example of how a school prepares.
