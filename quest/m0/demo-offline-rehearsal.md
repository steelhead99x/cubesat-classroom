# [S] Rehearse the demo without internet

## Goal

A second presenter can run the station and recover from a stopped process without internet access.

## Plan

1. Build the project with `cargo build --locked` while you have internet. On the demo computer, launch it with `cargo run --locked --offline -- classroom` and open the printed loopback address.
2. At a rehearsal you control, disconnect from the internet. Follow the [demo run sheet](/docs/demo-run-sheet.md) through sensor, camera, and contact recovery. Stop the station and relaunch it once.
3. Record the OS, how you started it, time to the first reading, and whether the camera and recovery worked. Have another presenter try the notes. Bring the local build and CAD drawing to the demo if venue internet is uncertain.
