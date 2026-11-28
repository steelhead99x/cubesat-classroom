# [L] Implement the pedal receive-audio path in Rust

## Goal

Build Rust firmware that captures the receiver ADC samples, produces controlled line-level output and keeps the DAC muted when clocks or data are unavailable.

## Plan

The bounded `ADC CLOCK TEST` configures 192 kHz FSYNC and 12.288 MHz BCLK while the DAC stays muted. Use Y1 as the ADC master clock source. The ADC supplies BCLK and FSYNC to the XIAO and DAC. Implement the ADC register configuration, XIAO I²S capture and output, level control and fault mute. Check the TI controller-mode edge timing and confirm sample alignment, Q polarity and bounded buffering with host fixtures. The pinned esp-hal I²S builder drives clocks. The bounded AUDIO BUS TEST uses the ESP32-S3 input matrix for external-clock RX and TX while XSMT stays low. Verify the DMA captures and transmit pattern on the prototype before reusing that setup for continuous demodulation. The bounded FM stream test exercises 192 kHz I/Q demodulation and duplex DMA with the DAC muted. Use a shielded signal to verify digital throughput, slot order, Q polarity and sample continuity on both bands. Qualify clock loss, DMA underrun and PLL unlock before enabling the analog output path. Finish level control and fault mute for listening mode.
