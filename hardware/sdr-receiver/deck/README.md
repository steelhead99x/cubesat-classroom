# CSC-R1 slide deck: source

This folder holds the source of the "CSC-R1 satellite receiver" deck. It has 17 slides covering why we're building our own receiver, the Rev A dev kit and its two fix rounds, the CSC-R1P pedal and its routing status, the simulation in the station, website and app, the Rev B HaLow plan, the metal-box RF review, risks, the bench-on-a-budget slide and next steps.

- **Live deck:** <https://claude.ai/artifact/2kUVSxTBFG2pXpMxAtsiCV>. It's private to its owner until shared. You can export it to PowerPoint or PDF from there.
- These files are a snapshot of the live deck at version `1795808779-56eb` (27 Nov 2026). Edit the live deck, then copy the files back here in a pull request.

| File | What it is |
| --- | --- |
| `deck.json` | Slide order, sections, title and fonts |
| `deck.css` | Shared styles for every slide |
| `slides/*.html` | One slide each. Each slide's `<aside>` holds its speaker notes. |

Notes:

- Pictures live in the live deck's asset store, and slides point to them as `/_blob/<id>`. Each one is also in git, so the deck is complete here:
  - board slide, `/_blob/efb9cfa44a963c5dd0a8d219d1d68b5e`: the same file as [`../kicad/csc-r1-top.png`](../kicad/csc-r1-top.png) (SHA-256 `3b7e9c7f…d6ad24`);
  - pedal slide, `/_blob/81515ccbcf1401f9748acc5c5ae3d712`: [`images/csc-r1p-face.png`](images/csc-r1p-face.png), the face cut from [`../pedal/face-template.svg`](../pedal/face-template.svg) (SHA-256 `e9253d8e…`);
  - simulation slide, `/_blob/164467ddd16e2c0efc56d7f440c85e45`: [`images/pedal-trainer-pass.png`](images/pedal-trainer-pass.png), a screenshot of the website's pedal trainer in pass mode (SHA-256 `5a92f0dd…`).
- Everything in the deck is desk design or desk simulation: calculations, estimates, rule checks and simulated passes. Nothing has been built or measured yet.
