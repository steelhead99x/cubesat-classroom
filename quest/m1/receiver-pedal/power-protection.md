# [M] Review pedal input protection and chassis wiring

## Goal

Close the 9 V protection, USB backfeed and chassis return findings before ordering or wiring the pedal.

## Plan

D2 now clamps VBUCK_IN_F downstream of F1. L2 is do-not-populate in the passive assembly to open the bias-feed DC path. Confirm Q1 polarity and gate clamp behavior. Select the exact PTC and adapter and qualify sustained fault behavior against the clamp rating and fuse trip curve. Check the D6 VBUS path, possible I/O backfeed, the isolated center-negative jack and the intended SMA, audio and standoff bonds. If the source changes, regenerate both the relevant checks and fabrication files. Record physical measurements during prototype bring-up.
