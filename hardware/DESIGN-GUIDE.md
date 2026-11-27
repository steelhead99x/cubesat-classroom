# CubeSat Classroom hardware design guide

How we design printed parts. **This guide applies to every CubeSat Classroom 3D design:** frames, plates, board and battery mounts, enclosures, ground-station and outdoor cases, fit tools and test coupons, and balloon, glider and rocket hardware. It applies in this repository, in every other CubeSat Classroom repository, and in forks, whoever designs the part, by hand or with an AI assistant. The rules come from what printed well (and badly) in class.

Every rule that can be checked is a check in [crates/amsat-kit](../crates/amsat-kit), and every check is proven able to fail by a broken design in the self-test. Other designs should do the same in their own CAD code. The same rules are packaged as an AI skill, [cubesat-printed-design](../.claude/skills/cubesat-printed-design/SKILL.md), with the other [public skills](../docs/skills.md).

## The three rules

1. **No loose pieces wherever possible, and no screws.** Every snap, hook, lip, pin and latch is part of a plate, the frame or the case, and every joint snaps apart and back together again. Nothing small gets dropped, lost or thrown away by mistake. When a part has to be separate (the outdoor lid's O-ring), it sits captive in its groove once it's built, and the README says why it can't be avoided.
2. **Rocket-launch ready and durable.** Every joint holds at the 20 g design load with margin. Nothing that flexes or carries load is thinner than 2 mm. Every snap holds its part tight, so nothing rattles, wears or works loose. Parts that go outside are printed in a material that survives sun and weather.
3. **Easy enough for kids.** A 13-year-old can build it on a budget printer, without tools, with a click to say each step is done. Parts go together one way round only. Steps that need an adult (soldering, wiring a battery) are marked as mentor steps.

When two rules pull against each other, the order above decides, except for safety, which always comes first.

One more thing applies to every part: **the look** (see [Design language](#design-language)).

## Joints

Use these joints, in this order of preference. Each is in [joints.rs](../crates/amsat-kit/src/joints.rs) and is checked the same way wherever it's used.

| Joint | Where | How it goes together |
| --- | --- | --- |
| **Keyed snap pin** | Plates onto columns | A pin with two flats and a 45° head. The plate's keyed slot drops on one way only, centred, and two flex fingers click in under the head. |
| **Lips and snap hooks** | Circuit boards | Tuck one edge under fixed lips, swing the board down, and snap hooks click over the other edge. |
| **Push-down end hooks** | Batteries and other thick parts | Press it straight down between side guides until a hook clicks over each end. Tuck-and-swing doesn't work for thick parts: their bottom edge swings into the lips. |
| **Lid latches** | Lids, including sealed outdoor lids | Press the lid straight down: a finger cut into each edge of the lid bends out over a lug on the rim and clicks under it. Press each finger outward with a thumb to lift the lid off again. |

No screws, inserts or nuts hold printed parts together, anywhere. (A bought cable gland or antenna bulkhead keeps its own nut.) A seal doesn't need clamping force if it is a radial seal (see [Outdoor cases](#outdoor-cases)).

## Design language

Every CubeSat Classroom part shares one look, borrowed from modern launch vehicles and spacecraft: clean, functional and engineered. It must never copy anyone's names, logos or trade dress.

- **Monochrome, two filaments.** White structure and coupons, black decks and plates inside. Print with white in slot 1 and black in slot 2; any colour still works.
- **Crisp edges.** Outer edges are chamfered with mitred corners, not rounded blobs. Inside corners that carry load keep a small fillet.
- **Isogrid.** Lighten a plate with triangular pockets between 2.4 mm ribs at 60°, never with free-form holes. Every plate uses the same pattern from `isogrid_pockets` in [parts.rs](../crates/amsat-kit/src/parts.rs).
- **Flush and hidden.** No visible fasteners (there are none). Latches and hooks sit inside the outline.
- **The CubeSat Classroom logo on every part,** test coupons included, inlaid flush 0.6 mm deep in the other colour (black on white parts, white on black). It's a second part (volume) of the same object in the Bambu project, so the shape doesn't change and a one-colour print still works. Big faces get the mark and the words; small parts get the mark alone. The logo stays 1 mm from every edge, hole and slit, so it's never on a flex finger, has plastic behind it, and has no stroke under 0.45 mm. [brand.rs](../crates/amsat-kit/src/brand.rs) places and checks it.
- **Function first.** The look never costs strength, fit or printability. A rule above always wins.

## Snap-fit rules

- **Flex only in the part's own print plane**, along its printed strands, never across its layers.
- **Strain at most 1.2%**: 15% of PLA's 8% elongation at break, so a snap can be used again and again without cracking or taking a set.
- **Nothing that flexes or carries load under 2 mm.** Clearances are 0.4 mm, except locating fits.
- **Close every snap on an angled face** that prints without supports. It pulls the joint tight whatever the print error. A hook whose underside is steeper than the friction angle can be pushed open by pulling the part, so board hooks use 10°, under PLA's 17°.
- **Give every flex finger room to bend**, and keep every other part off it. The checks enforce both.

## Locating: tight pins

- **At least half the locating pins are tight** (0.1 mm on each flat, the 1-dot fit that printed best).
- **Two tight pins sit on opposite corners.** Far apart, they set where the plate sits and stop it turning.
- **The rest are 0.15 mm looser**, so an error in hole spacing between two printed parts can't make the plate bind. Never make every pin tight.

## Launch loads and durability

- **Design load: 20 g quasi-static**, the same as the Rev C frame (classroom rockets reach about 10 g).
- **Stresses:** a factor of safety of 2. PLA's strength across layers is assumed to be 15 MPa (the Rev C value) until we measure it.
- **Snap retention: hand calculations, at least 1.5 times the launch load, confirmed by a pull test.** A plate on three snap pins must hold 1.5 × its launch load, about 24 N for the payload plate, so **one pin coupon should take at least 8 N to pull off.** Check that with a luggage scale and record it.
- **Batteries are held at both ends and both sides, for every cell size the listing allows.** Check the smallest and the largest: a holder sized only for the largest cell lets a smaller one lift out. Sprung hooks or end stops can take up the length range. A battery is never clamped tight: pouch cells swell, so any push on it stays light (a few newtons) and comes from something that gives.
- **Materials:** PLA indoors. ASA or PETG outdoors, because PLA softens in summer sun. A fit tuned in one material is rechecked in another: print the fit ladder again.

## Easy for kids

- **One way round:** keyed slots and asymmetric mounts, so a part can't go in wrong.
- **A click says it's done.** Every snap gives a click.
- **Marks instead of instructions:** dots for fits, an arrow for which way the Nano's USB goes.
- **Few steps, no tools** for the classroom kit. Mentor steps are labelled in the README.
- **Prints on any printer:** no supports, one big flat face per part for Bambu Studio's auto-orient, and a test coupon for every new joint.
- **Little plastic in test prints:** coupons of a few grams before any full plate, and an open-truss fit plate.

## Test coupons first

Every new joint or board mount gets a coupon: the same geometry, made by the same code, on a few grams of plastic.

- **Pin fit ladder:** three tabs (1, 2 and 3 dots, 0.1 to 0.3 mm). The best fit becomes the kit's fit.
- **Nano coupon** and **XIAO coupon:** the plate's mounts on their own, for the real boards.
- **Tripod-thread coupon:** the box's thread and one 0.15 mm looser.
- **Latch coupon:** a piece of the outdoor box's rim with one lug, and a piece of lid with one latch finger.
- **Camera coupon:** the camera side's cradle, guides and hooks on their own, for the real XIAO ESP32-S3 Sense.

Print each coupon in the material of the real part. Record what you find, and change the one number that controls it.

## Outdoor cases

- **Seal:** a radial O-ring in a groove round a skirt under the lid. The skirt drops into the box's opening and squeezes the O-ring sideways against the wall, 18–30%, with the groove at most 85% full. A radial seal needs no clamping force, so the lid latches on instead of screwing down. Use a nitrile O-ring of the cord size and inside diameter the build report gives.
- **Ports face down; antennas point up.** Use glands and bulkheads with their own O-rings, on flat parts of the wall.
- **A breathable vent**, so condensation and pressure changes can't pump water past the seal.
- **Tripod mount:** a 1/4"-20 UNC thread in a sealed boss in the middle of the back. Printed, no metal, and tested with the thread coupon.
- **Same plates inside:** an outdoor case uses the kit's columns and keyed pins, so the plates (and the coupons that proved them) carry over.

## Batteries

- **The battery:** a MakerFocus flat 3.7 V 3000 mAh LiPo with a JST PH 2.0 plug, listed at 67 × 36 × 12 mm, ±2 mm. Every case holds it.
- **Safety:** protected cells only. A mentor checks the cell and its polarity and does any wiring (see the [battery safety quest](../quest/m1/battery-safety.md)).
- **Power:** an Uno or Nano needs a 5 V boost board. A XIAO charges and runs from the cell directly. Wire one cell to one charger only.

## Checklist for any design

Before a new or changed part is printed beyond a coupon:

- [ ] **No loose pieces and no screws**, or each separate piece is justified and captive once built. Every joint comes apart and goes back together.
- [ ] **One way round:** it can't go together wrong, and a click says each step is done.
- [ ] **No supports**, and one big flat face to print on.
- [ ] **Snap rules:** flexing parts at most 1.2% strain, in their print plane, at least 2 mm thick, with room to bend.
- [ ] **Locating:** at least half the pins tight, two of them on opposite corners, and the rest looser.
- [ ] **Launch:** holds at 20 g with margin. Anything that flies is checked for its own loads too.
- [ ] **Material** right for where it lives: PLA indoors, ASA or PETG outdoors.
- [ ] **Coupon** printed for every new joint, in the real material, with the result recorded.
- [ ] **Checks:** every rule a check, every check shown able to fail, output reproducible.
- [ ] **Look:** crisp chamfered edges, isogrid pockets, flush faces, white and black, and the two-colour logo.
- [ ] **Labels:** the README says what's been printed and tested, and what's only been design-checked.

## Where existing designs stand

| Design | Meets the guide? |
| --- | --- |
| [Classroom kit](amsat-cubesatsim/README.md) (AMSAT frame + plates) | Yes, including the camera side. The 5 V boost board still needs a printed mount. The upstream AMSAT frame parts keep their own look. |
| [Outdoor ground station](outdoor-station/README.md) and [outdoor XIAO station](outdoor-xiao/README.md) | Yes. No screws: the lid latches on over a radial O-ring. The O-ring is the one separate piece. |
| Rev C frame (`crates/cubesat-cad`, PR #37) | Not yet: its board mounts are the rejected quarter-turn studs, lock keys and clamp. Port the keyed snap pin, lips and hooks, and battery hooks. |
| [Rev A bench box](p1s/README.md) | No: M3 screws. Archived as a historical test box. |
| [Rev B rail-fit set](ARCHIVE.md#rev-b-rail-fit-set) | No: needs supports, and has no board mounts. Archived. |
| [CSC-R1 receiver](sdr-receiver/README.md) | Its enclosure is a bought die-cast box. Any printed bracket, knob or faceplate follows the guide. |
| [Teachers in Space frame](tis/README.md) | Not ours, so it's linked, not changed. Mounts we print for it follow the guide. |
| Designs in other CubeSat Classroom repositories (balloon gondolas, rocket pods, solar faces) | Apply the guide the next time each is changed. |

When a design doesn't meet the guide yet, its README says so.

## Every result is labelled

Label every result as a design check, desk simulation or hardware bench test. Never present a design check as a measurement. Each README says what has and hasn't been printed.

## How the checks work

- **Every rule above that can be checked is a check.** `cargo run --release -p amsat-kit -- check` runs them all.
- **Every check is proven able to fail.** `-- self-test` builds deliberately broken designs, and each must be rejected.
- **The output is the same byte for byte on every run**, and CI fails if the committed files differ.
