---
title: Make first contact
patch: First contact
order: 1
summary: Ask the practice CubeSat for a sensor reading and record it with its units.
minutes: 5
skill: Measurement and units
beginner:
  steps:
    - Press **Ask for a reading**.
    - Watch the mission log. Your question goes up to the satellite, and its answer comes back down.
    - Write down the three numbers you get. Copy the little letters after each number too, like °C or V.
  check: You have three numbers, and each one has its unit next to it.
  discuss: If a friend wrote "temperature 20" with no unit, what might you guess wrong?
intermediate:
  steps:
    - Press **Get sensor reading** and note the time the command was sent.
    - Record temperature (°C), bus voltage (V) and light (lx), plus the reply time in milliseconds from the log.
    - Take two more readings. Is the reply time always the same?
  check: Your table has three readings, each with units and a reply time in ms.
  discuss: What could make one reply slower than another on a real radio link?
professional:
  steps:
    - Send a sensors request. The log shows the command published on the `classroom/commands` broadcast and the reply group on `exchange.v1`.
    - Open **Raw telemetry** and record the `sequence`, `captured_unix_ms` and `source` fields along with the three values.
    - Take three readings and compute the mean round-trip time and its spread.
  check: You logged three JSON samples with sequence numbers and a mean RTT with its range.
  discuss: The reply carries a capture timestamp from the payload clock. What has to be true about the two clocks before you trust an age calculation?
---
