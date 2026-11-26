#!/usr/bin/env python3
"""Log and summarise radio power for the CSC-R1 bench quests.

    power_log.py record --port /dev/ttyACM0 --minutes 10 --label "C5, 5 GHz, streaming" --out c5-5g.csv
    power_log.py summary c5-5g.csv --megabytes 210
    power_log.py summary *.csv

`record` reads the power-logger sketch over USB serial (needs pyserial:
`pip install pyserial`). `summary` needs only the standard library.

It also reads a Nordic Power Profiler (PPK2) CSV export: a file whose header
has "Timestamp(ms)" and "Current(uA)" columns, with --volts for the supply.
"""

import argparse
import csv
import datetime as dt
import json
import os
import statistics
import sys
import time

SHUNT_LSB_V = 2.5e-6   # INA226 shunt-voltage register
BUS_LSB_V = 1.25e-3    # INA226 bus-voltage register


def record(a):
    try:
        import serial  # pyserial
    except ImportError:
        sys.exit("needs pyserial: pip install pyserial")
    end = time.time() + a.minutes * 60
    n = 0
    with serial.Serial(a.port, 115200, timeout=2) as s, open(a.out, "w", newline="") as f:
        w = csv.writer(f)
        w.writerow(["t_ms", "current_ma", "bus_v", "power_mw"])
        while time.time() < end:
            line = s.readline().decode(errors="replace").strip()
            if not line or line.startswith("#"):
                if line:
                    print(line, file=sys.stderr)
                continue
            try:
                t, sh, bus = (int(x) for x in line.split(","))
            except ValueError:
                continue
            i_ma = sh * SHUNT_LSB_V / a.shunt * 1e3
            v = bus * BUS_LSB_V
            w.writerow([t, f"{i_ma:.3f}", f"{v:.4f}", f"{i_ma * v:.2f}"])
            n += 1
            if n % 200 == 0:
                print(f"{n} readings, now {i_ma:.1f} mA at {v:.3f} V", file=sys.stderr)
    info = {
        "label": a.label, "evidence": "hardware bench", "meter": "INA226",
        "shunt_ohm": a.shunt, "minutes": a.minutes,
        "utc": dt.datetime.now(dt.timezone.utc).isoformat(timespec="seconds"),
    }
    with open(a.out + ".json", "w") as f:
        json.dump(info, f, indent=2)
    print(f"saved {n} readings to {a.out}")


def load(path, volts):
    """Return [(t_s, power_mw)] from our logger CSV or a PPK2 export."""
    with open(path, newline="") as f:
        rows = list(csv.DictReader(f))
    if not rows:
        sys.exit(f"{path}: empty")
    keys = rows[0].keys()
    out = []
    if "power_mw" in keys:
        for r in rows:
            out.append((int(r["t_ms"]) / 1e3, float(r["power_mw"])))
    elif "Timestamp(ms)" in keys and "Current(uA)" in keys:
        if not volts:
            sys.exit(f"{path}: a PPK2 export has no voltage; pass --volts")
        for r in rows:
            out.append((float(r["Timestamp(ms)"]) / 1e3, float(r["Current(uA)"]) / 1e3 * volts))
    else:
        sys.exit(f"{path}: unknown columns {list(keys)}")
    return out


def summary(a):
    print("file | label | minutes | average mW | energy J | mJ per MB")
    for p in a.files:
        pts = load(p, a.volts)
        if len(pts) < 2:
            print(f"{p}: too few readings")
            continue
        # Trapezoid integral; skips gaps longer than 1 s instead of bridging them.
        e_mj, dur = 0.0, 0.0
        for (t0, p0), (t1, p1) in zip(pts, pts[1:]):
            dtt = t1 - t0
            if 0 < dtt <= 1.0:
                e_mj += (p0 + p1) / 2 * dtt
                dur += dtt
        avg = e_mj / dur if dur else statistics.mean(p for _, p in pts)
        side = p + ".json"
        label = json.load(open(side)).get("label", "") if os.path.exists(side) else ""
        per_mb = f"{e_mj / a.megabytes:.0f}" if a.megabytes else "-"
        print(f"{p} | {label} | {dur / 60:.1f} | {avg:.0f} | {e_mj / 1e3:.1f} | {per_mb}")
    if not a.megabytes:
        print("add --megabytes (delivered at the receiving end) for energy per megabyte")


def main(argv=None):
    p = argparse.ArgumentParser(description=__doc__.split("\n")[0])
    sub = p.add_subparsers(dest="cmd", required=True)
    r = sub.add_parser("record")
    r.add_argument("--port", required=True)
    r.add_argument("--minutes", type=float, default=10)
    r.add_argument("--shunt", type=float, default=0.1, help="shunt resistor in ohms")
    r.add_argument("--label", required=True)
    r.add_argument("--out", required=True)
    s = sub.add_parser("summary")
    s.add_argument("files", nargs="+")
    s.add_argument("--megabytes", type=float, help="data delivered during the run")
    s.add_argument("--volts", type=float, help="supply voltage, for PPK2 exports")
    a = p.parse_args(argv)
    (record if a.cmd == "record" else summary)(a)


if __name__ == "__main__":
    main()
