import { Tabs } from 'expo-router';
import { StatusBar } from 'expo-status-bar';
import { CrewProvider } from '../state';
import { theme } from '../theme';

export default function Layout() {
  return (
    <CrewProvider>
      <StatusBar style="dark" />
      <Tabs screenOptions={{
        headerShown: false,
        tabBarActiveTintColor: theme.blue,
        tabBarInactiveTintColor: theme.muted,
        tabBarActiveBackgroundColor: theme.sky,
        tabBarLabelStyle: { fontSize: 13, fontWeight: '700' },
        tabBarIconStyle: { display: 'none' },
        tabBarLabelPosition: 'beside-icon',
      }}>
        <Tabs.Screen name="index" options={{ title: 'Mission' }} />
        <Tabs.Screen name="station" options={{ title: 'Station' }} />
        <Tabs.Screen name="flight" options={{ title: 'Flight' }} />
        <Tabs.Screen name="quests" options={{ title: 'Learn' }} />
        <Tabs.Screen name="more" options={{ title: 'More' }} />
      </Tabs>
    </CrewProvider>
  );
}
