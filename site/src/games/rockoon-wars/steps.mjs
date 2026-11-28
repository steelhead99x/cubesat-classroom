// How to play Rockoon Battle, one action per step, like a building-instructions booklet. `pic` names an icon in
// ../icons.mjs.
// The page lists every step with its own link (/games/rockoon-wars/#step-aim); the game's
// first card and its "what to do now" line point at the same steps.
export const STEPS = [
  { id: 'players', pic: 'crew', title: 'Pick your crews', text: 'Press “Play on this computer” and choose 2, 3 or 4 players. To play with friends on other screens, press “Play online” and share the 5-letter room code.', keys: ['Play on this computer', '2', '3', '4'] },
  { id: 'shop', pic: 'crate', title: 'Choose your rockets', text: 'Each crew can spend credits on rockets that scan wider around the ring. The Scout is free and leaves a small dent. Tracker digs deeper, while Mapper clears a wider patch. Press Done when your crew is ready.', keys: ['Buy', 'Done'] },
  { id: 'wind', pic: 'wind', title: 'Check the wind and target', text: 'The arrow at the top shows the wind. It carries your balloon sideways as it climbs, then bends your rocket and drifts the target. The gold dashes show where the target will move during the first two seconds of rocket flight.', keys: ['Wind →'] },
  { id: 'aim', pic: 'aim', title: 'Aim', text: 'Drag from your balloon toward the gold ring, or press ◀ and ▶. The yellow dots show how your rocket will start.', keys: ['◀', '▶'] },
  { id: 'power', pic: 'power', title: 'Set the power', text: 'Drag farther for more power, or press − and +. More power flies higher and farther.', keys: ['−', '+'] },
  { id: 'launch', pic: 'rocket', title: 'Lift off, then release', text: 'Press Launch balloon. Wait for it to reach the release point. Adjust your aim if needed, then press Release rocket. If you wait too long, it releases by itself. A hit cuts away ground and can shift a nearby launch pad. Your next balloon takes off from the pad’s new spot.', keys: ['Launch balloon', 'Release rocket'] },
  { id: 'score', pic: 'ring', title: 'Score', text: 'From the outside in, the rings score 2, 4, 6 and 10. The 10-point centre earns a bullseye award. A near scan earns 1. Each crew gets three launches a round, and the most points after the last round wins.', keys: [] },
];

/** Keyboard players can do everything above with these keys. */
export const KEYS = 'Left and Right arrows aim. Up and Down set the power (Shift moves by 5). Q and E change the rocket. Space or Enter launches the balloon, then releases the rocket. P pauses. M turns sound on.';
