// Arduino bench CubeSat for the CubeSat Classroom.
//
// One Arduino Nano 33 BLE Sense Rev2 plays the whole spacecraft. Its built-in
// sensors are the payload and attitude sensors, its RGB LED is the status
// light, a jumper wire is the remove-before-flight pin, and the USB cable
// stands in for the radio. The Arduino IDE Serial Monitor (115200 baud,
// "Newline") is the ground station. See docs/bench-cubesats.md.
//
// Every live reading goes out as one CSC1 line, so the station's
// `sensor-bridge` can read this board too:
//
//   CSC1 seq=42 t_ms=61000 mode=NOMINAL temp_mc=23410 volt_mv=3297 ...
//
// Lines starting with # come from the bench test port. A real CubeSat would
// not send them. Lines starting with REC are stored readings sent after a
// pass begins. They are old, so they never use the CSC1 prefix.
//
// The radio is never switched on. Bluetooth stays off.
//
// Wiring (a mentor checks it before USB is connected):
//   Remove-before-flight jumper: one wire from D2 to GND. Nothing else.
//
// Status: written 25 Nov 2026. Compiled in CI. Not yet run on hardware.
// Label every reading "hardware bench" and check it against a thermometer,
// a barometer or weather report, and a light meter before trusting it.

#include <Arduino_APDS9960.h>
#include <Arduino_BMI270_BMM150.h>
#include <Arduino_HS300x.h>
#include <Arduino_LPS22HB.h>

#if !defined(ARDUINO_ARDUINO_NANO33BLE)
#error "Choose Tools > Board > Arduino Mbed OS Nano Boards > Arduino Nano 33 BLE (it covers the Sense Rev2)."
#endif

// ---- Mission settings: crews may change these -----------------------------

const char *SAT_NAME = "CSC-ARD";
const char *FIRMWARE = "arduino-cubesat 0.1.0";

const int RBF_PIN = 2;                       // jumper from D2 to GND = inhibited
const unsigned long DEPLOY_DELAY_MS = 30000; // demo value; providers set the real one
const unsigned long NOMINAL_PERIOD_MS = 1000;
const unsigned long SAFE_PERIOD_MS = 10000;
const unsigned long BEACON_MS = 10000;
const long SAFE_BELOW_MV = 3000;             // 3.3 V rail sagging below this = safe mode

// Light. The APDS9960 reports counts, not lux. Leave this at 0 until a crew
// has calibrated it against a light meter with the CAL command. Until then
// the board sends light_clear counts and no light_lux, so the station shows
// the light sensor as unavailable rather than showing a guessed lux value.
const float LUX_PER_COUNT = 0.0;

// ---- Spacecraft state -----------------------------------------------------

enum Mode { INHIBITED, DEPLOYING, NOMINAL, SAFE };
const char *MODE_NAMES[] = {"INHIBITED", "DEPLOYING", "NOMINAL", "SAFE"};

struct Sample {
  uint32_t seq;
  uint32_t t_ms;
  long volt_mv;
  bool hasTemp, hasHumidity, hasPressure, hasColor, hasAccel, hasGyro, hasMag;
  long temp_mc, humidity_mpct, pressure_pa;
  int red, green, blue, clear;
  long ax_mg, ay_mg, az_mg, gx_mdps, gy_mdps, gz_mdps, mx_nt, my_nt, mz_nt;
};

// A stored frame keeps the readings a ground crew needs most.
struct Frame {
  uint32_t seq;
  uint32_t t_ms;
  int32_t temp_mc;      // INT32_MIN = missing
  int32_t volt_mv;
  int32_t pressure_pa;  // INT32_MIN = missing
  int32_t light_clear;  // -1 = missing
};

const size_t STORE_FRAMES = 64;
Frame store[STORE_FRAMES];
size_t storeHead = 0;   // next slot to write
size_t storeCount = 0;
uint32_t storeDropped = 0;

Mode mode = INHIBITED;
unsigned long modeSince = 0;
bool inContact = true;
unsigned long contactSince = 0;
unsigned long orbitUpMs = 0, orbitDownMs = 0;  // 0 = orbit simulation off
unsigned long periodMs = NOMINAL_PERIOD_MS;
unsigned long lastSample = 0, lastBeacon = 0, lastBenchNote = 0;
uint32_t seq = 0;
float luxPerCount = LUX_PER_COUNT;
Sample last;
bool haveLast = false;

bool hasHs300x = false, hasBaro = false, hasApds = false, hasImu = false;

long recentIds[8] = {-1, -1, -1, -1, -1, -1, -1, -1};
size_t recentNext = 0;

char line[72];
size_t lineLen = 0;
bool lineTooLong = false;

// RBF debounce
int rbfStable = HIGH, rbfLastRaw = HIGH;
unsigned long rbfChangedAt = 0;

// ---- Status light (RGB LED is active low on this board) -------------------

unsigned long ledOffAt = 0;

void rgb(bool r, bool g, bool b) {
  digitalWrite(LEDR, r ? LOW : HIGH);
  digitalWrite(LEDG, g ? LOW : HIGH);
  digitalWrite(LEDB, b ? LOW : HIGH);
}

void flash(bool r, bool g, bool b, unsigned long ms) {
  rgb(r, g, b);
  ledOffAt = millis() + ms;
}

void updateLight(unsigned long now) {
  if (mode == DEPLOYING) {
    rgb(false, false, (now / 500) % 2 == 0);  // blue, 1 Hz: waiting to deploy
    return;
  }
  if (mode == SAFE && ledOffAt == 0) {
    rgb((now / 1000) % 4 == 0, false, false);  // slow red: safe mode
    return;
  }
  if (ledOffAt != 0 && (long)(now - ledOffAt) >= 0) {
    rgb(false, false, false);
    ledOffAt = 0;
  }
}

// ---- Electrical power: measure the board's own 3.3 V rail ----------------
//
// The nRF52840 can measure its own supply (VDD) with its ADC. With gain 1/6
// and the 0.6 V internal reference, full scale is 3.6 V over 4096 counts.
// This is the regulated 3.3 V bus, not a battery.

long readRailMillivolts() {
  static volatile int16_t result = 0;
  NRF_SAADC->ENABLE = SAADC_ENABLE_ENABLE_Enabled << SAADC_ENABLE_ENABLE_Pos;
  NRF_SAADC->RESOLUTION = SAADC_RESOLUTION_VAL_12bit;
  NRF_SAADC->CH[0].PSELN = SAADC_CH_PSELN_PSELN_NC;
  NRF_SAADC->CH[0].CONFIG =
      (SAADC_CH_CONFIG_GAIN_Gain1_6 << SAADC_CH_CONFIG_GAIN_Pos) |
      (SAADC_CH_CONFIG_REFSEL_Internal << SAADC_CH_CONFIG_REFSEL_Pos) |
      (SAADC_CH_CONFIG_TACQ_10us << SAADC_CH_CONFIG_TACQ_Pos);
  NRF_SAADC->CH[0].PSELP = SAADC_CH_PSELP_PSELP_VDD;
  NRF_SAADC->RESULT.PTR = (uint32_t)&result;
  NRF_SAADC->RESULT.MAXCNT = 1;

  NRF_SAADC->EVENTS_STARTED = 0;
  NRF_SAADC->TASKS_START = 1;
  while (NRF_SAADC->EVENTS_STARTED == 0) {}
  NRF_SAADC->EVENTS_END = 0;
  NRF_SAADC->TASKS_SAMPLE = 1;
  while (NRF_SAADC->EVENTS_END == 0) {}
  NRF_SAADC->EVENTS_STOPPED = 0;
  NRF_SAADC->TASKS_STOP = 1;
  while (NRF_SAADC->EVENTS_STOPPED == 0) {}

  NRF_SAADC->CH[0].PSELP = SAADC_CH_PSELP_PSELP_NC;
  NRF_SAADC->ENABLE = SAADC_ENABLE_ENABLE_Disabled << SAADC_ENABLE_ENABLE_Pos;
  long counts = result < 0 ? 0 : result;
  return counts * 3600L / 4096L;
}

// ---- Sensors ---------------------------------------------------------------

Sample takeSample(unsigned long now) {
  Sample s = {};
  s.seq = ++seq;
  s.t_ms = now;
  s.volt_mv = readRailMillivolts();

  if (hasHs300x) {
    float c = HS300x.readTemperature();
    float rh = HS300x.readHumidity();
    // The sensor sits next to the processor, so it reads warm. Compare it
    // with a thermometer and record the difference.
    if (!isnan(c)) { s.hasTemp = true; s.temp_mc = lroundf(c * 1000.0f); }
    if (!isnan(rh)) { s.hasHumidity = true; s.humidity_mpct = lroundf(rh * 1000.0f); }
  }
  if (hasBaro) {
    float kpa = BARO.readPressure();  // kilopascals
    if (kpa > 1.0f && kpa < 120.0f) { s.hasPressure = true; s.pressure_pa = lroundf(kpa * 1000.0f); }
  }
  if (hasApds && APDS.colorAvailable()) {
    APDS.readColor(s.red, s.green, s.blue, s.clear);
    s.hasColor = true;
  }
  if (hasImu) {
    float x, y, z;
    if (IMU.accelerationAvailable() && IMU.readAcceleration(x, y, z)) {  // g
      s.hasAccel = true;
      s.ax_mg = lroundf(x * 1000.0f); s.ay_mg = lroundf(y * 1000.0f); s.az_mg = lroundf(z * 1000.0f);
    }
    if (IMU.gyroscopeAvailable() && IMU.readGyroscope(x, y, z)) {  // degrees per second
      s.hasGyro = true;
      s.gx_mdps = lroundf(x * 1000.0f); s.gy_mdps = lroundf(y * 1000.0f); s.gz_mdps = lroundf(z * 1000.0f);
    }
    if (IMU.magneticFieldAvailable() && IMU.readMagneticField(x, y, z)) {  // microtesla
      s.hasMag = true;
      s.mx_nt = lroundf(x * 1000.0f); s.my_nt = lroundf(y * 1000.0f); s.mz_nt = lroundf(z * 1000.0f);
    }
  }
  return s;
}

void field(const char *key, long value) {
  Serial.print(' ');
  Serial.print(key);
  Serial.print('=');
  Serial.print(value);
}

void sendLive(const Sample &s) {
  Serial.print("CSC1");
  field("seq", s.seq);
  field("t_ms", s.t_ms);
  Serial.print(" mode=");
  Serial.print(MODE_NAMES[mode]);
  if (s.hasTemp) field("temp_mc", s.temp_mc);
  field("volt_mv", s.volt_mv);
  if (s.hasPressure) field("pressure_pa", s.pressure_pa);
  if (s.hasHumidity) field("humidity_mpct", s.humidity_mpct);
  if (s.hasColor) {
    if (luxPerCount > 0) field("light_lux", lroundf(s.clear * luxPerCount));
    field("light_clear", s.clear);
    field("light_r", s.red);
    field("light_g", s.green);
    field("light_b", s.blue);
  }
  if (s.hasAccel) { field("ax_mg", s.ax_mg); field("ay_mg", s.ay_mg); field("az_mg", s.az_mg); }
  if (s.hasGyro) { field("gx_mdps", s.gx_mdps); field("gy_mdps", s.gy_mdps); field("gz_mdps", s.gz_mdps); }
  if (s.hasMag) { field("mx_nt", s.mx_nt); field("my_nt", s.my_nt); field("mz_nt", s.mz_nt); }
  Serial.println();
}

// ---- Store and forward -----------------------------------------------------

void storeFrame(const Sample &s) {
  Frame &f = store[storeHead];
  f.seq = s.seq;
  f.t_ms = s.t_ms;
  f.temp_mc = s.hasTemp ? s.temp_mc : INT32_MIN;
  f.volt_mv = s.volt_mv;
  f.pressure_pa = s.hasPressure ? s.pressure_pa : INT32_MIN;
  f.light_clear = s.hasColor ? s.clear : -1;
  storeHead = (storeHead + 1) % STORE_FRAMES;
  if (storeCount < STORE_FRAMES) storeCount++;
  else storeDropped++;  // the oldest frame was overwritten
}

void sendStored() {
  size_t first = (storeHead + STORE_FRAMES - storeCount) % STORE_FRAMES;
  for (size_t i = 0; i < storeCount; i++) {
    const Frame &f = store[(first + i) % STORE_FRAMES];
    Serial.print("REC");
    field("seq", f.seq);
    field("t_ms", f.t_ms);
    if (f.temp_mc != INT32_MIN) field("temp_mc", f.temp_mc);
    field("volt_mv", f.volt_mv);
    if (f.pressure_pa != INT32_MIN) field("pressure_pa", f.pressure_pa);
    if (f.light_clear >= 0) field("light_clear", f.light_clear);
    Serial.println();
  }
  storeCount = 0;
}

// ---- Modes and contact -----------------------------------------------------

void setMode(Mode next, const char *why) {
  if (next == mode) return;
  mode = next;
  modeSince = millis();
  periodMs = (mode == SAFE) ? SAFE_PERIOD_MS : NOMINAL_PERIOD_MS;
  if (mode == DEPLOYING) {
    // A fresh deployment starts in contact with an empty store, like a power-up.
    inContact = true;
    orbitUpMs = orbitDownMs = 0;
    storeCount = 0;
    storeDropped = 0;
  }
  if (mode == NOMINAL || mode == SAFE) {
    Serial.print("EVT mode=");
    Serial.print(MODE_NAMES[mode]);
    Serial.print(" reason=");
    Serial.println(why);
  } else {
    Serial.print("# bench: mode=");
    Serial.print(MODE_NAMES[mode]);
    Serial.print(" reason=");
    Serial.println(why);
  }
  if (mode != SAFE) rgb(false, false, false);
}

void acquire(unsigned long now) {
  inContact = true;
  contactSince = now;
  Serial.print("EVT AOS stored=");
  Serial.print(storeCount);
  Serial.print(" dropped=");
  Serial.println(storeDropped);
  sendStored();
  storeDropped = 0;
  flash(true, true, true, 300);
}

void loseContact(unsigned long now) {
  inContact = false;
  contactSince = now;
  // Nothing is sent now: the satellite is out of range of the ground station.
}

void beacon() {
  Serial.print("BCN ");
  Serial.print(SAT_NAME);
  Serial.print(" mode=");
  Serial.print(MODE_NAMES[mode]);
  field("seq", seq);
  field("uptime_s", millis() / 1000);
  Serial.println();
}

// ---- Commands from the ground station --------------------------------------

void ack(long id, const char *cmd) {
  Serial.print("ACK ");
  if (id < 0) Serial.print('-'); else Serial.print(id);
  Serial.print(' ');
  Serial.print(cmd);
}

void nak(long id, const char *cmd, const char *why) {
  Serial.print("NAK ");
  if (id < 0) Serial.print('-'); else Serial.print(id);
  Serial.print(' ');
  Serial.print(cmd);
  Serial.print(' ');
  Serial.println(why);
}

void help() {
  Serial.println("# commands (optionally start with a number, e.g. \"7 PING\"):");
  Serial.println("#   PING  SENSE  STATUS  RATE <1-60 s>  SAFE  NOMINAL");
  Serial.println("#   LOS  AOS  ORBIT <contact s> <gap s>  ORBIT OFF  CAL <lux>  HELP");
}

void status(long id) {
  ack(id, "STATUS");
  Serial.print(" mode=");
  Serial.print(MODE_NAMES[mode]);
  field("rate_ms", periodMs);
  field("stored", storeCount);
  field("uptime_s", millis() / 1000);
  Serial.print(" orbit=");
  Serial.print(orbitUpMs ? "on" : "off");
  Serial.print(" sensors=");
  Serial.print(hasHs300x ? "temp+humidity," : "");
  Serial.print(hasBaro ? "pressure," : "");
  Serial.print(hasApds ? "color," : "");
  Serial.print(hasImu ? "imu," : "");
  Serial.print("rail");
  Serial.print(" lux_per_count=");
  Serial.println(luxPerCount, 5);
}

bool seenBefore(long id) {
  if (id < 0) return false;
  for (long r : recentIds) if (r == id) return true;
  recentIds[recentNext] = id;
  recentNext = (recentNext + 1) % 8;
  return false;
}

void handleLine(char *text) {
  unsigned long now = millis();
  // Cast to unsigned char: a byte of 0x80 or more is negative as a char, and toupper() of a
  // negative value is undefined.
  for (char *p = text; *p; p++) *p = toupper((unsigned char)*p);

  char *word = strtok(text, " \t");
  if (!word) return;
  long id = -1;
  if (isdigit((unsigned char)word[0])) {
    id = atol(word);
    word = strtok(nullptr, " \t");
    if (!word) return;
  }
  char *arg1 = strtok(nullptr, " \t");
  char *arg2 = strtok(nullptr, " \t");

  // In the deployer or waiting out the deploy timer, the radio is off.
  if (mode == INHIBITED || mode == DEPLOYING) {
    Serial.println("# bench: radio is off before deployment; command not heard");
    return;
  }
  // Out of contact, nothing reaches the satellite. AOS stands for the next pass.
  if (!inContact) {
    if (strcmp(word, "AOS") == 0) {
      orbitUpMs = orbitDownMs = 0;
      acquire(now);
      ack(id, "AOS");
      Serial.println();
    }
    return;
  }
  if (seenBefore(id)) { nak(id, word, "duplicate"); return; }

  if (strcmp(word, "PING") == 0) {
    ack(id, "PING");
    field("seq", seq);
    Serial.println();
  } else if (strcmp(word, "SENSE") == 0) {
    ack(id, "SENSE");
    Serial.println();
    last = takeSample(now);
    haveLast = true;
    sendLive(last);
  } else if (strcmp(word, "STATUS") == 0) {
    status(id);
  } else if (strcmp(word, "RATE") == 0) {
    long s = arg1 ? atol(arg1) : 0;
    if (s < 1 || s > 60) { nak(id, "RATE", "use 1 to 60 seconds"); return; }
    if (mode == SAFE) { nak(id, "RATE", "safe mode keeps 10 s"); return; }
    periodMs = s * 1000UL;
    ack(id, "RATE");
    field("rate_ms", periodMs);
    Serial.println();
  } else if (strcmp(word, "SAFE") == 0) {
    ack(id, "SAFE");
    Serial.println();
    setMode(SAFE, "command");
  } else if (strcmp(word, "NOMINAL") == 0) {
    if (mode != SAFE) { nak(id, "NOMINAL", "already nominal"); return; }
    if (readRailMillivolts() < SAFE_BELOW_MV) { nak(id, "NOMINAL", "rail still low"); return; }
    ack(id, "NOMINAL");
    Serial.println();
    setMode(NOMINAL, "command");
  } else if (strcmp(word, "LOS") == 0) {
    ack(id, "LOS");
    Serial.println();
    orbitUpMs = orbitDownMs = 0;
    loseContact(now);
  } else if (strcmp(word, "AOS") == 0) {
    nak(id, "AOS", "already in contact");
  } else if (strcmp(word, "ORBIT") == 0) {
    if (arg1 && strcmp(arg1, "OFF") == 0) {
      orbitUpMs = orbitDownMs = 0;
      ack(id, "ORBIT");
      Serial.println(" orbit=off");
      return;
    }
    long up = arg1 ? atol(arg1) : 0, down = arg2 ? atol(arg2) : 0;
    if (up < 10 || up > 600 || down < 10 || down > 600) {
      nak(id, "ORBIT", "use ORBIT <10-600> <10-600> or ORBIT OFF");
      return;
    }
    orbitUpMs = up * 1000UL;
    orbitDownMs = down * 1000UL;
    contactSince = now;
    ack(id, "ORBIT");
    field("contact_s", up);
    field("gap_s", down);
    Serial.println();
  } else if (strcmp(word, "CAL") == 0) {
    float lux = arg1 ? atof(arg1) : -1;
    if (lux == 0) {
      luxPerCount = 0;
      ack(id, "CAL");
      Serial.println(" lux_per_count=0 light_lux=off");
      return;
    }
    if (!haveLast || !last.hasColor || last.clear <= 0 || lux < 0) {
      nak(id, "CAL", "need CAL <reference lux> and a non-zero light_clear reading");
      return;
    }
    luxPerCount = lux / last.clear;
    ack(id, "CAL");
    Serial.print(" lux_per_count=");
    Serial.println(luxPerCount, 5);
    Serial.println("# bench: copy that number into LUX_PER_COUNT to keep it after a reset");
  } else if (strcmp(word, "HELP") == 0) {
    ack(id, "HELP");
    Serial.println();
    help();
  } else {
    nak(id, word, "unknown command; try HELP");
  }
}

void readCommands() {
  while (Serial.available()) {
    char c = Serial.read();
    if (c == '\r' || c == '\n') {
      if (lineTooLong) {
        Serial.println("NAK - LINE too long");
      } else if (lineLen > 0) {
        line[lineLen] = '\0';
        handleLine(line);
      }
      lineLen = 0;
      lineTooLong = false;
    } else if (lineLen < sizeof(line) - 1) {
      line[lineLen++] = c;
    } else {
      lineTooLong = true;
    }
  }
}

// ---- Remove-before-flight pin ---------------------------------------------

void watchInhibit(unsigned long now) {
  int raw = digitalRead(RBF_PIN);
  if (raw != rbfLastRaw) { rbfLastRaw = raw; rbfChangedAt = now; }
  if (now - rbfChangedAt < 50 || raw == rbfStable) return;
  rbfStable = raw;
  if (rbfStable == LOW) {
    setMode(INHIBITED, "rbf_jumper_installed");
  } else if (mode == INHIBITED) {
    setMode(DEPLOYING, "rbf_jumper_removed");
  }
}

// ---- Arduino entry points --------------------------------------------------

void setup() {
  pinMode(LEDR, OUTPUT);
  pinMode(LEDG, OUTPUT);
  pinMode(LEDB, OUTPUT);
  rgb(false, false, false);
  pinMode(RBF_PIN, INPUT_PULLUP);

  Serial.begin(115200);
  unsigned long waitStart = millis();
  while (!Serial && millis() - waitStart < 3000) {}

  hasHs300x = HS300x.begin();
  hasBaro = BARO.begin();
  hasApds = APDS.begin();
  hasImu = IMU.begin();

  Serial.print("# bench: boot ");
  Serial.print(SAT_NAME);
  Serial.print(' ');
  Serial.println(FIRMWARE);
  if (!hasHs300x) Serial.println("# bench: HS3003 temperature sensor not found (is this a Rev2 board?)");
  if (!hasBaro) Serial.println("# bench: LPS22HB pressure sensor not found");
  if (!hasApds) Serial.println("# bench: APDS9960 light sensor not found");
  if (!hasImu) Serial.println("# bench: BMI270/BMM150 motion sensors not found (is this a Rev2 board?)");
  help();

  delay(60);
  rbfStable = rbfLastRaw = digitalRead(RBF_PIN);
  rbfChangedAt = millis();
  if (rbfStable == LOW) {
    mode = INHIBITED;
    Serial.println("# bench: mode=INHIBITED reason=rbf_jumper_installed");
  } else {
    mode = DEPLOYING;
    Serial.println("# bench: no remove-before-flight jumper at boot; deploy timer started");
  }
  modeSince = millis();
}

void loop() {
  unsigned long now = millis();
  watchInhibit(now);
  readCommands();
  updateLight(now);

  if (mode == INHIBITED) {
    if (now - lastBenchNote >= 10000) {
      lastBenchNote = now;
      Serial.println("# bench: INHIBITED. Pull the D2 jumper to deploy.");
    }
    return;
  }
  if (mode == DEPLOYING) {
    if (now - modeSince >= DEPLOY_DELAY_MS) {
      setMode(NOMINAL, "deploy_timer_done");
      contactSince = now;
      beacon();
      lastBeacon = now;
    } else if (now - lastBenchNote >= 5000) {
      lastBenchNote = now;
      Serial.print("# bench: deploying, ");
      Serial.print((DEPLOY_DELAY_MS - (now - modeSince) + 999) / 1000);
      Serial.println(" s to go");
    }
    return;
  }

  // Simulated orbit: contact for orbitUpMs, then out of contact for orbitDownMs.
  if (orbitUpMs) {
    if (inContact && now - contactSince >= orbitUpMs) loseContact(now);
    else if (!inContact && now - contactSince >= orbitDownMs) acquire(now);
  }

  if (now - lastSample >= periodMs) {
    lastSample = now;
    last = takeSample(now);
    haveLast = true;
    // The nRF52840 stops working below about 1.7 V, so a lower reading means
    // the measurement itself is wrong, not the rail. Report it, but don't act on it.
    if (last.volt_mv < 1700) {
      if (now - lastBenchNote >= 10000) {
        lastBenchNote = now;
        Serial.println("# bench: the 3.3 V rail reading looks wrong; safe-mode check skipped");
      }
    } else if (mode == NOMINAL && last.volt_mv < SAFE_BELOW_MV) {
      setMode(SAFE, "rail_low");
    }
    if (inContact) {
      sendLive(last);
      flash(false, true, false, 60);  // green blink: telemetry sent
    } else {
      storeFrame(last);
      flash(true, true, false, 30);   // yellow blink: stored for later
    }
  }
  if (inContact && now - lastBeacon >= BEACON_MS) {
    lastBeacon = now;
    beacon();
  }
}
