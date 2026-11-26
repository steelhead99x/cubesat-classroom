// XIAO MG24 Sense, Seeed Arduino LSM6DS3 library. Values are milligravity.
#include <LSM6DS3.h>
#include <Wire.h>

LSM6DS3 imu(I2C_MODE, 0x6A);
uint32_t sequence = 0;

void setup() {
  Serial1.begin(115200);
  pinMode(PD5, OUTPUT);
  digitalWrite(PD5, HIGH);
  delay(300);
  while (imu.begin() != 0) delay(1000);
}

void loop() {
  Serial1.print(sequence++);
  Serial1.print(',');
  Serial1.print(millis());
  Serial1.print(',');
  Serial1.print(lroundf(1000 * imu.readFloatAccelX()));
  Serial1.print(',');
  Serial1.print(lroundf(1000 * imu.readFloatAccelY()));
  Serial1.print(',');
  Serial1.println(lroundf(1000 * imu.readFloatAccelZ()));
  delay(100);
}
