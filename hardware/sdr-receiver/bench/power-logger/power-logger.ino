// Power logger for the CSC-R1 bench quests: one INA226 breakout, read by a
// second board (any XIAO or Arduino with I2C), printed as CSV over USB serial.
//
// Wiring: INA226 VCC to 3.3 V, GND to GND, SDA/SCL to the board's I2C pins.
// The radio under test is powered THROUGH the INA226's shunt (IN+ from the
// supply, IN- to the radio). Address 0x40 is A0 and A1 tied to GND.
//
// Output, one line about every 50 ms:  t_ms,shunt_raw,bus_raw
//   shunt_raw is signed, 2.5 uV per count; bus_raw is 1.25 mV per count.
// power_log.py turns these into mA, V and mW using your shunt's value
// (most breakouts use 0.1 ohm: up to about 0.8 A).
//
// Register values are from TI's INA226 datasheet.

#include <Wire.h>

const uint8_t INA = 0x40;
const uint32_t PERIOD_MS = 50;

// Config: averaging 16, bus and shunt conversion 1.1 ms each, continuous.
// About 35 ms per averaged reading. Short radio bursts are averaged, not seen.
const uint16_t CONFIG = 0x4527;

bool writeReg(uint8_t reg, uint16_t v) {
  Wire.beginTransmission(INA);
  Wire.write(reg);
  Wire.write(v >> 8);
  Wire.write(v & 0xFF);
  return Wire.endTransmission() == 0;
}

bool readReg(uint8_t reg, uint16_t &v) {
  Wire.beginTransmission(INA);
  Wire.write(reg);
  if (Wire.endTransmission(false) != 0) return false;
  if (Wire.requestFrom(INA, (uint8_t)2) != 2) return false;
  v = (uint16_t)Wire.read() << 8;
  v |= Wire.read();
  return true;
}

void setup() {
  Serial.begin(115200);
  while (!Serial && millis() < 3000) {}
  Wire.begin();
  uint16_t id = 0;
  if (!readReg(0xFE, id) || id != 0x5449) {  // manufacturer ID "TI"
    Serial.println("# INA226 not found at 0x40: check wiring and address pins");
  }
  writeReg(0x00, CONFIG);
  Serial.println("# t_ms,shunt_raw,bus_raw");
}

void loop() {
  static uint32_t next = 0;
  uint32_t now = millis();
  if ((int32_t)(now - next) < 0) return;
  next = now + PERIOD_MS;
  uint16_t sh, bus;
  if (readReg(0x01, sh) && readReg(0x02, bus)) {
    Serial.print(now);
    Serial.print(',');
    Serial.print((int16_t)sh);
    Serial.print(',');
    Serial.println(bus);
  } else {
    Serial.println("# read failed");
  }
}
