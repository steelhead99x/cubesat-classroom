"""Ground tests for one rideshare FlatSat node on solar power. Uses only the Python standard library.

    python3 ground.py simulate run-1                 # desk simulation of a light/dark run
    python3 ground.py report run-1/power.csv         # charge in and out, per light and dark period

power.csv is the INA226 logger's format (hardware/sdr-receiver/bench/power_log.py record):
t_ms,current_ma,bus_v,power_mw, with the INA226 in the cell's lead, wired so charging reads positive.
"""
import argparse
import csv
import json
from pathlib import Path

# 1U solar faces, maximum-power point at rated sunlight (hardware/solar-face/gen/cells.py; desk figures).
FACES = {
    "si": dict(vmp=3 * 1.67, imp=8 * 0.0184, rated_w_m2=1000.0),
    "tj": dict(vmp=2 * 2.409, imp=0.5029, rated_w_m2=1367.0),
}


def charge_ma(face, sun_w_m2, charger_eff=0.8, cell_v=3.8):
    """Current into the cell from the face while lit, mA (desk estimate, straight-on light)."""
    f = FACES[face]
    return f["vmp"] * f["imp"] * (sun_w_m2 / f["rated_w_m2"]) * charger_eff / cell_v * 1000


def cell_v(soc):
    """A rough LiPo voltage for a state of charge 0..1 (desk estimate, not a real discharge curve)."""
    return 3.3 + 0.9 * min(max(soc, 0.0), 1.0)


def simulate(directory, face="si", sun_w_m2=800.0, use_ma=60.0, cell_mah=1000.0,
             lit_min=45, dark_min=45, cycles=4, start_soc=0.5, step_s=10):
    """Write power.csv: the node draws use_ma all the time; the face adds charge while lit."""
    directory.mkdir(parents=True, exist_ok=False)
    soc, t = start_soc, 0
    period = (lit_min + dark_min) * 60
    with (directory / "power.csv").open("w", newline="") as out:
        w = csv.writer(out)
        w.writerow(["t_ms", "current_ma", "bus_v", "power_mw"])
        while t < cycles * period:
            lit = (t % period) < lit_min * 60
            i = (charge_ma(face, sun_w_m2) if lit else 0.0) - use_ma
            if soc >= 1.0 and i > 0:
                i = 0.0                      # a full cell stops charging
            v = cell_v(soc)
            w.writerow([t * 1000, f"{i:.3f}", f"{v:.4f}", f"{i * v:.2f}"])
            soc += i * step_s / 3600 / cell_mah
            t += step_s
    info = dict(evidence="desk simulation", face=face, sun_w_m2=sun_w_m2, use_ma=use_ma, cell_mah=cell_mah,
                lit_min=lit_min, dark_min=dark_min, cycles=cycles)
    (directory / "power.csv.json").write_text(json.dumps(info, indent=2) + "\n")


def load(path):
    with open(path, newline="") as f:
        rows = list(csv.DictReader(f))
    if not rows or "current_ma" not in rows[0]:
        raise SystemExit(f"{path}: expected the INA226 logger's columns t_ms,current_ma,bus_v,power_mw")
    return [(int(r["t_ms"]) / 1000, float(r["current_ma"]), float(r["bus_v"])) for r in rows]


def periods(samples, threshold_ma=5.0):
    """Split into charging (+), discharging (-) and idle (0) runs. Each run: (sign, start_s, end_s, mAh)."""
    out = []
    for (t0, i0, _), (t1, _, _) in zip(samples, samples[1:]):
        sign = 1 if i0 > threshold_ma else -1 if i0 < -threshold_ma else 0
        mah = i0 * (t1 - t0) / 3600
        if out and out[-1][0] == sign:
            s, a, _, q = out[-1]
            out[-1] = (s, a, t1, q + mah)
        else:
            out.append((sign, t0, t1, mah))
    return out


def report(path, threshold_ma=5.0):
    samples = load(path)
    if len(samples) < 2:
        raise SystemExit(f"{path}: need at least two readings")
    runs = periods(samples, threshold_ma)
    name = {1: "charging", -1: "discharging", 0: "idle"}
    print(f"{'period':<12} {'start (min)':>11} {'length (min)':>12} {'mean (mA)':>10} {'charge (mAh)':>12}")
    for s, a, b, q in runs:
        print(f"{name[s]:<12} {a / 60:>11.1f} {(b - a) / 60:>12.1f} {q / ((b - a) / 3600):>10.1f} {q:>12.1f}")
    lit = [(b - a, q) for s, a, b, q in runs if s == 1]
    dark = [(b - a, q) for s, a, b, q in runs if s == -1]
    net = sum(q for *_, q in runs)
    volts = [v for *_, v in samples]
    print(f"\nnet into the cell: {net:+.1f} mAh over {(samples[-1][0] - samples[0][0]) / 3600:.2f} h; "
          f"cell {min(volts):.2f}-{max(volts):.2f} V")
    if lit and dark:
        i_lit = sum(q for _, q in lit) / (sum(d for d, _ in lit) / 3600)
        i_dark = -sum(q for _, q in dark) / (sum(d for d, _ in dark) / 3600)
        print(f"breaks even when lit {100 * i_dark / (i_lit + i_dark):.0f}% of the time "
              f"(net {i_lit:.1f} mA while lit, {i_dark:.1f} mA drawn while dark)")
    return runs


def main():
    parser = argparse.ArgumentParser(description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter)
    commands = parser.add_subparsers(dest="cmd", required=True)
    sim = commands.add_parser("simulate", help="desk simulation of a light/dark run")
    sim.add_argument("directory", type=Path)
    sim.add_argument("--face", choices=sorted(FACES), default="si")
    sim.add_argument("--sun-w-m2", type=float, default=800.0)
    sim.add_argument("--use-ma", type=float, default=60.0, help="the node's average current (measure yours)")
    sim.add_argument("--cell-mah", type=float, default=1000.0)
    sim.add_argument("--lit-min", type=int, default=45)
    sim.add_argument("--dark-min", type=int, default=45)
    sim.add_argument("--cycles", type=int, default=4)
    rep = commands.add_parser("report", help="charge in and out of the cell, per period")
    rep.add_argument("power", type=Path)
    rep.add_argument("--threshold-ma", type=float, default=5.0)
    a = parser.parse_args()
    if a.cmd == "simulate":
        try:
            simulate(a.directory, a.face, a.sun_w_m2, a.use_ma, a.cell_mah, a.lit_min, a.dark_min, a.cycles)
        except FileExistsError:
            parser.error(f"{a.directory} already exists; choose a new directory to keep earlier evidence")
        print(f"wrote {a.directory / 'power.csv'} (desk simulation)")
    else:
        report(a.power, a.threshold_ma)


if __name__ == "__main__":
    main()
