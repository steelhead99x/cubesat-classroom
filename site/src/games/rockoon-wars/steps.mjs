// How to play Rockoon Battle, one action per step, like a building-instructions booklet. `pic` names an icon in
// ../icons.mjs.
// The page lists every step with its own link (/games/rockoon-wars/#step-aim); the game's
// first card and its "what to do now" line point at the same steps.
export const STEPS = [
  { id: 'players', pic: 'crew', title: 'Pick your crews', text: 'Press “Play on this computer” and choose 2, 3 or 4 players. To play with friends on other screens, press “Play online” and share the 5-letter room code.', keys: ['Play on this computer', '2', '3', '4'] },
  { id: 'shop', pic: 'crate', title: 'Choose your rockets', text: 'Each crew can spend credits on rockets that scan wider around the ring. The Scout is free. Press Done when your crew is ready.', keys: ['Buy', 'Done'] },
  { id: 'wind', pic: 'wind', title: 'Check the wind', text: 'The arrow at the top shows the wind. It carries your balloon sideways as it climbs, then bends your rocket the same way.', keys: ['Wind →'] },
  { id: 'aim', pic: 'aim', title: 'Aim', text: 'Drag from your balloon toward the gold ring, or press ◀ and ▶. The yellow dots show how your rocket will start.', keys: ['◀', '▶'] },
  { id: 'power', pic: 'power', title: 'Set the power', text: 'Drag farther for more power, or press − and +. More power flies higher and farther.', keys: ['−', '+'] },
  { id: 'launch', pic: 'rocket', title: 'Launch', text: 'Press Launch, or the Space key. Your balloon rises, then lets the rocket go.', keys: ['Launch'] },
  { id: 'score', pic: 'ring', title: 'Score', text: 'Through the ring scores 3 points. Close to its edge scores 1. Each crew gets three launches a round, and the most points after the last round wins.', keys: [] },
];

/** Keyboard players can do everything above with these keys. */
export const KEYS = 'Left and Right arrows aim. Up and Down set the power (Shift moves by 5). Q and E change the rocket. Space or Enter launches. P pauses. M turns sound on.';
