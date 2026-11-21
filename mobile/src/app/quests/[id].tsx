import { router, useLocalSearchParams } from 'expo-router';
import { StyleSheet, Text, View } from 'react-native';
import { Button, Card, LevelPicker, Screen, shared } from '../../components/Screen';
import quests from '../../data/quests.json';
import { useCrew } from '../../state';
import { theme } from '../../theme';

// Lesson steps mark button names with **stars**. Show those words in bold.
function Rich({ text }: { text: string }) {
  const parts = text.split('**');
  return (
    <Text style={[shared.body, { flex: 1 }]}>
      {parts.map((part, i) => (i % 2 ? <Text key={i} style={{ color: theme.ink, fontWeight: '700' }}>{part}</Text> : part.replace(/`/g, '')))}
    </Text>
  );
}

export default function Quest() {
  const { id } = useLocalSearchParams<{ id: string }>();
  const { level, earned, earn } = useCrew();
  const quest = quests.find(q => q.id === id);
  if (!quest) {
    return (
      <Screen title="Lesson not found">
        <Button label="See all lessons" onPress={() => router.navigate('/quests')} />
      </Screen>
    );
  }
  const part = quest[level];
  const done = earned.includes(quest.id);
  const next = quests[quests.findIndex(q => q.id === quest.id) + 1];

  return (
    <Screen title={quest.title} summary={`${quest.minutes} minutes · ${quest.skill}`}>
      <Button label="All lessons" kind="quiet" onPress={() => router.navigate('/quests')} />
      <LevelPicker />
      <Card>
        <Text accessibilityRole="header" style={shared.heading}>Steps</Text>
        {part.steps.map((step, i) => (
          <View key={i} style={styles.step}>
            <Text style={styles.count}>{i + 1}</Text>
            <Rich text={step} />
          </View>
        ))}
        <Button label="Open the ground station" kind="quiet" onPress={() => router.navigate('/station')} />
      </Card>
      <Card tone="sky">
        <Text accessibilityRole="header" style={shared.heading}>Check your work</Text>
        <Rich text={part.check} />
      </Card>
      <Card tone="cream">
        <Text accessibilityRole="header" style={shared.heading}>Talk it over</Text>
        <Rich text={part.discuss} />
      </Card>
      <Button label={done ? `Patch earned: ${quest.patch}` : 'Mark this lesson done'} disabled={done} onPress={() => earn(quest.id)} />
      {next ? <Button label={`Next: ${next.title}`} kind="quiet" onPress={() => router.navigate({ pathname: '/quests/[id]', params: { id: next.id } })} /> : null}
    </Screen>
  );
}

const styles = StyleSheet.create({
  step: { flexDirection: 'row', gap: 12, alignItems: 'flex-start' },
  count: { width: 26, color: theme.blue, fontSize: 16, lineHeight: 24, fontWeight: '800' },
});
