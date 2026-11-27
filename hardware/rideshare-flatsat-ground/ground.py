"""Ground tests for one rideshare FlatSat node on solar power. Uses only the Python standard library.

    python3 ground.py simulate run-1                 # desk simulation of a light/dark run
    python3 ground.py report run-1/power.csv --max-gap-s 10         # charge in and out, per light and dark period

power.csv is the INA226 logger's format (csc-power-log record):
t_ms,current_ma,bus_v,power_mw, with the INA226 in the cell's lead, wired so charging reads positive.
"""
import argparse
import csv
import json
import math
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
    if (not all(math.isfinite(v) for v in (sun_w_m2, use_ma, cell_mah, start_soc, step_s))
            or min(sun_w_m2, use_ma, lit_min, dark_min) < 0
            or min(cell_mah, cycles, step_s, lit_min + dark_min) <= 0
            or not 0 <= start_soc <= 1):
        raise ValueError("use finite, nonnegative inputs, a positive capacity, duration and cycle count, and a charge fraction from 0 to 1")
    directory.mkdir(parents=True, exist_ok=False)
    soc, t = start_soc, 0
    lit_s = lit_min * 60
    period = (lit_min + dark_min) * 60
    end = cycles * period
    source_ma = charge_ma(face, sun_w_m2)
    with (directory / "power.csv").open("w", newline="") as out:
        w = csv.writer(out)
        w.writerow(["t_ms", "current_ma", "bus_v", "power_mw"])
        while t < end:
            phase = t % period
            lit = phase < lit_s
            dt = min(step_s, (lit_s if lit else period) - phase, end - t)
            desired_ma = (source_ma if lit else 0.0) - use_ma
            next_soc = min(max(soc + desired_ma * dt / 3600 / cell_mah, 0.0), 1.0)
            i = (next_soc - soc) * cell_mah * 3600 / dt
            v = cell_v(soc)
            w.writerow([round(t * 1000), f"{i:.3f}", f"{v:.4f}", f"{i * v:.2f}"])
            soc, t = next_soc, t + dt
        w.writerow([round(end * 1000), "0.000", f"{cell_v(soc):.4f}", "0.00"])
    info = dict(evidence="desk simulation", face=face, sun_w_m2=sun_w_m2, use_ma=use_ma, cell_mah=cell_mah,
                lit_min=lit_min, dark_min=dark_min, cycles=cycles)
    (directory / "power.csv.json").write_text(json.dumps(info, indent=2) + "\n")


def load(path):
    samples = []
    columns = {"t_ms", "current_ma", "bus_v", "power_mw"}
    with path.open(newline="") as f:
        rows = csv.DictReader(f)
        if not columns.issubset(rows.fieldnames or ()):
            raise ValueError(f"{path}: expected the INA226 logger's columns t_ms,current_ma,bus_v,power_mw")
        for row in rows:
            try:
                t = int(row["t_ms"]) / 1000
                current, voltage = float(row["current_ma"]), float(row["bus_v"])
                if not math.isfinite(current) or not math.isfinite(voltage):
                    raise ValueError("nonfinite measurement")
            except (TypeError, ValueError, OverflowError) as exc:
                raise ValueError(f"{path}: line {rows.line_num}: invalid reading") from exc
            if t < 0 or samples and t <= samples[-1][0]:
                raise ValueError(f"{path}: line {rows.line_num}: timestamps must increase; save each logger restart to a separate CSV")
            samples.append((t, current, voltage))
    if len(samples) < 2:
        raise ValueError(f"{path}: need at least two readings")
    return samples


def periods(samples, threshold_ma=5.0, max_gap_s=1.0):
    """Split into charging (+), discharging (-) and idle (0) runs. Each run: (sign, start_s, end_s, mAh)."""
    out = []
    for (t0, i0, _), (t1, _, _) in zip(samples, samples[1:]):
        if t1 - t0 > max_gap_s:
            continue
        sign = 1 if i0 > threshold_ma else -1 if i0 < -threshold_ma else 0
        mah = i0 * (t1 - t0) / 3600
        if out and out[-1][0] == sign and out[-1][2] == t0:
            s, a, _, q = out[-1]
            out[-1] = (s, a, t1, q + mah)
        else:
            out.append((sign, t0, t1, mah))
    return out


def report(path, threshold_ma=5.0, max_gap_s=1.0):
    if not math.isfinite(threshold_ma) or threshold_ma < 0 or not math.isfinite(max_gap_s) or max_gap_s <= 0:
        raise ValueError("use a finite, nonnegative current threshold and a finite, positive maximum gap")
    samples = load(path)
    runs = periods(samples, threshold_ma, max_gap_s)
    if not runs:
        raise ValueError(f"{path}: no intervals within the {max_gap_s:g} s gap limit")
    name = {1: "charging", -1: "discharging", 0: "idle"}
    print(f"{'period':<12} {'start (min)':>11} {'length (min)':>12} {'mean (mA)':>10} {'charge (mAh)':>12}")
    for sign, start, end, charge in runs:
        print(f"{name[sign]:<12} {start / 60:>11.1f} {(end - start) / 60:>12.1f} "
              f"{charge / ((end - start) / 3600):>10.1f} {charge:>12.1f}")
    net = sum(charge for _, _, _, charge in runs)
    span = samples[-1][0] - samples[0][0]
    measured = sum(end - start for _, start, end, _ in runs)
    volts = [v for _, _, v in samples]
    print(f"\nnet into the cell: {net:+.1f} mAh over {measured / 3600:.2f} h of included intervals; "
          f"cell {min(volts):.2f}–{max(volts):.2f} V")
    if span - measured > 1e-6:
        print(f"Excluded {span - measured:.1f} s across gaps longer than {max_gap_s:g} s.")
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
    rep.add_argument("--max-gap-s", type=float, default=1.0,
                     help="skip intervals longer than this many seconds (default: 1 for the bench logger)")
    a = parser.parse_args()
    if a.cmd == "simulate":
        try:
            simulate(a.directory, a.face, a.sun_w_m2, a.use_ma, a.cell_mah, a.lit_min, a.dark_min, a.cycles)
        except ValueError as exc:
            parser.error(str(exc))
        except FileExistsError:
            parser.error(f"{a.directory} already exists; choose a new directory to keep earlier evidence")
        print(f"wrote {a.directory / 'power.csv'} (desk simulation)")
    else:
        try:
            report(a.power, a.threshold_ma, a.max_gap_s)
        except (OSError, ValueError) as exc:
            parser.error(str(exc))


if __name__ == "__main__":
    main()
