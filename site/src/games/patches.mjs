// Mission patches for the games: round emblems in the CubeSat Classroom colours (navy field,
// white ring, flag-red and solar-gold accents), with the game's name set around the top like an
// AMSAT or NASA mission patch. Used on the Games page and each game's title card.
const NAVY = '#01017f', DEEP = '#00005c', FLAG = '#ea0303', GOLD = '#ffd23f', SKY = '#a4c9ff';

function frame(id, name, art) {
  return `<svg class="patch" viewBox="0 0 160 160" role="img" aria-label="${name} mission patch">
  <defs>
    <path id="${id}-top" d="M24 80a56 56 0 0 1 112 0"/>
    <path id="${id}-bot" d="M30 80a50 50 0 0 0 100 0"/>
    <clipPath id="${id}-clip"><circle cx="80" cy="80" r="47"/></clipPath>
  </defs>
  <circle cx="80" cy="80" r="77" fill="${NAVY}" stroke="#fff" stroke-width="3"/>
  <circle cx="80" cy="80" r="48.5" fill="${DEEP}" stroke="#fff" stroke-width="1.5"/>
  <g clip-path="url(#${id}-clip)">${art}</g>
  <text fill="#fff" font-family="IBM Plex Mono, ui-monospace, monospace" font-size="11.5" font-weight="600" letter-spacing="2.6"><textPath href="#${id}-top" startOffset="50%" text-anchor="middle">${name.toUpperCase()}</textPath></text>
  <text fill="${SKY}" font-family="IBM Plex Mono, ui-monospace, monospace" font-size="7.5" font-weight="500" letter-spacing="2.2"><textPath href="#${id}-bot" startOffset="50%" text-anchor="middle" dominant-baseline="hanging">CUBESAT CLASSROOM</textPath></text>
  <circle cx="21" cy="80" r="2.2" fill="${FLAG}"/><circle cx="139" cy="80" r="2.2" fill="${FLAG}"/>
</svg>`;
}

const ART = {
  // A balloon climbing off the hills, its rocket arcing through the gold ring.
  rockoon: `
    <path d="M30 112c14-10 26-7 38-3s24 2 34-6 20-7 30-2v40H30z" fill="#1d2a6b"/>
    <path d="M30 112c14-10 26-7 38-3s24 2 34-6 20-7 30-2" fill="none" stroke="#6fd3a0" stroke-width="2"/>
    <path d="M62 82c8-30 30-42 52-38" fill="none" stroke="${FLAG}" stroke-width="2" stroke-dasharray="1 5" stroke-linecap="round"/>
    <circle cx="104" cy="50" r="10" fill="none" stroke="${GOLD}" stroke-width="3"/>
    <path d="M58 58a9 9 0 0 0-9 9c0 5.6 4.9 10 9 11.4 4.1-1.4 9-5.8 9-11.4a9 9 0 0 0-9-9z" fill="#fff"/>
    <path d="M58 78.4v12M54 90.4h8v5h-8z" fill="none" stroke="#fff" stroke-width="1.6" stroke-linejoin="round"/>
    <path d="M118 40l5-4-1 6.4" fill="#fff"/>
    <path d="M110.5 45.5l9-6.5" stroke="#fff" stroke-width="3" stroke-linecap="round"/>`,
  // A ground station dish tracking a CubeSat along its pass, horizon to horizon.
  pass: `
    <path d="M28 104c18-8 34-10 52-10s34 2 52 10v40H28z" fill="#1d2a6b"/>
    <path d="M36 98C50 58 110 58 124 98" fill="none" stroke="#fff" stroke-width="1.6" stroke-dasharray="2 4" stroke-linecap="round" opacity=".8"/>
    <g transform="translate(92 62) rotate(18)">
      <rect x="-5" y="-6" width="10" height="12" rx="1.5" fill="#fff"/>
      <rect x="-19" y="-4" width="11" height="8" fill="${SKY}" stroke="#fff" stroke-width="1"/>
      <rect x="8" y="-4" width="11" height="8" fill="${SKY}" stroke="#fff" stroke-width="1"/>
    </g>
    <path d="M70 84a10 10 0 0 1 12-6M66 79a17 17 0 0 1 20-10" fill="none" stroke="${GOLD}" stroke-width="2.2" stroke-linecap="round"/>
    <path d="M50 100a12 12 0 0 0 17-17z" fill="#fff"/>
    <path d="M58.5 92 64 86.5M55 98l-4 10h12l-4-10" fill="none" stroke="#fff" stroke-width="2" stroke-linejoin="round"/>
    <path d="M28 104h104" stroke="${FLAG}" stroke-width="2"/>`,
};

/** A game's mission patch as inline SVG. `game` is 'rockoon' or 'pass'. */
export function patch(game, name) {
  return frame(`patch-${game}`, name, ART[game]);
}
