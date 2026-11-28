# Outdoor XIAO station: XIAO ESP32-S3 on Wi-Fi HaLow, XIAO MG24 Sense and a 3000 mAh battery

A fork of the [outdoor ground station](../outdoor-station/README.md): the same sealed box, latching lid and O-ring, battery pocket and tripod thread (no screws), with a **XIAO plate** in place of the payload plate, and two SMA bulkheads in the wall that faces up, for antennas. The rules behind it are in the [design guide](../DESIGN-GUIDE.md).

**Status: design checks only.** Nothing in this folder has been printed. Several board sizes are assumptions, so print the XIAO coupon and try your boards in it before you print a plate.

To open the full-size lid, press each latch finger inward toward the centre of the lid and lift.

## What it holds

- **Seeed Wi-Fi HaLow board for XIAO** (42 × 25 mm, Quectel FGH100M-H, 902–928 MHz), with a **XIAO ESP32-S3** plugged onto its headers. It goes in like the Nano in the kit: tuck one long edge under two lips, press the other edge down until two snap hooks click. Corner stops keep it from sliding.
- **XIAO MG24 Sense with its pin headers soldered.** Its pins go down through two slots in the plate, its USB end tucks under two narrow lips beside the USB-C socket, and one snap hook clicks over its far end. Its USB end faces the wall that faces down, where the cable gland is.
- **The 3000 mAh battery** in the pocket under the plate, held in by the plate, as in the outdoor station.

**Assumed, not measured** (the XIAO coupon checks all of them): board thicknesses (HaLow board 1.6 mm, XIAO 1.2 mm); 8.5 mm female headers, with the ESP32-S3 centred on the HaLow board; no parts within 2 mm of either board's edges on top, where the lips reach 0.8 mm; and XIAO pins on a 2.54 mm pitch in two rows 15.24 mm apart, 6 mm long under 2.5 mm header plastic. If a board doesn't fit the coupon, measure it and tell us: each of these is one number in [xiao.rs](../../crates/amsat-kit/src/xiao.rs).

## Power and antennas

- **Battery:** the XIAO ESP32-S3 and the MG24 Sense each have a LiPo charger and battery pads. Wire the cell to **one** board only: the ESP32-S3's BAT pads, since it also powers the HaLow board. Two chargers on one cell can fight. Power the MG24 from the ESP32-S3's 3V3 pin. A mentor checks this against Seeed's wiki for both boards before connecting the battery, and does the soldering.
- **Antennas:** two SMA bulkheads in the wall that faces up, each fed from a U.FL-to-SMA pigtail: one for the HaLow board's 900 MHz antenna, one for a 2.4 GHz antenna on the ESP32-S3 or the MG24 (both have a U.FL connector). Use an antenna that the module's certification allows. Inside the plastic box the boards' own 2.4 GHz antennas also work, at shorter range.
- **HaLow** is a licence-free (Part 15 in the US) ground link. It never carries anything to an amateur transmitter.

## What to print

The [outdoor fit prototypes](../outdoor-antenna-fit/README.md) cover the shared battery pocket, the snap-fit battery trial, battery cable feed-through, glove latches and both antenna sizes.

| File | Plate | What's on it |
| --- | --- | --- |
| **[p1s/outdoor-xiao-test-coupons.3mf](p1s/outdoor-xiao-test-coupons.3mf)** (print first) | Test coupons | The XIAO coupon (both boards' mounts), the tripod-thread coupon, the latch coupon, and the snap pin with its fit ladder. Each says what it tests on a raised label (below) |
| **[p1s/outdoor-xiao.3mf](p1s/outdoor-xiao.3mf)** | 1. Box | The box, with the two SMA holes |
| | 2. Lid and XIAO plate | The lid and the XIAO plate (14 cm³, on an isogrid) |

**What each coupon tests** (each one says so on its label tile, in raised letters beside the joint):

| Label | Tests | Try it |
|---|---|---|
| BOARD MOUNTS / HALOW + XIAO MG24 | The plate's two board mounts: the XIAO ESP32-S3 on the Seeed Wi-Fi HaLow board, and the XIAO MG24 Sense | Tuck one edge of each real board under its lips and swing it down past the snap hooks until it clicks. It should sit flat without rocking, and come out when you push the hooks back |
| TRIPOD THREAD / 1/4-20 SCREW IN | The box's tripod thread, and one 0.15 mm looser (the socket with a dot) | Screw a tripod's screw into each |
| LID LATCH: BOX and LID LATCH: LID | Two latches on opposite sides of a small box and lid | Press until both click, check that the lid holds, then push both fingers inward and lift. Follow the [shared two-latch coupon procedure](../outdoor-station/README.md#two-latch-coupon) |
| SNAP PIN and FIT 1, FIT 2, FIT 3 | The keyed snap pin the XIAO plate sits on, at three fits | Push each tab onto the pin; the one that clicks on without forcing or rattling is your fit |

The labels and raised logos are new in this version and haven't been printed yet; the joints behind them are unchanged.

Print in ASA or PETG, no supports. The [outdoor station's README](../outdoor-station/README.md) covers the material, the tripod-thread coupon, assembly, the water test and how the box fits.

## Payload retention prototype

Use the regular coupons for the first fit check. The optional restraint lid reuses the same case, plate, latches and O-ring. Four integral posts stop upward motion of the XIAO plate by bearing on its rigid rim. They have a nominal 0.35 mm gap when the plate is seated. They do not clamp the pouch cell or contact the boards.

Generate the optional one-colour lid with `cargo run --release --locked -p amsat-kit -- xiao-retention <output-directory>`. Fit it to an unpowered case with representative dummy masses. Check that the lid seats without forcing, the seal and cable routes stay clear, and each post catches the plate rim before it can climb over the snap-pin heads. Check the stacked ESP32-S3's header retention separately.

The posts transfer plate lift into the lid and depend on the lid remaining captive. The tripod thread and snap latches are intended for ground use. A launch installation needs a carrier fitted to the vehicle's approved interface that independently captures the case and lid. Verify the installed assembly with the mission's load, shock and vibration requirements. The classroom design guide's 20 g static check is a bench target. [NASA's GEVS](https://standards.nasa.gov/node/791) describes environmental verification.

## Buy (besides the outdoor station's list)

| Part | Notes |
| --- | --- |
| Seeed Wi-Fi HaLow board for XIAO, and a XIAO ESP32-S3 | The ESP32-S3 plugs onto the HaLow board. |
| Seeed XIAO MG24 Sense, with headers soldered | |
| 2 × SMA bulkhead jacks with U.FL pigtails (1/4"-36 thread, 6.6 mm hole), with O-rings | O-ring on the outside |
| A 900 MHz antenna and a 2.4 GHz antenna, SMA | Antennas the modules' certification allows |

This case needs no 5 V boost board: the XIAOs run from the cell.

## Checks

The outdoor station's checks, plus: each board goes in by its joint and is held; the MG24's pins pass through their slots; the two boards don't overlap; no mount stands on a snap pin's flex fingers or runs off the plate; and the SMA bulkheads sit on flat wall with room for their nuts and pigtails. From the repository root, `cargo run --release -p amsat-kit -- check`.
