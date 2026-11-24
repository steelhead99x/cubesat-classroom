import { useEffect, useRef, useState } from 'react';
import { StyleSheet, Text, View } from 'react-native';
import { Button, Card, LevelPicker, Screen, shared } from '../components/Screen';
import { useCrew } from '../state';
import { theme } from '../theme';

const FRAMES = 8;
const COLORS = ['#e14346', '#ffc443', '#3ca479', '#3271ca', '#303554', '#f3e8d0'];

// Same practice formulas as the browser demo. The sequence is the clock in
// seconds, so the light level drops to zero for part of each minute.
function sample(sequence: number, captured: number) {
  return {
    sequence,
    captured,
    source: 'simulation',
    temperature: 18 + (sequence % 11) / 2,
    voltage: (3900 + (sequence % 61)) / 1000,
    light: sequence % 30 < 20 ? 460 : 0,
  };
}

type Reading = ReturnType<typeof sample>;

const words = {
  beginner: { read: 'Ask for a reading', pics: 'Take pictures', volt: 'Battery' },
  intermediate: { read: 'Get sensor reading', pics: 'Start video', volt: 'Bus voltage' },
  professional: { read: 'Get sensor reading', pics: 'Start video', volt: 'Bus voltage' },
};

export default function Station() {
  const { level } = useCrew();
  const w = words[level];
  const [contact, setContact] = useState(true);
  const [reading, setReading] = useState<Reading | null>(null);
  const [frames, setFrames] = useState(0);
  const [incomplete, setIncomplete] = useState(false);
  const [sending, setSending] = useState(false);
  const [now, setNow] = useState(() => Date.now());
  const lineId = useRef(0);
  const [log, setLog] = useState<{ id: number; text: string }[]>([{ id: 0, text: 'Station ready.' }]);
  const contactRef = useRef(true);
  const framesRef = useRef(0);
  const timer = useRef<ReturnType<typeof setInterval> | null>(null);

  const note = (text: string) => setLog(lines => [{ id: ++lineId.current, text }, ...lines].slice(0, 12));
  const stopTimer = () => {
    if (timer.current) clearInterval(timer.current);
    timer.current = null;
  };
  useEffect(() => () => stopTimer(), []);
  useEffect(() => {
    const tick = setInterval(() => setNow(Date.now()), 500);
    return () => clearInterval(tick);
  }, []);

  const ask = async () => {
    if (!contactRef.current || sending) return;
    setSending(true);
    const started = Date.now();
    const delay = 280 + Math.floor(Math.random() * 280);
    await new Promise(resolve => setTimeout(resolve, delay));
    if (!contactRef.current) {
      note('No answer. The satellite could not hear us. Try again when contact is on.');
      setSending(false);
      return;
    }
    const captured = Date.now();
    const s = sample(Math.floor(captured / 1000), captured);
    setReading(s);
    const rtt = Date.now() - started;
    note(level === 'beginner'
      ? `The satellite answered: ${s.temperature.toFixed(1)} °C, ${s.voltage.toFixed(3)} V, ${s.light} lx.`
      : `Reply in ${rtt} ms, sample #${s.sequence}: ${s.temperature.toFixed(1)} °C, ${s.voltage.toFixed(3)} V, ${s.light} lx`);
    setSending(false);
  };

  const pictures = () => {
    if (!contactRef.current || sending) return;
    stopTimer();
    framesRef.current = 0;
    setFrames(0);
    setIncomplete(false);
    setSending(true);
    note(level === 'beginner' ? 'You asked the camera for pictures.' : 'Command sent: start test camera');
    timer.current = setInterval(() => {
      if (!contactRef.current) {
        stopTimer();
        setSending(false);
        setIncomplete(true);
        note(`Contact lost. Only ${framesRef.current} of ${FRAMES} pictures made it.`);
        return;
      }
      framesRef.current += 1;
      setFrames(framesRef.current);
      if (framesRef.current === FRAMES) {
        stopTimer();
        setSending(false);
        note(level === 'beginner' ? 'All 8 pictures made it down.' : 'Received all 8 frames, one every 0.5 s.');
      }
    }, 500);
  };

  const toggle = () => {
    if (contactRef.current) {
      const wasSending = timer.current !== null;
      const got = framesRef.current;
      contactRef.current = false;
      stopTimer();
      setSending(false);
      setContact(false);
      if (wasSending) {
        setIncomplete(true);
        note(`Contact lost. Only ${got} of ${FRAMES} pictures made it.`);
      } else {
        note('Contact paused. The last readings are historical.');
      }
    } else {
      contactRef.current = true;
      setContact(true);
      note('Contact is back. Try again.');
    }
  };

  const age = reading ? Math.max(0, now - reading.captured) : 0;
  const fresh = !!reading && contact && age <= 5000;
  const ageText = reading
    ? `${fresh ? 'Fresh' : 'Historical'}: this reading is ${Math.floor(age / 1000)} s old.`
    : 'No reading yet.';

  return (
    <Screen title="Ground station" summary="Practice mode. All data is simulated on your phone.">
      <LevelPicker />
      <Card tone="night">
        <View style={styles.status} accessibilityLiveRegion="polite">
          <View style={[styles.dot, { backgroundColor: contact ? theme.mint : '#ef6b6c' }]} />
          <Text style={styles.statusText}>{contact ? 'Connected to the practice CubeSat' : 'Contact paused'}</Text>
        </View>
        <Button label={w.read} onPress={ask} disabled={sending} />
        <Button label={w.pics} kind="light" onPress={pictures} disabled={sending} />
        <Button label={contact ? 'Pause contact' : 'Restore contact'} kind="light" onPress={toggle} />
      </Card>

      <Card>
        <Text accessibilityRole="header" style={shared.heading}>{reading ? `Reading ${reading.sequence}` : 'No reading yet'}</Text>
        <Row name="Temperature" value={reading ? `${reading.temperature.toFixed(1)} °C` : '--'} />
        <Row name={w.volt} value={reading ? `${reading.voltage.toFixed(3)} V` : '--'} />
        <Row name="Light" value={reading ? `${reading.light} lx` : '--'} />
        <Text accessibilityLiveRegion="polite" style={shared.small}>{ageText}</Text>
        {level === 'professional' && reading ? (
          <View>
            <Text style={shared.label}>Raw telemetry</Text>
            <Text style={styles.raw}>{JSON.stringify({
              sequence: reading.sequence,
              captured_unix_ms: reading.captured,
              source: reading.source,
              temperature_c: reading.temperature,
              bus_voltage_v: reading.voltage,
              light_lux: reading.light,
            }, null, 1)}</Text>
          </View>
        ) : null}
      </Card>

      <Card>
        <Text accessibilityRole="header" style={shared.heading}>Test camera</Text>
        <View
          style={styles.chart}
          accessibilityLabel={frames === 0 ? 'Test camera: waiting' : `Test camera: frame ${frames} of ${FRAMES}`}
        >
          {Array.from({ length: FRAMES }, (_, i) => (
            <View key={i} style={[styles.swatch, { backgroundColor: COLORS[i % COLORS.length] }]}>
              {frames === i + 1 ? <View style={styles.marker} /> : null}
            </View>
          ))}
        </View>
        <Text style={shared.body}>{frames} of {FRAMES} pictures{incomplete ? ', incomplete' : ''}</Text>
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
  statusText: { color: theme.white, fontSize: 16, fontWeight: '700', flexShrink: 1 },
  row: { flexDirection: 'row', flexWrap: 'wrap', justifyContent: 'space-between', gap: 6, paddingVertical: 8, borderBottomWidth: 1, borderBottomColor: theme.line },
  value: { color: theme.ink, fontSize: 19, fontWeight: '700', fontVariant: ['tabular-nums'] },
  raw: { color: theme.ink, backgroundColor: theme.paper, borderRadius: 8, padding: 10, fontSize: 13, fontFamily: 'monospace' },
  chart: { height: 72, borderRadius: 8, overflow: 'hidden', flexDirection: 'row', backgroundColor: '#0b1a33' },
  swatch: { flex: 1, alignItems: 'center', justifyContent: 'center' },
  marker: { width: 22, height: 22, borderRadius: 4, backgroundColor: '#fff', borderWidth: 3, borderColor: '#121c2e' },
});
