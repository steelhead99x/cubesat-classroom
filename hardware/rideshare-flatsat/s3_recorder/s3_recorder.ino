// XIAO ESP32S3 Sense with its Sense expansion board and microSD.
#include <SD.h>
#include <esp_system.h>

constexpr int NODE = 1;  // Assign a different number (1..5) on each recorder.
static_assert(NODE >= 1 && NODE <= 5, "NODE must be 1..5");
File logFile;
char line[80];
size_t used = 0;
// Drop the first line: the recorder may start partway through a sensor line.
bool discard = true;
uint32_t session;  // New random ID for each log file.
unsigned long previous;
bool havePrevious = false;

void stopRecording() {
  for (;;) {
    Serial.println("# SD error");
    delay(1000);
  }
}

void openRun() {
  char path[32];
  do {
    session = esp_random();
    snprintf(path, sizeof(path), "/N%d-%08lx.csv", NODE, (unsigned long)session);
  } while (SD.exists(path));
  logFile = SD.open(path, FILE_WRITE);
  if (!logFile) stopRecording();
}

void setup() {
  Serial.begin(115200);
  // A slow SD flush can take tens of milliseconds; keep about a second of sensor lines.
  Serial1.setRxBufferSize(2048);
  Serial1.begin(115200, SERIAL_8N1, D7, D6);
  if (!SD.begin(21)) stopRecording();
  openRun();
}

void recordLine() {
  unsigned long seq, ms;
  long x, y, z;
  char extra;
  // Accept only what the sensor sketch prints: digits for seq and ms, and accelerations
  // within the sensor's range (±16 g), so a corrupted line is not saved as evidence.
  if (line[0] < '0' || line[0] > '9') return;
  if (sscanf(line, "%lu,%lu,%ld,%ld,%ld%c", &seq, &ms, &x, &y, &z, &extra) != 5) return;
  if (labs(x) > 16000 || labs(y) > 16000 || labs(z) > 16000) return;
  // A restarted sensor begins a new acquisition. Session ID + sequence stays unique.
  if (havePrevious && seq <= previous) {
    logFile.close();
    openRun();
  }
  havePrevious = true;
  previous = seq;
  char row[128];
  int len = snprintf(row, sizeof(row), "hardware bench,%d,%08lx,%lu,%lu,%ld,%ld,%ld\n",
                     NODE, (unsigned long)session, seq, ms, x, y, z);
  if (logFile.write((const uint8_t*)row, len) != (size_t)len) stopRecording();
  logFile.flush();
  if (Serial) Serial.print(row);  // only with a USB host listening, so writes never stall
}

unsigned long lastByte = 0;
bool silent = false;

void loop() {
  // Say so once if the sensor goes quiet, so a dead sensor is not mistaken for a short run.
  if (!silent && millis() - lastByte > 2000) {
    silent = true;
    if (Serial) Serial.println("# no sensor data");
  }
  while (Serial1.available()) {
    char c = Serial1.read();
    lastByte = millis();
    silent = false;
    if (c == '\n') {
      line[used] = 0;
      if (!discard) recordLine();
      discard = false;
      used = 0;
    } else if (c != '\r') {
      if (used < sizeof(line) - 1) line[used++] = c;
      else discard = true;
    }
  }
}
