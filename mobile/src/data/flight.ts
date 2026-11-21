// Desk simulation of a suborbital research flight. Same model as the
// `flight-profile` command (src/flight.rs in the repository), built from public
// figures: release at 44,562 ft, a 60 s burn, apogee 54.4 miles, 70 minutes in all.
// Not flight data and not a flight plan.
const G = 9.80665;
const releaseM = 44_562 * 0.3048;
const apogeeM = 54.4 * 1609.344;
const burnS = 60;
const releaseS = 50 * 60;
const landingS = 70 * 60;

const half = burnS / 2;
const speed = G * (-half + Math.sqrt(half * half + (2 * (apogeeM - releaseM)) / G));
const burnoutM = releaseM + speed * half;
const coast = speed / G;
const burnoutS = releaseS + burnS;
const apogeeS = burnoutS + coast;
const reentryS = apogeeS + coast;
const glideM = 50_000 * 0.3048;
const glideS = reentryS + 180;

export const flight = {
  releaseS, burnoutS, apogeeS, reentryS, glideS, landingS,
  releaseM, burnoutM, apogeeM, glideM,
  microgravityS: 2 * coast,
  boostG: (speed / burnS + G) / G,
  cabinKpa: 82.7,
};

const smooth = (x: number) => x * x * (3 - 2 * x);

export function altitudeAt(t: number): number {
  if (t < releaseS) return releaseM * smooth(t / releaseS);
  if (t < burnoutS) return releaseM + 0.5 * (speed / burnS) * (t - releaseS) ** 2;
  if (t < reentryS) return apogeeM - 0.5 * G * (t - apogeeS) ** 2;
  if (t < glideS) return burnoutM + (glideM - burnoutM) * (1 - (1 - (t - reentryS) / 180) ** 3);
  if (t < landingS) return glideM * (1 - (t - glideS) / (landingS - glideS));
  return 0;
}

export type Phase = {
  id: string;
  name: string;
  start: number;
  end: number;
  feel: string;
  sensors: { beginner: string; intermediate: string; professional: string };
};

export const phases: Phase[] = [
  {
    id: 'climb', name: 'Climb under the carrier aircraft', start: 0, end: releaseS,
    feel: 'A long, slow climb. You feel your normal weight.',
    sensors: {
      beginner: 'Nothing much changes. The kit must already be switched on and recording.',
      intermediate: 'Acceleration stays near 1 g. Cabin pressure settles near 83 kPa, like a mountain town.',
      professional: 'About 1 g with jet-engine vibration. Cabin held near a 5,000 to 6,000 ft equivalent. Log a baseline for every channel here.',
    },
  },
  {
    id: 'boost', name: 'Release and rocket boost', start: releaseS, end: burnoutS,
    feel: 'A short drop, then a hard push into your seat for one minute.',
    sensors: {
      beginner: 'The push sensor jumps up. This part lasts only one minute.',
      intermediate: 'About 2.6 g in this model. At one reading a second you get only 60 readings.',
      professional: 'Model estimate of 2.6 g along the flight path; the provider publishes no g-loads. Sample acceleration at 10 Hz or faster.',
    },
  },
  {
    id: 'float', name: 'Microgravity', start: burnoutS, end: reentryS,
    feel: 'Everything floats for about three minutes.',
    sensors: {
      beginner: 'The push sensor reads almost zero. Things float!',
      intermediate: 'Acceleration near 0 g for about 193 seconds. Light changes as the ship turns.',
      professional: 'Ballistic coast: 193 s between engine cut-off and the same height on the way down, apogee 87.5 km. Expect rotation-rate and light changes during attitude manoeuvres.',
    },
  },
  {
    id: 'reentry', name: 'Re-entry', start: reentryS, end: glideS,
    feel: 'The ship meets thicker air and slows. Weight comes back strongly.',
    sensors: {
      beginner: 'The push sensor climbs again as the ship slows down.',
      intermediate: 'g-load rises, then eases. No figure is published, so the model leaves it blank.',
      professional: 'Feathered re-entry. No published loads; pick an accelerometer range with margin and ask the provider.',
    },
  },
  {
    id: 'glide', name: 'Glide and landing', start: glideS, end: landingS,
    feel: 'A quiet glide to the runway, like a normal aeroplane landing.',
    sensors: {
      beginner: 'Back to normal weight. A bump when the wheels touch.',
      intermediate: 'About 1 g. Cabin pressure may drift a little after re-entry.',
      professional: 'About 1 g; touchdown transient. Keep logging until the kit is switched off, and note that time by hand.',
    },
  },
];

export const clock = (s: number) => `T+${String(Math.floor(s / 60)).padStart(2, '0')}:${String(Math.round(s % 60)).padStart(2, '0')}`;
