#!/usr/bin/env python3
"""Spectrum sweeps with an RTL-SDR (Blog V4 or V3), for the CSC-R1 bench quests.

Standard library only. Recording needs `rtl_power` from the RTL-SDR Blog
drivers; every other command works on saved CSV files, so a crew can analyse
another crew's sweeps without a dongle.

    sweep.py record   --band 2m --label "load, gain 20" --out load.csv --dc-checked
    sweep.py summary  load.csv
    sweep.py compare  halow-off.csv halow-on.csv
    sweep.py response --ref through.csv --dut stub.csv --at 98e6 145.8e6
    sweep.py stub     --freq 98e6 --vf 0.66
    sweep.py stub     --freq 98e6 --length 0.51

All levels are relative dB from the dongle, not dBm. Differences between two
sweeps taken with the same dongle, gain and frequencies are meaningful; the
absolute numbers are not. `compare` and `response` refuse two sweeps whose
receiver, gain, bins or frequency range differ (from the .json labels that
`record` writes), unless you pass --force.
"""

import argparse
import csv
import datetime as dt
import json
import math
import os
import shutil
import statistics
import subprocess
import sys

C = 299_792_458.0

# Start, stop and bin width in Hz. Bins are fine enough to see a satellite
# channel but coarse enough to sweep in a few seconds.
BANDS = {
    "fm": (87.0e6, 109.0e6, 25e3),
    "2m": (143.5e6, 146.5e6, 5e3),
    "70cm": (434.5e6, 438.5e6, 5e3),
    "halow": (900.0e6, 930.0e6, 50e3),
    "filter2m": (80.0e6, 460.0e6, 250e3),
    "filter70cm": (100.0e6, 960.0e6, 500e3),
}


def load(path):
    """Read an rtl_power CSV. Returns sorted [(freq_hz, mean_db)].

    rtl_power writes one row per hop: date, time, f_low, f_high, step,
    samples, then one dB value per bin. Repeated sweeps of the same bin are
    averaged as power, not as dB.
    """
    acc = {}
    with open(path, newline="") as f:
        for row in csv.reader(f):
            if len(row) < 7 or row[0].startswith("#"):
                continue
            try:
                lo, step = float(row[2]), float(row[4])
                vals = [float(v) if v.strip() else math.nan for v in row[6:]]
            except ValueError:
                continue
            for i, db in enumerate(vals):
                if math.isnan(db) or db == math.inf:
                    continue
                fr = round(lo + (i + 0.5) * step)
                s, n = acc.get(fr, (0.0, 0))
                # rtl_power writes -inf for a bin with zero power: count it as 0 W.
                acc[fr] = (s + (0.0 if db == -math.inf else 10 ** (db / 10)), n + 1)
    if not acc:
        sys.exit(f"{path}: no rtl_power rows found")
    pts = [(fr, 10 * math.log10(s / n)) for fr, (s, n) in acc.items() if s > 0]
    dropped = len(acc) - len(pts)
    if dropped:
        print(f"{path}: skipped {dropped} bins with zero power in every sweep", file=sys.stderr)
    if not pts:
        sys.exit(f"{path}: every bin has zero power")
    return sorted(pts)


def meta(path):
    side = path + ".json"
    if os.path.exists(side):
        with open(side) as f:
            return json.load(f)
    return {}


def stats(pts):
    dbs = sorted(d for _, d in pts)
    peak = max(pts, key=lambda p: p[1])
    return {
        "bins": len(dbs),
        "floor_median_db": statistics.median(dbs),
        "floor_p10_db": dbs[len(dbs) // 10],
        "peak_db": peak[1],
        "peak_mhz": peak[0] / 1e6,
        "peak_above_floor_db": peak[1] - statistics.median(dbs),
    }


# Settings that must match before two sweeps can be subtracted.
MUST_MATCH = ("receiver", "gain_db", "bin_hz", "start_hz", "stop_hz", "ppm")


def check_compatible(a, b, force=False):
    """Stop if two sweeps were taken with different settings (from their .json labels)."""
    ma, mb = meta(a), meta(b)
    for path, label in ((a, ma), (b, mb)):
        valid = (
            isinstance(label, dict)
            and isinstance(label.get("receiver"), str)
            and bool(label["receiver"].strip())
            and all(type(label.get(k)) in (int, float) and math.isfinite(label[k])
                    for k in MUST_MATCH if k != "receiver")
            and label["bin_hz"] > 0
            and 0 <= label["start_hz"] < label["stop_hz"]
        )
        if not valid:
            msg = f"{path}.json is missing or has incomplete or invalid sweep settings"
            if not force:
                sys.exit(f"{msg}. Record with sweep.py record, or pass --force if you know they match.")
            print(f"warning: {msg}", file=sys.stderr)
            return
    diff = [k for k in MUST_MATCH if ma.get(k) != mb.get(k)]
    if diff:
        lines = "; ".join(f"{k}: {ma.get(k)} vs {mb.get(k)}" for k in diff)
        if not force:
            sys.exit(f"these sweeps can't be compared, settings differ ({lines}). "
                     "Re-record with the same settings, or pass --force to override.")
        print(f"warning: settings differ ({lines})", file=sys.stderr)


def nearest(pts, f):
    return min(pts, key=lambda p: abs(p[0] - f))


def pair(a, b):
    """Match only identical reported frequency centers."""
    bmap = dict(b)
    out = [(fr, db, bmap[fr]) for fr, db in a if fr in bmap]
    if not out:
        sys.exit("the sweeps have no shared finite-power bins")
    return out


def cmd_record(a):
    if a.band:
        lo, hi, step = BANDS[a.band]
    else:
        lo, hi, step = a.start, a.stop, a.step
    if not shutil.which("rtl_power"):
        sys.exit("rtl_power not found: install the RTL-SDR Blog drivers (see bench/README.md)")
    if not a.dc_checked:
        sys.exit("measure the DC on the dongle's SMA centre pin first (see bench/README.md, "
                 "\"Bias tee\"), then add --dc-checked. A dongle can keep its bias tee on even "
                 "when software turns it off, and DC can damage a load, filter or noise source.")
    if shutil.which("rtl_biast"):
        r = subprocess.run(["rtl_biast", "-b", "0"], capture_output=True, text=True)
        if r.returncode != 0:
            sys.exit("rtl_biast -b 0 failed, so the bias tee may still be on:\n"
                     + (r.stderr or r.stdout).strip())
    else:
        print("rtl_biast not found: relying on your DC measurement", file=sys.stderr)
    cmd = ["rtl_power", "-f", f"{int(lo)}:{int(hi)}:{int(step)}", "-g", str(a.gain),
           "-i", str(a.interval), "-e", f"{a.seconds}s", "-c", "0.2"]
    if a.ppm:
        cmd += ["-p", str(a.ppm)]
    cmd.append(a.out)
    print(" ".join(cmd), file=sys.stderr)
    subprocess.run(cmd, check=True)
    info = {
        "label": a.label,
        "evidence": a.evidence,
        "band": a.band or "custom",
        "start_hz": lo, "stop_hz": hi, "bin_hz": step,
        "gain_db": a.gain, "seconds": a.seconds, "ppm": a.ppm,
        "receiver": a.receiver,
        "dc_checked": True,
        "utc": dt.datetime.now(dt.timezone.utc).isoformat(timespec="seconds"),
        "levels": "relative dB from the dongle, not dBm",
    }
    with open(a.out + ".json", "w") as f:
        json.dump(info, f, indent=2)
    print(f"saved {a.out} and {a.out}.json")


def cmd_summary(a):
    for p in a.files:
        s = stats(load(p))
        m = meta(p)
        print(f"{p}  [{m.get('evidence', 'unlabelled')}] {m.get('label', '')}")
        print(f"  bins {s['bins']}, floor (median) {s['floor_median_db']:.1f} dB, "
              f"10th percentile {s['floor_p10_db']:.1f} dB")
        print(f"  strongest {s['peak_db']:.1f} dB at {s['peak_mhz']:.4f} MHz, "
              f"{s['peak_above_floor_db']:.1f} dB above the floor")


def cmd_compare(a):
    check_compatible(a.base, a.test, a.force)
    base, test = load(a.base), load(a.test)
    rows = pair(base, test)
    diffs = [t - b for _, b, t in rows]
    rise = statistics.median(diffs)
    worst = max(rows, key=lambda r: r[2] - r[1])
    print(f"base: {a.base}  {meta(a.base).get('label', '')}")
    print(f"test: {a.test}  {meta(a.test).get('label', '')}")
    print(f"  {len(rows)} shared bins")
    print(f"  noise floor change (median of all bins): {rise:+.1f} dB")
    print(f"  largest single-bin rise: {worst[2] - worst[1]:+.1f} dB at {worst[0] / 1e6:.4f} MHz")
    if abs(rise) < 0.5:
        print("  under 0.5 dB: within the run-to-run spread; repeat to be sure")


def cmd_response(a):
    """Insertion gain of a device: (noise through device) - (noise straight through)."""
    check_compatible(a.ref, a.dut, a.force)
    rows = pair(load(a.ref), load(a.dut))
    resp = [(fr, d - r) for fr, r, d in rows]
    if a.out:
        with open(a.out, "w", newline="") as f:
            w = csv.writer(f)
            w.writerow(["freq_mhz", "response_db"])
            for fr, g in resp:
                w.writerow([f"{fr / 1e6:.4f}", f"{g:.2f}"])
        print(f"saved {a.out}")
    best = max(resp, key=lambda p: p[1])
    low = min(resp, key=lambda p: p[1])
    print(f"  highest response {best[1]:+.1f} dB at {best[0] / 1e6:.3f} MHz")
    print(f"  deepest point    {low[1]:+.1f} dB at {low[0] / 1e6:.3f} MHz")
    for f in a.at:
        fr, g = nearest(resp, f)
        print(f"  at {f / 1e6:8.3f} MHz: {g:+.1f} dB")
    floor = statistics.median(sorted(g for _, g in resp)[: max(1, len(resp) // 20)])
    print(f"  deepest 5% of bins average {floor:+.1f} dB: readings near this are the "
          "limit of the method, not the device")


def cmd_stub(a):
    if a.length:
        vf = a.length * 4 * a.freq / C
        print(f"a {a.length:.3f} m open stub with its notch at {a.freq / 1e6:.2f} MHz "
              f"has velocity factor {vf:.3f}")
    else:
        L = C / (4 * a.freq) * a.vf
        print(f"quarter-wave open stub for {a.freq / 1e6:.2f} MHz with velocity factor "
              f"{a.vf}: {L:.3f} m ({L * 100:.1f} cm), measured from the tee")
        print(f"  it also notches odd multiples: {3 * a.freq / 1e6:.1f} and "
              f"{5 * a.freq / 1e6:.1f} MHz")


def main(argv=None):
    p = argparse.ArgumentParser(description=__doc__.split("\n")[0])
    sub = p.add_subparsers(dest="cmd", required=True)

    r = sub.add_parser("record", help="run rtl_power and save a labelled sweep")
    r.add_argument("--band", choices=sorted(BANDS))
    r.add_argument("--start", type=float)
    r.add_argument("--stop", type=float)
    r.add_argument("--step", type=float, default=10e3)
    r.add_argument("--gain", type=float, default=20.0, help="fixed tuner gain in dB; never auto")
    r.add_argument("--seconds", type=int, default=60)
    r.add_argument("--interval", type=int, default=2)
    r.add_argument("--ppm", type=int, default=0)
    r.add_argument("--label", required=True, help="what was connected, in a few words")
    r.add_argument("--evidence", default="hardware bench",
                   choices=["hardware bench", "received RF"])
    r.add_argument("--receiver", default="RTL-SDR Blog V4")
    r.add_argument("--out", required=True)
    r.add_argument("--dc-checked", action="store_true",
                   help="you measured under 0.5 V DC on the SMA centre pin with this dongle")

    s = sub.add_parser("summary", help="noise floor and strongest signal")
    s.add_argument("files", nargs="+")

    c = sub.add_parser("compare", help="noise-floor change from one sweep to another")
    c.add_argument("base")
    c.add_argument("test")
    c.add_argument("--force", action="store_true", help="compare even if settings differ")

    q = sub.add_parser("response", help="filter response from a noise source")
    q.add_argument("--ref", required=True, help="noise source straight into the dongle")
    q.add_argument("--dut", required=True, help="noise source through the device")
    q.add_argument("--at", type=float, nargs="*", default=[], help="frequencies to report, Hz")
    q.add_argument("--out")
    q.add_argument("--force", action="store_true", help="compare even if settings differ")

    t = sub.add_parser("stub", help="quarter-wave coax stub length or velocity factor")
    t.add_argument("--freq", type=float, required=True)
    t.add_argument("--vf", type=float, default=0.66)
    t.add_argument("--length", type=float, help="measured length in m: solve for velocity factor")

    a = p.parse_args(argv)
    if a.cmd == "record" and not a.band and (a.start is None or a.stop is None):
        p.error("record needs --band, or --start and --stop")
    {"record": cmd_record, "summary": cmd_summary, "compare": cmd_compare,
     "response": cmd_response, "stub": cmd_stub}[a.cmd](a)


if __name__ == "__main__":
    main()
