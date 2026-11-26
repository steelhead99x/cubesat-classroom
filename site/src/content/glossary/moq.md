---
term: "Media over QUIC (MoQ)"
slug: "moq"
summary: "An IETF protocol for live media and data that fans out through relays."
category: "Software and data"
aliases: ["MoQ", "Media over QUIC", "MoQ relay", "track", "fan-out"]
related: ["quic", "telemetry", "ground-station", "fec"]
seeAlso: [{"label": "Develop with MoQ", "href": "/hub/develop/"}, {"label": "Ground station hub", "href": "/hub/station/"}, {"label": "Simulator", "href": "/#demo"}]
sources: [{"label": "IETF Media over QUIC (moq) working group", "url": "https://datatracker.ietf.org/wg/moq/about/"}, {"label": "CubeSat Classroom: Why MoQ", "url": "https://github.com/steelhead99x/cubesat-classroom/blob/main/docs/why-moq.md"}]
deck: ["build-step-by-step", "live-video-915", "live-demo", "roadmap", "use-cases", "915-to-students"]
---

**Media over QUIC (MoQ)** is a protocol being developed at the IETF for sending live media and data with low delay. A publisher sends named **tracks** (for example `telemetry.v1`) to a **relay**, and the relay fans each track out to every subscriber, so one stream in can serve many classrooms out. MoQ runs on [QUIC](../quic/).

CubeSat Classroom uses MoQ to share each satellite pass from a [ground station](../ground-station/) with every classroom.
