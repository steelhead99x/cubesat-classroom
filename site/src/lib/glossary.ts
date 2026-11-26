import { getCollection, type CollectionEntry } from 'astro:content';

export type Term = CollectionEntry<'glossary'>;

/** Every glossary entry, A–Z by term, with `related` checked so a typo fails the build. */
export async function glossary(): Promise<Term[]> {
  const entries = (await getCollection('glossary')).sort((a, b) => a.data.term.localeCompare(b.data.term, 'en', { sensitivity: 'base' }));
  const ids = new Set(entries.map(entry => entry.id));
  for (const entry of entries) {
    if (entry.id !== entry.data.slug) throw new Error(`glossary/${entry.id}: slug "${entry.data.slug}" must match the file name`);
    for (const slug of entry.data.related) if (!ids.has(slug)) throw new Error(`glossary/${entry.id}: related "${slug}" is not an entry`);
    for (const slide of entry.data.deck) if (!(slide in deckSlides)) throw new Error(`glossary/${entry.id}: deck slide "${slide}" is not in deckSlides`);
  }
  return entries;
}

/** First letter for the A–Z index; digits go under "#". */
export const letterOf = (term: string) => {
  const c = term.normalize('NFD').replace(/[^A-Za-z0-9]/g, '').charAt(0).toUpperCase();
  return /[A-Z]/.test(c) ? c : '#';
};

/** The deck lives behind a password at hikd.ai/tis/deck/; each slide has a slug. */
export const deckUrl = (slide: string) => `https://hikd.ai/tis/deck/#${slide}`;

/** JSON for a <script type="application/ld+json"> block, safe inside HTML. */
export const ldJson = (value: unknown) => JSON.stringify(value).replace(/</g, '\\u003c');

/** Deck slide slugs and titles (hikd.ai/tis/deck/#<slug>). Keep in step with the deck repo. */
export const deckSlides: Record<string, string> = {
  'mission': 'Get more humans to space',
  'teachers-waiting': 'Trained teachers are waiting',
  'where-it-started': 'From Teacher in Space to classroom radio',
  'where-its-going': 'From classroom radio to flight',
  'build-step-by-step': 'Build the CubeSat, step by step',
  'rockoons': 'Rockoons reach toward space, not orbit',
  'fly-without-permit': 'Fly today without a permit',
  'rockoon-simulation': 'Rockoon simulation: Sacramento to Syracuse',
  'live-video-915': 'Live video over 915 MHz',
  'live-demo': 'One radio link, every classroom',
  'safe-by-default': 'Safe by default',
  'swag': 'Swag students earn',
  'roadmap': 'Same kit, more frontiers',
  'use-cases': 'What students fly',
  'watch-it-live': 'Every classroom, live for the return to the Moon',
  'why-you': 'Why SpaceX, xAI and Terafab',
  'the-ask': 'Fund by the school or flight',
  'cost-per-school': 'Cost per school',
  'flight-tier': 'Flight tier and add-ons',
  'example-1000-schools': 'Example: 1,000 schools, 9 satellites',
  '915-to-students': '915 MHz to every student',
};
