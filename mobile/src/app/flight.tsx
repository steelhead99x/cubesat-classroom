import { Link } from 'expo-router';
import { useState } from 'react';
import { Pressable, StyleSheet, Text, View } from 'react-native';
import { Button, Card, LevelPicker, Screen, shared } from '../components/Screen';
import { altitudeAt, clock, flight, phases } from '../data/flight';
import { useCrew } from '../state';
import { doc, theme } from '../theme';

const BARS = 42;
const TOP = 100_000;
const bars = Array.from({ length: BARS }, (_, i) => {
  // Half the bars cover the slow climb; the rest cover the last 20 minutes.
  const t = i < 14 ? (i / 14) * flight.releaseS : flight.releaseS + ((i - 14) / (BARS - 15)) * (flight.landingS - flight.releaseS);
  return { t, height: altitudeAt(t) };
});

export default function Flight() {
  const { level } = useCrew();
  const [index, setIndex] = useState(0);
  const phase = phases[index];
  const top = Math.max(...bars.filter(b => b.t >= phase.start && b.t <= phase.end).map(b => b.height), altitudeAt(phase.start), altitudeAt(phase.end));

  return (
    <Screen title="The research flight" summary="About 70 minutes from take-off to landing. The part in space lasts a few minutes.">
      <Card>
        <View style={styles.chart} accessibilityRole="image" accessibilityLabel={`Height during the flight. A slow climb to ${(flight.releaseM / 1000).toFixed(1)} kilometres, a steep rise to ${(flight.apogeeM / 1000).toFixed(1)} kilometres, then a glide to landing.`}>
          {bars.map((b, i) => {
            const on = b.t >= phase.start && b.t < phase.end;
            return <View key={i} style={[styles.bar, { height: `${Math.max(1.5, (b.height / TOP) * 100)}%`, backgroundColor: on ? theme.amber : '#b9c7dc' }]} />;
          })}
        </View>
        <View style={styles.axis}>
          <Text style={shared.small}>Take-off</Text>
          <Text style={shared.small}>Highest: {(flight.apogeeM / 1000).toFixed(1)} km</Text>
          <Text style={shared.small}>Landing</Text>
        </View>
        <Text style={shared.small}>Desk simulation from public figures. Not flight data. The last 20 minutes are stretched so you can see them.</Text>
      </Card>

      <View accessibilityRole="tablist" style={styles.steps}>
        {phases.map((p, i) => (
          <Pressable key={p.id} accessibilityRole="tab" accessibilityState={{ selected: i === index }} accessibilityLabel={p.name} onPress={() => setIndex(i)} style={[styles.step, i === index && styles.stepOn]}>
            <Text style={[styles.stepText, i === index && { color: theme.white }]}>{i + 1}</Text>
          </Pressable>
        ))}
      </View>

      <Card tone="cream">
        <Text style={[shared.label, { color: theme.amber }]}>{clock(phase.start)} to {clock(phase.end)}</Text>
        <Text accessibilityRole="header" style={shared.heading}>{phase.name}</Text>
        <Text style={shared.body}>{phase.feel}</Text>
        <Text style={shared.small}>Height in this part: up to {(top / 1000).toFixed(1)} km</Text>
      </Card>

      <Card>
        <Text accessibilityRole="header" style={shared.heading}>What the sensor kit should record</Text>
        <Text style={shared.body}>{phase.sensors[level]}</Text>
        <LevelPicker />
      </Card>

      <View style={shared.row}>
        <View style={{ flex: 1 }}><Button label="Back" kind="quiet" disabled={index === 0} onPress={() => setIndex(index - 1)} /></View>
        <View style={{ flex: 1 }}><Button label="Next part" disabled={index === phases.length - 1} onPress={() => setIndex(index + 1)} /></View>
      </View>

      <Card>
        <Text accessibilityRole="header" style={shared.heading}>Good to know</Text>
        <Text style={shared.body}>The cabin stays pressurised at about {flight.cabinKpa} kPa, like a mountain town. A pressure sensor inside will not show the near-vacuum outside.</Text>
        <Link href={doc('docs/suborbital-flight-plan.md')} style={shared.link}>Read the flight plan</Link>
        <Link href={doc('quest/m0/predict-the-spaceflight.md')} style={shared.link}>Make your own prediction</Link>
        <Link href={doc('quest/m0/one-sensor-three-rides.md')} style={shared.link}>Compare a glider, a spaceship and the ISS</Link>
      </Card>
    </Screen>
  );
}

const styles = StyleSheet.create({
  chart: { height: 170, flexDirection: 'row', alignItems: 'flex-end', gap: 2, borderBottomWidth: 2, borderBottomColor: theme.ink },
  bar: { flex: 1, borderTopLeftRadius: 2, borderTopRightRadius: 2 },
  axis: { flexDirection: 'row', justifyContent: 'space-between', gap: 6 },
  steps: { flexDirection: 'row', gap: 8 },
  step: { flex: 1, minHeight: 46, borderRadius: 10, alignItems: 'center', justifyContent: 'center', backgroundColor: theme.white, borderWidth: 1, borderColor: theme.line },
  stepOn: { backgroundColor: theme.amber, borderColor: theme.amber },
  stepText: { color: theme.ink, fontSize: 17, fontWeight: '800' },
});
