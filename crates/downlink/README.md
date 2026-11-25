# downlink

**Work in progress.** Desk simulation of a cleartext 915 MHz packet and the order in which classroom viewers receive it. This crate does not transmit.

| Piece | Role |
| --- | --- |
| Air frame | CRC-32 and optional Reed-Solomon parity (`reed-solomon-simd`). Data shards stay readable. |
| Board body | 18-byte telemetry for a beta LoRa board and a 128×64 OLED. |
| Assembler | Several virtual receivers, duplicate suppression, a bounded incomplete-object map. |
| Scheduler | JSON telemetry first. Image, audio, and video drop when the byte budget runs out. |

The commands are `downlink-sim`, `live-feed`, and `read-live` on the station binary. The client contract is [docs/client-contract.md](../../docs/client-contract.md). The packet layout is [docs/downlink.md](../../docs/downlink.md).

902–928 MHz is a Part 15 or Part 97 question for a licensed operator and the right authorization. Choosing a service here is a frame rule for the exercise, not an approval.
