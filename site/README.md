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

For Cloudflare Workers Static Assets, build from `site/` and use the authenticated [Cloudflare CLI](https://developers.cloudflare.com/cf/projects/).

```sh
npm run build
cf deploy ./dist
```

Use the `cubesat-classroom` Worker and add `cubesatclassroom.dev` as a [Worker Custom Domain](https://developers.cloudflare.com/workers/configuration/routing/custom-domains/) after the domain is active in the same Cloudflare account. The page keeps the GitHub Pages installer URL so it works on both hosts.

The Rust station and its MoQ/QUIC relays run locally during the lesson. The published site serves the classroom resources.
