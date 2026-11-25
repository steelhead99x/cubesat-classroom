# 915 MHz downlink desk simulation

**Status: work in progress.** `downlink-sim` and `live-feed` move bytes through a virtual channel on one computer. They do not open a radio. RSSI and SNR in the health track are chosen for the exercise, not measured. Do not describe a result from these commands as received RF, a flight, or an approval.

## Two listeners

```text
payload snapshot
    │  one cleartext packet (compact telemetry)
    ├────────────────────────────► beta LoRa board (OLED)
    │
    └─ virtual receivers (loss, duplicates)
           │  recover, then expand
           ▼
       MoQ relay ──► phones, browsers, school apps
                     tracks in docs/client-contract.md
```

Phones and school apps subscribe to Media over QUIC. A beta LoRa board with an OLED and a keyboard listens to the packet itself. Media (image, audio, video) is only on the MoQ side. The board keyboard does not transmit.

## Rules for 902–928 MHz

In the United States that range is used under FCC Part 15 (unlicensed, emission limits apply) or under Part 97 (the 33 cm amateur band). Picking `--service part15` or `--service part97` only changes the frame rules in this desk exercise. It is not a certification, a license check, or permission to transmit.

Part 97 frames are unencrypted and carry the control operator's station ID in the clear, in line with 47 CFR 97.113(a)(4) and 97.119. The CRC-32 and the Reed-Solomon parity exist so a damaged copy can be detected or rebuilt. A listener can read a data shard without the parity. This program does not use LoRaWAN. LoRaWAN encrypts the payload, which is the wrong shape for an amateur cleartext downlink.

`N0CALL` in examples is a documentation placeholder, not an assigned call sign. Do not put a student's name or call sign in the field.

## Beta board packet

The telemetry body is 18 bytes, fixed, big-endian:

| Offset | Size | Field |
| --- | --- | --- |
| 0 | 1 | Reserved 0 |
| 1 | 1 | Flags: bit 0 pressure is present, bit 1 source is the sensor files |
| 2 | 4 | Capture time, Unix seconds |
| 6 | 2 | Temperature, signed, 0.01 °C |
| 8 | 2 | Bus voltage, millivolts |
| 10 | 4 | Light, lux |
| 14 | 4 | Pressure, pascals (ignore unless flag bit 0 is set) |

That body sits in the air frame. The frame header carries the magic `C5 91`, version 1, kind 0 (telemetry), the service byte, session, sequence, shard index, and the station ID. A CRC-32/ISO-HDLC (the Ethernet polynomial, big-endian) covers every byte before it. The byte diagram is the comment at the top of `crates/downlink/src/frame.rs`.

A board reads shard index 0. That shard is the body. Shard index 1, when parity is on, is repair data for the ground assembler, not a second copy of the numbers. With the placeholder ID `N0CALL`, the displayable data frame is 50 bytes, inside a 51-byte budget. The longest station ID this program accepts (16 characters) stays inside 64 bytes. Both limits are size targets for a small LoRa payload, not a claim about a region's LoRaWAN dwell time or duty cycle.

The OLED preview is four lines of at most 16 characters, which fits a 128×64 panel with an 8-pixel font and also a 128×32 panel:

```text
N0CALL #7
21.5C 3.91V
460 lx
part97 desk
```

`Telemetry::oled_lines` builds those lines. There is no firmware here. A keyboard, if the beta board has one, only changes the local screen.

Time on the air is whole seconds. Temperature on the air is 0.01 °C and voltage is 1 mV. `live-feed` publishes the expanded JSON and CBOR from this body, so a phone and the OLED are showing the same quantized snapshot. `evidence` on that JSON is `desk-simulation`.

## What the virtual channel does

`downlink-sim` runs recorded cases. One burst of six frames erases an object whose shards sit together (3 of 4 objects recovered) and does not erase it when the shards are interleaved (4 of 4). At 10% independent frame loss, 400 snapshots, one receiver: 364 recovered with no parity, 395 recovered with one parity shard (34 of those rebuilt). At 40% loss, one receiver recovers 182 of 300; three receivers that each drop frames on their own recover 285, and 245 extra copies are discarded. Those counts are locked in the crate tests. They are not radio measurements.

`live-feed` publishes the recovered snapshot on `cubesat/classroom/live`. `--budget-bps` still sends JSON telemetry when the cap is too small for an image. `--media` adds the optional JPEG, PCM, and video tracks. The default is telemetry only.

```sh
cargo run --locked -- downlink-sim
cargo run --locked -- downlink-sim --json
cargo run --locked -- downlink-sim --bench
cargo run --locked -- relay
cargo run --locked -- live-feed --service simulation
cargo run --locked -- read-live --reading status
```

`live-feed` and `read-live` need the relay URL and fingerprint in the environment, the same way `simulate` does. See [client-contract.md](client-contract.md).
