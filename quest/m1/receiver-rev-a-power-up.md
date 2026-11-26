# [M] Bring up CSC-R1 power and control

## Goal

Power a new CSC-R1 Rev A kit for the first time, one rail at a time. Prove the reverse-polarity protection, measure each rail's current, and find the I²C devices. No radio blocks are driven yet. Label results "hardware bench".

## Plan

**Time:** 60 minutes. **Bring:**
- an assembled Rev A kit
- a multimeter with a mA range
- a 5 V supply: a bench supply with a current limit, or a USB supply with a 100 Ω resistor in series for the first power-on (it can't pass more than 50 mA)
- a XIAO ESP32-S3 with an I²C scanner sketch

A mentor checks the board for solder bridges and checks every connection before power is applied. Follow the [bring-up order](/hardware/sdr-receiver/README.md#bring-up-order).

### Steps

1. **Look first.** No XIAO, all six current jumpers out. Inspect the LTC5584 and ADF4351 pins under magnification for bridges. Measure the resistance from +5V to GND: it should not be near 0 Ω.
2. **Backwards, on purpose.** Connect the supply to the screw terminal the wrong way round, through the current limit or the 100 Ω. Measure the current. It should be about zero, because the AO3401A blocks it. Disconnect.
3. **Right way round.** Apply 5 V with the current limit. Write down the current with all jumpers out.
4. **One rail at a time.** Fit `3V3_DIG`, then each other jumper one at a time. Measure each rail's current with the meter across the jumper pins before fitting the shunt. The demodulator and LNA stay off until they're enabled, so expect small currents for now.
5. **Find the chips.** Fit the XIAO and run the scanner. Expect 0x20 (expander) and 0x60 (DAC). The ADC (0x4C) only answers after the [firmware](/quest/m1/receiver-rev-a-firmware.md) releases its shutdown line.
6. **Warm check.** After 10 minutes, touch-test or use an IR thermometer on each regulator. Write down anything hot.
7. Share a table of rail, current and temperature, with the board's serial or build number and the date.

### Check your work

- Any current over 50 mA with all jumpers out means stop: look for a short before going on.
- Each rail's current is written down before its shunt is fitted.
- All results are labelled "hardware bench" with the kit's build number.

## Required

- [First light with the RTL-SDR Blog V4](/quest/m0/receiver-v4-first-light.md) - the crew has a working reference receiver for the next quests.
