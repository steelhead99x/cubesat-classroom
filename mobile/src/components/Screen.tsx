import type { ReactNode } from 'react';
import { Pressable, ScrollView, StyleSheet, Text, View } from 'react-native';
import { SafeAreaView } from 'react-native-safe-area-context';
import { theme } from '../theme';
import { levels, useCrew } from '../state';

export function Screen({ title, summary, children }: { title: string; summary?: string; children: ReactNode }) {
  return (
    <SafeAreaView style={styles.safe} edges={['top', 'left', 'right']}>
      <ScrollView contentContainerStyle={styles.scroll}>
        <View style={styles.body}>
          <View style={styles.header}>
            <Text accessibilityRole="header" style={styles.title}>{title}</Text>
            {summary ? <Text style={styles.summary}>{summary}</Text> : null}
          </View>
          {children}
        </View>
      </ScrollView>
    </SafeAreaView>
  );
}

export function Card({ children, tone }: { children: ReactNode; tone?: 'sky' | 'cream' | 'blush' | 'night' }) {
  const background = tone === 'sky' ? theme.sky : tone === 'cream' ? theme.cream : tone === 'blush' ? theme.blush : tone === 'night' ? theme.navy : theme.white;
  return <View style={[shared.card, { backgroundColor: background }, tone === 'night' && { borderColor: theme.navy }]}>{children}</View>;
}

export function Button({ label, onPress, kind = 'primary', disabled }: { label: string; onPress: () => void; kind?: 'primary' | 'quiet' | 'light'; disabled?: boolean }) {
  return (
    <Pressable
      accessibilityRole="button"
      accessibilityState={{ disabled: !!disabled }}
      disabled={disabled}
      onPress={onPress}
      style={({ pressed }) => [shared.button, kind === 'quiet' && styles.quiet, kind === 'light' && styles.light, (pressed || disabled) && { opacity: disabled ? 0.45 : 0.8 }]}
    >
      <Text style={[shared.buttonText, kind === 'quiet' && { color: theme.ink }, kind === 'light' && { color: theme.white }]}>{label}</Text>
    </Pressable>
  );
}

export function LevelPicker() {
  const { level, setLevel } = useCrew();
  const current = levels.find(l => l.id === level)!;
  return (
    <View style={{ gap: 6 }}>
      <View accessibilityRole="radiogroup" accessibilityLabel="Pick your level" style={styles.levels}>
        {levels.map(l => (
          <Pressable
            key={l.id}
            accessibilityRole="radio"
            accessibilityState={{ checked: l.id === level }}
            onPress={() => setLevel(l.id)}
            style={[styles.level, l.id === level && styles.levelOn]}
          >
            <Text style={[styles.levelText, l.id === level && { color: theme.white }]}>{l.name}</Text>
          </Pressable>
        ))}
      </View>
      <Text style={shared.small}>{current.note}</Text>
    </View>
  );
}

export const shared = StyleSheet.create({
  card: { borderRadius: 16, padding: 18, gap: 10, borderWidth: 1, borderColor: theme.line },
  heading: { color: theme.ink, fontSize: 20, fontWeight: '700' },
  body: { color: theme.muted, fontSize: 16, lineHeight: 24 },
  small: { color: theme.muted, fontSize: 14, lineHeight: 20 },
  label: { color: theme.blue, fontSize: 13, fontWeight: '700' },
  button: { minHeight: 48, justifyContent: 'center', alignItems: 'center', backgroundColor: theme.blue, borderRadius: 10, paddingVertical: 12, paddingHorizontal: 16 },
  buttonText: { color: theme.white, fontSize: 16, fontWeight: '700', textAlign: 'center' },
  link: { color: theme.blue, fontSize: 16, lineHeight: 24, fontWeight: '700', textDecorationLine: 'underline', paddingVertical: 4 },
  row: { flexDirection: 'row', flexWrap: 'wrap', gap: 10 },
});

const styles = StyleSheet.create({
  safe: { flex: 1, backgroundColor: theme.paper },
  scroll: { paddingHorizontal: 16, paddingVertical: 16 },
  body: { width: '100%', maxWidth: 640, alignSelf: 'center', gap: 14 },
  header: { gap: 6, paddingTop: 6, paddingBottom: 2 },
  title: { color: theme.ink, fontWeight: '800', fontSize: 30, letterSpacing: -0.5 },
  summary: { color: theme.muted, fontSize: 17, lineHeight: 25 },
  quiet: { backgroundColor: theme.white, borderWidth: 2, borderColor: theme.ink },
  light: { backgroundColor: 'transparent', borderWidth: 2, borderColor: '#ffffff88' },
  levels: { flexDirection: 'row', backgroundColor: theme.white, borderRadius: 12, borderWidth: 1, borderColor: theme.line, padding: 4, gap: 4 },
  level: { flex: 1, minHeight: 44, alignItems: 'center', justifyContent: 'center', borderRadius: 8, paddingHorizontal: 4 },
  levelOn: { backgroundColor: theme.blue },
  levelText: { color: theme.ink, fontSize: 14, fontWeight: '700' },
});
