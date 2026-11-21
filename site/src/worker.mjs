import { hubs } from './hubs.mjs';

const homeByHost = new Map(hubs.filter(hub => hub.slug !== 'lessons').map(({ host, slug }) => [host, slug === 'volunteer' ? '/volunteer/' : '/hub/' + slug + '/']));

export default {
  fetch(request, env) {
    const url = new URL(request.url);
    const home = homeByHost.get(url.hostname);
    if (url.pathname === '/' && home) {
      url.pathname = home;
      return env.ASSETS.fetch(new Request(url, request));
    }
    return env.ASSETS.fetch(request);
  },
};
