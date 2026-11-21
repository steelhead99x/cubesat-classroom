import { router } from 'expo-router';
import { Pressable, StyleSheet, Text, View } from 'react-native';
import { LevelPicker, Screen, shared } from '../../components/Screen';
import quests from '../../data/quests.json';
import { useCrew } from '../../state';
import { theme } from '../../theme';

export default function Lessons() {
  const { earned } = useCrew();
  const total = quests.reduce((sum, q) => sum + q.minutes, 0);
  return (
    <Screen title="Lessons" summary={`Five exercises with the practice ground station. Allow about ${total} minutes for all of them.`}>
      <LevelPicker />
      {quests.map((q, i) => {
        const done = earned.includes(q.id);
        return (
          <Pressable
            key={q.id}
            accessibilityRole="button"
            accessibilityLabel={`Lesson ${i + 1}: ${q.title}. ${q.minutes} minutes.${done ? ' Done.' : ''}`}
            onPress={() => router.navigate({ pathname: '/quests/[id]', params: { id: q.id } })}
            style={({ pressed }) => [shared.card, styles.item, pressed && { opacity: 0.85 }]}
          >
            <View style={[styles.number, done && styles.numberDone]}>
              <Text style={[styles.numberText, done && { color: theme.white }]}>{done ? '✓' : i + 1}</Text>
            </View>
            <View style={{ flex: 1, gap: 4 }}>
              <Text style={shared.heading}>{q.title}</Text>
              <Text style={shared.body}>{q.summary}</Text>
              <Text style={shared.small}>{q.minutes} min · {q.skill}</Text>
            </View>
          </Pressable>
        );
      })}
    </Screen>
  );
}

const styles = StyleSheet.create({
  item: { flexDirection: 'row', backgroundColor: theme.white, gap: 14 },
  number: { width: 36, height: 36, borderRadius: 18, alignItems: 'center', justifyContent: 'center', backgroundColor: theme.sky },
  numberDone: { backgroundColor: theme.blue },
  numberText: { color: theme.blue, fontSize: 16, fontWeight: '800' },
});
