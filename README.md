# <img src="site/public/mark.svg" width="38" height="38" alt=""> CubeSat Classroom

**Build a small experiment. Bring back a measurement. Teach the next crew what you learned.**

CubeSat Classroom is an independent volunteer project for K–12 learners, educators and amateur-radio mentors, inspired by Teachers in Space and AMSAT.

[Project page](https://steelhead99x.github.io/cubesat-classroom/) · [Teachers in Space fundraiser](https://givebutter.com/fund-stem-training-for-100-educators-s14sb0)

## Help build it

This repository is a starting framework for a community project. We’re looking for teachers to try and improve lessons, radio volunteers to review station procedures, students to publish reproducible observations with an adult reviewer, and builders to contribute code, CAD and bench results. Start with the [quick-start guide](docs/quick-start.md) or [print the one-page handout](output/pdf/cubesat-classroom-quick-start.pdf), then pick [an open quest](/quest/README.md).

AI tools are welcome for drafting, coding and analyzing data. A contributor owns the result, an independent person reviews it, and measured claims need reproducible evidence. The classroom station runs without an AI service. Use our [contribution and review standards](CONTRIBUTING.md).

## Run the ground station

Paste this into a terminal on macOS, Linux, or WSL, follow the prompts, then open the local address the station prints.

```sh
curl -fsSL https://steelhead99x.github.io/cubesat-classroom/start.sh | bash
```

With an existing checkout, run `bash docs/start.sh`. The guide explains each step and asks before downloading files, installing Rust, or launching the station.

The commands and responses cross two pinned MoQ-over-QUIC relays on localhost. That is the first relay design exercise. The default readings and camera are simulated. The console can also read configured Linux sensor files. A separate [SubGHz H.264 rehearsal](docs/demo.md) replays the existing public [moq-subghz](https://github.com/steelhead99x/moq-subghz) format.

## Pick a project

- **Students and teachers:** [Run the balloon ground lab](docs/balloon-lab.md). Compare a measured pressure and temperature log with archived flight data, then teach another group how to repeat the experiment.
- **Builders:** [Print the 1U rail-fit set](hardware/README.md). The mechanical kit targets the Bambu P1S. Start with a PLA rail coupon, then repeat the fit check in ASA.
- **Radio volunteers:** [Review the relay and RF design](docs/mission.md). The SubGHz, S-band and X-band roles are proposed experiments.
- **Flight reviewers:** [Review the hosted experiment concept](docs/hosted-experiment-review.md).
- **Contributors:** [Choose a quest](quest/README.md) and follow the [volunteer contribution guide](CONTRIBUTING.md).

The hosted research-flight candidate centers on a self-contained sensor logger and post-flight data access. A balloon team can rehearse that workflow on the ground. An orbital relay and licensed on-air experiments are separate work that require additional integration and authorization.

## Work on the apps

See the [website](site/README.md), the [iOS and Android template](mobile/README.md), and the optional [MoQ.pro development guide](docs/moq-pro-development.md).

## License

Original material in this repository is available under either the [MIT License](LICENSE-MIT) or [Apache License 2.0](LICENSE-APACHE), at your option. Third-party material retains its own license.
