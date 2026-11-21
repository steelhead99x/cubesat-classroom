# <img src="site/public/mark.svg" width="38" height="38" alt=""> CubeSat Classroom

**A free space lab for K–12 classrooms.** Students send commands to a practice satellite, bring back a measurement, and teach the next class what they learned.

No radio license, hardware or account is needed to start.

## Start here

1. **Try it in your browser.** Open the [practice ground station and example quests](https://cubesatclass.org/quests/). Pick Beginner if you are new.
2. **Follow the quick start guide.** The [quick start guide](docs/quick-start.md) shows how to run the real station on a computer, step by step. You can also [print the one-page version](output/pdf/cubesat-classroom-quick-start.pdf).
3. **Come to a workshop.** Three in-person workshops a year bring teachers, students and radio mentors together with real parts.

Want to see the code run right away? On macOS, Linux or Windows (WSL), paste this into a terminal and follow the prompts:

```sh
curl -fsSL https://cubesat.club/start.sh | bash
```

## What's inside

- **Example quests:** five short lessons in the browser, each at three levels.
- **Ground station:** a practice station you run on your own computer.
- **Balloon lab:** compare your classroom's measurements with a real balloon flight. See the [balloon lab](docs/balloon-lab.md).
- **3D-printable CubeSat:** a 10 cm model to print and measure. See the [hardware guide](hardware/README.md).
- **Quest board:** more [quests](quest/README.md) for teachers, students, radio mentors and builders.

## Get involved

Everyone is welcome: teachers, students (with an adult), amateur radio operators and builders.

- [Pick a quest](quest/README.md)
- [Read how to contribute](CONTRIBUTING.md)
- [Give feedback](https://github.com/steelhead99x/cubesat-classroom/issues/new?template=feedback.yml)
- [Donate to Teachers in Space](https://givebutter.com/fund-stem-training-for-100-educators-s14sb0)

## For engineers and reviewers

- [Why Media over QUIC: evidence and roadmap](docs/why-moq.md)
- [Mission and radio design](docs/mission.md)
- [Free and paid plans, a draft for discussion](docs/assistant-plans.md)
- [Rules for AI assistants in this repository](AGENTS.md)
- [Website](site/README.md), [mobile app](mobile/README.md) and [SubGHz video rehearsal](docs/demo.md)

## Thanks

The station's networking runs on the open-source Media over QUIC libraries and IETF work from [moq.dev](https://moq.dev). Quests are organized with tools from [kixel.ai](https://kixel.ai). The project is an independent volunteer effort inspired by Teachers in Space and AMSAT.

## License

MIT or Apache 2.0, your choice. See [LICENSE-MIT](LICENSE-MIT) and [LICENSE-APACHE](LICENSE-APACHE). Third-party material keeps its own license.
