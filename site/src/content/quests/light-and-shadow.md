---
title: Light and shadow
patch: Eclipse
order: 5
summary: Take readings over a minute and explain why the light level sometimes drops to zero.
minutes: 10
skill: Patterns in data
beginner:
  steps:
    - Make a table with two columns, time and light.
    - Ask for a reading every 10 seconds for one minute.
    - Circle the rows where light is 0 lx.
  check: Your table has at least six rows, and you circled the zeros.
  discuss: Where could a satellite go that would make its light sensor read zero?
intermediate:
  steps:
    - Record time, light (lx) and temperature (°C) every 5 seconds for one minute.
    - Find how long the light stays on and how long it stays off.
    - Plot light against time on paper or in a spreadsheet.
  check: You measured the on and off durations and drew the pattern.
  discuss: The simulated cycle is 30 s. A real low-orbit pass around Earth takes about 90 minutes. What changes?
professional:
  steps:
    - Sample once per second for 40 s and record `sequence`, light and temperature.
    - Derive the light rule from the sequence numbers and confirm it against `Sample::synthetic` in the station source. In this demo the sequence is the time in seconds, like the station's `simulate` command; the installed classroom station counts requests instead.
    - Describe how you would replace the synthetic eclipse with one computed from an orbit model.
  check: You stated the rule (lit when sequence mod 30 is below 20) and verified it against the data.
  discuss: What inputs would an eclipse model need, and where would you get them for a real satellite?
---
