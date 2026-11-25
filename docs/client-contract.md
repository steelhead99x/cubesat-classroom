# Classroom client contract

**Status: work in progress.** This is a desk simulation. Nothing here has been on the air, and nothing in this repository transmits. Phone, browser, and school-app support is the path to build next. A LoRa board with an OLED and a keyboard is a **beta** listener only.

Two ways to read the same snapshot:

| Client | How it hears the snapshot | Status |
| --- | --- | --- |
| Android and iOS app, Chromebook, browser, LMS page | Media over QUIC tracks, below | Documented. The mobile app does not subscribe yet. |
| LoRa board with a small OLED and a keyboard | One cleartext 915 MHz packet, [downlink.md](downlink.md) | Beta. No board firmware in this repository. |

A client may use either path. It does not need both. Media (pictures, sound, low-rate video) is only on the MoQ tracks. The OLED path is telemetry.

## Regulatory label

902–928 MHz in the United States is used under FCC Part 15 (unlicensed, with emission limits) or under Part 97 (the 33 cm amateur band). This software does not transmit, and it is not an approval, a certification, or a coordination.

Amateur-mode frames stay readable and carry the control operator's station ID, because a licensed operator has to follow 47 CFR 97.113(a)(4) and 97.119. The CRC and Reed-Solomon parity repair loss. They do not hide the payload. Do not put a student's name or call sign in the station ID. `N0CALL` in examples is a documentation placeholder, not an assigned call sign.

## MoQ tracks

One broadcast: `cubesat/classroom/live`.

Subscribe only to the tracks you can show. Skipping a track is normal.

| Track | Required | What one group contains | moq-net priority (higher first) | IETF priority (lower first) |
| --- | --- | --- | --- | --- |
| `telemetry` | yes | One JSON snapshot | 240 | 15 |
| `telemetry.cbor` | no | The same snapshot in CBOR | 230 | 25 |
| `health` | no | Ground receiver totals, JSON | 200 | 55 |
| `image` | no | One JPEG still | 100 | 155 |
| `audio` | no | One mono PCM frame (`CSA1` header) | 80 | 175 |
| `video` | no | One independent JPEG | 40 | 215 |

This station uses moq-net, where a **higher** priority is more urgent. On the IETF wire the same urgency is `255 - priority`, and a **lower** number is more urgent. Set the subscriber priority from the column that matches your library. The Rust reader in `read-live` sets the moq-net column and asks only for the newest group.

Under a tight link, JSON telemetry is sent even when it is larger than the budget. Older unsent snapshots are replaced by the newest one. Image, audio, and video are dropped first. `telemetry.cbor` and `health` wait behind JSON telemetry. A client that only needs the numbers should subscribe to `telemetry` and to nothing else.

Each telemetry group is a full snapshot, not a difference from the previous one. A late joiner that asks for the newest group gets the current reading. A slow joiner does not get a backlog: when a newer group arrives, older groups are skipped. Check `captured_unix_ms`. This program treats a snapshot older than 5 seconds, or more than 2 seconds ahead of the receiver clock, as not fresh.

The relay command listens on loopback only. It prints a raw QUIC URL (`moqt://…`) and a WebTransport URL (`https://127.0.0.1:<port>/`) for the same UDP socket, plus the certificate fingerprint. Rust tools pin that fingerprint. A browser or Chromebook needs a certificate it already trusts; the generated certificate is not a public web certificate. Android and iOS apps can pin the fingerprint the way the Rust tools do. An LMS page on the public web needs a relay with a normal HTTPS certificate. That hosted step is not built here.

`read-live` is the reference reader. `live-feed` is the desk publisher. Neither transmits.

### JSON telemetry

`schema` is `cubesat.telemetry.v1`. Ignore fields you do not know. Numbers are SI units.

| Field | Meaning |
| --- | --- |
| `sequence` | Counts up by one for each snapshot the sender attempted |
| `captured_unix_ms` | Capture time, Unix milliseconds. After the radio hop this is whole seconds |
| `source` | `simulation` or `configured sensor inputs` |
| `evidence` | This program writes `desk-simulation` only |
| `service` | `simulation`, `part15`, or `part97` |
| `frequency_hz` | `915000000`, a lab label inside 902–928 MHz, not a channel assignment |
| `station_id` | Control-operator station ID, or absent |
| `temperature_c` | Degrees Celsius. The radio path keeps 0.01 °C |
| `bus_voltage_v` | Volts. The radio path keeps 1 mV |
| `light_lux` | Lux |
| `pressure_pa` | Pascals, or absent |

`telemetry.cbor` is the same object encoded as CBOR. Use a CBOR library. Do not expect a particular byte layout beyond what that library produces.

### Optional media

`image` and `video` are JPEG bytes (`FF D8` at the start). Each video group is its own picture, so a missed group does not spoil the next one. `audio` starts with the ASCII bytes `CSA1`, then a big-endian sequence (`u32`), capture time (`u64`), sample rate (`u32`), a channel count of 1, and little-endian signed 16-bit samples. Generated tones are a desk simulation, not a microphone.

`health` is ground-side. Its RSSI and SNR figures in this program are invented for the virtual receivers. Do not chart them as measured radio.

## Beta: LoRa board, OLED, keyboard

A board that hears 915 MHz directly does not open a MoQ session. It parses one air frame and draws four lines. The layout, the 51-byte and 64-byte size targets, and the four OLED lines are in [downlink.md](downlink.md).

This path is beta:

- There is no board firmware, OLED driver, or keyboard handler in this repository.
- The keyboard is local. Buttons may change the display. They do not transmit, and there is no uplink in this contract.
- Pictures and sound are not sent to the board. Those tracks are optional MoQ subscriptions for phones and school screens.
- A data frame (shard index 0) is a complete snapshot. A parity frame is for the ground assembler. A board that only wants the numbers can ignore parity frames.
- Treat the reading as a desk simulation until a licensed operator and a reviewed radio say otherwise.

## What the mobile app still needs

The app in `mobile/` is an offline practice station. `src/app/station.tsx` invents readings on the phone. It does not open a socket.

To show a live pass later, without building that app in this change:

1. Add a Media over QUIC subscribe client. None is in `mobile/package.json` today.
2. Take a relay URL and a certificate fingerprint from the control operator. Do not ask a student for an account, a phone number, or a call sign.
3. Subscribe to broadcast `cubesat/classroom/live`, track `telemetry`, at the priority in the table. Parse the JSON and ignore unknown fields.
4. Show `evidence`, the age of `captured_unix_ms`, and a stale mark after 5 seconds.
5. Subscribe to `image`, `audio`, or `video` only when the screen can show them. On a slow or metered link, stay on `telemetry`.
6. Keep the practice screen for when the relay is absent.

A browser lesson or an LMS page uses the same tracks over WebTransport once the relay certificate is one the browser trusts.
