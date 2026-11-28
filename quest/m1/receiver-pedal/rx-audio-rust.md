# [L] Implement the pedal receive-audio path in Rust

## Goal

Build Rust firmware that captures the receiver ADC samples, produces controlled line-level output and keeps the DAC muted when clocks or data are unavailable.

## Plan

Use Y1 as the ADC master clock source. The ADC supplies BCLK and FSYNC to the XIAO and DAC. Implement the ADC register configuration, XIAO I²S capture and output, level control and fault mute. Check the TI controller-mode edge timing and confirm sample alignment, Q polarity and bounded buffering with host fixtures. Build for the XIAO target and document the scope checks to run on the prototype.
