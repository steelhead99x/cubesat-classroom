# [M] Add bounded receive tuning to the Rust PLL driver

## Goal

Turn the two fixed ADF4351 bench presets into a driver that tunes supported receiver channels and keeps the signal path muted until lock is established.

## Plan

The Rust console accepts 1 kHz steps in the documented 144–146 MHz and 435–438 MHz windows. Host tests decode the programmed frequencies and confirm integer-boundary lock timing. The bench command powers down on timeout, loss of lock or completion. Compute the divider fields from the 25 MHz reference and verify them against the datasheet and existing presets. Test band edges, invalid frequencies, carry at integer boundaries, lock loss, and safe shutdown. Keep the measured lock and output-level checks on the assembled prototype.
