# [S] Close the pedal BOM and oscillator selection

## Goal

Produce an orderable board and panel BOM with approved substitutions and DNP rows.

## Plan

Farnell lists X1G0054210313 under order code 3597149. Confirm the full Epson X1G005421031327 suffix and availability for Y2 and verify each offered replacement's footprint, pinout, supply and clipped-sine output specification. Keep the loaded REFIN amplitude and PLL lock as bench checks. Verify connector part numbers and assembler sourcing or hand-fit decisions.

The KiCad quote review reconciled 213 fitted top SMT placements on the main board after marking L2 DNP, and 22 on the control board. The generated BOM covers all 235 fitted SMT placements with named part candidates or purchasing specifications. The assembler still needs to return exact MPNs for specification rows and confirm stock, voltage derating, RF substitutions and orderable suffixes. The prototype assembly scope leaves five main-board and eight control-board through-hole parts to hand fit. The control-board stacking socket is on the underside. TP3 is a plated test point outside the SMT BOM.
