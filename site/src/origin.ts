import { publicOrigin } from './hubs.mjs';

// The address the public site is served at, used for canonical and social-card URLs.
// `npm run build:github` sets PUBLIC_SITE_ORIGIN from docs/CNAME (or the github.io address
// while there is none). Other builds, such as the Worker's, use `publicOrigin` from hubs.mjs.
export const siteOrigin: string = (import.meta.env.PUBLIC_SITE_ORIGIN || publicOrigin).replace(/\/$/, '');
