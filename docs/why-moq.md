# Why Media over QUIC: evidence and roadmap

**Status: open for review.** This page separates what has been measured from what is proposed. Please challenge any claim in a [feedback issue](https://github.com/steelhead99x/cubesat-classroom/issues/new?template=feedback.yml&labels=feedback&title=Why%20MoQ%3A%20).

## The idea

A satellite pass lasts a few minutes, and only a station under it can hear it. The project's bet is that Media over QUIC (MoQ) relays can take what one ground station receives and copy it, frame by frame, to every classroom, phone app and quest guide that subscribes. A school with an old computer and no antenna can then take part in a live pass.

Three pieces make that possible:

- **The MoQ model** from [moq.dev](https://moq.dev) and the [IETF MoQ working group](https://datatracker.ietf.org/wg/moq/about/). Media is cut into groups (one keyframe interval) and objects (one frame). Relays fan subscriptions out, and a lost frame pauses the picture until the next keyframe instead of smearing it.
- **moq-subghz** ([source](https://github.com/steelhead99x/moq-subghz)). The same group and object model runs over a sub-GHz radio link built from a Raspberry Pi 2 and a common LoRa chip, with an optional republish to a moq.dev relay.
- **[rind](https://github.com/kixel-ai/rind)**, a pure-Rust encryption library. moq-subghz uses it for ChaCha20-Poly1305 frame encryption, light enough for a Pi 2.

## What has been measured

These figures are from the moq-subghz bench, described in [Broadcast over MoQ, starting small](https://hikd.ai/posts/broadcast-over-moq/) (hikd.ai, 19 November 2026) and the moq-subghz `results/` folder.

| Measurement | Result |
| --- | --- |
| Video | 320×240 at 15 fps, a test pattern rather than a camera, from a 2015 Raspberry Pi 2 |
| Radio | Semtech SX1276 in FSK mode at 300 kbit/s, 255 bytes per transmission, no LoRaWAN |
| Capture to screen | 123 to 139 ms across four 300 kbit/s runs, across a desk. About 132 ms typical in an encrypted 350 kbit/s run, which is above the chip's datasheet rate |
| Frames shown | About 95% Pi to laptop and about 90% laptop to Pi; about 96.5% in the best encrypted run |
| Frame repair | At 1% radio loss in replay, 42.4% of frames were never shown without repair, and 1.8% with one spare frame per eight |
| Encryption cost | 20 bytes per frame, about 95 µs of work per frame on the Pi 2 |
| Factory software | About 550 bit/s, against about 160 kbit/s for the live path |

The classroom station in this repository runs the relay path on one computer: browser, ground station, ground MoQ relay, CubeSat MoQ relay, payload. It shows request and response, contact loss and recovery, but not radio.

`downlink-sim` is a separate desk simulation in this repository: virtual receivers, invented RSSI, and a compact telemetry body sized for a beta LoRa board. Its counts are not the bench rows above. See [downlink.md](downlink.md).

## What has not been shown yet

The article is explicit about its limits, and so is this project:

- The radio receiver has been passed to a MoQ relay **with recordings, not yet with the radios live**.
- Fan-out to many viewers has not been measured yet. A mesh of relays between schools has not been built or tested.
- Neither bench is a broadcast product. Real distance, two lost frames in a row, and long runs still need work.
- Nothing has flown. Flight use needs spectrum authorization and the reviews in [mission.md](mission.md) and [iss-deployment-review.md](iss-deployment-review.md).

## Roadmap

Each step produces evidence another crew can repeat.

| Step | What to build | Evidence that it worked |
| --- | --- | --- |
| 1 | Live radio into a relay: moq-subghz receiver publishing to a local MoQ relay while the radios run | Frames shown and latency match the direct path within a stated margin |
| 2 | Classroom fan-out: one relay, many subscribers on mixed school hardware | Per-subscriber frames shown and latency for 5, 20 and 50 viewers |
| 3 | Old hardware: the same stream over QUIC and a TCP fallback on the oldest computers a partner school has | A table of machine, transport, CPU use and frames shown |
| 4 | Apps and agents: the mobile app and the quest guide subscribe to telemetry and video like any other viewer | A class completes a quest using only the app or the guide |
| 5 | Mesh between schools: relays in two schools sharing one pass | Both schools receive the same groups; the loss of one relay does not stop the other |
| 6 | Balloon, then orbit | A supervised balloon flight, then provider and spectrum review, before any orbital claim |

## Why this is worth investing in

- **It reaches schools without antennas.** One licensed ground station can serve many classrooms.
- **It fits old hardware.** The measured bench used a 2015 Raspberry Pi 2 and a common radio chip.
- **It is open.** The protocol work is public at the IETF, the libraries are open source, and the quests turn every test into a lesson.
- **It is measurable.** Each roadmap step has a number a reviewer can check.

The investment case is a proposal for reviewers to test, not a finished result.
