/* Arduino Nano 33 BLE Sense (nRF52840). The Arduino bootloader occupies the first
   64 KB of flash and starts the application at 0x10000, so this firmware is linked
   there and the bootloader stays in place: double-tap reset to return to it. */
MEMORY
{
  FLASH : ORIGIN = 0x00010000, LENGTH = 0xF0000
  RAM   : ORIGIN = 0x20000000, LENGTH = 256K
}
