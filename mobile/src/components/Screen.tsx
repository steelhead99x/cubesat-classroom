import type { ReactNode } from 'react';
import { ScrollView, StyleSheet, Text, View } from 'react-native';
import { SafeAreaView } from 'react-native-safe-area-context';
import { theme } from '../theme';

export function Screen({ title, summary, children }: { title: string; summary: string; children: ReactNode }) {
  return (
    <SafeAreaView style={styles.safe} edges={['top', 'left', 'right']}>
      <ScrollView contentContainerStyle={styles.scroll}>
        <View style={styles.body}>
          <View style={styles.header}>
            <Text accessibilityRole="header" style={styles.title}>{title}</Text>
            <Text style={styles.summary}>{summary}</Text>
          </View>
          {children}
        </View>
      </ScrollView>
    </SafeAreaView>
  );
}

export const shared = StyleSheet.create({
  card: {
    borderRadius: 20,
    padding: 18,
    gap: 10,
    backgroundColor: theme.white,
    borderWidth: 1,
    borderColor: theme.line,
  },
  heading: { color: theme.navy, fontSize: 21, fontWeight: '700' },
  body: { color: theme.muted, fontSize: 16, lineHeight: 24 },
  button: { minHeight: 48, justifyContent: 'center', alignItems: 'center', backgroundColor: theme.blue, borderRadius: 12, padding: 12 },
  buttonText: { color: theme.white, fontSize: 16, fontWeight: '700', textAlign: 'center' },
  link: { color: theme.blue, fontSize: 16, lineHeight: 24, fontWeight: '700', textDecorationLine: 'underline' },
});

const styles = StyleSheet.create({
  safe: { flex: 1, backgroundColor: theme.paper },
  scroll: { paddingHorizontal: 18, paddingVertical: 18 },
  body: { width: '100%', maxWidth: 620, alignSelf: 'center', gap: 16 },
  header: { backgroundColor: theme.navy, borderRadius: 22, padding: 22, gap: 8 },
  title: { color: theme.white, fontWeight: '800', fontSize: 34, letterSpacing: -1 },
  summary: { color: theme.sky, fontSize: 16, lineHeight: 24 },
});
