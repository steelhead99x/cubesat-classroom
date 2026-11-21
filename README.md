# CubeSat Classroom

CubeSat Classroom is an open-source satellite lab for learners 13 and up. Students request sensor data, receive test video, and see what happens when the link drops.

It is a starting kit, not a competitor to other space education projects: fork it and build your own class, club or mission site. See the [fork guide](docs/fork-guide.md).

## Who it's for

CubeSat Classroom is written for learners 13 and up, and for the teachers, guardians and mentors who guide them.

- **No account is needed** for the browser lessons, the mobile app or the local ground station.
- **To join the project** (comment, share a crew log, propose a change), use an account on a hosted Git service: GitHub, GitLab.com or Codeberg. Self-hosted Git servers are not supported. The main repository is on GitHub today, but you are not locked in to it. Work from a fork or mirror on the host you prefer and send a link to your branch, or a patch.
- **Students** may use their own account if they are old enough under that host's terms. GitHub, for example, requires users to be at least 13.
- **Teachers, guardians and mentors** can use their own accounts to guide the lessons and the project: they open issues, review changes and share results for their crew. Younger learners take part only alongside one of these adults, through the adult's account, and never need an account of their own.
- **No Git account?** The planned sign-in for the hosted app will also let teachers, guardians and mentors sign in with a phone number. Students sign in with a Git account or join through an adult; the project does not collect students' phone numbers.

## Demo

Start with [Make first contact](https://cubesatclass.org/quests/first-contact/). The browser exercise uses simulated data and needs no account or hardware.

To run the Rust ground station on macOS, Linux, or Windows (using WSL, the Windows Subsystem for Linux):

```sh
curl -fsSL https://cubesatclassroom.org/start.sh | bash
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
| [Suborbital flight plan](docs/suborbital-flight-plan.md) | Get a classroom sensor kit ready for a suborbital research flight, using public information. Predict the flight with `flight-profile`. |
| [Flight heritage quests](quest/README.md#flight-heritage-and-the-suborbital-research-flight) | Learn from the Perlan 2 glider and the ISS, then compare all three rides. |
| [Balloon lab](docs/balloon-lab.md) | Compare classroom pressure and temperature with archived flight data. |

The [quest board](quest/README.md) includes demo rehearsals, sensor tests, and hardware reviews. Share settings, measurements, and what another crew should check next. [Contribution guide](CONTRIBUTING.md)

## Project links

[Classroom site](https://cubesatclass.org/) · [CubeSat Club](https://cubesatclassroom.org/volunteer/) · [Mobile template](mobile/README.md) · [Mission and radio design](docs/mission.md)

## Credits

Networking uses open-source Media over QUIC libraries from [moq.dev](https://moq.dev/). Volunteer site work by [hikd.ai](https://hikd.ai/). CubeSat Classroom is an independent project inspired by Teachers in Space and AMSAT.

## License

Original work is available under [MIT](LICENSE-MIT) or [Apache 2.0](LICENSE-APACHE). Third-party material keeps its own license.
