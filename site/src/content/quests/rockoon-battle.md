---
title: Rockoon Battle
patch: Wind reader
order: 7
summary: Launch a rocket from a balloon, see how the wind moves it, and adjust your aim before the next shot.
minutes: 15
skill: Prediction and correction
tool:
  name: Rockoon Battle
  href: /games/rockoon-wars/#step-wind
  note: A game for 2 to 4 crews on one screen or online. The wind and flight are a desk simulation, not a real rocket.
beginner:
  steps:
    - Open the game, press **Play on this computer** and pick 2 crews.
    - Before each launch, look at the wind arrow at the top. Write down which way it points.
    - Launch. Did your rocket land on the side the wind was blowing toward? Change your aim and try again.
  check: You wrote down the wind for at least three launches and what you changed each time.
  discuss: The balloon carries the rocket before it launches. How does the wind move your rocket twice?
intermediate:
  steps:
    - The wind, the hills and the ring stay the same for a whole round and change when the next round starts, so compare launches within one round.
    - In one round, record the wind (a number out of 10), then make your first launch aimed straight at the ring. Record angle, power and score (3, 1 or 0).
    - For your later launches in the same round, aim upwind and change only how far upwind. Record each launch the same way.
  check: Your table has at least four launches from one round, all with the same wind, and a sentence on whether aiming upwind helped compared with your first launch.
  discuss: Real rockoon teams wait for low winds before they launch. Using your table, explain why.
professional:
  steps:
    - Read `balloonPosition`, `releasePoint` and `WIND_ACCEL` in `site/src/games/rockoon-wars/logic.mjs`. For a wind of 10, work out how far sideways the balloon drifts before release, in game pixels.
    - In the game, keep the same angle and power for several launches within one round, so the wind and the ring stay fixed. Note the wind (shown as a number out of 10) and whether you missed left or right of the ring. Repeat in a round with a different wind.
    - The readout also shows the air pressure at the pad and at release. Use the two values to explain why rockoons launch from a balloon at all.
  check: You have the drift from the code with working, at least four launches that show the wind's effect, and a sentence about the pressure difference.
  discuss: The game rules avoid `Math.pow` and use a seeded random number generator so every screen in an online room computes the same flight. Why does that matter when one computer replays another's moves?
---
