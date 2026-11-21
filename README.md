# CubeSat Classroom

CubeSat Classroom is an open-source K–12 satellite lab. Students request sensor data, receive test video, and see what happens when the link drops.

It is a starting kit, not a competitor to other space education projects: fork it and build your own class, club or mission site. See the [fork guide](docs/fork-guide.md).

## Demo

Start with [Make first contact](https://cubesatclass.org/quests/first-contact/). The browser exercise uses simulated data and needs no account or hardware.

To run the Rust ground station on macOS, Linux, or Windows (using WSL, the Windows Subsystem for Linux):

```sh
curl -fsSL https://cubesat.club/start.sh | bash
```

If that address asks for a password, the site is in private preview: clone this repository and run `bash docs/start.sh` instead.

The starter walks you through setup. Open the loopback address it prints. The station uses two local Media over QUIC relays to send commands and return readings and video.

[Five-minute demo run sheet](docs/demo-run-sheet.md) · [Printable quick start](output/pdf/cubesat-classroom-quick-start.pdf)

## Labs and quests

| Start with | What to do |
| --- | --- |
| [Browser lessons](https://cubesatclass.org/quests/) | Five lessons cover measurement, pictures, link loss, reading age, and light levels. |
| [Ground station build guide](docs/ground-station.md) | Connect classroom sensors, predict a pass, or plan a receive-only station. |
| [P1S case files](hardware/p1s/README.md) | Slice the PLA coupon first, then print and measure the case. Follow the [print and fit quests](quest/README.md#bambu-p1s-case-test-track). |
| [Suborbital flight plan](docs/suborbital-flight-plan.md) | Get a classroom sensor kit ready for a research spaceflight, using the public plans for the Teachers in Space flight on Virgin Galactic expected in 2027. Predict the flight with `flight-profile`. |
| [Flight heritage quests](quest/README.md#flight-heritage-and-the-suborbital-research-flight) | Learn from the Perlan 2 glider and the ISS, then compare all three rides. |
| [Balloon lab](docs/balloon-lab.md) | Compare classroom pressure and temperature with archived flight data. |

The [quest board](quest/README.md) includes demo rehearsals, sensor tests, and hardware reviews. Share settings, measurements, and what another crew should check next. [Contribution guide](CONTRIBUTING.md)

## Project links

[Classroom site](https://cubesatclass.org/) · [CubeSat Club](https://cubesat.club/) · [Mobile template](mobile/README.md) · [Mission and radio design](docs/mission.md)

## Credits

Networking uses open-source Media over QUIC libraries from [moq.dev](https://moq.dev/). Volunteer site work by [hikd.ai](https://hikd.ai/). CubeSat Classroom is an independent project inspired by Teachers in Space and AMSAT.

## License

Original work is available under [MIT](LICENSE-MIT) or [Apache 2.0](LICENSE-APACHE). Third-party material keeps its own license.
