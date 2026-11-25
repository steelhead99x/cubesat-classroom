#!/usr/bin/env python3
"""Battery energy budgets for the ground station and balloon payloads.

Superseded by the Rust port: run `cargo run --locked -- energy-budget`. The figures
now live in crates/mission-budget/src/energy.rs; edit them there. This copy stays only
until CI's `py_compile tools/*.py` step is updated, then it can be deleted.

Run: python3 tools/energy_budget.py

Every power figure below names its source in docs/ground-station.md or
docs/balloon-lab.md. Figures marked "estimate" have no measurement behind
them yet: replace them with your own bench readings (a USB power meter is
enough) and rerun. Budgets are planning numbers, not measured results.
"""

from __future__ import annotations

# One Energizer L91 lithium AA: 3500 mAh at 100 mA, 3400 mAh at 1000 mA
# (Energizer datasheet). Use 3000 mAh at about 1.4 V under load to leave room.
#
# Pack layout: four cells in series give about 6 V fresh, falling as they drain.
# A buck-boost regulator turns that into a steady 3.3 V (payload A) or 5 V
# (payload B). Eight cells are two strings of four in parallel ("4s2p"),
# so the voltage stays the same and the capacity doubles.
AA_LITHIUM_WH = 3.0 * 1.4
REGULATOR_EFFICIENCY = 0.85   # estimate for a small buck-boost regulator
COLD_DERATE = 0.75            # estimate: keep 25% in reserve for cold and ageing
MARGIN = 2.0                  # plan for twice the expected run time
HOURS = 6                     # preparation, about 3 h of flight, recovery


def usable_wh(cells: int) -> float:
    return cells * AA_LITHIUM_WH * REGULATOR_EFFICIENCY * COLD_DERATE


def report(title: str, loads: dict[str, float], hours: float, cells: int | None = None) -> None:
    watts = sum(loads.values())
    need = watts * hours * MARGIN
    print(f"\n{title}")
    for name, w in loads.items():
        print(f"  {name:<54} {w:5.2f} W")
    print(f"  {'Total':<54} {watts:5.2f} W")
    print(f"  Energy for {hours:g} h with {MARGIN:g}x margin: {need:.1f} Wh")
    if cells:
        have = usable_wh(cells)
        print(f"  {cells} lithium AA after derating: {have:.1f} Wh, about {have / watts:.0f} h of run time")
        print("  OK" if have >= need else "  NOT ENOUGH: add cells or cut the load")


RTL_SDR_W = 0.27 * 5  # RTL-SDR Blog V4: 250-270 mA at 5 V

report(
    "Ground station: Pi 4 + RTL-SDR, decoding at full CPU load",
    {"Raspberry Pi 4, full CPU load (pidramble 6.4 W)": 6.4, "RTL-SDR Blog V4": RTL_SDR_W},
    hours=HOURS,
)
print(f"  A 12 V 20 Ah battery stores {12 * 20} Wh before converter losses.")

report(
    "Ground station: Pi Zero 2 W + RTL-SDR (runs this project's station, not SatNOGS)",
    {"Raspberry Pi Zero 2 W (0.6-1.2 W, use top)": 1.2, "RTL-SDR Blog V4": RTL_SDR_W},
    hours=HOURS,
)

report(
    "Classroom sensors: Arduino Uno (about 40 mA at 5 V) from USB",
    {"Arduino Uno with TMP36 and BH1750": 0.040 * 5},
    hours=HOURS,
)

light = {
    "XIAO ESP32-C3, Wi-Fi off (25 mA modem-sleep at 3.3 V)": 0.025 * 3.3,
    "u-blox MAX-M10S GPS (under 25 mW tracking)": 0.025,
    "BME280 pressure + temperature (estimate)": 0.003,
    "microSD writes, average (estimate)": 0.020 * 3.3,
}
report("Balloon payload A: XIAO logger with GPS", light, hours=HOURS, cells=4)

camera = {
    "Raspberry Pi Zero 2 W (0.6-1.2 W, use top)": 1.2,
    "Camera module (estimate)": 0.3,
}
report("Balloon payload B: Pi Zero 2 W camera logger", camera, hours=HOURS, cells=4)
report("Balloon payload B with eight cells", camera, hours=HOURS, cells=8)
