# Mission and communications concept

The minimum mission is periodic sensor telemetry with provenance. Video is a pass-time experiment. Keep an autonomous spacecraft bus and RF inhibit chain independent of the media computer. The bus owns safe mode, watchdog recovery, EPS limits and authorized command handling. The media computer receives telemetry and may shut down without affecting those functions. The classroom station sends only bounded sensor and video requests to a simulated payload over two localhost relays. The proposed flight bus maintains its own separate control interface.

## RF roles to review

| Link | Candidate role | First evidence | Flight question |
| --- | --- | --- | --- |
| SubGHz | Robust housekeeping and low-rate fallback. Reuse moq-subghz FSK for the controlled video lab demo. | Replay first, then conducted attenuation and Doppler sweep with measured packet delivery and age. | Which authorized service, waveform and antenna pattern close the safe-mode link across likely attitudes? |
| S-band | Primary experiment-data link and candidate TT&C on authorized hardware. | Conducted modem interoperability, actual goodput, spectrum and power. | Can the station acquire at low elevation and maintain enough link margin with the available power and pointing? |
| X-band | Burst video or stored media on selected passes. | Conducted test and station compatibility, then antenna pattern and pointing loss measured with the integrated bus. | What is the contact-limited data return after acquisition, thermal duty cycle, pointing loss and retries? |

Treat 1U as the mechanical demo size. Trade a 1U telemetry payload and a larger hosted payload or bus for S/X work against available faces, RF isolation, mass, EPS energy during eclipse, radiator area and pointing needs. Bring up one band at a time in the lab and prove intermodulation/desense before any simultaneous use. Start X-band with stored clips and measure whether the supported payload goodput sustains pass-time live video. [NASA Small Spacecraft Communications](https://www.nasa.gov/smallsat-institute/sst-soa/soa-communications/)

Run `python scripts/link_budget.py hardware/rf-example.json --altitude-km 550 --elevation-deg 15` for a transparent static downlink sensitivity exercise. It uses a spherical Earth slant range, free-space loss, TX power/gain/feed loss, receive G/T, an explicit aggregate loss, information bit rate and modem threshold on the same Eb/N0 basis. The Doppler magnitude is independently driven by the assumed radial speed. Each value in the example is illustrative. Serve as inputs only after the mission orbit, ground station, azimuth/elevation mask, measured antenna patterns and licensed channel plan are known. Add separate uplink, interference, antenna acquisition, actual pass profiles, duty cycle, power and useful delivery models before picking equipment.

The lab radio's 915 MHz default and encrypted video transport require an independent spectrum review for any proposed flight. In the US, consult a licensing specialist and the applicable administration on amateur-satellite versus an experimental or other service, including content and coding rules. Frequency coordination and an appropriate authorization are separate tasks. [IARU satellite coordination](https://www.iaru.org/reference/satellites/) · [US amateur radio rules, 47 CFR Part 97](https://www.govinfo.gov/content/pkg/CFR-2024-title47-vol5/xhtml/CFR-2024-title47-vol5-part97.html)

## Data and contact behavior

- Housekeeping `telemetry.v1` ships one small snapshot per group. Carry units, sequence, capture time and source. Add boot/session ID, clock uncertainty and calibration metadata before real sensors. The ground station rejects stale or future-dated snapshots.
- Keep media in separate groups with in-band H.264 parameters and a keyframe at the start. On a decode gap, resume at a surviving keyframe. Timestamp capture, receipt, decode and display independently. Only calculate end-to-end age after verifying the clock relationship.
- Outside contact, retain bounded housekeeping and optionally store selected clips with integrity checks. On acquisition, send a current snapshot before bulk traffic. A congested or rebooting payload must still leave bus safe mode available.
- Schedule and cap high-power radio and camera operation from the bus. Prototype the RF gateway with replay first, then measure each modem at representative attenuation, Doppler, pass duration and packet loss.

## Flight evidence gates

For the free-flying ISS deployment case, see the [ISS hardware compliance review](iss-deployment-review.md).

| Gate | Evidence needed to pass |
| --- | --- |
| Interfaces and configuration | Actual integrator ICD, frozen mass and center-of-mass budget, deployment and electrical inhibits, clearances, battery safety, materials and traceable assembly. |
| Survival | Flight-like test articles, mission-tailored vibration/shock, thermal vacuum and thermal cycling, EMI/EMC and materials/outgassing evidence. |
| Radiation and recovery | Orbit-specific dose and single-event analysis, qualified or justified parts, watchdog and reset testing, brownout recovery, persistent-storage integrity and safe-mode fault injection. |
| Operations | Current spectrum and other applicable approvals, end-to-end rehearsal with the intended ground station, acquisition and RF inhibit validation, and a plan for communications outages and end-of-life. |

Use the actual mission integrator's requirements to choose tests and acceptance levels. [NASA CubeSat 101](https://science.nasa.gov/wp-content/uploads/2023/06/nasa_csli_cubesat_101_508.pdf) · [Cal Poly CubeSat design and testing resources](https://www.cubesat.org/cubesatinfo)

## Relay and ground-station increments

The classroom topology is browser → ground-station process → ground MoQ relay ↔ spacecraft MoQ relay → payload process. Only the outgoing classroom command namespace and the opposite-direction payload namespace cross the relay bridge. Each request carries an ID and a short expiry. The spacecraft responds once per recent ID with a sensor snapshot or eight independently decodable test images. The station correlates the response, checks sensor age and marks partial video as incomplete. Pausing contact disconnects the bridge and stops the classroom payload from handling requests.

The native relay needs a bidirectional IP bearer. The existing MoQ-inspired SubGHz framing can remain useful on a separate low-rate or replay path with a ground republisher. First decide, band by band, whether the flight waveform will carry native IP/QUIC during a pass or deliver data objects to a ground gateway. The two local QUIC hops do not establish either option on RF.

| Ground increment | Acceptance evidence |
| --- | --- |
| Classroom console | A class exercises requests, sees a lost contact and exports a reviewed crew log. |
| Receive and replay desk | A radio mentor documents a recorder that ingests an approved ground modem or capture with unit, source and timing metadata. |
| Pass operations | Compare predicted station visibility with observed acquisition and loss, log elevation mask and clock uncertainty, and replay a missed contact. |
| Command-capable flight station | Independent review of operator authorization, scoped commands, audit events, session and clock handling, payload health, and radio inhibit interfaces. Verify using the actual host and licensed operating configuration. |

A single pair of local relays is the first topology. Multi-spacecraft routing, orbit or balloon position, rotator control, actual radio front ends, and deployment are future contributions.
