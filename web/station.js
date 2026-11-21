const el = id => document.getElementById(id);
const requests = [...document.querySelectorAll('[data-request]')];
let station;
let frameId;
let pending = false;
let requestError = '';
let activeRequest = '';
const labels = Object.fromEntries(requests.map(button => [button.dataset.request, button.textContent]));

function ago(ms) {
  const seconds = Math.max(0, Math.round(ms / 1000));
  if (seconds < 60) return `${seconds} s ago`;
  const minutes = Math.round(seconds / 60);
  return `${minutes} min ago`;
}

function render() {
  const {online, fresh, display} = station;
  el('contactState').textContent = online ? 'Contact enabled' : 'Contact paused';
  el('contact').textContent = online ? 'Pause contact' : 'Restore contact';
  el('contact').disabled = false;
  el('contact').parentElement.classList.toggle('paused', !online);
  requests.forEach(button => {
    button.disabled = pending || !online;
    button.textContent = pending && activeRequest === button.dataset.request ? 'Waiting for reply…' : labels[button.dataset.request];
  });
  el('route').classList.toggle('paused', !online);
  el('route').classList.toggle('busy', online && pending);
  el('routeNote').textContent = !online
    ? 'Simulated radio link · contact paused'
    : pending ? 'Simulated radio link · request in flight' : 'Simulated radio link · ready';
  const sample = display.sample;
  if (sample) {
    el('temp').textContent = `${sample.temperature_c.toFixed(1)} °C`;
    el('voltage').textContent = `${sample.bus_voltage_v.toFixed(3)} V`;
    el('light').textContent = `${sample.light_lux} lx`;
  }
  el('readingStatus').textContent = sample
    ? `${fresh ? 'Fresh reading' : 'Last reading · historical'} · ${sample.source}`
    : 'No reading yet';
  el('readingAge').textContent = sample ? `Captured ${ago(Date.now() - sample.captured_unix_ms)}` : '';
  el('readingStatus').classList.toggle('fresh', !!(sample && fresh));
  const count = display.frame_count;
  el('frames').textContent = `${count} of 8 frames · simulated video`;
  el('video').hidden = count === 0;
  el('videoEmpty').hidden = count > 0;
  if (count && frameId !== display.frame_id) {
    frameId = display.frame_id;
    el('video').src = `/api/frame.jpg?v=${frameId}`;
  }
  let complete = 0;
  for (const [id, done] of [
    ['sensorQuest', !!sample], ['videoQuest', display.video_received], ['recoveryQuest', display.recovery_seen]
  ]) {
    el(id).textContent = done ? '✓' : '○';
    el(id).setAttribute('aria-label', done ? 'Complete' : 'Open');
    el(id).parentElement.classList.toggle('done', !!done);
    complete += done ? 1 : 0;
  }
  el('questCount').textContent = complete === 3 ? 'All 3 complete · write your crew log' : `${complete} of 3 complete`;
  el('questMeter').value = complete;
  const message = requestError || display.event;
  if (el('event').textContent !== message) el('event').textContent = message;
  el('event').classList.toggle('error', !!requestError);
}

async function refresh() {
  try {
    const response = await fetch('/api/state');
    if (!response.ok) throw Error();
    station = await response.json();
    render();
  } catch {
    requests.forEach(button => { button.disabled = true; });
    el('contact').disabled = true;
    el('contactState').textContent = 'Station unavailable';
    el('event').textContent = 'Waiting for the local station to reconnect.';
  }
}

async function post(path, value) {
  const response = await fetch(path, {
    method: 'POST', headers: {'Content-Type': 'application/json'}, body: JSON.stringify(value)
  });
  if (!response.ok) throw Error(await response.text());
}

requests.forEach(button => button.addEventListener('click', async () => {
  pending = true;
  activeRequest = button.dataset.request;
  requestError = '';
  render();
  try { await post('/api/request', button.dataset.request); }
  catch (error) { requestError = error.message; }
  finally { pending = false; await refresh(); }
}));

el('contact').addEventListener('click', async () => {
  if (!station) return;
  requestError = '';
  try { await post('/api/contact', !station.online); }
  catch (error) { requestError = error.message; }
  await refresh();
});

const log = el('log');
for (const name of ['alias', 'call']) {
  try { log.elements[name].value = localStorage.getItem(`cubesat.${name}`) || ''; } catch {}
}
log.addEventListener('submit', event => {
  event.preventDefault();
  const alias = log.elements.alias.value.trim();
  const call = log.elements.call.value.trim().toUpperCase();
  const observation = log.elements.observation.value.trim();
  if (!alias || !observation) {
    el('saved').textContent = 'Add a nickname and an observation first.';
    return;
  }
  if (call && (!/^[A-Z0-9]+(?:\/[A-Z0-9]+){0,2}$/.test(call) || !/[0-9]/.test(call))) {
    el('saved').textContent = 'Check the call sign, or leave it blank.';
    return;
  }
  try {
    localStorage.setItem('cubesat.alias', alias);
    localStorage.setItem('cubesat.call', call);
  } catch {}
  const evidence = station?.display;
  const sample = evidence?.sample;
  const lines = [
    '# CubeSat Classroom · Crew log', '', `Team: ${alias}`,
    ...(call ? [`Mentor call sign (self-reported): ${call}`] : []),
    `Date: ${new Date().toISOString().slice(0, 10)}`, '',
    '## Station record', '',
    sample ? `- Last sample: ${sample.temperature_c.toFixed(1)} °C, ${sample.bus_voltage_v.toFixed(3)} V, ${sample.light_lux} lx (${sample.source})` : '- No sensor reading received',
    `- Frames in latest burst: ${evidence?.frame_count ?? 0}/8 (simulated video)`,
    `- New reply after contact resumed: ${evidence?.recovery_seen ? 'yes' : 'not yet'}`, '',
    '## What we observed and would test next', '', observation, ''
  ];
  const url = URL.createObjectURL(new Blob([lines.join('\n')], {type: 'text/markdown'}));
  const file = Object.assign(document.createElement('a'), {href: url, download: 'cubesat-crew-log.md'});
  file.click();
  setTimeout(() => URL.revokeObjectURL(url), 1000);
  el('saved').textContent = 'Log downloaded. Review it with your teacher before sharing.';
});

async function poll() {
  await refresh();
  setTimeout(poll, 350);
}
poll();
