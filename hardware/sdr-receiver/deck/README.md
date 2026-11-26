# CSC-R1 slide deck: source

This folder holds the source of the "CSC-R1 satellite receiver" deck. It has 15 slides covering why we're building our own receiver, the Rev A dev kit and its checks, the Rev B stompbox with HaLow, the metal-box RF review, risks, the bench-on-a-budget slide and next steps.

- **Live deck:** <https://claude.ai/artifact/2kUVSxTBFG2pXpMxAtsiCV>. It's private to its owner until shared. You can export it to PowerPoint or PDF from there.
- These files are a snapshot of the live deck at version `1795655458-7f3a` (26 Nov 2026). Edit the live deck, then copy the files back here in a pull request.

| File | What it is |
| --- | --- |
| `deck.json` | Slide order, sections, title and fonts |
| `deck.css` | Shared styles for every slide |
| `slides/*.html` | One slide each. Each slide's `<aside>` holds its speaker notes. |

Notes:

- The board slide's image (`/_blob/…`) is stored in the live deck. It's the same render as [`../kicad/csc-r1-top.png`](../kicad/csc-r1-top.png).
- Everything in the deck is desk design: calculations, estimates and rule checks. Nothing has been built or measured yet.
