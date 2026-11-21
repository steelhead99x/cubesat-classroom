# Ground rehearsal

The video source is an FFmpeg test pattern. `moq-subghz-send` encodes it into H.264, writes the project's SubGHz frames to a loopback file, and `moq-subghz-recv` replays them with their timing. The existing republisher carries recovered Annex-B H.264 access units in Hang objects over native QUIC. Telemetry travels on its own MoQ broadcast on the same ground relay. The initial telemetry source is synthetic and bypasses the RF format.

## Live playback during paced replay

Use the dependency revision and builds in the [README](../README.md). Run one terminal per long-lived process. The relay prints its URL and the ephemeral certificate fingerprint. Export both values in the other terminals and pass the same fingerprint to the upstream publisher.

```bash
cargo build --locked
target/debug/cubesat-classroom relay
```

```bash
export CUBESAT_RELAY=moqt://localhost:4443
export CUBESAT_TLS_FINGERPRINT=PASTE_FINGERPRINT
export MOQ_SUBGHZ_TLS_FINGERPRINT="$CUBESAT_TLS_FINGERPRINT"
target/debug/cubesat-classroom simulate
```

Run `target/debug/cubesat-classroom read-telemetry --reading temperature` in another shell with the same exports. The command formats measured values with units, provenance and age. The five-second freshness gate uses same-host Unix time.

To view video, start the replay and the player in separate terminals with the same exports:

```bash
../moq-subghz/target/debug/moq-subghz-send --loopback /tmp/cubesat-air.txt --fec-k 8 --secs 20
../moq-subghz/target/debug/moq-subghz-recv --replay /tmp/cubesat-air.txt --paced \
  --out /tmp/cubesat-replay --publish "$CUBESAT_RELAY" --publish-name cubesat/classroom/video \
  --publish-bin ../moq-subghz/moq-subghz-publish/target/debug/moq-subghz-publish
```

```bash
set -o pipefail
target/debug/cubesat-classroom verify-video --broadcast cubesat/classroom/video --stream \
  | ffplay -loglevel warning -fflags nobuffer -flags low_delay -f h264 -i -
```

For a future camera bench, use the exact camera/SBC input path supported by the [upstream hardware HOWTO](https://github.com/steelhead99x/moq-subghz/blob/461f6d3f5e7e7683bd1e85e3ad382d735fa33b0a/HOWTO.md) and retain the same native QUIC subscriber. Record camera and modem identities, settings, keyframe recovery, decoded frame counts and capture-clock provenance.

## Demo sequence for visitors

1. Show the assembled mockup or the CAD model. Explain access to the payload, service slot, and the three proposed RF roles.
2. Request the telemetry status. Point to the sample number, timestamp-derived age and simulation label.
3. Start playback of the paced video and explain where the SubGHz framing ends and native QUIC begins.
4. Stop the simulator and request telemetry again. The query must fail closed or expire without reporting a new measurement.
5. Show the link budget assumptions and identify which measured modem or ground-station number should replace each placeholder.

## Configured sensor input

The publisher can read supported Linux sensor values from three explicitly configured files. Use calibrated paths that supply the listed units and expose a current reading on each read. The circuit, driver and calibration are hardware integration work. Publish on a separate broadcast while the simulator is running, then give the same broadcast name to `read-telemetry`.

```bash
target/debug/cubesat-classroom publish-sensors --broadcast cubesat/classroom/bench \
  --temperature-mc /path/to/temperature_millidegrees_celsius \
  --voltage-mv /path/to/bus_voltage_millivolts \
  --light-lux /path/to/light_lux
target/debug/cubesat-classroom read-telemetry --broadcast cubesat/classroom/bench --reading status
```

A failed input pauses publishing. If all inputs recover, the next sample resumes with a new sequence number. The publisher's timestamp records the start of each local read. For devices that may expose a cached reading, a hardware adapter must also supply and validate capture time or sequence from the sensor.

## Verification record · 2026-11-21

One host, one loopback relay, pinned certificate, SubGHz repository revision `461f6d3f5e7e7683bd1e85e3ad382d735fa33b0a`. The telemetry query received a simulated sensor snapshot over native QUIC. The paced SubGHz frame replay delivered 121 H.264 access units to the native QUIC subscriber, and FFmpeg decoded the capture. FFprobe counted 121 frames at 320 × 240. Capture SHA-256 is recorded below. The test used generated video and software framing. Live capture, RF hardware, and browser playback were outside this run.

`4899f9df6a925ab9eee38dfd5e1d32dfd687d2e872071fd459a8748c626bbf0b`

A separate loopback run used fixture files for the configured sensor input. It verified signed temperature conversion, bus voltage conversion, stale-data rejection after an input disappeared, and publication recovery after it returned.

## Classroom station and attached sensor files

The browser station is the easiest place to start. Run `cargo run --locked -- classroom` and open the exact localhost address printed by the command. Use a classroom nickname and optionally a mentor's call sign when you save an observation. Neither is sent to the relay.

To use documented Linux sensor files with the browser station, run `cargo run --locked -- classroom --sensor-dir ./sensors`. The directory must contain `temperature_mc` (millidegrees Celsius), `voltage_mv` (millivolts) and `light_lux` (lux). The camera remains a simulation.

## Classroom verification · 2026-11-21

A localhost browser run requested a new sensor reading, received all eight JPEG frames through the paired relays, paused and recovered contact, and downloaded a crew log containing the test alias, optional call sign and observations. The mobile layout was checked at a 390-pixel viewport. A separate balloon logger fixture test removed the pressure input and restored it while the temperature continued recording. The CSV showed the missing pressure value and subsequent recovery. These were software and fixture tests.
