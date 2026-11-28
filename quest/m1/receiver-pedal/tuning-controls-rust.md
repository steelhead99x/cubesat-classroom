# [M] Implement Rust tuning controls and live display

## Goal

Connect the panel encoder, band toggle, gain and level settings, ENGAGE switch and OLED to receiver state.

## Plan

Apply validated band and frequency commands through the Rust PLL driver. Make lock and muted status visible. Test encoder reversals, invalid edges, band boundaries and lock timeout with host fixtures. Choose the production sampling interval from the physical diagnostic results when available and label any interim timing as a default.

## Required

- [Rust PLL tuning](/quest/m1/receiver-pedal/pll-tuning-rust.md) - supplies bounded frequency and lock controls.

## Related

- [Assembled pedal bring-up](/quest/m1/receiver-pedal/first-power-up.md) - supplies measured encoder behavior and lock evidence.
