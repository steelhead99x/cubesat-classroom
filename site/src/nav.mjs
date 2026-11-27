// The one main menu for every CubeSat Classroom page. Pages pass `current` (a key below) to SiteHeader.
// `path` is on this site (the base URL is added); `href` is a full address on another site.
export const mainNav = [
  { key: 'lessons', label: 'Lessons', path: '/quests/' },
  { key: 'games', label: 'Games', path: '/games/' },
  { key: 'simulator', label: 'Simulator', path: '/#demo' },
  { key: 'mission', label: 'Mission', path: '/mission/' },
  { key: 'glossary', label: 'Glossary', path: '/wiki/' },
];

export const moreNav = [
  {
    group: 'Build',
    items: [
      { key: 'hardware', label: 'CAD and print files', path: '/hardware/' },
      { key: 'workbook', label: 'Rockoon workbook', path: '/rockoon/workbook/' },
      { key: 'labs', label: 'Hands-on labs', path: '/#labs' },
    ],
  },
  {
    group: 'Community',
    items: [
      { key: 'teachers', label: 'For teachers', path: '/#teachers' },
      { key: 'app', label: 'cubesat.app', href: 'https://cubesat.app/' },
      { key: 'open-source', label: 'Open source', path: '/open-source/' },
      { key: 'plans', label: 'Plans', path: '/plans/' },
    ],
  },
];

export const navHref = (item, base) => item.href ?? base + item.path;
