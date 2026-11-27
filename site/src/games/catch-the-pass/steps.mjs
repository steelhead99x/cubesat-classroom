// How to play Catch the Pass, one action per step, like a building-instructions booklet.
// The page lists every step with its own link (/games/catch-the-pass/#step-track); the game's
// first card and its "what to do now" line point at the same steps.
export const STEPS = [
  { id: 'level', pic: '⭐', title: 'Pick a level', text: 'Start with level 1. A star on a level unlocks the next one. Easy mode, on at first, makes passes slower and the antenna circle bigger.', keys: ['1 First light', 'Easy mode'] },
  { id: 'start', pic: '▶️', title: 'Press Start', text: 'Press Start, or the Enter key. After a short countdown the satellite rises on the left.', keys: ['Start'] },
  { id: 'track', pic: '📡', title: 'Keep the circle on the satellite', text: 'Press or hold ▲ ◀ ▶ ▼, drag on the sky, or use the arrow keys. When the signal bar turns green, you are catching packets.', keys: ['▲', '◀', '▶', '▼'] },
  { id: 'radio', pic: '📻', title: 'Pick a radio setting', text: 'Fast catches packets quickly when the satellite is high and close. Far still works when it is low and far away. Keys 1, 2 and 3 work too.', keys: ['Fast', 'Middle', 'Far'] },
  { id: 'battery', pic: '🔋', title: 'Look after the battery', text: 'In Earth’s shadow every packet uses stored energy. If the battery runs low, press Sending (or Space) to let the satellite rest.', keys: ['Sending'] },
  { id: 'receiver', pic: '🎛️', title: 'Tune the receiver', text: 'Keep the receiver pedal engaged and tuned to 915 MHz. Lower the gain if the input meter turns red.', keys: ['Engaged', 'Tune', 'Gain'] },
  { id: 'stars', pic: '🏆', title: 'Earn stars', text: 'Catch enough packets before the satellite sets to earn 1, 2 or 3 stars.', keys: [] },
];

/** Keyboard players can do everything above with these keys. */
export const KEYS = 'Arrow keys or W A S D move the antenna. 1, 2 and 3 pick the radio setting. Space switches sending. Enter starts. P or Esc pauses.';
