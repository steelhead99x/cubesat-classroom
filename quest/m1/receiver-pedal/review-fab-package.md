# [M] Review and release the KiCad prototype package

## Goal

Produce a versioned fabrication package for both boards with an independent sign-off before an order.

## Plan

Export the four-layer Gerbers, solder paste, plated and nonplated drill data, BOM and placement files from KiCad. Verify outline slots, exposed-pad apertures, the RF stackup and the reviewed protection changes. Record the source commit, DRC results and reviewer decisions.

## Required

- [Fab stackup and RF return paths](/quest/m1/receiver-pedal/fab-stackup-rf.md) - supplies the fab build and RF assumptions.
- [Input protection and chassis wiring](/quest/m1/receiver-pedal/power-protection.md) - closes electrical release findings.
- [Pedal stack and panel dry fit](/quest/m1/receiver-pedal/panel-fit.md) - confirms the enclosure and harness fit.
- [Orderable pedal BOM](/quest/m1/receiver-pedal/orderable-bom.md) - supplies purchasable parts and assembly decisions.

## Waiting

- Maintainer (action): Obtain an independent, dated review of both CSC-R1P KiCad boards and sign off the fabrication package before ordering.
