# CSC-R1 slide deck: source

This folder holds the source of the "CSC-R1 satellite receiver" deck. It has 15 slides covering why we're building our own receiver, the Rev A dev kit and its checks, the Rev B stompbox with HaLow, the metal-box RF review, risks, the bench-on-a-budget slide and next steps.

- **Live deck:** <https://claude.ai/artifact/2kUVSxTBFG2pXpMxAtsiCV>. It's private to its owner until shared. You can export it to PowerPoint or PDF from there.
- These files are a snapshot of the live deck at version `1795720958-8564` (26 Nov 2026). Edit the live deck, then copy the files back here in a pull request.

| File | What it is |
| --- | --- |
| `deck.json` | Slide order, sections, title and fonts |
| `deck.css` | Shared styles for every slide |
| `slides/*.html` | One slide each. Each slide's `<aside>` holds its speaker notes. |

Notes:

- Pictures live in the live deck's asset store, and slides point to them as `/_blob/<id>`. The deck's one picture, on the board slide (`/_blob/efb9cfa44a963c5dd0a8d219d1d68b5e`), is byte-for-byte the same file as [`../kicad/csc-r1-top.png`](../kicad/csc-r1-top.png) (SHA-256 `3b7e9c7f…d6ad24`), so the deck is complete in git.
- Everything in the deck is desk design: calculations, estimates and rule checks. Nothing has been built or measured yet.
