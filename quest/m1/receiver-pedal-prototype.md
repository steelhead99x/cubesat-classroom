# [L] Verify a CSC-R1P pedal prototype

## Goal

Take both routed CSC-R1P boards through fabrication review and bring a receive-only pedal through its first hardware bench checks. Use Rust for new pedal firmware. Record the board and firmware revision with each measurement.

## Plan

1. Import the main and control boards into Flux as separate projects. Compare copper layers, drills, slots, component orientation and nets with the KiCad originals. Record import discrepancies. Export fabrication files from KiCad and have an independent reviewer check both layouts, oscillator selection, power isolation, BOM and proposed stackup.
2. Confirm parts availability and dry-fit the [panel parts and wiring](/hardware/sdr-receiver/pedal/panel-bom.md). Verify the enclosure, standoffs, mated headers, footswitch clearance and isolated DC jack before releasing either board for assembly.
3. Run the [Rust console](/firmware/csc-r1p-bringup/README.md) on the assembled boards. The console now covers startup, switches, pots, LED and a 128×32 OLED border test. Record supply current, rail voltages and panel responses. Current evidence is host tests and an ESP32-S3 release build.
4. Implement Rust PLL tuning and lock detection, then ADC and DAC audio, encoder tracking, live OLED status and station telemetry. Confirm PLL lock on both bands before enabling audio. Verify receive audio and telemetry with a shielded test signal.
5. Measure the 5 V switching frequency and check for desense at the intended receive frequencies. Keep the raw bench log with the board and firmware revisions.

## Waiting

- Maintainer (action): Record the independent PCB review, enclosure dry fit and assembler stackup confirmation before releasing the CSC-R1P for fabrication.
- Maintainer (PC): Connect a CSC-R1P prototype and XIAO ESP32-S3 for the first Rust console bench run.

## Related

- [Write the CSC-R1 Rev A bring-up firmware](/quest/m1/receiver-rev-a-firmware.md) - related receiver blocks and tests.
