# CubeSat Classroom website

The public site connects K–12 classrooms and volunteer mentors to the ground station, balloon lesson, 3D-print files, and contribution quests. The Astro pages and Tailwind styles live in `src/`. The CubeSat mark lives in `public/mark.svg`. The banner uses the project's Rev B CAD preview.

## Preview locally

Use Node.js 24.

```sh
cd site
npm ci
npm run dev
```

## Publish

For GitHub Pages, run `npm run build:github` from `site/` and commit the generated files in `docs/` along with the source changes. GitHub Pages serves `docs/` on `main`, including the classroom installer at `start.sh`.

For Cloudflare, run `npm run build` from `site/`. Deploy `dist/` as [Worker Static Assets](https://developers.cloudflare.com/workers/static-assets/direct-upload/) on the `cubesat-classroom` Worker. Include `src/worker.mjs` as the Worker entry point and `src/hubs.mjs` as a module. Set the static assets binding to `ASSETS` and run the Worker first for `/`. Other paths serve the assets directly.

| Custom domain | Role |
| --- | --- |
| `cubesatclass.com` | Classroom home and quick start |
| `cubesat.app` | iOS and Android template |
| `cubesatclass.org` | Curriculum and teacher pilot |
| `cubesatclass.io` | Ground station and radio |
| `cubesatclass.dev` | Software development and MoQ.pro testing |
| `cubesat.club` | Volunteer contributions and fundraising links |

The [Worker Custom Domains](https://developers.cloudflare.com/workers/configuration/routing/custom-domains/) share one build. The site keeps the GitHub Pages installer URL for both hosts. The Rust station and its MoQ/QUIC relays run locally during the lesson.
