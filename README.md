# CubeSat Classroom

An open-source practice satellite lab for classrooms: a Rust ground station, browser lessons and printable hardware for learners 13 and up.

> **Status: work in progress.** This is an early, volunteer-run project. Everything here runs as a desk simulation or a bench test. Nothing has flown, no flight is booked, and no radio hardware in this repository has been reviewed for on-air use. Expect breaking changes.

Project overview: [hikd.ai/tis](https://hikd.ai/tis/)

## Quick start

Requires Rust (stable) on macOS, Linux or Windows (WSL).

```sh
git clone https://github.com/steelhead99x/cubesat-classroom.git
cd cubesat-classroom
bash docs/start.sh                   # guided setup
# or
cargo run --locked -- classroom      # two loopback MoQ relays + ground console on localhost
```

Open the loopback address it prints. No account or hardware is needed. The browser lessons are at [cubesatclass.org/quests](https://cubesatclass.org/quests/).

Run the checks (the same ones CI runs):

```sh
cargo fmt --check && cargo clippy --locked --all-targets -- -D warnings && cargo test --locked
(cd site && npm ci && npm run build && npm test)
(cd mobile && npm ci && npm run typecheck && npm run lint)
```

## What works today, and what is planned

| Works today (simulation or bench) | Planned, not built or not verified |
| --- | --- |
| Ground station on localhost: commands, simulated sensors, test video over Media over QUIC | Over-the-air operation with a licensed station |
| Pass prediction from TLEs, suborbital flight-profile desk simulation | Any balloon, suborbital or orbital flight |
| Sensor bridge for Arduino, XIAO and Raspberry Pi readings | Classroom fan-out and relay mesh between schools (not measured) |
| Five browser lessons, Expo mobile app (offline), printable P1S case files | Hosted AI quest guide, adult sign-in and paid plans (proposals only) |

## Repository layout

| Path | Contents |
| --- | --- |
| `src/` | Rust ground station (`cubesat-classroom` CLI) |
| `site/` | Astro website and Cloudflare Worker (`docs/` is its generated GitHub Pages copy) |
| `mobile/` | Expo / React Native app |
| `web/` | Ground console served by the station |
| `firmware/` | Arduino classroom sensor sketch |
| `hardware/` | CAD generators, STL and P1S print files, example RF link budget |
| `quest/` | Quest board: lessons, tests and review tasks |
| `docs/` | Design notes, guides and the generated site |
| `scripts/`, `tools/` | Installer, demo, PDF and budget helpers |

## Radio and safety

- This repository does not transmit on its own. The station and demos use loopback relays and simulated or recorded data.
- Transmitting on amateur bands requires a licensed control operator. Students take part under that operator.
- Amateur use stays open: telemetry must be identified with the station call sign, and messages must not be encrypted or obscured (in the US, 47 CFR 97.113(a)(4) and 97.119). The encrypted frame mode in the companion [moq-subghz](https://github.com/steelhead99x/moq-subghz) bench is for non-amateur bench work only and must be off on amateur frequencies.
- Any flight proposal needs an independent spectrum review and authorization. See [docs/mission.md](docs/mission.md).
- Never publish student names, schools, locations or contact details. See [CONTRIBUTING.md](CONTRIBUTING.md).

## Contributing and security

See [CONTRIBUTING.md](CONTRIBUTING.md). Report vulnerabilities privately as described in [SECURITY.md](SECURITY.md).

## License

Original work is dual-licensed under [MIT](LICENSE-MIT) or [Apache 2.0](LICENSE-APACHE), at your option. Third-party material keeps its own license (for example, `mobile/LICENSE` covers the Expo template the app started from).

CubeSat Classroom is an independent project inspired by Teachers in Space and AMSAT. It is not affiliated with or endorsed by NASA, AMSAT, Teachers in Space or any launch provider. Networking uses open-source Media over QUIC libraries from [moq.dev](https://moq.dev/).
