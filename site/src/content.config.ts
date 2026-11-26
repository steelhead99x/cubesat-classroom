import { defineCollection } from 'astro:content';
import { glob } from 'astro/loaders';
import { z } from 'astro/zod';

// Example quests that run against the in-browser station demo. Each quest has
// three levels; the page shows the one the visitor picks. The full quest board
// lives in /quest at the repository root.
const level = z.object({
  steps: z.array(z.string()).min(2),
  check: z.string(),
  discuss: z.string(),
});

const quests = defineCollection({
  loader: glob({ pattern: '*.md', base: './src/content/quests' }),
  schema: z.object({
    title: z.string(),
    patch: z.string().max(14),
    order: z.number(),
    summary: z.string(),
    minutes: z.number(),
    skill: z.string(),
    beginner: level,
    intermediate: level,
    professional: level,
  }),
});

// The public glossary at /wiki/. One Markdown file per term in src/content/glossary/.
// The file name (or `slug`) is the URL: /wiki/<slug>/. `related` lists other entries' slugs
// (checked at build time), `deck` lists slide slugs in the mission briefing deck.
const glossary = defineCollection({
  loader: glob({ pattern: '*.md', base: './src/content/glossary' }),
  schema: z.object({
    term: z.string(),
    slug: z.string().regex(/^[a-z0-9-]+$/),
    summary: z.string().max(160),
    category: z.enum(['Spacecraft', 'Launch and flight', 'Radio', 'Rules and safety', 'Software and data', 'Organizations and programs', 'Hardware']),
    aliases: z.array(z.string()).default([]),
    related: z.array(z.string()).default([]),
    seeAlso: z.array(z.object({ label: z.string(), href: z.string() })).default([]),
    sources: z.array(z.object({ label: z.string(), url: z.string().url() })).default([]),
    deck: z.array(z.string().regex(/^[a-z0-9-]+$/)).default([]),
  }),
});

export const collections = { quests, glossary };
