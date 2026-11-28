# [S] Verify the Flux paste apertures

## Goal

Account for all 17 main-board paste-only apertures in the Flux and KiCad F.Paste outputs.

## Plan

A registered overlay of the actual F.Paste exports found four separate KiCad apertures each at U10 and U12, and nine at U13. The Flux export renders one continuous opening at each site. Use the KiCad stencil for the prototype. The f26d445 reimports also added back-side paste openings on both boards that are absent in KiCad. Confirm the KiCad aperture pattern with the assembler and repeat the comparison after exporter corrections. See the [fabrication checks](/hardware/sdr-receiver/pedal/fabrication-checks.md).

KiCad IPC-2581C preserves all 17 exposed-pad window centers and represents no back-side paste openings. Recheck both paste layers against the KiCad stencil after importing by that path, including aperture shapes and sizes.
