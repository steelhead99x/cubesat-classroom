import { defineConfig } from 'astro/config';
import tailwindcss from '@tailwindcss/vite';

export default defineConfig({
  // assetsInlineLimit 0 keeps every script a file, so the CSP can say script-src 'self'.
  vite: { plugins: [tailwindcss()], build: { assetsInlineLimit: 0 } },
});
