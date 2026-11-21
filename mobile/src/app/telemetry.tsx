import { useState } from 'react';
import { Pressable, StyleSheet, Text, View } from 'react-native';
import { Screen, shared } from '../components/Screen';
import { theme } from '../theme';

const samples = [
  { temperature: 21.4, pressure: 1010.8, battery: 4.12 },
  { temperature: 21.1, pressure: 1010.6, battery: 4.11 },
  { temperature: 20.8, pressure: 1010.3, battery: 4.10 },
  { temperature: 20.5, pressure: 1010.1, battery: 4.09 },
];

export default function Telemetry() {
  const [index, setIndex] = useState(-1);
  const sample = samples[index];
  const isLast = index === samples.length - 1;

  return (
    <Screen title="Telemetry" summary="Telemetry is information a spacecraft sends back to the ground.">
      <View style={[shared.card, styles.practice]}>
        <Text accessibilityRole="header" style={shared.heading}>Offline practice</Text>
        <Text style={shared.body}>Read a set of four simulated sensor samples. Each request advances the exercise by one sample.</Text>
        <Pressable style={shared.button} accessibilityRole="button" onPress={() => setIndex(isLast ? -1 : index + 1)}>
          <Text style={shared.buttonText}>{isLast ? 'Reset practice' : 'Request a reading'}</Text>
        </Pressable>
      </View>
      <View style={shared.card}>
        <Text accessibilityRole="header" accessibilityLiveRegion="polite" style={shared.heading}>
          {sample ? `Sample ${index + 1} of ${samples.length}` : 'Waiting for a request'}
        </Text>
        {sample && (
          <>
            <Reading name="Temperature" value={`${sample.temperature.toFixed(1)} °C`} />
            <Reading name="Pressure" value={`${sample.pressure.toFixed(1)} hPa`} />
            <Reading name="Battery" value={`${sample.battery.toFixed(2)} V`} />
            <Text style={shared.body}>Compare this reading with the next one. Which value changed?</Text>
          </>
        )}
      </View>
    </Screen>
  );
}

function Reading({ name, value }: { name: string; value: string }) {
  return (
    <View style={styles.reading}>
      <Text style={shared.body}>{name}</Text>
      <Text style={styles.value}>{value}</Text>
    </View>
  );
}

const styles = StyleSheet.create({
  practice: { backgroundColor: theme.yellow },
  reading: { flexDirection: 'row', flexWrap: 'wrap', justifyContent: 'space-between', gap: 6, paddingVertical: 6, borderBottomWidth: 1, borderBottomColor: theme.line },
  value: { color: theme.navy, fontSize: 19, fontWeight: '700', fontVariant: ['tabular-nums'] },
});
