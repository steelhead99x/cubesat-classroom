---
name: cubesat-printed-design
description: Use when designing, changing or reviewing any 3D-printed part for CubeSat Classroom (frames, plates, board or battery mounts, enclosures, ground-station or outdoor cases, coupons, balloon or rocket hardware) or a fork of one.
---

# CubeSat Classroom printed-part design

Every CubeSat Classroom 3D design follows these rules, in every repository and fork. In the cubesat-classroom repository the full guide is `hardware/DESIGN-GUIDE.md`. Read it there if it exists: it is the source, and this skill is its summary.

## The three rules, in priority order (safety always first)

1. **No loose pieces wherever possible, and no screws.** Every snap, hook, lip, pin and latch is part of a plate, the frame or the case, and every joint comes apart and goes back together. If a part must be separate (an O-ring), make it captive once built, and say why it can't be avoided.
2. **Rocket-launch ready and durable.** Joints hold at the 20 g design load with margin. Nothing that flexes or carries load is under 2 mm. Every snap holds its part tight, so nothing rattles. Outdoor parts are printed in ASA or PETG, not PLA.
3. **Easy enough for kids.** A 13-year-old can build it on a budget printer, without tools. Parts go together one way round, and a click says each step is done. Adult steps (soldering, battery wiring) are marked as mentor steps.

## Joints to reach for, in this order

| Joint | For | Going together |
| --- | --- | --- |
| Keyed snap pin | Plates onto columns | A pin with two flats and a 45° head in a keyed slot: rigid walls on the flats, two straight flex fingers either side with teeth under the head. Drops on one way, centred; clicks. |
| Lips + snap hooks | Circuit boards | Tuck one edge under fixed lips, swing down, and hooks on in-plane fingers click over the other edge. Hook underside at 10°, under PLA's ~17° friction angle, so it locks itself. |
| Push-down end hooks | Thick parts: batteries | Press straight down between side guides until a hook clicks over each end. Tuck-and-swing fails for thick parts, because their bottom edge swings into the lips. |
| Lid latches | Lids, sealed ones too | Press the lid down: an in-plane finger cut into each lid edge bends over a lug on the rim (45° lead-in) and clicks under it. Thumb each finger outward to lift it off. |

No screws, inserts or nuts hold printed parts together. A seal that needs no clamping force (a radial O-ring) lets a lid latch on.

## Numbers that worked

- **Strain** ≤ 1.2% (15% of PLA's ~8% elongation at break). Flex only in the part's print plane. Cantilever strain = 1.5·w·δ/L².
- **Thickness and clearance:** ≥ 2 mm for anything that flexes or carries load. 0.4 mm general clearance.
- **Locating fit:** 0.1 mm per flat. This is the "1-dot" tab, which printed best on a Bambu P1S in PLA.
- **Locating pins:** at least half tight, two of them on opposite corners. The rest are 0.15 mm looser. Never make every pin tight.
- **Snap teeth:** preload ≥ 0.2 mm on a 45° face, so print error can't leave a joint loose.
- **Launch:** FS 2 on stresses, with PLA across layers assumed to be 15 MPa. Snap retention ≥ 1.5 × the launch load, confirmed by a pull test (a luggage scale on a coupon).
- **Batteries:** hold every cell size the listing allows, and check the smallest as well as the largest (a holder sized for the largest lets a smaller cell lift out). Never clamp a pouch cell: any push on it stays light and comes from something that gives, such as a sprung hook. The default cell is the MakerFocus 3.7 V 3000 mAh flat LiPo, 67 × 36 × 12 mm ±2, JST PH 2.0.
- **Outdoor:**
  - **Seal:** a radial O-ring in a groove round a skirt under the lid, squeezed 18–30% against the box wall, groove ≤ 85% full.
  - **Ports:** glands face down, antennas point up, and a breathable vent stops condensation.
  - **Tripod:** a 1/4"-20 UNC socket in a sealed boss, so water can't get in through it. A printed thread is fine if tested with a coupon.

## Look (all versions)

Clean and engineered, like a modern launch vehicle, without copying anyone's names or logos: white structure and black decks, crisp mitred chamfers on outer edges, triangular isogrid pockets (2.4 mm ribs at 60°) instead of free-form holes, flush faces with nothing showing, and the CubeSat Classroom logo on every part (coupons too), inlaid flush 0.6 mm in the other colour as a second volume, 1 mm clear of every edge, hole and slit, strokes at least 0.45 mm. The look never costs strength, fit or printability.

## Printability

- No supports. Overhangs ≥ 45°, or bridges ≤ 10 mm between ribs.
- One big flat face per part, so Bambu Studio's auto-orient picks it.
- No sealed air voids inside a part.

## Workflow

1. Restate which of the three rules the change touches.
2. Design with the joints above. Make every dimension one named number.
3. Make a few-gram **test coupon** of every new joint, made by the same code (see the printed-fit-coupons skill).
4. Write every rule as a check, and prove each check can fail with a deliberately broken design (see the rust-cad-with-checks skill).
5. Package prints with names, plates and settings (see the bambu-studio-projects skill).
6. **Label every result**: design check, desk simulation or hardware bench. Never present a check as a measurement, and say in the README what has been printed and what hasn't.
7. Run the checklist: no loose pieces or screws, one way round, no supports, snap rules, locating, launch, material, coupon, checks, look, labels.

When an existing design doesn't meet the guide, say so in its README rather than hiding it.
