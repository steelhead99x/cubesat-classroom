# [M] Measure a 1U solar face

## Goal

Measure an assembled silicon [1U solar face](/hardware/solar-face/README.md) in real sunlight. Find its open-circuit voltage, its short-circuit current and the load where it makes the most power, then compare them with the desk figures on the board's page. Label every result "hardware bench".

## Plan

**Time:** 60 minutes on a sunny day, around midday. **Bring:**
- an assembled silicon (`si`) solar face with a JST PH 3-pin lead
- a multimeter with a mA range
- the INA226 logger from the [receiver bench kit](/hardware/sdr-receiver/bench/README.md), or a second multimeter
- load resistors: 100 Ω, 68 Ω, 47 Ω, 33 Ω and 22 Ω, each rated 1 W or more
- a 10 kΩ resistor, for reading the thermistor
- a piece of card, for shading one row

A mentor assembles the cells (reflow or hot air) and checks the board for bridges before anyone connects it. Nothing on this board can shock you, but a shorted face gets warm in full sun. Never connect a battery straight to the face: use a charger made for solar input.

### Steps

1. **Point it at the Sun.** Lay the face flat on a table, or tilt it until its shadow is smallest. Write down the time and the sky: clear, hazy or broken cloud.
2. **Open circuit.** Measure the voltage from J1 pin 1 to pin 2 with nothing connected. That's after the blocking diode. Then measure TP3 to TP2, which is before the diode. Is the difference what you expect from a Schottky diode?
3. **Short circuit.** Put the meter on its mA range across pins 1 and 2 for a few seconds only. Write down the current.
4. **Power curve.** Connect each load resistor in turn, from 100 Ω down to 22 Ω. For each one, write down the voltage and the current, and work out the power (voltage × current). Which load gives the most power?
5. **Shade one row.** With the best load connected, cover one row of cells with card. Write down the current. Then cover a single cell instead. Talk it over: why does one shaded cell cost a whole row?
6. **Temperature.** Connect the 10 kΩ resistor from 3.3 V to J1 pin 3, and measure the voltage at pin 3. Work out the thermistor's resistance from the voltage divider, and look up its temperature in the Murata NCP18XH103F03RB table (about 10 kΩ at 25 °C). Repeat step 2 after ten minutes in the sun. Did the open-circuit voltage go up or down as the face warmed?
7. Share a table of load, voltage, current and power, plus the open-circuit and short-circuit readings, the temperature, the sky and the time. Include the board's build number.

### Check your work

- The open-circuit voltage is in the same range as the 6.21 V desk figure. Real sunlight is rarely 1000 W/m², so expect less.
- The power at the best load is below the 0.74 W desk figure, and you can say why: the Sun's angle, haze, the diode's drop, cell temperature.
- Every number is labelled "hardware bench", with the date, the time and the sky.

### Talk it over

- The flight version of this board carries two triple-junction space cells instead of 24 silicon ones. Why would a spacecraft pay so much more for its cells?
- In orbit, sunlight is about 1361 W/m² and never hazy, but the cells get hot and slowly lose power to radiation. Which of your measurements would change?

## Required

- [Compare radio power: 2.4 GHz, 5 GHz and HaLow](/quest/m0/receiver-radio-power.md) - the crew has used the INA226 logger and a meter's mA range.
