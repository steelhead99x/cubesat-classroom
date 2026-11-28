# [M] Approve the fab stackup and RF return paths

## Goal

Record a named fabricator's four-layer stackup and verify the receive-board RF route assumptions against it.

## Plan

Resolve the 1.30 mm Flux report against the 1.6 mm nominal KiCad board setting. The published 7628 candidate models at 50.1 Ω with the current 0.35 mm traces. See the [desk calculation](/hardware/sdr-receiver/pedal/fabrication-checks.md#rf-stackup-candidate). Record copper thickness, the F.Cu-to-In1.Cu dielectric spacing and dielectric constant. Check the 0.35 mm RF routes against the approved build and inspect the filled ground under the J2 launch and around U7. Record the drill and via requirements for the quote.
