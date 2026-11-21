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

export const collections = { quests };
