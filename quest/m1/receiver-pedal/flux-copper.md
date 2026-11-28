# [M] Reconcile PCB copper and drill exports in Flux

## Goal

Determine whether the two Flux imports preserve the KiCad pad connectivity, drilled geometry and copper. Record each discrepancy with its net, layer and board revision.

## Plan

Overlay the matched exports for F.Cu, In1.Cu, B.Cu and plated drills first. Investigate the main L5 pad 2 to U7 ground airwire and bottom-layer 3V3_PLL via sliver. Confirm the remaining copper layers, fills, pads, nets, orientation, both outlines and the main footswitch slot. Distinguish an import or checker difference from a source-board defect. Run DRC again on any source changes.

## Related

- [Compare the main board stencil](/quest/m1/receiver-pedal/flux-stencil.md)
