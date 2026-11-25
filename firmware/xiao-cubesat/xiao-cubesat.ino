// Seeed XIAO bench CubeSat for the CubeSat Classroom.
//
// A Seeed Studio XIAO on the Seeed XIAO Expansion Board plays the spacecraft,
// with Seeed Grove sensors as the payload:
//
//   Expansion Board: OLED status screen, PCF8563 real-time clock, microSD
//     data recorder, user button (deploy switch), buzzer
//   Grove BME280: temperature, humidity, pressure
//   Grove Digital Light Sensor (TSL2561): light in lux
//   Grove 6-Axis Accelerometer & Gyroscope (LSM6DS3): motion
//   XIAO ESP32-S3 Sense only: camera
//
// The USB cable stands in for the radio and the Serial Monitor (115200 baud,
// "Newline") is the ground station. Every part is optional: the sketch
// reports what it finds and leaves out what it cannot read. See
// docs/bench-cubesats.md.
//
// Live readings go out as CSC1 lines that the station's `sensor-bridge` reads:
//
//   CSC1 seq=42 t_ms=61000 mode=NOMINAL temp_mc=22410 pressure_pa=101210 ...
//
// Lines starting with # come from the bench test port; a real CubeSat would
// not send them. REC lines are stored readings sent after a pass begins. IMG
// lines carry a photo as hex. Neither uses the CSC1 prefix, because neither
// is a live reading.
//
// Wi-Fi and Bluetooth are never switched on.
//
// Boards: XIAO ESP32-S3 (and S3 Sense), ESP32-C3, RP2040 and SAMD21. Other
// boards stop at compile time. Board packages: esp32 by Espressif, Raspberry
// Pi Pico/RP2040 by Earle Philhower, Seeed SAMD Boards.
//
// Wiring (a mentor checks it before USB is connected):
//   XIAO into the Expansion Board socket, as the Seeed wiki shows.
//   Grove I2C Hub into one Grove I2C port; BME280, light sensor and IMU into
//   the hub with their Grove cables.
//   Optional voltage: two 100 kOhm resistors from the voltage under test to
//   A0 to GND, plus 100 nF from A0 to GND. Then set USE_VOLTAGE_DIVIDER to
//   true. It reads at most 6.6 V. Never connect more than that.
//
// Status: written 25 Nov 2026. Compiled in CI. Not yet run on hardware.
// Label every reading "hardware bench" and check it against a reference.

#include <stdarg.h>
#include <SD.h>
#include <SPI.h>
#include <U8x8lib.h>
#include <Wire.h>
#include <Adafruit_BME280.h>
#include <Adafruit_TSL2561_U.h>

// ---- Board support ---------------------------------------------------------

#if defined(ARDUINO_ARCH_ESP32) && defined(CONFIG_IDF_TARGET_ESP32S3)
  #define BOARD_NAME "XIAO ESP32-S3"
  #define HAS_CHIP_TEMP 1
  #define HAS_USER_LED 1
  const int LED_PIN = LED_BUILTIN;  // GPIO21, lit when LOW
  const int LED_ON = LOW;
  const int BUTTON_PIN = D1, BUZZER_PIN = A3, SD_CS_PIN = D2;
#elif defined(ARDUINO_ARCH_ESP32) && defined(CONFIG_IDF_TARGET_ESP32C3)
  #define BOARD_NAME "XIAO ESP32-C3"
  #define HAS_CHIP_TEMP 1
  #define HAS_USER_LED 0  // this XIAO has no user LED; the OLED and buzzer report status
  const int BUTTON_PIN = D1, BUZZER_PIN = A3, SD_CS_PIN = D2;
#elif defined(ARDUINO_ARCH_RP2040) && !defined(ARDUINO_ARCH_MBED)
  #define BOARD_NAME "XIAO RP2040"
  #define HAS_CHIP_TEMP 1
  #define HAS_USER_LED 1
  const int LED_PIN = LED_BUILTIN;  // lit when LOW
  const int LED_ON = LOW;
  const int BUTTON_PIN = D1, BUZZER_PIN = A3, SD_CS_PIN = D2;
#elif defined(ARDUINO_ARCH_SAMD)
  #define BOARD_NAME "XIAO SAMD21"
  #define HAS_CHIP_TEMP 0
  #define HAS_USER_LED 1
  const int LED_PIN = LED_BUILTIN;  // if it lights at the wrong time, set LED_ON to HIGH
  const int LED_ON = LOW;
  const int BUTTON_PIN = 1, BUZZER_PIN = A3, SD_CS_PIN = 2;
#else
  #error "Choose a Seeed XIAO ESP32S3, ESP32C3, RP2040 (Earle Philhower core) or SAMD21 board."
#endif

#if defined(CONFIG_IDF_TARGET_ESP32S3)
  #define CAMERA_SUPPORTED 1
  #include "esp_camera.h"
#else
  #define CAMERA_SUPPORTED 0
#endif

#if defined(ARDUINO_ARCH_ESP32)
  #define LOG_OPEN_MODE FILE_APPEND
#else
  #define LOG_OPEN_MODE FILE_WRITE
#endif

// ---- Mission settings: crews may change these -----------------------------

const char *SAT_NAME = "CSC-XIAO";
const char *FIRMWARE = "xiao-cubesat 0.1.0";

const unsigned long DEPLOY_DELAY_MS = 30000;  // demo value; providers set the real one
const unsigned long NOMINAL_PERIOD_MS = 1000;
const unsigned long SAFE_PERIOD_MS = 10000;
const unsigned long BEACON_MS = 10000;

const bool USE_VOLTAGE_DIVIDER = false;  // true only after a mentor wires the A0 divider
const float VOLTAGE_DIVIDER = 2.0;       // 100k over 100k
const long SAFE_BELOW_MV = 0;            // e.g. 3500 for a 1-cell LiPo; 0 = never
const bool USE_BUZZER = true;
const bool USE_CAMERA = true;            // only does anything on the XIAO ESP32-S3 Sense

// ---- Spacecraft state -----------------------------------------------------

enum Mode { INHIBITED, DEPLOYING, NOMINAL, SAFE };
const char *MODE_NAMES[] = {"INHIBITED", "DEPLOYING", "NOMINAL", "SAFE"};

struct Sample {
  uint32_t seq;
  uint32_t t_ms;
  bool hasTemp, hasHumidity, hasPressure, hasLight, lightSaturated, hasVolt, hasChipTemp, hasImu;
  long temp_mc, humidity_mpct, pressure_pa, light_lux, volt_mv, chip_temp_mc;
  long ax_mg, ay_mg, az_mg, gx_mdps, gy_mdps, gz_mdps;
  char rtc[20];  // empty if the clock is not set
};

struct Frame {
  uint32_t seq;
  uint32_t t_ms;
  int32_t temp_mc;      // INT32_MIN = missing
  int32_t pressure_pa;  // INT32_MIN = missing
  int32_t light_lux;    // -1 = missing
  int32_t volt_mv;      // -1 = missing
};

const size_t STORE_FRAMES = 64;
Frame store[STORE_FRAMES];
size_t storeHead = 0, storeCount = 0;
uint32_t storeDropped = 0;

Mode mode = INHIBITED;
unsigned long modeSince = 0;
bool inContact = true;
unsigned long contactSince = 0;
unsigned long orbitUpMs = 0, orbitDownMs = 0;
unsigned long periodMs = NOMINAL_PERIOD_MS;
unsigned long lastSample = 0, lastBeacon = 0, lastBenchNote = 0;
uint32_t seq = 0;
Sample last;

// Parts found at boot
bool hasOled = false, hasRtc = false, hasSd = false, hasBme = false, hasTsl = false, hasImu = false, hasCamera = false;
uint8_t imuAddr = 0;
uint32_t sdErrors = 0;

U8X8_SSD1306_128X64_NONAME_HW_I2C oled(U8X8_PIN_NONE);
Adafruit_BME280 bme;
Adafruit_TSL2561_Unified *tsl = nullptr;

long recentIds[8] = {-1, -1, -1, -1, -1, -1, -1, -1};
size_t recentNext = 0;

char line[72];
size_t lineLen = 0;
bool lineTooLong = false;

// Deploy switch (Expansion Board user button, active low)
int buttonStable = HIGH, buttonLastRaw = HIGH;
unsigned long buttonChangedAt = 0;

// ---- Small helpers ---------------------------------------------------------

bool i2cPresent(uint8_t addr) {
  Wire.beginTransmission(addr);
  return Wire.endTransmission() == 0;
}

bool i2cWrite(uint8_t addr, uint8_t reg, uint8_t value) {
  Wire.beginTransmission(addr);
  Wire.write(reg);
  Wire.write(value);
  return Wire.endTransmission() == 0;
}

bool i2cRead(uint8_t addr, uint8_t reg, uint8_t *buf, uint8_t len) {
  Wire.beginTransmission(addr);
  Wire.write(reg);
  if (Wire.endTransmission(false) != 0) return false;
  if (Wire.requestFrom(addr, len) != len) return false;
  for (uint8_t i = 0; i < len; i++) buf[i] = Wire.read();
  return true;
}

void field(const char *key, long value) {
  Serial.print(' ');
  Serial.print(key);
  Serial.print('=');
  Serial.print(value);
}

void beep(unsigned int hz, unsigned long ms) {
  if (USE_BUZZER && !hasCamera) tone(BUZZER_PIN, hz, ms);
}

#if HAS_USER_LED
unsigned long ledOffAt = 0;
void ledFlash(unsigned long ms) {
  digitalWrite(LED_PIN, LED_ON);
  ledOffAt = millis() + ms;
}
void updateLed(unsigned long now) {
  if (mode == DEPLOYING) {
    digitalWrite(LED_PIN, (now / 500) % 2 == 0 ? LED_ON : !LED_ON);
  } else if (ledOffAt != 0 && (long)(now - ledOffAt) >= 0) {
    digitalWrite(LED_PIN, !LED_ON);
    ledOffAt = 0;
  }
}
#else
void ledFlash(unsigned long) {}
void updateLed(unsigned long) {}
#endif

// ---- Analog voltage --------------------------------------------------------

long readMillivolts(int pin) {
#if defined(ARDUINO_ARCH_ESP32)
  analogReadMilliVolts(pin);
  return analogReadMilliVolts(pin);
#else
  analogReadResolution(12);  // XIAO RP2040 and SAMD21: 3.3 V reference
  analogRead(pin);           // first read after switching pins is discarded
  return (long)analogRead(pin) * 3300L / 4095L;
#endif
}

// ---- Real-time clock (PCF8563 on the Expansion Board, address 0x51) --------

const uint8_t RTC_ADDR = 0x51;

uint8_t bcd(uint8_t v) { return (v >> 4) * 10 + (v & 0x0F); }
uint8_t tobcd(uint8_t v) { return ((v / 10) << 4) | (v % 10); }

// Fills out with "YYYY-MM-DDTHH:MM:SS", or leaves it empty if the clock has
// lost power since it was set (the PCF8563 "voltage low" flag).
void readRtc(char *out, size_t size) {
  out[0] = '\0';
  uint8_t r[7];
  if (!hasRtc || !i2cRead(RTC_ADDR, 0x02, r, 7)) return;
  if (r[0] & 0x80) return;
  int year = 2000 + bcd(r[6]) + ((r[5] & 0x80) ? 100 : 0);
  snprintf(out, size, "%04d-%02d-%02dT%02d:%02d:%02d", year, bcd(r[5] & 0x1F), bcd(r[3] & 0x3F),
           bcd(r[2] & 0x3F), bcd(r[1] & 0x7F), bcd(r[0] & 0x7F));
}

bool setRtc(const char *text) {
  int y, mo, d, h, mi, s;
  if (sscanf(text, "%d-%d-%dT%d:%d:%d", &y, &mo, &d, &h, &mi, &s) != 6) return false;
  if (y < 2000 || y > 2199 || mo < 1 || mo > 12 || d < 1 || d > 31 || h > 23 || mi > 59 || s > 59) return false;
  Wire.beginTransmission(RTC_ADDR);
  Wire.write(0x02);
  Wire.write(tobcd(s));  // clears the voltage-low flag
  Wire.write(tobcd(mi));
  Wire.write(tobcd(h));
  Wire.write(tobcd(d));
  Wire.write(0);         // weekday, unused
  Wire.write(tobcd(mo) | (y >= 2100 ? 0x80 : 0));
  Wire.write(tobcd(y % 100));
  return Wire.endTransmission() == 0;
}

// ---- Motion sensor (Grove LSM6DS3, address 0x6A or 0x6B) ------------------

bool startImu() {
  const uint8_t addrs[] = {0x6A, 0x6B};
  for (uint8_t a : addrs) {
    uint8_t who = 0;
    if (!i2cRead(a, 0x0F, &who, 1)) continue;   // WHO_AM_I
    if (who != 0x69 && who != 0x6A) continue;   // LSM6DS3 or LSM6DS3TR-C
    imuAddr = a;
    // 104 Hz, +-2 g (0.061 mg per count); 104 Hz, 245 dps (8.75 mdps per count)
    return i2cWrite(a, 0x10, 0x40) && i2cWrite(a, 0x11, 0x40);
  }
  return false;
}

bool readImu(Sample &s) {
  uint8_t b[12];
  if (!i2cRead(imuAddr, 0x22, b, 12)) return false;  // gyro X..Z, then accel X..Z
  int16_t v[6];
  for (int i = 0; i < 6; i++) v[i] = (int16_t)(b[2 * i] | (b[2 * i + 1] << 8));
  s.gx_mdps = lroundf(v[0] * 8.75f);
  s.gy_mdps = lroundf(v[1] * 8.75f);
  s.gz_mdps = lroundf(v[2] * 8.75f);
  s.ax_mg = lroundf(v[3] * 0.061f);
  s.ay_mg = lroundf(v[4] * 0.061f);
  s.az_mg = lroundf(v[5] * 0.061f);
  return true;
}

// ---- Camera (XIAO ESP32-S3 Sense only) ------------------------------------

uint8_t *photo = nullptr;
size_t photoLen = 0;
uint32_t photoCrc = 0;
uint16_t photoNumber = 0;

uint32_t photoCrc32(const uint8_t *data, size_t len) {
  uint32_t crc = 0xFFFFFFFF;
  for (size_t i = 0; i < len; i++) {
    crc ^= data[i];
    for (int k = 0; k < 8; k++) crc = (crc >> 1) ^ (0xEDB88320 & (0 - (crc & 1)));
  }
  return ~crc;
}

#if CAMERA_SUPPORTED
bool startCamera() {
  camera_config_t c = {};
  c.pin_pwdn = -1;
  c.pin_reset = -1;
  c.pin_xclk = 10;
  c.pin_sccb_sda = 40;
  c.pin_sccb_scl = 39;
  c.pin_d7 = 48;
  c.pin_d6 = 11;
  c.pin_d5 = 12;
  c.pin_d4 = 14;
  c.pin_d3 = 16;
  c.pin_d2 = 18;
  c.pin_d1 = 17;
  c.pin_d0 = 15;
  c.pin_vsync = 38;
  c.pin_href = 47;
  c.pin_pclk = 13;
  c.xclk_freq_hz = 20000000;
  c.ledc_timer = LEDC_TIMER_3;      // keep clear of the timers tone() uses
  c.ledc_channel = LEDC_CHANNEL_7;
  c.pixel_format = PIXFORMAT_JPEG;
  c.frame_size = FRAMESIZE_QVGA;    // 320 x 240 keeps the downlink short
  c.jpeg_quality = 12;
  c.fb_count = 1;
  c.fb_location = psramFound() ? CAMERA_FB_IN_PSRAM : CAMERA_FB_IN_DRAM;
  c.grab_mode = CAMERA_GRAB_LATEST;
  return esp_camera_init(&c) == ESP_OK;
}

// Takes a photo into memory. Returns an error message, or nullptr on success.
const char *takePhoto() {
  camera_fb_t *fb = esp_camera_fb_get();  // the first frame can be stale
  if (fb) esp_camera_fb_return(fb);
  fb = esp_camera_fb_get();
  if (!fb) return "capture failed";
  if (fb->len > 120000) { esp_camera_fb_return(fb); return "photo too large"; }
  free(photo);
  photo = (uint8_t *)(psramFound() ? ps_malloc(fb->len) : malloc(fb->len));
  if (!photo) { photoLen = 0; esp_camera_fb_return(fb); return "out of memory"; }
  memcpy(photo, fb->buf, fb->len);
  photoLen = fb->len;
  esp_camera_fb_return(fb);
  photoCrc = photoCrc32(photo, photoLen);
  photoNumber++;
  return nullptr;
}
#endif

void savePhoto() {
  if (!hasSd || !photo) return;
  char name[16];
  snprintf(name, sizeof(name), "/IMG%04u.JPG", photoNumber);
  File f = SD.open(name, FILE_WRITE);
  if (!f) { sdErrors++; return; }
  f.write(photo, photoLen);
  f.close();
}

void downlinkPhoto(long id) {
  Serial.print("IMG BEGIN");
  field("id", id);
  field("n", photoNumber);
  field("bytes", (long)photoLen);
  Serial.print(" crc32=");
  Serial.println(photoCrc, HEX);
  const size_t CHUNK = 48;
  for (size_t off = 0; off < photoLen; off += CHUNK) {
    Serial.print("IMG ");
    Serial.print((unsigned long)off);
    Serial.print(' ');
    for (size_t i = off; i < off + CHUNK && i < photoLen; i++) {
      if (photo[i] < 16) Serial.print('0');
      Serial.print(photo[i], HEX);
    }
    Serial.println();
  }
  Serial.println("IMG END");
}

// ---- Sensors ---------------------------------------------------------------

Sample takeSample(unsigned long now) {
  Sample s = {};
  s.seq = ++seq;
  s.t_ms = now;
  if (hasBme) {
    float c = bme.readTemperature();
    float pa = bme.readPressure();
    float rh = bme.readHumidity();
    if (!isnan(c)) { s.hasTemp = true; s.temp_mc = lroundf(c * 1000.0f); }
    if (!isnan(pa) && pa > 1000.0f && pa < 120000.0f) { s.hasPressure = true; s.pressure_pa = lroundf(pa); }
    if (!isnan(rh)) { s.hasHumidity = true; s.humidity_mpct = lroundf(rh * 1000.0f); }
  }
  if (hasTsl) {
    sensors_event_t e;
    // Depending on the library version, a saturated (too bright) sensor makes
    // getEvent return false or report 65536 lux. Treat both as saturated.
    if (tsl->getEvent(&e) && e.light < 65535.0f) { s.hasLight = true; s.light_lux = lroundf(e.light); }
    else s.lightSaturated = true;
  }
  if (USE_VOLTAGE_DIVIDER) {
    s.hasVolt = true;
    s.volt_mv = (long)(readMillivolts(A0) * VOLTAGE_DIVIDER);
  }
#if HAS_CHIP_TEMP
  #if defined(ARDUINO_ARCH_ESP32)
  float chip = temperatureRead();
  #else
  float chip = analogReadTemp();
  #endif
  s.hasChipTemp = true;
  s.chip_temp_mc = lroundf(chip * 1000.0f);
#endif
  if (hasImu) s.hasImu = readImu(s);
  readRtc(s.rtc, sizeof(s.rtc));
  return s;
}

void sendLive(const Sample &s) {
  Serial.print("CSC1");
  field("seq", s.seq);
  field("t_ms", s.t_ms);
  Serial.print(" mode=");
  Serial.print(MODE_NAMES[mode]);
  if (s.rtc[0]) { Serial.print(" rtc="); Serial.print(s.rtc); }
  if (s.hasTemp) field("temp_mc", s.temp_mc);
  if (s.hasVolt) field("volt_mv", s.volt_mv);
  if (s.hasLight) field("light_lux", s.light_lux);
  if (s.lightSaturated) Serial.print(" light_saturated=1");
  if (s.hasPressure) field("pressure_pa", s.pressure_pa);
  if (s.hasHumidity) field("humidity_mpct", s.humidity_mpct);
  if (s.hasChipTemp) field("chip_temp_mc", s.chip_temp_mc);  // the processor, not the room
  if (s.hasImu) {
    field("ax_mg", s.ax_mg); field("ay_mg", s.ay_mg); field("az_mg", s.az_mg);
    field("gx_mdps", s.gx_mdps); field("gy_mdps", s.gy_mdps); field("gz_mdps", s.gz_mdps);
  }
  Serial.println();
}

// ---- Data recorder (microSD on the Expansion Board) -----------------------

void logSample(const Sample &s, bool live) {
  if (!hasSd) return;
  File f = SD.open("/CUBESAT.CSV", LOG_OPEN_MODE);
  if (!f) { sdErrors++; return; }
  // seq,t_ms,rtc,mode,in_contact,temp_mc,volt_mv,light_lux,pressure_pa,humidity_mpct,chip_temp_mc,ax_mg,ay_mg,az_mg
  f.print(s.seq); f.print(',');
  f.print(s.t_ms); f.print(',');
  f.print(s.rtc); f.print(',');
  f.print(MODE_NAMES[mode]); f.print(',');
  f.print(live ? 1 : 0); f.print(',');
  if (s.hasTemp) f.print(s.temp_mc);
  f.print(',');
  if (s.hasVolt) f.print(s.volt_mv);
  f.print(',');
  if (s.hasLight) f.print(s.light_lux);
  f.print(',');
  if (s.hasPressure) f.print(s.pressure_pa);
  f.print(',');
  if (s.hasHumidity) f.print(s.humidity_mpct);
  f.print(',');
  if (s.hasChipTemp) f.print(s.chip_temp_mc);
  f.print(',');
  if (s.hasImu) { f.print(s.ax_mg); f.print(','); f.print(s.ay_mg); f.print(','); f.print(s.az_mg); }
  else f.print(",,");
  f.println();
  f.close();
}

void startLog() {
  if (!hasSd || SD.exists("/CUBESAT.CSV")) return;
  File f = SD.open("/CUBESAT.CSV", LOG_OPEN_MODE);
  if (!f) { hasSd = false; return; }
  f.println("seq,t_ms,rtc,mode,in_contact,temp_mc,volt_mv,light_lux,pressure_pa,humidity_mpct,chip_temp_mc,ax_mg,ay_mg,az_mg");
  f.close();
}

// ---- Status screen ---------------------------------------------------------

// Writes one 16-character row, padded with spaces so old text is cleared.
void screenRow(uint8_t row, const char *fmt, ...) {
  char text[17];
  va_list args;
  va_start(args, fmt);
  vsnprintf(text, sizeof(text), fmt, args);
  va_end(args);
  size_t n = strlen(text);
  while (n < 16) text[n++] = ' ';
  text[16] = '\0';
  oled.drawString(0, row, text);
}

void showScreen() {
  if (!hasOled) return;
  screenRow(0, "%s", SAT_NAME);
  screenRow(1, "%s", MODE_NAMES[mode]);
  if (mode == DEPLOYING) {
    screenRow(2, "deploy in %lus", (DEPLOY_DELAY_MS - (millis() - modeSince) + 999) / 1000);
  } else if (mode == INHIBITED) {
    screenRow(2, "press to deploy");
  } else {
    screenRow(2, "%s", inContact ? "contact" : "no contact");
  }
  screenRow(3, "seq %lu", (unsigned long)seq);
  if (last.hasTemp) {
    long t = labs(last.temp_mc);
    screenRow(4, "T %s%ld.%ld C", last.temp_mc < 0 ? "-" : "", t / 1000, (t % 1000) / 100);
  } else {
    screenRow(4, "T --");
  }
  if (last.hasLight) screenRow(5, "L %ld lx", last.light_lux);
  else screenRow(5, "%s", last.lightSaturated ? "L saturated" : "L --");
  if (last.hasPressure) screenRow(6, "P %ld Pa", last.pressure_pa);
  else screenRow(6, "P --");
  screenRow(7, "stored %u", (unsigned)storeCount);
}

// ---- Store and forward -----------------------------------------------------

void storeFrame(const Sample &s) {
  Frame &f = store[storeHead];
  f.seq = s.seq;
  f.t_ms = s.t_ms;
  f.temp_mc = s.hasTemp ? s.temp_mc : INT32_MIN;
  f.pressure_pa = s.hasPressure ? s.pressure_pa : INT32_MIN;
  f.light_lux = s.hasLight ? s.light_lux : -1;
  f.volt_mv = s.hasVolt ? s.volt_mv : -1;
  storeHead = (storeHead + 1) % STORE_FRAMES;
  if (storeCount < STORE_FRAMES) storeCount++;
  else storeDropped++;
}

void sendStored() {
  size_t first = (storeHead + STORE_FRAMES - storeCount) % STORE_FRAMES;
  for (size_t i = 0; i < storeCount; i++) {
    const Frame &f = store[(first + i) % STORE_FRAMES];
    Serial.print("REC");
    field("seq", f.seq);
    field("t_ms", f.t_ms);
    if (f.temp_mc != INT32_MIN) field("temp_mc", f.temp_mc);
    if (f.volt_mv >= 0) field("volt_mv", f.volt_mv);
    if (f.light_lux >= 0) field("light_lux", f.light_lux);
    if (f.pressure_pa != INT32_MIN) field("pressure_pa", f.pressure_pa);
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
  Serial.print(mode == NOMINAL || mode == SAFE ? "EVT mode=" : "# bench: mode=");
  Serial.print(MODE_NAMES[mode]);
  Serial.print(" reason=");
  Serial.println(why);
  if (mode == DEPLOYING) {
    inContact = true;
    orbitUpMs = orbitDownMs = 0;
    storeCount = 0;
    storeDropped = 0;
  }
  showScreen();
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
  beep(1760, 120);
}

void loseContact(unsigned long now) {
  inContact = false;
  contactSince = now;
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
  Serial.println("#   LOS  AOS  ORBIT <contact s> <gap s>  ORBIT OFF");
  Serial.println("#   TIME <YYYY-MM-DDTHH:MM:SS>  PHOTO  DOWNLINK  HELP");
}

void status(long id) {
  ack(id, "STATUS");
  Serial.print(" mode=");
  Serial.print(MODE_NAMES[mode]);
  field("rate_ms", periodMs);
  field("stored", storeCount);
  field("uptime_s", millis() / 1000);
  field("sd_errors", sdErrors);
  Serial.print(" orbit=");
  Serial.print(orbitUpMs ? "on" : "off");
  Serial.print(" board=\"");
  Serial.print(BOARD_NAME);
  Serial.print("\" parts=");
  Serial.print(hasOled ? "oled," : "");
  Serial.print(hasRtc ? "rtc," : "");
  Serial.print(hasSd ? "sd," : "");
  Serial.print(hasBme ? "bme280," : "");
  Serial.print(hasTsl ? "tsl2561," : "");
  Serial.print(hasImu ? "lsm6ds3," : "");
  Serial.print(hasCamera ? "camera," : "");
  Serial.println(USE_VOLTAGE_DIVIDER ? "a0_divider" : "no_divider");
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
  // Commands work in any letter case.
  for (char *p = text; *p; p++) *p = toupper(*p);

  char *word = strtok(text, " \t");
  if (!word) return;
  long id = -1;
  if (isdigit(word[0])) {
    id = atol(word);
    word = strtok(nullptr, " \t");
    if (!word) return;
  }
  char *arg1 = strtok(nullptr, " \t");
  char *arg2 = strtok(nullptr, " \t");

  if (mode == INHIBITED || mode == DEPLOYING) {
    Serial.println("# bench: radio is off before deployment; command not heard");
    return;
  }
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
    if (SAFE_BELOW_MV > 0 && last.hasVolt && last.volt_mv < SAFE_BELOW_MV) { nak(id, "NOMINAL", "voltage still low"); return; }
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
  } else if (strcmp(word, "TIME") == 0) {
    if (!hasRtc) { nak(id, "TIME", "no real-time clock found"); return; }
    if (!arg1 || !setRtc(arg1)) { nak(id, "TIME", "use TIME 2026-11-25T14:30:00"); return; }
    ack(id, "TIME");
    Serial.println();
  } else if (strcmp(word, "PHOTO") == 0) {
#if CAMERA_SUPPORTED
    if (!hasCamera) { nak(id, "PHOTO", "camera not found"); return; }
    const char *err = takePhoto();
    if (err) { nak(id, "PHOTO", err); return; }
    savePhoto();
    ack(id, "PHOTO");
    field("n", photoNumber);
    field("bytes", (long)photoLen);
    Serial.print(" crc32=");
    Serial.println(photoCrc, HEX);
#else
    nak(id, "PHOTO", "no camera on this board");
#endif
  } else if (strcmp(word, "DOWNLINK") == 0) {
    if (!photo || photoLen == 0) { nak(id, "DOWNLINK", "no photo yet; send PHOTO first"); return; }
    ack(id, "DOWNLINK");
    Serial.println();
    downlinkPhoto(id);
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

// ---- Deploy switch ----------------------------------------------------------
//
// At power-up the satellite is still in its deployer: INHIBITED. A press of
// the Expansion Board button stands for the deployer door opening.

void watchDeploySwitch(unsigned long now) {
  int raw = digitalRead(BUTTON_PIN);
  if (raw != buttonLastRaw) { buttonLastRaw = raw; buttonChangedAt = now; }
  if (now - buttonChangedAt < 50 || raw == buttonStable) return;
  buttonStable = raw;
  if (buttonStable == LOW && mode == INHIBITED) {
    setMode(DEPLOYING, "deploy_switch");
    beep(880, 80);
  }
}

// ---- Arduino entry points --------------------------------------------------

void setup() {
#if HAS_USER_LED
  pinMode(LED_PIN, OUTPUT);
  digitalWrite(LED_PIN, !LED_ON);
#endif
  pinMode(BUTTON_PIN, INPUT_PULLUP);

  Serial.begin(115200);
  unsigned long waitStart = millis();
  while (!Serial && millis() - waitStart < 3000) {}

  Wire.begin();

  hasOled = i2cPresent(0x3C);
  if (hasOled) {
    oled.begin();
    oled.setFont(u8x8_font_chroma48medium8_r);
    oled.clearDisplay();
  }
  hasRtc = i2cPresent(RTC_ADDR);
  hasBme = bme.begin(0x76, &Wire) || bme.begin(0x77, &Wire);
  const uint8_t tslAddrs[] = {TSL2561_ADDR_LOW, TSL2561_ADDR_FLOAT, TSL2561_ADDR_HIGH};
  for (uint8_t a : tslAddrs) {
    if (!i2cPresent(a)) continue;
    tsl = new Adafruit_TSL2561_Unified(a, 2561);
    if (tsl->begin(&Wire)) {
      tsl->enableAutoRange(true);
      tsl->setIntegrationTime(TSL2561_INTEGRATIONTIME_101MS);
      hasTsl = true;
      break;
    }
    delete tsl;
    tsl = nullptr;
  }
  hasImu = startImu();
  hasSd = SD.begin(SD_CS_PIN);
  startLog();
#if CAMERA_SUPPORTED
  if (USE_CAMERA) hasCamera = startCamera();
#endif

  Serial.print("# bench: boot ");
  Serial.print(SAT_NAME);
  Serial.print(' ');
  Serial.print(FIRMWARE);
  Serial.print(" on ");
  Serial.println(BOARD_NAME);
  if (!hasOled) Serial.println("# bench: no OLED at 0x3C (is the XIAO on the Expansion Board?)");
  if (!hasRtc) Serial.println("# bench: no real-time clock at 0x51");
  if (!hasSd) Serial.println("# bench: no microSD card; nothing will be recorded");
  if (!hasBme) Serial.println("# bench: no BME280 at 0x76 or 0x77; temperature and pressure off");
  if (!hasTsl) Serial.println("# bench: no TSL2561 light sensor; light off");
  if (!hasImu) Serial.println("# bench: no LSM6DS3 motion sensor at 0x6A or 0x6B");
  if (!USE_VOLTAGE_DIVIDER) Serial.println("# bench: voltage off until USE_VOLTAGE_DIVIDER is true");
#if CAMERA_SUPPORTED
  if (USE_CAMERA && !hasCamera) Serial.println("# bench: no camera (only the S3 Sense has one)");
#endif
  char clock[20];
  readRtc(clock, sizeof(clock));
  if (hasRtc && !clock[0]) Serial.println("# bench: the clock lost power; set it with TIME");
  help();

  mode = INHIBITED;
  modeSince = millis();
  Serial.println("# bench: mode=INHIBITED reason=in_deployer");
  buttonStable = buttonLastRaw = digitalRead(BUTTON_PIN);
  buttonChangedAt = millis();
  showScreen();
}

void loop() {
  unsigned long now = millis();
  watchDeploySwitch(now);
  readCommands();
  updateLed(now);

  static unsigned long lastScreen = 0;
  if (now - lastScreen >= 1000) {
    lastScreen = now;
    showScreen();
  }

  if (mode == INHIBITED) {
    if (now - lastBenchNote >= 10000) {
      lastBenchNote = now;
      Serial.println("# bench: INHIBITED. Press the Expansion Board button (or touch D1 to GND) to deploy.");
    }
    return;
  }
  if (mode == DEPLOYING) {
    if (now - modeSince >= DEPLOY_DELAY_MS) {
      setMode(NOMINAL, "deploy_timer_done");
      contactSince = now;
      beep(1320, 300);
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

  if (orbitUpMs) {
    if (inContact && now - contactSince >= orbitUpMs) loseContact(now);
    else if (!inContact && now - contactSince >= orbitDownMs) acquire(now);
  }

  if (now - lastSample >= periodMs) {
    lastSample = now;
    last = takeSample(now);
    if (mode == NOMINAL && SAFE_BELOW_MV > 0 && last.hasVolt && last.volt_mv < SAFE_BELOW_MV) setMode(SAFE, "voltage_low");
    logSample(last, inContact);
    if (inContact) {
      sendLive(last);
      ledFlash(60);
    } else {
      storeFrame(last);
      ledFlash(15);
    }
  }
  if (inContact && now - lastBeacon >= BEACON_MS) {
    lastBeacon = now;
    beacon();
  }
}
