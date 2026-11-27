// The games' icon set: one family of line icons drawn for this subject (balloons, rockets,
// rings, antennas, satellites), on a 24-unit grid with round 1.75-unit strokes in the current
// text colour, so they sit with Inter and the mission-briefing look. Every icon is decorative:
// the button or label next to it carries the words.
const P = {
  help: '<circle cx="12" cy="12" r="8.5"/><path d="M9.6 9.4a2.5 2.5 0 1 1 3.4 2.3c-.7.3-1 .8-1 1.6v.4"/><path d="M12 16.6v.1"/>',
  'sound-on': '<path d="M4 9.6h3.2L11.5 6v12l-4.3-3.6H4z"/><path d="M15.2 9.2a4 4 0 0 1 0 5.6M17.8 6.6a7.6 7.6 0 0 1 0 10.8"/>',
  'sound-off': '<path d="M4 9.6h3.2L11.5 6v12l-4.3-3.6H4z"/><path d="M15.5 9.5l5 5M20.5 9.5l-5 5"/>',
  pause: '<path d="M9 6v12M15 6v12"/>',
  play: '<path d="M8 5.8v12.4L18.2 12z"/>',
  fullscreen: '<path d="M4.5 9V4.5H9M15 4.5h4.5V9M19.5 15v4.5H15M9 19.5H4.5V15"/>',
  exit: '<path d="M13.5 4.5h5v15h-5"/><path d="M9.5 8l-4 4 4 4M5.5 12h9"/>',
  restart: '<path d="M4.8 13a7.5 7.5 0 1 0 2.1-6.2"/><path d="M5 4.5v4h4"/>',
  credit: '<circle cx="12" cy="12" r="8"/><circle cx="12" cy="12" r="4"/>',
  star: '<path d="M12 4.2l2.4 4.9 5.4.8-3.9 3.8.9 5.4L12 16.6l-4.8 2.5.9-5.4-3.9-3.8 5.4-.8z"/>',
  balloon: '<path d="M12 3.5a5.8 5.8 0 0 0-5.8 5.8c0 3.5 3.1 6.4 5.8 7.3 2.7-.9 5.8-3.8 5.8-7.3A5.8 5.8 0 0 0 12 3.5z"/><path d="M11 16.6l1 1.4 1-1.4M12 18v2.5M10 20.5h4"/>',
  rocket: '<path d="M12 3.2c2.9 2.2 4.3 5.4 4.3 9.3v3.2H7.7v-3.2c0-3.9 1.4-7.1 4.3-9.3z"/><path d="M7.7 12.8 5 16.5h2.7M16.3 12.8l2.7 3.7h-2.7M10.4 18.5l1.6 2.3 1.6-2.3"/><circle cx="12" cy="9.6" r="1.4"/>',
  ring: '<circle cx="12" cy="12" r="8"/><circle cx="12" cy="12" r="3.2"/>',
  wind: '<path d="M3.5 9h10a2.8 2.8 0 1 0-2.8-2.8M3.5 15h13.5a2.8 2.8 0 1 1-2.8 2.8M3.5 12h6"/>',
  aim: '<path d="M4 19.5h16M4 19.5 15.5 8"/><path d="M10.5 19.5a6.5 6.5 0 0 0-1.9-4.6"/><path d="M12.5 8h3v3"/>',
  power: '<path d="M4.6 16.5a8 8 0 1 1 14.8 0"/><path d="m12 14.5 3.8-4.6"/><circle cx="12" cy="14.5" r="1.3"/>',
  crew: '<circle cx="9" cy="8.5" r="3"/><path d="M3.5 19.5a5.5 5.5 0 0 1 11 0"/><circle cx="17" cy="9.5" r="2.4"/><path d="M15.6 14.1a4.6 4.6 0 0 1 5.4 4.4"/>',
  crate: '<path d="M4 8l8-3.8L20 8v8l-8 3.8L4 16z"/><path d="M4 8l8 3.8L20 8M12 11.8v8"/>',
  satellite: '<rect x="9.5" y="8.5" width="5" height="7" rx="1"/><rect x="2.5" y="9.5" width="5.2" height="5" rx=".6"/><rect x="16.3" y="9.5" width="5.2" height="5" rx=".6"/><path d="M7.7 12h1.8M14.5 12h1.8M12 8.5V6.2M10.8 6.2h2.4"/>',
  dish: '<path d="M5.2 12.8a6.9 6.9 0 0 0 9.8 0L5.2 3a6.9 6.9 0 0 0 0 9.8z"/><path d="m10 7.9 3.6-3.6M14.6 4.3l1.4-1.4M8.6 16.6 7 20.5h7l-1.6-3.9"/>',
  signal: '<path d="M12 12.5v8M8.4 8.9a5 5 0 0 1 7.2 0M5.8 6.3a8.7 8.7 0 0 1 12.4 0"/><circle cx="12" cy="12" r="1.3"/>',
  battery: '<rect x="3" y="7.5" width="15.5" height="9" rx="1.6"/><path d="M21 10.5v3M6.2 10.7v2.6M9.2 10.7v2.6"/>',
  packets: '<rect x="4" y="4.5" width="16" height="4.5" rx="1"/><rect x="4" y="10.5" width="16" height="4.5" rx="1"/><path d="M4 18.5h9.5"/>',
  streak: '<path d="m4 16.5 5-5 3.5 3.5L19.5 8"/><path d="M15 8h4.5v4.5"/>',
  clock: '<circle cx="12" cy="12" r="8.5"/><path d="M12 7.5V12l3 2"/>',
  globe: '<circle cx="12" cy="12" r="8.5"/><path d="M3.5 12h17M12 3.5c2.4 2.3 3.4 5.2 3.4 8.5s-1 6.2-3.4 8.5c-2.4-2.3-3.4-5.2-3.4-8.5s1-6.2 3.4-8.5z"/>',
  levels: '<path d="M4 19.5h4.5v-5H4zM9.75 19.5h4.5v-9h-4.5zM15.5 19.5H20v-13h-4.5z"/>',
  fast: '<path d="m5 6.5 5.5 5.5L5 17.5M12.5 6.5 18 12l-5.5 5.5"/>',
  middle: '<path d="m9 6.5 5.5 5.5L9 17.5"/>',
  far: '<circle cx="4.5" cy="12" r="1.1"/><circle cx="9" cy="12" r="1.1"/><path d="m13 6.5 5.5 5.5-5.5 5.5"/>',
  transmit: '<path d="M12 11.5v9M9.5 20.5h5"/><path d="M8.6 7.6a4.8 4.8 0 0 0 0 7M15.4 7.6a4.8 4.8 0 0 1 0 7M6 5a8.5 8.5 0 0 0 0 12.2M18 5a8.5 8.5 0 0 1 0 12.2"/><circle cx="12" cy="11" r="1"/>',
  trophy: '<path d="M8 4.5h8v4.8a4 4 0 0 1-8 0z"/><path d="M8 6.5H5.2a3 3 0 0 0 3 3.6M16 6.5h2.8a3 3 0 0 1-3 3.6M12 13.3v3.2M8.5 20h7M10 16.5h4"/>',
  knob: '<circle cx="12" cy="12" r="6.5"/><path d="M12 5.5v3.5M3.5 12h1.6M18.9 12h1.6M6 6l1.1 1.1M18 6l-1.1 1.1"/>',
  start: '<path d="M8 5.8v12.4L18.2 12z"/>',
  link: '<path d="M10.2 13.8a3.8 3.8 0 0 0 5.4 0l3-3a3.8 3.8 0 0 0-5.4-5.4l-.9.9"/><path d="M13.8 10.2a3.8 3.8 0 0 0-5.4 0l-3 3a3.8 3.8 0 0 0 5.4 5.4l.9-.9"/>',
  check: '<path d="m5 12.5 4.5 4.5L19 7.5"/>',
};

export const ICON_NAMES = Object.keys(P);

/** An icon as inline SVG markup. Decorative: hidden from screen readers. */
export function icon(name, size = 20) {
  const body = P[name];
  if (!body) throw new Error(`no icon named ${name}`);
  return `<svg class="ico" width="${size}" height="${size}" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.75" stroke-linecap="round" stroke-linejoin="round" aria-hidden="true" focusable="false">${body}</svg>`;
}
