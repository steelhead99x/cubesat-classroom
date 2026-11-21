// Quest guide questions and prewritten answers.
//
// Visitors pick from these questions; there is no free-text box. The demo shows
// the prewritten answers. When the live guide is switched on (see site/README.md),
// src/worker.mjs sends the same fixed question to the model and falls back to these
// answers if anything goes wrong. Keep answers short, on-mission and level-aware.

/** @typedef {'beginner'|'intermediate'|'professional'} Level */

export const general = [
  {
    id: 'what-is-cubesat',
    question: 'What is a CubeSat?',
    answers: {
      beginner: 'A CubeSat is a tiny satellite, about the size of a box of tissues. Students and scientists build them to test ideas in space. The model you can print on this site is the same size: 10 cm on each side.',
      intermediate: 'A CubeSat is a small satellite built from standard 10 cm cubes called units, or "U". A 1U CubeSat weighs about a kilogram or two. Many carry a sensor, a radio and a battery, and talk to ground stations when they pass overhead.',
      professional: 'A CubeSat follows a standard form factor: 1U is 10 × 10 × 11.35 cm, and larger buses stack units. Most fly in low Earth orbit, deploy from a standard dispenser, and need a mission integrator, frequency coordination and a licensed operator. This project uses 1U as the classroom bench size.',
    },
  },
  {
    id: 'what-is-ground-station',
    question: 'What does a ground station do?',
    answers: {
      beginner: 'A ground station is the satellite\'s friend on Earth. It sends questions up, like "what is your temperature?", and listens for the answers coming back down.',
      intermediate: 'A ground station sends commands to a satellite and receives its data, called telemetry. It can only talk while the satellite is above the horizon, so it has to be ready when the pass begins.',
      professional: 'Here the ground station publishes commands on `classroom/commands`, subscribes to replies on `classroom/payload`, and checks each sample\'s age and request id. A flight station adds pass prediction, antenna pointing, licensed operation and an audit trail.',
    },
  },
  {
    id: 'why-pause',
    question: 'What happens when contact is lost?',
    answers: {
      beginner: 'When contact is lost, nothing new can come down. The station keeps the last reading, marks it as historical, and waits. When contact comes back, you ask again.',
      intermediate: 'Messages already in flight can be lost, so a picture burst may arrive incomplete. The station marks it incomplete instead of pretending it is whole, and old readings become historical after 5 seconds.',
      professional: 'Pausing contact closes the bridge session between the relays. In-flight groups stall and are marked incomplete; commands older than 5 s are rejected and recent request ids are deduplicated, so a delayed command cannot run twice after recovery.',
    },
  },
];

/** Two questions per example quest: a hint and why it matters. */
export const byQuest = {
  'first-contact': {
    hint: {
      beginner: 'Press "Ask for a reading" and watch the three boxes fill in. Copy each number and the letters right after it. Those letters are the unit.',
      intermediate: 'Look at the log line that starts with "Reply in". It shows how long the round trip took. Take three readings and compare the times.',
      professional: 'Open "Raw telemetry". Note `sequence` and `captured_unix_ms`, and compare the reply id with the id in the PUB line. Compute the mean and spread of three round-trip times.',
    },
    why: {
      beginner: 'A number without a unit is a mystery. 20 could be 20 degrees, 20 volts or 20 of anything. Units let the next crew understand you.',
      intermediate: 'Real missions log units, time and source with every value. Without them, data from two teams cannot be compared.',
      professional: 'Provenance (units, capture time, source, sequence) is what makes telemetry evidence instead of anecdote. The station rejects samples with impossible timestamps for the same reason.',
    },
  },
  'bring-back-a-picture': {
    hint: {
      beginner: 'Press "Take pictures" and count out loud as each picture appears. Watch where the white square goes.',
      intermediate: 'Pictures arrive every half second. Time the whole burst and divide by the number of frames to check.',
      professional: 'When the group completes, the log lists an estimated size for each JPEG frame. Add them up and divide by the burst time to get the bit rate.',
    },
    why: {
      beginner: 'Sending many small pictures means that if one gets lost, the others still make it down.',
      intermediate: 'Radio links lose data. Small, separate frames limit how much one loss can spoil.',
      professional: 'Each frame is independently decodable, so loss is bounded to that frame. With inter-frame video such as H.264, a lost reference frame stalls the group until the next keyframe, which is how moq-subghz handles it.',
    },
  },
  'lose-the-signal': {
    hint: {
      beginner: 'Start the pictures, wait until a few arrive, then press "Pause contact". Then press "Restore contact" and ask again.',
      intermediate: 'Pause during the burst, then read the log. It tells you how many frames arrived and how the burst was marked.',
      professional: 'Pause mid-group and note the stall point. After restoring, check that the next reply carries a new request id.',
    },
    why: {
      beginner: 'A real satellite flies out of sight after a few minutes. Ground crews practise what to do when the signal goes away.',
      intermediate: 'Every pass ends with loss of signal. Planning what to ask for first, when the next pass starts, saves precious minutes.',
      professional: 'Short commanding windows make idempotency and expiry essential. A command that arrives late must not run twice or run stale.',
    },
  },
  'fresh-or-historical': {
    hint: {
      beginner: 'Ask for a reading, then just watch the line under the numbers. Count the seconds until it changes from Fresh to Historical.',
      intermediate: 'Note the age when the label flips. Then pause contact and see whether a reading can still be fresh.',
      professional: 'Compare `captured_unix_ms` with the moment the label flips, and find `MAX_AGE` in `src/main.rs`.',
    },
    why: {
      beginner: 'Old news can trick you. If you think a reading is new but it is old, you might make the wrong choice.',
      intermediate: 'Decisions based on stale data can be worse than no data. A clear age limit stops the crew from being fooled.',
      professional: 'Freshness bounds make staleness explicit. A real system also needs clock-uncertainty metadata before trusting an age computed from two clocks.',
    },
  },
  'light-and-shadow': {
    hint: {
      beginner: 'Make a two-column table: time and light. Ask for a reading every 10 seconds and write each one down.',
      intermediate: 'Record a reading every 5 seconds for a minute, then look for how long the light stays on and off.',
      professional: 'Sample once per second and record `sequence`. Test the rule "lit when sequence mod 30 is below 20" against your data.',
    },
    why: {
      beginner: 'Satellites go around the Earth. Part of the time they are in the Earth\'s shadow, where it is dark.',
      intermediate: 'Eclipse matters for power and temperature. Solar panels stop charging and parts cool down in the shadow.',
      professional: 'Eclipse fraction drives the power budget, battery depth of discharge and thermal design. A real model computes it from the orbit and the sun vector.',
    },
  },
};

export const questTopics = [
  { key: 'hint', question: 'Give me a hint' },
  { key: 'why', question: 'Why does this matter?' },
];

const levels = ['beginner', 'intermediate', 'professional'];

/** Look up a prewritten answer. Returns null for unknown ids. */
export function answerFor(quest, questionId, level) {
  if (!levels.includes(level)) return null;
  const g = general.find(q => q.id === questionId);
  if (g) return g.answers[level];
  const q = Object.hasOwn(byQuest, quest) ? byQuest[quest] : null;
  if (q && Object.hasOwn(q, questionId)) return q[questionId][level];
  return null;
}

/** The fixed question text sent to the live model for a valid id. */
export function questionText(quest, questionId) {
  const g = general.find(q => q.id === questionId);
  if (g) return g.question;
  const topic = questTopics.find(t => t.key === questionId);
  return topic && Object.hasOwn(byQuest, quest) ? topic.question : null;
}
