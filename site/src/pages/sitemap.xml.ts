import { getCollection } from 'astro:content';
import { siteOrigin } from '../origin';
import { glossary } from '../lib/glossary';

// Canonical URLs only. The GitHub Pages mirror points here too, so one sitemap
// covers the public site whether it is served by the Worker or by Pages.
export async function GET() {
  const quests = (await getCollection('quests')).sort((a, b) => a.data.order - b.data.order);
  const terms = await glossary();
  const paths = [
    '/',
    '/mission/',
    '/quests/',
    ...quests.map(quest => `/quests/${quest.id}/`),
    '/hardware/',
    '/wiki/',
    ...terms.map(term => `/wiki/${term.id}/`),
    '/plans/',
    '/open-source/',
    '/volunteer/',
    '/hub/mobile/',
    '/hub/lessons/',
    '/hub/station/',
    '/hub/develop/',
  ];
  const body = [
    '<?xml version="1.0" encoding="UTF-8"?>',
    '<urlset xmlns="http://www.sitemaps.org/schemas/sitemap/0.9">',
    ...paths.map(path => `  <url><loc>${siteOrigin}${path}</loc></url>`),
    '</urlset>',
    '',
  ].join('\n');
  return new Response(body, {
    headers: { 'content-type': 'application/xml; charset=utf-8' },
  });
}
