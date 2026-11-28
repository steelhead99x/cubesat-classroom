# [M] Review pedal input protection and chassis wiring

## Goal

Close the 9 V protection, USB backfeed and chassis return findings before ordering or wiring the pedal.

## Plan

D2 now clamps VBUCK_IN_F downstream of F1. L2 is do-not-populate in the passive assembly to open the bias-feed DC path. Confirm Q1 polarity and gate clamp behavior. Select the exact PTC and adapter and qualify sustained fault behavior against the clamp rating and fuse trip curve. Check the D6 VBUS path, possible I/O backfeed, the isolated center-negative jack and the intended SMA, audio and standoff bonds. If the source changes, regenerate both the relevant checks and fabrication files. Record physical measurements during prototype bring-up.

For the adapter decision, compare its output limit and recovery behavior against the PTC's hold, trip and time-to-trip curves at the expected enclosure temperature. Include a supply stalled below the guaranteed trip current. D2's SMAJ rating covers a time-limited surge. Sustained overload needs to clear within the thermal limit for the actual footprint, or the design needs an overvoltage disconnect. Record normal and fault voltage at VBUCK_IN_F against the lowest applicable rating across all installed input components. [SMAJ datasheet](https://www.littelfuse.com/assetdocs/littelfuse_tvs_diode_smaj_datasheet?assetguid=13c2a823-03b8-4d1f-9ddc-9b44670aed9d)

Start with a current-limited bench supply and the XIAO disconnected. Record normal startup, rail stability, clamp leakage and temperature. For USB isolation, measure VBUS reverse current and the unpowered rail voltages with each supply alone. Check the mounted jack, audio sleeve and antenna shell against the intended chassis bonds. Combine power sources only after the separate checks. Record the actual supply and board revision. Leave L2 absent for the passive build.
