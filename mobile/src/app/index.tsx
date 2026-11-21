import { router } from 'expo-router';
import { StyleSheet, Text, View } from 'react-native';
import { Button, Card, LevelPicker, Screen, shared } from '../components/Screen';
import quests from '../data/quests.json';
import { useCrew } from '../state';
import { appName, theme } from '../theme';

export default function Mission() {
  const { earned } = useCrew();
  const next = quests.find(q => !earned.includes(q.id)) ?? quests[0];
  return (
    <Screen title={appName} summary="Get your class ready for spaceflight. Practise with a ground station, predict the flight, then compare.">
      <Card tone="night">
        <Text style={styles.nightLabel}>Flight preparation</Text>
        <Text accessibilityRole="header" style={styles.nightHeading}>A classroom experiment on a research spaceflight</Text>
        <Text style={styles.nightBody}>What would a small sensor kit record on a suborbital research flight? See the ride, minute by minute.</Text>
        <Button label="Explore the flight" kind="light" onPress={() => router.navigate('/flight')} />
        <Text style={styles.nightSmall}>An independent volunteer project. The flight host decides what flies.</Text>
      </Card>

      <Card>
        <Text style={shared.label}>Pick your level</Text>
        <LevelPicker />
      </Card>

      <Card tone="sky">
        <Text style={shared.label}>{earned.length === 0 ? 'Start here' : `Next lesson · ${earned.length} of ${quests.length} done`}</Text>
        <Text accessibilityRole="header" style={shared.heading}>{next.title}</Text>
        <Text style={shared.body}>{next.summary}</Text>
        <Button label="Open the lesson" onPress={() => router.navigate({ pathname: '/quests/[id]', params: { id: next.id } })} />
      </Card>

      <Card>
        <Text accessibilityRole="header" style={shared.heading}>Practice ground station</Text>
        <Text style={shared.body}>Ask a practice CubeSat for readings and pictures, then pause contact and see what is lost. Everything is simulated on your phone.</Text>
        <Button label="Open the station" kind="quiet" onPress={() => router.navigate('/station')} />
      </Card>

      <View style={styles.patches} accessibilityLabel={`Mission patches: ${earned.length} of ${quests.length} earned`}>
        {quests.map(q => (
          <View key={q.id} style={[styles.patch, earned.includes(q.id) && styles.patchOn]}>
            <Text style={[styles.patchText, earned.includes(q.id) && { color: theme.white }]}>{q.patch}</Text>
          </View>
        ))}
      </View>
    </Screen>
  );
}

const styles = StyleSheet.create({
  nightLabel: { color: '#ffb547', fontSize: 13, fontWeight: '700' },
  nightHeading: { color: theme.white, fontSize: 23, lineHeight: 29, fontWeight: '800' },
  nightBody: { color: '#d7e3f4', fontSize: 16, lineHeight: 24 },
  nightSmall: { color: '#c9d8ee', fontSize: 13, lineHeight: 19 },
  patches: { flexDirection: 'row', flexWrap: 'wrap', gap: 8 },
  patch: { borderRadius: 999, borderWidth: 2, borderColor: theme.line, borderStyle: 'dashed', paddingVertical: 6, paddingHorizontal: 12 },
  patchOn: { backgroundColor: theme.blue, borderColor: theme.blue, borderStyle: 'solid' },
  patchText: { color: theme.muted, fontSize: 13, fontWeight: '700' },
});
