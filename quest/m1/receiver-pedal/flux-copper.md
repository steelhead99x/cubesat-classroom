# [M] Reconcile PCB copper and drill exports in Flux

## Goal

Determine whether the two Flux imports preserve the KiCad pad connectivity, drilled geometry and copper. Record each discrepancy with its net, layer and board revision.

## Plan

The drill comparison found four component holes reduced from 0.75 mm to 0.70 mm. The six control-board routed slots match at 0.01 mm precision. See the [fabrication checks](/hardware/sdr-receiver/pedal/fabrication-checks.md). Overlay the matched exports for F.Cu, In1.Cu, B.Cu and plated drills first. Investigate the main L5 pad 2 to U7 ground airwire and bottom-layer 3V3_PLL via sliver. Confirm the remaining copper layers, fills, pads, nets, orientation, both outlines and the main footswitch slot. The fresh imports at f26d445 reproduced the four undersized holes, the U7 ground airwire and copper differences on both boards. Record layout or exporter fixes and rerun the comparison before trusting a Flux CAM export. Use reviewed KiCad CAM for the prototype.

## Related

- [Compare the main board stencil](/quest/m1/receiver-pedal/flux-stencil.md)
