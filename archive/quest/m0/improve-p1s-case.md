# [S] Review a case improvement against the fit evidence

## Goal

Turn one measured problem into one change that someone else has reviewed.

## Plan

Pick one problem from the fit test. Write down what you measured, what you think caused it, and how you'll know if a fix worked. Decide whether to change a slicer setting or the [archive CAD](/crates/cubesat-cad/src/archive.rs). Ask another builder to review the change. If the CAD changes, regenerate the STL and 3MF files as the [archive build instructions](/hardware/ARCHIVE.md#regenerate-the-files) describes. If nothing needs fixing, record that instead.

## Required

- [Case assembly fit test](/archive/quest/m0/test-p1s-case-fit.md) - provides the measurements to improve on.
