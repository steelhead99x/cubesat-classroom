# Build a classroom satellite receiver

Start with a laptop, a USB radio and two antenna rods. The phone app is the first classroom demo. The finished station has its own screen, speaker, volume and mute so the class can use it with phones put away.

After the bench test, use a pass prediction to plan when to listen for a CubeSat in Earth orbit. [Ground station guide](ground-station.md)

```text
2 m antenna ── coax ── USB SDR ── station ── built-in speaker
                                    │         and screen
                               Media over QUIC
                              /               \
                    local teacher mic     phone live stats
```

## 1. Build the antenna

Use a receive-only, center-fed dipole for a downlink near 145.8 MHz. The ARRL starting formula gives about **49 cm per side** at that frequency. With telescopic rods, measure from the feed terminal to the tip. For a wire build, start with two 50 cm pieces of stiff copper wire and leave room to adjust them equally. Use the actual downlink frequency when setting the length. [ARRL dipole guide](https://www.arrl.org/files/file/Technology/tis/info/pdf/9106023.pdf)

```text
tip <── about 49 cm ──> [ insulated mount ] <── about 49 cm ──> tip
                         coax center   coax shield
                                │
                            coax to SDR
```

1. Reuse a telescopic dipole kit, or attach the two rods to separate terminals on a plastic block. Connect the coax center conductor to one terminal and the shield to the other. Keep the rods electrically separate.
2. Secure the coax to the mount for strain relief. Run it away from the rods at roughly a right angle. Use a short cable with a connector that matches the dongle. Keep the SDR bias tee off for this passive antenna.
3. Mount the dipole horizontally on a temporary stand with a clear view of the sky, away from overhead power lines. Pick a confirmed active 2 m downlink and a high pass. Log the orientation and compare it with a second orientation if the signal fades.

The easiest first purchase is an RTL-SDR Blog V3 kit with a telescopic dipole, listed at **$44.95–49.95 USD** by the maker when checked. Reuse a laptop, its speakers and an SDR you already have if possible. A wire mount can be made from scrap plastic, or test-fit a mount in PLA on the Bambu P1S before trying an ASA version outdoors. This antenna is sized for 2 m. UHF, S-band and X-band can get their own antenna and receiver designs. [RTL-SDR kit listing](https://www.rtl-sdr.com/buy-rtl-sdr-dvb-t-dongles/)

## 2. Connect the mobile app over MoQ

Use the existing Rust MoQ stack for live data and audio. Add a station publisher, and give the mobile app a separate **Live station** view that subscribes to it. Start with one opt-in receive station on a private Wi-Fi network. Pair the phone with a station QR code and use HTTPS for setup. Leave the Rust classroom runner on loopback and use a dedicated publisher for the live station. The built-in display and speaker also use the local receiver so the station works without internet access.

| MoQ track | Content | Access |
| --- | --- | --- |
| `stats` | Versioned JSON snapshots with source, UTC time, tuned frequency in Hz, relative signal level in dBFS and optional decoded sensor values | Station publishes. Paired phones subscribe. |
| `receive-audio` | Optional mono Opus audio from the receiver, with codec information in the catalog | Station publishes. Paired phones subscribe. |
| `teacher-mic` | Mono Opus push-to-talk audio in a separate LAN-only broadcast | Paired teacher publishes. Station subscribes. |

### `stats` version 1

The station writes this layout now, from a simulator or from a real RTL-SDR, so the app and website can be built before the pedal is on the air. Run `cargo run --locked --bin cubesat-classroom -- receiver-sim` to print one pass cycle as JSON lines, or add `--relay` and `--station <nickname>` to publish to the `live` broadcast. `read-receiver --relay ...` reads one snapshot back. Pick the receiver with `--source pedal` or `--source rtl-sdr-v4`, the band with `--band 2m` or `--band 70cm`, and add `--track-doppler` to retune during the pass. For a real receiver, pipe `rtl_power` into `receiver-bridge`, for example `rtl_power -f 145.79M:145.81M:500 -g 30 -i 1 - | cubesat-classroom receiver-bridge --gain-db 30 --relay ...`. It labels snapshots `received-rf`, or `hardware-bench` with `--label hardware-bench`; name what you are hearing with `--satellite`. The code lives in [src/receiver.rs](../src/receiver.rs); the website's pass mode uses the same orbit model in [site/src/lib/pass.mjs](../site/src/lib/pass.mjs), and a test keeps the two in step.

The first block of fields is the contract the cubesat.app phone app reads (`StationSnapshot` in its `mobile/src/lib/snapshot.ts`). The app drops fields it doesn't know, so the receiver block can grow without breaking it. The default broadcast is `live`, the name the app subscribes to.

| Field | Meaning |
| --- | --- |
| `v` | Layout version, `1`. Readers refuse other versions. |
| `seq`, `session` | Counter, and an id that changes each time the publisher starts, so viewers can order readings when `seq` restarts |
| `station` | Station nickname such as `room-12`: lower-case letters, digits and hyphens. Never a name, school or address. |
| `source` | What is being heard, such as `Simulated satellite` or `ISS (ZARYA)` |
| `provenance` | `desk-simulation`, `hardware-bench` or `received-rf`. Show it beside every value. The simulator always writes `desk-simulation`. |
| `utc` | Capture time, RFC 3339 |
| `contact` | `acquired` (carrier heard), `lost` (satellite up, carrier not heard) or `idle` |
| `frequency_hz`, `level_dbfs` | Where the receiver is tuned, and the total level |
| `elevation_deg` | Satellite elevation. Absent from a real receiver without a pass prediction. |
| `sensors` | Decoded beacon values as `{"temperature": {"value": 18.0, "unit": "°C"}}`, absent when the signal is too weak or clipped |
| `receiver`, `band`, `nominal_hz` | `pedal` (CSC-R1P) or `rtl-sdr-v4`; `2m` or `70cm`; the published downlink frequency |
| `gain_db`, `noise_dbfs`, `snr_db`, `clipping` | Gain setting, noise floor, signal-to-noise ratio (`null` with no signal) and whether the input is near full scale |
| `doppler_hz`, `offset_hz`, `range_km`, `pass_s` | Doppler shift, received carrier minus tuned frequency, slant range, and seconds since rise (negative before it). Absent from a real receiver without a pass prediction. |

The simulated levels are planning numbers, not measurements. From `rtl_power`, `level_dbfs` is the strongest bin near the frequency and `noise_dbfs` the median of the other bins. Both are relative dB from the dongle, not calibrated, so compare readings only from the same dongle and gain.

Publish a fresh stats snapshot about once a second. The app should show its age and mark the feed stale when updates stop. Start with a subscribe-only app. When adding student interaction, use a separate request track for a predefined action such as asking for the latest sensor snapshot. Validate and rate-limit the requests at the station. Keep radio tuning and any transmitter controls out of that path.

Use a private MoQ project for hosted development and testing between networks. Keep the relay URL configurable and use short-lived credentials scoped to each station and role. Keep publisher secrets on the station or token issuer. Test that a subscriber credential cannot publish or read a different station's broadcast. Send simulated or approved non-personal data to the hosted relay. Pin a compatible MoQ protocol version and prove interoperability on physical iOS and Android devices before relying on the hosted path. [MoQ relay authentication](https://doc.moq.dev/bin/relay/)

For the LAN demo, give the teacher a **Hold to talk** button that publishes the microphone to the local MoQ relay. The station plays the voice and lowers the receive audio during speech. Allow one talker at a time. Request microphone access on first use, show when the mic is active and end capture on release, backgrounding or lost connection. Have the station release the talk slot if audio stops arriving. Keep the microphone broadcast out of recordings, the hosted relay and the satellite radio path. Bluetooth Low Energy can be added later for pairing or setup.

For the contributor building the mobile integration, test local-network access and microphone permissions on physical devices. [Apple local-network privacy](https://developer.apple.com/documentation/technotes/tn3179-understanding-local-network-privacy) · [Apple microphone permission](https://developer.apple.com/documentation/bundleresources/information-property-list/nsmicrophoneusagedescription) · [Android local-network permission](https://developer.android.com/privacy-and-security/local-network-permission) · [Android microphone recording](https://developer.android.com/media/platform/mediarecorder)

## 3. Build the station with its own speaker

| Part | First standalone build |
| --- | --- |
| Computer | Reuse a Pi or small PC. Try a Raspberry Pi Zero 2 W for basic reception and verify simultaneous audio and stats on the bench. Use a spare Pi 4 or laptop if the decoder needs more headroom. |
| Speaker | Put a small powered USB speaker with a physical volume control behind a front grille. Pick one that carries both audio and power over USB. |
| Screen and controls | Use a small HDMI touchscreen for frequency, signal and last update. Put **Listen / Mute** on the station screen and leave the speaker volume knob reachable. Reuse a spare monitor for the bench. |
| Power and USB | On the Zero 2 W, use a USB OTG adapter and a powered hub to connect the SDR, speaker and any USB touchscreen input. Keep access to the SDR antenna connector and power jack. |
| Enclosure | Print the front panel in PLA on the P1S and fit it to the actual speaker and screen before sizing the shell. Give the speaker its own grille, provide ventilation, and keep the antenna connector and coax away from the speaker and power wiring. |

The Raspberry Pi Zero 2 W lists for **$15** before power, storage, USB hub, display and speaker. Reusing the display and speaker makes the biggest difference to the cost of the standalone build. [Raspberry Pi Zero 2 W specifications](https://www.raspberrypi.com/products/raspberry-pi-zero-2-w/)

On the station, **Listen** plays demodulated receive audio for voice or tone signals. Use **Mute** during data-only passes and display decoded values when available. The app can offer a separate **Listen on phone** option later. Day-to-day tuning, listening and stats should work from the station itself.

## Bench checks

- **Receiver.** Connect the SDR with its bias tee off. Verify a known signal, then try a scheduled satellite pass. Record the source, time in UTC, frequency, gain, antenna length and orientation.
- **Phone.** Pair a teacher's phone on Wi-Fi, confirm the MoQ readings match the station, and disconnect Wi-Fi to check the stale indicator. Test LAN mic playback and verify it stops when the button is released, the app goes into the background or the phone disconnects.
- **Hosted relay.** Send a simulated snapshot from the station through the private relay to a phone on another network. Verify subscriber permissions and verify that the teacher mic is absent from hosted broadcasts.
- **Standalone.** Put the phone away. Verify the screen, listening and volume work at the station. Compare the SDR noise floor with the speaker and display powered on and off while keeping tuner gain fixed.

The [mobile app](../mobile/README.md) currently runs practice activities offline. The mobile MoQ client, station publisher, phone microphone and standalone controls still need implementation and hardware tests.
