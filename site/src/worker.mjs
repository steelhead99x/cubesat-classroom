import { hubs } from './hubs.mjs';

const homeByHost = new Map(hubs.map(({ host, slug }) => [host, '/hub/' + slug + '/']));

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
