# CSC-R1 slide deck: source

This folder holds the source of the "CSC-R1 satellite receiver" deck. It has 17 slides covering why we're building our own receiver, the Rev A dev kit and its two fix rounds, the CSC-R1P pedal and its routing status, the simulation in the station, website and app, the Rev B HaLow plan, the metal-box RF review, risks, the bench-on-a-budget slide and next steps.

- **Live deck:** <https://claude.ai/artifact/2kUVSxTBFG2pXpMxAtsiCV>. It's private to its owner until shared. You can export it to PowerPoint or PDF from there.
- The deck source started from live version `1795811688-c309` (27 Nov 2026). The cover, pedal status, risks and next-steps slides include newer KiCad results from this repository.

| File | What it is |
| --- | --- |
| `deck.json` | Slide order, sections, title and fonts |
| `deck.css` | Shared styles for every slide |
| `slides/*.html` | One slide each. Each slide's `<aside>` holds its speaker notes. |

Notes:

- Pictures live in the live deck's asset store, and slides point to them as `/_blob/<id>`. The deck's one picture, on the board slide (`/_blob/efb9cfa44a963c5dd0a8d219d1d68b5e`), is byte-for-byte the same file as [`../kicad/csc-r1-top.png`](../kicad/csc-r1-top.png) (SHA-256 `3b7e9c7f…d6ad24`), so the deck is complete in git.
- The other graphics are drawn in the slide files: SVG diagrams with SMIL animation (cover orbit, signal chain, pedal stack, HaLow chart) and one live `<x-embed>` on the simulation slide, which runs the same pass model as `src/receiver.rs` and `site/src/lib/pass.mjs`. Viewers can't click inside slides, so these play on their own. PowerPoint and PDF exports show a still frame.
- Everything in the deck is desk design or desk simulation: calculations, estimates, rule checks and simulated passes. Nothing has been built or measured yet.
