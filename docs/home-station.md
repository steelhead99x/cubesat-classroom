# Home station: an RTL-SDR V4 now, the pedal later, and a prototype CubeSat

This guide sets up a receive station at home that feeds the classroom app over Media over QUIC (MoQ). Phones and the website see the same `stats` track whether the data comes from the simulator, an RTL-SDR Blog V4 or, later, the CSC-R1P pedal. A bench CubeSat on the desk feeds the telemetry track beside it. The guide is written for the adult who owns the station. Students use the app and website and never need access to the home network or the relay.

Every snapshot carries a label. Keep it honest:

| Data from | Label |
| --- | --- |
| `receiver-sim` | desk simulation |
| V4 or pedal with a signal generator, test source or 50 Ω load | hardware bench |
| V4 or pedal on the antenna, hearing a real satellite | received RF |
| Bench CubeSat over USB | hardware bench |

Nothing in this guide transmits. The station only receives. The bench CubeSat talks over a USB cable.

## 1. Start with the simulator

Before buying or mounting anything, run the whole path with simulated data. On the station computer:

```sh
# One terminal: a local relay. It prints a moqt:// URL and a certificate fingerprint.
cargo run --locked --bin cubesat-classroom -- relay

# Another terminal: one simulated pass every 15 minutes, as the pedal would see it.
cargo run --locked --bin cubesat-classroom -- receiver-sim --relay moqt://... --fingerprint ... --station home --track-doppler

# A third: read one snapshot back.
cargo run --locked --bin cubesat-classroom -- read-receiver --relay moqt://... --fingerprint ...
```

The station name is a nickname made of lower-case letters, digits and hyphens. The command refuses anything else, so a family name or street can't end up in the feed. Leave out `--relay` to print a pass as JSON lines instead. Try `--source rtl-sdr-v4`, `--band 70cm` and different `--gain-db` values to see how the snapshot changes. The [stats contract](classroom-receiver.md#stats-version-1) lists every field. The website's [pedal trainer](https://cubesatclass.org/#pedal-lab) runs the same orbit model in its pass mode.

## 2. Add the RTL-SDR Blog V4

Do the [V4 first-light quest](../quest/m0/receiver-v4-first-light.md) first. It installs the drivers and `rtl_power`, checks the antenna socket for DC and records the noise floor that later readings are compared with. Keep the bias tee off unless a powered preamplifier is fitted.

Then pipe `rtl_power` into the bridge. This example watches 20 kHz around the ISS 2 m downlink at 30 dB gain, once a second:

```sh
rtl_power -f 145.79M:145.81M:500 -g 30 -i 1 - \
  | cargo run --locked --bin cubesat-classroom -- receiver-bridge --gain-db 30 --satellite "ISS (ZARYA)" \
      --station home --relay moqt://... --fingerprint ...
```

The bridge takes the strongest bin within 3 kHz of the frequency as the signal and the median of the other bins as the noise floor. Pass the same gain to both commands so the snapshot records it. The bridge reports relative FFT levels in dB. The CSV does not measure ADC clipping. Rows carry the sweep’s local capture time converted to UTC, so run the bridge in the same time zone as the recorder. With the antenna disconnected and a 50 Ω load on the socket, add `--label hardware-bench`.

A pass from the ISS shifts by about ±3.4 kHz on 2 m and ±10 kHz on 70 cm. To follow a real satellite, predict its passes first with `cargo run --locked --bin cubesat-classroom -- passes` (see [ground station level 2](ground-station.md#level-2-predict-a-pass)) and widen `-f` or `--passband-hz` to cover the shift. The bridge leaves the sky fields (`elevation_deg`, `doppler_hz` and the rest) empty, because it does not know where the satellite is.

## 3. Put the antenna up

Start indoors near a window or in the attic with the 2 m dipole from the [classroom receiver guide](classroom-receiver.md#1-build-the-antenna). Move outside only when the indoor noise floor is the limit.

- Keep any mast well clear of power lines: farther than the mast and antenna could reach if they fell.
- Follow your local electrical code for grounding the mast and the coax, and fit a coax lightning arrestor where the cable enters the house. Ask a qualified electrician or an experienced amateur radio operator to check the installation.
- Use low-loss coax and keep it short. Every metre loses signal before the receiver.
- Students do not climb ladders or work on roofs for this project.

## 4. Share the feed

The snapshots follow the layout the cubesat.app phone app reads, on its `live` broadcast, and its parser accepts them as they are. The loopback relay in step 1 serves tools on the station computer. For phones or a remote classroom, publish to an authenticated relay that the viewer can reach. The station only connects outward. Follow the hosting rules in the [classroom receiver guide](classroom-receiver.md#2-connect-the-mobile-app-over-moq): short-lived credentials scoped to this station, publisher secrets kept on the station, and a test that a subscriber can't publish.

The `stats` snapshot carries no location, name or call sign, and should stay that way. Don't add the station's address or coordinates to the broadcast name or the snapshot. Run the pass predictor locally; the coordinates it uses stay on the station computer.

## 5. Swap in the pedal

The [CSC-R1P pedal](../hardware/sdr-receiver/pedal/README.md) is designed but not built. When it is, it publishes the same snapshot with `"receiver": "pedal"`, so the app needs no changes. Until then, `receiver-sim --source pedal` stands in for it. Compare the pedal against the V4 on the same antenna with the [Rev A vs V4 quest](../quest/m1/receiver-rev-a-vs-v4.md).

## 6. Add the prototype CubeSat

Build one of the [bench CubeSats](bench-cubesats.md) and connect it to the station over USB with `sensor-bridge` ([how](bench-cubesats.md#connecting-to-the-classroom-station)). Its readings go out on the telemetry track, labelled hardware bench, beside the receiver's `stats`. Students can then watch the CubeSat's temperature and voltage next to a simulated or real pass.

The bench CubeSat has no radio downlink. To show what a downlink would add, run the [915 MHz downlink desk simulation](downlink.md) with `live-feed`, which is also desk simulation. A real radio link from the prototype is a separate project: it needs a licensed operator for amateur frequencies, or a Part 15 design checked against the rules, and a qualified reviewer. See [mission and radio design](mission.md).

## Checklist

- [ ] `receiver-sim` reaches the app through the relay, and the app shows "desk simulation".
- [ ] V4 first-light sweeps saved, with the bias tee off.
- [ ] `receiver-bridge` running from `rtl_power`, labelled hardware bench on a load and received RF on the antenna.
- [ ] Antenna installation checked by a qualified person.
- [ ] Hosted relay with scoped credentials, and no location in the feed.
- [ ] Bench CubeSat telemetry on the telemetry track, labelled hardware bench.
