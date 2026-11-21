// Settings for this site. A fork changes these three lines and the `hubs` list below.
// See docs/fork-guide.md.
export const repo = 'https://github.com/steelhead99x/cubesat-classroom';
export const fundraiser = 'https://givebutter.com/fund-stem-training-for-100-educators-s14sb0';
export const classroomHost = 'cubesatclass.org';

export const hubs = [
  {
    slug: 'mobile',
    host: 'cubesat.app',
    name: 'Mobile app',
    heading: 'Practice a mission on your phone',
    description: 'A practice ground station, the research flight and five lessons on your phone. The open-source Android and iOS app is ready for developer previews.',
    links: [
      ['Run the mobile template', repo + '/blob/main/mobile/README.md'],
      ['Browse the mobile source', repo + '/tree/main/mobile'],
      ['Open the classroom quick start', 'https://' + classroomHost + '/#start'],
    ],
  },
  {
    slug: 'lessons',
    host: 'cubesatclass.org',
    name: 'Lessons',
    heading: 'Space science in the classroom',
    description: 'Try a mission with your class. Record what happened so another crew can repeat it.',
    links: [
      ['Print the quick-start handout', repo + '/blob/main/output/pdf/cubesat-classroom-quick-start.pdf'],
      ['Prepare for the research spaceflight', 'https://' + classroomHost + '/mission/'],
      ['Try the balloon lab', repo + '/blob/main/docs/balloon-lab.md'],
      ['Choose a quest for your hardware', repo + '/blob/main/quest/README.md#start-with-what-you-have'],
      ['Teach the ground station lesson', repo + '/blob/main/quest/m0/repeat-the-station-lesson.md'],
      ['Print a CubeSat model', repo + '/tree/main/hardware'],
    ],
  },
  {
    slug: 'station',
    host: 'cubesatclass.io',
    name: 'Ground station',
    heading: 'Run the practice ground station',
    description: 'Send a command, read a simulated sensor, and watch practice video in the local Rust ground station.',
    links: [
      ['Try the station in your browser', 'https://' + classroomHost + '/#demo'],
      ['Install the station', 'https://' + classroomHost + '/#start'],
      ['Build a real ground station, step by step', repo + '/blob/main/docs/ground-station.md'],
      ['Read the relay and radio design', repo + '/blob/main/docs/mission.md'],
      ['Try the SubGHz video rehearsal', repo + '/blob/main/docs/demo.md'],
    ],
  },
  {
    slug: 'develop',
    host: 'cubesatclass.dev',
    name: 'Develop',
    heading: 'Build and test with Media over QUIC',
    description: 'Use MoQ.pro for optional development and testing with your own account. Use short-lived credentials and generated video in a private project. We welcome future contributions for other MoQ providers.',
    links: [
      ['Create or sign in to a MoQ.pro account', 'https://moq.pro/'],
      ['Run a private relay demo', repo + '/blob/main/docs/moq-pro-development.md'],
      ['Build the iOS or Android template', repo + '/tree/main/mobile'],
      ['Browse the source', repo],
    ],
  },
  {
    slug: 'volunteer',
    host: 'cubesat.club',
    name: 'Volunteer',
    heading: 'Help the next crew',
    description: 'CubeSat Classroom is a volunteer-built contribution to space education. Teachers, learners, and amateur-radio mentors keep it going by sharing what they tried and what they learned.',
    links: [
      ['Pick a contribution quest', repo + '/tree/main/quest'],
      ['Read the volunteer guide', repo + '/blob/main/CONTRIBUTING.md'],
      ['Donate directly to Teachers in Space', fundraiser],
    ],
  },
];
