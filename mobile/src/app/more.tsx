import { Link } from 'expo-router';
import { Text } from 'react-native';
import { Card, Screen, shared } from '../components/Screen';
import { doc, fundraiser, repository, theme } from '../theme';

const groups = [
  {
    title: 'Labs and guides',
    links: [
      ['Build a ground station, step by step', doc('docs/ground-station.md')],
      ['Balloon lab', doc('docs/balloon-lab.md')],
      ['Research flight plan', doc('docs/suborbital-flight-plan.md')],
      ['Print a CubeSat model', `${repository}/tree/main/hardware`],
    ],
  },
  {
    title: 'More quests',
    links: [
      ['Choose a quest for your hardware', doc('quest/README.md')],
      ['Fly with Perlan 2', doc('quest/m0/perlan-stratosphere.md')],
      ['Find the ISS', doc('quest/m0/iss-orbit.md')],
      ['Wildfires: get a message out by satellite', doc('quest/m0/wildfire-message-relay.md')],
      ['Track a pet by its collar', doc('quest/m0/pet-collar-tracking.md')],
      ['Wool, fibre and static', doc('quest/m0/wool-fiber-static.md')],
    ],
  },
  {
    title: 'Make it your own',
    links: [
      ['Fork guide: start your own club app and site', doc('docs/fork-guide.md')],
      ['Contribute a lesson or a fix', doc('CONTRIBUTING.md')],
    ],
  },
] as const;

export default function More() {
  return (
    <Screen title="More" summary="Guides, quests and ways to help. Links open the project on GitHub.">
      {groups.map(group => (
        <Card key={group.title}>
          <Text accessibilityRole="header" style={shared.heading}>{group.title}</Text>
          {group.links.map(([label, href]) => <Link key={href} href={href} style={shared.link}>{label}</Link>)}
        </Card>
      ))}
      <Card tone="blush">
        <Text accessibilityRole="header" style={shared.heading}>Built by volunteers</Text>
        <Text style={shared.body}>This open-source app is a starting kit, not a competitor to other space education projects. It does not collect accounts or personal details.</Text>
        <Link href={fundraiser} style={[shared.link, { color: theme.red }]}>Give directly to Teachers in Space</Link>
      </Card>
    </Screen>
  );
}
