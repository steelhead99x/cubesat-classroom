# CSC-R1P prototype parts outside the SMT build

Check these parts against the [panel drawing](face-template.svg) and the real box before releasing a fabrication order. Counts are per pedal. The main and control PCB BOMs are separate from this list.

| Qty | Part | Selection and fit check |
| --- | --- | --- |
| 1 | Enclosure | Hammond 1590N1. Verify inside height with the lid installed. |
| 1 | Controller | Seeed XIAO ESP32-S3 (non-Sense). Check both socket rows, USB plug depth and antenna connector on the actual module. |
| 1 | DC jack | Isolated 5.5 × 2.1 mm panel barrel jack. Centre is ground, sleeve is +9 V. With the jack mounted and power disconnected, verify the positive terminal is isolated from the enclosure. |
| 1 | Audio jack | ¼ inch mono panel jack. Tip is audio, sleeve is ground. Check clearance beside the main board and with a plug inserted. |
| 1 | Receive antenna lead | 50 Ω SMA female bulkhead to U.FL pigtail. Check the body and cable bend against the main-board edge. |
| 1 | XIAO antenna lead | U.FL to RP-SMA bulkhead. Confirm the mating connector and antenna before drilling. |
| 1 | USB extension | USB-C panel extension with data. Check the connector depth and cable bend with the boards stacked. |
| 1 | ENGAGE footswitch | Soft-touch momentary SPST. Verify body clears the 16 mm board slot through its full motion. |
| 1 | BAND toggle | Mini SPST or SPDT, wired closed for 70 cm. Check bushing and nut clearance. |
| 1 | LED bezel | Panel bezel for the 3 mm red LED on the control-board BOM. |
| 3 | Knobs | Match GAIN and LEVEL pot shafts and the TUNE encoder shaft. Check the TUNE knob can be pressed without touching the face. |
| 4 | Panel leads | Two-pin JST-PH mating leads for DC, audio, footswitch and toggle. Verify pin 1 and continuity end to end. |
| 3 | Standoffs | 15 mm M2.5 with matching screws. Confirm stack height and the final engagement depth of the 2×4 header/socket pair. |
| 1 | OLED | 0.91 inch SSD1306 128×32 module. Verify GND, VCC, SCL and SDA in that order and check the window against the purchased module. |

## PCB BOM override

Use Epson **X1G005421031327** as the first-prototype candidate for main board Y2. It is the 25 MHz TG2520SMN TCXO for the 2.8–3.63 V supply range. The checked-in pads match Epson's land pattern after rotation and connect 1 open, 2 ground, 3 output and 4 supply. The candidate's specified minimum output is 0.8 V peak to peak under the datasheet load. The ADF4351 specifies 0.7 V peak to peak minimum at REFIN. Measure at the PLL input with a low-capacitance probe and confirm lock on both bands. Confirm supply and lead time with the distributor before accepting substitutions. [Epson ordering table](https://download.epsondevice.com/td/pdf/td_tcxo_mhz/TG2520SMN_X1G0054210220_en.pdf) · [Epson footprint and electrical specifications](https://download.epsondevice.com/td/pdf/app/TG2520SMN_en.pdf) · [Analog Devices ADF4351 datasheet](https://www.analog.com/media/en/technical-documentation/data-sheets/ADF4351.pdf)
