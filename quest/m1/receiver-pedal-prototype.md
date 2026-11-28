# [L] Verify a CSC-R1P pedal prototype

## Goal

Bring one routed CSC-R1P pedal from design review through its first hardware bench checks. Record the hardware revision with each measurement.

## Plan

1. Have an independent reviewer check the copper, power isolation, oscillator selection and fabrication output for both boards. Verify the [panel parts and wiring](/hardware/sdr-receiver/pedal/panel-bom.md) on a dry fit before drilling or ordering assembled boards.
2. Follow the [Rust console bring-up](/firmware/csc-r1p-bringup/README.md). Record supply current, rail voltages and the actual switch, pot and LED responses.
3. Implement and test the remaining receiver stages in Rust. Check PLL lock in both bands, then ADC clocks and muted DAC output. Verify audio and station telemetry with a shielded test signal.
4. Record the 5 V switching frequency and check for receiver desense at the intended frequencies. Save the raw bench log with the board and firmware revisions.

## Related

- [Write the CSC-R1 Rev A bring-up firmware](/quest/m1/receiver-rev-a-firmware.md) - related receiver blocks and tests.
