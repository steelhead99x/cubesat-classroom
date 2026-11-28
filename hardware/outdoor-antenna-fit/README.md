# Outdoor case fit prototypes

These bench parts are shared by the HaLow/XIAO and Uno/Nano outdoor cases. Use the same nozzle and filament you plan to use for the case.

## Battery and connector

- **Stock pocket.** This reproduces the existing pocket's receiving surfaces and wire exit. Try an unpowered cell or a dummy with the same dimensions. Note movement along its length, width and thickness.
- **Battery snap fit.** This stand-alone prototype uses the classroom battery plate's end catches. Check that both catches engage and that the wire leaves beside the catch. A thin cell can still lift before it meets a catch. Keep the cell unpowered and stop if installation needs force or puts pressure on its pouch.
- **HaLow lead plate.** The XIAO mounting plate has a 12.6 × 5 mm rounded slot aligned with the pocket's centre cable exit. The original edge notch remains available for other cables.
- **CubeSat outdoor service plate.** The Uno-side obstructed snap position is a clearance hole for its existing head. The other two snaps are retained. Lift using the two raised grips with the boards installed. The two-snap hand estimate is about 18 N, below the classroom payload target of about 24 N with margin. Use this plate for bench serviceability testing.

The battery snap fixture is a fit test for future case revisions. The current cases use the stock pocket and the plate above it. Use the actual cell and connector dimensions and the measured free play to choose the final mount.

## Glove latch and seal

The closed cup uses two opposed release pads, 1.8 mm release slots, a thicker lid skin and a continuous radial seal inside the latches. Its O-ring uses a 2.5 mm cross section and an approximate 65.6 mm inside diameter. With the empty cup, check that both latches click, then press the opposed pads inward and lift with the gloves you plan to use for the bench trial. Check for water ingress on the separate cup with absorbent paper inside before changing either full-size case. The prototype has no EVA, pressure or ingress rating.

## Antenna gauges

- [Standard SMA fit plate](stl/outdoor-standard-sma-fit.stl) has 6.6 and 7.0 mm holes.
- [Large antenna fit plate](stl/outdoor-large-alfa-fit.stl) has 12, 15, 18, 21 and 24 mm holes. ALFA sells multiple 5 dBi models with different connectors. Try the actual bulkhead and check that its washer and nut seat flat on the 2.4 mm plate. Record the smallest clean fit before cutting the larger port into a case.

Both gauges are generated together by the standard CAD build. Their raised mark and hole sizes are visible in a single filament. The gauges print flat. Recheck the selected diameter on a vertical wall before updating the sealed case.

```sh
cargo run --release --locked -p amsat-kit -- outdoor-battery-fit /tmp/battery-fit
cargo run --release --locked -p amsat-kit -- outdoor-glove-seal /tmp/glove-seal
cargo run --release --locked -p amsat-kit -- outdoor-antenna-fit /tmp/antenna-fit
```
