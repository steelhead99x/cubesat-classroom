# CubeSat Classroom mobile

This Expo and React Native app runs on Android and iOS from one codebase. It works offline, does not connect to a live station, and does not collect accounts or personal details.

| Tab | What it does |
| --- | --- |
| Mission | The research flight at a glance, level picker, next lesson and earned patches |
| Station | Practice ground station: ask for readings, take pictures, pause contact, read the mission log |
| Flight | The simulated research flight in five parts, with what the sensor kit should record at each level |
| Learn | The five browser lessons at three levels, with a patch for each one finished |
| More | Guides, extra quests, the fork guide and ways to help |

Level and patches last only while the app is open. The flight numbers are a desk simulation (the same model as the `flight-profile` command), not flight data.

Lesson text comes from the website's lessons. After editing `site/src/content/quests/`, run `npm run sync:quests` to refresh `src/data/quests.json`. A fork changes its name and links at the bottom of `src/theme.ts`.

See the [classroom receiver design](../docs/classroom-receiver.md) for the proposed live station connection over Media over QUIC.

The practice station does not subscribe yet. A future live view should follow [the client contract](../docs/client-contract.md): broadcast `cubesat/classroom/live`, track `telemetry` (JSON, one full snapshot per group), and optional `image`, `audio`, and `video` tracks that a phone may skip. There is no Media over QUIC client in this app today. A beta LoRa board with an OLED would hear the radio packet instead; that path is not this app.

Use Node.js 24, then run:

```sh
cd mobile
npm ci
npm start
```

Open it with a compatible Expo Go build on your device, or use `npm run android` or `npm run ios` with a configured emulator or simulator. Local iOS Simulator development requires macOS and Xcode. The app configuration generates native projects when needed. Run `npm run lint` and `npm run typecheck` before submitting changes.

The Expo starter code retains its [MIT notice](LICENSE). Original contributions use the repository’s MIT or Apache-2.0 terms.
