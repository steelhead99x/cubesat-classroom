# [M] Bring up the assembled pedal on the bench

## Goal

Record first-power-up evidence for both boards, including the Rust panel, PLL and ADC checks.

## Plan

Follow the [Rust console guide](/firmware/csc-r1p-bringup/README.md). Record supply current and rail voltages before fitting the XIAO. Exercise switches, pots, LED and OLED. Record slow and fast encoder counts in both directions. Verify DC and USB backfeed behavior, then measure the LO monitor and verify MUXOUT lock and shutdown at both presets. Measure the loaded 25 MHz reference and run `ADC CHECK` and `ADC CLOCK TEST`. Verify the ADC clock rates and alignment with a scope. Run `AUDIO BUS TEST` to capture the ADC slots and the XIAO's transmit pattern with the DAC muted. Save the raw log and revision IDs.

## Required

- [KiCad fabrication package](/quest/m1/receiver-pedal/review-fab-package.md) - furnishes the approved board revision for assembly.

## Waiting

- Maintainer (PC): Make an assembled CSC-R1P, non-Sense XIAO ESP32-S3 and the approved bench setup available for first power-up.
