# Proposed experiment: microscope collection plate

**Status: open proposal. Looking for reviewers and contributors.** Nothing here is a flight commitment. Every stage needs its own review, and orbital stages need provider acceptance and spectrum authorization.

## The idea

Fly a small exposed collection plate with a microscope camera pointed at it. Image the plate on a schedule and send each image down to the ground. Schools with receive-only ground stations listen for the images, compare them with the previous pass and log what changed.

The same loop works at every step of the project's ladder: a classroom bench, a balloon, a hosted flight, and later orbit. It also sets up a pattern for future lunar work, where a team watches a surface change from far away over a slow, intermittent link.

## What a class could actually see

Plan for two measurement questions. One is likely to show a result. The other might not.

1. **Surface change (expected result).** In low Earth orbit, atomic oxygen erodes polymers, and NASA has measured this for decades on the MISSE experiments outside the ISS. Mount small witness coupons next to the plate, including a printed ASA coupon cut from the project's own rail material. A microscope can track texture and color change over weeks. This also feeds the [printed structure qualification](iss-deployment-review.md#printed-structure-qualification) work. [MISSE polymer erosion results (NASA NTRS)](https://ntrs.nasa.gov/citations/20240009151)
2. **Particle capture (rare event).** Micrometeoroid and debris impacts big enough for a small optic to resolve are uncommon. Before choosing plate area and optics, estimate the expected count for the actual orbit and pointing with NASA's ORDEM and MEM models. Treat "no impacts seen" as a valid, publishable result. [NASA ORDEM 3.2](https://www.orbitaldebris.jsc.nasa.gov/modeling/ordem.html)

On a balloon, the plate can collect stratospheric aerosols and dust instead. On the bench, students can collect classroom dust, pollen or salt and practise the imaging and logging workflow.

## Streaming it back

"Live" means pass-time, not continuous video. Housekeeping-class links carry still images during a pass. Faster bands could carry more images per pass after they are proven (see [mission.md](mission.md)).

- **Images as MoQ objects.** Each capture becomes one object with a sequence number, capture time, exposure, focus position, plate ID and a checksum. This is the same provenance rule the station already uses for `telemetry.v1`. A ground relay can fan the received objects out to every connected classroom.
- **Store and forward.** The payload keeps a bounded queue of captures between passes. On acquisition it sends the newest thumbnail first, then fills in full-resolution tiles as the link allows. A missed pass just leaves a visible gap in the sequence.
- **Receive-only schools.** Listening to a satellite downlink is generally possible without an operator license, but check local rules. Transmitting or commanding needs a licensed operator and an authorized station. If the experiment uses amateur frequencies, the image format must be published and unencrypted. [US amateur radio rules, 47 CFR Part 97](https://www.govinfo.gov/content/pkg/CFR-2024-title47-vol5/xhtml/CFR-2024-title47-vol5-part97.html)

## Path to lunar projects

Long one-way delay, short contact windows and lossy links are normal on lunar missions. Picking which object to send first under those limits is the same skill a lunar team needs. Lunar dust is also a known problem for surfaces and mechanisms, and watching it build up on a plate is a natural follow-on experiment. This proposal does not claim a lunar mission. It names the skills and data formats that would carry over.

## Staged plan

| Stage | What to build | Evidence that it worked |
| --- | --- | --- |
| Desk | Add a simulated microscope image request to the classroom station, using a stored image sequence. | A class requests an image, sees a gap after paused contact, and exports a crew log that cites image sequence numbers. |
| Bench | Swap in a USB microscope over a printed slide holder. Publish captures with focus and exposure metadata. | A second group reproduces the same image of the same slide and records the settings. |
| Balloon | Fly a sealed-until-altitude plate with a partner team. Store images on board and recover them. | A dated before-and-after image pair with a control plate kept on the ground. |
| Hosted or orbital | Only after host review: exposed plate, witness coupons, imaging schedule and store-and-forward downlink. | Provider acceptance, materials and outgassing review, spectrum authorization and an end-to-end ground rehearsal. |

## Open questions for reviewers

1. Which optic, illumination and focus approach survives vibration and thermal cycling at a classroom budget?
2. How do we keep contamination off the plate before exposure, and how do we prove the control plate saw the same handling?
3. What image size and compression fit one pass at the housekeeping data rate? Should the first downlink be a thumbnail?
4. Which ORDEM and MEM inputs should a student use to predict the impact count, and how do we explain the uncertainty?

Take the [collection plate quest](/quest/m1/micro-collection-plate.md) or follow the [contribution guide](../CONTRIBUTING.md).
