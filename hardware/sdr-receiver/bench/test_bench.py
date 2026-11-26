#!/usr/bin/env python3
"""Self-test for the bench scripts on synthetic files. No hardware needed.

    python3 hardware/sdr-receiver/bench/test_bench.py
"""

import contextlib
import csv
import io
import json
import math
import os
import random
import stat
import subprocess
import sys
import tempfile

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
import power_log  # noqa: E402
import sweep  # noqa: E402


def label(path, lo, hi, step, gain=20.0, receiver="RTL-SDR Blog V4", ppm=0):
    """Write the .json label that `sweep.py record` saves next to each sweep."""
    with open(path + ".json", "w") as f:
        json.dump({"receiver": receiver, "gain_db": gain, "bin_hz": step,
                   "start_hz": lo, "stop_hz": hi, "ppm": ppm}, f)


def refuses(fn, argv):
    """True if the command stops with an error (SystemExit with a message)."""
    try:
        with contextlib.redirect_stdout(io.StringIO()), contextlib.redirect_stderr(io.StringIO()):
            fn(argv)
    except SystemExit as e:
        return e.code not in (0, None)
    return False


def fake_sweep(path, lo, hi, step, level, notch=None, sweeps=3, gain=20.0):
    """Write rtl_power-style rows: 1 MHz hops, `level(f)` dB plus small noise, and a label."""
    label(path, lo, hi, step, gain)
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


def test_compare_refuses_mismatched_settings():
    # Review finding: 20 dB and 30 dB sweeps gave a "+10.0 dB" noise rise with no warning.
    d = tempfile.mkdtemp()
    a, b = os.path.join(d, "g20.csv"), os.path.join(d, "g30.csv")
    fake_sweep(a, 144e6, 146e6, 5e3, lambda f: -40.0, gain=20.0)
    fake_sweep(b, 144e6, 146e6, 5e3, lambda f: -30.0, gain=30.0)
    assert refuses(sweep.main, ["compare", a, b])
    assert refuses(sweep.main, ["response", "--ref", a, "--dut", b])
    out = run(sweep.main, ["compare", a, b, "--force"])  # an explicit override still works
    assert "+10.0 dB" in out, out
    os.remove(b + ".json")  # no label: can't check, so refuse unless forced
    assert refuses(sweep.main, ["compare", a, b])


def test_comparison_with_sparse_bins_and_incomplete_labels():
    with tempfile.TemporaryDirectory() as d:
        a, b = (os.path.join(d, name) for name in ("a.csv", "b.csv"))
        for path, levels in ((a, "-10,-inf,-inf,-10"), (b, "-inf,-50,-50,-inf")):
            with open(path, "w") as f:
                f.write(f"date,time,0,40,10,1,{levels}\n")
            with open(path + ".json", "w") as f:
                json.dump({"label": "incomplete"}, f)
        for cmd in (["compare", a, b], ["response", "--ref", a, "--dut", b]):
            with contextlib.redirect_stderr(io.StringIO()):
                try:
                    sweep.main(cmd)
                except SystemExit as e:
                    assert "incomplete or invalid" in str(e)
                else:
                    raise AssertionError("incomplete settings accepted")
        label(a, 0, 40, 10)
        label(b, 0, 40, 10)
        assert refuses(sweep.main, ["compare", a, b])


def test_zero_power_bins():
    # Review finding: rtl_power writes -inf for a zero-power bin, and summary crashed.
    d = tempfile.mkdtemp()
    p = os.path.join(d, "inf.csv")
    with open(p, "w", newline="") as f:
        w = csv.writer(f)
        w.writerow(["2026-11-26", "12:00:00", 144000000, 144020000, 5000, 100,
                    "-40.0", "-inf", "-41.0", "-39.0"])
        w.writerow(["2026-11-26", "12:00:02", 144000000, 144020000, 5000, 100,
                    "-40.0", "-inf", "-inf", "-39.0"])
    label(p, 144e6, 144.02e6, 5e3)
    pts = sweep.load(p)
    assert len(pts) == 3, pts  # the bin with zero power in every sweep is skipped
    assert abs(dict(pts)[144012500] - (-41.0 - 10 * math.log10(2))) < 0.01, pts
    out = run(sweep.main, ["summary", p])
    assert "floor (median)" in out, out


def test_record_needs_dc_check_and_working_biast():
    # Review finding: a failed bias-tee disable was ignored and the sweep went ahead.
    d = tempfile.mkdtemp()
    bindir = os.path.join(d, "bin")
    os.mkdir(bindir)

    def tool(name, body):
        path = os.path.join(bindir, name)
        with open(path, "w") as f:
            f.write("#!/bin/sh\n" + body + "\n")
        os.chmod(path, os.stat(path).st_mode | stat.S_IEXEC)

    tool("rtl_power", 'for a; do last="$a"; done; echo "2026-11-26, 12:00:00, 144000000, 144010000, 5000, 1, -40, -40" > "$last"')
    here = os.path.dirname(os.path.abspath(__file__))
    env = dict(os.environ, PATH=bindir + os.pathsep + os.environ["PATH"])

    def record(*extra):
        out = os.path.join(d, "s.csv")
        if os.path.exists(out):
            os.remove(out)
        r = subprocess.run([sys.executable, os.path.join(here, "sweep.py"), "record", "--band", "2m",
                            "--label", "load", "--out", out, *extra],
                           env=env, capture_output=True, text=True)
        return r.returncode, os.path.exists(out), r.stderr

    tool("rtl_biast", "exit 0")
    code, made, err = record()
    assert code != 0 and not made and "--dc-checked" in err, (code, made, err)
    code, made, err = record("--dc-checked")
    assert code == 0 and made, (code, made, err)
    tool("rtl_biast", "echo 'usb_claim_interface error' >&2; exit 1")
    code, made, err = record("--dc-checked")
    assert code != 0 and not made and "bias tee may still be on" in err, (code, made, err)


def test_power_megabytes_per_file():
    # Review finding: one --megabytes figure was applied to every file in *.csv.
    d = tempfile.mkdtemp()
    files = []
    for name, mw in (("a.csv", 500.0), ("b.csv", 1000.0)):
        p = os.path.join(d, name)
        with open(p, "w", newline="") as f:
            w = csv.writer(f)
            w.writerow(["t_ms", "current_ma", "bus_v", "power_mw"])
            for i in range(1201):  # 60 s at 50 ms
                w.writerow([i * 50, mw / 5, 5.0, mw])
        files.append(p)
    assert refuses(power_log.main, ["summary", *files, "--megabytes", "10"])
    out = run(power_log.main, ["summary", *files,
                               "--delivered", f"{files[0]}=10", "--delivered", f"{files[1]}=60"])
    rows = {line.split(" | ")[0]: line.split(" | ")[-1].strip()
            for line in out.splitlines() if " | " in line and not line.startswith("file")}
    assert rows[files[0]] == "3000" and rows[files[1]] == "1000", rows  # 30 J/10 MB, 60 J/60 MB
    assert refuses(power_log.main, ["summary", *files, "--delivered", "other.csv=5"])


if __name__ == "__main__":
    for t in (test_sweeps, test_response_stub, test_power, test_power_per_mb_covers_gaps,
              test_power_per_mb_needs_measured_time, test_compare_refuses_mismatched_settings,
              test_zero_power_bins, test_comparison_with_sparse_bins_and_incomplete_labels,
              test_record_needs_dc_check_and_working_biast,
              test_power_megabytes_per_file):
        t()
        print(f"ok  {t.__name__}")
