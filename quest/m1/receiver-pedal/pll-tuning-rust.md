# [M] Add bounded receive tuning to the Rust PLL driver

## Goal

Turn the two fixed ADF4351 bench presets into a driver that tunes supported receiver channels and keeps the signal path muted until lock is established.

## Plan

Choose exact receive windows and tuning steps for both bands. Compute the divider fields from the 25 MHz reference and verify them against the datasheet and existing presets. Test band edges, invalid frequencies, carry at integer boundaries, lock loss, and safe shutdown. Keep the measured lock and output-level checks on the assembled prototype.
