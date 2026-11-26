# Solar cost plan

**Goal:** bring the cost of space-grade solar power down for classrooms. Students learn one solar face, the [1U solar face](../hardware/solar-face/README.md), and only the cells change as a CubeSat moves from the bench toward orbit.

## Three steps

| Step | Cells | Cost per 1U face (cells) | Status |
| --- | --- | --- | --- |
| 1. Now, on Earth | 24 silicon SolarBITs (ANYSOLAR KXOB25-05X3F) on the `si` face | about $61–75 | Board designed and rule-checked; land pattern still to verify |
| 2. Ready for flight | 2 triple-junction space cells (AZUR SPACE 3G30A 8 × 4 cm) on the `tj` face, bonded by the class | about $600–1,100 | Board designed and rule-checked; land pattern still to verify |
| 3. A rideshare to orbit | The `tj` face on a CubeSat that passes the launch provider's tests | the launch slot dominates | Planning |

For comparison, a finished 1U side panel bought from a CubeSat supplier costs about $1,700 (Aphelion Orbitals' 2021 list price), and body-mounted panels for a 3U CubeSat can cost tens of thousands of dollars (Sorensen and Halliwell, SmallSat 2024).

**Why the cost falls:** space cells cost $250–450 per watt, and silicon costs cents per watt (University of Surrey review, 2025). Most of a bought panel's price is the cells and the work of fitting them, so a class that fits its own cells pays mostly for the cells. Students practise on silicon until their process is good enough for space cells.

**Ground stations** don't use space cells at all. An ordinary 10–20 W panel with a charge controller does the job.

## Step 3: getting to orbit

- **NASA CubeSat Launch Initiative (CSLI):** free launches for US schools that are selected.
- **Commercial rideshare** through an integrator, who fits the CubeSat into a deployer on a rideshare flight, for example a SpaceX Transporter mission. SpaceX's own rideshare price is per kilogram with a minimum, which is why small satellites go through integrators. The integrator's price covers the deployer, integration and testing.
- Either way, the CubeSat has to pass the provider's vibration, thermal and safety tests. The launch slot, not the solar panel, is the long-lead item. Book or apply early.

## Honest labels

Every figure on this page is a desk figure from the makers' data or a published price. Nothing has been built, measured or flown. The quest [Measure a 1U solar face](../quest/m1/solar-face-bench.md) produces the first hardware bench numbers.
