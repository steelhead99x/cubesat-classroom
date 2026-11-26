// Classroom sensors for the CubeSat Classroom ground station.
//
// Prints one line per second over USB serial at 115200 baud (with USE_TMP36 and
// USE_BATTERY turned on below):
//
//   CSC1 temp_mc=21500 volt_mv=3912 light_lux=460
//
// The station's `sensor-bridge` command reads these lines and writes the
// files that `classroom --sensor-dir` publishes. Every sensor is optional:
// turn one off below and its field is simply left out.
//
// Written for: Arduino Uno R3 and Nano (ATmega328P), and Seeed XIAO SAMD21,
// RP2040, ESP32-C3 and ESP32-S3. Other boards stop at compile time with a
// message, so a wrong voltage scale never produces a believable number.
//
// Wiring (a mentor checks it before power is connected):
//   TMP36 temperature sensor: +V to 3V3 (or 5V on Uno), GND, Vout to A0
//   Battery voltage: battery+ -> 100k -> A1 -> 100k -> GND, plus 100 nF from
//     A1 to GND. The divider halves the voltage, so the most it can measure is
//     twice the board's reference: 10 V on an Uno, 6.6 V on a 3.3 V board.
//     Never connect a higher voltage than that.
//   BH1750 light sensor: VCC to 3V3, GND, SDA, SCL (address 0x23).
//     Uno and Nano SDA/SCL are 5 V. Use a 5 V-tolerant module or a level
//     shifter. A 3.3 V XIAO can connect SDA and SCL directly. A mentor checks.
//
// Status: compiles for the boards named above (arduino-cli, 24 Nov 2026).
// Not yet run on hardware. Check readings against a thermometer and a meter,
// and label them "hardware bench".

#include <Wire.h>

// The analog sensors start OFF. A pin with nothing connected still reads a believable
// voltage, so the sketch would print a made-up temperature or battery voltage as if it
// were measured. Set each one to true only after it is wired and a mentor has checked it.
// The BH1750 is found on I2C at start-up, so a missing one is simply left out.
const bool USE_TMP36 = false;
const bool USE_BATTERY = false;
const bool USE_BH1750 = true;

const int TMP36_PIN = A0;
const int BATTERY_PIN = A1;
const float BATTERY_DIVIDER = 2.0;   // 100k over 100k
const uint8_t BH1750_ADDR = 0x23;
const unsigned long PERIOD_MS = 1000;

bool lightReady = false;

// Millivolts at an analog pin, for each board family. The first read after
// switching pins is thrown away, because the ADC's sampling capacitor still
// holds charge from the previous pin.
long readMillivolts(int pin) {
#if defined(ARDUINO_ARCH_ESP32)
  analogReadMilliVolts(pin);
  return analogReadMilliVolts(pin);
#elif defined(ARDUINO_ARCH_AVR)
  // Uno R3 and Nano: 5 V reference, 10-bit reads.
  analogRead(pin);
  return (long)analogRead(pin) * 5000L / 1023L;
#elif defined(ARDUINO_ARCH_SAMD) || defined(ARDUINO_ARCH_RP2040) || defined(ARDUINO_ARCH_MBED_RP2040)
  // XIAO SAMD21 and XIAO RP2040: 3.3 V reference, 12-bit reads.
  analogReadResolution(12);
  analogRead(pin);
  return (long)analogRead(pin) * 3300L / 4095L;
#else
#error "Add the ADC reference and resolution for this board to readMillivolts()."
#endif
}

bool startLight() {
  Wire.beginTransmission(BH1750_ADDR);
  Wire.write(0x10);  // continuous high-resolution mode, 1 lux steps
  return Wire.endTransmission() == 0;
}

// Returns lux, or -1 if the sensor did not answer.
long readLux() {
  // uint8_t sizes avoid an ambiguous requestFrom() on ESP32 core 3.
  if (Wire.requestFrom(BH1750_ADDR, (uint8_t)2) != 2) return -1;
  uint16_t hi = Wire.read();  // read high byte first, as two statements
  uint16_t lo = Wire.read();
  uint16_t raw = (hi << 8) | lo;
  return (long)(raw / 1.2);
}

void setup() {
  Serial.begin(115200);
  if (USE_BH1750) {
    Wire.begin();
    lightReady = startLight();
    // High-resolution mode needs about 180 ms before the first sample is real.
    if (lightReady) delay(180);
  }
}

void loop() {
  static unsigned long last = 0;
  if (millis() - last < PERIOD_MS) return;
  last = millis();

  Serial.print("CSC1");
  if (USE_TMP36) {
    // TMP36: 500 mV at 0 °C, 10 mV per °C, so milli-degrees = (mV - 500) * 100.
    long mv = readMillivolts(TMP36_PIN);
    Serial.print(" temp_mc=");
    Serial.print((mv - 500L) * 100L);
  }
  if (USE_BATTERY) {
    Serial.print(" volt_mv=");
    Serial.print((long)(readMillivolts(BATTERY_PIN) * BATTERY_DIVIDER));
  }
  if (USE_BH1750) {
    if (!lightReady) lightReady = startLight();
    long lux = lightReady ? readLux() : -1;
    if (lux >= 0) {
      Serial.print(" light_lux=");
      Serial.print(lux);
    } else {
      lightReady = false;
    }
  }
  Serial.println();
}
