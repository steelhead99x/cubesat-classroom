// XIAO ESP32S3 Sense with its Sense expansion board and microSD.
#include <SD.h>
#include <esp_system.h>

constexpr int NODE = 1;  // Assign a different number (1..5) on each recorder.
static_assert(NODE >= 1 && NODE <= 5);
File logFile;
char line[80];
size_t used = 0;
bool discard = false;
uint32_t boot;
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
    boot = esp_random();
    snprintf(path, sizeof(path), "/N%d-%08lx.csv", NODE, (unsigned long)boot);
  } while (SD.exists(path));
  logFile = SD.open(path, FILE_WRITE);
  if (!logFile) stopRecording();
}

void setup() {
  Serial.begin(115200);
  Serial1.begin(115200, SERIAL_8N1, D7, D6);
  if (!SD.begin(21)) stopRecording();
  openRun();
}

void recordLine() {
  unsigned long seq, ms;
  long x, y, z;
  char extra;
  if (sscanf(line, "%lu,%lu,%ld,%ld,%ld%c", &seq, &ms, &x, &y, &z, &extra) != 5) return;
  // A restarted sensor begins a new acquisition. Boot ID + sequence stays unique.
  if (havePrevious && seq <= previous) {
    logFile.close();
    openRun();
  }
  havePrevious = true;
  previous = seq;
  char row[128];
  int len = snprintf(row, sizeof(row), "hardware bench,%d,%08lx,%lu,%lu,%ld,%ld,%ld\n",
                     NODE, (unsigned long)boot, seq, ms, x, y, z);
  if (logFile.write((const uint8_t*)row, len) != (size_t)len) stopRecording();
  logFile.flush();
  Serial.print(row);
}

void loop() {
  while (Serial1.available()) {
    char c = Serial1.read();
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
