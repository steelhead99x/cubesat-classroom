# CSC-R1P prototype fabrication checks

Desk evidence for the CSC-R1P prototype. Use the reviewed KiCad files for fabrication. The Flux comparison below used `bd4be2a`, before the input protection and passive-bias changes.

## Flux export comparison

The four-layer Gerbers and Excellon files were exported from the Flux imports and registered to archived KiCad 10.0.8 plots using translations of 48.703014 mm for the main board and 47.080538 mm for the control board in X, and −40 mm in Y. This is an export comparison with no hardware measurements.

| Check | Main | Control |
| --- | --- | --- |
| Circular drills | 182 of 184 diameters and positions match within 0.01 mm. J1 pad 1 and J8 pad 1 change from 0.75 mm to 0.70 mm. | 76 of 78 match. J2 pad 1 and J3 pad 1 change from 0.75 mm to 0.70 mm. |
| Routed drills | None in either file. The footswitch slot is on Edge.Cuts. | All six routed drills match at 0.01 mm precision. |
| Top stencil | U10 and U12 each have four exposed-pad apertures and U13 has nine in KiCad. Flux combines them into one continuous opening per device. | KiCad remains the stencil source. |

The registered raster comparison at 40 pixels/mm also shows that the Flux main In1 pour differs across the board and that the control import has less copper on all four layers. The complete stencil images differ on both boards, including regions outside the three exposed pads. Confirm the KiCad exposed-pad stencil with the assembler. The Flux ground airwire at L5 pad 2 and U7 and the bottom-layer 3V3_PLL via still need a continuity comparison. The archived KiCad results report zero unconnected items on both source boards.

## RF stackup candidate

JLCPCB's published JLC04161H-7628 build is a candidate for a quote. It uses approximately 0.2104 mm of dielectric between F.Cu and In1.Cu. With 40.64 µm outer copper and assumed Dk 4.4, the existing repository microstrip model gives 50.1 Ω for a 0.35 mm trace. The same model with the thinner 3313 spacing gives 30.9 Ω. These estimates omit solder mask, etch shape and nearby side copper. Confirm the actual material and impedance tolerance with the fab and inspect the filled J2 and U7 returns before release. [JLCPCB stackup examples](https://jlcpcb.com/blog/pcb-ground-loop-noise-reduction) · [JLCPCB calculation assumptions](https://jlcpcb.com/help/article/user-guide-to-the-jlcpcb-impedance-calculator)

## Power review

Use a center-negative isolated panel jack. Q1 blocks steady reversed input. D2 now clamps `VBUCK_IN_F` downstream of F1. The main PCB carries this connection on B.Cu, and the In1.Cu ground fill was regenerated. Verify coordination between the selected 9 V adapter's current limit, the exact fuse trip curve and the TVS limits. Check chassis bonds and measure USB VBUS and I/O backfeed with each source alone before exercising both together.

Leave main-board L2 unpopulated in the passive build. This opens the DC route through the bias network into the 5 V rail. An active bias option needs separate backfeed qualification before L2 is fitted.

The revised main and control boards report zero enabled DRC violations and zero unconnected items in KiCad 10.0.8. The repository checks report zero schematic-to-design and zero PCB pad/net mismatches on 230 main and 33 control footprints. KiCad's optional parity output reports local net naming and metadata differences. Confirm the vendor stackup, stencil and mechanical fit before release.

## Y2 sourcing

The [panel BOM](panel-bom.md) records Epson X1G005421031327. Farnell lists the matching X1G0054210313 family under order code 3597149. Confirm the full suffix, stock and shipment terms with the assembler. Measure loaded PLL REFIN swing and lock before accepting a substitution. [Farnell listing](https://es.farnell.com/epson/x1g0054210313/tcxo-25mhz-0-5ppm-2-5mm-x-2mm/dp/3597149)
