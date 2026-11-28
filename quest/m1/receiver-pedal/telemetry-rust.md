# [M] Stream pedal status to the Rust station

## Goal

Connect physical pedal status and receiver readings to the existing station receiver contract.

## Plan

Reuse the receiver schema for tuning, lock, level and source identity. Define a bounded serial format, report unavailable values explicitly and test the normal station ingest path with fixture records, malformed lines and reconnects. Preserve simulation and hardware evidence labels.

## Required

- [Rust receive audio](/quest/m1/receiver-pedal/rx-audio-rust.md) - supplies the physical audio status and signal readings.
- [Rust tuning and display](/quest/m1/receiver-pedal/tuning-controls-rust.md) - supplies receiver state and lock transitions.
