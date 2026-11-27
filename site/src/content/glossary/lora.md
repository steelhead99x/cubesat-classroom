---
term: "LoRa"
slug: "lora"
summary: "A long-range, low-power radio modulation used by sensors and trackers."
category: "Radio"
aliases: ["LoRaWAN", "SX1276", "LoRa board"]
related: ["sub-ghz", "915-mhz", "ism-band", "part-15"]
seeAlso: [{"label": "Lesson: Catch the Pass", "href": "/quests/catch-the-pass/"}, {"label": "Game step: Pick a radio setting", "href": "/games/catch-the-pass/#step-radio"}, {"label": "Classroom receiver guide", "href": "https://github.com/steelhead99x/cubesat-classroom/blob/main/docs/classroom-receiver.md"}, {"label": "Develop with MoQ", "href": "/hub/develop/"}]
sources: [{"label": "Semtech: What is LoRa?", "url": "https://www.semtech.com/lora/what-is-lora"}, {"label": "LoRa Alliance: About LoRaWAN", "url": "https://lora-alliance.org/about-lorawan/"}]
deck: ["build-step-by-step", "fly-without-permit", "watch-it-live"]
---

**LoRa** (long range) is a radio modulation, developed by Semtech, that uses chirp spread spectrum to send small amounts of data over long distances on little power. It usually runs in [sub-GHz](../sub-ghz/) bands such as [915 MHz](../915-mhz/). **LoRaWAN** is a separate network protocol, maintained by the LoRa Alliance, that runs on top of LoRa. Semtech's SX1276 is a common LoRa chip.

### Spreading factor

A LoRa radio can trade speed for range with its **spreading factor** (SF), from SF7 to SF12. Each step up stretches every chirp to twice as long, so a packet takes about twice as long to send and uses about twice the energy. In return the receiver can decode a signal about 2.5 dB weaker. At SF7 it needs the signal no more than about 7.5 dB below the noise; at SF12, about 20 dB below. That is why a slow setting still works when a satellite is low and far away, and why it costs a battery more.

In [Catch the Pass](../../games/catch-the-pass/#step-radio), **Fast**, **Middle** and **Far** are SF7, SF9 and SF12, with those decoding limits. The game shortens the airtimes so a pass fits in a minute.
