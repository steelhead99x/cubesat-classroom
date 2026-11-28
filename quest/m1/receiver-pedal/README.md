# CSC-R1P pedal prototype

## Goal

Release a reviewed fabrication package for both receive-only pedal boards and verify one assembled prototype. Use Rust for the pedal firmware and record the source and hardware revisions in every bench log.

## Plan

The routed KiCad boards are the fabrication source. Both copies imported into Flux from `bd4be2a` match their board envelopes and counts of copper layers, footprints, traces and vias. Flux flagged differences in copper connectivity, stencil apertures and stackup. Resolve those differences before using a Flux export. A prototype fabricated from reviewed KiCad files can proceed independently of the Flux comparison.

The Rust console implements panel checks, bounded PLL tests and a muted external-clock I²S bus diagnostic. Software tests and target builds are recorded separately from bench measurements.

## Related

- [Complete the prototype evidence record](/quest/m1/receiver-pedal-prototype.md)
- [Write the CSC-R1 Rev A bring-up firmware](/quest/m1/receiver-rev-a-firmware.md)
