---
title: Catch the Pass
patch: Packet catcher
order: 6
summary: Point a ground antenna at a passing satellite and choose a radio setting that still works when it is low.
minutes: 15
skill: Tracking and radio links
tool:
  name: Catch the Pass
  href: /games/catch-the-pass/#step-track
  note: A game. Everything you see is a desk simulation of a LoRa CubeSat at 915 MHz, sped up so a pass fits in under a minute.
beginner:
  steps:
    - Open the game and play level 1, **First light**. Keep the circle on the satellite. The signal bar turns green when you are catching packets.
    - Write down your packets and stars.
    - Play level 2, **Low pass**. The satellite stays close to the ground. Try **Fast**, then try **Far**. Which one caught more?
  check: You have two scores written down, one for each level, and you know which radio setting worked better on the low pass.
  discuss: The satellite is farther away when it is low in the sky. Why would that make its signal weaker?
intermediate:
  steps:
    - Play **Low pass** twice. Use **Fast** the whole time the first run and **Far** the whole time the second. Record packets caught each time.
    - Play level 4, **Into the shadow**. Watch the battery after the satellite enters Earth's shadow. Use **Sending** to let it rest when the charge gets low.
    - Make a table of level, radio setting, packets caught and lowest battery percent.
  check: Your table has at least three runs, and you can say in one sentence when Far beats Fast.
  discuss: Far reaches weaker signals but each packet takes longer to send. What does a slow packet cost a satellite that is in the shadow?
professional:
  steps:
    - Read the `SF` table and `packetEnergy` in `site/src/games/catch-the-pass/logic.mjs`. Fast, Middle and Far are LoRa spreading factors 7, 9 and 12.
    - From `minSnr`, work out how many dB of extra margin Far gives over Fast. From `airtime`, work out how many times more energy each Far packet costs.
    - Play **Low pass** and, near the horizon, switch settings and watch which one the signal bar can still decode. Compare what you see with your numbers.
  check: You have the margin in dB and the energy ratio, with working, and one observation from the game that agrees or disagrees with them.
  discuss: The game shortens SF12 airtime so a pass fits in a minute. Which of your conclusions would change with real airtime, and which would not?
---
