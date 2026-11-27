---
name: printed-fit-coupons
description: Use before printing a full part that has a new snap, hook, thread, keyed fit or board mount: make a few-gram test coupon from the same code, with a fit ladder, then record the printed result and set the design's fit number from it.
---

# Printed fit coupons

A coupon is the joint on its own, a few grams of plastic, made by **the same functions** that make the real part. A fit checked on it holds for the real part. This is how CubeSat Classroom tunes fits on real classroom printers instead of guessing.

## Make the coupon

- **Same code, same geometry.** Cut it from the real part (the real part intersected with a box, plus a solid rim), or build it from the same joint functions. Never redraw it by hand.
- **The smallest base that holds the joint.** Pocket out everything else. Aim for 1–5 g per joint.
- **Real hardware where it matters:**
  - A board mount takes the actual board: the Nano coupon is the plate's Nano mount, sized for the bare board.
  - A thread coupon takes the real screw.
  - Size it from the real part. A 20 mm generic hook coupon was too wide for a 17.78 mm Nano.
- **Marks, not instructions:** raised dots for fit steps, an arrow for which way a board goes.
- **A label tile saying what it tests:** a thin flag beside the joint (never on it, and clear of every mating part, board and insertion path) with the logo and two or three short raised lines: what it tests, which board or part it's for, how to try it ("BOARD MOUNTS / HALOW + XIAO MG24 / CLICK EACH BOARD IN"; "FIT 1 / TIGHTEST"). Lettering 4 mm DejaVu Sans Bold, checked like the logo: strokes ≥ 0.65 mm and gaps ≥ 0.8 mm, so it prints on a 0.6 mm nozzle and a 0.4 mm one. See `crates/amsat-kit/src/labels.rs`.
- **Infill:** set flexing coupons to 100% infill in the print file.

## Fit ladder

For any clearance that depends on the printer, print the same joint at several fits:

- **Steps:** three tabs at 0.1, 0.2 and 0.3 mm, marked 1, 2 and 3 dots.
- **Move everything together:** walls and teeth both move by the same step, as one number.
- **Include the design's own fit.** A check should fail if the ladder leaves it out.
- **Threads get the same treatment:** the design's thread and one 0.15 mm looser (marked).

## Test and record

1. Print the coupons in the material of the real part. A fit tuned in PLA must be rechecked in ASA or PETG.
2. Test like a student would: line it up, press until it clicks. Then check that it doesn't wobble, rattle or need forcing, and that it releases the way it should.
3. For snaps, pull-test with a luggage scale against the retention the design needs (for example ≥ 8 N per snap pin for a plate on three pins).
4. Record the result as a **hardware bench** result: date, printer, material and which step fit best. Put it in the part's README.
5. **Set the design's fit number** from the result, regenerate, and keep the ladder including that fit.

## Then design around it

- Make the real part's joint exactly the coupon's that fit best.
- If a check then fails, change the least-tested thing. Example: fingers 0.5 mm longer to keep strain under the limit, instead of changing the tested fit.
- Say in the README which differences remain between the coupon and the part.
