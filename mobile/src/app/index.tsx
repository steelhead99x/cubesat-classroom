import { Link } from 'expo-router';
import { Image, Pressable, StyleSheet, Text, View } from 'react-native';
import { Screen, shared } from '../components/Screen';
import { theme, fundraiser, repository } from '../theme';

export default function Mission() {
  return (
    <Screen title="CubeSat Classroom" summary="Build a CubeSat. Run a practice mission. Teach the next crew.">
      <View style={[shared.card, styles.mission]}>
        <Image source={require('../../assets/android-icon-foreground.png')} style={styles.mark} accessibilityIgnoresInvertColors />
        <Text accessibilityRole="header" style={shared.heading}>Your first mission</Text>
        <Text style={shared.body}>Ask the simulator for a sensor reading. Write down what you measured and include its unit.</Text>
        <Link href="/telemetry" asChild>
          <Pressable accessibilityRole="link" style={shared.button}>
            <Text style={shared.buttonText}>Open telemetry</Text>
          </Pressable>
        </Link>
      </View>
      <View style={[shared.card, styles.support]}>
        <Text accessibilityRole="header" style={shared.heading}>Built and supported by volunteers</Text>
        <Text style={shared.body}>This open-source app is a volunteer contribution to space education. Help another classroom by sharing a lesson, bench result, or code fix.</Text>
        <Link href={`${repository}/blob/main/CONTRIBUTING.md`} style={shared.link}>Contribute to CubeSat Classroom</Link>
        <Link href={fundraiser} style={[shared.link, styles.donate]}>Give directly to Teachers in Space</Link>
      </View>
    </Screen>
  );
}

const styles = StyleSheet.create({
  mission: { backgroundColor: theme.sky },
  support: { backgroundColor: theme.blush },
  mark: { width: 54, height: 54, backgroundColor: theme.navy, borderRadius: 14 },
  donate: { color: theme.red },
});
