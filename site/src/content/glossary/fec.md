---
term: "Forward error correction (FEC)"
slug: "fec"
summary: "Extra data sent with a message so the receiver can repair losses without asking again."
category: "Software and data"
aliases: ["FEC", "error correction", "erasure coding"]
related: ["telemetry", "moq", "rssi-snr"]
seeAlso: [{"label": "Lesson: Lose the signal on purpose", "href": "/quests/lose-the-signal/"}, {"label": "Simulator", "href": "/#demo"}]
sources: []
deck: ["live-demo"]
---

**Forward error correction (FEC)** adds carefully designed extra data to a transmission so the receiver can rebuild lost or damaged pieces on its own, without asking for them again. It matters for satellite and long-range radio links, where asking again is slow or impossible. In the deck's demo, `FEC k=8` means groups of 8 data pieces are protected together.
