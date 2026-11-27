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

/**
 * Minimal inline formatting for quest steps: **bold**, `code` and [links](/path/). A link to a
 * site path (starting with /) gets the site's base path; # anchors and https links are kept;
 * anything else stays plain text.
 */
export function inline(text: string, base = ''): string {
  const escape = (value: string) => value.replace(/&/g, '&amp;').replace(/</g, '&lt;').replace(/>/g, '&gt;');
  const format = (value: string) => escape(value).replace(/\*\*(.+?)\*\*/g, '<strong>$1</strong>');
  const tokens = /`([^`]+)`|\[([^\]]+)\]\(([^)\s]+)\)/g;
  let out = '', end = 0;
  for (const match of text.matchAll(tokens)) {
    const [whole, code, label, href] = match;
    out += format(text.slice(end, match.index));
    end = match.index + whole.length;
    if (code !== undefined) {
      out += `<code>${escape(code)}</code>`;
      continue;
    }
    const local = /^\/(?![/\\])/.test(href) && !href.includes('\\');
    const allowed = local || href.startsWith('#') || href.startsWith('https://');
    const title = format(label).replace(/`(.+?)`/g, '<code>$1</code>');
    const url = escape((local ? base : '') + href).replace(/"/g, '&quot;');
    out += allowed ? `<a href="${url}">${title}</a>` : title;
  }
  return out + format(text.slice(end));
}
