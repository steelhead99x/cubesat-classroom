# CSC-R1P working-pedal prototype budget

Prepared November 27, 2026. All figures are USD. The estimate covers five enclosed receive-only 2 m and 70 cm pedals with power, antenna and line-level listening equipment. Each cell in the summary table shows the five-unit batch cost followed by the allocated cost per working pedal. The model assumes five accepted units from five pairs and an existing RF bench. It is a planning estimate. There is no signed turnkey assembly quote.

| Budget milestone | Low | Base | High |
| --- | ---: | ---: | ---: |
| Hardware, supplies and PCB vendor services | $1,392.68 / $278.54 | $2,035.13 / $407.03 | $3,246.63 / $649.33 |
| Five assembled and bench-tested pedals after design release | $1,880.18 / $376.04 | $3,097.63 / $619.53 | $5,446.63 / $1,089.33 |
| First run including remaining engineering | $2,920.18 / $584.04 | $5,817.63 / $1,163.53 | $12,486.63 / $2,497.33 |

The [cost worksheet](prototype-cost.csv) contains the estimates and totals in CSV format.

## Line items

| Item | Basis | Low | Base | High | Evidence or assumption |
| --- | --- | ---: | ---: | ---: | --- |
| 50 Ω main PCB and standard controls PCB | Five-unit batch | $69.58 | $69.58 | $69.58 | PCBWay via Flux. Two five-board lines combined. |
| Top-side SMT setup and placement | Five-unit batch | $70.20 | $70.20 | $70.20 | PCBWay standard-board estimate, carried over. Confirm setup, stencil, inspection and 50 Ω build together. |
| Eight SMT IC placements | Per pedal | $61.88 | $61.88 | $61.88 | Distributor anchors per set. PLL, demodulator, audio ADC/DAC, trim DAC, two I/O expanders and pot ADC. |
| Remaining specified SMT | Per pedal | $21.00 | $32.00 | $53.00 | Allowance per set for both oscillators, RF front end, U.FL connectors, power parts and sourcing overage. |
| Blank-MPN SMT groups | Per pedal | $8.00 | $16.00 | $30.00 | Allowance per set for 191 placements across 67 source BOM groups, including RF-grade parts and cut-tape minimums. |
| Thirteen through-hole board parts | Per pedal | $9.00 | $14.00 | $21.00 | Headers, encoder, two pots and LED. Hand-fit labor is below. |
| Non-Sense Seeed XIAO ESP32-S3 | Per pedal | $7.49 | $7.49 | $7.49 | 113991114 single-unit retail. |
| Hammond 1590N1 enclosure | Per pedal | $14.21 | $14.21 | $22.00 | Five-unit distributor tier rounded to cents. Upper scenario allows for alternate supply. |
| 128×32 OLED module | Per pedal | $6.00 | $12.50 | $17.00 | Fit and pinout pending. Retail anchor is for a comparable module. |
| Three knobs, footswitch, toggle and LED bezel | Per pedal | $8.00 | $15.00 | $28.00 | Panel BOM allowance. The board-mounted pots and encoder are covered above. |
| Chassis DC and audio jacks | Per pedal | $4.00 | $8.00 | $14.00 | Separate from the through-hole wire headers. |
| Two RF pigtails, USB-C data extender and harness leads | Per pedal | $12.00 | $21.00 | $38.00 | SMA/U.FL and XIAO antenna leads, four JST-PH leads and a USB data extension. |
| Standoffs, screws, wire and consumables | Per pedal | $5.00 | $9.00 | $15.00 | Allowance for mechanical assembly and wiring. |
| 9 V regulated center-negative supply | Per pedal | $15.00 | $27.99 | $34.00 | Retail price anchor. Finish input protection review before selecting a supply for bring-up. |
| Basic dual-band receive antenna and feed lead | Per pedal | $23.00 | $33.00 | $55.00 | Antenna anchor $22.95. Match SMA polarity before ordering. Budget for a short lead or adapter. |
| Powered line-input speaker, power and audio lead | Per pedal | $30.00 | $50.00 | $105.00 | Retail class allowance. Pedal outputs line-level audio. |
| Domestic and international freight | Per pedal | $8.00 | $15.00 | $30.00 | Allocated from a five-unit order. Confirm shipping destination and vendor terms. |
| Sales tax, duty and fees | Per pedal | $10.00 | $22.00 | $45.00 | Budget only. Actual amount depends on destination and fulfillment. |
| Spares and incremental setup reserve | Per pedal | $8.00 | $20.00 | $45.00 | Allowance for sourcing attrition, minor rework or quote extras. A fresh board spin is separate. |
| Hand fit, enclosure work, flash and unit acceptance | Per pedal | $97.50 | $212.50 | $440.00 | 1.5 h × $65, 2.5 h × $85, or 4 h × $110 per working unit. |
| Remaining design, Rust firmware and prototype validation | Five-unit batch | $1,040.00 | $2,720.00 | $7,040.00 | One-time scenario allowance. 16 h × $65, 32 h × $85, or 64 h × $110. |

## Verified quote scope

- PCBWay's Flux results showed $64.58 for five controlled-impedance main PCBs and $5.00 for five standard controls PCBs. The $70.20 SMT setup and placement subtotal was estimated for the standard board configuration. Ask the assembler to confirm the controlled-impedance main board and the specified stackup together with parts procurement, stencil, setup and inspection. The same quote thread showed $80.20 for five standard PCBWay PCB pairs with SMT service and $55.97 for five standard JLCPCB pairs with SMT service. Those service comparisons exclude components and do not establish equivalent 50 Ω builds.
- Flux's follow-up estimated $190 / $286 / $465 per unit for its scope. Its base $58 SMT allowance covers approximately the distributor subtotal of eight fitted ICs alone. The model above uses separate allowances for the remaining SMT, increases the labor estimate, and includes the listening setup and reserves for tax, duty and minor rework. Engineering is shown separately.
- Quote from the KiCad board files at revision `bd4be2a1367ff2c4fbb7cb82e8305efa2375e875`. The estimate used KiCad CAM, pick-and-place files and assembly BOMs for both boards. Main has 214 SMT placements and controls has 22. The 13 through-hole components are budgeted for hand fit, including the underside control-board socket. Keep TP3 out of automated placement. Flux export comparison found drill, paste-aperture and copper differences. Use the KiCad package for quoting and review the production exports independently before release.
- Once the design is released, the paid unit labor allowance covers hand soldering, drilling, enclosure wiring, flashing and acceptance tests. The one-time engineering allowance covers remaining parts selection, power/stackup/mechanical review, Rust firmware and first-article verification. Hours and rates are scenario inputs, with a published prototyping rate range used as an anchor. Actual engineering may exceed the range if the first article exposes a board or firmware issue. A full re-spin, new RF instruments and product certification are outside these totals. At base rates a repeat purchase of the same hardware and PCB services would add about $2,035 before engineering or retest labor. [Published US prototyping rates](https://www.upwork.com/hire/hw-prototyping-freelancers/us/)

## Price anchors used

| Placement or part | USD | Source |
| --- | ---: | --- |
| ADF4351BCPZ synthesizer, 1 | 25.16 | [DigiKey](https://www.digikey.com/en/products/detail/analog-devices-inc/ADF4351BCPZ/3340515) |
| LTC5584IUF#PBF I/Q demodulator, 1 | 19.31 | [DigiKey listing with direct substitute](https://www.digikey.com/en/products/detail/analog-devices-inc/LTC5584IUF-TRPBF/3175333) |
| TLV320ADC3140IRTWR audio ADC, 1 | 3.86 | [DigiKey](https://www.digikey.com/en/products/detail/texas-instruments/TLV320ADC3140IRTWR/11308863) |
| PCM5102APWR audio DAC, 1 | 3.81 | [Mouser](https://www.mouser.com/en/ProductDetail/Texas-Instruments/PCM5102APWR?qs=E2%2FxqS9xjzrfECkwEYoiyg%3D%3D) |
| MCP4728-E/UN trim DAC, 1 | 2.50 | [DigiKey](https://www.digikey.com/en/products/detail/microchip-technology/MCP4728-E-UN/2125526) |
| TCA9534PWR I/O expanders, 2 at $1.53 | 3.06 | [DigiKey](https://www.digikey.com/en/products/detail/texas-instruments/TCA9534PWR/5178451) |
| ADS1015IDGSR pot ADC, 1 | 4.18 | [DigiKey](https://www.digikey.com/en/product-highlight/t/texas-instruments/ads1013-4-5-12-bit-adc) |
| **Eight IC placements** | **61.88** | |
| Seeed XIAO ESP32-S3, non-Sense | 7.49 | [Seeed](https://www.seeedstudio.com/XIAO-ESP32S3-p-5627.html) |
| Hammond 1590N1, five-unit tier | 14.21 | [DigiKey](https://www.digikey.com/en/products/detail/hammond-manufacturing/1590N1/409860) |
| Comparable 128×32 OLED for pricing | 12.50 | [Adafruit](https://www.adafruit.com/product/4440) |
| 9 V regulated center-negative adapter | 27.99 | [Sweetwater](https://www.sweetwater.com/store/detail/1Spot9v--truetone-1-spot-slim-9v-dc-adapter) |
| Example 2 m/70 cm SMA antenna | 22.95 | [Ham Radio Outlet](https://www.hamradio.com/detail.cfm?pid=H0-006297) |

Public prices are small-quantity anchors. Have the assembler confirm its cut-tape minimums, inventory and the prices it can actually secure. The comparable OLED and example antenna still need fit and connector checks before selection.

## Release and acceptance

1. Approve the board stackup and released CAM using the routed KiCad files. Finish the input protection decision and verify enclosure clearances with both boards and panel parts installed.
2. Close 67 blank SMT MPN groups, check the descriptive part rows and confirm the supplier's stocked 25 MHz Y2 oscillator with the full suffix and the required output and pinout. Approve physical part substitutions on both boards. Ask for an itemized combined quote with the delivery postal code, sourcing minimums, stencil, inspection, setup, freight, duty and tax.
3. Finish Rust external-clock I²S capture and output, receive audio with fault mute, live tuning and panel behavior, and telemetry. The current USB bring-up console can test the controls, PLL and ADC clocks. Live audio is still pending.
4. Power a first article on a current-limited supply, measure the rails and verify the RF path, both bands, I²S clocks, usable audio, the panel controls and startup and fault mute. Record hardware and firmware revisions and repeat the acceptance checks on each unit. A host test or target build does not count as hardware acceptance.

The per-pedal figures allocate minimum-order and development costs over five accepted units. A standalone one-unit order needs its own assembly quote. If only four units pass and the batch cost stays at the base level, the first-run cost is about $1,454 per accepted unit before any further repair or remake. No fabrication or purchase order was placed.
