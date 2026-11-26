---
term: "LoRa"
slug: "lora"
summary: "A long-range, low-power radio modulation used by sensors and trackers."
category: "Radio"
aliases: ["LoRaWAN", "SX1276", "LoRa board"]
related: ["sub-ghz", "915-mhz", "ism-band", "part-15"]
seeAlso: [{"label": "Classroom receiver guide", "href": "https://github.com/steelhead99x/cubesat-classroom/blob/main/docs/classroom-receiver.md"}, {"label": "Develop with MoQ", "href": "/hub/develop/"}]
sources: [{"label": "Semtech: What is LoRa?", "url": "https://www.semtech.com/lora/what-is-lora"}, {"label": "LoRa Alliance: About LoRaWAN", "url": "https://lora-alliance.org/about-lorawan/"}]
deck: ["build-step-by-step", "fly-without-permit", "watch-it-live"]
---

**LoRa** (long range) is a radio modulation, developed by Semtech, that uses chirp spread spectrum to send small amounts of data over long distances on little power. It usually runs in [sub-GHz](../sub-ghz/) bands such as [915 MHz](../915-mhz/). **LoRaWAN** is a separate network protocol, maintained by the LoRa Alliance, that runs on top of LoRa. Semtech's SX1276 is a common LoRa chip.
