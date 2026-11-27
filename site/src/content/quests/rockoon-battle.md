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
    - Pick one crew and a round with wind. For that crew’s first launch, aim straight at the ring and record the wind, angle, power, rocket and score (3, 1 or 0).
    - Use the same power and rocket for that crew’s two remaining launches. Change only the angle to aim a little upwind.
    - Compare the three scores. If the launch pad shifted during the round, note that before comparing.
  check: Your table has three launches by the same crew in one round, and one sentence about what changed when you aimed upwind.
  discuss: Real rockoon teams wait for low winds before they launch. Using your table, explain why.
professional:
  steps:
    - Read `balloonPosition`, `releasePoint` and `WIND_ACCEL` in `site/src/games/rockoon-wars/logic.mjs`. For a wind of 10, work out how far sideways the balloon drifts before release, in game pixels.
    - Compare the dashed balloon path with your calculated drift. Check a second round with a different wind. Record the wind strength and direction for each observation.
    - Record the simulated air pressure at the pad and at release. Describe how it changed as the balloon climbed.
  check: You have the drift from the code with working, two wind observations, and a sentence about the pressure difference.
  discuss: The game rules avoid `Math.pow` and use a seeded random number generator so every screen in an online room computes the same flight. Why does that matter when one computer replays another's moves?
---
