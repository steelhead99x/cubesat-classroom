export const levels = [
  {
    id: 'beginner',
    name: 'Beginner',
    who: '13 and up and new to satellites, or anyone new',
    intro: 'A ground station is a computer on Earth that talks to a satellite. You ask a question, wait, and read the answer. Plain words, hints along the way.',
  },
  {
    id: 'intermediate',
    name: 'Intermediate',
    who: 'Middle and high school, or new radio operators',
    intro: 'Requests and replies take time, data can arrive incomplete, and old data can mislead you. You will record timing, units and sequence numbers.',
  },
  {
    id: 'professional',
    name: 'Professional',
    who: 'Advanced students, mentors and engineers',
    intro: 'See the station the way its software does: MoQ broadcasts and tracks, groups and frames, the raw telemetry.v1 JSON and round-trip times.',
  },
] as const;

export type LevelId = typeof levels[number]['id'];

/** Minimal inline formatting for quest steps: **bold** and `code`. */
export function inline(text: string): string {
  return text
    .replace(/&/g, '&amp;').replace(/</g, '&lt;').replace(/>/g, '&gt;')
    .replace(/\*\*(.+?)\*\*/g, '<strong>$1</strong>')
    .replace(/`(.+?)`/g, '<code>$1</code>');
}
