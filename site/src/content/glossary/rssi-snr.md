---
term: "RSSI and SNR"
slug: "rssi-snr"
summary: "Two measures of signal quality: received strength and signal-to-noise ratio."
category: "Radio"
aliases: ["RSSI", "SNR", "signal strength", "dBm"]
related: ["telemetry", "fec", "lora"]
seeAlso: [{"label": "Lesson: Lose the signal on purpose", "href": "/quests/lose-the-signal/"}, {"label": "Lesson: Catch the Pass", "href": "/quests/catch-the-pass/"}, {"label": "Classroom receiver guide", "href": "https://github.com/steelhead99x/cubesat-classroom/blob/main/docs/classroom-receiver.md"}]
sources: []
deck: ["live-demo"]
---

**RSSI** (received signal strength indicator) is how strong a received signal is, usually in dBm; numbers closer to zero are stronger. **SNR** (signal-to-noise ratio) compares the signal with the background noise, in dB; higher is cleaner. Together they show how close a radio link is to dropping out.

In [Catch the Pass](../../games/catch-the-pass/), the signal bar shows SNR. The white mark on it is the SNR the chosen radio setting needs, and the chart after each pass compares the two over the whole pass.
