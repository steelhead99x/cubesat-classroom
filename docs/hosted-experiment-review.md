# Open peer review for the hosted experiment

CubeSat Classroom welcomes feedback from educators, amateur-radio operators, balloon teams and hosted-payload reviewers. We’re developing a small pressure and temperature experiment that a classroom can build, rehearse on the ground and propose for a future hosted flight.

## Flight opportunity used as a design reference

On November 18, 2026, Teachers in Space announced plans for two educators to conduct research aboard a Virgin Galactic flight expected in 2027. The published concept describes a compact printed sensor kit and returning flight data to K–12 classrooms. Its publicly described dimensions are 3.5 × 3.5 inches in two axes. Request the actual outline, all three dimensions, allowable materials, retained mounting design, battery conditions and host review process before choosing an enclosure. The project's 100 mm bench enclosure is larger than those published dimensions. [Virgin Galactic mission announcement](https://www.virgingalactic.com/news/teachers-in-space-plans-to-send-two-educators-on-2027-research-flight-with-virgin-galactic)

The [suborbital flight plan](suborbital-flight-plan.md) collects the public flight profile, the provider's published payload rules, readiness gates and open questions.

**Candidate experiment.** Record temperature and pressure during a bench control run and during each host-approved phase of flight. Time-stamp each sample, record gaps explicitly, and retrieve the log after landing. Let students compare the traces with their predictions and identify which measurements came from the payload environment. Request reference flight-event timestamps from the host. A cabin pressure sensor measures its local cabin environment, so students should use that context when comparing it to an external balloon trace.

**Minimum payload.** A host-approved sensor kit, two measured channels, a time source, bounded local storage, a documented start/stop procedure and a reference run. Use the host's supplied or accepted sensor logger if available and adapt the classroom data tools to its export. Keep network transmission, a camera and a relay outside the initial flight experiment proposal. The Rust station and paired relays are the classroom exercise and the beginning of a separate orbital communications design.

**Review evidence.** Bring a worksheet, ground CSV with a deliberate data gap, sensor calibration notes, a reviewed mounting and power drawing, and a plan for returning a reproducible, de-identified classroom dataset. Meeting a published description does not establish payload acceptance or a flight slot.

## Questions for volunteer reviewers

1. Does this experiment fit the learning goals and operator time available? Would light or an accelerometer answer the question better?
2. Which host interface should a classroom team design against? Which mounting, power, logging and materials checks matter first?
3. Can an existing weather-balloon partner help the class test the data and recovery workflow on the ground, then decide whether a supervised flight adds useful evidence?
4. Which amateur satellite mentor should review the longer-term SubGHz, S-band and X-band work and teach the next crew how to write a repeatable test?

## How to contribute feedback

Pick a question above and explain what you would change, why it matters, and how a classroom could test it. Add a reference, lesson, or example from prior work if you can share it. See the [hosted-experiment quest](/quest/m1/hosted-review.md) and the [contribution guide](../CONTRIBUTING.md).
