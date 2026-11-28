# [L] Verify the pedal receive chain on the bench

## Goal

Verify end-to-end reception, line output and station status on both bands with repeatable bench evidence.

## Plan

Use a shielded test signal. Run the muted FM RF diagnostic on both bands. Record BCLK, FSYNC, slot order, audio tone and underrun behavior before enabling analog listening. Check tuning, decoded audio, clipping, level controls and telemetry on both bands. Measure the buck switching frequency and compare noise or desense with USB and display activity. Record the setup, board and firmware revisions, scope captures and raw logs.

## Required

- [Assembled pedal bring-up](/quest/m1/receiver-pedal/first-power-up.md) - verifies the rails, panel, LO and control connections.
- [Pedal station telemetry](/quest/m1/receiver-pedal/telemetry-rust.md) - supplies the tested controls, audio path and telemetry.
