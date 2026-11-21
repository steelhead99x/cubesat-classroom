import { Tabs } from 'expo-router';
import { StatusBar } from 'expo-status-bar';
import { theme } from '../theme';

export default function Layout() {
  return (
    <>
      <StatusBar style="dark" />
      <Tabs screenOptions={{
        headerShown: false,
        tabBarActiveTintColor: theme.blue,
        tabBarInactiveTintColor: theme.muted,
        tabBarActiveBackgroundColor: theme.sky,
        tabBarLabelStyle: { fontSize: 13, fontWeight: '700' },
      }}>
        <Tabs.Screen name="index" options={{ title: 'Mission' }} />
        <Tabs.Screen name="telemetry" options={{ title: 'Telemetry' }} />
        <Tabs.Screen name="lessons" options={{ title: 'Lessons' }} />
      </Tabs>
    </>
  );
}
