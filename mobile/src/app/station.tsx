import { useEffect, useRef, useState } from 'react';
import { StyleSheet, Text, View } from 'react-native';
import { Button, Card, LevelPicker, Screen, shared } from '../components/Screen';
import { useCrew } from '../state';
import { theme } from '../theme';

const FRAMES = 8;

// Same practice formulas as the desktop station's simulator (Sample::synthetic).
function sample(sequence: number) {
  return {
    sequence,
    temperature: 18 + (sequence % 11) / 2,
    voltage: (3900 + (sequence % 61)) / 1000,
    light: sequence % 30 < 20 ? 460 : 0,
  };
}

const words = {
  beginner: { read: 'Ask for a reading', pics: 'Take pictures', volt: 'Battery' },
  intermediate: { read: 'Get sensor reading', pics: 'Start test camera', volt: 'Bus voltage' },
  professional: { read: 'Request sensors', pics: 'Request video burst', volt: 'Bus voltage' },
};

export default function Station() {
  const { level } = useCrew();
  const w = words[level];
  const [contact, setContact] = useState(true);
  const [reading, setReading] = useState<ReturnType<typeof sample> | null>(null);
  const [frames, setFrames] = useState(0);
  const [sending, setSending] = useState(false);
  // Each line keeps its own id, so the list does not redraw every row when a line is added.
  const lineId = useRef(0);
  const [log, setLog] = useState<{ id: number; text: string }[]>([{ id: 0, text: 'Station ready.' }]);
  const sequence = useRef(0);
  const timer = useRef<ReturnType<typeof setInterval> | null>(null);

  const note = (text: string) => setLog(lines => [{ id: ++lineId.current, text }, ...lines].slice(0, 12));
  const stop = () => {
    if (timer.current) clearInterval(timer.current);
    timer.current = null;
    setSending(false);
  };
  useEffect(() => stop, []);

  const ask = () => {
    if (!contact) return note('No contact. The request was not sent.');
    sequence.current += 1;
    const s = sample(sequence.current);
    setReading(s);
    note(level === 'professional' ? `Reply ${s.sequence}: sample received, source "simulation".` : `Reading ${s.sequence} arrived.`);
  };

  const pictures = () => {
    if (!contact) return note('No contact. The camera request was not sent.');
    stop();
    setFrames(0);
    setSending(true);
    note('Camera started. Pictures are coming down.');
    let got = 0;
    timer.current = setInterval(() => {
      got += 1;
      setFrames(got);
      if (got === FRAMES) {
        stop();
        note(`All ${FRAMES} pictures arrived.`);
      }
    }, 500);
  };

  const toggle = () => {
    if (contact) {
      const was = sending;
      stop();
      setContact(false);
      note(was ? `Contact lost. Only ${frames} of ${FRAMES} pictures arrived.` : 'Contact paused. The satellite is out of sight.');
    } else {
      setContact(true);
      note('Contact is back.');
    }
  };

  return (
    <Screen title="Ground station" summary="Practice mode. All data is simulated on your phone.">
      <LevelPicker />
      <Card tone="night">
        <View style={styles.status} accessibilityLiveRegion="polite">
          <View style={[styles.dot, { backgroundColor: contact ? theme.mint : '#ef6b6c' }]} />
          <Text style={styles.statusText}>{contact ? 'Connected to the practice CubeSat' : 'No contact'}</Text>
        </View>
        <Button label={w.read} onPress={ask} />
        <Button label={w.pics} kind="light" onPress={pictures} disabled={sending} />
        <Button label={contact ? 'Pause contact' : 'Resume contact'} kind="light" onPress={toggle} />
      </Card>

      <Card>
        <Text accessibilityRole="header" style={shared.heading}>{reading ? `Reading ${reading.sequence}` : 'No reading yet'}</Text>
        <Row name="Temperature" value={reading ? `${reading.temperature.toFixed(1)} °C` : '--'} />
        <Row name={w.volt} value={reading ? `${reading.voltage.toFixed(3)} V` : '--'} />
        <Row name="Light" value={reading ? `${reading.light} lx` : '--'} />
        {level === 'professional' && reading ? (
          <Text style={styles.raw}>{JSON.stringify({ sequence: reading.sequence, source: 'simulation', temperature_c: reading.temperature, bus_voltage_v: reading.voltage, light_lux: reading.light }, null, 1)}</Text>
        ) : null}
      </Card>

      <Card>
        <Text accessibilityRole="header" style={shared.heading}>Test camera</Text>
        <View style={styles.frames} accessibilityLabel={`${frames} of ${FRAMES} pictures received`}>
          {Array.from({ length: FRAMES }, (_, i) => <View key={i} style={[styles.frame, i < frames && styles.frameOn]} />)}
        </View>
        <Text style={shared.body}>{frames} of {FRAMES} pictures</Text>
        <Text style={shared.small}>Tip: pause contact while pictures are coming in, then see what made it.</Text>
      </Card>

      <Card>
        <Text accessibilityRole="header" style={shared.heading}>Mission log</Text>
        <View accessibilityLiveRegion="polite" style={{ gap: 4 }}>
          {log.map((line, i) => <Text key={line.id} style={[shared.small, i === 0 && { color: theme.ink, fontWeight: '700' }]}>{line.text}</Text>)}
        </View>
      </Card>
    </Screen>
  );
}

function Row({ name, value }: { name: string; value: string }) {
  return (
    <View style={styles.row}>
      <Text style={shared.body}>{name}</Text>
      <Text style={styles.value}>{value}</Text>
    </View>
  );
}

const styles = StyleSheet.create({
  status: { flexDirection: 'row', alignItems: 'center', gap: 8 },
  dot: { width: 12, height: 12, borderRadius: 6 },
  statusText: { color: theme.white, fontSize: 16, fontWeight: '700' },
  row: { flexDirection: 'row', flexWrap: 'wrap', justifyContent: 'space-between', gap: 6, paddingVertical: 8, borderBottomWidth: 1, borderBottomColor: theme.line },
  value: { color: theme.ink, fontSize: 19, fontWeight: '700', fontVariant: ['tabular-nums'] },
  raw: { color: theme.ink, backgroundColor: theme.paper, borderRadius: 8, padding: 10, fontSize: 13, fontFamily: 'monospace' },
  frames: { flexDirection: 'row', gap: 6 },
  frame: { flex: 1, height: 34, borderRadius: 6, backgroundColor: theme.paper, borderWidth: 1, borderColor: theme.line },
  frameOn: { backgroundColor: theme.blue, borderColor: theme.blue },
});
