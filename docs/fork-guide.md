# Fork guide: start your own club, class or mission site

CubeSat Classroom is a starting kit. It is not meant to compete with other space education projects. Copy it, rename it and build your own site, club or mission on top. Share improvements back if they would help others, but you don't have to.

Original work is under MIT or Apache 2.0, your choice. Third-party material keeps its own license.

## What you get

| Piece | Where | Reuse it for |
| --- | --- | --- |
| Website (Astro and Tailwind) | `site/` | Your own home page, lessons and mission page |
| Browser lessons at three levels | `site/src/content/quests/` | Any simulated mission |
| Practice ground station (Rust) | `src/` | Sensor readings, test video, link loss |
| Flight tools | `passes`, `flight-profile`, `sensor-bridge`, `balloon-log` commands | Satellites, suborbital flights, benches, balloons |
| Quest board | `quest/` | Step-by-step work with reviewable evidence |
| Print files | `hardware/` | A bench model to measure |
| Rules for AI helpers | `AGENTS.md` | Student safety, accuracy and scope |

## Make it yours in five steps

1. **Fork** the repository on GitHub and clone your copy.
2. **Change the settings** at the top of `site/src/hubs.mjs`: `repo` (your fork), `classroomHost` (your web address) and `fundraiser` (your own link, or remove it). Edit the `hubs` list for your domains, or cut it down to one.
3. **Rename.** Search for "CubeSat Classroom" in `site/src/` and `README.md`, and replace it with your name. Replace `site/public/mark.svg` with your own mark. Please don't present your fork as this project or as Teachers in Space, AMSAT or a flight provider.
4. **Add your mission.** Copy a quest in `site/src/content/quests/` for browser lessons, or one in `quest/m0/` for bench work. For a new vehicle, copy `docs/suborbital-flight-plan.md` and replace the facts with your provider's, with sources.
5. **Publish.** `cd site && npm install && npm run build` makes a plain static site in `site/dist/` that any web host can serve. See `site/README.md` for Cloudflare, GitHub Pages, the optional preview password and the optional quest guide.

Check your work: `cargo test` passes, `npm run build` finishes, and your home page shows your name and links.

## Keep the parts that protect students

Whatever you change, keep these from `AGENTS.md`: no personal details from students, an honest label on every result (desk simulation, hardware bench, received RF, and so on), and licensed operators for anything that transmits.

## Working with AI coding tools (and saving tokens)

AI assistants charge by how much they read and write. A fork starts with most of the work already done, and a few habits keep the rest cheap:

- **Point the assistant at the right file.** `AGENTS.md` lists the starting points. Ask for a change in a named file rather than "improve the site".
- **Reuse before you generate.** Copy an existing quest or page and edit the words. The quest format, three levels and page layout are already decided.
- **Keep facts in one place.** Site settings live in `site/src/hubs.mjs`; the flight chart reads `site/src/data/flight-profile.json`, which is made by `cargo run -- flight-profile --json --step-s 10`. Regenerate that file when the flight numbers change.
- **Let the checks do the reviewing.** `cargo test`, `cargo clippy --all-targets` and `npm run build` catch most mistakes without another AI pass.
- **The quest guide needs no AI by default.** The site ships with prewritten answers. A live model is optional and capped (see `site/README.md`).

## Map of the repository

```text
AGENTS.md            Rules and starting points for AI helpers
README.md            Short introduction and quick start
docs/                Guides: ground station, balloon lab, flight plan, reviews
quest/               Quest board (m0 classroom, m1 design, m2 verification)
site/                Website source; builds to site/dist and docs/ (GitHub Pages)
src/                 Rust ground station and tools
firmware/            Arduino sketch for classroom sensors
hardware/            CAD and print files
tools/               Small scripts, such as the energy budget
```

## Sharing back

Open an issue or pull request on the original repository if a fix or a quest would help other classrooms. See [CONTRIBUTING.md](../CONTRIBUTING.md). Tell us about your fork and we will gladly link to it.
