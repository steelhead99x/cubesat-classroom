#!/usr/bin/env python3
"""Self-test for the bench scripts on synthetic files. No hardware needed.

    python3 hardware/sdr-receiver/bench/test_bench.py
"""

import contextlib
import csv
import io
import math
import os
import random
import sys
import tempfile

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
import power_log  # noqa: E402
import sweep  # noqa: E402


def fake_sweep(path, lo, hi, step, level, notch=None, sweeps=3):
    """Write rtl_power-style rows: 1 MHz hops, `level(f)` dB plus small noise."""
    rnd = random.Random(1)
    with open(path, "w", newline="") as f:
        w = csv.writer(f)
        for _ in range(sweeps):
            hop = lo
            while hop < hi:
                n = int(min(1e6, hi - hop) / step)
                vals = []
                for i in range(n):
                    fr = hop + (i + 0.5) * step
                    db = level(fr) + rnd.gauss(0, 0.3)
                    if notch:
                        db += notch(fr)
                    vals.append(f"{db:.2f}")
                w.writerow(["2026-11-26", "12:00:00", int(hop), int(hop + n * step), step, 100] + vals)
                hop += n * step


def run(fn, argv):
    buf = io.StringIO()
    with contextlib.redirect_stdout(buf):
        fn(argv)
    return buf.getvalue()


def test_power_per_mb_covers_gaps():
    # Review finding: a 10 s gap was left out of the energy but the megabytes were from the
    # whole run, so energy per MB came out low. 1 W steady for 30 s with a 10 s hole in the
    # readings: the run used about 30 J, so 30 MB is about 1000 mJ per MB, not 667.
    with tempfile.TemporaryDirectory() as d:
        p = os.path.join(d, "gap.csv")
        with open(p, "w", newline="") as f:
            w = csv.writer(f)
            w.writerow(["t_ms", "current_ma", "bus_v", "power_mw"])
            for t in [*range(0, 10_001, 100), *range(20_000, 30_001, 100)]:
                w.writerow([t, "200.000", "5.0000", "1000.00"])
        out = run(power_log.main, ["summary", p, "--megabytes", "30"])
        row = next(line for line in out.splitlines() if line.startswith(p + " |"))
        assert row.split(" | ")[-1].strip() == "1000", row
        assert "10.0 s of the 30.0 s run had no readings" in out, out


def test_power_per_mb_needs_measured_time():
    # Review finding: three readings 10 s apart gave 0.0 J next to 1000 mJ per MB, though
    # no interval was integrated. With nothing measured, the efficiency is left out.
    with tempfile.TemporaryDirectory() as d:
        p = os.path.join(d, "sparse.csv")
        with open(p, "w", newline="") as f:
            w = csv.writer(f)
            w.writerow(["t_ms", "current_ma", "bus_v", "power_mw"])
            for t in (0, 10_000, 20_000):
                w.writerow([t, "200.000", "5.0000", "1000.00"])
        out = run(power_log.main, ["summary", p, "--megabytes", "20"])
        row = next(line for line in out.splitlines() if line.startswith(p + " |"))
        assert row.split(" | ")[-1].strip() == "-", row
        assert "no energy was measured" in out, out


def test_sweeps():
    d = tempfile.mkdtemp()
    base, test = os.path.join(d, "off.csv"), os.path.join(d, "on.csv")
    fake_sweep(base, 144e6, 146e6, 5e3, lambda f: -40.0)
    fake_sweep(test, 144e6, 146e6, 5e3, lambda f: -37.0 + (20 if abs(f - 145.8e6) < 6e3 else 0))
    pts = sweep.load(base)
    assert len(pts) == 400, len(pts)
    assert abs(sweep.stats(pts)["floor_median_db"] + 40) < 0.3
    out = run(sweep.main, ["compare", base, test])
    assert "+3.0 dB" in out, out
    assert "145.7975" in out or "145.8025" in out, out
    out = run(sweep.main, ["summary", test])
    assert "145.7975" in out or "145.8025" in out, out


def test_response_stub():
    d = tempfile.mkdtemp()
    ref, dut = os.path.join(d, "ref.csv"), os.path.join(d, "dut.csv")
    flat = lambda f: -30.0 - 3 * (f - 80e6) / 300e6  # dongle's own slope cancels
    fake_sweep(ref, 80e6, 120e6, 250e3, flat)
    # A notch at 98 MHz, 25 dB deep.
    fake_sweep(dut, 80e6, 120e6, 250e3, flat,
               notch=lambda f: -25 * math.exp(-((f - 98e6) / 1.5e6) ** 2))
    out = run(sweep.main, ["response", "--ref", ref, "--dut", dut, "--at", "90e6", "98e6"])
    deepest = [l for l in out.splitlines() if "deepest point" in l][0]
    assert abs(float(deepest.split(" at ")[1].split()[0]) - 98.0) < 0.5, deepest
    line98 = [l for l in out.splitlines() if "at   98.000" in l][0]
    assert -26.5 < float(line98.split(":")[1].split()[0]) < -23.5, line98
    out = run(sweep.main, ["stub", "--freq", "98e6", "--vf", "0.66"])
    assert "0.505 m" in out, out
    out = run(sweep.main, ["stub", "--freq", "98e6", "--length", "0.505"])
    assert "0.66" in out, out


def test_power():
    d = tempfile.mkdtemp()
    p = os.path.join(d, "p.csv")
    with open(p, "w", newline="") as f:
        w = csv.writer(f)
        w.writerow(["t_ms", "current_ma", "bus_v", "power_mw"])
        for i in range(1201):  # 60 s at 50 ms
            w.writerow([i * 50, 100, 5.0, 500.0])
    out = run(power_log.main, ["summary", p, "--megabytes", "10"])
    assert "| 1.0 | 500 | 30.0 | 3000" in out, out
    ppk = os.path.join(d, "ppk.csv")
    with open(ppk, "w", newline="") as f:
        w = csv.writer(f)
        w.writerow(["Timestamp(ms)", "Current(uA)"])
        for i in range(10001):
            w.writerow([i * 0.01 * 100, 20000])  # 1 ms steps, 20 mA
    out = run(power_log.main, ["summary", ppk, "--volts", "3.3"])
    assert "| 66 |" in out, out


if __name__ == "__main__":
    for t in (test_sweeps, test_response_stub, test_power, test_power_per_mb_covers_gaps,
              test_power_per_mb_needs_measured_time):
        t()
        print(f"ok  {t.__name__}")
