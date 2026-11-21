import { createContext, useContext, useMemo, useState, type ReactNode } from 'react';

export const levels = [
  { id: 'beginner', name: 'Beginner', note: '13 and up and new to satellites, or anyone new' },
  { id: 'intermediate', name: 'Intermediate', note: 'Middle and high school: units, timing, simple sums' },
  { id: 'professional', name: 'Professional', note: 'Advanced students, mentors and engineers' },
] as const;
export type Level = (typeof levels)[number]['id'];

type Crew = {
  level: Level;
  setLevel: (level: Level) => void;
  earned: string[];
  earn: (id: string) => void;
};

const CrewContext = createContext<Crew | null>(null);

// Level and earned patches are kept only while the app is open. Nothing is
// stored on the phone or sent anywhere, and the app never asks who is using it.
export function CrewProvider({ children }: { children: ReactNode }) {
  const [level, setLevel] = useState<Level>('beginner');
  const [earned, setEarned] = useState<string[]>([]);
  const value = useMemo(
    () => ({ level, setLevel, earned, earn: (id: string) => setEarned(list => (list.includes(id) ? list : [...list, id])) }),
    [level, earned],
  );
  return <CrewContext.Provider value={value}>{children}</CrewContext.Provider>;
}

export function useCrew() {
  const crew = useContext(CrewContext);
  if (!crew) throw new Error('useCrew needs CrewProvider');
  return crew;
}
