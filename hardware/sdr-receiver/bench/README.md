# CSC-R1 bench kit

Scripts and a logger sketch for the receiver bench quests. They're built around cheap gear: an RTL-SDR Blog V4, a noise source, an ADF4351 module, attenuators and an INA226 breakout. A tinySA, a VNA or a Nordic PPK2 makes some steps easier, but none is required.

**Every result these tools make is "hardware bench" (or "received RF" for a real pass).** RTL-SDR levels are relative dB, not dBm. Compare only sweeps taken with the same dongle, gain and frequencies.

## The quests

Start with the first one. The Rev A quests need a built kit.

| Quest | Needs a Rev A kit? | Main gear |
| --- | --- | --- |
| [First light with the RTL-SDR Blog V4](/quest/m0/receiver-v4-first-light.md) | no | V4, 50 Ω load, antenna |
| [Measure FM broadcast at your antenna](/quest/m0/receiver-fm-at-your-site.md) | no | V4, antenna |
| [Build a shielded test signal](/quest/m0/receiver-test-signal.md) | no | ADF4351 module, attenuators, tin |
| [Sweep a filter with a noise source](/quest/m0/receiver-sweep-a-stub.md) | no | noise source, coax stub |
| [Does HaLow noise land in the receive bands?](/quest/m0/receiver-halow-noise.md) | no | two Seeed HaLow for XIAO boards |
| [Compare radio power: 2.4 GHz, 5 GHz and HaLow](/quest/m0/receiver-radio-power.md) | no | INA226 or PPK2, XIAO S3 and C5 |
| [Bring up CSC-R1 power and control](/quest/m1/receiver-rev-a-power-up.md) | yes | multimeter, current-limited 5 V |
| [Write the CSC-R1 Rev A bring-up firmware](/quest/m1/receiver-rev-a-firmware.md) | yes | XIAO ESP32-S3, Rust |
| [Check the LO and 5 V under load](/quest/m1/receiver-rev-a-lo-and-supply.md) | yes | V4, attenuators |
| [Measure the CSC-R1 band filters](/quest/m1/receiver-rev-a-filters.md) | yes | noise source, U.FL cables |
| [Measure HaLow desense on the CSC-R1](/quest/m1/receiver-rev-a-desense.md) | yes | test signal, HaLow pair |
| [Compare the CSC-R1 with the V4 on a real pass](/quest/m1/receiver-rev-a-vs-v4.md) | yes | splitter, antenna |

## Shopping list

Prices are estimates. Check before you buy.

| Item | About | Used in |
| --- | --- | --- |
| RTL-SDR Blog V4 | $30–40 | every quest |
| 50 Ω SMA load; SMA attenuators 10, 20 and 3 × 30 dB; SMA tee; cables | $40–60 | most |
| Broadband noise source module | $15–30 | filter quests |
| ADF4351 module with its own controller | $30–60 | test signal, desense, comparison |
| Metal tin with a lid | free | test signal |
| INA226 breakout, 0.1 Ω shunt | $3–10 | power |
| 2 × Seeed Wi-Fi HaLow for XIAO, plus XIAO ESP32-S3 and ESP32-C5 | check Seeed | HaLow and power |
| Two-way 50 Ω splitter | $10–30 | comparison |
| Optional: AD8318 power detector, return-loss bridge | $15–30 each | LO level, matching |

## Install

- **RTL-SDR Blog drivers.** The V4 needs them. Most current SDR apps already include them. For `rtl_power`, `rtl_test` and `rtl_biast`, follow the maker's [V4 users guide](https://www.rtl-sdr.com/v4). On Linux, build `rtlsdrblog/rtl-sdr-blog` from GitHub and blacklist the DVB-T kernel driver as the guide explains.
- **Python 3.** `sweep.py` needs nothing else. `power_log.py record` needs `pip install pyserial`.
- **Self-test, no hardware needed:** `python3 test_bench.py`.

## Tools

| File | Does |
| --- | --- |
| `sweep.py record` | Runs `rtl_power` at a fixed gain with the bias tee off. Saves the CSV plus a `.json` label (what was connected, gain, time, evidence type). |
| `sweep.py summary` | Noise floor (median) and the strongest signal |
| `sweep.py compare` | Noise-floor change from one sweep to another, such as HaLow off vs on |
| `sweep.py response` | Filter response: noise through the device minus noise straight through |
| `sweep.py stub` | Quarter-wave coax stub length, or velocity factor from a measured stub |
| `power-logger/power-logger.ino` | Reads an INA226 and prints raw readings over USB serial |
| `power_log.py record` / `summary` | Logs the sketch's output, then gives average mW, energy and mJ per MB. Also reads Nordic PPK2 CSV exports. |

## Safety

- A mentor checks wiring before anything is powered.
- The bias tee stays off unless a powered preamplifier is fitted. Loads, stubs, filters and noise sources must not get DC.
- Never connect a transmitter output to a dongle or the kit. That includes HaLow boards, radios and an ADF4351 without attenuators.
- The ADF4351 test signal stays in its closed tin, into attenuators and a cable, never an antenna. On 145 or 437 MHz a licensed operator approves the frequencies, and the crew follows their lead.
