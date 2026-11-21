import { Link } from 'expo-router';
import { Text, View } from 'react-native';
import { Screen, shared } from '../components/Screen';
import { repository } from '../theme';

const lessons = [
  { title: 'Run the desktop ground station', details: 'Use the guided terminal command, then request a reading and play practice video.', path: '/blob/main/docs/quick-start.md' },
  { title: 'Try the balloon lab', details: 'Compare classroom pressure and temperature with archived balloon flight data.', path: '/blob/main/docs/balloon-lab.md' },
  { title: 'Print a CubeSat model', details: 'Print a 1U bench model and check how the pieces fit.', path: '/tree/main/hardware' },
  { title: 'Choose a hardware quest', details: 'Choose Arduino, XIAO, Raspberry Pi, or computer only to find your first quest.', path: '/blob/main/quest/README.md#start-with-what-you-have' },
];

export default function Lessons() {
  return (
    <Screen title="Lessons" summary="Open a classroom activity with a teacher, mentor, or crew.">
      {lessons.map((lesson) => (
        <View key={lesson.path} style={shared.card}>
          <Text accessibilityRole="header" style={shared.heading}>{lesson.title}</Text>
          <Text style={shared.body}>{lesson.details}</Text>
          <Link href={`${repository}${lesson.path}`} style={shared.link}>Open {lesson.title.toLowerCase()}</Link>
        </View>
      ))}
      <View style={shared.card}>
        <Text accessibilityRole="header" style={shared.heading}>Developer relay tests</Text>
        <Text style={shared.body}>Adult mentors can test generated video in a private project with their own MoQ.pro account.</Text>
        <Link href={`${repository}/blob/main/docs/moq-pro-development.md`} style={shared.link}>Run a private relay demo</Link>
        <Link href="https://moq.pro/" style={shared.link}>Create or sign in to a MoQ.pro account</Link>
      </View>
    </Screen>
  );
}
