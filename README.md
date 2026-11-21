# CubeSat Classroom

CubeSat Classroom is an open-source K–12 satellite lab. Students request sensor data, receive test video, and see what happens when the link drops.

## Demo

Start with [Make first contact](https://cubesatclass.org/quests/first-contact/). The browser exercise uses simulated data and needs no account or hardware.

To run the Rust ground station on macOS, Linux, or Windows with WSL:

```sh
curl -fsSL https://cubesat.club/start.sh | bash
```

The starter walks you through setup. Open the loopback address it prints. The station uses two local Media over QUIC relays to send commands and return readings and video.

[Five-minute demo run sheet](docs/demo-run-sheet.md) · [Printable quick start](output/pdf/cubesat-classroom-quick-start.pdf)

## Labs and quests

| Start with | What to do |
| --- | --- |
| [Browser lessons](https://cubesatclass.org/quests/) | Five lessons cover measurement, pictures, link loss, reading age, and light levels. |
| [Ground station build guide](docs/ground-station.md) | Connect classroom sensors, predict a pass, or plan a receive-only station. |
| [P1S case files](hardware/p1s/README.md) | Slice the PLA coupon first, then print and measure the case. Follow the [print and fit quests](quest/README.md#bambu-p1s-case-test-track). |
| [Balloon lab](docs/balloon-lab.md) | Compare classroom pressure and temperature with archived flight data. |

The [quest board](quest/README.md) includes demo rehearsals, sensor tests, and hardware reviews. Share settings, measurements, and what another crew should check next. [Contribution guide](CONTRIBUTING.md)

## Project links

[Classroom site](https://cubesatclass.org/) · [CubeSat Club](https://cubesat.club/) · [Mobile template](mobile/README.md) · [Mission and radio design](docs/mission.md)

## Credits

Networking uses open-source Media over QUIC libraries from [moq.dev](https://moq.dev/). Volunteer site work by [hikd.ai](https://hikd.ai/). CubeSat Classroom is an independent project inspired by Teachers in Space and AMSAT.

## License

Original work is available under [MIT](LICENSE-MIT) or [Apache 2.0](LICENSE-APACHE). Third-party material keeps its own license.
